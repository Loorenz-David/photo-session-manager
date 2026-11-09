use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const FIFO_WINDOW_MS: i64 = 300_000;
pub const WATCHDOG_MIN_AGE_MS: i64 = 60_000;
pub const WATCHDOG_MAX_AGE_MS: i64 = 1_800_000;
pub const TRANSFER_EXPECTED_MS: i64 = 10_000;
pub const COMPLETED_NAMES_WINDOW_MS: i64 = 86_400_000;

pub mod secrets {
    pub const SERVICE: &str = "com.loorenz.sessionmanager.camera";
    pub const ACCESS_AUTH_USER: &str = "access-auth-user";
    pub const ACCESS_AUTH_PASSWORD: &str = "access-auth-password";
    pub const FTP_PASSWORD: &str = "ftp-password";
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Link {
    Cable,
    Wireless,
}

impl Link {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Cable => "cable",
            Self::Wireless => "wireless",
        }
    }
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "cable" => Some(Self::Cable),
            "wireless" => Some(Self::Wireless),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BusyReason {
    Exposure,
    Transfer,
    Switching,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NeedsActionReason {
    AuthenticationRefused,
    FingerprintMismatch,
    FingerprintUnconfirmed,
    PhoneAppConnected,
    HeldByOtherProgram,
    StoreDestinationUnverified,
    FtpUnreachable,
    FilesNotReceived { count: u32 },
    SupportNotInstalled,
}
impl NeedsActionReason {
    pub fn rank(&self) -> u8 {
        match self {
            Self::AuthenticationRefused => 0,
            Self::FingerprintMismatch => 1,
            Self::FingerprintUnconfirmed => 2,
            Self::PhoneAppConnected => 3,
            Self::HeldByOtherProgram => 4,
            Self::StoreDestinationUnverified => 5,
            Self::FtpUnreachable => 4,
            Self::FilesNotReceived { .. } => 6,
            Self::SupportNotInstalled => 7,
        }
    }
    pub fn code(&self) -> &'static str {
        match self {
            Self::AuthenticationRefused => "camera_auth_refused",
            Self::FingerprintMismatch => "camera_fingerprint_mismatch",
            Self::FingerprintUnconfirmed => "camera_fingerprint_unconfirmed",
            Self::PhoneAppConnected => "camera_phone_connected",
            Self::HeldByOtherProgram => "camera_held",
            Self::StoreDestinationUnverified => "camera_store_destination_unverified",
            Self::FtpUnreachable => "camera_ftp_unreachable",
            Self::FilesNotReceived { .. } => "camera_files_not_received",
            Self::SupportNotInstalled => "camera_support_not_installed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CameraStatus {
    Connected {
        name: String,
        link: Link,
    },
    Connecting {
        link: Link,
    },
    Busy {
        reason: BusyReason,
    },
    NeedsAction {
        reason: NeedsActionReason,
        holder: Option<String>,
        /// The MC-34 sentence for `HeldByOtherProgram` (R-I30, from `name_programs`); `None` for
        /// every other reason.
        message: Option<String>,
    },
    Disconnected,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CameraCapabilities {
    pub status: bool,
    pub capture: bool,
    pub transfer: bool,
    pub exposure_events: bool,
    pub settings_read: bool,
    pub settings_write_store_destination: bool,
    pub depends_on_editing_provider: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CaptureId(pub String);
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExposureInitiator {
    App,
    Camera,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExposureObserved {
    pub capture_id: Option<CaptureId>,
    pub at: i64,
    pub initiated: ExposureInitiator,
    pub provisional: bool,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Delivery {
    pub camera_filename: String,
    pub written_path: PathBuf,
    pub sha256: String,
    pub size: u64,
    pub via: Link,
    pub completed_at: i64,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CameraSettings {
    pub receiver_port: u16,
    pub passive_port_start: u16,
    pub passive_port_end: u16,
    #[serde(default = "default_auto_take_back")]
    pub auto_take_back: bool,
}
fn default_auto_take_back() -> bool {
    true
}
impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            receiver_port: 2121,
            passive_port_start: 50000,
            passive_port_end: 50009,
            auto_take_back: true,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeliveryOutcome {
    Photo,
    Duplicate,
    RenamedPhoto,
}

/// Snake-case text of the ledger enums as stored in SQLite (master plan §6.5, §6.6).
macro_rules! stored_enum {
    ($name:ident { $($variant:ident => $text:literal),+ $(,)? }) => {
        impl $name {
            pub fn as_str(self) -> &'static str {
                match self { $(Self::$variant => $text),+ }
            }
            pub fn parse(text: &str) -> Option<Self> {
                match text { $($text => Some(Self::$variant),)+ _ => None }
            }
        }
    };
}
stored_enum!(DeliveryOutcome { Photo => "photo", Duplicate => "duplicate", RenamedPhoto => "renamed_photo" });

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExposureStatus {
    Expecting,
    Received,
    /// Reserved (10E MC-33: an observation, never proof of loss); nothing writes it in P44.
    MissingAtDeadline,
    Uncertain,
    Dismissed,
}
stored_enum!(ExposureStatus {
    Expecting => "expecting",
    Received => "received",
    MissingAtDeadline => "missing_at_deadline",
    Uncertain => "uncertain",
    Dismissed => "dismissed",
});

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Association {
    Filename,
    Stem,
    Fifo,
    None,
}
stored_enum!(Association { Filename => "filename", Stem => "stem", Fifo => "fifo", None => "none" });

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeliveryState {
    Pending,
    Done,
}
stored_enum!(DeliveryState { Pending => "pending", Done => "done" });

impl ExposureInitiator {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::App => "app",
            Self::Camera => "camera",
        }
    }
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "app" => Some(Self::App),
            "camera" => Some(Self::Camera),
            _ => None,
        }
    }
}

/// A `camera_exposures` row (10E MC-33 R-I25).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Exposure {
    pub id: i64,
    pub capture_id: Option<String>,
    pub exposed_at: i64,
    pub initiated: ExposureInitiator,
    pub session_id: Option<i64>,
    pub status: ExposureStatus,
    pub expected_files: Option<u32>,
    pub created_at: i64,
}

/// The fields of a new `camera_exposures` row; `status` is always `expecting` and
/// `expected_files` unset (no input path sets it in 10E).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewExposure {
    pub capture_id: Option<String>,
    pub exposed_at: i64,
    pub initiated: ExposureInitiator,
    pub session_id: Option<i64>,
    pub created_at: i64,
}

/// A `camera_deliveries` row (10E MC-33 R-I25; `written_path` per master plan §6.5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeliveryRow {
    pub id: i64,
    pub exposure_id: Option<i64>,
    pub camera_filename: String,
    pub file_key: String,
    pub sha256: String,
    pub size: u64,
    pub via: Link,
    pub completed_at: i64,
    pub association: Association,
    pub outcome: DeliveryOutcome,
    pub session_id: Option<i64>,
    pub written_path: String,
    pub final_path: String,
    pub state: DeliveryState,
    pub created_at: i64,
}

/// The fields of a new `camera_deliveries` row (always written `pending`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewDelivery {
    pub exposure_id: Option<i64>,
    pub camera_filename: String,
    pub file_key: String,
    pub sha256: String,
    pub size: u64,
    pub via: Link,
    pub completed_at: i64,
    pub association: Association,
    pub outcome: DeliveryOutcome,
    pub session_id: Option<i64>,
    pub written_path: String,
    pub final_path: String,
    pub created_at: i64,
}

/// The take-over of a `pending` row by a re-sent staging file (P44 contract): every field the
/// new decision may change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingUpdate {
    pub sha256: String,
    pub size: u64,
    pub completed_at: i64,
    pub final_path: String,
    pub outcome: DeliveryOutcome,
    pub association: Association,
    pub exposure_id: Option<i64>,
    pub session_id: Option<i64>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CameraName(pub String);
