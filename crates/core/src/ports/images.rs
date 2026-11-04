use crate::{
    Error, Result,
    domain::{
        composition::{CompositionValues, Frame, OutputSlot},
        output::{OutputFormat, OutputInfo},
    },
};
use std::path::{Path, PathBuf};

pub trait Imaging: Send + Sync {
    fn dimensions(&self, path: &Path) -> Result<Option<Frame>>;
    fn thumbnail(&self, source: &Path, dest: &Path, long_edge: u32) -> Result<bool>;
    /// 10D MC-28 §Output render: the output of `slot` rendered from the master by the render
    /// contract of P29 (linear light, turns and flip, straighten, pixel rect, Lanczos3 down to
    /// `format.long_edge`, one JPEG encode). Only `dest` is written.
    fn render_output(
        &self,
        master: &Path,
        values: &CompositionValues,
        slot: OutputSlot,
        format: &OutputFormat,
        dest: &Path,
    ) -> Result<OutputInfo>;
}
pub struct NoImaging;
impl Imaging for NoImaging {
    fn dimensions(&self, _path: &Path) -> Result<Option<Frame>> {
        Ok(None)
    }
    fn thumbnail(&self, _source: &Path, _dest: &Path, _long_edge: u32) -> Result<bool> {
        Ok(false)
    }
    fn render_output(
        &self,
        _master: &Path,
        _values: &CompositionValues,
        _slot: OutputSlot,
        _format: &OutputFormat,
        _dest: &Path,
    ) -> Result<OutputInfo> {
        Err(Error::InvalidData("no renderer".into()))
    }
}

/// The master cache (`masters/<pk[0..2]>/<pk>/master-<adjusted_gen>.tif`, MC-28 §Master).
pub trait MasterStore: Send + Sync {
    fn path(&self, raw: &str, adjusted_gen: i64) -> PathBuf;
    /// Copies `source` in (temp + rename) as the master of `adjusted_gen`, then deletes every
    /// other master of the RAW. Returns the committed path.
    fn commit(&self, raw: &str, adjusted_gen: i64, source: &Path) -> Result<PathBuf>;
    /// Deletes every master of the RAW (the cache is released).
    fn remove_all(&self, raw: &str) -> Result<()>;
}

/// The output cache (`outputs/<pk[0..2]>/<pk>/<uid12>-<slot>-g<g>-r<rev>-<format_key>.jpg`): the
/// file name is the cache key (MC-28 §Cache key).
pub trait OutputStore: Send + Sync {
    fn path(
        &self,
        raw: &str,
        uid12: &str,
        slot: OutputSlot,
        adjusted_gen: i64,
        revision: i64,
        format_key: &str,
    ) -> PathBuf;
    /// Renames the rendered `temp` file to `dest`.
    fn commit(&self, dest: &Path, temp: &Path) -> Result<()>;
    /// Deletes the photo's other cached outputs: every `<uid12>-*.jpg` whose key
    /// (`-g…-r…-<format_key>`) differs from `keep`'s (the other slot of the same key stays).
    fn prune(&self, raw: &str, uid12: &str, keep: &Path) -> Result<()>;
}

/// The first 12 hexadecimal characters of a photo uid (dashes dropped), as in upload names.
pub fn uid12(uid: &str) -> String {
    uid.chars().filter(|c| *c != '-').take(12).collect()
}
