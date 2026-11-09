//! What the storage service (10F, P61) needs from the file system: listings with sizes and
//! times, a hash, a permanent delete, and the free space of a disk. Implemented in infra.

use crate::Result;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileEntry {
    pub path: PathBuf,
    pub bytes: u64,
    /// Last modification, ms since the epoch.
    pub modified_ms: i64,
}

/// Free space on the disk holding a folder, and a name for that disk the photographer knows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiskFree {
    pub available_bytes: u64,
    pub disk: String,
}

pub trait StorageFs: Send + Sync {
    /// The regular files directly in `dir` (names starting with `.` skipped); a missing folder
    /// has none.
    fn files(&self, dir: &Path) -> Result<Vec<FileEntry>>;
    /// The regular files anywhere under `dir`, skipping folders named in `skip_dirs` and folders
    /// whose name starts with `.`; a missing folder has none.
    fn files_deep(&self, dir: &Path, skip_dirs: &[&str]) -> Result<Vec<FileEntry>>;
    /// The current size and modification time of one file (`None` when it is gone).
    fn entry(&self, path: &Path) -> Result<Option<FileEntry>>;
    /// SHA-256 of the file's bytes, lowercase hex.
    fn sha256(&self, path: &Path) -> Result<String>;
    /// Deletes one file permanently (10F §Rules: never to the Trash). Gone already is fine.
    fn remove(&self, path: &Path) -> Result<()>;
    fn free(&self, path: &Path) -> Result<DiskFree>;
}
