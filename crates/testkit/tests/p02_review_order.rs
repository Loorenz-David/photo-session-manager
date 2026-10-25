use psm_core::{Error, domain::photo::Classification as C};
use psm_testkit::TestApp;

fn app() -> TestApp {
    TestApp::new().unwrap()
}
fn session(app: &TestApp, name: &str) -> i64 {
    let (item, _) = app
        .items()
        .create_local(name, name, "object", None)
        .unwrap();
    app.sessions().create(item.id).unwrap().session.id
}
fn add(app: &TestApp, s: i64, name: &str, class: C) -> i64 {
    let id = app.add_photo(s, name).unwrap().id;
    if class != C::Unreviewed {
        app.review().classify(id, class).unwrap();
    }
    id
}
fn state(app: &TestApp, s: i64, class: C) -> Vec<(i64, Option<i64>)> {
    let conn = rusqlite::Connection::open(app.app.paths.db()).unwrap();
    let mut stmt = conn.prepare("SELECT id, position FROM photos WHERE session_id=?1 AND classification=?2 ORDER BY position,id").unwrap();
    stmt.query_map(rusqlite::params![s, class.as_str()], |row| {
        Ok((row.get(0)?, row.get(1)?))
    })
    .unwrap()
    .map(Result::unwrap)
    .collect()
}
fn seq(app: &TestApp, s: i64) -> i64 {
    app.store().session(s).unwrap().session.change_seq
}
fn main(app: &TestApp, s: i64, names: &[&str]) -> Vec<i64> {
    names
        .iter()
        .map(|name| add(app, s, name, C::Main))
        .collect()
}

#[test]
fn c1a_classify_appends_main() {
    let a = app();
    let s = session(&a, "s1");
    let ids = main(&a, s, &["a", "b"]);
    let p = add(&a, s, "p", C::Unreviewed);
    a.review().classify(p, C::Main).unwrap();
    assert_eq!(
        state(&a, s, C::Main),
        vec![(ids[0], Some(1)), (ids[1], Some(2)), (p, Some(3))]
    );
}
#[test]
fn c1b_classify_first_condition() {
    let a = app();
    let s = session(&a, "s1");
    let p = add(&a, s, "p", C::Unreviewed);
    a.review().classify(p, C::Condition).unwrap();
    assert_eq!(state(&a, s, C::Condition), vec![(p, Some(1))]);
}
#[test]
fn c1c_main_to_condition_closes_gap() {
    let a = app();
    let s = session(&a, "s1");
    let ids = main(&a, s, &["a", "b", "c"]);
    let d = add(&a, s, "d", C::Condition);
    a.review().classify(ids[1], C::Condition).unwrap();
    assert_eq!(
        state(&a, s, C::Main),
        vec![(ids[0], Some(1)), (ids[2], Some(2))]
    );
    assert_eq!(
        state(&a, s, C::Condition),
        vec![(d, Some(1)), (ids[1], Some(2))]
    );
}
#[test]
fn c1d_condition_to_main_closes_gap() {
    let a = app();
    let s = session(&a, "s1");
    let x = main(&a, s, &["a"])[0];
    let d = add(&a, s, "d", C::Condition);
    let e = add(&a, s, "e", C::Condition);
    a.review().classify(d, C::Main).unwrap();
    assert_eq!(state(&a, s, C::Main), vec![(x, Some(1)), (d, Some(2))]);
    assert_eq!(state(&a, s, C::Condition), vec![(e, Some(1))]);
}
#[test]
fn c1e_same_classification_is_noop() {
    let a = app();
    let s = session(&a, "s1");
    let ids = main(&a, s, &["a", "b"]);
    let before = seq(&a, s);
    a.review().classify(ids[0], C::Main).unwrap();
    assert_eq!(
        state(&a, s, C::Main),
        vec![(ids[0], Some(1)), (ids[1], Some(2))]
    );
    assert_eq!(seq(&a, s), before);
}
#[test]
fn c2a_reorder_forward() {
    let a = app();
    let s = session(&a, "s1");
    let p = main(&a, s, &["a", "b", "c", "d"]);
    a.review().reorder(p[0], 3).unwrap();
    assert_eq!(
        state(&a, s, C::Main),
        vec![
            (p[1], Some(1)),
            (p[2], Some(2)),
            (p[0], Some(3)),
            (p[3], Some(4))
        ]
    );
}
#[test]
fn c2b_reorder_backward() {
    let a = app();
    let s = session(&a, "s1");
    let p = main(&a, s, &["a", "b", "c", "d"]);
    a.review().reorder(p[3], 2).unwrap();
    assert_eq!(
        state(&a, s, C::Main),
        vec![
            (p[0], Some(1)),
            (p[3], Some(2)),
            (p[1], Some(3)),
            (p[2], Some(4))
        ]
    );
}
#[test]
fn c2c_reorder_clamps_high() {
    let a = app();
    let s = session(&a, "s1");
    let p = main(&a, s, &["a", "b", "c"]);
    a.review().reorder(p[0], 9).unwrap();
    assert_eq!(
        state(&a, s, C::Main),
        vec![(p[1], Some(1)), (p[2], Some(2)), (p[0], Some(3))]
    );
}
#[test]
fn c2d_reorder_clamps_low() {
    let a = app();
    let s = session(&a, "s1");
    let p = main(&a, s, &["a", "b", "c"]);
    a.review().reorder(p[2], 0).unwrap();
    assert_eq!(
        state(&a, s, C::Main),
        vec![(p[2], Some(1)), (p[0], Some(2)), (p[1], Some(3))]
    );
}
#[test]
fn c3a_delete_hides_and_closes_gap() {
    let a = app();
    let s = session(&a, "s1");
    let p = main(&a, s, &["a", "b", "c"]);
    a.review().delete(p[1]).unwrap();
    assert_eq!(
        state(&a, s, C::Main),
        vec![(p[0], Some(1)), (p[2], Some(2))]
    );
    assert_eq!(state(&a, s, C::Deleted), vec![(p[1], None)]);
}
#[test]
fn c3b_undo_restores_position_and_active() {
    let a = app();
    let s = session(&a, "s1");
    let p = main(&a, s, &["a", "b", "c"]);
    a.review().delete(p[1]).unwrap();
    let result = a.review().undo(s).unwrap();
    assert_eq!(result.active_photo_id, Some(p[1]));
    assert_eq!(
        state(&a, s, C::Main),
        vec![(p[0], Some(1)), (p[1], Some(2)), (p[2], Some(3))]
    );
}
#[test]
fn c3c_only_last_delete_undoes() {
    let a = app();
    let s = session(&a, "s1");
    let p = main(&a, s, &["a", "b", "c"]);
    for id in [p[2], p[0], p[1]] {
        a.review().delete(id).unwrap();
    }
    a.review().undo(s).unwrap();
    assert_eq!(state(&a, s, C::Main), vec![(p[1], Some(1))]);
    assert_eq!(state(&a, s, C::Deleted), vec![(p[0], None), (p[2], None)]);
}
#[test]
fn c3d_undo_clamps_prior_position() {
    let a = app();
    let s = session(&a, "s1");
    let p = main(&a, s, &["a", "b", "c"]);
    a.review().delete(p[2]).unwrap();
    a.review().classify(p[0], C::Condition).unwrap();
    a.review().classify(p[1], C::Condition).unwrap();
    a.review().undo(s).unwrap();
    assert_eq!(state(&a, s, C::Main), vec![(p[2], Some(1))]);
    assert_eq!(
        state(&a, s, C::Condition),
        vec![(p[0], Some(1)), (p[1], Some(2))]
    );
}
#[test]
fn c3e_empty_undo_fails_without_change() {
    let a = app();
    let s = session(&a, "s1");
    let before = a.store().session(s).unwrap();
    assert!(matches!(a.review().undo(s), Err(Error::NothingToUndo)));
    assert_eq!(a.store().session(s).unwrap(), before);
}
#[test]
fn c3f_unreviewed_undo_stays_unpositioned() {
    let a = app();
    let s = session(&a, "s1");
    let u = add(&a, s, "u", C::Unreviewed);
    a.review().delete(u).unwrap();
    let result = a.review().undo(s).unwrap();
    assert_eq!(result.active_photo_id, Some(u));
    assert_eq!(state(&a, s, C::Unreviewed), vec![(u, None)]);
}
#[test]
fn c4a_move_keeps_source_order() {
    let a = app();
    let s = session(&a, "s1");
    let t = session(&a, "s2");
    let p = main(&a, s, &["a", "b", "c"]);
    let x = main(&a, t, &["x"])[0];
    a.review().move_photos(&[p[1], p[0]], t).unwrap();
    assert_eq!(
        state(&a, t, C::Main),
        vec![(x, Some(1)), (p[0], Some(2)), (p[1], Some(3))]
    );
    assert_eq!(state(&a, s, C::Main), vec![(p[2], Some(1))]);
}
#[test]
fn c4b_copy_preserves_raw_and_hash() {
    let a = app();
    let s = session(&a, "s1");
    let t = session(&a, "s2");
    let p = main(&a, s, &["a"])[0];
    let before = a.store().photo(p).unwrap();
    a.review().copy_photos(&[p], t).unwrap();
    let copied = state(&a, t, C::Main)[0];
    let after = a.store().photo(copied.0).unwrap();
    assert_ne!(after.id, p);
    assert_eq!(after.raw_path, before.raw_path);
    assert_eq!(after.raw_hash, before.raw_hash);
    assert_eq!(after.position, Some(1));
    assert_eq!(a.store().photo(p).unwrap(), before);
}
#[test]
fn c4c_move_unreviewed_keeps_null_position() {
    let a = app();
    let s = session(&a, "s1");
    let t = session(&a, "s2");
    let u = add(&a, s, "u", C::Unreviewed);
    a.review().move_photos(&[u], t).unwrap();
    assert_eq!(state(&a, t, C::Unreviewed), vec![(u, None)]);
    assert!(state(&a, s, C::Unreviewed).is_empty());
}
#[test]
fn c4d_existing_raw_refuses_atomically() {
    let a = app();
    let s = session(&a, "s1");
    let t = session(&a, "s2");
    let p = main(&a, s, &["a"])[0];
    let q = a.add_photo(t, "a").unwrap().id;
    let before = (
        a.store().photo(p).unwrap(),
        a.store().photo(q).unwrap(),
        seq(&a, s),
        seq(&a, t),
    );
    assert!(matches!(
        a.review().move_photos(&[p], t),
        Err(Error::TargetHasRaw)
    ));
    assert!(matches!(
        a.review().copy_photos(&[p], t),
        Err(Error::TargetHasRaw)
    ));
    assert_eq!(
        (
            a.store().photo(p).unwrap(),
            a.store().photo(q).unwrap(),
            seq(&a, s),
            seq(&a, t)
        ),
        before
    );
}
#[test]
fn c4e_published_target_refuses_atomically() {
    let a = app();
    let s = session(&a, "s1");
    let t = session(&a, "s2");
    let p = main(&a, s, &["a"])[0];
    a.mark_published(t).unwrap();
    let before = (a.store().photo(p).unwrap(), seq(&a, s), seq(&a, t));
    assert!(matches!(
        a.review().move_photos(&[p], t),
        Err(Error::TargetPublished)
    ));
    assert_eq!(
        (a.store().photo(p).unwrap(), seq(&a, s), seq(&a, t)),
        before
    );
}
#[test]
fn c4f_move_out_of_published_makes_dirty() {
    let a = app();
    let s = session(&a, "s1");
    let t = session(&a, "s2");
    let p = main(&a, s, &["a"])[0];
    a.mark_published(s).unwrap();
    a.review().move_photos(&[p], t).unwrap();
    assert_eq!(a.store().photo(p).unwrap().session_id, t);
    assert!(a.store().session(s).unwrap().dirty);
}
#[test]
fn c5a_each_operation_bumps_touched_sessions_once() {
    let a = app();
    let s = session(&a, "s1");
    let t = session(&a, "s2");
    let p = main(&a, s, &["a", "b"]);
    let u = add(&a, s, "u", C::Unreviewed);
    let n = seq(&a, s);
    a.review().classify(u, C::Main).unwrap();
    assert_eq!(seq(&a, s), n + 1);
    let n = seq(&a, s);
    a.review().reorder(u, 1).unwrap();
    assert_eq!(seq(&a, s), n + 1);
    let n = seq(&a, s);
    a.review().delete(u).unwrap();
    assert_eq!(seq(&a, s), n + 1);
    let n = seq(&a, s);
    a.review().undo(s).unwrap();
    assert_eq!(seq(&a, s), n + 1);
    let (ns, nt) = (seq(&a, s), seq(&a, t));
    a.review().move_photos(&[p[0]], t).unwrap();
    assert_eq!((seq(&a, s), seq(&a, t)), (ns + 1, nt + 1));
    let (ns, nt) = (seq(&a, s), seq(&a, t));
    a.review().copy_photos(&[p[1]], t).unwrap();
    assert_eq!((seq(&a, s), seq(&a, t)), (ns, nt + 1));
}
#[test]
fn c5b_photo_change_after_publish_is_dirty() {
    let a = app();
    let s = session(&a, "s1");
    for name in ["q1", "q2", "q3", "q4"] {
        add(&a, s, name, C::Unreviewed);
    }
    let p = add(&a, s, "p", C::Unreviewed);
    a.mark_published(s).unwrap();
    let published_seq = seq(&a, s);
    assert_eq!(published_seq, 5);
    a.review().classify(p, C::Main).unwrap();
    let dto = a.store().session(s).unwrap();
    assert_eq!(dto.session.published_seq, published_seq);
    assert_eq!(dto.session.change_seq, published_seq + 1);
    assert!(dto.dirty);
}
#[test]
fn c5c_dirty_requires_published_and_newer_change() {
    let a = app();
    let s = session(&a, "s1");
    let u = session(&a, "s2");
    let p = add(&a, u, "p", C::Unreviewed);
    a.mark_published(s).unwrap();
    a.review().classify(p, C::Main).unwrap();
    assert!(!a.store().session(s).unwrap().dirty);
    assert!(!a.store().session(u).unwrap().dirty);
    assert_eq!(
        a.store().session(s).unwrap().session.published_seq,
        seq(&a, s)
    );
    assert!(seq(&a, u) > 0);
}
#[test]
fn c6a_clock_stamps_all_touched_sessions() {
    let a = app();
    let s = session(&a, "s1");
    let t = session(&a, "s2");
    let p = main(&a, s, &["a"])[0];
    a.clock.advance(12345);
    a.review().move_photos(&[p], t).unwrap();
    assert_eq!(
        a.store().session(s).unwrap().session.last_activity_at,
        12345
    );
    assert_eq!(
        a.store().session(t).unwrap().session.last_activity_at,
        12345
    );
}
