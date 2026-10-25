use notify::{RecursiveMode, Watcher};
use psm_core::{Error, Result};
use std::path::{Path, PathBuf};
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct WatchEvent {
    pub path: PathBuf,
}

pub struct FsWatcher {
    _watcher: notify::RecommendedWatcher,
}

impl FsWatcher {
    pub fn watch(dir: &Path) -> Result<(Self, mpsc::UnboundedReceiver<WatchEvent>)> {
        let root = dir.canonicalize()?;
        let (tx, rx) = mpsc::unbounded_channel();
        let mut watcher =
            notify::recommended_watcher(move |event: notify::Result<notify::Event>| {
                if let Ok(event) = event {
                    for path in event.paths {
                        let absolute = if path.is_absolute() {
                            path
                        } else {
                            root.join(path)
                        };
                        if absolute
                            .strip_prefix(&root)
                            .is_ok_and(|relative| relative.components().count() <= 2)
                        {
                            let _ = tx.send(WatchEvent { path: absolute });
                        }
                    }
                }
            })
            .map_err(|e| Error::Provider(e.to_string()))?;
        watcher
            .watch(&dir.canonicalize()?, RecursiveMode::Recursive)
            .map_err(|e| Error::Provider(e.to_string()))?;
        Ok((Self { _watcher: watcher }, rx))
    }
}
