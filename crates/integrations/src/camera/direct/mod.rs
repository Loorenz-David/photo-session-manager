//! The direct camera adapter (10E): the Sony-SDK-sidecar `CameraProvider` (P46) and the camera
//! ownership guard (P48: MC-34 R-I30…R-I33).
pub mod adapter;
pub mod filenames;
pub mod ownership;
pub mod start_order;
pub mod take_back;

pub use adapter::{DirectCameraAdapter, holder_from_owners, needs_action_from_connect_failed};
pub use filenames::normalize_camera_filename;
pub use ownership::{HolderReport, KNOWN_CAMERA_PROGRAMS, name_programs};
pub use start_order::{StartOrder, StartReason, StartState};
pub use take_back::{TakeBackReport, TakeBackService};

/// R-I31: the longest an app-start launch of Capture One waits for the USB session.
pub const START_ORDER_WAIT_MS: u64 = 15_000;
/// R-I32 step 3: how long Capture One gets to quit.
pub const TAKE_BACK_QUIT_WAIT_MS: u64 = 20_000;
/// R-I32 step 4: a `usb.claimed_by` within this window gets another `usb-retry`.
pub const TAKE_BACK_FREE_WAIT_MS: u64 = 5_000;
/// R-I32 step 4: how long `connection.ready` on USB is waited for.
pub const TAKE_BACK_READY_WAIT_MS: u64 = 15_000;
/// P60 #25: the cable still held this long after Capture One quit, with no app using cameras, is
/// macOS's camera service stuck.
pub const TAKE_BACK_STUCK_SERVICE_MS: u64 = 10_000;
/// P60 #22: how long the take-back waits for the app's own Capture One scripts to finish before
/// it saves and quits.
pub const TAKE_BACK_DRAIN_WAIT_MS: u64 = 30_000;
/// P60 #23: the header while Capture One does not answer or does not quit.
pub const CAPTURE_ONE_WINDOW_OPEN: &str =
    "Capture One has a window open — close it so Session Manager can take the camera";
/// P60 #25: the header while the cable stays held after Capture One quit.
pub const CAMERA_SERVICE_STUCK: &str =
    "macOS's camera service is holding the cable with no app using it. Restart the Mac to free it.";
/// R-I32: an exposure younger than this still expecting a file blocks the take-back.
pub const TAKE_BACK_EXPOSURE_GRACE_MS: i64 = 10_000;
/// Mirrors the sidecar's `usb_failures_before_wifi_fallback`.
pub const USB_FAILURES_BEFORE_WIFI: u32 = 3;
/// The idle-sleep assertion's name while a USB session is open (R-I23).
pub const SLEEP_REASON: &str = "Session Manager: camera connected by cable";
