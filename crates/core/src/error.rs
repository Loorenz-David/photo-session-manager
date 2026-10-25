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
}

pub type Result<T> = std::result::Result<T, Error>;
