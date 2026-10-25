use std::path::{Path, PathBuf};
use notify::{Watcher, RecursiveMode};
use tokio::sync::mpsc;
use psm_core::{Error, Result};

#[derive(Debug, Clone)]
pub struct WatchEvent { pub path: PathBuf }

pub struct FsWatcher { _watcher: notify::RecommendedWatcher }

impl FsWatcher {
    pub fn watch(dir: &Path) -> Result<(Self, mpsc::UnboundedReceiver<WatchEvent>)> {
        let root = dir.canonicalize()?;
        let (tx, rx) = mpsc::unbounded_channel();
        let mut watcher = notify::recommended_watcher(move |event: notify::Result<notify::Event>| {
            if let Ok(event) = event { for path in event.paths {
                let absolute = if path.is_absolute() { path } else { root.join(path) };
                let _ = tx.send(WatchEvent { path: absolute });
            }}
        }).map_err(|e| Error::Provider(e.to_string()))?;
        watcher.watch(dir, RecursiveMode::Recursive).map_err(|e| Error::Provider(e.to_string()))?;
        Ok((Self { _watcher: watcher }, rx))
    }
}

