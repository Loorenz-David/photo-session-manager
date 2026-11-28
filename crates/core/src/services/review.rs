use crate::domain::{
    photo::{Classification, Photo},
    session::SessionDto,
};
use crate::mechanisms::position;
use crate::ports::{
    clock::Clock,
    store::{Store, StoreTx},
};
use crate::services::history::{
    History, PhotoPlace, Place, Step, StepKind, label_classify, label_reorder, label_transfer,
    photo_place,
};
use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewResult {
    pub sessions: Vec<SessionDto>,
    pub active_photo_id: Option<i64>,
}

pub struct ReviewService<'a> {
    store: &'a dyn Store,
    clock: &'a dyn Clock,
    history: &'a History,
}

impl<'a> ReviewService<'a> {
    pub fn new(store: &'a dyn Store, clock: &'a dyn Clock, history: &'a History) -> Self {
        Self {
            store,
            clock,
            history,
        }
    }

    fn result(&self, ids: &[i64], active_photo_id: Option<i64>) -> Result<ReviewResult> {
        Ok(ReviewResult {
            sessions: ids
                .iter()
                .map(|id| self.store.session(*id))
                .collect::<Result<Vec<_>>>()?,
            active_photo_id,
        })
    }

    pub fn classify(&self, photo_id: i64, target: Classification) -> Result<ReviewResult> {
        if target == Classification::Deleted {
            return self.delete(photo_id);
        }
        crate::services::retention::ensure_photo_not_cleared(self.store, photo_id)?;
        // P66: a denied photo whose files were deleted cannot come back.
        if self
            .store
            .photo_denial(photo_id)?
            .is_some_and(|denial| denial.files_deleted_at.is_some())
        {
            return Err(Error::InvalidValue(
                "the denied photo's file was deleted".into(),
            ));
        }
        let mut session_id = 0;
        let mut step = None;
        let at = self.clock.now_ms();
        self.store.transaction(&mut |tx| {
            let photo = tx.photo(photo_id)?;
            session_id = photo.session_id;
            if photo.classification == target {
                return Ok(());
            }
            if photo.classification.is_ordered() {
                let old = group(tx, session_id, photo.classification)?;
                tx.set_positions(&position::remove_at(&old, photo_id))?;
            }
            let next = if target.is_ordered() {
                let old = group(tx, session_id, target)?;
                Some(old.len() as i64 + 1)
            } else {
                None
            };
            tx.set_photo_review(photo_id, session_id, target, next)?;
            tx.photo_place_mark(photo_id, true, false, None)?;
            crate::services::template_placement::renumber_main(tx, session_id)?;
            tx.enqueue_mirror_tag(&photo.raw_path, photo_id, at)?;
            tx.touch_session(session_id, at)?;
            let before = photo_place(&photo);
            let after = Place {
                classification: target,
                position: next,
            };
            step = Some(Step {
                label: label_classify(before, after, 1),
                kind: StepKind::Classify {
                    session_id,
                    photos: vec![PhotoPlace {
                        photo_id,
                        before,
                        after,
                    }],
                },
            });
            Ok(())
        })?;
        if let Some(step) = step {
            self.history.record(session_id, step);
            self.history
                .announce(&[(photo_id, session_id)], &[session_id]);
        }
        self.result(&[session_id], None)
    }

    pub fn reorder(&self, photo_id: i64, position: i64) -> Result<ReviewResult> {
        crate::services::retention::ensure_photo_not_cleared(self.store, photo_id)?;
        let mut session_id = 0;
        let mut step = None;
        let at = self.clock.now_ms();
        self.store.transaction(&mut |tx| {
            let photo = tx.photo(photo_id)?;
            session_id = photo.session_id;
            if !photo.classification.is_ordered() {
                return Err(Error::InvalidValue("photo has no ordered group".into()));
            }
            let old = group(tx, session_id, photo.classification)?;
            let next = position::reorder(&old, photo_id, position);
            if old == next {
                return Ok(());
            }
            tx.set_positions(&next)?;
            let place_of_row = |tx: &mut dyn StoreTx| -> Result<(Option<i64>, bool)> {
                Ok(tx
                    .photo_place_row(photo_id)?
                    .map_or((None, false), |r| (r.slot, r.place_by_hand)))
            };
            let place_before = place_of_row(tx)?;
            tx.photo_place_mark(photo_id, true, false, Some(position))?;
            let place_after = place_of_row(tx)?;
            crate::services::template_placement::renumber_main(tx, session_id)?;
            tx.touch_session(session_id, at)?;
            let from = photo.position.unwrap();
            let to = next.iter().find(|(id, _)| *id == photo_id).unwrap().1;
            step = Some(Step {
                label: label_reorder(from, to),
                kind: StepKind::Reorder {
                    session_id,
                    photo_id,
                    from,
                    to,
                    place_before,
                    place_after,
                },
            });
            Ok(())
        })?;
        if let Some(step) = step {
            self.history.record(session_id, step);
            self.history
                .announce(&[(photo_id, session_id)], &[session_id]);
        }
        self.result(&[session_id], None)
    }

    pub fn delete(&self, photo_id: i64) -> Result<ReviewResult> {
        crate::services::retention::ensure_photo_not_cleared(self.store, photo_id)?;
        let mut session_id = 0;
        let mut step = None;
        let at = self.clock.now_ms();
        self.store.transaction(&mut |tx| {
            let photo = tx.photo(photo_id)?;
            session_id = photo.session_id;
            if photo.classification == Classification::Deleted {
                return Ok(());
            }
            if photo.classification.is_ordered() {
                let old = group(tx, session_id, photo.classification)?;
                tx.set_positions(&position::remove_at(&old, photo_id))?;
            }
            tx.set_photo_review(photo_id, session_id, Classification::Deleted, None)?;
            tx.photo_place_mark(photo_id, true, false, None)?;
            crate::services::template_placement::renumber_main(tx, session_id)?;
            // Changelog 46 (2): Deny starts the undo window; the RAW goes when it closes.
            tx.set_photo_denied(photo_id, at)?;
            tx.enqueue_mirror_tag(&photo.raw_path, photo_id, at)?;
            tx.touch_session(session_id, at)?;
            let before = photo_place(&photo);
            let after = Place {
                classification: Classification::Deleted,
                position: None,
            };
            step = Some(Step {
                label: label_classify(before, after, 1),
                kind: StepKind::Classify {
                    session_id,
                    photos: vec![PhotoPlace {
                        photo_id,
                        before,
                        after,
                    }],
                },
            });
            Ok(())
        })?;
        if let Some(step) = step {
            self.history.record(session_id, step);
            self.history
                .announce(&[(photo_id, session_id)], &[session_id]);
        }
        self.result(&[session_id], None)
    }

    /// Compatibility entry for the phone route; `delete` now announces to every surface.
    pub fn delete_announced(
        &self,
        _events: &dyn crate::ports::events::EventSink,
        photo_id: i64,
    ) -> Result<ReviewResult> {
        self.delete(photo_id)
    }

    /// "Keep as published" (owner correction A10, 2026-11-18): a published session that changed
    /// since (photos that arrived late, adjustments in Capture One, edits) is marked published
    /// again as it is, without republishing. Photos that arrived after the publish are denied, so
    /// a later publish never sends them. One undo step restores both.
    pub fn keep_published(&self, session_id: i64) -> Result<ReviewResult> {
        crate::services::retention::ensure_session_not_cleared(self.store, session_id)?;
        let mut step = None;
        let at = self.clock.now_ms();
        self.store.transaction(&mut |tx| {
            step = None;
            let session = tx.session(session_id)?;
            let Some(published_at) = session.published_at else {
                return Err(Error::InvalidData("this session is not published".into()));
            };
            if session.change_seq <= session.published_seq {
                return Ok(());
            }
            let late: Vec<Photo> = tx
                .photos(session_id)?
                .into_iter()
                .filter(|p| {
                    p.classification != Classification::Deleted && p.created_at > published_at
                })
                .collect();
            let mut denied = Vec::new();
            for photo in &late {
                let photo = tx.photo(photo.id)?;
                if photo.classification.is_ordered() {
                    let old = group(tx, session_id, photo.classification)?;
                    tx.set_positions(&position::remove_at(&old, photo.id))?;
                }
                tx.set_photo_review(photo.id, session_id, Classification::Deleted, None)?;
                tx.photo_place_mark(photo.id, true, false, None)?;
                tx.set_photo_denied(photo.id, at)?;
                tx.enqueue_mirror_tag(&photo.raw_path, photo.id, at)?;
                denied.push(PhotoPlace {
                    photo_id: photo.id,
                    before: photo_place(&photo),
                    after: Place {
                        classification: Classification::Deleted,
                        position: None,
                    },
                });
            }
            if !denied.is_empty() {
                crate::services::template_placement::renumber_main(tx, session_id)?;
                tx.touch_session(session_id, at)?;
            }
            let seq_after = tx.session(session_id)?.change_seq;
            tx.set_published_seq(session_id, seq_after)?;
            tracing::info!(
                target: "psm::arrival",
                "session.kept_as_published session={session_id} denied_late_photos={}",
                denied.len()
            );
            step = Some(Step {
                label: "Keep as published".into(),
                kind: StepKind::KeepPublished {
                    session_id,
                    photos: denied,
                    seq_before: session.published_seq,
                    seq_after,
                },
            });
            Ok(())
        })?;
        if let Some(step) = step {
            let photos = match &step.kind {
                StepKind::KeepPublished { photos, .. } => photos
                    .iter()
                    .map(|p| (p.photo_id, session_id))
                    .collect::<Vec<_>>(),
                _ => Vec::new(),
            };
            self.history.record(session_id, step);
            self.history.announce(&photos, &[session_id]);
        }
        self.result(&[session_id], None)
    }

    pub fn move_photos(&self, ids: &[i64], target: i64) -> Result<ReviewResult> {
        self.transfer(ids, target, false)
    }

    pub fn copy_photos(&self, ids: &[i64], target: i64) -> Result<ReviewResult> {
        self.transfer(ids, target, true)
    }

    fn transfer(&self, ids: &[i64], target: i64, copy: bool) -> Result<ReviewResult> {
        // 10F Q3: nothing leaves or enters a cleared item.
        crate::services::retention::ensure_session_not_cleared(self.store, target)?;
        for id in ids {
            crate::services::retention::ensure_photo_not_cleared(self.store, *id)?;
        }
        let mut touched = BTreeSet::new();
        let mut steps: BTreeMap<i64, Vec<PhotoPlace>> = BTreeMap::new();
        let mut copies: BTreeMap<i64, Vec<(i64, i64)>> = BTreeMap::new();
        let mut seq_after = 0;
        let mut target_name = String::new();
        let mut moved_files: Vec<(i64, String, String)> = Vec::new();
        let mut changed_photos = Vec::new();
        let mut target_folder = String::new();
        let at = self.clock.now_ms();
        self.store.transaction(&mut |tx| {
            moved_files.clear();
            changed_photos.clear();
            let target_session = tx.session(target)?;
            target_folder = match target_session.item_id {
                Some(item_id) => tx.item(item_id)?.folder_key,
                None => "_Unassigned".into(),
            };
            target_name = if let Some(item_id) = target_session.item_id {
                let item = tx.item(item_id)?;
                if item.title.is_empty() {
                    item.sku
                } else {
                    item.title
                }
            } else {
                "Unassigned".into()
            };
            if target_session.published_at.is_some()
                && target_session.change_seq <= target_session.published_seq
            {
                return Err(Error::TargetPublished);
            }
            let unique: BTreeSet<i64> = ids.iter().copied().collect();
            let mut photos: Vec<Photo> = unique
                .iter()
                .map(|id| tx.photo(*id))
                .collect::<Result<_>>()?;
            if photos.iter().any(|photo| photo.session_id == target) {
                return Err(Error::TargetHasRaw);
            }
            let existing = tx.photos(target)?;
            if photos.iter().any(|photo| {
                existing
                    .iter()
                    .any(|other| other.raw_path == photo.raw_path)
            }) || photos.iter().enumerate().any(|(index, photo)| {
                photos[..index]
                    .iter()
                    .any(|other| other.raw_path == photo.raw_path)
            }) {
                return Err(Error::TargetHasRaw);
            }
            photos.sort_by_key(|photo| {
                (
                    group_rank(photo.classification),
                    photo.position.unwrap_or(i64::MAX),
                    photo.captured_at.unwrap_or(i64::MAX),
                    photo.id,
                )
            });
            for photo in &photos {
                moved_files.push((
                    photo.session_id,
                    crate::services::arrival::file_name(&photo.raw_path),
                    crate::services::arrival::folder_name(&photo.raw_path),
                ));
                if !copy && photo.classification.is_ordered() {
                    let old = group(tx, photo.session_id, photo.classification)?;
                    tx.set_positions(&position::remove_at(&old, photo.id))?;
                }
                let next = if photo.classification.is_ordered() {
                    Some(group(tx, target, photo.classification)?.len() as i64 + 1)
                } else {
                    None
                };
                if copy {
                    let id = tx.copy_photo(photo, target, next)?;
                    changed_photos.push((id, target));
                    copies
                        .entry(photo.session_id)
                        .or_default()
                        .push((photo.id, id));
                } else {
                    tx.set_photo_review(photo.id, target, photo.classification, next)?;
                    changed_photos.push((photo.id, target));
                    tx.photo_place_delete(photo.id)?;
                    touched.insert(photo.session_id);
                    steps.entry(photo.session_id).or_default().push(PhotoPlace {
                        photo_id: photo.id,
                        before: photo_place(photo),
                        after: Place {
                            classification: photo.classification,
                            position: next,
                        },
                    });
                }
            }
            if !photos.is_empty() {
                touched.insert(target);
            }
            for session_id in &touched {
                tx.touch_session(*session_id, at)?;
            }
            seq_after = tx.session(target)?.change_seq;
            Ok(())
        })?;
        // A10 (owner 2026-11-18): one line per moved or copied photo. This step re-points the rows;
        // A21: the app's move then moves the RAW and Capture One's files into the target's
        // folder (`psm_integrations::photo_files`, logged `photos.files_moved`); a copy shares the
        // RAW, which stays.
        for (source, file, folder) in &moved_files {
            tracing::info!(
                target: "psm::capture_one",
                "photos.{} file={} from_session={} from_folder={} to_session={} to_folder={} files={}",
                if copy { "copied" } else { "moved" },
                file,
                source,
                folder,
                target,
                target_folder,
                if copy { "shared" } else { "follow" },
            );
        }
        if copy {
            for (source, pairs) in copies {
                self.history.record(
                    source,
                    Step {
                        label: label_transfer(true, pairs.len(), &target_name),
                        kind: StepKind::Copy {
                            source,
                            target,
                            seq_after,
                            copies: pairs,
                        },
                    },
                );
            }
        } else {
            for (source, photos) in steps {
                self.history.record(
                    source,
                    Step {
                        label: label_transfer(false, photos.len(), &target_name),
                        kind: StepKind::Move {
                            source,
                            target,
                            photos,
                        },
                    },
                );
            }
        }
        let touched: Vec<_> = touched.into_iter().collect();
        if !touched.is_empty() {
            self.history.announce(&changed_photos, &touched);
        }
        self.result(&touched, None)
    }
}

fn group_rank(classification: Classification) -> u8 {
    match classification {
        Classification::Main => 0,
        Classification::Condition => 1,
        Classification::Unreviewed => 2,
        Classification::Deleted => 3,
    }
}

fn group(
    tx: &dyn StoreTx,
    session_id: i64,
    classification: Classification,
) -> Result<position::Positioned> {
    Ok(tx
        .photos(session_id)?
        .into_iter()
        .filter(|photo| photo.classification == classification)
        .filter_map(|photo| photo.position.map(|position| (photo.id, position)))
        .collect())
}
