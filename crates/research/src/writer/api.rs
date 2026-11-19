//! The writer's app-facing surface (`master_plan_writer.md` §4 "API"). The wiring hands the same
//! `ResearchService` as `Arc<dyn WriterApi>` beside `Arc<dyn ResearchApi>`.

use psm_core::Result;
use serde::{Deserialize, Serialize};

use crate::{api::Reason, writer::note::NoteView};

/// The answer to a request that may queue a run: whether it did, and why not.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Requested {
    pub queued: bool,
    pub reason: Option<Reason>,
}

pub trait WriterApi: Send + Sync {
    /// W1: the session's product note as the owner's decisions show it, and its staleness.
    fn note(&self, session_id: i64) -> Result<NoteView>;
    /// W1 (WC1.8): **Look again**: queue a `look` run when the photos changed.
    fn look_again(&self, session_id: i64) -> Result<Requested>;
    /// W2 (WC3): queue a writer run of the session, or say why not.
    fn request(&self, session_id: i64, origin: crate::writer::run::Origin) -> Result<Requested>;
}
