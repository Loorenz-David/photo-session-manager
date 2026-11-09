//! P61 storage clean-up and budget (10F §1, §2, §4 levels 1–3, §6; 10D MC-28 masters): one
//! behaviour test per criterion row, at the storage service (`App::storage`), the files on disk,
//! the health report and the publish flow. The disk's free space is the test's stand-in.

use psm_core::{
    domain::{
        camera::{CaptureId, Delivery, DeliveryOutcome, ExposureInitiator, ExposureObserved, Link},
        photo::{Classification as C, NewPhoto, Photo, PhotoSource},
        session::PublishStep,
    },
    mechanisms::{arrival::path_key, hash::sha256_file},
    ports::{camera::CaptureSink, clock::Clock},
    services::storage::{Cleared, DISK_ALMOST_FULL, TickOutcome},
};
use psm_testkit::{TestApp, fixtures::synthetic_jpeg::SyntheticJpeg};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, SystemTime},
};

const H: i64 = 3_600_000;
const GB: u64 = 1_000_000_000;

// ---------------------------------------------------------------------------------------------
// helpers

fn sha(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// A local item with its session, active (deliveries go to its folder).
fn active_session(a: &TestApp, sku: &str) -> i64 {
    let (_, session) = a.items().create_local(sku, sku, "object", None).unwrap();
    a.sessions().activate(session.session.id).unwrap();
    session.session.id
}

fn expose(a: &TestApp) {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    a.ledger()
        .exposure_observed(ExposureObserved {
            capture_id: Some(CaptureId(format!(
                "p61-{}",
                NEXT.fetch_add(1, Ordering::Relaxed)
            ))),
            at: a.clock.now_ms(),
            initiated: ExposureInitiator::App,
            provisional: false,
        })
        .unwrap();
}

/// Hands one completed copy of `name` to the ledger, by cable or over Wi-Fi, completed now.
fn deliver(a: &TestApp, name: &str, bytes: &[u8], via: Link) -> DeliveryOutcome {
    let incoming = a.capture_root().join(".incoming");
    let written = match via {
        Link::Wireless => incoming.join(format!(".{name}.part")),
        Link::Cable => incoming.join("sdk-x").join(name),
    };
    fs::create_dir_all(written.parent().unwrap()).unwrap();
    fs::write(&written, bytes).unwrap();
    a.ledger()
        .delivery(Delivery {
            camera_filename: name.into(),
            written_path: written,
            sha256: sha(bytes),
            size: bytes.len() as u64,
            via,
            completed_at: a.clock.now_ms(),
        })
        .unwrap()
}

/// The cable copy is filed, the identical Wi-Fi copy goes to `.duplicates/` (MC-33) at the
/// current clock. Returns `(filed photo, duplicate)`.
fn cable_then_ftp(a: &TestApp, name: &str, bytes: &[u8]) -> (PathBuf, PathBuf) {
    expose(a);
    assert_eq!(deliver(a, name, bytes, Link::Cable), DeliveryOutcome::Photo);
    assert_eq!(
        deliver(a, name, bytes, Link::Wireless),
        DeliveryOutcome::Duplicate
    );
    let rows = a.ledger().deliveries().unwrap();
    let filed = rows
        .iter()
        .find(|r| r.camera_filename == name && r.outcome == DeliveryOutcome::Photo)
        .unwrap()
        .final_path
        .clone();
    let duplicate = a.capture_root().join(".duplicates").join(name);
    assert!(duplicate.is_file());
    (PathBuf::from(filed), duplicate)
}

/// Sets the stand-in disk's free space (shared with the master-render guard).
fn disk_free(a: &TestApp, bytes: u64) {
    a.app.disk_space().set(Arc::new(move |_: &Path| {
        Ok(psm_infra::disk::FreeSpace {
            available_bytes: bytes,
            disk: "the test disk".into(),
        })
    }));
}

fn tick(a: &TestApp) -> TickOutcome {
    a.app.storage.tick().unwrap()
}

/// Ticks past the batch pause until the job has nothing more to clear; every deletion, in order.
fn clear_all(a: &TestApp) -> Vec<Cleared> {
    let mut all = Vec::new();
    for _ in 0..10 {
        a.clock.advance(3_000);
        let outcome = tick(a);
        assert_eq!(outcome.paused, None);
        all.extend(outcome.cleared);
    }
    all
}

fn rules(cleared: &[Cleared]) -> Vec<(String, PathBuf, u64)> {
    cleared
        .iter()
        .map(|c| (c.rule.clone(), c.path.clone(), c.bytes))
        .collect()
}

fn document(a: &TestApp) -> PathBuf {
    fs::canonicalize(a.app.settings.effective_document_id(&a.app.paths)).unwrap()
}

fn write(path: &Path, bytes: &[u8]) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, bytes).unwrap();
}

fn write_at(path: &Path, bytes: &[u8], modified: SystemTime) {
    write(path, bytes);
    fs::File::options()
        .write(true)
        .open(path)
        .unwrap()
        .set_times(fs::FileTimes::new().set_modified(modified))
        .unwrap();
}

/// A master file in the app's master cache of a RAW nobody uses.
fn loose_master(a: &TestApp, raw: &str, bytes: usize, modified: SystemTime) -> PathBuf {
    let path = a.masters_dir(raw).join("master-1.tif");
    write_at(&path, &vec![7u8; bytes], modified);
    path
}

/// Capture One's export in `_session_manager/` with a byte-identical copy in the preview cache
/// (found by the start-up match, 10F §2).
fn copied_export(a: &TestApp, stem: &str, bytes: &[u8]) -> (PathBuf, PathBuf) {
    let export = document(a)
        .join("Output/_session_manager")
        .join(format!("{stem}.jpg"));
    write(&export, bytes);
    let key = path_key(&format!("/nowhere/{stem}.ARW"));
    let cache = a
        .app
        .paths
        .root
        .join("previews")
        .join(&key[..2])
        .join(&key)
        .join("adjusted-1.jpg");
    write(&cache, bytes);
    (export, cache)
}

/// A photo whose RAW is known to the fake Capture One document.
fn raw_photo(a: &TestApp, session: i64, name: &str) -> Photo {
    let dir = a.dir.path().join("Capture One Session/Capture/raws");
    fs::create_dir_all(&dir).unwrap();
    let raw = dir.join(format!("{name}.ARW"));
    fs::write(&raw, format!("raw bytes of {name}")).unwrap();
    let raw = fs::canonicalize(raw).unwrap();
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
    photo
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
// C1 — duplicates

#[test]
fn c1a_a_verified_duplicate_goes_24_hours_after_it_was_diverted() {
    let a = TestApp::with_camera().unwrap();
    active_session(&a, "dup");
    let bytes = b"II*\0 DSC01001 raw".to_vec();
    let (filed, duplicate) = cable_then_ftp(&a, "DSC01001.ARW", &bytes);

    a.clock.advance(23 * H);
    let outcome = tick(&a);
    assert_eq!(outcome, TickOutcome::default());
    assert!(duplicate.is_file(), "not deleted at T+23 h");

    a.clock.advance(2 * H);
    let buffer = Arc::new(Mutex::new(Vec::new()));
    let output = LogBytes(buffer.clone());
    let subscriber = tracing_subscriber::fmt()
        .with_writer(move || output.clone())
        .without_time()
        .with_ansi(false)
        .finish();
    let outcome = tracing::subscriber::with_default(subscriber, || tick(&a));
    assert_eq!(
        rules(&outcome.cleared),
        vec![(
            "duplicate_24h".to_string(),
            duplicate.clone(),
            bytes.len() as u64
        )]
    );
    assert!(!duplicate.exists(), "deleted at T+25 h");
    assert_eq!(fs::read(&filed).unwrap(), bytes, "the filed photo is kept");
    let log = String::from_utf8(buffer.lock().unwrap().clone()).unwrap();
    assert!(
        log.contains(&format!(
            "storage.cleared rule=duplicate_24h path={} bytes={}",
            duplicate.display(),
            bytes.len()
        )),
        "{log}"
    );
}

#[test]
fn c1b_a_file_with_another_hash_is_never_touched() {
    let a = TestApp::with_camera().unwrap();
    active_session(&a, "dup");
    // The camera reused a file number: same name, other bytes → a `__dup` photo (MC-33).
    expose(&a);
    assert_eq!(
        deliver(&a, "DSC02002.ARW", b"first shot", Link::Cable),
        DeliveryOutcome::Photo
    );
    assert_eq!(
        deliver(&a, "DSC02002.ARW", b"second shot", Link::Wireless),
        DeliveryOutcome::RenamedPhoto
    );
    let renamed = a
        .ledger()
        .deliveries()
        .unwrap()
        .into_iter()
        .find(|r| r.outcome == DeliveryOutcome::RenamedPhoto)
        .unwrap()
        .final_path;
    assert!(renamed.contains("__dup"));
    // A file in `.duplicates/` the ledger does not know, whose bytes match no filed photo.
    let stranger = a.capture_root().join(".duplicates/DSC09999.ARW");
    write_at(
        &stranger,
        b"bytes nobody filed",
        SystemTime::now() - Duration::from_secs(72 * 3600),
    );
    // Long past 24 h, and under disk pressure: still neither is a candidate.
    a.clock.advance(72 * H);
    disk_free(&a, GB);
    let cleared = clear_all(&a);
    assert_eq!(cleared, Vec::new());
    assert!(Path::new(&renamed).is_file());
    assert_eq!(fs::read(&stranger).unwrap(), b"bytes nobody filed");
}

// ---------------------------------------------------------------------------------------------
// C2 — preview exports

#[test]
fn c2a_a_preview_export_is_cleared_once_copied_into_the_cache() {
    let a = TestApp::with_fake_document().unwrap();
    let session = a
        .items()
        .create_local("P", "P", "object", None)
        .unwrap()
        .1
        .session
        .id;
    let photo = raw_photo(&a, session, "DSC1");
    let output = document(&a).join("Output");
    let tiff = output.join("DSC7.tif");
    write(&tiff, b"a TIFF someone exported");
    let other = output.join("other/DSC1.jpg");
    write(&other, b"another recipe's export");

    a.app.previews.request_initial(&photo.raw_path).unwrap();
    let export = fs::canonicalize(a.complete_preview_export(&last_issued(&a)).unwrap()).unwrap();
    assert_eq!(
        export,
        document(&a).join("Output/_session_manager/DSC1.jpg")
    );
    let exported = fs::read(&export).unwrap();
    let cache = PathBuf::from(
        a.store()
            .preview_state(&photo.raw_path)
            .unwrap()
            .unwrap()
            .adjusted_preview_path
            .unwrap(),
    );

    let cleared = clear_all(&a);
    assert_eq!(
        rules(&cleared),
        vec![(
            "preview_copied".to_string(),
            export.clone(),
            exported.len() as u64
        )]
    );
    assert!(!export.exists());
    assert_eq!(
        fs::read(&cache).unwrap(),
        exported,
        "the cache copy is intact"
    );
    assert_eq!(fs::read(&tiff).unwrap(), b"a TIFF someone exported");
    assert_eq!(fs::read(&other).unwrap(), b"another recipe's export");
}

// ---------------------------------------------------------------------------------------------
// C3 — masters after publish (10D MC-28)

#[test]
fn c3a_masters_go_after_publish_while_a_publishing_session_keeps_its_own() {
    let a = TestApp::with_fake_shop().unwrap();
    a.clock.advance(1_762_250_400_000);
    let session = |sku: &str| {
        a.items()
            .create_local(sku, sku, "Sideboard", None)
            .unwrap()
            .1
            .session
            .id
    };
    let photo = |s: i64, name: &str| {
        let p = raw_photo(&a, s, name);
        a.app.previews.request_initial(&p.raw_path).unwrap();
        a.complete_render_with(
            &last_issued(&a),
            &SyntheticJpeg::rgb(2560, 1707, [200, 100, 50]),
        )
        .unwrap();
        a.review().classify(p.id, C::Main).unwrap();
        a.store().photo(p.id).unwrap()
    };
    let first = session("A");
    let a1 = photo(first, "a1");
    let a2 = photo(first, "a2");
    let second = session("B");
    let b1 = photo(second, "b1");
    let b2 = photo(second, "b2");
    let publish = a.app.publish.as_ref().unwrap();
    publish.confirm(&[first], "Publish images").unwrap();
    publish.confirm(&[second], "Publish images").unwrap();
    // Every render completes except b2's master: B stays in its prepare step (publishing).
    let step = |s: i64| {
        let id: i64 = rusqlite::Connection::open(a.app.paths.db())
            .unwrap()
            .query_row(
                "SELECT id FROM publish_items WHERE session_id=?1 ORDER BY id DESC LIMIT 1",
                [s],
                |r| r.get(0),
            )
            .unwrap();
        a.store().publish_item(id).unwrap().step
    };
    for _ in 0..12 {
        a.tick(1_100).unwrap();
        let issued: Vec<String> = a
            .fake
            .state
            .lock()
            .unwrap()
            .issued
            .iter()
            .map(|(job, _, _)| job.clone())
            .collect();
        for job in issued {
            match a.app.editing.render_completion(&job) {
                Some(entry) if entry.raw != b2.raw_path => {
                    if a.is_master_job(&job) {
                        a.complete_master_render(
                            &job,
                            &psm_testkit::fixtures::synthetic_master::SyntheticMaster::solid(
                                600,
                                400,
                                psm_testkit::fixtures::synthetic_master::PAPER,
                            ),
                        )
                        .unwrap();
                    } else {
                        a.complete_render(&job).unwrap();
                    }
                }
                _ => {}
            }
        }
    }
    assert_eq!(step(first), PublishStep::Done);
    assert!(!a.store().session(first).unwrap().dirty);
    assert_eq!(step(second), PublishStep::Prepare, "B is still publishing");
    assert!(!a.masters_dir(&a1.raw_path).exists());
    assert!(!a.masters_dir(&a2.raw_path).exists());
    let kept: Vec<_> = fs::read_dir(a.masters_dir(&b1.raw_path))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(kept.len(), 1, "the publishing session's master is kept");
    assert!(kept[0].starts_with("master-") && kept[0].ends_with(".tif"));
}

// ---------------------------------------------------------------------------------------------
// C4 — the budget

#[test]
fn c4a_below_the_minimum_files_go_in_order_until_free_space_is_back() {
    let a = TestApp::with_camera().unwrap();
    active_session(&a, "budget");
    let dup_bytes = vec![1u8; 1_000];
    let (filed, duplicate) = cable_then_ftp(&a, "DSC03003.ARW", &dup_bytes);
    a.clock.advance(H); // a fresh duplicate: 1 h, not 24
    let (export, cache) = copied_export(&a, "DSC03004", &[2u8; 1_000]);
    let now = SystemTime::now();
    let old_master = loose_master(&a, "/x/old.ARW", 1_000, now - Duration::from_secs(7200));
    let new_master = loose_master(&a, "/x/new.ARW", 1_000, now - Duration::from_secs(60));
    // 2 500 bytes short of the 20 GB default minimum.
    disk_free(&a, 20 * GB - 2_500);

    let cleared = clear_all(&a);
    assert_eq!(
        rules(&cleared),
        vec![
            ("budget_duplicate".to_string(), duplicate.clone(), 1_000),
            ("preview_copied".to_string(), export.clone(), 1_000),
            ("budget_master".to_string(), old_master.clone(), 1_000),
        ]
    );
    assert!(
        new_master.is_file(),
        "free space reached: the newer master stays"
    );
    assert!(filed.is_file() && cache.is_file());
    assert_eq!(a.app.storage.warning(), None);
}

#[test]
fn c4b_nothing_more_to_clear_warns_and_touches_no_raw_or_unpublished_file() {
    let a = TestApp::with_camera().unwrap();
    let session = active_session(&a, "full");
    let (filed, duplicate) = cable_then_ftp(&a, "DSC04004.ARW", &[3u8; 500]);
    let (export, cache) = copied_export(&a, "DSC04005", &[4u8; 500]);
    // An unpublished item's own files: its RAW, preview and output.
    a.watcher_event(&filed).unwrap();
    a.tick(1_000).unwrap();
    assert_eq!(a.store().photos_in_session(session).unwrap().len(), 1);
    let output = a
        .outputs_dir(&filed.to_string_lossy())
        .join("x-single-g1-r0-jpeg-2048-q90.jpg");
    write(&output, b"an output");
    disk_free(&a, 1_234_000_000);

    let cleared = clear_all(&a);
    assert_eq!(
        rules(&cleared),
        vec![
            ("budget_duplicate".to_string(), duplicate, 500),
            ("preview_copied".to_string(), export, 500),
        ]
    );
    let text = "Disk almost full: 1.2 GB free; nothing more can be cleared safely";
    assert_eq!(a.app.storage.warning().as_deref(), Some(text));
    let health = a.app.health.report("", false, false);
    let item = health
        .items
        .iter()
        .find(|i| i.code == DISK_ALMOST_FULL)
        .expect("health item");
    assert_eq!((item.ok, item.detail.as_str()), (false, text));
    assert_eq!(fs::read(&filed).unwrap(), vec![3u8; 500], "the RAW is kept");
    assert!(cache.is_file());
    assert_eq!(fs::read(&output).unwrap(), b"an output");
}

// ---------------------------------------------------------------------------------------------
// C5 — background, paused, batches

#[test]
fn c5a_clean_up_waits_for_a_running_render_then_clears_in_batches_of_20() {
    let a = TestApp::with_fake_document().unwrap();
    let session = a
        .items()
        .create_local("R", "R", "object", None)
        .unwrap()
        .1
        .session
        .id;
    let exports: Vec<PathBuf> = (0..25)
        .map(|i| copied_export(&a, &format!("DSC5{i:03}"), format!("export {i}").as_bytes()).0)
        .collect();
    let photo = raw_photo(&a, session, "DSC6000");
    a.app.previews.request_initial(&photo.raw_path).unwrap();
    assert_eq!(a.app.editing.renders_in_flight(), 1);

    a.clock.advance(10 * 60_000);
    let outcome = tick(&a);
    assert_eq!(outcome.paused.as_deref(), Some("render"));
    assert!(outcome.cleared.is_empty());
    assert!(
        exports.iter().all(|e| e.is_file()),
        "no deletion during the render"
    );

    a.complete_render(&last_issued(&a)).unwrap();
    assert_eq!(a.app.editing.renders_in_flight(), 0);
    let first = tick(&a);
    assert_eq!(first.cleared.len(), 20, "one batch of 20");
    assert!(tick(&a).cleared.is_empty(), "a pause between batches");
    a.clock.advance(3_000);
    let second = tick(&a);
    assert_eq!(second.cleared.len(), 5);
    assert!(exports.iter().all(|e| !e.exists()));
}
