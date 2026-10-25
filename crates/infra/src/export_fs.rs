//! Export folder operations (05A MC-8): a hidden staging folder per run beside `<key>/`, filled by
//! copies (temp file + rename), then swapped in with renames so no stale file survives.

use psm_core::{Result, services::export::ExportFiles};
use std::{
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

pub struct ExportFs {
    /// The `export_folder` setting.
    pub root: PathBuf,
}

impl ExportFs {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }
}

impl ExportFiles for ExportFs {
    fn create_staging(&self, key: &str, run_id: i64) -> Result<PathBuf> {
        fs::create_dir_all(&self.root)?;
        let staging = self.root.join(format!(".{key}.partial-{run_id}"));
        if staging.exists() {
            fs::remove_dir_all(&staging)?;
        }
        fs::create_dir(&staging)?;
        Ok(staging)
    }
    fn copy_in(&self, staging: &Path, name: &str, source: &Path) -> Result<()> {
        let temp = staging.join(format!(".{name}.tmp"));
        fs::copy(source, &temp)?;
        fs::rename(&temp, staging.join(name))?;
        Ok(())
    }
    fn write_in(&self, staging: &Path, name: &str, bytes: &[u8]) -> Result<()> {
        let temp = staging.join(format!(".{name}.tmp"));
        fs::write(&temp, bytes)?;
        fs::rename(&temp, staging.join(name))?;
        Ok(())
    }
    fn rename_in(&self, staging: &Path, from: &str, to: &str) -> Result<()> {
        fs::rename(staging.join(from), staging.join(to))?;
        Ok(())
    }
    fn remove_in(&self, staging: &Path, name: &str) -> Result<()> {
        match fs::remove_file(staging.join(name)) {
            Err(error) if error.kind() != ErrorKind::NotFound => Err(error.into()),
            _ => Ok(()),
        }
    }
    fn swap(&self, staging: &Path, key: &str, run_id: i64) -> Result<()> {
        let current = self.folder(key);
        let old = self.root.join(format!(".{key}.old-{run_id}"));
        let had_current = match fs::rename(&current, &old) {
            Ok(()) => true,
            Err(error) if error.kind() == ErrorKind::NotFound => false,
            Err(error) => return Err(error.into()),
        };
        if let Err(error) = fs::rename(staging, &current) {
            if had_current {
                let _ = fs::rename(&old, &current);
            }
            return Err(error.into());
        }
        if had_current && let Err(error) = fs::remove_dir_all(&old) {
            tracing::warn!(%error, old = %old.display(), "previous export folder not removed");
        }
        Ok(())
    }
    fn discard(&self, staging: &Path) -> Result<()> {
        match fs::remove_dir_all(staging) {
            Err(error) if error.kind() != ErrorKind::NotFound => Err(error.into()),
            _ => Ok(()),
        }
    }
    fn folder(&self, key: &str) -> PathBuf {
        self.root.join(key)
    }
}
