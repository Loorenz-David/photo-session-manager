use crate::editing::capture_one::scripting::CaptureOneScripting;
use psm_core::{
    Result,
    domain::camera::{CameraCapabilities, CameraStatus, CaptureId, Link},
    ports::camera::CameraProvider,
};
use std::sync::Arc;

pub struct CameraViaCaptureOne {
    pub scripting: Arc<dyn CaptureOneScripting>,
    pub document_id: String,
}
impl CameraProvider for CameraViaCaptureOne {
    fn raw_extensions(&self) -> &'static [&'static str] {
        &["arw"]
    }
    fn status(&self) -> Result<CameraStatus> {
        Ok(self
            .scripting
            .document_info(&self.document_id)?
            .camera
            .map(|name| CameraStatus::Connected {
                name,
                link: Link::Cable,
            })
            .unwrap_or(CameraStatus::Disconnected))
    }
    fn capture(&self) -> Result<CaptureId> {
        self.scripting.capture()?;
        Ok(CaptureId(uuid::Uuid::new_v4().to_string()))
    }
    fn capabilities(&self) -> CameraCapabilities {
        CameraCapabilities {
            status: true,
            capture: true,
            transfer: true,
            exposure_events: false,
            settings_read: false,
            settings_write_store_destination: false,
            depends_on_editing_provider: true,
        }
    }
}
