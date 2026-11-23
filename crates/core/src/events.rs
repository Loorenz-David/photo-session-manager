use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AppEvent {
    ResearchChanged {
        session_id: i64,
    },
    HistoryChanged {
        session_id: i64,
        undo: Option<String>,
        redo: Option<String>,
    },
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
        status: crate::domain::camera::CameraStatus,
    },
    FingerprintPending {
        fingerprint: String,
        replace: bool,
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
    /// H2 (MC-H1.3, MC-H2.5): a picture was downloaded and kept on this Mac at `path`.
    PictureReady {
        target: crate::domain::remote::PictureTarget,
        path: String,
    },
    /// C1 (`master_plan_conversations.md` §4 Event): the copy, a read mark or the connection
    /// changed; `notices` = the conversations a completed check found news in.
    ConversationsChanged {
        notices: Vec<crate::domain::conversation::ConversationNotice>,
        chime: bool,
        banner: bool,
    },
}

/// `export_progress.state` (master plan §6.7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportState {
    Running,
    /// Changelog 45 (P65): a render waits for Capture One (`Waiting for Capture One…`).
    Waiting,
    Done,
    Failed,
}
