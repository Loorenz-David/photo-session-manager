use psm_core::{
    domain::camera::{
        CameraStatus, Delivery, ExposureInitiator, ExposureObserved, Link, NeedsActionReason,
    },
    domain::editing::RenderPurpose,
    ports::{camera::CaptureSink, clock::Clock},
};
use psm_testkit::TestApp;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    fs,
    net::{Ipv4Addr, UdpSocket},
    thread,
    time::{Duration, Instant},
};

const CLAIM: &str = concat!(
    "{\"timestamp\":\"2026-11-06T10:00:00.000Z\",\"event\":\"usb.arrived\",\"detail\":\"camera\"}\n",
    "{\"timestamp\":\"2026-11-06T10:00:00.100Z\",\"event\":\"usb.claimed_by\",\"owners\":\"pid 512, ptpcamerad\"}\n",
    "await usb-retry\n",
    "{\"timestamp\":\"2026-11-06T10:00:01.000Z\",\"event\":\"connection.ready\",\"transport\":\"usb\",\"model\":\"ILCE-7RM5\",\"store_destination\":\"verified\"}\n",
    "sleep 100000\n",
);
const SLOW_CLAIM: &str = concat!(
    "{\"timestamp\":\"2026-11-06T10:00:00.000Z\",\"event\":\"usb.arrived\",\"detail\":\"camera\"}\n",
    "{\"timestamp\":\"2026-11-06T10:00:00.100Z\",\"event\":\"usb.claimed_by\",\"owners\":\"pid 512, ptpcamerad\"}\n",
    "await usb-retry\n",
    "sleep 1000\n",
    "{\"timestamp\":\"2026-11-06T10:00:01.000Z\",\"event\":\"connection.ready\",\"transport\":\"usb\",\"model\":\"ILCE-7RM5\",\"store_destination\":\"verified\"}\n",
);

fn settle(a: &TestApp, pred: impl Fn() -> bool) {
    let end = Instant::now() + Duration::from_secs(8);
    while Instant::now() < end {
        a.tick(100).unwrap();
        if pred() {
            return;
        }
        thread::sleep(Duration::from_millis(10));
    }
    panic!("did not settle: {:?}", a.camera_status());
}
fn quit_count(a: &TestApp) -> usize {
    a.fake
        .state
        .lock()
        .unwrap()
        .ops
        .iter()
        .filter(|x| *x == "quit")
        .count()
}
fn launch_count(a: &TestApp) -> usize {
    a.fake
        .state
        .lock()
        .unwrap()
        .ops
        .iter()
        .filter(|x| *x == "launch")
        .count()
}
fn wait_for_fake_op(a: &TestApp, op: &str) {
    let end = Instant::now() + Duration::from_secs(3);
    while Instant::now() < end {
        if a.fake
            .state
            .lock()
            .unwrap()
            .ops
            .iter()
            .any(|seen| seen == op || seen.starts_with(op))
        {
            return;
        }
        thread::sleep(Duration::from_millis(5));
    }
    panic!(
        "fake Capture One did not start {op}; ops={:?}",
        a.fake.state.lock().unwrap().ops
    );
}
fn wait_for_process_count(a: &TestApp, count: usize) {
    let end = Instant::now() + Duration::from_secs(4);
    while Instant::now() < end {
        let seen = a
            .fake
            .state
            .lock()
            .unwrap()
            .ops
            .iter()
            .filter(|op| op.starts_with("process:"))
            .count();
        if seen >= count {
            return;
        }
        thread::sleep(Duration::from_millis(5));
    }
    panic!("fake Capture One did not finish {count} process calls");
}
fn claimed(a: &TestApp) {
    a.fake_processes.set(vec!["Capture One"]);
    {
        let mut s = a.fake.state.lock().unwrap();
        s.quit_after_polls = Some(1);
        s.running_after_polls = Some(1);
        s.ops.clear();
    }
}

#[test]
fn c1a_plug_claim_runs_once_and_reopens_after_usb() {
    let a = TestApp::with_camera_script(CLAIM).unwrap();
    claimed(&a);
    settle(&a, || launch_count(&a) == 1);
    assert!(matches!(
        a.camera_status().unwrap(),
        CameraStatus::Connected {
            link: Link::Cable,
            ..
        }
    ));
    let ops = a.fake.state.lock().unwrap().ops.clone();
    assert_eq!(quit_count(&a), 1);
    assert!(ops.iter().position(|x| x == "quit") < ops.iter().position(|x| x == "launch"));
    a.inject("usb.claimed_by", json!({"owners":"pid 512, ptpcamerad"}))
        .unwrap();
    for _ in 0..10 {
        a.tick(100).unwrap();
    }
    assert_eq!((quit_count(&a), launch_count(&a)), (1, 1));
}

#[test]
fn c1b_waits_for_export_then_takes_cable() {
    let a = TestApp::with_camera_script(CLAIM).unwrap();
    claimed(&a);
    let db = a.app.paths.db();
    let conn = rusqlite::Connection::open(db).unwrap();
    conn.execute("INSERT INTO jobs(kind,target,payload,status,created_at,updated_at) VALUES ('render_master','t','{}','running',0,0)", []).unwrap();
    settle(&a, || {
        format!("{:?}", a.camera_status().unwrap()).contains("Waiting to take the cable")
    });
    assert_eq!(quit_count(&a), 0);
    assert!(
        format!("{:?}", a.camera_status().unwrap())
            .contains("Waiting to take the cable: an export is running")
    );
    conn.execute(
        "UPDATE jobs SET status='done' WHERE kind='render_master'",
        [],
    )
    .unwrap();
    settle(&a, || quit_count(&a) == 1);
}

#[test]
fn c1c_setting_off_keeps_manual_take_back() {
    let mut a = TestApp::with_camera_script(CLAIM).unwrap();
    a.app.settings_set(&json!({"camera":{"receiver_port":2121,"passive_port_start":50000,"passive_port_end":50009,"auto_take_back":false}})).unwrap();
    claimed(&a);
    settle(&a, || a.app.health.has("camera_held"));
    for _ in 0..10 {
        a.tick(100).unwrap();
    }
    assert_eq!(quit_count(&a), 0);
    assert_eq!(a.take_back().blocked_reason().unwrap(), None);
}

#[test]
fn c2a_one_helper_line_reaches_status_without_a_later_line() {
    let a = TestApp::with_camera_script_running_and_delays("", 1500, 3000).unwrap();
    wait_for_fake_op(&a, "document_info_started");
    let ready = Instant::now() + Duration::from_secs(3);
    while a.fake_sidecar_recording().pids.is_empty() && Instant::now() < ready {
        thread::sleep(Duration::from_millis(5));
    }
    assert!(!a.fake_sidecar_recording().pids.is_empty());
    thread::sleep(Duration::from_millis(100));
    let before = Instant::now();
    a.inject(
        "connection.ready",
        json!({"transport":"wifi","model":"ILCE-7RM5","store_destination":"verified"}),
    )
    .unwrap();
    let deadline = before + Duration::from_millis(200);
    while Instant::now() < deadline
        && !matches!(
            a.camera_status(),
            Ok(psm_core::domain::camera::CameraStatus::Connected { .. })
        )
    {
        thread::sleep(Duration::from_millis(5));
    }
    assert!(
        matches!(
            a.camera_status(),
            Ok(psm_core::domain::camera::CameraStatus::Connected {
                link: Link::Wireless,
                ..
            })
        ),
        "line took {:?}",
        before.elapsed()
    );
}

#[test]
fn c3a_late_exposure_finds_filed_photo() {
    let a = TestApp::with_camera().unwrap();
    let bytes = b"II*\0 late";
    let path = a.capture_root().join(".incoming/.DSC05001.ARW.part");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, bytes).unwrap();
    a.ledger()
        .delivery(Delivery {
            camera_filename: "DSC05001.ARW".into(),
            written_path: path,
            sha256: Sha256::digest(bytes)
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect(),
            size: bytes.len() as u64,
            via: Link::Wireless,
            completed_at: a.clock.now_ms(),
        })
        .unwrap();
    a.clock.advance(12_000);
    let exposure = a
        .ledger()
        .exposure_observed(ExposureObserved {
            capture_id: None,
            at: a.clock.now_ms(),
            initiated: ExposureInitiator::Camera,
            provisional: false,
        })
        .unwrap();
    assert_eq!(
        a.ledger().deliveries().unwrap()[0].exposure_id,
        Some(exposure)
    );
    a.clock.advance(61_000);
    assert!(!a.ledger().watchdog(a.clock.now_ms(), false).unwrap().raised);
    assert!(!a.app.health.has("camera_files_not_received"));
}

#[test]
fn c4a_output_quality_save_is_fast_and_does_not_render_previews() {
    let mut a = TestApp::with_camera_script_running_and_delays("", 1500, 3000).unwrap();
    wait_for_fake_op(&a, "document_info_started");
    let (item, session) = a
        .items()
        .create_local("P57-SET", "Settings", "object", None)
        .unwrap();
    let photo = a.add_photo(session.session.id, "DSC05701.ARW").unwrap();
    a.fake.add_image(&photo.raw_path, &["v57"], false);
    let _ = item;
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
    let started = Instant::now();
    a.app
        .settings_set(&json!({"output_format":{"long_edge":2048,"quality":85}}))
        .unwrap();
    let save_elapsed = started.elapsed();
    assert!(save_elapsed < Duration::from_secs(1));
    wait_for_fake_op(&a, "process_started:v57");
    wait_for_process_count(&a, 1);
    thread::sleep(Duration::from_millis(200));
    assert_eq!(a.app.export.format().quality, 85);
    assert_eq!(
        a.fake
            .state
            .lock()
            .unwrap()
            .ops
            .iter()
            .filter(|op| op.starts_with("process:"))
            .count(),
        1
    );
}

#[test]
fn c5a_gate_prevents_early_relaunch_and_take_back_succeeds() {
    let a = TestApp::with_camera_script(SLOW_CLAIM).unwrap();
    claimed(&a);
    settle(&a, || quit_count(&a) == 1);
    assert_eq!(launch_count(&a), 0);
    assert!(a.launch_gate().is_held());
    thread::sleep(Duration::from_millis(200));
    assert_eq!(launch_count(&a), 0);
    settle(&a, || launch_count(&a) == 1);
    assert!(matches!(
        a.camera_status().unwrap(),
        CameraStatus::Connected {
            link: Link::Cable,
            ..
        }
    ));
    let ops = a.fake.state.lock().unwrap().ops.clone();
    let quit = ops.iter().position(|x| x == "quit").unwrap();
    let launch = ops.iter().position(|x| x == "launch").unwrap();
    assert!(quit < launch);
    assert_eq!(launch_count(&a), 1);
    settle(&a, || !a.launch_gate().is_held());
}

#[test]
fn c6a_ftp_error_names_address_and_port_then_clears() {
    let a = TestApp::with_camera_script("").unwrap();
    settle(&a, || !a.fake_sidecar_recording().pids.is_empty());
    thread::sleep(Duration::from_millis(100));
    a.inject(
        "camera.ftp_state",
        json!({"ftp_connection_error":{"value":16},"ftp_connection_status":{"value":0}}),
    )
    .unwrap();
    settle(&a, || a.app.health.has("camera_ftp_unreachable"));
    let address = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0))
        .ok()
        .and_then(|socket| {
            socket
                .connect((Ipv4Addr::new(8, 8, 8, 8), 9))
                .ok()
                .and_then(|_| socket.local_addr().ok())
        })
        .map(|a| a.ip().to_string())
        .unwrap_or_else(|| "this Mac's LAN address".into());
    let expected = format!(
        "The camera can't reach this Mac for uploads. On the camera, set the FTP server's host to {address} and port {}.",
        a.camera_runtime().receiver_port().unwrap()
    );
    assert!(
        matches!(a.camera_status().unwrap(), CameraStatus::NeedsAction { reason: NeedsActionReason::FtpUnreachable, message: Some(m), .. } if m == expected)
    );
    a.inject(
        "camera.ftp_state",
        json!({"ftp_connection_error":{"value":16},"ftp_connection_status":{"value":0}}),
    )
    .unwrap();
    settle(&a, || {
        a.sidecar_events()
            .iter()
            .filter(|e| {
                matches!(
                    e.event,
                    psm_integrations::camera::sidecar::protocol::SidecarEvent::CameraFtpState { .. }
                )
            })
            .count()
            >= 2
    });
    assert!(
        a.ftp()
            .stor("DSC05702.ARW", b"II*\0 ftp")
            .unwrap()
            .starts_with("226")
    );
    settle(&a, || !a.app.health.has("camera_ftp_unreachable"));
    a.inject(
        "camera.ftp_state",
        json!({"ftp_connection_error":{"value":16},"ftp_connection_status":{"value":0}}),
    )
    .unwrap();
    settle(&a, || a.app.health.has("camera_ftp_unreachable"));
    a.inject(
        "camera.ftp_state",
        json!({"ftp_connection_error":{"value":16},"ftp_connection_status":{"value":2}}),
    )
    .unwrap();
    settle(&a, || !a.app.health.has("camera_ftp_unreachable"));
}
