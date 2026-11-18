//! Photo files follow their session, end to end (owner correction A21): after every change of a
//! photo's session (moving photos, "Assign product", undo or redo of either) the storage service
//! moves the RAWs and Capture One's files of them into the session's capture folder, then Capture
//! One is asked to look at the folders that lost or gained files (`sync_folder`), so it lists the
//! photos where they now are without waiting for its own folder check. "Fix photo files location"
//! (the session's ⋯ menu, shown only when needed) runs the same for one session: it rescues photos
//! moved before A21, whose files stayed behind.
//!
//! A move is refused up front, with the owner's sentence, while an export or a publish step of a
//! session involved runs. Capture One not running never stops the files: the sync is logged.

use crate::AppHandles;
use psm_core::{
    Error, Result,
    ports::{clock::SystemClock, store::Store},
    services::{
        photos::PhotosService,
        review::{ReviewResult, ReviewService},
        storage::{FilesPlaced, MisplacedPhoto},
    },
};
use std::{collections::BTreeSet, path::Path};

/// Refused while an export or a publish step of one of `sessions` runs.
pub fn check(app: &AppHandles, sessions: &[i64]) -> Result<()> {
    match app.storage.files_blocked(sessions)? {
        Some(reason) => Err(Error::FilesBusy(reason)),
        None => Ok(()),
    }
}

/// Moves the misplaced photos of `sessions` into their folders, then asks Capture One to look at
/// every folder that changed. Never fails the caller's operation: what could not move is logged
/// and stays visible through [`misplaced`].
pub fn place(app: &AppHandles, sessions: &[i64]) -> FilesPlaced {
    let placed = match app
        .storage
        .place_files(sessions, Some(app.editing.as_ref()))
    {
        Ok(placed) => placed,
        Err(error) => {
            tracing::warn!(target: "psm::capture_one", %error, "photos.files placement failed");
            return FilesPlaced::default();
        }
    };
    if !placed.folders.is_empty() {
        sync(app, &placed.folders);
    }
    placed
}

/// Asks Capture One to re-read each folder. A folder Capture One has no collection for yet (a
/// session never shot into) is found through the capture root's collection instead.
fn sync(app: &AppHandles, folders: &[String]) {
    for folder in folders {
        let result =
            app.editing
                .sync_folder(folder)
                .or_else(|error| match Path::new(folder).parent() {
                    Some(root) => app.editing.sync_folder(&root.to_string_lossy()),
                    None => Err(error),
                });
        tracing::info!(
            target: "psm::capture_one",
            "capture_one.folder_synced folder={} reason=photo_files_moved result={}",
            Path::new(folder)
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default(),
            result.map_or_else(|e| format!("error: {e}"), |_| "ok".into())
        );
    }
}

/// The sessions the photos are in now.
fn sessions_of(app: &AppHandles, ids: &[i64]) -> Result<BTreeSet<i64>> {
    ids.iter()
        .map(|id| app.store.photo(*id).map(|photo| photo.session_id))
        .collect()
}

/// "Move to …": the rows (with their undo step), then the files.
pub fn move_photos(app: &AppHandles, ids: &[i64], target: i64) -> Result<ReviewResult> {
    let mut sessions = sessions_of(app, ids)?;
    sessions.insert(target);
    let sessions: Vec<i64> = sessions.into_iter().collect();
    check(app, &sessions)?;
    let result = ReviewService::new(app.store.as_ref(), &SystemClock, app.history.as_ref())
        .move_photos(ids, target)?;
    place(app, &sessions);
    Ok(result)
}

/// "Assign product": Unassigned's photos to the item's session, then their files.
pub fn assign_unassigned(app: &AppHandles, item_id: i64) -> Result<ReviewResult> {
    check(app, &[1])?;
    let result = PhotosService {
        arrival: app.arrival.as_ref(),
        editing: app.editing.as_ref(),
        store: app.store.as_ref(),
        clock: &SystemClock,
        history: app.history.as_ref(),
    }
    .assign_unassigned(item_id)?;
    let mut sessions = vec![1];
    if let Some(target) = app.store.session_for_item(item_id)? {
        sessions.push(target);
    }
    place(app, &sessions);
    Ok(result)
}

/// After an undo or redo: the files of the sessions it changed follow.
pub fn after_history(app: &AppHandles, sessions: &[i64]) {
    place(app, sessions);
}

/// What "Fix photo files location" would move for the session.
pub fn misplaced(app: &AppHandles, session_id: i64) -> Result<Vec<MisplacedPhoto>> {
    app.storage.misplaced_photos(session_id)
}

/// "Fix photo files location": refused while an export or publish step of the session runs.
pub fn fix(app: &AppHandles, session_id: i64) -> Result<FilesPlaced> {
    check(app, &[session_id])?;
    let placed = place(app, &[session_id]);
    tracing::info!(
        target: "psm::capture_one",
        "photos.files_fixed session={} moved={} sidecars={} left={}",
        session_id,
        placed.moved,
        placed.sidecars,
        placed.skipped.len()
    );
    Ok(placed)
}
