//! Publish vocabulary shared by the publish service and commerce adapters (07A MC-10…MC-14).
//! Everything here is platform-neutral (H7): remote ids are opaque strings, and a definition's
//! platform data travels as an opaque `DefinitionLink`.

use crate::{Error, Result, ports::store::Store};
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;

/// The mutation token of 07A MC-10. Fields are private and there is exactly one constructor,
/// `from_persisted`, which only succeeds for a `publish_runs` row committed with `confirmed_at`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfirmedRun {
    run_id: i64,
    confirmed_at: i64,
}

impl ConfirmedRun {
    /// `Err(not_confirmed)` unless the run exists and has `confirmed_at`.
    pub fn from_persisted(store: &dyn Store, run_id: i64) -> Result<ConfirmedRun> {
        match store.confirmed_run(run_id)? {
            Some(confirmed_at) => Ok(ConfirmedRun {
                run_id,
                confirmed_at,
            }),
            None => Err(Error::NotConfirmed),
        }
    }

    pub fn run_id(&self) -> i64 {
        self.run_id
    }

    pub fn confirmed_at(&self) -> i64 {
        self.confirmed_at
    }
}

/// Processing state of one remote media or file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RemoteMediaStatus {
    Uploaded,
    Processing,
    Ready,
    Failed,
}

/// One gallery media of a remote item. `position` is 1-based gallery order. `filename` is the
/// name the platform stores (it may carry a uniquifier); `None` while the platform has not
/// finished processing the media and does not expose it yet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteMedia {
    pub id: String,
    pub filename: Option<String>,
    pub status: RemoteMediaStatus,
    pub position: i64,
}

/// A remote item (Shopify: a product with its default variant). `item_search` may leave `media`
/// empty; `item_get` fills it in gallery order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteItem {
    pub external_id: String,
    pub title: String,
    pub sku: Option<String>,
    pub barcode: Option<String>,
    pub thumbnail_url: Option<String>,
    pub media: Vec<RemoteMedia>,
}

/// Identity of a remote item's default variant, as read back after a write (05B MC-22).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteIdentity {
    pub sku: String,
    pub barcode: Option<String>,
}

/// Adapter-owned data of one definition (table `attribute_definition_links`); opaque to core.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DefinitionLink {
    pub platform_type: String,
    pub platform_meta: Json,
}

/// One value to write: the core key, its link, and the canonical value (05A MC-9). For a
/// `media_list` key the value is the JSON array of remote file ids to set (07A MC-14).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WriteValue {
    pub key: String,
    pub link: DefinitionLink,
    pub value: Json,
}

/// A staged-upload target returned by the platform: where to PUT the bytes (`url`, with the
/// `parameters` sent as request headers) and the `resource_url` to reference afterwards.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StagedTarget {
    pub filename: String,
    pub url: String,
    pub resource_url: String,
    pub parameters: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublishPreviewItem {
    pub session_id: i64,
    pub main: Vec<i64>,
    pub condition: Vec<i64>,
    pub main_thumbnails: Vec<Option<String>>,
    pub condition_thumbnails: Vec<Option<String>>,
    pub details_filled: usize,
    pub details_total: usize,
    pub unreviewed: usize,
    pub announced_removals: Vec<String>,
    pub create_item: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublishPreview {
    pub items: Vec<PublishPreviewItem>,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublishItem {
    pub id: i64,
    pub run_id: i64,
    pub session_id: i64,
    pub create_item: bool,
    pub title: String,
    pub frozen_main: Vec<i64>,
    pub frozen_condition: Vec<i64>,
    pub frozen_attributes: std::collections::BTreeMap<String, Json>,
    pub announced_removals: Vec<String>,
    pub confirmed_change_seq: i64,
    pub step: crate::domain::session::PublishStep,
    pub failed_step: Option<crate::domain::session::PublishStep>,
    pub uploaded: i64,
    pub total: i64,
    pub error: Option<String>,
    pub export_run_id: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhotoLink {
    pub photo_id: i64,
    pub item_id: i64,
    pub role: String,
    pub external_id: Option<String>,
    pub upload_state: String,
    pub export_sha256: Option<String>,
    pub superseded_ids: Vec<String>,
    pub revision: i64,
    pub retouch_gen: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Notification {
    pub id: i64,
    pub item_id: i64,
    pub session_id: i64,
    pub run_id: i64,
    pub kind: String,
    pub text: String,
    pub created_at: i64,
}
