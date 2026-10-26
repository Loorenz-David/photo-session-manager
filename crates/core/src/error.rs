#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("operation timed out")]
    Timeout,
    #[error("I/O: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid data: {0}")]
    InvalidData(String),
    #[error("provider: {0}")]
    Provider(String),
    #[error("relative_path")]
    RelativePath,
    #[error("sku_collision: {existing_sku}")]
    SkuCollision { existing_sku: String },
    #[error("sku_in_shop: {sku}")]
    SkuInShop { sku: String },
    #[error("no_integration")]
    NoIntegration,
    #[error("several_products_carry_sku: {sku}")]
    SeveralProductsCarrySku { sku: String },
    #[error("foreign_media_added: {n}")]
    ForeignMediaAdded { n: usize },
    #[error("gallery_verification")]
    GalleryVerification,
    #[error("upload_incomplete")]
    UploadIncomplete,
    #[error("invalid_value: {0}")]
    InvalidValue(String),
    #[error("not_found: {0}")]
    NotFound(String),
    #[error("target_has_raw")]
    TargetHasRaw,
    #[error("target_published")]
    TargetPublished,
    #[error("nothing_to_undo")]
    NothingToUndo,
    #[error("capture_folder_not_set")]
    CaptureFolderNotSet,
    #[error("hooks_not_installed")]
    HooksNotInstalled,
    #[error("recipe_missing: {name}")]
    RecipeMissing { name: String },
    #[error("tag_verification")]
    TagVerification,
    #[error("variant_not_found")]
    VariantNotFound,
    #[error("document_not_open")]
    DocumentNotOpen,
    #[error("document_not_session")]
    DocumentNotSession,
    #[error("raw_changed: photo {photo_id}")]
    RawChanged { photo_id: i64 },
    #[error("raw_missing: photo {photo_id}")]
    RawMissing { photo_id: i64 },
    /// Attribute writes the platform refused or that did not read back (07A MC-13).
    #[error("attribute_verification: {}", keys.join(", "))]
    AttributeVerification { keys: Vec<String> },
    /// A mutation was attempted without a run committed as confirmed (07A MC-10).
    #[error("not_confirmed")]
    NotConfirmed,
    /// 09A MC-23 grant refusals, with the messages shown to the photographer.
    #[error("This Shopify store is not in the app's organization")]
    ShopNotPermitted,
    #[error("The Session Manager app is not installed on this store")]
    AppNotInstalled,
    #[error("Shopify credentials rejected")]
    CredentialsRejected,
}

pub type Result<T> = std::result::Result<T, Error>;
