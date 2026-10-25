use crate::{
    Result,
    domain::editing::{CameraCapabilities, CameraStatus},
};

pub trait CameraProvider: Send + Sync {
    fn status(&self) -> Result<CameraStatus>;
    fn capture(&self) -> Result<()>;
    fn capabilities(&self) -> CameraCapabilities;
}
