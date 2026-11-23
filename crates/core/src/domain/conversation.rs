//! Conversations with a provider (book 09B §3; `master_plan_conversations.md` §4 Vocabulary): the
//! questions an asker raises about one product and the messages exchanged, as this app keeps them.
//! Platform-neutral (H7): no provider name, address or wire word appears here. All times are UTC
//! milliseconds.

use serde::{Deserialize, Serialize};
use serde_json::Value as Json;

/// The Keychain account that holds the conversations key (through `ports::secrets`).
pub const KEY_ACCOUNT: &str = "conversations-api-key";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub store_id: String,
    pub product_id: String,
    pub sku: Option<String>,
    pub product_title: String,
    pub topic: String,
    pub blocking: bool,
    pub status: ConversationStatus,
    pub resolved_by: Option<ResolvedBy>,
    pub created_at: i64,
    pub updated_at: i64,
    pub read_mark: Option<i64>,
    pub messages: Vec<Message>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConversationStatus {
    Open,
    Answered,
    Resolved,
}

impl ConversationStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Answered => "answered",
            Self::Resolved => "resolved",
        }
    }
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "open" => Some(Self::Open),
            "answered" => Some(Self::Answered),
            "resolved" => Some(Self::Resolved),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResolvedBy {
    Asker,
    Activation,
}

impl ResolvedBy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Asker => "asker",
            Self::Activation => "activation",
        }
    }
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "asker" => Some(Self::Asker),
            "activation" => Some(Self::Activation),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub conversation_id: String,
    pub author: Author,
    pub text: String,
    pub quote: Option<Quote>,
    pub reply_to: Option<ReplyRef>,
    pub attachments: Vec<Attachment>,
    pub created_at: i64,
    pub edited_at: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Author {
    pub kind: AuthorKind,
    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorKind {
    /// The person who asked (the provider's user).
    Asker,
    /// This app's own account.
    Me,
    /// Another account of the provider.
    Other,
}

impl AuthorKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Asker => "asker",
            Self::Me => "me",
            Self::Other => "other",
        }
    }
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "asker" => Some(Self::Asker),
            "me" => Some(Self::Me),
            "other" => Some(Self::Other),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Quote {
    pub field: String,
    pub field_label: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplyRef {
    pub message_id: String,
    pub author_name: String,
    pub excerpt: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Attachment {
    pub media_id: String,
    pub thumb_url: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderStore {
    pub id: String,
    pub name: String,
    pub shop_domain: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderEvent {
    pub id: i64,
    pub kind: EventKind,
    pub conversation_id: Option<String>,
    /// The event concerns a conversation addressed to this app.
    pub for_us: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    ConversationCreated,
    ConversationUpdated {
        deleted: bool,
        resolved_by: Option<ResolvedBy>,
    },
    MessageCreated {
        by_asker: bool,
    },
    MessageUpdated,
    MessageDeleted,
    /// L1 (MC-L4.1): a product's label was printed or reset; never a conversation's.
    Label(crate::domain::label::LabelChange),
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventPage {
    pub events: Vec<ProviderEvent>,
    pub next_cursor: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewMessage {
    pub client_mutation_id: String,
    pub text: String,
    pub reply_to_message_id: Option<String>,
    pub attachment_media_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutboxMessage {
    pub client_mutation_id: String,
    pub conversation_id: String,
    pub text: String,
    pub reply_to: Option<ReplyRef>,
    pub attachment_media_ids: Vec<String>,
    pub state: OutboxState,
    pub failure: Option<String>,
    pub server_message_id: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutboxState {
    Sending,
    Sent,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionState {
    NoKey,
    NoShop,
    StoreUnmatched,
    Ready,
    Refused,
    StoreGone,
}

impl ConnectionState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NoKey => "no_key",
            Self::NoShop => "no_shop",
            Self::StoreUnmatched => "store_unmatched",
            Self::Ready => "ready",
            Self::Refused => "refused",
            Self::StoreGone => "store_gone",
        }
    }
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "no_key" => Some(Self::NoKey),
            "no_shop" => Some(Self::NoShop),
            "store_unmatched" => Some(Self::StoreUnmatched),
            "ready" => Some(Self::Ready),
            "refused" => Some(Self::Refused),
            "store_gone" => Some(Self::StoreGone),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Connection {
    pub state: ConnectionState,
    pub store: Option<ProviderStore>,
    pub visible_stores: Vec<String>,
    pub shop: Option<String>,
    pub last_ok_at: Option<i64>,
    pub last_error: Option<String>,
    pub last_error_at: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConversationNotice {
    pub conversation_id: String,
    pub sku: Option<String>,
    pub product_title: String,
    pub topic: String,
    pub kind: NoticeKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NoticeKind {
    Question,
    Reply,
}

/// The `conversations` settings slot (09B §4). `server` none = the provider's default address.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConversationSettings {
    pub server: Option<String>,
    pub sound: bool,
    pub banner: bool,
}

impl Default for ConversationSettings {
    fn default() -> Self {
        Self {
            server: None,
            sound: true,
            banner: true,
        }
    }
}

impl ConversationSettings {
    /// Tolerant read of the slot: a missing or ill-typed field takes its default.
    pub fn from_slot(slot: Option<&Json>) -> Self {
        let mut settings = Self::default();
        let Some(Json::Object(fields)) = slot else {
            return settings;
        };
        if let Some(server) = fields.get("server").and_then(Json::as_str) {
            let server = server.trim();
            if !server.is_empty() {
                settings.server = Some(server.to_owned());
            }
        }
        if let Some(sound) = fields.get("sound").and_then(Json::as_bool) {
            settings.sound = sound;
        }
        if let Some(banner) = fields.get("banner").and_then(Json::as_bool) {
            settings.banner = banner;
        }
        settings
    }
}

/// What a provider request can end in (MC-C1.8, MC-C2.2). The sentence shown for each comes only
/// from `ConversationsProvider::describe`; `Display` is a neutral code for logs.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ProviderError {
    #[error("unreachable: {0}")]
    Unreachable(String),
    #[error("bad_reply: {0}")]
    BadReply(String),
    #[error("refused")]
    Refused,
    #[error("store_gone")]
    StoreGone,
    #[error("not_found")]
    NotFound,
    #[error("resolved")]
    Resolved,
    #[error("rejected: {code}")]
    Rejected { code: String, message: String },
}

impl ProviderError {
    /// The short kind used in log lines (`conversations.sync_failed kind=`).
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Unreachable(_) => "unreachable",
            Self::BadReply(_) => "bad_reply",
            Self::Refused => "refused",
            Self::StoreGone => "store_gone",
            Self::NotFound => "not_found",
            Self::Resolved => "resolved",
            Self::Rejected { .. } => "rejected",
        }
    }
}

/// Which conversations a list shows (09B §7): all, or those of one session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConversationScope {
    All,
    Session(i64),
}

/// The panel's tabs (09B §7): Open = unresolved; Resolved = resolved in the last 30 days.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConversationTab {
    Open,
    Resolved,
}

/// One row of the panel's list (09B §7; A30: one row per **product**, its questions inside).
/// `id` = the question the row opens (the newest unresolved one, else the newest); `topic` is
/// that question's. `status` = open when any question is open, else answered when any is
/// answered, else resolved; `blocking` = any unresolved question blocks (any question when all
/// are resolved); `last_line`/`last_activity_at` = the newest message of all its questions;
/// `has_news` = any question has news; `questions` = how many questions the product has.
/// `session_id` none = "Not on this Mac".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConversationRow {
    pub id: String,
    pub product_id: String,
    pub sku: Option<String>,
    pub product_title: String,
    pub topic: String,
    pub status: ConversationStatus,
    pub blocking: bool,
    /// Blocking and not resolved ("Pricing is waiting").
    pub waiting: bool,
    /// The first line of the newest message.
    pub last_line: Option<String>,
    pub last_activity_at: i64,
    pub has_news: bool,
    pub session_id: Option<i64>,
    pub questions: i64,
}

/// A30 (09B §7): a product's questions together, as the thread shows them: every question of
/// the product in the copy, oldest first (by creation, then id). `session_id` none = "Not on
/// this Mac".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConversationThread {
    pub product_id: String,
    pub sku: Option<String>,
    pub product_title: String,
    pub session_id: Option<i64>,
    pub questions: Vec<ConversationView>,
}

/// The panel's list (09B §7): the connection state, its problem sentence when refused or gone,
/// the rows (one per product, A30), and the products with news outside the scope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConversationList {
    pub state: ConnectionState,
    pub problem: Option<String>,
    pub rows: Vec<ConversationRow>,
    pub news_elsewhere: i64,
}

/// The header count (09B §6; A30): products with news in the scope and outside it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ConversationCounts {
    pub here: i64,
    pub elsewhere: i64,
}

/// The sessions-list badge (09B §6): "N open", or "Pricing is waiting" when one is blocking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionBadge {
    pub session_id: i64,
    pub open: i64,
    pub waiting: bool,
}

/// The failure of a reply whose request may or may not have reached the provider (MC-C2.2).
pub const COULD_NOT_SEND: &str = "Couldn't send";

/// A photo attached to a reply not yet confirmed: its media id and, when this Mac has the photo,
/// its preview (MC-C2.6).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PendingAttachment {
    pub media_id: String,
    pub preview_path: Option<String>,
}

/// A reply of the outbox as the conversation shows it (MC-C2.6): `sending` or `failed` (a `sent`
/// row is shown through its message, but `reply` and `retry` may still answer one).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PendingView {
    pub client_mutation_id: String,
    pub text: String,
    pub reply_to: Option<ReplyRef>,
    pub attachments: Vec<PendingAttachment>,
    pub state: OutboxState,
    pub failure: Option<String>,
    pub created_at: i64,
}

/// One conversation as the view shows it (09B §7, MC-C2.6): the copy's messages in order, then
/// the replies not yet confirmed. `session_id` none = "Not on this Mac"; `can_send` false with
/// `off_reason` when the key is refused or the store is gone.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConversationView {
    pub id: String,
    pub sku: Option<String>,
    pub product_title: String,
    pub topic: String,
    pub blocking: bool,
    pub status: ConversationStatus,
    pub resolved_by: Option<ResolvedBy>,
    pub session_id: Option<i64>,
    pub messages: Vec<Message>,
    pub pending: Vec<PendingView>,
    pub can_send: bool,
    pub off_reason: Option<String>,
    /// C4 (09B §7 quote chip jump; projection L4): message id → the form field its quote names,
    /// only for messages whose quote names a field the form of this Mac's session has. Computed by
    /// `view`, never stored (the persisted `Quote` is unchanged).
    #[serde(default)]
    pub jump_keys: std::collections::BTreeMap<String, String>,
}

/// The two kinds of published photo a reply may carry (09B §8).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttachRole {
    Gallery,
    Condition,
}

/// A published photo of the conversation's product that a reply may attach (09B §8).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttachablePhoto {
    pub media_id: String,
    pub photo_id: i64,
    pub slot: crate::domain::composition::OutputSlot,
    pub role: AttachRole,
    pub preview_path: Option<String>,
}

/// Why a conversation offers no photo (09B §8, CV-7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttachReason {
    NotOnThisMac,
    NothingPublished,
}

/// The photos a reply may attach, in gallery order then condition order, or the reason none is.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Attachable {
    pub photos: Vec<AttachablePhoto>,
    pub reason: Option<AttachReason>,
}

/// The result of one `tick` of the conversations service: `more` = the next page is due at once.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Tick {
    pub more: bool,
}
