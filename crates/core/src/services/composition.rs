use crate::{
    Error, Result,
    domain::composition::{
        Composition, CompositionKind, CompositionValues, Flip, Orientation, Origin, Shape,
    },
    events::AppEvent,
    mechanisms::composition::{default_rect, turned_frame, validate},
    ports::{clock::Clock, events::EventSink, store::Store},
};
use std::sync::Arc;

pub struct CompositionService {
    pub store: Arc<dyn Store>,
    pub clock: Arc<dyn Clock>,
    pub events: Arc<dyn EventSink>,
    pub default_shape: Shape,
}
impl CompositionService {
    pub fn apply(&self, items: Vec<(i64, CompositionValues)>) -> ApplyResult {
        let mut result = ApplyResult::default();
        for (photo_id, values) in items {
            match self.set(photo_id, values) {
                Ok(_) => result.applied.push(photo_id),
                Err(error) => result.refused.push(ApplyRefusal {
                    photo_id,
                    reason: match error {
                        Error::CompositionInvalid { reason } => reason,
                        other => other.to_string(),
                    },
                }),
            }
        }
        result
    }
    pub fn set(&self, photo_id: i64, values: CompositionValues) -> Result<Composition> {
        let frame = self
            .store
            .composition(photo_id)?
            .frame()
            .ok_or(Error::FrameUnknown)?;
        validate(&values, frame).map_err(|reason| Error::CompositionInvalid {
            reason: reason.into(),
        })?;
        self.save(photo_id, &values, Origin::User)
    }
    pub fn reset(&self, photo_id: i64) -> Result<Composition> {
        let frame = self
            .store
            .composition(photo_id)?
            .frame()
            .ok_or(Error::FrameUnknown)?;
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
        self.save(photo_id, &values, Origin::Default)
    }
    fn save(
        &self,
        photo_id: i64,
        values: &CompositionValues,
        origin: Origin,
    ) -> Result<Composition> {
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
        Ok(result)
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
