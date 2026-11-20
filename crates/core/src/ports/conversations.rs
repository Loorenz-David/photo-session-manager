//! The conversations provider port (book 09B §3; `master_plan_conversations.md` §4 Ports). An
//! adapter translates the provider's shapes into the core vocabulary (H7); core never sees an
//! address, a wire word or a sentence that names the provider except through `describe`.

use crate::domain::conversation::{
    Conversation, EventPage, Message, NewMessage, ProviderError, ProviderStore,
};

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
