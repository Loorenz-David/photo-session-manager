//! The macOS Trash (owner correction A17, "Delete session"): the one place the app moves files
//! the owner asked to delete. Unlike clean-up's permanent deletes (10F, `StorageFs::remove`), a
//! file sent here can be put back from the Trash by the owner. Implemented in infra.

use crate::Result;
use std::path::Path;

pub trait Trash: Send + Sync {
    /// Moves the file or folder at `path` to the Trash. `Ok(false)` when nothing is there (gone
    /// already is fine); an error when it is there and could not be moved.
    fn trash(&self, path: &Path) -> Result<bool>;
}
