use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AppEvent {
    PhotoAdded {
        photo_id: i64,
        session_id: i64,
    },
    PhotoChanged {
        photo_id: i64,
        session_id: i64,
    },
    SessionChanged {
        session_id: i64,
    },
    HealthChanged,
    CameraStatusChanged {
        status: crate::domain::editing::CameraStatus,
    },
    ExportProgress {
        session_id: i64,
        done: usize,
        total: usize,
        state: ExportState,
    },
    PublishProgress {
        session_id: i64,
        item_id: i64,
        step: crate::domain::session::PublishStep,
        uploaded: i64,
        total: i64,
    },
}

/// `export_progress.state` (master plan §6.7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportState {
    Running,
    Done,
    Failed,
}
