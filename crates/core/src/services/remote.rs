//! Remote items (book 10H §3–§6, §8; master plan archive §4 Service, §5 MC-H1, MC-H3, MC-H5):
//! the record of an item's remote pictures, its first-open read, Refresh from the shop, the row
//! picture rule, and the background step that downloads queued pictures (at most
//! `PARALLEL_DOWNLOADS` at once), refreshes stale picture addresses and runs queued reads only
//! while the call budget allows. Core vocabulary only (H7): *remote picture*, *the shop*, *call
//! budget*.

use crate::{
    Error, Result,
    domain::{
        publish::MediaImage,
        remote::{CallBudget, PictureTarget, RemoteState},
        storage::ArchivedImage,
    },
    events::AppEvent,
    mechanisms::remote::{
        COVER_EDGE, PARALLEL_DOWNLOADS, REFRESH_COOLDOWN_MS, RemoteCover, SEARCH_EDGE,
        URL_REFRESH_CHUNK, cover_name, evict, must_wait, remote_state, search_name,
    },
    ports::{clock::Clock, commerce::CommerceAdapter, events::EventSink, store::Store},
    services::{
        health::HealthService,
        retention::{INTEGRATION, ROLE_CONDITION, ROLE_GALLERY, archived_file_name},
    },
};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex, MutexGuard,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

/// MC-H1.5: the Health code of pictures the shop no longer has.
pub const REMOTE_PICTURE_GONE: &str = "remote_picture_gone";

/// Who asked for covers (MC-H1.3): a new request replaces only its own source's downloads that
/// have not started.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoverSource {
    /// The page of the sessions list on screen.
    Page,
    /// The item search.
    Search,
}

/// MC-H2.4: an item's search picture: its session's row picture, its session, and whether its
/// cover is pending.
pub type ItemPicture = (Option<PathBuf>, Option<i64>, bool);

/// One queued picture download.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Download {
    kind: DownloadKind,
    url: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DownloadKind {
    /// A row's cover (MC-H1.2).
    Cover {
        session_id: i64,
        item_id: i64,
        media_id: String,
    },
    /// A shop search result's picture (MC-H2.5).
    Search { external_id: String },
}

impl Download {
    /// The file the download writes.
    fn path(&self, previews_root: &Path) -> PathBuf {
        match &self.kind {
            DownloadKind::Cover {
                item_id, media_id, ..
            } => cover_path(previews_root, *item_id, media_id),
            DownloadKind::Search { .. } => search_dir(previews_root).join(search_name(&self.url)),
        }
    }

    /// The item and media id of a cover download (the only kind whose address is refreshed).
    fn cover_media(&self) -> Option<(i64, &str)> {
        match &self.kind {
            DownloadKind::Cover {
                item_id, media_id, ..
            } => Some((*item_id, media_id.as_str())),
            DownloadKind::Search { .. } => None,
        }
    }
}

/// MC-H2.5: `<previews>/remote/search/` (one flat folder; item folders are numeric).
fn search_dir(previews_root: &Path) -> PathBuf {
    previews_root.join("remote").join("search")
}

/// The work `run_pending` takes (MC-H1.3, MC-H1.4, MC-H3.2).
#[derive(Default)]
struct Queue {
    /// Sessions whose first-open read is queued (MC-H3.2, MC-H5.3).
    reads: Vec<i64>,
    /// Not-started cover downloads, per source.
    page: Vec<Download>,
    search: Vec<Download>,
    /// Not-started shop search pictures (MC-H2.5).
    search_pictures: Vec<Download>,
    /// Covers whose address answered "gone", waiting for the address refresh the budget held
    /// back (MC-H5.3).
    stale: Vec<Download>,
}

/// The row picture inputs of every session, read once (MC-H1.1; one read for a whole list).
pub struct RowPictures {
    covers: HashMap<i64, RemoteCover>,
    previews_root: PathBuf,
    /// Without a commerce integration every item is local (H6).
    remote: bool,
}

impl RowPictures {
    /// MC-H1.1: the row picture of `session_id` from its main photo 1's stored preview paths,
    /// and whether its cover is pending. Never a path whose file is missing (M-H4).
    pub fn row_picture(
        &self,
        session_id: i64,
        adjusted: Option<&str>,
        original: Option<&str>,
    ) -> (Option<PathBuf>, bool) {
        if let Some(path) = local_picture(adjusted, original) {
            return (Some(path), false);
        }
        match self.cover_download(session_id) {
            CoverFile::Present(path) => (Some(path), false),
            CoverFile::Missing(_) => (None, true),
            CoverFile::None => (None, false),
        }
    }

    /// MC-H1.1(ii): the session's cover file when its item is remote and its cover picture is
    /// not gone.
    fn cover_download(&self, session_id: i64) -> CoverFile {
        if !self.remote {
            return CoverFile::None;
        }
        let Some(row) = self.covers.get(&session_id) else {
            return CoverFile::None;
        };
        if !matches!(remote_state(row.input), RemoteState::Remote(_)) {
            return CoverFile::None;
        }
        let Some(cover) = row.cover.as_ref().filter(|c| !c.url.is_empty()) else {
            return CoverFile::None;
        };
        let path = cover_path(&self.previews_root, row.item_id, &cover.media_id);
        if path.is_file() {
            CoverFile::Present(path)
        } else {
            CoverFile::Missing(Download {
                kind: DownloadKind::Cover {
                    session_id,
                    item_id: row.item_id,
                    media_id: cover.media_id.clone(),
                },
                url: cover.url.clone(),
            })
        }
    }
}

enum CoverFile {
    Present(PathBuf),
    Missing(Download),
    None,
}

/// MC-H1.1(i) and (iii), the rule without remote items: the adjusted preview if its file exists,
/// else the original preview if its file exists, else none.
pub fn local_picture(adjusted: Option<&str>, original: Option<&str>) -> Option<PathBuf> {
    [adjusted, original]
        .into_iter()
        .flatten()
        .map(PathBuf::from)
        .find(|path| path.is_file())
}

/// MC-H1.2: `<previews>/remote/<item_id>/<cover_name(media_id)>`.
fn cover_path(previews_root: &Path, item_id: i64, media_id: &str) -> PathBuf {
    previews_root
        .join("remote")
        .join(item_id.to_string())
        .join(cover_name(media_id))
}

/// What one `run_pending` did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pending {
    /// Nothing was queued.
    Idle,
    /// A step that calls the shop was held back by the low call budget (MC-H5.3) and nothing
    /// else was done.
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
    health: Arc<HealthService>,
    /// MC-H2.6: the most bytes the search-picture folder keeps.
    search_cache_limit: u64,
    /// MC-H2.6: one eviction of the search-picture folder at a time.
    evict_lock: Mutex<()>,
    /// MC-H3.2: one process-wide lock around reading and replacing a record, so two callers make
    /// one read.
    record_lock: Mutex<()>,
    /// Queued reads and downloads (never held while the shop is asked anything).
    queue: Mutex<Queue>,
    /// The last `run_pending` waited for the budget (the wait is logged once, when it starts).
    waiting: AtomicBool,
    /// MC-H1.4: when each media id's address was last asked for (monotonic, in memory).
    refreshed: Mutex<HashMap<String, Instant>>,
    /// MC-H1.5: items with gone pictures: item id → (SKU, the gone media ids).
    gone: Mutex<BTreeMap<i64, (String, BTreeSet<String>)>>,
}

impl RemoteItems {
    pub fn new(
        store: Arc<dyn Store>,
        commerce: Option<Arc<dyn CommerceAdapter>>,
        previews_root: PathBuf,
        clock: Arc<dyn Clock>,
        events: Arc<dyn EventSink>,
        health: Arc<HealthService>,
        search_cache_limit: u64,
    ) -> Self {
        Self {
            store,
            commerce,
            previews_root,
            clock,
            events,
            health,
            search_cache_limit,
            evict_lock: Mutex::new(()),
            record_lock: Mutex::new(()),
            queue: Mutex::new(Queue::default()),
            waiting: AtomicBool::new(false),
            refreshed: Mutex::new(HashMap::new()),
            gone: Mutex::new(BTreeMap::new()),
        }
    }

    /// MC-H1.1: the row picture inputs of every session, in one read.
    pub fn row_pictures(&self) -> Result<RowPictures> {
        let remote = self.commerce.is_some();
        let covers = if remote {
            self.store
                .remote_covers()?
                .into_iter()
                .map(|row| (row.session_id, row))
                .collect()
        } else {
            HashMap::new()
        };
        Ok(RowPictures {
            covers,
            previews_root: self.previews_root.clone(),
            remote,
        })
    }

    /// MC-H1.3: replaces `source`'s not-started cover downloads with those of `session_ids`
    /// whose cover is pending. Never downloads and never waits for a running download.
    pub fn request_covers(&self, source: CoverSource, session_ids: &[i64]) -> Result<()> {
        let downloads: Vec<Download> = if self.commerce.is_some() && !session_ids.is_empty() {
            let pictures = self.row_pictures()?;
            session_ids
                .iter()
                .filter_map(|id| match pictures.cover_download(*id) {
                    CoverFile::Missing(download) => Some(download),
                    _ => None,
                })
                .collect()
        } else {
            Vec::new()
        };
        let mut queue = self.lock_queue()?;
        match source {
            CoverSource::Page => queue.page = downloads,
            CoverSource::Search => queue.search = downloads,
        }
        Ok(())
    }

    /// MC-H2.4: the row picture of each item's session (none without a session), its session and
    /// whether its cover is pending; one read of the remote inputs for all of them.
    pub fn item_pictures(&self, item_ids: &[i64]) -> Result<Vec<ItemPicture>> {
        if item_ids.is_empty() {
            return Ok(Vec::new());
        }
        let pictures = self.row_pictures()?;
        let mut out = Vec::with_capacity(item_ids.len());
        for item_id in item_ids {
            let Some(session_id) = self.store.session_for_item(*item_id)? else {
                out.push((None, None, false));
                continue;
            };
            let main = self
                .store
                .photos_in_session(session_id)?
                .into_iter()
                .find(|photo| {
                    photo.classification == crate::domain::photo::Classification::Main
                        && photo.position == Some(1)
                });
            let (adjusted, original) = main
                .map(|photo| (photo.adjusted_preview_path, photo.original_preview_path))
                .unwrap_or((None, None));
            let (path, pending) =
                pictures.row_picture(session_id, adjusted.as_deref(), original.as_deref());
            out.push((path, Some(session_id), pending));
        }
        Ok(out)
    }

    /// MC-H2.4: `item_pictures` of one item.
    pub fn item_picture(&self, item_id: i64) -> Result<ItemPicture> {
        Ok(self
            .item_pictures(&[item_id])?
            .pop()
            .unwrap_or((None, None, false)))
    }

    /// MC-H2.5: the cached picture of a shop search result when it is on this Mac; otherwise its
    /// download (`SEARCH_EDGE`) is queued for the background and none is answered. Never
    /// downloads here.
    pub fn search_picture(&self, external_id: &str, url: &str) -> Result<Option<PathBuf>> {
        if self.commerce.is_none() || url.is_empty() {
            return Ok(None);
        }
        let download = Download {
            kind: DownloadKind::Search {
                external_id: external_id.to_owned(),
            },
            url: url.to_owned(),
        };
        let path = download.path(&self.previews_root);
        if path.is_file() {
            return Ok(Some(path));
        }
        let mut queue = self.lock_queue()?;
        if !queue.search_pictures.contains(&download) {
            queue.search_pictures.push(download);
        }
        Ok(None)
    }

    /// "Delete session" of an item whose row is gone (M-H5, 10H §9): its cache folders go (cache
    /// files, not photos) and it leaves the gone map (MC-H1.5). No shop call.
    pub fn forget_item(&self, item_id: i64) -> Result<()> {
        let item = item_id.to_string();
        for dir in [
            self.previews_root.join("remote").join(&item),
            self.previews_root.join("archived").join(&item),
        ] {
            if dir.is_dir()
                && let Err(error) = std::fs::remove_dir_all(&dir)
            {
                tracing::warn!(%error, item = item_id, "remote cache folder not deleted");
            }
        }
        self.leave_gone(item_id)
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
        if !queue.reads.contains(&session_id) {
            queue.reads.push(session_id);
        }
        Ok(())
    }

    /// The background step (master plan archive §4 Background): the queued picture downloads as
    /// one batch, at most `PARALLEL_DOWNLOADS` at once (MC-H1.3); then the batch's stale
    /// addresses, refreshed when the call budget allows and retried once (MC-H1.4) and Health
    /// (MC-H1.5); then the queued first-open reads, each only while the budget allows (MC-H3.2,
    /// MC-H5.3; what is not done stays queued). A failed read is logged and not retried here.
    pub fn run_pending(&self) -> Result<Pending> {
        let (batch, mut stale, mut queued) = {
            let mut queue = self.lock_queue()?;
            let mut batch = std::mem::take(&mut queue.page);
            for download in std::mem::take(&mut queue.search)
                .into_iter()
                .chain(std::mem::take(&mut queue.search_pictures))
            {
                if !batch.contains(&download) {
                    batch.push(download);
                }
            }
            (
                batch,
                std::mem::take(&mut queue.stale),
                std::mem::take(&mut queue.reads),
            )
        };
        if batch.is_empty() && stale.is_empty() && queued.is_empty() {
            return Ok(Pending::Idle);
        }
        let Some(commerce) = &self.commerce else {
            return Ok(Pending::Idle);
        };
        let commerce = commerce.as_ref();
        let mut worked = false;
        let mut held = false;
        if !batch.is_empty() {
            worked = true;
            stale.extend(self.download_all(commerce, batch));
        }
        match self.refresh_stale(commerce, stale)? {
            Refreshed::Nothing => {}
            Refreshed::Done => worked = true,
            Refreshed::Held(stale) => {
                held = true;
                self.lock_queue()?.stale.extend(stale);
            }
        }
        while !queued.is_empty() {
            let budget = commerce.budget();
            if must_wait(budget) {
                self.note_wait(budget);
                held = true;
                let mut queue = self.lock_queue()?;
                for session_id in queued {
                    if !queue.reads.contains(&session_id) {
                        queue.reads.push(session_id);
                    }
                }
                break;
            }
            self.waiting.store(false, Ordering::Relaxed);
            let session_id = queued.remove(0);
            worked = true;
            if let Err(error) = self.ensure_record(session_id) {
                tracing::warn!(%error, session = session_id, "remote.record not read");
            }
        }
        Ok(if held && !worked {
            Pending::Waiting
        } else {
            Pending::Worked
        })
    }

    /// Runs the downloads at most `PARALLEL_DOWNLOADS` at a time; answers those whose address
    /// the shop said is gone (MC-H1.4). Any other failure is logged and dropped for this batch.
    fn download_all(
        &self,
        commerce: &dyn CommerceAdapter,
        downloads: Vec<Download>,
    ) -> Vec<Download> {
        let next = AtomicUsize::new(0);
        let stale = Mutex::new(Vec::new());
        std::thread::scope(|scope| {
            for _ in 0..PARALLEL_DOWNLOADS.min(downloads.len()) {
                scope.spawn(|| {
                    loop {
                        let index = next.fetch_add(1, Ordering::SeqCst);
                        let Some(download) = downloads.get(index) else {
                            break;
                        };
                        match self.download(commerce, download) {
                            Ok(()) => {}
                            // MC-H1.4: a cover's address is refreshed; a search picture is
                            // dropped (the next search carries a fresh address, MC-H2.5).
                            Err(Error::ImageGone) if download.cover_media().is_some() => {
                                if let Ok(mut stale) = stale.lock() {
                                    stale.push(download.clone());
                                }
                            }
                            Err(error) => self.log_download(download, &format!("failed ({error})")),
                        }
                    }
                });
            }
        });
        stale.into_inner().unwrap_or_default()
    }

    /// One download: written to `<name>.part`, renamed, then (a cover) the item's other covers
    /// deleted (MC-H1.2), then `PictureReady`.
    fn download(&self, commerce: &dyn CommerceAdapter, download: &Download) -> Result<()> {
        let (edge, target) = match &download.kind {
            DownloadKind::Cover { session_id, .. } => (
                COVER_EDGE,
                PictureTarget::Cover {
                    session_id: *session_id,
                },
            ),
            DownloadKind::Search { external_id } => (
                SEARCH_EDGE,
                PictureTarget::Search {
                    external_id: external_id.clone(),
                },
            ),
        };
        let bytes = commerce.fetch_image(&download.url, edge)?;
        let path = download.path(&self.previews_root);
        let (Some(dir), Some(name)) = (path.parent(), path.file_name()) else {
            return Err(Error::InvalidData("picture cache path".into()));
        };
        let name = name.to_string_lossy().into_owned();
        std::fs::create_dir_all(dir)?;
        let partial = dir.join(format!("{name}.part"));
        std::fs::write(&partial, bytes)?;
        std::fs::rename(&partial, &path)?;
        match download.kind {
            DownloadKind::Cover { .. } => remove_files(dir, |other| {
                other != name && other.starts_with("cover-") && other.ends_with(".jpg")
            }),
            DownloadKind::Search { .. } => self.evict_search(&path),
        }
        self.log_download(download, "written");
        self.events.emit(AppEvent::PictureReady {
            target,
            path: path.to_string_lossy().into_owned(),
        });
        Ok(())
    }

    fn log_download(&self, download: &Download, outcome: &str) {
        match &download.kind {
            DownloadKind::Cover { item_id, .. } => {
                tracing::info!("remote.cover item={item_id} outcome={outcome}");
            }
            DownloadKind::Search { .. } => {
                tracing::info!("remote.search_picture outcome={outcome}");
            }
        }
    }

    /// MC-H2.6: after a search picture is written (its modified time set to the app's clock, so
    /// the folder's order follows it), the oldest files go until the folder fits the limit.
    fn evict_search(&self, written: &Path) {
        let _turn = self.evict_lock.lock().unwrap_or_else(|e| e.into_inner());
        let modified =
            std::time::UNIX_EPOCH + Duration::from_millis(self.clock.now_ms().max(0) as u64);
        if let Err(error) = std::fs::File::options()
            .write(true)
            .open(written)
            .and_then(|file| file.set_modified(modified))
        {
            tracing::warn!(%error, "search picture time not set");
        }
        let Some(dir) = written.parent() else {
            return;
        };
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        let files: Vec<(String, u64, i64)> = entries
            .flatten()
            .filter_map(|entry| {
                let name = entry.file_name().to_string_lossy().into_owned();
                let meta = entry.metadata().ok()?;
                if !meta.is_file() || !name.ends_with(".jpg") {
                    return None;
                }
                let at = meta
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map_or(0, |d| d.as_millis() as i64);
                Some((name, meta.len(), at))
            })
            .collect();
        for name in evict(&files, self.search_cache_limit) {
            if let Err(error) = std::fs::remove_file(dir.join(&name)) {
                tracing::warn!(%error, file = %name, "search picture not evicted");
            }
        }
    }

    /// MC-H1.4: the stale pictures' media ids, without those asked for less than
    /// `REFRESH_COOLDOWN_MS` ago, go to `media_urls` in chunks when the budget allows; a found
    /// address is stored and its download retried once, a missing one stores the picture as gone.
    fn refresh_stale(
        &self,
        commerce: &dyn CommerceAdapter,
        stale: Vec<Download>,
    ) -> Result<Refreshed> {
        if stale.is_empty() {
            return Ok(Refreshed::Nothing);
        }
        let now = self.clock.mono();
        let cooldown = Duration::from_millis(REFRESH_COOLDOWN_MS);
        let stale: Vec<Download> = {
            let refreshed = self.lock_refreshed()?;
            stale
                .into_iter()
                .filter(|download| {
                    let Some((item_id, media_id)) = download.cover_media() else {
                        return false;
                    };
                    let recent = refreshed
                        .get(media_id)
                        .is_some_and(|at| now.saturating_duration_since(*at) < cooldown);
                    if recent {
                        tracing::info!(
                            "remote.cover item={item_id} outcome=gone_recently_refreshed"
                        );
                    }
                    !recent
                })
                .collect()
        };
        if stale.is_empty() {
            return Ok(Refreshed::Nothing);
        }
        let budget = commerce.budget();
        if must_wait(budget) {
            self.note_wait(budget);
            return Ok(Refreshed::Held(stale));
        }
        self.waiting.store(false, Ordering::Relaxed);
        let mut ids: Vec<String> = Vec::new();
        for (_, media_id) in stale.iter().filter_map(Download::cover_media) {
            if !ids.iter().any(|id| id == media_id) {
                ids.push(media_id.to_owned());
            }
        }
        let mut found = 0;
        let mut gone = 0;
        let mut retries = Vec::new();
        let mut gone_items = false;
        for chunk in ids.chunks(URL_REFRESH_CHUNK) {
            {
                let mut refreshed = self.lock_refreshed()?;
                for id in chunk {
                    refreshed.insert(id.clone(), now);
                }
            }
            let answers = match commerce.media_urls(chunk) {
                Ok(answers) => answers,
                Err(error) => {
                    tracing::warn!(%error, "remote.urls_refreshed not read");
                    continue;
                }
            };
            for (media_id, answer) in chunk.iter().zip(answers) {
                let url = answer.as_ref().and_then(|image| image.url.clone());
                let bytes = answer.as_ref().and_then(|image| image.bytes);
                for download in stale
                    .iter()
                    .filter(|d| d.cover_media().is_some_and(|(_, id)| id == media_id))
                {
                    let Some((item_id, _)) = download.cover_media() else {
                        continue;
                    };
                    self.store
                        .update_remote_urls(item_id, &[(media_id.clone(), url.clone(), bytes)])?;
                    match &url {
                        Some(url) => {
                            found += 1;
                            retries.push(Download {
                                kind: download.kind.clone(),
                                url: url.clone(),
                            });
                        }
                        None => {
                            gone += 1;
                            gone_items = true;
                            self.mark_gone(item_id, media_id)?;
                        }
                    }
                }
            }
        }
        tracing::info!(
            "remote.urls_refreshed asked={} found={found} gone={gone}",
            ids.len()
        );
        if gone_items {
            self.report_gone()?;
        }
        // Retried once: a second "gone" waits for the next request (and the cool-down).
        for download in self.download_all(commerce, retries) {
            self.log_download(&download, "gone_after_refresh");
        }
        Ok(Refreshed::Done)
    }

    fn mark_gone(&self, item_id: i64, media_id: &str) -> Result<()> {
        let sku = self.store.item(item_id)?.sku;
        self.lock_gone()?
            .entry(item_id)
            .or_insert_with(|| (sku, BTreeSet::new()))
            .1
            .insert(media_id.to_owned());
        Ok(())
    }

    /// MC-H1.5: the item leaves the gone map (record replaced, brought back, session deleted).
    fn leave_gone(&self, item_id: i64) -> Result<()> {
        let removed = self.lock_gone()?.remove(&item_id).is_some();
        if removed {
            self.report_gone()?;
        }
        Ok(())
    }

    /// MC-H1.5: Health `remote_picture_gone` with the count and SKUs, or cleared when none.
    fn report_gone(&self) -> Result<()> {
        let detail = {
            let gone = self.lock_gone()?;
            let count: usize = gone.values().map(|(_, ids)| ids.len()).sum();
            let skus: Vec<&str> = gone.values().map(|(sku, _)| sku.as_str()).collect();
            (count > 0).then(|| {
                if count == 1 {
                    format!("1 picture is no longer in the shop: {}", skus.join(", "))
                } else {
                    format!(
                        "{count} pictures are no longer in the shop: {}",
                        skus.join(", ")
                    )
                }
            })
        };
        match detail {
            Some(detail) => self.health.set(REMOTE_PICTURE_GONE, detail),
            None => self.health.clear(REMOTE_PICTURE_GONE),
        }
        self.events.emit(AppEvent::HealthChanged);
        Ok(())
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

    fn lock_queue(&self) -> Result<MutexGuard<'_, Queue>> {
        self.queue
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))
    }

    fn lock_refreshed(&self) -> Result<MutexGuard<'_, HashMap<String, Instant>>> {
        self.refreshed
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))
    }

    #[allow(clippy::type_complexity)]
    fn lock_gone(&self) -> Result<MutexGuard<'_, BTreeMap<i64, (String, BTreeSet<String>)>>> {
        self.gone
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
        self.leave_gone(item_id)?;
        let state = remote_state(self.store.remote_input(session_id)?);
        tracing::info!(
            "remote.record item={item_id} kind={} pictures={}",
            state.as_str(),
            images.len()
        );
        self.events.emit(AppEvent::SessionChanged { session_id });
        Ok(state)
    }

    /// MC-H3.3: deletes the item's cover files other than the new cover's and its 1600 px cache
    /// files not named by a current picture.
    fn prune(&self, item_id: i64) -> Result<()> {
        let images = self.store.archived_images(item_id)?;
        let keep: HashSet<String> = images.iter().map(archived_file_name).collect();
        let cover = images
            .iter()
            .filter(|image| image.role == ROLE_GALLERY)
            .min_by_key(|image| image.position)
            .map(|image| cover_name(&image.media_id));
        let item = item_id.to_string();
        remove_files(&self.previews_root.join("remote").join(&item), |name| {
            name.starts_with("cover-") && cover.as_deref() != Some(name)
        });
        remove_files(&self.previews_root.join("archived").join(&item), |name| {
            !keep.contains(name)
        });
        Ok(())
    }
}

/// What the address refresh of one batch did.
enum Refreshed {
    /// No stale picture to refresh.
    Nothing,
    /// The shop was asked.
    Done,
    /// The call budget held the refresh back; these wait for the next step.
    Held(Vec<Download>),
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
