use psm_core::{
    domain::camera::{CameraStatus, Link},
    services::{camera::CameraService, health::HealthService},
};
use psm_testkit::TestApp;
use psm_testkit::fake_direct_camera::FakeDirectProvider;
use serde_json::json;
use std::{
    thread,
    time::{Duration, Instant},
};

fn alive(pid: u32) -> bool {
    unsafe { libc::kill(pid as i32, 0) == 0 }
}
fn settle(a: &TestApp, pred: impl Fn() -> bool) {
    let end = Instant::now() + Duration::from_secs(3);
    while Instant::now() < end {
        a.tick(0).unwrap();
        if pred() {
            return;
        }
        thread::sleep(Duration::from_millis(10));
    }
    panic!("camera result did not settle");
}
fn settle_absence(a: &TestApp) {
    let end = Instant::now() + Duration::from_millis(500);
    while Instant::now() < end {
        a.tick(0).unwrap();
        thread::sleep(Duration::from_millis(10));
    }
}
fn one(a: &TestApp) -> u32 {
    settle(a, || a.fake_sidecar_recording().pids.len() == 1);
    a.fake_sidecar_recording().pids[0]
}

#[test]
fn c1a_runtime_save_keeps_child() {
    let mut a = TestApp::with_camera().unwrap();
    let p = one(&a);
    a.settings_set(&json!({"sku_prefix":"PX"})).unwrap();
    settle_absence(&a);
    let r = a.fake_sidecar_recording();
    assert_eq!(r.pids, vec![p]);
    assert!(alive(p));
    assert!(!r.stdin.iter().any(|(_, line)| line == "quit"));
}
#[test]
fn c1b_theme_save_keeps_child() {
    let mut a = TestApp::with_camera().unwrap();
    let p = one(&a);
    a.settings_set(&json!({"theme":"dark"})).unwrap();
    settle_absence(&a);
    let r = a.fake_sidecar_recording();
    assert_eq!(r.pids, vec![p]);
    assert!(!r.stdin.iter().any(|(_, line)| line == "quit"));
}
#[test]
fn c1c_restart_reaps_old_child() {
    let mut a = TestApp::with_camera().unwrap();
    let p = one(&a);
    a.kill_and_restart().unwrap();
    settle(&a, || {
        a.fake_sidecar_recording().pids.len() == 2 && !alive(p)
    });
    let r = a.fake_sidecar_recording();
    assert_eq!(
        r.stdin.iter().rfind(|(pid, _)| *pid == p).unwrap().1,
        "quit"
    );
    assert!(r.exits.contains(&(p, 0)));
    assert!(alive(r.pids[1]));
}
#[test]
fn c2a_shutdown_is_graceful_and_terminal() {
    let a = TestApp::with_camera().unwrap();
    let p = one(&a);
    let t = Instant::now();
    a.camera_runtime().shutdown();
    assert!(t.elapsed() < Duration::from_secs(3));
    settle(&a, || !alive(p));
    let r = a.fake_sidecar_recording();
    assert_eq!(
        r.stdin.iter().rfind(|(pid, _)| *pid == p).unwrap().1,
        "quit"
    );
    assert_eq!(r.exits, vec![(p, 0)]);
    a.tick(60_000).unwrap();
    settle_absence(&a);
    assert_eq!(a.fake_sidecar_recording().pids, vec![p]);
    let t = Instant::now();
    a.camera_runtime().shutdown();
    assert!(t.elapsed() < Duration::from_millis(100));
}
#[test]
fn c2b_shutdown_kills_ignoring_child_after_grace() {
    let a = TestApp::with_camera_behaviour(json!({"ignore_quit":true})).unwrap();
    let p = one(&a);
    let t = Instant::now();
    a.camera_runtime().shutdown();
    assert!(!alive(p));
    assert!(a.fake_sidecar_recording().exits.is_empty());
    assert!(t.elapsed() >= Duration::from_millis(3000));
    assert!(t.elapsed() < Duration::from_secs(4));
}
#[test]
fn c3a_restarts_at_bounded_offsets() {
    let a = TestApp::with_camera_behaviour(json!({"exit_after_ms":0})).unwrap();
    let offsets = [0, 1000, 3000, 7000, 15000, 31000, 61000];
    let mut now = 0;
    for (index, target) in offsets.into_iter().enumerate() {
        if index > 0 {
            while now < target - 100 {
                a.tick(100).unwrap();
                now += 100;
            }
            settle_absence(&a);
            assert_eq!(a.fake_sidecar_recording().pids.len(), index);
            a.tick(100).unwrap();
            now += 100;
        }
        settle(&a, || a.fake_sidecar_recording().pids.len() == index + 1);
        let pid = a.fake_sidecar_recording().pids[index];
        settle(&a, || !alive(pid));
    }
}
#[test]
fn c3b_health_clears_on_line_without_resetting_backoff() {
    let a = TestApp::with_camera_behaviour(json!({"exit_after_ms":0})).unwrap();
    for (n, delta) in [(1, 0), (2, 1000), (3, 2000)] {
        if delta > 0 {
            a.tick(delta).unwrap();
        }
        settle(&a, || a.fake_sidecar_recording().pids.len() == n);
        let pid = a.fake_sidecar_recording().pids[n - 1];
        settle(&a, || !alive(pid));
    }
    assert!(a.app.health.has("camera_sidecar_unavailable"));
    a.set_fake_sidecar_behaviour(json!({})).unwrap();
    a.tick(4000).unwrap();
    settle(&a, || {
        a.fake_sidecar_recording().pids.len() == 4
            && !a.app.health.has("camera_sidecar_unavailable")
    });
    let p = a.fake_sidecar_recording().pids[3];
    unsafe {
        libc::kill(p as i32, libc::SIGKILL);
    }
    settle(&a, || !alive(p));
    a.tick(7_900).unwrap();
    settle_absence(&a);
    assert_eq!(a.fake_sidecar_recording().pids.len(), 4);
    a.tick(100).unwrap();
    settle(&a, || a.fake_sidecar_recording().pids.len() == 5);
}
#[test]
fn c4a_credentials_are_only_on_first_stdin_line() {
    let a = TestApp::with_camera().unwrap();
    let p = one(&a);
    settle(&a, || {
        a.fake_sidecar_recording()
            .stdin
            .iter()
            .any(|(pid, _)| *pid == p)
    });
    let r = a.fake_sidecar_recording();
    assert_eq!(
        r.stdin.iter().find(|(pid, _)| *pid == p).unwrap().1,
        "credentials user=u%20name password=p%2Fw"
    );
    for arg in &r.argv[0] {
        for secret in ["u name", "p/w", "u%20name", "p%2Fw"] {
            assert!(!arg.contains(secret));
        }
    }
    for value in r.env[0].values() {
        for secret in ["u name", "p/w", "u%20name", "p%2Fw"] {
            assert!(!value.contains(secret));
        }
    }
    assert_eq!(
        r.env[0]["CRLAB_FINGERPRINT_DIR"],
        a.app
            .paths
            .root
            .join("camera/fingerprints")
            .to_string_lossy()
    );
    assert_eq!(&r.argv[0][1..3], &["auto", "--save-dir"]);
    assert!(r.argv[0][3].starts_with(a.dir.path().to_str().unwrap()));
    assert!(r.argv[0][3].contains(a.camera_runtime().run_id()));
}
#[test]
fn c4b_stderr_is_redacted() {
    let a = TestApp::with_camera_behaviour(json!({"stderr":"ssh password p/w refused"})).unwrap();
    settle(&a, || !a.camera_runtime().helper_log().is_empty());
    assert_eq!(
        a.camera_runtime().helper_log(),
        vec!["ssh password [REDACTED] refused"]
    );
}
#[test]
fn c6a_direct_provider_ignores_editor_health() {
    let p = FakeDirectProvider;
    let h = HealthService::new();
    h.set("capture_folder_not_set", "bad folder");
    let service = CameraService::new(&p, &h);
    assert_eq!(
        service.status().unwrap(),
        CameraStatus::Connected {
            name: "X".into(),
            link: Link::Wireless
        }
    );
    assert!(!service.capture().unwrap().0.is_empty());
    assert!(
        h.report("", false, false)
            .items
            .iter()
            .any(|i| i.code == "camera" && i.ok && i.detail == "X · wireless")
    );
}
#[test]
fn c7a_camera_health_copied_on_rebuild() {
    // From P46 a support folder selects the direct adapter, which re-publishes its own codes;
    // P42's copy of the Capture One camera's health is shown without one.
    let mut a = TestApp::with_fake_document().unwrap();
    a.fake.state.lock().unwrap().camera = None;
    a.app.handles().health_check().unwrap();
    let old = a
        .app
        .health
        .report("", false, false)
        .items
        .into_iter()
        .find(|i| i.code == "camera_disconnected")
        .unwrap()
        .detail;
    a.settings_set(&json!({"sku_prefix":"PX"})).unwrap();
    assert_eq!(
        a.app
            .health
            .report("", false, false)
            .items
            .into_iter()
            .find(|i| i.code == "camera_disconnected")
            .unwrap()
            .detail,
        old
    );
    a.fake.state.lock().unwrap().camera = Some("Sony ZV-E10".into());
    a.app.handles().health_check().unwrap();
    a.settings_set(&json!({"sku_prefix":"QX"})).unwrap();
    assert!(
        a.app
            .health
            .report("", false, false)
            .items
            .iter()
            .any(|i| i.code == "camera" && i.ok)
    );
}
#[test]
fn c7b_runtime_identity_survives_rebuild_and_reopen() {
    let mut a = TestApp::with_camera().unwrap();
    let p = one(&a);
    let id = a.camera_runtime().run_id().to_string();
    a.settings_set(&json!({"sku_prefix":"PX"})).unwrap();
    assert_eq!(a.app.handles().camera_runtime.run_id(), id);
    a.app.reopen().unwrap();
    settle_absence(&a);
    assert_eq!(a.app.handles().camera_runtime.run_id(), id);
    assert_eq!(a.fake_sidecar_recording().pids, vec![p]);
}
