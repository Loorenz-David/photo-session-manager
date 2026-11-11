//! The commerce integration port (ch09 capabilities). Every method that mutates the platform
//! takes a `&ConfirmedRun` (07A MC-10): without a confirmed, persisted run there is no way to
//! call it. Adapters translate platform shapes into the core vocabulary (H7).

use crate::Result;
use crate::domain::attribute::Definition;
use crate::domain::publish::{
    ConfirmedRun, DefinitionLink, MediaReadback, RemoteIdentity, RemoteItem, RemoteMedia,
    StagedTarget, StockChoice, StockLocations, StockOutcome, WriteValue,
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
    fn check_token(&self) -> Result<()>;
    fn capabilities(&self) -> Vec<&'static str>;
    fn condition_photos_key(&self) -> &str;

    /// Items whose default variant SKU equals `sku` byte for byte after NFC (07A MC-11).
    fn item_search(&self, sku: &str) -> Result<Vec<RemoteItem>>;
    /// Read-only SKU or title search for the New session dialog.
    fn item_find(&self, query: &str) -> Result<Vec<RemoteItem>>;
    /// The item's identity and its gallery media in order.
    fn item_get(&self, external_id: &str) -> Result<RemoteItem>;
    fn item_types(&self) -> Result<Vec<String>>;
    /// Current product fields used for the confirmed description and three-way tag merge.
    fn product_fields(&self, external_id: &str) -> Result<(Vec<String>, Option<String>)>;
    /// The exact description representation stored by this platform.
    fn render_description(&self, paragraphs: &str) -> String;
    /// Creates a draft item whose default variant carries `sku` and `barcode`; returns its id.
    fn item_create(
        &self,
        run: &ConfirmedRun,
        sku: &str,
        barcode: Option<&str>,
        title: &str,
        item_type: &str,
    ) -> Result<String>;
    /// Changelog 45 (P65): the platform's active stock locations, read-only. A platform app
    /// without the permission answers `missing_access` (never an error).
    fn stock_locations(&self) -> Result<StockLocations> {
        Ok(StockLocations::default())
    }
    /// Changelog 45 (P65): creates the draft item like `item_create`, with stock tracking on and
    /// `stock.quantity` available at `stock.location_id`, in the same request, then reads it back.
    /// A platform that refuses the stock fields gets the item created untracked (never a failed
    /// create because of stock).
    #[allow(clippy::too_many_arguments)]
    fn item_create_stocked(
        &self,
        run: &ConfirmedRun,
        sku: &str,
        barcode: Option<&str>,
        title: &str,
        item_type: &str,
        stock: &StockChoice,
    ) -> Result<(String, StockOutcome)> {
        let _ = stock;
        self.item_create(run, sku, barcode, title, item_type)
            .map(|id| {
                (
                    id,
                    StockOutcome::Untracked {
                        reason: "stock is not supported by this platform".into(),
                    },
                )
            })
    }
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
    /// Reads the configured condition-photo file ids for post-write verification.
    fn condition_photos_readback(&self, external_id: &str) -> Result<Option<Json>>;
    /// Writes the values, batched at the platform's limit.
    fn attributes_write(
        &self,
        run: &ConfirmedRun,
        external_id: &str,
        values: &[WriteValue],
    ) -> Result<()>;

    /// Reserves an upload target named `<stem>.jpg` (07A MC-11, 07B MC-21, 10D MC-29); core
    /// builds the stem with `mechanisms::upload_name::stem`.
    fn staged_uploads_create(
        &self,
        run: &ConfirmedRun,
        stem: &str,
        byte_size: u64,
    ) -> Result<StagedTarget>;
    /// P64: one target per `(stem, byte_size)`, in order, reserved in as few platform requests
    /// as the platform allows (the adapter chunks at its limit).
    fn staged_uploads_create_many(
        &self,
        run: &ConfirmedRun,
        files: &[(String, u64)],
    ) -> Result<Vec<StagedTarget>> {
        files
            .iter()
            .map(|(stem, size)| self.staged_uploads_create(run, stem, *size))
            .collect()
    }
    /// Uploads the bytes to a reserved target. Called from several threads at once (P64: up to
    /// `mechanisms::publish::PARALLEL_UPLOADS` in flight).
    fn upload(&self, run: &ConfirmedRun, target: &StagedTarget, bytes: &[u8]) -> Result<()>;
    /// P64: attaches several uploaded images to the item's gallery in one request; returns their
    /// media ids in the order of `resource_urls`.
    fn product_create_media_many(
        &self,
        run: &ConfirmedRun,
        external_id: &str,
        resource_urls: &[String],
        alt: &str,
    ) -> Result<Vec<String>> {
        resource_urls
            .iter()
            .map(|url| self.product_create_media(run, external_id, url, alt))
            .collect()
    }
    /// P64: creates several shop files in one request; `files` = (resource URL, filename);
    /// returns the file ids in order.
    fn file_create_many(
        &self,
        run: &ConfirmedRun,
        files: &[(String, String)],
    ) -> Result<Vec<String>> {
        files
            .iter()
            .map(|(url, name)| self.file_create(run, url, name))
            .collect()
    }
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
    /// Read-only: the item's gallery media and the files of its condition-photos field, with
    /// ids and image URLs (10F Rules / M41: verified before an item's local files are cleared).
    fn media_readback(&self, external_id: &str) -> Result<MediaReadback>;
    /// Read-only: downloads one published image at most `max_edge` px on its long edge (10F §3:
    /// a cleared item's photos are shown from the platform).
    fn fetch_image(&self, url: &str, max_edge: u32) -> Result<Vec<u8>>;
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
    fn update_item_type(
        &self,
        run: &ConfirmedRun,
        external_id: &str,
        item_type: &str,
    ) -> Result<()>;
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

/// Changelog 45 (P65): the remembered stock default for new items (Settings → Shopify): the
/// location and quantity of the last successful create.
pub trait StockDefaults: Send + Sync {
    fn get(&self) -> Result<Option<StockChoice>>;
    fn remember(&self, choice: &StockChoice) -> Result<()>;
}
