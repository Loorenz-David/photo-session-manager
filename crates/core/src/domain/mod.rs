use serde::{Deserialize, Serialize};

pub const MECHANISMS_VERSION: &str = "1";
pub mod attribute;
pub mod camera;
pub mod composition;
pub mod editing;
pub mod item;
pub mod output;
pub mod photo;
pub mod publish;
pub mod session;
pub mod storage;

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
