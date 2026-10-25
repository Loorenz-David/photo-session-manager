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
}

pub type Result<T> = std::result::Result<T, Error>;
