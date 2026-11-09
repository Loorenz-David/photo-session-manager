//! P48 — the camera ownership guard (10E MC-34 R-I30…R-I33, MC-31 R-I23; M39, M40): one behaviour
//! test per criterion row through `TestApp`, the fake sidecar's scripts, the fake Capture One and
//! the fake process list. The unit rows (C1(a)–(d), C4's unit halves) live beside `name_programs`
//! and `preconditions` in `psm_integrations::camera::direct`; C5(g) and C8(a) are desktop
//! `p48_tests` (mock webview).
use psm_core::{
    Error,
    domain::{
        HealthItem,
        camera::{CameraStatus, ExposureInitiator, ExposureObserved, Link, NeedsActionReason},
        editing::{ProvisionReport, RenderPurpose},
    },
    ports::{camera::CaptureSink, clock::Clock},
};
use psm_integrations::camera::{
    direct::{SLEEP_REASON, StartReason, StartState, TakeBackReport},
    sidecar::protocol::SidecarEvent,
};
use psm_testkit::TestApp;
use serde_json::json;
use std::{
    thread,
    time::{Duration, Instant},
};

const READY_USB: &str = include_str!("../src/fixtures/sidecar_scripts/ready_usb.txt");
const READY_WIFI: &str = include_str!("../src/fixtures/sidecar_scripts/ready_wifi.txt");
const CLAIMED: &str = include_str!("../src/fixtures/sidecar_scripts/claimed.txt");
const CLAIMED_WIFI: &str = include_str!("../src/fixtures/sidecar_scripts/claimed_wifi.txt");
const TAKE_BACK: &str = include_str!("../src/fixtures/sidecar_scripts/take_back.txt");

const MODEL: &str = "ILCE-7RM5";
const CO_SENTENCE: &str =
    "Capture One is holding the camera's USB connection — Take the camera back";
const NOT_FOLLOWING: &str =
    "Capture One will not show this session's new photos until it is reachable";

fn app(script: &str) -> TestApp {
    TestApp::with_camera_script(script).unwrap()
}
/// `a.tick(0)` / 10 ms sleep until `pred` holds (10 s bound).
fn settle(a: &TestApp, pred: impl Fn() -> bool) {
    let end = Instant::now() + Duration::from_secs(10);
    while Instant::now() < end {
        a.tick(0).unwrap();
        if pred() {
            return;
        }
        thread::sleep(Duration::from_millis(10));
    }
    panic!(
        "did not settle: status={:?} events={:?} ops={:?}",
        a.camera_status(),
        a.sidecar_events(),
        ops(a)
    );
}
/// The negative read: 500 ms of `a.tick(0)` before asserting an absence.
fn settle_absence(a: &TestApp) {
    let end = Instant::now() + Duration::from_millis(500);
    while Instant::now() < end {
        a.tick(0).unwrap();
        thread::sleep(Duration::from_millis(10));
    }
}
fn st(a: &TestApp) -> CameraStatus {
    a.camera_status().unwrap()
}
fn connected(link: Link) -> CameraStatus {
    CameraStatus::Connected {
        name: MODEL.into(),
        link,
    }
}
fn held_by_capture_one() -> CameraStatus {
    CameraStatus::NeedsAction {
        reason: NeedsActionReason::HeldByOtherProgram,
        holder: Some("Capture One".into()),
        message: Some(CO_SENTENCE.into()),
    }
}
fn health_item(a: &TestApp, code: &str) -> Option<HealthItem> {
    a.app
        .health
        .report("", false, false)
        .items
        .into_iter()
        .find(|i| i.code == code)
}
fn health(a: &TestApp, code: &str) -> Option<(bool, String)> {
    health_item(a, code).map(|i| (i.ok, i.detail))
}
fn ops(a: &TestApp) -> Vec<String> {
    a.fake.state.lock().unwrap().ops.clone()
}
fn clear_ops(a: &TestApp) {
    a.fake.state.lock().unwrap().ops.clear();
}
/// Fixture rule (i): Capture One not running (and not launched), running again one poll after a
/// launch.
fn editor_stopped(a: &TestApp) {
    let mut s = a.fake.state.lock().unwrap();
    s.running = false;
    s.launched = false;
    s.running_after_polls = Some(1);
}
fn stdin(a: &TestApp) -> Vec<String> {
    a.fake_sidecar_recording()
        .stdin
        .into_iter()
        .map(|(_, line)| line)
        .collect()
}
fn usb_retries(a: &TestApp) -> usize {
    stdin(a)
        .iter()
        .filter(|l| l.split_whitespace().next() == Some("usb-retry"))
        .count()
}
fn processed(a: &TestApp, pred: impl Fn(&SidecarEvent) -> bool) -> bool {
    a.sidecar_events().iter().any(|e| pred(&e.event))
}
/// `needles` appear in `haystack` in this order (other entries may sit between them).
fn subsequence(haystack: &[String], needles: &[&str]) -> bool {
    let mut it = haystack.iter();
    needles.iter().all(|n| it.any(|h| h == n))
}
fn document_id(a: &TestApp) -> String {
    a.app
        .settings
        .effective_document_id(&a.app.paths)
        .to_string_lossy()
        .into_owned()
}
/// A session for a new local item; returns (session id, folder key).
fn new_session(a: &TestApp, sku: &str) -> (i64, String) {
    let (item, session) = a
        .items()
        .create_local(sku, "P48 item", "object", None)
        .unwrap();
    (session.session.id, item.folder_key)
}
fn line(event: &str, fields: serde_json::Value) -> String {
    let mut v = json!({"timestamp":"2026-11-06T10:00:00.100Z","source":"sdk","event":event});
    for (k, val) in fields.as_object().unwrap() {
        v[k] = val.clone();
    }
    format!("{v}\n")
}

/// Runs `f` on a scoped thread while the test thread ticks every `tick_ms` of real time and
/// advances the clock by `advance_ms` each tick; panics (after unblocking with a large advance)
/// when `f` has not returned within `bound`.
fn drive<T: Send>(
    a: &TestApp,
    advance_ms: i64,
    tick_ms: u64,
    bound: Duration,
    f: impl FnOnce() -> T + Send,
) -> T {
    thread::scope(|scope| {
        let handle = scope.spawn(f);
        let end = Instant::now() + bound;
        while !handle.is_finished() {
            if Instant::now() > end {
                a.clock.advance(120_000);
                let _ = handle.join();
                panic!("did not return within {bound:?}; ops={:?}", ops(a));
            }
            a.tick(advance_ms).unwrap();
            thread::sleep(Duration::from_millis(tick_ms));
        }
        handle.join().unwrap()
    })
}

/// C5's fixture (fixture rule iii): Capture One holds the interface over a Wi-Fi session, a session
/// is active, Capture One quits after one poll and runs one poll after a launch; ops cleared.
fn take_back_fixture(script: &str) -> (TestApp, String) {
    let a = app(script);
    a.fake_processes.set(vec!["Capture One"]);
    settle(&a, || a.app.health.has("camera_held"));
    let (s, key) = new_session(&a, "P48-TB");
    a.sessions().activate(s).unwrap();
    {
        let mut f = a.fake.state.lock().unwrap();
        f.quit_after_polls = Some(1);
        f.running_after_polls = Some(1);
    }
    clear_ops(&a);
    (a, key)
}

// ---- C1: holder naming at the status and health boundary ----

#[test]
fn c1e_status_and_health_name_the_holder_wifi_continues() {
    let a = app(CLAIMED_WIFI);
    a.fake_processes.set(vec!["Capture One"]);
    settle(&a, || st(&a) == held_by_capture_one());
    assert_eq!(health(&a, "camera_held"), Some((false, CO_SENTENCE.into())));
    a.capture().unwrap();
    settle(&a, || stdin(&a).iter().any(|l| l.starts_with("capture ")));
}

#[test]
fn c1f_cleared_when_we_hold_the_interface() {
    let a = app(CLAIMED_WIFI);
    a.fake_processes.set(vec!["Capture One"]);
    settle(&a, || st(&a) == held_by_capture_one());
    a.inject(
        "connection.ready",
        json!({"transport":"usb","model":MODEL,"store_destination":"verified"}),
    )
    .unwrap();
    settle(&a, || st(&a) == connected(Link::Cable));
    assert_eq!(health(&a, "camera_held"), None);
}

// ---- C2: the start order ----

#[test]
fn c2a_launch_waits_for_usb_ready() {
    let a = app(&format!("sleep 1500\n{READY_USB}"));
    editor_stopped(&a);
    let usb_ready = |a: &TestApp| {
        processed(a, |e| {
            matches!(e, SidecarEvent::ConnectionReady { transport, .. }
                if *transport == psm_integrations::camera::sidecar::protocol::Transport::Usb)
        })
    };
    let result = thread::scope(|scope| {
        let handle = scope.spawn(|| a.app.startup());
        let end = Instant::now() + Duration::from_secs(20);
        while !handle.is_finished() {
            assert!(Instant::now() < end, "startup did not return");
            if !usb_ready(&a) {
                assert!(
                    !ops(&a).contains(&"launch".to_string()),
                    "launched before the USB session: {:?}",
                    ops(&a)
                );
            }
            a.tick(0).unwrap();
            thread::sleep(Duration::from_millis(50));
        }
        handle.join().unwrap()
    });
    result.unwrap();
    assert!(usb_ready(&a));
    let ops = ops(&a);
    let info = ops.iter().position(|o| o == "editor_info").unwrap();
    let launch = ops.iter().position(|o| o == "launch").unwrap();
    assert!(info < launch, "{ops:?}");
    assert_eq!(
        a.app.start_order().state(),
        StartState::Satisfied(StartReason::UsbReady)
    );
}

/// C2(b)–(d): the condition arrives and the launch proceeds with no clock advance at all.
fn launches_at_once(script: &str, reason: StartReason) {
    let a = app(script);
    editor_stopped(&a);
    let started = a.clock.now_ms();
    drive(&a, 0, 10, Duration::from_secs(10), || a.app.startup()).unwrap();
    assert!(a.clock.now_ms() - started <= 100);
    assert!(ops(&a).contains(&"launch".to_string()), "{:?}", ops(&a));
    assert_eq!(a.app.start_order().state(), StartState::Satisfied(reason));
}

#[test]
fn c2b_no_camera_on_the_cable() {
    launches_at_once(
        &line("usb.initial", json!({"present":false})),
        StartReason::NoCameraOnCable,
    );
}

#[test]
fn c2c_fallback_to_wifi() {
    launches_at_once(
        &line(
            "usb.fallback_to_wifi",
            json!({"usb_failures":3,"usb_retry":"on next USB re-plug"}),
        ),
        StartReason::FallbackToWifi,
    );
}

#[test]
fn c2d_sidecar_unavailable() {
    launches_at_once(
        &format!(
            "{}exit 4\n",
            line("sdk.unavailable", json!({"error":"Sony runtime not found"}))
        ),
        StartReason::SidecarUnavailable,
    );
}

#[test]
fn c2e_the_15_s_bound_counts_from_the_first_wait() {
    let a = app("sleep 100000\n");
    editor_stopped(&a);
    thread::scope(|scope| {
        let handle = scope.spawn(|| a.app.startup());
        let end = Instant::now() + Duration::from_secs(10);
        while !a.app.start_order().is_waiting() {
            assert!(Instant::now() < end, "startup never waited");
            thread::sleep(Duration::from_millis(10));
        }
        // Real time passes; the clock does not: still waiting.
        thread::sleep(Duration::from_millis(300));
        assert!(!handle.is_finished());
        assert!(!ops(&a).contains(&"launch".to_string()));
        a.clock.advance(15_000);
        handle.join().unwrap().unwrap();
    });
    assert!(ops(&a).contains(&"launch".to_string()));
    assert_eq!(
        a.app.start_order().state(),
        StartState::Satisfied(StartReason::Timeout)
    );
}

#[test]
fn c2f_capture_one_already_running() {
    let a = app("sleep 100000\n");
    let started = a.clock.now_ms();
    drive(&a, 0, 10, Duration::from_secs(10), || a.app.startup()).unwrap();
    assert_eq!(a.clock.now_ms(), started);
    assert!(!ops(&a).contains(&"launch".to_string()));
    assert_eq!(
        a.app.start_order().state(),
        StartState::Satisfied(StartReason::AlreadyRunning)
    );
    editor_stopped(&a);
    drive(&a, 0, 10, Duration::from_secs(10), || a.app.health_check()).unwrap();
    assert_eq!(a.clock.now_ms(), started);
    assert!(ops(&a).contains(&"launch".to_string()), "{:?}", ops(&a));
}

#[test]
fn c2g_satisfied_once() {
    let a = app(&format!("sleep 1500\n{READY_USB}"));
    editor_stopped(&a);
    drive(&a, 0, 50, Duration::from_secs(20), || a.app.startup()).unwrap();
    let launches = |a: &TestApp| ops(a).iter().filter(|o| *o == "launch").count();
    assert_eq!(launches(&a), 1);
    editor_stopped(&a);
    let started = a.clock.now_ms();
    drive(&a, 0, 10, Duration::from_secs(10), || a.app.health_check()).unwrap();
    assert_eq!(a.clock.now_ms(), started);
    assert_eq!(launches(&a), 2);
}

// ---- C3: the editor-launch gate ----

/// Fixture rule (ii): the direct adapter connected over the cable (start order satisfied).
fn gate_fixture() -> TestApp {
    let a = app(READY_USB);
    settle(&a, || st(&a) == connected(Link::Cable));
    a
}

#[test]
fn c3a_gate_defers_ensure_at_launch() {
    let a = gate_fixture();
    let _g = a.launch_gate().hold();
    clear_ops(&a);
    editor_stopped(&a);
    let report = a.app.editing.ensure(&a.app.provision).unwrap();
    assert_eq!(report, ProvisionReport::deferred());
    assert_eq!(ops(&a), vec!["editor_info"]);
    assert!(!a.app.health.has("editor_not_running"));
    assert!(!a.app.health.has("editor_not_installed"));
}

#[test]
fn c3b_front_document_deferred() {
    let a = gate_fixture();
    let before = health(&a, "document_not_front");
    let _g = a.launch_gate().hold();
    clear_ops(&a);
    assert!(matches!(
        a.app.editing.bring_document_to_front(),
        Err(Error::EditorLaunchDeferred)
    ));
    assert!(ops(&a).is_empty(), "{:?}", ops(&a));
    assert_eq!(health(&a, "document_not_front"), before);
}

#[test]
fn c3c_make_document_deferred() {
    let a = gate_fixture();
    let _g = a.launch_gate().hold();
    a.fake.state.lock().unwrap().open_documents.clear();
    std::fs::remove_dir_all(document_id(&a)).unwrap();
    let report = a.app.editing.ensure(&a.app.provision).unwrap();
    assert!(report.deferred);
    assert!(
        !ops(&a).iter().any(|o| o.starts_with("make_document")),
        "{:?}",
        ops(&a)
    );
    assert!(!a.app.health.has("document_not_created"));
}

#[test]
fn c3d_open_document_deferred() {
    let a = gate_fixture();
    let _g = a.launch_gate().hold();
    a.fake.state.lock().unwrap().open_documents.clear();
    let report = a.app.editing.ensure(&a.app.provision).unwrap();
    assert!(report.deferred);
    assert!(
        !ops(&a).iter().any(|o| o.starts_with("open_document")),
        "{:?}",
        ops(&a)
    );
    assert!(!a.app.health.has("document_not_open"));
}

#[test]
fn c3e_ensure_recipe_deferred() {
    let a = gate_fixture();
    let _g = a.launch_gate().hold();
    let report = a.app.editing.ensure(&a.app.provision).unwrap();
    assert!(report.deferred);
    assert!(
        !ops(&a).iter().any(|o| o.starts_with("ensure_recipe")),
        "{:?}",
        ops(&a)
    );
    assert!(!a.app.health.has("recipe_missing"));
    assert_eq!(
        health(&a, "document"),
        Some((true, "Capture One Session.cosessiondb open".into()))
    );
}

#[test]
fn c3f_ensure_capture_style_deferred_on_the_user_lane() {
    let a = gate_fixture();
    let _g = a.launch_gate().hold();
    clear_ops(&a);
    assert!(matches!(
        a.user_lane()
            .ensure_capture_style(&document_id(&a), "PSM House"),
        Err(Error::EditorLaunchDeferred)
    ));
    assert!(ops(&a).is_empty(), "{:?}", ops(&a));
}

#[test]
fn c3g_a_health_check_while_held_never_launches() {
    let a = gate_fixture();
    let _g = a.launch_gate().hold();
    clear_ops(&a);
    editor_stopped(&a);
    let report = a.app.health_check().unwrap();
    assert_eq!(ops(&a), vec!["editor_info"]);
    assert!(
        !report
            .items
            .iter()
            .any(|i| i.code == "editor_not_running" || i.code == "document_not_open"),
        "{:?}",
        report.items
    );
}

#[test]
fn c3h_released() {
    let a = gate_fixture();
    let g = a.launch_gate().hold();
    drop(g);
    editor_stopped(&a);
    let report = a.app.editing.ensure(&a.app.provision).unwrap();
    assert!(!report.deferred);
    assert!(ops(&a).contains(&"launch".to_string()), "{:?}", ops(&a));
}

// ---- C4: the take-back's preconditions at the service boundary ----

fn blocked(a: &TestApp) -> String {
    match a.take_back().run() {
        Err(Error::TakeBackBlocked { reason }) => reason,
        other => panic!("not blocked: {other:?}"),
    }
}

#[test]
fn c4a_blocked_while_a_file_arrives() {
    let a = app(CLAIMED_WIFI);
    a.fake_processes.set(vec!["Capture One"]);
    settle(&a, || a.app.health.has("camera_held"));
    let mut f = a.ftp();
    let (reply, _data) = f.open_stor("DSC00048.ARW").unwrap();
    assert!(reply.starts_with("150"), "{reply}");
    settle(&a, || a.camera_runtime().uploads_in_progress() == 1);
    clear_ops(&a);
    assert_eq!(blocked(&a), "a file is still arriving");
    assert!(!ops(&a).contains(&"save".to_string()));
}

#[test]
fn c4b_blocked_while_a_photo_is_expected() {
    let a = gate_fixture();
    a.ledger()
        .exposure_observed(ExposureObserved {
            capture_id: None,
            at: a.clock.now_ms() - 5_000,
            initiated: ExposureInitiator::Camera,
            provisional: false,
        })
        .unwrap();
    assert_eq!(blocked(&a), "a photo is still expected");
    a.clock.advance(6_000);
    assert_eq!(blocked(&a), "Capture One is not holding the camera");
}

fn insert_job(a: &TestApp, kind: &str, status: &str) -> i64 {
    let conn = rusqlite::Connection::open(a.app.paths.db()).unwrap();
    conn.execute(
        "INSERT INTO jobs(kind,target,payload,status,created_at,updated_at) VALUES (?1,'t','{}',?2,0,0)",
        rusqlite::params![kind, status],
    )
    .unwrap();
    conn.last_insert_rowid()
}
fn set_job_status(a: &TestApp, id: i64, status: &str) {
    let conn = rusqlite::Connection::open(a.app.paths.db()).unwrap();
    conn.execute(
        "UPDATE jobs SET status=?2 WHERE id=?1",
        rusqlite::params![id, status],
    )
    .unwrap();
}

#[test]
fn c4c_blocked_while_an_export_runs() {
    let a = gate_fixture();
    let id = insert_job(&a, "render_master", "running");
    assert_eq!(blocked(&a), "an export is running");
    set_job_status(&a, id, "queued");
    assert_eq!(blocked(&a), "Capture One is not holding the camera");
}

#[test]
fn c4d_blocked_while_a_publish_runs() {
    let a = gate_fixture();
    insert_job(&a, "publish_step", "running");
    assert_eq!(blocked(&a), "a publish is running");
}

#[test]
fn c4e_blocked_while_a_preview_renders() {
    let a = gate_fixture();
    let (s, _) = new_session(&a, "P48-R");
    let photo = a.add_photo(s, "DSC04801.ARW").unwrap();
    a.fake.add_image(&photo.raw_path, &["v-4801"], false);
    a.app
        .editing
        .request_render(
            &photo.raw_path,
            RenderPurpose::Preview,
            1,
            1,
            a.clock.mono(),
            a.clock.now_ms(),
            false,
        )
        .unwrap();
    assert_eq!(a.app.editing.renders_in_flight(), 1);
    assert_eq!(blocked(&a), "a preview is rendering");
}

#[test]
fn c4f_blocked_when_capture_one_is_not_the_holder() {
    let ours = gate_fixture();
    assert_eq!(blocked(&ours), "Capture One is not holding the camera");
    let photos = app(CLAIMED);
    photos.fake_processes.set(vec!["Photos"]);
    settle(&photos, || photos.app.health.has("camera_held"));
    assert_eq!(blocked(&photos), "Capture One is not holding the camera");
}

// ---- C5: "Take the camera back" ----

#[test]
fn c5a_the_sequence_succeeds() {
    let (a, key) = take_back_fixture(TAKE_BACK);
    let tb = a.take_back();
    let r = drive(&a, 500, 50, Duration::from_secs(30), || tb.run());
    assert_eq!(
        r.unwrap(),
        TakeBackReport {
            completed_steps: 5,
            camera_held: true
        }
    );
    let folder = format!("set_captures:{}/{key}", a.capture_root().display());
    let ops = ops(&a);
    assert!(
        subsequence(
            &ops,
            &[
                "save",
                "quit",
                "editor_info",
                "launch",
                "editor_info",
                &folder
            ]
        ),
        "{ops:?}"
    );
    assert_eq!(usb_retries(&a), 1);
    settle(&a, || st(&a) == connected(Link::Cable));
    assert!(!a.launch_gate().is_held());
    assert!(!a.app.editing.ensure(&a.app.provision).unwrap().deferred);
}

#[test]
fn c5b_save_fails_nothing_else_happens() {
    let (a, _) = take_back_fixture(CLAIMED_WIFI);
    a.fake.state.lock().unwrap().fail_save = true;
    let r = a.take_back().run();
    assert!(
        matches!(r, Err(Error::TakeBackFailed { step: 2, .. })),
        "{r:?}"
    );
    assert_eq!(ops(&a), vec!["save"]);
    assert!(a.fake.state.lock().unwrap().running);
    assert!(!a.launch_gate().is_held());
}

#[test]
fn c5c_quit_never_finishes_never_killed() {
    let (a, _) = take_back_fixture(CLAIMED_WIFI);
    a.fake.state.lock().unwrap().quit_after_polls = None;
    let tb = a.take_back();
    let started = a.clock.now_ms();
    let r = drive(&a, 500, 10, Duration::from_secs(30), || tb.run());
    assert!(a.clock.now_ms() - started <= 21_500);
    assert!(
        matches!(r, Err(Error::TakeBackFailed { step: 3, .. })),
        "{r:?}"
    );
    assert!(a.fake.state.lock().unwrap().running);
    assert!(!ops(&a).contains(&"launch".to_string()));
    settle_absence(&a);
    assert_eq!(usb_retries(&a), 0);
    assert_eq!(a.fake_processes.names(), vec!["Capture One"]);
    assert!(!a.launch_gate().is_held());
}

#[test]
fn c5d_step_4_times_out_step_5_still_runs() {
    let (a, _) = take_back_fixture(&format!("{CLAIMED_WIFI}await usb-retry\nsleep 100000\n"));
    let tb = a.take_back();
    let started = a.clock.now_ms();
    // The clock moves 16 s in 500 ms steps, then stands while step 5 runs.
    let r = thread::scope(|scope| {
        let handle = scope.spawn(|| tb.run());
        let end = Instant::now() + Duration::from_secs(30);
        while !handle.is_finished() {
            assert!(Instant::now() < end, "take back did not return");
            let advance = if a.clock.now_ms() - started < 16_000 {
                500
            } else {
                0
            };
            a.tick(advance).unwrap();
            thread::sleep(Duration::from_millis(20));
        }
        handle.join().unwrap()
    });
    assert_eq!(
        r.unwrap(),
        TakeBackReport {
            completed_steps: 5,
            camera_held: false
        }
    );
    assert!(ops(&a).contains(&"launch".to_string()), "{:?}", ops(&a));
    assert_eq!(st(&a), held_by_capture_one());
}

#[test]
fn c5e_the_photographer_started_capture_one_meanwhile() {
    let (a, _) = take_back_fixture(CLAIMED_WIFI);
    a.fake.state.lock().unwrap().restart_after_quit = true;
    let r = a.take_back().run();
    match r {
        Err(Error::TakeBackFailed { step, cause }) => {
            assert_eq!(step, 4);
            assert_eq!(cause, "Capture One was started meanwhile");
        }
        other => panic!("{other:?}"),
    }
    assert!(!ops(&a).contains(&"launch".to_string()));
    settle_absence(&a);
    assert_eq!(usb_retries(&a), 0);
}

#[test]
fn c5f_capture_one_is_quit_only_on_click() {
    let a = app(CLAIMED_WIFI);
    a.fake_processes.set(vec!["Capture One"]);
    settle(&a, || a.app.health.has("camera_held"));
    for _ in 0..30 {
        a.tick(1_000).unwrap();
        thread::sleep(Duration::from_millis(10));
    }
    assert!(a.app.health.has("camera_held"));
    assert!(!ops(&a).contains(&"quit".to_string()), "{:?}", ops(&a));
}

// ---- C6: activation independent of Capture One ----

#[test]
fn c6a_activation_stores_first_capture_one_follows_or_not() {
    let a = gate_fixture();
    let (s, _) = new_session(&a, "P48-ACT");
    a.fake.state.lock().unwrap().fail_next_set_captures = true;
    a.sessions().activate(s).unwrap();
    assert_eq!(a.store().active_session_id().unwrap(), Some(s));
    assert_eq!(
        health(&a, "editor_not_following"),
        Some((false, NOT_FOLLOWING.into()))
    );
    assert!(!a.app.health.has("capture_folder_not_set"));
    assert_eq!(st(&a), connected(Link::Cable));
    a.capture().unwrap();
}

#[test]
fn c6b_retried_at_the_next_ensure() {
    let a = gate_fixture();
    let (s, key) = new_session(&a, "P48-RETRY");
    a.fake.state.lock().unwrap().fail_next_set_captures = true;
    a.sessions().activate(s).unwrap();
    assert!(a.app.health.has("editor_not_following"));
    clear_ops(&a);
    a.app.startup().unwrap();
    assert!(!a.app.health.has("editor_not_following"));
    let folder = format!("set_captures:{}/{key}", a.capture_root().display());
    assert!(ops(&a).contains(&folder), "{:?}", ops(&a));

    // The Capture One camera adapter: unchanged (refused, not stored).
    let c1 = TestApp::with_fake_document().unwrap();
    let (s, _) = new_session(&c1, "P48-C1");
    c1.fake.state.lock().unwrap().fail_next_set_captures = true;
    assert!(matches!(
        c1.sessions().activate(s),
        Err(Error::CaptureFolderNotSet)
    ));
    assert_eq!(c1.store().active_session_id().unwrap(), None);
}

// ---- C7: the idle-sleep assertion ----

#[test]
fn c7a_sleep_assertion_follows_the_usb_session() {
    let a = gate_fixture();
    assert_eq!(a.fake_sleep.held().as_deref(), Some(SLEEP_REASON));
    let before = a.sidecar_events().len();
    a.inject(
        "connection.ready",
        json!({"transport":"usb","model":MODEL,"store_destination":"verified"}),
    )
    .unwrap();
    settle(&a, || a.sidecar_events().len() > before);
    assert_eq!(a.fake_sleep.held().as_deref(), Some(SLEEP_REASON));
    assert_eq!(a.fake_sleep.holds(), 1);
    a.inject(
        "session.lost",
        json!({"transport":"usb","reason":"cable pulled","busy":"none"}),
    )
    .unwrap();
    settle(&a, || a.fake_sleep.held().is_none());

    let wifi = app(READY_WIFI);
    settle(&wifi, || st(&wifi) == connected(Link::Wireless));
    assert_eq!(wifi.fake_sleep.held(), None);
    assert_eq!(wifi.fake_sleep.holds(), 0);

    let crash = app(&format!("{READY_USB}sleep 200\nexit 1\n"));
    settle(&crash, || crash.fake_sleep.held().is_some());
    settle(&crash, || !crash.fake_sidecar_recording().exits.is_empty());
    settle(&crash, || crash.fake_sleep.held().is_none());
}
