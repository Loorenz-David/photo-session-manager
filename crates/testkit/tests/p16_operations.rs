use psm_core::{
    domain::{
        photo::{Classification, NewPhoto, PhotoSource},
        session::PublishStep,
    },
    mechanisms::hash::sha256_file,
    ports::clock::Clock,
};
use psm_testkit::TestApp;
use serde_json::{Value, json};
use std::{fs, os::unix::fs::PermissionsExt};

fn files(dir: &std::path::Path) -> Vec<String> {
    let mut names = fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    names.sort();
    names
}

fn stored(app: &TestApp) -> Value {
    serde_json::from_slice(&fs::read(app.app.paths.settings()).unwrap()).unwrap()
}

fn mode(app: &TestApp) -> u32 {
    fs::metadata(app.app.paths.settings())
        .unwrap()
        .permissions()
        .mode()
        & 0o777
}

fn shopify(secret: &str) -> Value {
    json!({"kind":"shopify","shop_domain":"example.myshopify.com","api_version":"2026-10",
        "client_id":"ID","client_secret":secret,"condition_photos_key":"custom.damage_report"})
}

#[test]
fn c1a_settings_store_delay_as_given_and_read_it_clamped() {
    let mut app = TestApp::new().unwrap();
    app.app
        .settings_set(&json!({"preview_delay_s": 40, "export_folder": "/tmp/x"}))
        .unwrap();
    let shown = app.app.settings_get().unwrap();
    assert_eq!(stored(&app)["preview_delay_s"], 40);
    assert_eq!(stored(&app)["export_folder"], "/tmp/x");
    assert_eq!(shown["preview_delay_s"], 40);
    assert_eq!(shown["effective_preview_delay_s"], 30);
    assert_eq!(mode(&app), 0o600);
}

#[test]
fn c1b_client_secret_is_masked_and_masked_save_keeps_it() {
    let mut app = TestApp::new().unwrap();
    app.app
        .settings_set(&json!({"integration": shopify("S-1234")}))
        .unwrap();
    assert_eq!(stored(&app)["integration"]["client_secret"], "S-1234");
    let shown = app.app.settings_get().unwrap();
    assert_eq!(shown["integration"]["client_secret"], "••••1234");
    assert_eq!(shown["integration"]["shop_domain"], "example.myshopify.com");
    assert_eq!(shown["integration"]["client_id"], "ID");
    assert_eq!(
        shown["integration"]["condition_photos_key"],
        "custom.damage_report"
    );
    app.app
        .settings_set(&json!({"integration": shown["integration"]}))
        .unwrap();
    assert_eq!(stored(&app)["integration"]["client_secret"], "S-1234");
}

#[test]
fn c2a_open_creates_readable_sqlite_backup() {
    let app = TestApp::new().unwrap();
    let names = files(&app.dir.path().join("backup"));
    assert_eq!(names.len(), 1);
    assert!(
        names[0].starts_with("session_manager-") && names[0].ends_with(".db"),
        "{names:?}"
    );
    let db = rusqlite::Connection::open(app.dir.path().join("backup").join(&names[0])).unwrap();
    let unassigned: i64 = db
        .query_row("SELECT count(*) FROM sessions WHERE id = 1", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(unassigned, 1);
}

#[test]
fn c2c_open_prunes_backups_to_newest_thirty() {
    let mut app = TestApp::new().unwrap();
    let dir = app.dir.path().join("backup");
    for name in files(&dir) {
        fs::remove_file(dir.join(name)).unwrap();
    }
    for n in 0..31 {
        fs::write(
            dir.join(format!("session_manager-20200101-{n:06}.db")),
            b"old",
        )
        .unwrap();
    }
    app.kill_and_restart().unwrap();
    let names = files(&dir);
    // 31 old + the start's own = 32; the oldest two names go.
    let mut expected = (2..31)
        .map(|n| format!("session_manager-20200101-{n:06}.db"))
        .collect::<Vec<_>>();
    let new = names
        .iter()
        .find(|name| !name.starts_with("session_manager-20200101-"))
        .expect("the start's own backup")
        .clone();
    expected.push(new);
    expected.sort();
    assert_eq!(names, expected);
}

fn publish_one_session(app: &TestApp) {
    app.clock.advance(1_761_350_400_000);
    let session = app
        .items()
        .create_local("BACK-1", "Chair", "Chair", None)
        .unwrap()
        .1
        .session
        .id;
    let folder = app.dir.path().join("Capture/raws");
    fs::create_dir_all(&folder).unwrap();
    let raw = folder.join("a.ARW");
    fs::write(&raw, b"raw").unwrap();
    let photo = app
        .store()
        .photo_insert_or_get(NewPhoto {
            session_id: session,
            raw_path: raw.to_string_lossy().into_owned(),
            raw_hash: Some(sha256_file(&raw).unwrap()),
            captured_at: Some(app.clock.now_ms()),
            source: PhotoSource::Capture,
            created_at: app.clock.now_ms(),
        })
        .unwrap();
    app.fake.add_image(&photo.raw_path, &["v1"], false);
    app.review()
        .classify(photo.id, Classification::Main)
        .unwrap();
    app.store()
        .set_missing_preview_paths(&photo.raw_path, "preview.jpg")
        .unwrap();
    app.app
        .publish
        .as_ref()
        .unwrap()
        .confirm(&[session], "Publish chair")
        .unwrap();
    for _ in 0..10 {
        app.tick(1100).unwrap();
        app.complete_all_renders().unwrap();
    }
    assert_eq!(app.store().publish_item(1).unwrap().step, PublishStep::Done);
}

#[test]
fn c2b_publish_done_creates_second_backup() {
    let app = TestApp::with_fake_shop().unwrap();
    assert_eq!(files(&app.dir.path().join("backup")).len(), 1);
    publish_one_session(&app);
    assert_eq!(files(&app.dir.path().join("backup")).len(), 2);
}

#[test]
fn c3a_health_reports_document_token_and_field_failures() {
    let app = TestApp::with_fake_shop_key("custom.nope").unwrap();
    app.fake.state.lock().unwrap().document_id = "/elsewhere/Other Session".into();
    app.fake_shop()
        .state
        .lock()
        .unwrap()
        .token_app_not_installed = true;
    let report = app.app.health_check().unwrap();
    let item = |code: &str| {
        report
            .items
            .iter()
            .find(|item| item.code == code)
            .unwrap_or_else(|| panic!("{code} in {:?}", report.items))
    };
    assert!(!item("document_not_open").ok);
    assert!(!item("integration_token_invalid").ok);
    assert_eq!(
        item("integration_token_invalid").detail,
        "The Session Manager app is not installed on this store"
    );
    assert!(!item("condition_field_missing").ok);
    assert_eq!(
        item("condition_field_missing").detail,
        "Condition photos field custom.nope not found in the shop"
    );
    let failing = report
        .items
        .iter()
        .filter(|item| !item.ok)
        .map(|item| item.code.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        failing,
        [
            "document_not_open",
            "integration_token_invalid",
            "condition_field_missing"
        ]
    );
}

#[test]
fn c5b_settings_file_is_0600_and_the_secret_lives_nowhere_else() {
    let mut app = TestApp::new().unwrap();
    app.app
        .settings_set(&json!({"integration": shopify("SECRETTOKEN123")}))
        .unwrap();
    app.app
        .settings_set(&json!({"preview_delay_s": 20}))
        .unwrap();
    assert_eq!(mode(&app), 0o600);
    fn inspect(dir: &std::path::Path, found: &mut Vec<String>) {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                inspect(&path, found);
            } else if String::from_utf8_lossy(&fs::read(&path).unwrap()).contains("SECRETTOKEN123")
            {
                found.push(path.file_name().unwrap().to_string_lossy().into_owned());
            }
        }
    }
    let mut found = Vec::new();
    inspect(app.dir.path(), &mut found);
    assert_eq!(found, ["settings.json"]);
}

#[derive(Clone)]
struct LogBytes(std::sync::Arc<std::sync::Mutex<Vec<u8>>>);
impl std::io::Write for LogBytes {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// Every log line `f` produces, written through the production log writer (redaction included).
fn capture_log(f: impl FnOnce()) -> String {
    let buffer = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let output = LogBytes(buffer.clone());
    let subscriber = tracing_subscriber::fmt()
        .with_writer(psm_infra::logging::redacting(move || output.clone()))
        .without_time()
        .with_ansi(false)
        .finish();
    tracing::subscriber::with_default(subscriber, f);
    String::from_utf8(buffer.lock().unwrap().clone()).unwrap()
}

#[test]
fn c4a_editing_provider_call_logs_operation_and_duration() {
    let temp = tempfile::tempdir().unwrap();
    let script = temp.path().join("inspect_document.js");
    fs::write(&script, "function run() { return 'ready'; }").unwrap();
    let log = capture_log(|| {
        let result = psm_infra::osascript::OsaRunner::new(temp.path())
            .run(&script, &[], std::time::Duration::from_secs(5))
            .unwrap();
        assert_eq!(result, "ready");
    });
    let line = log
        .lines()
        .find(|line| line.contains("provider call"))
        .unwrap_or_else(|| panic!("{log}"));
    assert!(line.contains("op=inspect_document"), "{line}");
    let duration = line
        .split("duration_ms=")
        .nth(1)
        .and_then(|rest| rest.split_whitespace().next())
        .unwrap_or_else(|| panic!("{line}"));
    assert!(duration.parse::<u64>().is_ok(), "{line}");
}

/// The real Shopify adapter (token from the client credentials grant, one product creation)
/// over the fake transport, with client secret `SECRETTOKEN123` and access token `ACCESSTOKEN456`.
fn shopify_mutation_log(run_id_app: &TestApp) -> (String, i64) {
    use psm_core::ports::commerce::CommerceAdapter;
    use psm_integrations::commerce::shopify::{PollConfig, ShopifyAdapter, ShopifyConfig};
    use psm_testkit::{
        clock::ManualClock, fake_transport::FakeTransport, fixtures::shopify::fixture,
    };
    use std::sync::Arc;
    let transport = Arc::new(FakeTransport::new());
    let mut token_reply = fixture("sv0_token").reply();
    token_reply.body["access_token"] = json!("ACCESSTOKEN456");
    transport.on_form_always(
        "https://example.myshopify.com/admin/oauth/access_token",
        token_reply,
    );
    transport.replay_any(&fixture("sv7_create"));
    let adapter = ShopifyAdapter::new(
        ShopifyConfig {
            shop_domain: "example.myshopify.com".into(),
            api_version: "2026-10".into(),
            client_id: "ID".into(),
            client_secret: "SECRETTOKEN123".into(),
            condition_photos_key: "custom.damage_report".into(),
        },
        transport,
        Arc::new(ManualClock::new(1000)),
        PollConfig {
            interval_ms: 0,
            attempts: 5,
            retries: 0,
            retry_wait_ms: 0,
        },
    );
    let run = run_id_app.confirmed_run().unwrap();
    let log = capture_log(|| {
        adapter
            .item_create(
                &run,
                "PSM-PROBE-SKU-1",
                Some("4006381333931"),
                "PSM-PROBE product (Session Manager grounding probe)",
            )
            .unwrap();
    });
    (log, run.run_id())
}

#[test]
fn c4b_every_adapter_mutation_line_carries_the_run_id() {
    let app = TestApp::new().unwrap();
    let (log, run_id) = shopify_mutation_log(&app);
    let mutations = log
        .lines()
        .filter(|line| line.contains("shopify mutation") || line.contains("staged upload"))
        .collect::<Vec<_>>();
    assert!(!mutations.is_empty(), "{log}");
    for line in mutations {
        assert!(line.contains(&format!("run_id={run_id}")), "{line}");
    }
}

#[test]
fn c5a_secrets_never_reach_logs_events_or_notifications() {
    let mut app = TestApp::with_fake_shop().unwrap();
    app.app
        .settings_set(&json!({"integration": shopify("SECRETTOKEN123")}))
        .unwrap();
    let mut log = capture_log(|| publish_one_session(&app));
    log.push_str(&shopify_mutation_log(&app).0);
    app.deliver_hook(r#"{"hook":"capture_done","argv":["benign"]}"#)
        .unwrap();
    assert!(log.contains("shopify mutation"), "{log}");
    assert!(!log.contains("SECRETTOKEN123"), "{log}");
    assert!(!log.contains("ACCESSTOKEN456"), "{log}");
    let conn = rusqlite::Connection::open(app.app.paths.db()).unwrap();
    for (table, column) in [("capture_events", "payload"), ("notifications", "text")] {
        let count: i64 = conn
            .query_row(&format!("SELECT count(*) FROM {table}"), [], |row| {
                row.get(0)
            })
            .unwrap();
        assert!(count > 0, "{table}");
        let leaked: i64 = conn
            .query_row(
                &format!(
                    "SELECT count(*) FROM {table} WHERE {column} LIKE '%SECRETTOKEN123%' \
                     OR {column} LIKE '%ACCESSTOKEN456%'"
                ),
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(leaked, 0, "{table}");
    }
}

#[test]
fn c6a_folder_lookup_finds_variant_without_switching_collection() {
    let script = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../integrations/src/editing/capture_one/jxa/find_image_in_folder.js");
    let harness = r#"
const fs = require('fs');
const source = fs.readFileSync(process.argv[1], 'utf8');
let switched = false;
const images = { whose: ({path}) => () => path.endsWith('12199.ARW') ? [{ variants: { whose: () => () => [{id: () => 130}] } }] : [] };
const folder = { file: () => ({toString: () => '/private/Users/davidloorenz/Pictures/session test 1/Capture/PSM_PROBE_A/'}), images };
const doc = { collections: () => [folder], set currentCollection(_) { switched = true; } };
global.Application = () => ({running: () => true, documents: {byId: () => doc}});
eval(source);
const answer = run(['/Users/davidloorenz/Pictures/session test 1', '/Users/davidloorenz/Pictures/session test 1/Capture/PSM_PROBE_A', '/Users/davidloorenz/Pictures/session test 1/Capture/PSM_PROBE_A/session test 12199.ARW']);
if (answer !== '"130"' || switched || source.includes('whose({file')) process.exit(1);
console.log(answer);
"#;
    let output = std::process::Command::new("node")
        .arg("-e")
        .arg(harness)
        .arg(script)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8(output.stdout).unwrap().trim(), "\"130\"");
}
