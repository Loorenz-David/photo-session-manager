//! Owner correction A16 (2026-11-18). Evidence (work Mac, 1,593 photos): a session switch's last
//! step looked at every photo of the library (a file check plus a preview retry per photo) and took
//! 4–8 minutes; the next switch waited behind it, so the photos taken for the newly created session
//! T1-181126 were filed into the previous one (T4-181126). The owner's rule: a switch looks only at
//! the opened session's photos; published and cleared sessions stay out of recurring checks.
use psm_core::{
    domain::photo::{NewPhoto, PhotoSource},
    mechanisms::canon::canon,
    ports::{camera::ArrivalFollow, clock::Clock},
    services::sessions::{SessionsService, SwitchOrder},
};
use psm_integrations::editing::capture_one::follow::DeferredFollow;
use psm_testkit::TestApp;
use std::{
    path::PathBuf,
    sync::{Arc, mpsc},
    time::{Duration, Instant},
};

fn session(a: &TestApp, sku: &str) -> (i64, String) {
    let (item, session) = a.items().create_local(sku, sku, "object", None).unwrap();
    (session.session.id, item.folder_key)
}
fn ops(a: &TestApp) -> Vec<String> {
    a.fake.state.lock().unwrap().ops.clone()
}
fn clear(a: &TestApp) {
    a.fake.state.lock().unwrap().ops.clear();
}
/// A photo row in `key`'s capture folder whose preview is due (as after its arrival), without
/// running the arrival's Capture One follow-up.
fn stale_photo(a: &TestApp, session_id: i64, key: &str, name: &str) -> String {
    let path = a.capture_root().join(key).join(name);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, name.as_bytes()).unwrap();
    let raw = canon(&path).unwrap();
    a.store()
        .photo_insert_or_get(NewPhoto {
            session_id,
            raw_path: raw.clone(),
            raw_hash: Some(format!("hash-{name}")),
            captured_at: Some(a.clock.now_ms()),
            source: PhotoSource::Capture,
            created_at: a.clock.now_ms(),
        })
        .unwrap();
    a.store().upsert_raw_state(&raw, None).unwrap();
    a.store().ensure_initial_generation(&raw).unwrap();
    raw
}
fn touches(ops: &[String], raws: &[String]) -> usize {
    ops.iter()
        .filter(|op| raws.iter().any(|raw| op.contains(raw.as_str())))
        .count()
}

/// The switch itself never looks at other sessions' photos, and returns without waiting on the
/// library; the follow-up (on the background loop) looks only at the opened session's photos.
#[test]
fn a16_a_switch_looks_only_at_the_opened_sessions_photos() {
    let a = TestApp::with_camera().unwrap();
    let (old, old_key) = session(&a, "A16-OLD");
    let (new, new_key) = session(&a, "A16-NEW");
    let library: Vec<String> = (0..1600)
        .map(|n| stale_photo(&a, old, &old_key, &format!("DSC{n:05}.ARW")))
        .collect();
    let opened: Vec<String> = (0..3)
        .map(|n| stale_photo(&a, new, &new_key, &format!("NEW{n}.ARW")))
        .collect();
    for raw in &opened {
        a.fake.add_image(raw, &[&format!("v-{}", raw.len())], false);
    }
    clear(&a);

    let started = Instant::now();
    a.sessions().activate(new).unwrap();
    let switch = started.elapsed();
    assert_eq!(a.store().active_session_id().unwrap(), Some(new));
    assert_eq!(
        touches(&ops(&a), &library),
        0,
        "the switch asked Capture One about other sessions' photos"
    );
    assert!(
        switch < Duration::from_secs(3),
        "the switch took {switch:?} with 1,600 photos elsewhere"
    );

    // The follow-up: the opened session's photos only.
    a.app.follow.tick().unwrap();
    let after = ops(&a);
    assert_eq!(
        touches(&after, &library),
        0,
        "{:?}",
        &after[..after.len().min(20)]
    );
    assert!(
        touches(&after, &opened) > 0,
        "the opened session's previews are asked for: {after:?}"
    );
}

/// The recurring preview pass and the sidecar check leave out deleted photos and published or
/// cleared sessions; the full list (startup, focus, reopen) still has them.
#[test]
fn a16_published_cleared_and_deleted_photos_are_left_out_of_recurring_work() {
    let a = TestApp::with_camera().unwrap();
    let (live, live_key) = session(&a, "A16-LIVE");
    let (published, pub_key) = session(&a, "A16-PUB");
    let (cleared, clear_key) = session(&a, "A16-CLR");
    let live_raw = stale_photo(&a, live, &live_key, "L.ARW");
    let deleted_raw = stale_photo(&a, live, &live_key, "D.ARW");
    let pub_raw = stale_photo(&a, published, &pub_key, "P.ARW");
    let clear_raw = stale_photo(&a, cleared, &clear_key, "C.ARW");
    let conn = rusqlite::Connection::open(a.app.paths.db()).unwrap();
    conn.execute(
        "UPDATE photos SET classification='deleted' WHERE raw_path=?1",
        [&deleted_raw],
    )
    .unwrap();
    a.mark_published(published).unwrap();
    conn.execute(
        "INSERT INTO item_retention(item_id,cleared_at) SELECT item_id,1 FROM sessions WHERE id=?1",
        [cleared],
    )
    .unwrap();

    let raws = |states: Vec<psm_core::ports::store::RawPreviewState>| {
        let mut raws: Vec<String> = states.into_iter().map(|s| s.raw_path).collect();
        raws.sort();
        raws
    };
    assert_eq!(
        raws(a.store().preview_states_live().unwrap()),
        vec![live_raw.clone()]
    );
    assert_eq!(
        raws(a.store().preview_states_in_session(live).unwrap()),
        vec![live_raw.clone()]
    );
    let mut all = vec![live_raw, deleted_raw, pub_raw.clone(), clear_raw];
    all.sort();
    assert_eq!(raws(a.store().preview_states().unwrap()), all);

    // A published session that changes again (a late photo, an edit) is back in the pass.
    let pub_photo: i64 = conn
        .query_row("SELECT id FROM photos WHERE raw_path=?1", [&pub_raw], |r| {
            r.get(0)
        })
        .unwrap();
    a.review()
        .classify(
            pub_photo,
            psm_core::domain::photo::Classification::Condition,
        )
        .unwrap();
    assert!(a.store().session(published).unwrap().dirty);
    assert!(raws(a.store().preview_states_live().unwrap()).contains(&pub_raw));
}

/// The bug the owner saw: while a switch is still busy in Capture One, the next session is
/// asked for and photos taken now must be filed to it at once.
#[test]
fn a16_photos_go_to_the_newly_opened_session_while_capture_one_is_still_switching() {
    let a = TestApp::with_camera().unwrap();
    let (t4, _) = session(&a, "T4-A16");
    let (t1, _) = session(&a, "T1-A16");
    a.sessions().activate(t4).unwrap();

    let order = Arc::new(SwitchOrder::new());
    // T4's switch is still in Capture One (it holds the order's turn).
    let first = order.ticket();
    let (hold_tx, hold_rx) = mpsc::channel::<()>();
    let (busy_tx, busy_rx) = mpsc::channel::<()>();
    let busy_order = order.clone();
    let slow = std::thread::spawn(move || {
        busy_order
            .run(first, "open session T4", || {
                busy_tx.send(()).unwrap();
                hold_rx.recv().unwrap();
                Ok(())
            })
            .unwrap();
    });
    busy_rx.recv().unwrap();

    // The owner opens T1: its filing target is claimed without waiting for T4's switch.
    let second = order.ticket();
    let claimed = {
        let (done_tx, done_rx) = mpsc::channel();
        let claim_order = order.clone();
        let store = a.app.store.clone();
        let camera = a.app.camera.clone();
        let clock = a.clock.clone();
        std::thread::spawn(move || {
            let _ = done_tx.send(claim_order.claim(second, || {
                SessionsService::new(store.as_ref(), clock.as_ref())
                    .with_camera(camera.as_ref())
                    .claim_target(Some(t1))
            }));
        });
        done_rx
            .recv_timeout(Duration::from_secs(2))
            .expect("the claim waited behind the busy Capture One switch")
            .unwrap()
    };
    assert!(claimed);
    assert_eq!(a.store().active_session_id().unwrap(), Some(t1));

    // A photo taken now goes to T1, not T4.
    let raw = a.arrival_photo(t1, "DSC06785.ARW").unwrap();
    let photo_session: i64 = rusqlite::Connection::open(a.app.paths.db())
        .unwrap()
        .query_row(
            "SELECT session_id FROM photos WHERE raw_path=?1",
            [canon(&PathBuf::from(&raw)).unwrap()],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(photo_session, t1);

    // T4's switch ends; T1's switch runs after it and keeps T1 as the target.
    hold_tx.send(()).unwrap();
    slow.join().unwrap();
    order
        .run(second, "open session T1", || {
            a.sessions().with_target_claimed(claimed).activate(t1)
        })
        .unwrap();
    assert_eq!(a.store().active_session_id().unwrap(), Some(t1));

    // A claim a newer request has overtaken is skipped.
    let stale = order.ticket();
    let _newer = order.ticket();
    assert!(!order.claim(stale, || Ok(true)).unwrap());
}

/// A10 queued the arrival's Capture One work for the background loop, and with it the style
/// eligibility: the photo's preview job could run first and render it without the style. The
/// eligibility and the wait for Capture One to list the photo are now recorded at once.
#[test]
fn a16_an_arrival_waits_for_its_style_even_before_the_loop_runs_its_follow_up() {
    let mut a = TestApp::with_camera().unwrap();
    a.settings_set(&serde_json::json!({"style_name":"Bright Contrast"}))
        .unwrap();
    a.fake.state.lock().unwrap().styles = vec!["Bright Contrast".into()];
    let (s, key) = session(&a, "A16-STYLE");
    a.sessions().activate(s).unwrap();
    let raw = stale_photo(&a, s, &key, "DSC06775.ARW");
    a.fake.add_image(&raw, &["v1"], false);
    clear(&a);

    let deferred = DeferredFollow::new(a.app.follow.clone());
    let folder = a.capture_root().join(&key);
    deferred.photo_arrived(s, &raw, &folder);
    // The preview job runs before the background loop has flushed the arrival.
    a.app.previews.request_initial(&raw).unwrap();
    assert!(
        !ops(&a).iter().any(|op| op.starts_with("process:")),
        "rendered before its style: {:?}",
        ops(&a)
    );

    deferred.flush();
    a.tick(1_000).unwrap();
    let ops = ops(&a);
    let applied = ops
        .iter()
        .position(|op| op == "apply_style:v1:Bright Contrast")
        .unwrap_or_else(|| panic!("style not applied: {ops:?}"));
    let processed = ops
        .iter()
        .position(|op| op == "process:v1:PSM Preview")
        .unwrap_or_else(|| panic!("not rendered: {ops:?}"));
    assert!(applied < processed);
    assert!(a.store().style_applied_at(&raw).unwrap().is_some());
}
