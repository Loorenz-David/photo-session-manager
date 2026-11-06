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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CameraName(pub String);
