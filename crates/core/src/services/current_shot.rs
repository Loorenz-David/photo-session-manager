//! The current shot (book 10J MC-LV3; `master_plan_live.md` §5 CS). One value per process, held in
//! memory only: a relaunch or a session switch starts again from the default. Every exposure of a
//! templated session is bound, inside the transaction that inserts it, to the shot that is current
//! at that moment; a photo is then placed by its binding whenever its file arrives (CS3).
//!
//! Lock order: the store's transaction first, then the held value (never the other way round).
use std::{
    collections::{HashMap, HashSet},
    sync::Mutex,
};

use crate::{
    Error, Result,
    domain::{
        camera::NewExposure,
        photo::Classification,
        template::{PlaceRow, ShotBinding, Template},
    },
    events::AppEvent,
    mechanisms::{
        capture_ledger::{seq_of, stem_key},
        current_shot::next_untaken,
        shot_order::{self, Occupant, Shot},
    },
    ports::{
        events::EventSink,
        store::{Store, StoreTx},
    },
};

/// The shot the next exposure is bound to: a template container (by id) or none (Condition).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Current {
    Shot(i64),
    Free,
}

impl Current {
    fn from_container(c: Option<i64>) -> Self {
        c.map_or(Self::Free, Self::Shot)
    }
    fn container(self) -> Option<i64> {
        match self {
            Self::Shot(c) => Some(c),
            Self::Free => None,
        }
    }
}

/// CS1: the held value, `(session id, current)`, for one session at a time.
#[derive(Default)]
pub struct CurrentShots {
    held: Mutex<Option<(i64, Current)>>,
}

/// One photo's place in the session's shot order (CS2, CS3).
#[derive(Debug, Clone, Copy)]
pub(crate) struct PhotoShot {
    /// CS3: `None` = unbound (shot before this track: placement never moves it).
    pub binding: Option<ShotBinding>,
    /// CS2: the exposure that owns the photo's slot, if any.
    pub owner: Option<i64>,
    /// 1-based position in the shot order (holds counted).
    pub rank: usize,
}

/// The session's shot order read for placement: each unreleased photo's binding and rank, and
/// the containers held by reported shutters whose files are on the way.
pub(crate) struct ShotPlan {
    pub photos: HashMap<i64, PhotoShot>,
    pub held: HashSet<i64>,
}

/// The containers of the template in shooting order (`shoot_no` ascending).
pub fn order(template: &Template) -> Vec<i64> {
    let mut containers: Vec<_> = template.containers.iter().collect();
    containers.sort_by_key(|c| (c.shoot_no, c.id));
    containers.into_iter().map(|c| c.id).collect()
}

/// The template of a templated session (10G P1): one exists for the item's type and has containers.
pub(crate) fn template_of(tx: &dyn StoreTx, session_id: i64) -> Result<Option<Template>> {
    Ok(tx
        .template_for_session(session_id)?
        .filter(|t| !t.containers.is_empty()))
}

/// CS3's last sentence: a binding to a container no longer in the template counts as `None`.
fn in_template(binding: ShotBinding, template: &Template) -> ShotBinding {
    ShotBinding {
        container_id: binding
            .container_id
            .filter(|c| template.containers.iter().any(|part| part.id == *c)),
    }
}

/// CS2 + CS3 at `now`: the slot owners of the session's unreleased shots and holds, then each
/// photo's binding (its place row's, else its slot owner's exposure's, else unbound).
pub(crate) fn shot_plan(
    tx: &dyn StoreTx,
    session_id: i64,
    now: i64,
    template: &Template,
) -> Result<ShotPlan> {
    let shots = tx
        .shot_keys(session_id)?
        .into_iter()
        .filter(|s| !s.released)
        .map(|s| Shot {
            photo_id: s.photo_id,
            at: s.at,
            counter: s.name.as_deref().and_then(|name| seq_of(&stem_key(name))),
            exposure: s.exposure_id,
        })
        .collect();
    let mut plan = ShotPlan {
        photos: HashMap::new(),
        held: HashSet::new(),
    };
    for (i, slot) in shot_order::slot_owners(shots, tx.hold_times(session_id, now)?)
        .into_iter()
        .enumerate()
    {
        let owner_binding = match slot.owner {
            Some(exposure) => tx.exposure_shot(exposure)?,
            None => None,
        };
        match slot.occupant {
            Occupant::Photo(photo_id) => {
                let row_binding = tx.photo_place_row(photo_id)?.and_then(|r| r.shot);
                plan.photos.insert(
                    photo_id,
                    PhotoShot {
                        binding: row_binding
                            .or(owner_binding)
                            .map(|b| in_template(b, template)),
                        owner: slot.owner,
                        rank: i + 1,
                    },
                );
            }
            Occupant::Hold(_) => {
                if let Some(c) = owner_binding.and_then(|b| in_template(b, template).container_id) {
                    plan.held.insert(c);
                }
            }
        }
    }
    Ok(plan)
}

/// CS4 `taken(s, now)`: the containers a photo is in (not Deleted, Main, its place row's slot is
/// the container's final number), and the bindings of the slots reported shutters hold.
pub fn taken(
    tx: &dyn StoreTx,
    session_id: i64,
    now: i64,
    template: &Template,
) -> Result<HashSet<i64>> {
    let mut result = shot_plan(tx, session_id, now, template)?.held;
    for photo in tx.photos(session_id)? {
        if photo.classification != Classification::Main {
            continue;
        }
        if let Some(slot) = tx.photo_place_row(photo.id)?.and_then(|r| r.slot)
            && let Some(c) = template.containers.iter().find(|c| c.final_no == slot)
        {
            result.insert(c.id);
        }
    }
    Ok(result)
}

/// CS4 `default_shot(s)`: the first container in shooting order that is not taken, else `Free`.
pub fn default_shot(
    tx: &dyn StoreTx,
    session_id: i64,
    now: i64,
    template: &Template,
) -> Result<Current> {
    advance(tx, session_id, now, template, None)
}

/// CS4 `advance(s, from)`: the first container after `from` in shooting order that is not taken,
/// else the first not taken, else `Free`. `from = None` (after a Free binding, or no shot yet)
/// is the first not taken.
pub fn advance(
    tx: &dyn StoreTx,
    session_id: i64,
    now: i64,
    template: &Template,
    from: Option<i64>,
) -> Result<Current> {
    let taken = taken(tx, session_id, now, template)?;
    Ok(Current::from_container(next_untaken(
        &order(template),
        &taken,
        from,
    )))
}

/// The move after a binding (CS5, CS6): the bound container counts as taken even before its
/// file is placed, then `advance` from it (from the first container after a Free binding).
fn advance_after_binding(
    tx: &dyn StoreTx,
    session_id: i64,
    now: i64,
    template: &Template,
    bound: Option<i64>,
) -> Result<Current> {
    let mut taken = taken(tx, session_id, now, template)?;
    taken.extend(bound);
    Ok(Current::from_container(next_untaken(
        &order(template),
        &taken,
        bound,
    )))
}

fn container_text(c: Option<i64>) -> String {
    c.map_or("none".to_owned(), |c| c.to_string())
}

impl CurrentShots {
    /// CS1 for the active session, inside the caller's transaction: hold the default when another
    /// session (or nothing) is held; move on from a held shot that is taken or left the template.
    fn resolve(
        held: &mut Option<(i64, Current)>,
        tx: &dyn StoreTx,
        session_id: i64,
        now: i64,
        template: &Template,
    ) -> Result<Current> {
        let value = match *held {
            Some((s, Current::Shot(c))) if s == session_id => {
                let in_order = template.containers.iter().any(|part| part.id == c);
                if in_order && !taken(tx, session_id, now, template)?.contains(&c) {
                    Current::Shot(c)
                } else {
                    let next = advance(tx, session_id, now, template, Some(c))?;
                    tracing::info!(
                        target: "psm::current_shot",
                        "current_shot.set session={session_id} container={} by=advance",
                        container_text(next.container())
                    );
                    next
                }
            }
            Some((s, Current::Free)) if s == session_id => Current::Free,
            _ => {
                let value = default_shot(tx, session_id, now, template)?;
                tracing::info!(
                    target: "psm::current_shot",
                    "current_shot.set session={session_id} container={} by=default",
                    container_text(value.container())
                );
                value
            }
        };
        *held = Some((session_id, value));
        Ok(value)
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Option<(i64, Current)>> {
        self.held.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// CS7: a session switch (`ActiveSessionChanged`) forgets the held value.
    pub fn clear(&self) {
        *self.lock() = None;
    }

    /// CS1 `current(s)`: `None` for an untemplated session (nothing held changes). For the active
    /// session the held value (resolved as CS1 says); for any other session its default, without
    /// touching what is held (CS8: the Mac shows another session's default).
    pub fn current(&self, store: &dyn Store, session_id: i64, now: i64) -> Result<Option<Current>> {
        let active = store.active_session_id()? == Some(session_id);
        let mut value = None;
        store.transaction(&mut |tx| {
            value = None;
            let Some(template) = template_of(tx, session_id)? else {
                return Ok(());
            };
            value = Some(if active {
                Self::resolve(&mut self.lock(), tx, session_id, now, &template)?
            } else {
                default_shot(tx, session_id, now, &template)?
            });
            Ok(())
        })?;
        Ok(value)
    }

    /// CS7 `set(s, Shot(c) | Free)`: the person (the phone) chooses the current shot.
    pub fn set(
        &self,
        store: &dyn Store,
        events: &dyn EventSink,
        session_id: i64,
        value: Current,
        now: i64,
    ) -> Result<()> {
        if store.active_session_id()? != Some(session_id) {
            return Err(Error::SessionChanged);
        }
        store.transaction(&mut |tx| {
            let template = template_of(tx, session_id)?.ok_or(Error::NoTemplate)?;
            if let Current::Shot(c) = value {
                if !template.containers.iter().any(|part| part.id == c) {
                    return Err(Error::NotInTemplate);
                }
                if taken(tx, session_id, now, &template)?.contains(&c) {
                    return Err(Error::ShotFilled);
                }
            }
            *self.lock() = Some((session_id, value));
            Ok(())
        })?;
        tracing::info!(
            target: "psm::current_shot",
            "current_shot.set session={session_id} container={} by=phone",
            container_text(value.container())
        );
        events.emit(AppEvent::SessionChanged { session_id });
        Ok(())
    }

    /// CS7 (Retake): after the denial committed, the retaken container becomes current.
    pub(crate) fn set_after_retake(&self, session_id: i64, container_id: i64) {
        *self.lock() = Some((session_id, Current::Shot(container_id)));
        tracing::info!(
            target: "psm::current_shot",
            "current_shot.set session={session_id} container={container_id} by=retake"
        );
    }

    /// CS5: inserts the exposure row bound to the current shot of its (active, templated) session
    /// in one transaction, then moves the current shot on with the new exposure counted as a hold.
    /// A late match to a file that was bound at arrival (CS6) gives the exposure that binding and
    /// does not advance again. Called under the ledger worker lock. Returns the exposure id.
    pub fn bind_exposure(
        &self,
        store: &dyn Store,
        events: &dyn EventSink,
        exposure: &NewExposure,
    ) -> Result<i64> {
        let mut id = 0;
        let mut bound = None;
        store.transaction(&mut |tx| {
            bound = None;
            let template = match exposure.session_id {
                Some(s) => template_of(tx, s)?.map(|t| (s, t)),
                None => None,
            };
            let Some((s, template)) = template else {
                id = tx.insert_exposure(&NewExposure {
                    shot: None,
                    ..exposure.clone()
                })?;
                tx.match_late_exposure(id, exposure.exposed_at)?;
                return Ok(());
            };
            let mut held = self.lock();
            let current = Self::resolve(&mut held, tx, s, exposure.created_at, &template)?;
            let binding = ShotBinding {
                container_id: current.container(),
            };
            id = tx.insert_exposure(&NewExposure {
                shot: Some(binding),
                ..exposure.clone()
            })?;
            match tx.match_late_exposure(id, exposure.exposed_at)? {
                Some(earlier) => bound = Some((s, earlier, false)),
                None => {
                    let next = advance_after_binding(
                        tx,
                        s,
                        exposure.created_at,
                        &template,
                        binding.container_id,
                    )?;
                    *held = Some((s, next));
                    bound = Some((s, binding, true));
                }
            }
            Ok(())
        })?;
        if let Some((s, binding, advanced)) = bound {
            tracing::info!(
                target: "psm::current_shot",
                "current_shot.bound session={s} exposure={id} container={}",
                container_text(binding.container_id)
            );
            if advanced {
                let next = self.lock().and_then(|(held, c)| (held == s).then_some(c));
                if let Some(next) = next {
                    tracing::info!(
                        target: "psm::current_shot",
                        "current_shot.set session={s} container={} by=advance",
                        container_text(next.container())
                    );
                }
            }
            events.emit(AppEvent::SessionChanged { session_id: s });
        }
        Ok(id)
    }

    /// CS6, inside the arrival's transaction: a templated session's photo with no slot owner and
    /// an unbound place row (Capture One tether, a file no exposure matched) is bound to the
    /// current shot on its place row, once; the current shot then moves on as after an exposure.
    /// Returns whether it bound the photo.
    pub fn bind_arrival(
        &self,
        tx: &mut dyn StoreTx,
        session_id: i64,
        photo_id: i64,
        now: i64,
    ) -> Result<bool> {
        let Some(template) = template_of(tx, session_id)? else {
            return Ok(false);
        };
        let mut row = tx.photo_place_row(photo_id)?.unwrap_or(PlaceRow {
            photo_id,
            ..PlaceRow::default()
        });
        if row.shot.is_some() {
            return Ok(false);
        }
        let mut held = self.lock();
        let current = Self::resolve(&mut held, tx, session_id, now, &template)?;
        let binding = ShotBinding {
            container_id: current.container(),
        };
        row.shot = Some(binding);
        tx.photo_place_put(&row)?;
        let next = advance_after_binding(tx, session_id, now, &template, binding.container_id)?;
        *held = Some((session_id, next));
        drop(held);
        tracing::info!(
            target: "psm::current_shot",
            "current_shot.bound session={session_id} exposure=none photo={photo_id} container={}",
            container_text(binding.container_id)
        );
        tracing::info!(
            target: "psm::current_shot",
            "current_shot.set session={session_id} container={} by=advance",
            container_text(next.container())
        );
        Ok(true)
    }
}
