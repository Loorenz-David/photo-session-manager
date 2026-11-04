//! The output setting of 10D MC-28: every exported or uploaded file is a JPEG the app renders from
//! the master (4:4:4 chroma, sRGB profile embedded; never upscaled).
use super::composition::Frame;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputFormat {
    pub long_edge: u32,
    pub quality: u8,
}
impl Default for OutputFormat {
    fn default() -> Self {
        Self {
            long_edge: 2048,
            quality: 90,
        }
    }
}
impl OutputFormat {
    /// The `output_format` part of an output's cache key (MC-28 §Cache key).
    pub fn format_key(&self) -> String {
        format!("jpeg-{}-q{}", self.long_edge, self.quality)
    }
}

/// What `Imaging::render_output` wrote: the output's size and the master's frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OutputInfo {
    pub width: u32,
    pub height: u32,
    pub frame: Frame,
}

/// The slot's name in file names, payloads and `manifest.json` (`single`, `left`, `right`).
pub fn slot_name(slot: super::composition::OutputSlot) -> &'static str {
    use super::composition::OutputSlot;
    match slot {
        OutputSlot::Single => "single",
        OutputSlot::Left => "left",
        OutputSlot::Right => "right",
    }
}
