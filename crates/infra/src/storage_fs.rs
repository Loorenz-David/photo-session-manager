//! The storage service's file system (10F, P61): listings with sizes and times, hashes,
//! permanent deletes, and the free space of a disk from a replaceable source (the disk itself, or
//! a test's stand-in).

use crate::disk::FreeSpace;
use psm_core::{
    Result,
    mechanisms::hash::sha256_file,
    ports::storage::{DiskFree, FileEntry, StorageFs},
};
use std::{fs, io::ErrorKind, path::Path, sync::Arc, time::UNIX_EPOCH};

pub type FreeSource = Arc<dyn Fn(&Path) -> std::io::Result<FreeSpace> + Send + Sync>;

pub struct LocalStorageFs {
    free: FreeSource,
}
impl LocalStorageFs {
    pub fn new(free: FreeSource) -> Self {
        Self { free }
    }
}

fn entry_of(path: &Path, meta: &fs::Metadata) -> FileEntry {
    FileEntry {
        path: path.to_path_buf(),
        bytes: meta.len(),
        modified_ms: meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map_or(0, |d| d.as_millis() as i64),
    }
}

fn hidden(name: &std::ffi::OsStr) -> bool {
    name.to_string_lossy().starts_with('.')
}

fn read_dir(dir: &Path) -> Result<Option<fs::ReadDir>> {
    match fs::read_dir(dir) {
        Ok(entries) => Ok(Some(entries)),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

impl StorageFs for LocalStorageFs {
    fn files(&self, dir: &Path) -> Result<Vec<FileEntry>> {
        let mut out = Vec::new();
        let Some(entries) = read_dir(dir)? else {
            return Ok(out);
        };
        for entry in entries {
            let entry = entry?;
            if hidden(&entry.file_name()) {
                continue;
            }
            let meta = entry.metadata()?;
            if meta.is_file() {
                out.push(entry_of(&entry.path(), &meta));
            }
        }
        out.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(out)
    }
    fn files_deep(&self, dir: &Path, skip_dirs: &[&str]) -> Result<Vec<FileEntry>> {
        let mut out = Vec::new();
        let mut stack = vec![dir.to_path_buf()];
        while let Some(folder) = stack.pop() {
            let Some(entries) = read_dir(&folder)? else {
                continue;
            };
            for entry in entries {
                let entry = entry?;
                let name = entry.file_name();
                if hidden(&name) {
                    continue;
                }
                let meta = entry.metadata()?;
                if meta.is_dir() {
                    if !skip_dirs.iter().any(|skip| name == *skip) {
                        stack.push(entry.path());
                    }
                } else if meta.is_file() {
                    out.push(entry_of(&entry.path(), &meta));
                }
            }
        }
        out.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(out)
    }
    fn entry(&self, path: &Path) -> Result<Option<FileEntry>> {
        match fs::metadata(path) {
            Ok(meta) if meta.is_file() => Ok(Some(entry_of(path, &meta))),
            Ok(_) => Ok(None),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error.into()),
        }
    }
    fn sha256(&self, path: &Path) -> Result<String> {
        sha256_file(path)
    }
    fn remove(&self, path: &Path) -> Result<()> {
        match fs::remove_file(path) {
            Err(error) if error.kind() != ErrorKind::NotFound => Err(error.into()),
            _ => Ok(()),
        }
    }
    fn free(&self, path: &Path) -> Result<DiskFree> {
        let free = (self.free)(path)?;
        Ok(DiskFree {
            available_bytes: free.available_bytes,
            disk: free.disk,
        })
    }
}
