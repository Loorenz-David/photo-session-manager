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
        }
    }
    fn running(&self) -> bool {
        self.scripting.editor_info().is_ok_and(|info| info.running)
    }
    fn switch(&self, folder: &Path) -> Result<()> {
        // The script compares against Capture One's live current collection, because the
        // photographer can browse away after the app last set it.
        self.editing
            .set_current_collection(&folder.to_string_lossy())?;
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
        match self.editing.apply_style(raw, &self.style_name)? {
            StyleOutcome::Applied => {
                self.store.mark_style_applied(raw, self.clock.now_ms())?;
                self.health.clear("style_not_found");
            }
            StyleOutcome::NotListed => self.health.set(
                "style_not_found",
                format!("Style '{}' not found in Capture One", self.style_name),
            ),
            StyleOutcome::NotRequested => {}
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
    fn collection_switched(&self) -> Result<()> {
        self.previews.check_missing_files()?;
        for state in self.store.preview_states()? {
            self.previews.retry_now(&state.raw_path)?;
        }
        Ok(())
    }
    fn photo_arrived(&self, _session_id: i64, raw: &str, folder: &Path) {
        if let Err(error) = self.store.mark_style_eligible(raw) {
            tracing::warn!(%error, raw, "style eligibility could not be recorded");
        }
        self.previews.wait_for_variant(raw);
        let now = self.clock.mono();
        if let Ok(mut waiting) = self.waiting.lock() {
            waiting.insert(
                raw.into(),
                WaitState {
                    since: now,
                    last_poll: now,
                },
            );
        }
        if !self.running() {
            return;
        }
        if let Err(error) = self.switch(folder) {
            tracing::warn!(%error, "Capture One collection switch deferred");
            self.health
                .set("editor_not_following", EDITOR_NOT_FOLLOWING);
        }
    }
}
