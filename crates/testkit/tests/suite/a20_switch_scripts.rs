//! Owner correction A20 (2026-11-18, "there is still delays when opening a session"). Evidence
//! (work Mac, A16 build): a switch took 15–22 s, nearly all of it Capture One answering scripts:
//! `document_info` walked every collection (three Apple events each, one per session: ~3.5 s a
//! call with ~125 collections) and was asked four times per switch, and every switch re-verified
//! both recipes and all three hooks. Now a switch's cost does not grow with the number of sessions,
//! and recipes and hooks are verified once per Capture One run (again after a restart, a failure,
//! or the Health page's check).
use psm_infra::paths::AppPaths;
use psm_integrations::editing::capture_one::jxa::JxaScripting;
use psm_testkit::TestApp;

fn session(a: &TestApp, sku: &str) -> i64 {
    a.items()
        .create_local(sku, sku, "object", None)
        .unwrap()
        .1
        .session
        .id
}
fn ops(a: &TestApp) -> Vec<String> {
    a.fake.state.lock().unwrap().ops.clone()
}
fn clear(a: &TestApp) {
    a.fake.state.lock().unwrap().ops.clear();
}
fn count(ops: &[String], prefix: &str) -> usize {
    ops.iter().filter(|op| op.starts_with(prefix)).count()
}
/// The recipe and hook checks a full ensure makes.
fn verification(ops: &[String]) -> usize {
    count(ops, "ensure_recipe:") + count(ops, "set_hook:") + count(ops, "get_hook:")
}

/// Once verified in this Capture One run, a switch skips the recipes and hooks and asks the
/// document's description once per step that needs it (not four times), whatever the session.
#[test]
fn a20_a_a_switch_skips_what_this_run_already_verified() {
    let a = TestApp::with_fake_document().unwrap();
    let first = session(&a, "A20-1");
    let second = session(&a, "A20-2");
    let third = session(&a, "A20-3");
    a.sessions().activate(first).unwrap();
    clear(&a);
    a.sessions().activate(second).unwrap();
    let switch = ops(&a);
    assert_eq!(verification(&switch), 0, "{switch:?}");
    // ensure (1), inspect_document (1), set_capture_folder's prior folder (1).
    assert_eq!(
        switch.iter().filter(|op| *op == "document_info").count(),
        3,
        "{switch:?}"
    );
    assert!(switch.iter().any(|op| op.starts_with("set_captures:")));
    assert_eq!(
        a.store().active_session_id().unwrap(),
        Some(second),
        "the switch still lands"
    );
    // Every later switch costs the same.
    clear(&a);
    a.sessions().activate(third).unwrap();
    let again = ops(&a);
    assert_eq!(verification(&again), 0);
    assert_eq!(again.len(), switch.len(), "{again:?} vs {switch:?}");
}

/// A restarted Capture One (another process) is verified in full again.
#[test]
fn a20_b_a_restart_verifies_recipes_and_hooks_again() {
    let a = TestApp::with_fake_document().unwrap();
    let first = session(&a, "A20-1");
    let second = session(&a, "A20-2");
    a.sessions().activate(first).unwrap();
    {
        // Quit outside the app and start again (`running_after_polls`: the next look sees it run).
        let mut s = a.fake.state.lock().unwrap();
        s.running = false;
        s.running_after_polls = Some(1);
    }
    clear(&a);
    a.sessions().activate(second).unwrap();
    let switch = ops(&a);
    assert_eq!(count(&switch, "ensure_recipe:"), 2, "{switch:?}");
    assert_eq!(count(&switch, "set_hook:"), 3, "{switch:?}");
    // The next switch in the same run skips them again.
    clear(&a);
    a.sessions().activate(first).unwrap();
    assert_eq!(verification(&ops(&a)), 0);
}

/// A failed ensure leaves nothing marked: the next switch verifies in full. A process that cannot be
/// told is always verified in full.
#[test]
fn a20_c_failure_or_unknown_process_verifies_in_full() {
    let a = TestApp::with_fake_document().unwrap();
    let first = session(&a, "A20-1");
    let second = session(&a, "A20-2");
    a.sessions().activate(first).unwrap();
    // A recipe that cannot be repaired fails a full ensure (here the Health page's).
    a.fake.state.lock().unwrap().fail_make_recipe = Some("refused".into());
    a.fake.state.lock().unwrap().recipes.remove("PSM Preview");
    a.app.health_check().unwrap();
    assert!(a.app.health.has("recipe_missing"));
    a.fake.state.lock().unwrap().fail_make_recipe = None;
    clear(&a);
    a.sessions().activate(second).unwrap();
    let switch = ops(&a);
    assert_eq!(count(&switch, "ensure_recipe:"), 2, "{switch:?}");
    assert_eq!(count(&switch, "set_hook:"), 3, "{switch:?}");

    a.fake.state.lock().unwrap().instance_unknown = true;
    for session in [first, second] {
        clear(&a);
        a.sessions().activate(session).unwrap();
        assert_eq!(count(&ops(&a), "ensure_recipe:"), 2);
    }
}

/// The Health page's check verifies in full, and the style is checked on every switch (the
/// photographer can change Next Capture Adjustments in Capture One at any time).
#[test]
fn a20_d_health_check_is_full_and_style_is_checked_every_switch() {
    let mut a = TestApp::with_fake_document().unwrap();
    a.settings_set(&serde_json::json!({"style_name": "PSM House"}))
        .unwrap();
    let first = session(&a, "A20-1");
    let second = session(&a, "A20-2");
    a.sessions().activate(first).unwrap();
    clear(&a);
    a.app.health_check().unwrap();
    assert_eq!(count(&ops(&a), "ensure_recipe:"), 2, "{:?}", ops(&a));
    assert_eq!(count(&ops(&a), "set_hook:"), 3);

    // The photographer picks another style in Capture One: the next switch puts the house style back.
    a.fake.state.lock().unwrap().next_capture_styles = vec!["Another".into()];
    clear(&a);
    a.sessions().activate(second).unwrap();
    let switch = ops(&a);
    assert_eq!(verification(&switch), 0);
    assert!(
        switch.contains(&"set_next_capture_styles:PSM House".to_owned()),
        "{switch:?}"
    );
    assert_eq!(
        a.fake.state.lock().unwrap().next_capture_styles,
        ["PSM House"]
    );
}

/// The scripts themselves: `collectionForFolder` (used by set_current_collection, sync_folder and
/// the photo lookups) and `document_info` send the same number of Apple events with 10 or 500
/// collections. Each property read on the mock counts as one event, as on Capture One.
#[test]
fn a20_e_script_events_do_not_grow_with_the_number_of_sessions() {
    let dir = tempfile::tempdir().unwrap();
    let paths = AppPaths::for_dir(dir.path());
    JxaScripting::new(&paths).unwrap();
    let scripts = paths.hooks().join("jxa");
    let harness = r#"
const fs = require('fs');
const [collectionSource, infoSource, n] = [fs.readFileSync(process.argv[1], 'utf8'), fs.readFileSync(process.argv[2], 'utf8'), Number(process.argv[3])];
let events = 0;
const ev = v => () => { events += 1; return v; };
const id = '/S';
const folders = [];
for (let i = 0; i < n; i++) folders.push(i % 7 === 0 ? null : {toString: () => id + '/Capture/K' + i});
const collection = i => ({id: ev(String(i)), name: ev('K' + i), folder: ev(folders[i]), file: ev(folders[i])});
const all = folders.map((_, i) => collection(i));
let current = all[1];
const collections = ev(all);
collections.folder = ev(folders);
collections.at = i => all[i];
const doc = {exists: ev(true), id: ev(id), kind: ev('session'), camera: ev(null), captures: ev({toString: () => id + '/Capture/K1'}), collections};
Object.defineProperty(doc, 'currentCollection', {get: () => () => { events += 1; return current; }, set: v => { events += 1; current = v; }});
const app = {running: ev(true), documents: {byId: () => doc}};
global.Application = () => app;
global.Path = v => v;
const target = id + '/Capture/K' + (n - 1);
eval(collectionSource);
events = 0;
run([id, target]);
const switched = events;
const found = current === all[n - 1];
eval(infoSource);
events = 0;
run([id]);
console.log(JSON.stringify({switched, info: events, found}));
"#;
    let measure = |n: usize| -> serde_json::Value {
        let output = std::process::Command::new("node")
            .arg("-e")
            .arg(harness)
            .arg(scripts.join("set_current_collection.js"))
            .arg(scripts.join("document_info.js"))
            .arg(n.to_string())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        serde_json::from_slice(&output.stdout).unwrap()
    };
    let small = measure(10);
    let large = measure(500);
    assert_eq!(small["found"], true, "{small}");
    assert_eq!(large["found"], true, "{large}");
    assert_eq!(small["switched"], large["switched"], "{small} vs {large}");
    assert_eq!(small["info"], large["info"], "{small} vs {large}");
    assert!(large["switched"].as_u64().unwrap() < 12, "{large}");
    assert!(large["info"].as_u64().unwrap() < 10, "{large}");
}
