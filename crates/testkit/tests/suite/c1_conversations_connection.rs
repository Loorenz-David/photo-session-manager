//! C1 (`docs/implementation/plans/C1_connection_and_sync.md`): the adapter over the fake Pricing
//! server (C1·C6(b)), and Settings → Pricing: the settings slot, the key in the Keychain, the
//! connection states and the store match (U2, C1·C7–C10). Every test runs the real
//! `BeyoPricingAdapter` over `FakePricing`; nothing calls Pricing or Shopify.

use psm_core::ports::conversations::ConversationsProvider;
use psm_integrations::conversations::beyo_pricing::BeyoPricingAdapter;
use psm_testkit::fake_pricing::FakePricing;
use std::{
    io::Write,
    sync::{Arc, Mutex},
};

#[derive(Clone)]
struct LogBytes(Arc<Mutex<Vec<u8>>>);
impl Write for LogBytes {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// Runs `f` with every log line it writes collected **raw** (no redacting writer, so a key that
/// reached a log call would show); returns its result and the log text.
fn logged_raw<T>(f: impl FnOnce() -> T) -> (T, String) {
    let buffer = Arc::new(Mutex::new(Vec::new()));
    let output = LogBytes(buffer.clone());
    psm_testkit::logs::keep_log_sites_interested();
    let subscriber = tracing_subscriber::fmt()
        .with_writer(move || output.clone())
        .with_max_level(tracing::Level::TRACE)
        .without_time()
        .with_ansi(false)
        .finish();
    let value = tracing::subscriber::with_default(subscriber, f);
    let text = String::from_utf8(buffer.lock().unwrap().clone()).unwrap();
    (value, text)
}

#[test]
fn c1_c6b_the_key_never_reaches_an_error_a_log_or_the_request_record() {
    const KEY: &str = "secret-key-123";
    let fake = Arc::new(FakePricing::new());
    let adapter = BeyoPricingAdapter::new(
        fake.clone(),
        || "https://pricing.example".into(),
        || Some(KEY.into()),
    );
    let (errors, log) = logged_raw(|| {
        // 401: the fake does not accept this key.
        let refused = adapter.stores().unwrap_err();
        fake.refuse_connection_next(1);
        let unreachable = adapter.stores().unwrap_err();
        fake.fail_matching("/stores", 1, 500);
        let server_error = adapter.stores().unwrap_err();
        vec![refused, unreachable, server_error]
    });
    assert_eq!(errors.len(), 3);
    assert_eq!(
        errors.iter().map(|e| e.kind()).collect::<Vec<_>>(),
        ["refused", "unreachable", "unreachable"]
    );
    for error in &errors {
        assert!(!error.to_string().contains(KEY), "{error}");
        assert!(!format!("{error:?}").contains(KEY), "{error:?}");
        let sentence = adapter.describe(error, Some("shop-a.myshopify.com"));
        assert!(!sentence.contains(KEY), "{sentence}");
    }
    assert!(!log.contains(KEY), "{log}");
    let requests = fake.requests();
    assert_eq!(requests.len(), 3);
    for request in &requests {
        assert!(!format!("{request:?}").contains(KEY), "{request:?}");
        assert!(!request.authorized);
    }
}

// ---------- U2: settings slot, key, connection states, store match ----------

use psm_core::{
    domain::conversation::{
        Author, AuthorKind, ConnectionState, Conversation, ConversationSettings,
        ConversationStatus, KEY_ACCOUNT, Message, OutboxMessage, OutboxState, ProviderError,
    },
    ports::{clock::Clock, conversations::ConversationStore, secrets::SecretStore},
    services::conversations::{ConnectError, ConversationSettingsPatch},
};
use psm_integrations::PreparedSettings;
use psm_testkit::{
    TestApp,
    test_app::{PRICING_KEY, PRICING_STORE_NAME},
};
use serde_json::json;

fn pricing_app() -> TestApp {
    TestApp::with_pricing().unwrap()
}

fn settings_bytes(a: &TestApp) -> Vec<u8> {
    std::fs::read(a.app.paths.settings()).unwrap()
}

fn slot(a: &TestApp) -> Option<serde_json::Value> {
    psm_infra::settings_file::Settings::open(&a.app.paths)
        .unwrap()
        .conversations
}

fn patch(server: Option<&str>, sound: Option<bool>) -> ConversationSettingsPatch {
    ConversationSettingsPatch {
        server: server.map(str::to_owned),
        sound,
        banner: None,
    }
}

/// The id of the fake's store whose domain matches the shop.
/// Connects with the default key; returns the saved store's id (requests cleared).
fn matching_store(a: &TestApp) -> String {
    a.app.conversations.key_set(Some(PRICING_KEY)).unwrap();
    let id = a.app.store.sync().unwrap().store.unwrap().id;
    a.fake_pricing().clear_requests();
    id
}

fn conversation(id: &str, store_id: &str) -> Conversation {
    Conversation {
        id: id.into(),
        store_id: store_id.into(),
        product_id: "15531808096409".into(),
        sku: Some("BEYO-1880".into()),
        product_title: "Vintage Brass Wall Mirror, Round".into(),
        topic: "Condition".into(),
        blocking: true,
        status: ConversationStatus::Open,
        resolved_by: None,
        created_at: 1_000,
        updated_at: 1_000,
        read_mark: None,
        messages: vec![Message {
            id: format!("{id}-m1"),
            conversation_id: id.into(),
            author: Author {
                kind: AuthorKind::Asker,
                name: "Elin Lindqvist".into(),
            },
            text: "Is the mirror glass original?".into(),
            quote: None,
            reply_to: None,
            attachments: vec![],
            created_at: 1_000,
            edited_at: None,
        }],
    }
}

fn copy_ids(a: &TestApp) -> Vec<String> {
    a.app
        .store
        .conversations()
        .unwrap()
        .into_iter()
        .map(|c| c.id)
        .collect()
}

#[test]
fn c1_c7a_settings_default_to_the_default_server_with_sound_and_banner() {
    let a = pricing_app();
    assert_eq!(slot(&a), None);
    assert_eq!(
        a.app.conversations.settings(),
        ConversationSettings {
            server: None,
            sound: true,
            banner: true,
        }
    );
}

#[test]
fn c1_c7b_settings_save_merges_and_normalises_the_server() {
    let a = pricing_app();
    a.app
        .conversations
        .settings_save(&patch(Some("https://pricing.example/"), None))
        .unwrap();
    a.app
        .conversations
        .settings_save(&patch(None, Some(false)))
        .unwrap();
    let stored = slot(&a);
    assert_eq!(
        stored,
        Some(json!({"server": "https://pricing.example", "sound": false, "banner": true}))
    );
    assert_eq!(
        ConversationSettings::from_slot(stored.as_ref()),
        ConversationSettings {
            server: Some("https://pricing.example".into()),
            sound: false,
            banner: true,
        }
    );
}

#[test]
fn c1_c7c_an_http_server_is_refused_and_nothing_is_written() {
    let a = pricing_app();
    let before = settings_bytes(&a);
    let error = a
        .app
        .conversations
        .settings_save(&patch(Some("http://pricing.example"), None))
        .unwrap_err();
    assert!(
        matches!(&error, psm_core::Error::InvalidValue(field) if field == "server"),
        "{error:?}"
    );
    assert_eq!(settings_bytes(&a), before);
}

#[test]
fn c1_c7d_a_conversations_patch_does_not_rebuild_the_app() {
    let a = pricing_app();
    let prepared = a
        .app
        .settings_set_context()
        .prepare(&json!({"conversations": {"sound": false}}))
        .unwrap();
    assert!(matches!(prepared, PreparedSettings::Simple(_)));
}

#[test]
fn c1_c8a_the_key_lives_only_in_the_keychain() {
    let a = pricing_app();
    a.app.conversations.key_set(Some("k-secret-9")).unwrap();
    assert_eq!(
        a.secrets.get(KEY_ACCOUNT).unwrap().as_deref(),
        Some("k-secret-9")
    );
    let file = String::from_utf8(settings_bytes(&a)).unwrap();
    assert!(!file.contains("k-secret-9"));
    let shown = serde_json::to_string(&a.app.conversations.connection().unwrap()).unwrap();
    assert!(!shown.contains("k-secret-9"), "{shown}");
}

#[test]
fn c1_c8b_removing_the_key_stops_every_call() {
    let a = pricing_app();
    a.app.conversations.key_set(Some(PRICING_KEY)).unwrap();
    a.app.conversations.key_set(None).unwrap();
    assert_eq!(a.secrets.get(KEY_ACCOUNT).unwrap(), None);
    assert_eq!(
        a.app.conversations.connection().unwrap().state,
        ConnectionState::NoKey
    );
    a.fake_pricing().clear_requests();
    a.app.conversations.tick(a.clock.now_ms()).unwrap();
    assert!(a.fake_pricing().requests().is_empty());
}

#[test]
fn c1_c8c_a_log_line_carrying_the_key_shows_the_redaction() {
    const KEY: &str = "k-secret-c8c-7d41";
    let a = pricing_app();
    a.fake_pricing().accept_key(KEY);
    a.app.conversations.key_set(Some(KEY)).unwrap();
    let buffer = Arc::new(Mutex::new(Vec::new()));
    let output = LogBytes(buffer.clone());
    psm_testkit::logs::keep_log_sites_interested();
    let subscriber = tracing_subscriber::fmt()
        .with_writer(psm_infra::logging::redacting(move || output.clone()))
        .without_time()
        .with_ansi(false)
        .finish();
    tracing::subscriber::with_default(subscriber, || {
        tracing::info!("c1_c8c key={KEY}");
    });
    let text = String::from_utf8(buffer.lock().unwrap().clone()).unwrap();
    assert!(text.contains("c1_c8c key=[REDACTED]"), "{text}");
    assert!(!text.contains(KEY), "{text}");
}

#[test]
fn c1_c9a_test_connection_saves_the_matching_store() {
    let a = pricing_app();
    a.fake_pricing().clear_stores();
    let id = a
        .fake_pricing()
        .add_store(PRICING_STORE_NAME, "Shop-A.myshopify.com");
    a.secrets.set(KEY_ACCOUNT, PRICING_KEY).unwrap();
    let connection = a.app.conversations.test_connection().unwrap();
    assert_eq!(connection.state, ConnectionState::Ready);
    let saved = a.app.store.sync().unwrap().store.unwrap();
    assert_eq!(saved.id, id);
    assert_eq!(saved.name, PRICING_STORE_NAME);
    assert_eq!(saved.shop_domain, "Shop-A.myshopify.com");
    assert_eq!(a.fake_pricing().request_lines(), ["GET /stores"]);
}

#[test]
fn c1_c9b_the_first_matching_store_in_reply_order_is_saved() {
    let a = pricing_app();
    let pricing = a.fake_pricing();
    pricing.clear_stores();
    pricing.add_store("Other store", "other.myshopify.com");
    let second = pricing.add_store("Second", "shop-a.myshopify.com");
    a.secrets.set(KEY_ACCOUNT, PRICING_KEY).unwrap();
    a.app.conversations.test_connection().unwrap();
    let saved = a.app.store.sync().unwrap().store.unwrap();
    assert_eq!(saved.id, second);
    assert_eq!(saved.name, "Second");
}

#[test]
fn c1_c9c_no_matching_store_lists_the_visible_ones_and_stays_off() {
    let a = pricing_app();
    let pricing = a.fake_pricing();
    pricing.clear_stores();
    pricing.add_store("One", "one.myshopify.com");
    pricing.add_store("Two", "two.myshopify.com");
    a.secrets.set(KEY_ACCOUNT, PRICING_KEY).unwrap();
    let connection = a.app.conversations.test_connection().unwrap();
    assert_eq!(connection.state, ConnectionState::StoreUnmatched);
    assert_eq!(connection.visible_stores, ["One", "Two"]);
    assert_eq!(a.app.store.sync().unwrap().store, None);
    pricing.clear_requests();
    a.app.conversations.tick(a.clock.now_ms()).unwrap();
    assert!(pricing.requests().is_empty());
}

#[test]
fn c1_c9d_without_a_shop_nothing_is_asked() {
    let a = TestApp::with_pricing_no_shop().unwrap();
    a.secrets.set(KEY_ACCOUNT, PRICING_KEY).unwrap();
    let connection = a.app.conversations.test_connection().unwrap();
    assert_eq!(connection.state, ConnectionState::NoShop);
    assert!(a.fake_pricing().requests().is_empty());
}

#[test]
fn c1_c9e_without_a_key_nothing_is_asked() {
    let a = pricing_app();
    let connection = a.app.conversations.test_connection().unwrap();
    assert_eq!(connection.state, ConnectionState::NoKey);
    assert!(a.fake_pricing().requests().is_empty());
}

#[test]
fn c1_c9f_a_refused_key_is_refused() {
    let a = pricing_app();
    a.secrets.set(KEY_ACCOUNT, "k-not-accepted").unwrap();
    let connection = a.app.conversations.test_connection().unwrap();
    assert_eq!(connection.state, ConnectionState::Refused);
}

#[test]
fn c1_c9f_a_time_out_keeps_the_ready_state_and_returns_unreachable() {
    let a = pricing_app();
    a.app.conversations.key_set(Some(PRICING_KEY)).unwrap();
    assert_eq!(
        a.app.conversations.connection().unwrap().state,
        ConnectionState::Ready
    );
    a.fake_pricing().timeout_next(1);
    let error = a.app.conversations.test_connection().unwrap_err();
    assert!(
        matches!(error, ConnectError::Provider(ProviderError::Unreachable(_))),
        "{error:?}"
    );
    assert_eq!(
        a.app.conversations.connection().unwrap().state,
        ConnectionState::Ready
    );
}

#[test]
fn c1_c10a_a_changed_shop_unmatches_the_saved_store_after_reopen() {
    let mut a = pricing_app();
    a.app.conversations.key_set(Some(PRICING_KEY)).unwrap();
    assert!(a.app.store.sync().unwrap().store.is_some());
    let mut settings = psm_infra::settings_file::Settings::open(&a.app.paths).unwrap();
    settings.integration.as_mut().unwrap()["shop_domain"] = json!("shop-b.myshopify.com");
    psm_infra::settings_file::Settings::set(&a.app.paths, &settings).unwrap();
    a.kill_and_restart().unwrap();
    let connection = a.app.conversations.connection().unwrap();
    assert_eq!(connection.state, ConnectionState::StoreUnmatched);
    assert!(connection.visible_stores.is_empty());
    a.fake_pricing().clear_requests();
    a.app.conversations.tick(a.clock.now_ms()).unwrap();
    assert!(a.fake_pricing().requests().is_empty());
}

#[test]
fn c1_c10b_a_new_key_for_the_same_store_keeps_the_copy() {
    let a = pricing_app();
    let store_id = matching_store(&a);
    let store = &a.app.store;
    store
        .apply(&conversation("c-1", &store_id), None, None)
        .unwrap();
    store
        .apply(&conversation("c-2", &store_id), None, None)
        .unwrap();
    store.finish_first_sync(40, 1).unwrap();
    store
        .record_error(ConnectionState::Refused, "refused", 2)
        .unwrap();
    assert_eq!(
        a.app.conversations.connection().unwrap().state,
        ConnectionState::Refused
    );
    a.fake_pricing().accept_key("k-other-2");
    let connection = a.app.conversations.key_set(Some("k-other-2")).unwrap();
    assert_eq!(connection.state, ConnectionState::Ready);
    assert_eq!(copy_ids(&a), ["c-1", "c-2"]);
    assert_eq!(store.sync().unwrap().cursor, Some(40));
}

#[test]
fn c1_c10c_a_different_store_empties_the_copy_and_the_outbox() {
    let a = pricing_app();
    let old_id = matching_store(&a);
    let store = &a.app.store;
    store
        .apply(&conversation("c-1", &old_id), None, None)
        .unwrap();
    store
        .apply(&conversation("c-2", &old_id), None, None)
        .unwrap();
    store.finish_first_sync(40, 1).unwrap();
    store
        .outbox_insert(
            &OutboxMessage {
                client_mutation_id: "m-1".into(),
                conversation_id: "c-1".into(),
                text: "Original glass.".into(),
                reply_to: None,
                attachment_media_ids: vec![],
                state: OutboxState::Failed,
                failure: Some("Couldn't send".into()),
                server_message_id: None,
                created_at: 3,
            },
            3,
        )
        .unwrap();
    let pricing = a.fake_pricing();
    pricing.remove_store(&old_id);
    let new_id = pricing.add_store(PRICING_STORE_NAME, "Shop-A.myshopify.com");
    a.app.conversations.test_connection().unwrap();
    assert!(copy_ids(&a).is_empty());
    assert!(store.outbox().unwrap().is_empty());
    let s = store.sync().unwrap();
    assert_eq!(s.cursor, None);
    assert!(!s.first_sync_done);
    assert_eq!(s.store.unwrap().id, new_id);
}

#[test]
fn c1_c10d_a_changed_server_asks_for_test_connection_after_reopen() {
    let mut a = pricing_app();
    let store_id = matching_store(&a);
    a.app.store.finish_first_sync(40, 1).unwrap();
    assert_eq!(a.app.store.sync().unwrap().server, None);
    a.app
        .conversations
        .settings_save(&patch(Some("https://other.example"), None))
        .unwrap();
    a.kill_and_restart().unwrap();
    let connection = a.app.conversations.connection().unwrap();
    assert_eq!(connection.state, ConnectionState::StoreUnmatched);
    assert!(connection.visible_stores.is_empty());
    a.fake_pricing().clear_requests();
    a.app.conversations.tick(a.clock.now_ms()).unwrap();
    assert!(a.fake_pricing().requests().is_empty());
    let s = a.app.store.sync().unwrap();
    assert_eq!(s.cursor, Some(40));
    assert_eq!(s.store.unwrap().id, store_id);
}
