use crate::{
    Result,
    domain::editing::RenderPurpose,
    events::AppEvent,
    mechanisms::{
        backoff::render_backoff,
        canon::canon,
        staleness::{clamp_delay, due_at, is_stale},
    },
    ports::{
        clock::Clock,
        editing::{EditingProvider, RenderCompletion},
        events::EventSink,
        store::Store,
    },
};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

pub trait PreviewFiles: Send + Sync {
    fn copy_adjusted(&self, raw: &str, generation: i64, source: &Path) -> Result<PathBuf>;
    fn remove(&self, path: &Path) -> Result<()>;
}

pub trait ExportStaging: Send + Sync {
    fn completed(&self, render: &RenderCompletion, output: &Path) -> Result<()>;
    /// An export render that will not complete (final timeout, MC-16).
    fn failed(&self, tracking_job_id: i64, reason: &str) -> Result<()>;
}

pub struct PreviewService {
    pub store: Arc<dyn Store>,
    pub editing: Arc<dyn EditingProvider>,
    pub events: Arc<dyn EventSink>,
    pub clock: Arc<dyn Clock>,
    pub files: Arc<dyn PreviewFiles>,
    pub delay: Duration,
    pub export_staging: Option<Arc<dyn ExportStaging>>,
    last_adjusted: Mutex<HashMap<String, Instant>>,
    seen_sidecars: Mutex<HashMap<String, SystemTime>>,
    failed_generation: Mutex<HashMap<String, i64>>,
    retry_after: Mutex<HashMap<String, (u32, Instant)>>,
}

impl PreviewService {
    pub fn new(
        store: Arc<dyn Store>,
        editing: Arc<dyn EditingProvider>,
        events: Arc<dyn EventSink>,
        clock: Arc<dyn Clock>,
        files: Arc<dyn PreviewFiles>,
        delay_setting: Option<&str>,
    ) -> Self {
        Self {
            store,
            editing,
            events,
            clock,
            files,
            delay: clamp_delay(delay_setting),
            export_staging: None,
            last_adjusted: Mutex::new(HashMap::new()),
            seen_sidecars: Mutex::new(HashMap::new()),
            failed_generation: Mutex::new(HashMap::new()),
            retry_after: Mutex::new(HashMap::new()),
        }
    }
    pub fn request_initial(&self, raw: &str) -> Result<()> {
        let Some(generation) = self.store.ensure_initial_generation(raw)? else {
            return Ok(());
        };
        if let Some(state) = self.store.preview_state(raw)?
            && !is_stale(state.adjusted_gen, state.rendered_gen)
            && let Some(path) = state.adjusted_preview_path
        {
            for (photo_id, session_id) in self.store.set_missing_preview_paths(raw, &path)? {
                self.events.emit(AppEvent::PhotoChanged {
                    photo_id,
                    session_id,
                });
            }
            return Ok(());
        }
        self.request(raw, generation)
    }
    fn request(&self, raw: &str, generation: i64) -> Result<()> {
        if self.editing.render_in_flight(raw) {
            return Ok(());
        }
        let now_ms = self.clock.now_ms();
        let tracking = self.store.new_render_job(raw, now_ms)?;
        if let Err(error) = self.editing.request_render(
            raw,
            RenderPurpose::Preview,
            generation,
            tracking,
            self.clock.mono(),
            now_ms,
        ) {
            self.store
                .finish_render_job(tracking, Some("provider"), now_ms)?;
            let mut retry = self
                .retry_after
                .lock()
                .map_err(|e| crate::Error::InvalidData(e.to_string()))?;
            let attempts = retry
                .get(raw)
                .map_or(1, |(previous, _)| previous.saturating_add(1));
            retry.insert(
                raw.into(),
                (attempts, self.clock.mono() + render_backoff(attempts)),
            );
            tracing::warn!(%error, raw, attempts, "preview render deferred");
            return Ok(());
        }
        self.retry_after
            .lock()
            .map_err(|e| crate::Error::InvalidData(e.to_string()))?
            .remove(raw);
        Ok(())
    }
    pub fn on_processing_done(&self, job: &str, raw: &str, outputs: &[String]) -> Result<()> {
        let Some(entry) = self.editing.render_completion(job) else {
            tracing::info!(job, "unknown render completion");
            return Ok(());
        };
        if canon(Path::new(raw)).ok().as_deref() != Some(entry.raw.as_str()) {
            return Ok(());
        }
        let Some(output) = outputs.first() else {
            return Ok(());
        };
        let output = Path::new(output);
        if !self
            .editing
            .accept_render_output(output, entry.issued_at_ms)
        {
            return Ok(());
        }
        match entry.purpose {
            RenderPurpose::Preview => {
                let dest = self
                    .files
                    .copy_adjusted(&entry.raw, entry.generation, output)?;
                let dest_str = dest.to_string_lossy().into_owned();
                let commit =
                    self.store
                        .complete_preview(&entry.raw, entry.generation, &dest_str)?;
                if let Some(prior) = commit.previous_path
                    && prior != dest_str
                {
                    self.files.remove(Path::new(&prior))?;
                }
                for (photo_id, session_id) in commit.photos {
                    self.events.emit(AppEvent::PhotoChanged {
                        photo_id,
                        session_id,
                    });
                }
            }
            RenderPurpose::Export => {
                if let Some(callback) = &self.export_staging {
                    callback.completed(&entry, output)?;
                }
            }
        }
        self.store
            .finish_render_job(entry.tracking_job_id, None, self.clock.now_ms())?;
        self.retry_after
            .lock()
            .map_err(|e| crate::Error::InvalidData(e.to_string()))?
            .remove(&entry.raw);
        self.editing
            .finish_render(job, self.clock.mono(), self.clock.now_ms())?;
        self.schedule_due(false)
    }
    pub fn on_variant_adjusted(&self, _variant_id: &str, raw: &str) -> Result<()> {
        let Ok(raw) = canon(Path::new(raw)) else {
            return Ok(());
        };
        let Some(state) = self.store.preview_state(&raw)? else {
            return Ok(());
        };
        let retouch = state
            .last_own_write_at
            .is_none_or(|own| self.clock.now_ms() > own.saturating_add(30_000));
        let sessions = self
            .store
            .adjusted_event(&raw, retouch)?
            .unwrap_or_default();
        self.failed_generation
            .lock()
            .map_err(|e| crate::Error::InvalidData(e.to_string()))?
            .remove(&raw);
        self.retry_after
            .lock()
            .map_err(|e| crate::Error::InvalidData(e.to_string()))?
            .remove(&raw);
        self.last_adjusted
            .lock()
            .map_err(|e| crate::Error::InvalidData(e.to_string()))?
            .insert(raw, self.clock.mono());
        for session_id in sessions {
            self.events.emit(AppEvent::SessionChanged { session_id });
        }
        Ok(())
    }
    pub fn tick(&self) -> Result<()> {
        for (id, generation) in self
            .editing
            .expire_renders(self.clock.mono(), self.clock.now_ms())?
        {
            let job = self.store.job(id)?;
            if job.kind == "render_export" {
                if let Some(callback) = &self.export_staging {
                    callback.failed(id, "timeout")?;
                }
            } else if self.store.preview_state(&job.target)?.is_some() {
                self.failed_generation
                    .lock()
                    .map_err(|e| crate::Error::InvalidData(e.to_string()))?
                    .insert(job.target, generation);
            }
            self.store
                .finish_render_job(id, Some("timeout"), self.clock.now_ms())?;
        }
        self.schedule_due(false)
    }
    fn schedule_due(&self, force: bool) -> Result<()> {
        let now = self.clock.mono();
        let last = self
            .last_adjusted
            .lock()
            .map_err(|e| crate::Error::InvalidData(e.to_string()))?;
        let failed = self
            .failed_generation
            .lock()
            .map_err(|e| crate::Error::InvalidData(e.to_string()))?;
        let retry = self
            .retry_after
            .lock()
            .map_err(|e| crate::Error::InvalidData(e.to_string()))?;
        let due: Vec<_> = self
            .store
            .preview_states()?
            .into_iter()
            .filter(|state| {
                is_stale(state.adjusted_gen, state.rendered_gen)
                    && failed.get(&state.raw_path) != Some(&state.adjusted_gen)
                    && retry.get(&state.raw_path).is_none_or(|(_, at)| now >= *at)
                    && !self.editing.render_in_flight(&state.raw_path)
                    && (force
                        || last
                            .get(&state.raw_path)
                            .is_none_or(|at| now >= due_at(*at, self.delay)))
            })
            .map(|s| (s.raw_path, s.adjusted_gen))
            .collect();
        drop(retry);
        drop(failed);
        drop(last);
        for (raw, generation) in due {
            if let Err(error) = self.request(&raw, generation) {
                tracing::warn!(%error, raw, "preview render deferred");
            }
        }
        Ok(())
    }
    pub fn on_focus(&self) -> Result<()> {
        self.schedule_due(true)?;
        self.check_sidecars()?;
        self.schedule_due(true)
    }
    pub fn on_startup(&self) -> Result<()> {
        self.check_sidecars()
    }
    fn check_sidecars(&self) -> Result<()> {
        for state in self.store.preview_states()? {
            let Some(mtime) = self.editing.settings_changed_at(&state.raw_path)? else {
                continue;
            };
            let rendered = state
                .adjusted_preview_path
                .as_ref()
                .and_then(|p| Path::new(p).metadata().ok())
                .and_then(|m| m.modified().ok())
                .unwrap_or(UNIX_EPOCH);
            let own = state
                .last_own_write_at
                .map(|ms| {
                    UNIX_EPOCH + Duration::from_millis(ms.max(0) as u64) + Duration::from_secs(30)
                })
                .unwrap_or(UNIX_EPOCH);
            let mut seen = self
                .seen_sidecars
                .lock()
                .map_err(|e| crate::Error::InvalidData(e.to_string()))?;
            if mtime > rendered
                && mtime > own
                && seen.get(&state.raw_path).is_none_or(|prior| mtime > *prior)
            {
                seen.insert(state.raw_path.clone(), mtime);
                drop(seen);
                self.on_variant_adjusted("", &state.raw_path)?;
            }
        }
        Ok(())
    }
}
