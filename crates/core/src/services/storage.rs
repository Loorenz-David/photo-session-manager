//! Storage clean-up and budget, part 1 (10F §1, §2, §4 levels 1–3, §6; P61).
//!
//! A low-priority background job: every tick it first checks whether a capture, an upload, a
//! render, a take-back, an export or a publish is running (then it does nothing), then deletes
//! at most [`BATCH_FILES`] files and waits [`BATCH_PAUSE`] before the next batch. What goes is
//! decided by [`plan_cleanup`] over three kinds of candidates only:
//! - verified duplicates in `<capture root>/.duplicates/` (the ledger knows the duplicate's hash,
//!   or the app hashed the file once, and a filed copy with that hash is on disk);
//! - Capture One's preview exports in `…/_session_manager/` the app has copied into its cache
//!   (byte-identical copy present);
//! - masters no running export or publish needs.
//!
//! Nothing else is ever deleted here: no RAW, no file of an item, nothing in Capture One's own
//! folders. Every deletion is permanent and logged (`storage.cleared rule=… path=… bytes=…`).

use crate::{
    Result,
    domain::{
        camera::{DeliveryOutcome, DeliveryState},
        storage::StorageReport,
    },
    events::AppEvent,
    mechanisms::{
        arrival::path_key,
        cleanup::{Candidate, CleanupKind, Deletion, plan_cleanup},
    },
    ports::{
        clock::Clock,
        events::EventSink,
        storage::{FileEntry, StorageFs},
        store::Store,
    },
    services::{export::ExportService, health::HealthService},
};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};

/// At most this many files per batch (10F §6).
pub const BATCH_FILES: usize = 20;
/// The pause between two batches (10F §6; delegated choice).
pub const BATCH_PAUSE: Duration = Duration::from_secs(2);
/// Sizes and the clean-up scan are refreshed at most this often (and after a clean-up).
pub const REFRESH_EVERY: Duration = Duration::from_secs(300);
/// Unknown files in `.duplicates/` hashed per scan (each is a RAW, ~47 MB).
pub const HASHES_PER_SCAN: usize = 4;
/// Health code (and header warning) when levels 1–3 cannot bring free space to the minimum.
pub const DISK_ALMOST_FULL: &str = "disk_almost_full";
/// The sub folder of Capture One's output location the app's recipes export into (10C).
pub const PREVIEW_EXPORT_FOLDER: &str = "_session_manager";
const DUPLICATES_FOLDER: &str = ".duplicates";

/// Where each kind of file lives.
#[derive(Debug, Clone)]
pub struct StorageLocations {
    /// `<document>/Capture` (MC-33's capture root).
    pub capture_root: PathBuf,
    /// `<document>/Output/_session_manager` (the app's recipes' export folder).
    pub preview_exports: PathBuf,
    /// `<data>/previews` (camera previews, adjusted previews, thumbnails).
    pub preview_cache: PathBuf,
    /// `<data>/masters` (10D MC-28).
    pub masters: PathBuf,
}

/// Why clean-up waits now (`None` = it may run): a capture, an upload, a render, a take-back,
/// an export or a publish in progress.
pub type PauseCheck = Arc<dyn Fn() -> Option<String> + Send + Sync>;

/// Preview exports the app has copied into its cache: export path → cache copy (10F §2).
#[derive(Default)]
pub struct CopiedExports(Mutex<HashMap<PathBuf, PathBuf>>);
impl CopiedExports {
    /// Called once the preview service copied `export` (Capture One's file) to `cache`. Only a
    /// JPEG in a `_session_manager` folder is ever noted.
    pub fn note(&self, export: &Path, cache: &Path) {
        if !is_preview_export(export) {
            return;
        }
        let export = resolved(export);
        if let Ok(mut map) = self.0.lock() {
            map.insert(export, cache.to_path_buf());
        }
    }
    fn snapshot(&self) -> Vec<(PathBuf, PathBuf)> {
        self.0
            .lock()
            .map(|map| map.iter().map(|(a, b)| (a.clone(), b.clone())).collect())
            .unwrap_or_default()
    }
    fn forget(&self, export: &Path) {
        if let Ok(mut map) = self.0.lock() {
            map.remove(export);
        }
    }
    fn is_empty(&self) -> bool {
        self.0.lock().map(|map| map.is_empty()).unwrap_or(true)
    }
}

fn resolved(path: &Path) -> PathBuf {
    match (path.parent(), path.file_name()) {
        (Some(parent), Some(name)) => std::fs::canonicalize(parent)
            .map(|parent| parent.join(name))
            .unwrap_or_else(|_| path.to_path_buf()),
        _ => path.to_path_buf(),
    }
}

fn is_jpeg(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("jpg") || e.eq_ignore_ascii_case("jpeg"))
}

fn is_preview_export(path: &Path) -> bool {
    is_jpeg(path)
        && path
            .parent()
            .and_then(Path::file_name)
            .is_some_and(|name| name == PREVIEW_EXPORT_FOLDER)
}

fn is_master_file(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| n.starts_with("master-") && n.ends_with(".tif"))
}

/// One file the service deleted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cleared {
    pub rule: String,
    pub path: PathBuf,
    pub bytes: u64,
}

/// What one tick did.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TickOutcome {
    /// Files deleted by this tick, in order.
    pub cleared: Vec<Cleared>,
    /// Set when the tick waited for running work.
    pub paused: Option<String>,
}

struct Queued {
    deletion: Deletion,
    modified_ms: i64,
}

#[derive(Default)]
struct Work {
    queue: VecDeque<Queued>,
    next_batch_at: Option<Instant>,
    last_scan: Option<Instant>,
    scan_requested: bool,
    exports_matched: bool,
    hashes: HashMap<(PathBuf, u64, i64), String>,
    sizes_at: Option<Instant>,
    sizes_stale: bool,
}

pub struct StorageService {
    store: Arc<dyn Store>,
    fs: Arc<dyn StorageFs>,
    clock: Arc<dyn Clock>,
    health: Arc<HealthService>,
    events: Arc<dyn EventSink>,
    export: Arc<ExportService>,
    locations: StorageLocations,
    pause: PauseCheck,
    copied: Arc<CopiedExports>,
    min_free_bytes: AtomicU64,
    work: Mutex<Work>,
    report: Mutex<Option<StorageReport>>,
    warning: Mutex<Option<String>>,
}

impl StorageService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        store: Arc<dyn Store>,
        fs: Arc<dyn StorageFs>,
        clock: Arc<dyn Clock>,
        health: Arc<HealthService>,
        events: Arc<dyn EventSink>,
        export: Arc<ExportService>,
        locations: StorageLocations,
        pause: PauseCheck,
        copied: Arc<CopiedExports>,
        min_free_bytes: u64,
    ) -> Self {
        Self {
            store,
            fs,
            clock,
            health,
            events,
            export,
            locations,
            pause,
            copied,
            min_free_bytes: AtomicU64::new(min_free_bytes),
            work: Mutex::new(Work::default()),
            report: Mutex::new(None),
            warning: Mutex::new(None),
        }
    }

    pub fn min_free_bytes(&self) -> u64 {
        self.min_free_bytes.load(Ordering::Relaxed)
    }
    /// A new minimum (Settings → Storage) is applied by the next tick.
    pub fn set_min_free_bytes(&self, bytes: u64) {
        self.min_free_bytes.store(bytes, Ordering::Relaxed);
        self.request_scan();
    }
    /// Has the next tick decide again (and refresh the sizes) instead of waiting 5 minutes.
    pub fn request_scan(&self) {
        if let Ok(mut work) = self.work.lock() {
            work.scan_requested = true;
            work.sizes_stale = true;
        }
    }
    /// The sizes last computed in the background (`None` until the first computation).
    pub fn report(&self) -> Option<StorageReport> {
        self.report.lock().ok().and_then(|r| r.clone())
    }
    /// `Disk almost full: …` while levels 1–3 cannot reach the minimum free space.
    pub fn warning(&self) -> Option<String> {
        self.warning.lock().ok().and_then(|w| w.clone())
    }

    /// One step of the background job (called every few hundred ms by its own thread).
    pub fn tick(&self) -> Result<TickOutcome> {
        if let Some(reason) = (self.pause)() {
            return Ok(TickOutcome {
                cleared: Vec::new(),
                paused: Some(reason),
            });
        }
        let now = self.clock.mono();
        let mut work = self
            .work
            .lock()
            .map_err(|e| crate::Error::InvalidData(e.to_string()))?;
        if work.next_batch_at.is_some_and(|at| now < at) {
            return Ok(TickOutcome::default());
        }
        work.next_batch_at = None;
        if work.queue.is_empty() {
            let due = work.scan_requested
                || !self.copied.is_empty()
                || work
                    .last_scan
                    .is_none_or(|at| now.saturating_duration_since(at) >= REFRESH_EVERY);
            if due {
                self.scan(&mut work, now)?;
                if work.queue.is_empty() {
                    // Nothing to clear: the next scan is at least one pause away.
                    work.next_batch_at = Some(now + BATCH_PAUSE);
                }
            }
        }
        let mut cleared = Vec::new();
        if !work.queue.is_empty() {
            let count = work.queue.len().min(BATCH_FILES);
            let batch: Vec<Queued> = work.queue.drain(..count).collect();
            cleared = self.delete(batch)?;
            work.next_batch_at = Some(now + BATCH_PAUSE);
            if !cleared.is_empty() {
                work.sizes_stale = true;
            }
        } else if work.sizes_stale
            || work
                .sizes_at
                .is_none_or(|at| now.saturating_duration_since(at) >= REFRESH_EVERY)
        {
            self.refresh_sizes()?;
            work.sizes_at = Some(now);
            work.sizes_stale = false;
        }
        Ok(TickOutcome {
            cleared,
            paused: None,
        })
    }

    fn capture_root(&self) -> PathBuf {
        std::fs::canonicalize(&self.locations.capture_root)
            .unwrap_or_else(|_| self.locations.capture_root.clone())
    }

    fn duplicates_dir(&self) -> PathBuf {
        self.capture_root().join(DUPLICATES_FOLDER)
    }

    fn free(&self) -> Option<(u64, String)> {
        match self.fs.free(&self.locations.capture_root) {
            Ok(free) => Some((free.available_bytes, free.disk)),
            Err(error) => {
                tracing::warn!(%error, "free disk space not read");
                None
            }
        }
    }

    /// Gathers the candidates, decides, queues the deletions and sets or clears the warning.
    fn scan(&self, work: &mut Work, now: Instant) -> Result<()> {
        work.scan_requested = false;
        work.last_scan = Some(now);
        let mut candidates: Vec<(Candidate, i64)> = Vec::new();
        self.duplicate_candidates(work, &mut candidates)?;
        self.export_candidates(work, &mut candidates)?;
        self.master_candidates(&mut candidates)?;
        let (free, _) = self.free().unwrap_or((u64::MAX, String::new()));
        let modified: HashMap<PathBuf, i64> = candidates
            .iter()
            .map(|(c, modified)| (c.path.clone(), *modified))
            .collect();
        let list: Vec<Candidate> = candidates.into_iter().map(|(c, _)| c).collect();
        let plan = plan_cleanup(&list, self.clock.now_ms(), free, self.min_free_bytes());
        work.queue = plan
            .deletions
            .into_iter()
            .map(|deletion| Queued {
                modified_ms: modified.get(&deletion.path).copied().unwrap_or_default(),
                deletion,
            })
            .collect();
        let warning = plan.short.then(|| {
            format!(
                "Disk almost full: {:.1} GB free; nothing more can be cleared safely",
                plan.free_after as f64 / 1e9
            )
        });
        self.set_warning(warning);
        Ok(())
    }

    fn set_warning(&self, warning: Option<String>) {
        let changed = match self.warning.lock() {
            Ok(mut slot) if *slot != warning => {
                *slot = warning.clone();
                true
            }
            _ => false,
        };
        match &warning {
            Some(text) => self.health.set(DISK_ALMOST_FULL, text.clone()),
            None => self.health.clear(DISK_ALMOST_FULL),
        }
        if changed {
            self.events.emit(AppEvent::HealthChanged);
        }
    }

    /// 10F §1: a file in `.duplicates/` is verified when a filed copy with the same SHA-256 is
    /// on disk; the ledger's hash is used when it knows the file, else the file is hashed once.
    fn duplicate_candidates(&self, work: &mut Work, out: &mut Vec<(Candidate, i64)>) -> Result<()> {
        let files = self.fs.files(&self.duplicates_dir())?;
        if files.is_empty() {
            return Ok(());
        }
        let rows = self.store.deliveries()?;
        let known: HashMap<&str, _> = rows
            .iter()
            .filter(|r| r.outcome == DeliveryOutcome::Duplicate)
            .map(|r| (r.final_path.as_str(), r))
            .collect();
        let mut filed: HashMap<&str, Vec<&str>> = HashMap::new();
        for row in rows
            .iter()
            .filter(|r| r.outcome != DeliveryOutcome::Duplicate && r.state == DeliveryState::Done)
        {
            filed
                .entry(row.sha256.as_str())
                .or_default()
                .push(row.final_path.as_str());
        }
        let filed_on_disk = |sha: &str| {
            filed.get(sha).is_some_and(|paths| {
                paths
                    .iter()
                    .any(|p| matches!(self.fs.entry(Path::new(p)), Ok(Some(_))))
            })
        };
        let mut hashed = 0;
        for file in files {
            let key = file.path.to_string_lossy().into_owned();
            let (sha, since_ms) = if let Some(row) = known.get(key.as_str()) {
                if row.size != file.bytes {
                    continue;
                }
                (row.sha256.clone(), row.completed_at)
            } else {
                let cache_key = (file.path.clone(), file.bytes, file.modified_ms);
                let sha = match work.hashes.get(&cache_key) {
                    Some(sha) => sha.clone(),
                    None if hashed < HASHES_PER_SCAN => {
                        hashed += 1;
                        let sha = self.fs.sha256(&file.path)?;
                        work.hashes.insert(cache_key, sha.clone());
                        sha
                    }
                    None => {
                        // The rest are hashed by the next scans, one batch pause apart.
                        work.scan_requested = true;
                        continue;
                    }
                };
                (sha, file.modified_ms)
            };
            if filed_on_disk(&sha) {
                out.push((
                    Candidate {
                        kind: CleanupKind::Duplicate,
                        path: file.path.clone(),
                        bytes: file.bytes,
                        since_ms,
                    },
                    file.modified_ms,
                ));
            }
        }
        Ok(())
    }

    /// 10F §2: an export the app copied (noted by the preview service, or, once per process, an
    /// export already in the folder whose byte-identical copy is in the cache).
    fn export_candidates(&self, work: &mut Work, out: &mut Vec<(Candidate, i64)>) -> Result<()> {
        if !work.exports_matched {
            work.exports_matched = true;
            self.match_existing_exports()?;
        }
        for (export, cache) in self.copied.snapshot() {
            let (Some(file), Some(copy)) = (self.fs.entry(&export)?, self.fs.entry(&cache)?) else {
                self.copied.forget(&export);
                continue;
            };
            if file.bytes != copy.bytes || self.fs.sha256(&export)? != self.fs.sha256(&cache)? {
                // Capture One wrote a newer export since; its own copy will note it again.
                self.copied.forget(&export);
                continue;
            }
            out.push((
                Candidate {
                    kind: CleanupKind::PreviewExport,
                    path: export,
                    bytes: file.bytes,
                    since_ms: file.modified_ms,
                },
                file.modified_ms,
            ));
        }
        Ok(())
    }

    fn match_existing_exports(&self) -> Result<()> {
        let exports: Vec<FileEntry> = self
            .fs
            .files(&self.locations.preview_exports)?
            .into_iter()
            .filter(|f| is_jpeg(&f.path))
            .collect();
        if exports.is_empty() {
            return Ok(());
        }
        let mut by_size: HashMap<u64, Vec<PathBuf>> = HashMap::new();
        for entry in self.fs.files_deep(&self.locations.preview_cache, &[])? {
            let adjusted = entry
                .path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("adjusted-") && is_jpeg(&entry.path));
            if adjusted {
                by_size.entry(entry.bytes).or_default().push(entry.path);
            }
        }
        for export in exports {
            let Some(copies) = by_size.get(&export.bytes) else {
                continue;
            };
            let sha = self.fs.sha256(&export.path)?;
            for copy in copies {
                if self.fs.sha256(copy)? == sha {
                    self.copied.note(&export.path, copy);
                    break;
                }
            }
        }
        Ok(())
    }

    /// The path keys of RAWs whose master a running export or publish needs.
    fn masters_needed(&self) -> Result<HashSet<String>> {
        let mut raws = self.export.masters_in_use()?;
        for id in self.store.active_publish_items()? {
            let item = self.store.publish_item(id)?;
            raws.extend(
                self.store
                    .photos_in_session(item.session_id)?
                    .into_iter()
                    .map(|p| p.raw_path),
            );
        }
        Ok(raws.iter().map(|raw| path_key(raw)).collect())
    }

    fn master_key(path: &Path) -> Option<String> {
        path.parent()
            .and_then(Path::file_name)
            .map(|n| n.to_string_lossy().into_owned())
    }

    /// 10F §4 level 3: masters no running publish or export needs (cleared only under pressure).
    fn master_candidates(&self, out: &mut Vec<(Candidate, i64)>) -> Result<()> {
        let files: Vec<FileEntry> = self
            .fs
            .files_deep(&self.locations.masters, &[])?
            .into_iter()
            .filter(|f| is_master_file(&f.path))
            .collect();
        if files.is_empty() {
            return Ok(());
        }
        let needed = self.masters_needed()?;
        for file in files {
            if Self::master_key(&file.path).is_some_and(|key| needed.contains(&key)) {
                continue;
            }
            out.push((
                Candidate {
                    kind: CleanupKind::Master,
                    path: file.path,
                    bytes: file.bytes,
                    since_ms: file.modified_ms,
                },
                file.modified_ms,
            ));
        }
        Ok(())
    }

    /// Whether `path` is in the one folder its kind may be cleared from (a guard: nothing
    /// outside these three places is ever deleted).
    fn allowed(&self, kind: CleanupKind, path: &Path) -> bool {
        match kind {
            CleanupKind::Duplicate => path.parent() == Some(self.duplicates_dir().as_path()),
            CleanupKind::PreviewExport => is_preview_export(path),
            CleanupKind::Master => {
                is_master_file(path)
                    && (path.starts_with(&self.locations.masters)
                        || std::fs::canonicalize(&self.locations.masters)
                            .is_ok_and(|root| path.starts_with(root)))
            }
        }
    }

    fn delete(&self, batch: Vec<Queued>) -> Result<Vec<Cleared>> {
        let needed = if batch.iter().any(|q| q.deletion.kind == CleanupKind::Master) {
            self.masters_needed()?
        } else {
            HashSet::new()
        };
        let mut cleared = Vec::new();
        for queued in batch {
            let deletion = queued.deletion;
            // The file must still be the one decided on (same size and time).
            let Some(current) = self.fs.entry(&deletion.path)? else {
                continue;
            };
            if current.bytes != deletion.bytes || current.modified_ms != queued.modified_ms {
                continue;
            }
            if deletion.kind == CleanupKind::Master
                && Self::master_key(&deletion.path).is_some_and(|key| needed.contains(&key))
            {
                continue;
            }
            if !self.allowed(deletion.kind, &deletion.path) {
                tracing::warn!(path = %deletion.path.display(), "storage: refused a path outside its folder");
                continue;
            }
            if let Err(error) = self.fs.remove(&deletion.path) {
                tracing::warn!(%error, path = %deletion.path.display(), "storage: not cleared");
                continue;
            }
            tracing::info!(
                "storage.cleared rule={} path={} bytes={}",
                deletion.rule.as_str(),
                deletion.path.display(),
                deletion.bytes
            );
            if deletion.kind == CleanupKind::PreviewExport {
                self.copied.forget(&deletion.path);
            }
            cleared.push(Cleared {
                rule: deletion.rule.as_str().into(),
                path: deletion.path,
                bytes: deletion.bytes,
            });
        }
        Ok(cleared)
    }

    fn refresh_sizes(&self) -> Result<()> {
        let sum = |files: Vec<FileEntry>| files.iter().map(|f| f.bytes).sum::<u64>();
        let (free_bytes, disk) = match self.free() {
            Some((bytes, disk)) => (Some(bytes), disk),
            None => (None, String::new()),
        };
        let report = StorageReport {
            free_bytes,
            disk,
            raws_bytes: sum(self.fs.files_deep(&self.capture_root(), &["CaptureOne"])?),
            duplicates_bytes: sum(self.fs.files(&self.duplicates_dir())?),
            preview_exports_bytes: sum(self.fs.files(&self.locations.preview_exports)?),
            preview_cache_bytes: sum(self.fs.files_deep(&self.locations.preview_cache, &[])?),
            masters_bytes: sum(self.fs.files_deep(&self.locations.masters, &[])?),
            computed_at_ms: self.clock.now_ms(),
        };
        if let Ok(mut slot) = self.report.lock() {
            *slot = Some(report);
        }
        Ok(())
    }
}
