use super::{
    recipes::RecipeSpec,
    renders::{PendingRender, RenderEntry, RenderRegistry, resolve_output},
    scripting::{CaptureOneScripting, ProcessStarted},
};
use psm_core::{
    Error, Result,
    domain::{
        editing::{
            DocumentState, EditingCapabilities, ProvisionInputs, ProvisionReport, RenderPurpose,
            StyleOutcome,
        },
        photo::Classification,
    },
    mechanisms::canon::canon,
    ports::{
        editing::{EditingProvider, RenderCompletion, SettingsSidecar},
        store::Store,
    },
    services::health::HealthService,
    services::previews::FILE_MISSING,
};
use psm_infra::{
    disk::{FreeSpace, free_space},
    paths::AppPaths,
};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, RwLock},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

/// Below this much free space on the session's disk no master (full-size TIFF, ~80 MB) render is
/// ordered (P56 C3(d); P52 F12: 64 GB of copies filled the disk in one afternoon).
pub const MASTER_MIN_FREE_BYTES: u64 = 5_000_000_000;
/// Health code while master renders are paused for disk space.
pub const DISK_SPACE_LOW: &str = "disk_space_low";

pub type FreeSpaceFn = Arc<dyn Fn(&Path) -> std::io::Result<FreeSpace> + Send + Sync>;

/// The free-space source of the master-render guard: the disk itself, or a test's stand-in. One
/// per app process, carried with the scripting lanes through every rebuild.
pub struct DiskSpace(RwLock<FreeSpaceFn>);
impl Default for DiskSpace {
    fn default() -> Self {
        Self(RwLock::new(Arc::new(free_space)))
    }
}
impl DiskSpace {
    pub fn set(&self, source: FreeSpaceFn) {
        if let Ok(mut slot) = self.0.write() {
            *slot = source;
        }
    }
    pub fn query(&self, path: &Path) -> std::io::Result<FreeSpace> {
        let source = self
            .0
            .read()
            .map_err(|e| std::io::Error::other(e.to_string()))?
            .clone();
        source(path)
    }
}

pub struct CaptureOneAdapter {
    pub user: Arc<dyn CaptureOneScripting>,
    pub background: Arc<dyn CaptureOneScripting>,
    pub store: Arc<dyn Store>,
    pub health: Arc<HealthService>,
    pub document_id: String,
    pub preview_recipe: String,
    pub master_recipe: String,
    pub paths: AppPaths,
    renders: Mutex<RenderRegistry>,
    pub(crate) provision_lock: Mutex<()>,
    disk: Arc<DiskSpace>,
    /// The document was chosen in Settings (not the app's own): when it is missing it is reported,
    /// never recreated (P63 #27).
    pub(crate) document_configured: bool,
}
impl CaptureOneAdapter {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        lanes: (Arc<dyn CaptureOneScripting>, Arc<dyn CaptureOneScripting>),
        store: Arc<dyn Store>,
        health: Arc<HealthService>,
        document_id: String,
        preview_recipe: String,
        master_recipe: String,
        paths: AppPaths,
    ) -> Self {
        Self {
            user: lanes.0,
            background: lanes.1,
            store,
            health,
            document_id,
            preview_recipe,
            master_recipe,
            paths,
            renders: Mutex::new(RenderRegistry::default()),
            provision_lock: Mutex::new(()),
            disk: Arc::new(DiskSpace::default()),
            document_configured: false,
        }
    }
    pub fn with_disk(mut self, disk: Arc<DiskSpace>) -> Self {
        self.disk = disk;
        self
    }
    /// The document was chosen in Settings (P63 #27).
    pub fn with_configured_document(mut self, configured: bool) -> Self {
        self.document_configured = configured;
        self
    }
    /// P56 C3(d): a master render needs 5 GB free on the disk holding the session (Capture One's
    /// output location defaults to the session's `Output/`). Below that it is refused and the health
    /// item names the disk; nothing is deleted. An unreadable disk never blocks a render.
    fn check_master_space(&self) -> Result<()> {
        let space = match self.disk.query(Path::new(&self.document_id)) {
            Ok(space) => space,
            Err(error) => {
                tracing::warn!(%error, "free disk space not read");
                return Ok(());
            }
        };
        if space.available_bytes >= MASTER_MIN_FREE_BYTES {
            self.health.clear(DISK_SPACE_LOW);
            return Ok(());
        }
        let gb = space.available_bytes as f64 / 1e9;
        self.health.set(
            DISK_SPACE_LOW,
            format!(
                "Only {gb:.1} GB free on {}. Full-size renders for export are paused until 5 GB are free.",
                space.disk
            ),
        );
        Err(Error::Provider(format!(
            "not enough disk space: {gb:.1} GB free on {}",
            space.disk
        )))
    }
    fn variant_background(&self, raw: &str) -> Result<String> {
        if let Some(id) = self.background.find_image(&self.document_id, raw)? {
            return Ok(id);
        }
        let folder = Path::new(raw)
            .parent()
            .ok_or(Error::VariantNotFound)?
            .to_string_lossy();
        self.background
            .find_image_in_folder(&self.document_id, &folder, raw)?
            .ok_or(Error::VariantNotFound)
    }
    fn apply_folder(&self, target: &str) -> Result<()> {
        self.user.save(&self.document_id)?;
        fs::create_dir_all(target)?;
        self.user.set_captures(&self.document_id, target)?;
        let readback = self.user.get_captures(&self.document_id)?;
        if !same_path(&readback, target) {
            return Err(Error::CaptureFolderNotSet);
        }
        Ok(())
    }
    /// Sends every queued render that can go now (P58, changelog 39: masters as soon as their RAW
    /// is free; one preview at a time, the focused photo's, then the newest arrivals, then the
    /// rest oldest-first). A request Capture One refuses is handed back through `refused_renders`.
    fn pump(&self, now: Instant, now_ms: i64) -> Result<()> {
        self.discard_missing();
        while self.send_next(now, now_ms)? {}
        let mut state = self
            .renders
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        if state.jobs.is_empty() && state.pending.is_empty() && state.busy.is_empty() {
            // The batch is over: the next render checks its recipe again (10F §5).
            state.recipes_checked.clear();
        }
        Ok(())
    }
    /// Changelog 43: a render whose RAW was deleted outside the app is dropped from the queue and
    /// from the renders out (it would never finish and would hold the take-back), handed back as
    /// refused with `file_missing`.
    fn discard_missing(&self) {
        let Ok(mut state) = self.renders.lock() else {
            return;
        };
        let gone: Vec<_> = state
            .jobs
            .iter()
            .filter(|(_, entry)| raw_missing(&entry.completion.raw))
            .map(|(id, entry)| (id.clone(), entry.completion.clone()))
            .collect();
        for (id, completion) in gone {
            state.jobs.remove(&id);
            state.release(&completion.raw, completion.purpose);
            state
                .refused
                .push((completion.tracking_job_id, FILE_MISSING.into()));
        }
        let pending = std::mem::take(&mut state.pending);
        for entry in pending {
            if raw_missing(&entry.raw) {
                state
                    .refused
                    .push((entry.tracking_job_id, FILE_MISSING.into()));
            } else {
                state.pending.push_back(entry);
            }
        }
    }
    /// Sends the next queued render that can go; `false` when there is none.
    fn send_next(&self, now: Instant, now_ms: i64) -> Result<bool> {
        let next = {
            let mut state = self
                .renders
                .lock()
                .map_err(|e| Error::InvalidData(e.to_string()))?;
            let Some(next) = state.pop_next() else {
                return Ok(false);
            };
            state.claim(&next.raw, next.purpose);
            next
        };
        let result = self.render_started(&next.raw, next.purpose);
        let mut state = self
            .renders
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        match result {
            Ok(started) => {
                let job_id = started.job_id;
                state.jobs.insert(
                    job_id.clone(),
                    RenderEntry {
                        completion: RenderCompletion {
                            job_id,
                            raw: next.raw,
                            purpose: next.purpose,
                            generation: next.generation,
                            issued_at_ms: now_ms,
                            tracking_job_id: next.tracking_job_id,
                            geometry: started.geometry,
                        },
                        issued_mono: now,
                        attempts: 0,
                        recipe_retried: false,
                    },
                );
                Ok(true)
            }
            // P60 #22: Capture One is held by the take-back (or waits for the camera helper): the
            // render keeps its place at the head of the queue and nothing more is sent for now.
            Err(Error::EditorLaunchDeferred) => {
                state.release(&next.raw, next.purpose);
                state.pending.push_front(next);
                Ok(false)
            }
            Err(error) => {
                tracing::warn!(%error, raw = next.raw, "queued render refused");
                state.release(&next.raw, next.purpose);
                state
                    .refused
                    .push((next.tracking_job_id, error.to_string()));
                Ok(true)
            }
        }
    }
    fn recipe_for(&self, purpose: RenderPurpose) -> (&String, RecipeSpec) {
        match purpose {
            RenderPurpose::Preview => (&self.preview_recipe, RecipeSpec::PREVIEW),
            RenderPurpose::Master => (&self.master_recipe, RecipeSpec::MASTER),
        }
    }
    /// 10F §5: reads the render's recipe back and repairs it (as at startup). `RecipeMissing`
    /// when it cannot be created or repaired.
    fn repair_recipe(&self, purpose: RenderPurpose) -> Result<()> {
        let (name, spec) = self.recipe_for(purpose);
        let want = spec.props();
        let checked = match self
            .background
            .ensure_recipe(&self.document_id, name, &want)
        {
            Err(Error::EditorLaunchDeferred) => return Err(Error::EditorLaunchDeferred),
            Err(error) => {
                tracing::warn!(%error, recipe = name, "recipe not repaired");
                return Err(Error::RecipeMissing { name: name.clone() });
            }
            Ok(checked) => checked,
        };
        if !checked.changed.is_empty() || checked.created {
            tracing::info!(recipe = name, created = checked.created, changed = ?checked.changed, "recipe repaired before rendering");
        }
        let mut got = checked.props;
        got.root_folder_type = None;
        if got != want {
            return Err(Error::RecipeMissing { name: name.clone() });
        }
        Ok(())
    }
    fn render_started(&self, raw: &str, purpose: RenderPurpose) -> Result<ProcessStarted> {
        let recipe = self.recipe_for(purpose).0;
        if purpose == RenderPurpose::Master {
            self.check_master_space()?;
        }
        // The variant first: its lookup never launches Capture One (P60 #22).
        let variant = self.variant_background(raw)?;
        // 10F §5: the recipe is checked and repaired once per render batch; a recipe changed or
        // removed by hand meanwhile is repaired before the next batch reaches Capture One.
        let checked = self
            .renders
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?
            .recipes_checked
            .contains(&purpose);
        if !checked {
            self.repair_recipe(purpose)?;
            self.renders
                .lock()
                .map_err(|e| Error::InvalidData(e.to_string()))?
                .recipes_checked
                .insert(purpose);
        }
        match self.background.process(&self.document_id, &variant, recipe) {
            Err(error)
                if !matches!(
                    error,
                    Error::EditorLaunchDeferred | Error::VariantNotFound | Error::Timeout
                ) && matches!(
                    self.background.recipe_exists(&self.document_id, recipe),
                    Ok(false)
                ) =>
            {
                // Refused because the recipe went missing mid-batch: one repair, one retry.
                tracing::warn!(%error, recipe, "render refused: recipe missing; repairing once");
                self.repair_recipe(purpose)?;
                self.background.process(&self.document_id, &variant, recipe)
            }
            result => result,
        }
    }
}
/// Whether a RAW is gone (changelog 43); an unreadable path counts as present.
fn raw_missing(raw: &str) -> bool {
    psm_core::mechanisms::missing::file_missing(Path::new(raw)).unwrap_or(false)
}
pub fn capture_root(document_id: &str) -> Result<PathBuf> {
    let root = Path::new(document_id).join("Capture");
    if !root.is_dir() {
        return Err(Error::CaptureFolderNotSet);
    }
    Ok(PathBuf::from(canon(&root)?))
}
fn same_path(a: &str, b: &str) -> bool {
    match (canon(Path::new(a)), canon(Path::new(b))) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    }
}
/// SHA-256 over the adjustment lists (`<DL>…</DL>`, every variant) of a `.cos` sidecar, or over
/// the whole text when it has none. Capture One rewrites the sidecar when it processes a variant
/// or saves the document, changing only bookkeeping outside those lists (`LowResProcessCounter`,
/// `LastMetadataModificationDate`); an edit changes a list (P56, P52 F12).
pub fn settings_fingerprint(text: &str) -> String {
    let mut lists = Vec::new();
    let mut rest = text;
    while let Some(start) = rest.find("<DL>") {
        let body = &rest[start..];
        let end = body.find("</DL>").map_or(body.len(), |i| i + "</DL>".len());
        lists.push(&body[..end]);
        rest = &body[end..];
    }
    let mut hasher = Sha256::new();
    if lists.is_empty() {
        hasher.update(text.as_bytes());
    } else {
        for list in lists {
            hasher.update(list.as_bytes());
        }
    }
    hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
pub fn tag_for(classification: Classification) -> i32 {
    match classification {
        Classification::Main => 4,
        Classification::Condition => 5,
        Classification::Deleted => 1,
        Classification::Unreviewed => 0,
    }
}
impl EditingProvider for CaptureOneAdapter {
    fn ensure(&self, inputs: &ProvisionInputs) -> Result<ProvisionReport> {
        self.ensure_provision(inputs)
    }
    fn bring_document_to_front(&self) -> Result<()> {
        self.front_provision_document()
    }
    fn settings_sidecar(&self, raw: &str) -> Result<Option<SettingsSidecar>> {
        let raw = Path::new(raw);
        let (Some(parent), Some(name)) = (raw.parent(), raw.file_name()) else {
            return Ok(None);
        };
        let sidecars = parent.join("CaptureOne");
        let entries = match fs::read_dir(sidecars) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error.into()),
        };
        let mut newest: Option<(SystemTime, PathBuf)> = None;
        for entry in entries {
            let entry = entry?;
            if !entry.file_name().to_string_lossy().starts_with("Settings")
                || !entry.file_type()?.is_dir()
            {
                continue;
            }
            let sidecar = entry.path().join(format!("{}.cos", name.to_string_lossy()));
            if let Ok(modified) = sidecar.metadata().and_then(|metadata| metadata.modified())
                && newest.as_ref().is_none_or(|(prior, _)| modified > *prior)
            {
                newest = Some((modified, sidecar));
            }
        }
        let Some((modified, path)) = newest else {
            return Ok(None);
        };
        let text = match fs::read(&path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error.into()),
        };
        Ok(Some(SettingsSidecar {
            modified,
            fingerprint: settings_fingerprint(&String::from_utf8_lossy(&text)),
        }))
    }

    fn inspect_document(&self) -> Result<DocumentState> {
        if self.document_id.is_empty() {
            return Err(Error::DocumentNotOpen);
        }
        let info = self
            .user
            .document_info(&self.document_id)
            .map_err(|_| Error::DocumentNotOpen)?;
        if info.kind != "session" {
            return Err(Error::DocumentNotSession);
        }
        let root = capture_root(&info.id).inspect_err(|_| {
            self.health.set(
                "capture_folder_not_set",
                format!("Capture root {}/Capture not found", info.id),
            );
        })?;
        Ok(DocumentState {
            capture_root: root.to_string_lossy().into_owned(),
            capture_folder: info.captures,
        })
    }
    fn set_capture_folder(&self, target: &str) -> Result<()> {
        let prior = self
            .user
            .document_info(&self.document_id)
            .map_err(|_| Error::CaptureFolderNotSet)?
            .captures;
        if self.apply_folder(target).is_err() {
            let _ = self.apply_folder(&prior);
            self.health.set(
                "capture_folder_not_set",
                "Capture folder could not be verified",
            );
            return Err(Error::CaptureFolderNotSet);
        }
        self.health.clear("capture_folder_not_set");
        Ok(())
    }
    fn set_current_collection(&self, folder: &str) -> Result<()> {
        self.user.set_current_collection(&self.document_id, folder)
    }
    fn sync_folder(&self, folder: &str) -> Result<()> {
        self.user.sync_folder(&self.document_id, folder)
    }
    fn apply_style(&self, raw: &str, name: &str) -> Result<StyleOutcome> {
        let variant = self.variant_background(raw)?;
        if self
            .background
            .apply_style(&self.document_id, &variant, name)?
        {
            Ok(StyleOutcome::Applied)
        } else {
            Ok(StyleOutcome::NotListed)
        }
    }
    fn styles(&self) -> Result<psm_core::domain::editing::EditorStyles> {
        let listed = self.user.list_styles()?;
        Ok(psm_core::domain::editing::EditorStyles {
            user: listed.user,
            built_in: listed.built_in,
        })
    }
    fn find_variant(&self, raw: &str) -> Result<bool> {
        Ok(self
            .background
            .find_image(&self.document_id, raw)?
            .is_some())
    }
    fn install_hooks(&self) -> Result<()> {
        let result = (|| {
            for name in ["capture_done", "processing_done", "variant_adjusted"] {
                let expected = self.paths.hooks().join(format!("{name}.scpt"));
                let path = expected.to_string_lossy();
                self.user.set_hook(name, &path)?;
                let read = self.user.get_hook(name)?;
                if !read.as_deref().is_some_and(|value| same_path(value, &path)) {
                    return Err(Error::HooksNotInstalled);
                }
            }
            Ok(())
        })();
        if result.is_err() {
            self.health
                .set("hooks_not_installed", "Hook path read-back mismatch");
        } else {
            self.health.clear("hooks_not_installed");
        }
        result
    }
    fn render(&self, raw: &str, purpose: RenderPurpose) -> Result<String> {
        self.render_started(raw, purpose)
            .map(|started| started.job_id)
    }
    #[allow(clippy::too_many_arguments)]
    fn request_render(
        &self,
        raw: &str,
        purpose: RenderPurpose,
        generation: i64,
        tracking_job_id: i64,
        now: Instant,
        now_ms: i64,
        new_arrival: bool,
    ) -> Result<()> {
        {
            let mut state = self
                .renders
                .lock()
                .map_err(|e| Error::InvalidData(e.to_string()))?;
            if purpose == RenderPurpose::Preview
                && (state.jobs.values().any(|e| {
                    e.completion.raw == raw && e.completion.purpose == RenderPurpose::Preview
                }) || state
                    .pending
                    .iter()
                    .any(|e| e.raw == raw && e.purpose == RenderPurpose::Preview))
            {
                return Ok(());
            }
            if !state.can_start(raw, purpose) {
                if purpose == RenderPurpose::Master {
                    // Refused now, as when it would have been sent at once (P56 disk guard).
                    drop(state);
                    self.check_master_space()?;
                    state = self
                        .renders
                        .lock()
                        .map_err(|e| Error::InvalidData(e.to_string()))?;
                }
                state.pending.push_back(PendingRender {
                    raw: raw.into(),
                    purpose,
                    generation,
                    tracking_job_id,
                    new_arrival,
                });
                return Ok(());
            }
            if state.jobs.is_empty() && state.pending.is_empty() && state.busy.is_empty() {
                // A new render batch: its recipe is checked again (10F §5).
                state.recipes_checked.clear();
            }
            state.claim(raw, purpose);
        }
        let result = self.render_started(raw, purpose);
        let mut state = self
            .renders
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        match result {
            Ok(started) => {
                let job_id = started.job_id;
                state.jobs.insert(
                    job_id.clone(),
                    RenderEntry {
                        completion: RenderCompletion {
                            job_id,
                            raw: raw.into(),
                            purpose,
                            generation,
                            issued_at_ms: now_ms,
                            tracking_job_id,
                            geometry: started.geometry,
                        },
                        issued_mono: now,
                        attempts: 0,
                        recipe_retried: false,
                    },
                );
                Ok(())
            }
            // P60 #22: held by the take-back (or waiting for the camera helper): queued, sent when
            // Capture One is back; not a refusal.
            Err(Error::EditorLaunchDeferred) => {
                state.release(raw, purpose);
                state.pending.push_back(PendingRender {
                    raw: raw.into(),
                    purpose,
                    generation,
                    tracking_job_id,
                    new_arrival,
                });
                Ok(())
            }
            Err(error) => {
                state.release(raw, purpose);
                Err(error)
            }
        }
    }
    fn set_render_focus(&self, raw: Option<&str>) {
        if let Ok(mut state) = self.renders.lock() {
            state.focused_raw = raw.map(str::to_owned);
        }
    }
    fn render_completion(&self, job: &str) -> Option<RenderCompletion> {
        self.renders
            .lock()
            .ok()?
            .jobs
            .get(job)
            .map(|e| e.completion.clone())
    }
    fn resolve_render_output(&self, output: &Path, issued_at_ms: i64) -> Option<PathBuf> {
        resolve_output(
            output,
            UNIX_EPOCH + Duration::from_millis(issued_at_ms.max(0) as u64),
        )
    }
    fn finish_render(&self, job: &str, now: Instant, now_ms: i64) -> Result<()> {
        {
            let mut state = self
                .renders
                .lock()
                .map_err(|e| Error::InvalidData(e.to_string()))?;
            let Some(entry) = state.jobs.remove(job) else {
                return Ok(());
            };
            state.release(&entry.completion.raw, entry.completion.purpose);
        }
        self.pump(now, now_ms)
    }
    fn recipe_fault(&self, job: &str, now: Instant, now_ms: i64) -> Result<bool> {
        let entry = {
            let mut state = self
                .renders
                .lock()
                .map_err(|e| Error::InvalidData(e.to_string()))?;
            match state.jobs.get(job) {
                None => return Ok(false),
                Some(entry) if entry.recipe_retried => return Ok(false),
                Some(_) => {}
            }
            let entry = state.jobs.remove(job).expect("render entry");
            state.recipes_checked.remove(&entry.completion.purpose);
            entry
        };
        tracing::warn!(
            job,
            raw = entry.completion.raw,
            "render came out in the wrong format; repairing its recipe and rendering once more"
        );
        // The render keeps its lane; `render_started` repairs the recipe first (not checked now).
        let result = self.render_started(&entry.completion.raw, entry.completion.purpose);
        let mut state = self
            .renders
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        match result {
            Ok(started) => {
                let mut entry = entry;
                entry.completion.job_id = started.job_id.clone();
                entry.completion.geometry = started.geometry;
                entry.completion.issued_at_ms = now_ms;
                entry.issued_mono = now;
                entry.recipe_retried = true;
                state.jobs.insert(started.job_id, entry);
                Ok(true)
            }
            Err(error) => {
                state.release(&entry.completion.raw, entry.completion.purpose);
                drop(state);
                self.pump(now, now_ms)?;
                Err(error)
            }
        }
    }
    fn expire_renders(&self, now: Instant, now_ms: i64) -> Result<Vec<(i64, i64)>> {
        let expired: Vec<String> = {
            let state = self
                .renders
                .lock()
                .map_err(|e| Error::InvalidData(e.to_string()))?;
            state
                .jobs
                .iter()
                .filter(|(_, entry)| {
                    now.duration_since(entry.issued_mono).as_secs()
                        >= match entry.completion.purpose {
                            RenderPurpose::Preview => 60,
                            RenderPurpose::Master => 120,
                        }
                })
                .map(|(id, _)| id.clone())
                .collect()
        };
        let mut failed = Vec::new();
        for old_id in expired {
            let entry = {
                let mut state = self
                    .renders
                    .lock()
                    .map_err(|e| Error::InvalidData(e.to_string()))?;
                let Some(entry) = state.jobs.remove(&old_id) else {
                    continue;
                };
                if entry.attempts > 0 {
                    state.release(&entry.completion.raw, entry.completion.purpose);
                }
                entry
            };
            if entry.attempts == 0 {
                // MC-16: re-issued once; it keeps the render lane.
                let result = self.render_started(&entry.completion.raw, entry.completion.purpose);
                let mut state = self
                    .renders
                    .lock()
                    .map_err(|e| Error::InvalidData(e.to_string()))?;
                match result {
                    Ok(started) => {
                        let new_id = started.job_id;
                        let mut entry = entry;
                        entry.completion.job_id = new_id.clone();
                        entry.completion.geometry = started.geometry;
                        entry.completion.issued_at_ms = now_ms;
                        entry.issued_mono = now;
                        entry.attempts = 1;
                        state.jobs.insert(new_id, entry);
                    }
                    Err(error) => {
                        state.jobs.insert(old_id, entry);
                        return Err(error);
                    }
                }
            } else {
                failed.push((
                    entry.completion.tracking_job_id,
                    entry.completion.generation,
                ));
            }
        }
        // A RAW or the preview lane may also be free after a refused request: nothing waits for a
        // new request to be sent.
        self.pump(now, now_ms)?;
        Ok(failed)
    }
    fn refused_renders(&self) -> Vec<(i64, String)> {
        self.renders
            .lock()
            .map(|mut state| std::mem::take(&mut state.refused))
            .unwrap_or_default()
    }
    fn render_in_flight(&self, raw: &str) -> bool {
        self.renders.lock().is_ok_and(|s| {
            s.jobs.values().any(|e| e.completion.raw == raw)
                || s.pending.iter().any(|e| e.raw == raw)
        })
    }
    fn renders_in_flight(&self) -> usize {
        self.discard_missing();
        self.renders
            .lock()
            // Pending work has not been handed to Capture One yet. It survives a quit and is
            // retried when the editor is available again; only issued jobs can be lost.
            .map(|s| s.out())
            .unwrap_or(0)
    }
    fn mirror_tag(&self, raw: &str, classification: Classification, now_ms: i64) -> Result<()> {
        let variant = self.variant_background(raw)?;
        let desired = tag_for(classification);
        if self.background.get_color_tag(&self.document_id, &variant)? == desired {
            return Ok(());
        }
        self.store.mark_own_write(raw, now_ms)?;
        let write = self
            .background
            .set_color_tag(&self.document_id, &variant, desired);
        if matches!(write, Err(Error::Timeout)) {
            return Err(Error::Timeout);
        }
        write?;
        if self.background.get_color_tag(&self.document_id, &variant)? != desired {
            return Err(Error::TagVerification);
        }
        Ok(())
    }
    fn capabilities(&self) -> EditingCapabilities {
        EditingCapabilities {
            capture_folder: true,
            hooks: true,
            render: true,
            mirror_tag: true,
            import: true,
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mc2_capture_root_is_document_capture_directory() {
        let dir = tempfile::tempdir().unwrap();
        let document = dir.path().join("Capture One Session");
        let root = document.join("Capture");
        fs::create_dir_all(&root).unwrap();
        assert_eq!(
            capture_root(&document.to_string_lossy()).unwrap(),
            root.canonicalize().unwrap()
        );
        fs::remove_dir(&root).unwrap();
        assert!(matches!(
            capture_root(&document.to_string_lossy()),
            Err(Error::CaptureFolderNotSet)
        ));
    }
    #[test]
    fn mc19_tag_mapping() {
        assert_eq!(
            [
                Classification::Unreviewed,
                Classification::Deleted,
                Classification::Main,
                Classification::Condition
            ]
            .map(tag_for),
            [0, 1, 4, 5]
        );
    }
}
