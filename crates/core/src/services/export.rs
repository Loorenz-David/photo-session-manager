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
    mechanisms::naming::{
        ExportGroup, export_file_name, folder_key, item_json, output_names, review_label,
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
}

impl Run {
    fn done(&self) -> usize {
        self.targets
            .iter()
            .filter(|t| t.progress == Progress::Copied)
            .count()
    }
}

#[derive(Default)]
struct State {
    runs: HashMap<i64, Run>,
    /// RAW → tracking job id of the master render in flight (one `process` per RAW; every run
    /// that needs it waits for it).
    masters: HashMap<String, i64>,
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
        }
        Ok(ExportRunId(id))
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
        if !state.masters.contains_key(&target.raw) {
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
                return Err(error);
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
        let in_use: HashSet<&str> = state
            .runs
            .values()
            .flat_map(|run| run.targets.iter())
            .filter(|t| t.progress != Progress::Copied)
            .map(|t| t.raw.as_str())
            .collect();
        let raws: BTreeSet<String> = self
            .store
            .photos_in_session(session_id)?
            .into_iter()
            .filter(|p| p.classification != Classification::Deleted)
            .map(|p| p.raw_path)
            .collect();
        for raw in raws {
            if in_use.contains(raw.as_str()) || state.masters.contains_key(&raw) {
                continue;
            }
            self.render.masters.remove_all(&raw)?;
        }
        Ok(())
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
        let raw = render.raw.as_str();
        let waiting = |t: &Target| t.raw == raw && t.progress == Progress::WaitingMaster;
        if !state
            .runs
            .values()
            .any(|run| run.targets.iter().any(waiting))
        {
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
        let Some(raw) = state
            .masters
            .iter()
            .find(|(_, id)| **id == tracking_job_id)
            .map(|(raw, _)| raw.clone())
        else {
            return Ok(());
        };
        state.masters.remove(&raw);
        self.fail_runs(
            &mut state,
            |t| t.raw == raw && t.progress == Progress::WaitingMaster,
            reason,
        );
        Ok(())
    }
}
