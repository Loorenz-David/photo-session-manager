//! P54 camera cable and take-back behaviour at the service and helper-event boundaries.
use psm_core::{
    domain::{
        camera::{CameraStatus, Link},
        editing::RenderPurpose,
    },
    ports::{clock::Clock, store::Store},
};
use psm_testkit::TestApp;
use serde_json::json;
use std::{
    thread,
    time::{Duration, Instant},
};

const READY_USB: &str = include_str!("../src/fixtures/sidecar_scripts/ready_usb.txt");
const READY_WIFI: &str = include_str!("../src/fixtures/sidecar_scripts/ready_wifi.txt");
const CLAIMED_WIFI: &str = include_str!("../src/fixtures/sidecar_scripts/claimed_wifi.txt");
const TAKE_BACK: &str = include_str!("../src/fixtures/sidecar_scripts/take_back.txt");

fn settle(a: &TestApp, until: impl Fn() -> bool) {
    let end = Instant::now() + Duration::from_secs(10);
    while Instant::now() < end {
        a.tick(0).unwrap();
        if until() {
            return;
        }
        thread::sleep(Duration::from_millis(10));
    }
    panic!(
        "camera did not settle: {:?}, events={:?}, log={:?}, rec={:?}",
        a.camera_status(),
        a.sidecar_events(),
        a.camera_runtime().helper_log(),
        a.fake_sidecar_recording().pids
    );
}
fn ops(a: &TestApp) -> Vec<String> {
    a.fake.state.lock().unwrap().ops.clone()
}
fn stopped(a: &TestApp) {
    let mut state = a.fake.state.lock().unwrap();
    state.running = false;
    state.launched = false;
    state.running_after_polls = Some(1);
}

#[test]
fn c1a_deferred_variant_does_not_block_take_back_and_retries_when_it_appears() {
    let a = TestApp::with_camera_script(TAKE_BACK).unwrap();
    a.fake_processes.set(vec!["Capture One"]);
    settle(&a, || a.app.health.has("camera_held"));
    let (_, session) = a
        .items()
        .create_local("P54-A", "Deferred", "object", None)
        .unwrap();
    let photo = a.add_photo(session.session.id, "DSC05401.ARW").unwrap();
    a.store().upsert_raw_state(&photo.raw_path, None).unwrap();
    a.app.previews.request_initial(&photo.raw_path).unwrap(); // variant_not_found
    assert_eq!(a.take_back().blocked_reason().unwrap(), None);
    a.sessions().activate(session.session.id).unwrap();
    {
        let mut state = a.fake.state.lock().unwrap();
        state.quit_after_polls = Some(1);
        state.running_after_polls = Some(1);
    }
    let take_back = a.take_back();
    let report = thread::scope(|scope| {
        let handle = scope.spawn(|| take_back.run());
        let end = Instant::now() + Duration::from_secs(30);
        while !handle.is_finished() {
            assert!(Instant::now() < end, "take-back did not finish");
            a.tick(500).unwrap();
            thread::sleep(Duration::from_millis(20));
        }
        handle.join().unwrap().unwrap()
    });
    assert_eq!(report.completed_steps, 5);
    assert!(ops(&a).iter().any(|op| op == "quit"));
    // The deferred request is still useful after Capture One has the document again.
    a.fake.add_image(&photo.raw_path, &["v-5401"], false);
    a.clock.advance(2_000);
    a.app.previews.variant_ready(&photo.raw_path).unwrap();
    assert!(
        a.app.editing.render_in_flight(&photo.raw_path),
        "state={:?} ops={:?}",
        a.app.store.preview_state(&photo.raw_path),
        ops(&a)
    );
}

#[test]
fn c1b_issued_render_still_blocks_take_back() {
    let a = TestApp::with_camera_script(CLAIMED_WIFI).unwrap();
    a.fake_processes.set(vec!["Capture One"]);
    settle(&a, || a.app.health.has("camera_held"));
    let (_, session) = a
        .items()
        .create_local("P54-B", "Rendering", "object", None)
        .unwrap();
    let photo = a.add_photo(session.session.id, "DSC05402.ARW").unwrap();
    a.fake.add_image(&photo.raw_path, &["v-5402"], false);
    a.app
        .editing
        .request_render(
            &photo.raw_path,
            RenderPurpose::Preview,
            1,
            1,
            a.clock.mono(),
            a.clock.now_ms(),
        )
        .unwrap();
    assert_eq!(
        a.take_back().blocked_reason().unwrap().as_deref(),
        Some("a preview is rendering")
    );
}

#[test]
fn c2a_relaunch_waits_for_our_usb_session_and_is_bounded() {
    let a = TestApp::with_camera_script(READY_USB).unwrap();
    settle(&a, || {
        a.camera_status().unwrap()
            == CameraStatus::Connected {
                name: "ILCE-7RM5".into(),
                link: Link::Cable,
            }
    });
    stopped(&a);
    a.fake.state.lock().unwrap().ops.clear();
    a.inject("usb.initial", json!({"present":true})).unwrap();
    a.inject(
        "session.closed",
        json!({"transport":"usb","reason":"Capture One released USB"}),
    )
    .unwrap();
    a.inject(
        "usb.claimed_by",
        json!({"owners":"pid 512, Capture One","hint":"Capture One released the cable"}),
    )
    .unwrap();
    settle(&a, || a.app.start_order().needs_relaunch_wait());
    thread::scope(|scope| {
        let handle = scope.spawn(|| a.user_lane().launch());
        settle(&a, || {
            a.fake_sidecar_recording()
                .stdin
                .iter()
                .any(|(_, line)| line.starts_with("usb-retry"))
        });
        assert_eq!(
            a.fake_sidecar_recording()
                .stdin
                .iter()
                .filter(|(_, line)| line.starts_with("usb-retry"))
                .count(),
            1
        );
        assert!(!ops(&a).iter().any(|op| op == "launch"));
        a.inject(
            "connection.ready",
            json!({"transport":"usb","model":"ILCE-7RM5","store_destination":"verified"}),
        )
        .unwrap();
        settle(&a, || !a.app.start_order().needs_relaunch_wait());
        handle.join().unwrap().unwrap();
    });
    assert!(ops(&a).iter().any(|op| op == "launch"));
    stopped(&a);
    a.fake.state.lock().unwrap().ops.clear();
    a.inject(
        "session.closed",
        json!({"transport":"usb","reason":"Capture One released USB"}),
    )
    .unwrap();
    settle(&a, || a.app.start_order().needs_relaunch_wait());
    thread::scope(|scope| {
        let handle = scope.spawn(|| a.user_lane().launch());
        settle(&a, || {
            a.fake_sidecar_recording()
                .stdin
                .iter()
                .filter(|(_, line)| line.starts_with("usb-retry"))
                .count()
                == 2
        });
        assert!(!ops(&a).iter().any(|op| op == "launch"));
        a.clock.advance(15_000);
        handle.join().unwrap().unwrap();
    });
    assert!(ops(&a).iter().any(|op| op == "launch"));
}

#[test]
fn c2b_relaunches_immediately_without_cable_or_when_usb_is_ours() {
    for (script, link) in [(READY_WIFI, Link::Wireless), (READY_USB, Link::Cable)] {
        let a = TestApp::with_camera_script(script).unwrap();
        settle(
            &a,
            || matches!(a.camera_status().unwrap(), CameraStatus::Connected {link: l, ..} if l == link),
        );
        let present = link == Link::Cable;
        a.inject("usb.initial", json!({"present":present})).unwrap();
        settle(&a, || {
            a.sidecar_events().iter().any(|stamped| matches!(
            stamped.event,
            psm_integrations::camera::sidecar::protocol::SidecarEvent::UsbInitial { present: value } if value == present
        ))
        });
        stopped(&a);
        a.fake.state.lock().unwrap().ops.clear();
        a.user_lane().launch().unwrap();
        assert!(ops(&a).iter().any(|op| op == "launch"));
        assert!(
            !a.fake_sidecar_recording()
                .stdin
                .iter()
                .any(|(_, line)| line.starts_with("usb-retry"))
        );
    }
}

#[test]
fn c4a_retryable_wifi_discovery_remains_connecting_until_ready() {
    let a = TestApp::with_camera_script(READY_WIFI).unwrap();
    settle(&a, || {
        matches!(
            a.camera_status().unwrap(),
            CameraStatus::Connected {
                link: Link::Wireless,
                ..
            }
        )
    });
    a.inject_all(&[
        ("session.closed", json!({"transport":"wifi","reason":"retry"})),
        ("transport.state", json!({"from":"DISCONNECTED","to":"DISCOVERING","reason":"looking for camera over wifi","transport":"none"})),
        ("transport.state", json!({"from":"DISCOVERING","to":"CONNECTING_WIFI","reason":"camera found","transport":"none"})),
        ("transport.state", json!({"from":"CONNECTING_WIFI","to":"DISCONNECTED","reason":"connect failed","transport":"none"})),
    ]).unwrap();
    settle(&a, || {
        a.camera_status().unwrap()
            == CameraStatus::Connecting {
                link: Link::Wireless,
            }
    });
    a.inject(
        "connection.ready",
        json!({"transport":"wifi","model":"ILCE-7RM5","store_destination":"verified"}),
    )
    .unwrap();
    settle(&a, || {
        matches!(
            a.camera_status().unwrap(),
            CameraStatus::Connected {
                link: Link::Wireless,
                ..
            }
        )
    });
}

#[test]
fn c5a_read_only_availability_reports_export_and_then_clears() {
    let a = TestApp::with_camera_script(CLAIMED_WIFI).unwrap();
    a.fake_processes.set(vec!["Capture One"]);
    settle(&a, || a.app.health.has("camera_held"));
    let conn = rusqlite::Connection::open(a.app.paths.db()).unwrap();
    conn.execute("INSERT INTO jobs(kind,target,payload,status,created_at,updated_at) VALUES ('render_master','t','{}','running',0,0)", []).unwrap();
    let before = ops(&a);
    assert_eq!(
        a.take_back().blocked_reason().unwrap().as_deref(),
        Some("an export is running")
    );
    assert_eq!(ops(&a), before);
    conn.execute(
        "UPDATE jobs SET status='done' WHERE kind='render_master'",
        [],
    )
    .unwrap();
    assert_eq!(a.take_back().blocked_reason().unwrap(), None);
}
