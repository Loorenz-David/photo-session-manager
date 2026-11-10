use crate::{
    Result,
    domain::composition::{Fingerprint, FrameSource},
    domain::editing::RenderPurpose,
    domain::photo::{Classification, Integrity},
    events::AppEvent,
    mechanisms::{
        backoff::render_backoff,
        canon::canon,
        missing::{
            MissingDecision, RenderEligibility, file_missing, missing_decision, render_eligibility,
        },
        staleness::{clamp_delay, due_at, is_stale},
    },
    ports::{
        clock::Clock,
        editing::{EditingProvider, RenderCompletion},
        events::EventSink,
        images::Imaging,
        store::Store,
    },
    services::{health::HealthService, integrity::IntegrityCheck},
};
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{Duration, Instant, UNIX_EPOCH},
};

pub trait PreviewFiles: Send + Sync {
    fn copy_adjusted(&self, raw: &str, generation: i64, source: &Path) -> Result<PathBuf>;
    fn thumbnail_path(&self, raw: &str, generation: i64) -> PathBuf;
    fn remove(&self, path: &Path) -> Result<()>;
}

pub trait ExportStaging: Send + Sync {
    /// Capture One finished a master render (10D MC-28 §Master).
    fn master_ready(&self, render: &RenderCompletion, output: &Path) -> Result<()>;
    /// A master render that will not complete (final timeout, MC-16).
    fn failed(&self, tracking_job_id: i64, reason: &str) -> Result<()>;
}

type BeforeRender = Arc<dyn Fn(&str) -> Result<()> + Send + Sync>;

/// The settings fingerprint recorded for a RAW whose sidecar did not exist at that moment.
pub const NO_SIDECAR: &str = "none";
/// Health code of the photos whose preview render failed for good (P56 C3(b)).
pub const PREVIEW_NOT_RENDERED: &str = "preview_not_rendered";
/// Health code (and the photos' reason) for RAWs deleted outside the app (changelog 43, P60).
pub const FILE_MISSING: &str = "file_missing";

/// Whether an export is named as a JPEG (the preview recipe's format).
fn is_jpeg_name(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("jpg") || e.eq_ignore_ascii_case("jpeg"))
}

/// Whether a RAW is gone; an unreadable path counts as present (the render then says why).
fn raw_missing(raw: &str) -> bool {
    file_missing(Path::new(raw)).unwrap_or_else(|error| {
        tracing::warn!(%error, raw, "RAW not checked");
        false
    })
}

pub struct PreviewService {
    pub store: Arc<dyn Store>,
    pub editing: Arc<dyn EditingProvider>,
    pub events: Arc<dyn EventSink>,
    pub clock: Arc<dyn Clock>,
    pub files: Arc<dyn PreviewFiles>,
    pub imaging: Arc<dyn Imaging>,
    pub delay: Duration,
    pub export_staging: Option<Arc<dyn ExportStaging>>,
    /// Where a preview that cannot be rendered is reported (P56 C3(b)); `None` in bare services.
    pub health: Option<Arc<HealthService>>,
    last_adjusted: Mutex<HashMap<String, Instant>>,
    failed_generation: Mutex<HashMap<String, i64>>,
    retry_after: Mutex<HashMap<String, (u32, Instant)>>,
    variant_first_request: Mutex<HashMap<String, Instant>>,
    variant_waiting: Mutex<HashSet<String>>,
    /// New captures whose first preview request has not reached the provider yet (P58): they
    /// keep their place before the backlog through deferrals and the wait for the variant.
    arrivals: Mutex<HashSet<String>>,
    before_render: Mutex<Option<BeforeRender>>,
    /// "Take the camera back" holds Capture One (P60 #22): no render request leaves meanwhile.
    render_hold: Mutex<Option<RenderHold>>,
    /// The RAWs the last missing-file check found gone (changelog 43).
    known_missing: Mutex<HashSet<String>>,
    /// 10F §2: where a copied preview export is noted for the storage clean-up.
    pub copied_exports: Option<Arc<crate::services::storage::CopiedExports>>,
}

type RenderHold = Arc<dyn Fn() -> bool + Send + Sync>;

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
            imaging: Arc::new(crate::ports::images::NoImaging),
            delay: clamp_delay(delay_setting),
            export_staging: None,
            health: None,
            last_adjusted: Mutex::new(HashMap::new()),
            failed_generation: Mutex::new(HashMap::new()),
            retry_after: Mutex::new(HashMap::new()),
            variant_first_request: Mutex::new(HashMap::new()),
            variant_waiting: Mutex::new(HashSet::new()),
            arrivals: Mutex::new(HashSet::new()),
            before_render: Mutex::new(None),
            render_hold: Mutex::new(None),
            known_missing: Mutex::new(HashSet::new()),
            copied_exports: None,
        }
    }
    /// Installs the take-back's hold (`true` while it holds Capture One).
    pub fn set_render_hold(&self, held: RenderHold) {
        if let Ok(mut slot) = self.render_hold.lock() {
            *slot = Some(held);
        }
    }
    fn held(&self) -> bool {
        self.render_hold
            .lock()
            .ok()
            .and_then(|slot| slot.clone())
            .is_some_and(|held| held())
    }
    /// Changelog 43: a photo whose RAW is gone is missing (`Integrity::RawMissing`; classification
    /// and history unchanged) and leaves the session views; a RAW back at the same path brings its
    /// photos back as they were. Health counts the missing photos that were not deleted. Runs at
    /// startup, on window focus, when Capture One (re)opens and when a request finds a RAW gone.
    pub fn check_missing_files(&self) -> Result<()> {
        let mut missing = 0usize;
        let mut gone_raws = HashSet::new();
        // P62: a RAW the app cleared after publishing is not missing (10F §3).
        let cleared = crate::services::retention::cleared_sessions(self.store.as_ref())?;
        for state in self.store.preview_states()? {
            let gone = match file_missing(Path::new(&state.raw_path)) {
                Ok(gone) => gone,
                Err(error) => {
                    tracing::warn!(%error, raw = state.raw_path, "RAW not checked");
                    continue;
                }
            };
            for photo in self.store.photos_for_raw(&state.raw_path)? {
                if cleared.contains(&photo.session_id) {
                    continue;
                }
                let now = match missing_decision(gone, photo.integrity) {
                    MissingDecision::Keep => photo.integrity,
                    MissingDecision::Missing => {
                        self.store
                            .set_photo_integrity(photo.id, Integrity::RawMissing)?;
                        tracing::info!(raw = state.raw_path, photo = photo.id, "RAW missing");
                        Integrity::RawMissing
                    }
                    MissingDecision::Returned => {
                        let back = IntegrityCheck::new(self.store.as_ref()).verify(photo.id)?;
                        tracing::info!(raw = state.raw_path, photo = photo.id, "RAW back");
                        back
                    }
                };
                if now != photo.integrity {
                    self.events.emit(AppEvent::PhotoChanged {
                        photo_id: photo.id,
                        session_id: photo.session_id,
                    });
                    self.events.emit(AppEvent::SessionChanged {
                        session_id: photo.session_id,
                    });
                }
                if now == Integrity::RawMissing && photo.classification != Classification::Deleted {
                    missing += 1;
                }
            }
            if gone {
                self.forget(&state.raw_path);
                gone_raws.insert(state.raw_path);
            }
        }
        if let Ok(mut known) = self.known_missing.lock() {
            *known = gone_raws;
        }
        if let Some(health) = &self.health {
            if missing == 0 {
                health.clear(FILE_MISSING);
            } else {
                health.set(
                    FILE_MISSING,
                    format!("{missing} photos' files were deleted outside Session Manager"),
                );
            }
        }
        Ok(())
    }
    /// Drops every waiting, backoff and failure mark of a RAW that is gone.
    fn forget(&self, raw: &str) {
        if let Ok(mut map) = self.retry_after.lock() {
            map.remove(raw);
        }
        if let Ok(mut set) = self.arrivals.lock() {
            set.remove(raw);
        }
        if let Ok(mut set) = self.variant_waiting.lock() {
            set.remove(raw);
        }
        if let Ok(mut map) = self.variant_first_request.lock() {
            map.remove(raw);
        }
        let failed = self
            .failed_generation
            .lock()
            .is_ok_and(|mut map| map.remove(raw).is_some());
        if failed {
            self.report_failures();
        }
    }
    /// Capture One (re)opened: missing files are found before anything is asked of it, and only
    /// photos that need a preview are requested (P60 #26).
    pub fn on_editor_opened(&self) -> Result<()> {
        self.check_missing_files()?;
        self.schedule_due(false)
    }
    pub fn request_initial(&self, raw: &str) -> Result<()> {
        if raw_missing(raw) {
            return self.check_missing_files();
        }
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
        self.request(raw, generation, true)
    }
    fn request(&self, raw: &str, generation: i64, new_arrival: bool) -> Result<()> {
        if raw_missing(raw) {
            return self.check_missing_files();
        }
        let new_arrival = {
            let mut arrivals = self
                .arrivals
                .lock()
                .map_err(|e| crate::Error::InvalidData(e.to_string()))?;
            if new_arrival {
                arrivals.insert(raw.into());
            }
            arrivals.contains(raw)
        };
        if self
            .variant_waiting
            .lock()
            .map_err(|e| crate::Error::InvalidData(e.to_string()))?
            .contains(raw)
        {
            return Ok(());
        }
        // The take-back holds Capture One: the request waits, keeping its place (an arrival stays
        // an arrival); the stale state is asked for again once the hold ends.
        if self.held() || self.editing.render_in_flight(raw) {
            return Ok(());
        }
        let now_ms = self.clock.now_ms();
        let tracking = self.store.new_render_job(raw, now_ms)?;
        let prepare = self
            .before_render
            .lock()
            .map_err(|e| crate::Error::InvalidData(e.to_string()))?
            .clone();
        let result = prepare
            .map_or(Ok(()), |prepare| prepare(raw))
            .and_then(|_| {
                self.editing.request_render(
                    raw,
                    RenderPurpose::Preview,
                    generation,
                    tracking,
                    self.clock.mono(),
                    now_ms,
                    new_arrival,
                )
            });
        if let Err(error) = result {
            self.store
                .finish_render_job(tracking, Some("provider"), now_ms)?;
            return self.defer(raw, &error);
        }
        self.arrivals
            .lock()
            .map_err(|e| crate::Error::InvalidData(e.to_string()))?
            .remove(raw);
        self.retry_after
            .lock()
            .map_err(|e| crate::Error::InvalidData(e.to_string()))?
            .remove(raw);
        self.variant_first_request
            .lock()
            .map_err(|e| crate::Error::InvalidData(e.to_string()))?
            .remove(raw);
        Ok(())
    }
    /// The render backoff after Capture One refused a preview request (MC-16).
    fn defer(&self, raw: &str, error: &crate::Error) -> Result<()> {
        // Capture One answering `variant_not_found` (or any refusal) for a RAW that is gone: the
        // photo is missing, never asked for again (changelog 43).
        if raw_missing(raw) {
            return self.check_missing_files();
        }
        // Capture One is held or waiting to be relaunched: asked again shortly, not counted.
        if matches!(error, crate::Error::EditorLaunchDeferred) {
            let mut retry = self
                .retry_after
                .lock()
                .map_err(|e| crate::Error::InvalidData(e.to_string()))?;
            let attempts = retry.get(raw).map_or(0, |(attempts, _)| *attempts);
            retry.insert(
                raw.into(),
                (attempts, self.clock.mono() + Duration::from_secs(1)),
            );
            return Ok(());
        }
        {
            let mut retry = self
                .retry_after
                .lock()
                .map_err(|e| crate::Error::InvalidData(e.to_string()))?;
            let mut attempts = retry
                .get(raw)
                .map_or(1, |(previous, _)| previous.saturating_add(1));
            if matches!(error, crate::Error::VariantNotFound) {
                let first = *self
                    .variant_first_request
                    .lock()
                    .map_err(|e| crate::Error::InvalidData(e.to_string()))?
                    .entry(raw.into())
                    .or_insert_with(|| self.clock.mono());
                if self.clock.mono().saturating_duration_since(first) < Duration::from_secs(15) {
                    attempts = 1;
                }
            }
            retry.insert(
                raw.into(),
                (attempts, self.clock.mono() + render_backoff(attempts)),
            );
            tracing::warn!(%error, raw, attempts, "preview render deferred");
        }
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
        let Some(reported) = outputs.first() else {
            return Ok(());
        };
        // 10A MC-16 §Completion: the reported path, or Capture One's suffixed copy beside it.
        let Some(output) = self
            .editing
            .resolve_render_output(Path::new(reported), entry.issued_at_ms)
        else {
            tracing::info!(
                job,
                reported,
                "render output not found; waiting for the timeout"
            );
            return Ok(());
        };
        let output = output.as_path();
        // 10F §5: a preview export that is not a JPEG came from a recipe changed outside the app.
        // It is never taken as a preview: the recipe is repaired and the render sent once more.
        if entry.purpose == RenderPurpose::Preview && !is_jpeg_name(output) {
            let now_ms = self.clock.now_ms();
            match self.editing.recipe_fault(job, self.clock.mono(), now_ms) {
                Ok(true) => return Ok(()),
                outcome => {
                    if let Err(error) = outcome {
                        tracing::warn!(%error, job, "recipe not repaired");
                    }
                    self.failed_generation
                        .lock()
                        .map_err(|e| crate::Error::InvalidData(e.to_string()))?
                        .insert(entry.raw.clone(), entry.generation);
                    self.report_failures();
                    self.store
                        .finish_render_job(entry.tracking_job_id, Some("recipe"), now_ms)?;
                    self.editing.finish_render(job, self.clock.mono(), now_ms)?;
                    return Ok(());
                }
            }
        }
        match entry.purpose {
            RenderPurpose::Preview => {
                let dest = self
                    .files
                    .copy_adjusted(&entry.raw, entry.generation, output)?;
                if let Some(copied) = &self.copied_exports {
                    copied.note(output, &dest);
                }
                let dest_str = dest.to_string_lossy().into_owned();
                let frame = self.imaging.dimensions(&dest)?;
                let thumb = self.files.thumbnail_path(&entry.raw, entry.generation);
                let made = self.imaging.thumbnail(&dest, &thumb, 640)?;
                let thumb_str = thumb.to_string_lossy().into_owned();
                let observed = frame.map(|frame| Fingerprint {
                    frame,
                    source: FrameSource::Preview,
                    geometry: entry.geometry.clone(),
                });
                let commit = self.store.complete_preview(
                    &entry.raw,
                    entry.generation,
                    &dest_str,
                    made.then_some(thumb_str.as_str()),
                    observed,
                )?;
                if let Some(prior) = commit.previous_path
                    && prior != dest_str
                {
                    self.files.remove(Path::new(&prior))?;
                }
                if let Some(prior) = commit.previous_thumbnail
                    && (!made || prior != thumb_str)
                {
                    self.files.remove(Path::new(&prior))?;
                }
                for (photo_id, session_id) in commit.photos {
                    self.events.emit(AppEvent::PhotoChanged {
                        photo_id,
                        session_id,
                    });
                }
                // The sidecar as Capture One left it for this render: a later rewrite with the
                // same adjustments (its processing of this export, a save) is not an edit.
                self.record_sidecar(&entry.raw);
                self.failed_generation
                    .lock()
                    .map_err(|e| crate::Error::InvalidData(e.to_string()))?
                    .remove(&entry.raw);
                self.report_failures();
            }
            RenderPurpose::Master => {
                if let Some(callback) = &self.export_staging {
                    callback.master_ready(&entry, output)?;
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
        self.report_failures();
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
        if self.held() {
            return Ok(());
        }
        for (id, generation) in self
            .editing
            .expire_renders(self.clock.mono(), self.clock.now_ms())?
        {
            let job = self.store.job(id)?;
            if job.kind == "render_master" {
                if let Some(callback) = &self.export_staging {
                    callback.failed(id, "timeout")?;
                }
            } else if self.store.preview_state(&job.target)?.is_some() {
                // MC-16: issued, re-issued once, no output either time. Not ordered again until
                // the photo is adjusted (MC-17), and the photographer is told.
                tracing::warn!(raw = job.target, generation, "preview not rendered");
                self.failed_generation
                    .lock()
                    .map_err(|e| crate::Error::InvalidData(e.to_string()))?
                    .insert(job.target, generation);
                self.report_failures();
            }
            self.store
                .finish_render_job(id, Some("timeout"), self.clock.now_ms())?;
        }
        // P58: a queued request Capture One refused when its turn came ends like a request
        // refused at once: a master fails its export, a preview is asked for again after backoff.
        for (id, error) in self.editing.refused_renders() {
            let job = self.store.job(id)?;
            self.store
                .finish_render_job(id, Some("provider"), self.clock.now_ms())?;
            if job.kind == "render_master" {
                if let Some(callback) = &self.export_staging {
                    callback.failed(id, &error)?;
                }
            } else {
                self.defer(&job.target, &crate::Error::Provider(error))?;
            }
        }
        self.schedule_due(false)
    }
    /// Health item naming the photos whose last preview render failed (cleared when none is left).
    fn report_failures(&self) {
        let Some(health) = &self.health else {
            return;
        };
        let mut names: Vec<String> = self
            .failed_generation
            .lock()
            .map(|failed| {
                failed
                    .keys()
                    .map(|raw| {
                        Path::new(raw)
                            .file_name()
                            .map_or_else(|| raw.clone(), |n| n.to_string_lossy().into_owned())
                    })
                    .collect()
            })
            .unwrap_or_default();
        names.sort();
        match names.as_slice() {
            [] => health.clear(PREVIEW_NOT_RENDERED),
            [one] => health.set(
                PREVIEW_NOT_RENDERED,
                format!("Capture One produced no preview for {one}"),
            ),
            many => health.set(
                PREVIEW_NOT_RENDERED,
                format!(
                    "Capture One produced no preview for {} photos ({})",
                    many.len(),
                    many.join(", ")
                ),
            ),
        }
    }
    /// Records the RAW's current settings-sidecar fingerprint (`NO_SIDECAR` without one).
    fn record_sidecar(&self, raw: &str) {
        let fingerprint = match self.editing.settings_sidecar(raw) {
            Ok(Some(sidecar)) => sidecar.fingerprint,
            Ok(None) => NO_SIDECAR.into(),
            Err(error) => {
                tracing::warn!(%error, raw, "settings sidecar not read");
                return;
            }
        };
        if let Err(error) = self.store.set_settings_fingerprint(raw, &fingerprint) {
            tracing::warn!(%error, raw, "settings fingerprint not recorded");
        }
    }
    pub fn retry_now(&self, raw: &str) -> Result<()> {
        self.retry_after
            .lock()
            .map_err(|e| crate::Error::InvalidData(e.to_string()))?
            .remove(raw);
        self.schedule_due(false)
    }
    pub fn wait_for_variant(&self, raw: &str) {
        if let Ok(mut arrivals) = self.arrivals.lock() {
            arrivals.insert(raw.into());
        }
        if let Ok(mut first) = self.variant_first_request.lock() {
            first.entry(raw.into()).or_insert_with(|| self.clock.mono());
        }
        if let Ok(mut waiting) = self.variant_waiting.lock() {
            waiting.insert(raw.into());
        }
    }
    pub fn set_before_render(&self, prepare: BeforeRender) {
        if let Ok(mut callback) = self.before_render.lock() {
            *callback = Some(prepare);
        }
    }
    pub fn variant_ready(&self, raw: &str) -> Result<()> {
        self.variant_waiting
            .lock()
            .map_err(|e| crate::Error::InvalidData(e.to_string()))?
            .remove(raw);
        self.retry_now(raw)
    }
    pub fn failed_generation_for(&self, raw: &str) -> Option<i64> {
        self.failed_generation.lock().ok()?.get(raw).copied()
    }
    /// The photo on the stage or in the editor (P58, changelog 39): its pending preview is the
    /// next one sent to the editing provider. `None` when no photo is shown.
    pub fn set_focus(&self, photo_id: Option<i64>) -> Result<()> {
        let raw = photo_id
            .map(|id| self.store.photo(id).map(|photo| photo.raw_path))
            .transpose()?;
        self.editing.set_render_focus(raw.as_deref());
        self.schedule_due(false)
    }
    fn schedule_due(&self, force: bool) -> Result<()> {
        let held = self.held();
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
        let known_missing = self
            .known_missing
            .lock()
            .map_err(|e| crate::Error::InvalidData(e.to_string()))?
            .clone();
        let mut disk_changed = false;
        let due: Vec<_> = self
            .store
            .preview_states()?
            .into_iter()
            .filter(|state| {
                // Only a stale RAW is looked at on disk. One deleted (or back) since the last check
                // is found here, before Capture One is ever asked for it (changelog 43).
                if !is_stale(state.adjusted_gen, state.rendered_gen) {
                    return false;
                }
                let gone = raw_missing(&state.raw_path);
                if gone != known_missing.contains(&state.raw_path) {
                    disk_changed = true;
                }
                render_eligibility(gone, state.adjusted_gen, state.rendered_gen, held)
                    == RenderEligibility::Request
                    && failed.get(&state.raw_path) != Some(&state.adjusted_gen)
                    && retry.get(&state.raw_path).is_none_or(|(_, at)| now >= *at)
                    && !self
                        .variant_waiting
                        .lock()
                        .is_ok_and(|waiting| waiting.contains(&state.raw_path))
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
        if disk_changed {
            self.check_missing_files()?;
        }
        for (raw, generation) in due {
            if let Err(error) = self.request(&raw, generation, false) {
                tracing::warn!(%error, raw, "preview render deferred");
            }
        }
        Ok(())
    }
    pub fn on_focus(&self) -> Result<()> {
        self.check_missing_files()?;
        self.schedule_due(true)?;
        self.check_sidecars()?;
        self.schedule_due(true)
    }
    pub fn on_startup(&self) -> Result<()> {
        self.check_missing_files()?;
        self.check_sidecars()
    }
    /// MC-17 §Fallback, P56: a RAW whose sidecar holds other adjustments than at its last render
    /// or check, and was written after that render and outside the own-write window, gets one
    /// adjusted event. A rewrite with the same adjustments (Capture One processing an export the
    /// app ordered, a document save) is never an edit; a RAW with no recorded fingerprint (before
    /// P56) adopts the current one.
    fn check_sidecars(&self) -> Result<()> {
        for state in self.store.preview_states()? {
            if raw_missing(&state.raw_path) {
                continue;
            }
            let Some(sidecar) = self.editing.settings_sidecar(&state.raw_path)? else {
                continue;
            };
            let Some(recorded) = state.settings_fingerprint.as_deref() else {
                self.store
                    .set_settings_fingerprint(&state.raw_path, &sidecar.fingerprint)?;
                continue;
            };
            if recorded == sidecar.fingerprint {
                continue;
            }
            let mtime = sidecar.modified;
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
            self.store
                .set_settings_fingerprint(&state.raw_path, &sidecar.fingerprint)?;
            if mtime > rendered && mtime > own {
                self.on_variant_adjusted("", &state.raw_path)?;
            }
        }
        Ok(())
    }
}
