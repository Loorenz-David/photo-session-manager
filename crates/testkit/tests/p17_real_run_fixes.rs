use psm_core::{Error, mechanisms::canon::canon};
use psm_infra::paths::AppPaths;
use psm_integrations::editing::capture_one::{jxa::JxaScripting, scripting::CaptureOneScripting};
use psm_testkit::TestApp;
use rusqlite::Connection;
use std::{
    fs,
    path::PathBuf,
    time::{Duration, Instant},
};

fn root(app: &TestApp) -> PathBuf {
    app.dir.path().join("Capture One Session/Capture")
}
fn document(app: &TestApp) -> String {
    app.dir
        .path()
        .join("Capture One Session")
        .to_string_lossy()
        .into_owned()
}
fn ops(app: &TestApp) -> Vec<String> {
    app.fake.state.lock().unwrap().ops.clone()
}
fn session(app: &TestApp, sku: &str) -> i64 {
    app.items()
        .create_local(sku, sku, "object", None)
        .unwrap()
        .1
        .session
        .id
}
fn raw(app: &TestApp, relative: &str) -> String {
    let path = root(app).join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, b"raw").unwrap();
    canon(&path).unwrap()
}
fn hook(app: &TestApp, raw: &str) {
    app.deliver_hook(&serde_json::json!({"hook":"capture_done","argv":[raw]}).to_string())
        .unwrap();
}
fn report_item<'a>(
    report: &'a psm_core::domain::HealthReport,
    code: &str,
) -> Vec<&'a psm_core::domain::HealthItem> {
    report
        .items
        .iter()
        .filter(|item| item.code == code)
        .collect()
}

#[test]
fn c1a_settings_rebuild_addresses_new_document() {
    let mut app = TestApp::new().unwrap();
    let id = document(&app);
    app.settings_set(&serde_json::json!({"document_id":id,"preview_recipe":"PSM Preview","web_recipe":"PSM Web"})).unwrap();
    let report = app.app.health_check().unwrap();
    assert!(report_item(&report, "document_not_open").is_empty());
    assert!(report_item(&report, "recipe_missing").is_empty());
    for code in ["document", "hooks", "recipes"] {
        assert_eq!(
            report_item(&report, code)
                .iter()
                .map(|x| x.ok)
                .collect::<Vec<_>>(),
            [true]
        );
    }
    assert!(ops(&app).contains(&"recipe_exists:PSM Preview".into()));
}
#[test]
fn c1b_recipe_names_take_effect_without_relaunch() {
    let mut app = TestApp::with_fake_document().unwrap();
    app.settings_set(&serde_json::json!({"preview_recipe":"Nope"}))
        .unwrap();
    let first = app.app.health_check().unwrap();
    assert_eq!(
        report_item(&first, "recipe_missing")
            .iter()
            .map(|x| x.detail.as_str())
            .collect::<Vec<_>>(),
        ["Nope"]
    );
    app.settings_set(&serde_json::json!({"preview_recipe":"PSM Preview"}))
        .unwrap();
    assert!(report_item(&app.app.health_check().unwrap(), "recipe_missing").is_empty());
}
#[test]
fn c1c_integration_rebuilds_without_network() {
    let mut app = TestApp::new().unwrap();
    app.settings_set(&serde_json::json!({"integration":{"kind":"shopify","shop_domain":"test.myshopify.com","api_version":"2026-10","client_id":"id","client_secret":"secret-1234","condition_photos_key":"custom.damage_report"}})).unwrap();
    assert!(app.app.commerce.is_some() && app.app.publish.is_some());
    assert_eq!(
        app.app.settings_get().unwrap()["integration"]["shop_domain"],
        "test.myshopify.com"
    );
    app.settings_set(&serde_json::json!({"integration":null}))
        .unwrap();
    assert!(app.app.commerce.is_none() && app.app.publish.is_none());
}
#[test]
fn c2a_written_jxa_scripts_share_one_folder_scan() {
    let dir = tempfile::tempdir().unwrap();
    let paths = AppPaths::for_dir(dir.path());
    JxaScripting::new(&paths).unwrap();
    let scripts = paths.hooks().join("jxa");
    let mut names = Vec::new();
    for entry in fs::read_dir(&scripts).unwrap() {
        let entry = entry.unwrap();
        let code = fs::read_to_string(entry.path()).unwrap();
        let compact: String = code.chars().filter(|c| !c.is_whitespace()).collect();
        assert!(
            !compact.contains("whose({file"),
            "{}",
            entry.path().display()
        );
        assert_eq!(code.matches("function collectionForFolder").count(), 1);
        names.push(entry.file_name().to_string_lossy().into_owned());
    }
    for name in [
        "process.js",
        "get_color_tag.js",
        "set_color_tag.js",
        "find_image_in_folder.js",
    ] {
        assert!(names.contains(&name.into()));
        let code = fs::read_to_string(scripts.join(name)).unwrap();
        assert!(code.contains("imageByPath(") || code.contains("collectionForFolder("));
    }
    assert_eq!(
        include_str!("../../integrations/src/editing/capture_one/jxa/lib.js")
            .matches("function collectionForFolder")
            .count(),
        1
    );
}
#[test]
fn c3a_capture_root_comes_from_document_id() {
    let app = TestApp::with_fake_document().unwrap();
    let target = root(&app).join("PSM_PROBE_B");
    fs::create_dir_all(&target).unwrap();
    app.fake
        .set_captures(&document(&app), &target.to_string_lossy())
        .unwrap();
    let doc = app.app.editing.inspect_document().unwrap();
    assert_eq!(doc.capture_root, canon(&root(&app)).unwrap());
    assert_eq!(doc.capture_folder, target.to_string_lossy());
    let state = app.fake.state.lock().unwrap();
    assert_eq!(state.collections[1].id, "capture");
    assert_eq!(
        state.collections[1].folder.as_deref(),
        Some(target.to_string_lossy().as_ref())
    );
}
#[test]
fn c3b_switches_never_nest() {
    let app = TestApp::with_fake_document().unwrap();
    let a = session(&app, "A");
    let b = session(&app, "B");
    app.sessions().activate(a).unwrap();
    app.sessions().activate(a).unwrap();
    app.sessions().deactivate().unwrap();
    app.sessions().activate(b).unwrap();
    let root = canon(&root(&app)).unwrap();
    assert_eq!(
        ops(&app)
            .into_iter()
            .filter(|op| op.starts_with("set_captures:"))
            .collect::<Vec<_>>(),
        [
            format!("set_captures:{root}/A"),
            format!("set_captures:{root}/A"),
            format!("set_captures:{root}/_Unassigned"),
            format!("set_captures:{root}/B")
        ]
    );
    assert_eq!(
        app.fake.get_captures(&document(&app)).unwrap(),
        format!("{root}/B")
    );
}
#[test]
fn c3c_arrivals_follow_derived_root() {
    let app = TestApp::with_fake_document().unwrap();
    let b = session(&app, "B");
    app.sessions().activate(b).unwrap();
    let x = raw(&app, "B/x.ARW");
    let y = raw(&app, "_Unassigned/y.ARW");
    hook(&app, &x);
    hook(&app, &y);
    let db = Connection::open(app.app.paths.db()).unwrap();
    let mut stmt = db
        .prepare("SELECT raw_path,session_id FROM photos ORDER BY raw_path")
        .unwrap();
    let rows = stmt
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect::<Vec<_>>();
    assert_eq!(rows, [(x, b), (y, 1)]);
}
#[test]
fn c3d_missing_capture_root_is_unhealthy() {
    let app = TestApp::with_fake_document().unwrap();
    fs::remove_dir_all(root(&app)).unwrap();
    assert!(matches!(
        app.app.editing.inspect_document(),
        Err(Error::CaptureFolderNotSet)
    ));
    let report = app.app.health_check().unwrap();
    let items = report_item(&report, "capture_folder_not_set");
    assert_eq!(items.len(), 1);
    assert!(!items[0].ok);
    assert!(
        items[0]
            .detail
            .contains(&root(&app).to_string_lossy().to_string())
    );
}
#[test]
fn c4c_focus_returns_before_provider_render() {
    let app = TestApp::with_fake_document().unwrap();
    let path = raw(&app, "_Unassigned/focus.ARW");
    app.fake.add_image(&path, &["v1"], false);
    hook(&app, &path);
    app.app.jobs.tick(0).unwrap();
    let job = app.fake.jobs_for("v1")[0].clone();
    app.complete_render(&job).unwrap();
    app.fake.state.lock().unwrap().call_delay_ms = 500;
    app.deliver_hook(
        &serde_json::json!({"hook":"variant_adjusted","variant_id":"v1","raw":path}).to_string(),
    )
    .unwrap();
    let start = Instant::now();
    app.app.handles().on_window_focus();
    assert!(start.elapsed() < Duration::from_millis(100));
    let until = Instant::now() + Duration::from_secs(2);
    while Instant::now() < until && app.fake.state.lock().unwrap().issued.len() < 2 {
        std::thread::sleep(Duration::from_millis(20));
    }
    assert_eq!(app.fake.state.lock().unwrap().issued.len(), 2);
}
fn failed_render_fixture() -> (TestApp, String) {
    let app = TestApp::with_fake_document().unwrap();
    let path = raw(&app, "_Unassigned/backoff.ARW");
    app.fake.add_image(&path, &["v1"], false);
    app.fake.state.lock().unwrap().fail_process = true;
    hook(&app, &path);
    app.app.jobs.tick(0).unwrap();
    (app, path)
}
#[test]
fn c5a_failed_renders_back_off_and_finish_job() {
    let (app, path) = failed_render_fixture();
    let mut times = Vec::new();
    let mut previous = 0;
    for second in 0..=60 {
        if second > 0 {
            app.tick(1000).unwrap();
        }
        let current = ops(&app)
            .iter()
            .filter(|op| op.starts_with("process:"))
            .count();
        if current > previous {
            times.push(second);
            previous = current;
        }
    }
    assert_eq!(times, [0, 1, 3, 7, 15, 31]);
    let operations = ops(&app);
    assert_eq!(
        operations
            .iter()
            .filter(|op| *op == "recipe_exists:PSM Preview")
            .count(),
        6
    );
    assert_eq!(
        operations
            .iter()
            .filter(|op| *op == &format!("find_image:{path}"))
            .count(),
        6
    );
    let db = Connection::open(app.app.paths.db()).unwrap();
    let status: String = db
        .query_row(
            "SELECT status FROM jobs WHERE kind='render_preview'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(status, "done");
}
#[test]
fn c5b_adjustment_resets_backoff_then_waits_delay() {
    let (app, path) = failed_render_fixture();
    for _ in 0..40 {
        app.tick(1000).unwrap();
    }
    app.fake.state.lock().unwrap().fail_process = false;
    app.deliver_hook(
        &serde_json::json!({"hook":"variant_adjusted","variant_id":"v1","raw":path}).to_string(),
    )
    .unwrap();
    let before = ops(&app)
        .iter()
        .filter(|op| op.starts_with("process:"))
        .count();
    for _ in 41..55 {
        app.tick(1000).unwrap();
    }
    assert_eq!(
        ops(&app)
            .iter()
            .filter(|op| op.starts_with("process:"))
            .count(),
        before
    );
    app.tick(1000).unwrap();
    assert_eq!(
        ops(&app)
            .iter()
            .filter(|op| op.starts_with("process:"))
            .count(),
        before + 1
    );
    assert!(app.app.editing.render_in_flight(&path));
}
#[test]
fn c5c_user_folder_switch_preempts_background_jobs() {
    let app = TestApp::with_fake_document().unwrap();
    let b = session(&app, "B");
    for n in 0..10 {
        let path = raw(&app, &format!("_Unassigned/tag{n}.ARW"));
        let photo = app.add_photo(1, &format!("tag{n}.ARW")).unwrap();
        app.fake
            .add_image(&photo.raw_path, &[&format!("v{n}")], false);
        app.review()
            .classify(photo.id, psm_core::domain::photo::Classification::Main)
            .unwrap();
        let _ = path;
    }
    app.fake.state.lock().unwrap().call_delay_ms = 20;
    std::thread::scope(|scope| {
        let worker = scope.spawn(|| {
            app.app.jobs.tick(0).unwrap();
        });
        std::thread::sleep(Duration::from_millis(50));
        let start = Instant::now();
        app.sessions().activate(b).unwrap();
        assert!(start.elapsed() < Duration::from_secs(1));
        worker.join().unwrap();
    });
    let operations = ops(&app);
    let end = operations
        .iter()
        .rposition(|op| op == "get_captures")
        .unwrap();
    let start = operations[..end]
        .iter()
        .rposition(|op| op == "document_info")
        .unwrap();
    let between = &operations[start..=end];
    for pair in ["document_info", "save", "set_captures:", "get_captures"].windows(2) {
        let left = between
            .iter()
            .position(|op| op.starts_with(pair[0]))
            .unwrap();
        let right = between
            .iter()
            .position(|op| op.starts_with(pair[1]))
            .unwrap();
        assert!(
            between[left + 1..right]
                .iter()
                .filter(|op| op.starts_with("find_image:")
                    || op.starts_with("find_image_in_folder:")
                    || op.starts_with("get_color_tag:")
                    || op.starts_with("set_color_tag:"))
                .count()
                <= 1
        );
    }
    assert_eq!(app.store().active_session_id().unwrap(), Some(b));
    let db = Connection::open(app.app.paths.db()).unwrap();
    let done: i64 = db
        .query_row(
            "SELECT COUNT(*) FROM jobs WHERE kind='mirror_tag' AND status='done'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(done, 10);
}

#[test]
fn c2b_process_finds_image_while_browsing_other_collection() {
    let dir = tempfile::tempdir().unwrap();
    let paths = AppPaths::for_dir(dir.path());
    JxaScripting::new(&paths).unwrap();
    let scripts = paths.hooks().join("jxa");
    let harness = r#"
const fs = require('fs');
const processSource = fs.readFileSync(process.argv[1], 'utf8');
const tagSource = fs.readFileSync(process.argv[2], 'utf8');
const documentId = '/Users/davidloorenz/Pictures/session test 1';
const folder = documentId + '/Capture/PSM_PROBE_A';
const raw = folder + '/session test 12199.ARW';
let currentCollection = 'Capture';
const image = { variants: { byId: id => ({ id, colorTag: () => 0 }) } };
const target = { folder: () => ({toString: () => folder}), images: {whose: ({path}) => () => path === raw ? [image] : []} };
const doc = { images: {whose: () => () => []}, collections: () => [target], set currentCollection(v) { currentCollection = v; } };
const app = {running: () => true, documents: {byId: id => id === documentId ? doc : null}, process: () => ({toString: () => 'job-130'})};
global.Application = () => app;
const args = [documentId, raw, folder, '130', 'PSM Preview'];
eval(processSource);
const job = run(args);
eval(tagSource);
const tag = run(args.slice(0, 4));
console.log(JSON.stringify({job, tag, currentCollection}));
"#;
    let output = std::process::Command::new("node")
        .arg("-e")
        .arg(harness)
        .arg(scripts.join("process.js"))
        .arg(scripts.join("get_color_tag.js"))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap(),
        serde_json::json!({"job":"\"job-130\"","tag":"0","currentCollection":"Capture"})
    );
}

#[test]
fn c3e_document_info_reports_moving_capture_favourite_and_fixed_root() {
    let dir = tempfile::tempdir().unwrap();
    let paths = AppPaths::for_dir(dir.path());
    JxaScripting::new(&paths).unwrap();
    let scripts = paths.hooks().join("jxa");
    let harness = r#"
const fs = require('fs');
const setSource = fs.readFileSync(process.argv[1], 'utf8');
const infoSource = fs.readFileSync(process.argv[2], 'utf8');
const id = '/Users/davidloorenz/Pictures/session test 1';
const root = id + '/Capture';
const target = root + '/PSM_PROBE_B';
let captures = root;
let currentCollection = 'Capture';
const rootCollection = {id: () => '3', name: () => 'Capture', folder: () => ({toString: () => root})};
const current = {id: () => 'capture', name: () => 'PSM_PROBE_B', folder: () => ({toString: () => captures})};
const doc = {exists: () => true, id: () => id, kind: () => 'session', camera: () => null, captures: () => ({toString: () => captures}), collections: () => [rootCollection, current], set currentCollection(v) { currentCollection = v; }};
const app = {running: () => true, documents: {byId: () => doc}};
global.Application = () => app;
global.Path = v => v;
// Model the dictionary's writable captures property.
Object.defineProperty(doc, 'captures', {get: () => () => ({toString: () => captures}), set: v => { captures = v; }});
eval(setSource);
run([id, target]);
eval(infoSource);
const info = JSON.parse(run([id]));
console.log(JSON.stringify({info, currentCollection}));
"#;
    let output = std::process::Command::new("node")
        .arg("-e")
        .arg(harness)
        .arg(scripts.join("set_captures.js"))
        .arg(scripts.join("document_info.js"))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        value["info"]["id"],
        "/Users/davidloorenz/Pictures/session test 1"
    );
    assert_eq!(
        value["info"]["captures"],
        "/Users/davidloorenz/Pictures/session test 1/Capture/PSM_PROBE_B"
    );
    assert_eq!(
        value["info"]["collections"][0]["folder"],
        "/Users/davidloorenz/Pictures/session test 1/Capture"
    );
    assert_eq!(value["info"]["collections"][1]["id"], "capture");
    assert_eq!(
        value["info"]["collections"][1]["folder"],
        value["info"]["captures"]
    );
    assert_eq!(value["currentCollection"], "Capture");
}
