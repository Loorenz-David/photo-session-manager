use crate::domain::{
    attribute::Value as AttributeValue,
    composition::{Composition, CompositionState, CompositionValues, Origin},
    photo::Classification,
    research::{Feedback, FeedbackAction},
    session::SessionDto,
    template::PlaceRow,
};
use crate::events::AppEvent;
use crate::mechanisms::{
    composition::{default_rect, turned_frame, validate},
    position, template,
};
use crate::ports::{
    clock::Clock,
    events::EventSink,
    store::{Store, StoreTx},
};
use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, MutexGuard};

pub const HISTORY_LIMIT: usize = 25;
pub type HistoryStore = Arc<Mutex<HashMap<i64, SessionHistory>>>;

#[derive(Default)]
pub struct SessionHistory {
    pub undo: VecDeque<Step>,
    pub redo: Vec<Step>,
}

pub struct History {
    store: HistoryStore,
    events: Arc<dyn EventSink>,
    operation: Mutex<()>,
}

#[derive(Debug, Clone)]
pub struct Step {
    pub label: String,
    pub kind: StepKind,
}

#[derive(Debug, Clone)]
pub enum StepKind {
    Classify {
        session_id: i64,
        photos: Vec<PhotoPlace>,
    },
    Reorder {
        session_id: i64,
        photo_id: i64,
        from: i64,
        to: i64,
        /// The photo's place row before and after the drag, as (slot, placed by hand): undo and
        /// redo write these back, so an extra that had no slot has none again (MC-30).
        place_before: (Option<i64>, bool),
        place_after: (Option<i64>, bool),
    },
    Move {
        source: i64,
        target: i64,
        photos: Vec<PhotoPlace>,
    },
    Copy {
        source: i64,
        target: i64,
        seq_after: i64,
        copies: Vec<(i64, i64)>,
    },
    Composition {
        session_id: i64,
        photos: Vec<CompositionChange>,
    },
    /// A template action (drop onto a container, Back to template; T4 retake, shift): the whole
    /// state of every photo it changed, before and after (10G §6.5).
    Template {
        session_id: i64,
        photos: Vec<TemplateChange>,
    },
    /// "Keep as published" (owner correction A10, 2026-11-18): the photos that arrived after the
    /// publish are denied and the session's changes count as published, without republishing.
    /// Undo restores the photos and the changed state.
    KeepPublished {
        session_id: i64,
        photos: Vec<PhotoPlace>,
        /// `published_seq` before the step, and the one it set.
        seq_before: i64,
        seq_after: i64,
    },
    Attribute {
        session_id: i64,
        item_id: i64,
        fields: Vec<FieldChange>,
        /// The step's id: the `step_token` of the feedback rows it wrote (MC-A12).
        token: String,
        /// The feedback rows an undo deleted, re-inserted by the redo (R5·U1 P6).
        feedback: Vec<Feedback>,
        /// The publish resolutions this step recorded (key → `{value, action, at, step_token}`),
        /// recorded again by its redo (MC-A10, R5·U2 N7).
        resolutions: Vec<(String, serde_json::Value)>,
    },
}

impl Step {
    /// Whether undoing or redoing this step would change the photo `photo_id`.
    fn names_photo(&self, photo_id: i64) -> bool {
        match &self.kind {
            StepKind::Classify { photos, .. }
            | StepKind::Move { photos, .. }
            | StepKind::KeepPublished { photos, .. } => {
                photos.iter().any(|p| p.photo_id == photo_id)
            }
            StepKind::Reorder { photo_id: id, .. } => *id == photo_id,
            StepKind::Copy { copies, .. } => copies
                .iter()
                .any(|(source, copy)| *source == photo_id || *copy == photo_id),
            StepKind::Composition { photos, .. } => photos.iter().any(|p| p.photo_id == photo_id),
            StepKind::Template { photos, .. } => photos.iter().any(|p| p.photo_id == photo_id),
            StepKind::Attribute { .. } => false,
        }
    }
    /// Whether this step reads or writes the session `session_id` (A17).
    fn names_session(&self, session_id: i64) -> bool {
        match &self.kind {
            StepKind::Classify { session_id: id, .. }
            | StepKind::Reorder { session_id: id, .. }
            | StepKind::Composition { session_id: id, .. }
            | StepKind::Template { session_id: id, .. }
            | StepKind::KeepPublished { session_id: id, .. }
            | StepKind::Attribute { session_id: id, .. } => *id == session_id,
            StepKind::Move { source, target, .. } | StepKind::Copy { source, target, .. } => {
                *source == session_id || *target == session_id
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Place {
    pub classification: Classification,
    pub position: Option<i64>,
}
#[derive(Debug, Clone)]
pub struct PhotoPlace {
    pub photo_id: i64,
    pub before: Place,
    pub after: Place,
}
#[derive(Debug, Clone)]
pub struct CompositionChange {
    pub photo_id: i64,
    pub before: (CompositionValues, Origin),
    pub after: (CompositionValues, Origin),
}
/// One photo's state as a template step records it: place, `photo_places` row, composition
/// (values with origin, or kind/shape/origin while the frame is unknown) and denial.
#[derive(Debug, Clone, PartialEq)]
pub struct PhotoState {
    pub classification: Classification,
    pub position: Option<i64>,
    pub row: Option<PlaceRow>,
    pub composition: Composition,
    pub denied_at: Option<i64>,
}
#[derive(Debug, Clone)]
pub struct TemplateChange {
    pub photo_id: i64,
    pub before: PhotoState,
    pub after: PhotoState,
}
/// The photo's state now, inside the caller's transaction.
pub fn photo_state(tx: &dyn StoreTx, photo_id: i64) -> Result<PhotoState> {
    let photo = tx.photo(photo_id)?;
    Ok(PhotoState {
        classification: photo.classification,
        position: photo.position,
        row: tx.photo_place_row(photo_id)?,
        composition: tx.composition(photo_id)?,
        denied_at: tx.photo_denied_at(photo_id)?,
    })
}
#[derive(Debug, Clone)]
pub enum FieldState {
    Row(Option<AttributeValue>),
    Title(String),
}
#[derive(Debug, Clone)]
pub struct FieldChange {
    pub key: String,
    pub before: FieldState,
    pub after: FieldState,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoryState {
    pub undo: Option<String>,
    pub redo: Option<String>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    Undone,
    Redone,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryOutcome {
    pub label: String,
    pub direction: Direction,
    pub sessions: Vec<SessionDto>,
    pub active_photo_id: Option<i64>,
}

impl History {
    pub fn new(events: Arc<dyn EventSink>) -> Self {
        Self::with_store(Arc::new(Mutex::new(HashMap::new())), events)
    }
    pub fn with_store(store: HistoryStore, events: Arc<dyn EventSink>) -> Self {
        Self {
            store,
            events,
            operation: Mutex::new(()),
        }
    }
    pub fn store(&self) -> HistoryStore {
        self.store.clone()
    }
    pub fn record(&self, session_id: i64, step: Step) {
        let op = self.operation.lock().unwrap();
        let merge = self
            .last_step(session_id)
            .is_some_and(|last| coalesces(&last, &step));
        self.record_held(op, session_id, step, merge);
    }
    /// Takes the history's operation lock: no undo, redo or record runs until the guard is
    /// dropped. An attribute action takes it **before** its store transaction (order: operation →
    /// store; R5·U1 T1) and hands it to [`History::record_held`].
    pub(crate) fn lock(&self) -> MutexGuard<'_, ()> {
        self.operation.lock().unwrap()
    }
    /// The session's last undo step, when a new step could merge into it (the redo list is
    /// empty). Read under the operation lock, it stays the last step until the lock is dropped.
    pub(crate) fn last_step(&self, session_id: i64) -> Option<Step> {
        let all = self.store.lock().unwrap();
        let list = all.get(&session_id)?;
        if !list.redo.is_empty() {
            return None;
        }
        list.undo.back().cloned()
    }
    /// Records `step` under the held operation lock. `merge` is the caller's coalescing decision,
    /// taken once in the action's transaction (R5·U1 Q2); it is not re-decided here.
    pub(crate) fn record_held(
        &self,
        op: MutexGuard<'_, ()>,
        session_id: i64,
        step: Step,
        merge: bool,
    ) {
        {
            let mut all = self.store.lock().unwrap();
            let list = all.entry(session_id).or_default();
            let merged = merge
                && list.redo.is_empty()
                && list
                    .undo
                    .back_mut()
                    .is_some_and(|last| coalesce_attribute(last, &step));
            if !merged {
                list.undo.push_back(step);
            }
            if list.undo.len() > HISTORY_LIMIT {
                list.undo.pop_front();
            }
            list.redo.clear();
        }
        drop(op);
        self.emit(session_id);
    }
    /// P66 (changelog 46 (2)): runs `act` while no undo or redo can run; when it returns `true`
    /// (the denied photo's files were deleted), every undo and redo step that names the photo is
    /// dropped — it can no longer be restored.
    pub fn retire_photo(&self, photo_id: i64, act: impl FnOnce() -> Result<bool>) -> Result<bool> {
        let op = self.operation.lock().unwrap();
        let retired = act()?;
        let mut changed = Vec::new();
        if retired {
            let mut all = self.store.lock().unwrap();
            for (session_id, list) in all.iter_mut() {
                let before = list.undo.len() + list.redo.len();
                list.undo.retain(|step| !step.names_photo(photo_id));
                list.redo.retain(|step| !step.names_photo(photo_id));
                if list.undo.len() + list.redo.len() != before {
                    changed.push(*session_id);
                }
            }
        }
        drop(op);
        for session_id in changed {
            self.emit(session_id);
        }
        Ok(retired)
    }
    /// Owner correction A17 ("Delete session"): runs `act` while no undo or redo can run; when it
    /// returns `true` (the session was deleted), the session's own undo and redo lists go, and so
    /// does every step of another session that names it or one of `photo_ids` (a move or copy
    /// between the two): none of them can apply any more.
    pub fn retire_session(
        &self,
        session_id: i64,
        photo_ids: &[i64],
        act: impl FnOnce() -> Result<bool>,
    ) -> Result<bool> {
        let op = self.operation.lock().unwrap();
        let retired = act()?;
        let mut changed = Vec::new();
        if retired {
            let mut all = self.store.lock().unwrap();
            if all.remove(&session_id).is_some() {
                changed.push(session_id);
            }
            for (other, list) in all.iter_mut() {
                let before = list.undo.len() + list.redo.len();
                let keep = |step: &Step| {
                    !step.names_session(session_id)
                        && !photo_ids.iter().any(|id| step.names_photo(*id))
                };
                list.undo.retain(keep);
                list.redo.retain(keep);
                if list.undo.len() + list.redo.len() != before {
                    changed.push(*other);
                }
            }
        }
        drop(op);
        for session_id in changed {
            self.emit(session_id);
        }
        Ok(retired)
    }
    pub fn state(&self, session_id: i64) -> HistoryState {
        let all = self.store.lock().unwrap();
        all.get(&session_id)
            .map(|list| HistoryState {
                undo: list.undo.back().map(|s| s.label.clone()),
                redo: list.redo.last().map(|s| s.label.clone()),
            })
            .unwrap_or_default()
    }
    /// MC-A10: a held publish item was released by a committed action, undo or redo.
    pub(crate) fn emit_released(&self, released: crate::services::publish::Released) {
        crate::services::publish::emit_released(self.events.as_ref(), released);
    }
    fn emit(&self, session_id: i64) {
        let state = self.state(session_id);
        self.events.emit(AppEvent::HistoryChanged {
            session_id,
            undo: state.undo,
            redo: state.redo,
        });
    }
}

/// Whether an "Edit" of `key` on `item_id` merges into `last` (typing in one field is one step).
pub(crate) fn coalesces_into(last: &Step, item_id: i64, key: &str) -> bool {
    let StepKind::Attribute {
        item_id: old_item,
        fields,
        resolutions,
        ..
    } = &last.kind
    else {
        return false;
    };
    // MC-A10 (N1): a step that recorded a resolution is never merged into.
    resolutions.is_empty()
        && last.label.starts_with("Edit ")
        && *old_item == item_id
        && fields.len() == 1
        && fields[0].key == key
}
fn coalesces(last: &Step, next: &Step) -> bool {
    let StepKind::Attribute {
        item_id, fields, ..
    } = &next.kind
    else {
        return false;
    };
    next.label.starts_with("Edit ")
        && fields.len() == 1
        && coalesces_into(last, *item_id, &fields[0].key)
}
fn coalesce_attribute(last: &mut Step, next: &Step) -> bool {
    if !coalesces(last, next) {
        return false;
    }
    let (StepKind::Attribute { fields: old, .. }, StepKind::Attribute { fields: new, .. }) =
        (&mut last.kind, &next.kind)
    else {
        return false;
    };
    old[0].after = new[0].after.clone();
    true
}

pub struct HistoryService<'a> {
    store: &'a dyn Store,
    clock: &'a dyn Clock,
    history: &'a History,
}
impl<'a> HistoryService<'a> {
    pub fn new(store: &'a dyn Store, clock: &'a dyn Clock, history: &'a History) -> Self {
        Self {
            store,
            clock,
            history,
        }
    }
    pub fn state(&self, session_id: i64) -> HistoryState {
        self.history.state(session_id)
    }
    pub fn undo(&self, session_id: i64) -> Result<HistoryOutcome> {
        self.reverse(session_id, Direction::Undone)
    }
    pub fn redo(&self, session_id: i64) -> Result<HistoryOutcome> {
        self.reverse(session_id, Direction::Redone)
    }
    fn reverse(&self, session_id: i64, direction: Direction) -> Result<HistoryOutcome> {
        crate::services::retention::ensure_session_not_cleared(self.store, session_id)?;
        let op = self.history.operation.lock().unwrap();
        let mut step = {
            let all = self.history.store.lock().unwrap();
            let list = all.get(&session_id);
            let step = match direction {
                Direction::Undone => list.and_then(|l| l.undo.back()),
                Direction::Redone => list.and_then(|l| l.redo.last()),
            };
            step.cloned().ok_or(match direction {
                Direction::Undone => Error::NothingToUndo,
                Direction::Redone => Error::NothingToRedo,
            })?
        };
        let mut touched = Vec::new();
        let mut active = None;
        let mut changed_photos = Vec::new();
        let mut released = None;
        let at = self.clock.now_ms();
        let result = self.store.transaction(&mut |tx| {
            released = None;
            apply(
                tx,
                &mut step,
                direction,
                at,
                &mut touched,
                &mut active,
                &mut changed_photos,
                &mut released,
            )?;
            // P7: Main stays in container order in every templated session the step touched.
            let mut sessions = touched.clone();
            sessions.sort_unstable();
            sessions.dedup();
            for session in sessions {
                crate::services::template_placement::renumber_main(tx, session)?;
            }
            Ok(())
        });
        match &result {
            Ok(()) | Err(Error::StepCannotApply { .. }) => {
                let mut all = self.history.store.lock().unwrap();
                let list = all.get_mut(&session_id).unwrap();
                match direction {
                    Direction::Undone => {
                        list.undo.pop_back();
                        if result.is_ok() {
                            list.redo.push(step.clone());
                        }
                    }
                    Direction::Redone => {
                        list.redo.pop();
                        if result.is_ok() {
                            list.undo.push_back(step.clone());
                        }
                    }
                }
            }
            Err(_) => {}
        }
        drop(op);
        if result.is_ok() || matches!(result, Err(Error::StepCannotApply { .. })) {
            self.history.emit(session_id);
        }
        result?;
        if let Some(released) = released {
            self.history.emit_released(released);
        }
        if !changed_photos.is_empty() {
            for photo_id in changed_photos {
                self.history.events.emit(AppEvent::PhotoChanged {
                    photo_id,
                    session_id,
                });
            }
            self.history
                .events
                .emit(AppEvent::SessionChanged { session_id });
        }
        Ok(HistoryOutcome {
            label: step.label,
            direction,
            sessions: touched
                .into_iter()
                .map(|id| self.store.session(id))
                .collect::<Result<_>>()?,
            active_photo_id: active,
        })
    }
}

fn cannot(step: &Step, direction: Direction, reason: &str) -> Error {
    Error::StepCannotApply {
        direction: match direction {
            Direction::Undone => "undo",
            Direction::Redone => "redo",
        }
        .into(),
        label: step.label.clone(),
        reason: reason.into(),
    }
}
fn group(tx: &dyn StoreTx, session: i64, class: Classification) -> Result<position::Positioned> {
    Ok(tx
        .photos(session)?
        .into_iter()
        .filter(|p| p.classification == class)
        .filter_map(|p| p.position.map(|n| (p.id, n)))
        .collect())
}
fn place_of(p: &crate::domain::photo::Photo) -> Place {
    Place {
        classification: p.classification,
        position: p.position,
    }
}
pub fn photo_place(p: &crate::domain::photo::Photo) -> Place {
    place_of(p)
}
fn inserted(tx: &mut dyn StoreTx, session: i64, id: i64, place: Place) -> Result<Option<i64>> {
    if !place.classification.is_ordered() {
        return Ok(None);
    }
    let next = position::undo_insert(
        &group(tx, session, place.classification)?,
        id,
        place.position.unwrap_or(1),
    );
    tx.set_positions(&next)?;
    Ok(next.into_iter().find(|(p, _)| *p == id).map(|(_, n)| n))
}
fn check_photo(
    tx: &dyn StoreTx,
    id: i64,
    session: i64,
    class: Classification,
    step: &Step,
    direction: Direction,
    missing: &str,
) -> Result<crate::domain::photo::Photo> {
    let p = tx.photo(id).map_err(|e| {
        if matches!(e, Error::NotFound(_)) {
            cannot(step, direction, missing)
        } else {
            e
        }
    })?;
    if p.session_id != session || p.classification != class {
        return Err(cannot(step, direction, "that photo already changed"));
    }
    Ok(p)
}
fn published_clean(tx: &dyn StoreTx, session_id: i64) -> Result<bool> {
    let s = tx.session(session_id)?;
    Ok(s.published_at.is_some() && s.change_seq <= s.published_seq)
}
#[allow(clippy::too_many_arguments)]
fn apply(
    tx: &mut dyn StoreTx,
    step: &mut Step,
    direction: Direction,
    at: i64,
    touched: &mut Vec<i64>,
    active: &mut Option<i64>,
    changed_photos: &mut Vec<i64>,
    released: &mut Option<crate::services::publish::Released>,
) -> Result<()> {
    match step.kind.clone() {
        StepKind::Classify { session_id, photos } => {
            let undo = direction == Direction::Undone;
            for change in &photos {
                let expected = if undo { change.after } else { change.before };
                check_photo(
                    tx,
                    change.photo_id,
                    session_id,
                    expected.classification,
                    step,
                    direction,
                    "that photo is gone",
                )?;
            }
            let mut changed = false;
            for change in &photos {
                let current = if undo { change.after } else { change.before };
                let wanted = if undo { change.before } else { change.after };
                if current == wanted {
                    continue;
                }
                changed = true;
                if current.classification.is_ordered() {
                    let old = group(tx, session_id, current.classification)?;
                    tx.set_positions(&position::remove_at(&old, change.photo_id))?;
                }
                tx.set_photo_review(change.photo_id, session_id, wanted.classification, None)?;
            }
            let mut sorted = photos.clone();
            sorted.sort_by_key(|c| {
                let p = if undo { c.before } else { c.after };
                (
                    p.classification.as_str().to_owned(),
                    p.position.unwrap_or(i64::MAX),
                )
            });
            for change in &sorted {
                let wanted = if undo { change.before } else { change.after };
                if (if undo { change.after } else { change.before }) == wanted {
                    continue;
                }
                let n = inserted(tx, session_id, change.photo_id, wanted)?;
                tx.set_photo_review(change.photo_id, session_id, wanted.classification, n)?;
                if wanted.classification == Classification::Deleted {
                    // P66: a redone Deny starts a new undo window.
                    tx.set_photo_denied(change.photo_id, at)?;
                }
                let raw = tx.photo(change.photo_id)?.raw_path;
                tx.enqueue_mirror_tag(&raw, change.photo_id, at)?;
            }
            if changed {
                tx.touch_session(session_id, at)?;
            }
            touched.push(session_id);
            *active = photos.first().map(|p| p.photo_id);
        }
        StepKind::Reorder {
            session_id,
            photo_id,
            from,
            to,
            place_before,
            place_after,
        } => {
            let p = tx.photo(photo_id).map_err(|e| {
                if matches!(e, Error::NotFound(_)) {
                    cannot(step, direction, "that photo is gone")
                } else {
                    e
                }
            })?;
            if p.session_id != session_id || !p.classification.is_ordered() {
                return Err(cannot(step, direction, "that photo already changed"));
            }
            let old = group(tx, session_id, p.classification)?;
            let target = if direction == Direction::Undone {
                from
            } else {
                to
            };
            let next = position::reorder(&old, photo_id, target);
            if old != next {
                tx.set_positions(&next)?;
                // A drag in a templated Main wrote a slot and the by-hand mark; undo restores
                // the row as it was before the drag, redo writes what the drag wrote.
                let (slot, by_hand) = if direction == Direction::Undone {
                    place_before
                } else {
                    place_after
                };
                let row = tx.photo_place_row(photo_id)?;
                if row.is_some() || slot.is_some() || by_hand {
                    let mut row = row.unwrap_or(PlaceRow {
                        photo_id,
                        ..PlaceRow::default()
                    });
                    row.slot = slot;
                    row.place_by_hand = by_hand;
                    tx.photo_place_put(&row)?;
                }
                tx.touch_session(session_id, at)?;
            }
            touched.push(session_id);
        }
        StepKind::Move {
            source,
            target,
            photos,
        } => {
            let undo = direction == Direction::Undone;
            let (from, to) = if undo {
                (target, source)
            } else {
                (source, target)
            };
            if published_clean(tx, to)? {
                return Err(cannot(step, direction, "that session is published"));
            }
            let current: Vec<_> = photos
                .iter()
                .map(|c| {
                    let expected = if undo { c.after } else { c.before };
                    check_photo(
                        tx,
                        c.photo_id,
                        from,
                        expected.classification,
                        step,
                        direction,
                        "that photo is gone",
                    )
                })
                .collect::<Result<_>>()?;
            let existing = tx.photos(to)?;
            if current
                .iter()
                .any(|p| existing.iter().any(|e| e.raw_path == p.raw_path))
            {
                return Err(cannot(
                    step,
                    direction,
                    "that session already has this photo",
                ));
            }
            for p in &current {
                if p.classification.is_ordered() {
                    tx.set_positions(&position::remove_at(
                        &group(tx, from, p.classification)?,
                        p.id,
                    ))?;
                }
                let wanted = photos.iter().find(|c| c.photo_id == p.id).unwrap();
                let wanted = if undo { wanted.before } else { wanted.after };
                tx.set_photo_review(p.id, to, wanted.classification, None)?;
            }
            let mut sorted = photos.clone();
            sorted.sort_by_key(|c| {
                let p = if undo { c.before } else { c.after };
                (
                    p.classification.as_str().to_owned(),
                    p.position.unwrap_or(i64::MAX),
                )
            });
            for c in &sorted {
                let wanted = if undo { c.before } else { c.after };
                let n = inserted(tx, to, c.photo_id, wanted)?;
                tx.set_photo_review(c.photo_id, to, wanted.classification, n)?;
            }
            if !photos.is_empty() {
                tx.touch_session(source, at)?;
                tx.touch_session(target, at)?;
            }
            touched.extend([source, target]);
        }
        StepKind::Copy {
            source,
            target,
            seq_after,
            copies,
        } => {
            if direction == Direction::Undone {
                let s = tx.session(target)?;
                if (s.published_at.is_some() && s.published_seq >= seq_after)
                    || tx.publish_confirmed_since(target, seq_after)?
                {
                    return Err(cannot(step, direction, "that session is published"));
                }
                for (_, copy_id) in &copies {
                    let p = tx.photo(*copy_id).map_err(|e| {
                        if matches!(e, Error::NotFound(_)) {
                            cannot(step, direction, "the copy is gone")
                        } else {
                            e
                        }
                    })?;
                    if p.session_id != target {
                        return Err(cannot(step, direction, "that photo already changed"));
                    }
                }
                for (source_id, copy_id) in &copies {
                    let p = tx.photo(*copy_id)?;
                    if p.classification.is_ordered() {
                        tx.set_positions(&position::remove_at(
                            &group(tx, target, p.classification)?,
                            *copy_id,
                        ))?;
                    }
                    tx.delete_photo_row(*copy_id)?;
                    tx.enqueue_mirror_tag(&p.raw_path, *source_id, at)?;
                }
                if !copies.is_empty() {
                    tx.touch_session(target, at)?;
                }
            } else {
                if published_clean(tx, target)? {
                    return Err(cannot(step, direction, "that session is published"));
                }
                let existing = tx.photos(target)?;
                let sources: Vec<_> = copies
                    .iter()
                    .map(|(id, _)| {
                        tx.photo(*id).map_err(|e| {
                            if matches!(e, Error::NotFound(_)) {
                                cannot(step, direction, "that photo is gone")
                            } else {
                                e
                            }
                        })
                    })
                    .collect::<Result<_>>()?;
                if sources.iter().any(|p| p.session_id != source) {
                    return Err(cannot(step, direction, "that photo already changed"));
                }
                if sources
                    .iter()
                    .any(|p| existing.iter().any(|e| e.raw_path == p.raw_path))
                {
                    return Err(cannot(
                        step,
                        direction,
                        "that session already has this photo",
                    ));
                }
                let mut next_copies = Vec::new();
                for p in &sources {
                    let pos = if p.classification.is_ordered() {
                        Some(group(tx, target, p.classification)?.len() as i64 + 1)
                    } else {
                        None
                    };
                    next_copies.push((p.id, tx.copy_photo(p, target, pos)?));
                }
                if !sources.is_empty() {
                    tx.touch_session(target, at)?;
                }
                let seq = tx.session(target)?.change_seq;
                step.kind = StepKind::Copy {
                    source,
                    target,
                    seq_after: seq,
                    copies: next_copies,
                };
            }
            touched.push(target);
        }
        StepKind::Composition { session_id, photos } => {
            for change in photos {
                let p = tx.photo(change.photo_id).map_err(|e| {
                    if matches!(e, Error::NotFound(_)) {
                        cannot(step, direction, "that photo is gone")
                    } else {
                        e
                    }
                })?;
                if p.session_id != session_id {
                    return Err(cannot(step, direction, "that photo already changed"));
                }
                let current = tx.composition(change.photo_id).map_err(|e| {
                    if matches!(e, Error::NotFound(_)) {
                        cannot(step, direction, "that photo is gone")
                    } else {
                        e
                    }
                })?;
                if current.state == CompositionState::NeedsReview {
                    return Err(cannot(
                        step,
                        direction,
                        "that photo changed in Capture One — re-frame it first",
                    ));
                }
                let frame = current
                    .frame()
                    .ok_or_else(|| cannot(step, direction, "that photo has no frame"))?;
                let target = if direction == Direction::Undone {
                    &change.before
                } else {
                    &change.after
                };
                validate(&target.0, frame).map_err(|_| {
                    cannot(step, direction, "the framing no longer fits this photo")
                })?;
                let (_, changed) = tx.save_composition(change.photo_id, &target.0, target.1, at)?;
                if changed {
                    changed_photos.push(change.photo_id);
                }
            }
            touched.push(session_id);
        }
        StepKind::Template { session_id, photos } => {
            apply_template(tx, step, direction, at, session_id, &photos, changed_photos)?;
            touched.push(session_id);
            *active = photos.first().map(|p| p.photo_id);
        }
        StepKind::KeepPublished {
            session_id,
            photos,
            seq_before,
            seq_after,
        } => {
            let undo = direction == Direction::Undone;
            let current = tx.session(session_id)?;
            if current.published_seq != if undo { seq_after } else { seq_before } {
                return Err(cannot(step, direction, "the session was published again"));
            }
            let mut classify = Step {
                label: step.label.clone(),
                kind: StepKind::Classify {
                    session_id,
                    photos: photos.clone(),
                },
            };
            apply(
                tx,
                &mut classify,
                direction,
                at,
                touched,
                active,
                changed_photos,
                released,
            )?;
            if undo {
                tx.set_published_seq(session_id, seq_before)?;
            } else {
                let now_seq = tx.session(session_id)?.change_seq;
                tx.set_published_seq(session_id, now_seq)?;
                step.kind = StepKind::KeepPublished {
                    session_id,
                    photos,
                    seq_before,
                    seq_after: now_seq,
                };
            }
            touched.push(session_id);
        }
        StepKind::Attribute {
            session_id,
            item_id,
            fields,
            token,
            feedback,
            mut resolutions,
        } => {
            tx.item(item_id).map_err(|e| {
                if matches!(e, Error::NotFound(_)) {
                    cannot(step, direction, "that product is gone")
                } else {
                    e
                }
            })?;
            let undo = direction == Direction::Undone;
            // MC-A12 (R-IA21): every field must still hold the row the step left (undo) or found
            // (redo); otherwise the whole step cannot apply. Title fields restore as before.
            let stored = tx.attribute_values(item_id)?;
            for field in &fields {
                let expected = if undo { &field.after } else { &field.before };
                if let FieldState::Row(expected) = expected {
                    let current = stored.iter().find(|v| v.key == field.key);
                    if !same_row(current, expected.as_ref()) {
                        return Err(cannot(
                            step,
                            direction,
                            "that field was changed by research",
                        ));
                    }
                }
            }
            let mut changed = false;
            for field in fields {
                let target = if undo { &field.before } else { &field.after };
                match target {
                    FieldState::Row(row) => {
                        let current = tx
                            .attribute_values(item_id)?
                            .into_iter()
                            .find(|v| v.key == field.key);
                        if &current != row {
                            match row {
                                Some(value) => tx.put_attribute_value(value)?,
                                None => tx.delete_attribute_value(item_id, &field.key)?,
                            }
                            changed = true;
                        }
                    }
                    FieldState::Title(title) => {
                        if tx.item(item_id)?.title != *title {
                            tx.update_item_title(item_id, title, at)?;
                            changed = true;
                        }
                    }
                }
            }
            if changed {
                tx.set_item_activity(item_id, at)?;
            }
            let fields = match &step.kind {
                StepKind::Attribute { fields, .. } => fields.clone(),
                _ => unreachable!(),
            };
            // MC-A12: the step's feedback rows exist exactly while the step is applied.
            let kept = if undo {
                let rows = tx.feedback_by_step(&token)?;
                tx.delete_feedback_by_step(&token)?;
                // MC-A10: its resolutions are removed while the item is still held.
                crate::services::publish::unresolve_in(tx, item_id, &token)?;
                rows
            } else {
                for row in &feedback {
                    tx.insert_feedback(row)?;
                }
                // MC-A10 (changelog 6, 10): a redo that changes a key pending without an entry on
                // a held item is that key's resolution, recorded as the action it replays.
                for field in &fields {
                    let (FieldState::Row(before), FieldState::Row(after)) =
                        (&field.before, &field.after)
                    else {
                        continue;
                    };
                    if same_row(before.as_ref(), after.as_ref()) {
                        continue;
                    }
                    let (value, action) = match resolutions.iter().find(|(k, _)| *k == field.key) {
                        Some((_, entry)) => (
                            entry.get("value").filter(|v| !v.is_null()).cloned(),
                            entry
                                .get("action")
                                .and_then(serde_json::Value::as_str)
                                .and_then(FeedbackAction::parse),
                        ),
                        None => (
                            after
                                .as_ref()
                                .filter(|row| row.is_live() && row.valid)
                                .and_then(AttributeValue::json),
                            feedback
                                .iter()
                                .find(|f| f.key == field.key)
                                .map(|f| f.action),
                        ),
                    };
                    let action =
                        action.unwrap_or(if after.as_ref().is_some_and(AttributeValue::is_live) {
                            FeedbackAction::Edited
                        } else {
                            FeedbackAction::Cleared
                        });
                    if let Some((entry, release)) = crate::services::publish::resolve_pending_in(
                        tx, item_id, &field.key, value, action, &token, at,
                    )? {
                        resolutions.retain(|(k, _)| *k != field.key);
                        resolutions.push((field.key.clone(), entry));
                        *released = released.or(release);
                    }
                }
                Vec::new()
            };
            step.kind = StepKind::Attribute {
                session_id,
                item_id,
                fields,
                token,
                feedback: kept,
                resolutions,
            };
            touched.push(session_id);
        }
    }
    Ok(())
}

/// Undo writes every photo's `before`, redo its `after`. Each photo must still be where the step
/// left it (session, classification, slot); otherwise nothing is written. Main is renumbered by
/// the caller (P7); the step's photos in ordered groups go back to their recorded positions.
fn apply_template(
    tx: &mut dyn StoreTx,
    step: &Step,
    direction: Direction,
    at: i64,
    session_id: i64,
    photos: &[TemplateChange],
    changed_photos: &mut Vec<i64>,
) -> Result<()> {
    let undo = direction == Direction::Undone;
    fn pick(c: &TemplateChange, before: bool) -> &PhotoState {
        if before { &c.before } else { &c.after }
    }
    let wanted = |c| pick(c, undo);
    let left = |c| pick(c, !undo);
    for change in photos {
        let expected = left(change);
        check_photo(
            tx,
            change.photo_id,
            session_id,
            expected.classification,
            step,
            direction,
            "that photo is gone",
        )?;
        let slot = tx.photo_place_row(change.photo_id)?.and_then(|r| r.slot);
        if slot != expected.row.as_ref().and_then(|r| r.slot) {
            return Err(cannot(step, direction, "that photo already changed"));
        }
    }
    for change in photos {
        let current = tx.photo(change.photo_id)?;
        if current.classification.is_ordered() {
            let old = group(tx, session_id, current.classification)?;
            tx.set_positions(&position::remove_at(&old, change.photo_id))?;
        }
        tx.set_photo_review(
            change.photo_id,
            session_id,
            wanted(change).classification,
            None,
        )?;
    }
    let mut sorted: Vec<_> = photos.iter().collect();
    sorted.sort_by_key(|c| {
        let w = wanted(c);
        (
            w.classification.as_str().to_owned(),
            w.position.unwrap_or(i64::MAX),
        )
    });
    for change in sorted {
        let target = wanted(change);
        let place = Place {
            classification: target.classification,
            position: target.position,
        };
        let n = inserted(tx, session_id, change.photo_id, place)?;
        tx.set_photo_review(change.photo_id, session_id, target.classification, n)?;
        if let Some(denied_at) = target.denied_at {
            tx.set_photo_denied(change.photo_id, denied_at)?;
        }
        restore_composition(tx, change.photo_id, target, at)?;
        match &target.row {
            Some(row) => tx.photo_place_put(row)?,
            None => tx.photo_place_delete(change.photo_id)?,
        }
        if left(change).classification != target.classification {
            let raw = tx.photo(change.photo_id)?.raw_path;
            tx.enqueue_mirror_tag(&raw, change.photo_id, at)?;
        }
        changed_photos.push(change.photo_id);
    }
    if !photos.is_empty() {
        tx.touch_session(session_id, at)?;
    }
    Ok(())
}

/// Writes the recorded composition back when it differs from the stored one. A composition
/// recorded while the frame was unknown is written as kind, shape and origin; when the frame has
/// become known since, it gets that kind's default rect (zoomed by the row's pending zoom).
fn restore_composition(
    tx: &mut dyn StoreTx,
    photo_id: i64,
    target: &PhotoState,
    at: i64,
) -> Result<()> {
    let current = tx.composition(photo_id)?;
    let wanted = &target.composition;
    if let Some(values) = wanted.values() {
        if current.values().as_ref() != Some(&values) || current.origin != wanted.origin {
            tx.save_composition(photo_id, &values, wanted.origin, at)?;
        }
        return Ok(());
    }
    let pending = target.row.as_ref().and_then(|r| r.pending_zoom);
    match current.frame() {
        Some(frame) => {
            let rect = default_rect(
                wanted.kind,
                wanted.shape,
                wanted.orientation,
                turned_frame(frame, wanted.quarter_turns),
            );
            let values = CompositionValues {
                kind: wanted.kind,
                shape: wanted.shape,
                orientation: wanted.orientation,
                quarter_turns: wanted.quarter_turns,
                flip: wanted.flip,
                straighten: wanted.straighten,
                rect: pending.map_or(rect, |zoom| template::zoomed(rect, zoom)),
            };
            if current.values().as_ref() != Some(&values) || current.origin != wanted.origin {
                tx.save_composition(photo_id, &values, wanted.origin, at)?;
            }
        }
        None => {
            if (current.kind, current.shape, current.origin)
                != (wanted.kind, wanted.shape, wanted.origin)
            {
                tx.template_frame_unknown(
                    photo_id,
                    wanted.kind,
                    wanted.shape,
                    wanted.origin,
                    pending,
                    at,
                )?;
            }
        }
    }
    Ok(())
}

/// The same stored row, `updated_at` aside (MC-A12's comparison).
fn same_row(a: Option<&AttributeValue>, b: Option<&AttributeValue>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(a), Some(b)) => {
            AttributeValue {
                updated_at: 0,
                ..a.clone()
            } == AttributeValue {
                updated_at: 0,
                ..b.clone()
            }
        }
        _ => false,
    }
}

pub fn label_classify(before: Place, after: Place, count: usize) -> String {
    let action = match after.classification {
        Classification::Main => "Approve",
        Classification::Condition => "Condition",
        Classification::Deleted => "Delete",
        Classification::Unreviewed => "Unreview",
    };
    if count > 1 {
        return format!("{action} {count} photos");
    }
    let number = if matches!(
        after.classification,
        Classification::Deleted | Classification::Unreviewed
    ) {
        before.position
    } else {
        after.position
    };
    number
        .map(|n| format!("{action} {n:02}"))
        .unwrap_or_else(|| format!("{action} photo"))
}
pub fn label_reorder(from: i64, to: i64) -> String {
    format!("Reorder {from:02} → {to:02}")
}
pub fn label_transfer(copy: bool, count: usize, name: &str) -> String {
    let verb = if copy { "Copy" } else { "Move" };
    if count == 1 {
        format!("{verb} to {name}")
    } else {
        format!("{verb} {count} photos to {name}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Sink;
    impl EventSink for Sink {
        fn emit(&self, _: AppEvent) {}
    }
    fn step(n: i64) -> Step {
        Step {
            label: format!("Reorder {n:02} → 01"),
            kind: StepKind::Reorder {
                session_id: 1,
                photo_id: n,
                from: n,
                to: 1,
                place_before: (None, false),
                place_after: (None, false),
            },
        }
    }
    #[test]
    fn mc30_record_limit_and_state() {
        let h = History::new(Arc::new(Sink));
        for n in 1..=26 {
            h.record(1, step(n));
        }
        let store = h.store();
        let all = store.lock().unwrap();
        let list = all.get(&1).unwrap();
        assert_eq!(list.undo.len(), 25);
        assert_eq!(list.undo.front().unwrap().label, "Reorder 02 → 01");
        assert_eq!(list.undo.back().unwrap().label, "Reorder 26 → 01");
        drop(all);
        assert_eq!(h.state(1).undo.as_deref(), Some("Reorder 26 → 01"));
    }
    #[test]
    fn mc30_record_clears_redo() {
        let h = History::new(Arc::new(Sink));
        h.record(1, step(1));
        h.store
            .lock()
            .unwrap()
            .get_mut(&1)
            .unwrap()
            .redo
            .push(step(2));
        h.record(1, step(3));
        assert_eq!(
            h.state(1),
            HistoryState {
                undo: Some("Reorder 03 → 01".into()),
                redo: None
            }
        );
    }
    #[test]
    fn mc30_attribute_steps_coalesce() {
        fn edit(label: &str, key: &str, before: &str, after: &str) -> Step {
            Step {
                label: label.into(),
                kind: StepKind::Attribute {
                    session_id: 1,
                    item_id: 2,
                    fields: vec![FieldChange {
                        key: key.into(),
                        before: FieldState::Title(before.into()),
                        after: FieldState::Title(after.into()),
                    }],
                    token: String::new(),
                    feedback: Vec::new(),
                    resolutions: Vec::new(),
                },
            }
        }
        let mut first = edit("Edit Title", "product.title", "Oak", "O");
        let next = edit("Edit Title", "product.title", "O", "Oak table");
        assert!(coalesce_attribute(&mut first, &next));
        match first.kind {
            StepKind::Attribute { fields, .. } => {
                assert!(matches!(&fields[0].before,FieldState::Title(v) if v=="Oak"));
                assert!(matches!(&fields[0].after,FieldState::Title(v) if v=="Oak table"));
            }
            _ => unreachable!(),
        }
        assert!(!coalesce_attribute(
            &mut edit("Edit Color", "color", "Oak", "Oak"),
            &edit("Clear Color", "color", "Oak", "")
        ));
        assert!(!coalesce_attribute(
            &mut edit("Accept all suggestions", "color", "Oak", "Oak"),
            &edit("Edit Color", "color", "Oak", "Teak")
        ));
        assert!(!coalesce_attribute(
            &mut edit("Edit Color", "color", "Oak", "Oak"),
            &edit("Edit Condition report", "report", "Oak", "Teak")
        ));
    }
}
