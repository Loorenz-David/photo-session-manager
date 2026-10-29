use psm_core::Result;
use std::{
    env, fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
pub struct AppPaths {
    pub root: PathBuf,
}

impl AppPaths {
    pub fn for_dir(path: impl AsRef<Path>) -> Self {
        Self {
            root: path.as_ref().to_path_buf(),
        }
    }

    pub fn from_env_or_default() -> Self {
        if let Some(path) = env::var_os("PSM_DATA_DIR") {
            return Self::for_dir(PathBuf::from(path));
        }
        let home = env::var_os("HOME").unwrap_or_default();
        Self::for_dir(
            PathBuf::from(home).join("Library/Application Support/com.loorenz.sessionmanager"),
        )
    }

    pub fn create(&self) -> Result<()> {
        for name in ["previews", "hooks", "spool", "spool/bad", "logs"] {
            fs::create_dir_all(self.root.join(name))?;
        }
        if let Err(error) = fs::create_dir_all(self.root.join("backup")) {
            tracing::warn!(%error, "backup directory unavailable");
        }
        Ok(())
    }

    pub fn db(&self) -> PathBuf {
        self.root.join("session_manager.db")
    }
    pub fn settings(&self) -> PathBuf {
        self.root.join("settings.json")
    }
    pub fn spool(&self) -> PathBuf {
        self.root.join("spool")
    }
    pub fn hooks(&self) -> PathBuf {
        self.root.join("hooks")
    }
}
