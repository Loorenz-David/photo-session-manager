//! Placement on arrival (10G P1–P9). Every read and write for an arrival shares one store transaction.
use crate::{
    Error, Result,
    domain::{
        composition::{CompositionKind, CompositionValues, Flip, Orientation, Origin},
        photo::{Classification, Photo},
        template::{Container, PlaceRow, Presets, Template, TemplatePhotoView, TemplateView},
    },
    events::AppEvent,
    mechanisms::{
        capture_ledger::{seq_of, stem_key},
        composition::{default_rect, turned_frame},
        position,
        shot_order::{self, Entry, Shot},
        template::{self, Expected},
    },
    ports::{
        clock::{Clock, SystemClock},
        events::EventSink,
        store::{Store, StoreTx},
    },
    services::{
        history::{History, PhotoState, Step, StepKind, TemplateChange, photo_state},
        review::ReviewResult,
    },
};
use std::collections::HashMap;

pub fn place_arrival(
    store: &dyn Store,
    clock: &dyn Clock,
    events: &dyn EventSink,
    photo_id: i64,
) -> Result<()> {
    let now = clock.now_ms();
    let mut changed = Vec::new();
    let mut session_id = 0;
    let mut templated = false;
    store.transaction(&mut |tx| {
        let arrived = tx.photo(photo_id)?;
        session_id = arrived.session_id;
        let Some(template) = template_of(tx, session_id)? else {
            return Ok(());
        };
        templated = true;
        if tx.photo_place_row(photo_id)?.is_none() {
            tx.photo_place_put(&PlaceRow {
                photo_id,
                ..PlaceRow::default()
            })?;
        }
        changed = place_template_photos(tx, session_id, &template, now, Some(photo_id))?;
        tx.touch_session(session_id, now)?;
        Ok(())
    })?;
    if templated {
        for id in changed {
            events.emit(AppEvent::PhotoChanged {
                photo_id: id,
                session_id,
            });
        }
        events.emit(AppEvent::SessionChanged { session_id });
    }
    Ok(())
}

/// P6–P8: every photo of the session that has a row with `place_by_hand = 0` and is not Deleted
/// goes to its expected place (P5) with that place's composition when its place changed (always
/// for `arrived`, the photo that just arrived); then Main and Condition are ordered (P7). Returns
/// the photos (other than `arrived`) whose place, classification or position changed.
fn place_template_photos(
    tx: &mut dyn StoreTx,
    session_id: i64,
    template: &Template,
    now: i64,
    arrived: Option<i64>,
) -> Result<Vec<i64>> {
    let mut changed = Vec::new();
    let before = tx.photos(session_id)?;
    let indexes = shot_indexes(tx, session_id, now)?;
    let mut condition_order = Vec::new();
    for photo in &before {
        let Some(mut row) = tx.photo_place_row(photo.id)? else {
            continue;
        };
        if row.place_by_hand || photo.classification == Classification::Deleted {
            continue;
        }
        let Some(&index) = indexes.get(&photo.id) else {
            continue;
        };
        let (class, slot, presets) = expected_place(index, &template.containers);
        let place_changed = row.slot != slot || photo.classification != class;
        if place_changed && Some(photo.id) != arrived && !changed.contains(&photo.id) {
            changed.push(photo.id);
        }
        row.slot = slot;
        tx.photo_place_put(&row)?;
        if class == Classification::Condition {
            condition_order.push((index, photo.id));
        }
        if photo.classification != class {
            tx.set_photo_review(photo.id, session_id, class, None)?;
            tx.enqueue_mirror_tag(&photo.raw_path, photo.id, now)?;
        }
        if place_changed || Some(photo.id) == arrived {
            apply_frame(tx, photo.id, presets, row.frame_by_hand, now)?;
        }
    }
    // Main's stored order is the final container order; photos with no slot follow it.
    let mut after = tx.photos(session_id)?;
    let main: Vec<_> = after
        .iter()
        .filter(|p| p.classification == Classification::Main)
        .collect();
    let mut main_keys = HashMap::new();
    for p in &main {
        main_keys.insert(p.id, tx.photo_place_row(p.id)?);
    }
    tx.set_positions(&main_positions(&main, &main_keys))?;
    let mut condition: Vec<_> = after
        .iter()
        .filter(|p| p.classification == Classification::Condition)
        .collect();
    condition.sort_by_key(|p| (p.position.unwrap_or(i64::MAX), p.id));
    condition_order.sort_by_key(|(index, id)| (*index, *id));
    let placed_ids: std::collections::HashSet<i64> =
        condition_order.iter().map(|(_, id)| *id).collect();
    let mut template_ids = condition_order.into_iter().map(|(_, id)| id);
    tx.set_positions(
        &condition
            .iter()
            .enumerate()
            .map(|(i, p)| {
                (
                    if placed_ids.contains(&p.id) {
                        template_ids.next().expect("template condition")
                    } else {
                        p.id
                    },
                    i as i64 + 1,
                )
            })
            .collect::<Vec<_>>(),
    )?;
    after = tx.photos(session_id)?;
    for p in &after {
        if Some(p.id) == arrived {
            continue;
        }
        if let Some(old) = before.iter().find(|b| b.id == p.id)
            && (old.classification != p.classification || old.position != p.position)
            && !changed.contains(&p.id)
        {
            changed.push(p.id);
        }
    }
    Ok(changed)
}

fn apply_frame(
    tx: &mut dyn StoreTx,
    photo_id: i64,
    presets: Option<&Presets>,
    frame_by_hand: bool,
    now: i64,
) -> Result<()> {
    if frame_by_hand {
        return Ok(());
    }
    let current = tx.composition(photo_id)?;
    let active = presets.is_some_and(has_presets);
    if !active && current.origin != Origin::User {
        return Ok(());
    }
    let kind = if presets.is_some_and(|p| p.split) {
        CompositionKind::Split
    } else {
        CompositionKind::Normal
    };
    let shape = if active {
        presets.and_then(|p| p.shape).unwrap_or(current.shape)
    } else {
        tx.default_shape()
    };
    let origin = if active {
        Origin::User
    } else {
        Origin::Default
    };
    let zoom = presets.map_or(0, |p| p.zoom);
    if let Some(frame) = current.frame() {
        let values = if active {
            let current_values = current.values().unwrap_or_else(|| CompositionValues {
                kind: current.kind,
                shape: current.shape,
                orientation: current.orientation,
                quarter_turns: current.quarter_turns,
                flip: current.flip,
                straighten: current.straighten,
                rect: current.rect.unwrap_or(default_rect(
                    current.kind,
                    current.shape,
                    current.orientation,
                    turned_frame(frame, current.quarter_turns),
                )),
            });
            template::preset_values(presets.expect("active preset"), &current_values, frame)
        } else {
            CompositionValues {
                kind,
                shape,
                orientation: Orientation::Named,
                quarter_turns: 0,
                flip: Flip::None,
                straighten: 0.0,
                rect: default_rect(kind, shape, Orientation::Named, frame),
            }
        };
        tx.save_composition(photo_id, &values, origin, now)?;
    } else {
        tx.template_frame_unknown(photo_id, kind, shape, origin, active.then_some(zoom), now)?;
    }
    Ok(())
}

/// The template of a templated session (P1): one exists for the item's type and has containers.
fn template_of(tx: &dyn StoreTx, session_id: i64) -> Result<Option<Template>> {
    Ok(tx
        .template_for_session(session_id)?
        .filter(|t| !t.containers.is_empty()))
}

/// P2–P4: the 1-based shot index of every shot of the session (holds counted), as placement
/// reads it at `now`.
fn shot_indexes(tx: &dyn StoreTx, session_id: i64, now: i64) -> Result<HashMap<i64, usize>> {
    shot_indexes_without(tx, session_id, now, &std::collections::HashSet::new())
}

/// The same indexes with the shots of `skipped` photos left out (as if they were released).
fn shot_indexes_without(
    tx: &dyn StoreTx,
    session_id: i64,
    now: i64,
    skipped: &std::collections::HashSet<i64>,
) -> Result<HashMap<i64, usize>> {
    let shots = tx
        .shot_keys(session_id)?
        .into_iter()
        .filter(|s| !s.released && !skipped.contains(&s.photo_id))
        .map(|s| Shot {
            photo_id: s.photo_id,
            at: s.at,
            counter: s.name.as_deref().and_then(|name| seq_of(&stem_key(name))),
        })
        .collect();
    Ok(shot_order::order(shots, tx.hold_times(session_id, now)?)
        .iter()
        .enumerate()
        .filter_map(|(i, e)| match e {
            Entry::Shot(s) => Some((s.photo_id, i + 1)),
            Entry::Hold { .. } => None,
        })
        .collect())
}

/// P5: classification, slot and presets of the place of shot `index`.
fn expected_place(
    index: usize,
    containers: &[Container],
) -> (Classification, Option<i64>, Option<&Presets>) {
    match template::expected(index, containers) {
        Expected::Main(c) => (Classification::Main, Some(c.final_no), Some(&c.presets)),
        Expected::Condition => (Classification::Condition, None, None),
    }
}

fn has_presets(presets: &Presets) -> bool {
    presets.split || presets.shape.is_some() || presets.zoom > 0
}

/// A photo's place as the template compares it: its classification, and its slot when Main.
fn place_of(photo: &Photo, row: Option<&PlaceRow>) -> (Classification, Option<i64>) {
    (
        photo.classification,
        if photo.classification == Classification::Main {
            row.and_then(|r| r.slot)
        } else {
            None
        },
    )
}

/// P9: whether the photo differs from the template. A photo without a shot index (released)
/// never differs.
fn differs(
    photo: &Photo,
    row: Option<&PlaceRow>,
    indexes: &HashMap<i64, usize>,
    containers: &[Container],
) -> bool {
    if photo.classification == Classification::Deleted {
        return false;
    }
    let Some(&index) = indexes.get(&photo.id) else {
        return false;
    };
    let (class, slot, presets) = expected_place(index, containers);
    let here = place_of(photo, row);
    let by_hand = row.is_none_or(|r| r.place_by_hand);
    (by_hand && here != (class, slot))
        || (row.is_some_and(|r| r.frame_by_hand)
            && class == Classification::Main
            && here == (class, slot)
            && presets.is_some_and(has_presets))
}

/// The session's template view, with "differs" read at the system clock's now.
pub fn session_view(store: &dyn Store, session_id: i64) -> Result<TemplateView> {
    session_view_at(store, SystemClock.now_ms(), session_id)
}

/// The session's template view; "differs" uses the shot order placement would use at `now`.
pub fn session_view_at(store: &dyn Store, now: i64, session_id: i64) -> Result<TemplateView> {
    let mut result = TemplateView {
        templated: false,
        template_id: None,
        containers: Vec::new(),
        photos: Vec::new(),
        differs: 0,
        can_shift_back: false,
    };
    store.transaction(&mut |tx| {
        result.photos.clear();
        result.differs = 0;
        let Some(template) = template_of(tx, session_id)? else {
            return Ok(());
        };
        let indexes = shot_indexes(tx, session_id, now)?;
        result.templated = true;
        result.template_id = Some(template.id);
        for photo in tx.photos(session_id)? {
            if photo.classification == Classification::Deleted {
                continue;
            }
            let row = tx.photo_place_row(photo.id)?;
            let differs = differs(&photo, row.as_ref(), &indexes, &template.containers);
            result.differs += i64::from(differs);
            result.photos.push(TemplatePhotoView {
                photo_id: photo.id,
                slot: place_of(&photo, row.as_ref()).1,
                by_hand: row.as_ref().is_none_or(|r| r.place_by_hand),
                differs,
            });
        }
        result.can_shift_back = can_shift_back(tx, session_id, &template, now, &indexes)?;
        result.containers = template.containers;
        Ok(())
    })?;
    Ok(result)
}

/// 10G §6.4: the button is offered only when pressing it would change something. Releasing the
/// session's unreleased Deleted photos must change the expected place (classification or slot,
/// P5) of at least one photo that is not Deleted and was placed by the template (a row with
/// `place_by_hand = 0`). A denied photo with nothing later, with only hand-placed photos later, or
/// beyond the last container (later photos stay Condition) leaves nothing to shift.
fn can_shift_back(
    tx: &dyn StoreTx,
    session_id: i64,
    template: &Template,
    now: i64,
    indexes: &HashMap<i64, usize>,
) -> Result<bool> {
    let photos = tx.photos(session_id)?;
    let denied: std::collections::HashSet<i64> = photos
        .iter()
        .filter(|p| p.classification == Classification::Deleted)
        .map(|p| p.id)
        .collect();
    if !denied.iter().any(|id| indexes.contains_key(id)) {
        return Ok(false);
    }
    let released = shot_indexes_without(tx, session_id, now, &denied)?;
    for p in photos
        .iter()
        .filter(|p| p.classification != Classification::Deleted)
    {
        if !tx.photo_place_row(p.id)?.is_some_and(|r| !r.place_by_hand) {
            continue;
        }
        let (Some(&now_index), Some(&then_index)) = (indexes.get(&p.id), released.get(&p.id))
        else {
            continue;
        };
        let (class_now, slot_now, _) = expected_place(now_index, &template.containers);
        let (class_then, slot_then, _) = expected_place(then_index, &template.containers);
        if (class_now, slot_now) != (class_then, slot_then) {
            return Ok(true);
        }
    }
    Ok(false)
}

/// 10G §6.4 Shift the rest back (P10): every Deleted, not released photo of the session is
/// released (it leaves the shot count), then the template-placed photos go to their expected
/// places with those places' compositions (P6, P8) and the orders are renumbered (P7). Not
/// templated → `no_template`; nothing to shift → `nothing_to_shift`. One transaction, one undo
/// step "Shift the rest back" naming every photo whose place, row or composition changed.
pub fn shift_back(h: &HandAction<'_>, session_id: i64) -> Result<ReviewResult> {
    crate::services::retention::ensure_session_not_cleared(h.store, session_id)?;
    let at = h.clock.now_ms();
    let mut step = None;
    h.store.transaction(&mut |tx| {
        step = None;
        let template = template_of(tx, session_id)?.ok_or(Error::NoTemplate)?;
        let indexes = shot_indexes(tx, session_id, at)?;
        if !can_shift_back(tx, session_id, &template, at, &indexes)? {
            return Err(Error::NothingToShift);
        }
        let photos = tx.photos(session_id)?;
        let ids: Vec<i64> = photos.iter().map(|p| p.id).collect();
        let before = snapshot(tx, &ids)?;
        for p in photos
            .iter()
            .filter(|p| p.classification == Classification::Deleted)
        {
            let row = tx.photo_place_row(p.id)?.unwrap_or(PlaceRow {
                photo_id: p.id,
                ..PlaceRow::default()
            });
            if !row.released {
                tx.photo_place_put(&PlaceRow {
                    released: true,
                    ..row
                })?;
            }
        }
        place_template_photos(tx, session_id, &template, at, None)?;
        tx.touch_session(session_id, at)?;
        let after = snapshot(tx, &ids)?;
        let mut changed = (Vec::new(), Vec::new(), Vec::new());
        for ((id, b), a) in ids.iter().zip(before).zip(after) {
            if b != a {
                changed.0.push(*id);
                changed.1.push(b);
                changed.2.push(a);
            }
        }
        step = Some(template_step(
            "Shift the rest back".into(),
            session_id,
            &changed.0,
            changed.1,
            changed.2,
        ));
        Ok(())
    })?;
    h.finish(session_id, step)
}

/// P7 for Main after any change in a templated session; nothing for an untemplated one.
pub fn renumber_main(tx: &mut dyn StoreTx, session_id: i64) -> Result<()> {
    if template_of(tx, session_id)?.is_none() {
        return Ok(());
    }
    let main: Vec<_> = tx
        .photos(session_id)?
        .into_iter()
        .filter(|p| p.classification == Classification::Main)
        .collect();
    let mut rows = HashMap::new();
    for photo in &main {
        rows.insert(photo.id, tx.photo_place_row(photo.id)?);
    }
    tx.set_positions(&main_positions(&main.iter().collect::<Vec<_>>(), &rows))
}

/// P7 for Main: positions stay dense (1…m), sorted by (has no slot, slot, previous position).
/// Photos that share a slot keep the order they had, so a drag (which writes the dragged
/// photo's position and slot first) keeps the dragged order, and a photo returned to its
/// container by undo or Back to template takes that container's place.
fn main_positions(main: &[&Photo], rows: &HashMap<i64, Option<PlaceRow>>) -> Vec<(i64, i64)> {
    let slot = |p: &Photo| {
        rows.get(&p.id)
            .and_then(Option::as_ref)
            .and_then(|r| r.slot)
    };
    let mut sorted: Vec<_> = main.to_vec();
    sorted.sort_by_key(|p| {
        (
            slot(p).is_none(),
            slot(p).unwrap_or(i64::MAX),
            p.position.unwrap_or(i64::MAX),
            p.id,
        )
    });
    sorted
        .into_iter()
        .enumerate()
        .map(|(i, p)| (p.id, i as i64 + 1))
        .collect()
}

/// The dependencies of a person's template action (drop onto a container, Back to template).
pub struct HandAction<'a> {
    pub store: &'a dyn Store,
    pub clock: &'a dyn Clock,
    pub events: &'a dyn EventSink,
    pub history: &'a History,
}

impl HandAction<'_> {
    /// After the commit: `photo_changed` per changed photo, `session_changed`, the step recorded.
    fn finish(&self, session_id: i64, step: Option<Step>) -> Result<ReviewResult> {
        if let Some(step) = step {
            if let StepKind::Template { photos, .. } = &step.kind {
                for change in photos {
                    self.events.emit(AppEvent::PhotoChanged {
                        photo_id: change.photo_id,
                        session_id,
                    });
                }
            }
            self.events.emit(AppEvent::SessionChanged { session_id });
            self.history.record(session_id, step);
        }
        Ok(ReviewResult {
            sessions: vec![self.store.session(session_id)?],
            active_photo_id: None,
        })
    }
}

fn snapshot(tx: &dyn StoreTx, ids: &[i64]) -> Result<Vec<PhotoState>> {
    ids.iter().map(|id| photo_state(tx, *id)).collect()
}

fn template_step(
    label: String,
    session_id: i64,
    ids: &[i64],
    before: Vec<PhotoState>,
    after: Vec<PhotoState>,
) -> Step {
    Step {
        label,
        kind: StepKind::Template {
            session_id,
            photos: ids
                .iter()
                .zip(before.into_iter().zip(after))
                .map(|(id, (before, after))| TemplateChange {
                    photo_id: *id,
                    before,
                    after,
                })
                .collect(),
        },
    }
}

/// The photo is placed by hand in `slot` (its row is created when missing).
fn put_by_hand(tx: &mut dyn StoreTx, photo_id: i64, slot: Option<i64>) -> Result<()> {
    let mut row = tx.photo_place_row(photo_id)?.unwrap_or(PlaceRow {
        photo_id,
        ..PlaceRow::default()
    });
    row.place_by_hand = true;
    row.slot = slot;
    tx.photo_place_put(&row)
}

/// Moves the photo to `class` at the end of that group (Condition stays dense); no change when
/// it is already there.
fn reclassify(tx: &mut dyn StoreTx, photo: &Photo, class: Classification, at: i64) -> Result<()> {
    if photo.classification == class {
        return Ok(());
    }
    let group = |tx: &dyn StoreTx, c: Classification| -> Result<Vec<(i64, i64)>> {
        Ok(tx
            .photos(photo.session_id)?
            .into_iter()
            .filter(|p| p.classification == c)
            .filter_map(|p| p.position.map(|n| (p.id, n)))
            .collect())
    };
    if photo.classification.is_ordered() {
        let old = group(tx, photo.classification)?;
        tx.set_positions(&position::remove_at(&old, photo.id))?;
    }
    let next = if class.is_ordered() {
        Some(group(tx, class)?.len() as i64 + 1)
    } else {
        None
    };
    tx.set_photo_review(photo.id, photo.session_id, class, next)?;
    tx.enqueue_mirror_tag(&photo.raw_path, photo.id, at)
}

/// 10G §6 rules 1, 3 and 5: the person puts a photo into container `n` (its final number). An
/// empty container is filled; a photo that has a container swaps with the one in `n` (one undo
/// step "Move to container <n>"; compositions are not touched). A photo without a container
/// dropped onto a filled one is a retake (P10): the photo that was there is denied and released,
/// the dropped photo takes its container, its key and the container's presets (P8); one undo
/// step "Retake container <n>". One transaction.
pub fn drop_on_container(h: &HandAction<'_>, photo_id: i64, n: i64) -> Result<ReviewResult> {
    crate::services::retention::ensure_photo_not_cleared(h.store, photo_id)?;
    let at = h.clock.now_ms();
    let mut session_id = 0;
    let mut step = None;
    h.store.transaction(&mut |tx| {
        step = None;
        let dropped = tx.photo(photo_id)?;
        session_id = dropped.session_id;
        let template = template_of(tx, session_id)?.ok_or(Error::NoTemplate)?;
        if !template.containers.iter().any(|c| c.final_no == n) {
            return Err(Error::InvalidValue(format!("no container {n}")));
        }
        if dropped.classification == Classification::Deleted {
            return Err(Error::InvalidValue("the photo is deleted".into()));
        }
        let mut occupant: Option<Photo> = None;
        for p in tx.photos(session_id)? {
            let slot = place_of(&p, tx.photo_place_row(p.id)?.as_ref()).1;
            if p.id != photo_id
                && slot == Some(n)
                && occupant
                    .as_ref()
                    .is_none_or(|o| (p.position, p.id) < (o.position, o.id))
            {
                occupant = Some(p);
            }
        }
        let own = place_of(&dropped, tx.photo_place_row(photo_id)?.as_ref()).1;
        if own == Some(n) {
            return Ok(());
        }
        let ids: Vec<i64> = match (&occupant, own) {
            (Some(o), _) => vec![photo_id, o.id],
            (None, _) => vec![photo_id],
        };
        let before = snapshot(tx, &ids)?;
        let mut label = format!("Move to container {n}");
        match occupant {
            Some(o) if own.is_none() => {
                retake(tx, &dropped, &o, &template, n, at)?;
                label = format!("Retake container {n}");
            }
            None => {
                reclassify(tx, &dropped, Classification::Main, at)?;
                put_by_hand(tx, photo_id, Some(n))?;
            }
            Some(o) => {
                // The two exchange containers and positions (the swap; owner card 1).
                if let (Some(a), Some(b)) = (dropped.position, o.position) {
                    tx.set_positions(&[(photo_id, b), (o.id, a)])?;
                }
                put_by_hand(tx, photo_id, Some(n))?;
                put_by_hand(tx, o.id, own)?;
            }
        }
        renumber_main(tx, session_id)?;
        tx.touch_session(session_id, at)?;
        let after = snapshot(tx, &ids)?;
        step = Some(template_step(label, session_id, &ids, before, after));
        Ok(())
    })?;
    h.finish(session_id, step)
}

/// 10G §6 rule 3 (P10): `old` is denied exactly as `ReviewService::delete` denies (Deleted, no
/// position, placed by hand, the denial row, the mirror tag) and released; `new` becomes Main in
/// container `n`, placed by hand, with `old`'s shot key (its own overrides, else its captured time
/// and camera file key) and container `n`'s presets (P8). Main is renumbered by the caller.
fn retake(
    tx: &mut dyn StoreTx,
    new: &Photo,
    old: &Photo,
    template: &Template,
    n: i64,
    at: i64,
) -> Result<()> {
    let key = tx
        .shot_keys(old.session_id)?
        .into_iter()
        .find(|k| k.photo_id == old.id)
        .ok_or_else(|| Error::NotFound(format!("photo {}", old.id)))?;
    if old.classification.is_ordered() {
        let group: Vec<(i64, i64)> = tx
            .photos(old.session_id)?
            .into_iter()
            .filter(|p| p.classification == old.classification)
            .filter_map(|p| p.position.map(|pos| (p.id, pos)))
            .collect();
        tx.set_positions(&position::remove_at(&group, old.id))?;
    }
    tx.set_photo_review(old.id, old.session_id, Classification::Deleted, None)?;
    let mut old_row = tx.photo_place_row(old.id)?.unwrap_or(PlaceRow {
        photo_id: old.id,
        ..PlaceRow::default()
    });
    old_row.place_by_hand = true;
    old_row.released = true;
    tx.photo_place_put(&old_row)?;
    tx.set_photo_denied(old.id, at)?;
    tx.enqueue_mirror_tag(&old.raw_path, old.id, at)?;

    let current = tx.photo(new.id)?;
    reclassify(tx, &current, Classification::Main, at)?;
    let row = tx.photo_place_row(new.id)?.unwrap_or(PlaceRow {
        photo_id: new.id,
        ..PlaceRow::default()
    });
    let framed_by_hand = row.frame_by_hand;
    tx.photo_place_put(&PlaceRow {
        slot: Some(n),
        place_by_hand: true,
        frame_by_hand: false,
        key_at: Some(key.at),
        key_name: key.name,
        ..row
    })?;
    let presets = template
        .containers
        .iter()
        .find(|c| c.final_no == n)
        .map(|c| &c.presets);
    // P8: a container without presets leaves a composition the person made alone.
    if presets.is_some_and(has_presets) || !framed_by_hand {
        apply_frame(tx, new.id, presets, false, at)?;
    }
    Ok(())
}

/// 10G §6 rule 2: the photos that differ (P9) go back to their expected place (P5) with
/// `place_by_hand = 0` and `frame_by_hand = 0`, and take that place's presets (P8: a container
/// with presets gives them; a container without presets and a Condition place leave the
/// composition as it is). The current template and shot order are used. One transaction, one
/// undo step "Back to template"; nothing differs → nothing is written and no step.
pub fn back_to_template(h: &HandAction<'_>, session_id: i64) -> Result<ReviewResult> {
    crate::services::retention::ensure_session_not_cleared(h.store, session_id)?;
    let at = h.clock.now_ms();
    let mut step = None;
    h.store.transaction(&mut |tx| {
        step = None;
        let template = template_of(tx, session_id)?.ok_or(Error::NoTemplate)?;
        let indexes = shot_indexes(tx, session_id, at)?;
        let mut differing = Vec::new();
        for photo in tx.photos(session_id)? {
            let row = tx.photo_place_row(photo.id)?;
            if differs(&photo, row.as_ref(), &indexes, &template.containers) {
                differing.push((indexes[&photo.id], photo, row));
            }
        }
        if differing.is_empty() {
            return Ok(());
        }
        differing.sort_by_key(|(index, photo, _)| (*index, photo.id));
        let ids: Vec<i64> = differing.iter().map(|(_, p, _)| p.id).collect();
        let before = snapshot(tx, &ids)?;
        for (index, photo, row) in &differing {
            let (class, slot, presets) = expected_place(*index, &template.containers);
            let current = tx.photo(photo.id)?;
            reclassify(tx, &current, class, at)?;
            tx.photo_place_put(&PlaceRow {
                slot,
                place_by_hand: false,
                frame_by_hand: false,
                ..row.clone().unwrap_or(PlaceRow {
                    photo_id: photo.id,
                    ..PlaceRow::default()
                })
            })?;
            if let Some(presets) = presets.filter(|p| has_presets(p)) {
                apply_frame(tx, photo.id, Some(presets), false, at)?;
            }
        }
        condition_in_shot_order(tx, session_id, &indexes)?;
        renumber_main(tx, session_id)?;
        tx.touch_session(session_id, at)?;
        let after = snapshot(tx, &ids)?;
        step = Some(template_step(
            "Back to template".into(),
            session_id,
            &ids,
            before,
            after,
        ));
        Ok(())
    })?;
    h.finish(session_id, step)
}

/// P7 for Condition: positions are kept; the template-placed Condition photos take, in shot
/// order, the positions that group holds.
fn condition_in_shot_order(
    tx: &mut dyn StoreTx,
    session_id: i64,
    indexes: &HashMap<i64, usize>,
) -> Result<()> {
    let mut condition: Vec<Photo> = tx
        .photos(session_id)?
        .into_iter()
        .filter(|p| p.classification == Classification::Condition)
        .collect();
    condition.sort_by_key(|p| (p.position.unwrap_or(i64::MAX), p.id));
    let mut placed = Vec::new();
    for p in &condition {
        if tx.photo_place_row(p.id)?.is_some_and(|r| !r.place_by_hand) {
            placed.push(p.id);
        }
    }
    let mut in_order = placed.clone();
    in_order.sort_by_key(|id| (indexes.get(id).copied().unwrap_or(usize::MAX), *id));
    let mut next = in_order.into_iter();
    let positions: Vec<(i64, i64)> = condition
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let id = if placed.contains(&p.id) {
                next.next().expect("template-placed condition photo")
            } else {
                p.id
            };
            (id, i as i64 + 1)
        })
        .collect();
    tx.set_positions(&positions)
}
