//! LV2·U4 — the phone server's HTTP routes (`master_plan_live.md` §5 PH2–PH4): rows LV2·C17–C23,
//! HTTP requests to `TestPhone` (the real server on 127.0.0.1 over the TestApp's handles). Fixture
//! T3 (A final 1 shoot 2 with an example, 1:1 zoom 20; B final 2 shoot 1, split; C final 3 shoot 3)
//! active with a photo in B unless stated.
use psm_core::{
    domain::{
        camera::{CaptureId, ExposureInitiator, ExposureObserved},
        composition::Shape,
        photo::Classification as C,
        template::Presets,
    },
    ports::{camera::CaptureSink, clock::Clock},
    services::{current_shot::Current, templates::TemplatesService},
};
use psm_infra::imaging::ImageOpsImpl;
use psm_integrations::phone::pairing::PhonePairings;
use psm_testkit::{
    TestApp,
    phone::{APP_JS, TEST_MAC_NAME, TestPhone},
};
use serde_json::{Value, json};
use std::{
    thread,
    time::{Duration, Instant},
};

const READY: &str = r#"{"timestamp":"2026-11-06T10:00:00.000Z","source":"sdk","event":"connection.ready","transport":"usb","model":"ILCE-7RM5","store_destination":"verified","store_value":"0x0003"}
"#;

struct Resp {
    status: u16,
    content_type: String,
    body: Vec<u8>,
}
impl Resp {
    fn json(&self) -> Value {
        serde_json::from_slice(&self.body)
            .unwrap_or_else(|_| panic!("not JSON: {}", String::from_utf8_lossy(&self.body)))
    }
    fn code(&self) -> String {
        self.json()["error"]["code"]
            .as_str()
            .unwrap_or_default()
            .to_owned()
    }
}
fn send(method: &str, url: &str, bearer: Option<&str>, body: Option<Value>) -> Resp {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(async {
        let client = reqwest::Client::new();
        let mut request = match method {
            "POST" => client.post(url),
            _ => client.get(url),
        };
        if let Some(token) = bearer {
            request = request.bearer_auth(token);
        }
        if let Some(body) = body {
            request = request.json(&body);
        }
        let response = request.send().await.unwrap();
        let status = response.status().as_u16();
        let content_type = response
            .headers()
            .get("content-type")
            .map(|v| v.to_str().unwrap().to_owned())
            .unwrap_or_default();
        let body = response.bytes().await.unwrap().to_vec();
        Resp {
            status,
            content_type,
            body,
        }
    })
}

struct Fx {
    a: TestApp,
    phone: TestPhone,
    token: String,
    s: i64,
    /// A, B, C
    ids: [i64; 3],
    pb: i64,
}
impl Fx {
    fn get(&self, path: &str) -> Resp {
        send("GET", &self.phone.url(path), Some(&self.token), None)
    }
    fn post(&self, path: &str, body: Value) -> Resp {
        send("POST", &self.phone.url(path), Some(&self.token), Some(body))
    }
    fn stdin(&self) -> Vec<String> {
        self.a
            .fake_sidecar_recording()
            .stdin
            .into_iter()
            .map(|(_, s)| s)
            .collect()
    }
    fn settle_stdin(&self, pred: impl Fn(&[String]) -> bool) {
        let end = Instant::now() + Duration::from_secs(5);
        while Instant::now() < end {
            if pred(&self.stdin()) {
                return;
            }
            thread::sleep(Duration::from_millis(10));
        }
        panic!("stdin: {:?}", self.stdin());
    }
    fn place(&self, id: i64) -> (C, Option<i64>, Option<i64>) {
        let p = self.a.store().photo(id).unwrap();
        let slot = self
            .a
            .store()
            .photo_place_row(id)
            .unwrap()
            .and_then(|r| r.slot);
        (p.classification, p.position, slot)
    }
    fn current(&self) -> Option<Current> {
        self.a
            .app
            .camera_runtime
            .current_shots()
            .current(self.a.store(), self.s, self.a.clock.now_ms())
            .unwrap()
    }
    fn other_session(&self) -> i64 {
        self.a
            .items()
            .create_local("LV2-OTHER", "Other", "Chair", None)
            .unwrap()
            .1
            .session
            .id
    }
}

fn templates(a: &TestApp) -> TemplatesService<'_> {
    TemplatesService::new(
        a.store(),
        a.clock.as_ref(),
        &ImageOpsImpl,
        a.app.paths.root.join("templates"),
    )
}
fn example(a: &TestApp) -> std::path::PathBuf {
    let path = a.app.paths.root.join("example-a.jpg");
    image::RgbImage::from_pixel(300, 200, image::Rgb([120, 90, 50]))
        .save(&path)
        .unwrap();
    path
}
fn arrive(a: &TestApp, s: i64, n: u32) -> i64 {
    let path = a.arrival_photo(s, &format!("DSC{n:05}.ARW")).unwrap();
    a.store()
        .photos_in_session(s)
        .unwrap()
        .into_iter()
        .find(|p| p.raw_path == path)
        .unwrap()
        .id
}
/// T3 active with a photo in B; `script` is the fake camera helper's (`None`: no camera script).
fn fixture(script: Option<&str>) -> Fx {
    let a = match script {
        Some(script) => TestApp::with_camera_script(script).unwrap(),
        None => TestApp::with_camera().unwrap(),
    };
    if script.is_some_and(|s| !s.is_empty()) {
        let end = Instant::now() + Duration::from_secs(5);
        while !a
            .camera_status()
            .is_ok_and(|s| matches!(s, psm_core::domain::camera::CameraStatus::Connected { .. }))
        {
            assert!(Instant::now() < end, "camera did not connect");
            a.tick(0).unwrap();
            thread::sleep(Duration::from_millis(10));
        }
    }
    let t = templates(&a).create("Chair").unwrap();
    let ex = example(&a);
    let t = templates(&a).container_add(t.id, Some(&ex)).unwrap();
    let t = templates(&a).container_add(t.id, None).unwrap();
    let t = templates(&a).container_add(t.id, None).unwrap();
    let ids = [t.containers[0].id, t.containers[1].id, t.containers[2].id];
    templates(&a)
        .shooting_order_set(t.id, &[ids[1], ids[0], ids[2]])
        .unwrap();
    templates(&a)
        .container_presets_set(
            ids[0],
            Presets {
                split: false,
                shape: Some(Shape::Square),
                zoom: 20,
            },
        )
        .unwrap();
    templates(&a)
        .container_presets_set(
            ids[1],
            Presets {
                split: true,
                shape: None,
                zoom: 0,
            },
        )
        .unwrap();
    let s = a
        .items()
        .create_local("LV2-HTTP", "Oak Lounge Chair", "Chair", None)
        .unwrap()
        .1
        .session
        .id;
    a.sessions().activate(s).unwrap();
    let pb = arrive(&a, s, 100);
    a.clock.advance(301_000);
    let phone = TestPhone::start(&a);
    let token = phone.pair(&a);
    Fx {
        a,
        phone,
        token,
        s,
        ids,
        pb,
    }
}

// ---- LV2·C17: hello and the token ----

#[test]
fn lv2_c17a_hello_without_a_token_is_401() {
    let f = fixture(None);
    let r = send("GET", &f.phone.url("/api/phone/v1/hello"), None, None);
    assert_eq!((r.status, r.code().as_str()), (401, "pairing_unknown"));
}

#[test]
fn lv2_c17b_hello_with_the_bearer_token() {
    let f = fixture(None);
    let r = f.get("/api/phone/v1/hello");
    assert_eq!(r.status, 200);
    let body = r.json();
    let id = body["pairing"]["id"].as_i64().unwrap();
    assert_eq!(
        body,
        json!({
            "api": 1,
            "pairing": {"id": id, "name": "Phone 1"},
            "mac_name": TEST_MAC_NAME,
            "features": ["live", "capture", "focus", "current_shot", "move"],
        })
    );
}

#[test]
fn lv2_c17c_hello_with_the_pair_query() {
    let f = fixture(None);
    let url = f
        .phone
        .url(&format!("/api/phone/v1/hello?pair={}", f.token));
    assert_eq!(send("GET", &url, None, None).status, 200);
}

#[test]
fn lv2_c17d_a_revoked_token_is_401() {
    let f = fixture(None);
    let pairings = PhonePairings::new(f.a.store(), f.a.clock.as_ref());
    let id = pairings.verify(&f.token).unwrap().unwrap().id;
    pairings.revoke(id).unwrap();
    assert_eq!(f.get("/api/phone/v1/hello").status, 401);
}

// ---- LV2·C18: static files and the manifest ----

#[test]
fn lv2_c18a_the_phone_page_needs_no_token() {
    let f = fixture(None);
    let r = send("GET", &f.phone.url("/phone/"), None, None);
    assert_eq!(r.status, 200);
    assert!(
        r.content_type.starts_with("text/html"),
        "{}",
        r.content_type
    );
    assert_eq!(r.body, b"<p>phone</p>");
}

#[test]
fn lv2_c18b_assets_need_no_token() {
    let f = fixture(None);
    let r = send("GET", &f.phone.url("/assets/app.js"), None, None);
    assert_eq!(r.status, 200);
    assert!(r.content_type.contains("javascript"), "{}", r.content_type);
    assert_eq!(r.body, APP_JS);
}

#[test]
fn lv2_c18c_the_manifest_starts_the_installed_app_with_its_token() {
    let f = fixture(None);
    let url = f
        .phone
        .url(&format!("/phone/manifest.webmanifest?pair={}", f.token));
    let r = send("GET", &url, None, None);
    assert_eq!(r.status, 200);
    assert_eq!(
        r.json(),
        json!({
            "name": "Shoot",
            "short_name": "Shoot",
            "start_url": format!("/phone/?pair={}", f.token),
            "scope": "/phone/",
            "display": "standalone",
            "background_color": "#ffffff",
            "theme_color": "#ffffff",
            "icons": [{"src": "/phone/icon.png", "sizes": "512x512", "type": "image/png"}],
        })
    );
}

#[test]
fn lv2_c18d_the_manifest_without_a_token_is_401() {
    let f = fixture(None);
    let r = send(
        "GET",
        &f.phone.url("/phone/manifest.webmanifest"),
        None,
        None,
    );
    assert_eq!(r.status, 401);
}

#[test]
fn lv2_c18e_no_file_outside_the_web_root() {
    let f = fixture(None);
    assert!(f.phone.dir.path().join("secret").is_file());
    let r = send("GET", &f.phone.url("/assets/..%2F..%2Fsecret"), None, None);
    assert_eq!(r.status, 404);
}

// ---- LV2·C19: the session ----

#[test]
fn lv2_c19a_the_session_is_the_phone_view_of_the_active_session() {
    let f = fixture(None);
    let [ca, cb, cc] = f.ids;
    let r = f.get("/api/phone/v1/session");
    assert_eq!(r.status, 200);
    let v = r.json();
    assert_eq!(v["session"]["id"], f.s);
    let order: Vec<i64> = v["session"]["shots"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["container_id"].as_i64().unwrap())
        .collect();
    assert_eq!(order, vec![cb, ca, cc]);
    assert_eq!(
        v["session"]["current"],
        json!({"kind": "shot", "container_id": ca})
    );
    assert_eq!(v["session"]["shots"][0]["photo"]["photo_id"], f.pb);
}

#[test]
fn lv2_c19b_the_session_follows_the_macs_switch() {
    let f = fixture(None);
    let other = f.other_session();
    f.a.sessions().activate(other).unwrap();
    assert_eq!(
        f.get("/api/phone/v1/session").json()["session"]["id"],
        other
    );
}

#[test]
fn lv2_c19c_no_active_session_is_null() {
    let f = fixture(None);
    f.a.sessions().deactivate().unwrap();
    assert_eq!(
        f.get("/api/phone/v1/session").json(),
        json!({"session": null})
    );
}

// ---- LV2·C20: capture ----

#[test]
fn lv2_c20a_capture_from_the_phone_reaches_the_camera() {
    let f = fixture(Some(READY));
    let r = f.post("/api/phone/v1/capture", json!({"session_id": f.s}));
    assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(&r.body));
    let id = r.json()["capture_id"].as_str().unwrap().to_owned();
    f.settle_stdin(|lines| {
        lines
            .iter()
            .any(|s| s.starts_with(&format!("capture id={id} af=1")))
    });
}

#[test]
fn lv2_c20b_capture_for_another_session_is_refused() {
    let f = fixture(Some(READY));
    let other = f.other_session();
    let r = f.post("/api/phone/v1/capture", json!({"session_id": other}));
    assert_eq!((r.status, r.code().as_str()), (409, "session_changed"));
    thread::sleep(Duration::from_millis(200));
    assert!(!f.stdin().iter().any(|s| s.starts_with("capture ")));
}

#[test]
fn lv2_c20c_capture_without_a_camera_is_refused() {
    let f = fixture(Some(""));
    let r = f.post("/api/phone/v1/capture", json!({"session_id": f.s}));
    assert_eq!((r.status, r.code().as_str()), (409, "camera_not_connected"));
    assert!(!f.stdin().iter().any(|s| s.starts_with("capture ")));
}

// ---- LV2·C21: current shot, retake, move, delete ----

#[test]
fn lv2_c21a_the_phone_chooses_the_current_shot() {
    let f = fixture(None);
    let [_, _, cc] = f.ids;
    let r = f.post(
        "/api/phone/v1/current",
        json!({"session_id": f.s, "container_id": cc}),
    );
    assert_eq!(r.status, 200);
    let e =
        f.a.ledger()
            .exposure_observed(ExposureObserved {
                capture_id: Some(CaptureId("c21a".into())),
                at: f.a.clock.now_ms(),
                initiated: ExposureInitiator::Camera,
                provisional: false,
            })
            .unwrap();
    let exposure =
        f.a.ledger()
            .exposures(None)
            .unwrap()
            .into_iter()
            .find(|x| x.id == e)
            .unwrap();
    assert_eq!(exposure.shot.unwrap().container_id, Some(cc));
}

#[test]
fn lv2_c21b_a_filled_shot_is_refused() {
    let f = fixture(None);
    let [_, cb, _] = f.ids;
    let r = f.post(
        "/api/phone/v1/current",
        json!({"session_id": f.s, "container_id": cb}),
    );
    assert_eq!((r.status, r.code().as_str()), (409, "shot_filled"));
}

#[test]
fn lv2_c21c_retake_from_the_phone() {
    let f = fixture(None);
    let [_, cb, _] = f.ids;
    let r = f.post(
        "/api/phone/v1/retake",
        json!({"session_id": f.s, "container_id": cb}),
    );
    assert_eq!(r.status, 200);
    assert_eq!(f.place(f.pb).0, C::Deleted);
    assert_eq!(f.current(), Some(Current::Shot(cb)));
}

#[test]
fn lv2_c21d_move_from_the_phone() {
    let f = fixture(None);
    let r = f.post(
        &format!("/api/phone/v1/photos/{}/move", f.pb),
        json!({"session_id": f.s, "target": {"kind": "extra_end"}}),
    );
    assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(&r.body));
    assert_eq!(f.place(f.pb), (C::Main, Some(1), None));
}

#[test]
fn lv2_c21e_delete_from_the_phone_denies() {
    let f = fixture(None);
    f.a.app
        .camera_runtime
        .current_shots()
        .set(
            f.a.store(),
            f.a.events.as_ref(),
            f.s,
            Current::Free,
            f.a.clock.now_ms(),
        )
        .unwrap();
    let q = arrive(&f.a, f.s, 101);
    assert_eq!(f.place(q).0, C::Condition);
    let r = f.post(
        &format!("/api/phone/v1/photos/{q}/delete"),
        json!({"session_id": f.s}),
    );
    assert_eq!(r.status, 200);
    assert_eq!(f.place(q).0, C::Deleted);
    assert!(f.a.store().photo_denial(q).unwrap().is_some());
}

#[test]
fn lv2_c21f_every_post_names_the_active_session() {
    let f = fixture(None);
    let [_, cb, cc] = f.ids;
    let other = f.other_session();
    let before = (f.place(f.pb), f.current());
    for (path, body) in [
        (
            "/api/phone/v1/current".to_owned(),
            json!({"session_id": other, "container_id": cc}),
        ),
        (
            "/api/phone/v1/retake".to_owned(),
            json!({"session_id": other, "container_id": cb}),
        ),
        (
            format!("/api/phone/v1/photos/{}/move", f.pb),
            json!({"session_id": other, "target": {"kind": "extra_end"}}),
        ),
        (
            format!("/api/phone/v1/photos/{}/delete", f.pb),
            json!({"session_id": other}),
        ),
    ] {
        let r = f.post(&path, body);
        assert_eq!(
            (r.status, r.code().as_str()),
            (409, "session_changed"),
            "{path}"
        );
    }
    assert_eq!((f.place(f.pb), f.current()), before);
}

// ---- LV2·C22: focus ----

#[test]
fn lv2_c22a_focus_at_the_centre() {
    let f = fixture(Some(READY));
    let r = f.post(
        "/api/phone/v1/focus",
        json!({"session_id": f.s, "x": 0.5, "y": 0.5}),
    );
    assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(&r.body));
    let id = r.json()["focus_id"].as_str().unwrap().to_owned();
    f.settle_stdin(|lines| lines.contains(&format!("focus x=320 y=240 af=1 cmd_id={id}")));
}

#[test]
fn lv2_c22b_clear_focus_recentres_without_autofocus() {
    let f = fixture(Some(READY));
    let r = f.post("/api/phone/v1/focus/clear", json!({"session_id": f.s}));
    assert_eq!(r.status, 200);
    let id = r.json()["focus_id"].as_str().unwrap().to_owned();
    f.settle_stdin(|lines| lines.contains(&format!("focus x=320 y=240 af=0 cmd_id={id}")));
}

#[test]
fn lv2_c22c_a_point_outside_the_frame_is_refused() {
    let f = fixture(Some(READY));
    let r = f.post(
        "/api/phone/v1/focus",
        json!({"session_id": f.s, "x": 2, "y": 0.5}),
    );
    assert_eq!((r.status, r.code().as_str()), (400, "invalid_value"));
    thread::sleep(Duration::from_millis(200));
    assert!(!f.stdin().iter().any(|s| s.starts_with("focus ")));
}

// ---- LV2·C23: images ----

#[test]
fn lv2_c23a_the_preview_the_session_view_shows() {
    let f = fixture(None);
    let preview = f.phone.dir.path().join("preview-b.jpg");
    std::fs::write(&preview, b"preview bytes of B").unwrap();
    rusqlite::Connection::open(f.a.app.paths.db())
        .unwrap()
        .execute(
            "UPDATE photos SET adjusted_preview_path=?1 WHERE id=?2",
            rusqlite::params![preview.to_string_lossy(), f.pb],
        )
        .unwrap();
    let url = f.phone.url(&format!(
        "/api/phone/v1/photos/{}/preview?pair={}",
        f.pb, f.token
    ));
    let r = send("GET", &url, None, None);
    assert_eq!(
        (r.status, r.body.as_slice()),
        (200, &b"preview bytes of B"[..])
    );
}

#[test]
fn lv2_c23b_the_container_example_as_stored() {
    let f = fixture(None);
    let [ca, _, _] = f.ids;
    let t = templates(&f.a).get("Chair").unwrap().unwrap();
    let file = t
        .containers
        .iter()
        .find(|c| c.id == ca)
        .unwrap()
        .example_file
        .clone()
        .unwrap();
    let stored = std::fs::read(
        f.a.app
            .paths
            .root
            .join("templates")
            .join(t.id.to_string())
            .join(file),
    )
    .unwrap();
    let url = f.phone.url(&format!(
        "/api/phone/v1/containers/{ca}/example?pair={}",
        f.token
    ));
    let r = send("GET", &url, None, None);
    assert_eq!(r.status, 200);
    assert_eq!(r.body, stored);
}

#[test]
fn lv2_c23c_a_photo_of_another_session_is_404() {
    let f = fixture(None);
    let other = f.other_session();
    f.a.sessions().activate(other).unwrap();
    let url = f.phone.url(&format!(
        "/api/phone/v1/photos/{}/preview?pair={}",
        f.pb, f.token
    ));
    assert_eq!(send("GET", &url, None, None).status, 404);
}
