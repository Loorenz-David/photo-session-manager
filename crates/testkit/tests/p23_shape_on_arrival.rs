use psm_core::{Error, mechanisms::canon::canon, services::photos::PhotosService};
use psm_infra::paths::AppPaths;
use psm_integrations::editing::capture_one::{jxa::JxaScripting, shape::CROP_GIVE_UP_ATTEMPTS};
use psm_testkit::TestApp;
use rusqlite::Connection;
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

fn script(config: Value, name: &str) -> Value {
    let dir = tempfile::tempdir().unwrap();
    let paths = AppPaths::for_dir(dir.path());
    JxaScripting::new(&paths).unwrap();
    let harness = r#"
const fs = require('fs');
const cfg = JSON.parse(process.argv[2]);
const ratios = ['Unconstrained','Original','Output','Square','1x2','1x3','2x3','3x4','4x5','5x7','16x9'];
let ratio = Object.prototype.hasOwnProperty.call(cfg, 'ratio') ? cfg.ratio : '',
 width = cfg.width ?? 5981, height = cfg.height ?? 3987;
let calls = [], writes = [];
const variant = {availableCropAspectRatios: () => ratios, cropAspectRatio: () => ratio,
 cropWidth: () => width, cropHeight: () => height};
Object.defineProperty(variant, 'cropAspectRatio', {get: () => () => ratio, set: v => {
 ratio = v; writes.push(v); if (v === 'Square') {width = 3987; height = 3987;}
}});
const image = {variants: Object.assign(() => [variant], {byId: () => variant})};
const images = Object.assign(() => [image], {whose: ({path}) => () => path === '/raw' ? [image] : []});
const doc = {images, collections: () => []};
const app = {running: () => true, documents: {byId: () => doc}, maximumCrop: (_v, options) => {
 calls.push(options); return cfg.null_max ? null : {x:3000,y:1993,width:2981,height:1994};
}};
global.Application = () => app;
const source = fs.readFileSync(process.argv[1], 'utf8');
eval(source);
let result, error;
try { result = JSON.parse(run(cfg.list ? ['/doc'] : ['/doc','/raw','/folder','7',cfg.request ?? 'Square'])); }
catch (e) { error = String(e); }
console.log(JSON.stringify({result,error,calls,writes}));
"#;
    let output = Command::new("node")
        .arg("-e")
        .arg(harness)
        .arg(paths.hooks().join("jxa").join(format!("{name}.js")))
        .arg(config.to_string())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}
fn app() -> TestApp {
    TestApp::with_fake_document().unwrap()
}
fn root(a: &TestApp) -> PathBuf {
    a.dir.path().join("Capture One Session/Capture")
}
fn file(a: &TestApp, name: &str) -> PathBuf {
    let path = root(a).join(name);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, b"raw").unwrap();
    path
}
fn hook(a: &TestApp, path: &Path) {
    a.deliver_hook(&json!({"hook":"capture_done","argv":[path.to_string_lossy()]}).to_string())
        .unwrap();
}
fn db(a: &TestApp) -> Connection {
    Connection::open(a.app.paths.db()).unwrap()
}
fn jobs(a: &TestApp) -> Vec<(String, String, i64, String, i64)> {
    let conn = db(a);
    let mut stmt = conn.prepare("SELECT target,status,attempts,coalesce(error,''),next_attempt_at FROM jobs WHERE kind='crop_on_arrival' ORDER BY id").unwrap();
    stmt.query_map([], |r| {
        Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
    })
    .unwrap()
    .map(Result::unwrap)
    .collect()
}
fn variant(a: &TestApp, id: &str) -> (String, (i64, i64)) {
    let state = a.fake.state.lock().unwrap();
    let v = state
        .images
        .values()
        .chain(state.folder_images.values())
        .flatten()
        .find(|v| v.id == id)
        .unwrap();
    (v.crop_ratio.clone(), v.crop)
}
fn ops(a: &TestApp, prefix: &str) -> Vec<String> {
    a.fake
        .state
        .lock()
        .unwrap()
        .ops
        .iter()
        .filter(|s| s.starts_with(prefix))
        .cloned()
        .collect()
}
fn service(a: &TestApp) -> PhotosService<'_> {
    PhotosService {
        arrival: a.app.arrival.as_ref(),
        editing: a.app.editing.as_ref(),
        store: a.store(),
        clock: a.clock.as_ref(),
    }
}
fn health(a: &TestApp, code: &str) -> Option<String> {
    a.app
        .health_check()
        .unwrap()
        .items
        .iter()
        .find(|i| i.code == code && !i.ok)
        .map(|i| i.detail.clone())
}

#[test]
fn c1a_untouched_frame_is_shaped_in_one_call() {
    let out = script(json!({"ratio":null}), "apply_crop");
    assert_eq!(out["result"]["outcome"], "applied");
    assert_eq!(out["result"]["ratio"], "Square");
    assert_eq!(out["calls"], json!([{"apply":false}]));
    assert_eq!(out["writes"], json!(["Square"]));

    let empty = script(json!({"ratio":""}), "apply_crop");
    assert_eq!(empty["result"]["outcome"], "applied");
    assert_eq!(empty["writes"], json!(["Square"]));
}
#[test]
fn c1b_hand_crops_are_skipped() {
    let fixed = script(json!({"ratio":"4x5","width":4784}), "apply_crop");
    assert_eq!(fixed["result"]["outcome"], "skipped");
    assert_eq!(fixed["calls"], json!([]));
    assert_eq!(fixed["writes"], json!([]));
    let free = script(
        json!({"ratio":"Unconstrained","width":3000,"height":2000}),
        "apply_crop",
    );
    assert_eq!(free["result"]["outcome"], "skipped");
    assert_eq!(free["calls"], json!([{"apply":false}]));
    assert_eq!(free["writes"], json!([]));
}
#[test]
fn c1c_unknown_ratio_and_unreadable_max_never_write() {
    let unknown = script(json!({"request":"Squircle"}), "apply_crop");
    assert_eq!(unknown["result"]["outcome"], "unknown_ratio");
    assert_eq!(
        unknown["result"]["available"],
        json!([
            "Unconstrained",
            "Original",
            "Output",
            "Square",
            "1x2",
            "1x3",
            "2x3",
            "3x4",
            "4x5",
            "5x7",
            "16x9"
        ])
    );
    assert_eq!(unknown["calls"], json!([]));
    assert_eq!(unknown["writes"], json!([]));
    let bad = script(json!({"null_max":true}), "apply_crop");
    assert!(
        bad["error"]
            .as_str()
            .unwrap()
            .contains("maximum crop size unavailable")
    );
    assert_eq!(bad["writes"], json!([]));
    let names = script(json!({"list":true}), "crop_aspect_ratios");
    assert_eq!(names["result"], unknown["result"]["available"]);
}
#[test]
fn c2a_capture_shapes_without_retouch() {
    let a = app();
    let sid = a
        .items()
        .create_local("K1", "K1", "object", None)
        .unwrap()
        .1
        .session
        .id;
    a.sessions().activate(sid).unwrap();
    let p = file(&a, "K1/a.ARW");
    let raw = canon(&p).unwrap();
    a.fake.add_image(&raw, &["7"], false);
    hook(&a, &p);
    a.tick(0).unwrap();
    assert_eq!(
        jobs(&a),
        vec![(raw.clone(), "done".into(), 0, "".into(), 0)]
    );
    assert_eq!(variant(&a, "7"), ("Square".into(), (3987, 3987)));
    let state = a.store().preview_state(&raw).unwrap().unwrap();
    assert_eq!(state.last_own_write_at, Some(0));
    assert_eq!(state.retouch_gen, 0);
    let before: i64 = db(&a)
        .query_row("SELECT change_seq FROM sessions WHERE id=?1", [sid], |r| {
            r.get(0)
        })
        .unwrap();
    a.deliver_hook(&json!({"hook":"variant_adjusted","variant_id":"7","raw":raw}).to_string())
        .unwrap();
    let state = a.store().preview_state(&raw).unwrap().unwrap();
    assert_eq!(state.retouch_gen, 0);
    assert!(state.adjusted_gen >= 1);
    let after: i64 = db(&a)
        .query_row("SELECT change_seq FROM sessions WHERE id=?1", [sid], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(after, before);
    let row_sid: i64 = db(&a)
        .query_row(
            "SELECT session_id FROM photos WHERE raw_path=?1",
            [raw],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(row_sid, sid);
}
#[test]
fn c2b_import_attempts_each_raw_only_once() {
    let mut a = app();
    let x = a.dir.path().join("x.ARW");
    let y = a.dir.path().join("y.ARW");
    fs::write(&x, b"x").unwrap();
    fs::write(&y, b"y").unwrap();
    let rx = canon(&x).unwrap();
    let ry = canon(&y).unwrap();
    a.fake.add_image(&rx, &["11"], false);
    a.fake.add_image(&ry, &["12"], false);
    a.fake.set_variant_crop("12", "4x5", (4784, 3987));
    service(&a)
        .import(&[
            x.to_string_lossy().into_owned(),
            y.to_string_lossy().into_owned(),
        ])
        .unwrap();
    a.tick(0).unwrap();
    assert_eq!(jobs(&a).len(), 2);
    assert!(jobs(&a).iter().all(|j| j.1 == "done"));
    assert_eq!(variant(&a, "11"), ("Square".into(), (3987, 3987)));
    assert_eq!(variant(&a, "12"), ("4x5".into(), (4784, 3987)));
    a.kill_and_restart().unwrap();
    a.scan().unwrap();
    service(&a)
        .import(&[
            x.to_string_lossy().into_owned(),
            y.to_string_lossy().into_owned(),
        ])
        .unwrap();
    a.tick(0).unwrap();
    assert_eq!(jobs(&a).len(), 2);
    assert_eq!(ops(&a, "set_crop:"), vec!["set_crop:11:Square"]);
}
#[test]
fn c2c_off_duplicate_delivery_and_copy_move_do_not_shape_again() {
    let mut a = app();
    a.settings_set(&json!({"crop_on_arrival":"off"})).unwrap();
    let p = file(&a, "K1/b.ARW");
    let raw = canon(&p).unwrap();
    a.fake.add_image(&raw, &["8"], false);
    hook(&a, &p);
    a.tick(0).unwrap();
    assert!(jobs(&a).is_empty());
    assert!(ops(&a, "apply_crop:").is_empty());
    let b = app();
    let p = file(&b, "K1/c.ARW");
    let raw = canon(&p).unwrap();
    b.fake.add_image(&raw, &["9"], false);
    hook(&b, &p);
    b.watcher_event(&p).unwrap();
    b.tick(1000).unwrap();
    assert_eq!(jobs(&b).len(), 1);
    assert_eq!(ops(&b, "set_crop:"), vec!["set_crop:9:Square"]);
    let c = app();
    let s1 = c
        .items()
        .create_local("S1", "S1", "object", None)
        .unwrap()
        .1
        .session
        .id;
    let s2 = c
        .items()
        .create_local("S2", "S2", "object", None)
        .unwrap()
        .1
        .session
        .id;
    let s3 = c
        .items()
        .create_local("S3", "S3", "object", None)
        .unwrap()
        .1
        .session
        .id;
    let photo = c.add_photo(s1, "d.ARW").unwrap();
    c.fake.add_image(&photo.raw_path, &["10"], false);
    c.mark_published(s1).unwrap();
    c.review().copy_photos(&[photo.id], s2).unwrap();
    c.review().move_photos(&[photo.id], s3).unwrap();
    c.tick(0).unwrap();
    assert!(jobs(&c).is_empty());
    assert!(ops(&c, "apply_crop:").is_empty());
}
#[test]
fn c3a_missing_variant_retries_then_folder_variant_applies() {
    let a = app();
    let p = file(&a, "K1/e.ARW");
    let raw = canon(&p).unwrap();
    hook(&a, &p);
    a.tick(0).unwrap();
    assert_eq!(
        jobs(&a)[0],
        (
            raw.clone(),
            "queued".into(),
            1,
            "variant_not_found".into(),
            1000
        )
    );
    assert_eq!(health(&a, "variant_not_found"), Some(raw.clone()));
    a.fake.add_image(&raw, &["13"], true);
    a.tick(1000).unwrap();
    assert_eq!(jobs(&a)[0].1, "done");
    assert_eq!(variant(&a, "13"), ("Square".into(), (3987, 3987)));
    assert!(!ops(&a, "find_image_in_folder:").is_empty());
    assert_eq!(health(&a, "variant_not_found"), None);
    assert_eq!(health(&a, "crop_not_applied"), None);
}
#[test]
fn c3b_retrying_shape_marks_preview_stale_and_rerenders() {
    let a = app();
    let p = file(&a, "K1/h.ARW");
    let raw = canon(&p).unwrap();
    a.fake.add_image(&raw, &["16"], false);
    a.fake.state.lock().unwrap().fail_apply_crop = Some("busy".into());
    hook(&a, &p);
    a.tick(0).unwrap();
    assert_eq!(
        jobs(&a)[0],
        (raw.clone(), "queued".into(), 1, "provider".into(), 1000)
    );
    assert_eq!(ops(&a, "process:16:PSM Preview").len(), 1);
    a.complete_all_renders().unwrap();
    a.fake.state.lock().unwrap().fail_apply_crop = None;
    a.tick(1000).unwrap();
    assert_eq!(jobs(&a)[0].1, "done");
    assert_eq!(variant(&a, "16").0, "Square");
    let state = a.store().preview_state(&raw).unwrap().unwrap();
    assert!(state.adjusted_gen > state.rendered_gen);
    assert_eq!(state.retouch_gen, 0);
    a.tick(15000).unwrap();
    assert_eq!(ops(&a, "process:16:PSM Preview").len(), 2);
}
fn drive_to_failure(a: &TestApp) {
    for delay in [0, 1000, 2000, 4000, 8000, 16000, 32000, 60000] {
        a.tick(delay).unwrap();
    }
    assert_eq!(jobs(a)[0].1, "failed");
}
#[test]
fn c3c_give_up_health_requeues_and_success_clears() {
    let a = app();
    let p = file(&a, "K1/f.ARW");
    let raw = canon(&p).unwrap();
    a.fake.add_image(&raw, &["14"], false);
    a.fake.state.lock().unwrap().fail_apply_crop = Some("busy".into());
    hook(&a, &p);
    drive_to_failure(&a);
    assert_eq!(jobs(&a)[0].2, CROP_GIVE_UP_ATTEMPTS);
    assert_eq!(jobs(&a)[0].3, "provider");
    assert_eq!(variant(&a, "14").0, "");
    assert_eq!(
        health(&a, "crop_not_applied"),
        Some("Shape not applied to 1 photo".into())
    );
    assert_eq!(jobs(&a)[0].1, "queued");
    assert_eq!(jobs(&a)[0].2, 0);
    drive_to_failure(&a);
    assert_eq!(
        health(&a, "crop_not_applied"),
        Some("Shape not applied to 1 photo".into())
    );
    a.fake.state.lock().unwrap().fail_apply_crop = None;
    a.tick(0).unwrap();
    assert_eq!(jobs(&a)[0].1, "done");
    assert_eq!(variant(&a, "14").0, "Square");
    assert_eq!(health(&a, "crop_not_applied"), None);
    let mut b = app();
    let p = file(&b, "K1/f.ARW");
    let raw = canon(&p).unwrap();
    b.fake.add_image(&raw, &["14"], false);
    b.fake.state.lock().unwrap().fail_apply_crop = Some("busy".into());
    hook(&b, &p);
    drive_to_failure(&b);
    b.kill_and_restart().unwrap();
    assert_eq!(jobs(&b)[0].1, "queued");
    assert_eq!(jobs(&b)[0].2, 0);
}
#[test]
fn c3d_unknown_name_has_distinct_health_then_retries_after_correction() {
    let mut a = app();
    a.settings_set(&json!({"crop_on_arrival":"Squircle"}))
        .unwrap();
    assert_eq!(
        health(&a, "crop_ratio_unknown"),
        Some("Shape on arrival: 'Squircle' is not a Capture One crop ratio".into())
    );
    let p = file(&a, "K1/g.ARW");
    let raw = canon(&p).unwrap();
    a.fake.add_image(&raw, &["15"], false);
    hook(&a, &p);
    a.tick(0).unwrap();
    assert_eq!(jobs(&a)[0].1, "failed");
    assert_eq!(jobs(&a)[0].3, "crop_ratio_unknown");
    assert_eq!(variant(&a, "15").0, "");
    assert!(ops(&a, "set_crop:").is_empty());
    assert_eq!(
        health(&a, "crop_ratio_unknown"),
        Some("Shape on arrival: 'Squircle' is not a Capture One crop ratio".into())
    );
    assert_eq!(health(&a, "crop_not_applied"), None);
    a.settings_set(&json!({"crop_on_arrival":"Square"}))
        .unwrap();
    assert_eq!(health(&a, "crop_ratio_unknown"), None);
    assert_eq!(jobs(&a)[0].1, "queued");
    a.tick(0).unwrap();
    assert_eq!(jobs(&a)[0].1, "done");
    assert_eq!(variant(&a, "15").0, "Square");
    assert_eq!(health(&a, "crop_ratio_unknown"), None);
    assert_eq!(health(&a, "crop_not_applied"), None);
}
#[test]
fn c4a_setting_defaults_validates_and_lists_provider_names() {
    let mut a = TestApp::new().unwrap();
    assert_eq!(a.app.settings_get().unwrap()["crop_on_arrival"], "Square");
    a.settings_set(&json!({"crop_on_arrival":"off"})).unwrap();
    let stored: Value = serde_json::from_slice(&fs::read(a.app.paths.settings()).unwrap()).unwrap();
    assert_eq!(stored["crop_on_arrival"], "off");
    assert_eq!(a.app.settings_get().unwrap()["crop_on_arrival"], "off");
    assert!(matches!(
        a.settings_set(&json!({"crop_on_arrival":""})),
        Err(Error::InvalidValue(_))
    ));
    assert!(matches!(
        a.settings_set(&json!({"crop_on_arrival":5})),
        Err(Error::InvalidValue(_))
    ));
    assert_eq!(a.app.settings_get().unwrap()["crop_on_arrival"], "off");
    let names = app().app.editing.crop_aspect_ratios().unwrap();
    assert_eq!(names.len(), 11);
    assert!(names.contains(&"Square".into()));
}
