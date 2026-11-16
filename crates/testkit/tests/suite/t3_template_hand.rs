//! T3 — your changes win, and there is a way back (book 10G §6; `master_plan_templates.md` §5 P7, P9).
//! Fixture Chair-3 of the T2 plan: K1, K2, K3; shot 1 → K2, shot 2 → K1, shot 3 → K3; K3 split +
//! zoom 20. S0 = F1…F5 arrived in order (F2 slot 1, F1 slot 2, F3 slot 3, F4 and F5 Condition).
use psm_core::{
    Error,
    domain::{
        composition::{
            CompositionKind, EditorGeometry, Fingerprint, Flip, Frame, FrameSource, Orientation,
            Rect, Shape,
        },
        photo::{Classification as C, Photo},
        template::{Presets, Template, TemplateView},
    },
    ports::clock::Clock,
    services::{
        template_placement::{HandAction, back_to_template, drop_on_container, session_view_at},
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
/// S0 in session `sku` (type "Chair"); `framed` adopts each frame right after its arrival.
fn s0(a: &TestApp, sku: &str, framed: bool) -> (i64, Vec<Photo>) {
    chair(a);
    let s = session(a, sku, "Chair");
    let photos = (1..=5)
        .map(|n| {
            let p = arrive(a, s, &format!("DSC{n:05}.ARW"));
            if framed {
                frame(a, &p);
            }
            p
        })
        .collect();
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
fn pos(a: &TestApp, id: i64) -> Option<i64> {
    a.store().photo(id).unwrap().position
}
fn hand(a: &TestApp) -> HandAction<'_> {
    HandAction {
        store: a.store(),
        clock: a.clock.as_ref(),
        events: a.events.as_ref(),
        history: a.app.history.as_ref(),
    }
}
fn drop_on(a: &TestApp, id: i64, n: i64) -> Result<(), Error> {
    drop_on_container(&hand(a), id, n).map(|_| ())
}
fn back(a: &TestApp, s: i64) {
    back_to_template(&hand(a), s).unwrap();
}
fn rect_eq(a: Rect, b: Rect) {
    for (x, y) in [(a.x, b.x), (a.y, b.y), (a.w, b.w), (a.h, b.h)] {
        assert!((x - y).abs() < 1e-9, "{x} != {y}");
    }
}
/// One stored photo: (id, classification, position, slot, place_by_hand).
type Stored = (i64, C, Option<i64>, Option<i64>, bool);
/// Every photo of the session.
fn state(a: &TestApp, s: i64) -> Vec<Stored> {
    a.store()
        .photos_in_session(s)
        .unwrap()
        .into_iter()
        .map(|p| {
            let r = a.store().photo_place_row(p.id).unwrap();
            (
                p.id,
                p.classification,
                p.position,
                r.as_ref().and_then(|r| r.slot),
                r.is_some_and(|r| r.place_by_hand),
            )
        })
        .collect()
}
fn own_normal(a: &TestApp, id: i64) {
    let mut values = a.composition(id).unwrap().values().unwrap();
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
    a.app.compositions.set(id, values).unwrap();
}

#[test]
fn t3_c1a_condition_photo_made_main_differs() {
    let a = app();
    let (s, f) = s0(&a, "C1A", false);
    a.review().classify(f[3].id, C::Main).unwrap();
    let v = view(&a, s);
    assert_eq!(marks(&v, f[3].id), (None, true, true));
    for p in v.photos.iter().filter(|p| p.photo_id != f[3].id) {
        assert!(!p.differs, "photo {} differs", p.photo_id);
    }
    assert_eq!(v.differs, 1);
}

#[test]
fn t3_c1b_photo_back_in_its_container_matches_again() {
    let a = app();
    let (s, f) = s0(&a, "C1B", false);
    a.review().classify(f[1].id, C::Condition).unwrap();
    a.review().classify(f[1].id, C::Main).unwrap();
    assert_eq!(place(&a, f[1].id), (C::Main, Some(1), Some(1)));
    assert_eq!(
        (pos(&a, f[1].id), pos(&a, f[0].id), pos(&a, f[2].id)),
        (Some(1), Some(2), Some(3))
    );
    let v = view(&a, s);
    assert_eq!(marks(&v, f[1].id), (Some(1), true, false));
    assert_eq!(v.differs, 0);
}

#[test]
fn t3_c1c_reframed_photo_differs_only_with_presets() {
    let a = app();
    let (s, f) = s0(&a, "C1C", true);
    own_normal(&a, f[2].id);
    let v = view(&a, s);
    assert!(marks(&v, f[2].id).2);
    assert_eq!(v.differs, 1);
    own_normal(&a, f[0].id);
    let v = view(&a, s);
    assert!(!marks(&v, f[0].id).2);
    assert_eq!(v.differs, 1);
}

#[test]
fn t3_c1d_moved_photo_differs_in_its_new_session() {
    let a = app();
    let (s, f) = s0(&a, "C1D", false);
    let s2 = session(&a, "C1D2", "Chair");
    a.review().move_photos(&[f[0].id], s2).unwrap();
    let v = view(&a, s);
    assert!(!v.photos.iter().any(|p| p.slot == Some(2)));
    assert_eq!(v.differs, 0);
    let v2 = view(&a, s2);
    assert_eq!(marks(&v2, f[0].id), (None, true, true));
    assert_eq!(v2.differs, 1);
}

#[test]
fn t3_c1e_copy_differs_original_stays() {
    let a = app();
    let (s, f) = s0(&a, "C1E", false);
    let s2 = session(&a, "C1E2", "Chair");
    a.review().copy_photos(&[f[0].id], s2).unwrap();
    let v = view(&a, s);
    assert_eq!(marks(&v, f[0].id), (Some(2), false, false));
    assert_eq!(v.differs, 0);
    let copy = a.store().photos_in_session(s2).unwrap();
    assert_eq!(copy.len(), 1);
    let v2 = view(&a, s2);
    let m = marks(&v2, copy[0].id);
    assert_eq!((m.1, m.2), (true, true));
}

#[test]
fn t3_c1f_photo_row_with_place_row_can_be_deleted() {
    let a = app();
    chair(&a);
    let s = session(&a, "C1F", "Chair");
    let f1 = arrive(&a, s, "DSC00001.ARW");
    assert!(a.store().photo_place_row(f1.id).unwrap().is_some());
    a.store()
        .transaction(&mut |tx| tx.delete_photo_row(f1.id))
        .unwrap();
    assert!(a.store().photo(f1.id).is_err());
    assert!(a.store().photo_place_row(f1.id).unwrap().is_none());
}

#[test]
fn t3_c1g_undo_keeps_container_order() {
    let a = app();
    chair(&a);
    let s = session(&a, "C1G", "Chair");
    let f1 = arrive(&a, s, "DSC00001.ARW");
    assert_eq!(place(&a, f1.id).2, Some(2));
    a.review().classify(f1.id, C::Condition).unwrap();
    let f2 = arrive(&a, s, "DSC00002.ARW");
    let f3 = arrive(&a, s, "DSC00003.ARW");
    assert_eq!((place(&a, f2.id).2, place(&a, f3.id).2), (Some(1), Some(3)));
    a.history().undo(s).unwrap();
    assert_eq!(place(&a, f1.id), (C::Main, Some(2), Some(2)));
    assert_eq!(
        (pos(&a, f2.id), pos(&a, f1.id), pos(&a, f3.id)),
        (Some(1), Some(2), Some(3))
    );
}

#[test]
fn t3_c2a_drop_fills_empty_container() {
    let a = app();
    chair(&a);
    let s = session(&a, "C2A", "Chair");
    let f1 = arrive(&a, s, "DSC00001.ARW");
    a.review().classify(f1.id, C::Condition).unwrap();
    drop_on(&a, f1.id, 3).unwrap();
    assert_eq!(place(&a, f1.id), (C::Main, Some(1), Some(3)));
    assert_eq!(marks(&view(&a, s), f1.id), (Some(3), true, true));
}

#[test]
fn t3_c2b_drop_on_filled_container_swaps() {
    let a = app();
    let (s, f) = s0(&a, "C2B", false);
    drop_on(&a, f[2].id, 2).unwrap();
    assert_eq!(
        (place(&a, f[2].id).2, place(&a, f[0].id).2),
        (Some(2), Some(3))
    );
    assert_eq!(
        (pos(&a, f[1].id), pos(&a, f[2].id), pos(&a, f[0].id)),
        (Some(1), Some(2), Some(3))
    );
    let v = view(&a, s);
    assert_eq!(marks(&v, f[2].id), (Some(2), true, true));
    assert_eq!(marks(&v, f[0].id), (Some(3), true, true));
    assert_eq!(v.differs, 2);
}

#[test]
fn t3_c2c_drop_on_own_container_changes_nothing() {
    let a = app();
    let (s, f) = s0(&a, "C2C", false);
    let before = state(&a, s);
    drop_on(&a, f[2].id, 3).unwrap();
    assert_eq!(state(&a, s), before);
    assert!(a.history().state(s).undo.is_none());
}

#[test]
fn t3_c2d_condition_photo_onto_filled_container_refused() {
    let a = app();
    let (s, f) = s0(&a, "C2D", false);
    let before = state(&a, s);
    assert!(matches!(
        drop_on(&a, f[3].id, 1),
        Err(Error::ContainerFilled)
    ));
    assert_eq!(state(&a, s), before);
    assert!(a.history().state(s).undo.is_none());
}

#[test]
fn t3_c2e_drop_refusals() {
    let a = app();
    let (_, f) = s0(&a, "C2E", false);
    assert!(matches!(
        drop_on(&a, f[2].id, 0),
        Err(Error::InvalidValue(_))
    ));
    assert!(matches!(
        drop_on(&a, f[2].id, 4),
        Err(Error::InvalidValue(_))
    ));
    let t = session(&a, "C2E2", "Table");
    let g = arrive(&a, t, "DSC00011.ARW");
    assert!(matches!(drop_on(&a, g.id, 1), Err(Error::NoTemplate)));
}

#[test]
fn t3_c2f_drop_is_one_undo_step() {
    let a = app();
    let (s, f) = s0(&a, "C2F", false);
    let s0_state = state(&a, s);
    drop_on(&a, f[2].id, 2).unwrap();
    let dropped = state(&a, s);
    assert_eq!(
        a.history().state(s).undo.as_deref(),
        Some("Move to container 2")
    );
    a.history().undo(s).unwrap();
    assert_eq!(state(&a, s), s0_state);
    let v = view(&a, s);
    assert_eq!((marks(&v, f[2].id).1, marks(&v, f[0].id).1), (false, false));
    assert_eq!(v.differs, 0);
    a.history().redo(s).unwrap();
    assert_eq!(state(&a, s), dropped);
    assert_eq!(view(&a, s).differs, 2);
}

#[test]
fn t3_c3a_back_to_template_undoes_the_swap() {
    let a = app();
    let (s, f) = s0(&a, "C3A", false);
    drop_on(&a, f[2].id, 2).unwrap();
    back(&a, s);
    assert_eq!(
        (place(&a, f[2].id).2, place(&a, f[0].id).2),
        (Some(3), Some(2))
    );
    assert_eq!(
        (pos(&a, f[1].id), pos(&a, f[0].id), pos(&a, f[2].id)),
        (Some(1), Some(2), Some(3))
    );
    let v = view(&a, s);
    assert_eq!((marks(&v, f[2].id).1, marks(&v, f[0].id).1), (false, false));
    assert_eq!(v.differs, 0);
}

#[test]
fn t3_c3b_back_to_template_reapplies_presets() {
    let a = app();
    let (s, f) = s0(&a, "C3B", true);
    own_normal(&a, f[2].id);
    back(&a, s);
    let c = a.composition(f[2].id).unwrap();
    assert_eq!(
        (c.kind, c.origin),
        (
            CompositionKind::Split,
            psm_core::domain::composition::Origin::User
        )
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
    assert!(
        !a.store()
            .photo_place_row(f[2].id)
            .unwrap()
            .unwrap()
            .frame_by_hand
    );
    assert_eq!(view(&a, s).differs, 0);
}

#[test]
fn t3_c3c_back_to_template_returns_extra_main_to_condition() {
    let a = app();
    let (s, f) = s0(&a, "C3C", false);
    a.review().classify(f[3].id, C::Main).unwrap();
    back(&a, s);
    assert_eq!(place(&a, f[3].id), (C::Condition, Some(1), None));
    assert_eq!(place(&a, f[4].id), (C::Condition, Some(2), None));
    let main = state(&a, s).into_iter().filter(|p| p.1 == C::Main).count();
    assert_eq!(main, 3);
    assert_eq!(view(&a, s).differs, 0);
}

#[test]
fn t3_c3d_back_to_template_is_one_undo_step() {
    let a = app();
    let (s, f) = s0(&a, "C3D", false);
    drop_on(&a, f[2].id, 2).unwrap();
    let swapped = state(&a, s);
    back(&a, s);
    let backed = state(&a, s);
    assert_eq!(
        a.history().state(s).undo.as_deref(),
        Some("Back to template")
    );
    a.history().undo(s).unwrap();
    assert_eq!(state(&a, s), swapped);
    assert_eq!(
        (place(&a, f[2].id).2, place(&a, f[0].id).2),
        (Some(2), Some(3))
    );
    let v = view(&a, s);
    assert_eq!((marks(&v, f[2].id).1, marks(&v, f[0].id).1), (true, true));
    assert_eq!(v.differs, 2);
    a.history().redo(s).unwrap();
    assert_eq!(view(&a, s).differs, 0);
    assert_eq!(state(&a, s), backed);
}

#[test]
fn t3_c3e_back_to_template_leaves_a_denial() {
    let a = app();
    let (s, f) = s0(&a, "C3E", false);
    a.review().delete(f[0].id).unwrap();
    let label = a.history().state(s).undo;
    assert!(label.is_some());
    assert_eq!(view(&a, s).differs, 0);
    back(&a, s);
    assert_eq!(a.store().photo(f[0].id).unwrap().classification, C::Deleted);
    assert!(a.store().photo_denial(f[0].id).unwrap().is_some());
    assert_eq!(view(&a, s).differs, 0);
    assert_eq!(a.history().state(s).undo, label);
}

#[test]
fn t3_c3f_photos_from_before_the_template_are_filed_by_back() {
    let a = app();
    let s = session(&a, "C3F", "Stool");
    let g1 = arrive(&a, s, "DSC00001.ARW");
    let g2 = arrive(&a, s, "DSC00002.ARW");
    let t = templates(&a).create("Stool").unwrap();
    let t = templates(&a).container_add(t.id, None).unwrap();
    let t = templates(&a).container_add(t.id, None).unwrap();
    let ids: Vec<_> = t.containers.iter().map(|c| c.id).collect();
    templates(&a)
        .shooting_order_set(t.id, &[ids[1], ids[0]])
        .unwrap();
    let v = view(&a, s);
    assert_eq!((marks(&v, g1.id).1, marks(&v, g1.id).2), (true, true));
    assert_eq!((marks(&v, g2.id).1, marks(&v, g2.id).2), (true, true));
    assert_eq!(v.differs, 2);
    back(&a, s);
    assert_eq!((place(&a, g1.id).2, place(&a, g2.id).2), (Some(2), Some(1)));
    assert_eq!((pos(&a, g2.id), pos(&a, g1.id)), (Some(1), Some(2)));
    for id in [g1.id, g2.id] {
        assert!(
            !a.store()
                .photo_place_row(id)
                .unwrap()
                .unwrap()
                .place_by_hand
        );
    }
    assert_eq!(view(&a, s).differs, 0);
}
