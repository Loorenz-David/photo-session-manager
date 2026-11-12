//! Local export (ch07 §Completion without an integration, 05A MC-8, 10A MC-16, 10D MC-28/MC-29):
//! every output of every main and condition photo — one per slot, in photo order — is the app's
//! own render of the composition from the RAW's master, cached under its key
//! `(master adjusted_gen, composition revision, output_format)`. A run copies cached outputs into
//! a staging folder, queues `render_output` jobs where the master exists, and asks Capture One for
//! the master (once per RAW) where it does not; `manifest.json` and `item.json` are written beside
//! the files and the staging folder is swapped in place of `<export folder>/<key>/`. The same path
//! is step one of every publish (the swap deferred).

use crate::{
    Error, Result,
    domain::{
        composition::{CompositionState, Fingerprint, FrameSource, OutputSlot},
        editing::RenderPurpose,
        item::Item,
        output::{OutputFormat, slot_name},
        photo::{Classification, Integrity},
    },
    events::{AppEvent, ExportState},
    mechanisms::{
        backoff::{
            EDITOR_WAIT_EXPIRED, EDITOR_WAIT_MAX_MS, EditorWait, editor_wait,
            editor_wait_backoff_ms, editor_wait_expired,
        },
        naming::{
            ExportGroup, export_file_name, folder_key, item_json, output_names, review_label,
        },
    },
    ports::{
        clock::Clock,
        editing::{EditingProvider, RenderCompletion},
        events::EventSink,
        images::{Imaging, MasterStore, OutputStore, uid12},
        store::Store,
    },
    services::{integrity::IntegrityCheck, previews::ExportStaging},
};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    path::{Path, PathBuf},
    sync::{Arc, Mutex, MutexGuard},
};

/// File operations of an export: a staging folder beside the export folder, then a swap by renames.
pub trait ExportFiles: Send + Sync {
    /// Creates `<export folder>/.<key>.partial-<run id>/` (empty) and returns it.
    fn create_staging(&self, key: &str, run_id: i64) -> Result<PathBuf>;
    /// Copies `source` into the staging folder as `name` (temp file + rename).
    fn copy_in(&self, staging: &Path, name: &str, source: &Path) -> Result<()>;
    fn write_in(&self, staging: &Path, name: &str, bytes: &[u8]) -> Result<()>;
    fn rename_in(&self, staging: &Path, from: &str, to: &str) -> Result<()>;
    fn remove_in(&self, staging: &Path, name: &str) -> Result<()>;
    /// Renames the current `<key>/` to `.<key>.old-<run id>`, the staging folder to `<key>/`, then
    /// deletes the old folder. If the second rename fails the old folder is put back.
    fn swap(&self, staging: &Path, key: &str, run_id: i64) -> Result<()>;
    /// Deletes a staging folder and everything in it.
    fn discard(&self, staging: &Path) -> Result<()>;
    /// `<export folder>/<key>/`.
    fn folder(&self, key: &str) -> PathBuf;
}

/// The render side of an export (10D MC-28): the renderer, the master and output caches and the
/// output setting in effect.
pub struct ExportRender {
    pub imaging: Arc<dyn Imaging>,
    pub masters: Arc<dyn MasterStore>,
    pub outputs: Arc<dyn OutputStore>,
    pub format: OutputFormat,
}

type Snapshot = (
    Item,
    Vec<i64>,
    Vec<i64>,
    BTreeMap<String, serde_json::Value>,
);

/// The id of one export run (the `render_export` job row that records its photo-list snapshot).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExportRunId(pub i64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Progress {
    /// The RAW has no master for the target's generation: Capture One renders it.
    WaitingMaster,
    /// A `render_output` job is queued for the target.
    Rendering,
    /// The output is in the staging folder.
    Copied,
}

struct Target {
    photo_id: i64,
    raw: String,
    uid12: String,
    group: ExportGroup,
    slot: OutputSlot,
    name: String,
    adjusted_gen: i64,
    revision: i64,
    progress: Progress,
}

struct Run {
    id: i64,
    format: OutputFormat,
    session_id: i64,
    key: String,
    staging: PathBuf,
    item: Item,
    /// Every valid value of the item at the start of the run, keyed by attribute key (MC-8).
    attributes: BTreeMap<String, serde_json::Value>,
    targets: Vec<Target>,
    defer_swap: bool,
    /// Changelog 45 (P65): last reported "waiting for Capture One" state of the run.
    waiting: bool,
    /// P69 (changelog 47): when the run started, when its last master arrived, and how many of
    /// its photos needed no master render (ready in advance) out of all its photos.
    started_ms: i64,
    masters_done_ms: Option<i64>,
    masters_ready: usize,
    masters_total: usize,
}

impl Run {
    fn done(&self) -> usize {
        self.targets
            .iter()
            .filter(|t| t.progress == Progress::Copied)
            .count()
    }
    /// P69: the run's master and output times so far (`now_ms` ends the running part).
    fn timing(&self, now_ms: i64) -> ExportTiming {
        let masters_end = self.masters_done_ms.unwrap_or(now_ms);
        ExportTiming {
            masters_ms: (masters_end - self.started_ms).max(0),
            outputs_ms: self
                .masters_done_ms
                .map_or(0, |done| (now_ms - done).max(0)),
            masters_ready: self.masters_ready,
            masters_total: self.masters_total,
        }
    }
    /// Notes the moment no target waits for a master any more.
    fn note_masters_done(&mut self, now_ms: i64) {
        if self.masters_done_ms.is_none()
            && !self
                .targets
                .iter()
                .any(|t| t.progress == Progress::WaitingMaster)
        {
            self.masters_done_ms = Some(now_ms);
        }
    }
}

/// P69 (changelog 47 §per-step times): what an export run spent waiting for Capture One's
/// masters and then rendering and copying its outputs, and how many of its photos' masters (or
/// outputs) were ready when it started.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ExportTiming {
    pub masters_ms: i64,
    pub outputs_ms: i64,
    pub masters_ready: usize,
    pub masters_total: usize,
}

/// Changelog 45 (P65): a master render the editor could not take yet (it has not loaded the photo,
/// is busy, or is closed). Retried with back-off; the runs needing it fail only after
/// `EDITOR_WAIT_MAX_MS`.
#[derive(Debug, Clone)]
struct Wait {
    since_ms: i64,
    next_ms: i64,
    attempts: u32,
    reason: EditorWait,
    run_id: i64,
    photo_id: i64,
    generation: i64,
}

#[derive(Default)]
struct State {
    runs: HashMap<i64, Run>,
    /// RAW → tracking job id of the master render in flight (one `process` per RAW; every run
    /// that needs it waits for it).
    masters: HashMap<String, i64>,
    /// RAW → its master render waiting for the editor (changelog 45).
    waits: HashMap<String, Wait>,
    /// RAW → when its master first had to wait; kept through retries until the master arrives,
    /// so a render that keeps timing out still ends after `EDITOR_WAIT_MAX_MS`.
    waited_since: HashMap<String, i64>,
    /// P69 (changelog 47 (2)): master renders asked for ahead of a publish, by tracking job id
    /// (RAW, adjusted_gen). Also held in `masters`, so a run that starts meanwhile waits for them.
    prepared: HashMap<i64, (String, i64)>,
    /// RAW → the generation whose prepared master could not be made (not asked for again).
    prepare_failed: HashMap<String, i64>,
}

pub struct ExportService {
    store: Arc<dyn Store>,
    editing: Arc<dyn EditingProvider>,
    events: Arc<dyn EventSink>,
    clock: Arc<dyn Clock>,
    files: Arc<dyn ExportFiles>,
    render: ExportRender,
    format_override: std::sync::RwLock<Option<OutputFormat>>,
    state: Mutex<State>,
    /// P69: the timing of runs that completed in this process, by run id.
    timings: Mutex<HashMap<i64, ExportTiming>>,
}

impl ExportService {
    pub fn new(
        store: Arc<dyn Store>,
        editing: Arc<dyn EditingProvider>,
        events: Arc<dyn EventSink>,
        clock: Arc<dyn Clock>,
        files: Arc<dyn ExportFiles>,
        render: ExportRender,
    ) -> Self {
        Self {
            store,
            editing,
            events,
            clock,
            files,
            render,
            format_override: std::sync::RwLock::new(None),
            state: Mutex::new(State::default()),
            timings: Mutex::new(HashMap::new()),
        }
    }

    fn state(&self) -> Result<MutexGuard<'_, State>> {
        self.state
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))
    }

    /// The output setting this service renders with.
    pub fn format(&self) -> OutputFormat {
        self.format_override
            .read()
            .unwrap()
            .unwrap_or(self.render.format)
    }
    pub fn set_format(&self, format: OutputFormat) {
        *self.format_override.write().unwrap() = Some(format);
    }

    /// Whether the run is in flight in this process (a `render_output` row of an earlier process
    /// finds none and closes).
    pub fn has_run(&self, run_id: i64) -> bool {
        self.state()
            .is_ok_and(|state| state.runs.contains_key(&run_id))
    }

    /// `<export folder>/<folder_key(current SKU)>/` of the session's item.
    pub fn folder(&self, session_id: i64) -> Result<PathBuf> {
        let (item, _, _, _) = self.snapshot(session_id)?;
        Ok(self.files.folder(&folder_key(&item.sku)?))
    }

    /// Starts an export of the session. Fails before any render when a photo's RAW changed or went
    /// missing (MC-3 check a) or a main or condition photo needs re-framing (MC-27 §Revisions); a
    /// second call while a run of the session is in flight returns that run.
    pub fn run(&self, session_id: i64) -> Result<ExportRunId> {
        // 10F Q3: a cleared item's photos have no local files to export.
        crate::services::retention::ensure_session_not_cleared(self.store.as_ref(), session_id)?;
        let mut state = self.state()?;
        if let Some(run) = state.runs.values().find(|r| r.session_id == session_id) {
            return Ok(ExportRunId(run.id));
        }
        match self.start(&mut state, session_id, None) {
            Ok(run) => {
                let id = run.id;
                let total = run.targets.len();
                self.emit(session_id, run.done(), total, ExportState::Running);
                if run.done() == total {
                    self.finalize(&state, run);
                } else {
                    state.runs.insert(id, run);
                    self.refresh_waiting(&mut state);
                }
                Ok(ExportRunId(id))
            }
            Err(error) => {
                self.emit(session_id, 0, 0, ExportState::Failed);
                Err(error)
            }
        }
    }

    /// Render a confirmed publish snapshot. The staging folder is kept until the publish reaches
    /// done; repeat calls during one process return the same export.
    pub fn run_staged(
        &self,
        session_id: i64,
        main: &[i64],
        condition: &[i64],
        attributes: &BTreeMap<String, serde_json::Value>,
    ) -> Result<ExportRunId> {
        let mut state = self.state()?;
        if let Some(run) = state
            .runs
            .values()
            .find(|r| r.session_id == session_id && r.defer_swap)
        {
            return Ok(ExportRunId(run.id));
        }
        let item = self.store.item(
            self.store
                .session(session_id)?
                .session
                .item_id
                .ok_or_else(|| Error::InvalidValue("unassigned".into()))?,
        )?;
        let run = self.start(
            &mut state,
            session_id,
            Some((item, main.to_vec(), condition.to_vec(), attributes.clone())),
        )?;
        let id = run.id;
        if run.done() == run.targets.len() {
            self.finalize(&state, run);
        } else {
            state.runs.insert(id, run);
            self.refresh_waiting(&mut state);
        }
        Ok(ExportRunId(id))
    }

    /// Changelog 45 (P65): whether the run waits for Capture One (a master render it needs could
    /// not be taken yet and is retried).
    pub fn waiting(&self, run_id: i64) -> bool {
        self.state()
            .is_ok_and(|state| state.runs.get(&run_id).is_some_and(|run| run.waiting))
    }

    /// Ends a run in flight without a result (a cancelled publish, changelog 44/45): its staging
    /// folder is deleted and the run recorded as `cancelled`; nothing else changes.
    pub fn cancel_run(&self, run_id: i64) -> Result<()> {
        let mut state = self.state()?;
        if let Some(run) = state.runs.remove(&run_id) {
            self.abandon(&run, "cancelled");
        }
        Ok(())
    }

    /// Reports each run whose waiting state changed: the run row's mark (the session row reads
    /// it) and an `export_progress` event.
    fn refresh_waiting(&self, state: &mut State) {
        let waits: HashSet<String> = state.waits.keys().cloned().collect();
        for run in state.runs.values_mut() {
            let waiting = run
                .targets
                .iter()
                .any(|t| t.progress == Progress::WaitingMaster && waits.contains(&t.raw));
            if waiting == run.waiting {
                continue;
            }
            run.waiting = waiting;
            if let Err(error) = self.store.set_export_waiting(run.id, waiting) {
                tracing::warn!(%error, run = run.id, "export waiting state not recorded");
            }
            let done = run.done();
            let total = run.targets.len();
            self.events.emit(AppEvent::ExportProgress {
                session_id: run.session_id,
                done,
                total,
                state: if waiting {
                    ExportState::Waiting
                } else {
                    ExportState::Running
                },
            });
        }
    }

    /// Records that the master render of `raw` waits for the editor (`reason`), keeping the
    /// start of a wait already under way.
    fn begin_wait(&self, state: &mut State, raw: &str, wait: Wait) {
        tracing::info!(
            raw,
            run = wait.run_id,
            reason = ?wait.reason,
            "export waits for Capture One"
        );
        let since = *state
            .waited_since
            .entry(raw.to_string())
            .or_insert(wait.since_ms);
        match state.waits.get_mut(raw) {
            Some(existing) => {
                existing.reason = wait.reason;
                existing.next_ms = existing.next_ms.min(wait.next_ms);
            }
            None => {
                state.waits.insert(
                    raw.to_string(),
                    Wait {
                        since_ms: since,
                        ..wait
                    },
                );
            }
        }
    }

    /// Changelog 45 (P65): retries the master renders that wait for the editor when due (1, 2,
    /// 4, 8, then every 15 s) and fails the runs that need one after 10 minutes of waiting.
    pub fn tick(&self) -> Result<()> {
        let mut state = self.state()?;
        let now = self.clock.now_ms();
        let raws: Vec<String> = state.waits.keys().cloned().collect();
        for raw in raws {
            let Some(wait) = state.waits.get(&raw).cloned() else {
                continue;
            };
            let needed = |t: &Target| t.raw == raw && t.progress == Progress::WaitingMaster;
            if !state
                .runs
                .values()
                .any(|run| run.targets.iter().any(needed))
            {
                state.waits.remove(&raw);
                state.waited_since.remove(&raw);
                continue;
            }
            if now - wait.since_ms >= EDITOR_WAIT_MAX_MS {
                state.waits.remove(&raw);
                state.waited_since.remove(&raw);
                let name = Path::new(&raw)
                    .file_name()
                    .map_or_else(|| raw.clone(), |n| n.to_string_lossy().into_owned());
                let sentence = editor_wait_expired(wait.reason, &name);
                tracing::warn!(raw, reason = ?wait.reason, "export stopped waiting for Capture One");
                self.fail_runs(
                    &mut state,
                    needed,
                    &format!("{EDITOR_WAIT_EXPIRED}{sentence}"),
                );
                continue;
            }
            if now < wait.next_ms {
                continue;
            }
            let tracking = self.store.new_master_render_job(
                &raw,
                wait.run_id,
                wait.photo_id,
                wait.generation,
                now,
            )?;
            match self.editing.request_render(
                &raw,
                RenderPurpose::Master,
                wait.generation,
                tracking,
                self.clock.mono(),
                now,
                false,
            ) {
                Ok(()) => {
                    tracing::info!(
                        raw,
                        waited_ms = now - wait.since_ms,
                        "export render no longer waits for Capture One"
                    );
                    state.waits.remove(&raw);
                    state.masters.insert(raw.clone(), tracking);
                }
                Err(error) => {
                    let text = error.to_string();
                    self.store.finish_render_job(tracking, Some(&text), now)?;
                    match editor_wait(&text) {
                        Some(reason) => {
                            if let Some(wait) = state.waits.get_mut(&raw) {
                                wait.attempts += 1;
                                wait.reason = reason;
                                wait.next_ms = now + editor_wait_backoff_ms(wait.attempts);
                            }
                        }
                        None => {
                            state.waits.remove(&raw);
                            state.waited_since.remove(&raw);
                            self.fail_runs(&mut state, needed, &text);
                        }
                    }
                }
            }
        }
        self.refresh_waiting(&mut state);
        Ok(())
    }

    pub fn staging_folder(&self, key: &str, run_id: i64) -> PathBuf {
        self.files
            .folder(key)
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(format!(".{key}.partial-{run_id}"))
    }
    pub fn publish_swap(&self, staging: &Path, key: &str, run_id: i64) -> Result<()> {
        self.files.swap(staging, key, run_id)
    }
    pub fn discard_staging(&self, staging: &Path) -> Result<()> {
        self.files.discard(staging)
    }

    /// Main and condition photos in position order (unreviewed and deleted excluded, N2), and the
    /// item's valid attribute values (cleared and invalid ones excluded, MC-8).
    fn snapshot(&self, session_id: i64) -> Result<Snapshot> {
        let mut snapshot = None;
        self.store.transaction(&mut |tx| {
            let session = tx.session(session_id)?;
            let item_id = session.item_id.ok_or_else(|| {
                Error::InvalidValue("Unassigned photos have no item to export".into())
            })?;
            let item = tx.item(item_id)?;
            let mut main = Vec::new();
            let mut condition = Vec::new();
            for photo in tx.photos(session_id)? {
                match photo.classification {
                    Classification::Main => main.push((photo.position, photo.id)),
                    Classification::Condition => condition.push((photo.position, photo.id)),
                    Classification::Unreviewed | Classification::Deleted => {}
                }
            }
            main.sort();
            condition.sort();
            let attributes = tx
                .attribute_values(item_id)?
                .into_iter()
                .filter(|v| v.valid && v.is_live())
                .filter_map(|v| v.json().map(|json| (v.key, json)))
                .collect();
            snapshot = Some((
                item,
                main.into_iter().map(|(_, id)| id).collect(),
                condition.into_iter().map(|(_, id)| id).collect(),
                attributes,
            ));
            Ok(())
        })?;
        snapshot.ok_or_else(|| Error::InvalidData("export snapshot".into()))
    }

    fn start(&self, state: &mut State, session_id: i64, frozen: Option<Snapshot>) -> Result<Run> {
        let defer_swap = frozen.is_some();
        let (item, main, condition, attributes) = match frozen {
            Some(snapshot) => snapshot,
            None => self.snapshot(session_id)?,
        };
        let key = folder_key(&item.sku)?;
        let check = IntegrityCheck::new(self.store.as_ref());
        for &photo_id in main.iter().chain(&condition) {
            match check.verify(photo_id)? {
                Integrity::Ok => {}
                Integrity::RawChanged => return Err(Error::RawChanged { photo_id }),
                Integrity::RawMissing => return Err(Error::RawMissing { photo_id }),
            }
        }
        let groups = [
            (ExportGroup::Main, &main),
            (ExportGroup::Condition, &condition),
        ];
        // M33: never rendered for export while a main or condition photo needs re-framing.
        let mut flagged = Vec::new();
        let mut compositions = HashMap::new();
        for (group, ids) in groups {
            for (index, &photo_id) in ids.iter().enumerate() {
                let composition = self.store.composition(photo_id)?;
                if composition.state == CompositionState::NeedsReview {
                    flagged.push(review_label(group, index + 1, ids.len()));
                }
                compositions.insert(photo_id, composition);
            }
        }
        if !flagged.is_empty() {
            return Err(Error::NeedsReview { photos: flagged });
        }
        let mut photos = HashMap::new();
        let mut retouch_gens = BTreeMap::new();
        for &photo_id in main.iter().chain(&condition) {
            let photo = self.store.photo(photo_id)?;
            let retouch = self
                .store
                .preview_state(&photo.raw_path)?
                .map_or(0, |state| state.retouch_gen);
            retouch_gens.insert(photo_id.to_string(), retouch);
            let generation = self.store.adjusted_gen(&photo.raw_path)?;
            photos.insert(photo_id, (photo, generation));
        }
        let now_ms = self.clock.now_ms();
        let payload = serde_json::json!({
            "session_id": session_id,
            "key": key,
            "main": main,
            "condition": condition,
            "retouch_gens": retouch_gens,
        })
        .to_string();
        let run_id = self.store.new_export_run(session_id, &payload, now_ms)?;
        let staging = match self.files.create_staging(&key, run_id) {
            Ok(staging) => staging,
            Err(error) => {
                self.store
                    .finish_render_job(run_id, Some("staging"), now_ms)?;
                return Err(error);
            }
        };
        let mut run = Run {
            id: run_id,
            format: self.format(),
            session_id,
            key,
            staging,
            item,
            attributes,
            targets: Vec::new(),
            defer_swap,
            waiting: false,
            started_ms: now_ms,
            masters_done_ms: None,
            masters_ready: 0,
            masters_total: 0,
        };
        for (group, ids) in groups {
            let slots: Vec<(i64, &[OutputSlot])> = ids
                .iter()
                .map(|id| (*id, compositions[id].kind.slots()))
                .collect();
            for (photo_id, slot, name) in output_names(&run.key, group, &slots) {
                let (photo, generation) = &photos[&photo_id];
                run.targets.push(Target {
                    photo_id,
                    raw: photo.raw_path.clone(),
                    uid12: uid12(&photo.uid),
                    group,
                    slot,
                    name,
                    adjusted_gen: *generation,
                    revision: compositions[&photo_id].revision,
                    progress: Progress::WaitingMaster,
                });
            }
        }
        for index in 0..run.targets.len() {
            if let Err(error) = self.begin_target(state, &mut run, index) {
                self.abandon(&run, "provider");
                return Err(error);
            }
        }
        // P69: a photo is ready in advance when none of its outputs waits for a master.
        let photos: BTreeSet<i64> = run.targets.iter().map(|t| t.photo_id).collect();
        let waiting: BTreeSet<i64> = run
            .targets
            .iter()
            .filter(|t| t.progress == Progress::WaitingMaster)
            .map(|t| t.photo_id)
            .collect();
        run.masters_total = photos.len();
        run.masters_ready = photos.len() - waiting.len();
        run.note_masters_done(now_ms);
        tracing::info!(
            run = run.id,
            masters_ready = run.masters_ready,
            masters_total = run.masters_total,
            "export started"
        );
        Ok(run)
    }

    /// A cached output is copied; else a master on disk gets a `render_output` job; else the
    /// master is requested from Capture One (once per RAW).
    fn begin_target(&self, state: &mut State, run: &mut Run, index: usize) -> Result<()> {
        let format_key = run.format.format_key();
        let target = &run.targets[index];
        let dest = self.render.outputs.path(
            &target.raw,
            &target.uid12,
            target.slot,
            target.adjusted_gen,
            target.revision,
            &format_key,
        );
        if dest.is_file() {
            self.files.copy_in(&run.staging, &target.name, &dest)?;
            run.targets[index].progress = Progress::Copied;
            return Ok(());
        }
        if self
            .render
            .masters
            .path(&target.raw, target.adjusted_gen)
            .is_file()
        {
            self.enqueue_output(run, target)?;
            run.targets[index].progress = Progress::Rendering;
            return Ok(());
        }
        if !state.masters.contains_key(&target.raw) && !state.waits.contains_key(&target.raw) {
            let now_ms = self.clock.now_ms();
            let tracking = self.store.new_master_render_job(
                &target.raw,
                run.id,
                target.photo_id,
                target.adjusted_gen,
                now_ms,
            )?;
            if let Err(error) = self.editing.request_render(
                &target.raw,
                RenderPurpose::Master,
                target.adjusted_gen,
                tracking,
                self.clock.mono(),
                now_ms,
                false,
            ) {
                self.store
                    .finish_render_job(tracking, Some("provider"), now_ms)?;
                // Changelog 45 (P65): Capture One not ready is not a failure: the run waits.
                let Some(reason) = editor_wait(&error.to_string()) else {
                    return Err(error);
                };
                let raw = target.raw.clone();
                let wait = Wait {
                    since_ms: now_ms,
                    next_ms: now_ms + editor_wait_backoff_ms(1),
                    attempts: 1,
                    reason,
                    run_id: run.id,
                    photo_id: target.photo_id,
                    generation: target.adjusted_gen,
                };
                self.begin_wait(state, &raw, wait);
                return Ok(());
            }
            state.masters.insert(target.raw.clone(), tracking);
        }
        Ok(())
    }

    fn enqueue_output(&self, run: &Run, target: &Target) -> Result<()> {
        let payload = serde_json::json!({
            "run_id": run.id,
            "photo_id": target.photo_id,
            "slot": slot_name(target.slot),
            "adjusted_gen": target.adjusted_gen,
            "revision": target.revision,
            "format_key": run.format.format_key(),
        })
        .to_string();
        self.store
            .enqueue_render_output(&target.raw, &payload, self.clock.now_ms())?;
        Ok(())
    }

    fn emit(&self, session_id: i64, done: usize, total: usize, state: ExportState) {
        self.events.emit(AppEvent::ExportProgress {
            session_id,
            done,
            total,
            state,
        });
    }

    /// Deletes the staging folder (the export folder is left as it was) and reports `failed`.
    fn fail(&self, run: Run, reason: &str) {
        self.abandon(&run, reason);
        self.emit(
            run.session_id,
            run.done(),
            run.targets.len(),
            ExportState::Failed,
        );
    }

    fn abandon(&self, run: &Run, reason: &str) {
        tracing::warn!(run = run.id, reason, "export failed");
        if let Err(error) = self.files.discard(&run.staging) {
            tracing::warn!(%error, "export staging not removed");
        }
        if let Err(error) = self
            .store
            .finish_render_job(run.id, Some(reason), self.clock.now_ms())
        {
            tracing::warn!(%error, "export run not recorded as failed");
        }
    }

    /// Fails every run holding a target that matches `holds`.
    fn fail_runs(&self, state: &mut State, holds: impl Fn(&Target) -> bool, reason: &str) {
        let ids: Vec<i64> = state
            .runs
            .values()
            .filter(|run| run.targets.iter().any(&holds))
            .map(|run| run.id)
            .collect();
        for id in ids {
            if let Some(run) = state.runs.remove(&id) {
                self.fail(run, reason);
            }
        }
    }

    /// All outputs are in: drop photos deleted or moved out of the session meanwhile (07A R-I6
    /// analogue), name the rest by their output order, write `manifest.json` and `item.json`,
    /// swap; after a local export of a session that is not dirty the run's masters are released.
    fn finalize(&self, state: &State, run: Run) {
        match self.write_and_swap(&run) {
            Ok(count) => {
                if let Ok(mut timings) = self.timings.lock() {
                    timings.insert(run.id, run.timing(self.clock.now_ms()));
                }
                if let Err(error) = self
                    .store
                    .finish_render_job(run.id, None, self.clock.now_ms())
                {
                    tracing::warn!(%error, "export run not recorded as done");
                }
                self.emit(run.session_id, count, count, ExportState::Done);
                if !run.defer_swap {
                    self.release_run_masters(state, &run);
                }
            }
            Err(error) => {
                tracing::warn!(%error, "export could not be completed");
                self.fail(run, "files");
            }
        }
    }

    /// MC-28 §Master after a publish (P30): the masters of the session's non-deleted photos are
    /// deleted (the caller checked the session is not dirty). A RAW an export in flight still
    /// needs keeps its master.
    pub fn release_masters(&self, session_id: i64) -> Result<()> {
        let state = self.state()?;
        let in_use = Self::in_use(&state);
        let raws: BTreeSet<String> = self
            .store
            .photos_in_session(session_id)?
            .into_iter()
            .filter(|p| p.classification != Classification::Deleted)
            .map(|p| p.raw_path)
            .collect();
        for raw in raws {
            if in_use.contains(raw.as_str()) {
                continue;
            }
            self.render.masters.remove_all(&raw)?;
        }
        Ok(())
    }

    /// The RAWs whose master a run in flight still needs (a target not yet copied, or a master
    /// render in flight); 10F §4 level 3 never clears those.
    pub fn masters_in_use(&self) -> Result<HashSet<String>> {
        let state = self.state()?;
        Ok(Self::in_use(&state)
            .into_iter()
            .map(str::to_owned)
            .collect())
    }

    /// Whether any export run (a local export or a publish's staging) is in flight.
    pub fn running(&self) -> bool {
        self.state().is_ok_and(|state| !state.runs.is_empty())
    }

    fn in_use(state: &State) -> HashSet<&str> {
        state
            .runs
            .values()
            .flat_map(|run| run.targets.iter())
            .filter(|t| t.progress != Progress::Copied)
            .map(|t| t.raw.as_str())
            .chain(state.masters.keys().map(String::as_str))
            .chain(state.waits.keys().map(String::as_str))
            .collect()
    }

    /// MC-28 §Master: masters are a cache, deleted once the export completes and the session is
    /// not dirty. A RAW another run still needs keeps its master.
    fn release_run_masters(&self, state: &State, run: &Run) {
        match self.store.session(run.session_id) {
            Ok(session) if !session.dirty => {}
            Ok(_) => return,
            Err(error) => {
                tracing::warn!(%error, "masters kept: session not read");
                return;
            }
        }
        let in_use: HashSet<&str> = state
            .runs
            .values()
            .filter(|other| other.id != run.id)
            .flat_map(|other| other.targets.iter())
            .filter(|t| t.progress != Progress::Copied)
            .map(|t| t.raw.as_str())
            .collect();
        let raws: BTreeSet<&str> = run.targets.iter().map(|t| t.raw.as_str()).collect();
        for raw in raws {
            if in_use.contains(raw) || state.masters.contains_key(raw) {
                continue;
            }
            if let Err(error) = self.render.masters.remove_all(raw) {
                tracing::warn!(%error, raw, "master not released");
            }
        }
    }

    fn write_and_swap(&self, run: &Run) -> Result<usize> {
        let mut kept: [Vec<&Target>; 2] = [Vec::new(), Vec::new()];
        for target in &run.targets {
            let photo = self.store.photo(target.photo_id)?;
            if photo.classification == Classification::Deleted || photo.session_id != run.session_id
            {
                self.files.remove_in(&run.staging, &target.name)?;
                continue;
            }
            let slot = match target.group {
                ExportGroup::Main => 0,
                ExportGroup::Condition => 1,
            };
            kept[slot].push(target);
        }
        let mut lists: [Vec<String>; 2] = [Vec::new(), Vec::new()];
        let mut manifest = serde_json::Map::new();
        for (slot, targets) in kept.iter().enumerate() {
            for (index, target) in targets.iter().enumerate() {
                let name = export_file_name(&run.key, index + 1, targets.len(), target.group);
                if name != target.name {
                    self.files.rename_in(&run.staging, &target.name, &name)?;
                }
                manifest.insert(
                    name.clone(),
                    serde_json::json!({
                        "photo_id": target.photo_id,
                        "slot": slot_name(target.slot),
                        "adjusted_gen": target.adjusted_gen,
                        "composition_rev": target.revision,
                    }),
                );
                lists[slot].push(name);
            }
        }
        let manifest = serde_json::json!({
            "format_key": run.format.format_key(),
            "outputs": manifest,
        });
        let mut text = serde_json::to_string_pretty(&manifest)
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        text.push('\n');
        self.files
            .write_in(&run.staging, "manifest.json", text.as_bytes())?;
        let json = item_json(&run.item, &run.attributes, &lists[0], &lists[1]);
        self.files
            .write_in(&run.staging, "item.json", json.as_bytes())?;
        if !run.defer_swap {
            self.files.swap(&run.staging, &run.key, run.id)?;
        }
        Ok(lists[0].len() + lists[1].len())
    }

    /// A `render_output` job wrote the output of `(photo, slot)` at `path` (rendered from
    /// composition revision `revision`): copied into the run's staging folder; the last one
    /// finalizes the run.
    pub fn output_ready(
        &self,
        run_id: i64,
        photo_id: i64,
        slot: OutputSlot,
        path: &Path,
        revision: i64,
    ) -> Result<()> {
        let mut state = self.state()?;
        let Some(mut run) = state.runs.remove(&run_id) else {
            return Ok(());
        };
        let Some(index) = run.targets.iter().position(|t| {
            t.photo_id == photo_id && t.slot == slot && t.progress == Progress::Rendering
        }) else {
            state.runs.insert(run_id, run);
            return Ok(());
        };
        if let Err(error) = self
            .files
            .copy_in(&run.staging, &run.targets[index].name, path)
        {
            tracing::warn!(%error, "export output not copied");
            self.fail(run, "files");
            return Ok(());
        }
        run.targets[index].progress = Progress::Copied;
        run.targets[index].revision = revision;
        let (done, total) = (run.done(), run.targets.len());
        if done == total {
            self.finalize(&state, run);
        } else {
            self.emit(run.session_id, done, total, ExportState::Running);
            state.runs.insert(run_id, run);
        }
        Ok(())
    }

    /// A `render_output` job could not render: the run fails (no retry, the failure is
    /// deterministic).
    pub fn output_failed(&self, run_id: i64, reason: &str) -> Result<()> {
        let mut state = self.state()?;
        if let Some(run) = state.runs.remove(&run_id) {
            self.fail(run, reason);
        }
        Ok(())
    }

    /// P69: the run's timing — so far while it runs, final once it completed in this process.
    pub fn timing(&self, run_id: i64) -> Option<ExportTiming> {
        if let Ok(state) = self.state()
            && let Some(run) = state.runs.get(&run_id)
        {
            return Some(run.timing(self.clock.now_ms()));
        }
        self.timings.lock().ok()?.get(&run_id).copied()
    }

    /// P69 (changelog 47 (2)): whether the RAW's master for `generation` is in the cache.
    pub fn master_on_disk(&self, raw: &str, generation: i64) -> bool {
        self.render.masters.path(raw, generation).is_file()
    }

    /// Whether a master asked for ahead of a publish is still with the editor.
    pub fn preparing(&self) -> bool {
        self.state().is_ok_and(|state| !state.prepared.is_empty())
    }

    /// Whether the prepared master of this RAW and generation could not be made (it is then
    /// rendered on demand by the publish or export that needs it).
    pub fn prepare_failed(&self, raw: &str, generation: i64) -> bool {
        self.state()
            .is_ok_and(|state| state.prepare_failed.get(raw) == Some(&generation))
    }

    /// P69 (changelog 47 (2)): asks the editing provider for the master of one photo ahead of a
    /// publish. Committed to the master cache when it arrives (like an export's master), so a
    /// later run finds it on disk; a run that starts meanwhile waits for it. Nothing is asked
    /// when a master render of the RAW is already out or waiting.
    pub fn prepare_master(&self, raw: &str, photo_id: i64, generation: i64) -> Result<bool> {
        let mut state = self.state()?;
        if state.masters.contains_key(raw) || state.waits.contains_key(raw) {
            return Ok(false);
        }
        let now_ms = self.clock.now_ms();
        let tracking = self
            .store
            .new_master_render_job(raw, 0, photo_id, generation, now_ms)?;
        if let Err(error) = self.editing.request_render(
            raw,
            RenderPurpose::Master,
            generation,
            tracking,
            self.clock.mono(),
            now_ms,
            false,
        ) {
            self.store
                .finish_render_job(tracking, Some("provider"), now_ms)?;
            return Err(error);
        }
        state.masters.insert(raw.to_string(), tracking);
        state
            .prepared
            .insert(tracking, (raw.to_string(), generation));
        Ok(true)
    }

    /// A prepared master arrived and no run waits for it: committed to the cache and its frame
    /// adopted (a photo whose frame no longer fits is flagged for review, as for any master).
    fn commit_prepared(&self, state: &mut State, render: &RenderCompletion, output: &Path) {
        let raw = render.raw.as_str();
        let committed = match self.render.masters.commit(raw, render.generation, output) {
            Ok(path) => path,
            Err(error) => {
                tracing::warn!(%error, raw, "prepared master not committed");
                state
                    .prepare_failed
                    .insert(raw.to_string(), render.generation);
                return;
            }
        };
        let frame = match self.render.imaging.dimensions(&committed) {
            Ok(Some(frame)) => frame,
            other => {
                tracing::warn!(master = %committed.display(), ok = other.is_ok(), "prepared master not readable");
                let _ = self.render.masters.remove_all(raw);
                state
                    .prepare_failed
                    .insert(raw.to_string(), render.generation);
                return;
            }
        };
        match self.store.adopt_frame(
            raw,
            Fingerprint {
                frame,
                source: FrameSource::Master,
                geometry: render.geometry.clone(),
            },
        ) {
            Ok(commit) => {
                for &(photo_id, session_id) in &commit.needs_review {
                    self.events.emit(AppEvent::PhotoChanged {
                        photo_id,
                        session_id,
                    });
                }
            }
            Err(error) => tracing::warn!(%error, raw, "prepared master's frame not adopted"),
        }
        tracing::info!(
            raw,
            adjusted_gen = render.generation,
            ms = (self.clock.now_ms() - render.issued_at_ms).max(0),
            "master prepared in advance"
        );
    }
}

impl ExportStaging for ExportService {
    /// Capture One delivered a master: committed to the cache, its size and geometry compared
    /// with the fingerprint (materialise, adopt the master size, or `needs_review`), then the
    /// waiting outputs of that RAW are queued — or every run holding a photo that now needs
    /// review fails with `needs_review` (the master stays).
    fn master_ready(&self, render: &RenderCompletion, output: &Path) -> Result<()> {
        tracing::info!(raw = %render.raw, adjusted_gen = render.generation, ms = (self.clock.now_ms() - render.issued_at_ms).max(0), "master rendered");
        let mut state = self.state()?;
        if state.masters.get(&render.raw) == Some(&render.tracking_job_id) {
            state.masters.remove(&render.raw);
        }
        state.waited_since.remove(&render.raw);
        let prepared = state.prepared.remove(&render.tracking_job_id).is_some();
        let raw = render.raw.as_str();
        let waiting = |t: &Target| t.raw == raw && t.progress == Progress::WaitingMaster;
        if !state
            .runs
            .values()
            .any(|run| run.targets.iter().any(waiting))
        {
            if prepared {
                self.commit_prepared(&mut state, render, output);
                return Ok(());
            }
            tracing::info!(job = render.job_id, "master for no running export ignored");
            return Ok(());
        }
        let generation = render.generation;
        let committed = match self.render.masters.commit(raw, generation, output) {
            Ok(path) => path,
            Err(error) => {
                tracing::warn!(%error, "master not committed");
                self.fail_runs(&mut state, waiting, "files");
                return Ok(());
            }
        };
        let Some(frame) = self.render.imaging.dimensions(&committed)? else {
            tracing::warn!(master = %committed.display(), "master not readable");
            self.fail_runs(&mut state, waiting, "render");
            return Ok(());
        };
        let commit = self.store.adopt_frame(
            raw,
            Fingerprint {
                frame,
                source: FrameSource::Master,
                geometry: render.geometry.clone(),
            },
        )?;
        if !commit.needs_review.is_empty() {
            let flagged: HashSet<i64> = commit.needs_review.iter().map(|(id, _)| *id).collect();
            for &(photo_id, session_id) in &commit.needs_review {
                self.events.emit(AppEvent::PhotoChanged {
                    photo_id,
                    session_id,
                });
            }
            self.fail_runs(
                &mut state,
                |t| flagged.contains(&t.photo_id),
                "needs_review",
            );
        }
        let mut broken = Vec::new();
        let now_ms = self.clock.now_ms();
        for run in state.runs.values_mut() {
            for index in 0..run.targets.len() {
                if !waiting(&run.targets[index]) {
                    continue;
                }
                run.targets[index].adjusted_gen = generation;
                if self.enqueue_output(run, &run.targets[index]).is_err() {
                    broken.push(run.id);
                    break;
                }
                run.targets[index].progress = Progress::Rendering;
            }
            run.note_masters_done(now_ms);
        }
        for id in broken {
            if let Some(run) = state.runs.remove(&id) {
                self.fail(run, "files");
            }
        }
        Ok(())
    }

    fn failed(&self, tracking_job_id: i64, reason: &str) -> Result<()> {
        let mut state = self.state()?;
        // P69: a prepared master that will not come is not asked for again at this generation
        // (unless the editor was only not ready); a run that needs it renders it on demand.
        if let Some((raw, generation)) = state.prepared.remove(&tracking_job_id) {
            tracing::info!(raw, reason, "prepared master not made");
            if editor_wait(reason).is_none() {
                state.prepare_failed.insert(raw, generation);
            }
        }
        let Some(raw) = state
            .masters
            .iter()
            .find(|(_, id)| **id == tracking_job_id)
            .map(|(raw, _)| raw.clone())
        else {
            return Ok(());
        };
        state.masters.remove(&raw);
        // Changelog 45 (P65): refused or timed out because Capture One was not ready: the runs
        // wait and the master is asked for again.
        if let Some(wait_reason) = editor_wait(reason) {
            let holder = state.runs.values().find_map(|run| {
                run.targets
                    .iter()
                    .find(|t| t.raw == raw && t.progress == Progress::WaitingMaster)
                    .map(|t| (run.id, t.photo_id, t.adjusted_gen))
            });
            if let Some((run_id, photo_id, generation)) = holder {
                let now = self.clock.now_ms();
                let wait = Wait {
                    since_ms: now,
                    next_ms: now + editor_wait_backoff_ms(1),
                    attempts: 1,
                    reason: wait_reason,
                    run_id,
                    photo_id,
                    generation,
                };
                self.begin_wait(&mut state, &raw, wait);
                self.refresh_waiting(&mut state);
            }
            return Ok(());
        }
        self.fail_runs(
            &mut state,
            |t| t.raw == raw && t.progress == Progress::WaitingMaster,
            reason,
        );
        Ok(())
    }

    fn tick(&self) -> Result<()> {
        ExportService::tick(self)
    }
}
