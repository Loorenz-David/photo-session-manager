//! LV8·U1 — the Mac's side of sort and arrange on the phone (`master_plan_live.md` §5 PH4a; book
//! 10J LV-D20): rows LV8·C1–C4, HTTP requests to `TestPhone` with a paired token, then the store,
//! the phone's session view, the history and the recorded events.
//!
//! Fixture U1: T3 (A final 1 shoot 2, B final 2 shoot 1, C final 3 shoot 3); templated session S
//! active; P in A, Q in C; Condition K1, K2, K3; extras E1, E2, E3 (Main 3, 4, 5); the current
//! shot is B. Untemplated session U: Main U1, U2, U3; Condition V1.
use psm_core::{
    domain::{photo::Classification as C, template::PlaceRow},
    events::AppEvent,
    ports::clock::Clock,
    services::{
        current_shot::Current,
        template_placement::{self, HandAction, MoveTarget},
        templates::TemplatesService,
    },
};
use psm_infra::imaging::ImageOpsImpl;
use psm_testkit::{TestApp, phone::TestPhone};
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

/// What the phone is told about: a changed photo, a changed session, a new undo step.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
enum Told {
    Photo(i64),
    Session(i64),
    History,
}

/// (classification, position, slot, placed by hand)
type Place = (C, Option<i64>, Option<i64>, bool);

struct Fx {
    a: TestApp,
    phone: TestPhone,
    token: String,
    /// The active session.
    s: i64,
}
impl Fx {
    fn start(a: TestApp, s: i64) -> Self {
        a.events.0.lock().unwrap().clear();
        let phone = TestPhone::start(&a);
        let token = phone.pair(&a);
        Self { a, phone, token, s }
    }
    fn post(&self, path: &str, body: Value) -> Resp {
        send("POST", &self.phone.url(path), &self.token, Some(body))
    }
    fn classify_in(&self, session: i64, photo: i64, to: &str) -> Resp {
        self.post(
            &format!("/api/phone/v1/photos/{photo}/classify"),
            json!({"session_id": session, "to": to}),
        )
    }
    fn classify(&self, photo: i64, to: &str) -> Resp {
        self.classify_in(self.s, photo, to)
    }
    fn insert(&self, photo: i64, kind: &str, index: i64) -> Resp {
        self.post(
            &format!("/api/phone/v1/photos/{photo}/move"),
            json!({"session_id": self.s, "target": {"kind": kind, "index": index}}),
        )
    }
    /// `current` of the phone's session view.
    fn current(&self) -> Value {
        let r = send(
            "GET",
            &self.phone.url("/api/phone/v1/session"),
            &self.token,
            None,
        );
        assert_eq!(r.status, 200);
        r.body["session"]["current"].clone()
    }
    fn place(&self, id: i64) -> Place {
        let p = self.a.store().photo(id).unwrap();
        let row = self.a.store().photo_place_row(id).unwrap();
        (
            p.classification,
            p.position,
            row.as_ref().and_then(|r| r.slot),
            row.is_some_and(|r| r.place_by_hand),
        )
    }
    /// The photo ids of a group of `session` in order; the positions are checked to be 1…n.
    fn group_of(&self, session: i64, class: C) -> Vec<i64> {
        let mut v: Vec<_> = self
            .a
            .store()
            .photos_in_session(session)
            .unwrap()
            .into_iter()
            .filter(|p| p.classification == class)
            .collect();
        v.sort_by_key(|p| p.position);
        let positions: Vec<_> = v.iter().map(|p| p.position).collect();
        let dense: Vec<_> = (1..=v.len() as i64).map(Some).collect();
        assert_eq!(positions, dense, "{class:?} positions");
        v.into_iter().map(|p| p.id).collect()
    }
    fn group(&self, class: C) -> Vec<i64> {
        self.group_of(self.s, class)
    }
    /// Every photo of `session`: its place and its place row.
    fn everything_in(&self, session: i64) -> Vec<(i64, Place, Option<PlaceRow>)> {
        let mut v: Vec<_> = self
            .a
            .store()
            .photos_in_session(session)
            .unwrap()
            .into_iter()
            .map(|p| {
                (
                    p.id,
                    self.place(p.id),
                    self.a.store().photo_place_row(p.id).unwrap(),
                )
            })
            .collect();
        v.sort_by_key(|(id, ..)| *id);
        v
    }
    fn everything(&self) -> Vec<(i64, Place, Option<PlaceRow>)> {
        self.everything_in(self.s)
    }
    fn place_rows(&self) -> Vec<Option<PlaceRow>> {
        self.everything().into_iter().map(|(.., row)| row).collect()
    }
    /// What was announced since the fixture, in order.
    fn told(&self) -> Vec<Told> {
        self.a
            .events
            .0
            .lock()
            .unwrap()
            .iter()
            .filter_map(|e| match e {
                AppEvent::PhotoChanged { photo_id, .. } => Some(Told::Photo(*photo_id)),
                AppEvent::SessionChanged { session_id } => Some(Told::Session(*session_id)),
                AppEvent::HistoryChanged { .. } => Some(Told::History),
                _ => None,
            })
            .collect()
    }
    /// The photo and session announcements (the undo step's own event left out).
    fn changes(&self) -> Vec<Told> {
        self.told()
            .into_iter()
            .filter(|t| *t != Told::History)
            .collect()
    }
    fn undo_label(&self) -> Option<String> {
        self.a.history().state(self.s).undo
    }
    fn in_container(&self, final_no: i64) -> Vec<i64> {
        self.group(C::Main)
            .into_iter()
            .filter(|id| self.place(*id).2 == Some(final_no))
            .collect()
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
fn activate(a: &TestApp, sku: &str, kind: &str) -> i64 {
    let s = a
        .items()
        .create_local(sku, sku, kind, None)
        .unwrap()
        .1
        .session
        .id;
    a.sessions().activate(s).unwrap();
    s
}
fn set(a: &TestApp, s: i64, target: Current) {
    a.app
        .camera_runtime
        .current_shots()
        .set(a.store(), a.events.as_ref(), s, target, a.clock.now_ms())
        .unwrap();
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

/// The ids fixture U1 made.
struct U1 {
    /// A, B, C
    containers: [i64; 3],
    p: i64,
    q: i64,
    k: [i64; 3],
    e: [i64; 3],
}

/// Fixture U1 built on `a` (T3, session S active and filled).
fn build_u1(a: &TestApp) -> (i64, U1) {
    let t = templates(a).create("Chair").unwrap();
    let t = (0..3).fold(t, |t, _| templates(a).container_add(t.id, None).unwrap());
    let [ca, cb, cc] = [t.containers[0].id, t.containers[1].id, t.containers[2].id];
    templates(a)
        .shooting_order_set(t.id, &[cb, ca, cc])
        .unwrap();
    let s = activate(a, "LV8-S", "Chair");
    set(a, s, Current::Shot(ca));
    let p = arrive(a, s, 100);
    set(a, s, Current::Shot(cc));
    let q = arrive(a, s, 101);
    let mut k = [0; 3];
    for (i, slot) in k.iter_mut().enumerate() {
        set(a, s, Current::Free);
        *slot = arrive(a, s, 110 + i as u32);
    }
    let mut e = [0; 3];
    for (i, slot) in e.iter_mut().enumerate() {
        set(a, s, Current::Free);
        *slot = arrive(a, s, 120 + i as u32);
        template_placement::move_to(
            &HandAction {
                store: a.store(),
                clock: a.clock.as_ref(),
                events: a.events.as_ref(),
                history: a.app.history.as_ref(),
                shots: a.app.camera_runtime.current_shots(),
            },
            *slot,
            MoveTarget::ExtraEnd,
        )
        .unwrap();
    }
    (
        s,
        U1 {
            containers: [ca, cb, cc],
            p,
            q,
            k,
            e,
        },
    )
}

/// Fixture U1 with the phone paired; the fixture itself is checked here once per test.
fn u1() -> (Fx, U1) {
    let a = TestApp::with_camera().unwrap();
    let (s, ids) = build_u1(&a);
    let f = Fx::start(a, s);
    let [e1, e2, e3] = ids.e;
    assert_eq!(f.group(C::Main), vec![ids.p, ids.q, e1, e2, e3]);
    assert_eq!(f.group(C::Condition), ids.k.to_vec());
    assert_eq!(
        (f.place(ids.p).2, f.place(ids.q).2, f.place(e1).2),
        (Some(1), Some(3), None)
    );
    assert_eq!(
        f.current(),
        json!({"kind": "shot", "container_id": ids.containers[1]})
    );
    (f, ids)
}

/// The ids of the untemplated session U.
struct Untemplated {
    s: i64,
    u: [i64; 3],
    v1: i64,
}

/// Session U on `a` (active when this returns): Main U1, U2, U3; Condition V1 (classified on the
/// Mac).
fn build_untemplated(a: &TestApp) -> Untemplated {
    let s = activate(a, "LV8-U", "Table");
    let u = [arrive(a, s, 200), arrive(a, s, 201), arrive(a, s, 202)];
    let v1 = arrive(a, s, 203);
    a.review().classify(v1, C::Condition).unwrap();
    Untemplated { s, u, v1 }
}

fn untemplated() -> (Fx, Untemplated) {
    let a = TestApp::with_camera().unwrap();
    let ids = build_untemplated(&a);
    let f = Fx::start(a, ids.s);
    assert_eq!(f.group(C::Main), ids.u.to_vec());
    assert_eq!(f.group(C::Condition), vec![ids.v1]);
    (f, ids)
}

/// The request answered 200 `{ok: true}` and changed nothing: no place, position or place row, no
/// announcement, and no undo step (one undo takes back the fixture's last step: E3 made an extra).
fn nothing_changed_by(f: &Fx, ids: &U1, request: impl FnOnce() -> Resp) {
    let before = f.everything();
    let r = request();
    assert_eq!((r.status, &r.body), (200, &json!({"ok": true})));
    assert_eq!(f.everything(), before);
    assert_eq!(f.told(), vec![]);
    f.a.history().undo(f.s).unwrap();
    assert_eq!(f.place(ids.e[2]).0, C::Condition);
}

// ---- LV8·C1: classify to Main ----

#[test]
fn lv8_c1a_a_condition_photo_goes_to_the_end_of_main() {
    let (f, ids) = u1();
    let [k1, k2, k3] = ids.k;
    let [e1, e2, e3] = ids.e;
    let r = f.classify(k2, "main");
    assert_eq!((r.status, &r.body), (200, &json!({"ok": true})));
    assert_eq!(f.group(C::Main), vec![ids.p, ids.q, e1, e2, e3, k2]);
    assert_eq!(f.place(k2), (C::Main, Some(6), None, true));
    assert_eq!(f.group(C::Condition), vec![k1, k3]);
    assert_eq!(f.undo_label().as_deref(), Some("Move image"));
    assert_eq!(
        f.changes(),
        vec![Told::Photo(k2), Told::Photo(k3), Told::Session(f.s)]
    );
    // The step is the classify's own (the fixture left "Move image" steps too): one undo takes
    // exactly it back.
    assert_eq!(f.a.history().undo(f.s).unwrap().label, "Move image");
    assert_eq!(f.group(C::Condition), vec![k1, k2, k3]);
    assert_eq!(f.group(C::Main), vec![ids.p, ids.q, e1, e2, e3]);
}

#[test]
fn lv8_c1b_a_template_shots_photo_becomes_an_extra_and_its_shot_current() {
    let (f, ids) = u1();
    let [ca, _, _] = ids.containers;
    let [e1, e2, e3] = ids.e;
    assert_eq!(f.classify(ids.p, "main").status, 200);
    assert_eq!(f.group(C::Main), vec![ids.q, e1, e2, e3, ids.p]);
    assert_eq!(f.place(ids.p), (C::Main, Some(5), None, true));
    assert_eq!(f.in_container(1), Vec::<i64>::new());
    assert_eq!(f.current(), json!({"kind": "shot", "container_id": ca}));
    f.a.history().undo(f.s).unwrap();
    assert_eq!(
        (f.place(ids.p).0, f.place(ids.p).1, f.place(ids.p).2),
        (C::Main, Some(1), Some(1))
    );
    assert_eq!(f.in_container(1), vec![ids.p]);
}

#[test]
fn lv8_c1c_an_extra_is_already_main() {
    let (f, ids) = u1();
    nothing_changed_by(&f, &ids, || f.classify(ids.e[1], "main"));
}

// ---- LV8·C2: classify to Condition ----

#[test]
fn lv8_c2a_a_template_shots_photo_goes_to_condition_and_its_shot_is_current() {
    let (f, ids) = u1();
    let [_, _, cc] = ids.containers;
    let [k1, k2, k3] = ids.k;
    assert_eq!(f.classify(ids.q, "condition").status, 200);
    assert_eq!(f.group(C::Condition), vec![k1, k2, k3, ids.q]);
    assert_eq!(f.in_container(3), Vec::<i64>::new());
    assert_eq!(f.current(), json!({"kind": "shot", "container_id": cc}));
    assert!(f.place(ids.q).3, "Q is placed by hand");
}

#[test]
fn lv8_c2b_an_extra_goes_to_condition_and_the_current_shot_stays() {
    let (f, ids) = u1();
    let [_, cb, _] = ids.containers;
    let [k1, k2, k3] = ids.k;
    let [e1, e2, e3] = ids.e;
    let b = json!({"kind": "shot", "container_id": cb});
    assert_eq!(f.current(), b);
    assert_eq!(f.classify(e1, "condition").status, 200);
    assert_eq!(f.group(C::Condition), vec![k1, k2, k3, e1]);
    assert_eq!(f.group(C::Main), vec![ids.p, ids.q, e2, e3]);
    assert_eq!(f.current(), b);
}

#[test]
fn lv8_c2c_a_condition_photo_is_already_condition() {
    let (f, ids) = u1();
    nothing_changed_by(&f, &ids, || f.classify(ids.k[0], "condition"));
}

#[test]
fn lv8_c2d_an_untemplated_session_classifies_without_a_current_shot() {
    let (f, ids) = untemplated();
    let [u1, u2, u3] = ids.u;
    assert_eq!(f.current(), Value::Null);
    assert_eq!(f.classify(ids.v1, "main").status, 200);
    assert_eq!(f.group(C::Main), vec![u1, u2, u3, ids.v1]);
    assert_eq!(f.group(C::Condition), Vec::<i64>::new());
    assert_eq!(f.current(), Value::Null);
    assert_eq!(f.classify(u1, "condition").status, 200);
    assert_eq!(f.group(C::Main), vec![u2, u3, ids.v1]);
    assert_eq!(f.group(C::Condition), vec![u1]);
    assert_eq!(f.current(), Value::Null);
}

// ---- LV8·C3: insert and shift ----

#[test]
fn lv8_c3a_an_extra_is_put_first_among_the_extras() {
    let (f, ids) = u1();
    let [e1, e2, e3] = ids.e;
    let rows = f.place_rows();
    let r = f.insert(e3, "extra_insert", 1);
    assert_eq!((r.status, &r.body), (200, &json!({"ok": true})));
    assert_eq!(f.group(C::Main), vec![ids.p, ids.q, e3, e1, e2]);
    assert_eq!((f.place(ids.p).2, f.place(ids.q).2), (Some(1), Some(3)));
    assert_eq!(f.place_rows(), rows);
    assert_eq!(f.undo_label().as_deref(), Some("Move image"));
    let mut changes = f.changes();
    assert_eq!(changes.pop(), Some(Told::Session(f.s)));
    changes.sort();
    assert_eq!(
        changes,
        vec![Told::Photo(e1), Told::Photo(e2), Told::Photo(e3)]
    );
    // One step: one undo puts the three back.
    f.a.history().undo(f.s).unwrap();
    assert_eq!(f.group(C::Main), vec![ids.p, ids.q, e1, e2, e3]);
}

#[test]
fn lv8_c3b_a_condition_photo_is_put_last_among_them() {
    let (f, ids) = u1();
    let [k1, k2, k3] = ids.k;
    assert_eq!(f.insert(k1, "condition_insert", 3).status, 200);
    assert_eq!(f.group(C::Condition), vec![k2, k3, k1]);
}

#[test]
fn lv8_c3c_an_untemplated_main_photo_is_put_first() {
    let (f, ids) = untemplated();
    let [u1, u2, u3] = ids.u;
    assert_eq!(f.insert(u3, "extra_insert", 1).status, 200);
    assert_eq!(f.group(C::Main), vec![u3, u1, u2]);
}

#[test]
fn lv8_c3d_the_same_place_changes_nothing() {
    let (f, ids) = u1();
    nothing_changed_by(&f, &ids, || f.insert(ids.e[1], "extra_insert", 2));
}

// ---- LV8·C4: refusals ----

#[test]
fn lv8_c4a_an_insert_outside_the_photos_own_group_or_range_is_refused() {
    let (f, ids) = u1();
    let before = f.everything();
    for (photo, kind, index, why) in [
        (
            ids.k[0],
            "extra_insert",
            1,
            "a Condition photo among extras",
        ),
        (
            ids.p,
            "extra_insert",
            1,
            "a template shot's photo among extras",
        ),
        (ids.e[0], "condition_insert", 1, "an extra among Condition"),
        (ids.e[0], "extra_insert", 0, "place 0"),
        (ids.e[0], "extra_insert", 4, "place 4 of 3"),
    ] {
        let r = f.insert(photo, kind, index);
        assert_eq!((r.status, r.code()), (400, "invalid_value"), "{why}");
        assert_eq!(f.everything(), before, "{why}");
        assert_eq!(f.told(), vec![], "{why}");
    }
}

#[test]
fn lv8_c4b_classify_for_another_session_is_refused() {
    let (f, ids) = u1();
    let other =
        f.a.items()
            .create_local("LV8-OTHER", "Other", "Chair", None)
            .unwrap()
            .1
            .session
            .id;
    let before = f.everything();
    let r = f.classify_in(other, ids.k[0], "main");
    assert_eq!((r.status, r.code()), (409, "session_changed"));
    assert_eq!(f.everything(), before);
    assert_eq!(f.told(), vec![]);
}

#[test]
fn lv8_c4c_a_photo_of_another_session_is_not_found() {
    let a = TestApp::with_camera().unwrap();
    let other = build_untemplated(&a);
    let (s, _) = build_u1(&a);
    let f = Fx::start(a, s);
    let before = (f.everything(), f.everything_in(other.s));
    let r = f.classify(other.v1, "main");
    assert_eq!((r.status, r.code()), (404, "not_found"));
    assert_eq!((f.everything(), f.everything_in(other.s)), before);
    assert_eq!(f.told(), vec![]);
}
