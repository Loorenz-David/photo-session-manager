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
//! P62 adds retention after publish (10F §3, §4 levels 4–5, M41): an item that is published and
//! clean, not pinned, with a commerce integration configured, is cleared `retention_days` after
//! its publish (and, under pressure, earlier, oldest publish first) — but only after
//! [`retention::verify`] read Shopify back and found every published image there. Clearing marks
//! the item cleared (read-only from then on) and queues its files: its RAWs, the Capture One
//! files named after them, its duplicates, previews, masters and outputs. A RAW another
//! session that is not cleared still uses is kept.
//!
//! Nothing else is ever deleted here: no RAW outside that path, nothing in Capture One's own
//! folders not named after a cleared RAW. Every deletion is permanent and logged
//! (`storage.cleared rule=… path=… bytes=…`).

use crate::{
    Result,
    domain::{
        camera::{DeliveryOutcome, DeliveryState},
        storage::StorageReport,
    },
    events::AppEvent,
    mechanisms::{
        arrival::path_key,
        cleanup::{Candidate, CleanupKind, plan_cleanup},
        retention::{
            Eligibility, RetentionCandidate, RetentionInput, RetentionRule, eligibility,
            plan_retention,
        },
    },
    ports::{
        clock::Clock,
        commerce::CommerceAdapter,
        events::EventSink,
        storage::{FileEntry, StorageFs},
        store::Store,
    },
    services::{
        export::ExportService,
        health::HealthService,
        retention::{self, Verification},
    },
};
use std::{
    collections::{BTreeMap, HashMap, HashSet, VecDeque},
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering},
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
/// Health code for items not cleared because Shopify does not hold every published image (P62).
pub const RETENTION_NOT_CLEARED: &str = "retention_not_cleared";
/// Items verified against Shopify and cleared per scan (10F §6: small batches); the rest follow
/// at the next scans.
pub const ITEMS_PER_SCAN: usize = 5;
/// An item Shopify did not verify is read again at most this often.
pub const VERIFY_RETRY: Duration = Duration::from_secs(900);
/// Capture One's folder beside the RAWs (settings sidecars, cache).
const CAPTURE_ONE_FOLDER: &str = "CaptureOne";

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
    /// `<data>/outputs` (10D MC-28; P62 clears a cleared item's outputs).
    pub outputs: PathBuf,
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

/// 10F §3 / Q4: a file of Capture One's folder `capture_one` (`<RAW folder>/CaptureOne`) named
/// after the RAW `name`: `Settings*/<name>.cos`, or `Cache/**/<name>.*`. Nothing else there
/// (another RAW's files, the catalogue) ever matches.
fn capture_one_file_of(capture_one: &Path, path: &Path, name: &str) -> bool {
    let Ok(relative) = path.strip_prefix(capture_one) else {
        return false;
    };
    let parts: Vec<String> = relative
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();
    let Some(file) = parts.last() else {
        return false;
    };
    match parts.first().map(String::as_str) {
        Some(first) if first.starts_with("Settings") => {
            parts.len() == 2 && *file == format!("{name}.cos")
        }
        Some("Cache") => parts.len() >= 2 && file.starts_with(&format!("{name}.")),
        _ => false,
    }
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

/// What a queued file is: one of part 1's kinds, or a file of an item being cleared (P62).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum QueuedKind {
    Cleanup(CleanupKind),
    Item(i64),
}

struct Queued {
    kind: QueuedKind,
    path: PathBuf,
    bytes: u64,
    rule: &'static str,
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
    /// Items not cleared because the verification failed: item id → health line (P62).
    not_cleared: BTreeMap<i64, String>,
    /// Items whose verification failed are not read again before this instant.
    verify_after: HashMap<i64, Instant>,
}

/// What happened to one item chosen for clearing.
enum ItemClearing {
    /// Verified, marked cleared, its files queued.
    Cleared,
    /// The verification failed: the health line.
    NotCleared(String),
    /// It no longer qualifies, or work started meanwhile: looked at again by the next scan.
    Skipped,
}

/// Retention after publish (P62): the commerce adapter the verification reads from. Without
/// one (H6) nothing is cleared by retention or by budget levels 4–5.
pub struct RetentionConfig {
    pub commerce: Arc<dyn CommerceAdapter>,
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
    retention_days: AtomicU32,
    retention: Option<RetentionConfig>,
    /// Set by [`StorageService::request_scan`] without waiting for a running scan.
    requested: AtomicBool,
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
            retention_days: AtomicU32::new(4),
            retention: None,
            requested: AtomicBool::new(false),
            work: Mutex::new(Work::default()),
            report: Mutex::new(None),
            warning: Mutex::new(None),
        }
    }

    /// P62: retention after publish, reading Shopify through `config` (H6: without it nothing
    /// is cleared by retention), kept `retention_days` after publish.
    pub fn with_retention(mut self, config: Option<RetentionConfig>, retention_days: u32) -> Self {
        self.retention = config;
        self.retention_days = AtomicU32::new(retention_days);
        self
    }
    pub fn retention_days(&self) -> u32 {
        self.retention_days.load(Ordering::Relaxed)
    }
    /// A new retention period (Settings → Storage) is applied by the next tick.
    pub fn set_retention_days(&self, days: u32) {
        self.retention_days.store(days, Ordering::Relaxed);
        self.request_scan();
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
        self.requested.store(true, Ordering::Relaxed);
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
        if self.requested.swap(false, Ordering::Relaxed) {
            work.scan_requested = true;
            work.sizes_stale = true;
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
                kind: QueuedKind::Cleanup(deletion.kind),
                modified_ms: modified.get(&deletion.path).copied().unwrap_or_default(),
                path: deletion.path,
                bytes: deletion.bytes,
                rule: deletion.rule.as_str(),
            })
            .collect();
        // Levels 4–5 (P62) start from what levels 1–3 leave.
        let free_after = self.retention_scan(work, plan.free_after, now)?;
        let short = free_after < self.min_free_bytes();
        let warning = short.then(|| {
            format!(
                "Disk almost full: {:.1} GB free; nothing more can be cleared safely",
                free_after as f64 / 1e9
            )
        });
        self.set_warning(warning);
        Ok(())
    }

    /// 10F §3 and §4 levels 4–5 (P62): decides which published items to clear, verifies each
    /// against Shopify (M41), marks the verified ones cleared and queues their files. Returns the
    /// free space expected once everything queued is deleted.
    fn retention_scan(
        &self,
        work: &mut Work,
        free_after_levels: u64,
        mono: Instant,
    ) -> Result<u64> {
        let Some(config) = &self.retention else {
            self.set_not_cleared(work, BTreeMap::new());
            return Ok(free_after_levels);
        };
        let min_free = self.min_free_bytes();
        let now = self.clock.now_ms();
        let days = self.retention_days();
        let cleared = retention::cleared_sessions(self.store.as_ref())?;
        let mut eligible: Vec<(i64, i64, String, RetentionCandidate)> = Vec::new();
        for record in self.store.session_list_records()? {
            let Some(item_id) = record.session.session.item_id else {
                continue;
            };
            let state = self.store.item_retention(item_id)?;
            let input = RetentionInput {
                integration: true,
                published_at: record.session.session.published_at,
                dirty: record.session.dirty,
                latest_step: record.latest_step,
                keep_files: state.keep_files,
                cleared: state.cleared_at.is_some(),
            };
            let due = match eligibility(input, now, days) {
                Eligibility::Never => continue,
                Eligibility::Due => true,
                Eligibility::NotYetDue { .. } => false,
            };
            eligible.push((
                item_id,
                record.session.session.id,
                record.sku,
                RetentionCandidate {
                    item_id,
                    published_at: record.session.session.published_at.unwrap_or_default(),
                    bytes: 0,
                    due,
                },
            ));
        }
        let needs_budget = free_after_levels < min_free;
        let mut remaining: Vec<RetentionCandidate> = Vec::new();
        work.verify_after.retain(|_, at| *at > mono);
        for (item_id, session_id, _, candidate) in eligible.iter_mut() {
            if work.verify_after.contains_key(item_id) {
                continue;
            }
            if candidate.due || needs_budget {
                candidate.bytes = self
                    .item_files(*item_id, *session_id, &cleared)?
                    .iter()
                    .map(|f| f.bytes)
                    .sum();
                remaining.push(*candidate);
            }
        }
        let mut free = free_after_levels;
        let mut not_cleared = BTreeMap::new();
        let mut processed = 0;
        loop {
            let plan = plan_retention(&remaining, free, min_free);
            let Some(&(item_id, rule)) = plan.first() else {
                break;
            };
            if processed == ITEMS_PER_SCAN {
                // The rest follow at the next scans; count them as freed for the warning.
                work.scan_requested = true;
                free = plan.iter().fold(free, |free, (id, _)| {
                    free.saturating_add(
                        remaining
                            .iter()
                            .find(|c| c.item_id == *id)
                            .map_or(0, |c| c.bytes),
                    )
                });
                break;
            }
            processed += 1;
            let index = remaining
                .iter()
                .position(|c| c.item_id == item_id)
                .expect("planned item is a candidate");
            let candidate = remaining.remove(index);
            let (_, session_id, sku, _) = eligible
                .iter()
                .find(|(id, ..)| *id == item_id)
                .expect("planned item is eligible")
                .clone();
            match self.clear_item(work, config, item_id, session_id, &sku, rule)? {
                ItemClearing::Cleared => free = free.saturating_add(candidate.bytes),
                ItemClearing::NotCleared(line) => {
                    work.verify_after.insert(item_id, mono + VERIFY_RETRY);
                    not_cleared.insert(item_id, line);
                }
                ItemClearing::Skipped => {}
            }
        }
        // Items checked earlier and not read again this time (retry delay) stay reported while
        // they would still be cleared.
        for (item_id, line) in &work.not_cleared {
            let still = eligible
                .iter()
                .any(|(id, _, _, c)| id == item_id && (c.due || needs_budget));
            if still && !not_cleared.contains_key(item_id) {
                not_cleared.insert(*item_id, line.clone());
            }
        }
        for (item_id, ..) in &eligible {
            if let Some(state) = self.store.item_retention(*item_id).ok()
                && state.cleared_at.is_some()
            {
                not_cleared.remove(item_id);
            }
        }
        self.set_not_cleared(work, not_cleared);
        Ok(free)
    }

    /// One item: verify against Shopify, re-check it is still published and clean, mark it
    /// cleared and queue its files.
    fn clear_item(
        &self,
        work: &mut Work,
        config: &RetentionConfig,
        item_id: i64,
        session_id: i64,
        sku: &str,
        rule: RetentionRule,
    ) -> Result<ItemClearing> {
        let images = match retention::verify(
            self.store.as_ref(),
            config.commerce.as_ref(),
            item_id,
            session_id,
        ) {
            Ok(Verification::Holds(images)) => images,
            Ok(Verification::Missing(n)) => {
                let line = format!("Not cleared: {sku} — Shopify is missing {n} images");
                tracing::warn!("storage.not_cleared item={sku} missing={n}");
                return Ok(ItemClearing::NotCleared(line));
            }
            Err(error) => {
                tracing::warn!(%error, "storage.not_cleared item={sku} read failed");
                return Ok(ItemClearing::NotCleared(format!(
                    "Not cleared: {sku} — Shopify could not be read"
                )));
            }
        };
        // The verification took a while: the item must still qualify (M41).
        let session = self.store.session(session_id)?;
        let state = self.store.item_retention(item_id)?;
        let latest = self.store.latest_publish(session_id)?;
        let still = !matches!(
            eligibility(
                RetentionInput {
                    integration: true,
                    published_at: session.session.published_at,
                    dirty: session.dirty,
                    latest_step: latest.map(|l| l.step),
                    keep_files: state.keep_files,
                    cleared: state.cleared_at.is_some(),
                },
                self.clock.now_ms(),
                0,
            ),
            Eligibility::Never
        );
        if !still || (self.pause)().is_some() {
            work.scan_requested = true;
            return Ok(ItemClearing::Skipped);
        }
        self.store
            .mark_item_cleared(item_id, self.clock.now_ms(), &images)?;
        tracing::info!(
            "storage.item_cleared item={sku} rule={} images={}",
            rule.as_str(),
            images.len()
        );
        self.events.emit(AppEvent::SessionChanged { session_id });
        let cleared = retention::cleared_sessions(self.store.as_ref())?;
        for file in self.item_files(item_id, session_id, &cleared)? {
            work.queue.push_back(Queued {
                kind: QueuedKind::Item(item_id),
                path: file.path,
                bytes: file.bytes,
                rule: rule.as_str(),
                modified_ms: file.modified_ms,
            });
        }
        Ok(ItemClearing::Cleared)
    }

    fn set_not_cleared(&self, work: &mut Work, not_cleared: BTreeMap<i64, String>) {
        if work.not_cleared == not_cleared {
            return;
        }
        if not_cleared.is_empty() {
            self.health.clear(RETENTION_NOT_CLEARED);
        } else {
            let text: Vec<&str> = not_cleared.values().map(String::as_str).collect();
            self.health.set(RETENTION_NOT_CLEARED, text.join("; "));
        }
        work.not_cleared = not_cleared;
        self.events.emit(AppEvent::HealthChanged);
    }

    /// The local files of an item (10F §3, Q1, Q4): for every RAW of its session (every
    /// classification) that no other, not cleared, session uses — the RAW, the files in its
    /// folder's `CaptureOne/Settings*/` named `<RAW name>.cos` and in `CaptureOne/Cache/` named
    /// `<RAW name>.*`, its previews, masters and outputs — and the files in `.duplicates/`
    /// whose SHA-256 is one of those RAWs'.
    fn item_files(
        &self,
        item_id: i64,
        session_id: i64,
        cleared: &HashSet<i64>,
    ) -> Result<Vec<FileEntry>> {
        let mut out: Vec<FileEntry> = Vec::new();
        let mut raws: Vec<String> = Vec::new();
        let mut hashes: HashSet<String> = HashSet::new();
        for photo in self.store.photos_in_session(session_id)? {
            if raws.contains(&photo.raw_path) {
                continue;
            }
            let shared = self
                .store
                .photos_for_raw(&photo.raw_path)?
                .iter()
                .any(|p| p.session_id != session_id && !cleared.contains(&p.session_id));
            if shared {
                tracing::info!(
                    item = item_id,
                    raw = photo.raw_path,
                    "storage: RAW kept, another session uses it"
                );
                raws.push(photo.raw_path.clone());
                continue;
            }
            if let Some(hash) = &photo.raw_hash {
                hashes.insert(hash.clone());
            }
            raws.push(photo.raw_path.clone());
            let raw = Path::new(&photo.raw_path);
            if let Some(file) = self.fs.entry(raw)? {
                out.push(file);
            }
            if let (Some(folder), Some(name)) = (raw.parent(), raw.file_name()) {
                let name = name.to_string_lossy();
                let capture_one = folder.join(CAPTURE_ONE_FOLDER);
                for file in self.fs.files_deep(&capture_one, &[])? {
                    if capture_one_file_of(&capture_one, &file.path, &name) {
                        out.push(file);
                    }
                }
            }
            let key = path_key(&photo.raw_path);
            for root in [
                &self.locations.preview_cache,
                &self.locations.masters,
                &self.locations.outputs,
            ] {
                out.extend(self.fs.files_deep(&root.join(&key[..2]).join(&key), &[])?);
            }
        }
        if !hashes.is_empty() {
            let names: HashSet<String> = raws
                .iter()
                .filter_map(|r| Path::new(r).file_name())
                .map(|n| n.to_string_lossy().into_owned())
                .collect();
            let known: HashMap<String, String> = self
                .store
                .deliveries()?
                .into_iter()
                .filter(|r| r.outcome == DeliveryOutcome::Duplicate)
                .map(|r| (r.final_path, r.sha256))
                .collect();
            for file in self.fs.files(&self.duplicates_dir())? {
                let key = file.path.to_string_lossy().into_owned();
                let sha = match known.get(&key) {
                    Some(sha) => sha.clone(),
                    None => {
                        let named = file
                            .path
                            .file_name()
                            .is_some_and(|n| names.contains(n.to_string_lossy().as_ref()));
                        if !named {
                            continue;
                        }
                        self.fs.sha256(&file.path)?
                    }
                };
                if hashes.contains(&sha) {
                    out.push(file);
                }
            }
        }
        out.sort_by(|a, b| a.path.cmp(&b.path));
        out.dedup_by(|a, b| a.path == b.path);
        Ok(out)
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
    /// outside these places is ever deleted).
    fn allowed(&self, kind: QueuedKind, path: &Path) -> Result<bool> {
        let kind = match kind {
            QueuedKind::Cleanup(kind) => kind,
            QueuedKind::Item(item_id) => return self.allowed_item_file(item_id, path),
        };
        Ok(match kind {
            CleanupKind::Duplicate => path.parent() == Some(self.duplicates_dir().as_path()),
            CleanupKind::PreviewExport => is_preview_export(path),
            CleanupKind::Master => {
                is_master_file(path)
                    && (path.starts_with(&self.locations.masters)
                        || std::fs::canonicalize(&self.locations.masters)
                            .is_ok_and(|root| path.starts_with(root)))
            }
        })
    }

    /// M41 at the moment of deletion: the item is marked cleared (it passed the verification),
    /// and the file is one of its RAWs, a Capture One file named after one of them, a file of
    /// one of their cache folders, or a duplicate in `.duplicates/`.
    fn allowed_item_file(&self, item_id: i64, path: &Path) -> Result<bool> {
        if self.store.item_retention(item_id)?.cleared_at.is_none() {
            return Ok(false);
        }
        if path.parent() == Some(self.duplicates_dir().as_path()) {
            return Ok(true);
        }
        let Some(session_id) = self.store.session_for_item(item_id)? else {
            return Ok(false);
        };
        let under = |root: &Path| {
            path.starts_with(root)
                || std::fs::canonicalize(root).is_ok_and(|root| path.starts_with(root))
        };
        for photo in self.store.photos_in_session(session_id)? {
            let raw = Path::new(&photo.raw_path);
            if raw == path {
                return Ok(true);
            }
            if let (Some(folder), Some(name)) = (raw.parent(), raw.file_name())
                && capture_one_file_of(
                    &folder.join(CAPTURE_ONE_FOLDER),
                    path,
                    &name.to_string_lossy(),
                )
            {
                return Ok(true);
            }
            let key = path_key(&photo.raw_path);
            for root in [
                &self.locations.preview_cache,
                &self.locations.masters,
                &self.locations.outputs,
            ] {
                if under(&root.join(&key[..2]).join(&key)) {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    fn delete(&self, batch: Vec<Queued>) -> Result<Vec<Cleared>> {
        let needed = if batch
            .iter()
            .any(|q| q.kind == QueuedKind::Cleanup(CleanupKind::Master))
        {
            self.masters_needed()?
        } else {
            HashSet::new()
        };
        let mut cleared = Vec::new();
        for queued in batch {
            // The file must still be the one decided on (same size and time).
            let Some(current) = self.fs.entry(&queued.path)? else {
                continue;
            };
            if current.bytes != queued.bytes || current.modified_ms != queued.modified_ms {
                continue;
            }
            if queued.kind == QueuedKind::Cleanup(CleanupKind::Master)
                && Self::master_key(&queued.path).is_some_and(|key| needed.contains(&key))
            {
                continue;
            }
            if !self.allowed(queued.kind, &queued.path)? {
                tracing::warn!(path = %queued.path.display(), "storage: refused a path outside its folder");
                continue;
            }
            if let Err(error) = self.fs.remove(&queued.path) {
                tracing::warn!(%error, path = %queued.path.display(), "storage: not cleared");
                continue;
            }
            tracing::info!(
                "storage.cleared rule={} path={} bytes={}",
                queued.rule,
                queued.path.display(),
                queued.bytes
            );
            if queued.kind == QueuedKind::Cleanup(CleanupKind::PreviewExport) {
                self.copied.forget(&queued.path);
            }
            cleared.push(Cleared {
                rule: queued.rule.into(),
                path: queued.path,
                bytes: queued.bytes,
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
