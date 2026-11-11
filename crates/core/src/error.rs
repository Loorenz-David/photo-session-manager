#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("composition_invalid: {reason}")]
    CompositionInvalid { reason: String },
    #[error("frame_unknown")]
    FrameUnknown,
    /// Export or publish refused while main or condition photos need re-framing (10D MC-27
    /// §Revisions); `photos` are their labels in display order (`Main 02`, `Condition 01`).
    #[error("Re-frame first: {}", photos.join(", "))]
    NeedsReview { photos: Vec<String> },
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
    #[error("nothing_to_redo")]
    NothingToRedo,
    #[error("Can't {direction} {label}: {reason}")]
    StepCannotApply {
        direction: String,
        label: String,
        reason: String,
    },
    #[error("capture_folder_not_set")]
    CaptureFolderNotSet,
    /// The direct camera adapter has no SDK session (10E MC-31).
    #[error("camera_not_connected")]
    CameraNotConnected,
    /// A reconnect is offered only on a Wi-Fi session (10E MC-33 R-I29).
    #[error("reconnect_on_cable_refused")]
    ReconnectOnCableRefused,
    /// The camera helper reported that it cannot run (`sdk.unavailable`).
    #[error("sidecar_unavailable")]
    SidecarUnavailable,
    #[error("support_archive_invalid: {file}")]
    SupportArchiveInvalid { file: String },
    #[error("support_version_unsupported: {version}")]
    SupportVersionUnsupported { version: String },
    #[error("support_not_installed")]
    SupportNotInstalled,
    #[error("fingerprint_not_pending")]
    FingerprintNotPending,
    #[error("fingerprint_changed")]
    FingerprintChanged,
    /// A script that would launch Capture One was refused while "Take the camera back" holds the
    /// editor-launch gate (10E MC-34 R-I32).
    #[error("editor_launch_deferred")]
    EditorLaunchDeferred,
    /// "Take the camera back" is not possible now; `reason` says why (R-I32, master plan §6.6).
    #[error("Take the camera back is not possible: {reason}")]
    TakeBackBlocked { reason: String },
    /// "Take the camera back" stopped at `step` (2–5) because of `cause` (R-I32).
    #[error("Take the camera back stopped at step {step}: {cause}")]
    TakeBackFailed { step: u8, cause: String },
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
    #[error("document_not_front")]
    DocumentNotFront,
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
    /// P68 (owner 2026-11-11): the sentence names the fields to check.
    #[error("The Shopify client ID or secret is wrong")]
    CredentialsRejected,
    /// P68: the stored shop domain cannot be a host name (no request was sent).
    #[error("The shop domain looks wrong: {domain}")]
    ShopDomainInvalid { domain: String },
    /// Anything that needs a cleared item's images was refused (10F Q3, P62).
    #[error("This item's files were cleared after publishing")]
    FilesCleared,
    /// Changelog 45 (P65): an export or publish waited 10 minutes for the editor; the sentence
    /// names the photo (`Capture One has not loaded DSC05162.ARW after 10 minutes`).
    #[error("{0}")]
    EditorWaitExpired(String),
}

pub type Result<T> = std::result::Result<T, Error>;
