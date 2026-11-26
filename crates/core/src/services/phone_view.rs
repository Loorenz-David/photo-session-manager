//! PH6 (`master_plan_live.md` §5): the session as the phone shows it — the active session's item,
//! its current shot, the template's shots in shooting order with the photo in each, the Main
//! photos without a container (extras) and the Condition photos. Read-only; the phone server
//! serializes it as is (H8: no rule lives in the handler).
use serde::Serialize;

use crate::{
    Result,
    domain::{
        composition::{CompositionKind, Shape},
        photo::{Classification, Photo},
        template::Presets,
    },
    mechanisms::template::shot_ratio,
    ports::store::{Store, StoreTx},
    services::current_shot::{self, Current, CurrentShots, template_of},
};

/// `GET /api/phone/v1/session`: `session` is `null` when no session is active.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PhoneSessionView {
    pub session: Option<PhoneSession>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PhoneSession {
    pub id: i64,
    pub item: PhoneItem,
    pub templated: bool,
    /// `null` for an untemplated session.
    pub current: Option<PhoneCurrent>,
    pub shots: Vec<PhoneShot>,
    pub extras: Vec<PhoneImage>,
    pub condition: Vec<PhoneImage>,
    pub counts: PhoneCounts,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PhoneItem {
    /// The item's title.
    pub name: String,
    pub sku: String,
    /// The item's barcode (owner card 1, 2026-11-26).
    pub article_number: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PhoneCurrent {
    Shot { container_id: i64 },
    Free,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PhoneShot {
    pub container_id: i64,
    pub final_no: i64,
    pub shoot_no: i64,
    /// The container's name, or "Container <final_no>".
    pub name: String,
    pub ratio: Option<f64>,
    pub split: bool,
    pub zoom: u8,
    /// Whether the container has an example image.
    pub example: bool,
    pub photo: Option<PhoneImage>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PhoneImage {
    pub photo_id: i64,
    /// The photo's composition revision (the phone re-reads the preview when it changes).
    pub revision: i64,
    pub ratio: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct PhoneCounts {
    pub main: i64,
    pub condition: i64,
}

/// An image's ratio: the shot ratio of its own composition (split, or its shape).
fn image(tx: &dyn StoreTx, photo: &Photo, default_shape: Shape) -> Result<PhoneImage> {
    let c = tx.composition(photo.id)?;
    Ok(PhoneImage {
        photo_id: photo.id,
        revision: c.revision,
        ratio: shot_ratio(
            &Presets {
                split: c.kind == CompositionKind::Split,
                shape: Some(c.shape),
                zoom: 0,
            },
            default_shape,
        ),
    })
}

/// PH6 for `session_id` (the active session; `None` → `{session: null}`). `now` is the time the
/// current shot is read at (reported shutters hold their containers for the FIFO window).
pub fn phone_view(
    store: &dyn Store,
    shots: &CurrentShots,
    default_shape: Shape,
    session_id: Option<i64>,
    now: i64,
) -> Result<PhoneSessionView> {
    let Some(session_id) = session_id else {
        return Ok(PhoneSessionView { session: None });
    };
    let session = store.session(session_id)?;
    let item = match session.session.item_id {
        Some(id) => {
            let item = store.item(id)?;
            PhoneItem {
                name: item.title,
                sku: item.sku,
                article_number: item.barcode.filter(|b| !b.trim().is_empty()),
            }
        }
        // Unassigned has no item.
        None => PhoneItem {
            name: "Unassigned".into(),
            sku: String::new(),
            article_number: None,
        },
    };
    let current = shots.current(store, session_id, now)?;
    let mut view = None;
    store.transaction(&mut |tx| {
        let template = template_of(tx, session_id)?;
        let mut photos = tx.photos(session_id)?;
        photos.sort_by_key(|p| (p.position.unwrap_or(i64::MAX), p.id));
        let mut main = Vec::new();
        let mut condition = Vec::new();
        for p in photos {
            match p.classification {
                Classification::Main => {
                    let slot = tx.photo_place_row(p.id)?.and_then(|r| r.slot);
                    main.push((p, slot));
                }
                Classification::Condition => condition.push(p),
                _ => {}
            }
        }
        let counts = PhoneCounts {
            main: main.len() as i64,
            condition: condition.len() as i64,
        };
        let mut shot_list = Vec::new();
        let mut in_container = std::collections::HashSet::new();
        if let Some(template) = &template {
            for id in current_shot::order(template) {
                let c = template
                    .containers
                    .iter()
                    .find(|c| c.id == id)
                    .expect("ordered container");
                let photo = main
                    .iter()
                    .find(|(_, slot)| *slot == Some(c.final_no))
                    .map(|(p, _)| p);
                if let Some(p) = photo {
                    in_container.insert(p.id);
                }
                shot_list.push(PhoneShot {
                    container_id: c.id,
                    final_no: c.final_no,
                    shoot_no: c.shoot_no,
                    name: c
                        .name
                        .clone()
                        .filter(|n| !n.trim().is_empty())
                        .unwrap_or_else(|| format!("Container {}", c.final_no)),
                    ratio: shot_ratio(&c.presets, default_shape),
                    split: c.presets.split,
                    zoom: c.presets.zoom,
                    example: c.example_file.is_some(),
                    photo: photo.map(|p| image(tx, p, default_shape)).transpose()?,
                });
            }
        }
        let extras = main
            .iter()
            .filter(|(p, _)| !in_container.contains(&p.id))
            .map(|(p, _)| image(tx, p, default_shape))
            .collect::<Result<Vec<_>>>()?;
        let condition = condition
            .iter()
            .map(|p| image(tx, p, default_shape))
            .collect::<Result<Vec<_>>>()?;
        view = Some(PhoneSession {
            id: session_id,
            item: item.clone(),
            templated: template.is_some(),
            current: current.map(|c| match c {
                Current::Shot(container_id) => PhoneCurrent::Shot { container_id },
                Current::Free => PhoneCurrent::Free,
            }),
            shots: shot_list,
            extras,
            condition,
            counts,
        });
        Ok(())
    })?;
    Ok(PhoneSessionView { session: view })
}
