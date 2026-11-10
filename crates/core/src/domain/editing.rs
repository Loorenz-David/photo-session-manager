use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct ProvisionInputs {
    pub style_name: String,
    pub wait: Duration,
}

/// The styles the editor lists for Settings → Style (P63, owner 2026-11-10): the photographer's own
/// styles first, then the built-in ones.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EditorStyles {
    pub user: Vec<String>,
    pub built_in: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StyleOutcome {
    Applied,
    NotListed,
    NotRequested,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProvisionReport {
    pub document_created: bool,
    pub recipes_created: Vec<String>,
    pub style: StyleOutcome,
    /// A step was refused because "Take the camera back" holds the editor-launch gate (10E MC-34
    /// R-I32): the ensure stopped there, without running that step's script or writing its
    /// health item.
    pub deferred: bool,
}
impl ProvisionReport {
    /// The report of an ensure the editor-launch gate deferred.
    pub fn deferred() -> Self {
        Self {
            document_created: false,
            recipes_created: Vec::new(),
            style: StyleOutcome::NotRequested,
            deferred: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentState {
    pub capture_root: String,
    pub capture_folder: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RenderPurpose {
    Preview,
    /// The full-size, uncropped 16-bit master (10D MC-28) that every output is rendered from.
    Master,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditingCapabilities {
    pub capture_folder: bool,
    pub hooks: bool,
    pub render: bool,
    pub mirror_tag: bool,
    pub import: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Job {
    pub id: i64,
    pub kind: String,
    pub target: String,
    pub payload: String,
    pub status: String,
    pub attempts: i64,
    pub next_attempt_at: Option<i64>,
    pub error: Option<String>,
}
