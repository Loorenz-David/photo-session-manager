//! C3·U1 (`docs/implementation/plans/C3_conversation_and_answering.md` C3·C2–C8): the outbox,
//! the send with `clientMutationId`, retry, discard, Mark answered, the view and the attachable
//! photos (MC-C2), through `ConversationsService` on `TestApp::with_pricing()` after a first
//! connection, driven by `tick` with the manual clock. Requests and events are counted from the
//! moment each test clears them.

use psm_core::{
    Error,
    domain::{
        composition::OutputSlot,
        conversation::{
            AttachReason, AttachRole, AuthorKind, ConnectionState, Conversation,
            ConversationNotice, ConversationStatus, NewMessage, OutboxMessage, OutboxState,
            PendingView, ResolvedBy,
        },
        photo::Classification as C,
        publish::PhotoLink,
    },
    events::AppEvent,
    ports::{clock::Clock, conversations::ConversationStore},
    services::conversations::ConversationError,
};
use psm_testkit::{TestApp, fake_pricing::FakePricing, test_app::PRICING_KEY};
use serde_json::json;

const SKU: &str = "BEYO-1880";
const REFUSED: &str = "Pricing refused the key: check Settings → Pricing";

/// A connected app after its first connection; requests and events cleared.
fn connected() -> TestApp {
    let a = TestApp::with_pricing().unwrap();
    a.clock.advance(1_000_000);
    a.app.conversations.key_set(Some(PRICING_KEY)).unwrap();
    run_check(&a);
    clear(&a);
    a
}

fn pricing(a: &TestApp) -> &FakePricing {
    a.fake_pricing()
}

fn store_id(a: &TestApp) -> String {
    a.app.store.sync().unwrap().store.unwrap().id
}

fn clear(a: &TestApp) {
    pricing(a).clear_requests();
    a.events.0.lock().unwrap().clear();
}

fn tick(a: &TestApp) -> bool {
    a.app.conversations.tick(a.clock.now_ms()).unwrap().more
}

fn run_check(a: &TestApp) {
    let mut n = 0;
    while tick(a) {
        n += 1;
        assert!(n < 50, "the check never completed");
    }
}

fn next_check(a: &TestApp) {
    a.clock.advance(10_000);
    run_check(a);
}

/// The pricer asks a question on `sku`, and the next check brings it into the copy.
fn asked(a: &TestApp, sku: Option<&str>) -> String {
    let case = pricing(a).ask(&store_id(a), sku, "Condition");
    next_check(a);
    clear(a);
    case
}

fn input(cmid: &str, text: &str) -> NewMessage {
    NewMessage {
        client_mutation_id: cmid.into(),
        text: text.into(),
        reply_to_message_id: None,
        attachment_media_ids: vec![],
    }
}

fn reply(a: &TestApp, case: &str, cmid: &str, text: &str) -> PendingView {
    a.app.conversations.reply(case, input(cmid, text)).unwrap()
}

fn outbox(a: &TestApp) -> Vec<OutboxMessage> {
    a.app.store.outbox().unwrap()
}

fn row(a: &TestApp, cmid: &str) -> OutboxMessage {
    outbox(a)
        .into_iter()
        .find(|r| r.client_mutation_id == cmid)
        .unwrap_or_else(|| panic!("no outbox row {cmid}"))
}

fn find(a: &TestApp, id: &str) -> Conversation {
    a.app.store.conversation(id).unwrap().expect("in the copy")
}

fn posts(a: &TestApp) -> Vec<serde_json::Value> {
    pricing(a)
        .requests()
        .into_iter()
        .filter(|r| r.method == "POST")
        .map(|r| r.body.unwrap_or_default())
        .collect()
}

fn patches(a: &TestApp) -> usize {
    pricing(a)
        .requests()
        .iter()
        .filter(|r| r.method == "PATCH")
        .count()
}

/// The collected `ConversationsChanged` events: (notices, chime).
fn changed(a: &TestApp) -> Vec<(Vec<ConversationNotice>, bool)> {
    a.events
        .0
        .lock()
        .unwrap()
        .iter()
        .filter_map(|e| match e {
            AppEvent::ConversationsChanged { notices, chime, .. } => {
                Some((notices.clone(), *chime))
            }
            _ => None,
        })
        .collect()
}

/// The ids of the messages of the app's own account the fake holds on the case.
fn own_messages_on_fake(a: &TestApp, case: &str) -> usize {
    pricing(a).message_ids(case).len() - 1
}

fn count_of(c: &Conversation, message_id: &str) -> usize {
    c.messages.iter().filter(|m| m.id == message_id).count()
}

/// The key is refused at the next check.
fn refused(a: &TestApp) {
    pricing(a).fail_matching("/events", 1, 401);
    next_check(a);
    assert_eq!(
        a.app.conversations.connection().unwrap().state,
        ConnectionState::Refused
    );
    clear(a);
}

fn storage(error: ConversationError) -> Error {
    match error {
        ConversationError::Storage(e) => e,
        other => panic!("expected a local error, got {other:?}"),
    }
}

// ---------- published photos ----------

/// A session whose item has `sku`: (session id, item id).
fn session(a: &TestApp, sku: &str) -> (i64, i64) {
    let (item, session) = a.items().create_local(sku, sku, "object", None).unwrap();
    (session.session.id, item.id)
}

/// A preview file `name` in the test's folder (H4, M-H4: a preview path is offered only when its
/// file exists); its path.
fn preview_file(a: &TestApp, name: &str) -> String {
    let path = a.dir.path().join(name);
    std::fs::write(&path, b"jpeg").unwrap();
    path.to_string_lossy().into_owned()
}

fn photo(a: &TestApp, session_id: i64, name: &str, class: C) -> (i64, String) {
    let p = a.add_photo(session_id, &format!("{name}.ARW")).unwrap();
    a.review().classify(p.id, class).unwrap();
    let preview = preview_file(a, &format!("orig-{name}.jpg"));
    a.store()
        .set_original_preview(&p.raw_path, Some(&preview), None)
        .unwrap();
    (p.id, p.raw_path)
}

fn link(
    a: &TestApp,
    photo_id: i64,
    item_id: i64,
    role: &str,
    slot: OutputSlot,
    external: Option<&str>,
    state: &str,
) {
    a.store()
        .upsert_photo_link(
            &PhotoLink {
                photo_id,
                item_id,
                role: role.into(),
                slot,
                external_id: external.map(str::to_owned),
                upload_state: state.into(),
                export_sha256: None,
                superseded_ids: vec![],
                revision: 0,
                retouch_gen: 0,
                composition_rev: 0,
            },
            a.clock.now_ms(),
        )
        .unwrap();
}

const CONDITION_ROLE: &str = "attribute:custom.damage_report";

/// C3·C8(a)'s product: gallery 1 (adjusted and original previews), gallery 2, one condition
/// photo, each with a `ready` link and an external id. Returns (session id, item id, photo ids).
fn published(a: &TestApp, sku: &str) -> (i64, i64, [i64; 3]) {
    let (s, item) = session(a, sku);
    let (g1, raw1) = photo(a, s, "g1", C::Main);
    a.store()
        .set_missing_preview_paths(&raw1, &preview_file(a, "adj-g1.jpg"))
        .unwrap();
    let (g2, _) = photo(a, s, "g2", C::Main);
    let (c1, _) = photo(a, s, "c1", C::Condition);
    link(
        a,
        g1,
        item,
        "gallery",
        OutputSlot::Single,
        Some("media-g1"),
        "ready",
    );
    link(
        a,
        g2,
        item,
        "gallery",
        OutputSlot::Single,
        Some("media-g2"),
        "ready",
    );
    link(
        a,
        c1,
        item,
        CONDITION_ROLE,
        OutputSlot::Single,
        Some("media-c1"),
        "ready",
    );
    (s, item, [g1, g2, c1])
}

fn media_ids(a: &TestApp, case: &str) -> Vec<String> {
    a.app
        .conversations
        .attachable(case)
        .unwrap()
        .photos
        .into_iter()
        .map(|p| p.media_id)
        .collect()
}

// ---------- C3·C2 accepting a reply ----------

#[test]
fn c3_c2a_a_reply_is_pending_at_once_without_a_request() {
    let a = connected();
    let case = asked(&a, Some(SKU));
    let pending = reply(&a, &case, "m-1", "Original glass.");
    assert_eq!(pending.state, OutboxState::Sending);
    assert_eq!(pending.client_mutation_id, "m-1");
    assert_eq!(pending.text, "Original glass.");
    assert!(posts(&a).is_empty());
    let events = changed(&a);
    assert_eq!(events.len(), 1);
    assert!(events[0].0.is_empty());
}

#[test]
fn c3_c2b_the_same_client_id_twice_keeps_one_row_with_the_first_text() {
    let a = connected();
    let case = asked(&a, Some(SKU));
    reply(&a, &case, "m-1", "Original glass.");
    let second = reply(&a, &case, "m-1", "Something else.");
    assert_eq!(second.text, "Original glass.");
    let rows = outbox(&a);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].text, "Original glass.");
}

#[test]
fn c3_c2c_an_unknown_conversation_or_a_foreign_reply_target_is_refused() {
    let a = connected();
    let one = asked(&a, Some(SKU));
    let other = asked(&a, Some("BEYO-7"));
    let foreign = find(&a, &other).messages[0].id.clone();
    let unknown = a
        .app
        .conversations
        .reply("case-unknown", input("m-1", "t"))
        .unwrap_err();
    assert!(matches!(storage(unknown), Error::NotFound(_)));
    let mut with_target = input("m-2", "t");
    with_target.reply_to_message_id = Some(foreign);
    let refused = a.app.conversations.reply(&one, with_target).unwrap_err();
    match storage(refused) {
        Error::InvalidValue(field) => assert_eq!(field, "reply_to_message_id"),
        other => panic!("expected invalid_value, got {other:?}"),
    }
    assert!(outbox(&a).is_empty());
}

// ---------- C3·C3 the send ----------

#[test]
fn c3_c3a_the_next_tick_sends_once_with_the_client_id() {
    let a = connected();
    let case = asked(&a, Some(SKU));
    reply(&a, &case, "m-1", "Original glass.");
    tick(&a);
    let bodies = posts(&a);
    assert_eq!(bodies.len(), 1);
    assert_eq!(bodies[0]["clientMutationId"], json!("m-1"));
    assert_eq!(bodies[0]["text"], json!("Original glass."));
    let fake_id = pricing(&a).message_ids(&case).last().unwrap().clone();
    let sent = row(&a, "m-1");
    assert_eq!(sent.state, OutboxState::Sent);
    assert_eq!(sent.server_message_id.as_deref(), Some(fake_id.as_str()));
    let c = find(&a, &case);
    let last = c.messages.last().unwrap();
    assert_eq!(last.id, fake_id);
    assert_eq!(last.author.kind, AuthorKind::Me);
    assert_eq!(c.status, ConversationStatus::Answered);
}

#[test]
fn c3_c3b_the_reply_target_and_the_photos_travel_in_order() {
    let a = connected();
    published(&a, SKU);
    let case = asked(&a, Some(SKU));
    let target = find(&a, &case).messages[0].id.clone();
    let media = media_ids(&a, &case);
    let mut message = input("m-1", "Here they are.");
    message.reply_to_message_id = Some(target.clone());
    message.attachment_media_ids = vec![media[1].clone(), media[0].clone()];
    a.app.conversations.reply(&case, message).unwrap();
    tick(&a);
    let bodies = posts(&a);
    assert_eq!(bodies.len(), 1);
    assert_eq!(bodies[0]["replyToMessageId"], json!(target));
    assert_eq!(bodies[0]["attachmentMediaIds"], json!([media[1], media[0]]));
}

#[test]
fn c3_c3c_the_echo_of_the_reply_is_recognised_by_its_id() {
    let a = connected();
    let case = asked(&a, Some(SKU));
    reply(&a, &case, "m-1", "Original glass.");
    tick(&a);
    let sent_id = row(&a, "m-1").server_message_id.unwrap();
    clear(&a);
    next_check(&a);
    assert!(
        pricing(&a)
            .request_lines()
            .iter()
            .any(|l| l.ends_with(&format!("/cases/{case}"))),
        "the echo re-read the conversation"
    );
    assert_eq!(count_of(&find(&a, &case), &sent_id), 1);
    let events = changed(&a);
    assert!(!events.is_empty());
    for (notices, chime) in events {
        assert!(notices.is_empty());
        assert!(!chime);
    }
}

#[test]
fn c3_c3d_replying_to_a_resolved_conversation_reopens_it_as_answered() {
    let a = connected();
    let case = asked(&a, Some(SKU));
    pricing(&a).resolve(&case);
    next_check(&a);
    let before = find(&a, &case);
    assert_eq!(before.status, ConversationStatus::Resolved);
    assert_eq!(before.resolved_by, Some(ResolvedBy::Asker));
    reply(&a, &case, "m-1", "Reopening.");
    tick(&a);
    let after = find(&a, &case);
    assert_eq!(after.status, ConversationStatus::Answered);
    assert_eq!(after.resolved_by, None);
}

#[test]
fn c3_c3e_the_view_after_the_send_ends_with_the_message_and_no_pending() {
    let a = connected();
    let case = asked(&a, Some(SKU));
    reply(&a, &case, "m-1", "Original glass.");
    tick(&a);
    let sent_id = row(&a, "m-1").server_message_id.unwrap();
    let view = a.app.conversations.view(&case).unwrap();
    assert_eq!(view.messages.last().unwrap().id, sent_id);
    assert!(view.pending.is_empty());
}

// ---------- C3·C4 failures ----------

#[test]
fn c3_c4a_a_timeout_fails_the_reply_and_it_is_never_resent_by_itself() {
    let a = connected();
    let case = asked(&a, Some(SKU));
    pricing(&a).timeout_next(1);
    reply(&a, &case, "m-1", "Original glass.");
    tick(&a);
    let failed = row(&a, "m-1");
    assert_eq!(failed.state, OutboxState::Failed);
    assert_eq!(failed.failure.as_deref(), Some("Couldn't send"));
    for _ in 0..60 {
        next_check(&a);
    }
    assert_eq!(own_messages_on_fake(&a, &case), 0);
    assert_eq!(posts(&a).len(), 1);
    assert_eq!(row(&a, "m-1").state, OutboxState::Failed);
}

#[test]
fn c3_c4b_retry_sends_the_same_client_id_once_more() {
    let a = connected();
    let case = asked(&a, Some(SKU));
    pricing(&a).timeout_next(1);
    reply(&a, &case, "m-1", "Original glass.");
    tick(&a);
    a.app.conversations.retry("m-1").unwrap();
    tick(&a);
    let bodies = posts(&a);
    assert_eq!(bodies.len(), 2);
    assert_eq!(bodies[1]["clientMutationId"], json!("m-1"));
    assert_eq!(row(&a, "m-1").state, OutboxState::Sent);
    assert_eq!(own_messages_on_fake(&a, &case), 1);
}

#[test]
fn c3_c4c_a_lost_answer_shows_the_message_and_the_failed_row_until_retry() {
    let a = connected();
    let case = asked(&a, Some(SKU));
    pricing(&a).drop_answer_next(1);
    reply(&a, &case, "m-1", "Original glass.");
    tick(&a);
    assert_eq!(row(&a, "m-1").state, OutboxState::Failed);
    next_check(&a);
    let delivered = pricing(&a).message_ids(&case).last().unwrap().clone();
    assert_eq!(count_of(&find(&a, &case), &delivered), 1);
    let view = a.app.conversations.view(&case).unwrap();
    assert_eq!(view.pending.len(), 1);
    assert_eq!(view.pending[0].state, OutboxState::Failed);
    a.app.conversations.retry("m-1").unwrap();
    tick(&a);
    let sent = row(&a, "m-1");
    assert_eq!(sent.state, OutboxState::Sent);
    assert_eq!(sent.server_message_id.as_deref(), Some(delivered.as_str()));
    assert_eq!(count_of(&find(&a, &case), &delivered), 1);
    assert_eq!(own_messages_on_fake(&a, &case), 1);
}

#[test]
fn c3_c4d_a_refused_attachment_fails_with_the_providers_words() {
    let a = connected();
    let case = asked(&a, Some(SKU));
    pricing(&a).fail_matching_with(
        "/messages",
        1,
        400,
        "invalid_attachment",
        "Not a photo of this product.",
    );
    reply(&a, &case, "m-1", "Original glass.");
    tick(&a);
    let failed = row(&a, "m-1");
    assert_eq!(failed.state, OutboxState::Failed);
    assert_eq!(
        failed.failure.as_deref(),
        Some("Pricing did not accept this reply: Not a photo of this product.")
    );
}

#[test]
fn c3_c4e_a_404_fails_as_no_longer_existing() {
    let a = connected();
    let case = asked(&a, Some(SKU));
    pricing(&a).fail_matching("/messages", 1, 404);
    reply(&a, &case, "m-1", "Original glass.");
    tick(&a);
    let failed = row(&a, "m-1");
    assert_eq!(failed.state, OutboxState::Failed);
    assert_eq!(
        failed.failure.as_deref(),
        Some("This conversation no longer exists")
    );
}

#[test]
fn c3_c4f_a_401_fails_with_the_refused_sentence_and_refuses_the_connection() {
    let a = connected();
    let case = asked(&a, Some(SKU));
    pricing(&a).fail_matching("/messages", 1, 401);
    reply(&a, &case, "m-1", "Original glass.");
    tick(&a);
    let failed = row(&a, "m-1");
    assert_eq!(failed.state, OutboxState::Failed);
    assert_eq!(failed.failure.as_deref(), Some(REFUSED));
    assert_eq!(
        a.app.conversations.connection().unwrap().state,
        ConnectionState::Refused
    );
}

#[test]
fn c3_c4g_after_a_timeout_the_other_replies_fail_without_a_request() {
    let a = connected();
    let case = asked(&a, Some(SKU));
    pricing(&a).timeout_next(1);
    reply(&a, &case, "m-1", "First.");
    reply(&a, &case, "m-2", "Second.");
    tick(&a);
    for cmid in ["m-1", "m-2"] {
        let failed = row(&a, cmid);
        assert_eq!(failed.state, OutboxState::Failed);
        assert_eq!(failed.failure.as_deref(), Some("Couldn't send"));
    }
    assert_eq!(posts(&a).len(), 1);
}

// ---------- C3·C5 restart, discard, retry of other states ----------

#[test]
fn c3_c5a_a_reply_survives_a_restart_and_is_sent_once() {
    let mut a = connected();
    let case = asked(&a, Some(SKU));
    reply(&a, &case, "m-1", "Original glass.");
    a.kill_and_restart().unwrap();
    assert_eq!(row(&a, "m-1").state, OutboxState::Sending);
    tick(&a);
    let bodies = posts(&a);
    assert_eq!(bodies.len(), 1);
    assert_eq!(bodies[0]["clientMutationId"], json!("m-1"));
    assert_eq!(own_messages_on_fake(&a, &case), 1);
    assert_eq!(row(&a, "m-1").state, OutboxState::Sent);
}

#[test]
fn c3_c5b_a_failed_reply_stays_failed_after_a_restart() {
    let mut a = connected();
    let case = asked(&a, Some(SKU));
    pricing(&a).timeout_next(1);
    reply(&a, &case, "m-1", "Original glass.");
    tick(&a);
    a.kill_and_restart().unwrap();
    clear(&a);
    run_check(&a);
    for _ in 0..3 {
        next_check(&a);
    }
    assert_eq!(row(&a, "m-1").state, OutboxState::Failed);
    assert!(posts(&a).is_empty());
}

#[test]
fn c3_c5c_discard_removes_a_failed_reply_and_refuses_a_sending_one() {
    let a = connected();
    let case = asked(&a, Some(SKU));
    pricing(&a).timeout_next(1);
    reply(&a, &case, "m-1", "First.");
    tick(&a);
    clear(&a);
    a.app.conversations.discard("m-1").unwrap();
    assert!(outbox(&a).iter().all(|r| r.client_mutation_id != "m-1"));
    assert!(pricing(&a).requests().is_empty());
    reply(&a, &case, "m-2", "Second.");
    let refused = a.app.conversations.discard("m-2").unwrap_err();
    assert!(matches!(storage(refused), Error::InvalidValue(_)));
    assert_eq!(row(&a, "m-2").state, OutboxState::Sending);
}

#[test]
fn c3_c5d_retry_leaves_a_sent_or_sending_reply_unchanged() {
    let a = connected();
    let case = asked(&a, Some(SKU));
    reply(&a, &case, "m-1", "First.");
    tick(&a);
    clear(&a);
    reply(&a, &case, "m-2", "Second.");
    let sent_before = row(&a, "m-1");
    a.app.conversations.retry("m-1").unwrap();
    a.app.conversations.retry("m-2").unwrap();
    assert_eq!(row(&a, "m-1"), sent_before);
    assert_eq!(row(&a, "m-2").state, OutboxState::Sending);
    tick(&a);
    let bodies = posts(&a);
    assert_eq!(bodies.len(), 1);
    assert_eq!(bodies[0]["clientMutationId"], json!("m-2"));
}

// ---------- C3·C6 refused and hidden states ----------

#[test]
fn c3_c6a_a_reply_while_refused_is_created_failed_and_never_sent() {
    let a = connected();
    let case = asked(&a, Some(SKU));
    refused(&a);
    let pending = reply(&a, &case, "m-1", "Original glass.");
    assert_eq!(pending.state, OutboxState::Failed);
    assert_eq!(pending.failure.as_deref(), Some(REFUSED));
    a.clock.advance(10_000);
    tick(&a);
    assert!(pricing(&a).requests().is_empty());
}

#[test]
fn c3_c6b_the_view_says_whether_a_reply_can_be_sent() {
    let a = connected();
    let case = asked(&a, Some(SKU));
    let ready = a.app.conversations.view(&case).unwrap();
    assert!(ready.can_send);
    assert_eq!(ready.off_reason, None);
    refused(&a);
    let off = a.app.conversations.view(&case).unwrap();
    assert!(!off.can_send);
    assert_eq!(off.off_reason.as_deref(), Some(REFUSED));
}

#[test]
fn c3_c6c_without_a_key_a_reply_is_refused_as_off_and_nothing_is_written() {
    let a = connected();
    let case = asked(&a, Some(SKU));
    a.app.conversations.key_set(None).unwrap();
    match a.app.conversations.reply(&case, input("m-1", "t")) {
        Err(ConversationError::Off { state, .. }) => assert_eq!(state, ConnectionState::NoKey),
        other => panic!("expected conversations_off, got {other:?}"),
    }
    assert!(outbox(&a).is_empty());
}

// ---------- C3·C7 Mark answered and the writes ----------

#[test]
fn c3_c7a_mark_answered_patches_once_and_updates_the_copy() {
    let a = connected();
    let case = asked(&a, Some(SKU));
    a.app.conversations.mark_answered(&case).unwrap();
    let patches: Vec<_> = pricing(&a)
        .requests()
        .into_iter()
        .filter(|r| r.method == "PATCH")
        .collect();
    assert_eq!(patches.len(), 1);
    assert_eq!(patches[0].body, Some(json!({"status": "answered"})));
    assert_eq!(find(&a, &case).status, ConversationStatus::Answered);
    assert_eq!(changed(&a).len(), 1);
}

#[test]
fn c3_c7b_mark_answered_on_a_conversation_resolved_meanwhile() {
    let a = connected();
    let case = asked(&a, Some(SKU));
    pricing(&a).resolve(&case);
    let error = a.app.conversations.mark_answered(&case).unwrap_err();
    assert!(matches!(error, ConversationError::Resolved));
    assert_eq!(find(&a, &case).status, ConversationStatus::Resolved);
}

#[test]
fn c3_c7c_mark_answered_when_pricing_is_unreachable() {
    let a = connected();
    let case = asked(&a, Some(SKU));
    let before = find(&a, &case);
    pricing(&a).timeout_next(1);
    match a.app.conversations.mark_answered(&case) {
        Err(ConversationError::Provider(error)) => assert_eq!(
            a.app.conversations.describe(&error),
            "Could not reach Pricing"
        ),
        other => panic!("expected a provider error, got {other:?}"),
    }
    assert_eq!(find(&a, &case), before);
}

#[test]
fn c3_c7d_the_only_writes_are_replies_and_mark_answered() {
    let a = TestApp::with_pricing().unwrap();
    a.clock.advance(1_000_000);
    a.app.conversations.key_set(Some(PRICING_KEY)).unwrap();
    run_check(&a);
    published(&a, SKU);
    let one = pricing(&a).ask(&store_id(&a), Some(SKU), "Condition");
    let two = pricing(&a).ask(&store_id(&a), Some("BEYO-7"), "Condition");
    next_check(&a);
    a.app.conversations.mark_read(&one).unwrap();
    let mut with_photo = input("m-1", "A photo.");
    with_photo.attachment_media_ids = vec![media_ids(&a, &one)[0].clone()];
    a.app.conversations.reply(&one, with_photo).unwrap();
    tick(&a);
    pricing(&a).timeout_next(1);
    reply(&a, &one, "m-2", "Lost.");
    tick(&a);
    a.app.conversations.retry("m-2").unwrap();
    tick(&a);
    a.app.conversations.mark_answered(&two).unwrap();
    pricing(&a).timeout_next(1);
    reply(&a, &two, "m-3", "Removed.");
    tick(&a);
    a.app.conversations.discard("m-3").unwrap();
    next_check(&a);
    // The whole record since the app was built.
    let requests = pricing(&a).requests();
    assert!(requests.iter().any(|r| r.method == "POST"));
    assert!(requests.iter().any(|r| r.method == "PATCH"));
    for r in requests {
        let allowed = r.method == "GET"
            || (r.method == "POST" && r.path.ends_with("/messages"))
            || (r.method == "PATCH" && r.body == Some(json!({"status": "answered"})));
        assert!(allowed, "unexpected write {}", r.line());
    }
}

#[test]
fn c3_c7e_mark_answered_while_refused_is_off_without_a_request() {
    let a = connected();
    let case = asked(&a, Some(SKU));
    refused(&a);
    let before = find(&a, &case);
    match a.app.conversations.mark_answered(&case) {
        Err(ConversationError::Off { state, message }) => {
            assert_eq!(state, ConnectionState::Refused);
            assert_eq!(message, REFUSED);
        }
        other => panic!("expected conversations_off, got {other:?}"),
    }
    assert_eq!(patches(&a), 0);
    assert_eq!(find(&a, &case), before);
}

// ---------- C3·C8 the attachable photos ----------

#[test]
fn c3_c8a_published_gallery_and_condition_photos_are_offered() {
    let a = connected();
    let (_, _, [g1, g2, c1]) = published(&a, SKU);
    let file = |name: &str| a.dir.path().join(name).to_string_lossy().into_owned();
    let case = asked(&a, Some(SKU));
    let attachable = a.app.conversations.attachable(&case).unwrap();
    assert_eq!(attachable.reason, None);
    let got: Vec<_> = attachable
        .photos
        .iter()
        .map(|p| {
            (
                p.media_id.as_str(),
                p.photo_id,
                p.role,
                p.preview_path.as_deref(),
            )
        })
        .collect();
    assert_eq!(
        got,
        [
            (
                "media-g1",
                g1,
                AttachRole::Gallery,
                Some(file("adj-g1.jpg").as_str())
            ),
            (
                "media-g2",
                g2,
                AttachRole::Gallery,
                Some(file("orig-g2.jpg").as_str())
            ),
            (
                "media-c1",
                c1,
                AttachRole::Condition,
                Some(file("orig-c1.jpg").as_str())
            ),
        ]
    );
}

#[test]
fn c3_c8b_unpublished_retired_and_deleted_photos_are_not_offered() {
    let a = connected();
    let (s, item, _) = published(&a, SKU);
    let (no_id, _) = photo(&a, s, "n1", C::Main);
    link(
        &a,
        no_id,
        item,
        "gallery",
        OutputSlot::Single,
        None,
        "ready",
    );
    let (retired, _) = photo(&a, s, "r1", C::Main);
    link(
        &a,
        retired,
        item,
        "gallery",
        OutputSlot::Single,
        Some("media-r1"),
        "retired",
    );
    let (deleted, _) = photo(&a, s, "d1", C::Main);
    link(
        &a,
        deleted,
        item,
        "gallery",
        OutputSlot::Single,
        Some("media-d1"),
        "ready",
    );
    a.review().classify(deleted, C::Deleted).unwrap();
    photo(&a, s, "u1", C::Main);
    let case = asked(&a, Some(SKU));
    assert_eq!(media_ids(&a, &case), ["media-g1", "media-g2", "media-c1"]);
}

#[test]
fn c3_c8c_both_halves_of_a_split_photo_are_offered_left_then_right() {
    let a = connected();
    let (s, item) = session(&a, SKU);
    let (split, _) = photo(&a, s, "s1", C::Main);
    link(
        &a,
        split,
        item,
        "gallery",
        OutputSlot::Right,
        Some("media-right"),
        "ready",
    );
    link(
        &a,
        split,
        item,
        "gallery",
        OutputSlot::Left,
        Some("media-left"),
        "ready",
    );
    let case = asked(&a, Some(SKU));
    let photos = a.app.conversations.attachable(&case).unwrap().photos;
    let got: Vec<_> = photos
        .iter()
        .map(|p| (p.media_id.as_str(), p.slot, p.photo_id))
        .collect();
    assert_eq!(
        got,
        [
            ("media-left", OutputSlot::Left, split),
            ("media-right", OutputSlot::Right, split),
        ]
    );
}

#[test]
fn c3_c8d_nothing_published_or_not_on_this_mac() {
    let a = connected();
    let (s, _) = session(&a, SKU);
    photo(&a, s, "u1", C::Main);
    let here = asked(&a, Some(SKU));
    let elsewhere = asked(&a, Some("BEYO-9999"));
    let nothing = a.app.conversations.attachable(&here).unwrap();
    assert!(nothing.photos.is_empty());
    assert_eq!(nothing.reason, Some(AttachReason::NothingPublished));
    let away = a.app.conversations.attachable(&elsewhere).unwrap();
    assert!(away.photos.is_empty());
    assert_eq!(away.reason, Some(AttachReason::NotOnThisMac));
}
