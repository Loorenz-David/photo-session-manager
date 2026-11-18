use super::scripting::CaptureOneScripting;
use psm_core::{
    Result,
    domain::editing::StyleOutcome,
    ports::{camera::ArrivalFollow, clock::Clock, editing::EditingProvider, store::Store},
    services::{
        editing_startup::EDITOR_NOT_FOLLOWING, health::HealthService, previews::PreviewService,
    },
};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

pub const VARIANT_WAIT_POLL_MS: u64 = 1_000;
pub const VARIANT_WAIT_MAX_MS: u64 = 15_000;
/// Changelog 45 (P65): at most one sync of a folder per this interval; arrivals meanwhile are
/// served by one more sync after it.
pub const SYNC_MIN_INTERVAL_MS: u64 = 1_000;

#[derive(Default)]
struct SyncState {
    last: Option<Instant>,
    pending: bool,
}

struct WaitState {
    since: Instant,
    last_poll: Instant,
}

pub struct CaptureOneFollow {
    pub scripting: Arc<dyn CaptureOneScripting>,
    pub editing: Arc<dyn EditingProvider>,
    pub store: Arc<dyn Store>,
    pub previews: Arc<PreviewService>,
    pub health: Arc<HealthService>,
    pub clock: Arc<dyn Clock>,
    pub style_name: String,
    current: Mutex<Option<PathBuf>>,
    waiting: Mutex<HashMap<String, WaitState>>,
    style_lock: Mutex<()>,
    /// Whether Capture One ran at the last tick (P60: its (re)opening is noticed here).
    was_running: Mutex<Option<bool>>,
    /// Changelog 45 (P65): the folders whose collection Capture One is asked to synchronise.
    syncs: Mutex<HashMap<PathBuf, SyncState>>,
    /// A16: a session switch reached Capture One; the next tick looks at the opened session's
    /// previews (the switch itself never waits for that).
    switched: Mutex<Option<Instant>>,
}
impl CaptureOneFollow {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        scripting: Arc<dyn CaptureOneScripting>,
        editing: Arc<dyn EditingProvider>,
        store: Arc<dyn Store>,
        previews: Arc<PreviewService>,
        health: Arc<HealthService>,
        clock: Arc<dyn Clock>,
        style_name: String,
    ) -> Self {
        Self {
            scripting,
            editing,
            store,
            previews,
            health,
            clock,
            style_name,
            current: Mutex::new(None),
            waiting: Mutex::new(HashMap::new()),
            style_lock: Mutex::new(()),
            was_running: Mutex::new(None),
            syncs: Mutex::new(HashMap::new()),
            switched: Mutex::new(None),
        }
    }
    /// A16: what follows a session switch, on the background loop: only the opened session's
    /// photos are looked at again (cost: that session's photos, never the library). The
    /// library-wide missing-file check stays at startup, window focus and Capture One reopening.
    fn switch_follow_up(&self) {
        let Some(asked) = self.switched.lock().ok().and_then(|mut s| s.take()) else {
            return;
        };
        let started = Instant::now();
        let session = self.store.active_session_id();
        let result = match &session {
            Ok(session) => self.previews.session_opened(*session),
            Err(error) => Err(psm_core::Error::InvalidData(error.to_string())),
        };
        tracing::info!(
            target: "psm::capture_one",
            "capture_one.switch_follow_up session={} photos={} waited_ms={} ms={} result={}",
            session
                .ok()
                .flatten()
                .map_or("none".to_owned(), |id| id.to_string()),
            result.as_ref().map_or(0, |n| *n),
            started.saturating_duration_since(asked).as_millis(),
            started.elapsed().as_millis(),
            result
                .as_ref()
                .map_or_else(|e| format!("error: {e}"), |_| "ok".into()),
        );
    }
    /// A16: the arrival's style eligibility and variant wait, recorded before anything can ask for
    /// its first preview (A10 deferred them with the Capture One work, so the preview job could
    /// run first and render the photo without its style).
    fn note_arrival(&self, raw: &str) {
        if let Err(error) = self.store.mark_style_eligible(raw) {
            tracing::warn!(%error, raw, "style eligibility could not be recorded");
        }
        self.previews.wait_for_variant(raw);
        let now = self.clock.mono();
        if let Ok(mut waiting) = self.waiting.lock() {
            waiting.entry(raw.into()).or_insert(WaitState {
                since: now,
                last_poll: now,
            });
        }
    }
    /// Changelog 45 (P65): a photo was filed into `folder`; its collection is synchronised.
    fn request_sync(&self, folder: &Path) {
        if let Ok(mut syncs) = self.syncs.lock() {
            syncs.entry(folder.to_path_buf()).or_default().pending = true;
        }
    }
    /// Sends the syncs that are due: at most one per folder per `SYNC_MIN_INTERVAL_MS`. A sync
    /// refused because "Take the camera back" holds Capture One stays pending and is sent at the
    /// first tick after it.
    fn run_syncs(&self) {
        let now = self.clock.mono();
        let due: Vec<PathBuf> = match self.syncs.lock() {
            Ok(syncs) => syncs
                .iter()
                .filter(|(_, state)| {
                    state.pending
                        && state.last.is_none_or(|last| {
                            now.saturating_duration_since(last)
                                >= Duration::from_millis(SYNC_MIN_INTERVAL_MS)
                        })
                })
                .map(|(folder, _)| folder.clone())
                .collect(),
            Err(_) => return,
        };
        for folder in due {
            let started = Instant::now();
            let result = self.editing.sync_folder(&folder.to_string_lossy());
            let Ok(mut syncs) = self.syncs.lock() else {
                return;
            };
            let state = syncs.entry(folder.clone()).or_default();
            match result {
                Ok(()) => {
                    state.pending = false;
                    state.last = Some(now);
                    tracing::info!(
                        folder = %folder.display(),
                        ms = started.elapsed().as_millis() as u64,
                        "capture_one.sync"
                    );
                }
                Err(psm_core::Error::EditorLaunchDeferred) => {}
                Err(error) => {
                    state.pending = false;
                    state.last = Some(now);
                    tracing::warn!(%error, folder = %folder.display(), "capture_one.sync failed");
                }
            }
        }
    }
    /// From the process list, never a script (P63 #22: the follower asks on every tick, also while
    /// the take-back has quit Capture One).
    fn running(&self) -> bool {
        self.scripting.is_running().unwrap_or(false)
    }
    fn switch(&self, folder: &Path) -> Result<()> {
        // The script compares against Capture One's live current collection, because the
        // photographer can browse away after the app last set it.
        let started = Instant::now();
        let from = self
            .current
            .lock()
            .ok()
            .and_then(|current| current.clone())
            .map(|f| folder_label(&f))
            .unwrap_or_else(|| "none".into());
        let result = self
            .editing
            .set_current_collection(&folder.to_string_lossy());
        // A10: every collection switch Capture One is asked for, with its outcome.
        tracing::info!(
            target: "psm::capture_one",
            "capture_one.switch reason=photo_arrived from={} to={} step=set_current_collection ms={} result={}",
            from,
            folder_label(folder),
            started.elapsed().as_millis(),
            result.as_ref().map_or_else(|e| format!("error: {e}"), |_| "ok".into()),
        );
        result?;
        *self
            .current
            .lock()
            .map_err(|e| psm_core::Error::InvalidData(e.to_string()))? = Some(folder.to_path_buf());
        self.health.clear("editor_not_following");
        let raws: Vec<_> = self
            .waiting
            .lock()
            .map_err(|e| psm_core::Error::InvalidData(e.to_string()))?
            .keys()
            .cloned()
            .collect();
        for raw in raws {
            self.previews.retry_now(&raw)?;
        }
        Ok(())
    }
    fn apply_style_once(&self, raw: &str) -> Result<()> {
        let _guard = self
            .style_lock
            .lock()
            .map_err(|e| psm_core::Error::InvalidData(e.to_string()))?;
        if self.style_name.is_empty()
            || !self.store.style_eligible_before_first_render(raw)?
            || self.store.style_applied_at(raw)?.is_some()
        {
            return Ok(());
        }
        // A16: every style decision is logged (`capture_one.style applied|skipped|failed`).
        match self.editing.apply_style(raw, &self.style_name) {
            Ok(StyleOutcome::Applied) => {
                self.store.mark_style_applied(raw, self.clock.now_ms())?;
                self.health.clear("style_not_found");
                tracing::info!(
                    target: "psm::capture_one",
                    "capture_one.style applied raw={} style=\"{}\"",
                    file_label(raw),
                    self.style_name
                );
            }
            Ok(StyleOutcome::NotListed) => {
                tracing::info!(
                    target: "psm::capture_one",
                    "capture_one.style skipped raw={} reason=style_not_listed",
                    file_label(raw)
                );
                self.health.set(
                    "style_not_found",
                    format!("Style '{}' not found in Capture One", self.style_name),
                )
            }
            Ok(StyleOutcome::NotRequested) => {}
            Err(error) => {
                tracing::info!(
                    target: "psm::capture_one",
                    "capture_one.style failed raw={} reason=\"{}\"",
                    file_label(raw),
                    error
                );
                return Err(error);
            }
        }
        Ok(())
    }
    pub fn prepare_render(&self, raw: &str) -> Result<()> {
        if self.style_name.is_empty()
            || !self.store.style_eligible_before_first_render(raw)?
            || self.store.style_applied_at(raw)?.is_some()
        {
            return Ok(());
        }
        if !self.editing.find_variant(raw)? {
            return Err(psm_core::Error::VariantNotFound);
        }
        self.apply_style_once(raw)
    }
    pub fn tick(&self) -> Result<()> {
        let running = self.running();
        let reopened = self
            .was_running
            .lock()
            .map(|mut was| was.replace(running) == Some(false) && running)
            .unwrap_or(false);
        if reopened {
            // Changelog 43, P60 #26: files deleted meanwhile are found before Capture One is asked
            // for anything; photos with a current preview are not asked for again.
            self.previews.on_editor_opened()?;
        }
        if !running {
            return Ok(());
        }
        self.switch_follow_up();
        self.run_syncs();
        let now = self.clock.mono();
        let due: Vec<_> = self
            .waiting
            .lock()
            .map_err(|e| psm_core::Error::InvalidData(e.to_string()))?
            .iter()
            .filter(|(_, state)| {
                now.saturating_duration_since(state.last_poll)
                    >= Duration::from_millis(VARIANT_WAIT_POLL_MS)
            })
            .map(|(raw, _)| raw.clone())
            .collect();
        for raw in due {
            let found = self.editing.find_variant(&raw);
            let mut waiting = self
                .waiting
                .lock()
                .map_err(|e| psm_core::Error::InvalidData(e.to_string()))?;
            let Some(state) = waiting.get_mut(&raw) else {
                continue;
            };
            state.last_poll = now;
            if matches!(found, Ok(true)) {
                // Changelog 45 (P65): arrival → Capture One lists the photo.
                tracing::info!(
                    raw = %raw,
                    after_ms = now.saturating_duration_since(state.since).as_millis() as u64,
                    "capture_one.variant_seen"
                );
            }
            let expired = now.saturating_duration_since(state.since)
                >= Duration::from_millis(VARIANT_WAIT_MAX_MS);
            if expired {
                waiting.remove(&raw);
            }
            drop(waiting);
            match found {
                Ok(true) => {
                    if let Err(error) = self.apply_style_once(&raw) {
                        tracing::warn!(%error, raw, "style deferred");
                    }
                    self.previews.variant_ready(&raw)?;
                    self.waiting
                        .lock()
                        .map_err(|e| psm_core::Error::InvalidData(e.to_string()))?
                        .remove(&raw);
                }
                Ok(false) => {
                    if expired {
                        self.previews.variant_ready(&raw)?;
                    }
                }
                Err(error) => tracing::warn!(%error, raw, "variant poll deferred"),
            }
        }
        Ok(())
    }
}
impl ArrivalFollow for CaptureOneFollow {
    /// A16 (owner 2026-11-18): this used to check every photo's file and retry every photo's
    /// preview inside the switch (8 minutes with 1,593 photos), holding the next switch back. It
    /// now only notes the switch; `tick` does the opened session's follow-up.
    fn collection_switched(&self) -> Result<()> {
        if let Ok(mut switched) = self.switched.lock() {
            switched.get_or_insert_with(Instant::now);
        }
        Ok(())
    }
    fn arrival_noted(&self, raw: &str) {
        self.note_arrival(raw);
    }
    fn photo_arrived(&self, _session_id: i64, raw: &str, folder: &Path) {
        self.note_arrival(raw);
        self.request_sync(folder);
        if !self.running() {
            return;
        }
        if let Err(error) = self.switch(folder) {
            tracing::warn!(%error, "Capture One collection switch deferred");
            self.health
                .set("editor_not_following", EDITOR_NOT_FOLLOWING);
        }
        self.run_syncs();
    }
}

/// Owner correction A10 (2026-11-18): the arrival thread hands a new photo's Capture One work
/// (style mark, variant wait, folder sync, collection switch) to the background loop instead of
/// doing it inline, so a busy Capture One can never hold the next photo's arrival up.
pub struct DeferredFollow {
    inner: Arc<dyn ArrivalFollow>,
    queue: Mutex<Vec<(i64, String, PathBuf)>>,
}
impl DeferredFollow {
    pub fn new(inner: Arc<dyn ArrivalFollow>) -> Self {
        Self {
            inner,
            queue: Mutex::new(Vec::new()),
        }
    }
    /// Runs the queued arrivals' Capture One work, in arrival order (the background loop).
    pub fn flush(&self) {
        let queued = match self.queue.lock() {
            Ok(mut queue) => std::mem::take(&mut *queue),
            Err(_) => return,
        };
        for (session_id, raw, folder) in queued {
            self.inner.photo_arrived(session_id, &raw, &folder);
        }
    }
}
impl ArrivalFollow for DeferredFollow {
    fn collection_switched(&self) -> Result<()> {
        self.inner.collection_switched()
    }
    fn arrival_noted(&self, raw: &str) {
        self.inner.arrival_noted(raw);
    }
    fn photo_arrived(&self, session_id: i64, raw: &str, folder: &Path) {
        // A16: the style eligibility and variant wait are recorded now, on the arrival thread,
        // before the photo's preview job can run; only Capture One's work waits for the loop.
        self.inner.arrival_noted(raw);
        if let Ok(mut queue) = self.queue.lock() {
            queue.push((session_id, raw.to_owned(), folder.to_path_buf()));
        }
    }
}

/// A file's folder and name, for log lines (never the user's home folder).
pub fn file_label(raw: &str) -> String {
    let path = Path::new(raw);
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    match path.parent() {
        Some(parent) => format!("{}/{}", folder_label(parent), name),
        None => name,
    }
}

/// A folder's last component, for log lines (never the user's home folder).
pub fn folder_label(folder: &Path) -> String {
    folder
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "?".into())
}

#[cfg(test)]
mod deferred_tests {
    use super::*;

    #[derive(Default)]
    struct Recording(Mutex<Vec<String>>);
    impl ArrivalFollow for Recording {
        fn photo_arrived(&self, session_id: i64, raw: &str, _folder: &Path) {
            self.0.lock().unwrap().push(format!("{session_id}:{raw}"));
        }
        fn arrival_noted(&self, raw: &str) {
            self.0.lock().unwrap().push(format!("noted:{raw}"));
        }
        fn collection_switched(&self) -> Result<()> {
            self.0.lock().unwrap().push("switched".into());
            Ok(())
        }
    }

    /// A16: the arrival's style eligibility and variant wait are noted on the arrival thread, at
    /// once; only the Capture One work waits for the loop.
    #[test]
    fn a16_an_arrival_is_noted_at_once_and_followed_by_the_loop() {
        let recording = Arc::new(Recording::default());
        let deferred = DeferredFollow::new(recording.clone());
        deferred.photo_arrived(7, "a.ARW", Path::new("/c/K1"));
        assert_eq!(*recording.0.lock().unwrap(), vec!["noted:a.ARW"]);
        deferred.flush();
        assert_eq!(*recording.0.lock().unwrap(), vec!["noted:a.ARW", "7:a.ARW"]);
    }

    /// A10: an arrival never waits for Capture One: its follow work is queued, then run in
    /// arrival order by the background loop.
    #[test]
    fn a10_arrival_follow_work_is_queued_and_run_in_order_by_the_loop() {
        let recording = Arc::new(Recording::default());
        let deferred = DeferredFollow::new(recording.clone());
        deferred.photo_arrived(7, "a.ARW", Path::new("/c/K1"));
        deferred.photo_arrived(7, "b.ARW", Path::new("/c/K1"));
        assert_eq!(
            *recording.0.lock().unwrap(),
            vec!["noted:a.ARW", "noted:b.ARW"],
            "no Capture One work runs on the arrival thread (A16: only the noting)"
        );
        recording.0.lock().unwrap().clear();
        deferred.flush();
        assert_eq!(*recording.0.lock().unwrap(), vec!["7:a.ARW", "7:b.ARW"]);
        deferred.flush();
        assert_eq!(
            recording.0.lock().unwrap().len(),
            2,
            "each arrival runs once"
        );
        deferred.collection_switched().unwrap();
        assert_eq!(recording.0.lock().unwrap().last().unwrap(), "switched");
    }
}
