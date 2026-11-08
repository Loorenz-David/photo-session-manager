use psm_core::{Error, events::AppEvent};
use psm_testkit::TestApp;
use std::{
    thread,
    time::{Duration, Instant},
};

fn settle(a: &TestApp, pred: impl Fn() -> bool) {
    let end = Instant::now() + Duration::from_secs(3);
    while Instant::now() < end {
        a.tick(0).unwrap();
        if pred() {
            return;
        }
        thread::sleep(Duration::from_millis(10));
    }
    panic!("fingerprint event did not settle");
}
fn unknown() -> TestApp {
    TestApp::with_camera_script(include_str!(
        "../src/fixtures/sidecar_scripts/fingerprint_unknown.txt"
    ))
    .unwrap()
}
#[test]
fn c3a_confirm_sends_exact_display_value() {
    let a = unknown();
    settle(&a, || {
        a.events.0.lock().unwrap().iter().any(|e| matches!(e, AppEvent::FingerprintPending { fingerprint, replace: false } if fingerprint == "ab:cd:ef"))
    });
    a.camera_runtime()
        .direct_adapter()
        .confirm_fingerprint("ab:cd:ef")
        .unwrap();
    settle(&a, || {
        a.fake_sidecar_recording()
            .stdin
            .iter()
            .any(|(_, line)| line.starts_with("accept-fingerprint fp=ab%3Acd%3Aef cmd_id="))
    });
    assert!(
        !a.fake_sidecar_recording()
            .stdin
            .iter()
            .any(|(_, line)| line.contains("replace=1"))
    );
}
#[test]
fn c3b_stale_confirmation_is_refused_without_sending() {
    let a = unknown();
    settle(&a, || {
        a.events
            .0
            .lock()
            .unwrap()
            .iter()
            .any(|e| matches!(e, AppEvent::FingerprintPending { .. }))
    });
    assert!(matches!(
        a.camera_runtime()
            .direct_adapter()
            .confirm_fingerprint("11:22"),
        Err(Error::FingerprintChanged)
    ));
    assert!(
        !a.fake_sidecar_recording()
            .stdin
            .iter()
            .any(|(_, line)| line.starts_with("accept-fingerprint"))
    );
    a.camera_runtime()
        .direct_adapter()
        .confirm_fingerprint("ab:cd:ef")
        .unwrap();
}
#[test]
fn c3c_mismatch_sends_replace_and_exact_value() {
    let a = TestApp::with_camera_script("{\"timestamp\":\"2026-11-06T10:00:00Z\",\"source\":\"sdk\",\"event\":\"ssh.fingerprint\",\"status\":\"MISMATCH\",\"success\":false,\"fingerprint\":\"ef:12\"}\n").unwrap();
    settle(&a, || {
        a.events.0.lock().unwrap().iter().any(|e| matches!(e, AppEvent::FingerprintPending { fingerprint, replace: true } if fingerprint == "ef:12"))
    });
    a.camera_runtime()
        .direct_adapter()
        .confirm_fingerprint("ef:12")
        .unwrap();
    settle(&a, || {
        a.fake_sidecar_recording()
            .stdin
            .iter()
            .any(|(_, line)| line.starts_with("accept-fingerprint fp=ef%3A12 replace=1 cmd_id="))
    });
}
#[test]
fn c3d_pending_events_carry_mismatch_state() {
    let a = unknown();
    settle(&a, || {
        a.events.0.lock().unwrap().iter().any(|e| matches!(e, AppEvent::FingerprintPending { fingerprint, replace: false } if fingerprint == "ab:cd:ef"))
    });
    a.inject(
        "ssh.fingerprint",
        serde_json::json!({"status":"MISMATCH","success":false,"fingerprint":"ef:12"}),
    )
    .unwrap();
    settle(&a, || {
        a.events.0.lock().unwrap().iter().any(|e| matches!(e, AppEvent::FingerprintPending { fingerprint, replace: true } if fingerprint == "ef:12"))
    });
}
