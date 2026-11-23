//! The labels wiring (book 09C; `master_plan_labels.md` §4 Service): builds `LabelsService` over
//! the Beyo Pricing adapter the conversations service already holds, and tells the conversations
//! service where to send what it learns (`set_labels_link`, a `Weak`: no cycle).

use psm_core::{
    ports::{
        clock::Clock,
        events::EventSink,
        labels::{LabelsLink, LabelsProvider},
    },
    services::{conversations::ConversationsService, labels::LabelsService},
};
use psm_infra::sqlite::store::SqliteStore;
use std::sync::{Arc, Weak};

/// The pieces `App::build` hands the labels wiring.
pub struct ServiceParts {
    pub conversations: Arc<ConversationsService>,
    /// The same adapter instance as the conversations service's.
    pub provider: Arc<dyn LabelsProvider>,
    pub store: Arc<SqliteStore>,
    pub clock: Arc<dyn Clock>,
    pub events: Arc<dyn EventSink>,
}

pub fn build_service(parts: ServiceParts) -> Arc<LabelsService> {
    let labels = Arc::new(LabelsService::new(
        parts.conversations.clone(),
        parts.provider,
        parts.store,
        parts.clock,
        parts.events,
    ));
    let weak: Weak<LabelsService> = Arc::downgrade(&labels);
    let link: Weak<dyn LabelsLink> = weak;
    parts.conversations.set_labels_link(link);
    labels
}
