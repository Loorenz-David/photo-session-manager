//! LV2·U2 — the phone's operations (`master_plan_live.md` §5 PH6, §6; card LV8): rows LV2·C9–C12(a)
//! at the wired TestApp. Fixture T3 of `lv2_current_shot.rs` (A final 1 shoot 2, 1:1 zoom 20; B
//! final 2 shoot 1, split; C final 3 shoot 3) plus untemplated sessions. LV2·C12(b) is the unit row
//! in `mechanisms::template`.
use psm_core::{
    domain::{
        camera::{CaptureId, ExposureInitiator, ExposureObserved},
        composition::Shape,
        photo::Classification as C,
        template::Presets,
    },
    events::AppEvent,
    ports::{camera::CaptureSink, clock::Clock},
    services::{
        current_shot::Current,
        phone_view::{PhoneCounts, PhoneCurrent, PhoneSession, phone_view},
        template_placement::{self, HandAction, MoveTarget},
        templates::TemplatesService,
    },
};
use psm_infra::imaging::ImageOpsImpl;
use psm_testkit::TestApp;
use serde_json::json;

fn app() -> TestApp {
    TestApp::with_camera().unwrap()
}
fn templates(a: &TestApp) -> TemplatesService<'_> {
    TemplatesService::new(
        a.store(),
        a.clock.as_ref(),
        &ImageOpsImpl,
        a.app.paths.root.join("templates"),
    )
}
/// T3; the ids of A, B, C.
fn t3(a: &TestApp) -> [i64; 3] {
    let t = templates(a).create("Chair").unwrap();
    let t = (0..3).fold(t, |t, _| templates(a).container_add(t.id, None).unwrap());
    let ids = [t.containers[0].id, t.containers[1].id, t.containers[2].id];
    let [ca, cb, cc] = ids;
    templates(a)
        .shooting_order_set(t.id, &[cb, ca, cc])
        .unwrap();
    templates(a)
        .container_presets_set(
            ca,
            Presets {
                split: false,
                shape: Some(Shape::Square),
                zoom: 20,
            },
        )
        .unwrap();
    templates(a)
        .container_presets_set(
            cb,
            Presets {
                split: true,
                shape: None,
                zoom: 0,
            },
        )
        .unwrap();
    ids
}
fn session_of(a: &TestApp, sku: &str, title: &str, kind: &str, barcode: Option<&str>) -> i64 {
    let s = a
        .items()
        .create_local(sku, title, kind, barcode)
        .unwrap()
        .1
        .session
        .id;
    a.sessions().activate(s).unwrap();
    s
}
fn session(a: &TestApp, sku: &str, kind: &str) -> i64 {
    session_of(a, sku, sku, kind, None)
}
fn view(a: &TestApp) -> Option<PhoneSession> {
    phone_view(
        a.store(),
        a.app.camera_runtime.current_shots().as_ref(),
        Shape::Square,
        a.store().active_session_id().unwrap(),
        a.clock.now_ms(),
    )
    .unwrap()
    .session
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
/// (classification, position, slot)
fn place(a: &TestApp, id: i64) -> (C, Option<i64>, Option<i64>) {
    let p = a.store().photo(id).unwrap();
    let slot = a.store().photo_place_row(id).unwrap().and_then(|r| r.slot);
    (p.classification, p.position, slot)
}
fn by_hand(a: &TestApp, id: i64) -> bool {
    a.store()
        .photo_place_row(id)
        .unwrap()
        .is_some_and(|r| r.place_by_hand)
}
fn hand(a: &TestApp) -> HandAction<'_> {
    HandAction {
        store: a.store(),
        clock: a.clock.as_ref(),
        events: a.events.as_ref(),
        history: a.app.history.as_ref(),
        shots: a.app.camera_runtime.current_shots(),
    }
}
fn move_to(a: &TestApp, id: i64, target: MoveTarget) {
    template_placement::move_to(&hand(a), id, target).unwrap();
}
/// The photo ids of a group in order.
fn group(a: &TestApp, s: i64, class: C) -> Vec<i64> {
    let mut v: Vec<_> = a
        .store()
        .photos_in_session(s)
        .unwrap()
        .into_iter()
        .filter(|p| p.classification == class)
        .collect();
    v.sort_by_key(|p| p.position);
    v.into_iter().map(|p| p.id).collect()
}
/// A T3 session with P in A and Q in C (B empty).
fn p_in_a_q_in_c(a: &TestApp, sku: &str) -> (i64, i64, i64) {
    let [ca, _, cc] = t3(a);
    let s = session(a, sku, "Chair");
    set(a, s, Current::Shot(ca));
    let p = arrive(a, s, 100);
    set(a, s, Current::Shot(cc));
    let q = arrive(a, s, 101);
    assert_eq!((place(a, p).2, place(a, q).2), (Some(1), Some(3)));
    (s, p, q)
}

// ---- LV2·C9: the phone's session view ----

#[test]
fn lv2_c9a_the_view_lists_the_shots_in_shooting_order_with_the_current_one() {
    let a = app();
    let [ca, cb, cc] = t3(&a);
    let s = session_of(&a, "BEY-123", "Oak Lounge Chair", "Chair", None);
    let pb = arrive(&a, s, 100);
    set(&a, s, Current::Free);
    let q = arrive(&a, s, 101);
    let v = view(&a).unwrap();
    assert_eq!(v.id, s);
    assert_eq!(
        (v.item.name.as_str(), v.item.sku.as_str()),
        ("Oak Lounge Chair", "BEY-123")
    );
    assert!(v.templated);
    assert_eq!(v.current, Some(PhoneCurrent::Shot { container_id: ca }));
    assert_eq!(
        serde_json::to_value(v.current).unwrap(),
        json!({"kind": "shot", "container_id": ca})
    );
    let shots: Vec<_> = v
        .shots
        .iter()
        .map(|s| {
            (
                s.container_id,
                s.final_no,
                s.ratio,
                s.split,
                s.zoom,
                s.photo.as_ref().map(|p| p.photo_id),
            )
        })
        .collect();
    assert_eq!(
        shots,
        vec![
            (cb, 2, Some(2.0), true, 0, Some(pb)),
            (ca, 1, Some(1.0), false, 20, None),
            (cc, 3, Some(1.0), false, 0, None),
        ]
    );
    assert!(v.extras.is_empty());
    assert_eq!(
        v.condition.iter().map(|i| i.photo_id).collect::<Vec<_>>(),
        vec![q]
    );
    assert_eq!(
        v.counts,
        PhoneCounts {
            main: 1,
            condition: 1
        }
    );
}

#[test]
fn lv2_c9b_the_article_number_is_the_items_barcode() {
    let a = app();
    session_of(&a, "BEY-200", "Chair", "Chair", Some("7350001234567"));
    assert_eq!(
        view(&a).unwrap().item.article_number.as_deref(),
        Some("7350001234567")
    );
    session_of(&a, "BEY-201", "Chair", "Chair", None);
    assert_eq!(view(&a).unwrap().item.article_number, None);
}

#[test]
fn lv2_c9c_an_untemplated_session_lists_main_as_extras() {
    let a = app();
    let s = session(&a, "LV2-C9C", "Table");
    let p1 = arrive(&a, s, 100);
    let p2 = arrive(&a, s, 101);
    let q = arrive(&a, s, 102);
    a.review().classify(q, C::Condition).unwrap();
    let v = view(&a).unwrap();
    assert!(!v.templated);
    assert!(v.shots.is_empty());
    assert_eq!(
        v.extras.iter().map(|i| i.photo_id).collect::<Vec<_>>(),
        vec![p1, p2]
    );
    assert_eq!(
        v.condition.iter().map(|i| i.photo_id).collect::<Vec<_>>(),
        vec![q]
    );
    assert_eq!(v.current, None);
}

#[test]
fn lv2_c9d_no_active_session_is_null() {
    let a = app();
    assert_eq!(a.store().active_session_id().unwrap(), None);
    assert_eq!(view(&a), None);
}

// ---- LV2·C10: Move (swap) ----

#[test]
fn lv2_c10a_move_onto_a_filled_container_swaps_both_by_hand_in_one_undo_step() {
    let a = app();
    let (s, p, q) = p_in_a_q_in_c(&a, "LV2-C10A");
    move_to(&a, p, MoveTarget::Container(3));
    assert_eq!((place(&a, p).2, place(&a, q).2), (Some(3), Some(1)));
    assert!(by_hand(&a, p) && by_hand(&a, q));
    assert_eq!(a.history().state(s).undo.as_deref(), Some("Move image"));
    a.history().undo(s).unwrap();
    assert_eq!((place(&a, p).2, place(&a, q).2), (Some(1), Some(3)));
    assert!(!by_hand(&a, p) && !by_hand(&a, q));
}

#[test]
fn lv2_c10b_move_onto_an_empty_container_moves() {
    let a = app();
    let [ca, _, _] = t3(&a);
    let s = session(&a, "LV2-C10B", "Chair");
    set(&a, s, Current::Shot(ca));
    let p = arrive(&a, s, 100);
    move_to(&a, p, MoveTarget::Container(3));
    assert_eq!(place(&a, p), (C::Main, Some(1), Some(3)));
    assert!(
        !group(&a, s, C::Main)
            .iter()
            .any(|id| place(&a, *id).2 == Some(1))
    );
}

#[test]
fn lv2_c10c_move_to_a_new_extra_appends_main_after_the_containers() {
    let a = app();
    let (s, p, q) = p_in_a_q_in_c(&a, "LV2-C10C");
    move_to(&a, p, MoveTarget::ExtraEnd);
    assert_eq!(place(&a, p), (C::Main, Some(2), None));
    assert_eq!(group(&a, s, C::Main), vec![q, p]);
    assert!(
        !group(&a, s, C::Main)
            .iter()
            .any(|id| place(&a, *id).2 == Some(1))
    );
}

#[test]
fn lv2_c10d_move_within_condition_swaps() {
    let a = app();
    t3(&a);
    let s = session(&a, "LV2-C10D", "Chair");
    set(&a, s, Current::Free);
    let cs = arrive(&a, s, 100);
    set(&a, s, Current::Free);
    let r = arrive(&a, s, 101);
    assert_eq!(group(&a, s, C::Condition), vec![cs, r]);
    move_to(&a, r, MoveTarget::Condition(1));
    assert_eq!(group(&a, s, C::Condition), vec![r, cs]);
}

#[test]
fn lv2_c10e_move_main_onto_a_condition_photo_swaps_places() {
    let a = app();
    let [ca, _, _] = t3(&a);
    let s = session(&a, "LV2-C10E", "Chair");
    set(&a, s, Current::Shot(ca));
    let p = arrive(&a, s, 100);
    set(&a, s, Current::Free);
    let cs = arrive(&a, s, 101);
    move_to(&a, p, MoveTarget::Condition(1));
    assert_eq!(place(&a, cs), (C::Main, Some(1), Some(1)));
    assert_eq!((place(&a, p).0, place(&a, p).1), (C::Condition, Some(1)));
}

#[test]
fn lv2_c10f_move_to_the_condition_end_appends() {
    let a = app();
    let [ca, _, _] = t3(&a);
    let s = session(&a, "LV2-C10F", "Chair");
    set(&a, s, Current::Shot(ca));
    let p = arrive(&a, s, 100);
    set(&a, s, Current::Free);
    let cs = arrive(&a, s, 101);
    move_to(&a, p, MoveTarget::ConditionEnd);
    assert_eq!(group(&a, s, C::Condition), vec![cs, p]);
    assert!(group(&a, s, C::Main).is_empty());
}

#[test]
fn lv2_c10g_untemplated_main_positions_swap() {
    let a = app();
    let s = session(&a, "LV2-C10G", "Table");
    let p1 = arrive(&a, s, 100);
    let p2 = arrive(&a, s, 101);
    let p3 = arrive(&a, s, 102);
    move_to(&a, p1, MoveTarget::MainPosition(3));
    assert_eq!(group(&a, s, C::Main), vec![p3, p2, p1]);
}

// ---- LV2·C11: Delete from the phone = Deny ----

#[test]
fn lv2_c11a_phone_delete_denies_and_undo_restores_the_place() {
    let a = app();
    let s = session(&a, "LV2-C11A", "Table");
    let p1 = arrive(&a, s, 100);
    let extra = arrive(&a, s, 101);
    let q1 = arrive(&a, s, 102);
    let q = arrive(&a, s, 103);
    a.review().classify(q1, C::Condition).unwrap();
    a.review().classify(q, C::Condition).unwrap();
    let before = (place(&a, extra), place(&a, q));
    a.review().delete(extra).unwrap();
    a.review().delete(q).unwrap();
    for id in [extra, q] {
        assert_eq!(place(&a, id).0, C::Deleted);
        assert!(a.store().photo_denial(id).unwrap().is_some());
    }
    a.history().undo(s).unwrap();
    a.history().undo(s).unwrap();
    assert_eq!((place(&a, extra), place(&a, q)), before);
    assert!(a.store().photo_denial(extra).unwrap().is_none());
    assert_eq!(group(&a, s, C::Main), vec![p1, extra]);
}

// ---- LV2·C12(a): every change of the current shot changes the session ----

#[test]
fn lv2_c12a_set_exposure_and_retake_each_emit_one_session_change() {
    let a = app();
    let [ca, _, cc] = t3(&a);
    let s = session(&a, "LV2-C12A", "Chair");
    arrive(&a, s, 100);
    arrive(&a, s, 101);
    a.clock.advance(301_000);
    a.events.0.lock().unwrap().clear();
    let changes = |a: &TestApp| {
        a.events
            .0
            .lock()
            .unwrap()
            .iter()
            .filter_map(|e| match e {
                AppEvent::SessionChanged { session_id } => Some(*session_id),
                _ => None,
            })
            .collect::<Vec<_>>()
    };
    set(&a, s, Current::Shot(cc));
    assert_eq!(changes(&a), vec![s]);
    a.ledger()
        .exposure_observed(ExposureObserved {
            capture_id: Some(CaptureId("c12a".into())),
            at: a.clock.now_ms(),
            initiated: ExposureInitiator::Camera,
            provisional: false,
        })
        .unwrap();
    assert_eq!(changes(&a), vec![s, s]);
    template_placement::retake(&hand(&a), s, ca).unwrap();
    assert_eq!(changes(&a), vec![s, s, s]);
}
