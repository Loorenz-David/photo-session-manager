//! Photo files follow their session (owner correction A21, 2026-11-18). Evidence (work Mac): eight
//! photos taken for T1-181126 were filed into T4-181126's folder (the A16 switch bug), then moved to
//! T1 in the app. Moving only re-pointed the rows: the RAWs and Capture One's files of them stayed
//! in T4's folder, T1's folder stayed empty, and T1's export waited forever ("Capture One has not
//! loaded …"): Capture One finds a photo through the collection it shows (T1, empty) or through
//! its own list of the photo's folder, which it refreshes only when it looks at that folder.
//!
//! Now a photo's RAW, with Capture One's files of it (its settings and masks under
//! `CaptureOne/Settings*/`, its proxies and thumbnails under `CaptureOne/Cache/`) and the app's
//! caches of it, is moved into its session's capture folder whenever they differ: after a move, an
//! undo or redo of one, "Assign product", or the "Fix photo files location" action. The
//! adjustments travel with the photo (Capture One reads them from the settings file next to it).
//!
//! Never overwritten: a file of the same name in the target folder leaves the photo where it is
//! (reported). Left alone: a RAW outside the capture root, a RAW another session also uses (a
//! copy), a RAW Capture One is rendering. Refused for a session while its export or a publish step
//! runs. If a file cannot move, what moved of that photo is moved back and its row is unchanged.

use super::{CAPTURE_ONE_FOLDER, StorageService, resolved};
use crate::{
    Error, Result, domain::session::PublishStep, events::AppEvent, mechanisms::arrival::path_key,
    ports::editing::EditingProvider,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// The folder of the Unassigned session (R-I28).
const UNASSIGNED_FOLDER: &str = "_Unassigned";

/// One photo whose RAW is not in its session's capture folder.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MisplacedPhoto {
    pub photo_id: i64,
    pub session_id: i64,
    pub raw_path: String,
    /// The session's capture folder (where the RAW belongs).
    pub folder: String,
}

/// What one placement did.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilesPlaced {
    /// RAWs moved into their session's folder.
    pub moved: usize,
    /// Capture One files moved with them.
    pub sidecars: usize,
    /// Photos left where they are, with why (file name, reason in the owner's words).
    pub skipped: Vec<(String, String)>,
    /// Capture folders that lost or gained files (Capture One is asked to look at them again).
    pub folders: Vec<String>,
}

/// One file to move and where.
struct Move {
    from: PathBuf,
    to: PathBuf,
}

impl StorageService {
    /// The capture folder of `session_id` (its item's folder key; `_Unassigned` for none).
    fn session_folder(&self, session_id: i64) -> Result<PathBuf> {
        let session = self.store.session(session_id)?;
        let key = match session.session.item_id {
            Some(item) => self.store.item(item)?.folder_key,
            None => UNASSIGNED_FOLDER.into(),
        };
        Ok(self.capture_root().join(key))
    }

    /// The photos of `session_id` whose RAW is in another folder than the session's (A21) and could
    /// be moved there: inside the capture root, still on disk, used by no other session.
    pub fn misplaced_photos(&self, session_id: i64) -> Result<Vec<MisplacedPhoto>> {
        let folder = self.session_folder(session_id)?;
        let root = self.capture_root();
        let mut out = Vec::new();
        for photo in self.store.photos_in_session(session_id)? {
            let raw = resolved(Path::new(&photo.raw_path));
            if raw.parent() == Some(folder.as_path()) || !raw.starts_with(&root) || !raw.is_file() {
                continue;
            }
            if self
                .store
                .photos_for_raw(&photo.raw_path)?
                .iter()
                .any(|p| p.session_id != session_id)
            {
                continue;
            }
            out.push(MisplacedPhoto {
                photo_id: photo.id,
                session_id,
                raw_path: photo.raw_path,
                folder: folder.to_string_lossy().into_owned(),
            });
        }
        Ok(out)
    }

    /// Why the files of these sessions cannot move now: an export of one runs, or a publish step.
    pub fn files_blocked(&self, sessions: &[i64]) -> Result<Option<String>> {
        for &session_id in sessions {
            if self.export.running_for(session_id) {
                return Ok(Some(
                    "An export of these photos is running. Wait for it to finish, then try again."
                        .into(),
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
                    "These photos are being published. Wait for the publish to finish, then try again."
                        .into(),
                ));
            }
        }
        Ok(None)
    }

    /// Moves every misplaced photo of `sessions` into its session's capture folder (A21). A RAW the
    /// editor is rendering now is left for later; the editor's queued renders follow a moved RAW.
    pub fn place_files(
        &self,
        sessions: &[i64],
        editor: Option<&dyn EditingProvider>,
    ) -> Result<FilesPlaced> {
        let mut placed = FilesPlaced::default();
        // The clean-up worker does not delete files while they move.
        let _work = self
            .work
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?;
        for &session_id in sessions {
            let misplaced = self.misplaced_photos(session_id)?;
            if misplaced.is_empty() {
                continue;
            }
            if let Some(reason) = self.files_blocked(&[session_id])? {
                for photo in &misplaced {
                    placed
                        .skipped
                        .push((file_name(&photo.raw_path), reason.clone()));
                }
                continue;
            }
            let mut done: Vec<String> = Vec::new();
            for photo in misplaced {
                if done.contains(&photo.raw_path) {
                    continue;
                }
                done.push(photo.raw_path.clone());
                let name = file_name(&photo.raw_path);
                if editor.is_some_and(|editor| editor.rendering(&photo.raw_path)) {
                    placed.skipped.push((
                        name,
                        "Capture One is rendering it. Try again in a moment.".into(),
                    ));
                    continue;
                }
                match self.place_one(&photo) {
                    Ok((sidecars, new_raw)) => {
                        if let Some(editor) = editor {
                            editor.raw_moved(&photo.raw_path, &new_raw);
                        }
                        placed.moved += 1;
                        placed.sidecars += sidecars;
                        let from = resolved(Path::new(&photo.raw_path))
                            .parent()
                            .map(|p| p.to_string_lossy().into_owned())
                            .unwrap_or_default();
                        for folder in [from, photo.folder.clone()] {
                            if !placed.folders.contains(&folder) {
                                placed.folders.push(folder);
                            }
                        }
                    }
                    Err(reason) => {
                        tracing::warn!(
                            target: "psm::capture_one",
                            "photos.files_not_moved file={} session={} to_folder={} reason={}",
                            name,
                            session_id,
                            folder_name(&photo.folder),
                            reason
                        );
                        placed.skipped.push((name, reason));
                    }
                }
            }
            self.events.emit(AppEvent::SessionChanged { session_id });
        }
        Ok(placed)
    }

    /// Moves one photo's RAW, Capture One's files of it and the app's caches of it; then its rows.
    /// On failure everything of it that moved is moved back. The error is the owner's sentence.
    /// Returns the Capture One files moved with the RAW and the RAW's new path.
    fn place_one(&self, photo: &MisplacedPhoto) -> std::result::Result<(usize, String), String> {
        let raw = resolved(Path::new(&photo.raw_path));
        let (Some(source), Some(name)) = (raw.parent(), raw.file_name()) else {
            return Err("its file name could not be read".into());
        };
        let target = PathBuf::from(&photo.folder);
        let name = name.to_string_lossy().into_owned();
        let mut files = vec![Move {
            from: raw.clone(),
            to: target.join(&name),
        }];
        let capture_one = source.join(CAPTURE_ONE_FOLDER);
        let listed = self
            .fs
            .files_deep(&capture_one, &[])
            .map_err(|e| format!("Capture One's files of it could not be listed: {e}"))?;
        for file in listed {
            if let Some(relative) = capture_one_file(&capture_one, &file.path, &name) {
                files.push(Move {
                    from: file.path,
                    to: target.join(CAPTURE_ONE_FOLDER).join(relative),
                });
            }
        }
        if let Some(taken) = files.iter().find(|m| m.to.exists()) {
            return Err(format!(
                "a file named {} is already in {}",
                taken
                    .to
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default(),
                folder_name(&photo.folder)
            ));
        }
        let mut moved: Vec<&Move> = Vec::new();
        for file in &files {
            if let Err(error) = self.fs.rename(&file.from, &file.to) {
                undo(self, &moved);
                return Err(format!(
                    "{} could not be moved: {error}",
                    file.from.display()
                ));
            }
            moved.push(file);
        }
        // The app's caches of the RAW are named after its path.
        let new_raw = target.join(&name).to_string_lossy().into_owned();
        let (old_key, new_key) = (path_key(&photo.raw_path), path_key(&new_raw));
        let mut caches: Vec<Move> = Vec::new();
        for cache in [
            &self.locations.preview_cache,
            &self.locations.masters,
            &self.locations.outputs,
        ] {
            let from = cache.join(&old_key[..2]).join(&old_key);
            let to = cache.join(&new_key[..2]).join(&new_key);
            if !from.exists() || to.exists() {
                continue;
            }
            match self.fs.rename(&from, &to) {
                Ok(()) => caches.push(Move { from, to }),
                Err(error) => {
                    tracing::warn!(%error, cache = %from.display(), "photos.files cache kept (renders again)")
                }
            }
        }
        let dirs: Vec<(String, String)> = caches
            .iter()
            .map(|m| {
                (
                    m.from.to_string_lossy().into_owned(),
                    m.to.to_string_lossy().into_owned(),
                )
            })
            .collect();
        if let Err(error) = self.store.relocate_raw(&photo.raw_path, &new_raw, &dirs) {
            undo(self, &caches.iter().collect::<Vec<_>>());
            undo(self, &moved);
            return Err(format!("its record could not be updated: {error}"));
        }
        let sidecars = files.len() - 1;
        tracing::info!(
            target: "psm::capture_one",
            "photos.files_moved file={} session={} from_folder={} to_folder={} sidecars={} caches={}",
            name,
            photo.session_id,
            folder_name(&source.to_string_lossy()),
            folder_name(&photo.folder),
            sidecars,
            caches.len()
        );
        Ok((sidecars, new_raw))
    }
}

/// Moves files back, newest first (best effort: each failure is logged).
fn undo(service: &StorageService, moved: &[&Move]) {
    for file in moved.iter().rev() {
        if let Err(error) = service.fs.rename(&file.to, &file.from) {
            tracing::warn!(%error, from = %file.to.display(), to = %file.from.display(), "photos.files not moved back");
        }
    }
}

/// `path`'s place under `capture_one` when it is Capture One's file of the RAW named `name`: its
/// settings or masks (`Settings*/<name>.*`) or a cache file (`Cache/…/<name>.*`).
fn capture_one_file(capture_one: &Path, path: &Path, name: &str) -> Option<PathBuf> {
    let relative = path.strip_prefix(capture_one).ok()?;
    let parts: Vec<String> = relative
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();
    let file = parts.last()?;
    if !file.starts_with(&format!("{name}.")) {
        return None;
    }
    match parts.first().map(String::as_str) {
        Some(first) if first.starts_with("Settings") && parts.len() == 2 => {
            Some(relative.to_path_buf())
        }
        Some("Cache") if parts.len() >= 2 => Some(relative.to_path_buf()),
        _ => None,
    }
}

fn file_name(raw: &str) -> String {
    Path::new(raw)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| raw.to_owned())
}

fn folder_name(folder: &str) -> String {
    Path::new(folder)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| folder.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a21_capture_one_files_of_a_raw() {
        let c1 = Path::new("/S/T4/CaptureOne");
        let of = |p: &str| capture_one_file(c1, Path::new(p), "DSC06785.ARW");
        assert_eq!(
            of("/S/T4/CaptureOne/Settings1690/DSC06785.ARW.cos"),
            Some(PathBuf::from("Settings1690/DSC06785.ARW.cos"))
        );
        assert_eq!(
            of("/S/T4/CaptureOne/Settings1690/DSC06785.ARW.comask"),
            Some(PathBuf::from("Settings1690/DSC06785.ARW.comask"))
        );
        assert_eq!(
            of("/S/T4/CaptureOne/Cache/Proxies/DSC06785.ARW.cop"),
            Some(PathBuf::from("Cache/Proxies/DSC06785.ARW.cop"))
        );
        assert!(of("/S/T4/CaptureOne/Cache/Thumbnails/DSC06785.ARW.[00c977eb].cot").is_some());
        assert_eq!(of("/S/T4/CaptureOne/Settings1690/DSC06786.ARW.cos"), None);
        assert_eq!(of("/S/T4/CaptureOne/Settings1690/DSC067851.ARW.cos"), None);
        assert_eq!(of("/S/T4/CaptureOne/Other/DSC06785.ARW.cos"), None);
        assert_eq!(of("/S/T4/CaptureOne/Settings1690/x/DSC06785.ARW.cos"), None);
    }
}
