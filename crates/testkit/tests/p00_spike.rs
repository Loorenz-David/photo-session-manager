use psm_core::{Error, ports::store::Store};
use psm_infra::{
    osascript::OsaRunner, paths::AppPaths, spool::Spool, sqlite::store::SqliteStore,
    watcher::FsWatcher,
};
use psm_integrations::{App, editing::capture_one::hooks};
use psm_testkit::TestApp;
use psm_testkit::clock::ManualClock;
use std::{
    fs,
    io::{Read, Write},
    net::TcpStream,
    os::unix::fs::PermissionsExt,
    path::Path,
    process::Command,
    thread,
    time::{Duration, Instant},
};

const PROCESSING: &str =
    r#"{"hook":"processing_done","job":"J1","raw":"/tmp/a.ARW","outputs":["/tmp/o.jpg"]}"#;

fn wait_for(mut condition: impl FnMut() -> bool) -> bool {
    let deadline = Instant::now() + Duration::from_secs(2);
    while Instant::now() < deadline {
        if condition() {
            return true;
        }
        thread::sleep(Duration::from_millis(20));
    }
    false
}

fn hook_files(dir: &Path, hook: &str) -> Vec<std::path::PathBuf> {
    fs::read_dir(dir)
        .unwrap()
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| {
            path.file_name()
                .unwrap()
                .to_string_lossy()
                .ends_with(&format!("-{hook}.json"))
        })
        .collect()
}

#[test]
fn c1a_first_open_creates_layout_and_defaults() {
    let app = TestApp::new().unwrap();
    let root = app.dir.path();
    for name in [
        "session_manager.db",
        "settings.json",
        "previews",
        "hooks",
        "spool",
        "spool/bad",
        "logs",
        "backup",
    ] {
        assert!(root.join(name).exists(), "missing {name}");
    }
    assert_eq!(
        fs::metadata(root.join("settings.json"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
    let settings: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("settings.json")).unwrap()).unwrap();
    assert_eq!(settings["document_id"], serde_json::Value::Null);
    assert_eq!(settings["preview_recipe"], "PSM Preview");
    assert_eq!(settings["web_recipe"], "PSM Web");
    assert_eq!(settings["style_name"], "PSM House");
    assert_eq!(
        settings["export_folder"],
        format!(
            "{}/Pictures/Session Manager Export",
            std::env::var("HOME").unwrap()
        )
    );
    assert_eq!(settings["preview_delay_s"], 15);
    assert_eq!(settings["theme"], "light");
    assert!(settings.get("sku_prefix").is_none());
    assert_eq!(settings["integration"], serde_json::Value::Null);
    assert_eq!(app.app.store.mechanism_version().unwrap(), "1");
}

#[test]
fn c1b_second_open_preserves_settings_and_migration() {
    let dir = tempfile::tempdir().unwrap();
    let paths = AppPaths::for_dir(dir.path());
    drop(App::open(paths.clone()).unwrap());
    let mut settings: serde_json::Value =
        serde_json::from_slice(&fs::read(paths.settings()).unwrap()).unwrap();
    settings["preview_delay_s"] = 20.into();
    let edited = serde_json::to_vec_pretty(&settings).unwrap();
    fs::write(paths.settings(), &edited).unwrap();
    let second = App::open(paths.clone()).unwrap();
    assert_eq!(fs::read(paths.settings()).unwrap(), edited);
    assert_eq!(second.store.mechanism_version().unwrap(), "1");
    assert_eq!(second.store.migration_version().unwrap(), 1);
}

#[test]
fn c2a_osascript_uses_data_directory_as_cwd() {
    let dir = tempfile::tempdir().unwrap();
    let script = dir.path().join("cwd.js");
    fs::write(
        &script,
        "$.NSFileManager.defaultManager.currentDirectoryPath.js",
    )
    .unwrap();
    let output = OsaRunner::new(dir.path())
        .run(&script, &[], Duration::from_secs(2))
        .unwrap();
    assert_eq!(output, dir.path().to_string_lossy());
}

#[test]
fn c2b_osascript_times_out_and_reaps_child() {
    let dir = tempfile::tempdir().unwrap();
    let script = dir.path().join("delay.js");
    fs::write(&script, "delay(3)").unwrap();
    let start = Instant::now();
    let result = OsaRunner::new(dir.path()).run(&script, &[], Duration::from_secs(1));
    assert!(matches!(result, Err(Error::Timeout)));
    assert!(start.elapsed() < Duration::from_secs(2));
    let processes = Command::new("pgrep")
        .args(["-fl", &script.to_string_lossy()])
        .output()
        .unwrap();
    assert!(
        processes.stdout.is_empty(),
        "child remains: {}",
        String::from_utf8_lossy(&processes.stdout)
    );
}

#[test]
fn c3a_compile_all_produces_three_scripts() {
    let dir = tempfile::tempdir().unwrap();
    let paths = AppPaths::for_dir(dir.path());
    paths.create().unwrap();
    hooks::compile_all(&paths, 9).unwrap();
    for name in ["capture_done", "processing_done", "variant_adjusted"] {
        assert!(
            fs::metadata(paths.hooks().join(format!("{name}.scpt")))
                .unwrap()
                .len()
                > 0
        );
    }
}

#[test]
fn c3b_processing_hook_returns_and_spools_without_listener() {
    let dir = tempfile::tempdir().unwrap();
    let paths = AppPaths::for_dir(dir.path());
    paths.create().unwrap();
    hooks::compile_all(&paths, 9).unwrap();
    let start = Instant::now();
    let output = Command::new("osascript")
        .arg(paths.hooks().join("processing_done.scpt"))
        .args(["J1", "/tmp/a.ARW", "/tmp/o.jpg"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(start.elapsed() < Duration::from_secs(2));
    let files = hook_files(&paths.spool(), "processing_done");
    assert_eq!(files.len(), 1);
    let value: serde_json::Value = serde_json::from_slice(&fs::read(&files[0]).unwrap()).unwrap();
    assert_eq!(
        value,
        serde_json::from_str::<serde_json::Value>(PROCESSING).unwrap()
    );
}

#[test]
fn c3c_capture_hook_spools_argv() {
    let dir = tempfile::tempdir().unwrap();
    let paths = AppPaths::for_dir(dir.path());
    paths.create().unwrap();
    hooks::compile_all(&paths, 9).unwrap();
    let output = Command::new("osascript")
        .arg(paths.hooks().join("capture_done.scpt"))
        .arg("/tmp/a.ARW")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let files = hook_files(&paths.spool(), "capture_done");
    assert_eq!(files.len(), 1);
    let value: serde_json::Value = serde_json::from_slice(&fs::read(&files[0]).unwrap()).unwrap();
    assert_eq!(
        value,
        serde_json::json!({"hook":"capture_done","argv":["/tmp/a.ARW"]})
    );
}

#[test]
fn c3d_processing_hook_escapes_quote_and_backslash() {
    let dir = tempfile::tempdir().unwrap();
    let paths = AppPaths::for_dir(dir.path());
    paths.create().unwrap();
    hooks::compile_all(&paths, 9).unwrap();
    let raw = "/tmp/we\"ird\\x.ARW";
    let output = Command::new("osascript")
        .arg(paths.hooks().join("processing_done.scpt"))
        .args(["J1", raw, "/tmp/o.jpg"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let files = hook_files(&paths.spool(), "processing_done");
    assert_eq!(files.len(), 1);
    let value: serde_json::Value = serde_json::from_slice(&fs::read(&files[0]).unwrap()).unwrap();
    assert_eq!(value["raw"], raw);
}

#[test]
fn c4a_valid_file_becomes_one_event_and_is_deleted() {
    let app = TestApp::new().unwrap();
    app.app
        .spool
        .write_for_test("0001-processing_done.json", PROCESSING)
        .unwrap();
    app.app
        .spool
        .drain(app.app.store.as_ref(), &app.clock)
        .unwrap();
    assert_eq!(
        app.app.store.event_rows().unwrap(),
        vec![("hook".into(), "processing_done".into(), PROCESSING.into())]
    );
    assert!(
        !app.app
            .paths
            .spool()
            .join("0001-processing_done.json")
            .exists()
    );
}

#[test]
fn c4b_malformed_file_goes_to_bad() {
    let app = TestApp::new().unwrap();
    app.app.spool.write_for_test("0001-bad.json", "{").unwrap();
    app.app
        .spool
        .drain(app.app.store.as_ref(), &app.clock)
        .unwrap();
    assert!(app.app.paths.spool().join("bad/0001-bad.json").exists());
    assert!(app.app.store.event_rows().unwrap().is_empty());
}

#[test]
fn c4c_drain_uses_filename_order() {
    let dir = tempfile::tempdir().unwrap();
    let paths = AppPaths::for_dir(dir.path());
    paths.create().unwrap();
    let store = SqliteStore::open(&paths).unwrap();
    let spool = Spool::new(paths);
    spool
        .write_for_test("0002-later.json", &PROCESSING.replace("J1", "J2"))
        .unwrap();
    spool
        .write_for_test("0001-earlier.json", PROCESSING)
        .unwrap();
    spool.drain(&store, &ManualClock::new(0)).unwrap();
    let rows = store.event_rows().unwrap();
    let later = PROCESSING.replace("J1", "J2");
    assert_eq!(
        rows.iter().map(|r| r.2.as_str()).collect::<Vec<_>>(),
        vec![PROCESSING, later.as_str()]
    );
}

#[test]
fn c4d_open_drains_file_left_while_closed() {
    let dir = tempfile::tempdir().unwrap();
    let paths = AppPaths::for_dir(dir.path());
    paths.create().unwrap();
    fs::write(paths.spool().join("0001-processing_done.json"), PROCESSING).unwrap();
    let app = App::open(paths.clone()).unwrap();
    assert_eq!(app.store.event_rows().unwrap().len(), 1);
    assert!(!paths.spool().join("0001-processing_done.json").exists());
}

fn request(port: u16, method: &str, body: &str) -> String {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
    stream.write_all(format!("{method} /nudge HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).as_bytes()).unwrap();
    let mut output = String::new();
    stream.read_to_string(&mut output).unwrap();
    output
}

#[test]
fn c5a_nudge_consumes_spool_file() {
    let app = TestApp::new().unwrap();
    app.app
        .spool
        .write_for_test("0001-processing_done.json", PROCESSING)
        .unwrap();
    let response = request(app.app.nudge_port(), "GET", "");
    assert!(response.starts_with("HTTP/1.1 204"));
    assert!(wait_for(|| app.app.store.event_rows().unwrap().len() == 1));
    assert!(
        !app.app
            .paths
            .spool()
            .join("0001-processing_done.json")
            .exists()
    );
}

#[test]
fn c5b_nudge_body_cannot_inject_event() {
    let app = TestApp::new().unwrap();
    let body = r#"{"raw":"/tmp/inject.ARW"}"#;
    let response = request(app.app.nudge_port(), "POST", body);
    assert!(response.starts_with("HTTP/1.1 204"));
    assert!(app.app.store.event_rows().unwrap().is_empty());
    assert_eq!(app.app.nudge_addr().ip().to_string(), "127.0.0.1");
}

#[test]
fn c6a_watcher_emits_absolute_path() {
    let dir = tempfile::tempdir().unwrap();
    let (_watcher, mut rx) = FsWatcher::watch(dir.path()).unwrap();
    let path = dir.path().join("x.ARW");
    fs::write(&path, b"raw").unwrap();
    assert!(wait_for(|| {
        while let Ok(event) = rx.try_recv() {
            if event.path == path {
                return true;
            }
        }
        false
    }));
}

#[test]
fn c7a_generated_bindings_expose_commands() {
    let bindings = fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../apps/desktop/src/services/bindings.ts"
    ))
    .unwrap();
    assert!(bindings.contains("healthGet"));
    assert!(bindings.contains("appPathsGet"));
}
