use std::{sync::Arc, time::Instant};

use super::camera::Link;

#[derive(Debug, Clone)]
pub struct CameraFrame {
    pub source: u64,
    pub frame_no: u32,
    pub width: u16,
    pub height: u16,
    pub jpeg: Vec<u8>,
    pub link: Link,
}
#[derive(Debug, Clone)]
pub struct LiveFrame {
    pub seq: u64,
    pub start: u32,
    pub received: Instant,
    pub captured_at_ms: i64,
    pub width: u16,
    pub height: u16,
    pub jpeg: Arc<[u8]>,
    pub link: Link,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ViewerId(pub u64);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ViewerDemand {
    pub r#gen: u32,
    pub live: bool,
    pub visible: bool,
}
impl ViewerDemand {
    pub fn wanting(self) -> bool {
        self.live && self.visible
    }
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FocusPoint {
    pub x: f64,
    pub y: f64,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusId(pub String);
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FocusRefusal {
    AreaNotMovable,
    NotConnected,
    Unsupported,
    BadPoint,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FocusOutcome {
    Focused,
    NotFocused,
    NoReport,
    Superseded,
    Refused(FocusRefusal),
    Failed,
}
#[derive(Debug, Clone, PartialEq)]
pub struct FocusFrame {
    pub cx: f64,
    pub cy: f64,
    pub w: f64,
    pub h: f64,
    pub state: FocusFrameState,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusFrameState {
    Focused,
    NotFocused,
    Unknown,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExposureReadout {
    pub shutter: String,
    pub aperture: String,
    pub iso: String,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LiveState {
    Off,
    Unavailable { reason: String },
    Live { link: Link },
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceStop {
    SessionClosed,
    BadRecord,
    SidecarExit,
    Unavailable { reason: String },
}
#[derive(Debug, Clone, PartialEq)]
pub enum LiveEvent {
    State(LiveState),
    Focus {
        focus_id: FocusId,
        outcome: FocusOutcome,
    },
    FocusFrames(Vec<FocusFrame>),
    Exposure(ExposureReadout),
    Capture {
        capture_id: String,
        state: CaptureReportState,
    },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureReportState {
    Acknowledged,
    Exposed,
    Failed,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PublishOutcome {
    Forwarded { seq: u64 },
    Repeat,
    Stale,
}
/// `settings.phone` (`master_plan_live.md` §4.5): the phone server is on by default, on port 8686.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct PhoneSettings {
    pub enabled: bool,
    pub port: u16,
}
impl Default for PhoneSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            port: 8686,
        }
    }
}

/// One paired phone (PH1): a `phone_pairings` row. The token itself is never stored (only its
/// SHA-256) and never read back.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct PhonePairing {
    pub id: i64,
    pub name: String,
    pub created_at: i64,
    pub last_seen_at: Option<i64>,
    pub revoked_at: Option<i64>,
}
