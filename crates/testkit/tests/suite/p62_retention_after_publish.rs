//! P62 retention after publish (10F §3, §4 levels 4–5, Rules, Q1–Q4, M41): one behaviour test per
//! criterion row, at the storage service (`App::storage`), the files on disk, the fake Shopify's
//! read-back, the health report, the services a cleared item refuses, and the archived view.
//! Every item is really published to the fake shop first (publish flow, fake Capture One). The
//! disk's free space is the test's stand-in; no test touches a real shop or the network.

use psm_core::{
    Error,
    domain::{
        attribute::{AttributeGroup, CoreType, Definition, Provenance},
        composition::{CompositionKind, CompositionValues, Flip, Orientation, Shape},
        photo::{Classification as C, NewPhoto, Photo, PhotoSource},
        publish::DefinitionLink,
        session::{ListFilter, PublishStep, RowLabel},
    },
    mechanisms::{
        composition::{default_rect, turned_frame},
        hash::sha256_file,
    },
    ports::{clock::Clock, commerce::CommerceAdapter},
    services::{
        attributes::AttributesService,
        photos::PhotosService,
        retention::{ARCHIVED_IMAGE_EDGE, ArchiveService},
        storage::{Cleared, RETENTION_NOT_CLEARED},
    },
};
use psm_testkit::{
    TestApp,
    fake_commerce::{fake_image_bytes, fake_image_url},
    fixtures::synthetic_jpeg::SyntheticJpeg,
};
use serde_json::json;
use std::{
    collections::BTreeSet,
    fs,
    io::Write,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

const NOW_MS: i64 = 1_762_250_400_000; // 2025-11-04T10:00:00Z
const DAY: i64 = 86_400_000;
const GB: u64 = 1_000_000_000;

// ---------------------------------------------------------------------------------------------
// helpers

fn shop_app() -> TestApp {
    let a = TestApp::with_fake_shop().unwrap();
    a.clock.advance(NOW_MS);
    condition_field(&a);
    a
}

/// The shop's condition-photos field (`custom.damage_report`, a file list).
fn condition_field(a: &TestApp) {
    let def = Definition {
        integration: "shopify".into(),
        key: "custom.damage_report".into(),
        label: "Damage report".into(),
        core_type: CoreType::MediaList,
        choices: Vec::new(),
        validations: json!({}),
        group: AttributeGroup::Condition,
        description: None,
        writable: true,
    };
    a.fake_shop().state.lock().unwrap().definitions = vec![(
        def,
        DefinitionLink {
            platform_type: "list.file_reference".into(),
            platform_meta: json!({}),
        },
    )];
    AttributesService::new(a.store(), a.clock.as_ref(), Some("shopify"))
        .refresh_schema(a.fake_shop())
        .unwrap();
}

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

fn capture_root(a: &TestApp) -> PathBuf {
    fs::canonicalize(a.dir.path().join("Capture One Session/Capture")).unwrap()
}

/// A RAW in `<capture root>/<folder>/`, known to the fake document, with a rendered first preview
/// (the composition's frame), classified.
fn photo(a: &TestApp, session: i64, folder: &str, name: &str, class: C) -> Photo {
    let dir = capture_root(a).join(folder);
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
    if class != C::Unreviewed {
        a.review().classify(photo.id, class).unwrap();
    }
    a.store().photo(photo.id).unwrap()
}

/// A Capture One settings sidecar and two cache files named after the RAW.
fn capture_one_files(raw: &str) -> Vec<PathBuf> {
    let raw = Path::new(raw);
    let folder = raw.parent().unwrap().join("CaptureOne");
    let name = raw.file_name().unwrap().to_string_lossy().into_owned();
    let files = vec![
        folder.join("Settings1680").join(format!("{name}.cos")),
        folder.join("Cache/Proxies").join(format!("{name}.cop")),
        folder.join("Cache/Thumbnails").join(format!("{name}.cot")),
    ];
    for file in &files {
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(file, format!("capture one data of {name}")).unwrap();
    }
    files
}

struct Item {
    session: i64,
    item: i64,
    photos: Vec<Photo>,
}

fn drain(a: &TestApp) {
    for _ in 0..12 {
        a.tick(1100).unwrap();
        a.complete_all_renders().unwrap();
    }
}

fn latest_step(a: &TestApp, session: i64) -> Option<PublishStep> {
    a.store().latest_publish(session).unwrap().map(|l| l.step)
}

/// An item with its photos (`main`, `condition`, plus a deleted one and a never-reviewed one),
/// not published.
fn item(a: &TestApp, sku: &str, main: usize, condition: usize) -> Item {
    let (_, session) = a
        .items()
        .create_local(sku, &format!("{sku} title"), "Sideboard", None)
        .unwrap();
    let session = session.session.id;
    let item = a.store().session(session).unwrap().session.item_id.unwrap();
    let mut photos = Vec::new();
    for i in 0..main {
        photos.push(photo(a, session, sku, &format!("{sku}_M{i}"), C::Main));
    }
    for i in 0..condition {
        photos.push(photo(a, session, sku, &format!("{sku}_C{i}"), C::Condition));
    }
    let hidden = photo(a, session, sku, &format!("{sku}_D"), C::Main);
    hidden_before_p66(a, hidden.id);
    photos.push(a.store().photo(hidden.id).unwrap());
    photos.push(photo(a, session, sku, &format!("{sku}_U"), C::Unreviewed));
    Item {
        session,
        item,
        photos,
    }
}

/// A photo hidden before P66 (no deny mark): its RAW stays until retention clears the item (Q1).
/// Since P66 a fresh deny is deleted once its undo window closes (changelog 46 (2); p66 tests).
fn hidden_before_p66(a: &TestApp, photo_id: i64) {
    a.review().delete(photo_id).unwrap();
    rusqlite::Connection::open(a.app.paths.db())
        .unwrap()
        .execute("DELETE FROM photo_denials WHERE photo_id=?1", [photo_id])
        .unwrap();
}

/// `item` published to the fake shop now (the publish flow runs to done).
fn published(a: &TestApp, sku: &str, main: usize, condition: usize) -> Item {
    let it = item(a, sku, main, condition);
    // A never-reviewed photo blocks the ready rule: review it as deleted before publishing.
    let unreviewed = it.photos.last().unwrap().id;
    hidden_before_p66(a, unreviewed);
    a.app
        .publish
        .as_ref()
        .unwrap()
        .confirm(&[it.session], "Publish images")
        .unwrap();
    drain(a);
    assert_eq!(latest_step(a, it.session), Some(PublishStep::Done), "{sku}");
    assert!(!a.store().session(it.session).unwrap().dirty, "{sku}");
    it
}

/// Every local file of the item on disk now: RAWs, the Capture One files the test wrote, the
/// preview / master / output cache folders of its RAWs.
fn files_of(a: &TestApp, it: &Item) -> BTreeSet<PathBuf> {
    let mut out = BTreeSet::new();
    for p in &it.photos {
        out.insert(PathBuf::from(&p.raw_path));
        let raw = Path::new(&p.raw_path);
        let name = raw.file_name().unwrap().to_string_lossy().into_owned();
        let capture_one = raw.parent().unwrap().join("CaptureOne");
        for sub in ["Settings1680", "Cache/Proxies", "Cache/Thumbnails"] {
            for ext in ["cos", "cop", "cot"] {
                let f = capture_one.join(sub).join(format!("{name}.{ext}"));
                if f.is_file() {
                    out.insert(f);
                }
            }
        }
        let key = psm_core::mechanisms::arrival::path_key(&p.raw_path);
        for kind in ["previews", "masters", "outputs"] {
            let dir = a.app.paths.root.join(kind).join(&key[..2]).join(&key);
            if let Ok(entries) = fs::read_dir(&dir) {
                for e in entries {
                    out.insert(e.unwrap().path());
                }
            }
        }
    }
    out
}

fn disk_free(a: &TestApp, bytes: u64) {
    a.app.disk_space().set(Arc::new(move |_: &Path| {
        Ok(psm_infra::disk::FreeSpace {
            available_bytes: bytes,
            disk: "the test disk".into(),
        })
    }));
}

/// Ticks the clean-up past its batch pauses until it has nothing more to do.
fn clear_all(a: &TestApp) -> Vec<Cleared> {
    let mut all = Vec::new();
    for _ in 0..40 {
        a.clock.advance(3_000);
        let outcome = a.app.storage.tick().unwrap();
        assert_eq!(outcome.paused, None);
        all.extend(outcome.cleared);
    }
    all
}

fn paths(cleared: &[Cleared], rule: &str) -> BTreeSet<PathBuf> {
    cleared
        .iter()
        .filter(|c| c.rule == rule)
        .map(|c| c.path.clone())
        .collect()
}

fn health_detail(a: &TestApp, code: &str) -> Option<String> {
    a.app
        .health
        .report("", false, false)
        .items
        .into_iter()
        .find(|i| i.code == code && !i.ok)
        .map(|i| i.detail)
}

fn product_of(a: &TestApp, it: &Item) -> String {
    a.store()
        .item_link(it.item, "shopify")
        .unwrap()
        .unwrap()
        .external_id
}

fn cleared_at(a: &TestApp, it: &Item) -> Option<i64> {
    a.store().item_retention(it.item).unwrap().cleared_at
}

struct LogBytes(Arc<Mutex<Vec<u8>>>);
impl Write for LogBytes {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
impl Clone for LogBytes {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

// ---------------------------------------------------------------------------------------------
// C1 — retention clears after N days

#[test]
fn c1a_a_published_clean_item_is_cleared_four_days_after_publish_and_nothing_else() {
    let a = shop_app();
    let pub_item = published(&a, "A", 2, 1);
    for p in &pub_item.photos {
        capture_one_files(&p.raw_path);
    }
    // An identical copy of one RAW in `.duplicates/` (the second delivery path).
    let first_raw = PathBuf::from(&pub_item.photos[0].raw_path);
    let duplicate = capture_root(&a)
        .join(".duplicates")
        .join(first_raw.file_name().unwrap());
    fs::create_dir_all(duplicate.parent().unwrap()).unwrap();
    fs::copy(&first_raw, &duplicate).unwrap();
    // Another item, not published, with its own files.
    let other = item(&a, "B", 1, 0);
    for p in &other.photos {
        capture_one_files(&p.raw_path);
    }
    let mut expected = files_of(&a, &pub_item);
    expected.insert(duplicate.clone());
    let other_files = files_of(&a, &other);
    assert!(expected.len() > 4 * 4, "RAWs, Capture One files, caches");

    a.clock.advance(3 * DAY);
    let cleared = clear_all(&a);
    assert_eq!(
        paths(&cleared, "retention"),
        BTreeSet::new(),
        "not at T+3 d"
    );
    assert!(expected.iter().all(|f| f.is_file()));
    assert_eq!(cleared_at(&a, &pub_item), None);

    a.clock.advance(DAY + 60_000);
    let buffer = Arc::new(Mutex::new(Vec::new()));
    let output = LogBytes(buffer.clone());
    // P70: keeps every log site wanted while this thread-scoped capture runs (one shared process).
    psm_testkit::logs::keep_log_sites_interested();
    let subscriber = tracing_subscriber::fmt()
        .with_writer(move || output.clone())
        .without_time()
        .with_ansi(false)
        .finish();
    let cleared = tracing::subscriber::with_default(subscriber, || clear_all(&a));
    assert_eq!(
        paths(&cleared, "retention"),
        expected,
        "cleared at T+4 d 1 min"
    );
    assert!(expected.iter().all(|f| !f.exists()), "every file is gone");
    assert!(
        other_files.iter().all(|f| f.is_file()),
        "the other item's files are untouched"
    );
    assert!(cleared_at(&a, &pub_item).is_some());
    let log = String::from_utf8(buffer.lock().unwrap().clone()).unwrap();
    for c in cleared.iter().filter(|c| c.rule == "retention") {
        assert!(
            log.contains(&format!(
                "storage.cleared rule=retention path={} bytes={}",
                c.path.display(),
                c.bytes
            )),
            "{}",
            c.path.display()
        );
    }
}

#[test]
fn c1c_clearing_interrupted_by_a_restart_finishes_after_it_and_touches_nothing_else() {
    let mut a = shop_app();
    let pub_item = published(&a, "RESUME", 2, 1);
    for p in &pub_item.photos {
        capture_one_files(&p.raw_path);
    }
    let other = item(&a, "KEEP", 1, 0);
    for p in &other.photos {
        capture_one_files(&p.raw_path);
    }
    let files = files_of(&a, &pub_item);
    let other_files = files_of(&a, &other);
    assert!(files.len() > 20, "more files than one batch");

    // Clearing starts: the item is marked cleared and the first batch is deleted.
    a.clock.advance(4 * DAY + 60_000);
    let mut deleted = 0;
    while deleted == 0 {
        a.clock.advance(3_000);
        deleted = a.app.storage.tick().unwrap().cleared.len();
    }
    assert!(cleared_at(&a, &pub_item).is_some());
    let left = files.iter().filter(|f| f.exists()).count();
    assert!(left > 0 && left < files.len(), "interrupted midway: {left}");

    // The app quits before the rest is deleted; after the restart the clean-up finishes it.
    a.kill_and_restart().unwrap();
    clear_all(&a);
    assert!(
        files.iter().all(|f| !f.exists()),
        "left: {:?}",
        files.iter().filter(|f| f.exists()).collect::<Vec<_>>()
    );
    assert!(
        other_files.iter().all(|f| f.is_file()),
        "the other item's files are untouched"
    );
}

#[test]
fn c1d_a_photo_that_joins_a_cleared_item_afterwards_is_never_deleted() {
    let a = shop_app();
    let it = published(&a, "LATE", 2, 1);
    for p in &it.photos {
        capture_one_files(&p.raw_path);
    }
    let files = files_of(&a, &it);
    a.clock.advance(4 * DAY + 60_000);
    clear_all(&a);
    assert!(cleared_at(&a, &it).is_some());
    assert!(files.iter().all(|f| !f.exists()), "the item is cleared");

    // A RAW that joined the cleared item's session after the clearing (never published).
    a.clock.advance(60_000);
    let late = photo(&a, it.session, "LATE", "LATE_new", C::Unreviewed);
    let late_files: Vec<PathBuf> = std::iter::once(PathBuf::from(&late.raw_path))
        .chain(capture_one_files(&late.raw_path))
        .collect();

    // Past the leftover interval, with the disk full so every level is open: it stays.
    a.clock.advance(10 * 60_000);
    disk_free(&a, 0);
    let cleared = clear_all(&a);
    for f in &late_files {
        assert!(f.is_file(), "deleted: {f:?}");
    }
    assert!(
        !cleared.iter().any(|c| late_files.contains(&c.path)),
        "{cleared:?}"
    );
}

#[test]
fn c1e_what_arrives_while_a_cleared_item_is_active_goes_to_unassigned() {
    let a = shop_app();
    let it = published(&a, "ARCH", 1, 0);
    a.clock.advance(4 * DAY + 60_000);
    clear_all(&a);
    assert!(cleared_at(&a, &it).is_some());
    a.store().set_active_session_id(Some(it.session)).unwrap();
    let before = a.store().photos_in_session(it.session).unwrap().len();

    // A capture arriving while the archived item is active.
    let captured = a.arrival_photo(it.session, "ARCH_shot.ARW").unwrap();
    // An import while the archived item is active.
    let outside = a.dir.path().join("outside_new.ARW");
    fs::write(&outside, b"a raw nobody published").unwrap();
    PhotosService {
        arrival: a.app.arrival.as_ref(),
        editing: a.app.editing.as_ref(),
        store: a.store(),
        clock: a.clock.as_ref(),
        history: a.app.history.as_ref(),
    }
    .import(&[outside.to_string_lossy().into_owned()])
    .unwrap();

    assert_eq!(
        a.store().photos_in_session(it.session).unwrap().len(),
        before,
        "the cleared item gained nothing"
    );
    let unassigned: Vec<String> = a
        .store()
        .photos_in_session(1)
        .unwrap()
        .into_iter()
        .map(|p| p.raw_path)
        .collect();
    assert!(
        unassigned.iter().any(|p| p.ends_with("ARCH_shot.ARW")),
        "{unassigned:?}"
    );
    assert!(
        unassigned.iter().any(|p| p.ends_with("outside_new.ARW")),
        "{unassigned:?}"
    );
    assert!(Path::new(&captured).is_file());
}

#[test]
fn c1b_dirty_unpublished_pinned_or_without_integration_nothing_is_deleted() {
    let a = shop_app();
    // Dirty: a photo reclassified after the publish.
    let dirty = published(&a, "DIRTY", 2, 0);
    a.review()
        .classify(dirty.photos[1].id, C::Condition)
        .unwrap();
    assert!(a.store().session(dirty.session).unwrap().dirty);
    let unpublished = item(&a, "NEW", 1, 0);
    let pinned = published(&a, "PIN", 1, 0);
    ArchiveService::new(a.store(), None, a.dir.path())
        .set_keep_files(pinned.item, true)
        .unwrap();
    let before: Vec<BTreeSet<PathBuf>> = [&dirty, &unpublished, &pinned]
        .iter()
        .map(|it| files_of(&a, it))
        .collect();
    // Long past retention and far below the minimum free space.
    a.clock.advance(30 * DAY);
    disk_free(&a, 0);
    let cleared = clear_all(&a);
    assert_eq!(paths(&cleared, "retention"), BTreeSet::new());
    assert_eq!(paths(&cleared, "budget_published"), BTreeSet::new());
    for files in &before {
        assert!(files.iter().all(|f| f.is_file()));
    }

    // No commerce integration (H6): an item marked published and done is never cleared.
    let b = TestApp::with_fake_document().unwrap();
    b.clock.advance(NOW_MS);
    let local = item(&b, "LOCAL", 1, 0);
    b.mark_published(local.session).unwrap();
    b.insert_publish_item(local.session, "done", NOW_MS, None)
        .unwrap();
    let files = files_of(&b, &local);
    b.clock.advance(30 * DAY);
    disk_free(&b, 0);
    let cleared = clear_all(&b);
    assert_eq!(paths(&cleared, "retention"), BTreeSet::new());
    assert_eq!(paths(&cleared, "budget_published"), BTreeSet::new());
    assert!(files.iter().all(|f| f.is_file()));
}

// ---------------------------------------------------------------------------------------------
// C2 — verify first

#[test]
fn c2a_shopify_missing_an_image_or_unreadable_clears_nothing_and_says_so_on_health() {
    let a = shop_app();
    let it = published(&a, "SKU-7", 2, 0);
    let files = files_of(&a, &it);
    // Someone removed one gallery image in the Shopify admin.
    let product = product_of(&a, &it);
    {
        let mut shop = a.fake_shop().state.lock().unwrap();
        let p = shop
            .products
            .iter_mut()
            .find(|p| p.external_id == product)
            .unwrap();
        assert_eq!(p.media.len(), 2);
        p.media.remove(1);
    }
    a.clock.advance(5 * DAY);
    let cleared = clear_all(&a);
    assert_eq!(paths(&cleared, "retention"), BTreeSet::new());
    assert!(files.iter().all(|f| f.is_file()));
    assert_eq!(cleared_at(&a, &it), None);
    assert_eq!(
        health_detail(&a, RETENTION_NOT_CLEARED).as_deref(),
        Some("Not cleared: SKU-7 — Shopify is missing 1 images")
    );

    // The read fails: nothing cleared either.
    let b = shop_app();
    let it = published(&b, "SKU-8", 1, 0);
    let files = files_of(&b, &it);
    b.fake_shop().state.lock().unwrap().unreachable = true;
    b.clock.advance(5 * DAY);
    let cleared = clear_all(&b);
    assert_eq!(paths(&cleared, "retention"), BTreeSet::new());
    assert!(files.iter().all(|f| f.is_file()));
    assert_eq!(
        health_detail(&b, RETENTION_NOT_CLEARED).as_deref(),
        Some("Not cleared: SKU-8 — Shopify could not be read")
    );
}

// ---------------------------------------------------------------------------------------------
// C3 — after clearing: Shopify images, read-only

#[test]
fn c3a_a_cleared_item_shows_its_photos_from_shopify_cached_and_is_archived_in_the_list() {
    let a = shop_app();
    let it = published(&a, "ARC", 2, 1);
    let product = product_of(&a, &it);
    let gallery: Vec<String> = a
        .fake_shop()
        .media_readback(&product)
        .unwrap()
        .gallery
        .into_iter()
        .map(|m| m.id)
        .collect();
    let condition: Vec<String> = a
        .fake_shop()
        .media_readback(&product)
        .unwrap()
        .condition
        .into_iter()
        .map(|m| m.id)
        .collect();
    assert_eq!((gallery.len(), condition.len()), (2, 1));
    a.clock.advance(4 * DAY);
    clear_all(&a);
    assert!(cleared_at(&a, &it).is_some());

    let cache = a.app.paths.root.join("previews/archived");
    let shop: &dyn CommerceAdapter = a.fake_shop();
    let view = ArchiveService::new(a.store(), Some(shop), &cache)
        .view(it.session)
        .unwrap()
        .unwrap();
    let shown: Vec<(String, i64, String)> = view
        .pictures
        .iter()
        .map(|p| (p.role.clone(), p.position, p.url.clone()))
        .collect();
    assert_eq!(
        shown,
        vec![
            ("gallery".into(), 1, fake_image_url(&gallery[0])),
            ("gallery".into(), 2, fake_image_url(&gallery[1])),
            ("condition".into(), 1, fake_image_url(&condition[0])),
        ]
    );
    for p in &view.pictures {
        let path = p.path.as_ref().expect("downloaded");
        assert!(path.starts_with(&cache));
        assert_eq!(fs::read(path).unwrap(), fake_image_bytes(&p.url));
    }
    let fetches = a.fake_shop().state.lock().unwrap().image_fetches.clone();
    assert_eq!(
        fetches,
        shown
            .iter()
            .map(|(_, _, url)| (url.clone(), ARCHIVED_IMAGE_EDGE))
            .collect::<Vec<_>>()
    );
    // Opened again: served from the cache, nothing downloaded.
    let again = ArchiveService::new(a.store(), Some(shop), &cache)
        .view(it.session)
        .unwrap()
        .unwrap();
    assert_eq!(again.pictures, view.pictures);
    assert_eq!(a.fake_shop().state.lock().unwrap().image_fetches.len(), 3);

    let list = a.sessions().list(ListFilter::All, None).unwrap();
    let row = list
        .groups
        .iter()
        .flat_map(|g| g.rows.iter())
        .find(|r| r.id == it.session)
        .unwrap();
    assert_eq!(row.label, RowLabel::Archived);
}

#[test]
fn c3b_a_cleared_item_refuses_what_needs_images_and_saves_an_attribute_without_images() {
    let a = shop_app();
    let it = published(&a, "RO", 2, 0);
    a.clock.advance(4 * DAY);
    clear_all(&a);
    assert!(cleared_at(&a, &it).is_some());
    let text = "This item's files were cleared after publishing";
    let refused = |result: Result<(), Error>| {
        let error = result.expect_err("refused");
        assert!(matches!(error, Error::FilesCleared), "{error:?}");
        assert_eq!(error.to_string(), text);
    };
    let main = &it.photos[0];
    refused(a.review().classify(main.id, C::Condition).map(|_| ()));
    refused(a.review().reorder(main.id, 2).map(|_| ()));
    let frame = a.composition(main.id).unwrap().frame().unwrap();
    let values = CompositionValues {
        kind: CompositionKind::Normal,
        shape: Shape::Square,
        orientation: Orientation::Named,
        quarter_turns: 0,
        flip: Flip::None,
        straighten: 0.0,
        rect: default_rect(
            CompositionKind::Normal,
            Shape::Square,
            Orientation::Named,
            turned_frame(frame, 0),
        ),
    };
    refused(a.app.compositions.set(main.id, values).map(|_| ()));
    refused(
        a.app
            .publish
            .as_ref()
            .unwrap()
            .confirm(&[it.session], "Publish images")
            .map(|_| ()),
    );
    // Nothing changed: the photo keeps its place.
    assert_eq!(a.store().photo(main.id).unwrap().classification, C::Main);
    // A text attribute needs no image: it saves.
    let saved = a
        .attributes(Some("shopify"))
        .set_value(
            it.item,
            "product.description",
            json!("Solid oak, two drawers."),
            Provenance::Human,
        )
        .unwrap();
    assert_eq!(saved.key, "product.description");
    let stored = a.attributes(Some("shopify")).values(it.item).unwrap();
    assert!(
        stored
            .iter()
            .any(|v| v.key == "product.description" && v.value.contains("Solid oak"))
    );
}

// ---------------------------------------------------------------------------------------------
// C4 — keep files

#[test]
fn c4a_a_pinned_item_is_never_cleared_by_retention_or_by_the_budget() {
    let a = shop_app();
    let pinned = published(&a, "KEEP", 1, 0);
    let free_item = published(&a, "FREE", 1, 0);
    let pinned_files = files_of(&a, &pinned);
    let free_files = files_of(&a, &free_item);
    ArchiveService::new(a.store(), None, a.dir.path())
        .set_keep_files(pinned.item, true)
        .unwrap();
    a.clock.advance(60 * DAY);
    disk_free(&a, 0);
    let cleared = clear_all(&a);
    assert_eq!(paths(&cleared, "retention"), free_files, "the job ran");
    assert!(
        pinned_files.iter().all(|f| f.is_file()),
        "the pinned item keeps its files"
    );
    assert_eq!(cleared_at(&a, &pinned), None);
    assert!(a.store().item_retention(pinned.item).unwrap().keep_files);
}

// ---------------------------------------------------------------------------------------------
// C5 — budget levels 4–5

#[test]
fn c5a_below_the_limit_the_oldest_publish_goes_first_then_the_next_only_if_still_below() {
    let setup = || {
        let a = shop_app();
        let old = published(&a, "OLD", 1, 0); // published 10 days before "now"
        a.clock.advance(8 * DAY);
        let recent = published(&a, "RECENT", 1, 0); // published 2 days before "now"
        let unpublished = item(&a, "OPEN", 1, 0);
        a.clock.advance(2 * DAY);
        (a, old, recent, unpublished)
    };

    // One byte short: clearing the old item (past retention) is enough; the recent one stays.
    let (a, old, recent, unpublished) = setup();
    let (old_files, recent_files, open_files) = (
        files_of(&a, &old),
        files_of(&a, &recent),
        files_of(&a, &unpublished),
    );
    disk_free(&a, 20 * GB - 1);
    let cleared = clear_all(&a);
    assert_eq!(paths(&cleared, "retention"), old_files);
    assert_eq!(paths(&cleared, "budget_published"), BTreeSet::new());
    assert!(recent_files.iter().all(|f| f.is_file()));
    assert!(open_files.iter().all(|f| f.is_file()));

    // Far short: the old item first, then the recent one; the unpublished item never.
    let (a, old, recent, unpublished) = setup();
    let (old_files, recent_files, open_files) = (
        files_of(&a, &old),
        files_of(&a, &recent),
        files_of(&a, &unpublished),
    );
    disk_free(&a, 0);
    let cleared = clear_all(&a);
    assert_eq!(paths(&cleared, "retention"), old_files);
    assert_eq!(paths(&cleared, "budget_published"), recent_files);
    let last_old = cleared
        .iter()
        .rposition(|c| old_files.contains(&c.path))
        .unwrap();
    let first_recent = cleared
        .iter()
        .position(|c| recent_files.contains(&c.path))
        .unwrap();
    assert!(last_old < first_recent, "the oldest publish goes first");
    assert!(open_files.iter().all(|f| f.is_file()));
    assert_eq!(cleared_at(&a, &unpublished), None);
}

// ---------------------------------------------------------------------------------------------
// C6 — Capture One untouched beyond the named files

#[test]
fn c6a_only_files_named_after_the_cleared_raws_go_and_the_catalogue_is_not_modified() {
    let a = shop_app();
    let it = published(&a, "SHARED", 1, 0);
    let ours: Vec<PathBuf> = it
        .photos
        .iter()
        .flat_map(|p| capture_one_files(&p.raw_path))
        .collect();
    // Another item's RAW in the same folder, with its own Capture One files.
    let (_, other_session) = a
        .items()
        .create_local("NEIGHBOUR", "Neighbour", "Chair", None)
        .unwrap();
    let neighbour = photo(&a, other_session.session.id, "SHARED", "DSC0002", C::Main);
    let theirs = capture_one_files(&neighbour.raw_path);
    let catalogue = a
        .dir
        .path()
        .join("Capture One Session/Capture One Session.cosessiondb");
    fs::write(&catalogue, b"catalogue bytes").unwrap();
    let catalogue_time = fs::metadata(&catalogue).unwrap().modified().unwrap();

    a.clock.advance(4 * DAY + 1);
    let cleared = clear_all(&a);
    let removed = paths(&cleared, "retention");
    assert!(ours.iter().all(|f| removed.contains(f) && !f.exists()));
    assert!(!Path::new(&it.photos[0].raw_path).exists());
    assert!(
        Path::new(&neighbour.raw_path).is_file(),
        "the other RAW stays"
    );
    assert!(
        theirs.iter().all(|f| f.is_file()),
        "its Capture One files stay"
    );
    assert!(theirs.iter().all(|f| !removed.contains(f)));
    assert_eq!(fs::read(&catalogue).unwrap(), b"catalogue bytes");
    assert_eq!(
        fs::metadata(&catalogue).unwrap().modified().unwrap(),
        catalogue_time
    );
}

// ---------------------------------------------------------------------------------------------
// C7 — background

#[test]
fn c7a_no_clearing_while_a_publish_runs_then_it_clears() {
    let a = shop_app();
    let it = published(&a, "WAIT", 1, 0);
    let files = files_of(&a, &it);
    // Another item's publish is running (its upload step).
    let busy = item(&a, "BUSY", 1, 0);
    a.insert_publish_item(busy.session, "upload", a.clock.now_ms(), None)
        .unwrap();
    a.clock.advance(5 * DAY);
    for _ in 0..5 {
        a.clock.advance(3_000);
        let outcome = a.app.storage.tick().unwrap();
        assert_eq!(outcome.paused.as_deref(), Some("publish"));
        assert!(outcome.cleared.is_empty());
    }
    assert!(files.iter().all(|f| f.is_file()));
    assert_eq!(cleared_at(&a, &it), None);

    // The publish ends: the next ticks clear the item.
    let id = a
        .store()
        .latest_publish(busy.session)
        .unwrap()
        .unwrap()
        .item_id;
    a.store()
        .set_publish_step(
            id,
            PublishStep::Failed,
            Some(PublishStep::Upload),
            Some("x"),
            a.clock.now_ms(),
        )
        .unwrap();
    let cleared = clear_all(&a);
    assert_eq!(paths(&cleared, "retention"), files);
}
