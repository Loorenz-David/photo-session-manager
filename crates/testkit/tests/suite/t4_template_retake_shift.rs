//! T4 — retakes and bad shots (book 10G §6 rules 3–5, §8; `master_plan_templates.md` §5 P10).
//! Fixture Chair-3 of the T2 plan: K1, K2, K3; shot 1 → K2, shot 2 → K1, shot 3 → K3; K3 split +
//! zoom 20. S0 = F1…F5 arrived in order (F2 slot 1, F1 slot 2, F3 slot 3, F4 and F5 Condition).
use psm_core::{
    Error,
    domain::{
        camera::{CaptureId, ExposureInitiator, ExposureObserved},
        composition::{
            CompositionKind, EditorGeometry, Fingerprint, Frame, FrameSource, Origin, Rect,
        },
        photo::{Classification as C, Photo},
        template::{PlaceRow, Presets, Template, TemplateView},
    },
    ports::{camera::CaptureSink, clock::Clock},
    services::{
        template_placement::{
            HandAction, back_to_template, drop_on_container, session_view_at, shift_back,
        },
        templates::TemplatesService,
    },
};
use psm_infra::imaging::ImageOpsImpl;
use psm_testkit::TestApp;

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
/// Files `DSC00001.ARW`… arrive in order in session `s`, each frame adopted after its arrival.
fn arrive_framed(a: &TestApp, s: i64, numbers: std::ops::RangeInclusive<u32>) -> Vec<Photo> {
    numbers
        .map(|n| {
            let p = arrive(a, s, &format!("DSC{n:05}.ARW"));
            frame(a, &p);
            p
        })
        .collect()
}
/// S0 in session `sku` (type "Chair"), frames adopted.
fn s0(a: &TestApp, sku: &str) -> (i64, Vec<Photo>) {
    chair(a);
    let s = session(a, sku, "Chair");
    let photos = arrive_framed(a, s, 1..=5);
    (s, photos)
}
fn view(a: &TestApp, s: i64) -> TemplateView {
    session_view_at(a.store(), a.clock.now_ms(), s).unwrap()
}
/// (slot, by_hand, differs) of one photo in the view.
fn marks(v: &TemplateView, id: i64) -> (Option<i64>, bool, bool) {
    let p = v.photos.iter().find(|p| p.photo_id == id).unwrap();
    (p.slot, p.by_hand, p.differs)
}
/// (classification, position, slot) of one stored photo.
fn place(a: &TestApp, id: i64) -> (C, Option<i64>, Option<i64>) {
    let p = a.store().photo(id).unwrap();
    let r = a.store().photo_place_row(id).unwrap();
    (p.classification, p.position, r.and_then(|r| r.slot))
}
fn row(a: &TestApp, id: i64) -> PlaceRow {
    a.store().photo_place_row(id).unwrap().unwrap()
}
fn denied(a: &TestApp, id: i64) -> bool {
    a.store().photo_denial(id).unwrap().is_some()
}
fn hand(a: &TestApp) -> HandAction<'_> {
    HandAction {
        store: a.store(),
        clock: a.clock.as_ref(),
        events: a.events.as_ref(),
        history: a.app.history.as_ref(),
    }
}
fn drop_on(a: &TestApp, id: i64, n: i64) {
    drop_on_container(&hand(a), id, n).unwrap();
}
fn rect_eq(a: Rect, b: Rect) {
    for (x, y) in [(a.x, b.x), (a.y, b.y), (a.w, b.w), (a.h, b.h)] {
        assert!((x - y).abs() < 1e-9, "{x} != {y}");
    }
}
const SPLIT_ZOOM_20: Rect = Rect {
    x: 0.1,
    y: 0.2,
    w: 0.8,
    h: 0.6,
};
/// (kind, origin, rect) of one photo's composition.
fn comp(a: &TestApp, id: i64) -> (CompositionKind, Origin, Option<Rect>) {
    let c = a.composition(id).unwrap();
    (c.kind, c.origin, c.rect)
}
/// One stored photo, everything a template step restores: classification, position, place row,
/// composition (kind, origin, rect) and whether it has a denial row.
type Full = (
    i64,
    C,
    Option<i64>,
    Option<PlaceRow>,
    (CompositionKind, Origin, Option<Rect>),
    bool,
);
fn full(a: &TestApp, s: i64) -> Vec<Full> {
    let mut v: Vec<Full> = a
        .store()
        .photos_in_session(s)
        .unwrap()
        .into_iter()
        .map(|p| {
            (
                p.id,
                p.classification,
                p.position,
                a.store().photo_place_row(p.id).unwrap(),
                comp(a, p.id),
                denied(a, p.id),
            )
        })
        .collect();
    v.sort_by_key(|f| f.0);
    v
}

// ---- Unit 1: the retake ----

#[test]
fn t4_c1a_retake_takes_the_container_and_denies_the_old_photo() {
    let a = app();
    let (s, f) = s0(&a, "C1A");
    drop_on(&a, f[3].id, 3);
    let f3 = a.store().photo(f[2].id).unwrap();
    assert_eq!(f3.classification, C::Deleted);
    assert!(denied(&a, f[2].id));
    assert!(row(&a, f[2].id).released);
    assert_eq!(place(&a, f[3].id), (C::Main, Some(3), Some(3)));
    let v = view(&a, s);
    let m = marks(&v, f[3].id);
    assert_eq!((m.1, m.2), (true, false));
    let (kind, origin, rect) = comp(&a, f[3].id);
    assert_eq!((kind, origin), (CompositionKind::Split, Origin::User));
    rect_eq(rect.unwrap(), SPLIT_ZOOM_20);
    assert_eq!(place(&a, f[1].id), (C::Main, Some(1), Some(1)));
    assert_eq!(place(&a, f[0].id), (C::Main, Some(2), Some(2)));
    assert_eq!(place(&a, f[4].id), (C::Condition, Some(1), None));
    assert_eq!(v.differs, 0);
    assert!(!v.can_shift_back);
}

#[test]
fn t4_c1b_arrival_after_a_retake_goes_to_condition() {
    let a = app();
    let (s, f) = s0(&a, "C1B");
    drop_on(&a, f[3].id, 3);
    let f6 = arrive(&a, s, "DSC00006.ARW");
    assert_eq!(place(&a, f6.id), (C::Condition, Some(2), None));
    assert_eq!(place(&a, f[3].id), (C::Main, Some(3), Some(3)));
    assert_eq!(place(&a, f[1].id), (C::Main, Some(1), Some(1)));
    assert_eq!(place(&a, f[0].id), (C::Main, Some(2), Some(2)));
    assert_eq!(place(&a, f[4].id), (C::Condition, Some(1), None));
}

#[test]
fn t4_c1c_retake_is_one_undo_step() {
    let a = app();
    let (s, f) = s0(&a, "C1C");
    let before = full(&a, s);
    drop_on(&a, f[3].id, 3);
    let retaken = full(&a, s);
    assert_eq!(
        a.history().state(s).undo.as_deref(),
        Some("Retake container 3")
    );
    a.history().undo(s).unwrap();
    assert_eq!(full(&a, s), before);
    assert_eq!(place(&a, f[2].id), (C::Main, Some(3), Some(3)));
    assert!(!denied(&a, f[2].id));
    assert!(!row(&a, f[2].id).released);
    assert_eq!(place(&a, f[3].id), (C::Condition, Some(1), None));
    let r4 = row(&a, f[3].id);
    assert_eq!((r4.key_at, r4.key_name), (None, None));
    let (kind, origin, _) = comp(&a, f[3].id);
    assert_eq!((kind, origin), (CompositionKind::Normal, Origin::Default));
    assert_eq!(place(&a, f[4].id), (C::Condition, Some(2), None));
    a.history().redo(s).unwrap();
    assert_eq!(full(&a, s), retaken);
}

#[test]
fn t4_c1d_retake_of_a_retake_carries_the_key_on() {
    let a = app();
    let (s, f) = s0(&a, "C1D");
    drop_on(&a, f[3].id, 3);
    drop_on(&a, f[4].id, 3);
    assert_eq!(a.store().photo(f[3].id).unwrap().classification, C::Deleted);
    assert!(row(&a, f[3].id).released);
    assert_eq!(place(&a, f[4].id), (C::Main, Some(3), Some(3)));
    assert_eq!(row(&a, f[4].id).key_name.as_deref(), Some("DSC00003.ARW"));
    let v = view(&a, s);
    assert!(!marks(&v, f[4].id).2);
    assert_eq!(v.differs, 0);
}

// ---- Unit 2: Deny leaves the container empty; Shift the rest back ----

fn shift(a: &TestApp, s: i64) -> Result<(), Error> {
    shift_back(&hand(a), s).map(|_| ())
}
/// Chair-3, session `sku`: files 1–`last` arrive in order, frames adopted.
fn chair_with(a: &TestApp, sku: &str, last: u32) -> (i64, Vec<Photo>) {
    chair(a);
    let s = session(a, sku, "Chair");
    let photos = arrive_framed(a, s, 1..=last);
    (s, photos)
}

#[test]
fn t4_c2a_deny_leaves_the_container_empty() {
    let a = app();
    let (s, f) = chair_with(&a, "C2A", 3);
    a.review().delete(f[1].id).unwrap();
    assert_eq!(a.store().photo(f[1].id).unwrap().classification, C::Deleted);
    assert_eq!(place(&a, f[0].id).2, Some(2));
    assert_eq!(place(&a, f[2].id).2, Some(3));
    let v = view(&a, s);
    assert!(!v.photos.iter().any(|p| p.slot == Some(1)));
    let main_slot_1 = a
        .store()
        .photos_in_session(s)
        .unwrap()
        .into_iter()
        .filter(|p| p.classification == C::Main)
        .any(|p| place(&a, p.id).2 == Some(1));
    assert!(!main_slot_1);
    assert!(v.can_shift_back);
}

#[test]
fn t4_c2b_arrival_after_a_deny_fills_the_empty_container_and_moves_nothing_else() {
    // Owner 2026-11-16: the next shot fills the container a deleted photo left empty.
    let a = app();
    let (s, f) = chair_with(&a, "C2B", 3);
    a.review().delete(f[1].id).unwrap();
    let f1 = place(&a, f[0].id);
    let f3 = place(&a, f[2].id);
    let f4 = arrive(&a, s, "DSC00004.ARW");
    assert_eq!(place(&a, f4.id), (C::Main, Some(1), Some(1)));
    assert!(row(&a, f[1].id).released);
    assert_eq!(row(&a, f4.id).key_name.as_deref(), Some("DSC00002.ARW"));
    // The other photos keep their containers (Main's positions follow the containers, P7).
    assert_eq!(place(&a, f[0].id).2, f1.2);
    assert_eq!(place(&a, f[2].id).2, f3.2);
    let v = view(&a, s);
    assert_eq!(v.differs, 0);
    assert!(!v.can_shift_back);
    assert!(!marks(&v, f4.id).1, "placed by the template, not by hand");
    let f5 = arrive(&a, s, "DSC00005.ARW");
    assert_eq!(place(&a, f5.id).0, C::Condition);
}

#[test]
fn t4_c2b2_several_empty_containers_fill_in_shooting_order() {
    let a = app();
    // Shot 1 → K2, shot 2 → K1, shot 3 → K3 (split, zoom 20).
    let (s, f) = chair_with(&a, "C2B2", 3);
    a.review().delete(f[2].id).unwrap();
    a.review().delete(f[0].id).unwrap();
    assert_eq!(view(&a, s).next_shot, Some(1));
    let g = arrive_framed(&a, s, 4..=6);
    assert_eq!(place(&a, g[0].id), (C::Main, Some(2), Some(2)));
    assert_eq!(place(&a, g[1].id), (C::Main, Some(3), Some(3)));
    let (kind, _, rect) = comp(&a, g[1].id);
    assert_eq!(kind, CompositionKind::Split);
    rect_eq(rect.unwrap(), SPLIT_ZOOM_20);
    assert_eq!(place(&a, g[2].id).0, C::Condition);
    assert_eq!(place(&a, f[1].id), (C::Main, Some(1), Some(1)));
    assert_eq!(view(&a, s).differs, 0);
}

#[test]
fn t4_c2b3_a_late_file_keeps_its_own_place() {
    // Files 1 and 3 arrive, 1 is deleted, then file 2 arrives late: it is not the latest shot,
    // so it does not take shot 1's container (that would move file 3).
    let a = app();
    chair(&a);
    let s = session(&a, "C2B3", "Chair");
    let f1 = arrive_one(&a, s, 1);
    let f3 = arrive_one(&a, s, 3);
    a.review().delete(f1.id).unwrap();
    let f2 = arrive_one(&a, s, 2);
    assert!(!row(&a, f1.id).released);
    assert_eq!(place(&a, f2.id), (C::Main, Some(1), Some(1)));
    assert_eq!(place(&a, f3.id).2, Some(3));
    assert_eq!(containers(&view(&a, s), 3)[1], None);
    // The next shot is the latest: it fills container 2 (shot 1).
    let f4 = arrive_one(&a, s, 4);
    assert_eq!(place(&a, f4.id), (C::Main, Some(2), Some(2)));
    assert_eq!(place(&a, f3.id).2, Some(3));
}

#[test]
fn t4_c2c_shift_the_rest_back_moves_later_photos_back_one() {
    let a = app();
    let (s, f) = chair_with(&a, "C2C", 4);
    assert_eq!(place(&a, f[2].id).2, Some(3));
    assert_eq!(place(&a, f[3].id).0, C::Condition);
    a.review().delete(f[1].id).unwrap();
    shift(&a, s).unwrap();
    assert_eq!(place(&a, f[2].id), (C::Main, Some(1), Some(1)));
    let (kind, origin, _) = comp(&a, f[2].id);
    assert_eq!((kind, origin), (CompositionKind::Normal, Origin::Default));
    assert_eq!(place(&a, f[3].id).0, C::Main);
    assert_eq!(place(&a, f[3].id).2, Some(3));
    let (kind, _, rect) = comp(&a, f[3].id);
    assert_eq!(kind, CompositionKind::Split);
    rect_eq(rect.unwrap(), SPLIT_ZOOM_20);
    let condition = a
        .store()
        .photos_in_session(s)
        .unwrap()
        .into_iter()
        .filter(|p| p.classification == C::Condition)
        .count();
    assert_eq!(condition, 0);
    assert!(row(&a, f[1].id).released);
    assert!(!view(&a, s).can_shift_back);
}

#[test]
fn t4_c2d_shift_is_one_undo_step_then_the_deny() {
    let a = app();
    let (s, f) = chair_with(&a, "C2D", 4);
    a.review().delete(f[1].id).unwrap();
    let denied_state = full(&a, s);
    shift(&a, s).unwrap();
    assert_eq!(
        a.history().state(s).undo.as_deref(),
        Some("Shift the rest back")
    );
    a.history().undo(s).unwrap();
    assert_eq!(full(&a, s), denied_state);
    assert_eq!(place(&a, f[2].id).2, Some(3));
    assert_eq!(comp(&a, f[2].id).0, CompositionKind::Split);
    assert_eq!(place(&a, f[3].id), (C::Condition, Some(1), None));
    let (kind, origin, _) = comp(&a, f[3].id);
    assert_eq!((kind, origin), (CompositionKind::Normal, Origin::Default));
    assert_eq!(a.store().photo(f[1].id).unwrap().classification, C::Deleted);
    assert!(!row(&a, f[1].id).released);
    assert!(view(&a, s).can_shift_back);
    a.history().undo(s).unwrap();
    let f2 = place(&a, f[1].id);
    assert_eq!((f2.0, f2.2), (C::Main, Some(1)));
}

#[test]
fn t4_c2e_nothing_later_nothing_to_shift() {
    let a = app();
    let (s, f) = s0(&a, "C2E");
    a.review().delete(f[4].id).unwrap();
    let label = a.history().state(s).undo;
    assert!(label.is_some());
    assert!(!view(&a, s).can_shift_back);
    assert!(matches!(shift(&a, s), Err(Error::NothingToShift)));
    assert_eq!(a.history().state(s).undo, label);
}

#[test]
fn t4_c2e2_denied_condition_shot_with_later_condition_photo_has_nothing_to_shift() {
    let a = app();
    let (s, f) = s0(&a, "C2E2");
    a.review().delete(f[3].id).unwrap();
    let label = a.history().state(s).undo;
    assert!(label.is_some());
    assert!(!view(&a, s).can_shift_back);
    assert!(matches!(shift(&a, s), Err(Error::NothingToShift)));
    assert_eq!(a.history().state(s).undo, label);
}

#[test]
fn t4_c2f_hand_placed_later_photos_are_not_shifted() {
    let a = app();
    let (s, f) = chair_with(&a, "C2F", 3);
    a.review().classify(f[2].id, C::Condition).unwrap();
    a.review().delete(f[1].id).unwrap();
    assert!(!view(&a, s).can_shift_back);
}

// ---- Unit 3: the owner's story (10G §8 A1–A9) ----

fn expose(a: &TestApp, at: i64, n: i64) {
    let now = a.clock.now_ms();
    a.clock.advance(at - now);
    a.ledger()
        .exposure_observed(ExposureObserved {
            capture_id: Some(CaptureId(format!("story-shot-{n}"))),
            at,
            initiated: ExposureInitiator::App,
            provisional: false,
        })
        .unwrap();
}
fn example(a: &TestApp, name: &str) -> std::path::PathBuf {
    let path = a.app.paths.root.join(name);
    image::RgbImage::from_pixel(300, 200, image::Rgb([120, 90, 50]))
        .save(&path)
        .unwrap();
    path
}
/// The photo ids in containers 1…n of the view (None for an empty container).
fn containers(v: &TemplateView, n: i64) -> Vec<Option<i64>> {
    (1..=n)
        .map(|k| {
            v.photos
                .iter()
                .find(|p| p.slot == Some(k))
                .map(|p| p.photo_id)
        })
        .collect()
}
fn arrive_one(a: &TestApp, s: i64, n: u32) -> Photo {
    let p = arrive(a, s, &format!("DSC{n:05}.ARW"));
    frame(a, &p);
    p
}

#[test]
fn t4_c4a_the_owners_story() {
    let mut a = app();

    // A1: the template editor; everything is kept across a restart.
    let t = templates(&a).create("Dining chair").unwrap();
    for n in 1..=4 {
        let path = example(&a, &format!("example-{n}.jpg"));
        templates(&a).container_add(t.id, Some(&path)).unwrap();
    }
    let t = templates(&a).get("Dining chair").unwrap().unwrap();
    let ids: Vec<i64> = t.containers.iter().map(|c| c.id).collect();
    assert_eq!(ids.len(), 4);
    assert!(t.containers.iter().all(|c| c.example_file.is_some()));
    // Another final order: the fourth example first.
    let t = templates(&a)
        .containers_reorder(t.id, &[ids[3], ids[0], ids[1], ids[2]])
        .unwrap();
    let by_final = |t: &Template, n: i64| t.containers.iter().find(|c| c.final_no == n).unwrap().id;
    let finals: Vec<i64> = (1..=4).map(|n| by_final(&t, n)).collect();
    assert_eq!(finals, vec![ids[3], ids[0], ids[1], ids[2]]);
    templates(&a)
        .container_presets_set(
            finals[2],
            Presets {
                split: true,
                shape: None,
                zoom: 20,
            },
        )
        .unwrap();
    // Shot 1 → container 2, shot 2 → container 1, shot 3 → container 3, shot 4 → container 4.
    templates(&a)
        .shooting_order_set(t.id, &[finals[1], finals[0], finals[2], finals[3]])
        .unwrap();
    let saved = templates(&a).get("Dining chair").unwrap().unwrap();
    let shoot = |t: &Template| -> Vec<(i64, i64)> {
        let mut v: Vec<_> = t
            .containers
            .iter()
            .map(|c| (c.final_no, c.shoot_no))
            .collect();
        v.sort();
        v
    };
    assert_eq!(shoot(&saved), vec![(1, 2), (2, 1), (3, 3), (4, 4)]);
    a.kill_and_restart().unwrap();
    let loaded = templates(&a).get("Dining chair").unwrap().unwrap();
    assert_eq!(loaded, saved);
    let c3 = loaded.containers.iter().find(|c| c.final_no == 3).unwrap();
    assert_eq!(
        c3.presets,
        Presets {
            split: true,
            shape: None,
            zoom: 20
        }
    );

    // A2: a new dining-chair session shows four empty containers.
    let s = session(&a, "STORY1", "Dining chair");
    let v = view(&a, s);
    assert!(v.templated);
    assert_eq!(v.containers.len(), 4);
    assert!(v.photos.is_empty());

    // A3: three shutters; files 1, 3, 2 arrive in that order.
    let base = a.clock.now_ms();
    for n in 1..=3 {
        expose(&a, base + n * 2000, n);
    }
    let f1 = arrive_one(&a, s, 1);
    let f3 = arrive_one(&a, s, 3);
    let v = view(&a, s);
    assert_eq!(
        containers(&v, 4),
        vec![None, Some(f1.id), Some(f3.id), None]
    );
    let f2 = arrive_one(&a, s, 2);
    let v = view(&a, s);
    assert_eq!(
        containers(&v, 4),
        vec![Some(f2.id), Some(f1.id), Some(f3.id), None]
    );
    let (kind, _, rect) = comp(&a, f3.id);
    assert_eq!(kind, CompositionKind::Split);
    rect_eq(rect.unwrap(), SPLIT_ZOOM_20);

    // A4: file 4 fills container 4; files 5–8 arrive as Condition.
    let f4 = arrive_one(&a, s, 4);
    assert_eq!(containers(&view(&a, s), 4)[3], Some(f4.id));
    let extra: Vec<Photo> = (5..=8).map(|n| arrive_one(&a, s, n)).collect();
    for p in &extra {
        assert_eq!(a.store().photo(p.id).unwrap().classification, C::Condition);
    }

    // A5: four Condition photos made Main: the warning counts 4; nothing reverts.
    for p in &extra {
        a.review().classify(p.id, C::Main).unwrap();
    }
    assert_eq!(view(&a, s).differs, 4);
    let f9 = arrive_one(&a, s, 9);
    assert_eq!(a.store().photo(f9.id).unwrap().classification, C::Condition);
    for p in &extra {
        assert_eq!(a.store().photo(p.id).unwrap().classification, C::Main);
    }
    assert_eq!(view(&a, s).differs, 4);

    // A6: the next chair; the split photo dragged to place 2; Back to template; undo; redo.
    let s2 = session(&a, "STORY2", "Dining chair");
    let g: Vec<Photo> = (11..=14).map(|n| arrive_one(&a, s2, n)).collect();
    assert_eq!(
        containers(&view(&a, s2), 4),
        vec![Some(g[1].id), Some(g[0].id), Some(g[2].id), Some(g[3].id)]
    );
    drop_on(&a, g[2].id, 2);
    assert_eq!(view(&a, s2).differs, 2);
    back_to_template(&hand(&a), s2).unwrap();
    let v = view(&a, s2);
    assert_eq!(v.differs, 0);
    assert_eq!(marks(&v, g[2].id).0, Some(3));
    a.history().undo(s2).unwrap();
    assert_eq!(view(&a, s2).differs, 2);
    a.history().redo(s2).unwrap();
    assert_eq!(view(&a, s2).differs, 0);

    // A7: Deny the photo in container 4: the container is empty and is next; file 5 fills it
    // (owner 2026-11-16; before, it went to Condition and needed Shift the rest back).
    a.review().delete(g[3].id).unwrap();
    let v = view(&a, s2);
    assert_eq!(containers(&v, 4)[3], None);
    assert!(!v.can_shift_back);
    assert_eq!(v.next_shot, Some(4));
    let g5 = arrive_one(&a, s2, 15);
    let v = view(&a, s2);
    assert_eq!(containers(&v, 4)[3], Some(g5.id));
    assert!(!v.can_shift_back);
    assert_eq!(v.differs, 0);

    // A8: a retake dropped onto container 1: the old photo is deleted (red); no warning.
    let g6 = arrive_one(&a, s2, 16);
    assert_eq!(a.store().photo(g6.id).unwrap().classification, C::Condition);
    let old = containers(&view(&a, s2), 4)[0].unwrap();
    drop_on(&a, g6.id, 1);
    assert_eq!(a.store().photo(old).unwrap().classification, C::Deleted);
    assert!(denied(&a, old));
    let v = view(&a, s2);
    assert_eq!(containers(&v, 4)[0], Some(g6.id));
    assert_eq!(v.differs, 0);

    // A9: a type without a template behaves as before (P66).
    let s3 = session(&a, "STORY3", "Table");
    let h: Vec<Photo> = (21..=23)
        .map(|n| arrive(&a, s3, &format!("DSC{n:05}.ARW")))
        .collect();
    for (i, p) in h.iter().enumerate() {
        let stored = a.store().photo(p.id).unwrap();
        assert_eq!(
            (stored.classification, stored.position),
            (C::Main, Some(i as i64 + 1))
        );
        assert!(a.store().photo_place_row(p.id).unwrap().is_none());
    }
}

// ---- Shooting guidance: the container the next shot goes to (10G §5, "next to shoot") ----

#[test]
fn next_shot_follows_the_shooting_order_and_counts_denied_and_reported_shots() {
    let a = app();
    chair(&a); // shot 1 → K2, shot 2 → K1, shot 3 → K3
    let s = session(&a, "NX1", "Chair");
    assert_eq!(
        view(&a, s).next_shot,
        Some(1),
        "nothing shot yet: shooting number 1 is next"
    );
    let f = arrive_framed(&a, s, 1..=2);
    assert_eq!(view(&a, s).next_shot, Some(3));
    // A denied photo leaves its container empty: that container is next (owner 2026-11-16).
    a.review().delete(f[1].id).unwrap();
    assert_eq!(view(&a, s).next_shot, Some(2));
    arrive_framed(&a, s, 3..=3);
    assert_eq!(view(&a, s).next_shot, Some(3), "the gap is filled");
    arrive_framed(&a, s, 4..=4);
    assert_eq!(
        view(&a, s).next_shot,
        None,
        "past the last container: Condition comes next"
    );

    // A reported shutter whose file has not arrived yet holds its place (P3): the guidance moves on.
    let b = app();
    chair(&b);
    let s2 = session(&b, "NX2", "Chair");
    assert_eq!(view(&b, s2).next_shot, Some(1));
    let at = b.clock.now_ms() + 1_000;
    expose(&b, at, 1);
    assert_eq!(view(&b, s2).next_shot, Some(2));
}
