//! LV2·U1 — the current shot (book 10J MC-LV3; `master_plan_live.md` §5 CS): rows LV2·C1–C8 at
//! the wired TestApp. Fixture T3: item type "Chair", containers A (final 1, shoot 2, 1:1 zoom 20),
//! B (final 2, shoot 1, split), C (final 3, shoot 3, no presets); shooting order B, A, C.
use psm_core::{
    Error,
    domain::{
        camera::{CaptureId, ExposureInitiator, ExposureObserved},
        composition::{CompositionKind, Origin, Shape},
        photo::Classification as C,
        template::{Presets, Template},
    },
    events::AppEvent,
    ports::{camera::CaptureSink, clock::Clock},
    services::{
        current_shot::Current,
        template_placement::{self, HandAction},
        templates::TemplatesService,
    },
};
use psm_infra::imaging::ImageOpsImpl;
use psm_testkit::TestApp;
use std::{
    fs,
    io::Write,
    sync::{Arc, Mutex},
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
/// T3; returns the template and the ids of A, B, C.
fn t3(a: &TestApp) -> (Template, [i64; 3]) {
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
    let t = templates(a)
        .container_presets_set(
            cb,
            Presets {
                split: true,
                shape: None,
                zoom: 0,
            },
        )
        .unwrap();
    (t, ids)
}
fn new_session(a: &TestApp, sku: &str, kind: &str) -> i64 {
    a.items()
        .create_local(sku, sku, kind, None)
        .unwrap()
        .1
        .session
        .id
}
fn session(a: &TestApp, sku: &str, kind: &str) -> i64 {
    let s = new_session(a, sku, kind);
    a.sessions().activate(s).unwrap();
    s
}
fn current(a: &TestApp, s: i64) -> Option<Current> {
    a.app
        .camera_runtime
        .current_shots()
        .current(a.store(), s, a.clock.now_ms())
        .unwrap()
}
fn set(a: &TestApp, s: i64, target: Current) -> Result<(), Error> {
    a.app.camera_runtime.current_shots().set(
        a.store(),
        a.events.as_ref(),
        s,
        target,
        a.clock.now_ms(),
    )
}
fn next_shot(a: &TestApp, s: i64) -> Option<i64> {
    template_placement::session_view_with_current(
        a.store(),
        a.app.camera_runtime.current_shots().as_ref(),
        a.clock.now_ms(),
        s,
    )
    .unwrap()
    .next_shot
}
/// One shutter reported by the camera now (through the app's `CaptureSink`).
fn expose(a: &TestApp, label: &str) -> i64 {
    a.ledger()
        .exposure_observed(ExposureObserved {
            capture_id: Some(CaptureId(label.into())),
            at: a.clock.now_ms(),
            initiated: ExposureInitiator::Camera,
            provisional: false,
        })
        .unwrap()
}
/// Past the ledger's FIFO window, so a new shutter cannot be matched to an earlier file.
fn after_window(a: &TestApp) {
    a.clock.advance(301_000);
}
/// Camera file `DSC<n>.ARW` delivered to the ledger, then taken in by the arrival scan.
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
fn db(a: &TestApp) -> rusqlite::Connection {
    rusqlite::Connection::open(a.app.paths.db()).unwrap()
}
/// `(shot_bound, shot_container_id)` of a `camera_exposures` row.
fn exposure_binding(a: &TestApp, id: i64) -> (i64, Option<i64>) {
    db(a)
        .query_row(
            "SELECT shot_bound,shot_container_id FROM camera_exposures WHERE id=?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap()
}
/// `(shot_bound, shot_container_id)` of a `photo_places` row.
fn place_binding(a: &TestApp, photo: i64) -> (i64, Option<i64>) {
    db(a)
        .query_row(
            "SELECT shot_bound,shot_container_id FROM photo_places WHERE photo_id=?1",
            [photo],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap()
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
fn clear_events(a: &TestApp) {
    a.events.0.lock().unwrap().clear();
}
fn session_changes(a: &TestApp, s: i64) -> usize {
    a.events
        .0
        .lock()
        .unwrap()
        .iter()
        .filter(|e| matches!(e, AppEvent::SessionChanged { session_id } if *session_id == s))
        .count()
}
fn denied(a: &TestApp, id: i64) -> bool {
    a.store().photo_denial(id).unwrap().is_some()
}
fn released(a: &TestApp, id: i64) -> bool {
    a.store().photo_place_row(id).unwrap().unwrap().released
}
/// A's presets on a photo whose frame is not known yet: shape 1:1, zoom 20 pending, set by the
/// template.
fn has_a_presets(a: &TestApp, id: i64) -> bool {
    let c = a.composition(id).unwrap();
    let row = a.store().photo_place_row(id).unwrap().unwrap();
    c.kind == CompositionKind::Normal
        && c.shape == Shape::Square
        && c.origin == Origin::User
        && row.pending_zoom == Some(20)
}

#[derive(Clone)]
struct LogBytes(Arc<Mutex<Vec<u8>>>);
impl Write for LogBytes {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
/// Runs `f` with the log lines it writes collected (P70's thread-scoped capture).
fn logged<T>(f: impl FnOnce() -> T) -> (T, String) {
    let buffer = Arc::new(Mutex::new(Vec::new()));
    let output = LogBytes(buffer.clone());
    psm_testkit::logs::keep_log_sites_interested();
    let subscriber = tracing_subscriber::fmt()
        .with_writer(move || output.clone())
        .without_time()
        .with_ansi(false)
        .finish();
    let value = tracing::subscriber::with_default(subscriber, f);
    let text = String::from_utf8(buffer.lock().unwrap().clone()).unwrap();
    (value, text)
}

// ---- LV2·C1: the current shot when read ----

#[test]
fn lv2_c1a_new_session_starts_at_the_first_container_in_shooting_order() {
    let a = app();
    let (_, [_, cb, _]) = t3(&a);
    let s = session(&a, "LV2-C1A", "Chair");
    assert_eq!(current(&a, s), Some(Current::Shot(cb)));
    assert_eq!(next_shot(&a, s), Some(1));
}

#[test]
fn lv2_c1b_a_reported_shutter_holds_its_container_across_a_relaunch() {
    let mut a = app();
    let (_, [ca, _, _]) = t3(&a);
    let s = session(&a, "LV2-C1B", "Chair");
    expose(&a, "c1b");
    a.kill_and_restart().unwrap();
    assert_eq!(current(&a, s), Some(Current::Shot(ca)));
}

#[test]
fn lv2_c1c_every_container_filled_means_free_and_no_next() {
    let a = app();
    t3(&a);
    let s = session(&a, "LV2-C1C", "Chair");
    for n in 100..103 {
        arrive(&a, s, n);
    }
    assert_eq!(current(&a, s), Some(Current::Free));
    assert_eq!(next_shot(&a, s), None);
}

#[test]
fn lv2_c1d_untemplated_session_binds_nothing_and_appends_main() {
    let a = app();
    t3(&a);
    let s = session(&a, "LV2-C1D", "Table");
    let earlier = arrive(&a, s, 90);
    after_window(&a);
    let e = expose(&a, "c1d");
    let p = arrive(&a, s, 100);
    assert_eq!(exposure_binding(&a, e), (0, None));
    assert_eq!(place(&a, earlier), (C::Main, Some(1), None));
    assert_eq!(place(&a, p), (C::Main, Some(2), None));
    assert!(a.store().photo_place_row(p).unwrap().is_none());
    assert_eq!(current(&a, s), None);
}

#[test]
fn lv2_c1e_another_sessions_view_shows_its_default_and_keeps_the_active_current() {
    let a = app();
    let (_, [_, _, cc]) = t3(&a);
    let s1 = session(&a, "LV2-C1E-1", "Chair");
    let s2 = new_session(&a, "LV2-C1E-2", "Chair");
    set(&a, s1, Current::Shot(cc)).unwrap();
    assert_eq!(next_shot(&a, s2), Some(1));
    assert_eq!(current(&a, s1), Some(Current::Shot(cc)));
    assert_eq!(next_shot(&a, s1), Some(3));
}

// ---- LV2·C2: binding an exposure ----

#[test]
fn lv2_c2a_an_exposure_binds_the_current_shot_and_moves_it_on() {
    let a = app();
    let (_, [ca, cb, _]) = t3(&a);
    let s = session(&a, "LV2-C2A", "Chair");
    clear_events(&a);
    let (e, log) = logged(|| expose(&a, "c2a"));
    assert_eq!(exposure_binding(&a, e), (1, Some(cb)));
    assert_eq!(session_changes(&a, s), 1);
    assert!(
        log.contains(&format!(
            "current_shot.bound session={s} exposure={e} container={cb}"
        )),
        "{log}"
    );
    assert_eq!(current(&a, s), Some(Current::Shot(ca)));
}

#[test]
fn lv2_c2b_after_the_last_container_the_current_shot_wraps_to_the_first_empty() {
    let a = app();
    let (_, [_, cb, cc]) = t3(&a);
    let s = session(&a, "LV2-C2B", "Chair");
    set(&a, s, Current::Shot(cc)).unwrap();
    let e = expose(&a, "c2b");
    assert_eq!(exposure_binding(&a, e), (1, Some(cc)));
    assert_eq!(current(&a, s), Some(Current::Shot(cb)));
}

#[test]
fn lv2_c2c_the_last_empty_container_bound_leaves_no_current_shot() {
    let a = app();
    let (_, [_, _, cc]) = t3(&a);
    let s = session(&a, "LV2-C2C", "Chair");
    arrive(&a, s, 100);
    arrive(&a, s, 101);
    assert_eq!(current(&a, s), Some(Current::Shot(cc)));
    after_window(&a);
    let e = expose(&a, "c2c");
    assert_eq!(exposure_binding(&a, e), (1, Some(cc)));
    assert_eq!(current(&a, s), Some(Current::Free));
}

#[test]
fn lv2_c2d_a_free_exposure_is_condition_and_the_current_shot_returns_to_an_empty_one() {
    let a = app();
    let (_, [ca, _, cc]) = t3(&a);
    let s = session(&a, "LV2-C2D", "Chair");
    let pb = arrive(&a, s, 100);
    set(&a, s, Current::Shot(cc)).unwrap();
    let pc = arrive(&a, s, 101);
    assert_eq!((place(&a, pb).2, place(&a, pc).2), (Some(2), Some(3)));
    set(&a, s, Current::Free).unwrap();
    after_window(&a);
    let e = expose(&a, "c2d");
    let p = arrive(&a, s, 102);
    assert_eq!(exposure_binding(&a, e), (1, None));
    assert_eq!(place(&a, p), (C::Condition, Some(1), None));
    assert_eq!(current(&a, s), Some(Current::Shot(ca)));
}

// ---- LV2·C3: late files go to the shot of their shutter ----

#[test]
fn lv2_c3a_late_files_follow_the_camera_counter_to_their_shutters_binding() {
    let a = app();
    let (_, [_, cb, _]) = t3(&a);
    let s = session(&a, "LV2-C3A", "Chair");
    let e0 = expose(&a, "c3a-0");
    let p100 = arrive(&a, s, 100);
    assert_eq!(exposure_binding(&a, e0), (1, Some(cb)));
    assert_eq!(place(&a, p100).2, Some(2));
    a.clock.advance(2_000);
    expose(&a, "c3a-1");
    a.clock.advance(2_000);
    expose(&a, "c3a-2");
    let p102 = arrive(&a, s, 102);
    assert_eq!(place(&a, p102).2, Some(3));
    let p101 = arrive(&a, s, 101);
    assert_eq!(place(&a, p101), (C::Main, Some(1), Some(1)));
    assert!(has_a_presets(&a, p101));
    assert_eq!(place(&a, p102).2, Some(3));
    assert_eq!(place(&a, p100).2, Some(2));
}

/// The counter rule needs an earlier file of the same run to read the counter against; the
/// session starts with one Condition shot (DSC00100) so the late files have it.
#[test]
fn lv2_c3b_a_late_file_after_a_tap_goes_to_its_shutters_shot() {
    let a = app();
    let (_, [_, cb, cc]) = t3(&a);
    let s = session(&a, "LV2-C3B", "Chair");
    set(&a, s, Current::Free).unwrap();
    expose(&a, "c3b-0");
    let p100 = arrive(&a, s, 100);
    assert_eq!(place(&a, p100).0, C::Condition);
    assert_eq!(current(&a, s), Some(Current::Shot(cb)));
    a.clock.advance(2_000);
    let e1 = expose(&a, "c3b-1");
    set(&a, s, Current::Shot(cc)).unwrap();
    a.clock.advance(2_000);
    let e2 = expose(&a, "c3b-2");
    assert_eq!(
        (exposure_binding(&a, e1), exposure_binding(&a, e2)),
        ((1, Some(cb)), (1, Some(cc)))
    );
    let p102 = arrive(&a, s, 102);
    assert_eq!(place(&a, p102).2, Some(3));
    let p101 = arrive(&a, s, 101);
    // No other photo changed its place when e1's file arrived.
    assert_eq!(place(&a, p101).2, Some(2));
    assert_eq!(place(&a, p102).2, Some(3));
    assert_eq!(place(&a, p100), (C::Condition, Some(1), None));
}

#[test]
fn lv2_c3c_the_binding_is_written_once() {
    let a = app();
    let (_, [ca, cb, _]) = t3(&a);
    let s = session(&a, "LV2-C3C", "Chair");
    let e1 = expose(&a, "c3c");
    let p = arrive(&a, s, 101);
    assert_eq!(place(&a, p).2, Some(2));
    assert_eq!(current(&a, s), Some(Current::Shot(ca)));
    after_window(&a);
    arrive(&a, s, 102);
    assert_eq!(exposure_binding(&a, e1), (1, Some(cb)));
    assert_eq!(place(&a, p).2, Some(2));
}

// ---- LV2·C4: the person sets the current shot ----

#[test]
fn lv2_c4a_a_chosen_empty_shot_takes_the_next_exposure() {
    let a = app();
    let (_, [_, _, cc]) = t3(&a);
    let s = session(&a, "LV2-C4A", "Chair");
    set(&a, s, Current::Shot(cc)).unwrap();
    let e = expose(&a, "c4a");
    assert_eq!(exposure_binding(&a, e), (1, Some(cc)));
}

#[test]
fn lv2_c4b_a_filled_shot_is_refused() {
    let a = app();
    let (_, [_, cb, _]) = t3(&a);
    let s = session(&a, "LV2-C4B", "Chair");
    arrive(&a, s, 100);
    let before = current(&a, s);
    assert!(matches!(
        set(&a, s, Current::Shot(cb)),
        Err(Error::ShotFilled)
    ));
    assert_eq!(current(&a, s), before);
}

#[test]
fn lv2_c4c_another_session_is_refused_and_nothing_held_changes() {
    let a = app();
    let (_, [_, cb, cc]) = t3(&a);
    let s = session(&a, "LV2-C4C", "Chair");
    let other = new_session(&a, "LV2-C4C-2", "Chair");
    assert!(matches!(
        set(&a, other, Current::Shot(cc)),
        Err(Error::SessionChanged)
    ));
    assert_eq!(current(&a, s), Some(Current::Shot(cb)));
}

#[test]
fn lv2_c4d_a_container_of_another_template_is_refused() {
    let a = app();
    t3(&a);
    let table = templates(&a).create("Table").unwrap();
    let table = templates(&a).container_add(table.id, None).unwrap();
    let s = session(&a, "LV2-C4D", "Chair");
    assert!(matches!(
        set(&a, s, Current::Shot(table.containers[0].id)),
        Err(Error::NotInTemplate)
    ));
}

#[test]
fn lv2_c4e_an_untemplated_session_has_no_current_shot_to_set() {
    let a = app();
    let s = session(&a, "LV2-C4E", "Table");
    assert!(matches!(set(&a, s, Current::Free), Err(Error::NoTemplate)));
}

// ---- LV2·C5: Retake ----

#[test]
fn lv2_c5a_retake_denies_the_photo_at_once_and_makes_its_container_current() {
    let a = app();
    let (_, [ca, _, _]) = t3(&a);
    let s = session(&a, "LV2-C5A", "Chair");
    arrive(&a, s, 100);
    let p = arrive(&a, s, 101);
    assert_eq!(place(&a, p).2, Some(1));
    clear_events(&a);
    template_placement::retake(&hand(&a), s, ca).unwrap();
    assert_eq!(place(&a, p).0, C::Deleted);
    assert!(denied(&a, p));
    assert!(released(&a, p));
    assert_eq!(current(&a, s), Some(Current::Shot(ca)));
    assert_eq!(a.history().state(s).undo.as_deref(), Some("Retake shot 1"));
    assert_eq!(session_changes(&a, s), 1);
}

#[test]
fn lv2_c5b_the_next_capture_fills_the_retaken_container_with_its_presets() {
    let a = app();
    let (_, [ca, _, cc]) = t3(&a);
    let s = session(&a, "LV2-C5B", "Chair");
    arrive(&a, s, 100);
    arrive(&a, s, 101);
    template_placement::retake(&hand(&a), s, ca).unwrap();
    after_window(&a);
    let e = expose(&a, "c5b");
    let next = arrive(&a, s, 102);
    assert_eq!(exposure_binding(&a, e), (1, Some(ca)));
    assert_eq!(place(&a, next), (C::Main, Some(1), Some(1)));
    assert!(has_a_presets(&a, next));
    assert_eq!(current(&a, s), Some(Current::Shot(cc)));
}

#[test]
fn lv2_c5c_undo_of_a_retake_brings_the_photo_back() {
    let a = app();
    let (_, [ca, _, cc]) = t3(&a);
    let s = session(&a, "LV2-C5C", "Chair");
    arrive(&a, s, 100);
    let p = arrive(&a, s, 101);
    template_placement::retake(&hand(&a), s, ca).unwrap();
    a.history().undo(s).unwrap();
    assert_eq!(place(&a, p), (C::Main, Some(1), Some(1)));
    assert!(!released(&a, p));
    assert!(!denied(&a, p));
    assert_eq!(current(&a, s), Some(Current::Shot(cc)));
}

#[test]
fn lv2_c5d_retake_of_an_empty_container_is_refused() {
    let a = app();
    let (_, [_, _, cc]) = t3(&a);
    let s = session(&a, "LV2-C5D", "Chair");
    arrive(&a, s, 100);
    arrive(&a, s, 101);
    let held = current(&a, s);
    let undo = a.history().state(s).undo;
    assert!(matches!(
        template_placement::retake(&hand(&a), s, cc),
        Err(Error::NotInTemplate)
    ));
    assert_eq!(current(&a, s), held);
    assert_eq!(a.history().state(s).undo, undo);
}

// ---- LV2·C6: the current shot never survives a switch or a relaunch ----

#[test]
fn lv2_c6a_a_session_switch_forgets_the_chosen_shot() {
    let a = app();
    let (_, [_, cb, cc]) = t3(&a);
    let s1 = session(&a, "LV2-C6A-1", "Chair");
    let s2 = new_session(&a, "LV2-C6A-2", "Chair");
    set(&a, s1, Current::Shot(cc)).unwrap();
    clear_events(&a);
    a.sessions().activate(s2).unwrap();
    a.sessions().activate(s1).unwrap();
    assert_eq!(current(&a, s1), Some(Current::Shot(cb)));
    let switches: Vec<Option<i64>> = a
        .events
        .0
        .lock()
        .unwrap()
        .iter()
        .filter_map(|e| match e {
            AppEvent::ActiveSessionChanged { session_id } => Some(*session_id),
            _ => None,
        })
        .collect();
    assert_eq!(switches, vec![Some(s2), Some(s1)]);
}

#[test]
fn lv2_c6b_a_relaunch_starts_from_the_default() {
    let mut a = app();
    let (_, [_, cb, cc]) = t3(&a);
    let s = session(&a, "LV2-C6B", "Chair");
    set(&a, s, Current::Shot(cc)).unwrap();
    a.kill_and_restart().unwrap();
    assert_eq!(current(&a, s), Some(Current::Shot(cb)));
}

// ---- LV2·C7: binding at arrival ----

#[test]
fn lv2_c7a_a_tethered_file_with_no_report_takes_the_current_shot() {
    let a = app();
    let (_, [ca, cb, _]) = t3(&a);
    let s = session(&a, "LV2-C7A", "Chair");
    let folder = a.capture_root().join(
        a.store()
            .item(a.store().session(s).unwrap().session.item_id.unwrap())
            .unwrap()
            .folder_key,
    );
    fs::create_dir_all(&folder).unwrap();
    let file = folder.join("tethered.ARW");
    fs::write(&file, b"raw").unwrap();
    a.app.arrival.watcher_event(&file).unwrap();
    a.tick(1_000).unwrap();
    let p = a
        .store()
        .photos_in_session(s)
        .unwrap()
        .into_iter()
        .find(|p| p.raw_path.ends_with("tethered.ARW"))
        .unwrap()
        .id;
    assert_eq!(place_binding(&a, p), (1, Some(cb)));
    assert_eq!(place(&a, p), (C::Main, Some(1), Some(2)));
    assert_eq!(current(&a, s), Some(Current::Shot(ca)));
}

#[test]
fn lv2_c7b_a_shutter_reported_after_its_file_takes_the_files_binding() {
    let a = app();
    let (_, [ca, cb, _]) = t3(&a);
    let s = session(&a, "LV2-C7B", "Chair");
    let p = arrive(&a, s, 100);
    assert_eq!(place_binding(&a, p), (1, Some(cb)));
    assert_eq!(current(&a, s), Some(Current::Shot(ca)));
    let e = expose(&a, "c7b");
    assert_eq!(exposure_binding(&a, e), (1, Some(cb)));
    assert_eq!(current(&a, s), Some(Current::Shot(ca)));
}

// ---- LV2·C8: hand actions under binding ----

#[test]
fn lv2_c8a_a_drop_onto_a_filled_container_retakes_it_by_template() {
    let a = app();
    let (_, [ca, _, _]) = t3(&a);
    let s = session(&a, "LV2-C8A", "Chair");
    arrive(&a, s, 100);
    let p = arrive(&a, s, 101);
    arrive(&a, s, 102);
    let x = arrive(&a, s, 103);
    assert_eq!(place(&a, x).0, C::Condition);
    template_placement::drop_on_container(&hand(&a), x, 1).unwrap();
    assert_eq!(place(&a, x).0, C::Main);
    assert_eq!(place(&a, x).2, Some(1));
    assert!(!a.store().photo_place_row(x).unwrap().unwrap().place_by_hand);
    assert_eq!(place_binding(&a, x), (1, Some(ca)));
    assert!(has_a_presets(&a, x));
    assert_eq!(place(&a, p).0, C::Deleted);
    assert!(released(&a, p));
    assert_eq!(
        a.history().state(s).undo.as_deref(),
        Some("Retake container 1")
    );
}

#[test]
fn lv2_c8b_shift_the_rest_back_rebinds_later_photos() {
    let a = app();
    let (_, [ca, _, _]) = t3(&a);
    let s = session(&a, "LV2-C8B", "Chair");
    arrive(&a, s, 100);
    let pa = arrive(&a, s, 101);
    let pc = arrive(&a, s, 102);
    a.review().delete(pa).unwrap();
    let snapshot = |a: &TestApp| {
        a.store()
            .photos_in_session(s)
            .unwrap()
            .into_iter()
            .map(|p| {
                (
                    p.id,
                    place(a, p.id),
                    a.store().photo_place_row(p.id).unwrap(),
                )
            })
            .collect::<Vec<_>>()
    };
    let before = snapshot(&a);
    template_placement::shift_back(&hand(&a), s).unwrap();
    assert_eq!(place(&a, pc).2, Some(1));
    assert_eq!(place_binding(&a, pc), (1, Some(ca)));
    assert!(has_a_presets(&a, pc));
    assert!(
        !a.store()
            .photos_in_session(s)
            .unwrap()
            .iter()
            .any(|p| p.classification == C::Main && place(&a, p.id).2 == Some(3))
    );
    assert!(released(&a, pa));
    a.history().undo(s).unwrap();
    assert_eq!(snapshot(&a), before);
}

#[test]
fn lv2_c8c_back_to_template_returns_a_photo_to_its_binding() {
    let a = app();
    t3(&a);
    let s = session(&a, "LV2-C8C", "Chair");
    arrive(&a, s, 100);
    let pa = arrive(&a, s, 101);
    a.review().classify(pa, C::Condition).unwrap();
    template_placement::back_to_template(&hand(&a), s).unwrap();
    assert_eq!(place(&a, pa).0, C::Main);
    assert_eq!(place(&a, pa).2, Some(1));
    assert!(
        !a.store()
            .photo_place_row(pa)
            .unwrap()
            .unwrap()
            .place_by_hand
    );
}

/// Condition order is shot order (10G §4 rule 4): the photo of the removed container goes after
/// the Condition shot taken before it.
#[test]
fn lv2_c8d_a_photo_whose_container_left_the_template_is_condition() {
    let a = app();
    let (_, [ca, _, _]) = t3(&a);
    let s = session(&a, "LV2-C8D", "Chair");
    arrive(&a, s, 100);
    set(&a, s, Current::Free).unwrap();
    let q = arrive(&a, s, 101);
    let pa = arrive(&a, s, 102);
    assert_eq!((place(&a, q).0, place(&a, pa).2), (C::Condition, Some(1)));
    templates(&a).container_remove(ca).unwrap();
    after_window(&a);
    arrive(&a, s, 103);
    assert_eq!(place(&a, q), (C::Condition, Some(1), None));
    assert_eq!(place(&a, pa), (C::Condition, Some(2), None));
    assert_eq!(place_binding(&a, pa), (1, Some(ca)));
}

#[test]
fn lv2_c8e_a_deny_does_not_refill_the_next_exposure_follows_the_current_shot() {
    let a = app();
    let (_, [ca, _, cc]) = t3(&a);
    let s = session(&a, "LV2-C8E", "Chair");
    arrive(&a, s, 100);
    let pa = arrive(&a, s, 101);
    assert_eq!(current(&a, s), Some(Current::Shot(cc)));
    a.review().delete(pa).unwrap();
    after_window(&a);
    let e = expose(&a, "c8e");
    assert_eq!(exposure_binding(&a, e), (1, Some(cc)));
    assert_eq!(current(&a, s), Some(Current::Shot(ca)));
}

#[test]
fn lv2_c8f_photos_placed_before_binding_stay_and_count_as_taken() {
    let mut a = app();
    let (_, [_, _, cc]) = t3(&a);
    let s = session(&a, "LV2-C8F", "Chair");
    let pb = arrive(&a, s, 100);
    let pa = arrive(&a, s, 101);
    // As placed before this change: no binding on the rows, no bound exposure.
    db(&a)
        .execute(
            "UPDATE photo_places SET shot_bound=0,shot_container_id=NULL",
            [],
        )
        .unwrap();
    a.kill_and_restart().unwrap();
    after_window(&a);
    assert_eq!(current(&a, s), Some(Current::Shot(cc)));
    let p = arrive(&a, s, 102);
    assert_eq!(place(&a, p).2, Some(3));
    assert_eq!(place(&a, pb), (C::Main, Some(2), Some(2)));
    assert_eq!(place(&a, pa), (C::Main, Some(1), Some(1)));
    assert_eq!(
        (place_binding(&a, pb), place_binding(&a, pa)),
        ((0, None), (0, None))
    );
}
