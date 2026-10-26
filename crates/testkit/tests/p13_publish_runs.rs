use psm_core::{
    domain::{
        attribute::{AttributeGroup, CoreType, Definition, Provenance},
        photo::{Classification as C, NewPhoto, PhotoSource},
        publish::{DefinitionLink, PhotoLink, RemoteMedia, RemoteMediaStatus},
        session::{PublishStep, SessionStatus},
    },
    mechanisms::{hash::sha256_file, reconcile},
    ports::{clock::Clock, commerce::CommerceAdapter},
    services::attributes::AttributesService,
};
use psm_infra::settings_file::Settings;
use psm_integrations::{commerce::shopify::adapter::ShopifyConfig, commerce_from_settings};
use psm_testkit::TestApp;
use std::{collections::HashSet, fs};

fn app() -> TestApp {
    let a = TestApp::with_fake_shop().unwrap();
    a.clock.advance(1_761_350_400_000);
    a
}
fn session(a: &TestApp, sku: &str) -> i64 {
    a.items()
        .create_local(sku, "Oak chair", "Chair", None)
        .unwrap()
        .1
        .session
        .id
}
fn photo(a: &TestApp, session_id: i64, name: &str, class: C) -> i64 {
    let folder = a.dir.path().join("Capture/raws");
    fs::create_dir_all(&folder).unwrap();
    let path = folder.join(format!("{name}.ARW"));
    fs::write(&path, name.as_bytes()).unwrap();
    let p = a
        .store()
        .photo_insert_or_get(NewPhoto {
            session_id,
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
fn condition(a: &TestApp) {
    let def = Definition {
        integration: "shopify".into(),
        key: "custom.damage_report".into(),
        label: "Damage report".into(),
        core_type: CoreType::MediaList,
        choices: Vec::new(),
        validations: serde_json::json!({}),
        group: AttributeGroup::Condition,
        description: None,
        writable: true,
    };
    a.fake_shop().state.lock().unwrap().definitions = vec![(
        def,
        DefinitionLink {
            platform_type: "list.file_reference".into(),
            platform_meta: serde_json::json!({}),
        },
    )];
    AttributesService::new(a.store(), a.clock.as_ref(), Some("shopify"))
        .refresh_schema(a.fake_shop())
        .unwrap();
}
fn definition(key: &str, kind: CoreType) -> (Definition, DefinitionLink) {
    (
        Definition {
            integration: "shopify".into(),
            key: key.into(),
            label: key.into(),
            core_type: kind,
            choices: Vec::new(),
            validations: serde_json::json!({}),
            group: AttributeGroup::Details,
            description: None,
            writable: true,
        },
        DefinitionLink {
            platform_type: "single_line_text_field".into(),
            platform_meta: serde_json::json!({}),
        },
    )
}
fn wait_for_links(a: &TestApp, n: usize) {
    for _ in 0..100 {
        if a.store().definition_links("shopify").unwrap().len() == n {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    panic!("startup schema did not reach {n} links");
}
fn drain(a: &TestApp) {
    for _ in 0..10 {
        a.tick(1100).unwrap();
        a.complete_all_renders().unwrap();
    }
}
fn publish(a: &TestApp, session_id: i64) -> i64 {
    let id = a
        .app
        .publish
        .as_ref()
        .unwrap()
        .confirm(&[session_id], "Publish images")
        .unwrap();
    drain(a);
    id
}
fn start_upload(a: &TestApp, session_id: i64) -> i64 {
    let run = a
        .app
        .publish
        .as_ref()
        .unwrap()
        .confirm(&[session_id], "Publish images")
        .unwrap();
    a.tick(1100).unwrap();
    a.complete_all_renders().unwrap();
    run
}
fn interrupted_publish(op: &str) -> (TestApp, i64, i64) {
    let mut a = app();
    let s = session(&a, "VF-CRASH");
    let p = photo(&a, s, "first", C::Main);
    photo(&a, s, "second", C::Main);
    let run = start_upload(&a, s);
    a.fake_shop().fail_after_effect(op);
    let killed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        a.tick(1100).unwrap();
    }));
    assert!(killed.is_err());
    a.kill_and_restart().unwrap();
    drain(&a);
    (a, run, p)
}

#[test]
fn c1a_confirm_freezes_lists_and_excludes_unreviewed() {
    let a = app();
    condition(&a);
    let s = session(&a, "VF-0001");
    let main = photo(&a, s, "a", C::Main);
    let second = photo(&a, s, "b", C::Main);
    let third = photo(&a, s, "c", C::Main);
    let chosen = photo(&a, s, "d", C::Condition);
    let excluded = photo(&a, s, "e", C::Condition);
    let _unreviewed = photo(&a, s, "u", C::Unreviewed);
    let item_id = a.store().session(s).unwrap().session.item_id.unwrap();
    AttributesService::new(a.store(), a.clock.as_ref(), Some("shopify"))
        .set_value(
            item_id,
            "custom.damage_report",
            serde_json::json!({"excluded":[excluded]}),
            Provenance::Human,
        )
        .unwrap();
    let preview = a.app.publish.as_ref().unwrap().preview(&[s]).unwrap();
    assert_eq!(preview.items[0].main, vec![main, second, third]);
    assert_eq!(preview.items[0].condition, vec![chosen]);
    assert!(
        preview
            .summary
            .contains("1 unreviewed photos will not be published")
    );
    assert!(
        preview
            .summary
            .contains("The product will be created on Shopify")
    );
    let run = a
        .app
        .publish
        .as_ref()
        .unwrap()
        .confirm(&[s], &preview.summary)
        .unwrap();
    let id = a.store().active_publish_items().unwrap()[0];
    let item = a.store().publish_item(id).unwrap();
    assert_eq!(item.run_id, run);
    assert_eq!(item.frozen_main, vec![main, second, third]);
    assert_eq!(item.frozen_condition, vec![chosen]);
}
#[test]
fn c1b_confirm_freezes_only_canonical_valid_values() {
    let a = app();
    let s = session(&a, "VF-C1B");
    let item_id = a.store().session(s).unwrap().session.item_id.unwrap();
    a.fake_shop().state.lock().unwrap().definitions = vec![
        definition("custom.wood", CoreType::Text),
        definition("custom.year", CoreType::Integer),
    ];
    AttributesService::new(a.store(), a.clock.as_ref(), Some("shopify"))
        .refresh_schema(a.fake_shop())
        .unwrap();
    let attrs = AttributesService::new(a.store(), a.clock.as_ref(), Some("shopify"));
    attrs
        .set_value(
            item_id,
            "custom.wood",
            serde_json::json!("oak"),
            Provenance::Human,
        )
        .unwrap();
    attrs
        .set_value(
            item_id,
            "custom.year",
            serde_json::json!("wrong"),
            Provenance::Human,
        )
        .unwrap();
    a.app
        .publish
        .as_ref()
        .unwrap()
        .confirm(&[s], "confirmed")
        .unwrap();
    let item = a.store().publish_item(1).unwrap();
    assert_eq!(
        item.frozen_attributes,
        [("custom.wood".into(), serde_json::json!("oak"))].into()
    );
}
#[test]
fn c1c_preview_removals_and_exact_confirmation_text() {
    let a = app();
    let s = session(&a, "VF-C1C");
    let p = photo(&a, s, "a", C::Main);
    let item_id = a.store().session(s).unwrap().session.item_id.unwrap();
    let external = a.fake_shop().add_product("VF-C1C", "Oak", None);
    a.items().link(item_id, "shopify", &external, None).unwrap();
    a.store()
        .upsert_photo_link(
            &PhotoLink {
                photo_id: p,
                item_id,
                role: "gallery".into(),
                external_id: Some("m1".into()),
                upload_state: "ready".into(),
                export_sha256: None,
                superseded_ids: Vec::new(),
                revision: 0,
                retouch_gen: 0,
            },
            a.clock.now_ms(),
        )
        .unwrap();
    a.fake_shop().state.lock().unwrap().products[0].media = vec![
        RemoteMedia {
            id: "m1".into(),
            filename: Some("own.jpg".into()),
            status: RemoteMediaStatus::Ready,
            position: 1,
        },
        RemoteMedia {
            id: "m9".into(),
            filename: Some("foreign.jpg".into()),
            status: RemoteMediaStatus::Ready,
            position: 2,
        },
    ];
    let preview = a.app.publish.as_ref().unwrap().preview(&[s]).unwrap();
    assert_eq!(preview.items[0].announced_removals, vec!["m9"]);
    assert!(
        preview
            .summary
            .contains("1 photo will be removed from the gallery")
    );
    a.app
        .publish
        .as_ref()
        .unwrap()
        .confirm(&[s], "EXACT text!\n")
        .unwrap();
    let conn = rusqlite::Connection::open(a.app.paths.db()).unwrap();
    let text: String = conn
        .query_row(
            "SELECT confirmed_summary FROM publish_runs WHERE id=1",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(text, "EXACT text!\n");
    assert_eq!(
        a.store().publish_item(1).unwrap().announced_removals,
        vec!["m9"]
    );
}

#[test]
fn ready_nameless_announced_media_is_removed_and_publish_completes() {
    let a = app();
    let s = session(&a, "VF-NAMELESS");
    let p = photo(&a, s, "main", C::Main);
    let item_id = a.store().session(s).unwrap().session.item_id.unwrap();
    let external = a.fake_shop().add_product("VF-NAMELESS", "Oak", None);
    a.items().link(item_id, "shopify", &external, None).unwrap();
    a.fake_shop().state.lock().unwrap().products[0].media = vec![RemoteMedia {
        id: "ready-video".into(),
        filename: None,
        status: RemoteMediaStatus::Ready,
        position: 1,
    }];

    let run = publish(&a, s);
    let item = a.store().publish_item(1).unwrap();
    let gallery = a.fake_shop().item_get(&external).unwrap().media;
    let desired = a
        .store()
        .photo_link(p, item_id, "gallery")
        .unwrap()
        .unwrap();
    assert_eq!(item.run_id, run);
    assert_eq!(item.step, PublishStep::Done);
    assert_eq!(
        gallery
            .iter()
            .map(|media| media.id.as_str())
            .collect::<Vec<_>>(),
        [desired.external_id.as_deref().unwrap()]
    );
}

#[test]
fn capture_arrival_is_recorded_while_publish_media_poll_is_held() {
    let a = app();
    let s = session(&a, "VF-PARALLEL");
    photo(&a, s, "publish", C::Main);
    start_upload(&a, s);
    let commerce = a.commerce.as_ref().unwrap().clone();
    commerce.hold_media_poll();
    a.clock.advance(10_000);
    let jobs = a.app.publish_jobs.clone();
    let clock = a.clock.clone();
    let upload = std::thread::spawn(move || jobs.tick(clock.now_ms()).unwrap());
    if !commerce.wait_for_media_poll() {
        commerce.release_media_poll();
        upload.join().unwrap();
        panic!("publish worker did not reach media poll");
    }

    a.store().set_active_session_id(Some(s)).unwrap();
    let capture_root = a.app.arrival.capture_root().unwrap();
    let capture_path = capture_root.join("during-publish.arw");
    fs::write(&capture_path, b"capture arriving during publish").unwrap();
    a.app.arrival.scan().unwrap();
    let arrived = a.store().photos_in_session(s).unwrap();
    let capture_path = capture_path.to_string_lossy().into_owned();
    let released = arrived.iter().any(|photo| photo.raw_path == capture_path);
    commerce.release_media_poll();
    upload.join().unwrap();
    assert!(
        released,
        "capture photo row was not recorded before media poll release"
    );
}
#[test]
fn c1d_confirm_stores_current_change_sequence_and_time() {
    let a = app();
    let s = session(&a, "VF-C1D");
    photo(&a, s, "a", C::Main);
    let seq = a.store().session(s).unwrap().session.change_seq;
    let at = a.clock.now_ms();
    let run = a
        .app
        .publish
        .as_ref()
        .unwrap()
        .confirm(&[s], "yes")
        .unwrap();
    let item = a.store().publish_item(1).unwrap();
    assert_eq!(item.confirmed_change_seq, seq);
    assert_eq!(a.store().confirmed_run(run).unwrap(), Some(at));
}
#[test]
fn c2a_every_remote_mutation_carries_a_previously_confirmed_run() {
    let a = app();
    let s = session(&a, "VF-C2A");
    photo(&a, s, "a", C::Main);
    let run = publish(&a, s);
    let mutations = &a.fake_shop().state.lock().unwrap().mutations;
    assert!(!mutations.is_empty());
    for m in mutations {
        assert_eq!(m.run_id, run);
        assert!(a.store().confirmed_run(m.run_id).unwrap().unwrap() <= m.at);
    }
}
#[test]
fn c3a_prepare_creates_product_then_persists_link_before_next_remote_effect() {
    let a = app();
    let s = session(&a, "VF-C3A");
    photo(&a, s, "a", C::Main);
    start_upload(&a, s);
    let item_id = a.store().session(s).unwrap().session.item_id.unwrap();
    let link = a.store().item_link(item_id, "shopify").unwrap().unwrap();
    let shop = a.fake_shop().state.lock().unwrap();
    assert_eq!(shop.mutations[0].op, "item_create");
    assert_eq!(shop.mutations.len(), 1);
    assert!(!link.create_pending);
    assert_eq!(link.external_id, shop.products[0].external_id);
}
#[test]
fn c3b_prepare_adopts_one_exact_sku_match_without_create() {
    let a = app();
    let s = session(&a, "VF-C3B");
    photo(&a, s, "a", C::Main);
    let remote = a.fake_shop().add_product("VF-C3B", "Already", None);
    start_upload(&a, s);
    let item_id = a.store().session(s).unwrap().session.item_id.unwrap();
    assert_eq!(
        a.store()
            .item_link(item_id, "shopify")
            .unwrap()
            .unwrap()
            .external_id,
        remote
    );
    assert!(
        !a.fake_shop()
            .state
            .lock()
            .unwrap()
            .mutations
            .iter()
            .any(|m| m.op == "item_create")
    );
}
#[test]
fn c3c_prepare_fails_when_two_products_carry_sku() {
    let a = app();
    let s = session(&a, "VF-C3C");
    photo(&a, s, "a", C::Main);
    a.fake_shop().add_product("VF-C3C", "One", None);
    a.fake_shop().add_product("VF-C3C", "Two", None);
    a.app
        .publish
        .as_ref()
        .unwrap()
        .confirm(&[s], "yes")
        .unwrap();
    a.tick(1100).unwrap();
    let item = a.store().publish_item(1).unwrap();
    assert_eq!(item.step, PublishStep::Failed);
    assert_eq!(item.failed_step, Some(PublishStep::Prepare));
    assert!(item.error.unwrap().contains("several_products_carry_sku"));
}
#[test]
fn c3d_changed_raw_blocks_prepare_before_upload() {
    let a = app();
    let s = session(&a, "VF-C3D");
    let p = photo(&a, s, "a", C::Main);
    a.app
        .publish
        .as_ref()
        .unwrap()
        .confirm(&[s], "yes")
        .unwrap();
    fs::write(a.store().photo(p).unwrap().raw_path, b"changed").unwrap();
    a.tick(1100).unwrap();
    let item = a.store().publish_item(1).unwrap();
    assert_eq!(item.failed_step, Some(PublishStep::Prepare));
    assert!(item.error.unwrap().contains("raw_changed"));
    assert!(
        !a.fake_shop()
            .state
            .lock()
            .unwrap()
            .mutations
            .iter()
            .any(|m| m.op == "staged_uploads_create")
    );
}
#[test]
fn c3e_prepare_exports_to_staging_and_records_exact_file_hash() {
    let a = app();
    let s = session(&a, "VF-C3E");
    let p = photo(&a, s, "a", C::Main);
    start_upload(&a, s);
    a.app.publish.as_ref().unwrap().run_item(1).unwrap();
    let item = a.store().publish_item(1).unwrap();
    assert_eq!(item.step, PublishStep::Upload);
    let file = a
        .app
        .export
        .staging_folder("VF-C3E", item.export_run_id.unwrap())
        .join("VF-C3E_01_main.jpg");
    assert!(file.exists());
    let item_id = a.store().session(s).unwrap().session.item_id.unwrap();
    let link = a
        .store()
        .photo_link(p, item_id, "gallery")
        .unwrap()
        .unwrap();
    assert_eq!(link.export_sha256, Some(sha256_file(&file).unwrap()));
    assert_eq!(link.upload_state, "exported");
}

#[test]
fn c5a_foreign_media_blocks_reconciliation() {
    let desired = vec!["m1".into(), "m2".into()];
    let current = vec!["x".into(), "m1".into(), "m2".into()];
    assert_eq!(
        reconcile::plan(&desired, &current, &HashSet::new(), &HashSet::new()),
        Err(reconcile::ReconcileError::Foreign(1))
    );
    let a = app();
    let s = session(&a, "VF-C5A");
    photo(&a, s, "a", C::Main);
    let item_id = a.store().session(s).unwrap().session.item_id.unwrap();
    let external = a.fake_shop().add_product("VF-C5A", "Oak", None);
    a.items().link(item_id, "shopify", &external, None).unwrap();
    start_upload(&a, s);
    a.fake_shop().state.lock().unwrap().products[0]
        .media
        .push(RemoteMedia {
            id: "foreign".into(),
            filename: Some("foreign.jpg".into()),
            status: RemoteMediaStatus::Ready,
            position: 1,
        });
    drain(&a);
    let item = a.store().publish_item(1).unwrap();
    assert_eq!(item.failed_step, Some(PublishStep::Update));
    assert!(item.error.unwrap().contains("foreign_media_added"));
    assert!(
        !a.fake_shop()
            .state
            .lock()
            .unwrap()
            .mutations
            .iter()
            .any(|m| m.op == "product_delete_media")
    );
}
#[test]
fn c5b_reconciliation_deletes_owned_and_announced_and_moves_all() {
    let desired = vec!["m1".into(), "m2".into()];
    let current = vec!["o1".into(), "m2".into(), "m1".into(), "a1".into()];
    let plan = reconcile::plan(
        &desired,
        &current,
        &HashSet::from(["o1".into()]),
        &HashSet::from(["a1".into()]),
    )
    .unwrap();
    assert_eq!(plan.deletes, HashSet::from(["o1".into(), "a1".into()]));
    assert_eq!(plan.moves, vec![("m1".into(), 0), ("m2".into(), 1)]);
}
#[test]
fn c7a_done_records_publish_and_notification() {
    let a = app();
    let s = session(&a, "VF-0002");
    photo(&a, s, "a", C::Main);
    let run = publish(&a, s);
    let item = a
        .store()
        .publish_item(
            a.store()
                .active_publish_items()
                .unwrap()
                .first()
                .copied()
                .unwrap_or(1),
        )
        .unwrap();
    assert_eq!(item.run_id, run);
    assert_eq!(item.step, PublishStep::Done);
    let dto = a.store().session(s).unwrap();
    assert_eq!(dto.session.published_seq, dto.session.change_seq);
    assert!(dto.session.published_at.is_some());
    let n = a.store().notifications(Some(s)).unwrap();
    assert_eq!(n.len(), 1);
    assert_eq!(n[0].kind, "published");
    assert!(n[0].text.starts_with("Published · "));
    assert_eq!(
        a.sessions()
            .list(psm_core::domain::session::ListFilter::All, None)
            .unwrap()
            .groups
            .iter()
            .flat_map(|g| g.rows.iter())
            .find(|r| r.id == s)
            .unwrap()
            .status,
        SessionStatus::Published
    );
}
#[test]
fn c4a_resume_after_staging_effect_has_one_media_per_photo() {
    let (a, _, _) = interrupted_publish("staged_uploads_create");
    let s = a.fake_shop().state.lock().unwrap();
    assert_eq!(s.products.len(), 1);
    assert_eq!(s.products[0].media.len(), 2);
    let item = a.store().publish_item(1).unwrap();
    assert_eq!(item.step, PublishStep::Done);
    assert_eq!(
        a.store()
            .photo_links_for_item(
                a.store()
                    .session(item.session_id)
                    .unwrap()
                    .session
                    .item_id
                    .unwrap()
            )
            .unwrap()
            .iter()
            .filter(|l| l.upload_state == "ready")
            .count(),
        2
    );
}
#[test]
fn c4b_resume_adopts_media_created_before_local_record() {
    let (a, _, _) = interrupted_publish("product_create_media");
    let s = a.fake_shop().state.lock().unwrap();
    assert_eq!(s.products[0].media.len(), 2);
    assert_eq!(
        s.mutations
            .iter()
            .filter(|m| m.op == "product_create_media")
            .count(),
        2
    );
}
#[test]
fn c4c_resume_polls_attached_media_without_creating_again() {
    let (a, _, _) = interrupted_publish("poll_media_ready");
    let s = a.fake_shop().state.lock().unwrap();
    assert_eq!(s.products[0].media.len(), 2);
    assert_eq!(
        s.mutations
            .iter()
            .filter(|m| m.op == "product_create_media")
            .count(),
        2
    );
}
#[test]
fn c4d_ready_link_skips_staging_on_next_publish() {
    let a = app();
    let s = session(&a, "VF-C4D");
    photo(&a, s, "a", C::Main);
    publish(&a, s);
    let before = a
        .fake_shop()
        .state
        .lock()
        .unwrap()
        .mutations
        .iter()
        .filter(|m| m.op == "staged_uploads_create")
        .count();
    publish(&a, s);
    let after = a
        .fake_shop()
        .state
        .lock()
        .unwrap()
        .mutations
        .iter()
        .filter(|m| m.op == "staged_uploads_create")
        .count();
    assert_eq!(after, before);
}
#[test]
fn c4e_moved_photo_uploads_for_new_item_without_changing_old_link() {
    let a = app();
    let first = session(&a, "VF-C4E-A");
    let second = session(&a, "VF-C4E-B");
    let p = photo(&a, first, "a", C::Main);
    publish(&a, first);
    let first_item = a.store().session(first).unwrap().session.item_id.unwrap();
    let old = a
        .store()
        .photo_link(p, first_item, "gallery")
        .unwrap()
        .unwrap();
    a.review().move_photos(&[p], second).unwrap();
    publish(&a, second);
    let second_item = a.store().session(second).unwrap().session.item_id.unwrap();
    let new = a
        .store()
        .photo_link(p, second_item, "gallery")
        .unwrap()
        .unwrap();
    assert_eq!(
        old,
        a.store()
            .photo_link(p, first_item, "gallery")
            .unwrap()
            .unwrap()
    );
    assert_eq!(new.upload_state, "ready");
    assert_ne!(new.external_id, old.external_id);
}
#[test]
fn c5c_update_mirrors_main_order_and_keeps_condition_out_of_gallery() {
    let a = app();
    condition(&a);
    let s = session(&a, "VF-C5C");
    let first = photo(&a, s, "a", C::Main);
    let second = photo(&a, s, "b", C::Main);
    let cond = photo(&a, s, "c", C::Condition);
    publish(&a, s);
    let item_id = a.store().session(s).unwrap().session.item_id.unwrap();
    let expected = [first, second].map(|p| {
        a.store()
            .photo_link(p, item_id, "gallery")
            .unwrap()
            .unwrap()
            .external_id
            .unwrap()
    });
    let shop = a.fake_shop().state.lock().unwrap();
    assert_eq!(
        shop.products[0]
            .media
            .iter()
            .map(|m| m.id.as_str())
            .collect::<Vec<_>>(),
        expected.iter().map(String::as_str).collect::<Vec<_>>()
    );
    assert_eq!(shop.products[0].media[0].id, expected[0]);
    assert!(
        a.store()
            .photo_link(cond, item_id, "attribute:custom.damage_report")
            .unwrap()
            .is_some()
    );
}
#[test]
fn c5d_wrong_order_readback_fails_at_update() {
    let a = app();
    let s = session(&a, "VF-C5D");
    photo(&a, s, "a", C::Main);
    photo(&a, s, "b", C::Main);
    a.fake_shop().state.lock().unwrap().wrong_order = true;
    a.app
        .publish
        .as_ref()
        .unwrap()
        .confirm(&[s], "yes")
        .unwrap();
    drain(&a);
    let item = a.store().publish_item(1).unwrap();
    assert_eq!(item.step, PublishStep::Failed);
    assert_eq!(item.failed_step, Some(PublishStep::Update));
    assert_eq!(item.error.as_deref(), Some("gallery_verification"));
}
#[test]
fn c5e_update_refuses_unready_desired_media_before_deletion() {
    let a = app();
    let s = session(&a, "VF-C5E");
    let p = photo(&a, s, "a", C::Main);
    let item_id = a.store().session(s).unwrap().session.item_id.unwrap();
    let remote = a.fake_shop().add_product("VF-C5E", "Oak", None);
    a.items().link(item_id, "shopify", &remote, None).unwrap();
    a.store()
        .upsert_photo_link(
            &PhotoLink {
                photo_id: p,
                item_id,
                role: "gallery".into(),
                external_id: Some("m1".into()),
                upload_state: "attached".into(),
                export_sha256: None,
                superseded_ids: Vec::new(),
                revision: 0,
                retouch_gen: 0,
            },
            a.clock.now_ms(),
        )
        .unwrap();
    a.app
        .publish
        .as_ref()
        .unwrap()
        .confirm(&[s], "yes")
        .unwrap();
    a.store()
        .set_publish_step(1, PublishStep::Update, None, None, a.clock.now_ms())
        .unwrap();
    a.app.publish.as_ref().unwrap().run_item(1).unwrap();
    let item = a.store().publish_item(1).unwrap();
    assert_eq!(item.failed_step, Some(PublishStep::Update));
    assert_eq!(item.error.as_deref(), Some("upload_incomplete"));
    assert!(
        !a.fake_shop()
            .state
            .lock()
            .unwrap()
            .mutations
            .iter()
            .any(|m| m.op == "product_delete_media")
    );
}
#[test]
fn c5f_not_found_delete_reply_counts_as_removed() {
    let a = app();
    let s = session(&a, "VF-C5F");
    photo(&a, s, "a", C::Main);
    let item_id = a.store().session(s).unwrap().session.item_id.unwrap();
    let remote = a.fake_shop().add_product("VF-C5F", "Oak", None);
    a.items().link(item_id, "shopify", &remote, None).unwrap();
    a.fake_shop().state.lock().unwrap().products[0]
        .media
        .push(RemoteMedia {
            id: "gone".into(),
            filename: Some("gone.jpg".into()),
            status: RemoteMediaStatus::Ready,
            position: 1,
        });
    a.fake_shop().state.lock().unwrap().delete_not_found = true;
    publish(&a, s);
    assert_eq!(a.store().publish_item(1).unwrap().step, PublishStep::Done);
    assert!(a.fake_shop().state.lock().unwrap().not_found_delivered);
    assert_eq!(
        a.fake_shop().state.lock().unwrap().products[0].media.len(),
        1
    );
}
#[test]
fn c6a_deleted_after_confirmation_is_not_uploaded_or_in_gallery() {
    let a = app();
    let s = session(&a, "VF-C6A");
    let keep = photo(&a, s, "a", C::Main);
    let gone = photo(&a, s, "b", C::Main);
    a.app
        .publish
        .as_ref()
        .unwrap()
        .confirm(&[s], "yes")
        .unwrap();
    a.review().delete(gone).unwrap();
    drain(&a);
    let item_id = a.store().session(s).unwrap().session.item_id.unwrap();
    assert!(
        a.store()
            .photo_link(gone, item_id, "gallery")
            .unwrap()
            .is_none()
    );
    assert!(
        a.store()
            .photo_link(keep, item_id, "gallery")
            .unwrap()
            .is_some()
    );
    assert_eq!(
        a.fake_shop().state.lock().unwrap().products[0].media.len(),
        1
    );
    assert_eq!(a.store().notifications(Some(s)).unwrap().len(), 1);
}
#[test]
fn c6b_moved_after_confirmation_is_skipped() {
    let a = app();
    let s = session(&a, "VF-C6B");
    let other = session(&a, "VF-C6B-OTHER");
    let p = photo(&a, s, "a", C::Main);
    a.app
        .publish
        .as_ref()
        .unwrap()
        .confirm(&[s], "yes")
        .unwrap();
    a.review().move_photos(&[p], other).unwrap();
    drain(&a);
    let item_id = a.store().session(s).unwrap().session.item_id.unwrap();
    assert!(
        a.store()
            .photo_link(p, item_id, "gallery")
            .unwrap()
            .is_none()
    );
    assert!(
        a.fake_shop().state.lock().unwrap().products[0]
            .media
            .is_empty()
    );
}
#[test]
fn c7b_photo_change_during_run_remains_dirty_after_done() {
    let a = app();
    let s = session(&a, "VF-C7B");
    let p = photo(&a, s, "a", C::Main);
    start_upload(&a, s);
    a.review().classify(p, C::Condition).unwrap();
    drain(&a);
    let dto = a.store().session(s).unwrap();
    assert!(dto.dirty);
    assert!(dto.session.change_seq > dto.session.published_seq);
}
#[test]
fn c7c_failed_upload_updates_activity_and_status() {
    let a = app();
    let s = session(&a, "VF-C7C");
    photo(&a, s, "a", C::Main);
    start_upload(&a, s);
    let before = a.store().session(s).unwrap().session.last_activity_at;
    a.fake_shop().fail_before_effect("upload");
    a.tick(1100).unwrap();
    let item = a.store().publish_item(1).unwrap();
    assert_eq!(item.failed_step, Some(PublishStep::Upload));
    assert!(a.store().session(s).unwrap().session.last_activity_at > before);
    let row = a
        .sessions()
        .list(psm_core::domain::session::ListFilter::All, None)
        .unwrap()
        .groups
        .into_iter()
        .flat_map(|g| g.rows)
        .find(|r| r.id == s)
        .unwrap();
    assert_eq!(row.status, SessionStatus::Failed);
    let b = app();
    let published = session(&b, "VF-C7C-PREV");
    photo(&b, published, "a", C::Main);
    publish(&b, published);
    photo(&b, published, "b", C::Main);
    start_upload(&b, published);
    b.fake_shop().fail_before_effect("upload");
    b.tick(1100).unwrap();
    let row = b
        .sessions()
        .list(psm_core::domain::session::ListFilter::All, None)
        .unwrap()
        .groups
        .into_iter()
        .flat_map(|g| g.rows)
        .find(|r| r.id == published)
        .unwrap();
    assert_eq!(
        row.label,
        psm_core::domain::session::RowLabel::PublishFailed
    );
}
#[test]
fn c8a_retry_upload_does_not_rerun_prepare() {
    let a = app();
    let s = session(&a, "VF-C8A");
    photo(&a, s, "a", C::Main);
    start_upload(&a, s);
    a.fake_shop().fail_before_effect("upload");
    a.tick(1100).unwrap();
    let reads = a.fake_shop().state.lock().unwrap().reads.len();
    a.app.publish.as_ref().unwrap().retry(1).unwrap();
    drain(&a);
    let shop = a.fake_shop().state.lock().unwrap();
    assert_eq!(a.store().publish_item(1).unwrap().step, PublishStep::Done);
    assert!(!shop.reads[reads..].iter().any(|r| r == "item_search"));
    assert_eq!(
        shop.mutations
            .iter()
            .filter(|m| m.op == "item_create")
            .count(),
        1
    );
}
#[test]
fn c8b_restart_requeues_running_upload_job_and_completes() {
    let mut a = app();
    let s = session(&a, "VF-C8B");
    photo(&a, s, "a", C::Main);
    start_upload(&a, s);
    a.fake_shop().fail_after_effect("product_create_media");
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| a.tick(1100).unwrap())).is_err()
    );
    let conn = rusqlite::Connection::open(a.app.paths.db()).unwrap();
    let status: String = conn
        .query_row(
            "SELECT status FROM jobs WHERE kind='publish_step' ORDER BY id DESC LIMIT 1",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(status, "running");
    drop(conn);
    a.kill_and_restart().unwrap();
    let conn = rusqlite::Connection::open(a.app.paths.db()).unwrap();
    let status: String = conn
        .query_row(
            "SELECT status FROM jobs WHERE kind='publish_step' ORDER BY id DESC LIMIT 1",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(status, "queued");
    drop(conn);
    drain(&a);
    assert_eq!(a.store().publish_item(1).unwrap().step, PublishStep::Done);
    assert_eq!(
        a.fake_shop().state.lock().unwrap().products[0].media.len(),
        1
    );
}
#[test]
fn c9a_settings_build_shopify_adapter_with_default_condition_key_or_none() {
    let mut settings = Settings::default();
    assert!(
        commerce_from_settings(
            &settings,
            std::sync::Arc::new(psm_testkit::clock::ManualClock::new(0))
        )
        .unwrap()
        .is_none()
    );
    settings.integration = Some(
        serde_json::json!({"kind":"shopify","shop_domain":"x.myshopify.com","api_version":"2026-10","client_id":"c","client_secret":"s"}),
    );
    let typed = settings.integration_settings().unwrap().unwrap();
    assert_eq!(
        ShopifyConfig::from(&typed).condition_photos_key,
        "custom.damage_report"
    );
    let adapter = commerce_from_settings(
        &settings,
        std::sync::Arc::new(psm_testkit::clock::ManualClock::new(0)),
    )
    .unwrap()
    .unwrap();
    assert_eq!(
        adapter.capabilities(),
        psm_core::ports::commerce::ALL_CAPABILITIES
    );
}
#[test]
fn c9b_schema_refresh_replaces_definitions_and_links_atomically() {
    let mut a = app();
    a.fake_shop().state.lock().unwrap().definitions = vec![
        definition("a", CoreType::Text),
        definition("b", CoreType::Integer),
        definition("c", CoreType::Boolean),
    ];
    a.restart_with_startup().unwrap();
    wait_for_links(&a, 3);
    assert_eq!(
        AttributesService::new(a.store(), a.clock.as_ref(), Some("shopify"))
            .definitions(Some("shopify"))
            .unwrap()
            .len(),
        6
    );
    a.fake_shop().state.lock().unwrap().definitions = vec![
        definition("a", CoreType::Text),
        definition("b", CoreType::Integer),
    ];
    a.restart_with_startup().unwrap();
    wait_for_links(&a, 2);
    assert_eq!(
        AttributesService::new(a.store(), a.clock.as_ref(), Some("shopify"))
            .definitions(Some("shopify"))
            .unwrap()
            .len(),
        5
    );
    a.fake_shop().state.lock().unwrap().schema_error = true;
    a.restart_with_startup().unwrap();
    std::thread::sleep(std::time::Duration::from_millis(100));
    assert_eq!(
        AttributesService::new(a.store(), a.clock.as_ref(), Some("shopify"))
            .definitions(Some("shopify"))
            .unwrap()
            .len(),
        5
    );
    assert_eq!(a.store().definition_links("shopify").unwrap().len(), 2);
}
#[test]
fn c9c_search_merges_local_and_unlinked_shop_hits_and_survives_outage() {
    let a = app();
    let local = a
        .items()
        .create_local("L-1", "Oak table", "", None)
        .unwrap()
        .0;
    let already = a.fake_shop().add_product("L-1", "Oak table", None);
    a.items().link(local.id, "shopify", &already, None).unwrap();
    let chair = a.fake_shop().add_product("K-1", "Oak chair", None);
    let search = || {
        a.items()
            .with_commerce(Some(a.fake_shop()))
            .search("Oak")
            .unwrap()
    };
    let found = search();
    assert_eq!(found.len(), 2);
    assert_eq!(found[0].sku, "L-1");
    assert_eq!(found[1].sku, "K-1");
    assert_eq!(found[1].id, 0);
    assert_eq!(found[1].external_id.as_deref(), Some(chair.as_str()));
    a.fake_shop().state.lock().unwrap().unreachable = true;
    let found = search();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].sku, "L-1");
}
#[test]
fn c9d_adopt_shop_hit_creates_linked_item_then_session() {
    let a = app();
    let remote = a.fake_shop().add_product("K-1", "Oak chair", Some("BAR-1"));
    let item = a
        .items()
        .with_commerce(Some(a.fake_shop()))
        .adopt(&remote)
        .unwrap();
    let s = a.sessions().create(item.id).unwrap();
    assert_eq!(item.sku, "K-1");
    assert_eq!(item.title, "Oak chair");
    assert_eq!(item.barcode.as_deref(), Some("BAR-1"));
    assert_eq!(item.origin, psm_core::domain::item::ItemOrigin::Linked);
    assert_eq!(
        a.store()
            .item_link(item.id, "shopify")
            .unwrap()
            .unwrap()
            .external_id,
        remote
    );
    assert_eq!(s.session.item_id, Some(item.id));
}
#[test]
fn c9e_shop_sku_collision_blocks_create_and_foreign_edit_but_allows_own_or_outage() {
    let a = app();
    let remote = a.fake_shop().add_product("K-7", "Shop item", None);
    let items = || a.items().with_commerce(Some(a.fake_shop()));
    assert!(
        matches!(items().create_local("K-7","Wrong","",None),Err(psm_core::Error::SkuInShop{sku}) if sku=="K-7")
    );
    let other = items().create_local("K-8", "Other", "", None).unwrap().0;
    assert!(
        matches!(items().update_identity(other.id,Some("K-7"),None),Err(psm_core::Error::SkuInShop{sku}) if sku=="K-7")
    );
    let own = items().adopt(&remote).unwrap();
    assert_eq!(
        items()
            .update_identity(own.id, Some("K-7"), None)
            .unwrap()
            .sku,
        "K-7"
    );
    a.fake_shop().add_product("K-9", "Remote", None);
    a.fake_shop().state.lock().unwrap().unreachable = true;
    assert_eq!(
        items()
            .update_identity(other.id, Some("K-9"), None)
            .unwrap()
            .sku,
        "K-9"
    );
}
#[test]
fn c9g_next_sku_skips_remote_candidates_then_falls_back_on_outage() {
    let a = app();
    a.items()
        .create_local("VF-0420", "Local", "", None)
        .unwrap();
    let items = || a.items().with_commerce(Some(a.fake_shop()));
    a.fake_shop().add_product("VF-0421", "Remote", None);
    assert_eq!(
        items().next_sku(Some("VF")).unwrap().as_deref(),
        Some("VF-0422")
    );
    a.fake_shop().add_product("VF-0422", "Remote", None);
    assert_eq!(
        items().next_sku(Some("VF")).unwrap().as_deref(),
        Some("VF-0423")
    );
    a.fake_shop().state.lock().unwrap().unreachable = true;
    assert_eq!(
        items().next_sku(Some("VF")).unwrap().as_deref(),
        Some("VF-0421")
    );
}
