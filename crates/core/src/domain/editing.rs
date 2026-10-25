use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentState {
    pub capture_root: String,
    pub capture_folder: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CameraStatus {
    Connected(String),
    Disconnected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CameraCapabilities {
    pub status: bool,
    pub capture: bool,
    pub transfer: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderPurpose {
    Preview,
    Export,
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
