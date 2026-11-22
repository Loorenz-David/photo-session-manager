//! C2·U1 (`docs/implementation/plans/C2_header_panel_notifications.md` C2·C1–C6): the panel's
//! list, the header counts, the sessions-list badges and the session match, through
//! `ConversationsService::{list, counts, session_badges}` on `TestApp::with_pricing()` with the
//! conversations brought into the copy by `tick` from `FakePricing`.

use psm_core::{
    domain::conversation::{
        ConversationCounts, ConversationList, ConversationRow, ConversationScope, ConversationTab,
        SessionBadge,
    },
    ports::{clock::Clock, conversations::ConversationStore},
};
use psm_integrations::session_delete;
use psm_testkit::{TestApp, fake_pricing::FakePricing, test_app::PRICING_KEY};

const DAY: i64 = 86_400_000;

/// A connected app (key saved, store matched).
fn connected() -> TestApp {
    let a = TestApp::with_pricing().unwrap();
    a.clock.advance(1_000_000);
    a.app.conversations.key_set(Some(PRICING_KEY)).unwrap();
    a
}

fn pricing(a: &TestApp) -> &FakePricing {
    a.fake_pricing()
}

fn store_id(a: &TestApp) -> String {
    a.app.store.sync().unwrap().store.unwrap().id
}

/// Runs a check to its end at the clock's time (the first connection, or a pull).
fn run_check(a: &TestApp) {
    let mut n = 0;
    while a.app.conversations.tick(a.clock.now_ms()).unwrap().more {
        n += 1;
        assert!(n < 50, "the check never completed");
    }
}

fn next_check(a: &TestApp) {
    a.clock.advance(10_000);
    run_check(a);
}

/// The pricer asks a (blocking) question on `sku`.
fn ask(a: &TestApp, sku: Option<&str>) -> String {
    pricing(a).ask(&store_id(a), sku, "Condition")
}

fn not_blocking(a: &TestApp, case: &str) {
    pricing(a).change_silently(case, |c| c.blocking = false);
}

fn iso(ms: i64) -> String {
    chrono::DateTime::from_timestamp_millis(ms)
        .unwrap()
        .format("%Y-%m-%dT%H:%M:%S%.3fZ")
        .to_string()
}

/// A session whose item has `sku`; returns the session id.
fn session(a: &TestApp, sku: &str) -> i64 {
    let (_, session) = a.items().create_local(sku, sku, "object", None).unwrap();
    session.session.id
}

/// A time after every conversation's last update (the Resolved tab keeps them all).
fn soon(a: &TestApp) -> i64 {
    a.app
        .store
        .conversations()
        .unwrap()
        .iter()
        .map(|c| c.updated_at)
        .max()
        .unwrap_or(0)
        + 1_000
}

fn list(a: &TestApp, scope: ConversationScope, tab: ConversationTab) -> ConversationList {
    a.app.conversations.list(scope, tab, soon(a)).unwrap()
}

fn open_all(a: &TestApp) -> Vec<ConversationRow> {
    list(a, ConversationScope::All, ConversationTab::Open).rows
}

fn row<'r>(rows: &'r [ConversationRow], id: &str) -> &'r ConversationRow {
    rows.iter()
        .find(|r| r.id == id)
        .unwrap_or_else(|| panic!("{id} not listed"))
}

fn ids(rows: &[ConversationRow]) -> Vec<String> {
    rows.iter().map(|r| r.id.clone()).collect()
}

fn read(a: &TestApp, case: &str) {
    a.app.conversations.mark_read(case).unwrap();
}

// ---------- C2·C1 the session match ----------

#[test]
fn c2_c1a_a_conversation_belongs_to_the_session_of_its_sku() {
    let a = connected();
    let s = session(&a, "BEYO-1880");
    let case = ask(&a, Some("BEYO-1880"));
    run_check(&a);
    assert_eq!(row(&open_all(&a), &case).session_id, Some(s));
}

#[test]
fn c2_c1b_the_sku_must_match_exactly() {
    let a = connected();
    session(&a, "BEYO-1880");
    let case = ask(&a, Some("beyo-1880"));
    run_check(&a);
    assert_eq!(row(&open_all(&a), &case).session_id, None);
}

#[test]
fn c2_c1c_no_sku_or_an_unknown_sku_is_not_on_this_mac_and_still_listed() {
    let a = connected();
    session(&a, "BEYO-1880");
    let without = ask(&a, None);
    let unknown = ask(&a, Some("BEYO-9999"));
    run_check(&a);
    let rows = open_all(&a);
    assert_eq!(rows.len(), 2);
    assert_eq!(row(&rows, &without).session_id, None);
    assert_eq!(row(&rows, &unknown).session_id, None);
}

#[test]
fn c2_c1d_a_deleted_session_leaves_its_conversation_listed_without_a_session() {
    let a = connected();
    let s = session(&a, "BEYO-1880");
    let case = ask(&a, Some("BEYO-1880"));
    run_check(&a);
    session_delete::delete(&a.app.handles(), s).unwrap();
    let rows = open_all(&a);
    assert_eq!(ids(&rows), std::slice::from_ref(&case));
    assert_eq!(row(&rows, &case).session_id, None);
}

// ---------- C2·C2(e) the Open tab's order ----------

#[test]
fn c2_c2e_the_open_tab_follows_the_list_order_without_resolved() {
    let a = connected();
    // p: news, blocking, oldest · q: no news, blocking · r: no news, not blocking ·
    // s: news, not blocking, newest · gone: resolved.
    let p = ask(&a, Some("BEYO-1"));
    let q = ask(&a, Some("BEYO-2"));
    let r = ask(&a, Some("BEYO-3"));
    not_blocking(&a, &r);
    let s = ask(&a, Some("BEYO-4"));
    not_blocking(&a, &s);
    let gone = ask(&a, Some("BEYO-5"));
    pricing(&a).resolve(&gone);
    run_check(&a);
    read(&a, &q);
    read(&a, &r);
    assert_eq!(ids(&open_all(&a)), [p, s, q, r]);
}

// ---------- C2·C3(b) the Resolved tab ----------

#[test]
fn c2_c3b_the_resolved_tab_keeps_the_last_30_days_newest_activity_first() {
    let a = connected();
    let now = 100 * DAY;
    let resolved = |activity: i64, days_ago: i64| {
        let case = ask(&a, Some(&format!("BEYO-{activity}")));
        pricing(&a).resolve(&case);
        pricing(&a).change_silently(&case, |c| {
            c.updated_at = iso(now - days_ago * DAY);
            for m in &mut c.messages {
                m.created_at = iso(activity);
            }
        });
        case
    };
    let two_days = resolved(500, 2);
    let five_days = resolved(900, 5);
    let _forty_days = resolved(700, 40);
    let _open = ask(&a, Some("BEYO-1"));
    run_check(&a);
    let got = a
        .app
        .conversations
        .list(ConversationScope::All, ConversationTab::Resolved, now)
        .unwrap();
    assert_eq!(ids(&got.rows), [five_days.clone(), two_days.clone()]);
    assert_eq!(got.rows[0].last_activity_at, 900);
    assert_eq!(got.rows[1].last_activity_at, 500);
}

// ---------- C2·C4 the row's words ----------

#[test]
fn c2_c4b_the_row_shows_the_newest_message_first_line_and_time() {
    let a = connected();
    let case = ask(&a, Some("BEYO-1880"));
    pricing(&a).pricer_reply(&case, "Close-up please\nof the dent");
    let t = 1_795_200_000_000;
    pricing(&a).change_silently(&case, |c| {
        c.messages.last_mut().unwrap().created_at = iso(t)
    });
    run_check(&a);
    let rows = open_all(&a);
    let got = row(&rows, &case);
    assert_eq!(got.last_line.as_deref(), Some("Close-up please"));
    assert_eq!(got.last_activity_at, t);
    assert!(got.has_news);
}

#[test]
fn c2_c4c_blocking_unresolved_conversations_are_waiting() {
    let a = connected();
    let open_blocking = ask(&a, Some("BEYO-1"));
    let answered_blocking = ask(&a, Some("BEYO-2"));
    pricing(&a).own_reply(&answered_blocking, "Yes, original.");
    let open_free = ask(&a, Some("BEYO-3"));
    not_blocking(&a, &open_free);
    run_check(&a);
    let rows = open_all(&a);
    assert!(row(&rows, &open_blocking).waiting);
    assert!(row(&rows, &answered_blocking).waiting);
    assert!(!row(&rows, &open_free).waiting);
}

#[test]
fn c2_c4d_a_resolved_blocking_conversation_is_not_waiting() {
    let a = connected();
    let case = ask(&a, Some("BEYO-1"));
    pricing(&a).resolve(&case);
    run_check(&a);
    let rows = list(&a, ConversationScope::All, ConversationTab::Resolved).rows;
    let got = row(&rows, &case);
    assert!(got.blocking);
    assert!(!got.waiting);
}

// ---------- C2·C5 scope, footer and counts ----------

#[test]
fn c2_c5a_a_session_scope_lists_its_rows_and_counts_news_elsewhere() {
    let a = connected();
    let sa = session(&a, "BEYO-A");
    session(&a, "BEYO-B");
    let in_a = ask(&a, Some("BEYO-A"));
    let _in_b = ask(&a, Some("BEYO-B"));
    let _elsewhere = ask(&a, Some("BEYO-X"));
    run_check(&a);
    read(&a, &in_a);
    let got = list(&a, ConversationScope::Session(sa), ConversationTab::Open);
    assert_eq!(ids(&got.rows), [in_a]);
    assert_eq!(got.news_elsewhere, 2);
}

#[test]
fn c2_c5b_without_a_session_every_conversation_with_news_counts_here() {
    let a = connected();
    session(&a, "BEYO-A");
    let _news_open = ask(&a, Some("BEYO-A"));
    let news_resolved = ask(&a, Some("BEYO-A2"));
    let _news_elsewhere = ask(&a, Some("BEYO-X"));
    let quiet_1 = ask(&a, Some("BEYO-A3"));
    let quiet_2 = ask(&a, Some("BEYO-A4"));
    run_check(&a);
    read(&a, &quiet_1);
    read(&a, &quiet_2);
    pricing(&a).resolve(&news_resolved);
    next_check(&a);
    assert_eq!(
        a.app.conversations.counts(None).unwrap(),
        ConversationCounts {
            here: 3,
            elsewhere: 0
        }
    );
}

#[test]
fn c2_c5c_inside_a_session_news_of_others_counts_elsewhere() {
    let a = connected();
    let sa = session(&a, "BEYO-A");
    session(&a, "BEYO-B");
    ask(&a, Some("BEYO-A"));
    ask(&a, Some("BEYO-B"));
    ask(&a, None);
    run_check(&a);
    assert_eq!(
        a.app.conversations.counts(Some(sa)).unwrap(),
        ConversationCounts {
            here: 1,
            elsewhere: 2
        }
    );
}

#[test]
fn c2_c5d_reading_a_conversation_takes_it_out_of_the_count() {
    let a = connected();
    let sa = session(&a, "BEYO-A");
    let case = ask(&a, Some("BEYO-A"));
    run_check(&a);
    assert_eq!(a.app.conversations.counts(Some(sa)).unwrap().here, 1);
    read(&a, &case);
    assert_eq!(a.app.conversations.counts(Some(sa)).unwrap().here, 0);
}

// ---------- C2·C6 the sessions-list badges ----------

#[test]
fn c2_c6a_unresolved_conversations_give_an_open_badge() {
    let a = connected();
    let sa = session(&a, "BEYO-A");
    for _ in 0..2 {
        let case = ask(&a, Some("BEYO-A"));
        not_blocking(&a, &case);
    }
    run_check(&a);
    assert_eq!(
        a.app.conversations.session_badges().unwrap(),
        [SessionBadge {
            session_id: sa,
            open: 2,
            waiting: false
        }]
    );
}

#[test]
fn c2_c6b_an_answered_blocking_conversation_still_makes_pricing_wait() {
    let a = connected();
    let sa = session(&a, "BEYO-A");
    let answered = ask(&a, Some("BEYO-A"));
    pricing(&a).own_reply(&answered, "Yes.");
    let open = ask(&a, Some("BEYO-A"));
    not_blocking(&a, &open);
    run_check(&a);
    assert_eq!(
        a.app.conversations.session_badges().unwrap(),
        [SessionBadge {
            session_id: sa,
            open: 2,
            waiting: true
        }]
    );
}

#[test]
fn c2_c6c_a_resolved_blocking_conversation_gives_no_badge() {
    let a = connected();
    session(&a, "BEYO-A");
    let case = ask(&a, Some("BEYO-A"));
    pricing(&a).resolve(&case);
    run_check(&a);
    assert_eq!(a.app.store.conversations().unwrap().len(), 1);
    assert!(a.app.conversations.session_badges().unwrap().is_empty());
}

#[test]
fn c2_c6d_a_conversation_not_on_this_mac_gives_no_badge() {
    let a = connected();
    session(&a, "BEYO-A");
    ask(&a, Some("BEYO-X"));
    run_check(&a);
    assert_eq!(a.app.store.conversations().unwrap().len(), 1);
    assert!(a.app.conversations.session_badges().unwrap().is_empty());
}

// ---------- A30 the product is the container ----------

/// Three questions on one product (topics Title, Other, Other) and one question on another.
fn three_on_one(a: &TestApp) -> (String, String, String, String) {
    let first = pricing(a).ask(&store_id(a), Some("Ch1-201126"), "Title");
    let second = pricing(a).ask(&store_id(a), Some("Ch1-201126"), "Other");
    let third = pricing(a).ask(&store_id(a), Some("Ch1-201126"), "Other");
    let other = ask(a, Some("Sdt1-191126"));
    (first, second, third, other)
}

fn ch1(a: &TestApp, tab: ConversationTab) -> Option<ConversationRow> {
    list(a, ConversationScope::All, tab)
        .rows
        .into_iter()
        .find(|r| r.sku.as_deref() == Some("Ch1-201126"))
}

#[test]
fn a30_c1_a_products_questions_are_one_row() {
    let a = connected();
    let (first, second, third, other) = three_on_one(&a);
    pricing(&a).own_reply(&first, "so im not in work pc");
    pricing(&a).own_reply(&second, "because");
    run_check(&a);
    let rows = open_all(&a);
    assert_eq!(rows.len(), 2);
    let got = ch1(&a, ConversationTab::Open).unwrap();
    assert_eq!(got.questions, 3);
    // The row opens the newest unresolved question; one open question makes the product open.
    assert_eq!(got.id, third);
    assert_eq!(got.status.as_str(), "open");
    assert_eq!(row(&rows, &other).questions, 1);
}

#[test]
fn a30_c2_open_while_any_question_is_unresolved_then_resolved() {
    let a = connected();
    let (first, second, third, _other) = three_on_one(&a);
    pricing(&a).resolve(&first);
    pricing(&a).resolve(&second);
    run_check(&a);
    assert!(ch1(&a, ConversationTab::Open).is_some());
    assert!(ch1(&a, ConversationTab::Resolved).is_none());
    pricing(&a).resolve(&third);
    next_check(&a);
    assert!(ch1(&a, ConversationTab::Open).is_none());
    let got = ch1(&a, ConversationTab::Resolved).unwrap();
    assert_eq!(got.questions, 3);
    assert_eq!(
        got.id, third,
        "all resolved: the row opens the newest question"
    );
}

#[test]
fn a30_c3_one_blocking_unresolved_question_makes_the_product_wait() {
    let a = connected();
    let (first, second, third, _other) = three_on_one(&a);
    not_blocking(&a, &first);
    not_blocking(&a, &third);
    run_check(&a);
    assert!(ch1(&a, ConversationTab::Open).unwrap().waiting);
    pricing(&a).resolve(&second);
    next_check(&a);
    assert!(
        !ch1(&a, ConversationTab::Open).unwrap().waiting,
        "the blocking question is resolved"
    );
}

#[test]
fn a30_c4_the_header_counts_products_with_news() {
    let a = connected();
    let (_first, _second, _third, other) = three_on_one(&a);
    run_check(&a);
    read(&a, &other);
    assert_eq!(
        a.app.conversations.counts(None).unwrap(),
        ConversationCounts {
            here: 1,
            elsewhere: 0
        }
    );
}

#[test]
fn a30_c5_the_thread_holds_every_question_of_the_product_oldest_first() {
    let a = connected();
    let (first, second, third, other) = three_on_one(&a);
    pricing(&a).resolve(&first);
    run_check(&a);
    for opened in [&first, &second, &third] {
        let thread = a.app.conversations.thread(opened).unwrap();
        assert_eq!(thread.sku.as_deref(), Some("Ch1-201126"));
        let ids: Vec<&str> = thread.questions.iter().map(|q| q.id.as_str()).collect();
        assert_eq!(ids, [first.as_str(), second.as_str(), third.as_str()]);
        let topics: Vec<&str> = thread.questions.iter().map(|q| q.topic.as_str()).collect();
        assert_eq!(topics, ["Title", "Other", "Other"]);
        assert_eq!(thread.questions[0].status.as_str(), "resolved");
    }
    assert_eq!(
        a.app.conversations.thread(&other).unwrap().questions.len(),
        1
    );
}

#[test]
fn a30_c6_reading_every_question_clears_the_products_news() {
    let a = connected();
    let (first, second, third, other) = three_on_one(&a);
    run_check(&a);
    read(&a, &other);
    read(&a, &first);
    read(&a, &second);
    assert!(
        ch1(&a, ConversationTab::Open).unwrap().has_news,
        "one question still unread"
    );
    read(&a, &third);
    assert!(!ch1(&a, ConversationTab::Open).unwrap().has_news);
}
