//! Remote items (book 10H §3, §6, §8; master plan archive §4 Service, §5 MC-H3, MC-H5): the
//! record of an item's remote pictures, its first-open read, Refresh from the shop, and the
//! background step that runs queued reads only while the call budget allows. Core vocabulary
//! only (H7): *remote picture*, *the shop*, *call budget*.

use crate::{
    Error, Result,
    domain::{
        publish::MediaImage,
        remote::{CallBudget, RemoteState},
        storage::ArchivedImage,
    },
    events::AppEvent,
    mechanisms::remote::{must_wait, remote_state},
    ports::{clock::Clock, commerce::CommerceAdapter, events::EventSink, store::Store},
    services::retention::{INTEGRATION, ROLE_CONDITION, ROLE_GALLERY, archived_file_name},
};
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex, MutexGuard,
        atomic::{AtomicBool, Ordering},
    },
};

/// What one `run_pending` did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pending {
    /// Nothing was queued.
    Idle,
    /// Work is queued but the call budget is low (MC-H5.3); nothing was read.
    Waiting,
    /// Queued work ran.
    Worked,
}

pub struct RemoteItems {
    store: Arc<dyn Store>,
    commerce: Option<Arc<dyn CommerceAdapter>>,
    /// `<data>/previews` (the `remote/` and `archived/` caches live under it).
    previews_root: PathBuf,
    clock: Arc<dyn Clock>,
    events: Arc<dyn EventSink>,
    /// MC-H3.2: one process-wide lock around reading and replacing a record, so two callers make
    /// one read.
    record_lock: Mutex<()>,
    /// Sessions whose first-open read is queued for the background (MC-H3.2, MC-H5.3).
    queue: Mutex<Vec<i64>>,
    /// The last `run_pending` waited for the budget (the wait is logged once, when it starts).
    waiting: AtomicBool,
}

impl RemoteItems {
    pub fn new(
        store: Arc<dyn Store>,
        commerce: Option<Arc<dyn CommerceAdapter>>,
        previews_root: PathBuf,
        clock: Arc<dyn Clock>,
        events: Arc<dyn EventSink>,
    ) -> Self {
        Self {
            store,
            commerce,
            previews_root,
            clock,
            events,
            record_lock: Mutex::new(()),
            queue: Mutex::new(Vec::new()),
            waiting: AtomicBool::new(false),
        }
    }

    /// MC-H3.1: where the session's item's pictures are; `Local` without an integration (H6).
    pub fn state(&self, session_id: i64) -> Result<RemoteState> {
        if self.commerce.is_none() {
            return Ok(RemoteState::Local);
        }
        Ok(remote_state(self.store.remote_input(session_id)?))
    }

    /// MC-H3.2, the first-open read: for an `Unknown` item, under the record lock, reads the
    /// item's pictures from the shop once and records them. A picture without an address (still
    /// processing) or a failed read writes nothing. Returns the state afterwards.
    pub fn ensure_record(&self, session_id: i64) -> Result<RemoteState> {
        let Some(commerce) = &self.commerce else {
            return Ok(RemoteState::Local);
        };
        let _lock = self.lock_record()?;
        let state = remote_state(self.store.remote_input(session_id)?);
        if state != RemoteState::Unknown {
            return Ok(state);
        }
        self.read_record(commerce.as_ref(), session_id)
    }

    /// MC-H3.3, Refresh from the shop (a press: never waits for the budget, MC-H5.4): for a
    /// remote item only, the same read replaces the record (`cleared_at` untouched), then the
    /// item's cache files that no longer match it are deleted.
    pub fn refresh(&self, session_id: i64) -> Result<RemoteState> {
        let Some(commerce) = &self.commerce else {
            return Err(Error::NotRemote);
        };
        let _lock = self.lock_record()?;
        if !matches!(
            remote_state(self.store.remote_input(session_id)?),
            RemoteState::Remote(_)
        ) {
            return Err(Error::NotRemote);
        }
        let state = self.read_record(commerce.as_ref(), session_id)?;
        if let Some(item_id) = self.store.session(session_id)?.session.item_id {
            self.prune(item_id)?;
        }
        Ok(state)
    }

    /// Queues the session's first-open read for the background (MC-H3.2).
    pub fn queue_record(&self, session_id: i64) -> Result<()> {
        let mut queue = self.lock_queue()?;
        if !queue.contains(&session_id) {
            queue.push(session_id);
        }
        Ok(())
    }

    /// The background step: the queued first-open reads, each only while the call budget allows
    /// (MC-H5.3; what is not read stays queued and the step answers `Waiting`). A failed read is
    /// logged and not retried by this step.
    pub fn run_pending(&self) -> Result<Pending> {
        let mut queued = std::mem::take(&mut *self.lock_queue()?);
        if queued.is_empty() {
            return Ok(Pending::Idle);
        }
        let Some(commerce) = &self.commerce else {
            return Ok(Pending::Idle);
        };
        let mut worked = false;
        while !queued.is_empty() {
            let budget = commerce.budget();
            if must_wait(budget) {
                self.note_wait(budget);
                let mut queue = self.lock_queue()?;
                for session_id in queued {
                    if !queue.contains(&session_id) {
                        queue.push(session_id);
                    }
                }
                return Ok(if worked {
                    Pending::Worked
                } else {
                    Pending::Waiting
                });
            }
            self.waiting.store(false, Ordering::Relaxed);
            let session_id = queued.remove(0);
            worked = true;
            if let Err(error) = self.ensure_record(session_id) {
                tracing::warn!(%error, session = session_id, "remote.record not read");
            }
        }
        Ok(Pending::Worked)
    }

    fn note_wait(&self, budget: Option<CallBudget>) {
        if !self.waiting.swap(true, Ordering::Relaxed)
            && let Some(budget) = budget
        {
            tracing::info!(
                "remote.budget_wait available={} maximum={}",
                budget.available,
                budget.maximum
            );
        }
    }

    fn lock_record(&self) -> Result<MutexGuard<'_, ()>> {
        self.record_lock
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))
    }

    fn lock_queue(&self) -> Result<MutexGuard<'_, Vec<i64>>> {
        self.queue
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))
    }

    /// Reads the item's pictures from the shop and, when every one has an address, replaces the
    /// record in one transaction and emits `SessionChanged`. Called under the record lock.
    fn read_record(&self, commerce: &dyn CommerceAdapter, session_id: i64) -> Result<RemoteState> {
        let item_id = self
            .store
            .session(session_id)?
            .session
            .item_id
            .ok_or_else(|| Error::NotFound(format!("item of session {session_id}")))?;
        let external = self
            .store
            .item_link(item_id, INTEGRATION)?
            .ok_or_else(|| Error::NotFound(format!("shop link of item {item_id}")))?
            .external_id;
        let readback = commerce.media_readback(&external)?;
        let mut images = Vec::new();
        for (held, role) in [
            (&readback.gallery, ROLE_GALLERY),
            (&readback.condition, ROLE_CONDITION),
        ] {
            for (index, MediaImage { id, url, bytes }) in held.iter().enumerate() {
                let Some(url) = url else {
                    return Err(Error::Provider(
                        "a picture of this item is still processing in the shop".into(),
                    ));
                };
                images.push(ArchivedImage {
                    role: role.into(),
                    position: index as i64 + 1,
                    media_id: id.clone(),
                    url: url.clone(),
                    bytes: *bytes,
                });
            }
        }
        self.store
            .set_remote_record(item_id, &images, self.clock.now_ms())?;
        let state = remote_state(self.store.remote_input(session_id)?);
        tracing::info!(
            "remote.record item={item_id} kind={} pictures={}",
            state.as_str(),
            images.len()
        );
        self.events.emit(AppEvent::SessionChanged { session_id });
        Ok(state)
    }

    /// MC-H3.3: deletes the item's cover files (H1 has no cover name yet: every `cover-*`) and
    /// its 1600 px cache files not named by a current picture.
    fn prune(&self, item_id: i64) -> Result<()> {
        let keep: HashSet<String> = self
            .store
            .archived_images(item_id)?
            .iter()
            .map(archived_file_name)
            .collect();
        let item = item_id.to_string();
        remove_files(&self.previews_root.join("remote").join(&item), |name| {
            name.starts_with("cover-")
        });
        remove_files(&self.previews_root.join("archived").join(&item), |name| {
            !keep.contains(name)
        });
        Ok(())
    }
}

/// Deletes the files of `dir` whose name `stale` accepts; a missing folder has none. A file that
/// cannot be deleted is logged and left.
fn remove_files(dir: &Path, stale: impl Fn(&str) -> bool) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        if path.is_file()
            && stale(&name)
            && let Err(error) = std::fs::remove_file(&path)
        {
            tracing::warn!(%error, file = %name, "remote cache file not deleted");
        }
    }
}
