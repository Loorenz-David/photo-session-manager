//! The two render caches of 10D MC-28 in app storage: masters (`masters/<pk[0..2]>/<pk>/
//! master-<adjusted_gen>.tif`, one per RAW, the previous generation deleted on commit) and outputs
//! (`outputs/<pk[0..2]>/<pk>/<uid12>-<slot>-g<g>-r<rev>-<format_key>.jpg`, the name being the cache
//! key). Plain file helpers; core decides when they are used.

use crate::paths::AppPaths;
use psm_core::{
    Result,
    domain::{composition::OutputSlot, output::slot_name},
    mechanisms::arrival::path_key,
    ports::images::{MasterStore, OutputStore},
};
use std::{
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

fn folder(paths: &AppPaths, kind: &str, raw: &str) -> PathBuf {
    let key = path_key(raw);
    paths.root.join(kind).join(&key[..2]).join(key)
}

fn remove_file(path: &Path) -> Result<()> {
    match fs::remove_file(path) {
        Err(error) if error.kind() != ErrorKind::NotFound => Err(error.into()),
        _ => Ok(()),
    }
}

pub struct MasterFiles(pub AppPaths);
impl MasterStore for MasterFiles {
    fn path(&self, raw: &str, adjusted_gen: i64) -> PathBuf {
        folder(&self.0, "masters", raw).join(format!("master-{adjusted_gen}.tif"))
    }
    fn commit(&self, raw: &str, adjusted_gen: i64, source: &Path) -> Result<PathBuf> {
        let dest = self.path(raw, adjusted_gen);
        let dir = dest.parent().map(Path::to_path_buf).unwrap_or_default();
        fs::create_dir_all(&dir)?;
        let temp = dir.join(format!(".master-{adjusted_gen}-{}.tmp", std::process::id()));
        fs::copy(source, &temp)?;
        fs::rename(&temp, &dest)?;
        for entry in fs::read_dir(&dir)? {
            let path = entry?.path();
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            if path != dest && name.starts_with("master-") && name.ends_with(".tif") {
                remove_file(&path)?;
            }
        }
        // Capture One's copy in its output folder has served its purpose (361 MB per 61 MP
        // master); the RAW and the committed master are untouched.
        if let Err(error) = remove_file(source) {
            tracing::warn!(%error, source = %source.display(), "editor master output not removed");
        }
        Ok(dest)
    }
    fn remove_all(&self, raw: &str) -> Result<()> {
        match fs::remove_dir_all(folder(&self.0, "masters", raw)) {
            Err(error) if error.kind() != ErrorKind::NotFound => Err(error.into()),
            _ => Ok(()),
        }
    }
}

pub struct OutputFiles(pub AppPaths);
impl OutputStore for OutputFiles {
    fn path(
        &self,
        raw: &str,
        uid12: &str,
        slot: OutputSlot,
        adjusted_gen: i64,
        revision: i64,
        format_key: &str,
    ) -> PathBuf {
        folder(&self.0, "outputs", raw).join(format!(
            "{uid12}-{}-g{adjusted_gen}-r{revision}-{format_key}.jpg",
            slot_name(slot)
        ))
    }
    fn commit(&self, dest: &Path, temp: &Path) -> Result<()> {
        if let Some(dir) = dest.parent() {
            fs::create_dir_all(dir)?;
        }
        fs::rename(temp, dest)?;
        Ok(())
    }
    fn prune(&self, raw: &str, uid12: &str, keep: &Path) -> Result<()> {
        let key_of = |name: &str| -> Option<String> {
            let rest = name
                .strip_prefix(&format!("{uid12}-"))?
                .strip_suffix(".jpg")?;
            let (_, key) = rest.split_once('-')?;
            Some(key.to_string())
        };
        let keep_name = keep.file_name().unwrap_or_default().to_string_lossy();
        let Some(keep_key) = key_of(&keep_name) else {
            return Ok(());
        };
        let dir = folder(&self.0, "outputs", raw);
        let entries = match fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(error.into()),
        };
        for entry in entries {
            let path = entry?.path();
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            if let Some(key) = key_of(&name)
                && key != keep_key
            {
                remove_file(&path)?;
            }
        }
        Ok(())
    }
}
