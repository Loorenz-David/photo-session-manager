//! The app-facing research surface. The engine is wired in a later milestone.

use std::path::{Path, PathBuf};

use psm_core::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    fn run(&self, session_id: i64) -> Result<()>;
    fn session_changed(&self, session_id: i64);
    fn arranged(&self, session_id: i64) -> Result<bool>;
    fn settings_changed(&self);
    fn validate_settings(&self, settings: &Json) -> Result<()>;
    fn secrets_set(&self, input: SecretsInput) -> Result<()>;
    fn secrets_state(&self) -> Result<SecretsState>;
    fn site_add(&self, site: &str) -> Result<Json>;
    fn site_remove(&self, site: &str) -> Result<Json>;
    fn sites_reorder(&self, sites: &[String]) -> Result<Json>;
    fn export(&self, selection: ExportSelection, dest: &Path) -> Result<PathBuf>;
    fn purge_expired(&self, now_ms: i64) -> Result<()>;
}
