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

/// The result of one `tick` of the conversations service: `more` = the next page is due at once.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Tick {
    pub more: bool,
}
