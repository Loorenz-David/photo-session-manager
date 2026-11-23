//! H4·U1 — the remote view, activation, what the screen is given (master plan archive §5
//! MC-H3.2–5, .8, MC-H1.3–4, M-H1, M-H4; book 10H §6). `RemoteItems::view` / `run_pending`,
//! `SessionsService::{activate, view}`, `AttributesService`, `ConversationsService::attachable`
//! on `TestApp::with_fake_shop()` (`with_pricing()` for the attachable row). Outcomes: returned
//! values, files, the operations the fake Capture One received, `active_session_id`, what
//! `FakeCommerce` recorded. Rows H4·C2(c) and H4·C4 are command tests (`h4_tests` in the desktop
//! crate).

use crate::h3_bring_back::{adopted, archived_of};
use crate::p62_retention_after_publish::shop_app;
use psm_core::{
    Error,
    domain::{
        attribute::Provenance,
        composition::OutputSlot,
        photo::Classification as C,
        publish::{PhotoLink, RemoteMedia, RemoteMediaStatus},
        remote::{PictureTarget, RemoteKind, RemoteState, RemoteView},
        storage::ArchivedImage,
    },
    events::AppEvent,
    ports::{clock::Clock, conversations::ConversationStore},
    services::sessions::SessionsService,
};
use psm_testkit::{
    TestApp,
    fake_commerce::{ImageFault, fake_image_url},
    test_app::PRICING_KEY,
};
use serde_json::json;
use sha2::Digest;
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

const ARCHIVED: RemoteState = RemoteState::Remote(RemoteKind::Archived);
const SHOP_ONLY: RemoteState = RemoteState::Remote(RemoteKind::ShopOnly);

// ---------------------------------------------------------------------------------------------
// fixtures

/// A local item whose files were cleared with this record (`mark_item_cleared`): gallery then
/// condition media ids, each at `fake_image_url(id)` with `bytes`. (session id, item id)
fn archived_record(
    a: &TestApp,
    sku: &str,
    gallery: &[String],
    condition: &[String],
    bytes: Option<i64>,
) -> (i64, i64) {
    let (item, session) = a
        .items()
        .create_local(sku, &format!("{sku} title"), "Sideboard", None)
        .unwrap();
    let mut images = Vec::new();
    for (role, ids) in [("gallery", gallery), ("condition", condition)] {
        for (i, id) in ids.iter().enumerate() {
            images.push(ArchivedImage {
                role: role.into(),
                position: i as i64 + 1,
                media_id: id.clone(),
                url: fake_image_url(id),
                bytes,
            });
        }
    }
    a.store()
        .mark_item_cleared(item.id, a.clock.now_ms(), &images)
        .unwrap();
    (session.session.id, item.id)
}

fn ids(prefix: &str, n: usize) -> Vec<String> {
    (1..=n).map(|i| format!("{prefix}{i:02}")).collect()
}

/// The archived item (3 + 2) of H4·C1(a): sizes 1 000 000 each, nothing cached.
fn three_and_two(a: &TestApp) -> (i64, i64) {
    archived_record(
        a,
        "ARC-5",
        &ids("arc5-g", 3),
        &ids("arc5-c", 2),
        Some(1_000_000),
    )
}

fn view(a: &TestApp, session: i64) -> RemoteView {
    a.remote().view(session).unwrap().unwrap()
}

/// `<previews>/archived/<item>/<role>-<NN>-<first 12 hex of sha256(media id)>.jpg`.
fn expected_file(a: &TestApp, item: i64, role: &str, position: i64, media: &str) -> PathBuf {
    let hex: String = sha2::Sha256::digest(media.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    a.app
        .paths
        .root
        .join("previews/archived")
        .join(item.to_string())
        .join(format!("{role}-{position:02}-{}.jpg", &hex[..12]))
}

fn fetches(a: &TestApp) -> Vec<(String, u32)> {
    a.fake_shop().image_fetches()
}

fn reads(a: &TestApp, op: &str) -> usize {
    a.fake_shop()
        .state
        .lock()
        .unwrap()
        .reads
        .iter()
        .filter(|r| r.as_str() == op)
        .count()
}

fn events(a: &TestApp) -> Vec<AppEvent> {
    a.events.0.lock().unwrap().clone()
}

fn session_changes(a: &TestApp, session: i64) -> usize {
    events(a)
        .iter()
        .filter(|e| matches!(e, AppEvent::SessionChanged { session_id } if *session_id == session))
        .count()
}

fn remote_ready(a: &TestApp) -> Vec<(i64, String, i64, String)> {
    events(a)
        .into_iter()
        .filter_map(|e| match e {
            AppEvent::PictureReady {
                target:
                    PictureTarget::Remote {
                        session_id,
                        role,
                        position,
                    },
                path,
            } => Some((session_id, role, position, path)),
            _ => None,
        })
        .collect()
}

fn ops(a: &TestApp) -> Vec<String> {
    a.fake.state.lock().unwrap().ops.clone()
}

fn with_remote(a: &TestApp) -> SessionsService<'_> {
    a.sessions().with_remote(a.remote())
}

/// An adopted shop product (2 gallery media) whose first-open read ran: `Remote(ShopOnly)`.
fn shop_only(a: &TestApp, sku: &str) -> (i64, i64) {
    let r = adopted(a, sku, &[&format!("{sku}-m1"), &format!("{sku}-m2")], &[]);
    assert_eq!(a.remote().ensure_record(r.session).unwrap(), SHOP_ONLY);
    (r.session, r.item)
}

// ---------------------------------------------------------------------------------------------
// H4·C1 — the remote view and its background downloads

#[test]
fn h4_c1a_the_view_answers_at_once_without_downloading() {
    let a = shop_app();
    let (session, item) = three_and_two(&a);
    let v = view(&a, session);
    assert_eq!((v.item_id, v.state), (item, ARCHIVED));
    let got: Vec<(String, i64, Option<PathBuf>, bool)> = v
        .pictures
        .iter()
        .map(|p| (p.role.clone(), p.position, p.path.clone(), p.gone))
        .collect();
    assert_eq!(
        got,
        vec![
            ("gallery".into(), 1, None, false),
            ("gallery".into(), 2, None, false),
            ("gallery".into(), 3, None, false),
            ("condition".into(), 1, None, false),
            ("condition".into(), 2, None, false),
        ]
    );
    assert_eq!(v.bring_back_bytes, 5_000_000);
    assert_eq!(fetches(&a), vec![]);
}

/// H4·C1(a) then (b)'s run: the second item's 10 pictures viewed too, fetches 50 ms each.
fn viewed_and_run(a: &TestApp) -> [(i64, i64); 2] {
    let first = three_and_two(a);
    let second = archived_record(a, "ARC-10", &ids("arc10-g", 10), &[], None);
    view(a, first.0);
    view(a, second.0);
    a.fake_shop().set_fetch_delay(50);
    a.remote().run_pending().unwrap();
    [first, second]
}

#[test]
fn h4_c1b_the_background_writes_each_picture_four_at_a_time() {
    let a = shop_app();
    let items = viewed_and_run(&a);
    let mut expected_fetches = Vec::new();
    let mut expected_events = BTreeSet::new();
    for (session, item) in items {
        for image in a.store().archived_images(item).unwrap() {
            let file = expected_file(&a, item, &image.role, image.position, &image.media_id);
            assert!(file.is_file(), "{file:?}");
            expected_fetches.push((image.url.clone(), 1600));
            expected_events.insert((
                session,
                image.role.clone(),
                image.position,
                file.to_string_lossy().into_owned(),
            ));
        }
    }
    let mut got = fetches(&a);
    got.sort();
    expected_fetches.sort();
    assert_eq!(got, expected_fetches);
    let ready = remote_ready(&a);
    assert_eq!(ready.len(), 15);
    assert_eq!(ready.into_iter().collect::<BTreeSet<_>>(), expected_events);
    assert_eq!(a.fake_shop().max_fetches_in_flight(), 4);
}

#[test]
fn h4_c1c_a_cached_picture_is_served_from_its_file() {
    let a = shop_app();
    let items = viewed_and_run(&a);
    let before = fetches(&a).len();
    for (session, item) in items {
        for p in view(&a, session).pictures {
            assert_eq!(
                p.path,
                Some(expected_file(&a, item, &p.role, p.position, &p.media_id))
            );
        }
    }
    a.remote().run_pending().unwrap();
    assert_eq!(fetches(&a).len(), before);
}

#[test]
fn h4_c1d_a_local_session_and_the_unassigned_session() {
    let a = shop_app();
    let (item, session) = a
        .items()
        .create_local("LOC-1", "Lamp", "Lamp", None)
        .unwrap();
    a.remote().set_keep_files(item.id, true).unwrap();
    let v = view(&a, session.session.id);
    assert_eq!(
        (v.state, v.pictures.len(), v.keep_files),
        (RemoteState::Local, 0, true)
    );
    assert_eq!(a.remote().view(1).unwrap(), None);
}

#[test]
fn h4_c1e_the_first_open_read_is_queued_by_the_view() {
    let a = shop_app();
    let r = adopted(&a, "SHOP-E", &["e-m1", "e-m2"], &[]);
    let first = view(&a, r.session);
    assert_eq!(
        (first.state, first.pictures.len(), first.check_failed),
        (RemoteState::Unknown, 0, false)
    );
    assert_eq!(reads(&a, "media_readback"), 0);
    a.remote().run_pending().unwrap();
    assert_eq!(reads(&a, "media_readback"), 1);
    assert_eq!(session_changes(&a, r.session), 1);
    let after = view(&a, r.session);
    assert_eq!((after.state, after.pictures.len()), (SHOP_ONLY, 2));
}

#[test]
fn h4_c1f_a_failed_first_open_read_is_reported_then_tried_again() {
    let a = shop_app();
    let r = adopted(&a, "SHOP-F", &["f-m1", "f-m2"], &[]);
    a.fake_shop().state.lock().unwrap().unreachable = true;
    view(&a, r.session);
    a.remote().run_pending().unwrap();
    let failed = view(&a, r.session);
    assert_eq!(
        (failed.state, failed.check_failed),
        (RemoteState::Unknown, true)
    );
    assert_eq!(session_changes(&a, r.session), 1);
    a.fake_shop().state.lock().unwrap().unreachable = false;
    view(&a, r.session);
    a.remote().run_pending().unwrap();
    let read = view(&a, r.session);
    assert_eq!((read.state, read.check_failed), (SHOP_ONLY, false));
}

#[test]
fn h4_c1g_gone_addresses_are_refreshed_in_chunks_of_25() {
    let a = shop_app();
    let gallery = ids("g30-", 30);
    {
        let mut s = a.fake_shop().state.lock().unwrap();
        for id in &gallery {
            s.files.push(RemoteMedia {
                id: id.clone(),
                filename: Some(format!("{id}.jpg")),
                status: RemoteMediaStatus::Ready,
                position: 0,
            });
        }
    }
    for id in &gallery {
        a.fake_shop()
            .fail_image(&fake_image_url(id), vec![ImageFault::Gone]);
    }
    let (session, item) = archived_record(&a, "G-30", &gallery, &[], None);
    view(&a, session);
    a.remote().run_pending().unwrap();
    let chunks: Vec<usize> = a
        .fake_shop()
        .url_reads()
        .iter()
        .map(|chunk| chunk.len())
        .collect();
    assert_eq!(chunks, vec![25, 5]);
    for (i, id) in gallery.iter().enumerate() {
        let file = expected_file(&a, item, "gallery", i as i64 + 1, id);
        assert!(file.is_file(), "{file:?}");
    }
}

#[test]
fn h4_c1h_viewing_and_downloading_change_nothing_in_the_shop() {
    let a = shop_app();
    let items = viewed_and_run(&a);
    for (session, _) in items {
        view(&a, session);
    }
    let r = adopted(&a, "SHOP-H", &["h-m1", "h-m2"], &[]);
    view(&a, r.session);
    a.remote().run_pending().unwrap();
    view(&a, r.session);
    let gone = ids("h-g", 2);
    {
        let mut s = a.fake_shop().state.lock().unwrap();
        for id in &gone {
            s.files.push(RemoteMedia {
                id: id.clone(),
                filename: Some(format!("{id}.jpg")),
                status: RemoteMediaStatus::Ready,
                position: 0,
            });
        }
    }
    for id in &gone {
        a.fake_shop()
            .fail_image(&fake_image_url(id), vec![ImageFault::Gone]);
    }
    let (session, _) = archived_record(&a, "H-GONE", &gone, &[], None);
    view(&a, session);
    a.remote().run_pending().unwrap();
    assert_eq!(a.fake_shop().mutation_ops(), Vec::<String>::new());
}

#[test]
fn h4_c1i_a_second_failed_read_in_a_row_announces_nothing() {
    let a = shop_app();
    let r = adopted(&a, "SHOP-I", &["i-m1", "i-m2"], &[]);
    a.fake_shop().state.lock().unwrap().unreachable = true;
    view(&a, r.session);
    a.remote().run_pending().unwrap();
    assert_eq!(session_changes(&a, r.session), 1);
    view(&a, r.session);
    a.remote().run_pending().unwrap();
    assert_eq!(session_changes(&a, r.session), 1);
    let v = view(&a, r.session);
    assert_eq!((v.state, v.check_failed), (RemoteState::Unknown, true));
}

// ---------------------------------------------------------------------------------------------
// H4·C2 — activation never moves the capture folder for a remote item

#[test]
fn h4_c2a_opening_an_archived_item_leaves_the_active_session() {
    let a = shop_app();
    let (b, _) = a
        .items()
        .create_local("B-LOCAL", "Chair", "Chair", None)
        .map(|(i, s)| (s.session.id, i.id))
        .unwrap();
    a.sessions().activate(b).unwrap();
    let (archived, _) = three_and_two(&a);
    let before = ops(&a).len();
    with_remote(&a).activate(archived).unwrap();
    assert_eq!(ops(&a)[before..].to_vec(), Vec::<String>::new());
    assert_eq!(a.store().active_session_id().unwrap(), Some(b));
}

#[test]
fn h4_c2b_opening_a_shop_only_item_with_no_session_active() {
    let a = shop_app();
    let (s, _) = shop_only(&a, "SO-B");
    assert_eq!(a.store().active_session_id().unwrap(), None);
    let before = ops(&a).len();
    with_remote(&a).activate(s).unwrap();
    assert_eq!(ops(&a)[before..].to_vec(), Vec::<String>::new());
    assert_eq!(a.store().active_session_id().unwrap(), None);
}

/// The capture-folder operations `activate(session)` sent, as (`set_captures` folder,
/// `set_current_collection` folder) when both end in `/<folder key>`.
fn activated(a: &TestApp, session: i64) -> (bool, bool) {
    let item = a.store().session(session).unwrap().session.item_id.unwrap();
    let key = a.store().item(item).unwrap().folder_key;
    let before = ops(a).len();
    with_remote(a).activate(session).unwrap();
    let after = ops(a)[before..].to_vec();
    let suffix = format!("/{key}");
    (
        after
            .iter()
            .any(|op| op.starts_with("set_captures:") && op.ends_with(&suffix)),
        after
            .iter()
            .any(|op| op.starts_with("set_current_collection:") && op.ends_with(&suffix)),
    )
}

#[test]
fn h4_c2d_local_and_unknown_items_activate_as_today() {
    let a = shop_app();
    let (_, session) = a
        .items()
        .create_local("L-ACT", "Desk", "Desk", None)
        .unwrap();
    let local = session.session.id;
    assert_eq!(activated(&a, local), (true, true));
    assert_eq!(a.store().active_session_id().unwrap(), Some(local));
    let unknown = adopted(&a, "U-ACT", &["u-m1"], &[]).session;
    assert_eq!(a.remote().state(unknown).unwrap(), RemoteState::Unknown);
    assert_eq!(activated(&a, unknown), (true, true));
    assert_eq!(a.store().active_session_id().unwrap(), Some(unknown));
}

#[test]
fn h4_c2e_after_bring_back_the_capture_folder_follows_it() {
    let a = shop_app();
    let r = archived_of(&a, "BB-ACT", "Sideboard", 3, 2);
    a.remote().bring_back(r.session).unwrap();
    assert_eq!(activated(&a, r.session), (true, true));
    assert_eq!(a.store().active_session_id().unwrap(), Some(r.session));
}

#[test]
fn h4_c2f_a_capture_after_opening_an_archived_item_goes_where_photos_went() {
    let a = shop_app();
    let (_, b) = a
        .items()
        .create_local("B-ARR", "Chair", "Chair", None)
        .unwrap();
    let b = b.session.id;
    a.sessions().activate(b).unwrap();
    let (archived, _) = three_and_two(&a);
    with_remote(&a).activate(archived).unwrap();
    let raw = a.arrival_photo(b, "B_after_archived.ARW").unwrap();
    let row = a
        .store()
        .photos_in_session(b)
        .unwrap()
        .into_iter()
        .find(|p| p.raw_path == raw);
    assert!(row.is_some(), "{raw} is not in session B");
    assert!(
        a.store()
            .photos_in_session(archived)
            .unwrap()
            .iter()
            .all(|p| p.raw_path != raw)
    );
}

// ---------------------------------------------------------------------------------------------
// H4·C3 — what other readers are given for a remote session (M-H4)

#[test]
fn h4_c3a_the_session_view_of_an_archived_item_has_no_photos() {
    let a = shop_app();
    let r = archived_of(&a, "SV-A", "Sideboard", 3, 2);
    assert_eq!(a.store().photos_in_session(r.session).unwrap().len(), 5);
    assert_eq!(with_remote(&a).view(r.session).unwrap().photos, vec![]);
    // As the `session_get` command builds it (the commerce integration, no `with_remote`).
    let shop: &dyn psm_core::ports::commerce::CommerceAdapter = a.fake_shop();
    let as_command = SessionsService::new(a.store(), a.clock.as_ref())
        .with_commerce(Some(shop))
        .view(r.session)
        .unwrap();
    assert_eq!(as_command.photos, vec![]);
}

#[test]
fn h4_c3b_after_bring_back_the_view_holds_the_brought_back_photos() {
    let a = shop_app();
    let r = archived_of(&a, "SV-B", "Sideboard", 3, 2);
    let outcome = a.remote().bring_back(r.session).unwrap();
    let photos = with_remote(&a).view(r.session).unwrap().photos;
    assert_eq!(
        photos.iter().map(|p| p.id).collect::<Vec<_>>(),
        outcome.photo_ids
    );
    assert_eq!(photos.len(), 5);
}

/// A connected Pricing app (C3's `connected`).
fn connected() -> TestApp {
    let a = TestApp::with_pricing().unwrap();
    a.clock.advance(1_000_000);
    a.app.conversations.key_set(Some(PRICING_KEY)).unwrap();
    run_check(&a);
    a
}

fn run_check(a: &TestApp) {
    let mut n = 0;
    while a.app.conversations.tick(a.clock.now_ms()).unwrap().more {
        n += 1;
        assert!(n < 50, "the check never completed");
    }
}

fn link(a: &TestApp, photo_id: i64, item_id: i64, role: &str, external: &str) {
    a.store()
        .upsert_photo_link(
            &PhotoLink {
                photo_id,
                item_id,
                role: role.into(),
                slot: OutputSlot::Single,
                external_id: Some(external.into()),
                upload_state: "ready".into(),
                export_sha256: None,
                superseded_ids: vec![],
                revision: 0,
                retouch_gen: 0,
                composition_rev: 0,
            },
            a.clock.now_ms(),
        )
        .unwrap();
}

#[test]
fn h4_c3c_attachable_photos_of_an_archived_item_carry_no_dead_preview() {
    let a = connected();
    let sku = "BEYO-4401";
    let (item, session) = a.items().create_local(sku, sku, "object", None).unwrap();
    let session = session.session.id;
    let mut expected = Vec::new();
    for (name, class, role, media) in [
        ("g1", C::Main, "gallery", "media-g1"),
        (
            "c1",
            C::Condition,
            "attribute:custom.damage_report",
            "media-c1",
        ),
    ] {
        let p = a.add_photo(session, &format!("{name}.ARW")).unwrap();
        a.review().classify(p.id, class).unwrap();
        let preview = a.dir.path().join(format!("orig-{name}.jpg"));
        fs::write(&preview, b"jpeg").unwrap();
        a.store()
            .set_original_preview(&p.raw_path, Some(&preview.to_string_lossy()), None)
            .unwrap();
        link(&a, p.id, item.id, role, media);
        // Cleared: the preview file is gone.
        fs::remove_file(&preview).unwrap();
        assert!(!Path::new(&preview).exists());
        expected.push((media.to_string(), p.id, None::<String>));
    }
    a.store()
        .mark_item_cleared(
            item.id,
            a.clock.now_ms(),
            &[ArchivedImage {
                role: "gallery".into(),
                position: 1,
                media_id: "media-g1".into(),
                url: fake_image_url("media-g1"),
                bytes: None,
            }],
        )
        .unwrap();
    let store_id = a.app.store.sync().unwrap().store.unwrap().id;
    let case = a.fake_pricing().ask(&store_id, Some(sku), "Condition");
    a.clock.advance(10_000);
    run_check(&a);
    let got: Vec<(String, i64, Option<String>)> = a
        .app
        .conversations
        .attachable(&case)
        .unwrap()
        .photos
        .into_iter()
        .map(|p| (p.media_id, p.photo_id, p.preview_path))
        .collect();
    assert_eq!(got, expected);
}

#[test]
fn h4_c3d_a_remote_item_refuses_media_lists_and_saves_other_attributes() {
    let a = shop_app();
    let (_, shop_item) = shop_only(&a, "SO-D");
    let (_, archived_item) = three_and_two(&a);
    let media = a.attributes(Some("shopify")).set_value(
        shop_item,
        "custom.damage_report",
        json!(["gid://shopify/MediaImage/1"]),
        Provenance::Human,
    );
    assert!(matches!(media, Err(Error::FilesCleared)), "{media:?}");
    for item in [shop_item, archived_item] {
        let saved = a
            .attributes(Some("shopify"))
            .set_value(
                item,
                "product.description",
                json!("Solid oak, two drawers."),
                Provenance::Human,
            )
            .unwrap();
        assert_eq!(saved.key, "product.description");
    }
}
