#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("template_exists")]
    TemplateExists,
    #[error("The app could not read that file as an image. Use a JPEG, PNG or TIFF.")]
    TemplateImageUnreadable,
    #[error("no_template")]
    NoTemplate,
    #[error("shot_filled")]
    ShotFilled,
    #[error("session_changed")]
    SessionChanged,
    #[error("not_in_template")]
    NotInTemplate,
    #[error("pairing_unknown")]
    PairingUnknown,
    #[error("phone_port_invalid")]
    PhonePortInvalid,
    /// T4: Shift the rest back with no denied photo before a template-placed one.
    #[error("There is nothing to shift back")]
    NothingToShift,
    /// Export research with a choice that matches no run (owner correction A8).
    #[error("No research runs match this choice.")]
    NothingToExport,
    /// "Delete session" refused (owner correction A17): the sentence the owner reads.
    #[error("{0}")]
    DeleteRefused(String),
    /// Photo files cannot move now (owner correction A21): the sentence the owner reads.
    #[error("{0}")]
    FilesBusy(String),
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
    #[error("focus_unsupported")]
    FocusUnsupported,
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
    /// The platform says the picture is not there (a download answered 403 or 404; 10H §4).
    #[error("image_gone")]
    ImageGone,
    /// Refresh (and later bring back) asked for an item whose photos are on this Mac (MC-H3.3).
    #[error("This item's photos are on this Mac, not in the shop")]
    NotRemote,
    /// MC-H4.1: a bring back of this session is already running.
    #[error("The photos are already being brought back")]
    BringBackRunning,
    /// MC-H4.1, .3: a bring back could not finish; nothing was imported.
    #[error("Could not bring back the photos: {reason}")]
    BringBackFailed { reason: String },
    /// Changelog 45 (P65): an export or publish waited 10 minutes for the editor; the sentence
    /// names the photo (`Capture One has not loaded DSC05162.ARW after 10 minutes`).
    #[error("{0}")]
    EditorWaitExpired(String),
    /// MC-A10 (R-IA22): the withheld count or the conflict set changed since the popover showed
    /// them; nothing was confirmed.
    #[error("AI suggestions changed while you were confirming")]
    SuggestionsChanged,
}

pub type Result<T> = std::result::Result<T, Error>;
