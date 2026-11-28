use psm_core::{
    domain::{
        camera::{CaptureId, ExposureInitiator, ExposureObserved, FIFO_WINDOW_MS},
        composition::{
            CompositionKind, CompositionState, EditorGeometry, Fingerprint, Flip, Frame,
            FrameSource, Orientation, Origin, Rect, Shape,
        },
        photo::{Classification as C, Photo},
        template::{Presets, Template},
    },
    events::AppEvent,
    ports::{camera::CaptureSink, clock::Clock},
    services::{template_placement::session_view, templates::TemplatesService},
};
use psm_infra::imaging::ImageOpsImpl;
use psm_testkit::TestApp;
use std::{
    fs,
    time::{Duration, UNIX_EPOCH},
};

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
fn session(a: &TestApp, sku: &str, kind: &str) -> i64 {
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
fn chair(a: &TestApp) -> Template {
    let t = templates(a).create("Chair").unwrap();
    let t = templates(a).container_add(t.id, None).unwrap();
    let t = templates(a).container_add(t.id, None).unwrap();
    let t = templates(a).container_add(t.id, None).unwrap();
    let ids: Vec<_> = t.containers.iter().map(|c| c.id).collect();
    let t = templates(a)
        .shooting_order_set(t.id, &[ids[1], ids[0], ids[2]])
        .unwrap();
    templates(a)
        .container_presets_set(
            ids[2],
            Presets {
                split: true,
                shape: None,
                zoom: 20,
            },
        )
        .unwrap();
    t
}
fn arrive(a: &TestApp, s: i64, name: &str) -> Photo {
    let path = a.arrival_photo(s, name).unwrap();
    a.store()
        .photos_in_session(s)
        .unwrap()
        .into_iter()
        .find(|p| p.raw_path == path)
        .unwrap()
}
fn photo(a: &TestApp, s: i64, name: &str) -> Photo {
    a.store()
        .photos_in_session(s)
        .unwrap()
        .into_iter()
        .find(|p| p.raw_path.ends_with(name))
        .unwrap()
}
fn place(a: &TestApp, id: i64) -> (C, Option<i64>, Option<i64>, bool) {
    let p = a.store().photo(id).unwrap();
    let r = a.store().photo_place_row(id).unwrap();
    (
        p.classification,
        p.position,
        r.as_ref().and_then(|r| r.slot),
        r.is_some_and(|r| r.place_by_hand),
    )
}
fn expose(a: &TestApp, at: i64, n: i64) {
    let now = a.clock.now_ms();
    a.clock.advance(at - now);
    a.ledger()
        .exposure_observed(ExposureObserved {
            capture_id: Some(CaptureId(format!("shot-{n}"))),
            at,
            initiated: ExposureInitiator::App,
            provisional: false,
        })
        .unwrap();
}
fn frame(a: &TestApp, p: &Photo) {
    a.store()
        .adopt_frame(
            &p.raw_path,
            Fingerprint {
                frame: Frame { w: 6000, h: 4000 },
                source: FrameSource::Camera,
                geometry: EditorGeometry::default(),
            },
        )
        .unwrap();
}
fn rect_eq(a: Rect, b: Rect) {
    for (x, y) in [(a.x, b.x), (a.y, b.y), (a.w, b.w), (a.h, b.h)] {
        assert!((x - y).abs() < 1e-9, "{x} != {y}");
    }
}

#[test]
fn t2_c3a_shooting_order_files_final_order() {
    let a = app();
    chair(&a);
    let s = session(&a, "C3A", "Chair");
    let f1 = arrive(&a, s, "DSC00001.ARW");
    let f2 = arrive(&a, s, "DSC00002.ARW");
    let f3 = arrive(&a, s, "DSC00003.ARW");
    assert_eq!(place(&a, f1.id), (C::Main, Some(2), Some(2), false));
    assert_eq!(place(&a, f2.id), (C::Main, Some(1), Some(1), false));
    assert_eq!(place(&a, f3.id), (C::Main, Some(3), Some(3), false));
}
#[test]
fn t2_c3b_extras_become_condition() {
    let a = app();
    chair(&a);
    let s = session(&a, "C3B", "Chair");
    for n in 1..=3 {
        arrive(&a, s, &format!("DSC{n:05}.ARW"));
    }
    let f4 = arrive(&a, s, "DSC00004.ARW");
    let f5 = arrive(&a, s, "DSC00005.ARW");
    assert_eq!(place(&a, f4.id), (C::Condition, Some(1), None, false));
    assert_eq!(place(&a, f5.id), (C::Condition, Some(2), None, false));
    assert_eq!(place(&a, photo(&a, s, "DSC00002.ARW").id).1, Some(1));
}
#[test]
fn t2_c3c_other_type_untouched() {
    let a = app();
    chair(&a);
    let s = session(&a, "C3C", "Table");
    let ids: Vec<_> = (1..=3)
        .map(|n| arrive(&a, s, &format!("DSC{n:05}.ARW")).id)
        .collect();
    for (i, id) in ids.iter().enumerate() {
        assert_eq!(place(&a, *id), (C::Main, Some(i as i64 + 1), None, false));
        assert!(a.store().photo_place_row(*id).unwrap().is_none());
    }
    assert!(!session_view(a.store(), s).unwrap().templated);
}
#[test]
fn t2_c3d_empty_template_untouched() {
    let a = app();
    templates(&a).create("Table").unwrap();
    let s = session(&a, "C3D", "Table");
    for n in 1..=3 {
        let p = arrive(&a, s, &format!("DSC{n:05}.ARW"));
        assert_eq!(p.position, Some(n));
        assert!(a.store().photo_place_row(p.id).unwrap().is_none());
    }
    assert!(!session_view(a.store(), s).unwrap().templated);
}
#[test]
fn t2_c3e_type_key_matches() {
    let a = app();
    chair(&a);
    let s = session(&a, "C3E", "chair ");
    let p = arrive(&a, s, "DSC00001.ARW");
    assert_eq!(place(&a, p.id).2, Some(2));
}
#[test]
fn t2_c3f_arrival_has_no_history() {
    let a = app();
    chair(&a);
    let s = session(&a, "C3F", "Chair");
    for n in 1..=5 {
        arrive(&a, s, &format!("DSC{n:05}.ARW"));
    }
    let h = a.history().state(s);
    assert!(h.undo.is_none() && h.redo.is_none());
}
#[test]
fn t2_c4a_exposure_hold_keeps_gap() {
    let a = app();
    chair(&a);
    let s = session(&a, "C4A", "Chair");
    for n in 1..=3 {
        expose(&a, n * 2000, n);
    }
    let f1 = arrive(&a, s, "DSC00001.ARW");
    let f3 = arrive(&a, s, "DSC00003.ARW");
    assert_eq!(place(&a, f3.id).2, Some(3));
    assert!(
        !session_view(a.store(), s)
            .unwrap()
            .photos
            .iter()
            .any(|p| p.slot == Some(1))
    );
    assert_eq!(place(&a, f1.id).1, Some(1));
    assert_eq!(place(&a, f3.id).1, Some(2));
    let f2 = arrive(&a, s, "DSC00002.ARW");
    assert_eq!(place(&a, f2.id).2, Some(1));
    assert_eq!(
        (place(&a, f2.id).1, place(&a, f1.id).1, place(&a, f3.id).1),
        (Some(1), Some(2), Some(3))
    );
}
/// Rewritten for 10J MC-LV3 (was 10G §4 rules 1/5 by file counter without a shutter report): a
/// late file with no shutter report takes the shot current when it arrives (CS6, P2); the photos
/// already placed stay; the session still changes.
#[test]
fn t2_c4b_late_file_without_report_takes_the_current_shot_and_emits() {
    let a = app();
    chair(&a);
    let s = session(&a, "C4B", "Chair");
    arrive(&a, s, "DSC00001.ARW");
    let f3 = arrive(&a, s, "DSC00003.ARW");
    assert_eq!(place(&a, f3.id).2, Some(1));
    a.events.0.lock().unwrap().clear();
    let f2 = arrive(&a, s, "DSC00002.ARW");
    assert_eq!(place(&a, f2.id).2, Some(3));
    assert_eq!(place(&a, f3.id).2, Some(1));
    let ev = a.events.0.lock().unwrap();
    assert!(
        !ev.iter()
            .any(|e| matches!(e,AppEvent::PhotoChanged{photo_id,..} if *photo_id==f3.id))
    );
    assert!(
        ev.iter()
            .any(|e| matches!(e,AppEvent::SessionChanged{session_id} if *session_id==s))
    );
}
#[test]
fn t2_c4c_expired_hold_closes_gap() {
    let a = app();
    chair(&a);
    let s = session(&a, "C4C", "Chair");
    for n in 1..=3 {
        expose(&a, n * 2000, n);
    }
    arrive(&a, s, "DSC00001.ARW");
    let f3 = arrive(&a, s, "DSC00003.ARW");
    assert_eq!(place(&a, f3.id).2, Some(3));
    a.clock.advance(FIFO_WINDOW_MS + 1000);
    // 10J MC-LV3: the three shutters were bound (B, A, C), so the current shot is none; the hold
    // on C expires, file 3 takes the second slot (A), and file 4, with no report, is Condition.
    let f4 = arrive(&a, s, "DSC00004.ARW");
    assert_eq!(place(&a, f3.id).2, Some(1));
    assert_eq!(place(&a, f4.id).0, C::Condition);
    assert!(
        !session_view(a.store(), s)
            .unwrap()
            .photos
            .iter()
            .any(|p| p.slot == Some(3))
    );
}
#[test]
fn t2_c4d_existing_photo_counts_but_stays() {
    let a = app();
    let s = session(&a, "C4D", "Stool");
    let g1 = arrive(&a, s, "DSC00001.ARW");
    let t = templates(&a).create("Stool").unwrap();
    let t = templates(&a).container_add(t.id, None).unwrap();
    templates(&a).container_add(t.id, None).unwrap();
    // 10J MC-LV3 (was: g1 counted as shot 1): g1 is unbound and in no container, so the
    // current shot is the first container; g1 stays where it was, after the containers.
    let g2 = arrive(&a, s, "DSC00002.ARW");
    assert_eq!(place(&a, g2.id), (C::Main, Some(1), Some(1), false));
    assert_eq!(place(&a, g1.id), (C::Main, Some(2), None, false));
    assert!(a.store().photo_place_row(g1.id).unwrap().is_none());
}
#[test]
fn t2_c4e_tether_uses_file_time() {
    let a = app();
    chair(&a);
    let s = session(&a, "C4E", "Chair");
    let folder = a.capture_root().join(
        a.store()
            .item(a.store().session(s).unwrap().session.item_id.unwrap())
            .unwrap()
            .folder_key,
    );
    fs::create_dir_all(&folder).unwrap();
    let m = 1_700_000_000_000i64;
    let b = folder.join("b.ARW");
    let first = folder.join("a.ARW");
    for (path, at) in [(&b, m + 1000), (&first, m)] {
        fs::write(path, b"raw").unwrap();
        fs::File::options()
            .write(true)
            .open(path)
            .unwrap()
            .set_times(
                fs::FileTimes::new().set_modified(UNIX_EPOCH + Duration::from_millis(at as u64)),
            )
            .unwrap();
    }
    a.app.arrival.watcher_event(&b).unwrap();
    a.tick(1000).unwrap();
    let bp = photo(&a, s, "b.ARW");
    assert_eq!(place(&a, bp.id).2, Some(2));
    a.app.arrival.watcher_event(&first).unwrap();
    a.tick(1000).unwrap();
    // 10J MC-LV3 (was: the earlier file time took container 2): each tether file takes the shot
    // current when it arrives; the earlier file time moves nothing.
    let ap = photo(&a, s, "a.ARW");
    assert_eq!(place(&a, ap.id).2, Some(1));
    assert_eq!(place(&a, bp.id).2, Some(2));
}
#[test]
fn t2_c5a_preset_materialises_on_first_frame() {
    let a = app();
    chair(&a);
    let s = session(&a, "C5A", "Chair");
    let f1 = arrive(&a, s, "DSC00001.ARW");
    arrive(&a, s, "DSC00002.ARW");
    let f3 = arrive(&a, s, "DSC00003.ARW");
    frame(&a, &f3);
    frame(&a, &f1);
    let c = a.composition(f3.id).unwrap();
    assert_eq!(
        (c.kind, c.origin, c.state),
        (CompositionKind::Split, Origin::User, CompositionState::Ok)
    );
    rect_eq(
        c.rect.unwrap(),
        Rect {
            x: 0.1,
            y: 0.2,
            w: 0.8,
            h: 0.6,
        },
    );
    let normal = a.composition(f1.id).unwrap();
    assert_eq!(
        (normal.kind, normal.origin),
        (CompositionKind::Normal, Origin::Default)
    );
}
/// Rewritten for 10J MC-LV3 (was 10G §4 rule 5: the late file moved the framed photo into the
/// preset container): the framed photo stays in its container with its composition; the late file
/// takes the current (preset) container and gets the presets on its frame.
#[test]
fn t2_c5b_late_file_gets_the_current_containers_preset() {
    let a = app();
    chair(&a);
    let s = session(&a, "C5B", "Chair");
    let f1 = arrive(&a, s, "DSC00001.ARW");
    frame(&a, &f1);
    let f3 = arrive(&a, s, "DSC00003.ARW");
    frame(&a, &f3);
    let f2 = arrive(&a, s, "DSC00002.ARW");
    frame(&a, &f2);
    let kept = a.composition(f3.id).unwrap();
    assert_eq!(
        (place(&a, f3.id).2, kept.kind, kept.origin),
        (Some(1), CompositionKind::Normal, Origin::Default)
    );
    let c = a.composition(f2.id).unwrap();
    assert_eq!(
        (place(&a, f2.id).2, c.kind, c.origin),
        (Some(3), CompositionKind::Split, Origin::User)
    );
    rect_eq(
        c.rect.unwrap(),
        Rect {
            x: 0.1,
            y: 0.2,
            w: 0.8,
            h: 0.6,
        },
    );
}
/// Rewritten for 10J MC-LV3 (was 10G §4 rule 5: a late file pushed the preset photo to
/// Condition): a photo becomes Condition when its container leaves the template (CS3), and its
/// preset composition is removed.
#[test]
fn t2_c5c_preset_removed_when_photo_becomes_condition() {
    let a = app();
    let t = chair(&a);
    let s = session(&a, "C5C", "Chair");
    arrive(&a, s, "DSC00001.ARW");
    arrive(&a, s, "DSC00002.ARW");
    let f4 = arrive(&a, s, "DSC00004.ARW");
    frame(&a, &f4);
    assert_eq!(place(&a, f4.id).2, Some(3));
    assert_eq!(a.composition(f4.id).unwrap().kind, CompositionKind::Split);
    templates(&a).container_remove(t.containers[2].id).unwrap();
    arrive(&a, s, "DSC00005.ARW");
    assert_eq!(place(&a, f4.id).0, C::Condition);
    let c = a.composition(f4.id).unwrap();
    assert_eq!(
        (c.kind, c.origin),
        (CompositionKind::Normal, Origin::Default)
    );
    rect_eq(
        c.rect.unwrap(),
        Rect {
            x: 1.0 / 6.0,
            y: 0.0,
            w: 2.0 / 3.0,
            h: 1.0,
        },
    );
}
#[test]
fn t2_c6a_view_has_containers_and_photos() {
    let a = app();
    chair(&a);
    let s = session(&a, "C6A", "Chair");
    let ids: Vec<_> = (1..=5)
        .map(|n| arrive(&a, s, &format!("DSC{n:05}.ARW")).id)
        .collect();
    let v = session_view(a.store(), s).unwrap();
    assert!(v.templated);
    assert_eq!(
        v.containers
            .iter()
            .map(|c| (c.final_no, c.shoot_no, c.presets.split, c.presets.zoom))
            .collect::<Vec<_>>(),
        vec![(1, 2, false, 0), (2, 1, false, 0), (3, 3, true, 20)]
    );
    assert_eq!(
        v.photos
            .iter()
            .map(|p| (p.photo_id, p.slot, p.by_hand, p.differs))
            .collect::<Vec<_>>(),
        vec![
            (ids[1], Some(1), false, false),
            (ids[0], Some(2), false, false),
            (ids[2], Some(3), false, false),
            (ids[3], None, false, false),
            (ids[4], None, false, false)
        ]
    );
    assert_eq!((v.differs, v.can_shift_back), (0, false));
}
#[test]
fn t2_c6b_no_template_view() {
    let a = app();
    let s = session(&a, "C6B", "Table");
    let v = session_view(a.store(), s).unwrap();
    assert!(!v.templated && v.containers.is_empty());
}
#[test]
fn t2_c7a_manual_classify_stays() {
    let a = app();
    chair(&a);
    let s = session(&a, "C7A", "Chair");
    for n in 1..=3 {
        arrive(&a, s, &format!("DSC{n:05}.ARW"));
    }
    let f4 = arrive(&a, s, "DSC00004.ARW");
    arrive(&a, s, "DSC00005.ARW");
    a.review().classify(f4.id, C::Main).unwrap();
    assert_eq!(place(&a, f4.id), (C::Main, Some(4), None, true));
    let f6 = arrive(&a, s, "DSC00006.ARW");
    assert_eq!(place(&a, f6.id).0, C::Condition);
    assert_eq!(place(&a, f4.id), (C::Main, Some(4), None, true));
}
#[test]
fn t2_c7b_manual_condition_stays() {
    let a = app();
    chair(&a);
    let s = session(&a, "C7B", "Chair");
    let f1 = arrive(&a, s, "DSC00001.ARW");
    a.review().classify(f1.id, C::Condition).unwrap();
    arrive(&a, s, "DSC00003.ARW");
    let f2 = arrive(&a, s, "DSC00002.ARW");
    // 10J MC-LV3: f1 keeps its shot (container 2, now empty by hand); file 3 took the next current
    // shot (container 1), the late file 2 the one after (container 3).
    assert_eq!(place(&a, f1.id).0, C::Condition);
    assert_eq!(place(&a, f2.id).2, Some(3));
    assert_eq!(place(&a, photo(&a, s, "DSC00003.ARW").id).2, Some(1));
}
#[test]
fn t2_c7c_move_loses_place_row() {
    let a = app();
    chair(&a);
    let s = session(&a, "C7C", "Chair");
    let f1 = arrive(&a, s, "DSC00001.ARW");
    arrive(&a, s, "DSC00002.ARW");
    arrive(&a, s, "DSC00003.ARW");
    let s2 = session(&a, "C7C2", "Chair");
    a.review().move_photos(&[f1.id], s2).unwrap();
    assert!(a.store().photo_place_row(f1.id).unwrap().is_none());
    // 10J MC-LV3 (was: the moved photo counted as shot 1): the moved photo is unbound in its new
    // session, so the first arrival there takes the first container in shooting order (2).
    let g = arrive(&a, s2, "DSC00011.ARW");
    assert_eq!(place(&a, g.id), (C::Main, Some(1), Some(2), false));
    assert_eq!(place(&a, f1.id), (C::Main, Some(2), None, false));
}
#[test]
fn t2_c7d_compose_edit_survives_late_arrival() {
    let a = app();
    chair(&a);
    let s = session(&a, "C7D", "Chair");
    arrive(&a, s, "DSC00001.ARW");
    let f3 = arrive(&a, s, "DSC00003.ARW");
    frame(&a, &f3);
    let mut values = a.composition(f3.id).unwrap().values().unwrap();
    values.kind = CompositionKind::Normal;
    values.shape = Shape::Original;
    values.orientation = Orientation::Named;
    values.quarter_turns = 0;
    values.flip = Flip::None;
    values.straighten = 0.0;
    values.rect = Rect {
        x: 0.1,
        y: 0.1,
        w: 0.8,
        h: 0.8,
    };
    let given = a.app.compositions.set(f3.id, values.clone()).unwrap();
    // 10J MC-LV3: the late file goes to the current shot (container 3); file 3 stays in 1.
    let f2 = arrive(&a, s, "DSC00002.ARW");
    assert_eq!(place(&a, f2.id).2, Some(3));
    assert_eq!(place(&a, f3.id).2, Some(1));
    assert_eq!(a.composition(f3.id).unwrap(), given);
    assert!(
        a.store()
            .photo_place_row(f3.id)
            .unwrap()
            .unwrap()
            .frame_by_hand
    );
}
#[test]
fn t2_b1_reorder_in_templated_session_keeps_the_dragged_order() {
    let a = app();
    chair(&a);
    let s = session(&a, "B1", "Chair");
    let f1 = arrive(&a, s, "DSC00001.ARW");
    let f2 = arrive(&a, s, "DSC00002.ARW");
    let f3 = arrive(&a, s, "DSC00003.ARW");
    let pos = |id| place(&a, id).1;
    assert_eq!(
        (pos(f2.id), pos(f1.id), pos(f3.id)),
        (Some(1), Some(2), Some(3))
    );
    a.review().reorder(f1.id, 1).unwrap();
    assert_eq!(
        (pos(f1.id), pos(f2.id), pos(f3.id)),
        (Some(1), Some(2), Some(3))
    );
    a.review().reorder(f3.id, 1).unwrap();
    assert_eq!(
        (pos(f3.id), pos(f1.id), pos(f2.id)),
        (Some(1), Some(2), Some(3))
    );
    // A later arrival does not move the hand-ordered photos.
    arrive(&a, s, "DSC00004.ARW");
    assert_eq!(
        (pos(f3.id), pos(f1.id), pos(f2.id)),
        (Some(1), Some(2), Some(3))
    );
}
