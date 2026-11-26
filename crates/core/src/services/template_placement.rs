//! Placement on arrival (10G P1–P9 as 10J MC-LV3 amends them: `master_plan_live.md` §5 CS3, CS8).
//! A photo is placed by its **binding** (the shot that was current when the camera reported it, or
//! when its file arrived without a report). Every read and write for an arrival shares one store
//! transaction.
use crate::{
    Error, Result,
    domain::{
        composition::{CompositionKind, CompositionValues, Flip, Orientation, Origin},
        photo::{Classification, Photo},
        template::{
            Container, PlaceRow, Presets, ShotBinding, Template, TemplatePhotoView, TemplateView,
        },
    },
    events::AppEvent,
    mechanisms::{
        composition::{default_rect, turned_frame},
        position, template,
    },
    ports::{
        clock::{Clock, SystemClock},
        events::EventSink,
        store::{Store, StoreTx},
    },
    services::{
        current_shot::{self, Current, CurrentShots, PhotoShot, ShotPlan, shot_plan, template_of},
        history::{History, PhotoState, Step, StepKind, TemplateChange, photo_state},
        review::ReviewResult,
    },
};
use std::collections::{HashMap, HashSet};

/// 10G §4 for one arrival: a file with no slot owner and an unbound place row is bound to the
/// current shot (CS6); then every template-placed photo goes to its binding's place (CS3).
pub fn place_arrival(
    store: &dyn Store,
    clock: &dyn Clock,
    events: &dyn EventSink,
    shots: &CurrentShots,
    photo_id: i64,
) -> Result<()> {
    let now = clock.now_ms();
    let mut changed = Vec::new();
    let mut session_id = 0;
    let mut templated = false;
    store.transaction(&mut |tx| {
        changed.clear();
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
        let plan = shot_plan(tx, session_id, now, &template)?;
        if plan
            .photos
            .get(&photo_id)
            .is_some_and(|p| p.owner.is_none() && p.binding.is_none())
        {
            shots.bind_arrival(tx, session_id, photo_id, now)?;
        }
        changed = place_template_photos(tx, session_id, &template, now, Some(photo_id))?;
        // A10: part of the arrival, not an edit in the app.
        tx.touch_session_on_arrival(session_id, now)?;
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

/// The template-placed photos of the session (a place row with `place_by_hand = 0`, not Deleted,
/// not released) that have a binding, in shot order.
fn template_placed<'p>(
    tx: &dyn StoreTx,
    photos: &'p [Photo],
    plan: &ShotPlan,
) -> Result<Vec<(&'p Photo, PlaceRow, PhotoShot, ShotBinding)>> {
    let mut result = Vec::new();
    for photo in photos {
        if photo.classification == Classification::Deleted {
            continue;
        }
        let Some(row) = tx.photo_place_row(photo.id)? else {
            continue;
        };
        if row.place_by_hand || row.released {
            continue;
        }
        let Some(shot) = plan.photos.get(&photo.id).copied() else {
            continue;
        };
        let Some(binding) = shot.binding else {
            continue;
        };
        result.push((photo, row, shot, binding));
    }
    result.sort_by_key(|(photo, _, shot, _)| (shot.rank, photo.id));
    Ok(result)
}

/// The containers hand-placed Main photos sit in (their slots).
fn hand_slots(tx: &dyn StoreTx, photos: &[Photo]) -> Result<HashSet<i64>> {
    let mut result = HashSet::new();
    for photo in photos {
        if photo.classification != Classification::Main {
            continue;
        }
        if let Some(row) = tx.photo_place_row(photo.id)?
            && row.place_by_hand
            && let Some(slot) = row.slot
        {
            result.insert(slot);
        }
    }
    Ok(result)
}

/// CS3 (P6–P8): every template-placed, bound photo goes to its expected place, with that place's
/// composition when its place changed (always for `arrived`); then Main and Condition are ordered
/// (P7). A binding to a container that already holds a hand-placed photo, or an earlier
/// template-placed photo of the same binding, gives Condition (hand wins); a photo already in its
/// container stays there. Returns the photos (other than `arrived`) whose place, classification
/// or position changed.
fn place_template_photos(
    tx: &mut dyn StoreTx,
    session_id: i64,
    template: &Template,
    now: i64,
    arrived: Option<i64>,
) -> Result<Vec<i64>> {
    let mut changed = Vec::new();
    let before = tx.photos(session_id)?;
    let plan = shot_plan(tx, session_id, now, template)?;
    let hand = hand_slots(tx, &before)?;
    let mut claimed = HashSet::new();
    let mut condition_order = Vec::new();
    for (photo, mut row, shot, binding) in template_placed(tx, &before, &plan)? {
        let target = binding
            .container_id
            .and_then(|c| template.containers.iter().find(|part| part.id == c));
        let (class, slot, presets) = match target {
            Some(c)
                if !claimed.contains(&c.final_no)
                    && (!hand.contains(&c.final_no)
                        || place_of(photo, Some(&row))
                            == (Classification::Main, Some(c.final_no))) =>
            {
                claimed.insert(c.final_no);
                (Classification::Main, Some(c.final_no), Some(&c.presets))
            }
            _ => (Classification::Condition, None, None),
        };
        let place_changed = row.slot != slot || photo.classification != class;
        if place_changed && Some(photo.id) != arrived && !changed.contains(&photo.id) {
            changed.push(photo.id);
        }
        row.slot = slot;
        tx.photo_place_put(&row)?;
        if class == Classification::Condition {
            condition_order.push((shot.rank, photo.id));
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
    condition_order.sort_by_key(|(rank, id)| (*rank, *id));
    let placed_ids: HashSet<i64> = condition_order.iter().map(|(_, id)| *id).collect();
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

/// CS3's expected place of a binding: Main in the container (its final number, its presets), or
/// Condition for `None`.
fn expected_place(
    binding: ShotBinding,
    containers: &[Container],
) -> (Classification, Option<i64>, Option<&Presets>) {
    match binding
        .container_id
        .and_then(|id| containers.iter().find(|c| c.id == id))
    {
        Some(c) => (Classification::Main, Some(c.final_no), Some(&c.presets)),
        None => (Classification::Condition, None, None),
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

/// P9: whether the photo differs from the template. A photo with no binding (unbound, or
/// released) never differs.
fn differs(
    photo: &Photo,
    row: Option<&PlaceRow>,
    plan: &ShotPlan,
    containers: &[Container],
) -> bool {
    if photo.classification == Classification::Deleted {
        return false;
    }
    let Some(binding) = plan.photos.get(&photo.id).and_then(|p| p.binding) else {
        return false;
    };
    let (class, slot, presets) = expected_place(binding, containers);
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

/// CS8: the Mac's view, its "Next" outline (`next_shot`) the `shoot_no` of the current shot for
/// the active session and of the default shot for any other.
pub fn session_view_with_current(
    store: &dyn Store,
    shots: &CurrentShots,
    now: i64,
    session_id: i64,
) -> Result<TemplateView> {
    let mut view = session_view_at(store, now, session_id)?;
    if view.templated {
        view.next_shot = match shots.current(store, session_id, now)? {
            Some(Current::Shot(id)) => view
                .containers
                .iter()
                .find(|c| c.id == id)
                .map(|c| c.shoot_no),
            _ => None,
        };
    }
    Ok(view)
}

/// The session's template view; "differs" uses the bindings placement would use at `now`, and
/// `next_shot` is the session's default shot (CS4; [`session_view_with_current`] gives the active
/// session's current shot).
pub fn session_view_at(store: &dyn Store, now: i64, session_id: i64) -> Result<TemplateView> {
    let mut result = TemplateView {
        templated: false,
        template_id: None,
        containers: Vec::new(),
        photos: Vec::new(),
        differs: 0,
        can_shift_back: false,
        next_shot: None,
    };
    store.transaction(&mut |tx| {
        result.photos.clear();
        result.differs = 0;
        let Some(template) = template_of(tx, session_id)? else {
            return Ok(());
        };
        let plan = shot_plan(tx, session_id, now, &template)?;
        result.templated = true;
        result.template_id = Some(template.id);
        for photo in tx.photos(session_id)? {
            if photo.classification == Classification::Deleted {
                continue;
            }
            let row = tx.photo_place_row(photo.id)?;
            let differs = differs(&photo, row.as_ref(), &plan, &template.containers);
            result.differs += i64::from(differs);
            result.photos.push(TemplatePhotoView {
                photo_id: photo.id,
                slot: place_of(&photo, row.as_ref()).1,
                by_hand: row.as_ref().is_none_or(|r| r.place_by_hand),
                differs,
            });
        }
        result.can_shift_back = !shift_plan(tx, session_id, &template, now)?.1.is_empty();
        result.next_shot = match current_shot::default_shot(tx, session_id, now, &template)? {
            Current::Shot(id) => template
                .containers
                .iter()
                .find(|c| c.id == id)
                .map(|c| c.shoot_no),
            Current::Free => None,
        };
        result.containers = template.containers;
        Ok(())
    })?;
    Ok(result)
}

/// `(the Deleted, unreleased photos, the changed bindings (photo, new binding))`.
type ShiftPlan = (Vec<i64>, Vec<(i64, ShotBinding)>);

/// CS8 Shift the rest back, as a plan: the Deleted, unreleased photos (all released by the
/// action) and the new place-row bindings. `G` = the containers left empty by those photos (bound
/// there, no photo in it); every template-placed photo whose binding has rank `r` in shooting
/// order (Condition-bound ones ranks `N+1, N+2, …` in shot order) is bound to the container at
/// rank `r − |{g ∈ G : rank(g) < r}|` (Condition beyond `N`). Only changed bindings are listed.
fn shift_plan(
    tx: &dyn StoreTx,
    session_id: i64,
    template: &Template,
    now: i64,
) -> Result<ShiftPlan> {
    let order = current_shot::order(template);
    let plan = shot_plan(tx, session_id, now, template)?;
    let photos = tx.photos(session_id)?;
    let mut filled = HashSet::new();
    for p in photos
        .iter()
        .filter(|p| p.classification == Classification::Main)
    {
        if let Some(slot) = tx.photo_place_row(p.id)?.and_then(|r| r.slot) {
            filled.insert(slot);
        }
    }
    let mut denied = Vec::new();
    let mut gaps = Vec::new();
    for p in photos
        .iter()
        .filter(|p| p.classification == Classification::Deleted)
    {
        if tx.photo_place_row(p.id)?.is_some_and(|r| r.released) {
            continue;
        }
        denied.push(p.id);
        let Some(c) = plan
            .photos
            .get(&p.id)
            .and_then(|s| s.binding)
            .and_then(|b| b.container_id)
        else {
            continue;
        };
        if let Some(rank) = order.iter().position(|id| *id == c)
            && let Some(container) = template.containers.iter().find(|part| part.id == c)
            && !filled.contains(&container.final_no)
        {
            gaps.push(rank);
        }
    }
    gaps.sort_unstable();
    gaps.dedup();
    let mut changes = Vec::new();
    if gaps.is_empty() {
        return Ok((denied, changes));
    }
    let mut condition_rank = order.len();
    for (photo, _, _, binding) in template_placed(tx, &photos, &plan)? {
        let rank = match binding.container_id {
            Some(c) => match order.iter().position(|id| *id == c) {
                Some(rank) => rank,
                None => continue,
            },
            None => {
                condition_rank += 1;
                condition_rank - 1
            }
        };
        let shift = gaps.iter().filter(|g| **g < rank).count();
        let next = ShotBinding {
            container_id: order.get(rank - shift).copied(),
        };
        if next != binding {
            changes.push((photo.id, next));
        }
    }
    Ok((denied, changes))
}

/// 10G §6 rule 4 Shift the rest back (P10, CS8): every Deleted, not released photo of the session
/// is released, the later template-placed photos are bound back (see [`shift_plan`]) and go to
/// their places with those places' compositions (P6, P8); the orders are renumbered (P7). Not
/// templated → `no_template`; nothing to shift → `nothing_to_shift`. One transaction, one undo
/// step "Shift the rest back" naming every photo whose place, row or composition changed.
pub fn shift_back(h: &HandAction<'_>, session_id: i64) -> Result<ReviewResult> {
    crate::services::retention::ensure_session_not_cleared(h.store, session_id)?;
    let at = h.clock.now_ms();
    let mut step = None;
    h.store.transaction(&mut |tx| {
        step = None;
        let template = template_of(tx, session_id)?.ok_or(Error::NoTemplate)?;
        let (denied, changes) = shift_plan(tx, session_id, &template, at)?;
        if changes.is_empty() {
            return Err(Error::NothingToShift);
        }
        let photos = tx.photos(session_id)?;
        let ids: Vec<i64> = photos.iter().map(|p| p.id).collect();
        let before = snapshot(tx, &ids)?;
        for id in &denied {
            let row = tx.photo_place_row(*id)?.unwrap_or(PlaceRow {
                photo_id: *id,
                ..PlaceRow::default()
            });
            tx.photo_place_put(&PlaceRow {
                released: true,
                ..row
            })?;
        }
        for (id, binding) in &changes {
            if let Some(mut row) = tx.photo_place_row(*id)? {
                row.shot = Some(*binding);
                tx.photo_place_put(&row)?;
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

/// The dependencies of a person's template action (drop onto a container, Back to template,
/// Shift the rest back, Retake).
pub struct HandAction<'a> {
    pub store: &'a dyn Store,
    pub clock: &'a dyn Clock,
    pub events: &'a dyn EventSink,
    pub history: &'a History,
    pub shots: std::sync::Arc<CurrentShots>,
}

/// CS7 Retake: the photo in container `container_id` of the active session is denied exactly as
/// `ReviewService::delete` denies (Deleted, no position, placed by hand, the denial row, the mirror
/// tag) **and released**, in one transaction with one undo step "Retake shot <final_no>"; then
/// that container is the current shot, so the next exposure fills it with its presets. Another
/// session → `session_changed`; no photo in the container → `not_in_template`.
pub fn retake(h: &HandAction<'_>, session_id: i64, container_id: i64) -> Result<ReviewResult> {
    if h.store.active_session_id()? != Some(session_id) {
        return Err(Error::SessionChanged);
    }
    crate::services::retention::ensure_session_not_cleared(h.store, session_id)?;
    let at = h.clock.now_ms();
    let mut step = None;
    h.store.transaction(&mut |tx| {
        step = None;
        let template = template_of(tx, session_id)?.ok_or(Error::NoTemplate)?;
        let container = template
            .containers
            .iter()
            .find(|c| c.id == container_id)
            .ok_or(Error::NotInTemplate)?;
        let photos = tx.photos(session_id)?;
        let mut old: Option<&Photo> = None;
        for p in photos
            .iter()
            .filter(|p| p.classification == Classification::Main)
        {
            if tx.photo_place_row(p.id)?.and_then(|r| r.slot) == Some(container.final_no)
                && old.is_none_or(|o| (p.position, p.id) < (o.position, o.id))
            {
                old = Some(p);
            }
        }
        let old = old.ok_or(Error::NotInTemplate)?;
        let ids: Vec<i64> = photos.iter().map(|p| p.id).collect();
        let before = snapshot(tx, &ids)?;
        reclassify(tx, old, Classification::Deleted, at)?;
        let mut row = tx.photo_place_row(old.id)?.unwrap_or(PlaceRow {
            photo_id: old.id,
            ..PlaceRow::default()
        });
        row.place_by_hand = true;
        row.released = true;
        tx.photo_place_put(&row)?;
        // Changelog 46 (2): Deny starts the undo window; the RAW goes when it closes (H2).
        tx.set_photo_denied(old.id, at)?;
        renumber_main(tx, session_id)?;
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
            format!("Retake shot {}", container.final_no),
            session_id,
            &changed.0,
            changed.1,
            changed.2,
        ));
        Ok(())
    })?;
    h.shots.set_after_retake(session_id, container_id);
    h.finish(session_id, step)
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
                replace_on_drop(tx, &dropped, &o, &template, n, at)?;
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

/// 10G §6 rule 3 (P10) under CS8: `old` is denied exactly as `ReviewService::delete` denies
/// (Deleted, no position, placed by hand, the denial row, the mirror tag) and released; `new`
/// becomes Main in container `n`, placed by the template with its place row bound to that
/// container, and takes container `n`'s presets (P8). Main is renumbered by the caller.
fn replace_on_drop(
    tx: &mut dyn StoreTx,
    new: &Photo,
    old: &Photo,
    template: &Template,
    n: i64,
    at: i64,
) -> Result<()> {
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
        place_by_hand: false,
        frame_by_hand: false,
        shot: Some(ShotBinding {
            container_id: template
                .containers
                .iter()
                .find(|c| c.final_no == n)
                .map(|c| c.id),
        }),
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

/// 10G §6 rule 2 (CS8): the photos that differ (P9) go back to their binding's place (CS3) with
/// `place_by_hand = 0` and `frame_by_hand = 0`, and take that place's presets (P8: a container
/// with presets gives them; a container without presets and a Condition place leave the
/// composition as it is). The current template and bindings are used. One transaction, one
/// undo step "Back to template"; nothing differs → nothing is written and no step.
pub fn back_to_template(h: &HandAction<'_>, session_id: i64) -> Result<ReviewResult> {
    crate::services::retention::ensure_session_not_cleared(h.store, session_id)?;
    let at = h.clock.now_ms();
    let mut step = None;
    h.store.transaction(&mut |tx| {
        step = None;
        let template = template_of(tx, session_id)?.ok_or(Error::NoTemplate)?;
        let plan = shot_plan(tx, session_id, at, &template)?;
        let mut differing = Vec::new();
        for photo in tx.photos(session_id)? {
            let row = tx.photo_place_row(photo.id)?;
            if differs(&photo, row.as_ref(), &plan, &template.containers) {
                differing.push((plan.photos[&photo.id], photo, row));
            }
        }
        if differing.is_empty() {
            return Ok(());
        }
        differing.sort_by_key(|(shot, photo, _)| (shot.rank, photo.id));
        let ids: Vec<i64> = differing.iter().map(|(_, p, _)| p.id).collect();
        let before = snapshot(tx, &ids)?;
        for (shot, photo, row) in &differing {
            let binding = shot.binding.expect("a differing photo is bound");
            let (class, slot, presets) = expected_place(binding, &template.containers);
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
        let indexes: HashMap<i64, usize> =
            plan.photos.iter().map(|(id, p)| (*id, p.rank)).collect();
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
