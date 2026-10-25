use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AppEvent {
    HealthChanged,
    CameraStatusChanged {
        status: crate::domain::editing::CameraStatus,
    },
}
