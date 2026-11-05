use crate::{
    Error, Result,
    domain::composition::{
        Composition, CompositionKind, CompositionValues, Flip, Orientation, Origin, Shape,
    },
    events::AppEvent,
    mechanisms::composition::{default_rect, turned_frame, validate},
    ports::{clock::Clock, events::EventSink, store::Store},
    services::history::{CompositionChange, History, Step, StepKind},
};
use std::sync::Arc;

pub struct CompositionService {
    pub store: Arc<dyn Store>,
    pub clock: Arc<dyn Clock>,
    pub events: Arc<dyn EventSink>,
    pub history: Arc<History>,
    pub default_shape: Shape,
}
struct SavedChange {
    composition: Composition,
    before: (CompositionValues, Origin),
    after: (CompositionValues, Origin),
    changed: bool,
}
impl CompositionService {
    pub fn apply(&self, items: Vec<(i64, CompositionValues)>) -> ApplyResult {
        let mut result = ApplyResult::default();
        let mut changes = Vec::new();
        let mut session_id = None;
        for (photo_id, values) in items {
            match self.set_unrecorded(photo_id, values) {
                Ok(saved) => {
                    result.applied.push(photo_id);
                    if saved.changed {
                        session_id.get_or_insert(
                            self.store.photo(photo_id).expect("saved photo").session_id,
                        );
                        changes.push(CompositionChange {
                            photo_id,
                            before: saved.before,
                            after: saved.after,
                        });
                    }
                }
                Err(error) => result.refused.push(ApplyRefusal {
                    photo_id,
                    reason: match error {
                        Error::CompositionInvalid { reason } => reason,
                        other => other.to_string(),
                    },
                }),
            }
        }
        if let Some(session_id) = session_id {
            let n = changes.len();
            self.history.record(
                session_id,
                Step {
                    label: format!(
                        "Apply framing to {n} photo{}",
                        if n == 1 { "" } else { "s" }
                    ),
                    kind: StepKind::Composition {
                        session_id,
                        photos: changes,
                    },
                },
            );
        }
        result
    }
    pub fn set(&self, photo_id: i64, values: CompositionValues) -> Result<Composition> {
        let saved = self.set_unrecorded(photo_id, values)?;
        if saved.changed {
            self.record_one(photo_id, saved.before, saved.after)?;
        }
        Ok(saved.composition)
    }
    fn set_unrecorded(&self, photo_id: i64, values: CompositionValues) -> Result<SavedChange> {
        let current = self.store.composition(photo_id)?;
        let frame = current.frame().ok_or(Error::FrameUnknown)?;
        validate(&values, frame).map_err(|reason| Error::CompositionInvalid {
            reason: reason.into(),
        })?;
        let before = (current.values().ok_or(Error::FrameUnknown)?, current.origin);
        let (result, changed) = self.save(photo_id, &values, Origin::User)?;
        Ok(SavedChange {
            composition: result,
            before,
            after: (values, Origin::User),
            changed,
        })
    }
    pub fn reset(&self, photo_id: i64) -> Result<Composition> {
        let current = self.store.composition(photo_id)?;
        let frame = current.frame().ok_or(Error::FrameUnknown)?;
        let values = CompositionValues {
            kind: CompositionKind::Normal,
            shape: self.default_shape,
            orientation: Orientation::Named,
            quarter_turns: 0,
            flip: Flip::None,
            straighten: 0.0,
            rect: default_rect(
                CompositionKind::Normal,
                self.default_shape,
                Orientation::Named,
                turned_frame(frame, 0),
            ),
        };
        let before = (current.values().ok_or(Error::FrameUnknown)?, current.origin);
        let (result, changed) = self.save(photo_id, &values, Origin::Default)?;
        if changed {
            self.record_one(photo_id, before, (values, Origin::Default))?;
        }
        Ok(result)
    }
    fn record_one(
        &self,
        photo_id: i64,
        before: (CompositionValues, Origin),
        after: (CompositionValues, Origin),
    ) -> Result<()> {
        let photo = self.store.photo(photo_id)?;
        let session_id = photo.session_id;
        let label = photo
            .position
            .map(|n| format!("Compose {n:02}"))
            .unwrap_or_else(|| "Compose photo".into());
        self.history.record(
            session_id,
            Step {
                label,
                kind: StepKind::Composition {
                    session_id,
                    photos: vec![CompositionChange {
                        photo_id,
                        before,
                        after,
                    }],
                },
            },
        );
        Ok(())
    }
    fn save(
        &self,
        photo_id: i64,
        values: &CompositionValues,
        origin: Origin,
    ) -> Result<(Composition, bool)> {
        let (result, changed) =
            self.store
                .save_composition(photo_id, values, origin, self.clock.now_ms())?;
        if changed {
            let session_id = self.store.photo(photo_id)?.session_id;
            self.events.emit(AppEvent::PhotoChanged {
                photo_id,
                session_id,
            });
            self.events.emit(AppEvent::SessionChanged { session_id });
        }
        Ok((result, changed))
    }
}

#[derive(Debug, Default)]
pub struct ApplyResult {
    pub applied: Vec<i64>,
    pub refused: Vec<ApplyRefusal>,
}

#[derive(Debug)]
pub struct ApplyRefusal {
    pub photo_id: i64,
    pub reason: String,
}
