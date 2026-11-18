//! "Delete session" (owner correction A17, 2026-11-18): for test items and mistakes, e.g. an item
//! whose Shopify product was deleted, so retention can never clear it. Scope is Session Manager
//! only (the owner's words): nothing is asked of or changed in Shopify.
//!
//! The session's files go to the macOS Trash, so the owner can put them back: every RAW only this
//! session uses, Capture One's files named after it, its verified duplicates, the app's previews,
//! masters and outputs of it, the session's exported JPGs, and the session's capture folder once
//! nothing but Capture One's data is left in it. A RAW another session still uses stays, and
//! nothing outside the capture root is ever touched. Then every row naming the session goes in one
//! transaction ([`Store::delete_session_rows`]), and its undo history with it.
//!
//! Refused: the Unassigned session; while an export, a publish step or a research run of the
//! session is running. The filing target leaves the session before the first file moves, so no
//! new photo can be filed into a folder that is going away. Rows go only when every file was
//! moved or was already gone; when a file cannot be moved, the session stays and the owner is told
//! (what did move is in the Trash).

use super::{CAPTURE_ONE_FOLDER, QueuedKind, StorageService, capture_one_file_of};
use crate::{
    Error, Result,
    domain::{photo::Classification, session::PublishStep},
    events::AppEvent,
    mechanisms::arrival::path_key,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

/// What the confirm dialog shows about the session to delete.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeletePreview {
    pub session_id: i64,
    pub sku: String,
    pub title: String,
    pub item_type: String,
    /// Photos shown in the session (not denied).
    pub photos: i64,
    /// It was published once (Shopify keeps the product; the app does not change it).
    pub published: bool,
    /// Its files were already cleared after publishing (only records are left).
    pub archived: bool,
    /// Photos are filed into it now (they go to Unassigned once it is deleted).
    pub active: bool,
}

/// What one "Delete session" did.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionDeleted {
    pub sku: String,
    /// Photo rows removed (denied ones included).
    pub photos: i64,
    /// Files and folders moved to the Trash.
    pub files_trashed: usize,
    /// Files the session named that were already gone (cleared by retention, deleted by hand).
    pub files_missing: usize,
    /// The session's capture folder went to the Trash too.
    pub folder_trashed: bool,
    pub research_runs: i64,
}

const UNASSIGNED: &str = "The Unassigned session cannot be deleted.";

impl StorageService {
    /// The confirm dialog's facts (A17).
    pub fn delete_preview(&self, session_id: i64) -> Result<DeletePreview> {
        let session = self.store.session(session_id)?;
        let item_id = session
            .session
            .item_id
            .ok_or_else(|| Error::DeleteRefused(UNASSIGNED.into()))?;
        let item = self.store.item(item_id)?;
        let photos = self
            .store
            .photos_in_session(session_id)?
            .iter()
            .filter(|p| p.classification != Classification::Deleted)
            .count() as i64;
        Ok(DeletePreview {
            session_id,
            sku: item.sku,
            title: item.title,
            item_type: item.item_type,
            photos,
            published: session.session.published_at.is_some(),
            archived: self.store.item_retention(item_id)?.cleared_at.is_some(),
            active: self.store.active_session_id()? == Some(session_id),
        })
    }

    /// Why the session cannot be deleted now (an export, a publish step or research running).
    fn delete_blocked(&self, session_id: i64) -> Result<Option<String>> {
        if self.export.running_for(session_id) {
            return Ok(Some(
                "An export of this session is running. Wait for it to finish, then delete.".into(),
            ));
        }
        if let Some(latest) = self.store.latest_publish(session_id)?
            && matches!(
                latest.step,
                PublishStep::Prepare
                    | PublishStep::Upload
                    | PublishStep::Update
                    | PublishStep::Describe
            )
        {
            return Ok(Some(
                "This session is being published. Wait for the publish to finish, then delete."
                    .into(),
            ));
        }
        if self.store.session_research_running(session_id)? {
            return Ok(Some(
                "Research is running for this item. Wait for it to finish, then delete.".into(),
            ));
        }
        Ok(None)
    }

    /// Deletes the session (A17): files to the Trash, then its rows and undo history.
    /// `capture_one` is what the caller did about Capture One first (for the log line).
    pub fn delete_session(&self, session_id: i64, capture_one: &str) -> Result<SessionDeleted> {
        let trash = self
            .trash
            .clone()
            .ok_or_else(|| Error::InvalidData("no Trash configured".into()))?;
        let session = self.store.session(session_id)?;
        let item_id = session
            .session
            .item_id
            .ok_or_else(|| Error::DeleteRefused(UNASSIGNED.into()))?;
        let item = self.store.item(item_id)?;
        if let Some(reason) = self.delete_blocked(session_id)? {
            return Err(Error::DeleteRefused(reason));
        }
        // The clean-up worker neither queues nor deletes this session's files meanwhile.
        let mut work = self
            .work
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        let photos = self.store.photos_in_session(session_id)?;
        let photo_ids: Vec<i64> = photos.iter().map(|p| p.id).collect();
        work.queue.retain(|q| q.kind != QueuedKind::Item(item_id));
        work.denied.retain(|id| !photo_ids.contains(id));

        let mut outcome = SessionDeleted {
            sku: item.sku.clone(),
            photos: 0,
            files_trashed: 0,
            files_missing: 0,
            folder_trashed: false,
            research_runs: 0,
        };
        let mut failed: Vec<String> = Vec::new();
        let mut act = || -> Result<bool> {
            // Checked again under the history lock: nothing started meanwhile.
            if let Some(reason) = self.delete_blocked(session_id)? {
                return Err(Error::DeleteRefused(reason));
            }
            // No new photo is filed into a folder that is going away.
            if self.store.active_session_id()? == Some(session_id) {
                self.store.set_active_session_id(None)?;
                tracing::info!(
                    target: "psm::arrival",
                    "arrival.target session=none folder=_Unassigned from={session_id} reason=session_deleted"
                );
            }
            for path in self.session_paths(session_id, &photos)? {
                match trash.trash(&path) {
                    Ok(true) => outcome.files_trashed += 1,
                    Ok(false) => outcome.files_missing += 1,
                    Err(error) => {
                        tracing::warn!(%error, path = %path.display(), "session.delete not moved to the Trash");
                        failed.push(path.display().to_string());
                    }
                }
            }
            if failed.is_empty() {
                // The export folder of the item, then the capture folder once only Capture
                // One's data is left in it.
                if let Ok(folder) = self.export.folder(session_id)
                    && trash.trash(&folder).unwrap_or_else(|error| {
                        tracing::warn!(%error, folder = %folder.display(), "session.delete export folder kept");
                        false
                    })
                {
                    outcome.files_trashed += 1;
                }
                let capture = self.capture_root().join(&item.folder_key);
                if only_capture_one_data(&capture) {
                    match trash.trash(&capture) {
                        Ok(moved) => outcome.folder_trashed = moved,
                        Err(error) => {
                            tracing::warn!(%error, folder = item.folder_key, "session.delete capture folder kept")
                        }
                    }
                } else {
                    tracing::info!(
                        folder = item.folder_key,
                        "session.delete capture folder kept: it holds other files"
                    );
                }
            } else {
                return Ok(false);
            }
            let rows = self.store.delete_session_rows(session_id)?;
            outcome.photos = rows.photos;
            outcome.research_runs = rows.research_runs;
            Ok(true)
        };
        let deleted = match &self.history {
            Some(history) => history.retire_session(session_id, &photo_ids, &mut act)?,
            None => act()?,
        };
        drop(work);
        if !deleted {
            return Err(Error::DeleteRefused(format!(
                "{} could not be moved to the Trash, so the session was kept ({} moved there already). First: {}",
                plural(failed.len(), "file"),
                outcome.files_trashed,
                failed.first().cloned().unwrap_or_default()
            )));
        }
        tracing::info!(
            target: "psm::session",
            "session.deleted session={} sku={} photos={} files_trashed={} files_missing={} folder_trashed={} research_runs={} capture_one_result={}",
            session_id,
            outcome.sku,
            outcome.photos,
            outcome.files_trashed,
            outcome.files_missing,
            outcome.folder_trashed,
            outcome.research_runs,
            capture_one
        );
        self.events.emit(AppEvent::SessionChanged { session_id });
        self.request_scan();
        Ok(outcome)
    }

    /// Everything of the session that goes to the Trash before its folder: per RAW only this
    /// session uses (inside the capture root), the RAW, Capture One's files named after it, the
    /// app's preview, master and output folders of it; and the verified duplicates of those RAWs.
    fn session_paths(
        &self,
        session_id: i64,
        photos: &[crate::domain::photo::Photo],
    ) -> Result<Vec<PathBuf>> {
        let root = self.capture_root();
        let mut raws: Vec<String> = Vec::new();
        let mut hashes: HashSet<String> = HashSet::new();
        let mut out: Vec<PathBuf> = Vec::new();
        for photo in photos {
            if raws.contains(&photo.raw_path) {
                continue;
            }
            raws.push(photo.raw_path.clone());
            let shared = self
                .store
                .photos_for_raw(&photo.raw_path)?
                .iter()
                .any(|p| p.session_id != session_id);
            if shared {
                tracing::info!(
                    raw = photo.raw_path,
                    "session.delete RAW kept: another session uses it"
                );
                continue;
            }
            let raw = Path::new(&photo.raw_path);
            if !super::resolved(raw).starts_with(&root) {
                tracing::warn!(
                    raw = photo.raw_path,
                    "session.delete RAW kept: outside the capture folder"
                );
                continue;
            }
            if let Some(hash) = &photo.raw_hash {
                hashes.insert(hash.clone());
            }
            out.push(raw.to_path_buf());
            if let (Some(folder), Some(name)) = (raw.parent(), raw.file_name()) {
                let name = name.to_string_lossy();
                let capture_one = folder.join(CAPTURE_ONE_FOLDER);
                for file in self.fs.files_deep(&capture_one, &[])? {
                    if capture_one_file_of(&capture_one, &file.path, &name) {
                        out.push(file.path);
                    }
                }
            }
            let key = path_key(&photo.raw_path);
            for cache in [
                &self.locations.preview_cache,
                &self.locations.masters,
                &self.locations.outputs,
            ] {
                out.push(cache.join(&key[..2]).join(&key));
            }
        }
        let own: Vec<String> = raws
            .into_iter()
            .filter(|raw| out.iter().any(|p| p == Path::new(raw)))
            .collect();
        out.extend(
            self.duplicates_of(&own, &hashes)?
                .into_iter()
                .map(|file| file.path),
        );
        out.dedup();
        Ok(out)
    }
}

/// Whether `folder` holds nothing but Capture One's own folder and hidden files (`.DS_Store`)
/// — then it is the session's folder with only Capture One's data of it left, and goes too.
fn only_capture_one_data(folder: &Path) -> bool {
    let Ok(entries) = std::fs::read_dir(folder) else {
        return false;
    };
    entries.flatten().all(|entry| {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        name == CAPTURE_ONE_FOLDER || name.starts_with('.')
    })
}

fn plural(n: usize, word: &str) -> String {
    if n == 1 {
        format!("1 {word}")
    } else {
        format!("{n} {word}s")
    }
}
