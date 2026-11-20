//! `ConversationsService` (book 09B §4–§6; `master_plan_conversations.md` §4 Service, §5 MC-C1):
//! the connection to a conversations provider, the key held in memory, the store match, and the
//! local copy kept up to date by `tick`. Platform-neutral (H7): every sentence comes from
//! `provider.describe`, every setting through the slot port, the key through `SecretStore`.

use crate::{
    Error, Result,
    domain::conversation::{
        Connection, ConnectionState, Conversation, ConversationCounts, ConversationList,
        ConversationNotice, ConversationRow, ConversationScope, ConversationSettings,
        ConversationStatus, ConversationTab, EventKind, EventPage, KEY_ACCOUNT, NoticeKind,
        ProviderError, ResolvedBy, SessionBadge, Tick,
    },
    events::AppEvent,
    mechanisms::conversation_rules::{
        check_backoff, entering_read_mark, first_line, has_news, list_order, resolved_recent,
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
use std::{
    collections::{HashMap, HashSet},
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
    clock: Arc<dyn Clock>,
    events: Arc<dyn EventSink>,
    lane: Arc<ConversationsLane>,
    memory: Mutex<Memory>,
}

/// A conversation of the copy with its session (none = "Not on this Mac").
type Placed = (Conversation, Option<i64>);

/// One list row of `c` (09B §7): `waiting` = blocking and not resolved; `last_line` = the first
/// line of the newest message; `last_activity_at` = that message's time, or `updated_at`.
fn row_of(c: &Conversation, session_id: Option<i64>) -> ConversationRow {
    let newest = c.messages.iter().max_by_key(|m| m.created_at);
    ConversationRow {
        id: c.id.clone(),
        sku: c.sku.clone(),
        product_title: c.product_title.clone(),
        topic: c.topic.clone(),
        status: c.status,
        blocking: c.blocking,
        waiting: c.blocking && c.status != ConversationStatus::Resolved,
        last_line: newest
            .map(|m| first_line(&m.text))
            .filter(|l| !l.is_empty()),
        last_activity_at: newest.map_or(c.updated_at, |m| m.created_at),
        has_news: has_news(c),
        session_id,
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
        let news_elsewhere = copy
            .iter()
            .filter(|(c, session)| !in_scope(*session) && has_news(c))
            .count() as i64;
        let mut rows: Vec<ConversationRow> = copy
            .into_iter()
            .filter(|(_, session)| in_scope(*session))
            .filter(|(c, _)| match tab {
                ConversationTab::Open => c.status != ConversationStatus::Resolved,
                ConversationTab::Resolved => {
                    c.status == ConversationStatus::Resolved
                        && resolved_recent(c.updated_at, now_ms)
                }
            })
            .map(|(c, session)| row_of(&c, session))
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

    /// C2 (09B §6): conversations with news (any status) of `session_id` and of all the others;
    /// without a session, `here` = all.
    pub fn counts(&self, session_id: Option<i64>) -> Result<ConversationCounts> {
        let (_, copy) = self.visible()?;
        let Some(copy) = copy else {
            return Ok(ConversationCounts::default());
        };
        let mut counts = ConversationCounts::default();
        for (_, session) in copy.iter().filter(|(c, _)| has_news(c)) {
            match session_id {
                Some(id) if *session != Some(id) => counts.elsewhere += 1,
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
