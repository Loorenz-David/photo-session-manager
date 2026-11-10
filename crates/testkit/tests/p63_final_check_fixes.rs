//! P63 — fixes from the final hands-on check: no script reaches Capture One while the take-back
//! keeps it closed (#22), a missing session is reported instead of recreated (#27), the camera
//! helper's staging follows the session (#28), the style list for Settings (owner 2026-11-10).
//! One test per criterion row, at the row's boundary.
use psm_core::{
    domain::camera::{CameraStatus, Link, NeedsActionReason},
    domain::editing::EditorStyles,
    domain::photo::{NewPhoto, PhotoSource},
    ports::{camera::ArrivalFollow, clock::Clock},
    services::editing_startup::DOCUMENT_MISSING,
};
use psm_integrations::camera::direct::TakeBackReport;
use psm_testkit::TestApp;
use std::{
    fs,
    path::{Path, PathBuf},
    thread,
    time::{Duration, Instant},
};

const CLAIMED_WIFI: &str = include_str!("../src/fixtures/sidecar_scripts/claimed_wifi.txt");
const USB_READY: &str = "{\"timestamp\":\"2026-11-06T10:00:02.000Z\",\"event\":\"connection.ready\",\"transport\":\"usb\",\"model\":\"ILCE-7RM5\",\"store_destination\":\"verified\"}\n";

fn settle(a: &TestApp, what: &str, until: impl Fn() -> bool) {
    let end = Instant::now() + Duration::from_secs(20);
    while Instant::now() < end {
        a.tick(100).unwrap();
        if until() {
            return;
        }
        thread::sleep(Duration::from_millis(10));
    }
    panic!("{what} did not happen: ops={:?}", ops(a));
}
fn ops(a: &TestApp) -> Vec<String> {
    a.fake.state.lock().unwrap().ops.clone()
}
fn on_cable(a: &TestApp) -> bool {
    matches!(
        a.camera_status(),
        Ok(CameraStatus::Connected {
            link: Link::Cable,
            ..
        })
    )
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
/// Every path under `dir`, recursively.
fn tree(dir: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            found.push(path.clone());
            if path.is_dir() {
                found.extend(tree(&path));
            }
        }
    }
    found
}

/// C1(a): "Take the camera back" past its quit step while the follower waits for a variant (its
/// poll is due every second) and a preview is asked for. Every Apple event reaching the quit
/// Capture One would relaunch it (as on the dev Mac). Between the quit and the relaunch nothing
/// but step 5's own `editor_info` reaches the fake, the one launch comes after our USB session,
/// and the take-back reports success.
#[test]
fn c1a_no_script_reaches_capture_one_while_the_take_back_keeps_it_closed() {
    let script = format!("{CLAIMED_WIFI}await usb-retry\nsleep 800\n{USB_READY}");
    let a = TestApp::with_camera_script(&script).unwrap();
    a.fake_processes.set(vec!["Capture One"]);
    {
        let mut f = a.fake.state.lock().unwrap();
        f.quit_after_polls = Some(1);
        f.running_after_polls = Some(1);
    }
    settle(&a, "the claim", || a.app.health.has("camera_held"));
    let (_, session) = a
        .items()
        .create_local("P63-C1", "Take back", "object", None)
        .unwrap();
    let session = session.session.id;
    a.sessions().activate(session).unwrap();
    // The follower waits for this photo's variant (Capture One has not listed it yet).
    let folder = a.capture_root().join("P63-C1");
    fs::create_dir_all(&folder).unwrap();
    let waiting = folder.join("DSC06401.ARW");
    fs::write(&waiting, b"waiting").unwrap();
    let waiting = waiting.to_string_lossy().into_owned();
    a.app.follow.photo_arrived(session, &waiting, &folder);
    // A photo whose preview is asked for once Capture One has quit.
    let rendered = a.dir.path().join("DSC06402.ARW");
    fs::write(&rendered, b"rendered").unwrap();
    let rendered = fs::canonicalize(&rendered)
        .unwrap()
        .to_string_lossy()
        .into_owned();
    a.store()
        .photo_insert_or_get(NewPhoto {
            session_id: session,
            raw_path: rendered.clone(),
            raw_hash: None,
            captured_at: Some(a.clock.now_ms()),
            source: PhotoSource::Capture,
            created_at: a.clock.now_ms(),
        })
        .unwrap();
    a.store().upsert_raw_state(&rendered, None).unwrap();
    a.store().ensure_initial_generation(&rendered).unwrap();
    a.fake.add_image(&rendered, &["v-DSC06402.ARW"], true);
    {
        let mut f = a.fake.state.lock().unwrap();
        f.apple_events_launch = true;
        f.ops.clear();
    }
    let take_back = a.take_back();
    let mut launched_early = false;
    let mut asked = false;
    let result = thread::scope(|scope| {
        let handle = scope.spawn(|| take_back.run());
        let end = Instant::now() + Duration::from_secs(30);
        while !handle.is_finished() {
            assert!(Instant::now() < end, "take-back did not finish");
            let now = ops(&a);
            if now.iter().any(|o| o == "launch") && !on_cable(&a) {
                launched_early = true;
            }
            if !asked && now.iter().any(|o| o == "quit") && !a.fake.state.lock().unwrap().running {
                a.app.previews.request_initial(&rendered).unwrap();
                asked = true;
            }
            a.tick(100).unwrap();
            thread::sleep(Duration::from_millis(10));
        }
        handle.join().unwrap()
    });
    assert!(asked, "the take-back never got past its quit step");
    assert_eq!(
        result.unwrap(),
        TakeBackReport {
            completed_steps: 5,
            camera_held: true
        }
    );
    // The follower was due all along: once Capture One is back it asks again.
    settle(&a, "the follower's next poll", || {
        ops(&a)
            .iter()
            .any(|o| o == &format!("find_image:{waiting}"))
    });
    let ops = ops(&a);
    assert!(!launched_early, "launched before our USB session: {ops:?}");
    assert_eq!(ops.iter().filter(|o| *o == "launch").count(), 1, "{ops:?}");
    let quit = ops.iter().position(|o| o == "quit").unwrap();
    let launch = ops.iter().position(|o| o == "launch").unwrap();
    assert_eq!(
        ops[quit + 1..launch],
        ["editor_info".to_string()],
        "scripts reached Capture One after it quit: {ops:?}"
    );
}

/// C2(a): the session chosen in Settings is gone (folder deleted) when the app starts. No folder
/// appears under its path after an FTP upload and a session activation; the upload is answered
/// 451 (the camera keeps it queued); the header and Health name the missing session.
#[test]
fn c2a_a_missing_session_is_reported_never_recreated() {
    let mut a = TestApp::with_camera().unwrap();
    let document = a.app.settings.effective_document_id(&a.app.paths);
    fs::remove_dir_all(&document).unwrap();
    a.kill_and_restart().unwrap();
    settle(&a, "the missing session named", || {
        health_detail(&a, "document_missing").is_some()
    });
    assert_eq!(
        health_detail(&a, "document_missing").as_deref(),
        Some(DOCUMENT_MISSING)
    );
    assert_eq!(
        DOCUMENT_MISSING,
        "The Capture One session in Settings is missing — choose one in Settings"
    );
    match a.camera_status().unwrap() {
        CameraStatus::NeedsAction { reason, .. } => {
            assert_eq!(reason, NeedsActionReason::DocumentMissing);
            assert_eq!(
                psm_core::mechanisms::camera_status::status_message(&reason, None),
                DOCUMENT_MISSING
            );
        }
        other => panic!("header: {other:?}"),
    }
    let mut ftp = a.ftp();
    let reply = ftp.stor("DSC07001.ARW", b"camera bytes").unwrap();
    assert!(reply.starts_with("451"), "{reply}");
    let (_, session) = a
        .items()
        .create_local("P63-C2", "Missing session", "object", None)
        .unwrap();
    let _ = a.sessions().activate(session.session.id);
    for _ in 0..5 {
        a.tick(1_000).unwrap();
    }
    assert!(
        !document.exists(),
        "created under the missing session: {:?}",
        tree(&document)
    );
    assert!(
        !a.fake
            .state
            .lock()
            .unwrap()
            .ops
            .iter()
            .any(|o| o.starts_with("make_document")),
        "the session was recreated"
    );
}

fn second_document(a: &TestApp) -> PathBuf {
    let document = a.dir.path().join("Second Session");
    fs::create_dir_all(document.join("Capture")).unwrap();
    fs::write(document.join("Second Session.cosessiondb"), b"").unwrap();
    document
}
fn save_dirs(a: &TestApp) -> Vec<String> {
    a.fake_sidecar_recording()
        .argv
        .iter()
        .filter_map(|argv| {
            argv.iter()
                .position(|arg| arg == "--save-dir")
                .map(|at| argv[at + 1].clone())
        })
        .collect()
}

/// C3(a): the session is changed in Settings while the helper runs; an upload is in progress, so
/// nothing happens until it ends; then the helper is started again with `--save-dir` under the new
/// session's capture root.
#[test]
fn c3a_the_helpers_staging_follows_the_session() {
    let mut a = TestApp::with_camera().unwrap();
    settle(&a, "the first spawn", || save_dirs(&a).len() == 1);
    let first = save_dirs(&a)[0].clone();
    let mut ftp = a.ftp();
    let (reply, data) = ftp.open_stor("DSC07101.ARW").unwrap();
    assert!(reply.starts_with("150"), "{reply}");
    let data = data.unwrap();
    settle(&a, "the upload under way", || {
        a.app.camera_runtime.uploads_in_progress() > 0
    });
    let document = second_document(&a);
    a.settings_set(&serde_json::json!({ "document_id": document.to_string_lossy() }))
        .unwrap();
    let end = Instant::now() + Duration::from_millis(800);
    while Instant::now() < end {
        a.tick(100).unwrap();
        thread::sleep(Duration::from_millis(20));
    }
    assert_eq!(save_dirs(&a), vec![first.clone()], "restarted mid-upload");
    let _ = ftp.finish_stor(data);
    settle(&a, "the restart", || save_dirs(&a).len() == 2);
    let expected = fs::canonicalize(document.join("Capture"))
        .unwrap()
        .join(".incoming")
        .join(format!("sdk-{}", a.app.camera_runtime.run_id()));
    let got = PathBuf::from(&save_dirs(&a)[1]);
    assert_eq!(
        fs::canonicalize(&got).unwrap_or(got.clone()),
        expected,
        "{:?}",
        save_dirs(&a)
    );
}

/// C6(b), the backend half: the styles Capture One lists, the photographer's own first; none
/// asked of a Capture One that is not running.
#[test]
fn c6b_capture_ones_styles_for_settings() {
    let a = TestApp::with_fake_document().unwrap();
    {
        let mut f = a.fake.state.lock().unwrap();
        f.styles = vec!["Vivid".into(), "test-style".into()];
        f.user_styles = vec!["test-style".into()];
    }
    assert_eq!(
        a.app.editing.styles().unwrap(),
        EditorStyles {
            user: vec!["test-style".into()],
            built_in: vec!["Vivid".into()],
        }
    );
    {
        let mut f = a.fake.state.lock().unwrap();
        f.running = false;
        f.ops.clear();
    }
    assert!(a.app.editing.styles().is_err());
    assert!(
        ops(&a).is_empty(),
        "asked a closed Capture One: {:?}",
        ops(&a)
    );
}
