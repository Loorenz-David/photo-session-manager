//! `LabelsService` (book 09C §4, §7, §8; `master_plan_labels.md` §4 Service, §5 ACC, MC-L3,
//! MC-L4): whether label calls may be made, the display copy of label state, the status read of
//! session markers, and the label changes the conversations pull hands over. Platform-neutral
//! (H7): the provider is a port, its sentences come from `describe`, its id forms from
//! `product_id_of`. Nothing about label state is stored (M-L1): the copy lives in memory; the
//! only stored fact is whether the saved key may read labels. Nothing here starts a thread or a
//! timer.
//!
//! L2 (MC-L1): the Labels box's list (rows in the provider's order, matched to local sessions,
//! with their pictures), its number, Select all, and opening an unmatched row (the existing
//! adopt: one shop read, no write). The counts and the SKUs of rows seen live in memory only.
//!
//! L3 (MC-L2): the check, the preview, generating, the result and the download live in the
//! sibling module `label_runs`; the one kept check is a field here.

use crate::{
    Error, Result,
    domain::{
        conversation::ConnectionState,
        label::{
            LabelChange, LabelCounts, LabelFilter, LabelPage, LabelProductPage, LabelRow,
            LabelState, LabelsAccess, LabelsError, LinkedItem, Selected, SessionLabel,
        },
        session::ListFilter,
    },
    events::AppEvent,
    mechanisms::labels::{
        COUNT_FRESH_MS, LIST_PAGE, LabelCopy, QUERY_MAX, SELECT_ALL_MAX_PAGES, SELECT_ALL_PAGE,
        STATUS_CHUNK, STATUS_FRESH_MS, chunks, match_product,
    },
    ports::{
        clock::Clock,
        commerce::CommerceAdapter,
        events::EventSink,
        labels::{LabelFiles, LabelStore, LabelsLink, LabelsProvider, LabelsResult},
        store::Store,
    },
    services::{
        conversations::ConversationsService, items::ItemsService, label_runs::KeptCheck,
        remote::RemoteItems, sessions::SessionsService,
    },
};
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex, MutexGuard},
    time::{Duration, Instant},
};

/// What the labels service answers when it cannot (master plan §4 Errors).
#[derive(Debug, thiserror::Error)]
pub enum LabelsFailure {
    /// The conversations connection is not ready; its state.
    #[error("labels off: {}", .0.as_str())]
    Off(ConnectionState),
    /// The saved key may not read labels.
    #[error("labels denied")]
    Denied,
    #[error("{0}")]
    Provider(LabelsError),
    #[error("Labels are being generated.")]
    Busy,
    #[error("None of these products can be generated.")]
    NothingToGenerate,
    #[error("Not found")]
    NotFound,
    #[error("The label preview can't be shown.")]
    PreviewFailed(String),
    #[error("Couldn't save the PDF: {0}.")]
    SaveFailed(String),
    #[error("{0}")]
    Storage(#[from] Error),
}

const LOG: &str = "psm::labels";

/// MC-L1.5: the counts of the last list answer (memory only), whether its request had a search,
/// and when it was answered (monotonic).
#[derive(Debug, Clone, Copy)]
struct Remembered {
    counts: LabelCounts,
    searched: bool,
    at: Instant,
}

/// MC-L1.1: the search sent to the provider: trimmed, none when empty, cut to `QUERY_MAX`
/// characters.
fn search_text(query: Option<&str>) -> Option<String> {
    let q = query?.trim();
    (!q.is_empty()).then(|| q.chars().take(QUERY_MAX).collect())
}

pub struct LabelsService {
    conversations: Arc<ConversationsService>,
    pub(super) provider: Arc<dyn LabelsProvider>,
    labels: Arc<dyn LabelStore>,
    store: Arc<dyn Store>,
    remote: Arc<RemoteItems>,
    commerce: Option<Arc<dyn CommerceAdapter>>,
    /// L3: preview pictures and saved label files.
    pub(super) files: Arc<dyn LabelFiles>,
    pub(super) clock: Arc<dyn Clock>,
    pub(super) events: Arc<dyn EventSink>,
    copy: Mutex<LabelCopy>,
    /// L3 (MC-L2.3): the one kept check (memory only, M-L1).
    pub(super) kept: Mutex<Option<KeptCheck>>,
    /// The stored access as last read or written (`None` = not read yet in this build).
    access: Mutex<Option<LabelsAccess>>,
    /// MC-L1.5, .6: the counts of the last list or summary answer.
    counts: Mutex<Option<Remembered>>,
    /// MC-L1.8: the SKU of every product a list answer showed (product id → SKU), so opening a
    /// row matches it by rule 3 as the list did; L4 (MC-L2.16): with its picture address, so the
    /// check can name a picture already on this Mac (rows of `select_all` too).
    seen: Mutex<HashMap<String, SeenRow>>,
}

/// MC-L1.8, MC-L2.16: what is remembered of a row a list or Select all answer carried.
#[derive(Debug, Clone, Default)]
struct SeenRow {
    sku: Option<String>,
    thumb_url: Option<String>,
}

impl LabelsService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        conversations: Arc<ConversationsService>,
        provider: Arc<dyn LabelsProvider>,
        labels: Arc<dyn LabelStore>,
        store: Arc<dyn Store>,
        remote: Arc<RemoteItems>,
        commerce: Option<Arc<dyn CommerceAdapter>>,
        files: Arc<dyn LabelFiles>,
        clock: Arc<dyn Clock>,
        events: Arc<dyn EventSink>,
    ) -> Self {
        Self {
            conversations,
            provider,
            labels,
            store,
            remote,
            commerce,
            files,
            clock,
            events,
            copy: Mutex::new(LabelCopy::new()),
            kept: Mutex::new(None),
            access: Mutex::new(None),
            counts: Mutex::new(None),
            seen: Mutex::new(HashMap::new()),
        }
    }

    fn counts(&self) -> MutexGuard<'_, Option<Remembered>> {
        self.counts.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn seen(&self) -> MutexGuard<'_, HashMap<String, SeenRow>> {
        self.seen.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// MC-L1.8, MC-L2.16: the rows of an answer are remembered (SKU and picture address).
    fn remember(&self, products: &[crate::domain::label::LabelProduct]) {
        let mut seen = self.seen();
        for product in products {
            seen.insert(
                product.product_id.clone(),
                SeenRow {
                    sku: product.sku.clone(),
                    thumb_url: product.thumb_url.clone(),
                },
            );
        }
    }

    pub(super) fn copy(&self) -> MutexGuard<'_, LabelCopy> {
        self.copy.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn stored_access(&self) -> Result<LabelsAccess> {
        let mut held = self.access.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(access) = *held {
            return Ok(access);
        }
        let access = self.labels.labels_access()?;
        *held = Some(access);
        Ok(access)
    }

    fn set_access(&self, access: LabelsAccess) -> Result<()> {
        let mut held = self.access.lock().unwrap_or_else(|e| e.into_inner());
        self.labels.set_labels_access(access)?;
        *held = Some(access);
        tracing::info!(target: LOG, "labels.access state={}", access.stored());
        Ok(())
    }

    /// ACC.7: `Off(state)` when the connection is not ready, else the stored access.
    pub fn access(&self) -> Result<LabelsAccess> {
        let connection = self.conversations.connection()?;
        if connection.state != ConnectionState::Ready {
            return Ok(LabelsAccess::Off(connection.state));
        }
        self.stored_access()
    }

    /// ACC.1, .6: the store a label call may address now (read from the connection when the call
    /// starts), or none when no label call may be made.
    fn allowed_store(&self) -> Result<Option<String>> {
        let connection = self.conversations.connection()?;
        if connection.state != ConnectionState::Ready {
            return Ok(None);
        }
        let Some(store) = connection.store else {
            return Ok(None);
        };
        if self.stored_access()? == LabelsAccess::Denied {
            return Ok(None);
        }
        Ok(Some(store.id))
    }

    /// ACC.1 for a press: the store a label call may address, else `Off(state)` or `Denied`.
    pub(super) fn gate(&self) -> std::result::Result<String, LabelsFailure> {
        let connection = self.conversations.connection()?;
        if connection.state != ConnectionState::Ready {
            return Err(LabelsFailure::Off(connection.state));
        }
        let Some(store) = connection.store else {
            return Err(LabelsFailure::Off(connection.state));
        };
        if self.stored_access()? == LabelsAccess::Denied {
            return Err(LabelsFailure::Denied);
        }
        Ok(store.id)
    }

    /// The sentence shown for a failure (the provider's words for its errors, the connected
    /// shop; empty for `Off`, whose words the screen has).
    pub fn describe(&self, failure: &LabelsFailure) -> String {
        match failure {
            LabelsFailure::Off(_) => String::new(),
            LabelsFailure::Denied => self.describe_error(&LabelsError::NotAllowed),
            LabelsFailure::Provider(error) => self.describe_error(error),
            other => other.to_string(),
        }
    }

    pub(super) fn describe_error(&self, error: &LabelsError) -> String {
        let shop = self.conversations.connection().ok().and_then(|c| c.shop);
        self.provider.describe(error, shop.as_deref())
    }

    /// ACC.4, learning by use, for every label call but `connected()`'s own: a success while the
    /// access is unknown makes it allowed; `NotAllowed` makes it denied, empties the copy and
    /// tells the screen. Never touches the conversations connection (ACC.5).
    pub(super) fn learn<T>(
        &self,
        result: LabelsResult<T>,
    ) -> std::result::Result<T, LabelsFailure> {
        match result {
            Ok(value) => {
                if self.stored_access()? == LabelsAccess::Unknown {
                    self.set_access(LabelsAccess::Allowed)?;
                }
                Ok(value)
            }
            Err(LabelsError::NotAllowed) => {
                self.set_access(LabelsAccess::Denied)?;
                self.copy().clear();
                self.events.emit(AppEvent::LabelsChanged {
                    product_ids: vec![],
                    access: true,
                });
                Err(LabelsFailure::Provider(LabelsError::NotAllowed))
            }
            Err(error) => Err(LabelsFailure::Provider(error)),
        }
    }

    /// MC-L3.3: each session with its product and the product's label state from the copy, in
    /// request order. With `refresh`, when calls are allowed and a product of the request has no
    /// answer younger than `STATUS_FRESH_MS`, the request's products are asked first (one status
    /// request per `STATUS_CHUNK`); an error is returned with the copy unchanged. When calls are
    /// not allowed nothing is asked and every state is none.
    pub fn session_labels(
        &self,
        session_ids: &[i64],
        refresh: bool,
    ) -> std::result::Result<Vec<SessionLabel>, LabelsFailure> {
        let linked: HashMap<i64, Option<String>> = self
            .labels
            .session_products(session_ids)?
            .into_iter()
            .collect();
        let products: Vec<(i64, Option<String>)> = session_ids
            .iter()
            .map(|id| {
                let product = linked
                    .get(id)
                    .cloned()
                    .flatten()
                    .and_then(|external| self.provider.product_id_of(&external));
                (*id, product)
            })
            .collect();
        let Some(store) = self.allowed_store()? else {
            return Ok(products
                .into_iter()
                .map(|(session_id, product_id)| SessionLabel {
                    session_id,
                    product_id,
                    state: None,
                })
                .collect());
        };
        if refresh {
            self.refresh(&store, &products)?;
        }
        let copy = self.copy();
        Ok(products
            .into_iter()
            .map(|(session_id, product_id)| SessionLabel {
                session_id,
                state: product_id.as_deref().and_then(|p| copy.get(p)),
                product_id,
            })
            .collect())
    }

    fn refresh(
        &self,
        store: &str,
        products: &[(i64, Option<String>)],
    ) -> std::result::Result<(), LabelsFailure> {
        let mut ids: Vec<String> = Vec::new();
        for id in products.iter().filter_map(|(_, p)| p.as_ref()) {
            if !ids.contains(id) {
                ids.push(id.clone());
            }
        }
        let max_age = Duration::from_millis(STATUS_FRESH_MS);
        let started = {
            let copy = self.copy();
            let now = self.clock.mono();
            if ids.iter().all(|id| copy.fresh(id, now, max_age)) {
                return Ok(());
            }
            copy.start()
        };
        let mut answers: Vec<(String, Option<LabelState>)> = Vec::with_capacity(ids.len());
        for range in chunks(ids.len(), STATUS_CHUNK) {
            let chunk = &ids[range];
            let states = self.learn(self.provider.status(store, chunk))?;
            answers.extend(chunk.iter().cloned().zip(states));
        }
        let answered = answers.iter().filter(|(_, s)| s.is_some()).count();
        let now = self.clock.mono();
        let mut copy = self.copy();
        for (id, state) in answers {
            copy.apply_answer(started, &id, state, now);
        }
        tracing::info!(
            target: LOG,
            "labels.status asked={} answered={answered}",
            ids.len()
        );
        Ok(())
    }

    /// ACC.2, .3: the copy, the remembered counts and the rows seen (SKUs and picture
    /// addresses) are emptied.
    fn forget(&self) {
        self.copy().clear();
        *self.counts() = None;
        self.seen().clear();
    }

    /// One products request whose rows' states are applied to the copy and whose counts are
    /// remembered (MC-L1.5).
    fn products(
        &self,
        store: &str,
        filter: LabelFilter,
        q: Option<&str>,
        cursor: Option<&str>,
        limit: u32,
    ) -> std::result::Result<LabelProductPage, LabelsFailure> {
        let started = self.copy().start();
        let page = self.learn(self.provider.products(store, filter, q, cursor, limit))?;
        let now = self.clock.mono();
        {
            let mut copy = self.copy();
            for product in &page.products {
                copy.apply_answer(started, &product.product_id, Some(product.state), now);
            }
        }
        *self.counts() = Some(Remembered {
            counts: page.counts,
            searched: q.is_some(),
            at: now,
        });
        Ok(page)
    }

    /// Every local item with its link's provider product id (MC-L1.3).
    fn match_items(&self) -> Result<Vec<(LinkedItem, Option<String>)>> {
        Ok(self
            .labels
            .linked_items()?
            .into_iter()
            .map(|item| {
                let product = item
                    .external_id
                    .as_deref()
                    .and_then(|external| self.provider.product_id_of(external));
                (item, product)
            })
            .collect())
    }

    /// MC-L1.1–.5: one page of the provider's label list (`LIST_PAGE` rows), each row matched to
    /// a local session (one sessions page for the matched ones) or given its cached picture.
    pub fn list(
        &self,
        filter: LabelFilter,
        query: Option<&str>,
        cursor: Option<&str>,
    ) -> std::result::Result<LabelPage, LabelsFailure> {
        let store = self.gate()?;
        let q = search_text(query);
        let page = self.products(&store, filter, q.as_deref(), cursor, LIST_PAGE)?;
        self.remember(&page.products);
        let items = self.match_items()?;
        let matched: Vec<Option<i64>> = page
            .products
            .iter()
            .map(|product| {
                match_product(&product.product_id, product.sku.as_deref(), &items)
                    .and_then(|i| items[i].0.session_id)
            })
            .collect();
        let mut ids: Vec<i64> = Vec::new();
        for id in matched.iter().flatten() {
            if !ids.contains(id) {
                ids.push(*id);
            }
        }
        let mut sessions: HashMap<i64, crate::domain::session::SessionRow> = HashMap::new();
        if !ids.is_empty() {
            let rows = SessionsService::new(self.store.as_ref(), self.clock.as_ref())
                .with_remote(self.remote.as_ref())
                .page(ListFilter::All, None, None, Some(&ids), 1)?;
            for row in rows.groups.into_iter().flat_map(|group| group.rows) {
                sessions.insert(row.id, row);
            }
        }
        let mut rows = Vec::with_capacity(page.products.len());
        for (product, session_id) in page.products.into_iter().zip(matched) {
            let commerce_id = self.provider.commerce_id_of(&product.product_id);
            let session = session_id.and_then(|id| sessions.get(&id).cloned());
            let picture_path = match (&session, &commerce_id, product.thumb_url.as_deref()) {
                (None, Some(external_id), Some(url))
                    if url.starts_with("https://") && self.commerce.is_some() =>
                {
                    self.remote.search_picture(external_id, url)?
                }
                _ => None,
            };
            rows.push(LabelRow {
                product_id: product.product_id,
                commerce_id,
                sku: product.sku,
                title: product.title,
                product_type: product.product_type,
                price_minor: product.price_minor,
                state: product.state,
                created_at: product.created_at,
                session,
                picture_path,
            });
        }
        tracing::info!(
            target: LOG,
            "labels.list filter={filter:?} rows={} more={}",
            rows.len(),
            page.next_cursor.is_some()
        );
        Ok(LabelPage {
            rows,
            next_cursor: page.next_cursor,
            counts: page.counts,
            work_from: page.work_from,
        })
    }

    /// MC-L2.16: a picture per checked product from what is on this Mac: the matched session's
    /// row picture (one sessions list with the remote items), else the cached search picture of
    /// its remembered `https://` address (none while its download waits in the queue). No
    /// provider or shop call, nothing downloaded; a failed read leaves the answer empty.
    pub(super) fn check_pictures(
        &self,
        products: &[(String, Option<String>)],
    ) -> HashMap<String, std::path::PathBuf> {
        match self.pictures_of(products) {
            Ok(pictures) => pictures,
            Err(error) => {
                tracing::warn!(target: LOG, %error, "label check pictures not read");
                HashMap::new()
            }
        }
    }

    fn pictures_of(
        &self,
        products: &[(String, Option<String>)],
    ) -> Result<HashMap<String, std::path::PathBuf>> {
        let items = self.match_items()?;
        let matched: Vec<Option<i64>> = products
            .iter()
            .map(|(product_id, sku)| {
                match_product(product_id, sku.as_deref(), &items)
                    .and_then(|i| items[i].0.session_id)
            })
            .collect();
        let mut covers: HashMap<i64, std::path::PathBuf> = HashMap::new();
        if matched.iter().any(Option::is_some) {
            let list = SessionsService::new(self.store.as_ref(), self.clock.as_ref())
                .with_remote(self.remote.as_ref())
                .list(ListFilter::All, None)?;
            for row in list.groups.into_iter().flat_map(|group| group.rows) {
                if let Some(path) = row.cover_preview_path {
                    covers.insert(row.id, std::path::PathBuf::from(path));
                }
            }
        }
        let seen = self.seen().clone();
        let mut pictures = HashMap::new();
        for ((product_id, _), session) in products.iter().zip(matched) {
            if let Some(path) = session.and_then(|id| covers.get(&id)) {
                pictures.insert(product_id.clone(), path.clone());
                continue;
            }
            let url = seen
                .get(product_id)
                .and_then(|row| row.thumb_url.as_deref())
                .filter(|url| url.starts_with("https://"));
            let commerce_id = self.provider.commerce_id_of(product_id);
            if let (Some(url), Some(external_id), Some(_)) = (url, commerce_id, &self.commerce)
                && let Some(path) = self.remote.search_picture(&external_id, url)?
            {
                pictures.insert(product_id.clone(), path);
            }
        }
        Ok(pictures)
    }

    /// MC-L1.6 (owner card 1): the box's number, the remembered `missing_price_tag`. With
    /// `refresh`, when calls are allowed and nothing is remembered, or the remembered answer had
    /// a search, or it is older than `COUNT_FRESH_MS`, one unsearched one-row request is made
    /// first. Not allowed → none, no request.
    pub fn summary(&self, refresh: bool) -> std::result::Result<Option<i64>, LabelsFailure> {
        let Some(store) = self.allowed_store()? else {
            return Ok(None);
        };
        if refresh {
            let stale = match *self.counts() {
                None => true,
                Some(r) => {
                    r.searched
                        || self.clock.mono().saturating_duration_since(r.at)
                            >= Duration::from_millis(COUNT_FRESH_MS)
                }
            };
            if stale {
                self.products(&store, LabelFilter::MissingPriceTag, None, None, 1)?;
            }
        }
        Ok(self.counts().map(|r| r.counts.missing_price_tag))
    }

    /// MC-L1.7: every product of the filter and search with a SKU, in the provider's order,
    /// paged to the end (`SELECT_ALL_PAGE` per request); a cursor seen twice or more than
    /// `SELECT_ALL_MAX_PAGES` pages is a reply this version does not understand. Any error is
    /// that error and nothing else.
    pub fn select_all(
        &self,
        filter: LabelFilter,
        query: Option<&str>,
    ) -> std::result::Result<Vec<Selected>, LabelsFailure> {
        let store = self.gate()?;
        let q = search_text(query);
        let mut selected = Vec::new();
        let mut seen: HashSet<String> = HashSet::new();
        let mut cursor: Option<String> = None;
        let mut pages = 0;
        loop {
            if pages == SELECT_ALL_MAX_PAGES {
                return Err(LabelsFailure::Provider(LabelsError::BadReply(
                    "the label list did not end".into(),
                )));
            }
            pages += 1;
            let page = self.learn(self.provider.products(
                &store,
                filter,
                q.as_deref(),
                cursor.as_deref(),
                SELECT_ALL_PAGE,
            ))?;
            self.remember(&page.products);
            selected.extend(page.products.into_iter().filter_map(|product| {
                product.sku.map(|sku| Selected {
                    product_id: product.product_id,
                    sku,
                })
            }));
            match page.next_cursor {
                None => break,
                Some(next) => {
                    if !seen.insert(next.clone()) {
                        return Err(LabelsFailure::Provider(LabelsError::BadReply(
                            "the label list repeated a cursor".into(),
                        )));
                    }
                    cursor = Some(next);
                }
            }
        }
        Ok(selected)
    }

    /// MC-L1.8: the session a row opens: the matched item's latest session (created when it has
    /// none); with no match, the shop product is adopted (one read, no write) and given a
    /// session. Errors of the adopt pass through and create nothing. Needs no label access.
    pub fn open_product(&self, product_id: &str) -> std::result::Result<i64, LabelsFailure> {
        let items = self.match_items()?;
        let sku = self.seen().get(product_id).and_then(|row| row.sku.clone());
        let sessions = SessionsService::new(self.store.as_ref(), self.clock.as_ref());
        if let Some(i) = match_product(product_id, sku.as_deref(), &items) {
            let item = &items[i].0;
            return Ok(match item.session_id {
                Some(id) => id,
                None => sessions.create(item.item_id)?.session.id,
            });
        }
        let external_id = self
            .provider
            .commerce_id_of(product_id)
            .ok_or_else(|| Error::InvalidValue("not a shop product id".into()))?;
        let item = ItemsService::new(self.store.as_ref(), self.clock.as_ref())
            .with_commerce(self.commerce.as_deref())
            .adopt(&external_id)?;
        Ok(sessions.create(item.id)?.session.id)
    }
}

impl LabelsLink for LabelsService {
    /// ACC.2: one templates read decides the access (`Ok` → allowed, `NotAllowed` → denied, any
    /// other error → unknown); the copy is emptied and the screen told once.
    fn connected(&self) {
        let store = match self.conversations.connection() {
            Ok(c) if c.state == ConnectionState::Ready => c.store.map(|s| s.id),
            _ => None,
        };
        let Some(store) = store else {
            return;
        };
        let access = match self.provider.templates(&store) {
            Ok(_) => LabelsAccess::Allowed,
            Err(LabelsError::NotAllowed) => LabelsAccess::Denied,
            Err(_) => LabelsAccess::Unknown,
        };
        if let Err(error) = self.set_access(access) {
            tracing::warn!(target: LOG, %error, "labels access not stored");
        }
        self.forget();
        self.events.emit(AppEvent::LabelsChanged {
            product_ids: vec![],
            access: true,
        });
    }

    /// ACC.3: the old key's answer is forgotten.
    fn key_changed(&self) {
        if let Err(error) = self.set_access(LabelsAccess::Unknown) {
            tracing::warn!(target: LOG, %error, "labels access not stored");
        }
        self.forget();
    }

    /// MC-L4.3: each change applied in order, then one event naming the distinct products.
    fn changed(&self, changes: &[LabelChange]) {
        let mut product_ids: Vec<String> = Vec::new();
        {
            let mut copy = self.copy();
            for change in changes {
                copy.apply_event(change);
                if !product_ids.contains(&change.product_id) {
                    product_ids.push(change.product_id.clone());
                }
            }
        }
        if product_ids.is_empty() {
            return;
        }
        tracing::info!(target: LOG, "labels.events changed={}", product_ids.len());
        self.events.emit(AppEvent::LabelsChanged {
            product_ids,
            access: false,
        });
    }
}
