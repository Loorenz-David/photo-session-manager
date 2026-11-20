//! C4 (`docs/implementation/plans/C4_visit_and_quote_jump.md`). U1 C4·C8: the reads a visit makes
//! (the session view, the conversation view, its attachable photos) leave the active session and
//! the capture folder alone (CR4, MC-C3 invariant), on `TestApp::with_pricing()` after a first
//! connection. U2 C4·C10: the quote's `jump_key` in `ConversationsService::view` (computed, never
//! stored).

use psm_core::{
    mechanisms::canon::canon,
    ports::{clock::Clock, conversations::ConversationStore},
};
use psm_integrations::conversations::beyo_pricing::types::Quote;
use psm_testkit::{TestApp, fake_pricing::FakePricing, test_app::PRICING_KEY};
use rusqlite::Connection;
use std::fs;

/// A connected app after its first connection.
fn connected() -> TestApp {
    let a = TestApp::with_pricing().unwrap();
    a.clock.advance(1_000_000);
    a.app.conversations.key_set(Some(PRICING_KEY)).unwrap();
    run_check(&a);
    a
}

fn pricing(a: &TestApp) -> &FakePricing {
    a.fake_pricing()
}

fn run_check(a: &TestApp) {
    let mut n = 0;
    while a.app.conversations.tick(a.clock.now_ms()).unwrap().more {
        n += 1;
        assert!(n < 50, "the check never completed");
    }
}

/// The pricer asks a question on `sku`, and the next check brings it into the copy.
fn asked(a: &TestApp, sku: &str) -> String {
    let store_id = a.app.store.sync().unwrap().store.unwrap().id;
    let case = pricing(a).ask(&store_id, Some(sku), "Condition");
    a.clock.advance(10_000);
    run_check(a);
    case
}

fn session(a: &TestApp, sku: &str) -> i64 {
    a.items()
        .create_local(sku, sku, "object", None)
        .unwrap()
        .1
        .session
        .id
}

/// Session A activated, then the reads a visit to session B makes.
fn visited(a: &TestApp) -> (i64, i64) {
    let session_a = session(a, "BEYO-A");
    let session_b = session(a, "BEYO-B");
    let case = asked(a, "BEYO-B");
    a.sessions().activate(session_a).unwrap();
    // The session view (`session_get`), the conversation view and its attachable photos.
    a.sessions().view(session_b).unwrap();
    let view = a.app.conversations.view(&case).unwrap();
    assert_eq!(view.session_id, Some(session_b));
    a.app.conversations.attachable(&case).unwrap();
    (session_a, session_b)
}

#[test]
fn c4_c8a_the_reads_of_a_visit_leave_the_active_session_alone() {
    let a = connected();
    let (session_a, _) = visited(&a);
    assert_eq!(a.sessions().active().unwrap(), Some(session_a));
}

#[test]
fn c4_c8b_after_the_reads_of_a_visit_a_capture_lands_in_the_active_session() {
    let a = connected();
    let (session_a, _) = visited(&a);
    // A capture arrives in the folder Capture One captures into.
    let folder = a.fake.state.lock().unwrap().captures.clone();
    let path = std::path::Path::new(&folder).join("C4-0001.ARW");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, b"raw").unwrap();
    a.deliver_hook(
        &serde_json::json!({"hook": "capture_done", "argv": [canon(&path).unwrap()]}).to_string(),
    )
    .unwrap();
    let owner: i64 = Connection::open(a.app.paths.db())
        .unwrap()
        .query_row(
            "SELECT session_id FROM photos WHERE raw_path = ?1",
            [canon(&path).unwrap()],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(owner, session_a);
}

/// The pricer asks on `sku` with the first message quoting `field`; the next check brings it in.
fn asked_quoting(a: &TestApp, sku: &str, field: &str) -> String {
    let store_id = a.app.store.sync().unwrap().store.unwrap().id;
    let case = pricing(a).ask(&store_id, Some(sku), "Details");
    pricing(a).change_silently(&case, |c| {
        c.messages[0].quote = Some(Quote {
            field: field.into(),
            field_label: "Field".into(),
            text: "Is this right?".into(),
        });
    });
    a.clock.advance(10_000);
    run_check(a);
    case
}

/// `custom.designer` imported for `shopify` in the Details step, then the item of `BEYO-1880`.
fn with_designer_field() -> TestApp {
    let a = connected();
    a.research_fields(&["custom.designer"]).unwrap();
    session(&a, "BEYO-1880");
    a
}

/// The `jump_key` of the view's first message.
fn first_jump_key(a: &TestApp, case: &str) -> Option<String> {
    let view = a.app.conversations.view(case).unwrap();
    let first = &view.messages[0];
    assert!(first.quote.is_some(), "the first message quotes a field");
    view.jump_keys.get(&first.id).cloned()
}

#[test]
fn c4_c10a_a_quoted_metafield_the_form_has_gets_its_jump_key() {
    let a = with_designer_field();
    let case = asked_quoting(&a, "BEYO-1880", "mf:custom.designer");
    assert_eq!(
        first_jump_key(&a, &case).as_deref(),
        Some("custom.designer")
    );
    // Never stored: the local copy's quote is the provider's, unchanged.
    let stored = a.app.store.conversation(&case).unwrap().unwrap();
    let quote: String = Connection::open(a.app.paths.db())
        .unwrap()
        .query_row(
            "SELECT quote FROM conversation_messages WHERE id = ?1",
            [&stored.messages[0].id],
            |r| r.get(0),
        )
        .unwrap();
    assert!(!quote.contains("jump"), "{quote}");
}

#[test]
fn c4_c10b_a_quoted_field_the_form_does_not_have_gets_none() {
    let a = with_designer_field();
    let case = asked_quoting(&a, "BEYO-1880", "mf:custom.unknown_field");
    assert_eq!(first_jump_key(&a, &case), None);
}

#[test]
fn c4_c10c_a_conversation_not_on_this_mac_gets_none() {
    let a = with_designer_field();
    let case = asked_quoting(&a, "BEYO-ELSEWHERE", "mf:custom.designer");
    assert_eq!(a.app.conversations.view(&case).unwrap().session_id, None);
    assert_eq!(first_jump_key(&a, &case), None);
}

#[test]
fn c4_c10d_a_quoted_title_names_the_product_title() {
    let a = with_designer_field();
    let case = asked_quoting(&a, "BEYO-1880", "title");
    assert_eq!(first_jump_key(&a, &case).as_deref(), Some("product.title"));
}
