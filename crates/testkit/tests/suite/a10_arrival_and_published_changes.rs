//! Owner correction A10 (2026-11-18). Evidence (work Mac, 2026-11-15): three photos the camera's
//! FTP receiver had filed were dropped as `stability_timeout` while the background loop was held
//! up by Capture One work for ~2.7 min, then imported into two already-published sessions by the
//! scan at the next launch, which reopened them under "Open" with no reason on screen.
use psm_core::{
    mechanisms::canon::canon,
    services::{arrival::STABILITY_CHECKS, sessions::SwitchOrder},
};
use psm_testkit::TestApp;
use rusqlite::Connection;
use std::{fs, path::PathBuf, sync::Arc};

fn app() -> TestApp {
    TestApp::with_fake_document().unwrap()
}
fn file(a: &TestApp, relative: &str, bytes: &[u8]) -> PathBuf {
    let path = a
        .dir
        .path()
        .join("Capture One Session")
        .join("Capture")
        .join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, bytes).unwrap();
    path
}
fn session(a: &TestApp, sku: &str) -> i64 {
    a.items()
        .create_local(sku, sku, "object", None)
        .unwrap()
        .1
        .session
        .id
}
fn db(a: &TestApp) -> Connection {
    Connection::open(a.app.paths.db()).unwrap()
}
fn photos(a: &TestApp) -> i64 {
    db(a)
        .query_row("SELECT COUNT(*) FROM photos", [], |r| r.get(0))
        .unwrap()
}
fn errors(a: &TestApp) -> i64 {
    db(a)
        .query_row(
            "SELECT COUNT(*) FROM capture_events WHERE error='stability_timeout'",
            [],
            |r| r.get(0),
        )
        .unwrap()
}
fn photo_of(a: &TestApp, path: &std::path::Path) -> (i64, String) {
    db(a)
        .query_row(
            "SELECT id,classification FROM photos WHERE raw_path=?1",
            [canon(path).unwrap()],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap()
}

/// The loss itself: a complete file seen by the watcher, then a loop held up for longer than the
/// old 120 s limit. It must become a photo at the next look, not be given up.
#[test]
fn a10_a_complete_file_is_kept_however_long_the_loop_was_held_up() {
    let a = app();
    session(&a, "K1");
    let p = file(&a, "K1/DSC06443.ARW", b"raw bytes");
    a.watcher_event(&p).unwrap();
    // One tick 130 s later: the background loop was busy elsewhere all that time.
    a.tick(130_000).unwrap();
    assert_eq!(photos(&a), 1, "the filed photo is added");
    assert_eq!(errors(&a), 0, "nothing is given up");
}

/// Giving up still exists, but only for a file that keeps changing (or stays missing) for
/// `STABILITY_CHECKS` looks, whatever the clock did between them.
#[test]
fn a10_a_file_that_keeps_changing_is_given_up_after_the_checks_not_the_clock() {
    let a = app();
    let p = file(&a, "K1/x.ARW", b"x");
    a.watcher_event(&p).unwrap();
    for n in 1..STABILITY_CHECKS as usize {
        fs::write(&p, vec![b'x'; n + 1]).unwrap();
        // Long gaps between looks: under the old rule the first one already gave up.
        a.tick(200_000).unwrap();
    }
    assert_eq!((photos(&a), errors(&a)), (0, 0), "still being watched");
    fs::write(&p, vec![b'x'; 9_999]).unwrap();
    a.tick(1000).unwrap();
    assert_eq!((photos(&a), errors(&a)), (0, 1));
}

/// A photo that arrives late in a published session reopens it with the reason; "Keep as
/// published" denies it and makes the session published again; undo and redo are one step.
#[test]
fn a10_a_late_photo_reopens_with_its_reason_and_keep_as_published_is_one_undo_step() {
    let a = app();
    let s = session(&a, "K1");
    let other = session(&a, "K2");
    a.mark_published(s).unwrap();
    a.store().set_active_session_id(Some(other)).unwrap();
    a.clock.advance(1000);
    let p = file(&a, "K1/DSC06444.ARW", b"late raw");
    a.watcher_event(&p).unwrap();
    a.tick(1000).unwrap();

    let dto = a.store().session(s).unwrap();
    assert!(dto.dirty);
    let changes = dto.changes.expect("a reason is given");
    assert_eq!(changes.new_photos, 1);
    assert_eq!(changes.editor_edits, 0);
    assert!(!changes.app_edits, "an arrival is not an edit in the app");
    let row = a
        .sessions()
        .list(psm_core::domain::session::ListFilter::Open, None)
        .unwrap()
        .groups
        .into_iter()
        .flat_map(|g| g.rows)
        .find(|r| r.id == s)
        .expect("listed under Open");
    assert_eq!(row.changes.map(|c| c.new_photos), Some(1));

    a.review().keep_published(s).unwrap();
    let dto = a.store().session(s).unwrap();
    assert!(!dto.dirty, "published again");
    assert_eq!(dto.changes, None);
    assert_eq!(photo_of(&a, &p).1, "deleted", "the late photo is denied");

    a.history().undo(s).unwrap();
    let dto = a.store().session(s).unwrap();
    assert!(dto.dirty, "undo reopens it");
    assert_eq!(dto.changes.map(|c| c.new_photos), Some(1));
    assert_eq!(photo_of(&a, &p).1, "main");

    a.history().redo(s).unwrap();
    let dto = a.store().session(s).unwrap();
    assert!(!dto.dirty);
    assert_eq!(photo_of(&a, &p).1, "deleted");
}

/// An adjustment Capture One reports after the publish names the photo; keeping it as published
/// changes no photo.
#[test]
fn a10_an_adjustment_in_capture_one_after_publishing_is_named_and_can_be_kept() {
    let a = app();
    let s = session(&a, "K1");
    a.store().set_active_session_id(Some(s)).unwrap();
    let p = file(&a, "K1/a.ARW", b"raw");
    a.watcher_event(&p).unwrap();
    a.tick(1000).unwrap();
    a.mark_published(s).unwrap();
    a.store().adjusted_event(&canon(&p).unwrap(), true).unwrap();
    let changes = a.store().session(s).unwrap().changes.expect("reopened");
    assert_eq!(
        (changes.new_photos, changes.editor_edits, changes.app_edits),
        (0, 1, false)
    );
    a.review().keep_published(s).unwrap();
    assert!(!a.store().session(s).unwrap().dirty);
    assert_eq!(photo_of(&a, &p).1, "main", "no photo is touched");
    a.history().undo(s).unwrap();
    assert!(a.store().session(s).unwrap().dirty, "undo reopens it");
}

/// An edit in the app after publishing is named as such.
#[test]
fn a10_an_edit_in_the_app_after_publishing_is_named() {
    let a = app();
    let s = session(&a, "K1");
    a.store().set_active_session_id(Some(s)).unwrap();
    let p = file(&a, "K1/a.ARW", b"raw");
    a.watcher_event(&p).unwrap();
    a.tick(1000).unwrap();
    a.mark_published(s).unwrap();
    let (id, _) = photo_of(&a, &p);
    a.review()
        .classify(id, psm_core::domain::photo::Classification::Condition)
        .unwrap();
    let changes = a.store().session(s).unwrap().changes.expect("reopened");
    assert_eq!(
        (changes.new_photos, changes.editor_edits, changes.app_edits),
        (0, 0, true)
    );
}

/// "Capture One sometimes stops opening the session": switch requests made one after another
/// apply in that order; one a newer request has overtaken is skipped, never applied last.
#[test]
fn a10_capture_target_switches_apply_in_the_order_asked_and_the_last_one_holds() {
    let order = Arc::new(SwitchOrder::new());
    let applied = Arc::new(std::sync::Mutex::new(Vec::new()));
    let first = order.ticket();
    let second = order.ticket();
    // The second request's worker runs first; the first one's worker finishes afterwards.
    let log = applied.clone();
    order
        .run(second, "open session 2", || {
            log.lock().unwrap().push(2);
            Ok(())
        })
        .unwrap();
    let log = applied.clone();
    order
        .run(first, "open session 1", || {
            log.lock().unwrap().push(1);
            Ok(())
        })
        .unwrap();
    assert_eq!(
        *applied.lock().unwrap(),
        vec![2],
        "the stale request is skipped"
    );
    let third = order.ticket();
    let log = applied.clone();
    order
        .run(third, "leave session", || {
            log.lock().unwrap().push(3);
            Ok(())
        })
        .unwrap();
    assert_eq!(*applied.lock().unwrap(), vec![2, 3]);
}
