use psm_core::{domain::camera::Link, ports::clock::Clock};
use psm_infra::ftp::{FtpReceiver, ReceiverConfig};
use psm_testkit::{
    TestApp, clock::ManualClock, collecting_sink::CollectingSink, ftp_client::FtpClient,
};
use sha2::{Digest, Sha256};
use std::{
    io::{Read, Write},
    net::{IpAddr, Ipv4Addr, TcpListener, TcpStream},
    path::Path,
    sync::Arc,
    time::Duration,
};

#[derive(Clone)]
struct LogWriter(Arc<std::sync::Mutex<Vec<u8>>>);
impl Write for LogWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn app() -> TestApp {
    TestApp::with_camera().unwrap()
}
fn bytes(n: usize, seed: u8) -> Vec<u8> {
    let mut b = vec![seed; n];
    b[..4].copy_from_slice(b"II*\0");
    b
}
fn part(root: &Path, name: &str) -> std::path::PathBuf {
    root.join(".incoming").join(format!(".{name}.part"))
}
/// Where the capture ledger (P44) puts a completed upload while no session is active.
fn unassigned(root: &Path, name: &str) -> std::path::PathBuf {
    root.join("_Unassigned").join(name)
}
fn code(s: &str) -> &str {
    &s[..3]
}
fn digest(data: &[u8]) -> String {
    Sha256::digest(data)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
fn receiver(
    root: std::path::PathBuf,
    peers: Vec<ipnet::IpNet>,
    passive: std::ops::RangeInclusive<u16>,
    sink: Arc<CollectingSink>,
) -> psm_infra::ftp::ReceiverHandle {
    FtpReceiver::start(ReceiverConfig {
        bind: IpAddr::V4(Ipv4Addr::LOCALHOST),
        port: 0,
        passive,
        allowed_peers: peers,
        username: "sessionmanager".into(),
        password: "ftp-pw-unique-1".into(),
        capture_root: Arc::new(move || Some(root.clone())),
        sink: sink.clone(),
        names: sink,
        clock: Arc::new(ManualClock::new(0)),
    })
    .unwrap()
}
fn local() -> Vec<ipnet::IpNet> {
    vec!["127.0.0.0/8".parse().unwrap()]
}
fn client(port: u16) -> FtpClient {
    let mut c = FtpClient::connect(port).unwrap();
    assert_eq!(
        code(&c.login("sessionmanager", "ftp-pw-unique-1").unwrap()),
        "230"
    );
    c
}
fn settle(mut f: impl FnMut() -> bool) {
    for _ in 0..300 {
        if f() {
            return;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    panic!("condition did not settle");
}

fn settle_for(timeout: Duration, mut f: impl FnMut() -> bool) {
    let deadline = std::time::Instant::now() + timeout;
    while std::time::Instant::now() < deadline {
        if f() {
            return;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(f(), "condition did not settle within {timeout:?}");
}

#[test]
fn c1a_completed_upload_integrity() {
    let a = app();
    let data = bytes(3 * 1024 * 1024, 1);
    let mut c = a.ftp();
    assert_eq!(code(&c.stor("DSC00001.ARW", &data).unwrap()), "226");
    assert_eq!(
        std::fs::read(unassigned(&a.capture_root(), "DSC00001.ARW")).unwrap(),
        data
    );
    assert_eq!(
        a.deliveries(),
        vec![psm_core::domain::camera::Delivery {
            camera_filename: "DSC00001.ARW".into(),
            written_path: part(&a.capture_root(), "DSC00001.ARW"),
            sha256: digest(&data),
            size: data.len() as u64,
            via: Link::Wireless,
            completed_at: a.clock.now_ms()
        }]
    );
    assert!(!a.capture_root().join("DSC00001.ARW").exists());
}
#[test]
fn c1b_virtual_subdirectory() {
    let a = app();
    let mut c = a.ftp();
    assert_eq!(code(&c.cmd("MKD 2026-11-05").unwrap()), "257");
    assert_eq!(code(&c.cmd("CWD 2026-11-05").unwrap()), "250");
    assert_eq!(
        code(&c.stor("DSC00003.ARW", &bytes(65536, 3)).unwrap()),
        "226"
    );
    assert!(unassigned(&a.capture_root(), "DSC00003.ARW").exists());
    assert_eq!(
        a.deliveries().last().unwrap().camera_filename,
        "DSC00003.ARW"
    );
}
#[test]
fn c1c_two_concurrent_uploads() {
    let a = app();
    let mut c1 = a.ftp();
    let mut c2 = a.ftp();
    let x = bytes(1024 * 1024, 4);
    let y = bytes(1024 * 1024, 5);
    let (_, mut d1) = c1.open_stor("A.ARW").unwrap();
    let mut d1 = d1.take().unwrap();
    d1.write_all(&x[..x.len() / 2]).unwrap();
    assert_eq!(code(&c2.stor("B.ARW", &y).unwrap()), "226");
    d1.write_all(&x[x.len() / 2..]).unwrap();
    assert_eq!(code(&c1.finish_stor(d1).unwrap()), "226");
    let d = a.deliveries();
    assert_eq!(d.len(), 2);
    assert!(
        d.iter()
            .any(|v| v.camera_filename == "A.ARW" && v.sha256 == digest(&x))
    );
    assert!(
        d.iter()
            .any(|v| v.camera_filename == "B.ARW" && v.sha256 == digest(&y))
    );
}
#[test]
fn c2a_reset_is_failure() {
    let a = app();
    let mut c = a.ftp();
    let data = bytes(4 * 1024 * 1024, 6);
    assert_eq!(
        code(&c.stor_rst("DSC00020.ARW", &data[..1024 * 1024]).unwrap()),
        "426"
    );
    assert!(a.deliveries().is_empty());
    let n = std::fs::metadata(part(&a.capture_root(), "DSC00020.ARW"))
        .unwrap()
        .len();
    assert!(n > 0 && n <= 1024 * 1024);
    assert!(!a.capture_root().join("DSC00020.ARW").exists());
}
#[test]
fn c2b_rest_resume() {
    let a = app();
    let mut c = a.ftp();
    let data = bytes(4 * 1024 * 1024, 6);
    assert_eq!(
        code(&c.stor_rst("DSC00020.ARW", &data[..1024 * 1024]).unwrap()),
        "426"
    );
    let off = std::fs::metadata(part(&a.capture_root(), "DSC00020.ARW"))
        .unwrap()
        .len();
    let (rest, done) = c
        .stor_resume("DSC00020.ARW", off, &data[off as usize..])
        .unwrap();
    assert_eq!(code(&rest), "350");
    assert_eq!(code(&done), "226");
    assert_eq!(
        std::fs::read(unassigned(&a.capture_root(), "DSC00020.ARW")).unwrap(),
        data
    );
    assert_eq!(a.deliveries().len(), 1);
    assert_eq!(a.deliveries()[0].sha256, digest(&data));
}
#[test]
fn c2c_rest_past_length() {
    let a = app();
    let mut c = a.ftp();
    assert_eq!(
        code(
            &c.stor_rst("X.ARW", &bytes(1024 * 1024, 7)[..65536])
                .unwrap()
        ),
        "426"
    );
    let path = part(&a.capture_root(), "X.ARW");
    let before = std::fs::read(&path).unwrap();
    let (_, r) = c.stor_resume("X.ARW", 2 * 1024 * 1024, b"tail").unwrap();
    assert_eq!(code(&r), "554");
    assert_eq!(std::fs::read(path).unwrap(), before);
    assert!(a.deliveries().is_empty());
}
#[test]
fn c2d_new_stor_truncates() {
    let a = app();
    let mut c = a.ftp();
    assert_eq!(code(&c.stor_rst("X.ARW", &bytes(65536, 7)).unwrap()), "426");
    let data = bytes(10240, 8);
    assert_eq!(code(&c.stor("X.ARW", &data).unwrap()), "226");
    assert_eq!(
        std::fs::read(unassigned(&a.capture_root(), "X.ARW")).unwrap(),
        data
    );
    assert_eq!(a.deliveries()[0].size, 10240);
}
#[test]
fn c2e_same_name_in_flight() {
    let a = app();
    let mut c1 = a.ftp();
    let mut c2 = a.ftp();
    let (_, mut d) = c1.open_stor("Y.ARW").unwrap();
    let mut d = d.take().unwrap();
    d.write_all(b"x").unwrap();
    assert_eq!(code(&c2.stor("Y.ARW", b"zz").unwrap()), "450");
    d.write_all(b"tail").unwrap();
    assert_eq!(code(&c1.finish_stor(d).unwrap()), "226");
    assert_eq!(a.deliveries().len(), 1);
}
#[test]
fn c2f_control_close_aborts() {
    let a = app();
    let mut c = a.ftp();
    let (_, mut d) = c.open_stor("Z.ARW").unwrap();
    let mut d = d.take().unwrap();
    d.write_all(&bytes(1024, 1)).unwrap();
    c.close_control();
    settle(|| a.camera_runtime().uploads_in_progress() == 0);
    assert!(a.deliveries().is_empty());
    assert!(part(&a.capture_root(), "Z.ARW").exists());
    drop(d);
}
#[test]
fn c2g_appe_at_staged_length() {
    let a = app();
    let mut c = a.ftp();
    let data = bytes(1024 * 1024, 2);
    assert_eq!(code(&c.stor_rst("W.ARW", &data[..65536]).unwrap()), "426");
    let off = std::fs::metadata(part(&a.capture_root(), "W.ARW"))
        .unwrap()
        .len() as usize;
    assert_eq!(code(&c.appe("W.ARW", &data[off..]).unwrap()), "226");
    assert_eq!(
        std::fs::read(unassigned(&a.capture_root(), "W.ARW")).unwrap(),
        data
    );
}
#[test]
fn c3a_delete_and_rename_refused() {
    let a = app();
    let mut c = a.ftp();
    c.stor("A.ARW", b"a").unwrap();
    c.stor("B.ARW", b"b").unwrap();
    let a0 = std::fs::read(unassigned(&a.capture_root(), "A.ARW")).unwrap();
    let b0 = std::fs::read(unassigned(&a.capture_root(), "B.ARW")).unwrap();
    for cmd in ["DELE A.ARW", "RMD x", "RNFR A.ARW"] {
        assert_eq!(code(&c.cmd(cmd).unwrap()), "550");
    }
    assert_eq!(
        std::fs::read(unassigned(&a.capture_root(), "A.ARW")).unwrap(),
        a0
    );
    assert_eq!(
        std::fs::read(unassigned(&a.capture_root(), "B.ARW")).unwrap(),
        b0
    );
}
#[test]
fn c3b_rnto_alone() {
    let a = app();
    assert_eq!(code(&a.ftp().cmd("RNTO C.ARW").unwrap()), "553");
}
#[test]
fn c3c_hidden_names_refused() {
    let a = app();
    let mut c = a.ftp();
    for name in [".DSC00030.ARW.part", ".hidden", "Q.ARW.part"] {
        assert_eq!(code(&c.stor(name, b"x").unwrap()), "553");
        assert!(!part(&a.capture_root(), name).exists());
    }
    assert!(a.deliveries().is_empty());
}
#[test]
fn c3d_traversal_refused() {
    let a = app();
    let mut c = a.ftp();
    for name in ["../escape.ARW", "/../../escape.ARW"] {
        assert_eq!(code(&c.stor(name, b"II*\0").unwrap()), "553");
    }
    assert!(!a.dir.path().join("escape.ARW").exists());
}
/// A standalone receiver whose listings come from `sink` (P44: a `TestApp`'s receiver lists the
/// ledger's names), on `clock`.
fn listing_receiver(
    sink: Arc<CollectingSink>,
    clock: Arc<ManualClock>,
) -> (tempfile::TempDir, psm_infra::ftp::ReceiverHandle, FtpClient) {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().to_path_buf();
    let handle = FtpReceiver::start(ReceiverConfig {
        bind: IpAddr::V4(Ipv4Addr::LOCALHOST),
        port: 0,
        passive: 0..=0,
        allowed_peers: local(),
        username: "sessionmanager".into(),
        password: "ftp-pw-unique-1".into(),
        capture_root: Arc::new(move || Some(root.clone())),
        sink: sink.clone(),
        names: sink,
        clock,
    })
    .unwrap();
    let c = client(handle.port());
    (dir, handle, c)
}
#[test]
fn c4a_virtual_listings() {
    let sink = Arc::new(CollectingSink::default());
    let clock = Arc::new(ManualClock::new(0));
    let t = clock.now_ms();
    sink.set_completed_names(vec![
        ("DSC00001.ARW".into(), 3145728, t),
        ("DSC00003.ARW".into(), 65536, t),
    ]);
    let (_dir, _handle, mut c) = listing_receiver(sink, clock);
    c.cmd("CWD anything").unwrap();
    let (_, nlst) = c.listing("NLST").unwrap();
    assert_eq!(
        nlst.lines().collect::<Vec<_>>(),
        vec!["DSC00001.ARW", "DSC00003.ARW", "226 Transfer complete"]
    );
    let (_, list) = c.listing("LIST").unwrap();
    let lines = list.lines().collect::<Vec<_>>();
    assert_eq!(lines.len(), 3);
    assert!(lines[0].contains("3145728") && lines[0].ends_with("DSC00001.ARW"));
    assert!(lines[1].contains("65536") && lines[1].ends_with("DSC00003.ARW"));
}
#[test]
fn c4b_size_and_mdtm() {
    let sink = Arc::new(CollectingSink::default());
    let clock = Arc::new(ManualClock::new(0));
    let t = 1_700_000_000_000;
    sink.set_completed_names(vec![("DSC00001.ARW".into(), 3145728, t)]);
    clock.advance(t);
    let (_dir, _handle, mut c) = listing_receiver(sink, clock);
    assert_eq!(c.cmd("SIZE DSC00001.ARW").unwrap(), "213 3145728");
    assert_eq!(c.cmd("MDTM DSC00001.ARW").unwrap(), "213 20231114221320");
    assert_eq!(code(&c.cmd("SIZE NOPE.ARW").unwrap()), "550");
}
#[test]
fn c4c_old_names_expire() {
    let sink = Arc::new(CollectingSink::default());
    let clock = Arc::new(ManualClock::new(0));
    sink.set_completed_names(vec![("OLD.ARW".into(), 42, 0)]);
    clock.advance(25 * 3600 * 1000);
    let (_dir, _handle, mut c) = listing_receiver(sink, clock);
    let (_, names) = c.listing("NLST").unwrap();
    assert_eq!(names, "226 Transfer complete");
    assert_eq!(code(&c.cmd("SIZE OLD.ARW").unwrap()), "550");
}
#[test]
fn c5b_public_policy_refuses_loopback() {
    let dir = tempfile::tempdir().unwrap();
    let sink = Arc::new(CollectingSink::default());
    let handle = receiver(
        dir.path().into(),
        vec!["10.0.0.0/8".parse().unwrap()],
        0..=0,
        sink,
    );
    let mut stream = TcpStream::connect(("127.0.0.1", handle.port())).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    let mut data = Vec::new();
    stream.read_to_end(&mut data).unwrap();
    assert!(
        String::from_utf8(data)
            .unwrap()
            .starts_with("421 Connections from this network are not allowed.")
    );
}
#[test]
fn c5c_auth_and_secret_redaction() {
    let captured = Arc::new(std::sync::Mutex::new(Vec::new()));
    let writer = captured.clone();
    let subscriber = tracing_subscriber::fmt()
        .with_writer(move || LogWriter(writer.clone()))
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();
    let a = app();
    let mut c = FtpClient::connect(a.receiver_port()).unwrap();
    assert_eq!(code(&c.login("sessionmanager", "wrong").unwrap()), "530");
    assert_eq!(
        code(&c.login("sessionmanager", "ftp-pw-unique-1").unwrap()),
        "230"
    );
    let mut other = FtpClient::connect(a.receiver_port()).unwrap();
    assert_eq!(
        code(&other.login("other", "ftp-pw-unique-1").unwrap()),
        "530"
    );
    let logs = a.camera_runtime().helper_log().join("\n");
    assert!(!logs.contains("wrong") && !logs.contains("ftp-pw-unique-1"));
    let trace = String::from_utf8(captured.lock().unwrap().clone()).unwrap();
    assert!(!trace.contains("wrong") && !trace.contains("ftp-pw-unique-1"));
}
#[test]
fn c5d_passive_only_and_range() {
    let a = app();
    let mut c = a.ftp();
    assert_eq!(code(&c.cmd("PORT 127,0,0,1,4,1").unwrap()), "502");
    assert_eq!(code(&c.cmd("EPRT |1|127.0.0.1|1025|").unwrap()), "502");
    let dir = tempfile::tempdir().unwrap();
    let sink = Arc::new(CollectingSink::default());
    let handle = receiver(dir.path().into(), local(), 50000..=50009, sink);
    let mut c = client(handle.port());
    assert!((50000..=50009).contains(&c.pasv().unwrap()));
}
#[test]
fn c6a_missing_capture_root() {
    let dir = tempfile::tempdir().unwrap();
    let sink = Arc::new(CollectingSink::default());
    let handle = receiver(dir.path().join("missing"), local(), 0..=0, sink.clone());
    assert_eq!(
        code(&client(handle.port()).stor("DSC1.ARW", b"II*\0").unwrap()),
        "451"
    );
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 0);
    assert!(sink.deliveries().is_empty());
}
#[test]
fn c6b_capture_root_from_settings() {
    let a = app();
    {
        let mut fake = a.fake.state.lock().unwrap();
        fake.running = false;
        fake.ops.clear();
    }
    let mut c = a.ftp();
    assert_eq!(code(&c.stor("DSC2.ARW", b"II*\0").unwrap()), "226");
    assert!(unassigned(&a.capture_root(), "DSC2.ARW").exists());
    assert!(
        !a.fake
            .state
            .lock()
            .unwrap()
            .ops
            .iter()
            .any(|op| op.contains("document_info"))
    );
}
#[test]
fn c7a_camera_settings_validation() {
    let mut a = app();
    let before = a.app.settings.camera;
    for camera in [
        serde_json::json!({"receiver_port":2121,"passive_port_start":50000,"passive_port_end":50000}),
        serde_json::json!({"receiver_port":2121,"passive_port_start":50000,"passive_port_end":49999}),
        serde_json::json!({"receiver_port":50005,"passive_port_start":50000,"passive_port_end":50009}),
        serde_json::json!({"receiver_port":80,"passive_port_start":50000,"passive_port_end":50009}),
    ] {
        assert!(matches!(
            a.settings_set(&serde_json::json!({"camera":camera})),
            Err(psm_core::Error::InvalidValue(_))
        ));
        assert_eq!(a.app.settings.camera, before);
    }
    let patch = serde_json::json!({"camera":{"receiver_port":2222,"passive_port_start":51000,"passive_port_end":51009}});
    let value = a.settings_set(&patch).unwrap();
    assert_eq!(value["camera"], patch["camera"]);
}
#[test]
fn c7b_port_reopens_in_place() {
    let mut a = app();
    let p0 = a.receiver_port();
    settle_for(Duration::from_secs(15), || {
        !a.fake_sidecar_recording().pids.is_empty()
    });
    let pid = a.fake_sidecar_recording().pids;
    let generation = a.camera_runtime().receiver_generation();
    let free = TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let patch = serde_json::json!({"camera":{"receiver_port":free,"passive_port_start":51000,"passive_port_end":51009}});
    a.settings_set(&patch).unwrap();
    assert!(TcpStream::connect(("127.0.0.1", p0)).is_err());
    assert_eq!(a.receiver_port(), free);
    assert_eq!(a.camera_runtime().receiver_generation(), generation + 1);
    assert_eq!(a.fake_sidecar_recording().pids, pid);
}
#[test]
fn c7c_unrelated_save_keeps_connection() {
    let mut a = app();
    let mut c = a.ftp();
    let port = a.receiver_port();
    let generation = a.camera_runtime().receiver_generation();
    a.settings_set(&serde_json::json!({"sku_prefix":"PX"}))
        .unwrap();
    assert_eq!(code(&c.noop().unwrap()), "200");
    assert_eq!(a.receiver_port(), port);
    assert_eq!(a.camera_runtime().receiver_generation(), generation);
}
#[test]
fn c8a_shutdown_keeps_staging() {
    let a = app();
    let mut c = a.ftp();
    let (_, mut d) = c.open_stor("V.ARW").unwrap();
    d.as_mut().unwrap().write_all(&bytes(1024, 2)).unwrap();
    settle(|| part(&a.capture_root(), "V.ARW").exists());
    a.camera_runtime().shutdown();
    assert!(a.deliveries().is_empty());
    assert!(part(&a.capture_root(), "V.ARW").exists());
    assert!(c.noop().map(|s| s.is_empty()).unwrap_or(true));
}
