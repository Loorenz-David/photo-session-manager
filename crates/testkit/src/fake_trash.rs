//! The suite's macOS Trash (A17): moves into a folder of the test's own directory and records
//! what it moved, and what the filing target was at that moment. It can be told to refuse a path.

use psm_core::{Error, Result, ports::trash::Trash};
use std::{
    path::{Path, PathBuf},
    sync::{
        Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};

type Observer = Box<dyn Fn(&Path) -> String + Send + Sync>;

pub struct FakeTrash {
    dir: PathBuf,
    next: AtomicUsize,
    /// `(path moved, what the observer said at that moment)` in order.
    pub moved: Mutex<Vec<(PathBuf, String)>>,
    /// A path containing this text is refused.
    pub refuse: Mutex<Option<String>>,
    observer: Mutex<Option<Observer>>,
}

impl FakeTrash {
    pub fn new(dir: PathBuf) -> Self {
        Self {
            dir,
            next: AtomicUsize::new(0),
            moved: Mutex::new(Vec::new()),
            refuse: Mutex::new(None),
            observer: Mutex::new(None),
        }
    }
    /// Called at every move; its answer is recorded beside the path.
    pub fn observe(&self, observer: impl Fn(&Path) -> String + Send + Sync + 'static) {
        *self.observer.lock().unwrap() = Some(Box::new(observer));
    }
    pub fn moved_paths(&self) -> Vec<PathBuf> {
        self.moved
            .lock()
            .unwrap()
            .iter()
            .map(|(p, _)| p.clone())
            .collect()
    }
    /// Where the Trash keeps what it was given.
    pub fn dir(&self) -> &Path {
        &self.dir
    }
}

impl Trash for FakeTrash {
    fn trash(&self, path: &Path) -> Result<bool> {
        if std::fs::symlink_metadata(path).is_err() {
            return Ok(false);
        }
        if let Some(text) = self.refuse.lock().unwrap().as_deref()
            && path.to_string_lossy().contains(text)
        {
            return Err(Error::InvalidData(format!(
                "{} is in use (test refusal)",
                path.display()
            )));
        }
        std::fs::create_dir_all(&self.dir)?;
        let n = self.next.fetch_add(1, Ordering::SeqCst);
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        std::fs::rename(path, self.dir.join(format!("{n:04}-{name}")))?;
        let note = self
            .observer
            .lock()
            .unwrap()
            .as_ref()
            .map(|observe| observe(path))
            .unwrap_or_default();
        self.moved.lock().unwrap().push((path.to_path_buf(), note));
        Ok(true)
    }
}
