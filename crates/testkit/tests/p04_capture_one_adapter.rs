use psm_core::{
    Error,
    domain::{
        editing::{CameraStatus, RenderPurpose},
        photo::Classification as C,
    },
    ports::clock::Clock,
    services::camera::CameraService,
};
use psm_integrations::editing::capture_one::scripting::CaptureOneScripting;
use psm_testkit::TestApp;

fn app() -> TestApp {
    TestApp::with_fake_document().unwrap()
}
fn session(a: &TestApp, sku: &str) -> i64 {
    a.items()
        .create_local(sku, sku, "object", None)
        .unwrap()
        .1
        .session
        .id
}
fn photo(a: &TestApp, s: i64, name: &str, id: &str) -> i64 {
    let p = a.add_photo(s, name).unwrap();
    a.fake.add_image(&p.raw_path, &[id], false);
    p.id
}
fn ops(a: &TestApp) -> Vec<String> {
    a.fake.state.lock().unwrap().ops.clone()
}
fn capture(a: &TestApp) -> String {
    a.fake.state.lock().unwrap().captures.clone()
}
fn status(a: &TestApp) -> CameraStatus {
    CameraService::new(a.app.camera.as_ref(), a.app.health.as_ref())
        .status()
        .unwrap()
}
fn tag(a: &TestApp, id: &str) -> i32 {
    a.fake.get_color_tag(id).unwrap()
}
fn last_job(a: &TestApp) -> (String, i64, Option<String>, Option<i64>) {
    let conn = rusqlite::Connection::open(a.app.paths.db()).unwrap();
    conn.query_row(
        "SELECT status,attempts,error,next_attempt_at FROM jobs ORDER BY id DESC LIMIT 1",
        [],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
    )
    .unwrap()
}

#[test]
fn c1a_activate_saves_creates_verifies_and_persists() {
    let a = app();
    let s = session(&a, "K1");
    a.fake.state.lock().unwrap().private_prefix = true;
    a.sessions().activate(s).unwrap();
    let path = a.dir.path().join("Capture/K1");
    assert!(path.is_dir());
    assert_eq!(
        std::fs::canonicalize(capture(&a)).unwrap(),
        std::fs::canonicalize(path).unwrap()
    );
    let o = ops(&a);
    assert!(
        o.iter().position(|x| x == "save").unwrap()
            < o.iter()
                .position(|x| x.starts_with("set_captures:"))
                .unwrap()
    );
    assert!(!a.app.health.has("capture_folder_not_set"));
    assert_eq!(a.store().active_session_id().unwrap(), Some(s));
}
#[test]
fn c1b_failed_set_restores_and_blocks_camera() {
    let a = app();
    let r = session(&a, "R");
    let s = session(&a, "K1");
    a.sessions().activate(r).unwrap();
    let previous = capture(&a);
    a.fake.state.lock().unwrap().fail_next_set_captures = true;
    assert!(matches!(
        a.sessions().activate(s),
        Err(Error::CaptureFolderNotSet)
    ));
    assert_eq!(capture(&a), previous);
    assert_eq!(a.store().active_session_id().unwrap(), Some(r));
    assert!(a.app.health.has("capture_folder_not_set"));
    assert_eq!(status(&a), CameraStatus::Disconnected);
    assert!(matches!(
        CameraService::new(a.app.camera.as_ref(), a.app.health.as_ref()).capture(),
        Err(Error::CaptureFolderNotSet)
    ));
    assert!(!ops(&a).contains(&"capture".into()));
}
#[test]
fn c1c_readback_mismatch_restores_and_blocks_camera() {
    let a = app();
    let r = session(&a, "R");
    let s = session(&a, "K1");
    a.sessions().activate(r).unwrap();
    let previous = capture(&a);
    a.fake.state.lock().unwrap().readback_garbage = true;
    assert!(matches!(
        a.sessions().activate(s),
        Err(Error::CaptureFolderNotSet)
    ));
    assert_eq!(capture(&a), previous);
    assert_eq!(a.store().active_session_id().unwrap(), Some(r));
    assert!(a.app.health.has("capture_folder_not_set"));
    assert_eq!(status(&a), CameraStatus::Disconnected);
    assert!(matches!(
        CameraService::new(a.app.camera.as_ref(), a.app.health.as_ref()).capture(),
        Err(Error::CaptureFolderNotSet)
    ));
    assert!(!ops(&a).contains(&"capture".into()));
}
#[test]
fn c1d_deactivate_targets_unassigned() {
    let a = app();
    let s = session(&a, "K1");
    a.sessions().activate(s).unwrap();
    a.sessions().deactivate().unwrap();
    let target = a.dir.path().join("Capture/_Unassigned");
    assert!(target.is_dir());
    assert_eq!(capture(&a), target.to_string_lossy());
    assert_eq!(a.store().active_session_id().unwrap(), None);
}
#[test]
fn c2a_hooks_read_back_canonically() {
    let a = app();
    a.fake.state.lock().unwrap().private_prefix = true;
    a.app.editing.install_hooks().unwrap();
    let s = a.fake.state.lock().unwrap();
    for name in ["capture_done", "processing_done", "variant_adjusted"] {
        assert_eq!(
            std::fs::canonicalize(s.hooks.get(name).unwrap()).unwrap(),
            std::fs::canonicalize(a.app.paths.hooks().join(format!("{name}.scpt"))).unwrap()
        );
    }
    assert!(!a.app.health.has("hooks_not_installed"));
}
#[test]
fn c2b_null_hook_readback_is_unhealthy() {
    let a = app();
    a.fake.state.lock().unwrap().hooks_readback_null = true;
    assert!(matches!(
        a.app.editing.install_hooks(),
        Err(Error::HooksNotInstalled)
    ));
    assert!(a.app.health.has("hooks_not_installed"));
}
#[test]
fn c3a_existing_recipes_are_corrected() {
    let a = app();
    assert!(a.app.editing.check_recipes().unwrap().is_empty());
    let s = a.fake.state.lock().unwrap();
    for name in ["PSM Preview", "PSM Web"] {
        assert_eq!(
            s.recipes.get(name).unwrap(),
            &("overwrite".into(), "_session_manager".into())
        );
    }
    assert!(!a.app.health.has("recipe_missing"));
}
#[test]
fn c3b_missing_recipe_only_blocks_its_render() {
    let a = app();
    a.fake.state.lock().unwrap().recipes.remove("PSM Web");
    let s = session(&a, "K1");
    let p = photo(&a, s, "a.ARW", "7");
    let raw = a.store().photo(p).unwrap().raw_path;
    assert_eq!(a.app.editing.check_recipes().unwrap(), vec!["PSM Web"]);
    assert_eq!(
        a.fake
            .state
            .lock()
            .unwrap()
            .recipes
            .get("PSM Preview")
            .unwrap(),
        &("overwrite".into(), "_session_manager".into())
    );
    let h = a.app.health.report("", true, true);
    assert_eq!(
        h.items
            .iter()
            .filter(|i| i.code == "recipe_missing")
            .map(|i| i.detail.as_str())
            .collect::<Vec<_>>(),
        ["PSM Web"]
    );
    assert!(
        matches!(a.app.editing.render(&raw,RenderPurpose::Export),Err(Error::RecipeMissing{name}) if name=="PSM Web")
    );
    assert_eq!(
        a.app.editing.render(&raw, RenderPurpose::Preview).unwrap(),
        "job-1"
    );
}
#[test]
fn c4a_render_picks_position_one_in_current_collection() {
    let a = app();
    let s = session(&a, "K1");
    let p = photo(&a, s, "a.ARW", "7");
    let raw = a.store().photo(p).unwrap().raw_path;
    a.fake.add_image(&raw, &["7", "3"], false);
    assert_eq!(
        a.app.editing.render(&raw, RenderPurpose::Preview).unwrap(),
        "job-1"
    );
    assert_eq!(
        ops(&a)
            .iter()
            .filter(|o| o.starts_with("process:"))
            .cloned()
            .collect::<Vec<_>>(),
        ["process:7:PSM Preview"]
    );
}
#[test]
fn c4b_render_falls_back_to_folder_collection() {
    let a = app();
    let s = session(&a, "K1");
    let p = a.add_photo(s, "a.ARW").unwrap();
    a.fake.add_image(&p.raw_path, &["9", "3"], true);
    assert_eq!(
        a.app
            .editing
            .render(&p.raw_path, RenderPurpose::Preview)
            .unwrap(),
        "job-1"
    );
    assert_eq!(
        ops(&a)
            .iter()
            .filter(|o| o.starts_with("process:"))
            .cloned()
            .collect::<Vec<_>>(),
        ["process:9:PSM Preview"]
    );
}
#[test]
fn c4c_missing_variant_retries_and_reports_health() {
    let a = app();
    let s = session(&a, "K1");
    let p = a.add_photo(s, "a.ARW").unwrap();
    a.review().classify(p.id, C::Main).unwrap();
    a.tick(0).unwrap();
    assert_eq!(
        last_job(&a),
        (
            "queued".into(),
            1,
            Some("variant_not_found".into()),
            Some(1000)
        )
    );
    assert!(a.app.health.has("variant_not_found"));
}
#[test]
fn c5a_coalesced_changes_write_latest_once() {
    let a = app();
    let s = session(&a, "K1");
    let p = photo(&a, s, "a.ARW", "7");
    a.review().classify(p, C::Condition).unwrap();
    a.review().delete(p).unwrap();
    a.review().classify(p, C::Main).unwrap();
    a.tick(0).unwrap();
    assert_eq!(tag(&a, "7"), 4);
    let o = ops(&a);
    assert_eq!(
        o.iter()
            .filter(|x| x.starts_with("set_color_tag:7:"))
            .count(),
        1
    );
    let write = o.iter().position(|x| x == "set_color_tag:7:4").unwrap();
    assert!(o.iter().skip(write + 1).any(|x| x == "get_color_tag:7"));
}
#[test]
fn c5b_dropped_tag_write_retries() {
    let a = app();
    let s = session(&a, "K1");
    let p = photo(&a, s, "a.ARW", "7");
    a.fake.state.lock().unwrap().drop_tag_writes = true;
    a.review().classify(p, C::Main).unwrap();
    a.tick(0).unwrap();
    assert_eq!(
        last_job(&a),
        (
            "queued".into(),
            1,
            Some("tag_verification".into()),
            Some(1000)
        )
    );
}
#[test]
fn c5c_own_write_timestamp_is_upserted() {
    let a = app();
    let s = session(&a, "K1");
    let p = photo(&a, s, "a.ARW", "7");
    let raw = a.store().photo(p).unwrap().raw_path;
    a.clock.advance(2123);
    a.review().classify(p, C::Main).unwrap();
    a.tick(0).unwrap();
    let conn = rusqlite::Connection::open(a.app.paths.db()).unwrap();
    let stamp: i64 = conn
        .query_row(
            "SELECT last_own_write_at FROM raw_state WHERE raw_path=?1",
            [raw],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(stamp, a.clock.now_ms());
}
#[test]
fn c5d_last_enqueued_copy_row_wins() {
    let a = app();
    let s1 = session(&a, "K1");
    let s2 = session(&a, "K2");
    let p = photo(&a, s1, "a.ARW", "7");
    let raw = a.store().photo(p).unwrap().raw_path;
    a.review().copy_photos(&[p], s2).unwrap();
    let conn = rusqlite::Connection::open(a.app.paths.db()).unwrap();
    let copy: i64 = conn
        .query_row(
            "SELECT id FROM photos WHERE session_id=?1 AND raw_path=?2",
            rusqlite::params![s2, raw],
            |r| r.get(0),
        )
        .unwrap();
    a.review().classify(copy, C::Condition).unwrap();
    a.clock.advance(1);
    a.review().classify(p, C::Main).unwrap();
    a.tick(0).unwrap();
    assert_eq!(tag(&a, "7"), 4);
}
#[test]
fn c5e_four_classifications_map_to_tags() {
    let a = app();
    let s = session(&a, "K1");
    let p1 = photo(&a, s, "1.ARW", "1");
    let p2 = photo(&a, s, "2.ARW", "2");
    let p3 = photo(&a, s, "3.ARW", "3");
    let p4 = photo(&a, s, "4.ARW", "4");
    a.review().delete(p1).unwrap();
    a.review().classify(p2, C::Main).unwrap();
    a.review().classify(p2, C::Unreviewed).unwrap();
    a.review().classify(p3, C::Main).unwrap();
    a.review().classify(p4, C::Condition).unwrap();
    a.tick(0).unwrap();
    assert_eq!(
        [tag(&a, "1"), tag(&a, "2"), tag(&a, "3"), tag(&a, "4")],
        [1, 0, 4, 5]
    );
}
#[test]
fn c6a_timeout_read_first_on_retry() {
    let a = app();
    let s = session(&a, "K1");
    let p = photo(&a, s, "a.ARW", "7");
    let raw = a.store().photo(p).unwrap().raw_path;
    a.fake.state.lock().unwrap().timeout_next_set_tag = true;
    a.review().classify(p, C::Main).unwrap();
    a.tick(0).unwrap();
    let conn = rusqlite::Connection::open(a.app.paths.db()).unwrap();
    let stamp: i64 = conn
        .query_row(
            "SELECT last_own_write_at FROM raw_state WHERE raw_path=?1",
            [raw],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(stamp, 0);
    a.tick(1000).unwrap();
    assert_eq!(tag(&a, "7"), 4);
    assert_eq!(
        ops(&a)
            .iter()
            .filter(|x| x.starts_with("set_color_tag:7:"))
            .count(),
        1
    );
}
#[test]
fn c7a_connected_camera_name() {
    let a = app();
    assert_eq!(status(&a), CameraStatus::Connected("Sony ZV-E10".into()));
}
#[test]
fn c7b_absent_camera_is_disconnected() {
    let a = app();
    a.fake.state.lock().unwrap().camera = None;
    assert_eq!(status(&a), CameraStatus::Disconnected);
    assert!(a.app.health.has("camera_disconnected"));
}
#[test]
fn c7c_healthy_capture_reaches_provider() {
    let a = app();
    let s = session(&a, "K1");
    a.sessions().activate(s).unwrap();
    CameraService::new(a.app.camera.as_ref(), a.app.health.as_ref())
        .capture()
        .unwrap();
    assert_eq!(ops(&a).iter().filter(|x| *x == "capture").count(), 1);
}
#[test]
fn c8a_catalog_stops_before_hook_install() {
    let a = app();
    a.fake.state.lock().unwrap().kind = "catalog".into();
    assert!(matches!(a.app.startup(), Err(Error::DocumentNotSession)));
    assert!(a.app.health.has("document_not_session"));
    assert!(!ops(&a).iter().any(|x| x.starts_with("set_hook:")));
}
