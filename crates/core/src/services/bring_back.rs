//! Bring back (book 10H §7; master plan archive §5 MC-H4): a remote item's pictures are
//! downloaded at full size into its session folder and, only when every one is on disk, made the
//! session's photos in one transaction (M-H2). Nothing is written to the shop (M-H1) and nothing
//! is deleted in the editor's catalogue; a failed attempt removes only the files it wrote.
//! Core vocabulary only (H7): *the shop*, *remote picture*.

use crate::{
    Error, Result,
    domain::{
        remote::{
            BringBackInput, BringBackOutcome, BringBackProgress, BringBackState, BroughtBack,
            RemoteState,
        },
        session::PublishStep,
        storage::ArchivedImage,
    },
    events::AppEvent,
    mechanisms::remote::{
        DOWNLOAD_RETRIES, PARALLEL_DOWNLOADS, URL_REFRESH_CHUNK, bring_back_names,
    },
    ports::commerce::CommerceAdapter,
    services::{
        remote::RemoteItems,
        retention::{ROLE_CONDITION, ROLE_GALLERY},
    },
};
use std::{
    cell::RefCell,
    path::{Path, PathBuf},
    sync::{
        Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};

/// MC-H4.3 (rule 17): a downloaded body that does not start with the JPEG marker is a failed
/// download.
const JPEG_MARKER: [u8; 3] = [0xFF, 0xD8, 0xFF];

fn failed(reason: impl Into<String>) -> Error {
    Error::BringBackFailed {
        reason: reason.into(),
    }
}

/// `1 photo <one>` / `<n> photos <many>` (master plan archive §4 Words).
fn photos(n: usize, one: &str, many: &str) -> String {
    if n == 1 {
        format!("1 photo {one}")
    } else {
        format!("{n} photos {many}")
    }
}

fn not_downloaded(n: usize) -> Error {
    failed(photos(
        n,
        "could not be downloaded",
        "could not be downloaded",
    ))
}

fn no_longer_in_the_shop(n: usize) -> Error {
    failed(photos(
        n,
        "is no longer in the shop",
        "are no longer in the shop",
    ))
}

/// The outcome of one picture's download attempts.
enum Fetched {
    /// On disk at its name; the bytes' SHA-256 (hex) and size.
    Written(String, usize),
    /// The shop said the address is not there (first pass only: its address is refreshed).
    Gone,
    /// Still missing after every try.
    Failed,
}

/// Clears the session's running mark on every exit path (MC-H4.1).
struct Running<'a> {
    items: &'a RemoteItems,
    session_id: i64,
}

impl Drop for Running<'_> {
    fn drop(&mut self) {
        self.items
            .bring_backs
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(&self.session_id);
    }
}

/// The files one attempt wrote (`.part` and final names): exactly what a failure removes.
#[derive(Default)]
struct Written(Mutex<Vec<PathBuf>>);

impl Written {
    fn note(&self, path: &Path) {
        self.0
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push(path.to_path_buf());
    }

    /// M-H2: deletes every file this attempt wrote, nothing else.
    fn remove_all(&self) {
        for path in self.0.lock().unwrap_or_else(|e| e.into_inner()).drain(..) {
            if path.is_file()
                && let Err(error) = std::fs::remove_file(&path)
            {
                tracing::warn!(%error, file = %path.display(), "remote.bring_back file not removed");
            }
        }
    }
}

impl RemoteItems {
    /// MC-H4: brings the session's remote pictures back to this Mac as its photos, all or none.
    pub fn bring_back(&self, session_id: i64) -> Result<BringBackOutcome> {
        let Some(commerce) = self.commerce.clone() else {
            return Err(Error::NoIntegration);
        };
        let mut state = self.state(session_id)?;
        if state == RemoteState::Unknown {
            // MC-H3.2, MC-H5.4: a press reads an unknown item at once.
            state = self.ensure_record(session_id)?;
        }
        if !matches!(state, RemoteState::Remote(_)) {
            return Err(Error::NotRemote);
        }
        let _running = self.start_running(session_id)?;
        let written = Written::default();
        match self.bring_back_running(commerce.as_ref(), session_id, &written) {
            Ok(outcome) => Ok(outcome),
            Err(error) => {
                written.remove_all();
                let progress = self.bring_back_progress(session_id).unwrap_or_default();
                tracing::info!(
                    "remote.bring_back session={session_id} photos=0 bytes=0 outcome=failed"
                );
                self.events.emit(AppEvent::BringBack {
                    session_id,
                    done: progress.done,
                    total: progress.total,
                    state: BringBackState::Failed,
                    error: Some(error.to_string()),
                });
                Err(error)
            }
        }
    }

    /// MC-H4.1: the progress of the session's running bring back, if one runs.
    pub fn bring_back_progress(&self, session_id: i64) -> Option<BringBackProgress> {
        self.bring_backs
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get(&session_id)
            .copied()
    }

    fn start_running(&self, session_id: i64) -> Result<Running<'_>> {
        let mut running = self.bring_backs.lock().unwrap_or_else(|e| e.into_inner());
        if running.contains_key(&session_id) {
            return Err(Error::BringBackRunning);
        }
        running.insert(session_id, BringBackProgress { done: 0, total: 0 });
        Ok(Running {
            items: self,
            session_id,
        })
    }

    fn set_progress(&self, session_id: i64, progress: BringBackProgress) {
        self.bring_backs
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(session_id, progress);
    }

    /// The rest of MC-H4.1, then MC-H4.2–5. Every file it writes is noted in `written`.
    fn bring_back_running(
        &self,
        commerce: &dyn CommerceAdapter,
        session_id: i64,
        written: &Written,
    ) -> Result<BringBackOutcome> {
        let item_id = self
            .store
            .session(session_id)?
            .session
            .item_id
            .ok_or(Error::NotRemote)?;
        if let Some(latest) = self.store.latest_publish(session_id)?
            && matches!(
                latest.step,
                PublishStep::Held
                    | PublishStep::Prepare
                    | PublishStep::Upload
                    | PublishStep::Update
                    | PublishStep::Describe
            )
        {
            return Err(failed("wait until the publish has finished"));
        }
        let root = (self.capture_root)().map_err(|error| {
            tracing::info!(%error, "remote.bring_back capture root not known");
            failed("the capture folder is not known yet")
        })?;
        let item = self.store.item(item_id)?;
        let pictures = self.store.archived_images(item_id)?;
        let gone = pictures.iter().filter(|p| p.url.is_empty()).count();
        if gone > 0 {
            return Err(no_longer_in_the_shop(gone));
        }
        let folder = root.join(&item.folder_key);
        let names = self.names(&folder, &item.folder_key, &pictures)?;
        let total = pictures.len();
        self.set_progress(session_id, BringBackProgress { done: 0, total });
        std::fs::create_dir_all(&folder)?;
        let paths: Vec<PathBuf> = names.iter().map(|name| folder.join(name)).collect();
        let done = Mutex::new(0usize);
        let progress = |_: usize| {
            let mut done = done.lock().unwrap_or_else(|e| e.into_inner());
            *done += 1;
            let now = BringBackProgress { done: *done, total };
            self.set_progress(session_id, now);
            self.events.emit(AppEvent::BringBack {
                session_id,
                done: now.done,
                total,
                state: BringBackState::Running,
                error: None,
            });
        };

        // MC-H4.3: every picture, `PARALLEL_DOWNLOADS` at a time; a gone address is refreshed
        // once for all stale pictures, then tried again with the full tries.
        let mut hashes: Vec<Option<(String, usize)>> = vec![None; total];
        let first: Vec<(usize, String)> = pictures
            .iter()
            .enumerate()
            .map(|(i, p)| (i, p.url.clone()))
            .collect();
        let mut stale = Vec::new();
        let mut missing = 0;
        for (index, fetched) in
            self.download_originals(commerce, &first, &paths, true, written, &progress)
        {
            match fetched {
                Fetched::Written(sha, size) => hashes[index] = Some((sha, size)),
                Fetched::Gone => stale.push(index),
                Fetched::Failed => missing += 1,
            }
        }
        let mut gone_now = 0;
        if !stale.is_empty() {
            let mut retry = Vec::new();
            for chunk in stale.chunks(URL_REFRESH_CHUNK) {
                let ids: Vec<String> = chunk
                    .iter()
                    .map(|i| pictures[*i].media_id.clone())
                    .collect();
                let answers = match commerce.media_urls(&ids) {
                    Ok(answers) => answers,
                    Err(error) => {
                        tracing::warn!(%error, "remote.urls_refreshed not read");
                        missing += chunk.len();
                        continue;
                    }
                };
                for (k, index) in chunk.iter().enumerate() {
                    let Some(answer) = answers.get(k) else {
                        missing += 1;
                        continue;
                    };
                    let media_id = &pictures[*index].media_id;
                    let url = answer.as_ref().and_then(|image| image.url.clone());
                    let bytes = answer.as_ref().and_then(|image| image.bytes);
                    self.store
                        .update_remote_urls(item_id, &[(media_id.clone(), url.clone(), bytes)])?;
                    match url {
                        Some(url) => retry.push((*index, url)),
                        None => {
                            gone_now += 1;
                            self.mark_gone(item_id, media_id)?;
                        }
                    }
                }
            }
            tracing::info!(
                "remote.urls_refreshed asked={} found={} gone={gone_now}",
                stale.len(),
                retry.len()
            );
            if gone_now > 0 {
                self.report_gone()?;
            }
            for (index, fetched) in
                self.download_originals(commerce, &retry, &paths, false, written, &progress)
            {
                match fetched {
                    Fetched::Written(sha, size) => hashes[index] = Some((sha, size)),
                    Fetched::Gone | Fetched::Failed => missing += 1,
                }
            }
        }
        if gone_now > 0 {
            return Err(no_longer_in_the_shop(gone_now));
        }
        if missing > 0 || hashes.iter().any(Option::is_none) {
            return Err(not_downloaded(missing.max(1)));
        }

        // MC-H4.4: the import, one transaction, inside the undo lock (MC-H4.5).
        let mut files = Vec::with_capacity(total);
        let mut bytes = 0;
        for ((picture, path), hash) in pictures.iter().zip(&paths).zip(hashes) {
            let (sha256, size) = hash.ok_or_else(|| not_downloaded(1))?;
            bytes += size;
            files.push(BroughtBack {
                role: picture.role.clone(),
                position: picture.position,
                media_id: picture.media_id.clone(),
                path: std::fs::canonicalize(path)?.to_string_lossy().into_owned(),
                sha256,
            });
        }
        let input = BringBackInput {
            session_id,
            item_id,
            files,
            link_role_condition: format!("attribute:{}", commerce.condition_photos_key()),
            at: self.clock.now_ms(),
        };
        let retired: Vec<i64> = self
            .store
            .photos_in_session(session_id)?
            .iter()
            .map(|photo| photo.id)
            .collect();
        let mut photo_ids = Vec::new();
        self.history.retire_session(session_id, &retired, || {
            photo_ids = self.store.bring_back(input)?;
            Ok(true)
        })?;

        // MC-H4.5: after the transaction nothing is undone; the editor is asked to list the folder.
        let synced = match self.editing.sync_folder(&folder.to_string_lossy()) {
            Ok(()) => true,
            Err(error) => {
                tracing::warn!(%error, "remote.bring_back folder not synced");
                false
            }
        };
        self.leave_gone(item_id)?;
        tracing::info!(
            "remote.bring_back item={item_id} photos={} bytes={bytes} outcome=done",
            photo_ids.len()
        );
        self.events.emit(AppEvent::BringBack {
            session_id,
            done: total,
            total,
            state: BringBackState::Done,
            error: None,
        });
        self.events.emit(AppEvent::SessionChanged { session_id });
        Ok(BringBackOutcome { photo_ids, synced })
    }

    /// MC-H4.2: the names, `taken` = a photo row of any session has the canonical path. A store
    /// error is kept, ends the names and fails the bring back before any download.
    fn names(
        &self,
        folder: &Path,
        folder_key: &str,
        pictures: &[ArchivedImage],
    ) -> Result<Vec<String>> {
        let error: RefCell<Option<Error>> = RefCell::new(None);
        let taken = |name: &str| -> bool {
            if error.borrow().is_some() {
                return false;
            }
            match self
                .store
                .photos_for_raw(&folder.join(name).to_string_lossy())
            {
                Ok(rows) => !rows.is_empty(),
                Err(e) => {
                    *error.borrow_mut() = Some(e);
                    false
                }
            }
        };
        let gallery = pictures.iter().filter(|p| p.role == ROLE_GALLERY).count();
        let condition = pictures.iter().filter(|p| p.role == ROLE_CONDITION).count();
        let names = bring_back_names(folder_key, gallery, condition, &taken);
        match error.into_inner() {
            Some(error) => Err(error),
            None => Ok(names),
        }
    }

    /// Downloads `(index, url)` to `paths[index]`, at most `PARALLEL_DOWNLOADS` at once;
    /// `progress` runs after each file is on disk.
    fn download_originals(
        &self,
        commerce: &dyn CommerceAdapter,
        jobs: &[(usize, String)],
        paths: &[PathBuf],
        gone_is_stale: bool,
        written: &Written,
        progress: &(dyn Fn(usize) + Sync),
    ) -> Vec<(usize, Fetched)> {
        let next = AtomicUsize::new(0);
        let results = Mutex::new(Vec::with_capacity(jobs.len()));
        std::thread::scope(|scope| {
            for _ in 0..PARALLEL_DOWNLOADS.min(jobs.len()) {
                scope.spawn(|| {
                    while let Some((index, url)) = jobs.get(next.fetch_add(1, Ordering::SeqCst)) {
                        let fetched =
                            fetch_one(commerce, url, &paths[*index], gone_is_stale, written);
                        if matches!(fetched, Fetched::Written(..)) {
                            progress(*index);
                        }
                        results
                            .lock()
                            .unwrap_or_else(|e| e.into_inner())
                            .push((*index, fetched));
                    }
                });
            }
        });
        results.into_inner().unwrap_or_else(|e| e.into_inner())
    }
}

/// One picture: 1 + `DOWNLOAD_RETRIES` tries; a body that is not a JPEG is a failed try; a gone
/// address ends the first pass at once (its refresh follows).
fn fetch_one(
    commerce: &dyn CommerceAdapter,
    url: &str,
    path: &Path,
    gone_is_stale: bool,
    written: &Written,
) -> Fetched {
    for _ in 0..=DOWNLOAD_RETRIES {
        match commerce.fetch_original(url) {
            Ok(bytes) if bytes.starts_with(&JPEG_MARKER) => match write(path, &bytes, written) {
                Ok(sha) => return Fetched::Written(sha, bytes.len()),
                Err(error) => tracing::warn!(%error, "remote.bring_back file not written"),
            },
            Ok(_) => tracing::info!("remote.bring_back download is not a JPEG"),
            Err(Error::ImageGone) if gone_is_stale => return Fetched::Gone,
            Err(error) => tracing::info!(%error, "remote.bring_back download failed"),
        }
    }
    Fetched::Failed
}

/// `<name>.part`, then renamed to `<name>` (replacing a leftover of an interrupted attempt, which
/// no photo row names, MC-H4.2); answers the bytes' SHA-256.
fn write(path: &Path, bytes: &[u8], written: &Written) -> Result<String> {
    use sha2::Digest;
    let name = path
        .file_name()
        .ok_or_else(|| Error::InvalidData("bring back file name".into()))?
        .to_string_lossy()
        .into_owned();
    let partial = path.with_file_name(format!("{name}.part"));
    written.note(&partial);
    std::fs::write(&partial, bytes)?;
    written.note(path);
    std::fs::rename(&partial, path)?;
    Ok(sha2::Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect())
}
