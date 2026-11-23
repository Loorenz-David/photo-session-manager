//! H3·U2 — bring back (master plan archive §5 MC-H4.1–5, M-H1, M-H2, M-H5; book 10H §7).
//! `RemoteItems::bring_back` on `TestApp::with_fake_shop()` with the fake Capture One document.
//! Outcomes: the files in the session folder, stored rows, the session view, what `FakeCommerce`
//! and the fake Capture One recorded, emitted events. "Archived item (3 + 2)" = 3 main and 2
//! condition photos, published, then cleared by the real clearing; each of its shop pictures is
//! registered with `set_image` as a distinct `SyntheticJpeg`. Rows H3·C3 are unit tests of
//! `mechanisms::remote`.

use crate::p62_retention_after_publish::{clear_all, shop_app};
use psm_core::{
    Error,
    domain::{
        composition::{CompositionKind, Flip, OutputSlot, Rect, Shape},
        photo::{Classification as C, Integrity, NewPhoto, Photo, PhotoSource},
        publish::{RemoteMedia, RemoteMediaStatus},
        remote::{BringBackState, CallBudget, RemoteKind, RemoteState},
        session::{ListFilter, PublishStep, RowLabel},
    },
    events::AppEvent,
    mechanisms::hash::sha256_file,
    ports::clock::Clock,
    services::{items::ItemsService, templates::TemplatesService},
};
use psm_infra::imaging::ImageOpsImpl;
use psm_testkit::{
    ConfirmAsShown, TestApp,
    fake_commerce::{ImageFault, fake_refreshed_url},
    fixtures::{synthetic_jpeg::SyntheticJpeg, synthetic_raw::SyntheticRaw},
};
use serde_json::json;
use std::{
    collections::{BTreeSet, HashMap},
    fs,
    path::{Path, PathBuf},
};

const DAY: i64 = 86_400_000;
const CONDITION_ROLE: &str = "attribute:custom.damage_report";

// ---------------------------------------------------------------------------------------------
// fixtures

fn last_issued(a: &TestApp) -> String {
    a.fake
        .state
        .lock()
        .unwrap()
        .issued
        .last()
        .unwrap()
        .0
        .clone()
}

/// A RAW in `<capture root>/<folder>/`, known to the fake document, with a rendered first
/// preview, classified (the `p62` shape, without its hidden photos).
fn raw_photo(a: &TestApp, session: i64, folder: &str, name: &str, class: C) -> Photo {
    let dir = a.capture_root().join(folder);
    fs::create_dir_all(&dir).unwrap();
    let raw = dir.join(format!("{name}.ARW"));
    fs::write(&raw, format!("raw bytes of {name}")).unwrap();
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
    a.fake
        .add_image(&photo.raw_path, &[&format!("v{}", photo.id)], false);
    a.store().upsert_raw_state(&photo.raw_path, None).unwrap();
    a.app.previews.request_initial(&photo.raw_path).unwrap();
    a.complete_render_with(
        &last_issued(a),
        &SyntheticJpeg::rgb(600, 400, [200, 100, 50]),
    )
    .unwrap();
    a.review().classify(photo.id, class).unwrap();
    a.store().photo(photo.id).unwrap()
}

fn drain(a: &TestApp) {
    for _ in 0..12 {
        a.tick(1100).unwrap();
        a.complete_all_renders().unwrap();
    }
}

/// A remote item ready to be brought back.
struct Remote {
    session: i64,
    item: i64,
    folder: PathBuf,
    /// Gallery then condition media ids, in record order.
    media: Vec<String>,
    /// The bytes each media serves at full size.
    bytes: HashMap<String, Vec<u8>>,
}

impl Remote {
    fn url(&self, a: &TestApp, media: &str) -> String {
        a.store()
            .archived_images(self.item)
            .unwrap()
            .into_iter()
            .find(|i| i.media_id == media)
            .unwrap()
            .url
    }
}

/// Registers each record picture of `item` with a distinct real JPEG (`640 + 16·i` × 480).
fn register_pictures(a: &TestApp, item: i64) -> (Vec<String>, HashMap<String, Vec<u8>>) {
    let mut media = Vec::new();
    let mut bytes = HashMap::new();
    for (i, image) in a.store().archived_images(item).unwrap().iter().enumerate() {
        let jpeg = SyntheticJpeg::rgb(
            640 + 16 * i as u32,
            480,
            [30 * i as u8, 200 - 20 * i as u8, 90],
        );
        a.fake_shop().set_image(&image.media_id, jpeg.clone());
        media.push(image.media_id.clone());
        bytes.insert(image.media_id.clone(), jpeg);
    }
    (media, bytes)
}

/// An item of `kind` with `main` and `condition` photos, published to the fake shop, then cleared
/// by the real clearing; its pictures registered.
fn archived_of(a: &TestApp, sku: &str, kind: &str, main: usize, condition: usize) -> Remote {
    let (_, created) = a
        .items()
        .create_local(sku, &format!("{sku} title"), kind, None)
        .unwrap();
    let session = created.session.id;
    let item = a.store().session(session).unwrap().session.item_id.unwrap();
    let folder_key = a.store().item(item).unwrap().folder_key;
    for i in 0..main {
        raw_photo(a, session, &folder_key, &format!("{sku}_M{i}"), C::Main);
    }
    for i in 0..condition {
        raw_photo(
            a,
            session,
            &folder_key,
            &format!("{sku}_C{i}"),
            C::Condition,
        );
    }
    a.app
        .publish
        .as_ref()
        .unwrap()
        .confirm_as_shown(&[session], "Publish images")
        .unwrap();
    drain(a);
    assert_eq!(
        a.store().latest_publish(session).unwrap().map(|l| l.step),
        Some(PublishStep::Done)
    );
    a.clock.advance(4 * DAY + 60_000);
    clear_all(a);
    assert!(
        a.store().item_retention(item).unwrap().cleared_at.is_some(),
        "{sku} cleared"
    );
    let (media, bytes) = register_pictures(a, item);
    Remote {
        session,
        item,
        folder: a.capture_root().join(folder_key),
        media,
        bytes,
    }
}

fn archived(a: &TestApp, sku: &str) -> Remote {
    archived_of(a, sku, "Sideboard", 3, 2)
}

/// A shop product adopted from the search with its session; its pictures `gallery` and
/// `condition` registered (`640 + 16·i` × 480).
fn adopted(a: &TestApp, sku: &str, gallery: &[&str], condition: &[&str]) -> Remote {
    let product = a
        .fake_shop()
        .add_product(sku, &format!("{sku} title"), None);
    {
        let mut s = a.fake_shop().state.lock().unwrap();
        let p = s
            .products
            .iter_mut()
            .find(|p| p.external_id == product)
            .unwrap();
        p.media = gallery
            .iter()
            .enumerate()
            .map(|(i, id)| RemoteMedia {
                id: (*id).into(),
                filename: Some(format!("{id}.jpg")),
                status: RemoteMediaStatus::Ready,
                position: i as i64 + 1,
            })
            .collect();
        for id in condition {
            s.files.push(RemoteMedia {
                id: (*id).into(),
                filename: Some(format!("{id}.jpg")),
                status: RemoteMediaStatus::Ready,
                position: 0,
            });
        }
        let key = a.fake_shop().condition_key.clone();
        s.values
            .entry(product.clone())
            .or_default()
            .push((key, json!(condition)));
    }
    let item = ItemsService::new(a.store(), a.clock.as_ref())
        .with_commerce(Some(a.fake_shop()))
        .adopt(&product)
        .unwrap();
    let session = a.sessions().create(item.id).unwrap().session.id;
    let mut bytes = HashMap::new();
    let mut media = Vec::new();
    for (i, id) in gallery.iter().chain(condition).enumerate() {
        let jpeg = SyntheticJpeg::rgb(640 + 16 * i as u32, 480, [40, 40 * i as u8, 120]);
        a.fake_shop().set_image(id, jpeg.clone());
        bytes.insert(id.to_string(), jpeg);
        media.push(id.to_string());
    }
    let folder_key = a.store().item(item.id).unwrap().folder_key;
    Remote {
        session,
        item: item.id,
        folder: a.capture_root().join(folder_key),
        media,
        bytes,
    }
}

// ---------------------------------------------------------------------------------------------
// observations

/// The visible photos of the session view: (classification, position, file name, raw hash).
fn view(a: &TestApp, session: i64) -> Vec<Photo> {
    a.sessions().view(session).unwrap().photos
}

fn file_names(dir: &Path) -> BTreeSet<String> {
    match fs::read_dir(dir) {
        Ok(entries) => entries
            .flatten()
            .filter(|e| e.path().is_file())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect(),
        Err(_) => BTreeSet::new(),
    }
}

fn names(list: &[&str]) -> BTreeSet<String> {
    list.iter().map(|s| s.to_string()).collect()
}

const FIVE: [&str; 5] = [
    "BEYO-12-main-01.jpg",
    "BEYO-12-main-02.jpg",
    "BEYO-12-main-03.jpg",
    "BEYO-12-condition-01.jpg",
    "BEYO-12-condition-02.jpg",
];

fn ops(a: &TestApp) -> Vec<String> {
    a.fake.state.lock().unwrap().ops.clone()
}

fn events(a: &TestApp) -> Vec<AppEvent> {
    a.events.0.lock().unwrap().clone()
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

fn fetches_of(a: &TestApp, url: &str) -> usize {
    a.fake_shop()
        .original_fetches()
        .iter()
        .filter(|u| u.as_str() == url)
        .count()
}

fn links_for(a: &TestApp, photo: i64, item: i64) -> i64 {
    rusqlite::Connection::open(a.app.paths.db())
        .unwrap()
        .query_row(
            "SELECT COUNT(*) FROM photo_links WHERE photo_id=?1 AND item_id=?2",
            [photo, item],
            |r| r.get(0),
        )
        .unwrap()
}

fn photo_rows(a: &TestApp, session: i64) -> Vec<Photo> {
    a.store().photos_in_session(session).unwrap()
}

fn failed_reason(result: psm_core::Result<psm_core::domain::remote::BringBackOutcome>) -> String {
    match result {
        Err(error @ Error::BringBackFailed { .. }) => error.to_string(),
        other => panic!("not BringBackFailed: {other:?}"),
    }
}

/// The state of a failed bring back that must equal the state before it (M-H2).
fn unchanged(a: &TestApp, r: &Remote) -> (Vec<Photo>, Vec<Photo>, Option<i64>) {
    (
        view(a, r.session),
        photo_rows(a, r.session),
        a.store().item_retention(r.item).unwrap().cleared_at,
    )
}

fn no_bring_back_files(r: &Remote) {
    let present = file_names(&r.folder);
    for name in FIVE {
        assert!(!present.contains(name), "{name} left behind");
    }
    assert!(!present.iter().any(|n| n.ends_with(".part")), "{present:?}");
}

// ---------------------------------------------------------------------------------------------
// H3·C4 — an archived item comes back

#[test]
fn h3_c4a_the_five_pictures_are_written_into_the_session_folder() {
    let a = shop_app();
    let r = archived(&a, "BEYO-12");
    let before = file_names(&r.folder);
    a.remote().bring_back(r.session).unwrap();
    let mut expected = before.clone();
    expected.extend(names(&FIVE));
    assert_eq!(file_names(&r.folder), expected);
    for (name, media) in FIVE.iter().zip(&r.media) {
        assert_eq!(&fs::read(r.folder.join(name)).unwrap(), &r.bytes[media]);
    }
}

#[test]
fn h3_c4b_full_size_downloads_only() {
    let a = shop_app();
    let r = archived(&a, "BEYO-12");
    let urls: BTreeSet<String> = r.media.iter().map(|m| r.url(&a, m)).collect();
    let sized = a.fake_shop().image_fetches();
    a.remote().bring_back(r.session).unwrap();
    let fetched: Vec<String> = a.fake_shop().original_fetches();
    assert_eq!(fetched.len(), 5);
    assert_eq!(fetched.into_iter().collect::<BTreeSet<_>>(), urls);
    assert_eq!(a.fake_shop().image_fetches(), sized);
}

#[test]
fn h3_c4c_the_session_view_holds_the_five_photos_in_published_order() {
    let a = shop_app();
    let r = archived(&a, "BEYO-12");
    a.remote().bring_back(r.session).unwrap();
    let photos = view(&a, r.session);
    let got: Vec<(C, Option<i64>, String)> = photos
        .iter()
        .map(|p| {
            (
                p.classification,
                p.position,
                Path::new(&p.raw_path)
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned(),
            )
        })
        .collect();
    assert_eq!(
        got,
        vec![
            (C::Main, Some(1), FIVE[0].to_string()),
            (C::Main, Some(2), FIVE[1].to_string()),
            (C::Main, Some(3), FIVE[2].to_string()),
            (C::Condition, Some(1), FIVE[3].to_string()),
            (C::Condition, Some(2), FIVE[4].to_string()),
        ]
    );
    for p in &photos {
        assert_eq!(
            p.raw_hash.as_deref(),
            Some(sha256_file(Path::new(&p.raw_path)).unwrap().as_str())
        );
    }
}

#[test]
fn h3_c4d_the_earlier_rows_are_retired_not_deleted() {
    let a = shop_app();
    let r = archived(&a, "BEYO-12");
    let earlier: Vec<i64> = photo_rows(&a, r.session).iter().map(|p| p.id).collect();
    assert_eq!(earlier.len(), 5);
    a.remote().bring_back(r.session).unwrap();
    let visible: Vec<i64> = view(&a, r.session).iter().map(|p| p.id).collect();
    for id in earlier {
        let p = a.store().photo(id).unwrap();
        assert_eq!(p.classification, C::Deleted);
        assert_eq!(p.position, None);
        assert_eq!(
            (
                p.original_preview_path,
                p.adjusted_preview_path,
                p.thumbnail_path
            ),
            (None, None, None)
        );
        assert_eq!(p.integrity, Integrity::RawMissing);
        assert_eq!(links_for(&a, id, r.item), 0);
        assert!(!visible.contains(&id));
    }
}

#[test]
fn h3_c4e_retention_restored_and_still_published() {
    let a = shop_app();
    let r = archived(&a, "BEYO-12");
    let seq = a.store().session(r.session).unwrap().session.change_seq;
    a.remote().bring_back(r.session).unwrap();
    let retention = a.store().item_retention(r.item).unwrap();
    assert_eq!(retention.cleared_at, None);
    assert_eq!(retention.restored_at, Some(a.clock.now_ms()));
    assert_eq!(a.remote().state(r.session).unwrap(), RemoteState::Local);
    assert_eq!(
        a.store().session(r.session).unwrap().session.change_seq,
        seq
    );
    let list = a.sessions().list(ListFilter::All, None).unwrap();
    let row = list
        .groups
        .iter()
        .flat_map(|g| g.rows.iter())
        .find(|row| row.id == r.session)
        .unwrap();
    assert_eq!(row.label, RowLabel::Published);
}

#[test]
fn h3_c4f_each_photo_starts_exactly_as_published() {
    let a = shop_app();
    let r = archived(&a, "BEYO-12");
    let outcome = a.remote().bring_back(r.session).unwrap();
    assert_eq!(outcome.photo_ids.len(), 5);
    for id in outcome.photo_ids {
        let c = a.composition(id).unwrap();
        assert_eq!(c.kind, CompositionKind::Normal);
        assert_eq!(c.shape, Shape::Original);
        assert_eq!(c.quarter_turns, 0);
        assert_eq!(c.flip, Flip::None);
        assert_eq!(c.straighten, 0.0);
        assert_eq!(
            c.rect,
            Some(Rect {
                x: 0.0,
                y: 0.0,
                w: 1.0,
                h: 1.0
            })
        );
        assert_eq!(c.revision, 0);
    }
}

#[test]
fn h3_c4g_each_photo_is_linked_to_its_media_as_uploaded() {
    let a = shop_app();
    let r = archived(&a, "BEYO-12");
    let outcome = a.remote().bring_back(r.session).unwrap();
    for (i, (id, media)) in outcome.photo_ids.iter().zip(&r.media).enumerate() {
        let role = if i < 3 { "gallery" } else { CONDITION_ROLE };
        assert_eq!(links_for(&a, *id, r.item), 1);
        let link = a
            .store()
            .photo_link(*id, r.item, role, OutputSlot::Single)
            .unwrap()
            .expect("link");
        assert_eq!(link.upload_state, "ready");
        assert_eq!(link.external_id.as_deref(), Some(media.as_str()));
        assert_eq!(link.revision, 0);
        assert!(link.superseded_ids.is_empty());
    }
}

#[test]
fn h3_c4h_capture_one_lists_the_folder_and_applies_no_style() {
    let a = shop_app();
    let r = archived(&a, "BEYO-12");
    let mark = ops(&a).len();
    a.remote().bring_back(r.session).unwrap();
    let during = ops(&a)[mark..].to_vec();
    assert!(
        during.contains(&format!("sync_folder:{}", r.folder.display())),
        "{during:?}"
    );
    a.tick(0).unwrap();
    let after = ops(&a)[mark..].to_vec();
    assert!(
        !after.iter().any(|op| op.starts_with("apply_style:")),
        "{after:?}"
    );
}

#[test]
fn h3_c4i_progress_events_then_done() {
    let a = shop_app();
    let r = archived(&a, "BEYO-12");
    let mark = events(&a).len();
    a.remote().bring_back(r.session).unwrap();
    let after = events(&a)[mark..].to_vec();
    let progress: Vec<(usize, usize, BringBackState)> = after
        .iter()
        .filter_map(|e| match e {
            AppEvent::BringBack {
                session_id,
                done,
                total,
                state,
                ..
            } if *session_id == r.session => Some((*done, *total, *state)),
            _ => None,
        })
        .collect();
    assert_eq!(
        progress,
        vec![
            (1, 5, BringBackState::Running),
            (2, 5, BringBackState::Running),
            (3, 5, BringBackState::Running),
            (4, 5, BringBackState::Running),
            (5, 5, BringBackState::Running),
            (5, 5, BringBackState::Done),
        ]
    );
    assert!(
        after.iter().any(
            |e| matches!(e, AppEvent::SessionChanged { session_id } if *session_id == r.session)
        )
    );
}

#[test]
fn h3_c4j_the_undo_list_is_emptied() {
    let a = shop_app();
    let r = archived(&a, "BEYO-12");
    assert!(
        a.history().state(r.session).undo.is_some(),
        "an undo step before the bring back"
    );
    a.remote().bring_back(r.session).unwrap();
    let state = a.history().state(r.session);
    assert_eq!((state.undo, state.redo), (None, None));
}

// ---------------------------------------------------------------------------------------------
// H3·C5 — a shop-only item comes back

#[test]
fn h3_c5a_a_shop_only_item_with_a_record() {
    let a = shop_app();
    let r = adopted(&a, "SHOP-5", &["m1", "m2"], &["c1"]);
    assert_eq!(
        a.remote().ensure_record(r.session).unwrap(),
        RemoteState::Remote(RemoteKind::ShopOnly)
    );
    let outcome = a.remote().bring_back(r.session).unwrap();
    let photos = view(&a, r.session);
    let key = r.folder.file_name().unwrap().to_string_lossy().into_owned();
    let got: Vec<(C, Option<i64>, String)> = photos
        .iter()
        .map(|p| {
            (
                p.classification,
                p.position,
                Path::new(&p.raw_path)
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned(),
            )
        })
        .collect();
    assert_eq!(
        got,
        vec![
            (C::Main, Some(1), format!("{key}-main-01.jpg")),
            (C::Main, Some(2), format!("{key}-main-02.jpg")),
            (C::Condition, Some(1), format!("{key}-condition-01.jpg")),
        ]
    );
    let roles = ["gallery", "gallery", CONDITION_ROLE];
    for ((id, media), role) in outcome.photo_ids.iter().zip(["m1", "m2", "c1"]).zip(roles) {
        let link = a
            .store()
            .photo_link(*id, r.item, role, OutputSlot::Single)
            .unwrap()
            .unwrap();
        assert_eq!(
            (link.upload_state.as_str(), link.external_id.as_deref()),
            ("ready", Some(media))
        );
    }
    assert!(
        a.store()
            .item_retention(r.item)
            .unwrap()
            .restored_at
            .is_some()
    );
    assert_eq!(a.remote().state(r.session).unwrap(), RemoteState::Local);
}

#[test]
fn h3_c5b_previews_and_frames_after_the_jobs() {
    let a = shop_app();
    let r = adopted(&a, "SHOP-5", &["m1", "m2"], &["c1"]);
    a.remote().ensure_record(r.session).unwrap();
    let outcome = a.remote().bring_back(r.session).unwrap();
    a.tick(0).unwrap();
    for (i, id) in outcome.photo_ids.iter().enumerate() {
        let photo = a.store().photo(*id).unwrap();
        let preview = photo.original_preview_path.expect("original preview");
        assert!(Path::new(&preview).is_file());
        let frame = a.composition(*id).unwrap().frame().expect("frame");
        assert_eq!((frame.w, frame.h), (640 + 16 * i as i64, 480));
    }
}

#[test]
fn h3_c5c_an_unknown_item_is_read_once_then_brought_back() {
    let a = shop_app();
    let r = adopted(&a, "SHOP-6", &["m1", "m2"], &[]);
    assert_eq!(a.remote().state(r.session).unwrap(), RemoteState::Unknown);
    let outcome = a.remote().bring_back(r.session).unwrap();
    assert_eq!(outcome.photo_ids.len(), 2);
    assert_eq!(reads(&a, "media_readback"), 1);
}

// ---------------------------------------------------------------------------------------------
// H3·C6 — all or nothing

#[test]
fn h3_c6a_a_picture_failing_three_times_imports_nothing() {
    let a = shop_app();
    let r = archived(&a, "BEYO-12");
    let m2 = r.url(&a, &r.media[1]);
    a.fake_shop().fail_image(&m2, vec![ImageFault::Fail; 3]);
    let before = unchanged(&a, &r);
    let result = a.remote().bring_back(r.session);
    assert_eq!(
        failed_reason(result),
        "Could not bring back the photos: 1 photo could not be downloaded"
    );
    assert_eq!(fetches_of(&a, &m2), 3);
    assert_eq!(unchanged(&a, &r), before);
    assert!(before.2.is_some());
    no_bring_back_files(&r);
    let last = events(&a)
        .into_iter()
        .rev()
        .find(|e| matches!(e, AppEvent::BringBack { .. }))
        .unwrap();
    assert!(matches!(
        last,
        AppEvent::BringBack {
            state: BringBackState::Failed,
            ..
        }
    ));
}

#[test]
fn h3_c6b_two_failures_then_success_is_retried_through() {
    let a = shop_app();
    let r = archived(&a, "BEYO-12");
    let m2 = r.url(&a, &r.media[1]);
    a.fake_shop().fail_image(&m2, vec![ImageFault::Fail; 2]);
    let outcome = a.remote().bring_back(r.session).unwrap();
    assert_eq!(outcome.photo_ids.len(), 5);
    assert_eq!(view(&a, r.session).len(), 5);
}

#[test]
fn h3_c6c_a_gone_address_is_refreshed_once_and_downloaded_at_the_new_one() {
    let a = shop_app();
    let r = archived(&a, "BEYO-12");
    let m2 = &r.media[1];
    a.fake_shop()
        .fail_image(&r.url(&a, m2), vec![ImageFault::Gone]);
    a.fake_shop().set_budget(Some(CallBudget {
        available: 100.0,
        maximum: 1000.0,
    }));
    let outcome = a.remote().bring_back(r.session).unwrap();
    assert_eq!(a.fake_shop().url_reads().len(), 1);
    assert_eq!(r.url(&a, m2), fake_refreshed_url(m2, 1));
    assert_eq!(outcome.photo_ids.len(), 5);
    assert_eq!(view(&a, r.session).len(), 5);
}

#[test]
fn h3_c6d_a_gone_picture_in_the_record_refuses_before_any_download() {
    let a = shop_app();
    let r = archived(&a, "BEYO-12");
    a.store()
        .update_remote_urls(r.item, &[(r.media[1].clone(), None, None)])
        .unwrap();
    let before = unchanged(&a, &r);
    let result = a.remote().bring_back(r.session);
    assert_eq!(
        failed_reason(result),
        "Could not bring back the photos: 1 photo is no longer in the shop"
    );
    assert!(a.fake_shop().original_fetches().is_empty());
    assert_eq!(unchanged(&a, &r), before);
    no_bring_back_files(&r);
}

#[test]
fn h3_c6e_a_kill_during_the_downloads_leaves_the_item_remote_and_resumes() {
    let mut a = shop_app();
    let r = archived(&a, "BEYO-12");
    let before = view(&a, r.session);
    a.fake_shop().fail_after_effect("fetch_original");
    let killed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        a.remote().bring_back(r.session)
    }));
    assert!(killed.is_err(), "the simulated kill");
    a.kill_and_restart().unwrap();
    assert_eq!(
        a.remote().state(r.session).unwrap(),
        RemoteState::Remote(RemoteKind::Archived)
    );
    assert_eq!(view(&a, r.session), before);
    a.remote().bring_back(r.session).unwrap();
    let present = file_names(&r.folder);
    for (name, media) in FIVE.iter().zip(&r.media) {
        assert!(present.contains(*name), "{name}");
        assert_eq!(&fs::read(r.folder.join(name)).unwrap(), &r.bytes[media]);
    }
    let jpgs = present.iter().filter(|n| n.starts_with("BEYO-12-")).count();
    assert_eq!(jpgs, 5, "{present:?}");
}

#[test]
fn h3_c6f_a_picture_that_is_not_a_jpeg_counts_as_not_downloaded() {
    let a = shop_app();
    let r = archived(&a, "BEYO-12");
    let mut png = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    png.extend_from_slice(b"not a jpeg body");
    a.fake_shop().set_image(&r.media[1], png);
    let m2 = r.url(&a, &r.media[1]);
    let before = unchanged(&a, &r);
    let result = a.remote().bring_back(r.session);
    assert_eq!(
        failed_reason(result),
        "Could not bring back the photos: 1 photo could not be downloaded"
    );
    assert_eq!(fetches_of(&a, &m2), 3);
    assert_eq!(unchanged(&a, &r), before);
    assert!(before.2.is_some());
    no_bring_back_files(&r);
}

// ---------------------------------------------------------------------------------------------
// H3·C7 — refusals

#[test]
fn h3_c7a_a_local_item_is_not_remote() {
    let a = shop_app();
    let (_, created) = a
        .items()
        .create_local("LOC-1", "Local", "Chair", None)
        .unwrap();
    let result = a.remote().bring_back(created.session.id);
    assert!(matches!(result, Err(Error::NotRemote)), "{result:?}");
    assert!(a.fake_shop().original_fetches().is_empty());
}

#[test]
fn h3_c7b_without_an_integration() {
    let a = TestApp::with_fake_document().unwrap();
    let (_, created) = a
        .items()
        .create_local("LOC-2", "Local", "Chair", None)
        .unwrap();
    let result = a.remote().bring_back(created.session.id);
    assert!(matches!(result, Err(Error::NoIntegration)), "{result:?}");
}

#[test]
fn h3_c7c_a_second_call_while_one_runs_is_refused() {
    let a = shop_app();
    let r = archived(&a, "BEYO-12");
    a.fake_shop().set_fetch_delay(150);
    let remote = a.remote();
    let (first, second) = std::thread::scope(|scope| {
        let first = scope.spawn(|| remote.bring_back(r.session));
        let started = std::time::Instant::now();
        while a.fake_shop().original_fetches().is_empty() {
            assert!(started.elapsed().as_secs() < 10, "first call never fetched");
            std::thread::yield_now();
        }
        let second = remote.bring_back(r.session);
        (first.join().unwrap(), second)
    });
    assert_eq!(first.unwrap().photo_ids.len(), 5);
    assert!(matches!(second, Err(Error::BringBackRunning)), "{second:?}");
    assert_eq!(view(&a, r.session).len(), 5);
}

#[test]
fn h3_c7d_a_publish_in_flight_refuses() {
    let a = shop_app();
    let r = archived(&a, "BEYO-12");
    rusqlite::Connection::open(a.app.paths.db())
        .unwrap()
        .execute(
            "UPDATE publish_items SET step='update' WHERE id=(SELECT MAX(id) FROM publish_items WHERE session_id=?1)",
            [r.session],
        )
        .unwrap();
    let reason = failed_reason(a.remote().bring_back(r.session));
    assert!(
        reason.contains("wait until the publish has finished"),
        "{reason}"
    );
    assert!(a.fake_shop().original_fetches().is_empty());
}

#[test]
fn h3_c7e_an_unknown_capture_folder_refuses() {
    let a = shop_app();
    let r = archived(&a, "BEYO-12");
    a.fake.state.lock().unwrap().open_documents.clear();
    let reason = failed_reason(a.remote().bring_back(r.session));
    assert!(
        reason.contains("the capture folder is not known yet"),
        "{reason}"
    );
    assert!(a.fake_shop().original_fetches().is_empty());
}

// ---------------------------------------------------------------------------------------------
// H3·C8 — what bring back never does

#[test]
fn h3_c8a_bring_back_never_writes_to_the_shop() {
    // C4(a)
    let a = shop_app();
    let r = archived(&a, "BEYO-12");
    let mark = a.fake_shop().mutation_ops();
    a.remote().bring_back(r.session).unwrap();
    assert_eq!(a.fake_shop().mutation_ops(), mark);
    // C5(a)
    let a = shop_app();
    let r = adopted(&a, "SHOP-5", &["m1", "m2"], &["c1"]);
    a.remote().ensure_record(r.session).unwrap();
    a.remote().bring_back(r.session).unwrap();
    assert!(a.fake_shop().mutation_ops().is_empty());
    // C6(a)
    let a = shop_app();
    let r = archived(&a, "BEYO-12");
    a.fake_shop()
        .fail_image(&r.url(&a, &r.media[1]), vec![ImageFault::Fail; 3]);
    let mark = a.fake_shop().mutation_ops();
    assert!(a.remote().bring_back(r.session).is_err());
    assert_eq!(a.fake_shop().mutation_ops(), mark);
    // C6(c)
    let a = shop_app();
    let r = archived(&a, "BEYO-12");
    a.fake_shop()
        .fail_image(&r.url(&a, &r.media[1]), vec![ImageFault::Gone]);
    let mark = a.fake_shop().mutation_ops();
    a.remote().bring_back(r.session).unwrap();
    assert_eq!(a.fake_shop().mutation_ops(), mark);
}

#[test]
fn h3_c8b_capture_one_is_asked_only_to_read_and_list() {
    let a = shop_app();
    let r = archived(&a, "BEYO-12");
    let mark = ops(&a).len();
    a.remote().bring_back(r.session).unwrap();
    assert_eq!(
        ops(&a)[mark..].to_vec(),
        vec![
            "document_info_started".to_string(),
            "document_info".to_string(),
            format!("sync_folder:{}", r.folder.display()),
        ]
    );
}

#[test]
fn h3_c8c_delete_session_trashes_the_jpegs_and_asks_the_shop_nothing() {
    let a = shop_app();
    let r = adopted(&a, "SHOP-5", &["m1", "m2"], &["c1"]);
    a.remote().ensure_record(r.session).unwrap();
    let outcome = a.remote().bring_back(r.session).unwrap();
    let files: Vec<PathBuf> = outcome
        .photo_ids
        .iter()
        .map(|id| PathBuf::from(a.store().photo(*id).unwrap().raw_path))
        .collect();
    let reads_before = a.fake_shop().state.lock().unwrap().reads.len();
    let mutations_before = a.fake_shop().mutation_ops().len();
    psm_integrations::session_delete::delete(&a.app.handles(), r.session).unwrap();
    let moved = a.trash.moved_paths();
    for file in &files {
        assert!(moved.contains(file), "{file:?} not in {moved:?}");
        assert!(!file.exists());
    }
    assert_eq!(
        a.fake_shop().state.lock().unwrap().reads.len(),
        reads_before
    );
    assert_eq!(a.fake_shop().mutation_ops().len(), mutations_before);
}

#[test]
fn h3_c8d_health_reports_no_missing_file_for_the_retired_rows() {
    let a = shop_app();
    let r = archived(&a, "BEYO-12");
    a.remote().bring_back(r.session).unwrap();
    a.focus().unwrap();
    let missing: Vec<_> = a
        .app
        .health
        .report("", false, false)
        .items
        .into_iter()
        .filter(|i| i.code == "file_missing" && !i.ok)
        .collect();
    assert!(missing.is_empty(), "{missing:?}");
}

// ---------------------------------------------------------------------------------------------
// H3·C9 — a templated session

/// A shooting template for item type `Lamp` with 3 containers whose shooting order equals their
/// final order.
fn lamp_template(a: &TestApp) {
    let templates = TemplatesService::new(
        a.store(),
        a.clock.as_ref(),
        &ImageOpsImpl,
        a.app.paths.root.join("templates"),
    );
    let t = templates.create("Lamp").unwrap();
    for _ in 0..3 {
        templates.container_add(t.id, None).unwrap();
    }
}

fn templated(a: &TestApp, sku: &str, main: usize) -> (Remote, Vec<i64>, Vec<i64>) {
    let r = archived_of(a, sku, "Lamp", main, 0);
    let retired: Vec<i64> = photo_rows(a, r.session).iter().map(|p| p.id).collect();
    lamp_template(a);
    let outcome = a.remote().bring_back(r.session).unwrap();
    (r, outcome.photo_ids, retired)
}

/// A new RAW written into the session's folder and delivered by the watcher.
fn arrive(a: &TestApp, r: &Remote, name: &str) -> Photo {
    a.clock.advance(60_000);
    let (bytes, _) =
        SyntheticRaw::build_with_frame(&[(1600, 1067, true)], 1, None, Some((6000, 4000)));
    let path = r.folder.join(name);
    fs::write(&path, bytes).unwrap();
    a.watcher_event(&path).unwrap();
    a.tick(1_000).unwrap();
    let canon = fs::canonicalize(&path).unwrap();
    photo_rows(a, r.session)
        .into_iter()
        .find(|p| Path::new(&p.raw_path) == canon)
        .expect("arrival filed into the session")
}

fn main_order(a: &TestApp, session: i64) -> Vec<(i64, Option<i64>)> {
    view(a, session)
        .iter()
        .filter(|p| p.classification == C::Main)
        .map(|p| (p.id, p.position))
        .collect()
}

#[test]
fn h3_c9a_published_order_kept_and_retired_shots_released() {
    let a = shop_app();
    let (r, ids, retired) = templated(&a, "TPL-3", 3);
    assert_eq!(
        main_order(&a, r.session),
        vec![(ids[0], Some(1)), (ids[1], Some(2)), (ids[2], Some(3))]
    );
    for (k, id) in ids.iter().enumerate() {
        let photo = a.store().photo(*id).unwrap();
        assert!(photo.raw_path.ends_with(&format!("-main-0{}.jpg", k + 1)));
        let row = a.store().photo_place_row(*id).unwrap().expect("place row");
        assert!(row.place_by_hand);
        assert_eq!(row.slot, Some(k as i64 + 1));
    }
    assert_eq!(retired.len(), 3);
    for id in retired {
        let row = a.store().photo_place_row(id).unwrap().expect("place row");
        assert!(row.released, "{id}");
    }
}

#[test]
fn h3_c9b_a_later_arrival_goes_past_the_containers_to_condition() {
    let a = shop_app();
    let (r, ids, _) = templated(&a, "TPL-3", 3);
    let late = arrive(&a, &r, "DSC09001.ARW");
    let late = a.store().photo(late.id).unwrap();
    assert_eq!(
        (late.classification, late.position),
        (C::Condition, Some(1))
    );
    assert_eq!(
        main_order(&a, r.session),
        vec![(ids[0], Some(1)), (ids[1], Some(2)), (ids[2], Some(3))]
    );
}

#[test]
fn h3_c9c_a_free_container_is_filled_by_the_next_arrival() {
    let a = shop_app();
    let (r, ids, _) = templated(&a, "TPL-2", 2);
    let late = arrive(&a, &r, "DSC09002.ARW");
    let late = a.store().photo(late.id).unwrap();
    assert_eq!((late.classification, late.position), (C::Main, Some(3)));
    assert_eq!(
        main_order(&a, r.session),
        vec![(ids[0], Some(1)), (ids[1], Some(2)), (late.id, Some(3))]
    );
}
