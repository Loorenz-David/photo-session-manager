//! `LabelsService` (book 09C §4, §7, §8; `master_plan_labels.md` §4 Service, §5 ACC, MC-L3,
//! MC-L4): whether label calls may be made, the display copy of label state, the status read of
//! session markers, and the label changes the conversations pull hands over. Platform-neutral
//! (H7): the provider is a port, its sentences come from `describe`, its id forms from
//! `product_id_of`. Nothing about label state is stored (M-L1): the copy lives in memory; the
//! only stored fact is whether the saved key may read labels. Nothing here starts a thread or a
//! timer.

use crate::{
    Error, Result,
    domain::{
        conversation::ConnectionState,
        label::{LabelChange, LabelState, LabelsAccess, LabelsError, SessionLabel},
    },
    events::AppEvent,
    mechanisms::labels::{LabelCopy, STATUS_CHUNK, STATUS_FRESH_MS, chunks},
    ports::{
        clock::Clock,
        events::EventSink,
        labels::{LabelStore, LabelsLink, LabelsProvider, LabelsResult},
    },
    services::conversations::ConversationsService,
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, MutexGuard},
    time::Duration,
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

pub struct LabelsService {
    conversations: Arc<ConversationsService>,
    provider: Arc<dyn LabelsProvider>,
    labels: Arc<dyn LabelStore>,
    clock: Arc<dyn Clock>,
    events: Arc<dyn EventSink>,
    copy: Mutex<LabelCopy>,
    /// The stored access as last read or written (`None` = not read yet in this build).
    access: Mutex<Option<LabelsAccess>>,
}

impl LabelsService {
    pub fn new(
        conversations: Arc<ConversationsService>,
        provider: Arc<dyn LabelsProvider>,
        labels: Arc<dyn LabelStore>,
        clock: Arc<dyn Clock>,
        events: Arc<dyn EventSink>,
    ) -> Self {
        Self {
            conversations,
            provider,
            labels,
            clock,
            events,
            copy: Mutex::new(LabelCopy::new()),
            access: Mutex::new(None),
        }
    }

    fn copy(&self) -> MutexGuard<'_, LabelCopy> {
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

    fn describe_error(&self, error: &LabelsError) -> String {
        let shop = self.conversations.connection().ok().and_then(|c| c.shop);
        self.provider.describe(error, shop.as_deref())
    }

    /// ACC.4, learning by use, for every label call but `connected()`'s own: a success while the
    /// access is unknown makes it allowed; `NotAllowed` makes it denied, empties the copy and
    /// tells the screen. Never touches the conversations connection (ACC.5).
    fn learn<T>(&self, result: LabelsResult<T>) -> std::result::Result<T, LabelsFailure> {
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
        self.copy().clear();
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
        self.copy().clear();
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
