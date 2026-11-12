use psm_core::Error;
use psm_integrations::camera::support::{
    manifest::{Manifest, ManifestFile},
    sony,
};
use psm_testkit::TestApp;
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{Cursor, Write},
    thread,
    time::{Duration, Instant},
};
use zip::{ZipWriter, write::SimpleFileOptions};

const NAMES: [&str; 7] = [
    "libCr_Core.dylib",
    "libmonitor_protocol.dylib",
    "libmonitor_protocol_pf.dylib",
    "CrAdapter/libCr_PTP_USB.dylib",
    "CrAdapter/libCr_PTP_IP.dylib",
    "CrAdapter/libssh2.dylib",
    "CrAdapter/libusb-1.0.0.dylib",
];

fn archive(a: &TestApp, omit: Option<&str>, change: Option<&str>) -> std::path::PathBuf {
    let mut nested = ZipWriter::new(Cursor::new(Vec::new()));
    for name in NAMES {
        if Some(name) == omit {
            continue;
        }
        nested
            .start_file(
                format!("external/crsdk/{name}"),
                SimpleFileOptions::default(),
            )
            .unwrap();
        nested
            .write_all(if Some(name) == change {
                b"wrong"
            } else {
                name.as_bytes()
            })
            .unwrap();
    }
    let bytes = nested.finish().unwrap().into_inner();
    let path = a.dir.path().join("CrSDK_v2.02.00_Mac.zip");
    let mut outer = ZipWriter::new(fs::File::create(&path).unwrap());
    outer
        .start_file(
            "CrSDK_v2.02.00_Mac/RemoteCli.zip",
            SimpleFileOptions::default(),
        )
        .unwrap();
    outer.write_all(&bytes).unwrap();
    outer.finish().unwrap();
    path
}

fn setup() -> (TestApp, std::path::PathBuf) {
    let a = TestApp::with_fake_document().unwrap();
    a.with_test_manifest(Manifest {
        version: "2.02.00".into(),
        files: NAMES
            .iter()
            .map(|name| ManifestFile {
                path: (*name).into(),
                sha256: Sha256::digest(name.as_bytes())
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect(),
            })
            .collect(),
    });
    let bundled = a.bundled_sidecar_path();
    (a, bundled)
}

#[test]
fn c1a_sony_download_installs_seven_verified_files() {
    let (a, bundled) = setup();
    let installed = sony::install(&a.app.paths, &archive(&a, None, None), &bundled).unwrap();
    for name in NAMES {
        assert_eq!(
            fs::read(installed.folder.join(name)).unwrap(),
            name.as_bytes()
        );
    }
}

#[test]
fn c1b_missing_and_changed_are_distinct() {
    let (a, bundled) = setup();
    let file = "CrAdapter/libCr_PTP_IP.dylib";
    assert!(
        matches!(sony::install(&a.app.paths, &archive(&a, Some(file), None), &bundled),
        Err(Error::SupportArchiveInvalid { file: message }) if message == format!("missing {file}"))
    );
    assert!(
        matches!(sony::install(&a.app.paths, &archive(&a, None, Some(file)), &bundled),
        Err(Error::SupportArchiveInvalid { file: message }) if message == format!("changed {file}"))
    );
}

#[test]
fn c4a_first_unexpected_exit_restarts_within_300ms_then_backs_off() {
    let a = TestApp::with_camera_behaviour(serde_json::json!({"exit_after_ms":0})).unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    while a.fake_sidecar_recording().pids.is_empty() && Instant::now() < deadline {
        a.tick(0).unwrap();
        thread::sleep(Duration::from_millis(10));
    }
    let first = a.fake_sidecar_recording().pids[0];
    while a
        .fake_sidecar_recording()
        .exits
        .iter()
        .all(|(pid, _)| *pid != first)
        && Instant::now() < deadline
    {
        a.tick(0).unwrap();
        thread::sleep(Duration::from_millis(10));
    }
    let start = Instant::now();
    while a.fake_sidecar_recording().pids.len() < 2 && start.elapsed() < Duration::from_millis(300)
    {
        a.tick(0).unwrap();
        thread::sleep(Duration::from_millis(10));
    }
    assert!(
        a.fake_sidecar_recording().pids.len() >= 2,
        "first restart took over 300 ms"
    );
    // The second consecutive failure must wait for the existing one-second backoff.
    let second = a.fake_sidecar_recording().pids[1];
    while a
        .fake_sidecar_recording()
        .exits
        .iter()
        .all(|(pid, _)| *pid != second)
        && Instant::now() < deadline
    {
        a.tick(0).unwrap();
        thread::sleep(Duration::from_millis(10));
    }
    for _ in 0..5 {
        a.tick(0).unwrap();
        thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(a.fake_sidecar_recording().pids.len(), 2);
    a.tick(1000).unwrap();
    while a.fake_sidecar_recording().pids.len() < 3 && Instant::now() < deadline {
        a.tick(0).unwrap();
        thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(a.fake_sidecar_recording().pids.len(), 3);
}
