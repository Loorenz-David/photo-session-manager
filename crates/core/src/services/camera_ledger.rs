//! 10E MC-33 — the capture ledger service: the one worker that decides where a delivered camera
//! file goes, writes the `pending` row with its final path, renames without ever replacing a
//! file, and marks the row `done` (R-I25, R-I27, R-I28); the watchdog count (R-I29).
use crate::{
    Error, Result,
    domain::camera::{
        Association, Delivery, DeliveryOutcome, DeliveryRow, DeliveryState, Exposure,
        ExposureObserved, ExposureStatus, Link, NewDelivery, NewExposure, PendingUpdate,
        WATCHDOG_MAX_AGE_MS, WATCHDOG_MIN_AGE_MS,
    },
    mechanisms::{
        capture_ledger::{
            DuplicateDecision, FifoCandidate, Gap, PriorDelivery, associate, dup_name,
            duplicate_decision, file_key, is_complete, pending_count, seq_of, sequence_gaps,
            stem_key, target_folder,
        },
        hash::sha256_file,
    },
    ports::{
        camera::{CaptureSink, CompletedNames, DeliveryFs, RenameOutcome},
        clock::Clock,
        events::EventSink,
        store::{Store, StoreTx},
    },
    services::health::HealthService,
};
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::{Arc, Mutex, MutexGuard},
};
use unicode_normalization::UnicodeNormalization;

pub const FILES_NOT_RECEIVED: &str = "camera_files_not_received";
pub const PHOTO_SESSION_UNCERTAIN: &str = "camera_photo_session_uncertain";

/// One per app process, owned by the camera runtime and shared by every `LedgerService` it
/// builds (R-I18; P44 projection L3): the worker lock, the watchdog's raised flag and the
/// camera's last FTP report (P46).
#[derive(Default)]
pub struct LedgerShared {
    pub worker: Mutex<()>,
    pub raised: Mutex<bool>,
    /// The short form of the last `camera.ftp_state` (R-I29), appended to the
    /// `camera_files_not_received` detail while the watchdog raises; cleared by `dismiss()` and
    /// by a watchdog run that does not raise.
    pub ftp_note: Mutex<Option<String>>,
    pub gaps_seen: Mutex<HashMap<(String, u64, u64), i64>>,
    pub gaps_logged: Mutex<Vec<Gap>>,
    pub gaps_dismissed: Mutex<HashSet<(String, u64, u64)>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatchdogState {
    pub pending_count: usize,
    /// The numbering gaps over deliveries completed in the last 30 min; `g` = its length.
    pub gaps: Vec<Gap>,
    pub raised: bool,
    /// True on the transition into raised (P46 sends `ftp-status` then).
    pub first_raise: bool,
}

pub struct LedgerService {
    pub store: Arc<dyn Store>,
    pub clock: Arc<dyn Clock>,
    pub fs: Arc<dyn DeliveryFs>,
    pub events: Arc<dyn EventSink>,
    pub health: Arc<HealthService>,
    pub capture_root: Arc<dyn Fn() -> Option<PathBuf> + Send + Sync>,
    pub shared: Arc<LedgerShared>,
}

/// One file on its way to its final path.
struct Placement {
    id: i64,
    written: PathBuf,
    final_path: PathBuf,
    outcome: DeliveryOutcome,
    name: String,
    sha256: String,
    size: u64,
    completed_at: i64,
    association: Association,
    exposure_id: Option<i64>,
    session_id: Option<i64>,
    /// The matched exposure whose status this delivery decides (it was a candidate).
    matched: Option<Exposure>,
}

fn locked<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn path_text(path: &Path) -> String {
    path.to_string_lossy().nfc().collect()
}

fn base_name(name: &str) -> String {
    Path::new(name)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| name.to_owned())
}

/// The `n` of a trailing `__dup<n>` in the file stem of `path` (0 when none).
fn dup_suffix(path: &Path) -> u32 {
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    stem.rfind("__dup")
        .and_then(|at| stem[at + 5..].parse().ok())
        .unwrap_or(0)
}

fn prior_of(row: &DeliveryRow) -> PriorDelivery {
    PriorDelivery {
        id: row.id,
        file_key: row.file_key.clone(),
        stem_key: stem_key(&row.file_key),
        sha256: row.sha256.clone(),
        exposure_id: row.exposure_id,
        session_id: row.session_id,
        outcome: row.outcome,
        completed_at: row.completed_at,
        final_name: base_name(&row.final_path),
    }
}

/// The stored folder key of the session's item; `None` (→ `_Unassigned`) for no session, the
/// Unassigned session, or a session that no longer exists (R-I28).
fn folder_key_of(tx: &dyn StoreTx, session_id: Option<i64>) -> Result<Option<String>> {
    let Some(id) = session_id else {
        return Ok(None);
    };
    let session = match tx.session(id) {
        Ok(session) => session,
        Err(Error::NotFound(_)) => return Ok(None),
        Err(error) => return Err(error),
    };
    match session.item_id {
        Some(item) => Ok(Some(tx.item(item)?.folder_key)),
        None => Ok(None),
    }
}

impl LedgerService {
    fn root(&self) -> Result<PathBuf> {
        (self.capture_root)().ok_or_else(|| Error::Provider("capture root unavailable".into()))
    }

    /// The status the matched exposure gets from its non-duplicate deliveries (pending or done,
    /// `exclude` left out, plus this one when `adding`): `received` once the count reaches
    /// `expected_files` (1 when unknown), else `expecting`.
    fn status_for(&self, e: &Exposure, exclude: i64, adding: bool) -> Result<ExposureStatus> {
        let count = self
            .store
            .deliveries_for_exposure(e.id)?
            .iter()
            .filter(|r| r.id != exclude && r.outcome != DeliveryOutcome::Duplicate)
            .count()
            + usize::from(adding);
        Ok(if count >= e.expected_files.unwrap_or(1) as usize {
            ExposureStatus::Received
        } else {
            ExposureStatus::Expecting
        })
    }

    fn session_title(&self, session_id: Option<i64>) -> String {
        let title = session_id
            .and_then(|id| self.store.session(id).ok())
            .and_then(|dto| dto.session.item_id)
            .and_then(|item| self.store.item(item).ok())
            .map(|item| {
                if item.title.trim().is_empty() {
                    item.sku
                } else {
                    item.title
                }
            });
        title.unwrap_or_else(|| "Unassigned".into())
    }

    fn decide_and_place(&self, d: &Delivery) -> Result<DeliveryOutcome> {
        let root = self.root()?;
        let written = path_text(&d.written_path);
        let takeover = self.store.pending_delivery_for_written_path(&written)?;
        let takeover_id = takeover.as_ref().map(|t| t.id).unwrap_or(0);
        let key = file_key(&d.camera_filename);
        let stem = stem_key(&key);
        let mut rows = self.store.deliveries_by_key(&key)?;
        rows.extend(self.store.deliveries_by_stem(&stem)?);
        rows.retain(|r| r.state == DeliveryState::Done && r.id != takeover_id);
        rows.sort_by_key(|r| r.id);
        rows.dedup_by_key(|r| r.id);
        let prior: Vec<PriorDelivery> = rows.iter().map(prior_of).collect();
        let mut exposures = self.store.exposures(Some(ExposureStatus::Expecting))?;
        // A re-sent file taking over its pending row: the exposure that row's FIFO pick made
        // `received` is a candidate again (the decision is re-made, P44 contract).
        let mut reverted = None;
        if let Some(t) = &takeover
            && t.association == Association::Fifo
            && let Some(e) = t.exposure_id
            && !exposures.iter().any(|x| x.id == e)
            && let Some(mut x) = self
                .store
                .exposures(Some(ExposureStatus::Received))?
                .into_iter()
                .find(|x| x.id == e)
            && self.status_for(&x, t.id, false)? == ExposureStatus::Expecting
        {
            x.status = ExposureStatus::Expecting;
            exposures.push(x);
            exposures.sort_by_key(|x| (x.exposed_at, x.id));
            reverted = Some(e);
        }
        let candidates: Vec<FifoCandidate> = exposures
            .iter()
            .map(|e| FifoCandidate {
                exposure_id: e.id,
                exposed_at: e.exposed_at,
                session_id: e.session_id,
                expected_files: e.expected_files,
            })
            .collect();
        let decision = associate(&key, &stem, &d.sha256, d.completed_at, &prior, &candidates);
        let session_id = if decision.association == Association::None {
            self.store.active_session_id()?
        } else {
            decision.session_id
        };
        let (outcome, n) = match duplicate_decision(&key, &d.sha256, &prior) {
            DuplicateDecision::Photo => (DeliveryOutcome::Photo, 0),
            DuplicateDecision::Duplicate { .. } => (DeliveryOutcome::Duplicate, 0),
            DuplicateDecision::RenamedPhoto { n } => (DeliveryOutcome::RenamedPhoto, n),
        };
        let matched = decision
            .exposure_id
            .and_then(|id| exposures.iter().find(|x| x.id == id).cloned());
        let matched_status = match &matched {
            Some(e) => {
                Some(self.status_for(e, takeover_id, outcome != DeliveryOutcome::Duplicate)?)
            }
            None => None,
        };
        let name = base_name(&d.camera_filename);
        let mut final_path = PathBuf::new();
        let mut id = takeover_id;
        // The folder key is read in the same transaction that commits the pending row, so a
        // SKU edit cannot slip between the read and the freeze (R-I28, 05B MC-22).
        self.store.transaction(&mut |tx| {
            let target = target_folder(&root, folder_key_of(tx, session_id)?.as_deref());
            final_path = match outcome {
                DeliveryOutcome::Duplicate => root.join(".duplicates").join(&name),
                DeliveryOutcome::Photo => target.join(&name),
                DeliveryOutcome::RenamedPhoto => target.join(dup_name(&name, n)),
            };
            if takeover.is_some() {
                tx.update_pending_delivery(
                    id,
                    &PendingUpdate {
                        sha256: d.sha256.clone(),
                        size: d.size,
                        completed_at: d.completed_at,
                        final_path: path_text(&final_path),
                        outcome,
                        association: decision.association,
                        exposure_id: decision.exposure_id,
                        session_id,
                    },
                )?;
            } else {
                id = tx.insert_delivery(&NewDelivery {
                    exposure_id: decision.exposure_id,
                    camera_filename: d.camera_filename.clone(),
                    file_key: key.clone(),
                    sha256: d.sha256.clone(),
                    size: d.size,
                    via: d.via.clone(),
                    completed_at: d.completed_at,
                    association: decision.association,
                    outcome,
                    session_id,
                    written_path: written.clone(),
                    final_path: path_text(&final_path),
                    created_at: self.clock.now_ms(),
                })?;
            }
            if let (Some(e), Some(status)) = (&matched, matched_status) {
                tx.set_exposure_status(e.id, status)?;
            }
            if let Some(e) = reverted
                && decision.exposure_id != Some(e)
            {
                tx.set_exposure_status(e, ExposureStatus::Expecting)?;
            }
            Ok(())
        })?;
        let mut placement = Placement {
            id,
            written: d.written_path.clone(),
            final_path,
            outcome,
            name: name.clone(),
            sha256: d.sha256.clone(),
            size: d.size,
            completed_at: d.completed_at,
            association: decision.association,
            exposure_id: decision.exposure_id,
            session_id,
            matched,
        };
        self.place(&mut placement)?;
        if let Some(other) = decision.ambiguous_with.first() {
            self.health.set(
                PHOTO_SESSION_UNCERTAIN,
                format!(
                    "Photo {} may belong to {}",
                    d.camera_filename,
                    self.session_title(*other)
                ),
            );
        }
        Ok(placement.outcome)
    }

    /// Rename → `done`. On `EEXIST`: the same SHA-256 → `.duplicates/`; other bytes → the next
    /// `__dupN` in the same folder; a collision in `.duplicates/` → its `__dupN` there. The row's
    /// final path is rewritten before each further attempt. Any other rename error is returned
    /// and the row stays `pending` (R-I25, R-I27).
    fn place(&self, p: &mut Placement) -> Result<()> {
        loop {
            if let Some(parent) = p.final_path.parent() {
                self.fs.create_dir_all(parent)?;
            }
            match self.fs.rename_exclusive(&p.written, &p.final_path)? {
                RenameOutcome::Moved => break,
                RenameOutcome::Exists => {
                    let folder = p
                        .final_path
                        .parent()
                        .map(Path::to_path_buf)
                        .unwrap_or_default();
                    let became_duplicate = p.outcome != DeliveryOutcome::Duplicate
                        && sha256_file(&p.final_path)? == p.sha256;
                    p.final_path = if became_duplicate {
                        p.outcome = DeliveryOutcome::Duplicate;
                        let root = folder.parent().map(Path::to_path_buf).unwrap_or_default();
                        root.join(".duplicates").join(&p.name)
                    } else {
                        if p.outcome == DeliveryOutcome::Photo {
                            p.outcome = DeliveryOutcome::RenamedPhoto;
                        }
                        folder.join(dup_name(&p.name, dup_suffix(&p.final_path) + 1))
                    };
                    let status = match (&p.matched, became_duplicate) {
                        (Some(e), true) => Some((e.id, self.status_for(e, p.id, false)?)),
                        _ => None,
                    };
                    let update = PendingUpdate {
                        sha256: p.sha256.clone(),
                        size: p.size,
                        completed_at: p.completed_at,
                        final_path: path_text(&p.final_path),
                        outcome: p.outcome,
                        association: p.association,
                        exposure_id: p.exposure_id,
                        session_id: p.session_id,
                    };
                    self.store.transaction(&mut |tx| {
                        tx.update_pending_delivery(p.id, &update)?;
                        if let Some((e, s)) = status {
                            tx.set_exposure_status(e, s)?;
                        }
                        Ok(())
                    })?;
                }
            }
        }
        self.store.set_delivery_done(p.id)
    }

    /// R-I25 at every bind, under the worker lock: a `pending` row whose staging file exists is
    /// renamed again to its stored final path (the folder key is not re-read); one whose final
    /// file exists becomes `done`; neither → left `pending` and logged.
    pub fn recover_pending(&self) -> Result<()> {
        let _worker = locked(&self.shared.worker);
        for row in self.store.pending_deliveries()? {
            let written = PathBuf::from(&row.written_path);
            let final_path = PathBuf::from(&row.final_path);
            if written.exists() {
                let mut placement = Placement {
                    id: row.id,
                    written,
                    final_path,
                    outcome: row.outcome,
                    name: base_name(&row.camera_filename),
                    sha256: row.sha256.clone(),
                    size: row.size,
                    completed_at: row.completed_at,
                    association: row.association,
                    exposure_id: row.exposure_id,
                    session_id: row.session_id,
                    matched: None,
                };
                if let Err(error) = self.place(&mut placement) {
                    tracing::warn!(id = row.id, %error, "camera.recovery_failed");
                }
            } else if final_path.exists() {
                self.store.set_delivery_done(row.id)?;
            } else {
                tracing::warn!(
                    id = row.id,
                    written = %row.written_path,
                    final_path = %row.final_path,
                    "camera.pending_delivery_lost"
                );
            }
        }
        Ok(())
    }

    /// MC-33: whether the exposure's expected count is known and met by its non-duplicate
    /// deliveries (informational; nothing in the product waits for it).
    pub fn complete_for(&self, exposure_id: i64) -> Result<bool> {
        let expected = self
            .store
            .exposures(None)?
            .into_iter()
            .find(|e| e.id == exposure_id)
            .and_then(|e| e.expected_files);
        let count = self
            .store
            .deliveries_for_exposure(exposure_id)?
            .iter()
            .filter(|r| r.outcome != DeliveryOutcome::Duplicate)
            .count();
        Ok(is_complete(expected, count as u32))
    }

    /// R-I29: raises `camera_files_not_received` iff the pending count is above 0 and no
    /// transfer is active; clears it otherwise.
    pub fn watchdog(&self, now_ms: i64, transfer_active: bool) -> Result<WatchdogState> {
        let exposures: Vec<(ExposureStatus, i64)> = self
            .store
            .exposures(Some(ExposureStatus::Expecting))?
            .iter()
            .map(|e| (e.status, e.exposed_at))
            .collect();
        let count = pending_count(&exposures, now_ms);
        let names = self
            .store
            .completed_names_since(now_ms - WATCHDOG_MAX_AGE_MS + 1)?;
        let stems: Vec<String> = names
            .iter()
            .map(|(name, _, _)| stem_key(&file_key(name)))
            .collect();
        let gaps = sequence_gaps(&stems.iter().map(String::as_str).collect::<Vec<_>>());
        let mut seen = locked(&self.shared.gaps_seen);
        let keys: HashSet<_> = gaps
            .iter()
            .map(|g| (g.prefix.clone(), g.after, g.before))
            .collect();
        seen.retain(|key, _| keys.contains(key));
        locked(&self.shared.gaps_dismissed).retain(|key| keys.contains(key));
        for key in &keys {
            let detected_at = names
                .iter()
                .filter_map(|(name, _, at)| {
                    let (prefix, number) = seq_of(&stem_key(&file_key(name)))?;
                    (prefix == key.0 && number == key.2).then_some(*at)
                })
                .min()
                .unwrap_or(now_ms);
            seen.entry(key.clone()).or_insert(detected_at);
        }
        let old_count: usize = gaps
            .iter()
            .filter(|g| {
                let key = (g.prefix.clone(), g.after, g.before);
                now_ms - seen[&key] > WATCHDOG_MIN_AGE_MS
                    && now_ms - seen[&key] < WATCHDOG_MAX_AGE_MS
                    && !locked(&self.shared.gaps_dismissed).contains(&key)
            })
            .map(|g| g.missing as usize)
            .sum();
        drop(seen);
        let mut logged = locked(&self.shared.gaps_logged);
        if *logged != gaps {
            for gap in &gaps {
                tracing::info!(target: "psm::camera",
                    prefix = %gap.prefix,
                    after = gap.after,
                    before = gap.before,
                    missing = gap.missing,
                    "camera.sequence_gap"
                );
            }
            *logged = gaps.clone();
        }
        drop(logged);
        let count = count.max(old_count);
        let raise = count > 0 && !transfer_active;
        let mut raised = locked(&self.shared.raised);
        let first_raise = raise && !*raised;
        if raise {
            let mut detail = format!(
                "{count} photos were taken but not received. The camera may have stopped sending: on the camera, switch FTP Function off and on."
            );
            if !gaps.is_empty() {
                detail.push_str(&format!(" · {} gaps in the camera's numbering", gaps.len()));
            }
            if let Some(note) = locked(&self.shared.ftp_note).as_deref() {
                detail.push_str(&format!(" · {note}"));
            }
            self.health.set(FILES_NOT_RECEIVED, detail);
        } else {
            *locked(&self.shared.ftp_note) = None;
            self.health.clear(FILES_NOT_RECEIVED);
        }
        *raised = raise;
        Ok(WatchdogState {
            pending_count: count,
            gaps,
            raised: raise,
            first_raise,
        })
    }

    /// R-I29: the photographer dismissed the notice: the exposures it counted become
    /// `dismissed` and the item clears.
    pub fn dismiss(&self) -> Result<()> {
        let _worker = locked(&self.shared.worker);
        let now = self.clock.now_ms();
        for e in self.store.exposures(Some(ExposureStatus::Expecting))? {
            let age = now - e.exposed_at;
            if age > WATCHDOG_MIN_AGE_MS && age < WATCHDOG_MAX_AGE_MS {
                self.store
                    .set_exposure_status(e.id, ExposureStatus::Dismissed)?;
            }
        }
        let gaps = locked(&self.shared.gaps_seen)
            .keys()
            .cloned()
            .collect::<Vec<_>>();
        locked(&self.shared.gaps_dismissed).extend(gaps);
        *locked(&self.shared.raised) = false;
        *locked(&self.shared.ftp_note) = None;
        self.health.clear(FILES_NOT_RECEIVED);
        Ok(())
    }

    /// P46: the camera's FTP report (the short form of `camera.ftp_state`), shown on the
    /// `camera_files_not_received` item from the next raising watchdog run.
    pub fn set_ftp_note(&self, note: Option<String>) {
        *locked(&self.shared.ftp_note) = note;
    }

    /// Whether the watchdog's notice is raised (process-wide; survives a settings save).
    pub fn raised(&self) -> bool {
        *locked(&self.shared.raised)
    }

    /// Test and diagnostics reads.
    pub fn exposures(&self, status: Option<ExposureStatus>) -> Result<Vec<Exposure>> {
        self.store.exposures(status)
    }
    pub fn deliveries(&self) -> Result<Vec<DeliveryRow>> {
        self.store.deliveries()
    }
}

impl CaptureSink for LedgerService {
    fn exposure_observed(&self, e: ExposureObserved) -> Result<i64> {
        let _worker = locked(&self.shared.worker);
        let session_id = self.store.active_session_id()?;
        let at = e.at;
        let id = self.store.insert_exposure(&NewExposure {
            capture_id: e.capture_id.map(|c| c.0),
            exposed_at: at,
            initiated: e.initiated,
            session_id,
            created_at: self.clock.now_ms(),
        })?;
        self.store.match_late_exposure(id, at)?;
        Ok(id)
    }
    fn delivery(&self, d: Delivery) -> Result<DeliveryOutcome> {
        let _worker = locked(&self.shared.worker);
        self.decide_and_place(&d)
    }
    fn session_uncertain(&self) -> Result<()> {
        let _worker = locked(&self.shared.worker);
        self.store
            .set_exposures_status_where(ExposureStatus::Expecting, ExposureStatus::Uncertain)?;
        Ok(())
    }
    /// A no-op in P44: P46 computes `transfer_active` itself.
    fn transfer_in_progress(&self, _source: Link, _active: bool) {}
}

impl CompletedNames for LedgerService {
    fn completed_names_since(&self, since: i64) -> Vec<(String, u64, i64)> {
        self.store
            .completed_names_since(since)
            .unwrap_or_else(|error| {
                tracing::warn!(%error, "camera ledger names unavailable");
                Vec::new()
            })
    }
}
