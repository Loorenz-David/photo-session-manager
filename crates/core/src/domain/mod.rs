use serde::{Deserialize, Serialize};

pub const MECHANISMS_VERSION: &str = "1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthItem {
    pub code: String,
    pub ok: bool,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    pub items: Vec<HealthItem>,
}
