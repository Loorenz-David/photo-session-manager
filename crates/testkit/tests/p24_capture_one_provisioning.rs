use psm_core::{
    Error,
    domain::editing::{CameraStatus, ProvisionInputs},
    services::camera::CameraService,
};
use psm_integrations::editing::capture_one::recipes::RecipeSpec;
use psm_testkit::TestApp;
use serde_json::json;
use std::{fs, time::Duration};

fn item(a: &TestApp, sku: &str) -> i64 {
    a.items()
        .create_local(sku, sku, "object", None)
        .unwrap()
        .1
        .session
        .id
}
fn inputs(a: &TestApp, wait: Duration) -> ProvisionInputs {
    ProvisionInputs {
        style_name: a.app.settings.style_name.clone(),
        wait,
    }
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
fn c1a_fresh_default_document_is_created_alongside_and_recipes_are_idempotent() {
    let a = TestApp::new().unwrap();
    let home = a.dir.path();
    let doc = home.join("Pictures/Session Manager");
    let own = a.fake.state.lock().unwrap().document_id.clone();
    a.fake.state.lock().unwrap().recipes.clear();
    let owner = RecipeSpec::WEB.props();
    a.fake
        .state
        .lock()
        .unwrap()
        .recipes
        .insert("Owner JPEG".into(), owner.clone());
    a.app.health_check().unwrap();
    assert!(doc.join("Session Manager.cosessiondb").is_file());
    assert!(doc.join("Capture").is_dir());
    let s = a.fake.state.lock().unwrap();
    assert!(s.open_documents.contains(&own));
    assert!(
        s.open_documents
            .contains(&doc.to_string_lossy().into_owned())
    );
    assert_eq!(s.recipes["Owner JPEG"], owner);
    for (name, spec) in [
        ("PSM Preview", RecipeSpec::PREVIEW),
        ("PSM Master", RecipeSpec::MASTER),
        ("PSM Web", RecipeSpec::WEB),
    ] {
        let mut got = s.recipes[name].clone();
        got.root_folder_type = None;
        assert_eq!(got, spec.props());
    }
    drop(s);
    assert_eq!(
        ops(&a)
            .iter()
            .filter(|x| x.starts_with("make_document:"))
            .count(),
        1
    );
    assert!(ops(&a).contains(&"make_recipe:PSM Preview".into()));
    assert!(ops(&a).contains(&"make_recipe:PSM Master".into()));
    assert!(ops(&a).contains(&"make_recipe:PSM Web".into()));
    assert_eq!(
        detail(&a, "document"),
        Some((true, "Session Manager.cosessiondb open".into()))
    );
    assert_eq!(detail(&a, "recipes"), Some((true, "Ready".into())));
    assert_eq!(detail(&a, "hooks"), Some((true, "Installed".into())));
    for code in [
        "document_not_open",
        "document_not_session",
        "document_not_front",
        "editor_not_running",
        "recipe_missing",
    ] {
        assert!(detail(&a, code).is_none(), "{code}");
    }
    assert!(!ops(&a).iter().any(|op| op.starts_with("open_document:")));
    a.fake.state.lock().unwrap().ops.clear();
    a.app.health_check().unwrap();
    for prefix in [
        "make_document:",
        "open_document:",
        "make_recipe:",
        "set_recipe_props:",
    ] {
        let n = ops(&a).iter().filter(|op| op.starts_with(prefix)).count();
        assert_eq!(n, 0, "{prefix}");
    }
    let shown = a.app.settings_get().unwrap();
    assert!(shown["document_id"].is_null());
    assert_eq!(
        shown["effective_document_id"],
        doc.to_string_lossy().as_ref()
    );
}

#[test]
fn c1b_drifted_recipe_repairs_only_dependent_fields() {
    let a = TestApp::with_fake_document().unwrap();
    a.app.health_check().unwrap();
    a.fake.state.lock().unwrap().ops.clear();
    let preview = a.fake.state.lock().unwrap().recipes["PSM Preview"].clone();
    {
        let mut s = a.fake.state.lock().unwrap();
        let web = s.recipes.get_mut("PSM Web").unwrap();
        web.enabled = true;
        web.jpeg_quality = 70;
        web.output_sub_folder = "".into();
        web.root_folder_type = Some("image folder".into());
    }
    a.app.health_check().unwrap();
    assert!(
        ops(&a).contains(&"set_recipe_props:PSM Web:enabled,jpegQuality,outputSubFolder".into())
    );
    let s = a.fake.state.lock().unwrap();
    assert_eq!(s.recipes["PSM Preview"], preview);
    assert!(!s.ops.iter().any(|op| op.starts_with("make_document:")
        || op.starts_with("open_document:")
        || op.starts_with("make_recipe:")));
    let mut web = s.recipes["PSM Web"].clone();
    assert_eq!(web.root_folder_type.as_deref(), Some("image folder"));
    web.root_folder_type = None;
    assert_eq!(web, RecipeSpec::WEB.props());
}

#[test]
fn c1c_launch_installed_missing_and_bounded_timeout() {
    let a = TestApp::with_fake_document().unwrap();
    a.fake.state.lock().unwrap().ops.clear();
    {
        let mut s = a.fake.state.lock().unwrap();
        s.running = false;
        s.running_after_polls = Some(2);
    }
    a.app
        .editing
        .ensure(&inputs(&a, Duration::from_secs(2)))
        .unwrap();
    assert_eq!(
        &ops(&a)[..4],
        ["editor_info", "launch", "editor_info", "editor_info"]
    );
    let b = TestApp::with_fake_document().unwrap();
    b.fake.state.lock().unwrap().ops.clear();
    {
        let mut s = b.fake.state.lock().unwrap();
        s.running = false;
        s.launch_fails = Some("Unable to find application named 'Capture One'".into());
    }
    assert!(
        b.app
            .editing
            .ensure(&inputs(&b, Duration::from_secs(2)))
            .is_err()
    );
    assert_eq!(
        detail(&b, "editor_not_installed"),
        Some((false, "Capture One is not installed on this Mac".into()))
    );
    assert_eq!(ops(&b), ["editor_info", "launch"]);
    for code in ["document", "recipes", "hooks"] {
        assert!(detail(&b, code).is_none());
    }
    let c = TestApp::with_fake_document().unwrap();
    c.fake.state.lock().unwrap().ops.clear();
    c.fake.state.lock().unwrap().running = false;
    assert!(
        c.app
            .editing
            .ensure(&inputs(&c, Duration::from_millis(50)))
            .is_err()
    );
    assert!(
        detail(&c, "editor_not_running")
            .unwrap()
            .1
            .starts_with("Capture One did not start")
    );
    assert!(
        !ops(&c)
            .iter()
            .any(|op| op.starts_with("make_document:") || op.starts_with("ensure_recipe:"))
    );
}

#[test]
fn c1d_automation_and_edition_fail_before_document() {
    for (denied, code, exact) in [
        (
            true,
            "editor_automation_denied",
            "macOS has not allowed Session Manager to control Capture One. Allow it under System Settings › Privacy & Security › Automation",
        ),
        (
            false,
            "editor_not_pro",
            "Capture One Pro is required; the installed edition cannot be scripted",
        ),
    ] {
        let a = TestApp::with_fake_document().unwrap();
        {
            let mut s = a.fake.state.lock().unwrap();
            s.automation_denied = denied;
            s.pro = denied;
        }
        a.app.health_check().unwrap();
        assert_eq!(detail(&a, code), Some((false, exact.into())));
        assert!(!ops(&a).iter().any(|op| op.starts_with("make_document:")
            || op.starts_with("ensure_recipe:")
            || op.starts_with("set_hook:")));
        for code in ["document", "recipes", "hooks"] {
            assert!(detail(&a, code).is_none());
        }
    }
}

#[test]
fn c1e_non_session_folder_and_closed_document() {
    let a = TestApp::new().unwrap();
    let folder = a.dir.path().join("Pictures/Session Manager");
    fs::create_dir_all(&folder).unwrap();
    fs::write(folder.join("notes.txt"), "notes").unwrap();
    a.app.health_check().unwrap();
    assert_eq!(
        detail(&a, "document_not_session"),
        Some((
            false,
            format!(
                "{} exists but is not a Capture One session",
                folder.display()
            )
        ))
    );
    assert!(
        !ops(&a)
            .iter()
            .any(|op| op.starts_with("make_document:") || op.starts_with("open_document:"))
    );
    assert_eq!(fs::read_dir(&folder).unwrap().count(), 1);
    let b = TestApp::with_fake_document().unwrap();
    {
        let mut s = b.fake.state.lock().unwrap();
        s.open_documents.clear();
        s.fail_open_document = Some("busy".into());
    }
    b.app.health_check().unwrap();
    assert!(
        detail(&b, "document_not_open")
            .unwrap()
            .1
            .starts_with("Capture One could not open")
    );
    assert!(ops(&b).iter().any(|op| op.starts_with("open_document:")));
    assert!(!ops(&b).iter().any(|op| op.starts_with("ensure_recipe:")));
    let c = TestApp::with_fake_document().unwrap();
    c.fake.state.lock().unwrap().open_documents.clear();
    c.app.health_check().unwrap();
    assert_eq!(
        ops(&c)
            .iter()
            .filter(|op| op.starts_with("open_document:"))
            .count(),
        1
    );
    assert_eq!(
        detail(&c, "document"),
        Some((true, "Capture One Session.cosessiondb open".into()))
    );
    assert_eq!(detail(&c, "recipes"), Some((true, "Ready".into())));
    assert_eq!(detail(&c, "hooks"), Some((true, "Installed".into())));
}

#[test]
fn c1f_style_setting_applies_reports_missing_and_preserves_when_empty() {
    let mut a = TestApp::with_fake_document().unwrap();
    a.settings_set(&json!({"style_name":"PSM House"})).unwrap();
    a.app.health_check().unwrap();
    a.app.health_check().unwrap();
    assert_eq!(
        a.fake.state.lock().unwrap().next_capture_styles,
        ["PSM House"]
    );
    assert_eq!(
        ops(&a)
            .iter()
            .filter(|op| op.starts_with("set_next_capture_styles:"))
            .count(),
        1
    );
    a.settings_set(&json!({"style_name":"Moody"})).unwrap();
    a.app.health_check().unwrap();
    assert_eq!(
        detail(&a, "style_not_found"),
        Some((false, "Style 'Moody' not found in Capture One".into()))
    );
    assert_eq!(detail(&a, "hooks"), Some((true, "Installed".into())));
    a.settings_set(&json!({"style_name":""})).unwrap();
    a.fake.state.lock().unwrap().ops.clear();
    a.app.health_check().unwrap();
    assert!(detail(&a, "style_not_found").is_none());
    assert!(
        !ops(&a)
            .iter()
            .any(|op| op.starts_with("ensure_capture_style:"))
    );
    assert_eq!(
        a.fake.state.lock().unwrap().next_capture_styles,
        ["PSM House"]
    );
}

#[test]
fn c2a_activation_fronts_own_document_before_capture_folder() {
    for launch in [false, true] {
        let a = TestApp::with_fake_document().unwrap();
        let session = item(&a, "K1");
        let doc = a.fake.state.lock().unwrap().document_id.clone();
        {
            let mut s = a.fake.state.lock().unwrap();
            s.open_documents.push("/elsewhere/Instagram".into());
            s.front_document = Some("/elsewhere/Instagram".into());
            if launch {
                s.running = false;
                s.running_after_polls = Some(1);
            }
        }
        a.sessions().activate(session).unwrap();
        let s = a.fake.state.lock().unwrap();
        assert_eq!(s.front_document.as_deref(), Some(doc.as_str()));
        assert!(s.open_documents.contains(&"/elsewhere/Instagram".into()));
        assert_eq!(a.store().active_session_id().unwrap(), Some(session));
        assert!(s.captures.ends_with("/Capture/K1"));
        let front = s
            .ops
            .iter()
            .position(|op| op == &format!("front:{doc}"))
            .unwrap();
        let capture = s
            .ops
            .iter()
            .position(|op| op.starts_with("set_captures:"))
            .unwrap();
        assert!(front < capture);
        if launch {
            assert!(s.ops.iter().position(|op| op == "launch").unwrap() < front);
        }
    }
}

#[test]
fn c2b_unverified_front_blocks_capture_then_retry_recovers() {
    let a = TestApp::with_fake_document().unwrap();
    let session = item(&a, "K1");
    {
        let mut s = a.fake.state.lock().unwrap();
        s.fail_front = true;
        s.front_document = Some("/elsewhere/Instagram".into());
    }
    assert!(matches!(
        a.sessions().activate(session),
        Err(Error::DocumentNotFront)
    ));
    assert_eq!(
        detail(&a, "document_not_front"),
        Some((
            false,
            "Capture One is showing another session; captures would land there".into()
        ))
    );
    assert!(!ops(&a).iter().any(|op| op.starts_with("set_captures:")));
    assert_eq!(a.store().active_session_id().unwrap(), None);
    let camera = CameraService::new(a.app.camera.as_ref(), a.app.health.as_ref());
    assert_eq!(camera.status().unwrap(), CameraStatus::Disconnected);
    assert!(matches!(camera.capture(), Err(Error::DocumentNotFront)));
    assert!(!ops(&a).contains(&"capture".into()));
    a.fake.state.lock().unwrap().fail_front = false;
    a.sessions().activate(session).unwrap();
    assert!(detail(&a, "document_not_front").is_none());
    assert_eq!(a.store().active_session_id().unwrap(), Some(session));
}

#[test]
fn c2c_health_reasserts_front_only_while_active() {
    let a = TestApp::with_fake_document().unwrap();
    let session = item(&a, "K1");
    a.sessions().activate(session).unwrap();
    let doc = a.fake.state.lock().unwrap().document_id.clone();
    a.fake.state.lock().unwrap().front_document = Some("/elsewhere/Instagram".into());
    let before = ops(&a).iter().filter(|op| op.starts_with("front:")).count();
    a.app.health_check().unwrap();
    assert_eq!(
        ops(&a).iter().filter(|op| op.starts_with("front:")).count(),
        before + 1
    );
    assert_eq!(
        a.fake.state.lock().unwrap().front_document.as_deref(),
        Some(doc.as_str())
    );
    a.sessions().deactivate().unwrap();
    a.fake.state.lock().unwrap().front_document = Some("/elsewhere/Instagram".into());
    let before = ops(&a).iter().filter(|op| op.starts_with("front:")).count();
    a.app.health_check().unwrap();
    assert_eq!(
        ops(&a).iter().filter(|op| op.starts_with("front:")).count(),
        before
    );
    assert_eq!(
        a.fake.state.lock().unwrap().front_document.as_deref(),
        Some("/elsewhere/Instagram")
    );
}

#[test]
fn c3a_document_setting_defaults_clears_validates_and_honours_existing() {
    let mut a = TestApp::new().unwrap();
    let default = a.dir.path().join("Pictures/Session Manager");
    assert!(a.app.settings_get().unwrap()["document_id"].is_null());
    assert_eq!(
        a.app.settings_get().unwrap()["effective_document_id"],
        default.to_string_lossy().as_ref()
    );
    let shown = a
        .settings_set(&json!({"document_id":"/Users/me/Pictures/Other"}))
        .unwrap();
    assert_eq!(shown["effective_document_id"], "/Users/me/Pictures/Other");
    assert_eq!(
        a.app.settings_get().unwrap()["document_id"],
        "/Users/me/Pictures/Other"
    );
    let shown = a.settings_set(&json!({"document_id":""})).unwrap();
    assert!(shown["document_id"].is_null());
    assert_eq!(
        shown["effective_document_id"],
        default.to_string_lossy().as_ref()
    );
    assert!(matches!(
        a.settings_set(&json!({"document_id":"relative/x"})),
        Err(Error::RelativePath)
    ));
    assert!(a.app.settings_get().unwrap()["document_id"].is_null());
    let mut stored: serde_json::Value =
        serde_json::from_slice(&fs::read(a.app.paths.settings()).unwrap()).unwrap();
    stored["document_id"] = json!("/Users/davidloorenz/Pictures/session test 1");
    fs::write(a.app.paths.settings(), serde_json::to_vec(&stored).unwrap()).unwrap();
    let before = fs::read(a.app.paths.settings()).unwrap();
    a.kill_and_restart().unwrap();
    let shown = a.app.settings_get().unwrap();
    assert_eq!(shown["document_id"], stored["document_id"]);
    assert_eq!(shown["effective_document_id"], stored["document_id"]);
    assert_eq!(fs::read(a.app.paths.settings()).unwrap(), before);
}

const JXA_HARNESS: &str = r#"
const fs=require('fs'), vm=require('vm');
const [file,mode,wantText]=process.argv.slice(1);
const want=wantText?JSON.parse(wantText):null;
const calls=[],writes=[];
global.Path=v=>({toString:()=>v});
let front='/elsewhere/Instagram';
let props=mode==='recipe_new'?{enabled:false,outputFormat:'TIFF',bits:16,tiffCompression:'Uncompressed',exportCropMethod:'respect',colorProfile:'Adobe RGB (1998)',scalingMethod:'Fixed',scalingUnit:'percent',primaryScalingValue:100,jpegQuality:100,existingFiles:'add suffix',outputSubFolder:'',rootFolderType:'output location'}:
  mode==='recipe_drift'?{...want,enabled:true,jpegQuality:70,rootFolderType:'image folder',sharpening:'standard'}:{...want,rootFolderType:'output location'};
let exists=mode!=='recipe_new';
const recipe=new Proxy({}, {
  get(_target,key){ if(key==='exists')return()=>exists; if(key==='name')return()=> 'PSM Web'; if(typeof key==='string')return()=>props[key]; },
  set(_target,key,value){props[key]=value;writes.push([key,value]);return true;}
});
let applied=[];
const next=new Proxy({}, {get(_target,key){if(key==='applyStyles')return()=>applied;},set(_target,key,value){if(key==='applyStyles'){applied=value;writes.push(value);return true;}return false;}});
const doc={id:()=>'/tmp/x/Session Manager',recipes:{byName:()=>recipe,name:()=>[],[Symbol.iterator]:function*(){}},nextCaptureSettings:next};
const app={documents:{byId:()=>doc},availableStyles:()=>['PSM House','Clean'],
  make(x){calls.push({op:'make',value:{new:x.new,at:!!x.at,withProperties:{...x.withProperties,path:x.withProperties?.path?.toString()}}});exists=true;return mode==='make_document'?doc:recipe;},
  open(x){calls.push({op:'open',value:x.toString()});},
  currentDocument(){return {id:()=>front,name:()=>front.split('/').at(-1)};}};
Object.defineProperty(app,'currentDocument',{get(){return ()=>({id:()=>front,name:()=>front.split('/').at(-1)});},set(value){calls.push({op:'front',value:value.id()});front=value.id();}});
global.Application=()=>app;
vm.runInThisContext(fs.readFileSync(file,'utf8'));
let args;
if(mode.startsWith('recipe'))args=['/tmp/x/Session Manager','PSM Web',JSON.stringify(want)];
else if(mode==='make_document')args=['/tmp/x','Session Manager'];
else if(mode==='open_document')args=['/tmp/x/Session Manager/Session Manager.cosessiondb','/tmp/x/scratch'];
else if(mode==='front_document')args=['/tmp/x/Session Manager'];
else args=['/tmp/x/Session Manager',mode==='style_missing'?'Moody':'PSM House'];
const first=JSON.parse(run(args));
if(mode==='style'){const second=JSON.parse(run(args));const third=JSON.parse(run(['/tmp/x/Session Manager','Moody']));console.log(JSON.stringify({first,second,third,calls,writes,props}));}
else console.log(JSON.stringify({first,calls,writes,props}));
"#;

fn script_result(name: &str, mode: &str, want: Option<&serde_json::Value>) -> serde_json::Value {
    let dir = tempfile::tempdir().unwrap();
    let paths = psm_infra::paths::AppPaths::for_dir(dir.path());
    let _ = psm_integrations::editing::capture_one::jxa::JxaScripting::new(&paths).unwrap();
    let file = paths.hooks().join("jxa").join(format!("{name}.js"));
    let output = std::process::Command::new("node")
        .arg("-e")
        .arg(JXA_HARNESS)
        .arg(file)
        .arg(mode)
        .arg(want.map_or(String::new(), serde_json::Value::to_string))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn c4a_recipe_script_creates_repairs_only_drift_and_skips_exact() {
    let want = serde_json::to_value(RecipeSpec::WEB.props()).unwrap();
    let created = script_result("ensure_recipe", "recipe_new", Some(&want));
    assert_eq!(created["first"]["created"], true);
    assert_eq!(
        created["first"]["changed"],
        json!([
            "outputFormat",
            "colorProfile",
            "scalingMethod",
            "scalingUnit",
            "primaryScalingValue",
            "jpegQuality",
            "existingFiles",
            "outputSubFolder",
            "bits"
        ])
    );
    assert_eq!(created["calls"].as_array().unwrap().len(), 1);
    assert_eq!(created["calls"][0]["value"]["new"], "recipe");
    assert_eq!(
        created["calls"][0]["value"]["withProperties"]["name"],
        "PSM Web"
    );
    let drift = script_result("ensure_recipe", "recipe_drift", Some(&want));
    assert_eq!(drift["first"]["created"], false);
    assert_eq!(drift["first"]["changed"], json!(["enabled", "jpegQuality"]));
    assert_eq!(
        drift["writes"],
        json!([["enabled", false], ["jpegQuality", 90]])
    );
    assert_eq!(drift["props"]["rootFolderType"], "image folder");
    assert_eq!(drift["props"]["sharpening"], "standard");
    let exact = script_result("ensure_recipe", "recipe_exact", Some(&want));
    assert_eq!(exact["first"]["changed"], json!([]));
    assert_eq!(exact["writes"], json!([]));
}

#[test]
fn c4b_document_front_and_style_scripts_use_grounded_dictionary_forms() {
    let made = script_result("make_document", "make_document", None);
    assert_eq!(made["first"], "/tmp/x/Session Manager");
    assert_eq!(made["calls"][0]["value"]["new"], "document");
    assert_eq!(
        made["calls"][0]["value"]["withProperties"]["name"],
        "Session Manager"
    );
    assert_eq!(
        made["calls"][0]["value"]["withProperties"]["path"],
        "/tmp/x"
    );
    assert!(
        made["calls"][0]["value"]["withProperties"]
            .get("kind")
            .is_none()
    );
    let opened = script_result("open_document", "open_document", None);
    assert_eq!(
        opened["calls"],
        json!([
            {
                "op":"make",
                "value":{
                    "new":"document",
                    "at":false,
                    "withProperties":{"name":"PSM Open Window","path":"/tmp/x/scratch"}
                }
            },
            {"op":"open","value":"/tmp/x/Session Manager/Session Manager.cosessiondb"}
        ])
    );
    let front = script_result("front_document", "front_document", None);
    assert_eq!(front["first"], "/tmp/x/Session Manager");
    assert_eq!(
        front["calls"],
        json!([{"op":"front","value":"/tmp/x/Session Manager"}])
    );
    let style = script_result("ensure_capture_style", "style", None);
    assert_eq!(
        style["first"],
        json!({"listed":true,"applied":["PSM House"]})
    );
    assert_eq!(style["second"], style["first"]);
    assert_eq!(
        style["third"],
        json!({"listed":false,"applied":["PSM House"]})
    );
    assert_eq!(style["writes"], json!([["PSM House"]]));
}
