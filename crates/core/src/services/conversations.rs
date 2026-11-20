//! `ConversationsService` (book 09B §4–§6; `master_plan_conversations.md` §4 Service, §5 MC-C1):
//! the connection to a conversations provider, the key held in memory, the store match, and the
//! local copy kept up to date by `tick`. Platform-neutral (H7): every sentence comes from
//! `provider.describe`, every setting through the slot port, the key through `SecretStore`.

use crate::{
    Error, Result,
    domain::conversation::{
        Connection, ConnectionState, ConversationSettings, KEY_ACCOUNT, ProviderError, Tick,
    },
    ports::{
        clock::Clock,
        conversations::{
            ConversationSettingsSlot, ConversationStore, ConversationsProvider, SyncRecord,
        },
        events::EventSink,
        secrets::SecretStore,
    },
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex, MutexGuard};

/// The process-wide lane (MC-C1 notation): held for one provider request and the transaction that
/// applies its answer, so requests and applies never interleave, not even across an old and a new
/// service during an App rebuild.
#[derive(Default)]
pub struct ConversationsLane(Mutex<()>);

impl ConversationsLane {
    pub fn new() -> Self {
        Self::default()
    }
    fn enter(&self) -> MutexGuard<'_, ()> {
        self.0.lock().unwrap_or_else(|e| e.into_inner())
    }
}

/// What `test_connection` and `key_set` can fail with: the provider's `Unreachable`/`BadReply` of
/// `stores()` (the state is unchanged), or a local storage error.
#[derive(Debug, thiserror::Error)]
pub enum ConnectError {
    #[error("{0}")]
    Provider(ProviderError),
    #[error("{0}")]
    Storage(#[from] Error),
}

/// A `conversations_settings_save` patch: the fields given replace those stored. `server: ""`
/// returns to the provider's default.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ConversationSettingsPatch {
    pub server: Option<String>,
    pub sound: Option<bool>,
    pub banner: Option<bool>,
}

#[derive(Default)]
enum HeldKey {
    #[default]
    Unread,
    Read(Option<String>),
}

#[derive(Default)]
struct Memory {
    key: HeldKey,
}

pub struct ConversationsService {
    store: Arc<dyn ConversationStore>,
    provider: Arc<dyn ConversationsProvider>,
    secrets: Arc<dyn SecretStore>,
    slot: Arc<dyn ConversationSettingsSlot>,
    shop: Option<String>,
    clock: Arc<dyn Clock>,
    #[allow(dead_code)]
    events: Arc<dyn EventSink>,
    lane: Arc<ConversationsLane>,
    memory: Mutex<Memory>,
}

/// `≈` of MC-C1: equal after trimming, ignoring ASCII case.
fn same_domain(a: &str, b: &str) -> bool {
    a.trim().eq_ignore_ascii_case(b.trim())
}

impl ConversationsService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        store: Arc<dyn ConversationStore>,
        provider: Arc<dyn ConversationsProvider>,
        secrets: Arc<dyn SecretStore>,
        slot: Arc<dyn ConversationSettingsSlot>,
        shop: Option<String>,
        clock: Arc<dyn Clock>,
        events: Arc<dyn EventSink>,
        lane: Arc<ConversationsLane>,
    ) -> Self {
        Self {
            store,
            provider,
            secrets,
            slot,
            shop: shop.filter(|s| !s.trim().is_empty()),
            clock,
            events,
            lane,
            memory: Mutex::new(Memory::default()),
        }
    }

    fn memory(&self) -> MutexGuard<'_, Memory> {
        self.memory.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// The key held in memory, without reading the Keychain (the adapter's `key` closure).
    pub fn held_key(&self) -> Option<String> {
        match &self.memory().key {
            HeldKey::Read(key) => key.clone(),
            HeldKey::Unread => None,
        }
    }

    /// The key: read from the Keychain once per build, then held in memory (09B §10 rule 5).
    fn key(&self) -> Option<String> {
        if let HeldKey::Read(key) = &self.memory().key {
            return key.clone();
        }
        let read = match self.secrets.get(KEY_ACCOUNT) {
            Ok(value) => value.filter(|k| !k.trim().is_empty()),
            Err(error) => {
                tracing::warn!(target: "psm::conversations", %error, "conversations key not readable");
                None
            }
        };
        let mut memory = self.memory();
        if let HeldKey::Unread = memory.key {
            memory.key = HeldKey::Read(read);
        }
        match &memory.key {
            HeldKey::Read(key) => key.clone(),
            HeldKey::Unread => None,
        }
    }

    /// The settings as stored (live, through the slot).
    pub fn settings(&self) -> ConversationSettings {
        ConversationSettings::from_slot(self.slot.read().as_ref())
    }

    /// MC-C1.2 and the server rule of MC-C1.3: the state reported for `s`.
    fn derive(
        &self,
        s: &SyncRecord,
        key_held: bool,
        settings: &ConversationSettings,
    ) -> (ConnectionState, Vec<String>) {
        let Some(shop) = &self.shop else {
            return (ConnectionState::NoShop, vec![]);
        };
        if !key_held {
            return (ConnectionState::NoKey, vec![]);
        }
        match &s.store {
            None if s.state == ConnectionState::StoreUnmatched => {
                (ConnectionState::StoreUnmatched, s.visible_stores.clone())
            }
            // A key refused before any store was saved (MC-C1.3 `Refused → refused`).
            None if s.state == ConnectionState::Refused => (ConnectionState::Refused, vec![]),
            None => (ConnectionState::StoreUnmatched, vec![]),
            Some(store) if !same_domain(&store.shop_domain, shop) => {
                (ConnectionState::StoreUnmatched, vec![])
            }
            Some(_) if settings.server != s.server => (ConnectionState::StoreUnmatched, vec![]),
            Some(_) => match s.state {
                state @ (ConnectionState::Ready
                | ConnectionState::Refused
                | ConnectionState::StoreGone) => (state, vec![]),
                _ => (ConnectionState::StoreUnmatched, vec![]),
            },
        }
    }

    fn connection_from(&self, s: &SyncRecord, settings: &ConversationSettings) -> Connection {
        let (state, visible_stores) = self.derive(s, self.key().is_some(), settings);
        Connection {
            state,
            store: s.store.clone(),
            visible_stores,
            shop: self.shop.clone(),
            last_ok_at: s.last_ok_at,
            last_error: s.last_error.clone(),
            last_error_at: s.last_error_at,
        }
    }

    /// The connection as Settings shows it (never takes the lane; no request).
    pub fn connection(&self) -> Result<Connection> {
        let s = self.store.sync()?;
        Ok(self.connection_from(&s, &self.settings()))
    }

    /// MC-C1.3 Connect: Test connection.
    pub fn test_connection(&self) -> std::result::Result<Connection, ConnectError> {
        self.connect()
    }

    /// MC-C1.3: stores the key (trimmed; empty = remove), holds it in memory, then connects.
    pub fn key_set(&self, key: Option<&str>) -> std::result::Result<Connection, ConnectError> {
        match key.map(str::trim).filter(|k| !k.is_empty()) {
            None => {
                self.secrets.delete(KEY_ACCOUNT)?;
                self.memory().key = HeldKey::Read(None);
                Ok(self.connection()?)
            }
            Some(key) => {
                self.secrets.set(KEY_ACCOUNT, key)?;
                self.memory().key = HeldKey::Read(Some(key.to_owned()));
                self.connect()
            }
        }
    }

    fn connect(&self) -> std::result::Result<Connection, ConnectError> {
        let Some(shop) = self.shop.clone() else {
            return Ok(self.connection()?);
        };
        if self.key().is_none() {
            return Ok(self.connection()?);
        }
        let settings = self.settings();
        let now = self.clock.now_ms();
        {
            let _lane = self.lane.enter();
            match self.provider.stores() {
                Err(ProviderError::Refused) => {
                    let sentence = self.provider.describe(&ProviderError::Refused, Some(&shop));
                    self.store
                        .record_error(ConnectionState::Refused, &sentence, now)?;
                }
                Err(error) => return Err(ConnectError::Provider(error)),
                Ok(stores) => match stores.iter().find(|s| same_domain(&s.shop_domain, &shop)) {
                    None => {
                        let names: Vec<String> = stores.iter().map(|s| s.name.clone()).collect();
                        self.store.store_unmatched(&names)?;
                    }
                    Some(found) => {
                        let saved = self.store.sync()?;
                        let reset = saved.store.as_ref().map(|s| s.id.as_str()) != Some(&found.id);
                        self.store
                            .connect_store(found, settings.server.as_deref(), reset)?;
                    }
                },
            }
        }
        let connection = self.connection()?;
        tracing::info!(
            target: "psm::conversations",
            "conversations.connection state={} store={}",
            connection.state.as_str(),
            if connection.state == ConnectionState::Ready { "yes" } else { "no" }
        );
        Ok(connection)
    }

    /// MC-C1.1: provider calls are allowed only in the `ready` state (with the key held, the shop
    /// matching and the connected server).
    fn calls_allowed(&self) -> Result<Option<SyncRecord>> {
        let s = self.store.sync()?;
        let settings = self.settings();
        let (state, _) = self.derive(&s, self.key().is_some(), &settings);
        Ok((state == ConnectionState::Ready).then_some(s))
    }

    /// One turn of the background loop (MC-C1.1, .5–.9).
    pub fn tick(&self, _now_ms: i64) -> Result<Tick> {
        if self.calls_allowed()?.is_none() {
            return Ok(Tick::default());
        }
        Ok(Tick::default())
    }

    /// Saves the fields given over the stored settings (the slot validates the server).
    pub fn settings_save(&self, patch: &ConversationSettingsPatch) -> Result<ConversationSettings> {
        let mut next = self.settings();
        if let Some(server) = &patch.server {
            let server = server.trim();
            next.server = (!server.is_empty()).then(|| server.to_owned());
        }
        if let Some(sound) = patch.sound {
            next.sound = sound;
        }
        if let Some(banner) = patch.banner {
            next.banner = banner;
        }
        self.slot.write(next)?;
        Ok(self.settings())
    }
}
