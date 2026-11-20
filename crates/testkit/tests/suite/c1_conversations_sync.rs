//! C1·U3 (`docs/implementation/plans/C1_connection_and_sync.md`): the local copy, the first
//! connection, the pull, read marks, errors and notices (MC-C1.4–.10), through
//! `app.conversations.tick(now)` on `TestApp::with_pricing()` after a successful `key_set`, with the
//! manual clock. Requests and events are counted from after `key_set` returns.

use psm_core::{
    domain::conversation::{
        ConnectionState, Conversation, ConversationNotice, ConversationStatus, NoticeKind,
        ResolvedBy,
    },
    events::AppEvent,
    mechanisms::conversation_rules::has_news,
    ports::{clock::Clock, conversations::ConversationStore},
    services::conversations::ConversationSettingsPatch,
};
use psm_integrations::session_delete;
use psm_testkit::{
    TestApp,
    fake_pricing::FakePricing,
    test_app::{PRICING_KEY, PRICING_SHOP},
};
use serde_json::json;

const SKU: &str = "BEYO-1880";

/// A connected app (key saved, store matched); requests and events cleared.
fn connected() -> TestApp {
    let a = TestApp::with_pricing().unwrap();
    a.clock.advance(1_000_000);
    a.app.conversations.key_set(Some(PRICING_KEY)).unwrap();
    a.fake_pricing().clear_requests();
    clear_events(&a);
    a
}

fn pricing(a: &TestApp) -> &FakePricing {
    a.fake_pricing()
}

fn store_id(a: &TestApp) -> String {
    a.app.store.sync().unwrap().store.unwrap().id
}

fn tick(a: &TestApp) -> bool {
    a.app.conversations.tick(a.clock.now_ms()).unwrap().more
}

/// Ticks at the same moment until `more` is false; returns how many ticks ran.
fn run_check(a: &TestApp) -> usize {
    let mut n = 1;
    while tick(a) {
        n += 1;
        assert!(n < 50, "the check never completed");
    }
    n
}

/// The first connection, then requests and events cleared.
fn first_connected(a: &TestApp) {
    run_check(a);
    assert!(a.app.store.sync().unwrap().first_sync_done);
    pricing(a).clear_requests();
    clear_events(a);
}

/// The next scheduled check (10 s later), run to its end.
fn next_check(a: &TestApp) {
    a.clock.advance(10_000);
    run_check(a);
}

fn copy(a: &TestApp) -> Vec<Conversation> {
    a.app.store.conversations().unwrap()
}

fn copy_ids(a: &TestApp) -> Vec<String> {
    copy(a).into_iter().map(|c| c.id).collect()
}

fn find(a: &TestApp, id: &str) -> Conversation {
    a.app
        .store
        .conversation(id)
        .unwrap()
        .unwrap_or_else(|| panic!("{id} not in the copy"))
}

fn message_ids(c: &Conversation) -> Vec<String> {
    c.messages.iter().map(|m| m.id.clone()).collect()
}

fn clear_events(a: &TestApp) {
    a.events.0.lock().unwrap().clear();
}

/// The collected `ConversationsChanged` events: (notices, chime, banner).
fn changed(a: &TestApp) -> Vec<(Vec<ConversationNotice>, bool, bool)> {
    a.events
        .0
        .lock()
        .unwrap()
        .iter()
        .filter_map(|e| match e {
            AppEvent::ConversationsChanged {
                notices,
                chime,
                banner,
            } => Some((notices.clone(), *chime, *banner)),
            _ => None,
        })
        .collect()
}

fn lines(a: &TestApp) -> Vec<String> {
    pricing(a).request_lines()
}

fn events_line(store: &str, after: i64) -> String {
    format!("GET /stores/{store}/events?after={after}&limit=500")
}

fn case_line(store: &str, case: &str) -> String {
    format!("GET /stores/{store}/cases/{case}")
}

fn gets_of(a: &TestApp, case: &str) -> usize {
    let store = store_id(a);
    lines(a)
        .iter()
        .filter(|l| **l == case_line(&store, case))
        .count()
}

fn events_requests(a: &TestApp) -> Vec<String> {
    lines(a)
        .into_iter()
        .filter(|l| l.contains("/events?"))
        .collect()
}

/// The fake's four cases of C1·C11(a): open A, answered B, resolved C, Operations D.
fn four_cases(a: &TestApp) -> (String, String, String, String) {
    let p = pricing(a);
    let s = store_id(a);
    let case_a = p.ask(&s, Some("BEYO-1"), "Condition");
    let case_b = p.ask(&s, Some("BEYO-2"), "Dimensions");
    p.own_reply(&case_b, "Measured: 60 cm.");
    let case_c = p.ask(&s, Some("BEYO-3"), "Other");
    p.resolve(&case_c);
    let case_d = p.ask_operations(&s, Some("BEYO-4"));
    (case_a, case_b, case_c, case_d)
}

fn question(a: &TestApp, sku: &str) -> String {
    pricing(a).ask(&store_id(a), Some(sku), "Condition")
}

// ---------- C1·C11 first connection ----------

#[test]
fn c1_c11a_the_first_connection_copies_the_photographer_conversations() {
    let a = connected();
    let (case_a, case_b, case_c, _case_d) = four_cases(&a);
    let last = pricing(&a).last_event_id();
    run_check(&a);
    assert_eq!(
        copy_ids(&a),
        [case_a.clone(), case_b.clone(), case_c.clone()]
    );
    for case in [&case_a, &case_b, &case_c] {
        assert_eq!(
            message_ids(&find(&a, case)),
            pricing(&a).message_ids(case),
            "{case}"
        );
    }
    let s = a.app.store.sync().unwrap();
    assert_eq!(s.cursor, Some(last));
    assert!(s.first_sync_done);
    assert_eq!(s.last_ok_at, Some(a.clock.now_ms()));
}

#[test]
fn c1_c11b_the_first_connection_pages_then_lists_then_reads_the_rest() {
    let a = connected();
    let (_case_a, _case_b, case_c, _case_d) = four_cases(&a);
    let last = pricing(&a).last_event_id();
    run_check(&a);
    let s = store_id(&a);
    assert_eq!(
        lines(&a),
        [
            events_line(&s, 0),
            events_line(&s, last),
            format!("GET /stores/{s}/cases?recipient=photographer&status=unresolved"),
            case_line(&s, &case_c),
        ]
    );
}

#[test]
fn c1_c11c_the_first_connection_is_silent() {
    let a = connected();
    four_cases(&a);
    run_check(&a);
    assert_eq!(changed(&a), [(vec![], false, false)]);
}

#[test]
fn c1_c11d_open_conversations_enter_unread_the_others_read() {
    let a = connected();
    let (case_a, case_b, case_c, _) = four_cases(&a);
    run_check(&a);
    assert!(has_news(&find(&a, &case_a)));
    assert!(!has_news(&find(&a, &case_b)));
    assert!(!has_news(&find(&a, &case_c)));
}

#[test]
fn c1_c11e_deleted_and_missing_conversations_are_skipped() {
    let a = connected();
    let p = pricing(&a);
    let s = store_id(&a);
    let deleted = p.ask(&s, Some("BEYO-5"), "Other");
    let only = p.message_ids(&deleted)[0].clone();
    p.delete_message(&deleted, &only);
    let missing = p.ask(&s, Some("BEYO-6"), "Other");
    p.resolve(&missing);
    p.delete_case(&missing);
    run_check(&a);
    assert!(copy_ids(&a).is_empty());
    assert_eq!(gets_of(&a, &deleted), 0);
    assert_eq!(gets_of(&a, &missing), 1);
    assert!(a.app.store.sync().unwrap().first_sync_done);
}

#[test]
fn c1_c11f_an_interrupted_first_connection_starts_again_and_ends_the_same() {
    let a = connected();
    let (case_a, case_b, case_c, _) = four_cases(&a);
    let last = pricing(&a).last_event_id();
    pricing(&a).fail_matching("/cases", 1, 500);
    run_check(&a);
    let s = a.app.store.sync().unwrap();
    assert_eq!(s.cursor, None);
    assert!(!s.first_sync_done);
    assert_eq!(s.last_error.as_deref(), Some("Could not reach Pricing"));
    a.clock.advance(10_000);
    run_check(&a);
    assert_eq!(
        copy_ids(&a),
        [case_a.clone(), case_b.clone(), case_c.clone()]
    );
    for case in [&case_a, &case_b, &case_c] {
        assert_eq!(message_ids(&find(&a, case)), pricing(&a).message_ids(case));
    }
    let s = a.app.store.sync().unwrap();
    assert_eq!(s.cursor, Some(last));
    assert!(s.first_sync_done);
}

#[test]
fn c1_c11g_the_first_connection_pages_500_at_a_time() {
    let a = connected();
    let s = store_id(&a);
    let ids: Vec<i64> = (0..520)
        .map(|_| pricing(&a).raw_event(&s, "inbox.updated", None, json!({})))
        .collect();
    run_check(&a);
    assert_eq!(
        events_requests(&a),
        [
            events_line(&s, 0),
            events_line(&s, ids[499]),
            events_line(&s, ids[519])
        ]
    );
    assert_eq!(a.app.store.sync().unwrap().cursor, Some(ids[519]));
}

#[test]
fn c1_c11h_a_cursor_that_does_not_advance_is_a_bad_reply() {
    let a = connected();
    let s = store_id(&a);
    pricing(&a).raw_event(&s, "inbox.updated", None, json!({}));
    pricing(&a).stale_cursor_next();
    run_check(&a);
    assert_eq!(lines(&a), [events_line(&s, 0)]);
    let connection = a.app.conversations.connection().unwrap();
    assert_eq!(connection.state, ConnectionState::Ready);
    assert_eq!(
        connection.last_error.as_deref(),
        Some("Pricing answered in a way this version does not understand")
    );
    let sync = a.app.store.sync().unwrap();
    assert_eq!(sync.cursor, None);
    assert!(!sync.first_sync_done);
    assert!(copy_ids(&a).is_empty());
}

// ---------- C1·C12 the pull ----------

#[test]
fn c1_c12a_a_new_question_enters_the_copy() {
    let a = connected();
    first_connected(&a);
    let case = question(&a, SKU);
    let last = pricing(&a).last_event_id();
    next_check(&a);
    let c = find(&a, &case);
    assert_eq!(message_ids(&c), pricing(&a).message_ids(&case));
    assert_eq!(c.messages.len(), 1);
    assert_eq!(a.app.store.sync().unwrap().cursor, Some(last));
    assert_eq!(gets_of(&a, &case), 1);
}

#[test]
fn c1_c12b_an_edited_message_is_replaced() {
    let a = connected();
    let case = question(&a, SKU);
    first_connected(&a);
    let message = pricing(&a).message_ids(&case)[0].clone();
    pricing(&a).edit_message(&case, &message, "Is the glass original, really?");
    next_check(&a);
    let m = &find(&a, &case).messages[0];
    assert_eq!(m.text, "Is the glass original, really?");
    assert!(m.edited_at.is_some());
}

#[test]
fn c1_c12c_a_deleted_message_leaves_the_copy() {
    let a = connected();
    let case = question(&a, SKU);
    let second = pricing(&a).pricer_reply(&case, "And the frame?");
    first_connected(&a);
    let first = pricing(&a).message_ids(&case)[0].clone();
    pricing(&a).delete_message(&case, &first);
    next_check(&a);
    assert_eq!(message_ids(&find(&a, &case)), [second]);
}

#[test]
fn c1_c12d_a_resolved_conversation_says_by_whom() {
    let a = connected();
    let case = question(&a, SKU);
    first_connected(&a);
    pricing(&a).resolve(&case);
    next_check(&a);
    let c = find(&a, &case);
    assert_eq!(c.status, ConversationStatus::Resolved);
    assert_eq!(c.resolved_by, Some(ResolvedBy::Asker));
}

#[test]
fn c1_c12d_a_conversation_resolved_by_activation() {
    let a = connected();
    let case = question(&a, SKU);
    first_connected(&a);
    pricing(&a).resolve_by_activation(&case);
    next_check(&a);
    let c = find(&a, &case);
    assert_eq!(c.status, ConversationStatus::Resolved);
    assert_eq!(c.resolved_by, Some(ResolvedBy::Activation));
}

#[test]
fn c1_c12e_a_conversation_whose_last_message_is_deleted_leaves_the_copy() {
    let a = connected();
    let case = question(&a, SKU);
    first_connected(&a);
    let only = pricing(&a).message_ids(&case)[0].clone();
    pricing(&a).delete_message(&case, &only);
    let last = pricing(&a).last_event_id();
    next_check(&a);
    assert!(copy_ids(&a).is_empty());
    assert_eq!(gets_of(&a, &case), 0);
    assert_eq!(a.app.store.sync().unwrap().cursor, Some(last));
}

#[test]
fn c1_c12f_a_conversation_the_provider_no_longer_has_leaves_the_copy() {
    let a = connected();
    let case = question(&a, SKU);
    first_connected(&a);
    pricing(&a).pricer_reply(&case, "Still there?");
    pricing(&a).delete_case(&case);
    let last = pricing(&a).last_event_id();
    next_check(&a);
    assert!(copy_ids(&a).is_empty());
    assert_eq!(a.app.store.sync().unwrap().cursor, Some(last));
}

#[test]
fn c1_c12g_an_operations_question_only_moves_the_cursor() {
    let a = connected();
    let case = question(&a, SKU);
    first_connected(&a);
    let before = copy(&a);
    let ops = pricing(&a).ask_operations(&store_id(&a), Some("BEYO-9"));
    let last = pricing(&a).last_event_id();
    next_check(&a);
    assert_eq!(a.app.store.sync().unwrap().cursor, Some(last));
    assert_eq!(gets_of(&a, &ops), 0);
    assert_eq!(gets_of(&a, &case), 0);
    assert_eq!(copy(&a), before);
    assert!(changed(&a).is_empty());
}

#[test]
fn c1_c12h_an_event_of_another_type_only_moves_the_cursor() {
    let a = connected();
    question(&a, SKU);
    first_connected(&a);
    let before = copy(&a);
    let id = pricing(&a).raw_event(&store_id(&a), "inbox.updated", None, json!({}));
    next_check(&a);
    assert_eq!(a.app.store.sync().unwrap().cursor, Some(id));
    assert!(lines(&a).iter().all(|l| l.contains("/events?")));
    assert_eq!(copy(&a), before);
    assert!(changed(&a).is_empty());
}

#[test]
fn c1_c12i_a_redelivered_event_is_skipped() {
    let a = connected();
    let case = question(&a, SKU);
    let applied = pricing(&a).last_event_id();
    first_connected(&a);
    let before = copy(&a);
    pricing(&a).change_silently(&case, |c| {
        c.messages[0].text = Some("Changed without an event".into())
    });
    pricing(&a).redeliver(applied);
    next_check(&a);
    assert_eq!(gets_of(&a, &case), 0);
    assert_eq!(copy(&a), before);
}

#[test]
fn c1_c12j_a_new_pricer_message_on_an_answered_conversation_is_news() {
    let a = connected();
    let case = question(&a, SKU);
    pricing(&a).own_reply(&case, "Original glass.");
    first_connected(&a);
    let mark = find(&a, &case).read_mark;
    assert!(mark.is_some());
    assert!(!has_news(&find(&a, &case)));
    pricing(&a).pricer_reply(&case, "Thanks, one more photo?");
    next_check(&a);
    let c = find(&a, &case);
    assert_eq!(message_ids(&c), pricing(&a).message_ids(&case));
    assert_eq!(c.read_mark, mark);
    assert!(has_news(&c));
}

// ---------- C1·C13 interruptions and paging ----------

#[test]
fn c1_c13a_an_error_mid_page_keeps_what_was_applied() {
    let a = connected();
    let case_a = question(&a, "BEYO-1");
    let case_b = question(&a, "BEYO-2");
    first_connected(&a);
    let before_b = find(&a, &case_b);
    pricing(&a).pricer_reply(&case_a, "A again");
    let a_event = pricing(&a).last_event_id();
    pricing(&a).pricer_reply(&case_b, "B again");
    let b_event = pricing(&a).last_event_id();
    pricing(&a).fail_matching(&format!("/cases/{case_b}"), 1, 500);
    a.clock.advance(10_000);
    run_check(&a);
    assert_eq!(find(&a, &case_a).messages.len(), 2);
    assert_eq!(a.app.store.sync().unwrap().cursor, Some(a_event));
    assert_eq!(find(&a, &case_b), before_b);
    next_check(&a);
    assert_eq!(
        message_ids(&find(&a, &case_a)),
        pricing(&a).message_ids(&case_a)
    );
    assert_eq!(
        message_ids(&find(&a, &case_b)),
        pricing(&a).message_ids(&case_b)
    );
    assert_eq!(a.app.store.sync().unwrap().cursor, Some(b_event));
}

#[test]
fn c1_c13b_the_cursor_and_the_copy_survive_a_reopen() {
    let mut a = connected();
    let case = question(&a, SKU);
    first_connected(&a);
    pricing(&a).pricer_reply(&case, "More?");
    next_check(&a);
    let cursor = a.app.store.sync().unwrap().cursor;
    let before = copy(&a);
    a.kill_and_restart().unwrap();
    assert_eq!(a.app.store.sync().unwrap().cursor, cursor);
    assert_eq!(copy(&a), before);
    pricing(&a).clear_requests();
    tick(&a);
    assert_eq!(
        lines(&a).first(),
        Some(&events_line(&store_id(&a), cursor.unwrap()))
    );
}

#[test]
fn c1_c13c_several_events_of_one_conversation_read_it_once() {
    let a = connected();
    let case = question(&a, SKU);
    first_connected(&a);
    pricing(&a).pricer_reply(&case, "one");
    pricing(&a).pricer_reply(&case, "two");
    pricing(&a).pricer_reply(&case, "three");
    let third = pricing(&a).last_event_id();
    next_check(&a);
    assert_eq!(gets_of(&a, &case), 1);
    assert_eq!(a.app.store.sync().unwrap().cursor, Some(third));
}

#[test]
fn c1_c13d_pages_follow_each_other_without_waiting() {
    let a = connected();
    first_connected(&a);
    let s = store_id(&a);
    let start = a.app.store.sync().unwrap().cursor.unwrap();
    for _ in 0..620 {
        pricing(&a).raw_event(&s, "inbox.updated", None, json!({}));
    }
    a.clock.advance(10_000);
    assert!(tick(&a), "the first page answers more");
    assert!(tick(&a), "the second page answers more");
    assert!(!tick(&a), "the empty page completes the check");
    assert_eq!(
        events_requests(&a),
        [
            events_line(&s, start),
            events_line(&s, start + 500),
            events_line(&s, start + 620)
        ]
    );
}

// ---------- C1·C14 when a check runs ----------

#[test]
fn c1_c14a_a_check_runs_at_once_after_key_set() {
    let a = connected();
    first_connected(&a);
    a.clock.advance(1_000);
    a.app.conversations.key_set(Some(PRICING_KEY)).unwrap();
    pricing(&a).clear_requests();
    tick(&a);
    assert_eq!(events_requests(&a).len(), 1);
}

#[test]
fn c1_c14b_checks_follow_every_10_seconds() {
    let a = connected();
    first_connected(&a);
    a.clock.advance(9_999);
    tick(&a);
    assert!(lines(&a).is_empty());
    a.clock.advance(1);
    tick(&a);
    assert_eq!(events_requests(&a).len(), 1);
}

#[test]
fn c1_c14c_a_check_runs_at_launch() {
    let mut a = connected();
    first_connected(&a);
    a.kill_and_restart().unwrap();
    pricing(&a).clear_requests();
    tick(&a);
    assert_eq!(events_requests(&a).len(), 1);
}

#[test]
fn c1_c14d_a_check_runs_on_waking() {
    let a = connected();
    first_connected(&a);
    a.clock.advance(3_600_000);
    tick(&a);
    assert_eq!(events_requests(&a).len(), 1);
}

// ---------- C1·C15(h) backoff in the loop ----------

#[test]
fn c1_c15h_failures_back_off_and_a_success_returns_to_10_seconds() {
    let a = connected();
    first_connected(&a);
    pricing(&a).fail_matching("/events", 2, 500);
    let requests_after = |advance: i64| {
        a.clock.advance(advance);
        pricing(&a).clear_requests();
        tick(&a);
        lines(&a).len()
    };
    assert_eq!(requests_after(10_000), 1, "first failure");
    assert_eq!(requests_after(9_999), 0);
    assert_eq!(requests_after(1), 1, "second failure");
    assert_eq!(requests_after(19_999), 0);
    assert_eq!(requests_after(1), 1, "success");
    assert_eq!(a.app.store.sync().unwrap().last_error, None);
    assert_eq!(requests_after(9_999), 0);
    assert_eq!(requests_after(1), 1);
}

// ---------- C1·C16 errors ----------

#[test]
fn c1_c16a_a_refused_key_stops_checking() {
    let a = connected();
    let case = question(&a, SKU);
    first_connected(&a);
    pricing(&a).fail_matching("/events", 1, 401);
    next_check(&a);
    let connection = a.app.conversations.connection().unwrap();
    assert_eq!(connection.state, ConnectionState::Refused);
    assert_eq!(
        connection.last_error.as_deref(),
        Some("Pricing refused the key: check Settings → Pricing")
    );
    assert_eq!(copy_ids(&a), [case]);
    assert_eq!(changed(&a), [(vec![], false, false)]);
    pricing(&a).clear_requests();
    for _ in 0..6 {
        a.clock.advance(600_000);
        tick(&a);
    }
    assert!(lines(&a).is_empty());
}

#[test]
fn c1_c16b_a_new_key_resumes_checking() {
    let a = connected();
    first_connected(&a);
    pricing(&a).fail_matching("/events", 1, 401);
    next_check(&a);
    assert_eq!(
        a.app.conversations.connection().unwrap().state,
        ConnectionState::Refused
    );
    pricing(&a).accept_key("k-replaced-3");
    a.app.conversations.key_set(Some("k-replaced-3")).unwrap();
    pricing(&a).clear_requests();
    run_check(&a);
    let connection = a.app.conversations.connection().unwrap();
    assert_eq!(connection.state, ConnectionState::Ready);
    assert_eq!(events_requests(&a).len(), 1);
    assert_eq!(connection.last_error, None);
}

#[test]
fn c1_c16c_a_store_that_is_gone_stops_checking() {
    let a = connected();
    first_connected(&a);
    pricing(&a).fail_matching("/events", 1, 404);
    next_check(&a);
    let connection = a.app.conversations.connection().unwrap();
    assert_eq!(connection.state, ConnectionState::StoreGone);
    assert_eq!(
        connection.last_error,
        Some(format!("This key is no longer tied to {PRICING_SHOP}"))
    );
    pricing(&a).clear_requests();
    a.clock.advance(600_000);
    tick(&a);
    assert!(lines(&a).is_empty());
}

fn transient_failure(script: impl Fn(&FakePricing), sentence: &str) {
    let a = connected();
    let case = question(&a, SKU);
    first_connected(&a);
    let before = copy(&a);
    let ok_at = a.app.store.sync().unwrap().last_ok_at;
    script(pricing(&a));
    pricing(&a).pricer_reply(&case, "unseen");
    next_check(&a);
    let connection = a.app.conversations.connection().unwrap();
    assert_eq!(connection.state, ConnectionState::Ready);
    assert_eq!(copy(&a), before);
    assert_eq!(connection.last_ok_at, ok_at);
    assert_eq!(connection.last_error.as_deref(), Some(sentence));
}

#[test]
fn c1_c16d_a_server_error_keeps_the_copy() {
    transient_failure(
        |p| p.fail_matching("/events", 1, 500),
        "Could not reach Pricing",
    );
}

#[test]
fn c1_c16d_a_time_out_keeps_the_copy() {
    transient_failure(|p| p.timeout_next(1), "Could not reach Pricing");
}

#[test]
fn c1_c16d_a_reply_that_is_not_an_event_page_keeps_the_copy() {
    transient_failure(
        |p| p.fail_matching("/events", 1, 200),
        "Pricing answered in a way this version does not understand",
    );
}

// ---------- C1·C17 notices ----------

fn notice(a: &TestApp, case: &str, kind: NoticeKind) -> ConversationNotice {
    let c = find(a, case);
    ConversationNotice {
        conversation_id: c.id,
        sku: c.sku,
        product_title: c.product_title,
        topic: c.topic,
        kind,
    }
}

#[test]
fn c1_c17a_a_new_question_notifies_with_chime_and_banner() {
    let a = connected();
    first_connected(&a);
    let case = question(&a, SKU);
    next_check(&a);
    let events = changed(&a);
    assert_eq!(events.len(), 1);
    assert_eq!(
        events[0].0,
        [ConversationNotice {
            conversation_id: case,
            sku: Some(SKU.into()),
            product_title: format!("Product {SKU}"),
            topic: "Condition".into(),
            kind: NoticeKind::Question,
        }]
    );
    assert!(events[0].1, "chime");
    assert!(events[0].2, "banner");
}

#[test]
fn c1_c17b_a_pricer_reply_notifies_as_a_reply() {
    let a = connected();
    let case = question(&a, SKU);
    first_connected(&a);
    pricing(&a).pricer_reply(&case, "And the back?");
    next_check(&a);
    assert_eq!(
        changed(&a),
        [(vec![notice(&a, &case, NoticeKind::Reply)], true, true)]
    );
}

#[test]
fn c1_c17c_own_replies_never_notify() {
    let a = connected();
    let case = question(&a, SKU);
    first_connected(&a);
    pricing(&a).own_reply(&case, "Original glass.");
    next_check(&a);
    assert_eq!(
        message_ids(&find(&a, &case)),
        pricing(&a).message_ids(&case)
    );
    assert_eq!(changed(&a), [(vec![], false, false)]);
}

#[test]
fn c1_c17d_one_notice_per_conversation_in_the_order_of_their_first_event() {
    let a = connected();
    let other = question(&a, "BEYO-7");
    first_connected(&a);
    let new = question(&a, SKU);
    pricing(&a).pricer_reply(&new, "two");
    pricing(&a).pricer_reply(&new, "three");
    pricing(&a).pricer_reply(&other, "one");
    pricing(&a).pricer_reply(&other, "two");
    next_check(&a);
    assert_eq!(
        changed(&a),
        [(
            vec![
                notice(&a, &new, NoticeKind::Question),
                notice(&a, &other, NoticeKind::Reply)
            ],
            true,
            true
        )]
    );
}

#[test]
fn c1_c17e_mute_turns_the_chime_off_only() {
    let a = connected();
    first_connected(&a);
    a.app
        .conversations
        .settings_save(&ConversationSettingsPatch {
            sound: Some(false),
            ..Default::default()
        })
        .unwrap();
    clear_events(&a);
    question(&a, SKU);
    next_check(&a);
    let events = changed(&a);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].0.len(), 1);
    assert!(!events[0].1, "chime");
    assert!(events[0].2, "banner");
}

#[test]
fn c1_c17f_a_resolution_does_not_notify() {
    let a = connected();
    let case = question(&a, SKU);
    first_connected(&a);
    pricing(&a).resolve(&case);
    next_check(&a);
    assert_eq!(changed(&a), [(vec![], false, false)]);
}

#[test]
fn c1_c17i_a_launch_catches_up_with_one_chime() {
    let mut a = connected();
    let third = question(&a, "BEYO-3");
    first_connected(&a);
    let one = question(&a, "BEYO-1");
    let two = question(&a, "BEYO-2");
    pricing(&a).resolve(&third);
    a.kill_and_restart().unwrap();
    clear_events(&a);
    run_check(&a);
    assert_eq!(copy_ids(&a), [third.clone(), one.clone(), two.clone()]);
    assert_eq!(find(&a, &third).status, ConversationStatus::Resolved);
    assert_eq!(
        changed(&a),
        [(
            vec![
                notice(&a, &one, NoticeKind::Question),
                notice(&a, &two, NoticeKind::Question)
            ],
            true,
            true
        )]
    );
}

// ---------- C1·C18 read marks and Delete session ----------

#[test]
fn c1_c18c_mark_read_then_a_new_pricer_message() {
    let a = connected();
    let case = question(&a, SKU);
    first_connected(&a);
    assert!(has_news(&find(&a, &case)));
    a.app.conversations.mark_read(&case).unwrap();
    assert!(!has_news(&find(&a, &case)));
    assert_eq!(changed(&a), [(vec![], false, false)]);
    pricing(&a).pricer_reply(&case, "Any news?");
    next_check(&a);
    assert!(has_news(&find(&a, &case)));
}

#[test]
fn c1_c18d_deleting_a_session_keeps_its_conversations_and_asks_nothing() {
    let a = connected();
    let case = question(&a, SKU);
    first_connected(&a);
    let (_, session) = a.items().create_local(SKU, SKU, "object", None).unwrap();
    let before = copy(&a);
    session_delete::delete(&a.app.handles(), session.session.id).unwrap();
    assert_eq!(copy(&a), before);
    assert_eq!(copy_ids(&a), [case]);
    assert!(lines(&a).is_empty());
}

// ---------- C1·C21 Pricing never holds up a screen ----------

#[test]
fn c1_c21a_screens_answer_while_a_request_is_held() {
    let a = connected();
    a.items().create_local(SKU, SKU, "object", None).unwrap();
    pricing(&a).hold();
    let service = a.app.conversations.clone();
    let now = a.clock.now_ms();
    let ticking = std::thread::spawn(move || service.tick(now).map(|t| t.more));
    assert!(pricing(&a).wait_until_held(), "the tick reached Pricing");
    let sessions = a
        .sessions()
        .list(psm_core::domain::session::ListFilter::Open, None)
        .unwrap();
    let rows: usize = sessions.groups.iter().map(|g| g.rows.len()).sum();
    assert_eq!(rows, 1);
    let connection = a.app.conversations.connection().unwrap();
    assert_eq!(connection.state, ConnectionState::Ready);
    assert!(!ticking.is_finished(), "the tick is still held");
    pricing(&a).release();
    ticking.join().unwrap().unwrap();
    assert!(a.app.store.sync().unwrap().first_sync_done);
}
