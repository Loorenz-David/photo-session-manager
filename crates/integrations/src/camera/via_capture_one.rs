use crate::editing::capture_one::scripting::CaptureOneScripting;
use psm_core::{
    Result,
    domain::editing::{CameraCapabilities, CameraStatus},
    ports::camera::CameraProvider,
};
use std::sync::Arc;

pub struct CameraViaCaptureOne {
    pub scripting: Arc<dyn CaptureOneScripting>,
    pub document_id: String,
}
impl CameraProvider for CameraViaCaptureOne {
    fn status(&self) -> Result<CameraStatus> {
        Ok(self
            .scripting
            .document_info(&self.document_id)?
            .camera
            .map(CameraStatus::Connected)
            .unwrap_or(CameraStatus::Disconnected))
    }
    fn capture(&self) -> Result<()> {
        self.scripting.capture()
    }
    fn capabilities(&self) -> CameraCapabilities {
        CameraCapabilities {
            status: true,
            capture: true,
            transfer: true,
        }
    }
}
