//! LV10·U1 — the composition on the phone wire and the phone's two save routes
//! (`master_plan_live.md` §5 PH4b, PH6c; book 10J LV-D21 (6), 10D MC-27): rows LV10·C1–C4, HTTP
//! requests to `TestPhone` with a paired token, then the store, the history and the recorded
//! events.
//!
//! Fixture (as `p28_composition_foundation.rs`'s `ready()`): session S active; photo P with a
//! 2560 × 1707 adjusted preview and the default 1:1 composition at revision 0, classified Main.
use psm_core::{
    domain::{
        composition::{
            CompositionKind, CompositionValues, EditorGeometry, Fingerprint, Flip, Frame,
            FrameSource, Orientation, Origin, Rect, Shape,
        },
        photo::{Classification, Photo},
    },
    events::AppEvent,
};
use psm_testkit::{TestApp, fixtures::synthetic_jpeg::SyntheticJpeg, phone::TestPhone};
use serde_json::{Value, json};

struct Resp {
    status: u16,
    body: Value,
}
impl Resp {
    fn code(&self) -> &str {
        self.body["error"]["code"].as_str().unwrap_or_default()
    }
}
fn send(method: &str, url: &str, bearer: &str, body: Option<Value>) -> Resp {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(async {
        let client = reqwest::Client::new();
        let mut request = match method {
            "POST" => client.post(url),
            _ => client.get(url),
        }
        .bearer_auth(bearer);
        if let Some(body) = body {
            request = request.json(&body);
        }
        let response = request.send().await.unwrap();
        let status = response.status().as_u16();
        let bytes = response.bytes().await.unwrap();
        Resp {
            status,
            body: serde_json::from_slice(&bytes).unwrap_or(Value::Null),
        }
    })
}

fn session(a: &TestApp, name: &str) -> i64 {
    a.items()
        .create_local(name, name, "object", None)
        .unwrap()
        .1
        .session
        .id
}
fn add(a: &TestApp, s: i64, name: &str) -> Photo {
    let photo = a.add_photo(s, name).unwrap();
    a.store().upsert_raw_state(&photo.raw_path, None).unwrap();
    a.fake
        .add_image(&photo.raw_path, &[&format!("v{}", photo.id)], false);
    photo
}
/// Capture One's adjusted preview of `photo`, 2560 × 1707: the composition gets its frame.
fn preview(a: &TestApp, photo: &Photo) {
    a.app.previews.request_initial(&photo.raw_path).unwrap();
    let job = a
        .fake
        .state
        .lock()
        .unwrap()
        .issued
        .last()
        .unwrap()
        .0
        .clone();
    a.complete_render_with(&job, &SyntheticJpeg::rgb(2560, 1707, [200, 100, 50]))
        .unwrap();
}
/// The default 1:1 rect on 2560 × 1707.
fn square() -> Rect {
    Rect {
        x: (1.0 - 1707.0 / 2560.0) / 2.0,
        y: 0.0,
        w: 1707.0 / 2560.0,
        h: 1.0,
    }
}
fn default_values() -> CompositionValues {
    CompositionValues {
        kind: CompositionKind::Normal,
        shape: Shape::Square,
        orientation: Orientation::Named,
        quarter_turns: 0,
        flip: Flip::None,
        straighten: 0.0,
        rect: square(),
    }
}
/// `p28`'s `values()`: valid on the 2560 × 1707 frame (a quarter turn, flip, 2.5°).
fn values() -> CompositionValues {
    CompositionValues {
        kind: CompositionKind::Normal,
        shape: Shape::Square,
        orientation: Orientation::Turned,
        quarter_turns: 1,
        flip: Flip::Horizontal,
        straighten: 2.5,
        rect: Rect {
            x: 0.1,
            y: 0.3,
            w: 0.8,
            h: 1365.6 / 2560.0,
        },
    }
}
/// The same values as the phone sends them (the wire's spellings, written out).
fn values_json() -> Value {
    json!({
        "kind": "normal", "shape": "1:1", "orientation": "turned", "quarter_turns": 1,
        "flip": "horizontal", "straighten": 2.5,
        "rect": {"x": 0.1, "y": 0.3, "w": 0.8, "h": 1365.6 / 2560.0},
    })
}

struct Fx {
    a: TestApp,
    phone: TestPhone,
    token: String,
    /// The active session.
    s: i64,
    /// P's id.
    p: i64,
}
impl Fx {
    fn ready() -> Self {
        let a = TestApp::with_fake_document().unwrap();
        let s = session(&a, "S1");
        let p = add(&a, s, "a.ARW");
        preview(&a, &p);
        a.review().classify(p.id, Classification::Main).unwrap();
        a.sessions().activate(s).unwrap();
        Self::start(a, s, p.id)
    }
    fn start(a: TestApp, s: i64, p: i64) -> Self {
        let phone = TestPhone::start(&a);
        let token = phone.pair(&a);
        a.events.0.lock().unwrap().clear();
        Self {
            a,
            phone,
            token,
            s,
            p,
        }
    }
    fn post(&self, path: &str, body: Value) -> Resp {
        send("POST", &self.phone.url(path), &self.token, Some(body))
    }
    fn save_in(&self, session: i64, photo: i64, revision: i64, values: Value) -> Resp {
        self.post(
            &format!("/api/phone/v1/photos/{photo}/composition"),
            json!({"session_id": session, "revision": revision, "values": values}),
        )
    }
    fn save(&self, revision: i64, values: Value) -> Resp {
        self.save_in(self.s, self.p, revision, values)
    }
    fn reset(&self, revision: i64) -> Resp {
        self.post(
            &format!("/api/phone/v1/photos/{}/composition/reset", self.p),
            json!({"session_id": self.s, "revision": revision}),
        )
    }
    /// `session` of the phone's session view.
    fn view(&self) -> Value {
        let r = send(
            "GET",
            &self.phone.url("/api/phone/v1/session"),
            &self.token,
            None,
        );
        assert_eq!(r.status, 200);
        r.body["session"].clone()
    }
    /// (values, origin, revision) of a photo's stored composition.
    fn stored_of(&self, photo: i64) -> (CompositionValues, Origin, i64) {
        let c = self.a.composition(photo).unwrap();
        (c.values().unwrap(), c.origin, c.revision)
    }
    fn stored(&self) -> (CompositionValues, Origin, i64) {
        self.stored_of(self.p)
    }
    /// The label of the Mac's next undo in the active session.
    fn undo_label(&self) -> Option<String> {
        self.a.app.history.state(self.s).undo
    }
    /// What the phone is told about since the fixture was ready (PH8), in order.
    fn announced(&self) -> Vec<Told> {
        self.a
            .events
            .0
            .lock()
            .unwrap()
            .iter()
            .filter_map(|e| match e {
                AppEvent::PhotoChanged {
                    photo_id,
                    session_id,
                } => Some(Told::Photo(*photo_id, *session_id)),
                AppEvent::PhotoAdded { photo_id, .. } => Some(Told::Added(*photo_id)),
                AppEvent::SessionChanged { session_id } => Some(Told::Session(*session_id)),
                _ => None,
            })
            .collect()
    }
}

/// A change announcement the phone's socket forwards (PH5, PH8).
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Told {
    /// (photo, session)
    Photo(i64, i64),
    Added(i64),
    Session(i64),
}

#[test]
fn lv10_c1a_the_view_carries_the_composition_and_the_default_shape() {
    let fx = Fx::ready();
    let rect = fx.a.composition(fx.p).unwrap().rect.unwrap();
    assert_eq!(rect, square());
    let v = fx.view();
    assert_eq!(v["extras"][0]["photo_id"], json!(fx.p));
    assert_eq!(
        v["extras"][0]["composition"],
        json!({
            "kind": "normal", "shape": "1:1", "orientation": "named", "quarter_turns": 0,
            "flip": "none", "straighten": 0.0,
            "rect": {"x": rect.x, "y": rect.y, "w": rect.w, "h": rect.h},
            "frame": {"w": 2560, "h": 1707},
            "state": "ok",
        })
    );
    assert_eq!(v["default_shape"], json!("1:1"));
}

#[test]
fn lv10_c1b_a_photo_without_the_full_frame_preview_has_no_composition() {
    let fx = Fx::ready();
    // Q: its frame came from the camera's own preview, as `OriginalPreviewJob` records it.
    let q = add(&fx.a, fx.s, "b.ARW");
    let original = fx.a.dir.path().join("b-original.jpg");
    std::fs::write(&original, b"jpeg").unwrap();
    fx.a.store()
        .set_original_preview(&q.raw_path, Some(original.to_str().unwrap()), None)
        .unwrap();
    fx.a.store()
        .adopt_frame(
            &q.raw_path,
            Fingerprint {
                frame: Frame { w: 1616, h: 1080 },
                source: FrameSource::Camera,
                geometry: EditorGeometry::default(),
            },
        )
        .unwrap();
    fx.a.review().classify(q.id, Classification::Main).unwrap();
    let stored = fx.a.composition(q.id).unwrap();
    assert!(stored.rect.is_some() && stored.frame() == Some(Frame { w: 1616, h: 1080 }));
    assert_eq!(
        fx.a.store().photo(q.id).unwrap().adjusted_preview_path,
        None
    );

    let v = fx.view();
    assert_eq!(v["extras"][1]["photo_id"], json!(q.id));
    assert_eq!(v["extras"][1]["composition"], Value::Null);
    // The camera's preview is what the route serves for Q; P keeps its composition.
    assert!(v["extras"][1]["preview"].is_string());
    assert_eq!(v["extras"][0]["composition"]["state"], json!("ok"));
}

#[test]
fn lv10_c2a_a_save_from_the_phone_is_the_macs_own_composition() {
    let fx = Fx::ready();
    let r = fx.save(0, values_json());
    assert_eq!((r.status, &r.body), (200, &json!({"revision": 1})));
    assert_eq!(fx.stored(), (values(), Origin::User, 1));
    assert_eq!(fx.undo_label().as_deref(), Some("Compose 01"));
    assert_eq!(
        fx.announced(),
        [Told::Photo(fx.p, fx.s), Told::Session(fx.s)]
    );
    // One undo on the Mac takes the phone's framing back.
    fx.a.history().undo(fx.s).unwrap();
    let (after_undo, origin, _) = fx.stored();
    assert_eq!((after_undo, origin), (default_values(), Origin::Default));
}

#[test]
fn lv10_c2b_unchanged_values_save_nothing() {
    let fx = Fx::ready();
    let before = fx.undo_label();
    let r = fx.save(
        0,
        json!({
            "kind": "normal", "shape": "1:1", "orientation": "named", "quarter_turns": 0,
            "flip": "none", "straighten": 0.0,
            "rect": {"x": square().x, "y": 0.0, "w": square().w, "h": 1.0},
        }),
    );
    assert_eq!((r.status, &r.body), (200, &json!({"revision": 0})));
    assert_eq!(fx.stored(), (default_values(), Origin::Default, 0));
    assert_eq!(fx.undo_label(), before);
    assert_eq!(fx.announced(), []);
}

#[test]
fn lv10_c3a_an_invalid_framing_is_refused() {
    let fx = Fx::ready();
    // Straightened 10° with the full-height rect: its corners leave the image.
    let r = fx.save(
        0,
        json!({
            "kind": "normal", "shape": "1:1", "orientation": "named", "quarter_turns": 0,
            "flip": "none", "straighten": 10.0,
            "rect": {"x": square().x, "y": 0.0, "w": square().w, "h": 1.0},
        }),
    );
    assert_eq!((r.status, r.code()), (400, "composition_invalid"));
    assert_eq!(fx.stored(), (default_values(), Origin::Default, 0));
}

#[test]
fn lv10_c3b_a_framing_changed_on_the_mac_is_not_overwritten() {
    let fx = Fx::ready();
    fx.a.app.compositions.set(fx.p, values()).unwrap();
    // Valid on its own (the default square moved left): only the revision refuses it.
    let r = fx.save(
        0,
        json!({
            "kind": "normal", "shape": "1:1", "orientation": "named", "quarter_turns": 0,
            "flip": "none", "straighten": 0.0,
            "rect": {"x": 0.1, "y": 0.0, "w": square().w, "h": 1.0},
        }),
    );
    assert_eq!((r.status, r.code()), (409, "composition_changed"));
    assert_eq!(fx.stored(), (values(), Origin::User, 1));
}

#[test]
fn lv10_c3c_the_guard_and_the_photos_session_refuse() {
    let a = TestApp::with_fake_document().unwrap();
    let s = session(&a, "S1");
    let p = add(&a, s, "a.ARW");
    preview(&a, &p);
    a.review().classify(p.id, Classification::Main).unwrap();
    let other = session(&a, "S2");
    let r = add(&a, other, "r.ARW");
    preview(&a, &r);
    a.review().classify(r.id, Classification::Main).unwrap();
    a.sessions().activate(s).unwrap();
    let fx = Fx::start(a, s, p.id);

    // A session that is not the active one.
    let refused = fx.save_in(other, fx.p, 0, values_json());
    assert_eq!((refused.status, refused.code()), (409, "session_changed"));
    // A photo of another session, with S active.
    let missing = fx.save_in(fx.s, r.id, 0, values_json());
    assert_eq!((missing.status, missing.code()), (404, "not_found"));

    assert_eq!(fx.stored(), (default_values(), Origin::Default, 0));
    assert_eq!(fx.stored_of(r.id), (default_values(), Origin::Default, 0));
    assert_eq!(fx.announced(), []);
}

#[test]
fn lv10_c4a_reset_saves_the_default_framing() {
    let fx = Fx::ready();
    assert_eq!(fx.save(0, values_json()).status, 200);
    let r = fx.reset(1);
    assert_eq!((r.status, &r.body), (200, &json!({"revision": 2})));
    assert_eq!(fx.stored(), (default_values(), Origin::Default, 2));
    // One undo step: one undo brings the saved framing back.
    assert_eq!(fx.undo_label().as_deref(), Some("Compose 01"));
    fx.a.history().undo(fx.s).unwrap();
    let (after_undo, origin, _) = fx.stored();
    assert_eq!((after_undo, origin), (values(), Origin::User));
}

#[test]
fn lv10_c4b_a_stale_reset_is_refused() {
    let fx = Fx::ready();
    assert_eq!(fx.save(0, values_json()).status, 200);
    let r = fx.reset(0);
    assert_eq!((r.status, r.code()), (409, "composition_changed"));
    assert_eq!(fx.stored(), (values(), Origin::User, 1));
}
