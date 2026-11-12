//! P45 — camera sidecar import, build, and the protocol client (10E MC-31).
use psm_integrations::camera::sidecar::{
    client::CommandOutcome,
    protocol::{Command, SidecarEvent, Transport},
};
use psm_testkit::TestApp;
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Read,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command as Process, Output, Stdio},
    sync::OnceLock,
    thread,
    time::{Duration, Instant},
};

const LAB_LINES: &str = include_str!("../../src/fixtures/sidecar_scripts/lab_lines.txt");
const CAPTURE: &str = include_str!("../../src/fixtures/sidecar_scripts/capture.txt");
const CAPTURE_ID: &str = "11111111-2222-4333-8444-555555555555";

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}
fn sidecar_dir() -> PathBuf {
    repo().join("native/camera-sidecar")
}
fn installed_binary() -> PathBuf {
    repo().join("apps/desktop/src-tauri/binaries/camera-sidecar-aarch64-apple-darwin")
}
/// `tools/sidecar/build.sh --stub --test`, run once per test process (C2(a), C2(b), C6 share the
/// cmake build dir and the `binaries/` file).
fn build() -> &'static Output {
    static BUILD: OnceLock<Output> = OnceLock::new();
    BUILD.get_or_init(|| {
        Process::new("bash")
            .arg("tools/sidecar/build.sh")
            .args(["--stub", "--test"])
            .current_dir(repo())
            .output()
            .expect("run tools/sidecar/build.sh")
    })
}
fn build_stdout() -> String {
    String::from_utf8_lossy(&build().stdout).into_owned()
}
fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        out.push(path.clone());
        if path.is_dir() {
            walk(&path, out);
        }
    }
}
fn source_text(dir: &Path) -> Vec<(PathBuf, String)> {
    let mut files = Vec::new();
    walk(dir, &mut files);
    files
        .into_iter()
        .filter(|p| p.is_file())
        .map(|p| {
            let text = fs::read_to_string(&p).unwrap();
            (p, text)
        })
        .collect()
}
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
        "sidecar result did not settle: stdin={:?} events={:?} log={:?} exits={:?}",
        a.fake_sidecar_recording().stdin,
        a.sidecar_events(),
        a.camera_runtime().helper_log(),
        a.fake_sidecar_recording().exits
    );
}
fn events(a: &TestApp) -> Vec<SidecarEvent> {
    a.sidecar_events().into_iter().map(|s| s.event).collect()
}
fn s(v: &str) -> String {
    v.to_owned()
}

#[test]
fn c1a_no_vendor_file_in_tree() {
    let root = sidecar_dir();
    let mut paths = Vec::new();
    walk(&root, &mut paths);
    let forbidden = |name: &str| {
        name == "CameraRemote_SDK.h"
            || (name.starts_with("Cr") && name.ends_with(".h"))
            || name.ends_with(".dylib")
            || name.ends_with(".a")
            || name.ends_with(".framework")
            || name.contains("sdk_shim")
            || name == "vendor"
            || name.starts_with(".env")
            || name.ends_with(".fingerprint")
            || name.to_ascii_uppercase().ends_with(".ARW")
    };
    let offending: Vec<_> = paths
        .iter()
        .filter(|p| forbidden(&p.file_name().unwrap().to_string_lossy()))
        .collect();
    assert!(offending.is_empty(), "vendor-like paths: {offending:?}");
    for name in [
        "include/crlab/camera.hpp",
        "include/crlab/json_log.hpp",
        "include/crlab/sha256.hpp",
        "include/crlab/transport.hpp",
        "include/crlab/usb_monitor.hpp",
        "src/sdk_stub.cpp",
    ] {
        assert!(root.join(name).is_file(), "{name} missing");
    }
}

#[test]
fn c1b_ignore_rules() {
    let gitignore = fs::read_to_string(repo().join(".gitignore")).unwrap();
    let lines: Vec<&str> = gitignore.lines().map(str::trim).collect();
    for entry in [
        "apps/desktop/src-tauri/binaries/",
        "native/camera-sidecar/build/",
        "*.fingerprint",
        "vendor/",
        ".env",
    ] {
        assert!(lines.contains(&entry), "{entry} not ignored");
    }
}

#[test]
fn c1c_no_secret_env_read() {
    let sources = source_text(&sidecar_dir().join("src"));
    let all: String = sources.iter().map(|(_, t)| t.as_str()).collect();
    assert_eq!(all.matches("CAMERA_SSH_USER").count(), 0);
    assert_eq!(all.matches("CAMERA_SSH_PASSWORD").count(), 0);
    assert_eq!(all.matches("\"CRLAB_FINGERPRINT_DIR\"").count(), 1);
    // Every environment read names its variable literally; none is a credential.
    let mut names = Vec::new();
    for (_, text) in &sources {
        for call in ["env(\"", "getenv(\""] {
            for (i, _) in text.match_indices(call) {
                let rest = &text[i + call.len()..];
                names.push(rest[..rest.find('"').unwrap()].to_owned());
            }
        }
    }
    assert!(names.iter().any(|n| n == "CRLAB_FINGERPRINT_DIR"));
    for name in &names {
        let upper = name.to_ascii_uppercase();
        assert!(
            !["SSH", "PASSWORD", "USER", "CREDENTIAL", "SECRET"]
                .iter()
                .any(|bad| upper.contains(bad)),
            "credential read from the environment: {name}"
        );
    }
}

#[test]
fn c1d_host_pc_alone_never_written() {
    const NAME: &str = "CrStillImageStoreDestination_HostPC";
    let mut alone = Vec::new();
    for (path, text) in source_text(&sidecar_dir().join("src")) {
        for (i, _) in text.match_indices(NAME) {
            if !text[i + NAME.len()..].starts_with("AndMemoryCard") {
                alone.push(path.clone());
            }
        }
    }
    assert!(alone.is_empty(), "Host PC alone in {alone:?}");
}

#[test]
fn c2a_stub_build_and_cpp_tests() {
    let out = build();
    let stdout = build_stdout();
    assert!(
        out.status.success(),
        "build failed: {stdout}\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let bin = installed_binary();
    assert!(bin.is_file());
    assert_ne!(fs::metadata(&bin).unwrap().permissions().mode() & 0o111, 0);
    assert!(stdout.contains("0 failed"), "{stdout}");
    assert!(stdout.contains("100% tests passed out of 1"), "{stdout}");
}

#[test]
fn c2b_stub_refuses_camera_work() {
    assert!(build().status.success());
    let version = Process::new(installed_binary())
        .arg("version")
        .output()
        .unwrap();
    let text = String::from_utf8(version.stdout).unwrap();
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 1, "{text}");
    let line: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
    assert_eq!(line["event"], "crlab.version");
    assert_eq!(line["sdk_compiled_in"], false);

    let tmp = tempfile::tempdir().unwrap();
    let started = Instant::now();
    let mut child = Process::new(installed_binary())
        .args(["auto", "--save-dir"])
        .arg(tmp.path().join("x"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        assert!(
            started.elapsed() < Duration::from_secs(2),
            "auto did not exit within 2 s"
        );
        thread::sleep(Duration::from_millis(10));
    };
    assert!(started.elapsed() < Duration::from_secs(2));
    assert_eq!(status.code(), Some(4));
    let mut out = String::new();
    child
        .stdout
        .take()
        .unwrap()
        .read_to_string(&mut out)
        .unwrap();
    assert!(
        out.lines().any(|l| {
            serde_json::from_str::<serde_json::Value>(l)
                .is_ok_and(|v| v["event"] == "sdk.unavailable")
        }),
        "{out}"
    );
}

#[test]
fn c2c_manifest_over_synthetic_tree() {
    let t = tempfile::tempdir().unwrap();
    let files = [
        "lib/libCr_Core.dylib",
        "lib/libmonitor_protocol.dylib",
        "lib/libmonitor_protocol_pf.dylib",
        "lib/CrAdapter/libCr_PTP_IP.dylib",
        "lib/CrAdapter/libCr_PTP_USB.dylib",
        "README.txt",
    ];
    for rel in files {
        let path = t.path().join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, rel.as_bytes()).unwrap();
    }
    let out_dir = tempfile::tempdir().unwrap();
    let out = out_dir.path().join("sony-9.99.99.json");
    let before = {
        let mut p = Vec::new();
        walk(&sidecar_dir(), &mut p);
        p
    };
    let status = Process::new("bash")
        .arg(repo().join("tools/sidecar/manifest.sh"))
        .arg(t.path())
        .arg("9.99.99")
        .arg(&out)
        .status()
        .unwrap();
    assert!(status.success());
    let manifest: serde_json::Value = serde_json::from_slice(&fs::read(&out).unwrap()).unwrap();
    assert_eq!(manifest["version"], "9.99.99");
    let sha = |bytes: &[u8]| -> String {
        Sha256::digest(bytes)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect()
    };
    let expected = serde_json::json!([
        {"path": "CrAdapter/libCr_PTP_IP.dylib", "sha256": sha(b"lib/CrAdapter/libCr_PTP_IP.dylib")},
        {"path": "CrAdapter/libCr_PTP_USB.dylib", "sha256": sha(b"lib/CrAdapter/libCr_PTP_USB.dylib")},
        {"path": "libCr_Core.dylib", "sha256": sha(b"lib/libCr_Core.dylib")},
        {"path": "libmonitor_protocol.dylib", "sha256": sha(b"lib/libmonitor_protocol.dylib")},
        {"path": "libmonitor_protocol_pf.dylib", "sha256": sha(b"lib/libmonitor_protocol_pf.dylib")},
    ]);
    assert_eq!(manifest["files"], expected);
    let after = {
        let mut p = Vec::new();
        walk(&sidecar_dir(), &mut p);
        p
    };
    assert_eq!(
        before, after,
        "manifest.sh wrote under native/camera-sidecar"
    );
}

#[test]
fn c5a_events_flow_in_order() {
    let a = TestApp::with_camera_script(LAB_LINES).unwrap();
    settle(&a, || a.sidecar_events().len() >= 13);
    let id = s("53d7…");
    let expected = vec![
        SidecarEvent::TransportState {
            from: s("DISCONNECTED"),
            to: s("DISCOVERING"),
            reason: s("looking for camera over wifi"),
            transport: Transport::None,
        },
        SidecarEvent::ConnectionReady {
            transport: Transport::Wifi,
            connect_ms: Some(914),
            model: None,
            store_destination: None,
            store_value: None,
        },
        SidecarEvent::CmdReceived {
            cmd: s("capture"),
            cmd_id: Some(s("4db6be91")),
        },
        SidecarEvent::CameraBusy {
            reason: s("exposure"),
            safe_point: false,
        },
        SidecarEvent::CaptureRequested {
            capture_id: id.clone(),
            transport: Transport::Wifi,
            af: true,
        },
        SidecarEvent::CaptureAf {
            capture_id: id.clone(),
            af_result: s("focused"),
        },
        SidecarEvent::CaptureAcknowledged {
            capture_id: id.clone(),
            success: true,
            sdk_error: s("0x00000000"),
        },
        SidecarEvent::CaptureExposed {
            capture_id: Some(id),
            transport: Transport::Wifi,
            association: s("latest_release_provisional"),
        },
        SidecarEvent::CmdDone {
            cmd: s("capture"),
            cmd_id: Some(s("4db6be91")),
        },
        SidecarEvent::UsbArrived {
            detail: s("pid=0x0E0C ILCE-7RM5"),
        },
        SidecarEvent::SessionClosed {
            transport: Transport::Wifi,
            reason: s("switching wifi -> usb"),
        },
        SidecarEvent::SessionLost {
            transport: Transport::Usb,
            reason: s("usb device removed"),
            busy: s("none"),
        },
        SidecarEvent::SwitchDeferred {
            to: s("usb"),
            busy: s("pixel_shift"),
        },
    ];
    assert_eq!(events(&a), expected);
    let rec = a.fake_sidecar_recording();
    assert_eq!(
        rec.stdin.first().map(|(_, line)| line.as_str()),
        Some("credentials user=u%20name password=p%2Fw")
    );
}

#[test]
fn c5b_command_correlation() {
    let a = TestApp::with_camera_script(CAPTURE).unwrap();
    let rt = a.camera_runtime();
    let id = rt
        .send_command(Command::Capture {
            id: s(CAPTURE_ID),
            af: true,
            cmd_id: String::new(),
        })
        .unwrap();
    assert!(!id.is_empty());
    settle(&a, || {
        events(&a)
            .iter()
            .any(|e| matches!(e, SidecarEvent::CmdDone { .. }))
    });
    let wire = format!("capture id={CAPTURE_ID} af=1 cmd_id={id}");
    let rec = a.fake_sidecar_recording();
    assert_eq!(rec.stdin.iter().filter(|(_, l)| *l == wire).count(), 1);
    assert_eq!(rt.command_outcome(&id), Some(CommandOutcome::Done));
    let exposed: Vec<_> = events(&a)
        .into_iter()
        .filter_map(|e| match e {
            SidecarEvent::CaptureExposed { capture_id, .. } => Some(capture_id),
            _ => None,
        })
        .collect();
    assert_eq!(exposed, vec![Some(s(CAPTURE_ID))]);
}

#[test]
fn c5c_rejected_command() {
    let script = r#"await capture
{"timestamp":"2026-11-06T10:00:00.000Z","source":"sdk","event":"cmd.received","cmd":"capture","cmd_id":"{cmd_id}"}
{"timestamp":"2026-11-06T10:00:00.001Z","source":"sdk","event":"cmd.rejected","cmd":"capture","reason":"not connected (DISCOVERING)"}
{"timestamp":"2026-11-06T10:00:00.002Z","source":"sdk","event":"cmd.done","cmd":"capture","cmd_id":"{cmd_id}"}
"#;
    let a = TestApp::with_camera_script(script).unwrap();
    let rt = a.camera_runtime();
    let id = rt
        .send_command(Command::Capture {
            id: s(CAPTURE_ID),
            af: true,
            cmd_id: String::new(),
        })
        .unwrap();
    settle(&a, || {
        events(&a)
            .iter()
            .any(|e| matches!(e, SidecarEvent::CmdDone { .. }))
    });
    assert_eq!(
        rt.command_outcome(&id),
        Some(CommandOutcome::Rejected {
            reason: s("not connected (DISCOVERING)")
        })
    );
}

#[test]
fn c5d_malformed_line_does_not_stop_stream() {
    let script = r#"{bad
{"timestamp":"2026-11-06T10:00:00.000Z","source":"sdk","event":"transport.state","from":"DISCONNECTED","to":"DISCOVERING","reason":"looking for camera over usb","transport":"none"}
"#;
    let a = TestApp::with_camera_script(script).unwrap();
    settle(&a, || !a.sidecar_events().is_empty());
    assert_eq!(
        events(&a),
        vec![SidecarEvent::TransportState {
            from: s("DISCONNECTED"),
            to: s("DISCOVERING"),
            reason: s("looking for camera over usb"),
            transport: Transport::None,
        }]
    );
    let unparsable: Vec<_> = a
        .camera_runtime()
        .helper_log()
        .into_iter()
        .filter(|l| l.contains("unparsable sidecar line"))
        .collect();
    assert_eq!(unparsable.len(), 1, "{unparsable:?}");
}

/// C6 rows are C++ `TEST_CASE`s run through ctest by `build()` (C2(a)); each passes by name.
fn cpp_case_passed(name: &str) {
    let out = build();
    assert!(out.status.success());
    let stdout = build_stdout();
    assert!(
        stdout.contains(&format!("[ OK ] {name}\n")),
        "{name}: {stdout}"
    );
}

#[test]
fn c6a_usb_retry_after_fallback() {
    cpp_case_passed("usb-retry gives usb a fresh chance without a re-plug");
}

#[test]
fn c6b_stop_waits_for_safe_point() {
    cpp_case_passed("stop is deferred while busy, bounded by 10 s");
}

#[test]
fn c6c_credentials_percent_decoding() {
    cpp_case_passed("credentials percent-decoding reverses the app's encoding");
}

#[test]
fn c7a_packaging_wiring() {
    let tauri = repo().join("apps/desktop/src-tauri");
    let conf: serde_json::Value =
        serde_json::from_slice(&fs::read(tauri.join("tauri.conf.json")).unwrap()).unwrap();
    assert_eq!(
        conf["bundle"]["externalBin"],
        serde_json::json!(["binaries/camera-sidecar"])
    );
    let plist = fs::read_to_string(tauri.join("Info.plist")).unwrap();
    let key = "<key>NSLocalNetworkUsageDescription</key>";
    let after = &plist[plist.find(key).expect("NSLocalNetworkUsageDescription") + key.len()..];
    let value = after
        .trim_start()
        .strip_prefix("<string>")
        .and_then(|v| v.split_once("</string>"))
        .map(|(v, _)| v.trim())
        .expect("a <string> value");
    assert!(!value.is_empty());
}
