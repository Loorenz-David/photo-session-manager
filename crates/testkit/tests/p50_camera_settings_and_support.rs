use psm_core::{
    Error,
    domain::camera::{CameraStatus, Link, NeedsActionReason, secrets},
    events::AppEvent,
    ports::secrets::SecretStore,
    services::camera::CameraService,
};
use psm_integrations::camera::support::{
    self,
    manifest::{Manifest, ManifestFile},
};
use psm_testkit::TestApp;
use sha2::{Digest, Sha256};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::Path,
    process::Command,
    thread,
    time::{Duration, Instant},
};

const NAMES: [&str; 7] = [
    "libCr_Core.dylib",
    "libmonitor_protocol.dylib",
    "libmonitor_protocol_pf.dylib",
    "CrAdapter/libCr_PTP_USB.dylib",
    "CrAdapter/libCr_PTP_IP.dylib",
    "CrAdapter/libssh2.dylib",
    "CrAdapter/libusb-1.0.0.dylib",
];
fn entries() -> Vec<(&'static str, &'static [u8])> {
    NAMES.iter().map(|name| (*name, name.as_bytes())).collect()
}
fn manifest() -> Manifest {
    Manifest {
        version: "2.02.00".into(),
        files: entries()
            .iter()
            .map(|(path, bytes)| ManifestFile {
                path: (*path).into(),
                sha256: Sha256::digest(bytes)
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect(),
            })
            .collect(),
    }
}
fn configured() -> (TestApp, std::path::PathBuf) {
    let mut a = TestApp::with_fake_document().unwrap();
    let port = std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    a.settings_set(&serde_json::json!({"camera":{"receiver_port":port,"passive_port_start":50000,"passive_port_end":50009}})).unwrap();
    a.with_test_manifest(manifest());
    let bundled = a.bundled_sidecar_path();
    (a, bundled)
}
fn assert_no_install(a: &TestApp) {
    assert!(support::installed(&a.app.paths).is_none());
    let root = a.app.paths.root.join("camera");
    assert!(!root.join("sony").exists());
    assert!(
        fs::read_dir(root)
            .map(|mut i| i.all(|e| !e
                .unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with(".install-")))
            .unwrap_or(true)
    );
}

#[test]
fn c1a_archive_install_has_signed_unquarantined_runtime_and_sidecar() {
    let (mut a, bundled) = configured();
    let archive = a.support_archive(&entries());
    let _ = Command::new("xattr")
        .args(["-w", "com.apple.quarantine", "0083;test"])
        .arg(&archive)
        .status();
    let installed = a.camera_support_install(&archive, &bundled).unwrap();
    assert_eq!(installed.make, "sony");
    assert_eq!(installed.version, "2.02.00");
    for (name, bytes) in entries() {
        let path = installed.folder.join(name);
        assert_eq!(fs::read(&path).unwrap(), bytes);
        assert!(
            !String::from_utf8_lossy(
                &Command::new("xattr")
                    .arg("-l")
                    .arg(&path)
                    .output()
                    .unwrap()
                    .stdout
            )
            .contains("com.apple.quarantine")
        );
        if let Some(rest) = name.strip_prefix("CrAdapter/") {
            let framework = installed
                .folder
                .join("Contents/Frameworks/CrAdapter")
                .join(rest);
            assert_eq!(fs::read(&framework).unwrap(), bytes);
            assert!(
                !String::from_utf8_lossy(
                    &Command::new("xattr")
                        .arg("-l")
                        .arg(&framework)
                        .output()
                        .unwrap()
                        .stdout
                )
                .contains("com.apple.quarantine")
            );
        }
    }
    assert_eq!(
        Sha256::digest(fs::read(&installed.sidecar_path).unwrap()).as_slice(),
        Sha256::digest(fs::read(&bundled).unwrap()).as_slice()
    );
    assert_ne!(
        fs::metadata(&installed.sidecar_path)
            .unwrap()
            .permissions()
            .mode()
            & 0o111,
        0
    );
    assert!(
        !String::from_utf8_lossy(
            &Command::new("xattr")
                .arg("-l")
                .arg(&installed.sidecar_path)
                .output()
                .unwrap()
                .stdout
        )
        .contains("com.apple.quarantine")
    );
    assert!(
        Command::new("codesign")
            .arg("--verify")
            .arg(&installed.sidecar_path)
            .status()
            .unwrap()
            .success()
    );
    assert_eq!(support::installed(&a.app.paths).unwrap().version, "2.02.00");
}
#[test]
fn c1b_install_starts_its_sidecar_and_receiver() {
    let (mut a, bundled) = configured();
    let archive = a.support_archive(&entries());
    let installed = a.camera_support_install(&archive, &bundled).unwrap();
    a.tick(0).unwrap();
    let rec = fs::read_to_string(installed.folder.join("rec/pids")).unwrap();
    assert_eq!(rec.lines().count(), 1);
    let pid = rec.trim();
    let argv: Vec<String> = serde_json::from_slice(
        &fs::read(installed.folder.join(format!("rec/argv.{pid}.json"))).unwrap(),
    )
    .unwrap();
    assert_eq!(argv[0], installed.sidecar_path.to_string_lossy());
    assert_ne!(a.app.camera_runtime.receiver_port().unwrap_or(0), 0);
    assert!(!a.app.camera.capabilities().depends_on_editing_provider);
}

#[test]
fn c4a_damaged_support_falls_back_with_health() {
    let (mut a, bundled) = configured();
    let archive = a.support_archive(&entries());
    let installed = a.camera_support_install(&archive, &bundled).unwrap();
    let before = fs::read_to_string(installed.folder.join("rec/pids")).unwrap_or_default();
    fs::remove_file(installed.folder.join("libmonitor_protocol_pf.dylib")).unwrap();
    a.kill_and_restart().unwrap();
    assert!(support::installed(&a.app.paths).is_none());
    assert!(a.app.camera.capabilities().depends_on_editing_provider);
    assert!(a.app.health.has("camera_support_not_installed"));
    let item = a
        .app
        .health
        .report("", false, false)
        .items
        .into_iter()
        .find(|i| i.code == "camera_support_not_installed")
        .unwrap();
    assert_eq!(
        item.detail,
        "Camera support for Sony is damaged: add it again in Settings → Camera"
    );
    assert_eq!(
        fs::read_to_string(installed.folder.join("rec/pids")).unwrap_or_default(),
        before
    );
}
#[test]
fn c3a_update_refreshes_copy_once() {
    let (mut a, bundled) = configured();
    let archive = a.support_archive(&entries());
    let installed = a.camera_support_install(&archive, &bundled).unwrap();
    let second_build = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../apps/desktop/src-tauri/binaries/camera-sidecar-aarch64-apple-darwin");
    fs::copy(second_build, &bundled).unwrap();
    let plist = a.dir.path().join("entitlements.plist");
    assert!(
        Command::new("codesign")
            .args([
                "-s",
                "-",
                "--force",
                "--options",
                "runtime",
                "--entitlements"
            ])
            .arg(plist)
            .arg(&bundled)
            .status()
            .unwrap()
            .success()
    );
    a.kill_and_restart().unwrap();
    assert_eq!(
        Sha256::digest(fs::read(&installed.sidecar_path).unwrap()).as_slice(),
        Sha256::digest(fs::read(&bundled).unwrap()).as_slice()
    );
    assert!(
        Command::new("codesign")
            .arg("--verify")
            .arg(&installed.sidecar_path)
            .status()
            .unwrap()
            .success()
    );
    let modified = fs::metadata(&installed.sidecar_path)
        .unwrap()
        .modified()
        .unwrap();
    a.kill_and_restart().unwrap();
    assert_eq!(
        fs::metadata(&installed.sidecar_path)
            .unwrap()
            .modified()
            .unwrap(),
        modified
    );
}

#[test]
fn c2a_missing_file_names_it_and_leaves_no_install() {
    let (a, bundled) = configured();
    let rows: Vec<_> = entries()
        .into_iter()
        .filter(|(n, _)| *n != "CrAdapter/libssh2.dylib")
        .collect();
    let archive = a.support_archive(&rows);
    assert!(
        matches!(support::sony::install(&a.app.paths, &archive, &bundled), Err(Error::SupportArchiveInvalid { file }) if file == "missing CrAdapter/libssh2.dylib")
    );
    assert_no_install(&a);
}
#[test]
fn c2b_wrong_hash_names_the_file() {
    let (a, bundled) = configured();
    let mut rows = entries();
    rows[0].1 = b"changed";
    let archive = a.support_archive(&rows);
    assert!(
        matches!(support::sony::install(&a.app.paths, &archive, &bundled), Err(Error::SupportArchiveInvalid { file }) if file == "changed libCr_Core.dylib")
    );
    assert_no_install(&a);
}
#[test]
fn c2e_text_archive_is_refused_by_name() {
    let (a, bundled) = configured();
    let archive = a.dir.path().join("bad.txt");
    fs::write(&archive, "not a zip").unwrap();
    assert!(
        matches!(support::sony::install(&a.app.paths, &archive, &bundled), Err(Error::SupportArchiveInvalid { file }) if file == "bad.txt")
    );
    assert_no_install(&a);
}
#[test]
fn c2c_unsupported_sdk_version_is_refused() {
    let (a, bundled) = configured();
    fs::write(
        bundled.parent().unwrap().join("rec/behaviour.json"),
        "{\"version_sdk\":\"2.01.00\"}",
    )
    .unwrap();
    let archive = a.support_archive(&entries());
    assert!(
        matches!(support::sony::install(&a.app.paths, &archive, &bundled), Err(Error::SupportVersionUnsupported { version }) if version == "2.01.00")
    );
    assert_no_install(&a);
}
#[test]
fn c2d_stub_sdk_version_is_refused() {
    let (a, bundled) = configured();
    fs::write(
        bundled.parent().unwrap().join("rec/behaviour.json"),
        "{\"version_sdk\":null}",
    )
    .unwrap();
    let archive = a.support_archive(&entries());
    assert!(
        matches!(support::sony::install(&a.app.paths, &archive, &bundled), Err(Error::SupportVersionUnsupported { version }) if version == "unavailable")
    );
    assert_no_install(&a);
}
#[test]
fn c4b_no_support_is_normal_capture_one_mode() {
    let a = TestApp::with_fake_document().unwrap();
    assert!(support::installed(&a.app.paths).is_none());
    assert!(!a.app.health.has("camera_support_not_installed"));
    assert!(a.app.camera.capabilities().depends_on_editing_provider);
}

#[test]
fn c5e_no_support_does_not_read_keychain_secrets() {
    let a = TestApp::with_fake_document().unwrap();
    assert!(support::installed(&a.app.paths).is_none());

    a.tick(0).unwrap();
    thread::sleep(Duration::from_millis(100));

    assert_eq!(a.secrets.get_count(), 0);
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
    panic!("camera condition did not settle: {:?}", a.camera_status());
}
fn setting_secrets(a: &TestApp, user: &str, password: &str) {
    CameraService::set_secrets(a.secrets.as_ref(), user, password, || {
        a.camera_runtime().secrets_changed();
    })
    .unwrap();
}
#[test]
fn c5d_startup_keychain_read_restarts_after_usb_session_lost() {
    let a = TestApp::with_camera_script_no_cache(include_str!(
        "../src/fixtures/sidecar_scripts/ready_usb.txt"
    ))
    .unwrap();
    settle(&a, || {
        matches!(
            a.camera_status().unwrap(),
            CameraStatus::Connected {
                link: Link::Cable,
                ..
            }
        )
    });
    // Give the asynchronous Keychain-like read time to finish while USB still owns the camera.
    thread::sleep(Duration::from_millis(500));
    let first = a.fake_sidecar_recording().pids[0];
    assert!(
        a.fake_sidecar_recording()
            .stdin
            .iter()
            .any(|(pid, line)| { *pid == first && line == "credentials user= password=" })
    );
    a.inject(
        "session.lost",
        serde_json::json!({"transport":"usb","reason":"lost","busy":false}),
    )
    .unwrap();
    let end = Instant::now() + Duration::from_secs(3);
    while Instant::now() < end && a.fake_sidecar_recording().pids.len() < 2 {
        a.tick(1_000).unwrap();
        thread::sleep(Duration::from_millis(10));
    }
    let recording = a.fake_sidecar_recording();
    assert_eq!(recording.pids.len(), 2);
    let second = recording.pids[1];
    settle(&a, || {
        a.fake_sidecar_recording()
            .stdin
            .iter()
            .any(|(pid, line)| *pid == second && line == "credentials user=u%20name password=p%2Fw")
    });
}
#[test]
fn c5a_camera_credentials_are_keychain_only_and_redacted() {
    let a = TestApp::with_camera().unwrap();
    setting_secrets(&a, "u2", "p2");
    assert_eq!(
        a.secrets.get(secrets::ACCESS_AUTH_USER).unwrap().as_deref(),
        Some("u2")
    );
    assert_eq!(
        a.secrets
            .get(secrets::ACCESS_AUTH_PASSWORD)
            .unwrap()
            .as_deref(),
        Some("p2")
    );
    let settings = fs::read_to_string(a.app.paths.settings()).unwrap();
    assert!(!settings.contains("u2"));
    assert!(!settings.contains("p2"));
    assert_eq!(
        psm_infra::logging::redact_registered("x p2 y"),
        "x [REDACTED] y"
    );
}
#[test]
fn c5b_wifi_credentials_change_restarts_with_new_first_line() {
    let a = TestApp::with_camera_script(include_str!(
        "../src/fixtures/sidecar_scripts/ready_wifi.txt"
    ))
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
    let first = a.fake_sidecar_recording().pids[0];
    setting_secrets(&a, "u3", "p3");
    settle(&a, || a.fake_sidecar_recording().pids.len() == 2);
    let rec = a.fake_sidecar_recording();
    assert!(
        rec.stdin
            .iter()
            .any(|(pid, line)| *pid == first && line == "quit")
    );
    let second = rec.pids[1];
    settle(&a, || {
        a.fake_sidecar_recording()
            .stdin
            .iter()
            .any(|(pid, line)| *pid == second && line == "credentials user=u3 password=p3")
    });
}
#[test]
fn c5c_usb_change_waits_for_next_spawn() {
    let a = TestApp::with_camera_script(include_str!(
        "../src/fixtures/sidecar_scripts/ready_usb.txt"
    ))
    .unwrap();
    settle(&a, || {
        matches!(
            a.camera_status().unwrap(),
            CameraStatus::Connected {
                link: Link::Cable,
                ..
            }
        )
    });
    setting_secrets(&a, "u4", "p4");
    let first = a.fake_sidecar_recording().pids[0];
    thread::sleep(Duration::from_millis(500));
    a.tick(0).unwrap();
    let rec = a.fake_sidecar_recording();
    assert_eq!(rec.pids, vec![first]);
    assert!(!rec.stdin.iter().any(|(_, line)| line == "quit"));
    a.inject(
        "session.lost",
        serde_json::json!({"transport":"usb","reason":"lost","busy":false}),
    )
    .unwrap();
    a.inject_exit(0).unwrap();
    let end = Instant::now() + Duration::from_secs(3);
    while Instant::now() < end && a.fake_sidecar_recording().pids.len() < 2 {
        a.tick(1_000).unwrap();
        thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(a.fake_sidecar_recording().pids.len(), 2);
    let second = a.fake_sidecar_recording().pids[1];
    settle(&a, || {
        a.fake_sidecar_recording()
            .stdin
            .iter()
            .any(|(pid, line)| *pid == second && line == "credentials user=u4 password=p4")
    });
}
fn generate(a: &TestApp) -> String {
    let mut bytes = [0u8; 64];
    let mut pos = bytes.len();
    CameraService::generate_ftp_password(
        a.secrets.as_ref(),
        || {
            if pos == bytes.len() {
                getrandom::fill(&mut bytes).unwrap();
                pos = 0;
            }
            let out = bytes[pos];
            pos += 1;
            out
        },
        |pw| {
            psm_infra::logging::add_secret(pw);
            a.camera_runtime().ftp_password_changed();
        },
    )
    .unwrap()
}
#[test]
fn c6b_generated_password_reopens_listener_and_is_stored_only_in_keychain() {
    let a = TestApp::with_camera_script(include_str!(
        "../src/fixtures/sidecar_scripts/ready_wifi.txt"
    ))
    .unwrap();
    settle(&a, || a.camera_runtime().receiver_port().is_some());
    let port = a.receiver_port();
    let generation = a.camera_runtime().receiver_generation();
    let pids = a.fake_sidecar_recording().pids;
    let pw = generate(&a);
    assert_eq!(pw.len(), 32);
    assert_eq!(
        a.secrets.get(secrets::FTP_PASSWORD).unwrap().as_deref(),
        Some(pw.as_str())
    );
    assert_eq!(a.camera_runtime().receiver_generation(), generation + 1);
    assert_eq!(a.receiver_port(), port);
    let mut ftp = psm_testkit::ftp_client::FtpClient::connect(port).unwrap();
    assert!(ftp.login("sessionmanager", "f").unwrap().starts_with("530"));
    assert!(ftp.login("sessionmanager", &pw).unwrap().starts_with("230"));
    assert_eq!(a.fake_sidecar_recording().pids, pids);
    assert!(
        !fs::read_to_string(a.app.paths.settings())
            .unwrap()
            .contains(&pw)
    );
}
#[test]
fn c6c_regeneration_invalidates_old_password() {
    let a = TestApp::with_camera().unwrap();
    let first = generate(&a);
    let second = generate(&a);
    assert_ne!(first, second);
    let mut ftp = psm_testkit::ftp_client::FtpClient::connect(a.receiver_port()).unwrap();
    assert!(
        ftp.login("sessionmanager", &first)
            .unwrap()
            .starts_with("530")
    );
    assert!(
        ftp.login("sessionmanager", &second)
            .unwrap()
            .starts_with("230")
    );
}
#[test]
fn c7a_unknown_fingerprint_raises_event_status_and_health() {
    let a = TestApp::with_camera_script(include_str!(
        "../src/fixtures/sidecar_scripts/fingerprint_unknown.txt"
    ))
    .unwrap();
    settle(&a, || {
        a.events
            .0
            .lock()
            .unwrap()
            .iter()
            .any(|e| matches!(e, AppEvent::FingerprintPending { .. }))
    });
    assert!(a.events.0.lock().unwrap().iter().any(
        |e| matches!(e, AppEvent::FingerprintPending { fingerprint, replace: false } if fingerprint == "ab:cd:ef")
    ));
    assert!(matches!(
        a.camera_status().unwrap(),
        CameraStatus::NeedsAction {
            reason: NeedsActionReason::FingerprintUnconfirmed,
            ..
        }
    ));
    let item = a
        .app
        .health
        .report("", false, false)
        .items
        .into_iter()
        .find(|i| i.code == "camera_fingerprint_unconfirmed")
        .unwrap();
    assert_eq!(item.detail, "Confirm the camera's fingerprint");
}
#[test]
fn c7b_confirmation_sends_accept_and_pin_clears_notice() {
    let a = TestApp::with_camera_script(include_str!(
        "../src/fixtures/sidecar_scripts/fingerprint_unknown.txt"
    ))
    .unwrap();
    settle(&a, || a.app.health.has("camera_fingerprint_unconfirmed"));
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
    a.inject(
        "ssh.fingerprint",
        serde_json::json!({"status":"pinned","success":true,"fingerprint":"ab:cd:ef"}),
    )
    .unwrap();
    settle(&a, || !a.app.health.has("camera_fingerprint_unconfirmed"));
}
#[test]
fn c7c_rejection_keeps_pending_and_sends_nothing() {
    let a = TestApp::with_camera_script(include_str!(
        "../src/fixtures/sidecar_scripts/fingerprint_unknown.txt"
    ))
    .unwrap();
    settle(&a, || a.app.health.has("camera_fingerprint_unconfirmed"));
    let before = a.fake_sidecar_recording().stdin;
    a.camera_runtime()
        .direct_adapter()
        .reject_fingerprint()
        .unwrap();
    thread::sleep(Duration::from_millis(500));
    a.tick(0).unwrap();
    assert_eq!(a.fake_sidecar_recording().stdin, before);
    assert!(a.app.health.has("camera_fingerprint_unconfirmed"));
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
}
#[test]
fn c7d_mismatch_confirmation_replaces() {
    let a = TestApp::with_camera_script("{\"timestamp\":\"2026-11-06T10:00:00Z\",\"source\":\"sdk\",\"event\":\"ssh.fingerprint\",\"status\":\"MISMATCH\",\"success\":false,\"fingerprint\":\"ef:12\"}\n").unwrap();
    settle(&a, || a.app.health.has("camera_fingerprint_mismatch"));
    assert!(matches!(
        a.camera_status().unwrap(),
        CameraStatus::NeedsAction {
            reason: NeedsActionReason::FingerprintMismatch,
            ..
        }
    ));
    let item = a
        .app
        .health
        .report("", false, false)
        .items
        .into_iter()
        .find(|i| i.code == "camera_fingerprint_mismatch")
        .unwrap();
    assert_eq!(
        item.detail,
        "The camera's fingerprint changed: confirm it in Settings → Camera before shooting over Wi-Fi"
    );
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
fn c7e_auth_refusal_never_sends_a_retry() {
    let a = TestApp::with_camera_script(include_str!(
        "../src/fixtures/sidecar_scripts/auth_refused.txt"
    ))
    .unwrap();
    settle(&a, || {
        matches!(
            a.camera_status().unwrap(),
            CameraStatus::NeedsAction {
                reason: NeedsActionReason::AuthenticationRefused,
                ..
            }
        )
    });
    a.tick(30_000).unwrap();
    let lines = a.fake_sidecar_recording().stdin;
    assert!(!lines.iter().any(|(_, line)| line.starts_with("usb-retry")
        || line.starts_with("reconnect")
        || line.starts_with("accept-fingerprint")));
}
