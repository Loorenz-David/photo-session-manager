//! Masters prepared in the background (changelog 47 (2), 10D MC-28, P69): once a session is
//! Ready (07) and Capture One is idle, its photos' masters are asked for one at a time, so a
//! later publish or export reuses them (cache key `(adjusted_gen, composition revision,
//! output_format)`, unchanged) and only renders what is missing. Low priority: nothing is asked
//! while a capture is expected, a take-back holds Capture One, a render is out or a preview is
//! waiting for one, or any export or publish runs. Masters made this way are a cache like any
//! other: deleted after publish (MC-28), cleared by the storage budget (10F §4 level 3), and made
//! again on demand.

use crate::{
    Result,
    domain::photo::{Classification, Integrity},
    mechanisms::{missing::file_missing, staleness::is_stale},
    ports::{clock::Clock, editing::EditingProvider, store::Store},
    services::{
        export::ExportService, previews::PreviewService, sessions::ready_session_ids,
        storage::PauseCheck,
    },
};
use std::{
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicI64, Ordering},
    },
};

/// After a refused request (Capture One closed or busy) the next one waits this long.
pub const PREPARE_RETRY_MS: i64 = 30_000;

/// What one tick did.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PrepareOutcome {
    /// The photo whose master was asked for.
    pub requested: Option<i64>,
    /// Why nothing was asked for (`off`, `capture`, `take-back`, `disk`, `preparing`,
    /// `export`, `publish`, `render`, `preview`, `editor`).
    pub paused: Option<String>,
}

impl PrepareOutcome {
    fn paused(reason: &str) -> Self {
        Self {
            requested: None,
            paused: Some(reason.to_owned()),
        }
    }
}

pub struct PrepareMastersService {
    store: Arc<dyn Store>,
    editing: Arc<dyn EditingProvider>,
    export: Arc<ExportService>,
    previews: Arc<PreviewService>,
    clock: Arc<dyn Clock>,
    /// The app's own conditions (a capture expected, a take-back, too little free space).
    pause: PauseCheck,
    enabled: AtomicBool,
    retry_at_ms: AtomicI64,
}

impl PrepareMastersService {
    pub fn new(
        store: Arc<dyn Store>,
        editing: Arc<dyn EditingProvider>,
        export: Arc<ExportService>,
        previews: Arc<PreviewService>,
        clock: Arc<dyn Clock>,
        pause: PauseCheck,
        enabled: bool,
    ) -> Self {
        Self {
            store,
            editing,
            export,
            previews,
            clock,
            pause,
            enabled: AtomicBool::new(enabled),
            retry_at_ms: AtomicI64::new(i64::MIN),
        }
    }

    /// `export.prepare_masters` (Settings → Storage), applied from the next tick.
    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::Relaxed);
    }
    pub fn enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    /// One step of the background job: asks for at most one master, and only while nothing
    /// else uses Capture One.
    pub fn tick(&self) -> Result<PrepareOutcome> {
        if !self.enabled() {
            return Ok(PrepareOutcome::paused("off"));
        }
        if let Some(reason) = self.busy()? {
            return Ok(PrepareOutcome::paused(&reason));
        }
        let now = self.clock.now_ms();
        if now < self.retry_at_ms.load(Ordering::Relaxed) {
            return Ok(PrepareOutcome::paused("editor"));
        }
        let Some((photo_id, raw, generation)) = self.next()? else {
            return Ok(PrepareOutcome::default());
        };
        match self.export.prepare_master(&raw, photo_id, generation) {
            Ok(true) => {
                tracing::info!(
                    raw,
                    photo_id,
                    adjusted_gen = generation,
                    "master asked for in advance"
                );
                Ok(PrepareOutcome {
                    requested: Some(photo_id),
                    paused: None,
                })
            }
            Ok(false) => Ok(PrepareOutcome::paused("preparing")),
            Err(error) => {
                tracing::info!(%error, raw, "master preparation deferred");
                self.retry_at_ms
                    .store(now + PREPARE_RETRY_MS, Ordering::Relaxed);
                Ok(PrepareOutcome::paused("editor"))
            }
        }
    }

    /// Why Capture One is not idle for this job, if it is not.
    fn busy(&self) -> Result<Option<String>> {
        if let Some(reason) = (self.pause)() {
            return Ok(Some(reason));
        }
        let reason = if self.export.preparing() {
            "preparing"
        } else if self.export.running() {
            "export"
        } else if !self.store.active_publish_items()?.is_empty() {
            "publish"
        } else if self.editing.renders_in_flight() > 0 {
            "render"
        } else if self.preview_waiting()? {
            "preview"
        } else {
            return Ok(None);
        };
        Ok(Some(reason.to_owned()))
    }

    /// A preview the app will still ask Capture One for: its RAW is adjusted past its last
    /// render, the file is there and its last render did not fail for good.
    fn preview_waiting(&self) -> Result<bool> {
        for state in self.store.preview_states()? {
            if !is_stale(state.adjusted_gen, state.rendered_gen)
                || self.previews.failed_generation_for(&state.raw_path) == Some(state.adjusted_gen)
                || file_missing(Path::new(&state.raw_path)).unwrap_or(false)
            {
                continue;
            }
            return Ok(true);
        }
        Ok(false)
    }

    /// The first photo of the newest Ready session whose master for its current generation is
    /// not in the cache: main and unreviewed photos in position order, then condition photos.
    fn next(&self) -> Result<Option<(i64, String, i64)>> {
        for session_id in ready_session_ids(self.store.as_ref())? {
            let mut photos: Vec<_> = self
                .store
                .photos_in_session(session_id)?
                .into_iter()
                .filter(|p| {
                    p.classification != Classification::Deleted && p.integrity == Integrity::Ok
                })
                .collect();
            photos.sort_by_key(|p| {
                (
                    p.classification == Classification::Condition,
                    p.position.unwrap_or(i64::MAX),
                    p.id,
                )
            });
            for photo in photos {
                let generation = self.store.adjusted_gen(&photo.raw_path)?;
                if generation < 1
                    || self.export.master_on_disk(&photo.raw_path, generation)
                    || self.export.prepare_failed(&photo.raw_path, generation)
                    || self.editing.render_in_flight(&photo.raw_path)
                {
                    continue;
                }
                return Ok(Some((photo.id, photo.raw_path, generation)));
            }
        }
        Ok(None)
    }
}
