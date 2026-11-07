use crate::{
    Result,
    domain::camera::{
        CameraCapabilities, CameraStatus, CaptureId, Delivery, DeliveryOutcome, ExposureObserved,
        Link,
    },
};
use std::path::Path;

pub trait CameraProvider: Send + Sync {
    fn raw_extensions(&self) -> &'static [&'static str];
    fn status(&self) -> Result<CameraStatus>;
    fn capture(&self) -> Result<CaptureId>;
    fn capabilities(&self) -> CameraCapabilities;
    /// Closes and reopens the camera session (10E MC-33 R-I29: Wi-Fi only). Fails closed for
    /// providers that cannot.
    fn reconnect(&self) -> Result<()> {
        Err(crate::Error::Provider("not supported".into()))
    }
    /// Asks the camera for its FTP state (R-I29: sent when the watchdog first raises).
    fn request_ftp_status(&self) -> Result<()> {
        Err(crate::Error::Provider("not supported".into()))
    }
}
pub trait CaptureSink: Send + Sync {
    fn exposure_observed(&self, e: ExposureObserved) -> Result<i64>;
    fn delivery(&self, d: Delivery) -> Result<DeliveryOutcome>;
    fn session_uncertain(&self) -> Result<()>;
    fn transfer_in_progress(&self, source: Link, active: bool);
}
pub trait ArrivalSource: Send + Sync {
    fn via(&self) -> Link;
    fn transfer_in_progress(&self) -> bool;
}
pub trait ArrivalFollow: Send + Sync {
    fn photo_arrived(&self, session_id: i64, raw: &str, folder: &Path);
    fn collection_switched(&self) -> Result<()>;
}
/// The names the FTP receiver lists (MC-32 R-I24): `(camera_filename, size, completed_at)` of the
/// files completed since `since` (UTC ms). The capture ledger implements it (P44; defined by P43 in
/// `psm_infra::ftp`, which re-exports it).
pub trait CompletedNames: Send + Sync {
    fn completed_names_since(&self, since: i64) -> Vec<(String, u64, i64)>;
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenameOutcome {
    Moved,
    /// The destination exists; nothing was moved or replaced.
    Exists,
}
/// The file operations of the capture ledger (10E MC-33 R-I27): a rename that never replaces
/// an existing file, and nothing that deletes.
pub trait DeliveryFs: Send + Sync {
    fn rename_exclusive(&self, from: &Path, to: &Path) -> Result<RenameOutcome>;
    fn create_dir_all(&self, dir: &Path) -> Result<()>;
    fn file_size(&self, path: &Path) -> Result<u64>;
}
