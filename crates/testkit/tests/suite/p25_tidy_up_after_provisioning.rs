use psm_core::{
    Error,
    domain::{
        camera::{CameraStatus, Link},
        photo::{Classification, Integrity, NewPhoto, PhotoSource},
    },
    mechanisms::hash::sha256_file,
    ports::clock::Clock,
    services::{camera::CameraService, integrity::IntegrityCheck},
};
use psm_testkit::TestApp;
use serde_json::{Value, json};
use std::{fs, path::Path, process::Command};

fn item(a: &TestApp) -> i64 {
    a.items()
        .create_local("K1", "K1", "object", None)
        .unwrap()
        .1
        .session
        .id
}
fn ops(a: &TestApp) -> Vec<String> {
    a.fake.state.lock().unwrap().ops.clone()
}
fn detail(a: &TestApp, code: &str) -> Option<(bool, String)> {
    a.app
        .health
        .report("", true, true)
        .items
        .into_iter()
        .find(|i| i.code == code)
        .map(|i| (i.ok, i.detail))
}

#[test]
fn c1a_open_document_closes_only_its_throwaway_on_silent_refusal() {
    let dir = tempfile::tempdir().unwrap();
    let paths = psm_infra::paths::AppPaths::for_dir(dir.path());
    psm_integrations::editing::capture_one::jxa::JxaScripting::new(&paths).unwrap();
    let file = paths.hooks().join("jxa/open_document.js");
    let harness = r#"
const fs=require('fs'),vm=require('vm');
const [file,mode]=process.argv.slice(1), calls=[];
let front=null;
global.Path=x=>({toString:()=>x});
const made={id:()=>'/tmp/x/scratch/PSM Open Window',name:()=> 'PSM Open Window'};
const target={id:()=>'/tmp/x/Session Manager',name:()=> 'Session Manager.cosessiondb'};
const app={
 make(x){calls.push({op:'make',new:x.new,name:x.withProperties.name,path:x.withProperties.path.toString()});front=made;},
 open(x){calls.push({op:'open',path:x.toString()});front=mode==='replaced'?target:mode==='null'?null:made;},
 currentDocument(){return front;}, close(x){calls.push({op:'close',value:x.id()});front=null;}
};
global.Application=()=>app;
vm.runInThisContext(fs.readFileSync(file,'utf8'));
let result,error;
try{result=JSON.parse(run(['/tmp/x/Session Manager/Session Manager.cosessiondb','/tmp/x/scratch']));}
catch(e){error=String(e);}
console.log(JSON.stringify({result,error,calls}));
"#;
    let expected = json!([
        {"op":"make","new":"document","name":"PSM Open Window","path":"/tmp/x/scratch"},
        {"op":"open","path":"/tmp/x/Session Manager/Session Manager.cosessiondb"}
    ]);
    for mode in ["replaced", "null", "refused"] {
        let output = Command::new("node")
            .arg("-e")
            .arg(harness)
            .arg(&file)
            .arg(mode)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let got: Value = serde_json::from_slice(&output.stdout).unwrap();
        if mode == "refused" {
            assert!(
                got["error"]
                    .as_str()
                    .unwrap()
                    .contains("/tmp/x/Session Manager/Session Manager.cosessiondb")
            );
            let mut calls = expected.as_array().unwrap().clone();
            calls.push(json!({"op":"close","value":"/tmp/x/scratch/PSM Open Window"}));
            assert_eq!(got["calls"], json!(calls));
        } else {
            assert_eq!(got["result"], Value::Null);
            assert_eq!(got["calls"], expected);
        }
    }
}

#[test]
fn c2a_failed_activation_lapses_when_no_session_is_active() {
    let a = TestApp::with_fake_document().unwrap();
    let s = item(&a);
    {
        let mut fake = a.fake.state.lock().unwrap();
        fake.fail_front = true;
        fake.front_document = Some("/elsewhere/Instagram".into());
    }
    assert!(matches!(
        a.sessions().activate(s),
        Err(Error::DocumentNotFront)
    ));
    assert!(matches!(detail(&a, "document_not_front"), Some((false, _))));
    {
        let mut fake = a.fake.state.lock().unwrap();
        fake.fail_front = false;
        fake.ops.clear();
    }
    a.app.health_check().unwrap();
    assert_eq!(detail(&a, "document_not_front"), None);
    assert_eq!(a.store().active_session_id().unwrap(), None);
    assert!(!ops(&a).iter().any(|op| op.starts_with("front:")));
    assert!(
        ops(&a)
            .iter()
            .any(|op| op.ends_with("/_Unassigned") && op.starts_with("set_captures:"))
    );
    let camera = CameraService::new(a.app.camera.as_ref(), a.app.health.as_ref());
    assert_eq!(
        camera.status().unwrap(),
        CameraStatus::Connected {
            name: "Sony ZV-E10".into(),
            link: Link::Cable
        }
    );
    camera.capture().unwrap();
    assert!(ops(&a).contains(&"capture".into()));
}

#[test]
fn c2b_deactivation_clears_front_warning() {
    let a = TestApp::with_fake_document().unwrap();
    let s = item(&a);
    a.sessions().activate(s).unwrap();
    {
        let mut fake = a.fake.state.lock().unwrap();
        fake.fail_front = true;
        fake.front_document = Some("/elsewhere/Instagram".into());
    }
    a.app.health_check().unwrap();
    assert_eq!(
        detail(&a, "document_not_front"),
        Some((
            false,
            "Capture One is showing another session; captures would land there".into()
        ))
    );
    a.sessions().deactivate().unwrap();
    assert_eq!(detail(&a, "document_not_front"), None);
    assert_eq!(a.store().active_session_id().unwrap(), None);
    assert!(
        ops(&a)
            .iter()
            .rev()
            .find(|op| op.starts_with("set_captures:"))
            .unwrap()
            .ends_with("/_Unassigned")
    );
    {
        let mut fake = a.fake.state.lock().unwrap();
        fake.front_document = Some("/elsewhere/Instagram".into());
        fake.ops.clear();
    }
    a.app.health_check().unwrap();
    assert_eq!(detail(&a, "document_not_front"), None);
    assert!(!ops(&a).iter().any(|op| op.starts_with("front:")));
}

#[test]
fn c3a_default_document_path_stays_constant_across_settings_changes() {
    let mut a = TestApp::new().unwrap();
    let default = a
        .dir
        .path()
        .join("Pictures/Session Manager")
        .to_string_lossy()
        .into_owned();
    let first = a.app.settings_get().unwrap();
    assert_eq!(first["default_document_id"], default);
    assert_eq!(first["effective_document_id"], default);
    let explicit = a
        .settings_set(&json!({"document_id":"/Users/me/Pictures/Other"}))
        .unwrap();
    assert_eq!(explicit["default_document_id"], default);
    assert_eq!(
        explicit["effective_document_id"],
        "/Users/me/Pictures/Other"
    );
    assert_eq!(explicit["document_id"], "/Users/me/Pictures/Other");
    assert_eq!(
        a.app.settings_get().unwrap()["default_document_id"],
        default
    );
    let cleared = a.settings_set(&json!({"document_id":""})).unwrap();
    assert_eq!(cleared["default_document_id"], default);
    assert_eq!(cleared["effective_document_id"], default);
}

#[test]
fn c4a_fresh_settings_leave_next_capture_style_alone() {
    let a = TestApp::new().unwrap();
    assert_eq!(a.app.settings_get().unwrap()["style_name"], "");
    let stored: Value = serde_json::from_slice(&fs::read(a.app.paths.settings()).unwrap()).unwrap();
    assert_eq!(stored["style_name"], "");
    let report = a.app.health_check().unwrap();
    assert!(!report.items.iter().any(|i| i.code == "style_not_found"));
    assert_eq!(detail(&a, "hooks"), Some((true, "Installed".into())));
    assert!(
        !ops(&a)
            .iter()
            .any(|op| op.starts_with("ensure_capture_style:"))
    );
    assert!(a.fake.state.lock().unwrap().next_capture_styles.is_empty());
}

#[test]
fn c4b_existing_style_name_is_preserved_and_applied() {
    let mut a = TestApp::with_fake_document().unwrap();
    let path = a.app.paths.settings();
    let mut settings: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    settings["style_name"] = json!("PSM House");
    fs::write(&path, serde_json::to_vec(&settings).unwrap()).unwrap();
    let before = fs::read(&path).unwrap();
    a.kill_and_restart().unwrap();
    assert_eq!(a.app.settings_get().unwrap()["style_name"], "PSM House");
    a.app.health_check().unwrap();
    assert_eq!(
        a.fake.state.lock().unwrap().next_capture_styles,
        ["PSM House"]
    );
    assert_eq!(
        ops(&a)
            .iter()
            .filter(|op| *op == "set_next_capture_styles:PSM House")
            .count(),
        1
    );
    assert!(detail(&a, "style_not_found").is_none());
    assert_eq!(fs::read(&path).unwrap(), before);
    let mut b = TestApp::new().unwrap();
    let path = b.app.paths.settings();
    let mut settings: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    settings.as_object_mut().unwrap().remove("style_name");
    fs::write(&path, serde_json::to_vec(&settings).unwrap()).unwrap();
    b.kill_and_restart().unwrap();
    assert_eq!(b.app.settings_get().unwrap()["style_name"], "");
}

#[test]
fn c5a_first_verification_fills_missing_hash_then_detects_change() {
    let a = TestApp::new().unwrap();
    let session = item(&a);
    let raw = a.dir.path().join("fresh.ARW");
    fs::write(&raw, b"original").unwrap();
    let p = add_unhashed(&a, session, &raw);
    let check = IntegrityCheck::new(a.store());
    let hash = sha256_file(&raw).unwrap();
    assert_eq!(check.verify(p).unwrap(), Integrity::Ok);
    assert_eq!(a.store().photo(p).unwrap().raw_hash, Some(hash.clone()));
    assert_eq!(a.store().photo(p).unwrap().integrity, Integrity::Ok);
    fs::write(&raw, b"originalchanged").unwrap();
    assert_eq!(check.verify(p).unwrap(), Integrity::RawChanged);
    assert_eq!(a.store().photo(p).unwrap().raw_hash, Some(hash));
    assert_eq!(a.store().photo(p).unwrap().integrity, Integrity::RawChanged);
    let missing = a.dir.path().join("missing.ARW");
    fs::write(&missing, b"raw").unwrap();
    let id = add_unhashed(&a, session, &missing);
    fs::remove_file(&missing).unwrap();
    assert_eq!(check.verify(id).unwrap(), Integrity::RawMissing);
    assert_eq!(a.store().photo(id).unwrap().raw_hash, None);
    assert_eq!(
        a.store().photo(id).unwrap().integrity,
        Integrity::RawMissing
    );
}

fn add_unhashed(a: &TestApp, session_id: i64, raw: &Path) -> i64 {
    a.store()
        .photo_insert_or_get(NewPhoto {
            session_id,
            raw_path: raw.to_string_lossy().into_owned(),
            raw_hash: None,
            captured_at: Some(a.clock.now_ms()),
            source: PhotoSource::Capture,
            created_at: a.clock.now_ms(),
        })
        .unwrap()
        .id
}

#[test]
fn c5b_publish_of_recent_unhashed_photo_passes_prepare() {
    let a = TestApp::with_fake_shop().unwrap();
    let s = item(&a);
    let raw = a.dir.path().join("Capture/raws/recent.ARW");
    fs::create_dir_all(raw.parent().unwrap()).unwrap();
    fs::write(&raw, b"recent raw").unwrap();
    let id = add_unhashed(&a, s, &raw);
    let stored_path = a.store().photo(id).unwrap().raw_path;
    a.fake.add_image(&stored_path, &[&format!("v{id}")], false);
    a.review().classify(id, Classification::Main).unwrap();
    a.store()
        .set_missing_preview_paths(&stored_path, "preview.jpg")
        .unwrap();
    a.app
        .publish
        .as_ref()
        .unwrap()
        .confirm(&[s], "yes")
        .unwrap();
    a.tick(1100).unwrap();
    a.complete_all_renders().unwrap();
    a.tick(1100).unwrap();
    let db = rusqlite::Connection::open(a.app.paths.db()).unwrap();
    let (failed_step, error): (Option<String>, Option<String>) = db.query_row("SELECT failed_step,error FROM publish_items WHERE session_id=?1 ORDER BY id DESC LIMIT 1", [s], |r| Ok((r.get(0)?, r.get(1)?))).unwrap();
    assert_eq!(failed_step, None);
    assert_eq!(error, None);
    assert_eq!(
        a.store().photo(id).unwrap().raw_hash,
        Some(sha256_file(&raw).unwrap())
    );
    assert_eq!(a.store().photo(id).unwrap().integrity, Integrity::Ok);
    assert!(
        a.commerce
            .as_ref()
            .unwrap()
            .state
            .lock()
            .unwrap()
            .mutations
            .iter()
            .any(|m| m.op == "staged_uploads_create")
    );
}

#[test]
fn c5c_export_of_recent_unhashed_photo_writes_image() {
    let a = TestApp::with_fake_document().unwrap();
    let s = item(&a);
    let raw = a.dir.path().join("Capture/raws/recent.ARW");
    fs::create_dir_all(raw.parent().unwrap()).unwrap();
    fs::write(&raw, b"recent raw").unwrap();
    let id = add_unhashed(&a, s, &raw);
    let stored_path = a.store().photo(id).unwrap().raw_path;
    a.fake.add_image(&stored_path, &[&format!("v{id}")], false);
    a.review().classify(id, Classification::Main).unwrap();
    a.store()
        .set_missing_preview_paths(&stored_path, "preview.jpg")
        .unwrap();
    a.app.export.run(s).unwrap();
    a.complete_all_renders().unwrap();
    a.tick(0).unwrap(); // P29: the `render_output` job writes the exported file
    assert!(a.dir.path().join("Export/K1/K1_01_main.jpg").exists());
    assert_eq!(
        a.store().photo(id).unwrap().raw_hash,
        Some(sha256_file(&raw).unwrap())
    );
    assert_eq!(a.store().photo(id).unwrap().integrity, Integrity::Ok);
}
