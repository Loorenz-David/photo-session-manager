//! The commerce integration port (ch09 capabilities). Every method that mutates the platform
//! takes a `&ConfirmedRun` (07A MC-10): without a confirmed, persisted run there is no way to
//! call it. Adapters translate platform shapes into the core vocabulary (H7).

use crate::Result;
use crate::domain::attribute::Definition;
use crate::domain::publish::{
    ConfirmedRun, DefinitionLink, RemoteIdentity, RemoteItem, RemoteMedia, StagedTarget, WriteValue,
};
use serde_json::Value as Json;

/// The nine capabilities of ch09 §Capabilities.
pub const ALL_CAPABILITIES: [&str; 9] = [
    "item.search",
    "item.get",
    "item.create",
    "attributes.schema",
    "attributes.read",
    "attributes.write",
    "media.publish",
    "media.attribute_publish",
    "description.publish",
];

pub trait CommerceAdapter: Send + Sync {
    fn capabilities(&self) -> Vec<&'static str>;
    fn condition_photos_key(&self) -> &str;

    /// Items whose default variant SKU equals `sku` byte for byte after NFC (07A MC-11).
    fn item_search(&self, sku: &str) -> Result<Vec<RemoteItem>>;
    /// Read-only SKU or title search for the New session dialog.
    fn item_find(&self, query: &str) -> Result<Vec<RemoteItem>>;
    /// The item's identity and its gallery media in order.
    fn item_get(&self, external_id: &str) -> Result<RemoteItem>;
    /// Current product fields used for the confirmed description and three-way tag merge.
    fn product_fields(&self, external_id: &str) -> Result<(Vec<String>, Option<String>)>;
    /// Creates a draft item whose default variant carries `sku` and `barcode`; returns its id.
    fn item_create(
        &self,
        run: &ConfirmedRun,
        sku: &str,
        barcode: Option<&str>,
        title: &str,
    ) -> Result<String>;
    /// Writes only the identity fields that differ (a `None` barcode is never sent) and returns
    /// the read-back (05B MC-22).
    fn item_set_identity(
        &self,
        run: &ConfirmedRun,
        external_id: &str,
        sku: &str,
        barcode: Option<&str>,
    ) -> Result<RemoteIdentity>;

    /// The platform's definitions translated to core definitions, each with its link.
    fn attributes_schema(&self) -> Result<Vec<(Definition, DefinitionLink)>>;
    /// The item's current values as raw core-shaped JSON (to be canonicalized by the caller). A
    /// value the core cannot represent is returned verbatim as the platform's JSON, which the
    /// caller's canonicalization rejects, so it is stored `valid = false` (05A MC-9).
    fn attributes_read(&self, external_id: &str) -> Result<Vec<(String, Json)>>;
    /// Writes the values, batched at the platform's limit.
    fn attributes_write(
        &self,
        run: &ConfirmedRun,
        external_id: &str,
        values: &[WriteValue],
    ) -> Result<()>;

    /// Reserves an upload target for the photo's upload filename (07A MC-11, 07B MC-21).
    fn staged_uploads_create(
        &self,
        run: &ConfirmedRun,
        folder_key: &str,
        uid: &str,
        revision: i64,
        byte_size: u64,
    ) -> Result<StagedTarget>;
    fn upload(&self, run: &ConfirmedRun, target: &StagedTarget, bytes: &[u8]) -> Result<()>;
    /// Attaches the uploaded image to the item's gallery; returns the new media id.
    fn product_create_media(
        &self,
        run: &ConfirmedRun,
        external_id: &str,
        resource_url: &str,
        alt: &str,
    ) -> Result<String>;
    fn poll_media_ready(&self, media_id: &str) -> Result<()>;
    /// One reorder request with a move for every id, `newPosition` = index in `media_ids`;
    /// returns the job id (07A MC-12 step 4).
    fn product_reorder_media(
        &self,
        run: &ConfirmedRun,
        external_id: &str,
        media_ids: &[String],
    ) -> Result<String>;
    fn poll_job(&self, job_id: &str) -> Result<()>;
    fn list_media(&self, external_id: &str) -> Result<Vec<RemoteMedia>>;
    fn list_files_by_name(&self, stem: &str) -> Result<Vec<RemoteMedia>>;
    /// Removes the media from the item; a not-found reply counts as removed (MC-12 step 3).
    fn product_delete_media(
        &self,
        run: &ConfirmedRun,
        external_id: &str,
        media_ids: &[String],
    ) -> Result<()>;
    /// Creates a shop file from an uploaded resource; returns the file id (07A MC-14).
    fn file_create(&self, run: &ConfirmedRun, resource_url: &str, filename: &str)
    -> Result<String>;
    fn poll_file(&self, file_id: &str) -> Result<()>;

    fn update_title(&self, run: &ConfirmedRun, external_id: &str, title: &str) -> Result<()>;
    /// Sends the description (blank-line-separated paragraphs) as HTML (07A MC-13).
    fn update_description(
        &self,
        run: &ConfirmedRun,
        external_id: &str,
        paragraphs: &str,
    ) -> Result<()>;
    /// Sends exactly `tags` (the three-way merge is the caller's, P14).
    fn update_tags(&self, run: &ConfirmedRun, external_id: &str, tags: &[String]) -> Result<()>;
}
