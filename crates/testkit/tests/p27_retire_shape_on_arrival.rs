use psm_core::{Error, mechanisms::canon::canon, services::photos::PhotosService};
use psm_integrations::editing::capture_one::recipes::{
    MASTER_TIFF_COMPRESSION, RecipeSpec, SRGB_PROFILE,
};
use psm_testkit::TestApp;
use rusqlite::Connection;
use serde_json::{Value, json};
use std::{fs, path::PathBuf};

fn jobs(a: &TestApp, raw: &str) -> Vec<String> {
    let conn = Connection::open(a.app.paths.db()).unwrap();
    let mut stmt = conn
        .prepare("SELECT kind FROM jobs WHERE target=?1 ORDER BY kind")
        .unwrap();
    stmt.query_map([raw], |r| r.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect()
}
fn capture(a: &TestApp, relative: &str) -> PathBuf {
    let path = a
        .dir
        .path()
        .join("Capture One Session/Capture")
        .join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, b"raw").unwrap();
    path
}
fn ops(a: &TestApp) -> Vec<String> {
    a.fake.state.lock().unwrap().ops.clone()
}
fn no_crop_ops(a: &TestApp) {
    assert!(
        !ops(a)
            .iter()
            .any(|op| op.starts_with("apply_crop:") || op.starts_with("crop_aspect_ratios"))
    );
}
fn stored(a: &TestApp) -> Value {
    serde_json::from_slice(&fs::read(a.app.paths.settings()).unwrap()).unwrap()
}

#[test]
fn c1a_capture_arrives_without_crop_job_or_crop_write_and_renders_preview() {
    let a = TestApp::with_fake_document().unwrap();
    let session = a
        .items()
        .create_local("K1", "K1", "object", None)
        .unwrap()
        .1
        .session
        .id;
    a.sessions().activate(session).unwrap();
    let path = capture(&a, "K1/a.ARW");
    let raw = canon(&path).unwrap();
    a.fake.add_image(&raw, &["7"], false);
    a.watcher_event(&path).unwrap();
    a.deliver_hook(&json!({"hook":"capture_done","argv":[raw]}).to_string())
        .unwrap();
    assert_eq!(
        jobs(&a, &raw),
        ["hash", "original_preview", "render_preview"]
    );
    a.tick(0).unwrap();
    a.tick(20_000).unwrap();
    no_crop_ops(&a);
    assert!(
        a.fake
            .state
            .lock()
            .unwrap()
            .issued
            .iter()
            .any(|(_, variant, recipe)| variant == "7" && recipe == "PSM Preview")
    );
}

#[test]
fn c1b_import_arrives_without_crop_job_or_crop_write() {
    let a = TestApp::with_fake_document().unwrap();
    let path = a.dir.path().join("x.ARW");
    fs::write(&path, b"raw").unwrap();
    let raw = canon(&path).unwrap();
    a.fake.add_image(&raw, &["7"], false);
    PhotosService {
        arrival: a.app.arrival.as_ref(),
        editing: a.app.editing.as_ref(),
        store: a.store(),
        clock: a.clock.as_ref(),
    }
    .import(std::slice::from_ref(&raw))
    .unwrap();
    assert_eq!(
        jobs(&a, &raw),
        ["hash", "original_preview", "render_preview"]
    );
    a.tick(0).unwrap();
    no_crop_ops(&a);
}

#[test]
fn c2a_missing_new_keys_use_defaults_and_save_them() {
    let mut a = TestApp::new().unwrap();
    let mut data = stored(&a);
    data.as_object_mut().unwrap().remove("default_shape");
    data.as_object_mut().unwrap().remove("master_recipe");
    fs::write(a.app.paths.settings(), serde_json::to_vec(&data).unwrap()).unwrap();
    a.kill_and_restart().unwrap();
    let shown = a.app.settings_get().unwrap();
    assert_eq!(shown["default_shape"], "1:1");
    assert_eq!(shown["master_recipe"], "PSM Master");
    assert!(shown.get("crop_on_arrival").is_none());
    a.settings_set(&json!({"theme":"dark"})).unwrap();
    assert_eq!(stored(&a)["default_shape"], "1:1");
    assert_eq!(stored(&a)["master_recipe"], "PSM Master");
}

#[test]
fn c2b_legacy_shapes_map_and_disappear_after_save() {
    for (legacy, want) in [
        ("Square", "1:1"),
        ("off", "original"),
        ("4x5", "4:5"),
        ("2x3", "2:3"),
        ("16x9", "16:9"),
        ("Unconstrained", "1:1"),
    ] {
        let mut a = TestApp::new().unwrap();
        let mut data = stored(&a);
        data.as_object_mut().unwrap().remove("default_shape");
        data["crop_on_arrival"] = json!(legacy);
        fs::write(a.app.paths.settings(), serde_json::to_vec(&data).unwrap()).unwrap();
        a.kill_and_restart().unwrap();
        assert_eq!(
            a.app.settings_get().unwrap()["default_shape"],
            want,
            "{legacy}"
        );
        a.settings_set(&json!({"theme":"dark"})).unwrap();
        let saved = stored(&a);
        assert_eq!(saved["default_shape"], want, "{legacy}");
        assert!(saved.get("crop_on_arrival").is_none(), "{legacy}");
    }
}

#[test]
fn c2c_settings_validate_shape_and_master_recipe() {
    let mut a = TestApp::new().unwrap();
    a.settings_set(&json!({"default_shape":"4:5"})).unwrap();
    for value in ["Square", ""] {
        assert!(
            matches!(a.settings_set(&json!({"default_shape":value})), Err(Error::InvalidValue(key)) if key == "default_shape")
        );
        assert_eq!(stored(&a)["default_shape"], "4:5");
    }
    assert!(
        matches!(a.settings_set(&json!({"crop_on_arrival":"off"})), Err(Error::InvalidValue(key)) if key == "crop_on_arrival")
    );
    a.settings_set(&json!({"master_recipe":"My Master"}))
        .unwrap();
    assert_eq!(a.app.settings_get().unwrap()["master_recipe"], "My Master");
    assert!(
        matches!(a.settings_set(&json!({"master_recipe":""})), Err(Error::InvalidValue(key)) if key == "master_recipe")
    );
}

#[test]
fn c3a_ensure_repairs_preview_to_2560_crop_ignored() {
    let a = TestApp::with_fake_document().unwrap();
    a.app.startup().unwrap();
    let s = a.fake.state.lock().unwrap();
    assert!(s.ops.contains(&"ensure_recipe:PSM Preview".into()));
    let change = s
        .ops
        .iter()
        .find(|op| op.starts_with("set_recipe_props:PSM Preview:"))
        .unwrap();
    for key in [
        "primaryScalingValue",
        "jpegQuality",
        "exportCropMethod",
        "bits",
        "existingFiles",
        "outputSubFolder",
    ] {
        assert!(change.contains(key), "{key}");
    }
    let mut got = s.recipes["PSM Preview"].clone();
    got.root_folder_type = None;
    assert_eq!(got, RecipeSpec::PREVIEW.props());
    assert_eq!(
        (
            got.output_format.as_str(),
            got.bits,
            got.scaling_method.as_str(),
            got.scaling_unit.as_str(),
            got.primary_scaling_value,
            got.jpeg_quality,
            got.export_crop_method.as_str(),
            got.color_profile.as_str(),
            got.enabled
        ),
        (
            "JPEG",
            8,
            "Long_Edge",
            "pixels",
            2560,
            85,
            "ignore",
            SRGB_PROFILE,
            false
        )
    );
}

#[test]
fn c3b_ensure_creates_uncompressed_master() {
    let a = TestApp::with_fake_document().unwrap();
    a.app.startup().unwrap();
    let s = a.fake.state.lock().unwrap();
    assert!(s.ops.contains(&"ensure_recipe:PSM Master".into()));
    assert!(s.ops.contains(&"make_recipe:PSM Master".into()));
    let mut got = s.recipes["PSM Master"].clone();
    got.root_folder_type = None;
    assert_eq!(got, RecipeSpec::MASTER.props());
    assert_eq!(
        (
            got.output_format.as_str(),
            got.bits,
            got.tiff_compression.as_str(),
            got.scaling_method.as_str(),
            got.scaling_unit.as_str(),
            got.primary_scaling_value,
            got.export_crop_method.as_str(),
            got.color_profile.as_str(),
            got.enabled,
            got.existing_files.as_str(),
            got.output_sub_folder.as_str()
        ),
        (
            "TIFF",
            16,
            MASTER_TIFF_COMPRESSION,
            "Fixed",
            "percent",
            100,
            "ignore",
            SRGB_PROFILE,
            false,
            "overwrite",
            "_session_manager"
        )
    );
}

#[test]
fn c3c_web_remains_and_renamed_master_leaves_old_recipe() {
    let mut a = TestApp::with_fake_document().unwrap();
    a.app.startup().unwrap();
    assert!(ops(&a).contains(&"ensure_recipe:PSM Web".into()));
    a.settings_set(&json!({"master_recipe":"PSM Master 2"}))
        .unwrap();
    a.fake.state.lock().unwrap().ops.clear();
    a.app.startup().unwrap();
    let s = a.fake.state.lock().unwrap();
    assert!(s.ops.contains(&"ensure_recipe:PSM Master 2".into()));
    assert!(s.ops.contains(&"make_recipe:PSM Master 2".into()));
    assert!(s.recipes.contains_key("PSM Master"));
}

#[test]
fn c3d_only_drifted_preview_property_is_written() {
    let a = TestApp::with_fake_document().unwrap();
    let mut props = RecipeSpec::PREVIEW.props();
    props.export_crop_method = "respect".into();
    props.root_folder_type = Some("output location".into());
    a.fake
        .state
        .lock()
        .unwrap()
        .recipes
        .insert("PSM Preview".into(), props);
    a.app.startup().unwrap();
    let s = a.fake.state.lock().unwrap();
    assert_eq!(
        s.ops
            .iter()
            .filter(|op| op.starts_with("set_recipe_props:PSM Preview:"))
            .cloned()
            .collect::<Vec<_>>(),
        ["set_recipe_props:PSM Preview:exportCropMethod"]
    );
    assert_eq!(s.recipes["PSM Preview"].export_crop_method, "ignore");
}

#[test]
fn c3e_jxa_repairs_master_fields_and_reads_them_back() {
    let script = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../integrations/src/editing/capture_one/jxa/ensure_recipe.js"
    );
    let want = serde_json::to_value(RecipeSpec::MASTER.props()).unwrap();
    let harness = r#"const fs=require('fs'),vm=require('vm');const [file,wantText]=process.argv.slice(1);const want=JSON.parse(wantText);const props={...want,exportCropMethod:'respect',tiffCompression:'LZW',rootFolderType:'output location'};const writes=[];const recipe=new Proxy({}, {get(_,key){if(key==='exists')return()=>true;return()=>props[key]},set(_,key,value){props[key]=value;writes.push([key,value]);return true}});const doc={recipes:{byName:()=>recipe}};global.Application=()=>({documents:{byId:()=>doc}});vm.runInThisContext(fs.readFileSync(file,'utf8'));console.log(JSON.stringify({result:JSON.parse(run(['doc','PSM Master',JSON.stringify(want)])),writes}));"#;
    let out = std::process::Command::new("node")
        .arg("-e")
        .arg(harness)
        .arg(script)
        .arg(want.to_string())
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let result: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(
        result["writes"],
        json!([
            ["exportCropMethod", "ignore"],
            ["tiffCompression", MASTER_TIFF_COMPRESSION]
        ])
    );
    assert_eq!(
        result["result"]["changed"],
        json!(["exportCropMethod", "tiffCompression"])
    );
    for key in [
        "exportCropMethod",
        "bits",
        "tiffCompression",
        "rootFolderType",
    ] {
        assert!(result["result"]["props"].get(key).is_some(), "{key}");
    }
}

#[test]
fn c4a_migration_removes_old_jobs_and_health_codes() {
    let mut a = TestApp::with_fake_document().unwrap();
    {
        let conn = Connection::open(a.app.paths.db()).unwrap();
        for status in ["queued", "failed"] {
            conn.execute("INSERT INTO jobs(kind,target,payload,status,attempts,next_attempt_at,error,created_at,updated_at) VALUES ('crop_on_arrival','/tmp/old.ARW','{}',?1,0,0,?2,0,0)", (status, if status == "failed" {Some("provider")} else {None})).unwrap();
        }
        conn.pragma_update(None, "user_version", 2).unwrap();
    }
    a.kill_and_restart().unwrap();
    a.tick(0).unwrap();
    let conn = Connection::open(a.app.paths.db()).unwrap();
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM jobs WHERE kind='crop_on_arrival' OR error='unknown_job_kind'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 0);
    let health = a.app.health_check().unwrap();
    assert!(
        !health
            .items
            .iter()
            .any(|i| i.code == "crop_not_applied" || i.code == "crop_ratio_unknown")
    );
}
