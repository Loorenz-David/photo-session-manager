use crate::domain::{
    attribute::Value as AttributeValue,
    composition::{CompositionState, CompositionValues, Origin},
    photo::Classification,
    session::SessionDto,
};
use crate::events::AppEvent;
use crate::mechanisms::{composition::validate, position};
use crate::ports::{
    clock::Clock,
    events::EventSink,
    store::{Store, StoreTx},
};
use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

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
    Attribute {
        session_id: i64,
        item_id: i64,
        fields: Vec<FieldChange>,
    },
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
        {
            let mut all = self.store.lock().unwrap();
            let list = all.entry(session_id).or_default();
            let merged = if list.redo.is_empty() {
                list.undo
                    .back_mut()
                    .is_some_and(|last| coalesce_attribute(last, &step))
            } else {
                false
            };
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
    pub fn state(&self, session_id: i64) -> HistoryState {
        let all = self.store.lock().unwrap();
        all.get(&session_id)
            .map(|list| HistoryState {
                undo: list.undo.back().map(|s| s.label.clone()),
                redo: list.redo.last().map(|s| s.label.clone()),
            })
            .unwrap_or_default()
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

fn coalesce_attribute(last: &mut Step, next: &Step) -> bool {
    if !last.label.starts_with("Edit ") || !next.label.starts_with("Edit ") {
        return false;
    }
    let (
        StepKind::Attribute {
            item_id: old_item,
            fields: old,
            ..
        },
        StepKind::Attribute {
            item_id: new_item,
            fields: new,
            ..
        },
    ) = (&mut last.kind, &next.kind)
    else {
        return false;
    };
    if old_item != new_item || old.len() != 1 || new.len() != 1 || old[0].key != new[0].key {
        return false;
    }
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
        let at = self.clock.now_ms();
        let result = self.store.transaction(&mut |tx| {
            apply(
                tx,
                &mut step,
                direction,
                at,
                &mut touched,
                &mut active,
                &mut changed_photos,
            )
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
fn apply(
    tx: &mut dyn StoreTx,
    step: &mut Step,
    direction: Direction,
    at: i64,
    touched: &mut Vec<i64>,
    active: &mut Option<i64>,
    changed_photos: &mut Vec<i64>,
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
            let next = position::reorder(
                &old,
                photo_id,
                if direction == Direction::Undone {
                    from
                } else {
                    to
                },
            );
            if old != next {
                tx.set_positions(&next)?;
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
        StepKind::Attribute {
            session_id,
            item_id,
            fields,
        } => {
            tx.item(item_id).map_err(|e| {
                if matches!(e, Error::NotFound(_)) {
                    cannot(step, direction, "that product is gone")
                } else {
                    e
                }
            })?;
            let mut changed = false;
            for field in fields {
                let target = if direction == Direction::Undone {
                    &field.before
                } else {
                    &field.after
                };
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
            touched.push(session_id);
        }
    }
    Ok(())
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
