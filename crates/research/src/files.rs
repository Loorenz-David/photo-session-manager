//! The research file store: `research/images/<sha[0..2]>/<sha>.jpg` under the data dir, named by
//! the SHA-256 of the bytes. Research copies of our photos and third-party listing images both
//! live here; the folder is created on first use.

use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use psm_core::Result;
use sha2::{Digest, Sha256};

static TEMP: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone)]
pub struct ResearchFiles {
    root: PathBuf,
}

/// Lowercase hex SHA-256 of `bytes`.
pub fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

impl ResearchFiles {
    /// `data_dir` is the app's data folder (`AppPaths::root`).
    pub fn new(data_dir: &Path) -> Self {
        Self {
            root: data_dir.join("research").join("images"),
        }
    }

    pub fn path(&self, sha256: &str) -> PathBuf {
        let prefix = sha256.get(..2).unwrap_or("00");
        self.root.join(prefix).join(format!("{sha256}.jpg"))
    }

    pub fn exists(&self, sha256: &str) -> bool {
        self.path(sha256).is_file()
    }

    /// A fresh temporary path inside the store (same file system, so the final move is a rename).
    pub fn temp_path(&self) -> Result<PathBuf> {
        fs::create_dir_all(&self.root)?;
        let n = TEMP.fetch_add(1, Ordering::Relaxed);
        Ok(self
            .root
            .join(format!(".tmp-{}-{n}.jpg", std::process::id())))
    }

    /// Moves a finished temporary file to its SHA-256 name. An existing copy with the same name
    /// holds the same bytes, so the temporary file is dropped.
    pub fn commit_temp(&self, temp: &Path) -> Result<(String, PathBuf, u64)> {
        let bytes = fs::read(temp)?;
        let sha = sha256_hex(&bytes);
        let path = self.path(&sha);
        if path.is_file() {
            fs::remove_file(temp)?;
        } else {
            fs::create_dir_all(path.parent().unwrap_or(&self.root))?;
            fs::rename(temp, &path)?;
        }
        Ok((sha, path, bytes.len() as u64))
    }

    /// Deletes the stored copy named `sha256` (MC-A11 retention); a missing file is no error.
    pub fn remove(&self, sha256: &str) -> Result<()> {
        match fs::remove_file(self.path(sha256)) {
            Err(error) if error.kind() != std::io::ErrorKind::NotFound => Err(error.into()),
            _ => Ok(()),
        }
    }

    /// Stores bytes (a listing's image) under their SHA-256 and returns it.
    pub fn put(&self, bytes: &[u8]) -> Result<String> {
        let temp = self.temp_path()?;
        fs::write(&temp, bytes)?;
        let (sha, _, _) = self.commit_temp(&temp)?;
        Ok(sha)
    }
}
