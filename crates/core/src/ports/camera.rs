use crate::{
    Result,
    domain::camera::{CameraCapabilities, CameraStatus, CaptureId},
};

pub trait CameraProvider: Send + Sync {
    fn raw_extensions(&self) -> &'static [&'static str];
    fn status(&self) -> Result<CameraStatus>;
    fn capture(&self) -> Result<CaptureId>;
    fn capabilities(&self) -> CameraCapabilities;
}
