use chrono::TimeZone;
use chrono_tz::Europe::Stockholm;
use psm_core::domain::{
    photo::Classification as C,
    session::{ListFilter, PublishStep as P, RowLabel as L, SessionStatus as S},
};
use psm_core::mechanisms::{
    status::{StatusInput, derive_status, progress_percent, row_label},
    time::group_by_activity,
};
use psm_testkit::TestApp;

fn input() -> StatusInput {
    StatusInput {
        latest_step: None,
        failed_step: None,
        published_at: None,
        dirty: false,
        main_count: 0,
        unreviewed_count: 0,
        non_deleted_missing_preview_count: 0,
        uploaded: 0,
        total: 0,
    }
}
fn app() -> TestApp {
    TestApp::new().unwrap()
}
fn session(a: &TestApp, sku: &str, title: &str) -> i64 {
    a.items()
        .create_local(sku, title, "object", None)
        .unwrap()
        .1
        .session
        .id
}
fn row(a: &TestApp, id: i64) -> psm_core::domain::session::SessionRow {
    a.sessions()
        .list(ListFilter::All, None)
        .unwrap()
        .groups
        .into_iter()
        .flat_map(|g| g.rows)
        .find(|r| r.id == id)
        .unwrap()
}
fn preview(a: &TestApp, photo_id: i64, path: &str) {
    let db = rusqlite::Connection::open(a.app.paths.db()).unwrap();
    db.execute(
        "UPDATE photos SET adjusted_preview_path=?1 WHERE id=?2",
        rusqlite::params![path, photo_id],
    )
    .unwrap();
}
fn add(a: &TestApp, id: i64, name: &str, c: C, has_preview: bool) -> i64 {
    let p = a.add_photo(id, name).unwrap().id;
    if c != C::Unreviewed {
        a.review().classify(p, c).unwrap();
    }
    if has_preview {
        preview(a, p, &format!("/{name}.jpg"));
    }
    p
}
fn local(y: i32, m: u32, d: u32, h: u32, min: u32) -> i64 {
    Stockholm
        .with_ymd_and_hms(y, m, d, h, min, 0)
        .single()
        .unwrap()
        .timestamp_millis()
}
fn group(rows: &[(i64, i64)], now: i64) -> Vec<psm_core::domain::session::ListGroup<i64>> {
    group_by_activity(rows, now, &Stockholm)
}
fn label_for(id: i64, at: i64, now: i64) -> String {
    group(&[(id, at)], now)[0].label.clone()
}

#[test]
fn c1a_active_steps_publish() {
    for step in [P::Prepare, P::Upload, P::Update, P::Describe] {
        let mut i = input();
        i.latest_step = Some(step);
        i.published_at = Some(1);
        i.dirty = false;
        assert_eq!(derive_status(i), S::Publishing);
    }
}
#[test]
fn c1b_failed_never_published() {
    let mut i = input();
    i.latest_step = Some(P::Failed);
    assert_eq!(derive_status(i), S::Failed);
}
#[test]
fn c1c_failed_published_clean() {
    let mut i = input();
    i.latest_step = Some(P::Failed);
    i.published_at = Some(1);
    assert_eq!(derive_status(i), S::Published);
}
#[test]
fn c1d_published_dirty_ready() {
    let mut i = input();
    i.published_at = Some(1);
    i.dirty = true;
    i.main_count = 1;
    assert_eq!(derive_status(i), S::Ready);
}
#[test]
fn c1e_published_dirty_unreviewed() {
    let mut i = input();
    i.published_at = Some(1);
    i.dirty = true;
    i.main_count = 1;
    i.unreviewed_count = 1;
    assert_eq!(derive_status(i), S::InProgress);
}
#[test]
fn c1f_unpublished_ready() {
    let mut i = input();
    i.main_count = 2;
    assert_eq!(derive_status(i), S::Ready);
}
#[test]
fn c1g_stale_preview_present() {
    let a = app();
    let s = session(&a, "S1", "oak");
    let p = add(&a, s, "p", C::Main, true);
    let raw = a.store().photo(p).unwrap().raw_path;
    let db = rusqlite::Connection::open(a.app.paths.db()).unwrap();
    db.execute(
        "INSERT INTO raw_state(raw_path,adjusted_gen,rendered_gen) VALUES (?1,2,1)",
        [raw],
    )
    .unwrap();
    assert_eq!(row(&a, s).status, S::Ready);
}
#[test]
fn c1h_missing_preview() {
    let mut i = input();
    i.main_count = 2;
    i.non_deleted_missing_preview_count = 1;
    assert_eq!(derive_status(i), S::InProgress);
}
#[test]
fn c1i_conditions_only() {
    let a = app();
    let s = session(&a, "S1", "oak");
    for name in ["c1", "c2", "c3"] {
        add(&a, s, name, C::Condition, true);
    }
    assert_eq!(row(&a, s).status, S::InProgress);
}
#[test]
fn c1j_empty() {
    assert_eq!(derive_status(input()), S::InProgress);
}
#[test]
fn c1k_deleted_missing_ignored() {
    let a = app();
    let s = session(&a, "S1", "oak");
    add(&a, s, "p", C::Main, true);
    add(&a, s, "d", C::Deleted, false);
    assert_eq!(row(&a, s).status, S::Ready);
}
#[test]
fn c1l_latest_confirmed_wins() {
    let a = app();
    let s = session(&a, "S1", "oak");
    a.insert_publish_item(s, "failed", 1, Some("upload"))
        .unwrap();
    a.insert_publish_item(s, "done", 2, None).unwrap();
    a.mark_published(s).unwrap();
    assert_eq!(row(&a, s).status, S::Published);
    assert_eq!(row(&a, s).label, L::Published);
}

#[test]
fn c2a_progress_steps() {
    let mut i = input();
    for (step, uploaded, total, expected) in [
        (P::Prepare, 0, 0, 0),
        (P::Upload, 2, 4, 42),
        (P::Update, 0, 0, 75),
        (P::Describe, 0, 0, 85),
    ] {
        i.latest_step = Some(step);
        i.uploaded = uploaded;
        i.total = total;
        assert_eq!(progress_percent(step, uploaded, total), expected);
        assert_eq!(row_label(i), L::Publishing(expected));
    }
}
#[test]
fn c2b_failed_upload_after_publish() {
    let mut i = input();
    i.latest_step = Some(P::Failed);
    i.failed_step = Some(P::Upload);
    i.published_at = Some(1);
    assert_eq!(row_label(i), L::PublishFailed);
}
#[test]
fn c2c_failed_describe() {
    let mut i = input();
    i.latest_step = Some(P::Failed);
    i.failed_step = Some(P::Describe);
    assert_eq!(row_label(i), L::DescriptionFailed);
}
#[test]
fn c2d_published_label() {
    let mut i = input();
    i.published_at = Some(1);
    assert_eq!(row_label(i), L::Published);
}
#[test]
fn c2e_ready_label() {
    let mut i = input();
    i.main_count = 1;
    assert_eq!(row_label(i), L::Ready);
}
#[test]
fn c2f_in_progress_count() {
    let mut i = input();
    i.unreviewed_count = 3;
    assert_eq!(row_label(i), L::InProgress(3));
}

#[test]
fn c3a_today_dst_day() {
    let at = local(2026, 10, 25, 9, 0);
    for now in [local(2026, 10, 25, 0, 30), local(2026, 10, 25, 23, 30)] {
        assert_eq!(label_for(1, at, now), "Today");
    }
}
#[test]
fn c3b_yesterday_dst_day() {
    let at = local(2026, 10, 24, 23, 59);
    for now in [local(2026, 10, 25, 0, 30), local(2026, 10, 25, 23, 30)] {
        assert_eq!(label_for(1, at, now), "Yesterday");
    }
}
#[test]
fn c3c_this_week() {
    let at = local(2026, 10, 19, 0, 0);
    for now in [local(2026, 10, 25, 0, 30), local(2026, 10, 25, 23, 30)] {
        assert_eq!(label_for(1, at, now), "This week");
    }
}
#[test]
fn c3d_last_week() {
    let at = local(2026, 10, 18, 23, 59);
    for now in [local(2026, 10, 25, 0, 30), local(2026, 10, 25, 23, 30)] {
        assert_eq!(label_for(1, at, now), "Last week");
    }
}
#[test]
fn c3e_month_groups_order() {
    for now in [local(2026, 10, 25, 0, 30), local(2026, 10, 25, 23, 30)] {
        let g = group(
            &[
                (1, local(2026, 10, 18, 23, 59)),
                (2, local(2026, 10, 11, 0, 0)),
                (3, local(2026, 9, 30, 0, 0)),
            ],
            now,
        );
        assert_eq!(
            g.iter().map(|x| x.label.as_str()).collect::<Vec<_>>(),
            vec!["Last week", "October 2026", "September 2026"]
        );
    }
}
#[test]
fn c3f_ties_by_descending_id() {
    for now in [local(2026, 10, 25, 0, 30), local(2026, 10, 25, 23, 30)] {
        let at = local(2026, 10, 25, 9, 0);
        assert_eq!(
            group(&[(7, at), (9, at), (3, at + 1000)], now)[0].rows,
            vec![3, 9, 7]
        );
    }
}

fn four(a: &TestApp) -> (i64, i64, i64, i64) {
    let x = session(a, "A", "A");
    let b = session(a, "B", "B");
    let c = session(a, "C", "C");
    let d = session(a, "D", "D");
    add(a, b, "b", C::Main, true);
    a.mark_published(c).unwrap();
    a.insert_publish_item(d, "prepare", 1, None).unwrap();
    (x, b, c, d)
}
#[test]
fn c4a_open_filter_and_counts() {
    let a = app();
    let (x, b, _c, d) = four(&a);
    let list = a.sessions().list(ListFilter::Open, None).unwrap();
    let ids = list
        .groups
        .into_iter()
        .flat_map(|g| g.rows)
        .map(|r| r.id)
        .collect::<Vec<_>>();
    assert_eq!(ids.len(), 3);
    assert!(ids.contains(&x) && ids.contains(&b) && ids.contains(&d));
    assert_eq!(
        (list.counts.open, list.counts.published, list.counts.all),
        (3, 1, 4)
    );
}
#[test]
fn c4b_published_filter() {
    let a = app();
    let (_x, _b, c, _d) = four(&a);
    let ids = a
        .sessions()
        .list(ListFilter::Published, None)
        .unwrap()
        .groups
        .into_iter()
        .flat_map(|g| g.rows)
        .map(|r| r.id)
        .collect::<Vec<_>>();
    assert_eq!(ids, vec![c]);
}
#[test]
fn c4c_search_title_or_sku() {
    let a = app();
    let oak = session(&a, "A", "Oak sideboard");
    session(&a, "B", "Teak desk");
    let sku = session(&a, "OAK-2", "Chair");
    let ids = a
        .sessions()
        .list(ListFilter::All, Some("oak".into()))
        .unwrap()
        .groups
        .into_iter()
        .flat_map(|g| g.rows)
        .map(|r| r.id)
        .collect::<Vec<_>>();
    assert_eq!(ids, vec![sku, oak]);
}
#[test]
fn c4d_unassigned_card_count() {
    let a = app();
    let s = session(&a, "A", "A");
    a.add_photo(1, "u1").unwrap();
    a.add_photo(1, "u2").unwrap();
    let list = a.sessions().list(ListFilter::Open, None).unwrap();
    assert_eq!(list.unassigned_count, 2);
    assert_eq!(
        list.groups
            .into_iter()
            .flat_map(|g| g.rows)
            .map(|r| r.id)
            .collect::<Vec<_>>(),
        vec![s]
    );
}
