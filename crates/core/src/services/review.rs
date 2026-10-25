use crate::domain::{
    photo::{Classification, Photo, UndoSlot},
    session::SessionDto,
};
use crate::mechanisms::position;
use crate::ports::{
    clock::Clock,
    store::{Store, StoreTx},
};
use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewResult {
    pub sessions: Vec<SessionDto>,
    pub active_photo_id: Option<i64>,
}

pub struct ReviewService<'a> {
    store: &'a dyn Store,
    clock: &'a dyn Clock,
}

impl<'a> ReviewService<'a> {
    pub fn new(store: &'a dyn Store, clock: &'a dyn Clock) -> Self {
        Self { store, clock }
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
        let mut session_id = 0;
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
            tx.enqueue_mirror_tag(&photo.raw_path, photo_id, at)?;
            tx.touch_session(session_id, at)
        })?;
        self.result(&[session_id], None)
    }

    pub fn reorder(&self, photo_id: i64, position: i64) -> Result<ReviewResult> {
        let mut session_id = 0;
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
            tx.touch_session(session_id, at)
        })?;
        self.result(&[session_id], None)
    }

    pub fn delete(&self, photo_id: i64) -> Result<ReviewResult> {
        let mut session_id = 0;
        let at = self.clock.now_ms();
        self.store.transaction(&mut |tx| {
            let photo = tx.photo(photo_id)?;
            session_id = photo.session_id;
            if photo.classification == Classification::Deleted {
                return Ok(());
            }
            let slot = UndoSlot {
                photo_id,
                prior_classification: photo.classification,
                prior_position: photo.position,
            };
            if photo.classification.is_ordered() {
                let old = group(tx, session_id, photo.classification)?;
                tx.set_positions(&position::remove_at(&old, photo_id))?;
            }
            tx.set_photo_review(photo_id, session_id, Classification::Deleted, None)?;
            tx.enqueue_mirror_tag(&photo.raw_path, photo_id, at)?;
            tx.set_undo_slot(session_id, Some(&slot))?;
            tx.touch_session(session_id, at)
        })?;
        self.result(&[session_id], None)
    }

    pub fn undo(&self, session_id: i64) -> Result<ReviewResult> {
        let mut active = None;
        let at = self.clock.now_ms();
        self.store.transaction(&mut |tx| {
            let session = tx.session(session_id)?;
            let slot: UndoSlot =
                serde_json::from_str(session.undo_slot.as_deref().ok_or(Error::NothingToUndo)?)
                    .map_err(|e| Error::InvalidData(e.to_string()))?;
            let photo = tx.photo(slot.photo_id)?;
            if photo.session_id != session_id || photo.classification != Classification::Deleted {
                return Err(Error::NothingToUndo);
            }
            let next = if slot.prior_classification.is_ordered() {
                let old = group(tx, session_id, slot.prior_classification)?;
                let updated =
                    position::undo_insert(&old, slot.photo_id, slot.prior_position.unwrap_or(1));
                tx.set_positions(&updated)?;
                updated
                    .iter()
                    .find(|(id, _)| *id == slot.photo_id)
                    .map(|(_, p)| *p)
            } else {
                None
            };
            tx.set_photo_review(slot.photo_id, session_id, slot.prior_classification, next)?;
            tx.enqueue_mirror_tag(&photo.raw_path, slot.photo_id, at)?;
            tx.set_undo_slot(session_id, None)?;
            tx.touch_session(session_id, at)?;
            active = Some(slot.photo_id);
            Ok(())
        })?;
        self.result(&[session_id], active)
    }

    pub fn move_photos(&self, ids: &[i64], target: i64) -> Result<ReviewResult> {
        self.transfer(ids, target, false)
    }

    pub fn copy_photos(&self, ids: &[i64], target: i64) -> Result<ReviewResult> {
        self.transfer(ids, target, true)
    }

    fn transfer(&self, ids: &[i64], target: i64, copy: bool) -> Result<ReviewResult> {
        let mut touched = BTreeSet::new();
        let at = self.clock.now_ms();
        self.store.transaction(&mut |tx| {
            let target_session = tx.session(target)?;
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
                    tx.copy_photo(photo, target, next)?;
                } else {
                    tx.set_photo_review(photo.id, target, photo.classification, next)?;
                    touched.insert(photo.session_id);
                }
            }
            if !photos.is_empty() {
                touched.insert(target);
            }
            for session_id in &touched {
                tx.touch_session(*session_id, at)?;
            }
            Ok(())
        })?;
        self.result(&touched.into_iter().collect::<Vec<_>>(), None)
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
