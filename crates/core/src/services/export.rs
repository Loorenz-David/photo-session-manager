//! Local export (ch07 §Completion without an integration, 05A MC-8, 10A MC-16): every main and
//! condition photo rendered through the web recipe into a staging folder under its position name,
//! `item.json` beside them, then the staging folder swapped in place of `<export folder>/<key>/`.
//! The same path is step one of every publish.

use crate::{
    Error, Result,
    domain::{
        editing::RenderPurpose,
        item::Item,
        photo::{Classification, Integrity},
    },
    events::{AppEvent, ExportState},
    mechanisms::naming::{ExportGroup, export_file_name, folder_key, item_json},
    ports::{
        clock::Clock,
        editing::{EditingProvider, RenderCompletion},
        events::EventSink,
        store::Store,
    },
    services::{integrity::IntegrityCheck, previews::ExportStaging},
};
use std::{
    collections::{BTreeMap, HashMap},
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

type Snapshot = (
    Item,
    Vec<i64>,
    Vec<i64>,
    BTreeMap<String, serde_json::Value>,
);

/// The id of one export run (the `render_export` job row that records its photo-list snapshot).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExportRunId(pub i64);

struct Target {
    photo_id: i64,
    group: ExportGroup,
    name: String,
    tracking_job_id: i64,
    copied: bool,
}

struct Run {
    id: i64,
    session_id: i64,
    key: String,
    staging: PathBuf,
    item: Item,
    /// Every valid value of the item at the start of the run, keyed by attribute key (MC-8).
    attributes: BTreeMap<String, serde_json::Value>,
    targets: Vec<Target>,
}

impl Run {
    fn done(&self) -> usize {
        self.targets.iter().filter(|t| t.copied).count()
    }
}

pub struct ExportService {
    store: Arc<dyn Store>,
    editing: Arc<dyn EditingProvider>,
    events: Arc<dyn EventSink>,
    clock: Arc<dyn Clock>,
    files: Arc<dyn ExportFiles>,
    runs: Mutex<HashMap<i64, Run>>,
}

impl ExportService {
    pub fn new(
        store: Arc<dyn Store>,
        editing: Arc<dyn EditingProvider>,
        events: Arc<dyn EventSink>,
        clock: Arc<dyn Clock>,
        files: Arc<dyn ExportFiles>,
    ) -> Self {
        Self {
            store,
            editing,
            events,
            clock,
            files,
            runs: Mutex::new(HashMap::new()),
        }
    }

    fn runs(&self) -> Result<MutexGuard<'_, HashMap<i64, Run>>> {
        self.runs
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))
    }

    /// `<export folder>/<folder_key(current SKU)>/` of the session's item.
    pub fn folder(&self, session_id: i64) -> Result<PathBuf> {
        let (item, _, _, _) = self.snapshot(session_id)?;
        Ok(self.files.folder(&folder_key(&item.sku)?))
    }

    /// Starts an export of the session. Fails before any render when a photo's RAW changed or went
    /// missing (MC-3 check a); a second call while a run of the session is in flight returns that run.
    pub fn run(&self, session_id: i64) -> Result<ExportRunId> {
        let mut runs = self.runs()?;
        if let Some(run) = runs.values().find(|r| r.session_id == session_id) {
            return Ok(ExportRunId(run.id));
        }
        match self.start(session_id) {
            Ok(run) => {
                let id = run.id;
                let total = run.targets.len();
                self.emit(session_id, 0, total, ExportState::Running);
                if total == 0 {
                    self.finalize(run);
                } else {
                    runs.insert(id, run);
                }
                Ok(ExportRunId(id))
            }
            Err(error) => {
                self.emit(session_id, 0, 0, ExportState::Failed);
                Err(error)
            }
        }
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

    fn start(&self, session_id: i64) -> Result<Run> {
        let (item, main, condition, attributes) = self.snapshot(session_id)?;
        let key = folder_key(&item.sku)?;
        let check = IntegrityCheck::new(self.store.as_ref());
        for &photo_id in main.iter().chain(&condition) {
            match check.verify(photo_id)? {
                Integrity::Ok => {}
                Integrity::RawChanged => return Err(Error::RawChanged { photo_id }),
                Integrity::RawMissing => return Err(Error::RawMissing { photo_id }),
            }
        }
        let now_ms = self.clock.now_ms();
        let payload = serde_json::json!({
            "session_id": session_id,
            "key": key,
            "main": main,
            "condition": condition,
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
            session_id,
            key,
            staging,
            item,
            attributes,
            targets: Vec::new(),
        };
        for (group, ids) in [
            (ExportGroup::Main, &main),
            (ExportGroup::Condition, &condition),
        ] {
            for (index, &photo_id) in ids.iter().enumerate() {
                let issued = self.issue(&run, photo_id, group, index + 1, ids.len());
                match issued {
                    Ok(target) => run.targets.push(target),
                    Err(error) => {
                        self.abandon(&run, "provider");
                        return Err(error);
                    }
                }
            }
        }
        Ok(run)
    }

    fn issue(
        &self,
        run: &Run,
        photo_id: i64,
        group: ExportGroup,
        position: usize,
        n_g: usize,
    ) -> Result<Target> {
        let photo = self.store.photo(photo_id)?;
        let now_ms = self.clock.now_ms();
        let tracking_job_id =
            self.store
                .new_export_render_job(&photo.raw_path, run.id, photo_id, now_ms)?;
        if let Err(error) = self.editing.request_render(
            &photo.raw_path,
            RenderPurpose::Export,
            run.id,
            tracking_job_id,
            self.clock.mono(),
            now_ms,
        ) {
            self.store
                .finish_render_job(tracking_job_id, Some("provider"), now_ms)?;
            return Err(error);
        }
        Ok(Target {
            photo_id,
            group,
            name: export_file_name(&run.key, position, n_g, group),
            tracking_job_id,
            copied: false,
        })
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

    /// All renders are in: drop photos deleted or moved out of the session meanwhile (07A R-I6
    /// analogue), name the rest by their position in the frozen lists, write `item.json`, swap.
    fn finalize(&self, run: Run) {
        match self.write_and_swap(&run) {
            Ok(count) => {
                if let Err(error) = self
                    .store
                    .finish_render_job(run.id, None, self.clock.now_ms())
                {
                    tracing::warn!(%error, "export run not recorded as done");
                }
                self.emit(run.session_id, count, count, ExportState::Done);
            }
            Err(error) => {
                tracing::warn!(%error, "export could not be completed");
                self.fail(run, "files");
            }
        }
    }

    fn write_and_swap(&self, run: &Run) -> Result<usize> {
        let mut kept: BTreeMap<u8, Vec<&Target>> = BTreeMap::new();
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
            kept.entry(slot).or_default().push(target);
        }
        let mut lists: [Vec<String>; 2] = [Vec::new(), Vec::new()];
        for (slot, targets) in &kept {
            for (index, target) in targets.iter().enumerate() {
                let name = export_file_name(&run.key, index + 1, targets.len(), target.group);
                if name != target.name {
                    self.files.rename_in(&run.staging, &target.name, &name)?;
                }
                lists[usize::from(*slot)].push(name);
            }
        }
        let json = item_json(&run.item, &run.attributes, &lists[0], &lists[1]);
        self.files
            .write_in(&run.staging, "item.json", json.as_bytes())?;
        self.files.swap(&run.staging, &run.key, run.id)?;
        Ok(lists[0].len() + lists[1].len())
    }
}

impl ExportStaging for ExportService {
    fn completed(&self, render: &RenderCompletion, output: &Path) -> Result<()> {
        let mut runs = self.runs()?;
        let Some(run_id) = runs
            .values()
            .find(|run| {
                run.targets
                    .iter()
                    .any(|t| t.tracking_job_id == render.tracking_job_id && !t.copied)
            })
            .map(|run| run.id)
        else {
            tracing::info!(job = render.job_id, "render for no running export ignored");
            return Ok(());
        };
        let Some(mut run) = runs.remove(&run_id) else {
            return Ok(());
        };
        let Some(index) = run
            .targets
            .iter()
            .position(|t| t.tracking_job_id == render.tracking_job_id)
        else {
            return Ok(());
        };
        if let Err(error) = self
            .files
            .copy_in(&run.staging, &run.targets[index].name, output)
        {
            tracing::warn!(%error, "export render not copied");
            self.fail(run, "files");
            return Ok(());
        }
        run.targets[index].copied = true;
        let (done, total) = (run.done(), run.targets.len());
        if done == total {
            self.finalize(run);
        } else {
            self.emit(run.session_id, done, total, ExportState::Running);
            runs.insert(run_id, run);
        }
        Ok(())
    }

    fn failed(&self, tracking_job_id: i64, reason: &str) -> Result<()> {
        let mut runs = self.runs()?;
        let run_id = runs
            .values()
            .find(|run| {
                run.targets
                    .iter()
                    .any(|t| t.tracking_job_id == tracking_job_id)
            })
            .map(|run| run.id);
        if let Some(run) = run_id.and_then(|id| runs.remove(&id)) {
            self.fail(run, reason);
        }
        Ok(())
    }
}
