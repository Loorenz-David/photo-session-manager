//! P46 — the direct camera adapter (10E MC-31 behaviour, MC-33 integration): one behaviour test
//! per criterion row through `TestApp`, the fake sidecar's scripts and its inject channel. The
//! unit rows (C2(d), C4(c)'s and C7(a)'s unit halves) live beside the functions in
//! `psm_integrations::camera::direct`; C8(b) and C6(c)'s command half are desktop `p46_tests`.
use psm_core::{
    domain::{
        HealthItem,
        camera::{
            Association, BusyReason, CameraStatus, DeliveryOutcome, DeliveryRow, DeliveryState,
            Exposure, ExposureInitiator, ExposureStatus, Link, NeedsActionReason,
        },
    },
    events::AppEvent,
    services::camera::CameraService,
};
use psm_integrations::camera::sidecar::protocol::SidecarEvent;
use psm_testkit::{TestApp, test_app::SCRIPT_EPOCH_MS};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    thread,
    time::{Duration, Instant},
};

const DISCOVERING: &str = include_str!("../src/fixtures/sidecar_scripts/discovering.txt");
const READY_USB: &str = include_str!("../src/fixtures/sidecar_scripts/ready_usb.txt");
const READY_WIFI: &str = include_str!("../src/fixtures/sidecar_scripts/ready_wifi.txt");
const READY_USB_STORE_FAILED: &str =
    include_str!("../src/fixtures/sidecar_scripts/ready_usb_store_failed.txt");
const CAPTURE_USB: &str = include_str!("../src/fixtures/sidecar_scripts/capture_usb.txt");
const CAPTURE: &str = include_str!("../src/fixtures/sidecar_scripts/capture.txt");
const CAMERA_SHUTTER: &str = include_str!("../src/fixtures/sidecar_scripts/camera_shutter.txt");
const CLAIMED: &str = include_str!("../src/fixtures/sidecar_scripts/claimed.txt");
const FINGERPRINT_UNKNOWN: &str =
    include_str!("../src/fixtures/sidecar_scripts/fingerprint_unknown.txt");
const AUTH_REFUSED: &str = include_str!("../src/fixtures/sidecar_scripts/auth_refused.txt");

/// The exposure stamp of `capture_usb.txt` and of `capture.txt` / `camera_shutter.txt`.
const T_E_USB: i64 = SCRIPT_EPOCH_MS + 2_550;
const T_E_WIFI: i64 = SCRIPT_EPOCH_MS + 550;
const MODEL: &str = "ILCE-7RM5";

fn app(scripts: &[&str]) -> TestApp {
    TestApp::with_camera_script(&scripts.concat()).unwrap()
}
/// P42's loop: `a.tick(0)` / 10 ms sleep until `pred` holds.
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
        "camera result did not settle: status={:?} events={:?} log={:?} stdin={:?}",
        a.camera_status(),
        a.sidecar_events(),
        a.camera_runtime().helper_log(),
        a.fake_sidecar_recording().stdin
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
fn needs(reason: NeedsActionReason, holder: Option<&str>) -> CameraStatus {
    CameraStatus::NeedsAction {
        reason,
        holder: holder.map(str::to_owned),
        message: None,
    }
}
fn busy(reason: BusyReason) -> CameraStatus {
    CameraStatus::Busy { reason }
}
/// The `camera_status_changed` payloads the app emitted, in order.
fn ev(a: &TestApp) -> Vec<CameraStatus> {
    a.events
        .0
        .lock()
        .unwrap()
        .iter()
        .filter_map(|e| match e {
            AppEvent::CameraStatusChanged { status } => Some(status.clone()),
            _ => None,
        })
        .collect()
}
fn health_item(a: &TestApp, code: &str) -> Option<HealthItem> {
    a.app
        .health
        .report("", false, false)
        .items
        .into_iter()
        .find(|i| i.code == code)
}
/// `(ok, detail)` of the health item `code`.
fn health(a: &TestApp, code: &str) -> Option<(bool, String)> {
    health_item(a, code).map(|i| (i.ok, i.detail))
}
fn stdin(a: &TestApp) -> Vec<String> {
    a.fake_sidecar_recording()
        .stdin
        .into_iter()
        .map(|(_, line)| line)
        .collect()
}
fn count_stdin(a: &TestApp, head: &str) -> usize {
    stdin(a).iter().filter(|l| l.starts_with(head)).count()
}
fn events(a: &TestApp) -> Vec<SidecarEvent> {
    a.sidecar_events().into_iter().map(|s| s.event).collect()
}
fn cmd_done(a: &TestApp) -> bool {
    events(a)
        .iter()
        .any(|e| matches!(e, SidecarEvent::CmdDone { .. }))
}
/// The end of `capture_usb.txt`: `cmd.done` and the `camera.busy file_transfer` after it.
fn usb_capture_done(a: &TestApp) -> bool {
    cmd_done(a)
        && events(a).iter().any(
            |e| matches!(e, SidecarEvent::CameraBusy { reason, .. } if reason == "file_transfer"),
        )
}
fn exposures(a: &TestApp) -> Vec<Exposure> {
    a.ledger().exposures(None).unwrap()
}
fn deliveries(a: &TestApp) -> Vec<DeliveryRow> {
    a.ledger().deliveries().unwrap()
}
fn done_count(a: &TestApp) -> usize {
    deliveries(a)
        .iter()
        .filter(|d| d.state == DeliveryState::Done)
        .count()
}
/// A local item and its session, activated.
fn active_session(a: &TestApp) -> (i64, i64) {
    let (item, session) = a
        .items()
        .create_local("P46-ITEM", "P46 item", "object", None)
        .unwrap();
    a.sessions().activate(session.session.id).unwrap();
    (item.id, session.session.id)
}
fn key_of(a: &TestApp, item: i64) -> String {
    a.store().item(item).unwrap().folder_key
}
fn photos(a: &TestApp, session: i64) -> Vec<psm_core::domain::photo::Photo> {
    a.store().photos_in_session(session).unwrap()
}
fn watch(a: &TestApp, path: impl AsRef<Path>) {
    a.watcher_event(path.as_ref()).unwrap();
    a.tick(1_000).unwrap();
}
fn sha(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
fn data(tag: &str) -> Vec<u8> {
    format!("II*\0 raw bytes of {tag}").into_bytes()
}
fn files_in(dir: &Path) -> Vec<PathBuf> {
    fs::read_dir(dir)
        .map(|r| r.flatten().map(|e| e.path()).collect())
        .unwrap_or_default()
}
/// `ready_usb.txt + capture_usb.txt` until connected, an active session, one app capture, until
/// the script's end; then the clock moves past the exposure (T0 + 5 s).
fn usb_capture(a: &TestApp) -> (i64, i64, String) {
    settle(a, || st(a) == connected(Link::Cable));
    let (item, s) = active_session(a);
    let id = a.capture().unwrap();
    settle(a, || usb_capture_done(a));
    (item, s, id.0)
}

#[test]
fn c1a_status_from_usb_session() {
    let a = app(&[READY_USB]);
    settle(&a, || ev(&a).last() == Some(&connected(Link::Cable)));
    // Health is read before any `CameraService::status()` call: the adapter published it.
    assert_eq!(
        health(&a, "camera"),
        Some((true, "ILCE-7RM5 · cable".into()))
    );
    assert_eq!(health(&a, "camera_disconnected"), None);
    assert_eq!(health(&a, "camera_connecting"), None);
    let events = ev(&a);
    assert_eq!(
        events[events.len() - 2..],
        [
            CameraStatus::Connecting { link: Link::Cable },
            connected(Link::Cable)
        ]
    );
    assert_eq!(st(&a), connected(Link::Cable));
}

#[test]
fn c1b_status_from_wifi_session() {
    let a = app(&[READY_WIFI]);
    settle(&a, || ev(&a).last() == Some(&connected(Link::Wireless)));
    assert_eq!(st(&a), connected(Link::Wireless));
}

#[test]
fn c1c_session_lost_connecting_then_sidecar_exit_disconnected() {
    let a = app(&[READY_USB]);
    settle(&a, || st(&a) == connected(Link::Cable));
    a.inject_all(&[
        (
            "session.lost",
            json!({"transport":"usb","reason":"usb device removed","busy":"none"}),
        ),
        (
            "transport.state",
            json!({"from":"CONNECTED_USB","to":"RECOVERING","reason":"usb device removed","transport":"none"}),
        ),
        (
            "transport.state",
            json!({"from":"RECOVERING","to":"DISCOVERING","reason":"looking for camera over wifi","transport":"none"}),
        ),
    ])
    .unwrap();
    let connecting = CameraStatus::Connecting {
        link: Link::Wireless,
    };
    settle(&a, || ev(&a).last() == Some(&connecting));
    assert_eq!(
        health(&a, "camera_connecting"),
        Some((false, "Connecting over Wi-Fi…".into()))
    );
    assert_eq!(st(&a), connecting);
    // The clock does not move: the supervisor's restart after 1 s of mono time never comes.
    a.inject_exit(3).unwrap();
    settle(&a, || ev(&a).last() == Some(&CameraStatus::Disconnected));
    assert_eq!(
        health(&a, "camera_disconnected"),
        Some((false, "No camera connected".into()))
    );
    assert_eq!(health(&a, "camera"), None);
    assert_eq!(health(&a, "camera_connecting"), None);
    assert_eq!(st(&a), CameraStatus::Disconnected);
    assert!(matches!(
        a.capture(),
        Err(psm_core::Error::CameraNotConnected)
    ));
    assert_eq!(a.fake_sidecar_recording().pids.len(), 1);
}

#[test]
fn c1d_switching_is_busy_across_the_switch() {
    let a = app(&[READY_WIFI]);
    settle(&a, || st(&a) == connected(Link::Wireless));
    a.inject_all(&[
        ("usb.arrived", json!({"detail":"pid=0x0E0C ILCE-7RM5"})),
        (
            "transport.state",
            json!({"from":"CONNECTED_WIFI","to":"SWITCHING","reason":"usb available; preferring usb","transport":"wifi"}),
        ),
        (
            "session.closed",
            json!({"transport":"wifi","reason":"switching wifi -> usb"}),
        ),
    ])
    .unwrap();
    settle(&a, || {
        events(&a)
            .iter()
            .any(|e| matches!(e, SidecarEvent::SessionClosed { .. }))
    });
    assert_eq!(st(&a), busy(BusyReason::Switching));
    assert_eq!(
        health(&a, "camera_busy"),
        Some((false, "Switching connection".into()))
    );
    a.inject_all(&[
        (
            "transport.state",
            json!({"from":"SWITCHING","to":"DISCOVERING","reason":"looking for camera over usb","transport":"none"}),
        ),
        (
            "transport.state",
            json!({"from":"DISCOVERING","to":"CONNECTING_USB","reason":"camera found","transport":"none"}),
        ),
        (
            "transport.state",
            json!({"from":"CONNECTING_USB","to":"CONNECTED_USB","reason":"connected","transport":"usb"}),
        ),
        (
            "connection.ready",
            json!({"transport":"usb","model":MODEL,"store_destination":"verified","store_value":"0x0003"}),
        ),
    ])
    .unwrap();
    settle(&a, || st(&a) == connected(Link::Cable));
    assert_eq!(health(&a, "camera_busy"), None);
    assert!(
        !ev(&a).contains(&CameraStatus::Connecting { link: Link::Cable }),
        "Switching must hold through the intermediate lines: {:?}",
        ev(&a)
    );
}

#[test]
fn c2a_capture_returns_id_and_records_exposure() {
    let a = app(&[READY_USB, CAPTURE_USB]);
    settle(&a, || st(&a) == connected(Link::Cable));
    let (_, s) = active_session(&a);
    let id = a.capture().unwrap();
    settle(&a, || st(&a) == busy(BusyReason::Exposure));
    settle(&a, || cmd_done(&a));
    let wire = format!("capture id={} af=1 cmd_id=", id.0);
    assert_eq!(count_stdin(&a, &wire), 1, "{:?}", stdin(&a));
    let rows = exposures(&a);
    assert_eq!(rows.len(), 1);
    let row = &rows[0];
    assert_eq!(row.capture_id.as_deref(), Some(id.0.as_str()));
    assert_eq!(row.initiated, ExposureInitiator::App);
    assert_eq!(row.session_id, Some(s));
    assert_eq!(row.status, ExposureStatus::Expecting);
    assert_eq!(row.exposed_at, T_E_USB);
    assert_eq!(st(&a), busy(BusyReason::Transfer));
}

#[test]
fn c2b_camera_initiated_shot() {
    let a = app(&[READY_WIFI, CAMERA_SHUTTER]);
    let (_, s) = active_session(&a);
    settle(&a, || exposures(&a).len() == 1);
    let row = &exposures(&a)[0];
    assert_eq!(row.capture_id, None);
    assert_eq!(row.initiated, ExposureInitiator::Camera);
    assert_eq!(row.session_id, Some(s));
    assert_eq!(row.status, ExposureStatus::Expecting);
    assert_eq!(row.exposed_at, T_E_WIFI);
    settle(&a, || st(&a) == connected(Link::Wireless));
    settle_absence(&a);
    assert_eq!(st(&a), connected(Link::Wireless));
}

#[test]
fn c2c_capture_refused_without_session() {
    let a = app(&[DISCOVERING]);
    settle(&a, || {
        st(&a) == CameraStatus::Connecting { link: Link::Cable }
    });
    assert!(matches!(
        a.capture(),
        Err(psm_core::Error::CameraNotConnected)
    ));
    settle_absence(&a);
    assert_eq!(
        stdin(&a),
        vec!["credentials user=u%20name password=p%2Fw".to_owned()]
    );
    assert!(exposures(&a).is_empty());
}

#[test]
fn c3a_save_to_pc_completion_becomes_cable_delivery() {
    let a = app(&[READY_USB, CAPTURE_USB]);
    let (item, s, _) = usb_capture(&a);
    a.tick(5_000).unwrap();
    let bytes = data("DSC00007");
    a.sdk_download("DSC00007.ARW", &bytes).unwrap();
    settle(&a, || done_count(&a) == 1);
    let rows = deliveries(&a);
    assert_eq!(rows.len(), 1);
    let row = &rows[0];
    assert_eq!(row.camera_filename, "DSC00007.ARW");
    assert_eq!(row.via, Link::Cable);
    assert_eq!(row.sha256, sha(&bytes));
    assert_eq!(row.size, bytes.len() as u64);
    assert_eq!(row.association, Association::Fifo);
    assert_eq!(row.outcome, DeliveryOutcome::Photo);
    assert_eq!(row.completed_at, SCRIPT_EPOCH_MS + 5_000);
    let final_path = a.capture_root().join(key_of(&a, item)).join("DSC00007.ARW");
    assert_eq!(row.final_path, final_path.to_string_lossy());
    assert_eq!(fs::read(&final_path).unwrap(), bytes);
    assert!(files_in(&a.staging_dir()).is_empty());
    watch(&a, &final_path);
    let photos = photos(&a, s);
    assert_eq!(photos.len(), 1);
    assert_eq!(photos[0].captured_at, Some(T_E_USB));
}

#[test]
fn c3b_busy_transfer_until_completion_or_10s() {
    // (i) the completion ends it.
    let a = app(&[READY_USB, CAPTURE_USB]);
    usb_capture(&a);
    assert_eq!(st(&a), busy(BusyReason::Transfer));
    a.sdk_download("DSC00011.ARW", &data("DSC00011")).unwrap();
    settle(&a, || st(&a) == connected(Link::Cable));
    assert_eq!(health(&a, "camera_busy"), None);
    // (ii) no download: 10 s of mono time end it.
    let b = app(&[READY_USB, CAPTURE_USB]);
    usb_capture(&b);
    assert_eq!(st(&b), busy(BusyReason::Transfer));
    b.tick(9_999).unwrap();
    assert_eq!(st(&b), busy(BusyReason::Transfer));
    b.tick(1).unwrap();
    assert_eq!(st(&b), connected(Link::Cable));
    assert_eq!(health(&b, "camera_busy"), None);
}

#[test]
fn c3c_zero_or_missing_download_never_promoted() {
    let a = app(&[READY_USB]);
    settle(&a, || st(&a) == connected(Link::Cable));
    let before = a.camera_runtime().helper_log().len();
    let x = a.staging_dir().join("X.ARW");
    fs::write(&x, b"").unwrap();
    a.inject(
        "sdk.download.completed",
        json!({"filename": x.to_string_lossy(), "size_bytes": 0, "success": true, "transport": "usb"}),
    )
    .unwrap();
    a.inject(
        "sdk.download.completed",
        json!({"filename": a.staging_dir().join("Y.ARW").to_string_lossy(), "size_bytes": 10, "success": true, "transport": "usb"}),
    )
    .unwrap();
    settle(&a, || a.camera_runtime().helper_log().len() >= before + 2);
    settle_absence(&a);
    assert!(deliveries(&a).is_empty());
    assert!(x.is_file());
    assert_eq!(fs::metadata(&x).unwrap().len(), 0);
    assert_eq!(
        a.camera_runtime().helper_log()[before..],
        [
            "download ignored: X.ARW (size 0)".to_owned(),
            "download ignored: Y.ARW (missing)".to_owned()
        ]
    );
}

#[test]
fn c3d_resend_suffix_by_cable_keys_to_original() {
    let a = app(&[READY_USB]);
    settle(&a, || st(&a) == connected(Link::Cable));
    let (item, s) = active_session(&a);
    a.tick(5_000).unwrap();
    let bytes = data("DSC00008");
    assert!(
        a.ftp()
            .stor("DSC00008.ARW", &bytes)
            .unwrap()
            .starts_with("226")
    );
    settle(&a, || done_count(&a) == 1);
    a.sdk_download("DSC00008_1.ARW", &bytes).unwrap();
    settle(&a, || done_count(&a) == 2);
    let second = deliveries(&a)
        .into_iter()
        .find(|d| d.via == Link::Cable)
        .unwrap();
    assert_eq!(second.camera_filename, "DSC00008.ARW");
    assert_eq!(second.outcome, DeliveryOutcome::Duplicate);
    let duplicate = a.capture_root().join(".duplicates/DSC00008.ARW");
    assert_eq!(second.final_path, duplicate.to_string_lossy());
    assert_eq!(fs::read(&duplicate).unwrap(), bytes);
    watch(
        &a,
        a.capture_root().join(key_of(&a, item)).join("DSC00008.ARW"),
    );
    watch(&a, &duplicate);
    assert_eq!(photos(&a, s).len(), 1);
}

#[test]
fn c4a_store_destination_failed_keeps_session() {
    let a = app(&[READY_USB_STORE_FAILED]);
    let expected = needs(NeedsActionReason::StoreDestinationUnverified, None);
    settle(&a, || ev(&a).last() == Some(&expected));
    assert_eq!(
        health(&a, "camera_store_destination_unverified"),
        Some((
            false,
            "The camera did not confirm where photos are saved".into()
        ))
    );
    assert_eq!(
        health(&a, "camera"),
        Some((true, "ILCE-7RM5 · cable".into()))
    );
    assert_eq!(st(&a), expected);
    let id = a.capture().unwrap();
    let wire = format!("capture id={} af=1 cmd_id=", id.0);
    settle(&a, || count_stdin(&a, &wire) == 1);
}

#[test]
fn c4b_verified_clears_store_destination() {
    let a = app(&[READY_USB_STORE_FAILED]);
    let failed = needs(NeedsActionReason::StoreDestinationUnverified, None);
    settle(&a, || st(&a) == failed);
    a.inject_all(&[
        (
            "session.lost",
            json!({"transport":"usb","reason":"usb device removed","busy":"none"}),
        ),
        (
            "connection.ready",
            json!({"transport":"usb","model":MODEL,"store_destination":"verified","store_value":"0x0003"}),
        ),
    ])
    .unwrap();
    settle(&a, || st(&a) == connected(Link::Cable));
    assert_eq!(health(&a, "camera_store_destination_unverified"), None);
}

#[test]
fn c4c_authentication_refused_then_cleared_by_wifi_session() {
    let a = app(&[AUTH_REFUSED]);
    let refused = needs(NeedsActionReason::AuthenticationRefused, None);
    settle(&a, || st(&a) == refused);
    assert_eq!(
        health(&a, "camera_auth_refused"),
        Some((false, "The camera refused the password".into()))
    );
    a.inject(
        "connection.ready",
        json!({"transport":"wifi","model":MODEL,"store_destination":"verified","store_value":"0x0001"}),
    )
    .unwrap();
    settle(&a, || st(&a) == connected(Link::Wireless));
    assert_eq!(health(&a, "camera_auth_refused"), None);
}

#[test]
fn c4d_fingerprint_unknown_until_pinned() {
    let a = app(&[FINGERPRINT_UNKNOWN]);
    let unconfirmed = needs(NeedsActionReason::FingerprintUnconfirmed, None);
    settle(&a, || st(&a) == unconfirmed);
    assert_eq!(
        health(&a, "camera_fingerprint_unconfirmed"),
        Some((false, "Confirm the camera's fingerprint".into()))
    );
    a.inject("ssh.fingerprint", json!({"status":"pinned","success":true}))
        .unwrap();
    settle(&a, || !matches!(st(&a), CameraStatus::NeedsAction { .. }));
    assert_eq!(st(&a), CameraStatus::Disconnected);
    assert_eq!(health(&a, "camera_fingerprint_unconfirmed"), None);
}

#[test]
fn c5a_first_copy_wins_cable_then_ftp() {
    let a = app(&[READY_USB, CAPTURE_USB]);
    let (item, s, _) = usb_capture(&a);
    a.tick(5_000).unwrap();
    let bytes = data("DSC04622");
    a.sdk_download("DSC04622.ARW", &bytes).unwrap();
    settle(&a, || done_count(&a) == 1);
    assert!(
        a.ftp()
            .stor("DSC04622.ARW", &bytes)
            .unwrap()
            .starts_with("226")
    );
    settle(&a, || done_count(&a) == 2);
    let photo_path = a.capture_root().join(key_of(&a, item)).join("DSC04622.ARW");
    let duplicate = a.capture_root().join(".duplicates/DSC04622.ARW");
    let rows = deliveries(&a);
    let cable = rows.iter().find(|d| d.via == Link::Cable).unwrap();
    let wireless = rows.iter().find(|d| d.via == Link::Wireless).unwrap();
    assert_eq!(
        (cable.outcome, cable.final_path.as_str()),
        (DeliveryOutcome::Photo, &*photo_path.to_string_lossy())
    );
    assert_eq!(
        (wireless.outcome, wireless.final_path.as_str()),
        (DeliveryOutcome::Duplicate, &*duplicate.to_string_lossy())
    );
    assert_eq!(fs::read(&photo_path).unwrap(), bytes);
    assert_eq!(fs::read(&duplicate).unwrap(), bytes);
    watch(&a, &photo_path);
    watch(&a, &duplicate);
    assert_eq!(photos(&a, s).len(), 1);
}

#[test]
fn c5b_first_copy_wins_ftp_then_cable() {
    let a = app(&[READY_USB, CAPTURE_USB]);
    let (item, s, _) = usb_capture(&a);
    a.tick(5_000).unwrap();
    let bytes = data("DSC04623");
    assert!(
        a.ftp()
            .stor("DSC04623.ARW", &bytes)
            .unwrap()
            .starts_with("226")
    );
    settle(&a, || done_count(&a) == 1);
    a.sdk_download("DSC04623.ARW", &bytes).unwrap();
    settle(&a, || done_count(&a) == 2);
    let photo_path = a.capture_root().join(key_of(&a, item)).join("DSC04623.ARW");
    let duplicate = a.capture_root().join(".duplicates/DSC04623.ARW");
    let rows = deliveries(&a);
    let cable = rows.iter().find(|d| d.via == Link::Cable).unwrap();
    let wireless = rows.iter().find(|d| d.via == Link::Wireless).unwrap();
    assert_eq!(
        (wireless.outcome, wireless.final_path.as_str()),
        (DeliveryOutcome::Photo, &*photo_path.to_string_lossy())
    );
    assert_eq!(
        (cable.outcome, cable.final_path.as_str()),
        (DeliveryOutcome::Duplicate, &*duplicate.to_string_lossy())
    );
    assert_eq!(fs::read(&photo_path).unwrap(), bytes);
    assert_eq!(fs::read(&duplicate).unwrap(), bytes);
    watch(&a, &photo_path);
    watch(&a, &duplicate);
    assert_eq!(photos(&a, s).len(), 1);
}

/// `ready_wifi.txt + capture.txt`, one app capture with no file, 61 s later three 1-s ticks.
fn raised_watchdog() -> TestApp {
    let a = app(&[READY_WIFI, CAPTURE]);
    settle(&a, || st(&a) == connected(Link::Wireless));
    active_session(&a);
    a.capture().unwrap();
    settle(&a, || exposures(&a).len() == 1);
    a.clock.advance(61_000);
    for _ in 0..3 {
        a.tick(1_000).unwrap();
    }
    a
}
const NOT_RECEIVED_1: &str = "1 photos were taken but not received. The camera may have stopped sending: on the camera, switch FTP Function off and on.";

#[test]
fn c6a_watchdog_raise_sends_ftp_status_once() {
    let a = raised_watchdog();
    settle(&a, || count_stdin(&a, "ftp-status cmd_id=") >= 1);
    settle_absence(&a);
    assert_eq!(count_stdin(&a, "ftp-status"), 1, "{:?}", stdin(&a));
    assert_eq!(
        st(&a),
        needs(NeedsActionReason::FilesNotReceived { count: 1 }, None)
    );
    assert_eq!(
        health(&a, "camera_files_not_received"),
        Some((false, NOT_RECEIVED_1.into()))
    );
}

#[test]
fn c6b_transfer_in_progress_defers_the_raise() {
    let a = app(&[READY_WIFI, CAPTURE, CAPTURE]);
    settle(&a, || st(&a) == connected(Link::Wireless));
    active_session(&a);
    a.capture().unwrap();
    settle(&a, || exposures(&a).len() == 1);
    a.capture().unwrap();
    settle(&a, || exposures(&a).len() == 2);
    let mut f = a.ftp();
    let (reply, data_conn) = f.open_stor("DSC00030.ARW").unwrap();
    let mut data_conn = data_conn.unwrap_or_else(|| panic!("STOR refused: {reply}"));
    data_conn.write_all(b"I").unwrap();
    data_conn.flush().unwrap();
    settle(&a, || a.camera_runtime().uploads_in_progress() == 1);
    a.clock.advance(61_000);
    a.tick(1_000).unwrap();
    a.tick(1_000).unwrap();
    settle_absence(&a);
    assert_eq!(st(&a), connected(Link::Wireless));
    assert_eq!(count_stdin(&a, "ftp-status"), 0);
    assert_eq!(health(&a, "camera_files_not_received"), None);
    data_conn.write_all(b"I*\0 rest of DSC00030").unwrap();
    assert!(f.finish_stor(data_conn).unwrap().starts_with("226"));
    settle(&a, || done_count(&a) == 1);
    assert_eq!(
        deliveries(&a)[0].completed_at,
        SCRIPT_EPOCH_MS + 63_000,
        "completion stamped by the test clock"
    );
    a.tick(1_000).unwrap();
    a.tick(1_000).unwrap();
    assert_eq!(
        st(&a),
        needs(NeedsActionReason::FilesNotReceived { count: 1 }, None)
    );
    settle(&a, || count_stdin(&a, "ftp-status cmd_id=") >= 1);
    settle_absence(&a);
    assert_eq!(count_stdin(&a, "ftp-status"), 1);
}

#[test]
fn c6c_dismiss_clears_the_notice() {
    let a = raised_watchdog();
    assert_eq!(
        st(&a),
        needs(NeedsActionReason::FilesNotReceived { count: 1 }, None)
    );
    let ledger = a.ledger();
    CameraService::new(a.app.camera.as_ref(), a.app.health.as_ref())
        .with_ledger(ledger.as_ref())
        .dismiss_files_not_received()
        .unwrap();
    a.tick(0).unwrap();
    assert_eq!(st(&a), connected(Link::Wireless));
    assert_eq!(exposures(&a)[0].status, ExposureStatus::Dismissed);
    assert_eq!(health(&a, "camera_files_not_received"), None);
}

#[test]
fn c6d_reconnect_only_on_wifi() {
    let wifi = app(&[READY_WIFI]);
    settle(&wifi, || st(&wifi) == connected(Link::Wireless));
    wifi.app.camera.reconnect().unwrap();
    settle(&wifi, || count_stdin(&wifi, "reconnect cmd_id=") >= 1);
    settle_absence(&wifi);
    assert_eq!(count_stdin(&wifi, "reconnect"), 1);

    let usb = app(&[READY_USB]);
    settle(&usb, || st(&usb) == connected(Link::Cable));
    assert!(matches!(
        usb.app.camera.reconnect(),
        Err(psm_core::Error::ReconnectOnCableRefused)
    ));
    settle_absence(&usb);
    assert_eq!(count_stdin(&usb, "reconnect"), 0);

    let none = app(&[DISCOVERING]);
    settle(&none, || {
        st(&none) == CameraStatus::Connecting { link: Link::Cable }
    });
    assert!(matches!(
        none.app.camera.reconnect(),
        Err(psm_core::Error::CameraNotConnected)
    ));
    settle_absence(&none);
    assert_eq!(count_stdin(&none, "reconnect"), 0);
}

#[test]
fn c6e_camera_ftp_report_reaches_health() {
    let a = raised_watchdog();
    a.inject(
        "camera.ftp_state",
        json!({"ftp_function": 1, "ftp_connection_status": 2, "request_ftp_transfer_result_error": "0x00000000"}),
    )
    .unwrap();
    settle(&a, || {
        events(&a)
            .iter()
            .any(|e| matches!(e, SidecarEvent::CameraFtpState { .. }))
    });
    a.tick(1_000).unwrap();
    assert_eq!(
        health(&a, "camera_files_not_received"),
        Some((
            false,
            format!(
                "{NOT_RECEIVED_1} · camera FTP: ftp_connection_status=2, ftp_function=1, request_ftp_transfer_result_error=0x00000000"
            )
        ))
    );
    let ledger = a.ledger();
    CameraService::new(a.app.camera.as_ref(), a.app.health.as_ref())
        .with_ledger(ledger.as_ref())
        .dismiss_files_not_received()
        .unwrap();
    a.tick(1_000).unwrap();
    assert_eq!(health(&a, "camera_files_not_received"), None);
    // A later raise with no new `camera.ftp_state` carries no FTP suffix.
    a.inject(
        "capture.exposed",
        json!({"success":true,"transport":"wifi","association":"none: no SDK capture pending (camera-initiated?)"}),
    )
    .unwrap();
    settle(&a, || exposures(&a).len() == 2);
    a.clock.advance(61_000);
    a.tick(1_000).unwrap();
    assert_eq!(
        health(&a, "camera_files_not_received"),
        Some((false, NOT_RECEIVED_1.into()))
    );
}

#[test]
fn c7a_holder_reported() {
    let a = app(&[CLAIMED]);
    let held = CameraStatus::NeedsAction {
        reason: NeedsActionReason::HeldByOtherProgram,
        holder: Some("ptpcamerad".into()),
        message: Some(
            "The system camera service is holding the camera: close apps that use cameras".into(),
        ),
    };
    settle(&a, || st(&a) == held);
    assert!(a.app.health.has("camera_held"));
}

#[test]
fn c8a_adapter_selection() {
    let c1 = TestApp::with_fake_document().unwrap();
    let caps = c1.app.camera.capabilities();
    assert!(caps.depends_on_editing_provider);
    assert_eq!(
        st(&c1),
        CameraStatus::Connected {
            name: "Sony ZV-E10".into(),
            link: Link::Cable
        }
    );
    assert!(c1.fake_sidecar_recording().pids.is_empty());

    let direct = app(&[READY_USB]);
    let caps = direct.app.camera.capabilities();
    assert!(!caps.depends_on_editing_provider);
    assert!(caps.settings_write_store_destination);
    settle(&direct, || st(&direct) == connected(Link::Cable));
    assert_eq!(direct.fake_sidecar_recording().pids.len(), 1);
}

fn document_info_asked(a: &TestApp) -> bool {
    a.fake
        .state
        .lock()
        .unwrap()
        .ops
        .iter()
        .any(|op| op.contains("document_info"))
}

#[test]
fn c8c_direct_arrival_capture_root_from_settings() {
    // (i) direct adapter: Capture One "not running" is never asked for the capture root.
    // Reopened after `ops.clear()` like (ii), so the startup scan's capture root is in view too.
    let mut a = app(&[READY_USB, CAPTURE_USB]);
    let (item, s) = active_session(&a);
    {
        let mut fake = a.fake.state.lock().unwrap();
        fake.running = false;
        fake.ops.clear();
    }
    a.kill_and_restart().unwrap();
    settle(&a, || st(&a) == connected(Link::Cable));
    a.capture().unwrap();
    settle(&a, || usb_capture_done(&a));
    a.tick(5_000).unwrap();
    let bytes = data("DSC00020");
    assert!(
        a.ftp()
            .stor("DSC00020.ARW", &bytes)
            .unwrap()
            .starts_with("226")
    );
    settle(&a, || done_count(&a) == 1);
    let final_path = a.capture_root().join(key_of(&a, item)).join("DSC00020.ARW");
    watch(&a, &final_path);
    let photos_i = photos(&a, s);
    assert_eq!(photos_i.len(), 1);
    assert_eq!(photos_i[0].raw_path, final_path.to_string_lossy());
    assert!(!document_info_asked(&a));

    // (ii) Capture One camera adapter: the arrival path still asks `inspect_document`. The
    // arrival service caches the root from its startup scan (`services/arrival.rs`, unchanged),
    // so the app is reopened after `ops.clear()` to give the arrival path a fresh cache.
    let mut b = TestApp::with_fake_document().unwrap();
    let (item, s) = active_session(&b);
    b.fake.state.lock().unwrap().ops.clear();
    b.kill_and_restart().unwrap();
    let path = b.capture_root().join(key_of(&b, item)).join("DSC00020.ARW");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, &bytes).unwrap();
    watch(&b, &path);
    let photos_ii = photos(&b, s);
    assert_eq!(photos_ii.len(), 1);
    assert_eq!(photos_ii[0].raw_path, path.to_string_lossy());
    assert!(document_info_asked(&b));
}
