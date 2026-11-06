use crate::{
    Result,
    domain::camera::{
        CameraCapabilities, CameraStatus, CaptureId, Delivery, DeliveryOutcome, ExposureObserved,
        Link,
    },
};

pub trait CameraProvider: Send + Sync {
    fn raw_extensions(&self) -> &'static [&'static str];
    fn status(&self) -> Result<CameraStatus>;
    fn capture(&self) -> Result<CaptureId>;
    fn capabilities(&self) -> CameraCapabilities;
}
pub trait CaptureSink: Send + Sync {
    fn exposure_observed(&self, e: ExposureObserved) -> Result<i64>;
    fn delivery(&self, d: Delivery) -> Result<DeliveryOutcome>;
    fn session_uncertain(&self) -> Result<()>;
    fn transfer_in_progress(&self, source: Link, active: bool);
}
pub trait ArrivalSource: Send + Sync {
    fn via(&self) -> Link;
    fn transfer_in_progress(&self) -> bool;
}
