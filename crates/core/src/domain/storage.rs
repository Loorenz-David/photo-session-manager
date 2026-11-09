//! Settings → Storage (10F §4).

use serde::{Deserialize, Serialize};

/// `settings.json` key `storage` (10F §4): the minimum free space, in GB (10⁹ bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct StorageSettings {
    pub min_free_gb: u32,
}
impl Default for StorageSettings {
    fn default() -> Self {
        Self { min_free_gb: 20 }
    }
}
impl StorageSettings {
    pub fn min_free_bytes(&self) -> u64 {
        u64::from(self.min_free_gb) * 1_000_000_000
    }
}

/// The space each kind of file uses, computed in the background (10F §4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageReport {
    /// Free space on the disk holding the capture root (`None` when it could not be read).
    pub free_bytes: Option<u64>,
    pub disk: String,
    pub raws_bytes: u64,
    pub duplicates_bytes: u64,
    pub preview_exports_bytes: u64,
    pub preview_cache_bytes: u64,
    pub masters_bytes: u64,
    pub computed_at_ms: i64,
}
