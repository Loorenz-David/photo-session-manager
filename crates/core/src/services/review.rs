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
                },
            });
            Ok(())
        })?;
        if let Some(step) = step {
            self.history.record(session_id, step);
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
        let at = self.clock.now_ms();
        self.store.transaction(&mut |tx| {
            let target_session = tx.session(target)?;
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
                    copies
                        .entry(photo.session_id)
                        .or_default()
                        .push((photo.id, id));
                } else {
                    tx.set_photo_review(photo.id, target, photo.classification, next)?;
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
