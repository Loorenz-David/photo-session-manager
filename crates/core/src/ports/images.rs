use crate::{Result, domain::composition::Frame};
use std::path::Path;

pub trait Imaging: Send + Sync {
    fn dimensions(&self, path: &Path) -> Result<Option<Frame>>;
    fn thumbnail(&self, source: &Path, dest: &Path, long_edge: u32) -> Result<bool>;
}
pub struct NoImaging;
impl Imaging for NoImaging {
    fn dimensions(&self, _path: &Path) -> Result<Option<Frame>> {
        Ok(None)
    }
    fn thumbnail(&self, _source: &Path, _dest: &Path, _long_edge: u32) -> Result<bool> {
        Ok(false)
    }
}
