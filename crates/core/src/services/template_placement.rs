//! Placement on arrival (10G P1–P9). Every read and write for an arrival shares one store transaction.
use crate::{
    Result,
    domain::{
        composition::{CompositionKind, CompositionValues, Flip, Orientation, Origin},
        photo::Classification,
        template::{PlaceRow, TemplatePhotoView, TemplateView},
    },
    events::AppEvent,
    mechanisms::{
        capture_ledger::{seq_of, stem_key},
        composition::{default_rect, turned_frame},
        shot_order::{self, Entry, Shot},
        template::{self, Expected},
    },
    ports::{
        clock::Clock,
        events::EventSink,
        store::{Store, StoreTx},
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
        let Some(template) = tx.template_for_session(session_id)? else {
            return Ok(());
        };
        if template.containers.is_empty() {
            return Ok(());
        }
        templated = true;
        if tx.photo_place_row(photo_id)?.is_none() {
            tx.photo_place_put(&PlaceRow {
                photo_id,
                ..PlaceRow::default()
            })?;
        }
        let before = tx.photos(session_id)?;
        let shots = tx
            .shot_keys(session_id)?
            .into_iter()
            .filter(|s| !s.released)
            .map(|s| Shot {
                photo_id: s.photo_id,
                at: s.at,
                counter: s.name.as_deref().and_then(|name| seq_of(&stem_key(name))),
            })
            .collect();
        let ordered = shot_order::order(shots, tx.hold_times(session_id, now)?);
        let indexes: HashMap<i64, usize> = ordered
            .iter()
            .enumerate()
            .filter_map(|(i, e)| match e {
                Entry::Shot(s) => Some((s.photo_id, i + 1)),
                Entry::Hold { .. } => None,
            })
            .collect();
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
            let expected = template::expected(index, &template.containers);
            let (class, slot, presets) = match expected {
                Expected::Main(c) => (Classification::Main, Some(c.final_no), Some(&c.presets)),
                Expected::Condition => (Classification::Condition, None, None),
            };
            let place_changed = row.slot != slot || photo.classification != class;
            if place_changed && photo.id != photo_id && !changed.contains(&photo.id) {
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
            if place_changed || photo.id == photo_id {
                apply_frame(tx, photo.id, presets, row.frame_by_hand, now)?;
            }
        }
        // Main's stored order is the final container order; photos with no slot follow it.
        let mut after = tx.photos(session_id)?;
        let mut main: Vec<_> = after
            .iter()
            .filter(|p| p.classification == Classification::Main)
            .collect();
        let mut main_keys = HashMap::new();
        for p in &main {
            main_keys.insert(p.id, tx.photo_place_row(p.id)?);
        }
        main.sort_by_key(|p| {
            let row = main_keys.get(&p.id).and_then(Option::as_ref);
            (
                row.and_then(|r| r.slot).is_none(),
                row.and_then(|r| r.slot).unwrap_or(i64::MAX),
                row.is_some_and(|r| r.place_by_hand),
                p.position.unwrap_or(i64::MAX),
                p.id,
            )
        });
        tx.set_positions(
            &main
                .iter()
                .enumerate()
                .map(|(i, p)| (p.id, i as i64 + 1))
                .collect::<Vec<_>>(),
        )?;
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
            if p.id == photo_id {
                continue;
            }
            if let Some(old) = before.iter().find(|b| b.id == p.id)
                && (old.classification != p.classification || old.position != p.position)
                && !changed.contains(&p.id)
            {
                changed.push(p.id);
            }
        }
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

fn apply_frame(
    tx: &mut dyn StoreTx,
    photo_id: i64,
    presets: Option<&crate::domain::template::Presets>,
    frame_by_hand: bool,
    now: i64,
) -> Result<()> {
    if frame_by_hand {
        return Ok(());
    }
    let current = tx.composition(photo_id)?;
    let active = presets.is_some_and(|p| p.split || p.shape.is_some() || p.zoom > 0);
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

pub fn session_view(store: &dyn Store, session_id: i64) -> Result<TemplateView> {
    let mut result = TemplateView {
        templated: false,
        template_id: None,
        containers: Vec::new(),
        photos: Vec::new(),
        differs: 0,
        can_shift_back: false,
    };
    store.transaction(&mut |tx| {
        let Some(template) = tx.template_for_session(session_id)? else {
            return Ok(());
        };
        if template.containers.is_empty() {
            return Ok(());
        }
        result.templated = true;
        result.template_id = Some(template.id);
        result.containers = template.containers;
        for photo in tx.photos(session_id)? {
            if photo.classification == Classification::Deleted {
                continue;
            }
            let row = tx.photo_place_row(photo.id)?;
            result.photos.push(TemplatePhotoView {
                photo_id: photo.id,
                slot: if photo.classification == Classification::Main {
                    row.as_ref().and_then(|r| r.slot)
                } else {
                    None
                },
                by_hand: row.as_ref().is_none_or(|r| r.place_by_hand),
                differs: false,
            });
        }
        Ok(())
    })?;
    Ok(result)
}

pub fn renumber_main(tx: &mut dyn StoreTx, session_id: i64) -> Result<()> {
    let Some(template) = tx.template_for_session(session_id)? else {
        return Ok(());
    };
    if template.containers.is_empty() {
        return Ok(());
    }
    let mut main: Vec<_> = tx
        .photos(session_id)?
        .into_iter()
        .filter(|p| p.classification == Classification::Main)
        .collect();
    let mut rows = HashMap::new();
    for photo in &main {
        rows.insert(photo.id, tx.photo_place_row(photo.id)?);
    }
    main.sort_by_key(|p| {
        let r = rows.get(&p.id).and_then(Option::as_ref);
        (
            r.and_then(|r| r.slot).is_none(),
            r.and_then(|r| r.slot).unwrap_or(i64::MAX),
            r.is_some_and(|r| r.place_by_hand),
            p.position.unwrap_or(i64::MAX),
            p.id,
        )
    });
    tx.set_positions(
        &main
            .iter()
            .enumerate()
            .map(|(i, p)| (p.id, i as i64 + 1))
            .collect::<Vec<_>>(),
    )
}
