use psm_core::{
    Error,
    domain::{
        attribute::{AttributeGroup, CoreType, Definition, Provenance},
        photo::{Classification, NewPhoto, PhotoSource},
        publish::DefinitionLink,
        session::PublishStep,
    },
    mechanisms::hash::sha256_file,
    ports::clock::Clock,
    services::{attributes::AttributesService, items::ItemsService},
};
use psm_infra::jobs::JobRunner;
use psm_testkit::TestApp;
use serde_json::{Value, json};
use std::fs;

fn stored(a: &TestApp) -> Value {
    serde_json::from_slice(&fs::read(a.app.paths.settings()).unwrap()).unwrap()
}
fn shopify(secret: &str) -> Value {
    json!({"kind":"shopify","shop_domain":"x.myshopify.com","client_id":"ID",
        "client_secret":secret,"api_version":"2026-10",
        "condition_photos_key":"custom.damage_report"})
}
fn health<'a>(
    report: &'a psm_core::domain::HealthReport,
    code: &str,
) -> &'a psm_core::domain::HealthItem {
    report.items.iter().find(|item| item.code == code).unwrap()
}
fn main_photo(a: &TestApp, session: i64) {
    let raw = a.dir.path().join("a.ARW");
    fs::write(&raw, b"raw").unwrap();
    let photo = a
        .store()
        .photo_insert_or_get(NewPhoto {
            session_id: session,
            raw_path: raw.to_string_lossy().into_owned(),
            raw_hash: Some(sha256_file(&raw).unwrap()),
            captured_at: Some(a.clock.now_ms()),
            source: PhotoSource::Capture,
            created_at: a.clock.now_ms(),
        })
        .unwrap();
    a.fake.add_image(&photo.raw_path, &["v1"], false);
    a.review().classify(photo.id, Classification::Main).unwrap();
    a.store()
        .set_missing_preview_paths(&photo.raw_path, "preview.jpg")
        .unwrap();
}
fn publish(a: &TestApp, session: i64) {
    a.app
        .publish
        .as_ref()
        .unwrap()
        .confirm(&[session], "Publish chair")
        .unwrap();
    for _ in 0..10 {
        a.tick(1100).unwrap();
        a.complete_all_renders().unwrap();
    }
    assert_eq!(a.store().publish_item(1).unwrap().step, PublishStep::Done);
}
fn session(a: &TestApp) -> (i64, i64) {
    let created = a
        .items()
        .create_local("P21-1", "Oak", "Chair", None)
        .unwrap();
    let id = created.1.session.id;
    (id, a.store().session(id).unwrap().session.item_id.unwrap())
}

#[test]
fn c1a_damaged_settings_start_and_save_repairs_file() {
    let mut a = TestApp::new().unwrap();
    let broken = b"{ \"preview_delay_s\": ";
    fs::write(a.app.paths.settings(), broken).unwrap();
    a.kill_and_restart().unwrap();
    let shown = a.app.settings_get().unwrap();
    assert_eq!(shown["preview_delay_s"], 15);
    assert!(shown["document_id"].is_null());
    let report = a.app.health_check().unwrap();
    assert!(!health(&report, "settings_invalid").ok);
    assert!(
        health(&report, "settings_invalid")
            .detail
            .starts_with("settings.json: ")
    );
    assert_eq!(fs::read(a.app.paths.settings()).unwrap(), broken);
    a.settings_set(&json!({"preview_delay_s":20})).unwrap();
    assert_eq!(stored(&a)["preview_delay_s"], 20);
    assert!(
        !a.app
            .health_check()
            .unwrap()
            .items
            .iter()
            .any(|i| i.code == "settings_invalid")
    );
}

#[test]
fn c1b_incomplete_integration_starts_in_export_mode() {
    let mut a = TestApp::new().unwrap();
    let document = a.fake.state.lock().unwrap().document_id.clone();
    let mut data = stored(&a);
    data["document_id"] = json!(document);
    data["integration"] = json!({"kind":"shopify","shop_domain":"x.myshopify.com"});
    fs::write(a.app.paths.settings(), serde_json::to_vec(&data).unwrap()).unwrap();
    a.kill_and_restart().unwrap();
    assert!(a.app.commerce.is_none());
    assert!(a.app.publish.is_none());
    let shown = a.app.settings_get().unwrap();
    assert_eq!(shown["document_id"], data["document_id"]);
    assert!(shown["integration"].is_null());
    let item = a.app.health_check().unwrap();
    assert!(!health(&item, "settings_invalid").ok);
    assert!(
        health(&item, "settings_invalid")
            .detail
            .starts_with("integration: ")
    );
    assert!(
        health(&item, "settings_invalid")
            .detail
            .contains("client_id")
    );
}

#[test]
fn c1c_failed_backups_do_not_stop_start_or_publish() {
    let mut a = TestApp::with_fake_shop().unwrap();
    let backup = a.dir.path().join("backup");
    fs::remove_dir_all(&backup).unwrap();
    fs::write(&backup, b"not a directory").unwrap();
    a.kill_and_restart().unwrap();
    let report = a.app.health_check().unwrap();
    assert!(!health(&report, "backup_failed").ok);
    assert!(!health(&report, "backup_failed").detail.is_empty());
    a.clock.advance(1_761_350_400_000);
    let (id, _) = session(&a);
    main_photo(&a, id);
    publish(&a, id);
    let conn = rusqlite::Connection::open(a.app.paths.db()).unwrap();
    let state: String = conn
        .query_row(
            "SELECT status FROM jobs WHERE kind='publish_step' ORDER BY id DESC LIMIT 1",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(state, "done");
    assert!(!health(&a.app.health_check().unwrap(), "backup_failed").ok);
}

#[test]
fn c2b_exact_secret_mask_only() {
    let mut a = TestApp::new().unwrap();
    a.settings_set(&json!({"integration":shopify("S-1234")}))
        .unwrap();
    let mut block = a.app.settings_get().unwrap()["integration"].clone();
    assert_eq!(block["client_secret"], "••••1234");
    a.settings_set(&json!({"integration":block})).unwrap();
    assert_eq!(stored(&a)["integration"]["client_secret"], "S-1234");
    block["client_secret"] = json!("••••1234-typed");
    a.settings_set(&json!({"integration":block})).unwrap();
    assert_eq!(stored(&a)["integration"]["client_secret"], "••••1234-typed");
    block["client_secret"] = json!("••••9999");
    a.settings_set(&json!({"integration":block})).unwrap();
    assert_eq!(stored(&a)["integration"]["client_secret"], "••••9999");
}

#[test]
fn c2c_integration_defaults_appear_on_read() {
    let mut a = TestApp::new().unwrap();
    let mut data = stored(&a);
    data["integration"] = json!({"kind":"shopify","shop_domain":"x.myshopify.com",
        "client_id":"ID","client_secret":"S-1234"});
    fs::write(a.app.paths.settings(), serde_json::to_vec(&data).unwrap()).unwrap();
    a.kill_and_restart().unwrap();
    let shown = a.app.settings_get().unwrap();
    assert_eq!(shown["integration"]["api_version"], "2026-10");
    assert_eq!(
        shown["integration"]["condition_photos_key"],
        "custom.damage_report"
    );
    assert_eq!(shown["integration"]["client_secret"], "••••1234");
    assert_eq!(shown["integration"]["shop_domain"], "x.myshopify.com");
    assert!(a.app.commerce.is_some());
}

#[test]
fn c3a_handlerless_runner_claims_nothing() {
    let a = TestApp::with_fake_shop().unwrap();
    let (session, _) = session(&a);
    a.insert_publish_item(session, "upload", 0, None).unwrap();
    a.store().enqueue_publish_item(1, 0).unwrap();
    let runner = JobRunner::new(a.app.store.clone()).only_kind("publish_step");
    assert_eq!(runner.tick(0).unwrap(), 0);
    assert_eq!(runner.tick(0).unwrap(), 0);
    let conn = rusqlite::Connection::open(a.app.paths.db()).unwrap();
    let row: (String, i64) = conn
        .query_row(
            "SELECT status,attempts FROM jobs WHERE kind='publish_step'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(row, ("queued".into(), 0));
}

#[test]
fn c3b_unknown_kind_retries_and_known_job_completes() {
    let a = TestApp::new().unwrap();
    let (session, _) = session(&a);
    let photo = a.add_photo(session, "hash.ARW").unwrap();
    let conn = rusqlite::Connection::open(a.app.paths.db()).unwrap();
    for (kind, target) in [("nope", "unknown"), ("hash", photo.raw_path.as_str())] {
        conn.execute("INSERT INTO jobs(kind,target,payload,status,attempts,next_attempt_at,created_at,updated_at) VALUES (?1,?2,'{}','queued',0,0,0,0)", [kind,target]).unwrap();
    }
    let now = 123_000;
    assert_eq!(a.app.jobs.tick(now).unwrap(), 2);
    let unknown: (String, String, i64) = conn
        .query_row(
            "SELECT status,error,next_attempt_at FROM jobs WHERE kind='nope'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap();
    assert_eq!(
        unknown,
        ("queued".into(), "unknown_job_kind".into(), now + 60_000)
    );
    let known: String = conn
        .query_row("SELECT status FROM jobs WHERE kind='hash'", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(known, "done");
}

#[test]
fn c4a_shop_search_surfaces_provider_error_local_search_stays_local() {
    let a = TestApp::with_fake_shop().unwrap();
    a.fake_shop().state.lock().unwrap().unreachable = true;
    let items = ItemsService::new(a.store(), a.clock.as_ref()).with_commerce(Some(a.fake_shop()));
    assert!(matches!(items.search_shop("oak"), Err(Error::Provider(_))));
    assert!(items.search_local("oak").unwrap().is_empty());
    assert_eq!(a.fake_shop().state.lock().unwrap().reads, ["item_find"]);
}

#[test]
fn c5a_publish_notification_is_plain_text_with_stored_time() {
    let a = TestApp::with_fake_shop().unwrap();
    a.clock.advance(1_761_350_400_000);
    let (id, _) = session(&a);
    main_photo(&a, id);
    let confirmed = a.clock.now_ms();
    publish(&a, id);
    let notifications = a.store().notifications(Some(id)).unwrap();
    assert_eq!(notifications.len(), 1);
    assert_eq!(notifications[0].kind, "published");
    assert_eq!(notifications[0].text, "Published");
    assert!(notifications[0].created_at >= confirmed);
    assert!(notifications[0].created_at <= a.clock.now_ms());
}

#[test]
fn c5b_unchanged_description_is_not_sent_twice() {
    let a = TestApp::with_fake_shop().unwrap();
    a.clock.advance(1_761_350_400_000);
    let (id, item) = session(&a);
    main_photo(&a, id);
    let external = a.fake_shop().add_product("P21-1", "Oak", None);
    a.items().link(item, "shopify", &external, None).unwrap();
    AttributesService::new(a.store(), a.clock.as_ref(), Some("shopify"))
        .set_value(
            item,
            "product.description",
            json!("a\n\nb"),
            Provenance::Human,
        )
        .unwrap();
    publish(&a, id);
    AttributesService::new(a.store(), a.clock.as_ref(), Some("shopify"))
        .set_value(item, "product.tags", json!(["x"]), Provenance::Human)
        .unwrap();
    a.app
        .publish
        .as_ref()
        .unwrap()
        .confirm(&[id], "Publish again")
        .unwrap();
    for _ in 0..10 {
        a.tick(1100).unwrap();
        a.complete_all_renders().unwrap();
    }
    let shop = a.fake_shop().state.lock().unwrap();
    assert_eq!(
        shop.mutations
            .iter()
            .filter(|m| m.op == "update_description")
            .count(),
        1
    );
    assert_eq!(shop.descriptions[&external], "<p>a</p><p>b</p>");
    drop(shop);
    assert_eq!(a.store().publish_item(2).unwrap().step, PublishStep::Done);
}

#[test]
fn c6a_empty_barcode_clears_and_none_keeps() {
    let a = TestApp::new().unwrap();
    let (item, _) = a
        .items()
        .create_local("B-1", "Oak", "Chair", Some("4006381333931"))
        .unwrap();
    assert_eq!(
        a.items()
            .update_identity(item.id, None, Some(""))
            .unwrap()
            .barcode,
        None
    );
    assert_eq!(
        a.items()
            .update_identity(item.id, None, Some("123"))
            .unwrap()
            .barcode
            .as_deref(),
        Some("123")
    );
    let final_item = a.items().update_identity(item.id, None, None).unwrap();
    assert_eq!(final_item.barcode.as_deref(), Some("123"));
    assert_eq!(final_item.sku, "B-1");
}

fn field() -> (Definition, DefinitionLink) {
    (
        Definition {
            integration: "shopify".into(),
            key: "custom.damage_report".into(),
            label: "Damage report".into(),
            core_type: CoreType::MediaList,
            choices: vec![],
            validations: json!({}),
            group: AttributeGroup::Condition,
            description: None,
            writable: true,
        },
        DefinitionLink {
            platform_type: "list.file_reference".into(),
            platform_meta: json!({}),
        },
    )
}

#[test]
fn c7a_health_reads_field_before_judging_it() {
    let a = TestApp::with_fake_shop().unwrap();
    a.fake_shop().state.lock().unwrap().definitions = vec![field()];
    let report = a.app.health_check().unwrap();
    assert!(
        a.fake_shop()
            .state
            .lock()
            .unwrap()
            .reads
            .contains(&"attributes_schema".into())
    );
    assert!(health(&report, "condition_field_missing").ok);
    assert_eq!(
        health(&report, "condition_field_missing").detail,
        "Condition photos field custom.damage_report found"
    );
    let mut definitions = Vec::new();
    a.store()
        .transaction(&mut |tx| {
            definitions = tx.attribute_definitions("shopify")?;
            Ok(())
        })
        .unwrap();
    assert!(definitions.iter().any(|d| d.key == "custom.damage_report"));
}

#[test]
fn c7b_health_distinguishes_unread_from_stored_fields() {
    let a = TestApp::with_fake_shop().unwrap();
    a.fake_shop().state.lock().unwrap().unreachable = true;
    let report = a.app.health_check().unwrap();
    assert!(!health(&report, "condition_field_missing").ok);
    assert_eq!(
        health(&report, "condition_field_missing").detail,
        "Condition photos field custom.damage_report unknown: shop fields not read yet"
    );
    let b = TestApp::with_fake_shop().unwrap();
    b.fake_shop().state.lock().unwrap().definitions = vec![field()];
    b.app.health_check().unwrap();
    b.fake_shop().state.lock().unwrap().unreachable = true;
    let report = b.app.health_check().unwrap();
    assert!(health(&report, "condition_field_missing").ok);
    assert_eq!(
        health(&report, "condition_field_missing").detail,
        "Condition photos field custom.damage_report found"
    );
}
