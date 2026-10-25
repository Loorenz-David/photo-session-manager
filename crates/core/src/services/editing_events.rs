use crate::{
    Result,
    mechanisms::hook_payload::HookEvent,
    services::{arrival::ArrivalService, previews::PreviewService},
};
use std::sync::Arc;

pub trait EditingEvents: Send + Sync {
    fn processing_done(&self, job: &str, raw: &str, outputs: &[String]) -> Result<()>;
    fn variant_adjusted(&self, variant_id: &str, raw: &str) -> Result<()>;
}

impl EditingEvents for PreviewService {
    fn processing_done(&self, job: &str, raw: &str, outputs: &[String]) -> Result<()> {
        self.on_processing_done(job, raw, outputs)
    }
    fn variant_adjusted(&self, variant_id: &str, raw: &str) -> Result<()> {
        self.on_variant_adjusted(variant_id, raw)
    }
}

pub struct EditingEventsDispatcher {
    pub arrival: Arc<ArrivalService>,
    pub handler: Arc<dyn EditingEvents>,
}
impl EditingEventsDispatcher {
    pub fn dispatch(&self, event: &HookEvent) -> Result<()> {
        match event {
            HookEvent::CaptureDone { .. } => self.arrival.on_hook_paths(&event.candidates()),
            HookEvent::ProcessingDone { job, raw, outputs } => {
                self.handler.processing_done(job, raw, outputs)
            }
            HookEvent::VariantAdjusted { variant_id, raw } => {
                self.handler.variant_adjusted(variant_id, raw)
            }
        }
    }
}
