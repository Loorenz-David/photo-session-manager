pub mod runtime;
pub mod sidecar;
pub mod support;
pub mod via_capture_one;

use psm_core::{
    Result,
    domain::camera::{Delivery, DeliveryOutcome, ExposureObserved, Link},
    ports::camera::CaptureSink,
};
pub struct NoopSink;
impl CaptureSink for NoopSink {
    fn exposure_observed(&self, _: ExposureObserved) -> Result<i64> {
        Ok(0)
    }
    fn delivery(&self, d: Delivery) -> Result<DeliveryOutcome> {
        tracing::debug!(name=%d.camera_filename, "camera delivery waiting for ledger");
        Ok(DeliveryOutcome::Photo)
    }
    fn session_uncertain(&self) -> Result<()> {
        Ok(())
    }
    fn transfer_in_progress(&self, _: Link, _: bool) {}
}
