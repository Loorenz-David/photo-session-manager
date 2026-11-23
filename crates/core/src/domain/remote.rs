//! Remote items (book 10H §3): items whose pictures are only on the commerce platform, and the
//! platform's call budget (10H §8). Core vocabulary only: no platform name here (H7).

use serde::{Deserialize, Serialize};

/// The platform's call budget as last read and projected to now (MC-H5.2): the budget available
/// and the most the budget holds.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CallBudget {
    pub available: f64,
    pub maximum: f64,
}

/// What a downloaded picture is for (master plan archive §4 Events): a row's cover, a shop
/// search result's picture, or one picture of a remote item's view (H4, MC-H3.5).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PictureTarget {
    Cover {
        session_id: i64,
    },
    Search {
        external_id: String,
    },
    Remote {
        session_id: i64,
        role: String,
        position: i64,
    },
}

/// One remote picture as the workspace shows it (MC-H3.5): its role (`gallery` / `condition`),
/// its position in that role, its media id, its 1600 px cache file when on this Mac, and whether
/// the shop no longer has it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemotePictureView {
    pub role: String,
    pub position: i64,
    pub media_id: String,
    pub path: Option<std::path::PathBuf>,
    pub gone: bool,
}

/// What the workspace is given for a session's item (MC-H3.5).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RemoteView {
    pub item_id: i64,
    pub state: RemoteState,
    pub keep_files: bool,
    pub pictures: Vec<RemotePictureView>,
    pub bring_back_bytes: i64,
    /// The last first-open read of this item failed (MC-H3.2).
    pub check_failed: bool,
    /// The running bring back's progress (H3), if one runs.
    pub bring_back: Option<BringBackProgress>,
}

/// The two kinds of remote item (10H §3): files cleared on this Mac (`Archived`), or a shop
/// product never photographed on this Mac (`ShopOnly`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RemoteKind {
    Archived,
    ShopOnly,
}

/// Where an item's pictures are (MC-H3.1). Wire form: `local`, `unknown`, `archived`,
/// `shop_only`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteState {
    /// On this Mac (or no integration).
    Local,
    /// An adopted shop product not yet read (its first open reads it).
    Unknown,
    Remote(RemoteKind),
}

impl RemoteState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Unknown => "unknown",
            Self::Remote(RemoteKind::Archived) => "archived",
            Self::Remote(RemoteKind::ShopOnly) => "shop_only",
        }
    }
}

impl Serialize for RemoteState {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for RemoteState {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        match text.as_str() {
            "local" => Ok(Self::Local),
            "unknown" => Ok(Self::Unknown),
            "archived" => Ok(Self::Remote(RemoteKind::Archived)),
            "shop_only" => Ok(Self::Remote(RemoteKind::ShopOnly)),
            other => Err(serde::de::Error::unknown_variant(
                other,
                &["local", "unknown", "archived", "shop_only"],
            )),
        }
    }
}

/// How far a bring back is (master plan archive §4 Vocabulary; MC-H4.3): `done` of `total`
/// pictures are on this Mac.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BringBackProgress {
    pub done: usize,
    pub total: usize,
}

/// Where a bring back is (MC-H4.3, .5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BringBackState {
    Running,
    Done,
    Failed,
}

/// What a finished bring back did (MC-H4.5): the new photos, and whether the editor was asked
/// to list the folder.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BringBackOutcome {
    pub photo_ids: Vec<i64>,
    pub synced: bool,
}

/// One downloaded picture of a bring back (MC-H4.4(b), (c); H3 projection r0 L12).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BroughtBack {
    /// `gallery` (main) or `condition`, as `ArchivedImage::role`.
    pub role: String,
    /// 1-based order within the role.
    pub position: i64,
    pub media_id: String,
    /// The file's canonical path.
    pub path: String,
    /// The file's SHA-256 (hex).
    pub sha256: String,
}

/// The import of a bring back, one transaction (`Store::bring_back`, MC-H4.4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BringBackInput {
    pub session_id: i64,
    pub item_id: i64,
    /// Gallery then condition, each in position order.
    pub files: Vec<BroughtBack>,
    /// The link role of a condition photo, `attribute:<condition photos key>`.
    pub link_role_condition: String,
    pub at: i64,
}
