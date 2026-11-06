use psm_core::ports::camera::CompletedNames;
use psm_core::{
    Result,
    domain::camera::{Delivery, DeliveryOutcome, ExposureObserved, Link},
    ports::camera::CaptureSink,
};
use std::sync::Mutex;

#[derive(Default)]
pub struct CollectingSink {
    deliveries: Mutex<Vec<Delivery>>,
    names: Mutex<Vec<(String, u64, i64)>>,
}
impl CollectingSink {
    pub fn deliveries(&self) -> Vec<Delivery> {
        self.deliveries.lock().unwrap().clone()
    }
    pub fn set_completed_names(&self, names: Vec<(String, u64, i64)>) {
        *self.names.lock().unwrap() = names;
    }
}
impl CaptureSink for CollectingSink {
    fn exposure_observed(&self, _: ExposureObserved) -> Result<i64> {
        Ok(0)
    }
    fn delivery(&self, d: Delivery) -> Result<DeliveryOutcome> {
        self.deliveries.lock().unwrap().push(d);
        Ok(DeliveryOutcome::Photo)
    }
    fn session_uncertain(&self) -> Result<()> {
        Ok(())
    }
    fn transfer_in_progress(&self, _: Link, _: bool) {}
}
impl CompletedNames for CollectingSink {
    fn completed_names_since(&self, since: i64) -> Vec<(String, u64, i64)> {
        self.names
            .lock()
            .unwrap()
            .iter()
            .filter(|(_, _, at)| *at >= since)
            .cloned()
            .collect()
    }
}
