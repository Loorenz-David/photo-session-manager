use psm_core::{mechanisms::canon::canon, services::photos::PhotosService};
use psm_testkit::TestApp;
use std::{
    fs,
    path::{Path, PathBuf},
    thread,
    time::{Duration, Instant},
};

const TAKE_BACK: &str = include_str!("../src/fixtures/sidecar_scripts/take_back.txt");

fn app() -> TestApp {
    TestApp::with_camera().unwrap()
}
fn session(a: &TestApp, sku: &str) -> (i64, String) {
    let (item, session) = a.items().create_local(sku, sku, "object", None).unwrap();
    (session.session.id, item.folder_key)
}
fn folder(a: &TestApp, key: &str) -> PathBuf {
    a.capture_root().join(key)
}
fn other(a: &TestApp) -> String {
    let path = folder(a, "Other");
    fs::create_dir_all(&path).unwrap();
    path.to_string_lossy().into_owned()
}
fn ops(a: &TestApp) -> Vec<String> {
    a.fake.state.lock().unwrap().ops.clone()
}
fn clear(a: &TestApp) {
    a.fake.state.lock().unwrap().ops.clear();
}
fn arrived(a: &TestApp, s: i64, name: &str) -> String {
    a.arrival_photo(s, name).unwrap()
}
fn import(a: &TestApp, path: &Path) {
    PhotosService {
        arrival: a.app.arrival.as_ref(),
        editing: a.app.editing.as_ref(),
        store: a.store(),
        clock: a.clock.as_ref(),
        history: a.app.history.as_ref(),
    }
    .import(&[path.to_string_lossy().into_owned()])
    .unwrap();
}
fn import_rows(a: &TestApp) -> Vec<(i64, String, String)> {
    let conn = rusqlite::Connection::open(a.app.paths.db()).unwrap();
    let mut stmt = conn
        .prepare("SELECT session_id,raw_path,source FROM photos ORDER BY id")
        .unwrap();
    stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect()
}
fn style(a: &mut TestApp, name: &str) {
    a.settings_set(&serde_json::json!({"style_name":name}))
        .unwrap();
}

#[test]
fn c1a_activation_sets_collection() {
    let a = app();
    let (s, key) = session(&a, "P49A");
    clear(&a);
    a.sessions().activate(s).unwrap();
    let folder = folder(&a, &key).to_string_lossy().into_owned();
    let ops = ops(&a);
    let captures = ops
        .iter()
        .position(|op| op == &format!("set_captures:{folder}"))
        .unwrap();
    let collection = ops
        .iter()
        .position(|op| op == &format!("set_current_collection:{folder}"))
        .unwrap();
    assert!(captures < collection);
    assert_eq!(a.fake.state.lock().unwrap().current_collection, folder);
}
#[test]
fn c1b_failed_collection_is_health_and_startup_repairs() {
    let a = app();
    let (s, key) = session(&a, "P49B");
    a.fake.state.lock().unwrap().fail_set_current_collection = true;
    a.sessions().activate(s).unwrap();
    assert_eq!(a.store().active_session_id().unwrap(), Some(s));
    assert!(a.app.health.has("editor_not_following"));
    {
        let mut f = a.fake.state.lock().unwrap();
        f.fail_set_current_collection = false;
        f.current_collection = other(&a);
    }
    a.app.startup().unwrap();
    assert_eq!(
        a.fake.state.lock().unwrap().current_collection,
        folder(&a, &key).to_string_lossy()
    );
    assert!(!a.app.health.has("editor_not_following"));
}
#[test]
fn c1c_startup_switches_collection() {
    let a = app();
    let (s, key) = session(&a, "P49C");
    a.sessions().activate(s).unwrap();
    a.fake.state.lock().unwrap().current_collection = other(&a);
    clear(&a);
    a.app.startup().unwrap();
    assert!(ops(&a).contains(&format!(
        "set_current_collection:{}",
        folder(&a, &key).display()
    )));

    let b = TestApp::with_camera_script(TAKE_BACK).unwrap();
    b.fake_processes.set(vec!["Capture One"]);
    let deadline = Instant::now() + Duration::from_secs(10);
    while !b.app.health.has("camera_held") && Instant::now() < deadline {
        b.tick(0).unwrap();
        thread::sleep(Duration::from_millis(10));
    }
    assert!(b.app.health.has("camera_held"));
    let (active, key) = session(&b, "P49C-TB");
    b.sessions().activate(active).unwrap();
    {
        let mut state = b.fake.state.lock().unwrap();
        state.current_collection = other(&b);
        state.running_after_polls = Some(1);
    }
    clear(&b);
    let take_back = b.take_back();
    thread::scope(|scope| {
        let handle = scope.spawn(|| take_back.run());
        let deadline = Instant::now() + Duration::from_secs(30);
        while !handle.is_finished() && Instant::now() < deadline {
            b.tick(500).unwrap();
            thread::sleep(Duration::from_millis(20));
        }
        assert!(handle.is_finished(), "take-back did not finish");
        assert_eq!(handle.join().unwrap().unwrap().completed_steps, 5);
    });
    assert!(ops(&b).contains(&format!(
        "set_current_collection:{}",
        folder(&b, &key).display()
    )));
}
#[test]
fn c2a_arrival_switches_from_other_collection() {
    let a = app();
    let (s, key) = session(&a, "P49D");
    a.sessions().activate(s).unwrap();
    a.fake.state.lock().unwrap().current_collection = other(&a);
    clear(&a);
    arrived(&a, s, "DSC1.ARW");
    assert!(ops(&a).contains(&format!(
        "set_current_collection:{}",
        folder(&a, &key).display()
    )));
}
#[test]
fn c2b_arrival_in_current_collection_does_not_switch() {
    let a = app();
    let (s, _) = session(&a, "P49E");
    a.sessions().activate(s).unwrap();
    clear(&a);
    arrived(&a, s, "DSC2.ARW");
    assert!(
        !ops(&a)
            .iter()
            .any(|op| op.starts_with("set_current_collection:"))
    );
}
#[test]
fn c2c_each_arrival_switches_again_after_browsing_away() {
    let a = app();
    let (s, key) = session(&a, "P49F");
    a.sessions().activate(s).unwrap();
    a.fake.state.lock().unwrap().current_collection = other(&a);
    clear(&a);
    arrived(&a, s, "DSC3.ARW");
    a.fake.state.lock().unwrap().current_collection = other(&a);
    arrived(&a, s, "DSC4.ARW");
    assert_eq!(
        ops(&a)
            .iter()
            .filter(|op| *op == &format!("set_current_collection:{}", folder(&a, &key).display()))
            .count(),
        2
    );
}
#[test]
fn c2d_stopped_editor_is_never_scripted_or_launched() {
    let a = app();
    let (s, _) = session(&a, "P49G");
    a.sessions().activate(s).unwrap();
    a.fake.state.lock().unwrap().running = false;
    clear(&a);
    let raw = arrived(&a, s, "DSC5.ARW");
    assert!(a.store().preview_state(&raw).unwrap().is_some());
    assert!(!ops(&a).iter().any(|op| {
        [
            "set_current_collection:",
            "find_image:",
            "launch",
            "apply_style:",
        ]
        .iter()
        .any(|p| op.starts_with(p))
    }));
}
#[test]
fn c3a_variant_wait_polls_once_per_second_without_render() {
    let a = app();
    let (s, _) = session(&a, "P49H");
    a.sessions().activate(s).unwrap();
    let raw = arrived(&a, s, "DSC6.ARW");
    clear(&a);
    for _ in 0..3 {
        a.tick(1_000).unwrap();
    }
    assert_eq!(
        ops(&a)
            .iter()
            .filter(|op| *op == &format!("find_image:{raw}"))
            .count(),
        3
    );
    assert!(a.fake.state.lock().unwrap().issued.is_empty());
    assert_eq!(a.app.previews.failed_generation_for(&raw), None);
}
#[test]
fn c3b_style_is_applied_before_first_render() {
    let mut a = app();
    let (s, _) = session(&a, "P49I");
    a.sessions().activate(s).unwrap();
    a.fake.state.lock().unwrap().styles = vec!["Bright Contrast".into()];
    let raw = arrived(&a, s, "DSC7.ARW");
    style(&mut a, "Bright Contrast");
    clear(&a);
    a.fake.add_image(&raw, &["v1"], false);
    a.tick(1_000).unwrap();
    let ops = ops(&a);
    let applied = ops
        .iter()
        .position(|op| op == "apply_style:v1:Bright Contrast")
        .unwrap();
    let processed = ops
        .iter()
        .position(|op| op == "process:v1:PSM Preview")
        .unwrap();
    assert!(applied < processed);
    assert_eq!(a.fake.state.lock().unwrap().issued.len(), 1);
}

#[test]
fn f1_retouch_after_first_render_does_not_receive_late_style() {
    let mut a = app();
    let (s, _) = session(&a, "P49F1");
    a.sessions().activate(s).unwrap();
    let raw = arrived(&a, s, "DSC7F1.ARW");
    a.fake.add_image(&raw, &["v1"], false);
    a.tick(1_000).unwrap();
    let first_job = a.fake.state.lock().unwrap().issued[0].0.clone();
    a.complete_render(&first_job).unwrap();

    a.fake.state.lock().unwrap().styles = vec!["Bright Contrast".into()];
    style(&mut a, "Bright Contrast");
    clear(&a);
    a.app.previews.on_variant_adjusted("v1", &raw).unwrap();
    a.tick(16_000).unwrap();

    assert!(a.fake.state.lock().unwrap().issued.len() >= 2);
    assert!(!ops(&a).iter().any(|op| op.starts_with("apply_style:")));
    assert_eq!(a.store().style_applied_at(&raw).unwrap(), None);
}

#[test]
fn tethered_capture_with_style_renders_without_applying_script_style() {
    let mut a = TestApp::with_fake_document().unwrap();
    let (s, key) = session(&a, "P49TETHERED");
    a.sessions().activate(s).unwrap();
    a.fake.state.lock().unwrap().styles = vec!["Bright Contrast".into()];
    style(&mut a, "Bright Contrast");

    let raw_path = folder(&a, &key).join("TETHERED.ARW");
    fs::write(&raw_path, b"tethered capture").unwrap();
    a.fake
        .add_image(raw_path.to_str().unwrap(), &["tethered-v1"], false);
    a.watcher_event(&raw_path).unwrap();
    a.tick(1_000).unwrap();

    assert_eq!(
        a.fake.state.lock().unwrap().issued.last().unwrap().1,
        "tethered-v1"
    );
    assert!(!ops(&a).iter().any(|op| op.starts_with("apply_style:")));
    let conn = rusqlite::Connection::open(a.app.paths.db()).unwrap();
    let style_marker: Option<i64> = conn
        .query_row(
            "SELECT style_applied_at FROM raw_state WHERE raw_path=?1",
            [canon(&raw_path).unwrap()],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(style_marker, None);
    assert_eq!(
        a.store()
            .style_applied_at(raw_path.to_str().unwrap())
            .unwrap(),
        None
    );
}

#[test]
fn c3c_late_variant_recovers_with_backoff_and_no_failure() {
    let mut a = app();
    let (s, _) = session(&a, "P49J");
    a.fake.state.lock().unwrap().styles = vec!["Bright Contrast".into()];
    style(&mut a, "Bright Contrast");
    a.sessions().activate(s).unwrap();
    let raw = arrived(&a, s, "DSC8.ARW");
    clear(&a);
    let mut polls_at = Vec::new();
    let mut previous = 0;
    for second in 1..=46 {
        a.tick(1_000).unwrap();
        assert_eq!(a.app.previews.failed_generation_for(&raw), None);
        let count = ops(&a)
            .iter()
            .filter(|op| *op == &format!("find_image:{raw}"))
            .count();
        for _ in previous..count {
            polls_at.push(second);
        }
        previous = count;
    }
    let mut expected: Vec<_> = (1..=15).collect();
    expected.extend([15, 16, 18, 22, 30, 46]);
    assert_eq!(polls_at, expected);
    a.fake.add_image(&raw, &["late"], false);
    a.tick(32_000).unwrap();
    assert_eq!(a.fake.state.lock().unwrap().issued[0].1, "late");
    assert!(a.store().style_applied_at(&raw).unwrap().is_some());
    let ops = ops(&a);
    assert!(
        ops.iter()
            .position(|op| op == "apply_style:late:Bright Contrast")
            .unwrap()
            < ops
                .iter()
                .position(|op| op == "process:late:PSM Preview")
                .unwrap()
    );
}
#[test]
fn c3d_collection_switch_retries_immediately() {
    let a = app();
    let (s, _) = session(&a, "P49K");
    a.sessions().activate(s).unwrap();
    let raw = arrived(&a, s, "DSC9.ARW");
    a.tick(20_000).unwrap();
    a.fake.add_image(&raw, &["v9"], false);
    a.fake.state.lock().unwrap().current_collection = other(&a);
    a.sessions().activate(s).unwrap();
    assert_eq!(a.fake.state.lock().unwrap().issued.last().unwrap().1, "v9");
}
#[test]
fn c4a_style_once_even_after_retouch() {
    let mut a = app();
    let (s, _) = session(&a, "P49L");
    a.sessions().activate(s).unwrap();
    a.fake.state.lock().unwrap().styles = vec!["Bright Contrast".into()];
    style(&mut a, "Bright Contrast");
    let raw = arrived(&a, s, "DSC10.ARW");
    a.fake.add_image(&raw, &["v10"], false);
    a.tick(1_000).unwrap();
    let job = a.fake.state.lock().unwrap().issued[0].0.clone();
    a.complete_render(&job).unwrap();
    a.app.previews.on_variant_adjusted("v10", &raw).unwrap();
    a.tick(16_000).unwrap();
    assert_eq!(
        ops(&a)
            .iter()
            .filter(|op| op.starts_with("apply_style:v10:"))
            .count(),
        1
    );
    assert!(a.store().style_applied_at(&raw).unwrap().is_some());
    assert_eq!(a.fake.state.lock().unwrap().issued.len(), 2);
}
#[test]
fn c4b_style_not_reapplied_after_restart() {
    let mut a = app();
    let (s, _) = session(&a, "P49M");
    a.sessions().activate(s).unwrap();
    a.fake.state.lock().unwrap().styles = vec!["Bright Contrast".into()];
    style(&mut a, "Bright Contrast");
    let raw = arrived(&a, s, "DSC11.ARW");
    a.fake.add_image(&raw, &["v11"], false);
    a.tick(1_000).unwrap();
    assert!(a.store().style_applied_at(&raw).unwrap().is_some());
    clear(&a);
    a.kill_and_restart().unwrap();
    a.app.startup().unwrap();
    a.tick(2_000).unwrap();
    assert!(!ops(&a).iter().any(|op| op.starts_with("apply_style:")));
}
#[test]
fn c4c_unlisted_style_sets_health_but_renders() {
    let mut a = app();
    let (s, _) = session(&a, "P49N");
    a.sessions().activate(s).unwrap();
    style(&mut a, "Nope");
    let raw = arrived(&a, s, "DSC12.ARW");
    a.fake.add_image(&raw, &["v12"], false);
    a.tick(1_000).unwrap();
    assert!(!ops(&a).iter().any(|op| op.starts_with("apply_style:")));
    let report = a.app.health.report("test", true, true);
    assert!(report.items.iter().any(
        |h| h.code == "style_not_found" && h.detail == "Style 'Nope' not found in Capture One"
    ));
    assert_eq!(a.fake.state.lock().unwrap().issued[0].1, "v12");
}
#[test]
fn c4d_empty_style_neither_applies_nor_warns() {
    let a = app();
    let (s, _) = session(&a, "P49O");
    a.sessions().activate(s).unwrap();
    let raw = arrived(&a, s, "DSC13.ARW");
    a.fake.add_image(&raw, &["v13"], false);
    a.tick(1_000).unwrap();
    assert!(!ops(&a).iter().any(|op| op.starts_with("apply_style:")));
    assert!(!a.app.health.has("style_not_found"));
}
#[test]
fn c4e_each_raw_receives_style_on_its_own_variant() {
    let mut a = app();
    let (s, _) = session(&a, "P49P");
    a.sessions().activate(s).unwrap();
    a.fake.state.lock().unwrap().styles = vec!["Bright Contrast".into()];
    style(&mut a, "Bright Contrast");
    let one = arrived(&a, s, "DSC14.ARW");
    let two = arrived(&a, s, "DSC15.ARW");
    a.fake.add_image(&one, &["v14"], false);
    a.fake.add_image(&two, &["v15"], false);
    a.tick(1_000).unwrap();
    assert_eq!(
        ops(&a)
            .iter()
            .filter(|op| *op == "apply_style:v14:Bright Contrast")
            .count(),
        1
    );
    assert_eq!(
        ops(&a)
            .iter()
            .filter(|op| *op == "apply_style:v15:Bright Contrast")
            .count(),
        1
    );
}
#[test]
fn c5a_import_copies_and_leaves_original() {
    let a = app();
    let (s, key) = session(&a, "P49Q");
    a.sessions().activate(s).unwrap();
    let original = a.dir.path().join("IMG.ARW");
    fs::write(&original, b"x").unwrap();
    import(&a, &original);
    let dest = folder(&a, &key).join("IMG.ARW");
    assert_eq!(fs::read(&dest).unwrap(), b"x");
    assert_eq!(fs::read(&original).unwrap(), b"x");
    assert_eq!(
        import_rows(&a),
        vec![(s, canon(&dest).unwrap(), "import".into())]
    );
    assert!(!ops(&a).iter().any(|op| op.starts_with("import:")));
}
#[test]
fn c5b_import_collision_keeps_both_files_and_rows() {
    let a = app();
    let (s, key) = session(&a, "P49R");
    a.sessions().activate(s).unwrap();
    let original = a.dir.path().join("IMG.ARW");
    fs::write(&original, b"x").unwrap();
    import(&a, &original);
    fs::write(&original, b"y").unwrap();
    import(&a, &original);
    assert_eq!(fs::read(folder(&a, &key).join("IMG.ARW")).unwrap(), b"x");
    assert_eq!(
        fs::read(folder(&a, &key).join("IMG__dup1.ARW")).unwrap(),
        b"y"
    );
    assert_eq!(import_rows(&a).len(), 2);
}
#[test]
fn c5c_unassigned_import_follows_its_folder() {
    let a = app();
    a.sessions().deactivate().unwrap();
    a.fake.state.lock().unwrap().current_collection = other(&a);
    clear(&a);
    let original = a.dir.path().join("IMG2.ARW");
    fs::write(&original, b"z").unwrap();
    import(&a, &original);
    let dest = folder(&a, "_Unassigned").join("IMG2.ARW");
    assert_eq!(fs::read(&dest).unwrap(), b"z");
    assert_eq!(
        import_rows(&a),
        vec![(1, canon(&dest).unwrap(), "import".into())]
    );
    assert!(ops(&a).contains(&format!(
        "set_current_collection:{}",
        folder(&a, "_Unassigned").display()
    )));
}
