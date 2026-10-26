use psm_core::{
    domain::{
        photo::{Classification, NewPhoto, PhotoSource},
        publish::PhotoLink,
        session::{ListFilter, PublishStep},
    },
    mechanisms::hash::sha256_file,
    ports::{clock::Clock, commerce::CommerceAdapter},
    services::sessions::SessionsService,
};
use psm_testkit::TestApp;
use std::fs;

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
fn photo(a: &TestApp, s: i64, name: &str) -> i64 {
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
    a.review().classify(p.id, Classification::Main).unwrap();
    a.store()
        .set_missing_preview_paths(&p.raw_path, "preview.jpg")
        .unwrap();
    p.id
}
fn service<'a>(a: &'a TestApp) -> SessionsService<'a> {
    SessionsService::new(a.store(), a.clock.as_ref())
        .with_commerce(a.commerce.as_deref().map(|c| c as &dyn CommerceAdapter))
}
fn failed(a: &TestApp, s: i64) -> i64 {
    a.app
        .publish
        .as_ref()
        .unwrap()
        .confirm(&[s], "Publish images")
        .unwrap();
    a.tick(1100).unwrap();
    a.complete_all_renders().unwrap();
    a.fake_shop().fail_before_effect("upload");
    a.tick(1100).unwrap();
    service(a).view(s).unwrap().publish.unwrap().item_id
}
fn drain(a: &TestApp) {
    for _ in 0..10 {
        a.tick(1100).unwrap();
        a.complete_all_renders().unwrap();
    }
}

#[test]
fn c7a_list_and_session_show_latest_confirmed_failure() {
    let a = app();
    let s = session(&a, "P15-A");
    photo(&a, s, "a");
    let other = session(&a, "P15-B");
    let item_id = failed(&a, s);
    let row = service(&a)
        .list(ListFilter::All, None)
        .unwrap()
        .groups
        .into_iter()
        .flat_map(|g| g.rows)
        .find(|r| r.id == s)
        .unwrap();
    let expected = Some(psm_core::domain::session::LatestPublish {
        item_id,
        step: PublishStep::Failed,
        failed_step: Some(PublishStep::Upload),
        error: Some("provider: upload unavailable".into()),
        uploaded: 0,
        total: 1,
    });
    assert_eq!(row.publish, expected);
    assert_eq!(service(&a).view(s).unwrap().publish, expected);
    assert_eq!(service(&a).view(other).unwrap().publish, None);
}

#[test]
fn c7b_session_photos_show_link_states_only_with_integration() {
    let a = app();
    let s = session(&a, "P15-C");
    let first = photo(&a, s, "first");
    let second = photo(&a, s, "second");
    let item = a.store().session(s).unwrap().session.item_id.unwrap();
    a.app
        .publish
        .as_ref()
        .unwrap()
        .confirm(&[s], "Publish images")
        .unwrap();
    a.store()
        .upsert_photo_link(
            &PhotoLink {
                photo_id: first,
                item_id: item,
                role: "gallery".into(),
                external_id: Some("remote-1".into()),
                upload_state: "ready".into(),
                export_sha256: None,
                superseded_ids: vec![],
                revision: 0,
                retouch_gen: 0,
            },
            a.clock.now_ms(),
        )
        .unwrap();
    a.store()
        .upsert_photo_link(
            &PhotoLink {
                photo_id: second,
                item_id: item,
                role: "gallery".into(),
                external_id: None,
                upload_state: "none".into(),
                export_sha256: None,
                superseded_ids: vec![],
                revision: 0,
                retouch_gen: 0,
            },
            a.clock.now_ms(),
        )
        .unwrap();
    assert_eq!(
        service(&a).view(s).unwrap().upload_states,
        vec![Some("ready".into()), Some("none".into())]
    );
    let no_shop = TestApp::new().unwrap();
    let bare = session(&no_shop, "P15-D");
    photo(&no_shop, bare, "third");
    assert_eq!(
        SessionsService::new(no_shop.store(), no_shop.clock.as_ref())
            .view(bare)
            .unwrap()
            .upload_states,
        vec![None]
    );
}

#[test]
fn c7c_failure_notification_links_to_retry_and_published_history() {
    let a = app();
    let s = session(&a, "P15-E");
    photo(&a, s, "a");
    let item_id = failed(&a, s);
    let first = a.store().notifications(None).unwrap();
    assert_eq!(first.len(), 1);
    assert_eq!(
        (
            first[0].kind.as_str(),
            first[0].text.as_str(),
            first[0].item_id
        ),
        (
            "failed",
            "Publish failed · provider: upload unavailable",
            item_id
        )
    );
    a.app.publish.as_ref().unwrap().retry(item_id).unwrap();
    drain(&a);
    let history = a.store().notifications(None).unwrap();
    assert_eq!(history.len(), 2);
    assert_eq!(history[0].kind, "published");
    assert_eq!(history[1].kind, "failed");
    assert_eq!((history[0].item_id, history[1].item_id), (item_id, item_id));
}
