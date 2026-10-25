use crate::Result;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Extracted {
    pub orientation: Option<u8>,
}
pub trait RawPreviews: Send + Sync {
    fn extract(&self, raw: &Path, dest: &Path) -> Result<Option<Extracted>>;
}
