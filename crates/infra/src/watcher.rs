use notify::{Config, RecursiveMode, Watcher};
use psm_core::{Error, Result};
use std::path::{Path, PathBuf};
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct WatchEvent {
    pub path: PathBuf,
}

pub struct FsWatcher {
    _watcher: Box<dyn Watcher + Send>,
}

fn event_handler(
    root: PathBuf,
    returned_root: PathBuf,
    tx: mpsc::UnboundedSender<WatchEvent>,
) -> impl FnMut(notify::Result<notify::Event>) + Send {
    move |event| {
        if let Ok(event) = event {
            for path in event.paths {
                let absolute = if path.is_absolute() {
                    path
                } else {
                    root.join(path)
                };
                if let Ok(relative) = absolute.strip_prefix(&root)
                    && relative.components().count() <= 2
                {
                    let _ = tx.send(WatchEvent {
                        path: returned_root.join(relative),
                    });
                }
            }
        }
    }
}

impl FsWatcher {
    pub fn watch(dir: &Path) -> Result<(Self, mpsc::UnboundedReceiver<WatchEvent>)> {
        let root = dir.canonicalize()?;
        let returned_root = if dir.is_absolute() {
            dir.to_path_buf()
        } else {
            std::env::current_dir()?.join(dir)
        };
        let (tx, rx) = mpsc::unbounded_channel();
        let native = notify::recommended_watcher(event_handler(
            root.clone(),
            returned_root.clone(),
            tx.clone(),
        ))
        .ok()
        .and_then(|mut watcher| {
            watcher
                .watch(&root, RecursiveMode::Recursive)
                .ok()
                .map(|_| watcher)
        });
        let watcher: Box<dyn Watcher + Send> = match native {
            Some(native) => Box::new(native),
            None => {
                let mut poll = notify::PollWatcher::new(
                    event_handler(root.clone(), returned_root, tx),
                    Config::default().with_poll_interval(std::time::Duration::from_millis(100)),
                )
                .map_err(|e| Error::Provider(e.to_string()))?;
                poll.watch(&root, RecursiveMode::Recursive)
                    .map_err(|e| Error::Provider(e.to_string()))?;
                Box::new(poll)
            }
        };
        Ok((Self { _watcher: watcher }, rx))
    }
}
