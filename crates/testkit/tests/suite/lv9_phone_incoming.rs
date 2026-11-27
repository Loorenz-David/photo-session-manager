//! LV9·U1 — what is on its way, as the phone is told (`master_plan_live.md` §5 PH6b; book 10J
//! LV-D21 (1)): rows LV9·C1–C3 at `phone_view`, C3(a) through `TestPhone`.
//!
//! Fixture (as `lv5_current_shot.rs`): template "Chair" with containers A final 1 shoot 2, B final
//! 2 shoot 1, C final 3 shoot 3; templated session S active; the current shot is B.
use psm_core::{
    domain::{
        camera::{CaptureId, ExposureInitiator, ExposureObserved},
        composition::Shape,
        template::Template,
    },
    ports::{camera::CaptureSink, clock::Clock},
    services::{
        current_shot::Current,
        phone_view::{PhoneCurrent, PhoneIncoming, PhoneSession, phone_view},
        templates::TemplatesService,
    },
};
use psm_infra::imaging::ImageOpsImpl;
use psm_testkit::{TestApp, phone::TestPhone};
use serde_json::{Value, json};

/// (app, session S, [A, B, C])
fn fixture() -> (TestApp, i64, [i64; 3]) {
    let a = TestApp::with_camera().unwrap();
    let templates = || {
        TemplatesService::new(
            a.store(),
            a.clock.as_ref(),
            &ImageOpsImpl,
            a.app.paths.root.join("templates"),
        )
    };
    let t = templates().create("Chair").unwrap();
    let t = (0..3).fold(t, |t: Template, _| {
        templates().container_add(t.id, None).unwrap()
    });
    let ids = [t.containers[0].id, t.containers[1].id, t.containers[2].id];
    templates()
        .shooting_order_set(t.id, &[ids[1], ids[0], ids[2]])
        .unwrap();
    let s = a
        .items()
        .create_local("LV9-U1", "LV9-U1", "Chair", None)
        .unwrap()
        .1
        .session
        .id;
    a.sessions().activate(s).unwrap();
    (a, s, ids)
}
/// The camera reports a shutter.
fn expose(a: &TestApp, n: u32) -> i64 {
    a.ledger()
        .exposure_observed(ExposureObserved {
            capture_id: Some(CaptureId(format!("lv9-{n}"))),
            at: a.clock.now_ms(),
            initiated: ExposureInitiator::Camera,
            provisional: false,
        })
        .unwrap()
}
/// The shutter's file is filed, not yet taken in.
fn file(a: &TestApp, n: u32) -> String {
    a.file_photo(&format!("DSC{n:05}.ARW")).unwrap()
}
/// The shutter's file is filed and taken in: its photo exists.
fn arrive(a: &TestApp, n: u32) -> String {
    a.arrival_photo(0, &format!("DSC{n:05}.ARW")).unwrap()
}
fn photo_id(a: &TestApp, s: i64, path: &str) -> i64 {
    a.store()
        .photos_in_session(s)
        .unwrap()
        .into_iter()
        .find(|p| p.raw_path == path)
        .unwrap()
        .id
}
fn view(a: &TestApp, s: i64) -> PhoneSession {
    phone_view(
        a.store(),
        a.app.camera_runtime.current_shots().as_ref(),
        Shape::Square,
        Some(s),
        a.clock.now_ms(),
    )
    .unwrap()
    .session
    .unwrap()
}
/// (photo id, incoming) of the shot of `container`.
fn shot(v: &PhoneSession, container: i64) -> (Option<i64>, bool) {
    let shot = v
        .shots
        .iter()
        .find(|shot| shot.container_id == container)
        .unwrap();
    (shot.photo.as_ref().map(|p| p.photo_id), shot.incoming)
}
const NONE: PhoneIncoming = PhoneIncoming {
    extras: 0,
    condition: 0,
};

#[test]
fn lv9_c1a_a_reported_shutter_marks_its_shot_incoming() {
    let (a, s, [ca, cb, cc]) = fixture();
    expose(&a, 101);
    let v = view(&a, s);
    assert_eq!(shot(&v, cb), (None, true));
    assert_eq!(shot(&v, ca), (None, false));
    assert_eq!(shot(&v, cc), (None, false));
    assert_eq!(v.incoming, NONE);
    assert_eq!(v.current, Some(PhoneCurrent::Shot { container_id: ca }));
}

#[test]
fn lv9_c1b_a_filed_file_is_still_on_its_way() {
    let (a, s, [_, cb, _]) = fixture();
    expose(&a, 101);
    file(&a, 101);
    assert_eq!(shot(&view(&a, s), cb), (None, true));
}

#[test]
fn lv9_c1c_the_photo_ends_incoming() {
    let (a, s, [_, cb, _]) = fixture();
    expose(&a, 101);
    let path = arrive(&a, 101);
    assert_eq!(
        shot(&view(&a, s), cb),
        (Some(photo_id(&a, s, &path)), false)
    );
}

#[test]
fn lv9_c2a_a_free_shot_on_its_way_counts_for_condition() {
    let (a, s, [ca, cb, cc]) = fixture();
    a.app
        .camera_runtime
        .current_shots()
        .set(
            a.store(),
            a.events.as_ref(),
            s,
            Current::Free,
            a.clock.now_ms(),
        )
        .unwrap();
    expose(&a, 102);
    let v = view(&a, s);
    assert_eq!(
        v.incoming,
        PhoneIncoming {
            extras: 0,
            condition: 1
        }
    );
    assert_eq!(
        [shot(&v, ca), shot(&v, cb), shot(&v, cc)],
        [(None, false); 3]
    );
    let path = arrive(&a, 102);
    let v = view(&a, s);
    assert_eq!(v.incoming, NONE);
    assert_eq!(
        v.condition.iter().map(|p| p.photo_id).collect::<Vec<_>>(),
        [photo_id(&a, s, &path)]
    );
}

#[test]
fn lv9_c2b_an_untemplated_shutter_counts_for_main() {
    let a = TestApp::with_camera().unwrap();
    let u = a
        .items()
        .create_local("LV9-U", "LV9-U", "Table", None)
        .unwrap()
        .1
        .session
        .id;
    a.sessions().activate(u).unwrap();
    expose(&a, 103);
    let v = view(&a, u);
    assert_eq!(
        v.incoming,
        PhoneIncoming {
            extras: 1,
            condition: 0
        }
    );
    assert!(v.shots.is_empty());
    let path = arrive(&a, 103);
    let v = view(&a, u);
    assert_eq!(v.incoming, NONE);
    assert_eq!(
        v.extras.iter().map(|p| p.photo_id).collect::<Vec<_>>(),
        [photo_id(&a, u, &path)]
    );
}

#[test]
fn lv9_c2c_a_file_that_never_arrives_stops_being_incoming() {
    let (a, s, [_, cb, _]) = fixture();
    expose(&a, 104);
    file(&a, 104);
    a.clock.advance(301_000);
    let v = view(&a, s);
    assert_eq!(shot(&v, cb), (None, false));
    assert_eq!(v.incoming, NONE);
}

#[test]
fn lv9_c3a_the_session_route_carries_incoming() {
    let (a, _, _) = fixture();
    expose(&a, 101);
    let phone = TestPhone::start(&a);
    let token = phone.pair(&a);
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let body: Value = runtime.block_on(async {
        let response = reqwest::Client::new()
            .get(phone.url("/api/phone/v1/session"))
            .bearer_auth(&token)
            .send()
            .await
            .unwrap();
        assert_eq!(response.status().as_u16(), 200);
        response.json().await.unwrap()
    });
    // B is first in shooting order, A second.
    assert_eq!(body["session"]["shots"][0]["incoming"], json!(true));
    assert_eq!(body["session"]["shots"][1]["incoming"], json!(false));
    assert_eq!(
        body["session"]["incoming"],
        json!({"extras": 0, "condition": 0})
    );
}

#[test]
fn lv9_c3b_a_denied_photo_is_not_on_its_way() {
    let (a, s, [_, cb, _]) = fixture();
    expose(&a, 101);
    let path = arrive(&a, 101);
    a.review().delete(photo_id(&a, s, &path)).unwrap();
    assert_eq!(shot(&view(&a, s), cb), (None, false));
}
