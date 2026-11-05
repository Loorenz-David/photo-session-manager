use psm_core::{
    Error,
    domain::photo::Classification as C,
    ports::clock::Clock,
    services::history::{Direction, HistoryState},
};
use psm_testkit::TestApp;

fn app() -> TestApp {
    TestApp::new().unwrap()
}
fn session(a: &TestApp, name: &str) -> i64 {
    let (item, _) = a.items().create_local(name, name, "object", None).unwrap();
    a.sessions().create(item.id).unwrap().session.id
}
fn add(a: &TestApp, s: i64, name: &str) -> i64 {
    a.add_photo(s, name).unwrap().id
}
fn main(a: &TestApp, s: i64, names: &[&str]) -> Vec<i64> {
    names
        .iter()
        .map(|n| {
            let id = add(a, s, n);
            a.review().classify(id, C::Main).unwrap();
            id
        })
        .collect()
}
fn state(a: &TestApp, s: i64, c: C) -> Vec<(i64, Option<i64>)> {
    let db = rusqlite::Connection::open(a.app.paths.db()).unwrap();
    let mut q=db.prepare("SELECT id,position FROM photos WHERE session_id=?1 AND classification=?2 ORDER BY position,id").unwrap();
    q.query_map(rusqlite::params![s, c.as_str()], |r| {
        Ok((r.get(0)?, r.get(1)?))
    })
    .unwrap()
    .map(Result::unwrap)
    .collect()
}
fn seq(a: &TestApp, s: i64) -> i64 {
    a.store().session(s).unwrap().session.change_seq
}
fn hs(a: &TestApp, s: i64) -> HistoryState {
    a.history().state(s)
}
fn err(e: Error, direction: &str, label: &str, reason: &str) {
    match e {
        Error::StepCannotApply {
            direction: d,
            label: l,
            reason: r,
        } => assert_eq!(
            (d.as_str(), l.as_str(), r.as_str()),
            (direction, label, reason)
        ),
        other => panic!("{other:?}"),
    }
}

#[test]
fn c1a_limit_25() {
    let a = app();
    let s = session(&a, "s");
    let ids: Vec<_> = (1..=26).map(|n| add(&a, s, &format!("u{n}"))).collect();
    for id in &ids {
        a.review().classify(*id, C::Main).unwrap();
    }
    for _ in 0..25 {
        a.history().undo(s).unwrap();
    }
    assert_eq!(state(&a, s, C::Main), vec![(ids[0], Some(1))]);
    assert_eq!(
        state(&a, s, C::Unreviewed),
        ids[1..].iter().map(|id| (*id, None)).collect::<Vec<_>>()
    );
    assert!(matches!(a.history().undo(s), Err(Error::NothingToUndo)));
}
#[test]
fn c1b_new_step_clears_redo() {
    let a = app();
    let s = session(&a, "s");
    let p = main(&a, s, &["a", "b"]);
    a.history().undo(s).unwrap();
    assert_eq!(hs(&a, s).redo.as_deref(), Some("Approve 02"));
    let c = add(&a, s, "c");
    a.review().classify(c, C::Condition).unwrap();
    assert_eq!(
        hs(&a, s),
        HistoryState {
            undo: Some("Condition 01".into()),
            redo: None
        }
    );
    assert!(matches!(a.history().redo(s), Err(Error::NothingToRedo)));
    assert_eq!(state(&a, s, C::Main), vec![(p[0], Some(1))]);
}
#[test]
fn c1c_session_isolation() {
    let a = app();
    let s = session(&a, "s");
    let t = session(&a, "t");
    main(&a, s, &["a"]);
    assert_eq!(hs(&a, s).undo.as_deref(), Some("Approve 01"));
    assert_eq!(hs(&a, t), HistoryState::default());
    assert!(matches!(a.history().undo(t), Err(Error::NothingToUndo)));
}
#[test]
fn c1d_state_labels() {
    let a = app();
    let s = session(&a, "s");
    let p = main(&a, s, &["a", "b"]);
    a.review().reorder(p[1], 1).unwrap();
    assert_eq!(
        hs(&a, s),
        HistoryState {
            undo: Some("Reorder 02 → 01".into()),
            redo: None
        }
    );
    a.history().undo(s).unwrap();
    assert_eq!(
        hs(&a, s),
        HistoryState {
            undo: Some("Approve 02".into()),
            redo: Some("Reorder 02 → 01".into())
        }
    );
    a.history().undo(s).unwrap();
    assert_eq!(
        hs(&a, s),
        HistoryState {
            undo: Some("Approve 01".into()),
            redo: Some("Approve 02".into())
        }
    );
}
#[test]
fn c2a_classify_undo_mirror_and_sequence() {
    let a = app();
    let s = session(&a, "s");
    let p = main(&a, s, &["a", "b", "c"]);
    let u = add(&a, s, "u");
    let n = seq(&a, s);
    let j = a.store().count_jobs("mirror_tag", "queued", None).unwrap();
    a.review().classify(u, C::Main).unwrap();
    a.clock.advance(1000);
    let t = a.clock.now_ms();
    let out = a.history().undo(s).unwrap();
    assert_eq!(
        (out.direction, out.label, out.active_photo_id),
        (Direction::Undone, "Approve 04".into(), Some(u))
    );
    assert_eq!(
        state(&a, s, C::Main),
        p.iter()
            .enumerate()
            .map(|(i, id)| (*id, Some(i as i64 + 1)))
            .collect::<Vec<_>>()
    );
    assert_eq!(state(&a, s, C::Unreviewed), vec![(u, None)]);
    assert_eq!(seq(&a, s), n + 2);
    assert_eq!(
        a.store().count_jobs("mirror_tag", "queued", None).unwrap(),
        j + 1
    );
    let db = rusqlite::Connection::open(a.app.paths.db()).unwrap();
    let raw = a.store().photo(u).unwrap().raw_path;
    let times:(i64,i64)=db.query_row("SELECT next_attempt_at,updated_at FROM jobs WHERE kind='mirror_tag' AND target=?1 AND status='queued'",[raw],|r|Ok((r.get(0)?,r.get(1)?))).unwrap();
    assert_eq!(times, (t, t));
}
#[test]
fn c2b_delete_undo_clamps_after_drop() {
    let a = app();
    let s = session(&a, "s");
    let t = session(&a, "t");
    let p = main(&a, s, &["a", "b", "c"]);
    a.review().delete(p[2]).unwrap();
    a.review().move_photos(&p[..2], t).unwrap();
    a.mark_published(s).unwrap();
    err(
        a.history().undo(s).unwrap_err(),
        "undo",
        "Move 2 photos to t",
        "that session is published",
    );
    let out = a.history().undo(s).unwrap();
    assert_eq!(
        (out.label, out.active_photo_id),
        ("Delete 03".into(), Some(p[2]))
    );
    assert_eq!(state(&a, s, C::Main), vec![(p[2], Some(1))]);
    assert!(state(&a, s, C::Deleted).is_empty());
    assert_eq!(
        state(&a, t, C::Main),
        vec![(p[0], Some(1)), (p[1], Some(2))]
    );
}
#[test]
fn c2c_unreviewed_delete_undo() {
    let a = app();
    let s = session(&a, "s");
    let u = add(&a, s, "u");
    a.review().delete(u).unwrap();
    let out = a.history().undo(s).unwrap();
    assert_eq!(
        (out.label, out.active_photo_id),
        ("Delete photo".into(), Some(u))
    );
    assert_eq!(state(&a, s, C::Unreviewed), vec![(u, None)]);
}
#[test]
fn c2d_classify_redo() {
    let a = app();
    let s = session(&a, "s");
    let p = main(&a, s, &["a", "b"]);
    a.review().classify(p[1], C::Condition).unwrap();
    a.history().undo(s).unwrap();
    let n = seq(&a, s);
    let out = a.history().redo(s).unwrap();
    assert_eq!(
        (out.direction, out.label),
        (Direction::Redone, "Condition 01".into())
    );
    assert_eq!(state(&a, s, C::Main), vec![(p[0], Some(1))]);
    assert_eq!(state(&a, s, C::Condition), vec![(p[1], Some(1))]);
    assert_eq!(seq(&a, s), n + 1);
    assert_eq!(
        hs(&a, s),
        HistoryState {
            undo: Some("Condition 01".into()),
            redo: None
        }
    );
}
#[test]
fn c2e_changed_photo_drops_step() {
    let a = app();
    let s = session(&a, "s");
    let p = main(&a, s, &["a"]);
    a.review().classify(p[0], C::Condition).unwrap();
    a.store()
        .transaction(&mut |tx| tx.set_photo_review(p[0], s, C::Main, Some(1)))
        .unwrap();
    err(
        a.history().undo(s).unwrap_err(),
        "undo",
        "Condition 01",
        "that photo already changed",
    );
    assert_eq!(state(&a, s, C::Main), vec![(p[0], Some(1))]);
    assert!(state(&a, s, C::Condition).is_empty());
    assert_eq!(
        hs(&a, s),
        HistoryState {
            undo: Some("Approve 01".into()),
            redo: None
        }
    );
}
#[test]
fn c3a_reorder_undo() {
    let a = app();
    let s = session(&a, "s");
    let p = main(&a, s, &["a", "b", "c"]);
    a.review().reorder(p[2], 1).unwrap();
    let n = seq(&a, s);
    a.history().undo(s).unwrap();
    assert_eq!(
        state(&a, s, C::Main),
        vec![(p[0], Some(1)), (p[1], Some(2)), (p[2], Some(3))]
    );
    assert_eq!(seq(&a, s), n + 1);
    assert_eq!(hs(&a, s).redo.as_deref(), Some("Reorder 03 → 01"));
}
#[test]
fn c3b_reorder_redo() {
    let a = app();
    let s = session(&a, "s");
    let p = main(&a, s, &["a", "b", "c"]);
    a.review().reorder(p[2], 1).unwrap();
    a.history().undo(s).unwrap();
    a.history().redo(s).unwrap();
    assert_eq!(
        state(&a, s, C::Main),
        vec![(p[2], Some(1)), (p[0], Some(2)), (p[1], Some(3))]
    );
}
#[test]
fn c4a_move_undo() {
    let a = app();
    let s = session(&a, "s");
    let t = session(&a, "t");
    let p = main(&a, s, &["a", "b", "c"]);
    let x = main(&a, t, &["x"]);
    a.review().move_photos(&[p[1]], t).unwrap();
    assert_eq!(hs(&a, s).undo.as_deref(), Some("Move to t"));
    assert_eq!(hs(&a, t).undo.as_deref(), Some("Approve 01"));
    let (ns, nt) = (seq(&a, s), seq(&a, t));
    let out = a.history().undo(s).unwrap();
    assert_eq!(
        state(&a, s, C::Main),
        vec![(p[0], Some(1)), (p[1], Some(2)), (p[2], Some(3))]
    );
    assert_eq!(state(&a, t, C::Main), vec![(x[0], Some(1))]);
    assert_eq!((seq(&a, s), seq(&a, t)), (ns + 1, nt + 1));
    assert_eq!(out.sessions.len(), 2);
}
#[test]
fn c4b_copy_undo_deletes_row_and_composition() {
    let a = app();
    let s = session(&a, "s");
    let t = session(&a, "t");
    let p = main(&a, s, &["a"]);
    let x = main(&a, t, &["x"]);
    a.review().copy_photos(&p, t).unwrap();
    let copy = state(&a, t, C::Main)[1].0;
    let nt = seq(&a, t);
    a.history().undo(s).unwrap();
    assert!(matches!(a.store().photo(copy), Err(Error::NotFound(_))));
    let db = rusqlite::Connection::open(a.app.paths.db()).unwrap();
    let count: i64 = db
        .query_row(
            "SELECT count(*) FROM compositions WHERE photo_id=?1",
            [copy],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 0);
    assert_eq!(state(&a, t, C::Main), vec![(x[0], Some(1))]);
    assert_eq!(state(&a, s, C::Main), vec![(p[0], Some(1))]);
    assert_eq!(seq(&a, t), nt + 1);
    assert_eq!(hs(&a, s).redo.as_deref(), Some("Copy to t"));
}
#[test]
fn c4c_published_move_drop_then_noop_reorder() {
    let a = app();
    let s = session(&a, "s");
    let t = session(&a, "t");
    let p = main(&a, s, &["a", "b"]);
    a.review().reorder(p[1], 1).unwrap();
    a.review().move_photos(&[p[0]], t).unwrap();
    a.mark_published(s).unwrap();
    let n = seq(&a, s);
    let before = a.events.0.lock().unwrap().len();
    err(
        a.history().undo(s).unwrap_err(),
        "undo",
        "Move to t",
        "that session is published",
    );
    let events = a.events.0.lock().unwrap();
    let changes: Vec<_> = events[before..]
        .iter()
        .filter_map(|e| {
            if let psm_core::events::AppEvent::HistoryChanged {
                session_id,
                undo,
                redo,
            } = e
            {
                (*session_id == s).then_some((undo.clone(), redo.clone()))
            } else {
                None
            }
        })
        .collect();
    assert_eq!(changes, vec![(Some("Reorder 02 → 01".into()), None)]);
    drop(events);
    assert_eq!(state(&a, t, C::Main), vec![(p[0], Some(1))]);
    assert_eq!(state(&a, s, C::Main), vec![(p[1], Some(1))]);
    assert_eq!(hs(&a, s).undo.as_deref(), Some("Reorder 02 → 01"));
    let out = a.history().undo(s).unwrap();
    assert_eq!(out.label, "Reorder 02 → 01");
    assert_eq!(state(&a, s, C::Main), vec![(p[1], Some(1))]);
    assert_eq!(seq(&a, s), n);
}
#[test]
fn c4d_published_copy_drop() {
    let a = app();
    let s = session(&a, "s");
    let t = session(&a, "t");
    let p = main(&a, s, &["a"]);
    main(&a, t, &["x"]);
    a.review().copy_photos(&p, t).unwrap();
    a.mark_published(t).unwrap();
    err(
        a.history().undo(s).unwrap_err(),
        "undo",
        "Copy to t",
        "that session is published",
    );
    assert_eq!(state(&a, t, C::Main).len(), 2);
    assert_eq!(hs(&a, s).undo.as_deref(), Some("Approve 01"));
}
#[test]
fn c4e_move_redo() {
    let a = app();
    let s = session(&a, "s");
    let t = session(&a, "t");
    let p = main(&a, s, &["a", "b", "c"]);
    let x = main(&a, t, &["x"]);
    a.review().move_photos(&[p[1]], t).unwrap();
    a.history().undo(s).unwrap();
    a.history().redo(s).unwrap();
    assert_eq!(
        state(&a, s, C::Main),
        vec![(p[0], Some(1)), (p[2], Some(2))]
    );
    assert_eq!(
        state(&a, t, C::Main),
        vec![(x[0], Some(1)), (p[1], Some(2))]
    );
    assert_eq!(
        hs(&a, s),
        HistoryState {
            undo: Some("Move to t".into()),
            redo: None
        }
    );
}
#[test]
fn c4f_multi_move_order() {
    let a = app();
    let s = session(&a, "s");
    let t = session(&a, "t");
    let p = main(&a, s, &["a", "b", "c", "d"]);
    let x = main(&a, t, &["x"]);
    a.review().move_photos(&p[1..3], t).unwrap();
    assert_eq!(hs(&a, s).undo.as_deref(), Some("Move 2 photos to t"));
    a.history().undo(s).unwrap();
    assert_eq!(
        state(&a, s, C::Main),
        p.iter()
            .enumerate()
            .map(|(i, id)| (*id, Some(i as i64 + 1)))
            .collect::<Vec<_>>()
    );
    assert_eq!(state(&a, t, C::Main), vec![(x[0], Some(1))]);
    a.history().redo(s).unwrap();
    assert_eq!(
        state(&a, t, C::Main),
        vec![(x[0], Some(1)), (p[1], Some(2)), (p[2], Some(3))]
    );
    assert_eq!(
        state(&a, s, C::Main),
        vec![(p[0], Some(1)), (p[3], Some(2))]
    );
}
#[test]
fn c4g_confirmed_running_publish_blocks_copy_undo() {
    let a = app();
    let s = session(&a, "s");
    let t = session(&a, "t");
    let p = main(&a, s, &["a"]);
    main(&a, t, &["x"]);
    a.review().copy_photos(&p, t).unwrap();
    let copy = state(&a, t, C::Main)[1].0;
    a.insert_publish_item(t, "upload", 0, None).unwrap();
    let db = rusqlite::Connection::open(a.app.paths.db()).unwrap();
    db.execute("UPDATE publish_items SET confirmed_change_seq=(SELECT change_seq FROM sessions WHERE id=?1) WHERE session_id=?1",[t]).unwrap();
    err(
        a.history().undo(s).unwrap_err(),
        "undo",
        "Copy to t",
        "that session is published",
    );
    assert_eq!(state(&a, t, C::Main)[1], (copy, Some(2)));
    assert_eq!(hs(&a, s).undo.as_deref(), Some("Approve 01"));
}
#[test]
fn c4h_raw_collision_blocks_move_undo() {
    let a = app();
    let s = session(&a, "s");
    let t = session(&a, "t");
    let p = main(&a, s, &["a"]);
    a.review().move_photos(&p, t).unwrap();
    a.review().copy_photos(&p, s).unwrap();
    err(
        a.history().undo(s).unwrap_err(),
        "undo",
        "Move to t",
        "that session already has this photo",
    );
    assert_eq!(state(&a, t, C::Main), vec![(p[0], Some(1))]);
    assert_eq!(state(&a, s, C::Main).len(), 1);
    assert_eq!(hs(&a, s).undo.as_deref(), Some("Approve 01"));
    a.history().undo(s).unwrap();
}
#[test]
fn c4i_copy_undo_repoints_mirror() {
    let a = app();
    let s = session(&a, "s");
    let t = session(&a, "t");
    let p = main(&a, s, &["a"]);
    main(&a, t, &["x"]);
    a.review().copy_photos(&p, t).unwrap();
    a.clock.advance(1000);
    let now = a.clock.now_ms();
    a.history().undo(s).unwrap();
    let db = rusqlite::Connection::open(a.app.paths.db()).unwrap();
    let raw = a.store().photo(p[0]).unwrap().raw_path;
    let (payload,due):(String,i64)=db.query_row("SELECT payload,next_attempt_at FROM jobs WHERE kind='mirror_tag' AND target=?1 AND status='queued'",[raw],|r|Ok((r.get(0)?,r.get(1)?))).unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&payload).unwrap()["photo_id"],
        p[0]
    );
    assert_eq!(due, now);
    a.tick(0).unwrap();
    assert_eq!(
        a.store().count_jobs("mirror_tag", "failed", None).unwrap(),
        0
    );
}
#[test]
fn c5a_arrival_create_settings_not_steps() {
    let mut a = TestApp::with_fake_document().unwrap();
    let s = session(&a, "s");
    let raw = a.dir.path().join("Capture One Session/Capture/s/n.ARW");
    std::fs::create_dir_all(raw.parent().unwrap()).unwrap();
    std::fs::write(&raw, b"raw").unwrap();
    a.deliver_hook(
        &serde_json::json!({"hook":"capture_done","argv":[raw.to_string_lossy()]}).to_string(),
    )
    .unwrap();
    assert_eq!(a.store().photos_in_session(s).unwrap().len(), 1);
    let t = session(&a, "t");
    a.settings_set(&serde_json::json!({"theme":"dark"}))
        .unwrap();
    assert_eq!(hs(&a, s), HistoryState::default());
    assert_eq!(hs(&a, t), HistoryState::default());
}
#[test]
fn c5b_settings_rebuild_keeps_history() {
    let mut a = app();
    let s = session(&a, "s");
    let p = main(&a, s, &["a"]);
    a.settings_set(&serde_json::json!({"sku_prefix":"PX"}))
        .unwrap();
    assert_eq!(hs(&a, s).undo.as_deref(), Some("Approve 01"));
    a.history().undo(s).unwrap();
    assert_eq!(state(&a, s, C::Unreviewed), vec![(p[0], None)]);
}
#[test]
fn c5c_quit_clears_history() {
    let mut a = app();
    let s = session(&a, "s");
    let p = main(&a, s, &["a"]);
    a.kill_and_restart().unwrap();
    assert_eq!(hs(&a, s), HistoryState::default());
    assert!(matches!(a.history().undo(s), Err(Error::NothingToUndo)));
    assert_eq!(state(&a, s, C::Main), vec![(p[0], Some(1))]);
}
#[test]
fn c6a_history_changed_events() {
    let a = app();
    let s = session(&a, "s");
    main(&a, s, &["a"]);
    a.history().undo(s).unwrap();
    a.history().redo(s).unwrap();
    let events = a.events.0.lock().unwrap();
    let states: Vec<_> = events
        .iter()
        .filter_map(|e| {
            if let psm_core::events::AppEvent::HistoryChanged {
                session_id,
                undo,
                redo,
            } = e
            {
                (*session_id == s).then_some((undo.clone(), redo.clone()))
            } else {
                None
            }
        })
        .collect();
    assert_eq!(
        states,
        vec![
            (Some("Approve 01".into()), None),
            (None, Some("Approve 01".into())),
            (Some("Approve 01".into()), None)
        ]
    );
}
