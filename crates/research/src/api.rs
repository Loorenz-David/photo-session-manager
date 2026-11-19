//! The app-facing research surface. The engine is wired in a later milestone.

use std::path::{Path, PathBuf};

use psm_core::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Reason {
    pub code: String,
    pub sentence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepView {
    pub seq: i64,
    pub stage: String,
    pub kind: String,
    pub summary: String,
    pub detail: String,
    pub source_url: Option<String>,
    pub source_title: Option<String>,
    pub photo_ids: Vec<i64>,
    pub field_keys: Vec<String>,
    /// A `read` step whose page could not be read (owner correction A7): shown as a warning.
    pub unread: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunView {
    pub id: i64,
    pub state: String,
    pub trigger: String,
    pub stage_reached: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchStatus {
    pub reason: Option<Reason>,
    pub run: Option<RunView>,
    pub steps: Vec<StepView>,
    pub fields_filled: usize,
    pub to_review: usize,
    pub can_run: bool,
    /// With the `WaitingForEditor` reason: when the run's wait for Capture One's previews ends
    /// (UTC ms; R7·U1). The panel says "Waiting for Capture One" from then on.
    pub wait_ends_at: Option<i64>,
    /// A6: the latest run is queued: its place in the line and the session being researched.
    pub queue: Option<QueueView>,
}

/// A6: where a queued run stands. Research runs one session at a time (MC-A6).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueueView {
    /// 1 = the next to start, in MC-A6 start order (`manual` first, then oldest).
    pub position: usize,
    /// The session whose run is running now, if any.
    pub running_session_id: Option<i64>,
    /// Its name as the sessions list shows it (item type, or the title when there is none) and SKU.
    pub running_name: Option<String>,
    /// The run was asked for by hand (Research / Run again), so it goes before arranged ones.
    pub manual: bool,
}

/// A6: one session's research state for the sessions list and the session header.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionResearch {
    pub session_id: i64,
    /// `researching` · `waiting` · `to_review` · `done` · `failed` · `blocked`.
    pub state: String,
    /// With `waiting`: the place in the line (1 = next).
    pub position: Option<usize>,
    /// Live unchecked AI suggestions of the item (as the form's "n to review").
    pub to_review: usize,
    /// With `blocked`: why research cannot run (the first failing prerequisite).
    pub reason: Option<Reason>,
    /// A24: the session's latest writer run, when it waits or writes (`writing`) or ended
    /// `failed`, `done` or `stopped`. A session with a writer run and no research run is listed
    /// with state `none`.
    pub writer: Option<WriterOverview>,
}

/// A24: the writer's state of one session in the overview, and its item's SKU (for the toast).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WriterOverview {
    /// `writing` · `done` · `stopped` · `failed`.
    pub state: String,
    pub sku: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SecretsInput {
    pub model_key: Option<String>,
    pub search_key: Option<String>,
    pub reader_key: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SecretsState {
    pub model: bool,
    pub search: bool,
    pub reader: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportSelection {
    Range { from: i64, to: i64 },
    Items(Vec<i64>),
}

pub trait ResearchApi: Send + Sync {
    fn status(&self, session_id: i64) -> Result<ResearchStatus>;
    /// A6: the research state of every session that has one (or of `session` only), in one call.
    fn overview(&self, session: Option<i64>) -> Result<Vec<SessionResearch>>;
    fn run(&self, session_id: i64) -> Result<()>;
    fn session_changed(&self, session_id: i64);
    fn arranged(&self, session_id: i64) -> Result<bool>;
    fn settings_changed(&self);
    fn settings_reset(&self) -> Result<Json>;
    fn validate_settings(&self, settings: &Json) -> Result<()>;
    fn secrets_set(&self, input: SecretsInput) -> Result<()>;
    fn secrets_state(&self) -> Result<SecretsState>;
    fn site_add(&self, site: &str) -> Result<Json>;
    fn site_remove(&self, site: &str) -> Result<Json>;
    fn sites_reorder(&self, sites: &[String]) -> Result<Json>;
    fn export(&self, selection: ExportSelection, dest: &Path) -> Result<PathBuf>;
    fn purge_expired(&self, now_ms: i64) -> Result<()>;
    /// A12: the visual matches research already found for the session (no new call).
    fn browse_matches(&self, session_id: i64) -> Result<Vec<crate::browse::VisualMatch>>;
    /// A12: one new visual search of a photo of the session (a counted research call).
    fn browse_search(
        &self,
        session_id: i64,
        photo_id: i64,
    ) -> Result<Vec<crate::browse::VisualMatch>>;
    /// A12: "Ask AI…" on selected page text; the answer becomes AI suggestions.
    fn browse_ask(&self, request: &crate::browse::AskRequest) -> Result<crate::browse::AskAnswer>;
}
