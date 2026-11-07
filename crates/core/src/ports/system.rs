//! Operating-system facts the camera ownership guard reads (10E MC-34, MC-31 R-I23).
use crate::Result;

/// The names of the processes running on this Mac (R-I30: who could take the camera). Read-only:
/// nothing in the app ever stops another process (M39).
pub trait ProcessList: Send + Sync {
    fn running(&self) -> Result<Vec<String>>;
}

/// The macOS idle-sleep assertion held while a USB camera session is open (R-I23).
pub trait SleepInhibitor: Send + Sync {
    fn hold(&self, reason: &str) -> Result<()>;
    fn release(&self);
}
