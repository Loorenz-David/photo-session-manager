//! L1·U3 (`docs/implementation/plans/L1_labels_provider_and_marker.md` L1·C13, C14): the status
//! read of session markers (MC-L3.3) and the label events of the 09B pull (MC-L4), through
//! `App.labels` and `App.conversations` on `TestApp::with_pricing()` with the fake Pricing server
//! and the manual clock. "Connected" = `key_set` answered `Ready` and the first connection was
//! run by `tick`.

use psm_core::{
    domain::{
        conversation::ConnectionState,
        label::{LabelState, LabelStatus, LabelsError, ResetReason, SessionLabel},
    },
    events::AppEvent,
    ports::{clock::Clock, conversations::ConversationStore},
    services::{items::ItemsService, labels::LabelsFailure},
};
use psm_testkit::{TestApp, fake_pricing::LabelSpec, test_app::PRICING_KEY};
use rusqlite::Connection;
use serde_json::json;

const P1: &str = "15531808096409";
const P2: &str = "15531808096410";
const P9: &str = "15531808096499";
/// 2026-11-20T09:12:00Z.
const T: i64 = 1_795_165_920_000;

fn app() -> TestApp {
    let a = TestApp::with_pricing().unwrap();
    a.clock.advance(1_000_000);
    a
}

/// A shop product `gid://shopify/Product/<product>` adopted and given a session; the fake
/// Pricing holds the label product (priced, not printed) when `in_pricing`.
fn linked(a: &TestApp, product: &str, sku: &str, in_pricing: bool) -> i64 {
    let external = a.fake_shop().add_product_with_id(
        &format!("gid://shopify/Product/{product}"),
        sku,
        &format!("{sku} title"),
        None,
    );
    let item = ItemsService::new(a.store(), a.clock.as_ref())
        .with_commerce(Some(a.fake_shop()))
        .adopt(&external)
        .unwrap();
    if in_pricing {
        a.fake_pricing().add_label_product(LabelSpec {
            product_id: product.into(),
            sku: Some(sku.into()),
            title: format!("{sku} title"),
            price_minor: Some(100_000),
            ..LabelSpec::default()
        });
    }
    a.sessions().create(item.id).unwrap().session.id
}

fn connect(a: &TestApp) {
    let connection = a.app.conversations.key_set(Some(PRICING_KEY)).unwrap();
    assert_eq!(connection.state, ConnectionState::Ready);
    run_check(a);
}

fn run_check(a: &TestApp) {
    let mut n = 0;
    while a.app.conversations.tick(a.clock.now_ms()).unwrap().more {
        n += 1;
        assert!(n < 50, "the check never completed");
    }
}

/// The next scheduled check (10 s later), run to its end.
fn next_check(a: &TestApp) {
    a.clock.advance(10_000);
    run_check(a);
}

fn clear(a: &TestApp) {
    a.fake_pricing().clear_requests();
    a.events.0.lock().unwrap().clear();
}

fn store_id(a: &TestApp) -> String {
    a.app.store.sync().unwrap().store.unwrap().id
}

fn cursor(a: &TestApp) -> Option<i64> {
    a.app.store.sync().unwrap().cursor
}

fn label_requests(a: &TestApp) -> Vec<String> {
    a.fake_pricing()
        .request_lines()
        .into_iter()
        .filter(|line| line.contains("/labels/"))
        .collect()
}

fn labels_changed(a: &TestApp) -> Vec<(Vec<String>, bool)> {
    a.events
        .0
        .lock()
        .unwrap()
        .iter()
        .filter_map(|e| match e {
            AppEvent::LabelsChanged {
                product_ids,
                access,
            } => Some((product_ids.clone(), *access)),
            _ => None,
        })
        .collect()
}

fn conversations_changed(a: &TestApp) -> usize {
    a.events
        .0
        .lock()
        .unwrap()
        .iter()
        .filter(|e| matches!(e, AppEvent::ConversationsChanged { .. }))
        .count()
}

fn state(a: &TestApp, session: i64) -> Option<LabelState> {
    a.app.labels.session_labels(&[session], false).unwrap()[0].state
}

fn printed(at: i64) -> LabelState {
    LabelState {
        status: LabelStatus::Printed,
        printed_at: Some(at),
        has_price: Some(true),
        reset: None,
    }
}

/// L1·C13(a)'s app: connected; `s1` linked to `P1`, printed in the fake at `T`; `s2` a session
/// whose item has no link; the markers read once. Answers (app, s1, s2, the answer).
fn markers_read() -> (TestApp, i64, i64, Vec<SessionLabel>) {
    let a = app();
    let s1 = linked(&a, P1, "TEST-1802", true);
    a.fake_pricing().set_label(P1, |l| l.printed_at = Some(T));
    let (_, local) = a
        .items()
        .create_local("LOCAL-1", "Local", "object", None)
        .unwrap();
    let s2 = local.session.id;
    connect(&a);
    clear(&a);
    let answer = a.app.labels.session_labels(&[s1, s2], true).unwrap();
    (a, s1, s2, answer)
}

/// A connected app with `s1` linked to `P1` (priced, not printed); requests and events cleared.
fn connected_with(products: &[(&str, &str)]) -> (TestApp, Vec<i64>) {
    let a = app();
    let sessions = products
        .iter()
        .map(|(product, sku)| linked(&a, product, sku, true))
        .collect();
    connect(&a);
    clear(&a);
    (a, sessions)
}

#[test]
fn l1_c13a_one_status_request_answers_the_sessions_in_order() {
    let (a, s1, s2, answer) = markers_read();
    assert_eq!(
        label_requests(&a),
        vec![format!(
            "GET /stores/{}/labels/status?productIds={P1}",
            store_id(&a)
        )]
    );
    assert_eq!(
        answer,
        vec![
            SessionLabel {
                session_id: s1,
                product_id: Some(P1.into()),
                state: Some(printed(T)),
            },
            SessionLabel {
                session_id: s2,
                product_id: None,
                state: None,
            },
        ]
    );
}

#[test]
fn l1_c13b_without_refresh_the_copy_answers_and_nothing_is_asked() {
    let (a, s1, _, _) = markers_read();
    a.fake_pricing().clear_requests();
    assert_eq!(state(&a, s1), Some(printed(T)));
    assert!(label_requests(&a).is_empty());
    let (fresh, sessions) = connected_with(&[(P1, "TEST-1802")]);
    assert_eq!(state(&fresh, sessions[0]), None);
    assert!(label_requests(&fresh).is_empty());
}

#[test]
fn l1_c13c_a_fresh_answer_is_not_asked_again_for_ten_seconds() {
    let (a, s1, _, _) = markers_read();
    a.fake_pricing().clear_requests();
    a.clock.advance(9_000);
    a.app.labels.session_labels(&[s1], true).unwrap();
    assert!(label_requests(&a).is_empty());
    a.clock.advance(2_000);
    a.app.labels.session_labels(&[s1], true).unwrap();
    let requests = label_requests(&a);
    assert_eq!(requests.len(), 1, "{requests:?}");
    assert!(requests[0].contains("/labels/status?"), "{requests:?}");
}

#[test]
fn l1_c13d_a_product_pricing_does_not_hold_has_no_state() {
    let a = app();
    let s = linked(&a, P9, "TEST-9", false);
    connect(&a);
    assert_eq!(
        a.app.labels.session_labels(&[s], true).unwrap()[0].state,
        None
    );
}

#[test]
fn l1_c13e_a_failed_status_read_keeps_the_copy() {
    let (a, s1, _, _) = markers_read();
    a.fake_pricing().fail_matching("/labels/status", 1, 500);
    a.clock.advance(11_000);
    let error = a.app.labels.session_labels(&[s1], true).unwrap_err();
    assert!(
        matches!(error, LabelsFailure::Provider(LabelsError::Unreachable(_))),
        "{error:?}"
    );
    assert_eq!(state(&a, s1), Some(printed(T)));
}

#[test]
fn l1_c13f_nothing_about_label_state_is_stored() {
    let (a, s1, _, _) = markers_read();
    a.fake_pricing().set_label(P1, |l| l.printed_at = None);
    a.fake_pricing().label_printed(P1);
    next_check(&a);
    assert_eq!(state(&a, s1).map(|s| s.status), Some(LabelStatus::Printed));
    let conn = Connection::open(a.app.paths.db()).unwrap();
    let tables: Vec<String> = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap();
    assert!(
        !tables.iter().any(|t| t.to_lowercase().contains("label")),
        "{tables:?}"
    );
    let mut columns = Vec::new();
    for table in &tables {
        let names: Vec<String> = conn
            .prepare(&format!("SELECT name FROM pragma_table_info('{table}')"))
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap();
        for name in names {
            if name.to_lowercase().contains("label") {
                columns.push(format!("{table}.{name}"));
            }
        }
    }
    columns.sort();
    assert_eq!(
        columns,
        [
            "attribute_definitions.label",
            "conversation_sync.labels_access"
        ]
    );
    let value: String = conn
        .query_row(
            "SELECT labels_access FROM conversation_sync WHERE id=1",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert!(
        ["unknown", "allowed", "denied"].contains(&value.as_str()),
        "{value}"
    );
}

#[test]
fn l1_c14a_a_printed_event_updates_the_marker_once() {
    let (a, sessions) = connected_with(&[(P1, "TEST-1802")]);
    let event = a.fake_pricing().label_printed(P1);
    let at = a.fake_pricing().event_at_ms(event);
    next_check(&a);
    assert_eq!(state(&a, sessions[0]), Some(printed(at)));
    assert_eq!(cursor(&a), Some(event));
    assert_eq!(labels_changed(&a), vec![(vec![P1.to_owned()], false)]);
}

#[test]
fn l1_c14b_a_reset_event_makes_the_label_stale() {
    let (a, sessions) = connected_with(&[(P1, "TEST-1802")]);
    a.fake_pricing().label_printed(P1);
    next_check(&a);
    a.fake_pricing().label_reset(P1, "price_changed");
    next_check(&a);
    assert_eq!(
        state(&a, sessions[0]),
        Some(LabelState {
            status: LabelStatus::NotPrinted,
            printed_at: None,
            has_price: Some(true),
            reset: Some(ResetReason::PriceChanged),
        })
    );
    assert_eq!(
        labels_changed(&a),
        vec![(vec![P1.to_owned()], false), (vec![P1.to_owned()], false)]
    );
}

#[test]
fn l1_c14c_a_label_event_is_silent() {
    let (a, _) = connected_with(&[(P1, "TEST-1802")]);
    a.fake_pricing().label_printed(P1);
    next_check(&a);
    assert_eq!(labels_changed(&a).len(), 1);
    assert_eq!(conversations_changed(&a), 0);
}

#[test]
fn l1_c14d_a_redelivered_event_changes_nothing() {
    let (a, sessions) = connected_with(&[(P1, "TEST-1802")]);
    let event = a.fake_pricing().label_printed(P1);
    next_check(&a);
    let before = state(&a, sessions[0]);
    a.fake_pricing().redeliver(event);
    next_check(&a);
    assert_eq!(labels_changed(&a).len(), 1);
    assert_eq!(state(&a, sessions[0]), before);
}

#[test]
fn l1_c14e_one_page_is_told_once_with_the_distinct_products() {
    let (a, sessions) = connected_with(&[(P1, "TEST-1802"), (P2, "TEST-1803")]);
    a.fake_pricing().label_printed(P1);
    a.fake_pricing().label_printed(P2);
    a.fake_pricing().label_reset(P1, "price_changed");
    next_check(&a);
    assert_eq!(
        labels_changed(&a),
        vec![(vec![P1.to_owned(), P2.to_owned()], false)]
    );
    let p1 = state(&a, sessions[0]).unwrap();
    assert_eq!(p1.status, LabelStatus::NotPrinted);
    assert_eq!(p1.reset, Some(ResetReason::PriceChanged));
    assert_eq!(
        state(&a, sessions[1]).map(|s| s.status),
        Some(LabelStatus::Printed)
    );
}

#[test]
fn l1_c14f_label_changes_are_told_when_the_page_stops_on_an_error() {
    let a = app();
    let s1 = linked(&a, P1, "TEST-1802", true);
    a.app.conversations.key_set(Some(PRICING_KEY)).unwrap();
    let store = store_id(&a);
    let case = a.fake_pricing().ask(&store, Some("TEST-1802"), "Condition");
    run_check(&a);
    clear(&a);
    let event = a.fake_pricing().label_printed(P1);
    a.fake_pricing().pricer_reply(&case, "One more thing");
    a.fake_pricing()
        .fail_matching(&format!("/cases/{case}"), 1, 500);
    next_check(&a);
    assert_eq!(labels_changed(&a), vec![(vec![P1.to_owned()], false)]);
    assert_eq!(state(&a, s1).map(|s| s.status), Some(LabelStatus::Printed));
    assert_eq!(cursor(&a), Some(event));
    assert!(a.app.store.sync().unwrap().last_error.is_some());
}

#[test]
fn l1_c14g_the_first_connection_tells_nothing() {
    let a = app();
    let s1 = linked(&a, P1, "TEST-1802", true);
    a.fake_pricing().label_printed(P1);
    let last = a.fake_pricing().label_reset(P1, "price_changed");
    connect(&a);
    assert!(
        labels_changed(&a).iter().all(|(ids, _)| ids.is_empty()),
        "{:?}",
        labels_changed(&a)
    );
    assert_eq!(state(&a, s1), None);
    assert_eq!(cursor(&a), Some(last));
}

#[test]
fn l1_c14h_a_reset_for_a_product_without_a_session_is_held() {
    let (a, _) = connected_with(&[(P1, "TEST-1802")]);
    a.fake_pricing().label_reset(P9, "sku_changed");
    next_check(&a);
    let s9 = linked(&a, P9, "TEST-9", false);
    assert_eq!(
        state(&a, s9),
        Some(LabelState {
            status: LabelStatus::NotPrinted,
            printed_at: None,
            has_price: None,
            reset: Some(ResetReason::SkuChanged),
        })
    );
}

#[test]
fn l1_c14i_a_label_event_without_a_product_is_passed_over() {
    let (a, _) = connected_with(&[(P1, "TEST-1802")]);
    let store = store_id(&a);
    let event =
        a.fake_pricing()
            .raw_event(&store, "label.printed", None, json!({"sku": "TEST-1802"}));
    next_check(&a);
    assert_eq!(cursor(&a), Some(event));
    assert!(labels_changed(&a).is_empty());
}
