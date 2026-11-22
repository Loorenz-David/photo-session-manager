//! `ConversationsService` (book 09B §4–§6; `master_plan_conversations.md` §4 Service, §5 MC-C1):
//! the connection to a conversations provider, the key held in memory, the store match, and the
//! local copy kept up to date by `tick`. Platform-neutral (H7): every sentence comes from
//! `provider.describe`, every setting through the slot port, the key through `SecretStore`.

use crate::{
    Error, Result,
    domain::{
        composition::OutputSlot,
        conversation::{
            AttachReason, AttachRole, Attachable, AttachablePhoto, COULD_NOT_SEND, Connection,
            ConnectionState, Conversation, ConversationCounts, ConversationList,
            ConversationNotice, ConversationRow, ConversationScope, ConversationSettings,
            ConversationStatus, ConversationTab, ConversationThread, ConversationView, EventKind,
            EventPage, KEY_ACCOUNT, Message, NewMessage, NoticeKind, OutboxMessage, OutboxState,
            PendingAttachment, PendingView, ProviderError, ReplyRef, ResolvedBy, SessionBadge,
            Tick,
        },
        photo::Classification,
    },
    events::AppEvent,
    mechanisms::conversation_rules::{
        check_backoff, checked_reply, entering_read_mark, first_line, has_news, list_order,
        reply_excerpt, resolved_recent,
    },
    ports::{
        clock::Clock,
        conversations::{
            ConversationSettingsSlot, ConversationStore, ConversationsProvider, SessionLink,
            SyncRecord,
        },
        events::EventSink,
        secrets::SecretStore,
    },
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    sync::{Arc, Mutex, MutexGuard},
};

/// Events asked per page (the package's maximum).
const PAGE: u32 = 500;
/// MC-C1.7: the wait after a complete check.
const CHECK_EVERY_MS: i64 = 10_000;
/// The package's maximum number of cases in one list (a first connection logs `truncated`).
const LIST_MAX: usize = 200;

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

/// What the C3 methods (`view`, `reply`, `retry`, `discard`, `mark_answered`, `attachable`) can
/// fail with (MC-C2.1, .7; projection L5, L6): `Off` = the state does not allow it (`message` =
/// the state's sentence when the key is refused or the store is gone, empty in the hidden
/// states); `Resolved` = the provider had already resolved the conversation; `Provider` = any
/// other provider error of Mark answered; `Storage` = a local error (`invalid_value`,
/// `not_found`, …).
#[derive(Debug, thiserror::Error)]
pub enum ConversationError {
    #[error("{message}")]
    Off {
        state: ConnectionState,
        message: String,
    },
    #[error("resolved")]
    Resolved,
    #[error("{0}")]
    Provider(ProviderError),
    #[error("{0}")]
    Storage(#[from] Error),
}

/// The states in which the copy is hidden (09B §4): every C3 method answers `Off`.
fn hidden(state: ConnectionState) -> bool {
    matches!(
        state,
        ConnectionState::NoKey | ConnectionState::NoShop | ConnectionState::StoreUnmatched
    )
}

/// The preview of a linked photo (the session-cover precedent): adjusted, else original.
fn link_preview(link: &SessionLink) -> Option<String> {
    link.adjusted_preview_path
        .clone()
        .or_else(|| link.original_preview_path.clone())
}

/// The role of a link a reply may attach (publish writes only `gallery` and
/// `attribute:<condition key>`).
fn attach_role(role: &str) -> Option<AttachRole> {
    if role == "gallery" {
        Some(AttachRole::Gallery)
    } else if role.starts_with("attribute:") {
        Some(AttachRole::Condition)
    } else {
        None
    }
}

fn slot_rank(slot: OutputSlot) -> u8 {
    match slot {
        OutputSlot::Single => 0,
        OutputSlot::Left => 1,
        OutputSlot::Right => 2,
    }
}

/// 09B §8: the published photos of `session_id` a reply may attach: a link with an external id,
/// upload state `ready`, a gallery or condition role, and a photo still in that session and not
/// deleted; gallery first, then condition, each by the photo's (position, captured time, id)
/// with a missing position or time last, then `left` before `right`.
fn attachable_photos(session_id: i64, links: &[SessionLink]) -> Vec<AttachablePhoto> {
    let mut offered: Vec<(&SessionLink, AttachRole, String)> = links
        .iter()
        .filter(|l| l.upload_state == "ready")
        .filter(|l| l.photo_session_id == session_id && l.classification != Classification::Deleted)
        .filter_map(|l| {
            let role = attach_role(&l.role)?;
            let media = l.external_id.clone().filter(|m| !m.is_empty())?;
            Some((l, role, media))
        })
        .collect();
    offered.sort_by_key(|(l, role, _)| {
        (
            *role == AttachRole::Condition,
            l.position.is_none(),
            l.position,
            l.captured_at.is_none(),
            l.captured_at,
            l.photo_id,
            slot_rank(l.slot),
        )
    });
    offered
        .into_iter()
        .map(|(l, role, media_id)| AttachablePhoto {
            media_id,
            photo_id: l.photo_id,
            slot: l.slot,
            role,
            preview_path: link_preview(l),
        })
        .collect()
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

/// A conversation a running check found news in (MC-C1.6 Notices).
struct Mark {
    conversation_id: String,
    kind: NoticeKind,
    event_id: i64,
}

/// What the service keeps in memory (MC-C1.7: `next_check_at`, `failures`; the running check's
/// marks and whether it changed the copy).
#[derive(Default)]
struct Memory {
    key: HeldKey,
    /// Wall clock; 0 at build = "now".
    next_check_at: i64,
    failures: u32,
    more: bool,
    marks: Vec<Mark>,
    applied: bool,
    events: usize,
}

impl Memory {
    fn fresh_check(&mut self, next_check_at: i64) {
        self.next_check_at = next_check_at;
        self.failures = 0;
        self.more = false;
        self.marks.clear();
        self.applied = false;
        self.events = 0;
    }
}

/// Why a check stopped early.
enum Stop {
    Provider(ProviderError),
    Storage(Error),
    /// The connection or the cursor changed under the check (another connect or service).
    Changed,
}

impl From<ProviderError> for Stop {
    fn from(error: ProviderError) -> Self {
        Self::Provider(error)
    }
}

impl From<Error> for Stop {
    fn from(error: Error) -> Self {
        Self::Storage(error)
    }
}

pub struct ConversationsService {
    store: Arc<dyn ConversationStore>,
    provider: Arc<dyn ConversationsProvider>,
    secrets: Arc<dyn SecretStore>,
    slot: Arc<dyn ConversationSettingsSlot>,
    shop: Option<String>,
    /// C4: the connected integration's kind (the form's integration definitions depend on it).
    integration: Option<String>,
    clock: Arc<dyn Clock>,
    events: Arc<dyn EventSink>,
    lane: Arc<ConversationsLane>,
    memory: Mutex<Memory>,
}

/// A conversation of the copy with its session (none = "Not on this Mac").
type Placed = (Conversation, Option<i64>);

/// A30: the product a conversation belongs to: its product id, else its SKU, else itself (a
/// question without either is a product of its own).
fn product_key(c: &Conversation) -> String {
    if !c.product_id.trim().is_empty() {
        return format!("p:{}", c.product_id);
    }
    match &c.sku {
        Some(sku) if !sku.trim().is_empty() => format!("s:{sku}"),
        _ => format!("c:{}", c.id),
    }
}

/// A30: the copy grouped by product, each group's questions oldest first (creation, then id),
/// the groups in the order their first question appears.
fn by_product(copy: Vec<Placed>) -> Vec<Vec<Placed>> {
    let mut groups: Vec<(String, Vec<Placed>)> = Vec::new();
    for placed in copy {
        let key = product_key(&placed.0);
        match groups.iter_mut().find(|(k, _)| *k == key) {
            Some((_, group)) => group.push(placed),
            None => groups.push((key, vec![placed])),
        }
    }
    groups
        .into_iter()
        .map(|(_, mut group)| {
            group.sort_by(|(a, _), (b, _)| a.created_at.cmp(&b.created_at).then(a.id.cmp(&b.id)));
            group
        })
        .collect()
}

/// A30: a product group is unresolved when any of its questions is.
fn unresolved(group: &[Placed]) -> bool {
    group
        .iter()
        .any(|(c, _)| c.status != ConversationStatus::Resolved)
}

/// One list row of a product group (09B §7, A30; see [`ConversationRow`]): `waiting` = an
/// unresolved question blocks; `last_line` = the first line of the newest message of all its
/// questions; `last_activity_at` = that message's time, or the newest `updated_at`.
fn row_of(group: &[Placed]) -> ConversationRow {
    let open: Vec<&Conversation> = group
        .iter()
        .map(|(c, _)| c)
        .filter(|c| c.status != ConversationStatus::Resolved)
        .collect();
    let shown = open
        .last()
        .copied()
        .unwrap_or_else(|| &group[group.len() - 1].0);
    let status = if open.iter().any(|c| c.status == ConversationStatus::Open) {
        ConversationStatus::Open
    } else if open.is_empty() {
        ConversationStatus::Resolved
    } else {
        ConversationStatus::Answered
    };
    let waiting = open.iter().any(|c| c.blocking);
    let blocking = if open.is_empty() {
        group.iter().any(|(c, _)| c.blocking)
    } else {
        waiting
    };
    let newest = group
        .iter()
        .flat_map(|(c, _)| c.messages.iter())
        .max_by_key(|m| m.created_at);
    let updated = group.iter().map(|(c, _)| c.updated_at).max().unwrap_or(0);
    ConversationRow {
        id: shown.id.clone(),
        product_id: shown.product_id.clone(),
        sku: shown.sku.clone(),
        product_title: shown.product_title.clone(),
        topic: shown.topic.clone(),
        status,
        blocking,
        waiting,
        last_line: newest
            .map(|m| first_line(&m.text))
            .filter(|l| !l.is_empty()),
        last_activity_at: newest.map_or(updated, |m| m.created_at),
        has_news: group.iter().any(|(c, _)| has_news(c)),
        session_id: group.iter().find_map(|(_, s)| *s),
        questions: group.len() as i64,
    }
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
        integration: Option<String>,
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
            integration,
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

    /// Whether a key is saved (reads the Keychain at most once per build).
    pub fn key_saved(&self) -> bool {
        self.key().is_some()
    }

    /// The sentence shown for a provider error (the provider's words, the connected shop).
    pub fn describe(&self, error: &ProviderError) -> String {
        self.provider.describe(error, self.shop.as_deref())
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
        let before = self.connection()?.state;
        let after = self.connect()?;
        self.emit_if_changed(before, after.state);
        Ok(after)
    }

    /// MC-C1.3: stores the key (trimmed; empty = remove), holds it in memory, then connects.
    pub fn key_set(&self, key: Option<&str>) -> std::result::Result<Connection, ConnectError> {
        let before = self.connection()?.state;
        let after = match key.map(str::trim).filter(|k| !k.is_empty()) {
            None => {
                self.secrets.delete(KEY_ACCOUNT)?;
                self.memory().key = HeldKey::Read(None);
                self.connection()?
            }
            Some(key) => {
                self.secrets.set(KEY_ACCOUNT, key)?;
                {
                    let mut memory = self.memory();
                    memory.key = HeldKey::Read(Some(key.to_owned()));
                    memory.next_check_at = self.clock.now_ms();
                }
                self.connect()?
            }
        };
        self.emit_if_changed(before, after.state);
        Ok(after)
    }

    fn emit(&self, notices: Vec<ConversationNotice>) {
        let settings = self.settings();
        let any = !notices.is_empty();
        self.events.emit(AppEvent::ConversationsChanged {
            notices,
            chime: settings.sound && any,
            banner: settings.banner && any,
        });
    }

    /// MC-C1 §4 Event: emitted without notices when the connection state changes.
    fn emit_if_changed(&self, before: ConnectionState, after: ConnectionState) {
        if before != after {
            self.emit(vec![]);
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
                        // MC-C1.7: the next `tick` checks at once.
                        self.memory().fresh_check(now);
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

    /// One turn of the background loop (MC-C1.1, .5–.9): a check runs iff calls are allowed and
    /// the next page is due at once or `next_check_at` has passed. `more` = the next page is due.
    pub fn tick(&self, now_ms: i64) -> Result<Tick> {
        let Some(s) = self.calls_allowed()? else {
            return Ok(Tick::default());
        };
        // MC-C2.2: the outbox is flushed before the due test (projection L15), so a reply leaves
        // within a quarter of a second even when no check is due.
        if let Some(store) = &s.store {
            self.flush(&store.id, now_ms)?;
        }
        {
            let memory = self.memory();
            if !(memory.more || now_ms >= memory.next_check_at) {
                return Ok(Tick::default());
            }
        }
        let Some(store) = s.store else {
            return Ok(Tick::default());
        };
        let outcome = if s.first_sync_done {
            self.pull(&store.id, now_ms)
        } else {
            self.first_connection(&store.id, now_ms)
                .map(|()| Tick::default())
        };
        match outcome {
            Ok(tick) => Ok(tick),
            Err(Stop::Provider(error)) => {
                self.check_failed(error, now_ms)?;
                Ok(Tick::default())
            }
            Err(Stop::Storage(error)) => Err(error),
            Err(Stop::Changed) => {
                self.memory().more = false;
                Ok(Tick::default())
            }
        }
    }

    /// One provider request (or a transaction) and its apply inside the lane, after checking that
    /// calls are still allowed for `store_id` and, when given, that the cursor is still `cursor`
    /// (MC-C1 lane: the cursor of each request is read from `S` inside the lane).
    fn in_lane<T>(
        &self,
        store_id: &str,
        cursor: Option<Option<i64>>,
        work: impl FnOnce(&SyncRecord) -> std::result::Result<T, Stop>,
    ) -> std::result::Result<T, Stop> {
        let _lane = self.lane.enter();
        let s = self.calls_allowed()?.ok_or(Stop::Changed)?;
        if s.store.as_ref().map(|st| st.id.as_str()) != Some(store_id) {
            return Err(Stop::Changed);
        }
        if let Some(expected) = cursor
            && s.cursor != expected
        {
            return Err(Stop::Changed);
        }
        work(&s)
    }

    /// MC-C1.5: the first connection, silent.
    fn first_connection(&self, store_id: &str, now: i64) -> std::result::Result<(), Stop> {
        // (a) page the log to its end, remembering the conversations it names.
        let mut after = 0;
        let mut named: Vec<String> = Vec::new();
        let mut last_deleted: HashMap<String, bool> = HashMap::new();
        loop {
            let page = self.in_lane(store_id, None, |s| {
                if s.first_sync_done {
                    return Err(Stop::Changed);
                }
                Ok(self.provider.events_after(store_id, after, PAGE)?)
            })?;
            if page.events.is_empty() {
                break;
            }
            if page.next_cursor <= after {
                return Err(Stop::Provider(ProviderError::BadReply(
                    "the event cursor did not advance".into(),
                )));
            }
            for event in &page.events {
                if let (true, Some(id)) = (event.for_us, &event.conversation_id) {
                    if !named.contains(id) {
                        named.push(id.clone());
                    }
                    let deleted = matches!(
                        event.kind,
                        EventKind::ConversationUpdated { deleted: true, .. }
                    );
                    last_deleted.insert(id.clone(), deleted);
                }
            }
            after = page.next_cursor;
        }
        // (b) the unresolved conversations.
        let listed = self.in_lane(store_id, None, |_| {
            let list = self.provider.unresolved(store_id)?;
            for conversation in &list {
                self.store
                    .apply(conversation, entering_read_mark(conversation), None)?;
            }
            Ok(list.into_iter().map(|c| c.id).collect::<Vec<_>>())
        })?;
        let truncated = listed.len() >= LIST_MAX;
        let listed: HashSet<String> = listed.into_iter().collect();
        // (c) the conversations the log names that the list did not hold; one whose last event
        // removed it is forgotten (and dropped from a copy an earlier attempt left).
        for id in &named {
            if last_deleted.get(id) == Some(&true) {
                self.in_lane(store_id, None, |_| {
                    Ok(self.store.delete_conversation(id, None)?)
                })?;
                continue;
            }
            if listed.contains(id) {
                continue;
            }
            self.in_lane(store_id, None, |_| {
                match self.provider.conversation(store_id, id)? {
                    Some(conversation) => {
                        self.store
                            .apply(&conversation, entering_read_mark(&conversation), None)?;
                    }
                    None => {
                        self.store.delete_conversation(id, None)?;
                    }
                }
                Ok(())
            })?;
        }
        // (d) the cursor; the first connection is a complete check.
        self.in_lane(store_id, None, |_| {
            Ok(self.store.finish_first_sync(after, now)?)
        })?;
        self.memory().fresh_check(now + CHECK_EVERY_MS);
        let count = self.store.conversations()?.len();
        tracing::info!(
            target: "psm::conversations",
            "conversations.first_sync conversations={count} cursor={after} truncated={truncated}"
        );
        self.emit(vec![]);
        Ok(())
    }

    /// MC-C1.6 `resolved_by` of a re-read conversation.
    fn resolved_by(
        conversation: &Conversation,
        page: &EventPage,
        stored: Option<ResolvedBy>,
    ) -> Option<ResolvedBy> {
        if conversation.status != ConversationStatus::Resolved {
            return None;
        }
        let updates: Vec<&Option<ResolvedBy>> = page
            .events
            .iter()
            .filter(|e| e.conversation_id.as_deref() == Some(&conversation.id))
            .filter_map(|e| match &e.kind {
                EventKind::ConversationUpdated { resolved_by, .. } => Some(resolved_by),
                _ => None,
            })
            .collect();
        updates
            .iter()
            .rev()
            .find_map(|r| **r)
            .or(stored)
            .or_else(|| (!updates.is_empty()).then_some(ResolvedBy::Asker))
    }

    /// MC-C1.6 Notices: what one processed event marks.
    fn mark(&self, kind: &EventKind, conversation_id: &str, event_id: i64) {
        let mut memory = self.memory();
        let existing = memory
            .marks
            .iter()
            .position(|m| m.conversation_id == conversation_id);
        match (kind, existing) {
            (EventKind::ConversationCreated, Some(i)) => {
                memory.marks[i].kind = NoticeKind::Question
            }
            (EventKind::ConversationCreated, None) => memory.marks.push(Mark {
                conversation_id: conversation_id.to_owned(),
                kind: NoticeKind::Question,
                event_id,
            }),
            (EventKind::MessageCreated { by_asker: true }, None) => memory.marks.push(Mark {
                conversation_id: conversation_id.to_owned(),
                kind: NoticeKind::Reply,
                event_id,
            }),
            _ => {}
        }
    }

    fn unmark(&self, conversation_id: &str) {
        self.memory()
            .marks
            .retain(|m| m.conversation_id != conversation_id);
    }

    /// MC-C1.6: one page of the pull.
    fn pull(&self, store_id: &str, now: i64) -> std::result::Result<Tick, Stop> {
        let (page, mut cursor) = self.in_lane(store_id, None, |s| {
            let cursor = s.cursor.unwrap_or(0);
            Ok((
                self.provider.events_after(store_id, cursor, PAGE)?,
                s.cursor,
            ))
        })?;
        if page.events.is_empty() {
            self.complete_check(now, cursor.unwrap_or(0))?;
            return Ok(Tick::default());
        }
        self.memory().events += page.events.len();
        for (index, event) in page.events.iter().enumerate() {
            if event.id <= cursor.unwrap_or(0) {
                continue;
            }
            let expected = Some(cursor);
            let target = match (&event.conversation_id, event.for_us) {
                (Some(id), true) => id,
                _ => {
                    self.in_lane(store_id, expected, |_| {
                        Ok(self.store.set_cursor(event.id)?)
                    })?;
                    cursor = Some(event.id);
                    continue;
                }
            };
            if let EventKind::ConversationUpdated { deleted: true, .. } = event.kind {
                let changed = self.in_lane(store_id, expected, |_| {
                    Ok(self.store.delete_conversation(target, Some(event.id))?)
                })?;
                self.memory().applied |= changed;
                self.unmark(target);
                cursor = Some(event.id);
                continue;
            }
            let read_later = page.events[index + 1..]
                .iter()
                .any(|later| later.conversation_id.as_deref() == Some(target.as_str()));
            if read_later {
                self.in_lane(store_id, expected, |_| {
                    Ok(self.store.set_cursor(event.id)?)
                })?;
                self.mark(&event.kind, target, event.id);
                cursor = Some(event.id);
                continue;
            }
            let found = self.in_lane(store_id, expected, |_| {
                match self.provider.conversation(store_id, target)? {
                    Some(mut conversation) => {
                        let stored = self.store.conversation(target)?.and_then(|c| c.resolved_by);
                        conversation.resolved_by = Self::resolved_by(&conversation, &page, stored);
                        let changed = self.store.apply(
                            &conversation,
                            entering_read_mark(&conversation),
                            Some(event.id),
                        )?;
                        Ok((true, changed))
                    }
                    None => Ok((
                        false,
                        self.store.delete_conversation(target, Some(event.id))?,
                    )),
                }
            })?;
            self.memory().applied |= found.1;
            if found.0 {
                self.mark(&event.kind, target, event.id);
            } else {
                self.unmark(target);
            }
            cursor = Some(event.id);
        }
        if page.next_cursor > cursor.unwrap_or(0) {
            let next = page.next_cursor;
            self.in_lane(store_id, Some(cursor), |_| {
                Ok(self.store.set_cursor(next)?)
            })?;
        }
        self.memory().more = true;
        Ok(Tick { more: true })
    }

    /// The marked conversations still in the copy, in the order of the event that first marked
    /// them (MC-C1.9); the marks are cleared.
    fn take_notices(&self) -> Result<Vec<ConversationNotice>> {
        let mut marks = std::mem::take(&mut self.memory().marks);
        marks.sort_by_key(|m| m.event_id);
        let mut notices = Vec::new();
        for mark in marks {
            if let Some(c) = self.store.conversation(&mark.conversation_id)? {
                notices.push(ConversationNotice {
                    conversation_id: c.id,
                    sku: c.sku,
                    product_title: c.product_title,
                    topic: c.topic,
                    kind: mark.kind,
                });
            }
        }
        Ok(notices)
    }

    /// MC-C1.9: a check completed (an empty page).
    fn complete_check(&self, now: i64, cursor: i64) -> Result<()> {
        let had_error = self.store.sync()?.last_error.is_some();
        self.store.complete_check(now)?;
        let (applied, events) = {
            let mut memory = self.memory();
            let applied = memory.applied;
            let events = memory.events;
            memory.next_check_at = now + CHECK_EVERY_MS;
            memory.failures = 0;
            memory.more = false;
            memory.applied = false;
            memory.events = 0;
            (applied, events)
        };
        let notices = self.take_notices()?;
        tracing::info!(
            target: "psm::conversations",
            "conversations.sync cursor={cursor} events={events} applied={applied} notices={}",
            notices.len()
        );
        if applied || had_error || !notices.is_empty() {
            self.emit(notices);
        }
        Ok(())
    }

    /// MC-C1.8: a check or a first connection stopped on `error`.
    fn check_failed(&self, error: ProviderError, now: i64) -> Result<()> {
        let sentence = self.provider.describe(&error, self.shop.as_deref());
        let (state, failures, next_s) = {
            let mut memory = self.memory();
            memory.more = false;
            memory.applied = false;
            memory.events = 0;
            match error {
                ProviderError::Refused => (ConnectionState::Refused, memory.failures, 0),
                ProviderError::StoreGone => (ConnectionState::StoreGone, memory.failures, 0),
                _ => {
                    memory.failures += 1;
                    let wait = check_backoff(memory.failures).as_millis() as i64;
                    memory.next_check_at = now + wait;
                    (ConnectionState::Ready, memory.failures, wait / 1000)
                }
            }
        };
        {
            let _lane = self.lane.enter();
            self.store.record_error(state, &sentence, now)?;
        }
        tracing::info!(
            target: "psm::conversations",
            "conversations.sync_failed kind={} failures={failures} next_s={next_s}",
            error.kind()
        );
        let notices = self.take_notices()?;
        self.emit(notices);
        Ok(())
    }

    /// MC-C1.10: the read mark moves to the newest message (no change → no event).
    pub fn mark_read(&self, conversation_id: &str) -> Result<()> {
        if self.store.mark_read(conversation_id)? {
            self.emit(vec![]);
        }
        Ok(())
    }

    /// The copy as the panel sees it: each conversation with its session (09B §3: the session of
    /// the item whose SKU is exactly the conversation's), or none when the state hides the copy
    /// (`no_key`, `no_shop`, `store_unmatched`). Never takes the lane, never asks the provider.
    fn visible(&self) -> Result<(Connection, Option<Vec<Placed>>)> {
        let connection = self.connection()?;
        if matches!(
            connection.state,
            ConnectionState::NoKey | ConnectionState::NoShop | ConnectionState::StoreUnmatched
        ) {
            return Ok((connection, None));
        }
        let sessions: HashMap<String, i64> = self.store.item_sessions()?.into_iter().collect();
        let copy = self
            .store
            .conversations()?
            .into_iter()
            .map(|c| {
                let session = c.sku.as_ref().and_then(|sku| sessions.get(sku).copied());
                (c, session)
            })
            .collect();
        Ok((connection, Some(copy)))
    }

    /// C2 (09B §7 List): the panel's rows for `scope` and `tab` at `now`.
    pub fn list(
        &self,
        scope: ConversationScope,
        tab: ConversationTab,
        now_ms: i64,
    ) -> Result<ConversationList> {
        let (connection, copy) = self.visible()?;
        let problem = matches!(
            connection.state,
            ConnectionState::Refused | ConnectionState::StoreGone
        )
        .then(|| connection.last_error.clone())
        .flatten();
        let Some(copy) = copy else {
            return Ok(ConversationList {
                state: connection.state,
                problem,
                rows: vec![],
                news_elsewhere: 0,
            });
        };
        let in_scope = |session: Option<i64>| match scope {
            ConversationScope::All => true,
            ConversationScope::Session(id) => session == Some(id),
        };
        let groups = by_product(copy);
        let group_session = |group: &[Placed]| group.iter().find_map(|(_, s)| *s);
        let news_elsewhere = groups
            .iter()
            .filter(|g| !in_scope(group_session(g)) && g.iter().any(|(c, _)| has_news(c)))
            .count() as i64;
        // A30: Open = a product with any unresolved question; Resolved = every question resolved,
        // the last update within 30 days.
        let mut rows: Vec<ConversationRow> = groups
            .iter()
            .filter(|g| in_scope(group_session(g)))
            .filter(|g| match tab {
                ConversationTab::Open => unresolved(g),
                ConversationTab::Resolved => {
                    !unresolved(g)
                        && resolved_recent(
                            g.iter().map(|(c, _)| c.updated_at).max().unwrap_or(0),
                            now_ms,
                        )
                }
            })
            .map(|g| row_of(g))
            .collect();
        match tab {
            ConversationTab::Open => rows.sort_by(list_order),
            ConversationTab::Resolved => rows.sort_by(|a, b| {
                b.last_activity_at
                    .cmp(&a.last_activity_at)
                    .then(a.id.cmp(&b.id))
            }),
        }
        Ok(ConversationList {
            state: connection.state,
            problem,
            rows,
            news_elsewhere,
        })
    }

    /// C2 (09B §6; A30): products with news (any status) of `session_id` and of all the others;
    /// without a session, `here` = all.
    pub fn counts(&self, session_id: Option<i64>) -> Result<ConversationCounts> {
        let (_, copy) = self.visible()?;
        let Some(copy) = copy else {
            return Ok(ConversationCounts::default());
        };
        let mut counts = ConversationCounts::default();
        for group in by_product(copy)
            .iter()
            .filter(|g| g.iter().any(|(c, _)| has_news(c)))
        {
            let session = group.iter().find_map(|(_, s)| *s);
            match session_id {
                Some(id) if session != Some(id) => counts.elsewhere += 1,
                _ => counts.here += 1,
            }
        }
        Ok(counts)
    }

    /// C2 (09B §6): one badge per session with an unresolved conversation; `waiting` when one of
    /// them is blocking.
    pub fn session_badges(&self) -> Result<Vec<SessionBadge>> {
        let (_, copy) = self.visible()?;
        let mut badges: Vec<SessionBadge> = Vec::new();
        for (c, session) in copy.unwrap_or_default() {
            let Some(session_id) = session else { continue };
            if c.status == ConversationStatus::Resolved {
                continue;
            }
            match badges.iter_mut().find(|b| b.session_id == session_id) {
                Some(badge) => {
                    badge.open += 1;
                    badge.waiting |= c.blocking;
                }
                None => badges.push(SessionBadge {
                    session_id,
                    open: 1,
                    waiting: c.blocking,
                }),
            }
        }
        badges.sort_by_key(|b| b.session_id);
        Ok(badges)
    }

    // ---------- C3: the outbox, the view, Mark answered, the attachable photos (MC-C2) ----------

    /// MC-C2.2 Flush: the `sending` rows oldest first, each sent in the lane with exactly its
    /// stored id, text, reply target and media ids, its outcome applied in the same lane turn. An
    /// error here changes only outbox rows (no `failures`, no schedule, no `last_error`), except
    /// `Refused` / `StoreGone`, which apply MC-C1.8.
    fn flush(&self, store_id: &str, now: i64) -> Result<()> {
        let rows: Vec<OutboxMessage> = self
            .store
            .outbox()?
            .into_iter()
            .filter(|r| r.state == OutboxState::Sending)
            .collect();
        let mut changed = false;
        for (index, row) in rows.iter().enumerate() {
            let rest: Vec<String> = rows[index..]
                .iter()
                .map(|r| r.client_mutation_id.clone())
                .collect();
            let outcome = self.in_lane(store_id, None, |_| {
                let still = self.store.outbox_row(&row.client_mutation_id)?;
                if still.as_ref().map(|r| r.state) != Some(OutboxState::Sending) {
                    return Ok(None);
                }
                let message = NewMessage {
                    client_mutation_id: row.client_mutation_id.clone(),
                    text: row.text.clone(),
                    reply_to_message_id: row.reply_to.as_ref().map(|r| r.message_id.clone()),
                    attachment_media_ids: row.attachment_media_ids.clone(),
                };
                let sent = self
                    .provider
                    .post_message(store_id, &row.conversation_id, &message);
                Ok(Some(self.apply_send(row, sent, &rest, now)?))
            });
            match outcome {
                Ok(None) => {}
                Ok(Some(stop)) => {
                    changed = true;
                    if stop {
                        break;
                    }
                }
                Err(Stop::Storage(error)) => return Err(error),
                Err(Stop::Changed | Stop::Provider(_)) => break,
            }
        }
        if changed {
            self.emit(vec![]);
        }
        Ok(())
    }

    /// MC-C2.2: one transaction for the outcome of the send of `row`; `true` when the flush stops
    /// (`rest` = this row and the ones after it, which fail with it without a request).
    fn apply_send(
        &self,
        row: &OutboxMessage,
        sent: std::result::Result<crate::domain::conversation::Message, ProviderError>,
        rest: &[String],
        now: i64,
    ) -> Result<bool> {
        let id = std::slice::from_ref(&row.client_mutation_id);
        let (outcome, stop) = match sent {
            Ok(message) => {
                self.store
                    .outbox_sent(&row.client_mutation_id, &message, now)?;
                ("sent", false)
            }
            Err(error @ (ProviderError::Unreachable(_) | ProviderError::BadReply(_))) => {
                self.store.outbox_move(
                    rest,
                    OutboxState::Sending,
                    OutboxState::Failed,
                    Some(COULD_NOT_SEND),
                    now,
                )?;
                (error.kind(), true)
            }
            Err(error @ (ProviderError::Refused | ProviderError::StoreGone)) => {
                // MC-C1.8; the rows after this one cannot be sent in that state either.
                let sentence = self.describe(&error);
                self.store.outbox_move(
                    rest,
                    OutboxState::Sending,
                    OutboxState::Failed,
                    Some(&sentence),
                    now,
                )?;
                let state = if error == ProviderError::Refused {
                    ConnectionState::Refused
                } else {
                    ConnectionState::StoreGone
                };
                self.store.record_error(state, &sentence, now)?;
                self.memory().more = false;
                (error.kind(), true)
            }
            Err(error) => {
                let sentence = self.describe(&error);
                self.store.outbox_move(
                    id,
                    OutboxState::Sending,
                    OutboxState::Failed,
                    Some(&sentence),
                    now,
                )?;
                (error.kind(), false)
            }
        };
        tracing::info!(
            target: "psm::conversations",
            "conversations.send outcome={outcome} conversation={}",
            row.conversation_id
        );
        Ok(stop)
    }

    /// The state as `connection()` reports it, with `S`.
    fn state_now(&self) -> Result<(ConnectionState, SyncRecord)> {
        let s = self.store.sync()?;
        let (state, _) = self.derive(&s, self.key().is_some(), &self.settings());
        Ok((state, s))
    }

    /// The state's sentence (projection L5): in `refused` / `store_gone`, `describe` of that
    /// state's error, which is `S.last_error`; none in the other states.
    fn state_sentence(&self, state: ConnectionState, s: &SyncRecord) -> Option<String> {
        let error = match state {
            ConnectionState::Refused => ProviderError::Refused,
            ConnectionState::StoreGone => ProviderError::StoreGone,
            _ => return None,
        };
        Some(
            s.last_error
                .clone()
                .unwrap_or_else(|| self.describe(&error)),
        )
    }

    fn off(&self, state: ConnectionState, s: &SyncRecord) -> ConversationError {
        ConversationError::Off {
            state,
            message: self.state_sentence(state, s).unwrap_or_default(),
        }
    }

    /// The state for a C3 method: `Off` in the hidden states (nothing is written).
    fn shown_state(&self) -> std::result::Result<(ConnectionState, SyncRecord), ConversationError> {
        let (state, s) = self.state_now()?;
        if hidden(state) {
            return Err(self.off(state, &s));
        }
        Ok((state, s))
    }

    fn copy_conversation(&self, id: &str) -> std::result::Result<Conversation, ConversationError> {
        self.store
            .conversation(id)?
            .ok_or_else(|| Error::NotFound(format!("conversation {id}")).into())
    }

    /// The session of the item whose SKU is exactly the conversation's (09B §3).
    fn session_of(&self, conversation: &Conversation) -> Result<Option<i64>> {
        let Some(sku) = &conversation.sku else {
            return Ok(None);
        };
        Ok(self
            .store
            .item_sessions()?
            .into_iter()
            .find(|(item_sku, _)| item_sku == sku)
            .map(|(_, session)| session))
    }

    /// The preview of each media id this Mac holds a link for, in `session_id`.
    fn previews(&self, session_id: Option<i64>) -> Result<HashMap<String, Option<String>>> {
        let Some(session_id) = session_id else {
            return Ok(HashMap::new());
        };
        let mut previews = HashMap::new();
        for link in self.store.session_links(session_id)? {
            if let Some(media) = &link.external_id {
                previews
                    .entry(media.clone())
                    .or_insert_with(|| link_preview(&link));
            }
        }
        Ok(previews)
    }

    fn pending_view(row: OutboxMessage, previews: &HashMap<String, Option<String>>) -> PendingView {
        PendingView {
            attachments: row
                .attachment_media_ids
                .iter()
                .map(|media_id| PendingAttachment {
                    media_id: media_id.clone(),
                    preview_path: previews.get(media_id).cloned().flatten(),
                })
                .collect(),
            client_mutation_id: row.client_mutation_id,
            text: row.text,
            reply_to: row.reply_to,
            state: row.state,
            failure: row.failure,
            created_at: row.created_at,
        }
    }

    /// One outbox row as the view shows it, its previews read from its conversation's session.
    fn pending_of(&self, row: OutboxMessage) -> Result<PendingView> {
        let session = match self.store.conversation(&row.conversation_id)? {
            Some(c) => self.session_of(&c)?,
            None => None,
        };
        Ok(Self::pending_view(row, &self.previews(session)?))
    }

    /// MC-C2.6: the conversation, its messages in order, then its `sending` and `failed` replies,
    /// oldest first. Local only: never takes the lane, never asks the provider.
    pub fn view(
        &self,
        conversation_id: &str,
    ) -> std::result::Result<ConversationView, ConversationError> {
        let (state, s) = self.shown_state()?;
        let c = self.copy_conversation(conversation_id)?;
        let session_id = self.session_of(&c)?;
        let previews = self.previews(session_id)?;
        let pending = self
            .store
            .outbox()?
            .into_iter()
            .filter(|r| r.conversation_id == c.id && r.state != OutboxState::Sent)
            .map(|r| Self::pending_view(r, &previews))
            .collect();
        let jump_keys = self.jump_keys(&c.messages, session_id)?;
        Ok(ConversationView {
            id: c.id,
            sku: c.sku,
            product_title: c.product_title,
            topic: c.topic,
            blocking: c.blocking,
            status: c.status,
            resolved_by: c.resolved_by,
            session_id,
            messages: c.messages,
            pending,
            can_send: state == ConnectionState::Ready,
            off_reason: self.state_sentence(state, &s),
            jump_keys,
        })
    }

    /// A30 (09B §7 Thread): the product of `conversation_id` with all its questions, oldest
    /// first, each as [`Self::view`] shows it. Local only, like `view`.
    pub fn thread(
        &self,
        conversation_id: &str,
    ) -> std::result::Result<ConversationThread, ConversationError> {
        self.shown_state()?;
        let target = self.copy_conversation(conversation_id)?;
        let key = product_key(&target);
        let mut ids: Vec<(i64, String)> = self
            .store
            .conversations()?
            .into_iter()
            .filter(|c| product_key(c) == key)
            .map(|c| (c.created_at, c.id))
            .collect();
        ids.sort();
        let questions = ids
            .iter()
            .map(|(_, id)| self.view(id))
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let session_id = questions.iter().find_map(|q| q.session_id);
        Ok(ConversationThread {
            product_id: target.product_id,
            sku: target.sku,
            product_title: target.product_title,
            session_id,
            questions,
        })
    }

    /// C4 (09B §7 quote chip jump; projection L4): per quoted message, the attribute key its
    /// quote's field names (`provider.attribute_key`), kept only when the conversation's session
    /// is on this Mac and that key is a field of the session's form. Nothing is stored.
    fn jump_keys(
        &self,
        messages: &[Message],
        session_id: Option<i64>,
    ) -> Result<BTreeMap<String, String>> {
        let Some(session_id) = session_id else {
            return Ok(BTreeMap::new());
        };
        let quoted: Vec<(&Message, String)> = messages
            .iter()
            .filter_map(|m| {
                let quote = m.quote.as_ref()?;
                Some((m, self.provider.attribute_key(&quote.field)?))
            })
            .collect();
        if quoted.is_empty() {
            return Ok(BTreeMap::new());
        }
        let form = self
            .store
            .form_keys(session_id, self.integration.as_deref())?;
        Ok(quoted
            .into_iter()
            .filter(|(_, key)| form.contains(key))
            .map(|(m, key)| (m.id.clone(), key))
            .collect())
    }

    /// MC-C2.1 Accept: one transaction, no request. Idempotent on `client_mutation_id`: a row
    /// with that id is returned as it is. A new row is `sending`, or `failed` with the state's
    /// sentence when the key is refused or the store is gone.
    pub fn reply(
        &self,
        conversation_id: &str,
        input: NewMessage,
    ) -> std::result::Result<PendingView, ConversationError> {
        let (state, s) = self.shown_state()?;
        let message = checked_reply(input)?;
        if let Some(existing) = self.store.outbox_row(&message.client_mutation_id)? {
            return Ok(self.pending_of(existing)?);
        }
        let c = self.copy_conversation(conversation_id)?;
        let reply_to = match &message.reply_to_message_id {
            None => None,
            Some(target) => {
                let m = c
                    .messages
                    .iter()
                    .find(|m| &m.id == target)
                    .ok_or_else(|| Error::InvalidValue("reply_to_message_id".into()))?;
                Some(ReplyRef {
                    message_id: m.id.clone(),
                    author_name: m.author.name.clone(),
                    excerpt: reply_excerpt(&m.text),
                })
            }
        };
        let failure = self.state_sentence(state, &s);
        let now = self.clock.now_ms();
        let row = OutboxMessage {
            client_mutation_id: message.client_mutation_id,
            conversation_id: c.id,
            text: message.text,
            reply_to,
            attachment_media_ids: message.attachment_media_ids,
            state: if failure.is_some() {
                OutboxState::Failed
            } else {
                OutboxState::Sending
            },
            failure,
            server_message_id: None,
            created_at: now,
        };
        let (row, inserted) = self.store.outbox_accept(&row, now)?;
        if inserted {
            self.emit(vec![]);
        }
        Ok(self.pending_of(row)?)
    }

    /// MC-C2.3 Retry: a `failed` row becomes `sending` with its failure cleared (or `failed`
    /// again with the state's sentence when the key is refused or the store is gone); any other
    /// row is unchanged. The row keeps its `client_mutation_id`.
    pub fn retry(
        &self,
        client_mutation_id: &str,
    ) -> std::result::Result<PendingView, ConversationError> {
        let (state, s) = self.shown_state()?;
        let row = self
            .store
            .outbox_row(client_mutation_id)?
            .ok_or_else(|| Error::NotFound(format!("reply {client_mutation_id}")))?;
        if row.state == OutboxState::Failed {
            let failure = self.state_sentence(state, &s);
            let to = if failure.is_some() {
                OutboxState::Failed
            } else {
                OutboxState::Sending
            };
            let moved = self.store.outbox_move(
                std::slice::from_ref(&row.client_mutation_id),
                OutboxState::Failed,
                to,
                failure.as_deref(),
                self.clock.now_ms(),
            )?;
            if moved > 0 {
                self.emit(vec![]);
            }
        }
        let row = self
            .store
            .outbox_row(client_mutation_id)?
            .ok_or_else(|| Error::NotFound(format!("reply {client_mutation_id}")))?;
        Ok(self.pending_of(row)?)
    }

    /// MC-C2.3 Discard: a `failed` row is deleted (local only, no request); any other is refused.
    pub fn discard(&self, client_mutation_id: &str) -> std::result::Result<(), ConversationError> {
        let row = self
            .store
            .outbox_row(client_mutation_id)?
            .ok_or_else(|| Error::NotFound(format!("reply {client_mutation_id}")))?;
        if row.state != OutboxState::Failed {
            return Err(Error::InvalidValue("client_mutation_id".into()).into());
        }
        if self.store.outbox_discard(client_mutation_id)? {
            self.emit(vec![]);
        }
        Ok(())
    }

    /// MC-C2.7 Mark answered, in the lane: `Off` when calls are not allowed (no request); the
    /// answer replaces the copy (MC-C1.4); `Resolved` re-reads the conversation and applies it;
    /// any other error leaves the copy unchanged (`Refused` / `StoreGone` also set the state).
    pub fn mark_answered(
        &self,
        conversation_id: &str,
    ) -> std::result::Result<(), ConversationError> {
        let (state, s) = self.state_now()?;
        if state != ConnectionState::Ready {
            return Err(self.off(state, &s));
        }
        self.copy_conversation(conversation_id)?;
        let Some(store_id) = s.store.map(|st| st.id) else {
            return Err(self.off(state, &self.store.sync()?));
        };
        let now = self.clock.now_ms();
        let (changed, result) = {
            let _lane = self.lane.enter();
            let current = self.calls_allowed()?;
            if current.and_then(|s| s.store).map(|st| st.id).as_deref() != Some(&store_id) {
                drop(_lane);
                let (state, s) = self.state_now()?;
                return Err(self.off(state, &s));
            }
            match self.provider.mark_answered(&store_id, conversation_id) {
                Ok(conversation) => (self.apply_own(&conversation)?, Ok(())),
                Err(ProviderError::Resolved) => {
                    let changed = match self.provider.conversation(&store_id, conversation_id) {
                        Ok(Some(conversation)) => self.apply_own(&conversation)?,
                        Ok(None) => self.store.delete_conversation(conversation_id, None)?,
                        Err(_) => false,
                    };
                    (changed, Err(ConversationError::Resolved))
                }
                Err(error @ (ProviderError::Refused | ProviderError::StoreGone)) => {
                    let sentence = self.describe(&error);
                    let state = if error == ProviderError::Refused {
                        ConnectionState::Refused
                    } else {
                        ConnectionState::StoreGone
                    };
                    self.store.record_error(state, &sentence, now)?;
                    (true, Err(ConversationError::Provider(error)))
                }
                Err(error) => (false, Err(ConversationError::Provider(error))),
            }
        };
        if changed {
            self.emit(vec![]);
        }
        result
    }

    /// MC-C1.4 replace of a conversation the provider answered outside a check, keeping the
    /// stored `resolved_by` while it is resolved.
    fn apply_own(&self, conversation: &Conversation) -> Result<bool> {
        let mut next = conversation.clone();
        next.resolved_by = if next.status == ConversationStatus::Resolved {
            self.store
                .conversation(&next.id)?
                .and_then(|c| c.resolved_by)
        } else {
            None
        };
        self.store.apply(&next, entering_read_mark(&next), None)
    }

    /// 09B §8: the published photos of the conversation's product a reply may attach.
    pub fn attachable(
        &self,
        conversation_id: &str,
    ) -> std::result::Result<Attachable, ConversationError> {
        self.shown_state()?;
        let c = self.copy_conversation(conversation_id)?;
        let Some(session_id) = self.session_of(&c)? else {
            return Ok(Attachable {
                photos: vec![],
                reason: Some(AttachReason::NotOnThisMac),
            });
        };
        let photos = attachable_photos(session_id, &self.store.session_links(session_id)?);
        let reason = photos.is_empty().then_some(AttachReason::NothingPublished);
        Ok(Attachable { photos, reason })
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
        let before = self.connection()?.state;
        self.slot.write(next)?;
        self.emit_if_changed(before, self.connection()?.state);
        Ok(self.settings())
    }
}
