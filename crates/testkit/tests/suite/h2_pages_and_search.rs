//! H2·U2 behaviour tests (`plans/H2_list_pictures_pages_search.md`, rows H2·C7): the page of
//! the sessions list at `SessionsService::page` on a `TestApp` whose sessions are created through
//! `ItemsService::create_local` at chosen clock times. Outcomes: the page's rows, groups, totals
//! and counts. (H2·C6 are inline unit tests of `page_of`; H2·C8 are desktop command tests.)

use psm_core::{
    domain::session::{ListCounts, ListFilter, SessionPage},
    ports::clock::Clock,
};
use psm_testkit::TestApp;

/// 2025-11-04T12:00:00Z: an hour after it is the same calendar day in every time zone from
/// UTC−11 to UTC+11.
const NOON_MS: i64 = 1_762_257_600_000;
const MINUTE: i64 = 60_000;

/// `n` sessions created one minute apart, oldest first: (id, SKU, created at).
fn sessions(a: &TestApp, n: usize) -> Vec<(i64, String, i64)> {
    a.clock.advance(NOON_MS);
    (0..n)
        .map(|i| {
            a.clock.advance(MINUTE);
            let sku = format!("PG-A{i:03}");
            let (_, session) = a
                .items()
                .create_local(&sku, &format!("Page item {i}"), "object", None)
                .unwrap();
            (session.session.id, sku, a.clock.now_ms())
        })
        .collect()
}

/// H2·C7's store: 60 sessions, the 30 oldest published. Newest first.
fn sixty() -> (TestApp, Vec<(i64, String, i64)>) {
    let a = TestApp::with_fake_shop().unwrap();
    let mut all = sessions(&a, 60);
    for (id, _, _) in &all[..30] {
        a.mark_published(*id).unwrap();
    }
    all.reverse();
    (a, all)
}

fn page(
    a: &TestApp,
    filter: ListFilter,
    query: Option<&str>,
    created_from: Option<i64>,
    only_ids: Option<&[i64]>,
    page: u32,
) -> SessionPage {
    a.sessions()
        .with_remote(a.remote())
        .page(filter, query.map(Into::into), created_from, only_ids, page)
        .unwrap()
}

fn ids(page: &SessionPage) -> Vec<i64> {
    page.groups
        .iter()
        .flat_map(|g| g.rows.iter().map(|r| r.id))
        .collect()
}

fn newest(all: &[(i64, String, i64)], range: std::ops::Range<usize>) -> Vec<i64> {
    all[range].iter().map(|s| s.0).collect()
}

#[test]
fn h2_c7a_the_first_page_holds_the_25_newest() {
    let (a, all) = sixty();
    let p = page(&a, ListFilter::All, None, None, None, 1);
    assert_eq!(ids(&p), newest(&all, 0..25));
    assert_eq!((p.total, p.pages, p.page), (60, 3, 1));
}

#[test]
fn h2_c7b_the_last_page_holds_the_10_oldest() {
    let (a, all) = sixty();
    let p = page(&a, ListFilter::All, None, None, None, 3);
    assert_eq!(ids(&p), newest(&all, 50..60));
}

#[test]
fn h2_c7c_a_search_finds_a_session_of_any_page() {
    let (a, all) = sixty();
    let (id, sku, _) = &all[39];
    let p = page(&a, ListFilter::All, Some(sku), None, None, 1);
    assert_eq!(ids(&p), vec![*id]);
    assert_eq!((p.total, p.pages), (1, 1));
}

#[test]
fn h2_c7d_the_filter_applies_before_the_cut() {
    let (a, _) = sixty();
    let p = page(&a, ListFilter::Published, None, None, None, 2);
    assert_eq!((ids(&p).len(), p.total, p.pages), (5, 30, 2));
}

#[test]
fn h2_c7e_the_time_filter() {
    let (a, all) = sixty();
    let p = page(&a, ListFilter::All, None, Some(all[9].2), None, 1);
    assert_eq!(p.total, 10);
}

#[test]
fn h2_c7f_to_review_keeps_only_the_given_ids() {
    let (a, all) = sixty();
    let wanted = [all[29].0, all[0].0, all[54].0];
    let p = page(&a, ListFilter::All, None, None, Some(&wanted), 1);
    assert_eq!(p.total, 3);
    assert_eq!(ids(&p), vec![all[0].0, all[29].0, all[54].0]);
}

#[test]
fn h2_c7g_groups_are_cut_and_keep_their_totals() {
    let a = TestApp::with_fake_shop().unwrap();
    sessions(&a, 30);
    let shape = |n: u32| {
        page(&a, ListFilter::All, None, None, None, n)
            .groups
            .iter()
            .map(|g| (g.label.clone(), g.rows.len(), g.total))
            .collect::<Vec<_>>()
    };
    assert_eq!(
        (shape(1), shape(2)),
        (
            vec![("Today".to_owned(), 25, 30)],
            vec![("Today".to_owned(), 5, 30)]
        )
    );
}

#[test]
fn h2_c7h_counts_are_over_all_sessions() {
    let (a, all) = sixty();
    let want = ListCounts {
        open: 30,
        published: 30,
        all: 60,
    };
    let searched = page(&a, ListFilter::All, Some(&all[39].1), None, None, 1);
    let published = page(&a, ListFilter::Published, None, None, None, 2);
    assert_eq!((searched.counts, published.counts), (want.clone(), want));
}
