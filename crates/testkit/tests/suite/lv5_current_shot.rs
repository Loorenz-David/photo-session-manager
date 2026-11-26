use psm_core::{
    Error,
    domain::{
        camera::{CaptureId, ExposureInitiator, ExposureObserved},
        composition::Shape,
        photo::Classification,
        template::Template,
    },
    ports::{camera::CaptureSink, clock::Clock},
    services::{
        current_shot::Current,
        phone_view::{PhoneCurrent, phone_view},
        template_placement::session_view_with_current,
        templates::TemplatesService,
    },
};
use psm_infra::imaging::ImageOpsImpl;
use psm_testkit::TestApp;
use std::{
    thread,
    time::{Duration, Instant},
};

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
        .create_local("LV5-U1", "LV5-U1", "Chair", None)
        .unwrap()
        .1
        .session
        .id;
    a.sessions().activate(s).unwrap();
    (a, s, ids)
}
fn expose(a: &TestApp, n: u32) -> i64 {
    a.ledger()
        .exposure_observed(ExposureObserved {
            capture_id: Some(CaptureId(format!("u1-{n}"))),
            at: a.clock.now_ms(),
            initiated: ExposureInitiator::Camera,
            provisional: false,
        })
        .unwrap()
}
fn file(a: &TestApp, n: u32) -> String {
    a.file_photo(&format!("DSC{n:05}.ARW")).unwrap()
}
fn arrive(a: &TestApp, n: u32) -> String {
    a.arrival_photo(0, &format!("DSC{n:05}.ARW")).unwrap()
}
fn current(a: &TestApp, s: i64) -> Current {
    a.app
        .camera_runtime
        .current_shots()
        .current(a.store(), s, a.clock.now_ms())
        .unwrap()
        .unwrap()
}
fn binding(a: &TestApp, e: i64) -> (i64, Option<i64>) {
    rusqlite::Connection::open(a.app.paths.db())
        .unwrap()
        .query_row(
            "SELECT shot_bound,shot_container_id FROM camera_exposures WHERE id=?",
            [e],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap()
}
fn photo_at(a: &TestApp, s: i64, path: &str) -> (Classification, Option<i64>) {
    let p = a
        .store()
        .photos_in_session(s)
        .unwrap()
        .into_iter()
        .find(|p| p.raw_path == path)
        .unwrap();
    (
        p.classification,
        a.store()
            .photo_place_row(p.id)
            .unwrap()
            .and_then(|r| r.slot),
    )
}
fn fixture4() -> (TestApp, i64, [i64; 4]) {
    let ready = r#"{"timestamp":"2026-11-06T10:00:00.000Z","source":"sdk","event":"connection.ready","transport":"usb","model":"ILCE-7RM5","store_destination":"verified","store_value":"0x0003"}"#;
    let a = TestApp::with_camera_script(ready).unwrap();
    let until = Instant::now() + Duration::from_secs(15);
    while Instant::now() < until {
        a.tick(0).unwrap();
        if a.camera_status().is_ok_and(|status| {
            matches!(
                status,
                psm_core::domain::camera::CameraStatus::Connected { .. }
            )
        }) {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
    assert!(a.camera_status().is_ok_and(|status| matches!(
        status,
        psm_core::domain::camera::CameraStatus::Connected { .. }
    )));
    let templates = || {
        TemplatesService::new(
            a.store(),
            a.clock.as_ref(),
            &ImageOpsImpl,
            a.app.paths.root.join("templates"),
        )
    };
    let t = templates().create("Chair").unwrap();
    let t = (0..4).fold(t, |t: Template, _| {
        templates().container_add(t.id, None).unwrap()
    });
    let ids = [
        t.containers[0].id,
        t.containers[1].id,
        t.containers[2].id,
        t.containers[3].id,
    ];
    // W X Y Z in shooting order, final order Y W Z X.
    templates()
        .shooting_order_set(t.id, &[ids[1], ids[3], ids[0], ids[2]])
        .unwrap();
    let s = a
        .items()
        .create_local("LV5-T4", "LV5-T4", "Chair", None)
        .unwrap()
        .1
        .session
        .id;
    a.sessions().activate(s).unwrap();
    (a, s, [ids[1], ids[3], ids[0], ids[2]])
}
fn at(a: &TestApp, origin: i64, offset: i64) {
    a.clock.advance(origin + offset - a.clock.now_ms());
}
fn inject_exposure(a: &TestApp, expected: usize) -> i64 {
    a.inject("capture.exposed",serde_json::json!({"success":true,"transport":"wifi","association":"none: no SDK capture pending (camera-initiated?)"})).unwrap();
    let until = Instant::now() + Duration::from_secs(15);
    while Instant::now() < until {
        a.tick(0).unwrap();
        let rows = a.ledger().exposures(None).unwrap();
        if rows.len() == expected {
            return rows.last().unwrap().id;
        }
        thread::sleep(Duration::from_millis(10));
    }
    panic!("camera exposure was not processed");
}

#[test]
fn lv5_c1a_filed_photo_holds_its_shot_until_intake() {
    let (a, s, [ca, cb, cc]) = fixture();
    expose(&a, 101);
    let pb = arrive(&a, 101);
    assert_eq!(photo_at(&a, s, &pb).1, Some(2));
    let e2 = expose(&a, 102);
    assert_eq!(binding(&a, e2), (1, Some(ca)));
    let _pending = file(&a, 102);
    let e3 = expose(&a, 103);
    assert_eq!(binding(&a, e3), (1, Some(cc)));
    assert_eq!(current(&a, s), Current::Free);
    assert_ne!(cb, ca);
}
#[test]
fn lv5_c1b_a_filed_last_container_makes_the_next_shot_free() {
    let (a, s, [ca, cb, cc]) = fixture();
    expose(&a, 101);
    arrive(&a, 101);
    expose(&a, 102);
    arrive(&a, 102);
    assert_eq!(current(&a, s), Current::Shot(cc));
    let e3 = expose(&a, 103);
    assert_eq!(binding(&a, e3), (1, Some(cc)));
    file(&a, 103);
    let e4 = expose(&a, 104);
    assert_eq!(binding(&a, e4), (1, None));
    assert_eq!(current(&a, s), Current::Free);
    assert_ne!(ca, cb);
}
#[test]
fn lv5_c1c_filed_file_that_never_arrives_stops_holding_after_window() {
    let (a, s, [_, cb, _]) = fixture();
    expose(&a, 101);
    file(&a, 101);
    a.clock.advance(301_000);
    a.app
        .camera_runtime
        .current_shots()
        .set(
            a.store(),
            a.events.as_ref(),
            s,
            Current::Shot(cb),
            a.clock.now_ms(),
        )
        .unwrap();
    assert_eq!(current(&a, s), Current::Shot(cb));
}
#[test]
fn lv5_c2a_intake_keeps_the_current_shot_free() {
    let (a, s, [ca, _, cc]) = fixture();
    expose(&a, 101);
    arrive(&a, 101);
    expose(&a, 102);
    let pending = file(&a, 102);
    expose(&a, 103);
    assert_eq!(current(&a, s), Current::Free);
    a.take_in_photo(&pending).unwrap();
    assert_eq!(photo_at(&a, s, &pending), (Classification::Main, Some(1)));
    assert_eq!(current(&a, s), Current::Free);
    assert_ne!(ca, cc);
}
#[test]
fn lv5_c2b_the_late_file_and_free_file_keep_the_current_free() {
    let (a, s, [_, _, cc]) = fixture();
    expose(&a, 101);
    arrive(&a, 101);
    expose(&a, 102);
    arrive(&a, 102);
    expose(&a, 103);
    let pending = file(&a, 103);
    let e4 = expose(&a, 104);
    assert_eq!(binding(&a, e4), (1, None));
    a.take_in_photo(&pending).unwrap();
    let free = arrive(&a, 104);
    assert_eq!(photo_at(&a, s, &pending), (Classification::Main, Some(3)));
    assert_eq!(photo_at(&a, s, &free).0, Classification::Condition);
    assert_eq!(current(&a, s), Current::Free);
    assert!(cc > 0);
}
#[test]
fn lv5_c4a_mac_and_phone_show_the_next_unfilled_shot_in_the_gap() {
    let (a, s, [_, _, cc]) = fixture();
    expose(&a, 101);
    arrive(&a, 101);
    expose(&a, 102);
    file(&a, 102);
    let view = session_view_with_current(
        a.store(),
        a.app.camera_runtime.current_shots().as_ref(),
        a.clock.now_ms(),
        s,
    )
    .unwrap();
    assert_eq!(view.next_shot, Some(3));
    let phone = phone_view(
        a.store(),
        a.app.camera_runtime.current_shots().as_ref(),
        Shape::Square,
        Some(s),
        a.clock.now_ms(),
    )
    .unwrap();
    assert_eq!(
        phone.session.unwrap().current,
        Some(PhoneCurrent::Shot { container_id: cc })
    );
}
#[test]
fn lv5_c4b_set_rejects_a_shot_with_a_filed_file() {
    let (a, s, [ca, _, cc]) = fixture();
    expose(&a, 101);
    arrive(&a, 101);
    expose(&a, 102);
    file(&a, 102);
    assert!(matches!(
        a.app.camera_runtime.current_shots().set(
            a.store(),
            a.events.as_ref(),
            s,
            Current::Shot(ca),
            a.clock.now_ms()
        ),
        Err(Error::ShotFilled)
    ));
    assert_eq!(current(&a, s), Current::Shot(cc));
}

#[test]
fn lv5_c3a_five_fast_wifi_shutters_keep_distinct_bindings_through_late_files() {
    let (a, s, [w, x, y, z]) = fixture4();
    let t = a.clock.now_ms();
    let e1 = inject_exposure(&a, 1);
    at(&a, t, 1200);
    let e2 = inject_exposure(&a, 2);
    at(&a, t, 2400);
    let e3 = inject_exposure(&a, 3);
    at(&a, t, 3600);
    let e4 = inject_exposure(&a, 4);
    at(&a, t, 4000);
    let p1 = file(&a, 101);
    at(&a, t, 4800);
    let e5 = inject_exposure(&a, 5);
    assert_eq!(
        [
            binding(&a, e1),
            binding(&a, e2),
            binding(&a, e3),
            binding(&a, e4),
            binding(&a, e5)
        ],
        [
            (1, Some(w)),
            (1, Some(x)),
            (1, Some(y)),
            (1, Some(z)),
            (1, None)
        ]
    );
    at(&a, t, 5000);
    a.take_in_photo(&p1).unwrap();
    at(&a, t, 6000);
    let p3 = file(&a, 103);
    let p2 = file(&a, 102);
    at(&a, t, 7000);
    a.take_in_photo(&p3).unwrap();
    a.take_in_photo(&p2).unwrap();
    at(&a, t, 8000);
    let p4 = file(&a, 104);
    at(&a, t, 8800);
    let p5 = file(&a, 105);
    a.take_in_photo(&p4).unwrap();
    a.take_in_photo(&p5).unwrap();
    assert_eq!(
        [
            photo_at(&a, s, &p1).1,
            photo_at(&a, s, &p2).1,
            photo_at(&a, s, &p3).1,
            photo_at(&a, s, &p4).1
        ],
        [Some(2), Some(4), Some(1), Some(3)]
    );
    assert_eq!(photo_at(&a, s, &p5).0, Classification::Condition);
}

#[test]
fn lv5_c3b_a_phone_tap_and_out_of_order_files_keep_each_shutters_shot() {
    let (a, s, [w, x, y, z]) = fixture4();
    let t = a.clock.now_ms();
    let e1 = inject_exposure(&a, 1);
    at(&a, t, 500);
    a.app
        .camera_runtime
        .current_shots()
        .set(
            a.store(),
            a.events.as_ref(),
            s,
            Current::Shot(z),
            a.clock.now_ms(),
        )
        .unwrap();
    at(&a, t, 1200);
    let e2 = inject_exposure(&a, 2);
    at(&a, t, 2400);
    let e3 = inject_exposure(&a, 3);
    at(&a, t, 3600);
    let e4 = inject_exposure(&a, 4);
    at(&a, t, 4000);
    let p2 = file(&a, 102);
    at(&a, t, 4400);
    let p1 = file(&a, 101);
    at(&a, t, 4800);
    let e5 = inject_exposure(&a, 5);
    assert_eq!(
        [
            binding(&a, e1),
            binding(&a, e2),
            binding(&a, e3),
            binding(&a, e4),
            binding(&a, e5)
        ],
        [
            (1, Some(w)),
            (1, Some(z)),
            (1, Some(x)),
            (1, Some(y)),
            (1, None)
        ]
    );
    at(&a, t, 5000);
    a.take_in_photo(&p2).unwrap();
    a.take_in_photo(&p1).unwrap();
    at(&a, t, 7000);
    let p4 = file(&a, 104);
    let p3 = file(&a, 103);
    at(&a, t, 8000);
    a.take_in_photo(&p4).unwrap();
    a.take_in_photo(&p3).unwrap();
    at(&a, t, 8800);
    let p5 = file(&a, 105);
    a.take_in_photo(&p5).unwrap();
    assert_eq!(
        [
            photo_at(&a, s, &p1).1,
            photo_at(&a, s, &p2).1,
            photo_at(&a, s, &p3).1,
            photo_at(&a, s, &p4).1
        ],
        [Some(2), Some(3), Some(4), Some(1)]
    );
    assert_eq!(photo_at(&a, s, &p5).0, Classification::Condition);
}
