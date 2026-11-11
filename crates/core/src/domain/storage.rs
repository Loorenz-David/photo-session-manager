//! Settings → Storage (10F §4).

use serde::{Deserialize, Serialize};

/// `settings.json` key `storage` (10F §3, §4): the minimum free space, in GB (10⁹ bytes), and
/// the days after publish an item's local files are kept (P62).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct StorageSettings {
    pub min_free_gb: u32,
    pub retention_days_after_publish: u32,
}
impl Default for StorageSettings {
    fn default() -> Self {
        Self {
            min_free_gb: 20,
            retention_days_after_publish: 4,
        }
    }
}
impl StorageSettings {
    pub fn min_free_bytes(&self) -> u64 {
        u64::from(self.min_free_gb) * 1_000_000_000
    }
}

/// An item's retention state (10F §3, Q2; P62): the "keep files on this Mac" pin, and when its
/// local files were cleared after publishing (then the item is read-only, Q3).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemRetention {
    pub keep_files: bool,
    pub cleared_at: Option<i64>,
}

/// One published image of a cleared item, as Shopify held it when the item was verified
/// (10F §3: the item's photos are shown from Shopify after clearing).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArchivedImage {
    /// `gallery` (main photos) or `condition`.
    pub role: String,
    /// 1-based order within the role.
    pub position: i64,
    pub media_id: String,
    pub url: String,
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

/// A photo denied since P66 (changelog 46 (2)): when it was denied and, once the undo window
/// closed, when its RAW and the app's files of it were deleted (it cannot be restored after that).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhotoDenial {
    pub photo_id: i64,
    pub denied_at: i64,
    pub files_deleted_at: Option<i64>,
}
