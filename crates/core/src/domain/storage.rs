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

/// `settings.json` key `export` (changelog 47 (2), P69): whether masters are prepared in the
/// background once a session is Ready (Settings → Storage; default on).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ExportSettings {
    pub prepare_masters: bool,
}
impl Default for ExportSettings {
    fn default() -> Self {
        Self {
            prepare_masters: true,
        }
    }
}

/// An item's retention state (10F §3, Q2; P62): the "keep files on this Mac" pin, and when its
/// local files were cleared after publishing (then the item is read-only, Q3).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemRetention {
    pub keep_files: bool,
    pub cleared_at: Option<i64>,
    /// When the item's pictures were last brought back to this Mac (10H §7, MC-H4.8).
    pub restored_at: Option<i64>,
    /// When the item's remote pictures were last read from the shop (MC-H3.2).
    pub remote_checked_at: Option<i64>,
}

/// One remote picture of an item (10H §3): a published image of a cleared item as the shop held
/// it when the item was verified (10F §3), or a picture of a shop-only item read at its first
/// open (MC-H3.2). An empty `url` means the shop no longer has it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArchivedImage {
    /// `gallery` (main photos) or `condition`.
    pub role: String,
    /// 1-based order within the role.
    pub position: i64,
    pub media_id: String,
    pub url: String,
    /// The original file's size in bytes, when the shop told it (MC-H4.1).
    pub bytes: Option<i64>,
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

/// What "Delete session" removed from the database (owner correction A17).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeletedRows {
    /// The session's photo rows.
    pub photos: i64,
    /// The session's item row (an item has one session).
    pub item: bool,
    /// Research runs of the session (with their steps, calls, outbound and stage-output rows).
    pub research_runs: i64,
}
