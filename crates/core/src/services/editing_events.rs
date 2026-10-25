use crate::{Result, mechanisms::hook_payload::HookEvent, services::arrival::ArrivalService};
use std::sync::Arc;

pub trait EditingEvents: Send + Sync {
    fn processing_done(&self, job: &str, raw: &str, outputs: &[String]) -> Result<()>;
    fn variant_adjusted(&self, variant_id: &str, raw: &str) -> Result<()>;
}

pub struct LoggingEditingEvents;
impl EditingEvents for LoggingEditingEvents {
    fn processing_done(&self, job: &str, raw: &str, _outputs: &[String]) -> Result<()> {
        tracing::info!(job, raw, "processing_done awaits P06");
        Ok(())
    }
    fn variant_adjusted(&self, variant_id: &str, raw: &str) -> Result<()> {
        tracing::info!(variant_id, raw, "variant_adjusted awaits P06");
        Ok(())
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
