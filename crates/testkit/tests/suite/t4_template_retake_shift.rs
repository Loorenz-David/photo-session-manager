//! T4 — retakes and bad shots (book 10G §6 rules 3–5, §8; `master_plan_templates.md` §5 P10).
//! Fixture Chair-3 of the T2 plan: K1, K2, K3; shot 1 → K2, shot 2 → K1, shot 3 → K3; K3 split +
//! zoom 20. S0 = F1…F5 arrived in order (F2 slot 1, F1 slot 2, F3 slot 3, F4 and F5 Condition).
use psm_core::{
    domain::{
        composition::{
            CompositionKind, EditorGeometry, Fingerprint, Frame, FrameSource, Origin, Rect,
        },
        photo::{Classification as C, Photo},
        template::{PlaceRow, Presets, Template, TemplateView},
    },
    ports::clock::Clock,
    services::{
        template_placement::{HandAction, drop_on_container, session_view_at},
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
