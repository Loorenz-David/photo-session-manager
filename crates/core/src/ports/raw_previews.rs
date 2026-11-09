use crate::Result;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Extracted {
    pub orientation: Option<u8>,
    /// Oriented JPEG frame, only when it matches the RAW's declared sensor frame.
    pub compose_frame: Option<crate::domain::composition::Frame>,
}
pub trait RawPreviews: Send + Sync {
    fn extract(&self, raw: &Path, dest: &Path) -> Result<Option<Extracted>>;
}
