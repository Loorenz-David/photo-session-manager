//! P56 (P52 F12): the endless Capture One export loop. Capture One's "add suffix" output naming
//! (`DSC04979 14.tif` written, `DSC04979.tif` reported) and its own rewrites of the settings sidecar
//! after processing made the app miss finished renders and take its own exports for edits. Rows
//! C3(a)–(d) at the boundary the fake Capture One records: the `process` calls it received.
//! (C1/C2, the helper's transport policy, are C++ tests in `native/camera-sidecar/tests`.)

use psm_core::{
    domain::photo::{Classification as C, NewPhoto, Photo, PhotoSource},
    mechanisms::hash::sha256_file,
    ports::clock::Clock,
};
use psm_infra::disk::FreeSpace;
use psm_testkit::{TestApp, fixtures::synthetic_jpeg::SyntheticJpeg};
use std::{
    fs,
    path::Path,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

fn app() -> TestApp {
    let a = TestApp::with_fake_document().unwrap();
    a.clock.advance(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64,
    );
    {
        // Capture One as observed on 2026-11-08: add-suffix naming, sidecar rewritten by processing.
        let mut s = a.fake.state.lock().unwrap();
        s.add_suffix = true;
        s.touch_settings_on_process = true;
    }
    a
}

fn now(a: &TestApp) -> SystemTime {
    UNIX_EPOCH + Duration::from_millis(a.clock.now_ms() as u64)
}

fn session(a: &TestApp, sku: &str) -> i64 {
    a.items()
        .create_local(sku, "Oak", "Sideboard", None)
        .unwrap()
        .1
        .session
        .id
}

/// A RAW on disk known to the fake document, with its sidecar as Capture One wrote it at import
/// and an export of it from earlier in the day already in `Output/` (as `DSC04767.jpg` was).
fn photo(a: &TestApp, session_id: i64, name: &str) -> Photo {
    let dir = a.dir.path().join("Capture/raws");
    fs::create_dir_all(&dir).unwrap();
    let raw = dir.join(format!("{name}.ARW"));
    fs::write(&raw, format!("raw bytes of {name}")).unwrap();
    let photo = a
        .store()
        .photo_insert_or_get(NewPhoto {
            session_id,
            raw_path: raw.to_string_lossy().into_owned(),
            raw_hash: Some(sha256_file(&raw).unwrap()),
            captured_at: Some(a.clock.now_ms()),
            source: PhotoSource::Capture,
            created_at: a.clock.now_ms(),
        })
        .unwrap();
    a.store().upsert_raw_state(&photo.raw_path, None).unwrap();
    a.fake
        .add_image(&photo.raw_path, &[&variant(&photo)], false);
    let morning = now(a) - Duration::from_secs(6 * 3600);
    a.fake.edit_settings(&photo.raw_path, 0, morning).unwrap();
    let output = a.dir.path().join("Output");
    fs::create_dir_all(&output).unwrap();
    for ext in ["jpg", "tif"] {
        let old = output.join(format!("{name}.{ext}"));
        fs::write(&old, b"this morning's export").unwrap();
        fs::File::options()
            .write(true)
            .open(&old)
            .unwrap()
            .set_times(fs::FileTimes::new().set_modified(morning))
            .unwrap();
    }
    photo
}

fn variant(photo: &Photo) -> String {
    format!("v{}", photo.id)
}

/// The `process` calls the fake Capture One received for `photo` with `recipe`.
fn processes(a: &TestApp, photo: &Photo, recipe: &str) -> usize {
    a.fake
        .state
        .lock()
        .unwrap()
        .issued
        .iter()
        .filter(|(_, v, r)| *v == variant(photo) && r == recipe)
        .count()
}

fn last_job(a: &TestApp) -> String {
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

fn health_detail(a: &TestApp, code: &str) -> Option<String> {
    a.app
        .health
        .report("", true, true)
        .items
        .into_iter()
        .find(|i| i.code == code && !i.ok)
        .map(|i| i.detail)
}

/// The photographer comes back to Session Manager every minute for an hour; Capture One saves the
/// document (rewriting every sidecar, no adjustment changed) every ten minutes.
fn an_hour_of_use(a: &TestApp, photos: &[&Photo]) {
    for minute in 1..=60 {
        a.tick(60_000).unwrap();
        if minute % 10 == 0 {
            for photo in photos {
                a.fake.rewrite_settings(&photo.raw_path, now(a)).unwrap();
            }
        }
        a.focus().unwrap();
    }
}

fn render_rows(a: &TestApp, kind: &str, status: &str) -> i64 {
    rusqlite::Connection::open(a.app.paths.db())
        .unwrap()
        .query_row(
            "SELECT COUNT(*) FROM jobs WHERE kind=?1 AND status=?2",
            [kind, status],
            |r| r.get(0),
        )
        .unwrap()
}

#[test]
fn c3a_suffixed_export_is_recorded_once_and_never_ordered_again() {
    let a = app();
    let s = session(&a, "K");
    let p = photo(&a, s, "DSC04767");
    a.app.previews.request_initial(&p.raw_path).unwrap();
    assert_eq!(processes(&a, &p, "PSM Preview"), 1);
    // Capture One writes `DSC04767 1.jpg`, reports `DSC04767.jpg` (this morning's file), then
    // rewrites the sidecar a second later.
    let bytes = FakeBytes::of("first render");
    let written = a.complete_render_with(&last_job(&a), &bytes.0).unwrap();
    assert_eq!(
        written.file_name().unwrap().to_string_lossy(),
        "DSC04767 1.jpg"
    );
    // Recorded complete once: the preview is that file, generation 1 rendered, the job done.
    let state = a.store().preview_state(&p.raw_path).unwrap().unwrap();
    assert_eq!((state.adjusted_gen, state.rendered_gen), (1, 1));
    assert_eq!(
        fs::read(state.adjusted_preview_path.unwrap()).unwrap(),
        bytes.0
    );
    assert_eq!(render_rows(&a, "render_preview", "done"), 1);
    assert_eq!(render_rows(&a, "render_preview", "running"), 0);
    // An hour of focus changes and sidecar rewrites: never another `process` for the photo.
    an_hour_of_use(&a, &[&p]);
    assert_eq!(processes(&a, &p, "PSM Preview"), 1);
    assert_eq!(
        a.store()
            .preview_state(&p.raw_path)
            .unwrap()
            .unwrap()
            .adjusted_gen,
        1
    );
    // Control: a real edit made while Session Manager did not see the hook still renders once.
    a.clock.advance(60_000);
    a.fake.edit_settings(&p.raw_path, 25, now(&a)).unwrap();
    a.focus().unwrap();
    a.focus().unwrap();
    assert_eq!(processes(&a, &p, "PSM Preview"), 2);
}

#[test]
fn c3b_render_that_never_produces_output_stops_after_two_attempts() {
    let a = app();
    let s = session(&a, "K");
    let p = photo(&a, s, "DSC04800");
    a.app.previews.request_initial(&p.raw_path).unwrap();
    assert_eq!(processes(&a, &p, "PSM Preview"), 1);
    // No processing-done ever arrives. MC-16: re-issued once after 60 s, failed after 60 more.
    a.tick(61_000).unwrap();
    assert_eq!(processes(&a, &p, "PSM Preview"), 2);
    assert_eq!(health_detail(&a, "preview_not_rendered"), None);
    a.tick(61_000).unwrap();
    assert_eq!(
        health_detail(&a, "preview_not_rendered").as_deref(),
        Some("Capture One produced no preview for DSC04800.ARW")
    );
    an_hour_of_use(&a, &[&p]);
    assert_eq!(processes(&a, &p, "PSM Preview"), 2);
    assert_eq!(render_rows(&a, "render_preview", "running"), 0);
}

#[test]
fn c3c_restart_with_every_preview_current_orders_nothing() {
    let mut a = app();
    let s = session(&a, "K");
    let photos = [photo(&a, s, "DSC04766"), photo(&a, s, "DSC04768")];
    for p in &photos {
        a.app.previews.request_initial(&p.raw_path).unwrap();
        a.complete_render(&last_job(&a)).unwrap();
    }
    let before = a.fake.state.lock().unwrap().issued.len();
    // A render row the previous process left `running` (the owner's database had 274), and
    // Capture One saving the document while Session Manager was closed.
    a.store()
        .new_render_job(&photos[0].raw_path, a.clock.now_ms())
        .unwrap();
    a.clock.advance(5 * 60_000);
    for p in &photos {
        a.fake.rewrite_settings(&p.raw_path, now(&a)).unwrap();
    }
    a.restart_with_startup().unwrap();
    for _ in 0..5 {
        a.tick(1_000).unwrap();
    }
    a.focus().unwrap();
    std::thread::sleep(Duration::from_millis(500));
    a.tick(1_000).unwrap();
    assert_eq!(a.fake.state.lock().unwrap().issued.len(), before);
    assert_eq!(render_rows(&a, "render_preview", "running"), 0);
    for p in &photos {
        let state = a.store().preview_state(&p.raw_path).unwrap().unwrap();
        assert_eq!((state.adjusted_gen, state.rendered_gen), (1, 1));
    }
}

#[test]
fn c3d_master_is_rendered_once_and_not_at_all_on_a_nearly_full_disk() {
    let a = app();
    let s = session(&a, "K");
    let p = photo(&a, s, "DSC04979");
    // The preview gives the composition its frame.
    a.app.previews.request_initial(&p.raw_path).unwrap();
    a.complete_render_with(&last_job(&a), &SyntheticJpeg::rgb(600, 400, [200, 100, 50]))
        .unwrap();
    a.review().classify(p.id, C::Main).unwrap();
    a.app.export.run(s).unwrap();
    assert_eq!(processes(&a, &p, "PSM Master"), 1);
    // Capture One writes `DSC04979 1.tif` and reports this morning's `DSC04979.tif`.
    let master = a.complete_all_renders().unwrap();
    assert_eq!(master.len(), 1);
    for _ in 0..10 {
        a.tick(60_000).unwrap();
    }
    let export = a.export_root().join("K");
    assert!(export.join("K_01_main.jpg").is_file());
    // Asked again: the master is not ordered a second time.
    a.app.export.run(s).unwrap();
    for _ in 0..5 {
        a.tick(60_000).unwrap();
    }
    assert_eq!(processes(&a, &p, "PSM Master"), 1);

    // 3 GB free: no master render is ordered, the health item names the disk, nothing deleted.
    a.app.disk_space().set(Arc::new(|_: &Path| {
        Ok(FreeSpace {
            available_bytes: 3_000_000_000,
            disk: "the disk \u{201c}Photos\u{201d}".into(),
        })
    }));
    let s2 = session(&a, "L");
    let q = photo(&a, s2, "DSC04980");
    a.app.previews.request_initial(&q.raw_path).unwrap();
    a.complete_render_with(&last_job(&a), &SyntheticJpeg::rgb(600, 400, [200, 100, 50]))
        .unwrap();
    a.review().classify(q.id, C::Main).unwrap();
    assert!(a.app.export.run(s2).is_err());
    a.tick(60_000).unwrap();
    assert_eq!(processes(&a, &q, "PSM Master"), 0);
    assert_eq!(
        health_detail(&a, "disk_space_low").as_deref(),
        Some(
            "Only 3.0 GB free on the disk \u{201c}Photos\u{201d}. Full-size renders for export are paused until 5 GB are free."
        )
    );
    // The photographer's export and earlier files stay (the app's own master copy in `Output/` is
    // released after caching, as before this phase).
    assert!(export.join("K_01_main.jpg").is_file());
    assert!(a.dir.path().join("Output/DSC04979.tif").is_file());
    assert!(a.dir.path().join("Output/DSC04980.tif").is_file());
}

/// Distinct bytes per render, so the recorded preview can be traced to the file it came from.
struct FakeBytes(Vec<u8>);
impl FakeBytes {
    fn of(label: &str) -> Self {
        let mut bytes = vec![0xFF, 0xD8, 0xFF, 0xE0];
        bytes.extend_from_slice(label.as_bytes());
        bytes.extend_from_slice(&[0xFF, 0xD9]);
        Self(bytes)
    }
}
