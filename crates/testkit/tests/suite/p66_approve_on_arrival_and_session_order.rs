//! P66 approve on arrival, Deny deletes the RAW after the undo window, stable session order
//! (changelog 46 (1)(2)): one behaviour test per criterion row, at the boundary the row names —
//! the stored photos after an arrival, the fake Capture One's colour tags, the session list, the
//! storage job's deletions and log, the files on disk, the undo history and the fake shop's
//! uploads. Temp folders only; no Capture One, camera or shop is needed.

use psm_core::{
    domain::{
        attribute::{AttributeGroup, CoreType, Definition},
        photo::{Classification as C, NewPhoto, Photo, PhotoSource},
        publish::DefinitionLink,
        session::{ListFilter, PublishStep, SessionStatus},
    },
    mechanisms::{arrival::path_key, hash::sha256_file},
    ports::clock::Clock,
    services::{attributes::AttributesService, photos::PhotosService, storage::Cleared},
};
use psm_testkit::{TestApp, fixtures::synthetic_jpeg::SyntheticJpeg};
use serde_json::json;
use std::{
    collections::BTreeSet,
    fs,
    io::Write,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

const NOW_MS: i64 = 1_762_840_000_000; // 2025-11-11
const HOUR: i64 = 3_600_000;
const DAY: i64 = 24 * HOUR;
/// Capture One's colour tags (D2): main → green.
const GREEN: i32 = 4;

// ---------- helpers ----------

#[derive(Clone)]
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
/// Runs `f` with the log lines it writes collected; returns its result and the log text.
fn logged<T>(f: impl FnOnce() -> T) -> (T, String) {
    let buffer = Arc::new(Mutex::new(Vec::new()));
    let output = LogBytes(buffer.clone());
    // P70: keeps every log site wanted while this thread-scoped capture runs (one shared process).
    psm_testkit::logs::keep_log_sites_interested();
    let subscriber = tracing_subscriber::fmt()
        .with_writer(move || output.clone())
        .without_time()
        .with_ansi(false)
        .finish();
    let value = tracing::subscriber::with_default(subscriber, f);
    let text = String::from_utf8(buffer.lock().unwrap().clone()).unwrap();
    (value, text)
}

fn capture_root(a: &TestApp) -> PathBuf {
    fs::canonicalize(a.dir.path().join("Capture One Session/Capture")).unwrap()
}

/// An item session made active (captures and imports join it); returns (session, folder).
fn active_session(a: &TestApp, sku: &str) -> (i64, PathBuf) {
    let (item, session) = a.items().create_local(sku, sku, "object", None).unwrap();
    a.sessions().activate(session.session.id).unwrap();
    (session.session.id, a.capture_root().join(item.folder_key))
}

/// The session's photos as (RAW file name, classification, position), in id order.
fn placed(a: &TestApp, session: i64) -> Vec<(String, C, Option<i64>)> {
    let mut photos = a.store().photos_in_session(session).unwrap();
    photos.sort_by_key(|p| p.id);
    photos
        .into_iter()
        .map(|p| {
            (
                Path::new(&p.raw_path)
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned(),
                p.classification,
                p.position,
            )
        })
        .collect()
}

fn status(a: &TestApp, session: i64) -> SessionStatus {
    a.sessions()
        .list(ListFilter::All, None)
        .unwrap()
        .groups
        .into_iter()
        .flat_map(|g| g.rows)
        .find(|r| r.id == session)
        .unwrap()
        .status
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

/// A RAW in `<capture root>/<folder>/`, known to the fake document, with a rendered preview,
/// classified `main` (as it arrives since P66).
fn main_photo(a: &TestApp, session: i64, folder: &str, name: &str) -> Photo {
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
    a.review().classify(photo.id, C::Main).unwrap();
    a.store().photo(photo.id).unwrap()
}

fn item_session(a: &TestApp, sku: &str) -> i64 {
    a.items()
        .create_local(sku, &format!("{sku} title"), "Sideboard", None)
        .unwrap()
        .1
        .session
        .id
}

/// The RAW and every file the app keeps of it (previews, thumbnail, masters, outputs).
fn files_of(a: &TestApp, raw: &str) -> BTreeSet<PathBuf> {
    let mut out = BTreeSet::new();
    if Path::new(raw).is_file() {
        out.insert(PathBuf::from(raw));
    }
    let key = path_key(raw);
    for kind in ["previews", "masters", "outputs"] {
        let dir = a.app.paths.root.join(kind).join(&key[..2]).join(&key);
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries {
                out.insert(entry.unwrap().path());
            }
        }
    }
    out
}

/// Ticks the storage job past its batch pauses until it has nothing more to do.
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

fn denied_paths(cleared: &[Cleared]) -> BTreeSet<PathBuf> {
    cleared
        .iter()
        .filter(|c| c.rule == "denied")
        .map(|c| c.path.clone())
        .collect()
}

fn assert_logged(log: &str, cleared: &[Cleared]) {
    for c in cleared.iter().filter(|c| c.rule == "denied") {
        let line = format!(
            "storage.cleared rule=denied path={} bytes={}",
            c.path.display(),
            c.bytes
        );
        assert!(log.contains(&line), "{line} not in the log");
    }
}

fn shop_app() -> TestApp {
    let a = TestApp::with_fake_shop().unwrap();
    a.clock.advance(NOW_MS);
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
    a
}

fn drain(a: &TestApp) {
    for _ in 0..12 {
        a.tick(1100).unwrap();
        a.complete_all_renders().unwrap();
    }
}

// ---------- C1 approved on arrival ----------

#[test]
fn c1a_three_arrivals_are_main_in_arrival_order_mirrored_green_and_ready_once_previews_exist() {
    let a = TestApp::with_camera().unwrap();
    let (session, folder) = active_session(&a, "P66A");
    for (name, variant) in [
        ("DSC00001.ARW", "v-1"),
        ("IMPORTED.ARW", "v-2"),
        ("DSC00003.ARW", "v-3"),
    ] {
        a.fake.list_after_sync(name, variant);
    }
    // FTP, then an import, then FTP again.
    a.arrival_photo(session, "DSC00001.ARW").unwrap();
    a.tick(1_000).unwrap();
    let source = a.dir.path().join("IMPORTED.ARW");
    fs::write(&source, b"an imported raw").unwrap();
    PhotosService {
        arrival: a.app.arrival.as_ref(),
        editing: a.app.editing.as_ref(),
        store: a.store(),
        clock: a.clock.as_ref(),
        history: a.app.history.as_ref(),
    }
    .import(&[source.to_string_lossy().into_owned()])
    .unwrap();
    a.tick(1_000).unwrap();
    a.arrival_photo(session, "DSC00003.ARW").unwrap();

    assert_eq!(
        placed(&a, session),
        vec![
            ("DSC00001.ARW".into(), C::Main, Some(1)),
            ("IMPORTED.ARW".into(), C::Main, Some(2)),
            ("DSC00003.ARW".into(), C::Main, Some(3)),
        ]
    );
    assert!(
        folder.join("IMPORTED.ARW").is_file(),
        "the import was filed into the session's folder"
    );
    assert_eq!(
        status(&a, session),
        SessionStatus::InProgress,
        "no previews yet"
    );

    // Capture One lists the photos once it synchronised the folder; the mirror is retried.
    for _ in 0..40 {
        a.tick(2_000).unwrap();
        a.complete_all_renders().unwrap();
    }
    for variant in ["v-1", "v-2", "v-3"] {
        assert_eq!(a.fake.get_color_tag(variant).unwrap(), GREEN, "{variant}");
    }
    assert!(
        a.store()
            .photos_in_session(session)
            .unwrap()
            .iter()
            .all(|p| p.adjusted_preview_path.is_some())
    );
    assert_eq!(status(&a, session), SessionStatus::Ready);
}

#[test]
fn c1b_photos_already_unreviewed_stay_unreviewed_and_a_new_arrival_is_main() {
    let a = TestApp::with_fake_document().unwrap();
    let (session, folder) = active_session(&a, "P66B");
    let old1 = a.add_photo(session, "OLD1.ARW").unwrap();
    let old2 = a.add_photo(session, "OLD2.ARW").unwrap();
    assert_eq!(old1.classification, C::Unreviewed);
    fs::create_dir_all(&folder).unwrap();
    let raw = folder.join("NEW.ARW");
    fs::write(&raw, b"a new capture").unwrap();
    a.watcher_event(&raw).unwrap();
    a.tick(1_000).unwrap();

    assert_eq!(
        placed(&a, session),
        vec![
            ("OLD1.ARW".into(), C::Unreviewed, None),
            ("OLD2.ARW".into(), C::Unreviewed, None),
            ("NEW.ARW".into(), C::Main, Some(1)),
        ]
    );
    assert_eq!(
        a.store().photo(old2.id).unwrap().classification,
        C::Unreviewed
    );
}

// ---------- C2 Deny ----------

#[test]
fn c2a_deny_then_undo_within_the_window_restores_the_photo_and_never_touches_the_raw() {
    let a = TestApp::with_fake_document().unwrap();
    a.clock.advance(NOW_MS);
    let session = item_session(&a, "DENY-UNDO");
    let photos: Vec<Photo> = (1..=3)
        .map(|n| main_photo(&a, session, "DENY-UNDO", &format!("P{n}")))
        .collect();
    let denied = &photos[1];
    let bytes = fs::read(&denied.raw_path).unwrap();
    let files = files_of(&a, &denied.raw_path);
    assert!(files.len() > 1, "the RAW and its previews");

    a.review().delete(denied.id).unwrap();
    assert_eq!(
        a.store().photo(denied.id).unwrap().classification,
        C::Deleted
    );
    a.clock.advance(HOUR);
    let undone = a.history().undo(session).unwrap();
    assert_eq!(undone.active_photo_id, Some(denied.id));
    let back = a.store().photo(denied.id).unwrap();
    assert_eq!((back.classification, back.position), (C::Main, Some(2)));
    assert_eq!(
        placed(&a, session)
            .into_iter()
            .map(|(_, c, p)| (c, p))
            .collect::<Vec<_>>(),
        vec![(C::Main, Some(1)), (C::Main, Some(2)), (C::Main, Some(3))]
    );

    // Long after the window would have closed, nothing of the restored photo is deleted.
    a.clock.advance(2 * DAY);
    let cleared = clear_all(&a);
    assert_eq!(denied_paths(&cleared), BTreeSet::new());
    assert_eq!(fs::read(&denied.raw_path).unwrap(), bytes);
    assert_eq!(files_of(&a, &denied.raw_path), files);
}

#[test]
fn c2b_i_deny_deletes_the_raw_when_the_sessions_next_publish_completes() {
    let a = shop_app();
    let session = item_session(&a, "DENYPUB");
    let m1 = main_photo(&a, session, "DENYPUB", "M1");
    let m2 = main_photo(&a, session, "DENYPUB", "M2");
    let denied = main_photo(&a, session, "DENYPUB", "D");
    let restored = main_photo(&a, session, "DENYPUB", "R");
    a.review().delete(denied.id).unwrap();
    a.review().delete(restored.id).unwrap();
    a.history().undo(session).unwrap();
    let denied_files = files_of(&a, &denied.raw_path);
    let kept: Vec<(PathBuf, Vec<u8>)> = [&m1, &m2, &restored]
        .iter()
        .map(|p| (PathBuf::from(&p.raw_path), fs::read(&p.raw_path).unwrap()))
        .collect();

    // Before the publish the window is open.
    a.clock.advance(HOUR);
    assert_eq!(denied_paths(&clear_all(&a)), BTreeSet::new());
    assert!(Path::new(&denied.raw_path).is_file());

    a.app
        .publish
        .as_ref()
        .unwrap()
        .confirm(&[session], "Publish images")
        .unwrap();
    drain(&a);
    assert_eq!(
        a.store().latest_publish(session).unwrap().map(|l| l.step),
        Some(PublishStep::Done)
    );
    // The denied photo was never published: the three others were uploaded, nothing else.
    assert_eq!(a.fake_shop().staged_filenames().len(), 3);

    let (cleared, log) = logged(|| clear_all(&a));
    assert_eq!(denied_paths(&cleared), denied_files);
    assert!(denied_files.iter().all(|f| !f.exists()));
    assert_logged(&log, &cleared);
    for (raw, bytes) in kept {
        assert_eq!(fs::read(&raw).unwrap(), bytes, "{}", raw.display());
    }
    assert_eq!(
        a.store().photo(denied.id).unwrap().classification,
        C::Deleted,
        "the row stays, denied"
    );
}

#[test]
fn c2b_ii_deny_deletes_the_raw_24_hours_after_the_deny_without_a_publish() {
    let a = TestApp::with_fake_document().unwrap();
    a.clock.advance(NOW_MS);
    let session = item_session(&a, "DENY24");
    let denied = main_photo(&a, session, "DENY24", "D");
    let restored = main_photo(&a, session, "DENY24", "R");
    let other = main_photo(&a, session, "DENY24", "M");
    a.review().delete(denied.id).unwrap();
    let denied_at = a.clock.now_ms();
    a.clock.advance(60_000);
    a.review().delete(restored.id).unwrap();
    a.history().undo(session).unwrap();
    assert_eq!(
        a.history().state(session).undo.as_deref(),
        Some("Delete 01"),
        "the deny can be undone"
    );
    let denied_files = files_of(&a, &denied.raw_path);
    let restored_files = files_of(&a, &restored.raw_path);
    let other_files = files_of(&a, &other.raw_path);

    // 23 h after the deny the photo can still be restored: nothing is deleted.
    a.clock.advance(denied_at + 23 * HOUR - a.clock.now_ms());
    assert_eq!(denied_paths(&clear_all(&a)), BTreeSet::new());
    assert_eq!(files_of(&a, &denied.raw_path), denied_files);

    a.clock.advance(denied_at + DAY + 60_000 - a.clock.now_ms());
    let (cleared, log) = logged(|| clear_all(&a));
    assert_eq!(denied_paths(&cleared), denied_files);
    assert!(denied_files.iter().all(|f| !f.exists()));
    assert_logged(&log, &cleared);
    assert_eq!(files_of(&a, &restored.raw_path), restored_files);
    assert_eq!(files_of(&a, &other.raw_path), other_files);
    assert_eq!(
        a.store().photo(denied.id).unwrap().classification,
        C::Deleted
    );
    // The deny can no longer be undone: its steps were dropped; the next undo is the last
    // step of another photo (approving M), and the denied photo stays denied.
    assert_eq!(
        a.history().state(session).undo.as_deref(),
        Some("Approve 03")
    );
    a.history().undo(session).unwrap();
    assert_eq!(
        a.store().photo(denied.id).unwrap().classification,
        C::Deleted
    );
    assert_eq!(
        a.store().photo(other.id).unwrap().classification,
        C::Unreviewed
    );
    assert!(a.review().classify(denied.id, C::Main).is_err());
}

// ---------- C3 session order ----------

#[test]
fn c3a_sessions_stay_in_creation_order_newest_first_and_are_grouped_by_creation_time() {
    let a = TestApp::with_fake_document().unwrap();
    a.clock.advance(NOW_MS - 40 * DAY);
    let (item_a, dto_a) = a
        .items()
        .create_local("P66-A", "Alpha", "object", None)
        .unwrap();
    let session_a = dto_a.session.id;
    a.clock.advance(40 * DAY);
    let session_b = item_session(&a, "P66-B");
    a.clock.advance(60_000);
    let session_c = item_session(&a, "P66-C");
    a.clock.advance(60_000);

    // Open A, edit B (a photo classified), capture into A.
    a.sessions().view(session_a).unwrap();
    a.sessions().activate(session_a).unwrap();
    a.clock.advance(60_000);
    let photo = a.add_photo(session_b, "B1.ARW").unwrap();
    a.review().classify(photo.id, C::Condition).unwrap();
    a.clock.advance(60_000);
    let folder = a.capture_root().join(&item_a.folder_key);
    fs::create_dir_all(&folder).unwrap();
    let raw = folder.join("A1.ARW");
    fs::write(&raw, b"a capture into A").unwrap();
    a.watcher_event(&raw).unwrap();
    a.tick(1_000).unwrap();
    assert_eq!(a.store().photos_in_session(session_a).unwrap().len(), 1);

    for filter in [ListFilter::Open, ListFilter::All] {
        for query in [None, Some("P66".to_owned())] {
            let list = a.sessions().list(filter, query.clone()).unwrap();
            let order: Vec<i64> = list
                .groups
                .iter()
                .flat_map(|g| g.rows.iter().map(|r| r.id))
                .collect();
            assert_eq!(
                order,
                vec![session_c, session_b, session_a],
                "{filter:?} {query:?}"
            );
            let group_of = |id: i64| {
                list.groups
                    .iter()
                    .find(|g| g.rows.iter().any(|r| r.id == id))
                    .unwrap()
                    .label
                    .clone()
            };
            assert_eq!(group_of(session_c), "Today");
            assert_eq!(group_of(session_b), "Today");
            assert_ne!(group_of(session_a), "Today", "A was created 40 days ago");
        }
    }
}
