//! The conversations provider port (book 09B §3; `master_plan_conversations.md` §4 Ports). An
//! adapter translates the provider's shapes into the core vocabulary (H7); core never sees an
//! address, a wire word or a sentence that names the provider except through `describe`.

use crate::Result;
use crate::domain::composition::OutputSlot;
use crate::domain::conversation::{
    ConnectionState, Conversation, ConversationSettings, EventPage, Message, NewMessage,
    OutboxMessage, OutboxState, ProviderError, ProviderStore,
};
use crate::domain::photo::Classification;

pub type ProviderResult<T> = std::result::Result<T, ProviderError>;

pub trait ConversationsProvider: Send + Sync {
    /// The stores the key may see, in the provider's order.
    fn stores(&self) -> ProviderResult<Vec<ProviderStore>>;
    /// The store's unresolved conversations addressed to this app, oldest first.
    fn unresolved(&self, store_id: &str) -> ProviderResult<Vec<Conversation>>;
    /// One conversation; `None` = the provider no longer has it, or it is not addressed to this app.
    fn conversation(&self, store_id: &str, id: &str) -> ProviderResult<Option<Conversation>>;
    /// The store's events after `cursor`, oldest first, at most `limit`.
    fn events_after(&self, store_id: &str, cursor: i64, limit: u32) -> ProviderResult<EventPage>;
    /// Writes a reply (MC-C2.2; nothing calls it in C1).
    fn post_message(
        &self,
        store_id: &str,
        conversation_id: &str,
        message: &NewMessage,
    ) -> ProviderResult<Message>;
    /// Marks a conversation answered without a message (MC-C2.7; nothing calls it in C1).
    fn mark_answered(&self, store_id: &str, conversation_id: &str) -> ProviderResult<Conversation>;
    /// The sentence shown for `error` (pure, no request); `shop` fills the store-gone sentence.
    fn describe(&self, error: &ProviderError, shop: Option<&str>) -> String;
}

/// The `conversation_sync` row (`S` of MC-C1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncRecord {
    /// The stored state: only connect and check outcomes are written here (MC-C1.2).
    pub state: ConnectionState,
    pub store: Option<ProviderStore>,
    /// The server of the last successful connect (`None` = the provider's default).
    pub server: Option<String>,
    pub visible_stores: Vec<String>,
    pub cursor: Option<i64>,
    pub first_sync_done: bool,
    pub last_ok_at: Option<i64>,
    pub last_error: Option<String>,
    pub last_error_at: Option<i64>,
}

/// One photo link of a session's item with the linked photo's place (C3 attachable photos).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionLink {
    pub photo_id: i64,
    pub role: String,
    pub slot: OutputSlot,
    pub external_id: Option<String>,
    pub upload_state: String,
    /// The session the photo is in now.
    pub photo_session_id: i64,
    pub classification: Classification,
    pub position: Option<i64>,
    pub captured_at: Option<i64>,
    pub adjusted_preview_path: Option<String>,
    pub original_preview_path: Option<String>,
}

/// The local copy and the sync state (MC-C1). Each method is one transaction.
pub trait ConversationStore: Send + Sync {
    fn sync(&self) -> Result<SyncRecord>;
    /// MC-C1.3 found: when `reset`, the copy and the outbox are emptied, `cursor` null and
    /// `first_sync_done` 0; then the store and `server` are saved, `state = ready`, the error
    /// cleared.
    fn connect_store(&self, store: &ProviderStore, server: Option<&str>, reset: bool)
    -> Result<()>;
    /// MC-C1.3 none matches: `state = store_unmatched` with `visible`, the saved store cleared.
    fn store_unmatched(&self, visible: &[String]) -> Result<()>;
    /// MC-C1.8: `state`, `last_error` and `last_error_at` written together.
    fn record_error(&self, state: ConnectionState, error: &str, at: i64) -> Result<()>;
    /// MC-C1.5(d): `cursor`, `first_sync_done = 1`, `last_ok_at`, the error cleared.
    fn finish_first_sync(&self, cursor: i64, now: i64) -> Result<()>;
    /// `cursor` only (an event that changes nothing in the copy).
    fn set_cursor(&self, cursor: i64) -> Result<()>;
    /// MC-C1.4: inserts `conversation` with `entering_read_mark`, or replaces every field and its
    /// messages keeping the stored read mark; `cursor` (when given) in the same transaction.
    /// `true` when the copy changed.
    fn apply(
        &self,
        conversation: &Conversation,
        entering_read_mark: Option<i64>,
        cursor: Option<i64>,
    ) -> Result<bool>;
    /// MC-C1.6: removes the conversation, its messages and its outbox rows not `sent`; `cursor`
    /// (when given) in the same transaction. `true` when the copy changed.
    fn delete_conversation(&self, id: &str, cursor: Option<i64>) -> Result<bool>;
    /// MC-C1.9: a check completed: `last_ok_at = now`, the error cleared.
    fn complete_check(&self, now: i64) -> Result<()>;
    /// MC-C1.10: the read mark moves to the newest message; `true` when it changed.
    fn mark_read(&self, id: &str) -> Result<bool>;
    /// One conversation of the copy.
    fn conversation(&self, id: &str) -> Result<Option<Conversation>>;
    /// The copy, with messages in order.
    fn conversations(&self) -> Result<Vec<Conversation>>;
    /// Adds an outbox row.
    fn outbox_insert(&self, message: &OutboxMessage, now: i64) -> Result<()>;
    /// Every outbox row, oldest first.
    fn outbox(&self) -> Result<Vec<OutboxMessage>>;
    /// MC-C2.1 Accept: the row with `message.client_mutation_id` when one exists (nothing is
    /// written), otherwise `message` inserted; `true` when it was inserted.
    fn outbox_accept(&self, message: &OutboxMessage, now: i64) -> Result<(OutboxMessage, bool)>;
    /// One outbox row.
    fn outbox_row(&self, client_mutation_id: &str) -> Result<Option<OutboxMessage>>;
    /// MC-C2.2 `Ok(m)`: the row `sent` with `m.id`; `m` appended to its conversation unless a
    /// message with that id is there; the conversation `answered`, `resolved_by` null,
    /// `last_activity_at = m.created_at`.
    fn outbox_sent(&self, client_mutation_id: &str, message: &Message, now: i64) -> Result<()>;
    /// MC-C2.2, .3: each row named that is in state `from` moves to `to` with `failure`; the
    /// number of rows changed.
    fn outbox_move(
        &self,
        client_mutation_ids: &[String],
        from: OutboxState,
        to: OutboxState,
        failure: Option<&str>,
        now: i64,
    ) -> Result<usize>;
    /// MC-C2.3 Discard: the row is deleted when it is `failed`; `true` when it was.
    fn outbox_discard(&self, client_mutation_id: &str) -> Result<bool>;
    /// C3 (projection L2): the photo links of the item of `session_id`, each with its photo's
    /// session, classification, position, captured time and previews. Unfiltered.
    fn session_links(&self, session_id: i64) -> Result<Vec<SessionLink>>;
    /// Each item's SKU with the session of that item (the item's latest session, as
    /// `Store::session_for_item`); items without a session are left out (09B §3).
    fn item_sessions(&self) -> Result<Vec<(String, i64)>>;
}

/// The `conversations` slot of the settings file (master plan §4 Settings): `read` hands back the
/// stored slot with an invalid `server` as none; `write` validates and stores the full object.
pub trait ConversationSettingsSlot: Send + Sync {
    fn read(&self) -> Option<serde_json::Value>;
    fn write(&self, settings: ConversationSettings) -> Result<()>;
}
