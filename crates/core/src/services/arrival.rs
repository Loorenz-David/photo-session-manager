use crate::{
    Result,
    domain::photo::{NewPhoto, PhotoSource},
    events::AppEvent,
    mechanisms::{
        arrival::{attribute, is_candidate, path_key},
        canon::canon,
    },
    ports::{
        camera::CameraProvider, clock::Clock, editing::EditingProvider, events::EventSink,
        store::Store,
    },
};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
    time::{Duration, Instant, UNIX_EPOCH},
};

struct Pending {
    first: (u64, std::time::SystemTime),
    since: Instant,
    next: Instant,
    event_ids: Vec<i64>,
}

pub struct ArrivalService {
    pub store: std::sync::Arc<dyn Store>,
    pub editing: std::sync::Arc<dyn EditingProvider>,
    pub camera: std::sync::Arc<dyn CameraProvider>,
    pub events: std::sync::Arc<dyn EventSink>,
    pub clock: std::sync::Arc<dyn Clock>,
    pub shape_on_arrival: bool,
    capture_root_cache: Mutex<Option<PathBuf>>,
    pending: Mutex<HashMap<PathBuf, Pending>>,
}

impl ArrivalService {
    pub fn new(
        store: std::sync::Arc<dyn Store>,
        editing: std::sync::Arc<dyn EditingProvider>,
        camera: std::sync::Arc<dyn CameraProvider>,
        events: std::sync::Arc<dyn EventSink>,
        clock: std::sync::Arc<dyn Clock>,
    ) -> Self {
        Self {
            store,
            editing,
            camera,
            events,
            clock,
            shape_on_arrival: false,
            capture_root_cache: Mutex::new(None),
            pending: Mutex::new(HashMap::new()),
        }
    }
    pub fn with_shape_on_arrival(mut self, enabled: bool) -> Self {
        self.shape_on_arrival = enabled;
        self
    }
    pub fn capture_root(&self) -> Result<PathBuf> {
        let mut cache = self
            .capture_root_cache
            .lock()
            .map_err(|e| crate::Error::InvalidData(e.to_string()))?;
        if let Some(root) = cache.as_ref() {
            return Ok(root.clone());
        }
        let root = fs::canonicalize(self.editing.inspect_document()?.capture_root)?;
        *cache = Some(root.clone());
        Ok(root)
    }
    fn candidate(&self, path: &Path) -> Result<bool> {
        if !path.is_file() {
            return Ok(false);
        }
        Ok(is_candidate(
            Path::new(&canon(path)?),
            &self.capture_root()?,
            self.camera.raw_extensions(),
        ))
    }
    pub fn on_hook_paths(&self, paths: &[String]) -> Result<()> {
        for path in paths {
            let path = Path::new(path);
            if self.candidate(path)? {
                self.accept(path, PhotoSource::Capture)?;
            }
        }
        Ok(())
    }
    pub fn watcher_event(&self, path: &Path) -> Result<()> {
        self.observe(path, "watcher")
    }
    pub fn scan(&self) -> Result<()> {
        let root = self.capture_root()?;
        let mut candidates = Vec::new();
        for entry in fs::read_dir(&root)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                if entry.file_name().to_string_lossy().starts_with('.')
                    || entry.file_name() == "CaptureOne"
                {
                    continue;
                }
                for file in fs::read_dir(entry.path())? {
                    candidates.push(file?.path());
                }
            } else {
                candidates.push(entry.path());
            }
        }
        let mut first = Vec::new();
        for path in candidates {
            if self.candidate(&path)? {
                let canonical = PathBuf::from(canon(&path)?);
                let event_id = self.store.record_arrival_event(
                    "scan",
                    &canonical.to_string_lossy(),
                    self.clock.now_ms(),
                    None,
                )?;
                let meta = fs::metadata(&canonical)?;
                first.push((canonical, meta.len(), meta.modified()?, event_id));
            }
        }
        if !first.is_empty() {
            std::thread::sleep(Duration::from_secs(1));
        }
        for (path, len, mtime, event_id) in first {
            if let Ok(meta) = fs::metadata(&path) {
                if meta.len() == len && meta.modified()? == mtime {
                    self.accept(&path, PhotoSource::Capture)?;
                } else {
                    let now = self.clock.mono();
                    let mut pending = self
                        .pending
                        .lock()
                        .map_err(|e| crate::Error::InvalidData(e.to_string()))?;
                    if let Some(state) = pending.get_mut(&path) {
                        state.event_ids.push(event_id);
                    } else {
                        pending.insert(
                            path,
                            Pending {
                                first: (meta.len(), meta.modified()?),
                                since: now,
                                next: now + Duration::from_secs(1),
                                event_ids: vec![event_id],
                            },
                        );
                    }
                }
            }
        }
        Ok(())
    }
    fn observe(&self, path: &Path, source: &str) -> Result<()> {
        if !self.candidate(path)? {
            return Ok(());
        }
        let canonical = PathBuf::from(canon(path)?);
        let event_id = self.store.record_arrival_event(
            source,
            &canonical.to_string_lossy(),
            self.clock.now_ms(),
            None,
        )?;
        let metadata = fs::metadata(&canonical)?;
        let now = self.clock.mono();
        let mut pending = self
            .pending
            .lock()
            .map_err(|e| crate::Error::InvalidData(e.to_string()))?;
        if let Some(state) = pending.get_mut(&canonical) {
            state.event_ids.push(event_id);
        } else {
            pending.insert(
                canonical,
                Pending {
                    first: (metadata.len(), metadata.modified()?),
                    since: now,
                    next: now + Duration::from_secs(1),
                    event_ids: vec![event_id],
                },
            );
        }
        Ok(())
    }
    pub fn tick(&self) -> Result<()> {
        let now = self.clock.mono();
        let mut ready = Vec::new();
        let mut timed_out = Vec::new();
        {
            let mut pending = self
                .pending
                .lock()
                .map_err(|e| crate::Error::InvalidData(e.to_string()))?;
            let due: Vec<_> = pending
                .iter()
                .filter(|(_, v)| now >= v.next)
                .map(|(p, _)| p.clone())
                .collect();
            for path in due {
                let Some(state) = pending.get_mut(&path) else {
                    continue;
                };
                if now.duration_since(state.since) >= Duration::from_secs(120) {
                    if let Some(expired) = pending.remove(&path) {
                        timed_out.extend(expired.event_ids);
                    }
                    continue;
                }
                if let Ok(meta) = fs::metadata(&path) {
                    let current = (meta.len(), meta.modified()?);
                    if current == state.first {
                        ready.push(path.clone());
                        pending.remove(&path);
                    } else {
                        state.first = current;
                        state.next = now + Duration::from_secs(1);
                    }
                } else {
                    state.next = now + Duration::from_secs(1);
                }
            }
        }
        for id in timed_out {
            self.store
                .set_capture_event_error(id, "stability_timeout")?;
        }
        for path in ready {
            self.accept(&path, PhotoSource::Capture)?;
        }
        Ok(())
    }
    pub fn import(&self, path: &Path) -> Result<()> {
        if !path.is_file() {
            return Err(crate::Error::InvalidValue(
                "import path is not a file".into(),
            ));
        }
        self.store
            .record_arrival_event("import", &canon(path)?, self.clock.now_ms(), None)?;
        self.accept(path, PhotoSource::Import)
    }
    fn accept(&self, path: &Path, source: PhotoSource) -> Result<()> {
        let canonical = canon(path)?;
        let root = self.capture_root()?;
        let parent = Path::new(&canonical)
            .parent()
            .expect("canonical file has parent");
        let session_id = if source == PhotoSource::Import {
            self.store.active_session_id()?.unwrap_or(1)
        } else {
            attribute(
                parent,
                &root,
                &self.store.session_folders()?,
                self.store.active_session_id()?,
            )
        };
        let metadata = fs::metadata(&canonical)?;
        let captured_at = metadata
            .modified()?
            .duration_since(UNIX_EPOCH)
            .map_err(|e| crate::Error::InvalidData(e.to_string()))?
            .as_millis() as i64;
        let key = path_key(&canonical);
        let (photo, inserted) = self.store.arrival_insert(
            NewPhoto {
                session_id,
                raw_path: canonical,
                raw_hash: None,
                captured_at: Some(captured_at),
                source,
                created_at: self.clock.now_ms(),
            },
            &key,
            self.shape_on_arrival,
        )?;
        if inserted {
            self.events.emit(AppEvent::PhotoAdded {
                photo_id: photo.id,
                session_id,
            });
        }
        Ok(())
    }
}
