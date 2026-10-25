use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AppEvent {
    PhotoAdded {
        photo_id: i64,
        session_id: i64,
    },
    HealthChanged,
    CameraStatusChanged {
        status: crate::domain::editing::CameraStatus,
    },
}
