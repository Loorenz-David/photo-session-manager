use psm_core::{
    domain::camera::{Delivery, DeliveryOutcome, ExposureObserved, Link},
    ports::{
        camera::{CaptureSink, CompletedNames},
        clock::Clock,
    },
};
use psm_infra::ftp::{FtpReceiver, ReceiverConfig};
use psm_testkit::{TestApp, clock::ManualClock, ftp_client::FtpClient};
use sha2::{Digest, Sha256};
use std::{
    net::{IpAddr, Ipv4Addr},
    path::PathBuf,
    sync::{Arc, Condvar, Mutex},
    thread,
    time::{Duration, Instant},
};

#[derive(Clone)]
struct LogWriter(Arc<Mutex<Vec<u8>>>);
impl std::io::Write for LogWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[derive(Default)]
struct WaitingSink {
    received: Mutex<Vec<Delivery>>,
    gate: (Mutex<bool>, Condvar),
}
impl WaitingSink {
    fn release(&self) {
        *self.gate.0.lock().unwrap() = true;
        self.gate.1.notify_all();
    }
    fn count(&self) -> usize {
        self.received.lock().unwrap().len()
    }
}
impl CaptureSink for WaitingSink {
    fn exposure_observed(&self, _: ExposureObserved) -> psm_core::Result<i64> {
        Ok(0)
    }
    fn delivery(&self, d: Delivery) -> psm_core::Result<DeliveryOutcome> {
        let mut released = self.gate.0.lock().unwrap();
        while !*released {
            released = self.gate.1.wait(released).unwrap();
        }
        self.received.lock().unwrap().push(d);
        Ok(DeliveryOutcome::Photo)
    }
    fn session_uncertain(&self) -> psm_core::Result<()> {
        Ok(())
    }
    fn transfer_in_progress(&self, _: Link, _: bool) {}
}
impl CompletedNames for WaitingSink {
    fn completed_names_since(&self, _: i64) -> Vec<(String, u64, i64)> {
        vec![]
    }
}
fn receiver(root: PathBuf, sink: Arc<WaitingSink>) -> psm_infra::ftp::ReceiverHandle {
    FtpReceiver::start(ReceiverConfig {
        bind: IpAddr::V4(Ipv4Addr::LOCALHOST),
        port: 0,
        passive: 50000..=50009,
        allowed_peers: vec!["127.0.0.0/8".parse().unwrap()],
        username: "sessionmanager".into(),
        password: "pw".into(),
        capture_root: Arc::new(move || Some(root.clone())),
        sink: sink.clone(),
        names: sink,
        clock: Arc::new(ManualClock::new(0)),
    })
    .unwrap()
}
fn client(port: u16) -> FtpClient {
    let mut c = FtpClient::connect(port).unwrap();
    assert!(c.login("sessionmanager", "pw").unwrap().starts_with("230"));
    c
}
fn wait(mut f: impl FnMut() -> bool) {
    let end = Instant::now() + Duration::from_secs(3);
    while Instant::now() < end {
        if f() {
            return;
        }
        thread::sleep(Duration::from_millis(10));
    }
    assert!(f());
}
fn payload() -> Vec<u8> {
    vec![7; 1024 * 1024]
}
fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

#[test]
fn c1a_reply_precedes_filing() {
    let root = tempfile::tempdir().unwrap();
    let sink = Arc::new(WaitingSink::default());
    let mut server = receiver(root.path().to_path_buf(), sink.clone());
    let mut c = client(server.port());
    let data = payload();
    assert!(c.stor("DSC00001.ARW", &data).unwrap().starts_with("226"));
    assert_eq!(sink.count(), 0);
    sink.release();
    wait(|| sink.count() == 1);
    let d = &sink.received.lock().unwrap()[0];
    assert_eq!(d.size, data.len() as u64);
    assert_eq!(d.sha256, digest(&data));
    server.stop();
}

#[test]
fn c1b_completed_marker_recovered_at_start() {
    let root = tempfile::tempdir().unwrap();
    let incoming = root.path().join(".incoming");
    let data = payload();
    let blocked = Arc::new(WaitingSink::default());
    let mut first = receiver(root.path().to_path_buf(), blocked.clone());
    assert!(
        client(first.port())
            .stor("DSC00001.ARW", &data)
            .unwrap()
            .starts_with("226")
    );
    let marker = incoming.join(".DSC00001.ARW.done");
    assert!(marker.exists());
    first.stop();
    assert!(marker.exists());
    blocked.release();
    let sink = Arc::new(WaitingSink::default());
    sink.release();
    let mut second = receiver(root.path().to_path_buf(), sink.clone());
    wait(|| sink.count() == 1);
    assert_eq!(sink.received.lock().unwrap()[0].sha256, digest(&data));
    second.stop();
}

#[test]
fn c1c_waiting_file_is_present_to_camera() {
    let root = tempfile::tempdir().unwrap();
    let sink = Arc::new(WaitingSink::default());
    let mut server = receiver(root.path().to_path_buf(), sink.clone());
    let mut c = client(server.port());
    let data = payload();
    assert!(c.stor("DSC00001.ARW", &data).unwrap().starts_with("226"));
    assert_eq!(
        c.cmd("SIZE DSC00001.ARW").unwrap(),
        format!("213 {}", data.len())
    );
    sink.release();
    server.stop();
}

fn gap_app() -> TestApp {
    TestApp::with_camera().unwrap()
}
fn deliver(a: &TestApp, name: &str) {
    let path = a
        .capture_root()
        .join(".incoming")
        .join(format!(".{name}.part"));
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, name).unwrap();
    a.ledger()
        .delivery(Delivery {
            camera_filename: name.into(),
            written_path: path,
            sha256: digest(name.as_bytes()),
            size: name.len() as u64,
            via: Link::Wireless,
            completed_at: a.clock.now_ms(),
        })
        .unwrap();
}
#[test]
fn c2a_gap_becomes_header_notice_after_a_minute() {
    let a = gap_app();
    deliver(&a, "DSC00010.ARW");
    deliver(&a, "DSC00013.ARW");
    a.clock.advance(59_000);
    assert!(!a.ledger().watchdog(a.clock.now_ms(), false).unwrap().raised);
    a.clock.advance(2_000);
    let state = a.ledger().watchdog(a.clock.now_ms(), false).unwrap();
    assert!(state.raised);
    assert_eq!(state.pending_count, 2);
    assert!(
        a.app
            .health
            .report("", false, false)
            .items
            .iter()
            .any(|i| i.code == "camera_files_not_received"
                && i.detail.contains("switch FTP Function off and on"))
    );
}
#[test]
fn c2b_gap_closes_on_missing_files() {
    let a = gap_app();
    deliver(&a, "DSC00010.ARW");
    deliver(&a, "DSC00013.ARW");
    a.clock.advance(61_000);
    assert!(a.ledger().watchdog(a.clock.now_ms(), false).unwrap().raised);
    deliver(&a, "DSC00011.ARW");
    assert_eq!(
        a.ledger()
            .watchdog(a.clock.now_ms(), false)
            .unwrap()
            .pending_count,
        1
    );
    deliver(&a, "DSC00012.ARW");
    assert!(!a.ledger().watchdog(a.clock.now_ms(), false).unwrap().raised);
}
#[test]
fn c2c_counter_reset_has_no_gap() {
    let a = gap_app();
    deliver(&a, "DSC09998.ARW");
    deliver(&a, "DSC00002.ARW");
    a.clock.advance(61_000);
    assert!(!a.ledger().watchdog(a.clock.now_ms(), false).unwrap().raised);
}

#[test]
fn c3a_camera_events_and_uploads_are_logged_without_secrets() {
    let log = Arc::new(Mutex::new(Vec::new()));
    let target = log.clone();
    tracing_subscriber::fmt()
        .with_writer(move || LogWriter(target.clone()))
        .with_max_level(tracing::Level::INFO)
        .try_init()
        .unwrap();
    let script = concat!(
        "{\"timestamp\":\"2026-11-06T10:00:00.000Z\",\"event\":\"connection.ready\",\"transport\":\"wifi\",\"password\":\"secret\"}\n",
        "{\"timestamp\":\"2026-11-06T10:00:00.001Z\",\"event\":\"camera.busy\",\"reason\":\"idle\",\"safe_point\":true}\n"
    );
    let a = TestApp::with_camera_script(script).unwrap();
    wait(|| {
        a.tick(0).unwrap();
        String::from_utf8_lossy(&log.lock().unwrap()).contains("camera.event connection.ready")
    });
    let mut c = a.ftp();
    assert!(c.stor("DSC00001.ARW", b"II*\0").unwrap().starts_with("226"));
    wait(|| {
        String::from_utf8_lossy(&log.lock().unwrap())
            .contains("ftp.upload filed name=DSC00001.ARW outcome=photo")
    });
    let lines = String::from_utf8(log.lock().unwrap().clone()).unwrap();
    assert!(lines.contains("camera.event connection.ready") && lines.contains("transport=wifi"));
    assert!(lines.contains("ftp.upload completed name=DSC00001.ARW"));
    assert!(!lines.contains("secret"));
}
