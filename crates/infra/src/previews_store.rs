use crate::paths::AppPaths;
use psm_core::{Result, mechanisms::arrival::path_key, services::previews::PreviewFiles};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub struct PreviewStore(pub AppPaths);
impl PreviewFiles for PreviewStore {
    fn copy_adjusted(&self, raw: &str, generation: i64, source: &Path) -> Result<PathBuf> {
        let key = path_key(raw);
        let dir = self.0.root.join("previews").join(&key[..2]).join(key);
        fs::create_dir_all(&dir)?;
        let dest = dir.join(format!("adjusted-{generation}.jpg"));
        let temp = dir.join(format!(".adjusted-{generation}-{}.tmp", std::process::id()));
        fs::copy(source, &temp)?;
        fs::rename(&temp, &dest)?;
        Ok(dest)
    }
    fn remove(&self, path: &Path) -> Result<()> {
        if path.exists() {
            fs::remove_file(path)?;
        }
        Ok(())
    }
}
