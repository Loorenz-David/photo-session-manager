use psm_core::{
    domain::{
        attribute::{AttributeGroup, CoreType, Definition, Provenance, Value as AttributeValue},
        photo::{Classification as C, NewPhoto, PhotoSource},
        publish::{DefinitionLink, RemoteIdentity},
        session::PublishStep,
    },
    mechanisms::{attribute_write::merge_tags, hash::sha256_file},
    ports::{clock::Clock, commerce::CommerceAdapter},
    services::attributes::AttributesService,
};
use psm_testkit::TestApp;
use serde_json::{Value as Json, json};
use std::fs;

fn app() -> TestApp {
    let a = TestApp::with_fake_shop().unwrap();
    a.clock.advance(1_761_350_400_000);
    a
}
fn session(a: &TestApp, sku: &str) -> (i64, i64) {
    let s = a
        .items()
        .create_local(sku, "Oak", "Chair", None)
        .unwrap()
        .1
        .session
        .id;
    (s, a.store().session(s).unwrap().session.item_id.unwrap())
}
fn photo(a: &TestApp, s: i64, name: &str, class: C) -> i64 {
    let folder = a.dir.path().join("Capture/raws");
    fs::create_dir_all(&folder).unwrap();
    let path = folder.join(format!("{name}.ARW"));
    fs::write(&path, name.as_bytes()).unwrap();
    let p = a
        .store()
        .photo_insert_or_get(NewPhoto {
            session_id: s,
            raw_path: path.to_string_lossy().into_owned(),
            raw_hash: Some(sha256_file(&path).unwrap()),
            captured_at: Some(a.clock.now_ms()),
            source: PhotoSource::Capture,
            created_at: a.clock.now_ms(),
        })
        .unwrap();
    a.fake
        .add_image(&p.raw_path, &[&format!("v{}", p.id)], false);
    if class != C::Unreviewed {
        a.review().classify(p.id, class).unwrap();
    }
    a.store()
        .set_missing_preview_paths(&p.raw_path, "preview.jpg")
        .unwrap();
    p.id
}
fn def(key: &str, kind: CoreType, writable: bool) -> (Definition, DefinitionLink) {
    (
        Definition {
            integration: "shopify".into(),
            key: key.into(),
            label: key.into(),
            core_type: kind,
            choices: vec![],
            validations: json!({}),
            group: AttributeGroup::Details,
            description: None,
            writable,
        },
        DefinitionLink {
            platform_type: if kind == CoreType::MediaList {
                "list.file_reference".into()
            } else {
                "single_line_text_field".into()
            },
            platform_meta: json!({}),
        },
    )
}
fn schema(a: &TestApp, defs: Vec<(Definition, DefinitionLink)>) {
    a.fake_shop().state.lock().unwrap().definitions = defs;
    AttributesService::new(a.store(), a.clock.as_ref(), Some("shopify"))
        .refresh_schema(a.fake_shop())
        .unwrap();
}
fn value(a: &TestApp, item: i64, key: &str, value: Json) {
    AttributesService::new(a.store(), a.clock.as_ref(), Some("shopify"))
        .set_value(item, key, value, Provenance::Human)
        .unwrap();
}
fn drain(a: &TestApp) {
    for _ in 0..10 {
        a.tick(1100).unwrap();
        a.complete_all_renders().unwrap();
    }
}
fn run(a: &TestApp, s: i64) -> i64 {
    let service = a.app.publish.as_ref().unwrap();
    let run = service.confirm(&[s], "Publish images").unwrap();
    drain(a);
    run
}
fn linked(a: &TestApp, item: i64, sku: &str, title: &str, barcode: Option<&str>) -> String {
    let id = a.fake_shop().add_product(sku, title, barcode);
    a.items().link(item, "shopify", &id, None).unwrap();
    id
}
fn latest(a: &TestApp, s: i64) -> psm_core::domain::publish::PublishItem {
    let conn = rusqlite::Connection::open(a.app.paths.db()).unwrap();
    let id: i64 = conn
        .query_row(
            "SELECT id FROM publish_items WHERE session_id=?1 ORDER BY id DESC LIMIT 1",
            [s],
            |r| r.get(0),
        )
        .unwrap();
    a.store().publish_item(id).unwrap()
}
fn remote_value(a: &TestApp, id: &str, key: &str) -> Option<Json> {
    a.fake_shop()
        .state
        .lock()
        .unwrap()
        .values
        .get(id)
        .and_then(|rows| rows.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone()))
}
#[test]
fn c1a_changed_confirmed_attribute_only() {
    let a = app();
    let (s, item) = session(&a, "P14-A");
    photo(&a, s, "p", C::Main);
    schema(
        &a,
        vec![
            def("custom.years", CoreType::Text, true),
            def("custom.country", CoreType::Text, true),
            def("custom.designer", CoreType::Text, true),
        ],
    );
    value(&a, item, "custom.years", json!("1950-1960s"));
    value(&a, item, "custom.country", json!("Sweden"));
    let id = linked(&a, item, "P14-A", "Oak", None);
    a.fake_shop().state.lock().unwrap().values.insert(
        id.clone(),
        vec![
            ("custom.years".into(), json!("1950-1960s")),
            ("custom.country".into(), json!("Denmark")),
            ("custom.designer".into(), json!("X")),
        ],
    );
    run(&a, s);
    assert_eq!(
        remote_value(&a, &id, "custom.country"),
        Some(json!("Sweden"))
    );
    assert_eq!(remote_value(&a, &id, "custom.designer"), Some(json!("X")));
    assert_eq!(
        a.fake_shop()
            .state
            .lock()
            .unwrap()
            .attribute_writes
            .iter()
            .flat_map(|batch| batch.iter().map(|v| v.key.as_str()))
            .collect::<Vec<_>>(),
        vec!["custom.country"]
    );
    assert_eq!(
        a.fake_shop()
            .state
            .lock()
            .unwrap()
            .mutations
            .iter()
            .filter(|m| m.op == "attributes_write")
            .count(),
        1
    );
}
#[test]
fn c1b_absent_local_never_clears_remote() {
    let a = app();
    let (s, item) = session(&a, "P14-B");
    photo(&a, s, "p", C::Main);
    schema(&a, vec![def("custom.country", CoreType::Text, true)]);
    let id = linked(&a, item, "P14-B", "Oak", None);
    a.fake_shop().state.lock().unwrap().values.insert(
        id.clone(),
        vec![("custom.country".into(), json!("Denmark"))],
    );
    run(&a, s);
    assert_eq!(
        remote_value(&a, &id, "custom.country"),
        Some(json!("Denmark"))
    );
    assert!(
        !a.fake_shop()
            .state
            .lock()
            .unwrap()
            .mutations
            .iter()
            .any(|m| m.op == "attributes_write")
    );
}
#[test]
fn c1c_invalid_local_is_not_written() {
    let a = app();
    let (s, item) = session(&a, "P14-C");
    photo(&a, s, "p", C::Main);
    schema(&a, vec![def("custom.country", CoreType::Text, true)]);
    a.store()
        .transaction(&mut |tx| {
            tx.put_attribute_value(&AttributeValue {
                item_id: item,
                key: "custom.country".into(),
                value: json!("Sweden").to_string(),
                valid: false,
                provenance: Provenance::Human,
                ai_confidence: None,
                ai_status: None,
                ai_reason: None,
                ai_sources: None,
                updated_at: 0,
            })
        })
        .unwrap();
    let id = linked(&a, item, "P14-C", "Oak", None);
    run(&a, s);
    assert_eq!(remote_value(&a, &id, "custom.country"), None);
}
#[test]
fn c1d_read_only_reference_is_not_written() {
    let a = app();
    let (s, item) = session(&a, "P14-D");
    photo(&a, s, "p", C::Main);
    schema(
        &a,
        vec![
            def("custom.ref", CoreType::Reference, false),
            def("custom.date_time", CoreType::Reference, false),
        ],
    );
    a.store()
        .transaction(&mut |tx| {
            tx.put_attribute_value(&AttributeValue {
                item_id: item,
                key: "custom.ref".into(),
                value: json!("gid://x").to_string(),
                valid: true,
                provenance: Provenance::Human,
                ai_confidence: None,
                ai_status: None,
                ai_reason: None,
                ai_sources: None,
                updated_at: 0,
            })
        })
        .unwrap();
    a.store()
        .transaction(&mut |tx| {
            tx.put_attribute_value(&AttributeValue {
                item_id: item,
                key: "custom.date_time".into(),
                value: json!("2026-10-26T10:00:00Z").to_string(),
                valid: true,
                provenance: Provenance::Human,
                ai_confidence: None,
                ai_status: None,
                ai_reason: None,
                ai_sources: None,
                updated_at: 0,
            })
        })
        .unwrap();
    let id = linked(&a, item, "P14-D", "Oak", None);
    run(&a, s);
    assert_eq!(remote_value(&a, &id, "custom.ref"), None);
    assert_eq!(remote_value(&a, &id, "custom.date_time"), None);
}
#[test]
fn c1e_readback_mismatch_fails_update_with_key() {
    let a = app();
    let (s, item) = session(&a, "P14-E");
    photo(&a, s, "p", C::Main);
    schema(&a, vec![def("custom.country", CoreType::Text, true)]);
    value(&a, item, "custom.country", json!("Sweden"));
    linked(&a, item, "P14-E", "Oak", None);
    a.fake_shop().state.lock().unwrap().readback_override =
        Some(("custom.country".into(), json!("wrong")));
    run(&a, s);
    let p = latest(&a, s);
    assert_eq!(p.step, PublishStep::Failed);
    assert_eq!(p.failed_step, Some(PublishStep::Update));
    assert!(p.error.unwrap().contains("custom.country"));
}
#[test]
fn c2a_admin_added_tag_kept_without_write() {
    let base = vec!["a".into(), "b".into()];
    assert_eq!(
        merge_tags(&base, &base, &["a".into(), "b".into(), "c".into()]),
        None
    );
}
#[test]
fn c2b_local_removal_preserves_admin_addition() {
    assert_eq!(
        merge_tags(
            &["a".into(), "b".into()],
            &["a".into()],
            &["a".into(), "b".into(), "c".into()]
        ),
        Some(vec!["a".into(), "c".into()])
    );
}
#[test]
fn c2c_local_addition_merges_and_persists_base() {
    let a = app();
    let (s, item) = session(&a, "P14-TAGS");
    photo(&a, s, "p", C::Main);
    value(&a, item, "product.tags", json!(["x"]));
    let id = linked(&a, item, "P14-TAGS", "Oak", None);
    a.fake_shop()
        .state
        .lock()
        .unwrap()
        .tags
        .insert(id.clone(), vec!["y".into()]);
    run(&a, s);
    assert_eq!(
        a.fake_shop().state.lock().unwrap().tags[&id],
        vec!["y", "x"]
    );
    assert_eq!(
        a.store()
            .item_link(item, "shopify")
            .unwrap()
            .unwrap()
            .tags_base
            .as_deref(),
        Some("[\"y\",\"x\"]")
    );
}
#[test]
fn c3a_title_written_only_when_different() {
    let a = app();
    let (s, item) = session(&a, "P14-TITLE");
    photo(&a, s, "p", C::Main);
    let id = linked(&a, item, "P14-TITLE", "Oak", None);
    run(&a, s);
    assert!(
        !a.fake_shop()
            .state
            .lock()
            .unwrap()
            .mutations
            .iter()
            .any(|m| m.op == "update_title")
    );
    a.fake_shop()
        .state
        .lock()
        .unwrap()
        .products
        .iter_mut()
        .find(|p| p.external_id == id)
        .unwrap()
        .title = "Teak".into();
    run(&a, s);
    assert_eq!(a.fake_shop().item_get(&id).unwrap().title, "Oak");
    assert_eq!(
        a.fake_shop()
            .state
            .lock()
            .unwrap()
            .mutations
            .iter()
            .filter(|m| m.op == "update_title")
            .count(),
        1
    );
}
#[test]
fn c3b_manual_description_html_and_absence() {
    let a = app();
    let (s, item) = session(&a, "P14-DESC");
    photo(&a, s, "p", C::Main);
    let id = linked(&a, item, "P14-DESC", "Oak", None);
    run(&a, s);
    assert!(
        !a.fake_shop()
            .state
            .lock()
            .unwrap()
            .mutations
            .iter()
            .any(|m| m.op == "update_description")
    );
    value(&a, item, "product.description", json!("a\n\nb"));
    run(&a, s);
    assert_eq!(
        a.fake_shop().state.lock().unwrap().descriptions[&id],
        "<p>a</p><p>b</p>"
    );
}
#[test]
fn c3c_identity_sku_only_preserves_barcode_and_mismatch_fails() {
    let a = app();
    let (s, item) = session(&a, "K-2");
    photo(&a, s, "p", C::Main);
    let id = linked(&a, item, "K-1", "Oak", Some("123"));
    run(&a, s);
    let remote = a.fake_shop().item_get(&id).unwrap();
    assert_eq!(remote.sku.as_deref(), Some("K-2"));
    assert_eq!(remote.barcode.as_deref(), Some("123"));
    a.fake_shop()
        .state
        .lock()
        .unwrap()
        .products
        .iter_mut()
        .find(|p| p.external_id == id)
        .unwrap()
        .sku = Some("K-1".into());
    a.fake_shop().state.lock().unwrap().identity_override = Some(RemoteIdentity {
        sku: "wrong".into(),
        barcode: Some("123".into()),
    });
    run(&a, s);
    let p = latest(&a, s);
    assert_eq!(p.failed_step, Some(PublishStep::Update));
}
#[test]
fn c4a_selected_condition_files_in_order() {
    let a = app();
    let (s, item) = session(&a, "P14-C4A");
    schema(
        &a,
        vec![def("custom.damage_report", CoreType::MediaList, true)],
    );
    let d = photo(&a, s, "d", C::Condition);
    let e = photo(&a, s, "e", C::Condition);
    let f = photo(&a, s, "f", C::Condition);
    value(&a, item, "custom.damage_report", json!({"excluded":[e]}));
    let id = linked(&a, item, "P14-C4A", "Oak", None);
    run(&a, s);
    let role = "attribute:custom.damage_report";
    let fd = a
        .store()
        .photo_link(d, item, role)
        .unwrap()
        .unwrap()
        .external_id
        .unwrap();
    let ff = a
        .store()
        .photo_link(f, item, role)
        .unwrap()
        .unwrap()
        .external_id
        .unwrap();
    assert_eq!(
        remote_value(&a, &id, "custom.damage_report"),
        Some(json!([fd, ff]))
    );
    assert_eq!(a.fake_shop().state.lock().unwrap().files.len(), 2);
}
#[test]
fn c4b_republish_condition_set_keeps_old_file() {
    let a = app();
    let (s, item) = session(&a, "P14-C4B");
    schema(
        &a,
        vec![def("custom.damage_report", CoreType::MediaList, true)],
    );
    let d = photo(&a, s, "d", C::Condition);
    let f = photo(&a, s, "f", C::Condition);
    let id = linked(&a, item, "P14-C4B", "Oak", None);
    run(&a, s);
    let role = "attribute:custom.damage_report";
    let old = a
        .store()
        .photo_link(f, item, role)
        .unwrap()
        .unwrap()
        .external_id
        .unwrap();
    a.review().classify(f, C::Main).unwrap();
    let g = photo(&a, s, "g", C::Condition);
    run(&a, s);
    let fd = a
        .store()
        .photo_link(d, item, role)
        .unwrap()
        .unwrap()
        .external_id
        .unwrap();
    let fg = a
        .store()
        .photo_link(g, item, role)
        .unwrap()
        .unwrap()
        .external_id
        .unwrap();
    assert_eq!(
        remote_value(&a, &id, "custom.damage_report"),
        Some(json!([fd, fg]))
    );
    assert!(
        a.fake_shop()
            .state
            .lock()
            .unwrap()
            .files
            .iter()
            .any(|file| file.id == old)
    );
}
#[test]
fn c4c_no_selected_or_local_condition_value_keeps_remote() {
    let a = app();
    let (s, item) = session(&a, "P14-C4C");
    photo(&a, s, "m", C::Main);
    schema(
        &a,
        vec![def("custom.damage_report", CoreType::MediaList, true)],
    );
    let id = linked(&a, item, "P14-C4C", "Oak", None);
    a.fake_shop().state.lock().unwrap().values.insert(
        id.clone(),
        vec![("custom.damage_report".into(), json!(["existing"]))],
    );
    run(&a, s);
    assert_eq!(
        remote_value(&a, &id, "custom.damage_report"),
        Some(json!(["existing"]))
    );
    value(&a, item, "custom.damage_report", json!({"excluded":[]}));
    run(&a, s);
    assert_eq!(
        remote_value(&a, &id, "custom.damage_report"),
        Some(json!([]))
    );
}
#[test]
fn c4d_missing_condition_field_skips_files_and_notifies() {
    let a = TestApp::with_fake_shop_key("custom.nope").unwrap();
    a.clock.advance(1_761_350_400_000);
    let (s, item) = session(&a, "P14-C4D");
    photo(&a, s, "m", C::Main);
    photo(&a, s, "d", C::Condition);
    schema(&a, vec![def("custom.country", CoreType::Text, true)]);
    value(&a, item, "custom.country", json!("Sweden"));
    let id = linked(&a, item, "P14-C4D", "Oak", None);
    run(&a, s);
    assert_eq!(latest(&a, s).step, PublishStep::Done);
    assert_eq!(
        remote_value(&a, &id, "custom.country"),
        Some(json!("Sweden"))
    );
    assert!(a.fake_shop().state.lock().unwrap().files.is_empty());
    assert_eq!(a.fake_shop().item_get(&id).unwrap().media.len(), 1);
    assert!(
        a.store().notifications(Some(s)).unwrap()[0]
            .text
            .ends_with("Condition photos not published: field custom.nope not found in the shop")
    );
}
#[test]
fn c5a_retouch_prepares_revision_and_new_filename() {
    let a = app();
    let (s, item) = session(&a, "P14-RET");
    let p = photo(&a, s, "p", C::Main);
    let id = linked(&a, item, "P14-RET", "Oak", None);
    run(&a, s);
    let before = a.store().photo_link(p, item, "gallery").unwrap().unwrap();
    let old = before.external_id.unwrap();
    let raw = a.store().photo(p).unwrap().raw_path;
    a.store().adjusted_event(&raw, true).unwrap();
    a.fake_shop().fail_before_effect("staged_uploads_create");
    run(&a, s);
    let link = a.store().photo_link(p, item, "gallery").unwrap().unwrap();
    assert_eq!(link.revision, 1);
    assert_eq!(link.retouch_gen, 1);
    assert_eq!(link.upload_state, "none");
    assert_eq!(link.superseded_ids, vec![old]);
    assert!(
        a.fake_shop().item_get(&id).unwrap().media.iter().all(|m| !m
            .filename
            .as_deref()
            .unwrap_or("")
            .contains("-r1"))
    );
    a.app
        .publish
        .as_ref()
        .unwrap()
        .retry(latest(&a, s).id)
        .unwrap();
    drain(&a);
    let uid = a.store().photo(p).unwrap().uid.replace('-', "");
    let expected = format!("P14-RET-{}-r1.jpg", &uid[..12]);
    assert_eq!(
        a.fake_shop().item_get(&id).unwrap().media[0]
            .filename
            .as_deref(),
        Some(expected.as_str())
    );
}
#[test]
fn c5b_retouch_main_replaces_in_same_gallery_position() {
    let a = app();
    let (s, item) = session(&a, "P14-RMAIN");
    photo(&a, s, "first", C::Main);
    let p = photo(&a, s, "second", C::Main);
    let id = linked(&a, item, "P14-RMAIN", "Oak", None);
    run(&a, s);
    let old = a
        .store()
        .photo_link(p, item, "gallery")
        .unwrap()
        .unwrap()
        .external_id
        .unwrap();
    a.store()
        .adjusted_event(&a.store().photo(p).unwrap().raw_path, true)
        .unwrap();
    run(&a, s);
    let media = a.fake_shop().item_get(&id).unwrap().media;
    let new = a
        .store()
        .photo_link(p, item, "gallery")
        .unwrap()
        .unwrap()
        .external_id
        .unwrap();
    assert_eq!(media.len(), 2);
    assert_eq!(media[1].id, new);
    assert_ne!(old, new);
    assert!(!media.iter().any(|m| m.id == old));
}
#[test]
fn c5c_retouch_condition_points_to_new_file_and_keeps_old() {
    let a = app();
    let (s, item) = session(&a, "P14-RCOND");
    schema(
        &a,
        vec![def("custom.damage_report", CoreType::MediaList, true)],
    );
    let p = photo(&a, s, "d", C::Condition);
    let id = linked(&a, item, "P14-RCOND", "Oak", None);
    run(&a, s);
    let old = a
        .store()
        .photo_link(p, item, "attribute:custom.damage_report")
        .unwrap()
        .unwrap()
        .external_id
        .unwrap();
    a.store()
        .adjusted_event(&a.store().photo(p).unwrap().raw_path, true)
        .unwrap();
    run(&a, s);
    let new = a
        .store()
        .photo_link(p, item, "attribute:custom.damage_report")
        .unwrap()
        .unwrap()
        .external_id
        .unwrap();
    assert_ne!(old, new);
    assert_eq!(
        remote_value(&a, &id, "custom.damage_report"),
        Some(json!([new]))
    );
    assert!(
        a.fake_shop()
            .state
            .lock()
            .unwrap()
            .files
            .iter()
            .any(|f| f.id == old)
    );
}
#[test]
fn c5d_crash_adopts_only_current_revision() {
    let mut a = app();
    let (s, item) = session(&a, "P14-ADOPT");
    let p = photo(&a, s, "p", C::Main);
    let id = linked(&a, item, "P14-ADOPT", "Oak", None);
    run(&a, s);
    let old = a
        .store()
        .photo_link(p, item, "gallery")
        .unwrap()
        .unwrap()
        .external_id
        .unwrap();
    a.store()
        .adjusted_event(&a.store().photo(p).unwrap().raw_path, true)
        .unwrap();
    a.app
        .publish
        .as_ref()
        .unwrap()
        .confirm(&[s], "yes")
        .unwrap();
    a.fake_shop().fail_after_effect("product_create_media");
    let killed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| drain(&a)));
    assert!(killed.is_err());
    a.kill_and_restart().unwrap();
    drain(&a);
    let link = a.store().photo_link(p, item, "gallery").unwrap().unwrap();
    assert_eq!(link.revision, 1);
    assert_ne!(link.external_id.as_deref(), Some(old.as_str()));
    assert_eq!(a.fake_shop().item_get(&id).unwrap().media.len(), 1);
}
#[test]
fn c6a_summary_counts_exclusions_removal_and_create() {
    let a = app();
    let (s, _item) = session(&a, "P14-SUM");
    for name in ["m1", "m2", "m3"] {
        photo(&a, s, name, C::Main);
    }
    photo(&a, s, "d", C::Condition);
    photo(&a, s, "u1", C::Unreviewed);
    photo(&a, s, "u2", C::Unreviewed);
    let preview = a.app.publish.as_ref().unwrap().preview(&[s]).unwrap();
    assert!(preview.summary.contains("Publish 4 images"));
    assert!(
        preview
            .summary
            .contains("2 unreviewed photos will not be published")
    );
    assert!(
        preview
            .summary
            .contains("The product will be created on Shopify")
    );
    // A linked product with one foreign gallery photo yields the removal sentence.
    let (_s2, item2) = session(&a, "P14-SUM2");
    let id = linked(&a, item2, "P14-SUM2", "Oak", None);
    a.fake_shop()
        .state
        .lock()
        .unwrap()
        .products
        .iter_mut()
        .find(|p| p.external_id == id)
        .unwrap()
        .media
        .push(psm_core::domain::publish::RemoteMedia {
            id: "foreign".into(),
            filename: Some("foreign.jpg".into()),
            status: psm_core::domain::publish::RemoteMediaStatus::Ready,
            position: 1,
        });
    let preview = a.app.publish.as_ref().unwrap().preview(&[s, _s2]).unwrap();
    assert!(
        preview
            .summary
            .contains("1 photo will be removed from the gallery")
    );
}
#[test]
fn c7a_full_run_emits_prepare_upload_update_done_without_describe() {
    let a = app();
    let (s, item) = session(&a, "P14-STEPS");
    photo(&a, s, "m", C::Main);
    linked(&a, item, "P14-STEPS", "Oak", None);
    run(&a, s);
    let mut steps = Vec::new();
    for event in a.events.0.lock().unwrap().iter() {
        if let psm_core::events::AppEvent::PublishProgress {
            session_id, step, ..
        } = event
            && *session_id == s
            && steps.last() != Some(step)
        {
            steps.push(*step);
        }
    }
    assert_eq!(
        steps,
        vec![
            PublishStep::Prepare,
            PublishStep::Upload,
            PublishStep::Update,
            PublishStep::Done
        ]
    );
}
