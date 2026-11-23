//! L1·U3 (`docs/implementation/plans/L1_labels_provider_and_marker.md` L1·C12): when a label call
//! may be made (ACC, M-L6), through `App.labels` and `App.conversations` on
//! `TestApp::with_pricing()` with the fake Pricing server. "No label request" = no recorded request
//! line contains `/labels/`.

use psm_core::{
    domain::{
        conversation::ConnectionState,
        label::{LabelsAccess, LabelsError, SessionLabel},
    },
    events::AppEvent,
    ports::{clock::Clock, conversations::ConversationStore, labels::LabelStore},
    services::{
        conversations::ConversationSettingsPatch, items::ItemsService, labels::LabelsFailure,
    },
};
use psm_testkit::{TestApp, fake_pricing::LabelSpec, test_app::PRICING_KEY};

const P: &str = "15531808096409";

fn app() -> TestApp {
    let a = TestApp::with_pricing().unwrap();
    a.clock.advance(1_000_000);
    a
}

/// A shop product `gid://shopify/Product/<product>` adopted and given a session; the fake
/// Pricing holds the label product. Answers the session id.
fn linked_session(a: &TestApp, product: &str, sku: &str) -> i64 {
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
    a.fake_pricing().add_label_product(LabelSpec {
        product_id: product.into(),
        sku: Some(sku.into()),
        title: format!("{sku} title"),
        price_minor: Some(100_000),
        ..LabelSpec::default()
    });
    a.sessions().create(item.id).unwrap().session.id
}

/// `key_set` answered `Ready`, then the first connection run by `tick`.
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

fn store_id(a: &TestApp) -> String {
    a.app.store.sync().unwrap().store.unwrap().id
}

fn label_requests(a: &TestApp) -> Vec<String> {
    a.fake_pricing()
        .request_lines()
        .into_iter()
        .filter(|line| line.contains("/labels/"))
        .collect()
}

fn clear(a: &TestApp) {
    a.fake_pricing().clear_requests();
    a.events.0.lock().unwrap().clear();
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

fn access(a: &TestApp) -> LabelsAccess {
    a.app.labels.access().unwrap()
}

fn stored_access(a: &TestApp) -> LabelsAccess {
    LabelStore::labels_access(a.app.store.as_ref()).unwrap()
}

fn labels(a: &TestApp, sessions: &[i64], refresh: bool) -> Vec<SessionLabel> {
    a.app.labels.session_labels(sessions, refresh).unwrap()
}

/// A connected app whose key may read labels, with one linked session for `P`.
fn connected_allowed() -> (TestApp, i64) {
    let a = app();
    let s = linked_session(&a, P, "TEST-1802");
    connect(&a);
    assert_eq!(access(&a), LabelsAccess::Allowed);
    clear(&a);
    (a, s)
}

#[test]
fn l1_c12a_without_a_key_labels_are_off_and_nothing_is_asked() {
    let a = app();
    let s = linked_session(&a, P, "TEST-1802");
    assert_eq!(access(&a), LabelsAccess::Off(ConnectionState::NoKey));
    assert_eq!(
        labels(&a, &[s], true),
        vec![SessionLabel {
            session_id: s,
            product_id: Some(P.into()),
            state: None,
        }]
    );
    assert!(label_requests(&a).is_empty());
}

#[test]
fn l1_c12b_a_key_without_labels_is_asked_once_and_remembered_denied() {
    let a = app();
    a.fake_pricing().deny_labels();
    let connection = a.app.conversations.key_set(Some(PRICING_KEY)).unwrap();
    assert_eq!(connection.state, ConnectionState::Ready);
    assert_eq!(
        label_requests(&a),
        vec![format!("GET /stores/{}/labels/templates", store_id(&a))]
    );
    assert_eq!(access(&a), LabelsAccess::Denied);
    assert_eq!(labels_changed(&a), vec![(vec![], true)]);
}

#[test]
fn l1_c12c_a_denied_key_asks_nothing_more() {
    let a = app();
    let s = linked_session(&a, P, "TEST-1802");
    a.fake_pricing().deny_labels();
    a.app.conversations.key_set(Some(PRICING_KEY)).unwrap();
    a.fake_pricing().clear_requests();
    for _ in 0..2 {
        let got = labels(&a, &[s], true);
        assert!(got.iter().all(|l| l.state.is_none()), "{got:?}");
    }
    assert!(label_requests(&a).is_empty());
}

#[test]
fn l1_c12d_an_allowed_key_is_remembered_across_a_restart() {
    let mut a = app();
    a.app.conversations.key_set(Some(PRICING_KEY)).unwrap();
    assert_eq!(access(&a), LabelsAccess::Allowed);
    a.fake_pricing().clear_requests();
    a.kill_and_restart().unwrap();
    assert_eq!(access(&a), LabelsAccess::Allowed);
    assert!(label_requests(&a).is_empty());
}

#[test]
fn l1_c12e_a_failed_templates_read_leaves_the_connection_alone() {
    let a = app();
    a.fake_pricing().fail_matching("/labels/templates", 1, 500);
    let connection = a.app.conversations.key_set(Some(PRICING_KEY)).unwrap();
    assert_eq!(connection.state, ConnectionState::Ready);
    assert_eq!(connection.last_error, None);
    assert_eq!(access(&a), LabelsAccess::Unknown);
}

#[test]
fn l1_c12f_a_successful_label_call_learns_allowed() {
    let a = app();
    let s = linked_session(&a, P, "TEST-1802");
    a.fake_pricing().fail_matching("/labels/templates", 1, 500);
    a.app.conversations.key_set(Some(PRICING_KEY)).unwrap();
    a.fake_pricing().clear_requests();
    labels(&a, &[s], true);
    let requests = label_requests(&a);
    assert_eq!(requests.len(), 1, "{requests:?}");
    assert!(
        requests[0].starts_with(&format!("GET /stores/{}/labels/status?", store_id(&a))),
        "{requests:?}"
    );
    assert_eq!(access(&a), LabelsAccess::Allowed);
}

#[test]
fn l1_c12g_a_refused_label_call_denies_labels_but_keeps_conversations() {
    let (mut a, s) = connected_allowed();
    a.fake_pricing().deny_labels();
    let error = a.app.labels.session_labels(&[s], true).unwrap_err();
    assert!(
        matches!(error, LabelsFailure::Provider(LabelsError::NotAllowed)),
        "{error:?}"
    );
    assert_eq!(access(&a), LabelsAccess::Denied);
    assert_eq!(labels_changed(&a), vec![(vec![], true)]);
    assert_eq!(
        a.app.conversations.connection().unwrap().state,
        ConnectionState::Ready
    );
    a.fake_pricing().clear_requests();
    a.clock.advance(10_000);
    a.app.conversations.tick(a.clock.now_ms()).unwrap();
    assert!(
        a.fake_pricing()
            .request_lines()
            .iter()
            .any(|line| line.contains("/events")),
        "{:?}",
        a.fake_pricing().request_lines()
    );
    a.kill_and_restart().unwrap();
    assert_eq!(access(&a), LabelsAccess::Denied);
}

#[test]
fn l1_c12h_a_changed_server_turns_labels_off() {
    let (a, s) = connected_allowed();
    a.app
        .conversations
        .settings_save(&ConversationSettingsPatch {
            server: Some("https://pricing-other.example".into()),
            ..ConversationSettingsPatch::default()
        })
        .unwrap();
    labels(&a, &[s], true);
    assert_eq!(
        access(&a),
        LabelsAccess::Off(ConnectionState::StoreUnmatched)
    );
    assert!(label_requests(&a).is_empty());
}

#[test]
fn l1_c12i_a_key_change_forgets_the_access() {
    // (1) The key removed.
    let (a, _) = connected_allowed();
    a.app.conversations.key_set(None).unwrap();
    assert_eq!(access(&a), LabelsAccess::Off(ConnectionState::NoKey));
    assert_eq!(stored_access(&a), LabelsAccess::Unknown);
    // (2) Another key whose stores read fails.
    let (b, _) = connected_allowed();
    b.fake_pricing().refuse_connection_next(1);
    let _ = b.app.conversations.key_set(Some("k-other"));
    assert_eq!(stored_access(&b), LabelsAccess::Unknown);
}
