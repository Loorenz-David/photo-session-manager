use crate::{
    Error, Result,
    domain::camera::{CameraStatus, CaptureId, Link},
    events::AppEvent,
    ports::{camera::CameraProvider, events::EventSink},
    services::health::HealthService,
};

pub struct CameraService<'a> {
    provider: &'a dyn CameraProvider,
    health: &'a HealthService,
    events: Option<&'a dyn EventSink>,
}
impl<'a> CameraService<'a> {
    pub fn new(provider: &'a dyn CameraProvider, health: &'a HealthService) -> Self {
        Self {
            provider,
            health,
            events: None,
        }
    }
    pub fn with_events(mut self, events: &'a dyn EventSink) -> Self {
        self.events = Some(events);
        self
    }
    pub fn status(&self) -> Result<CameraStatus> {
        if self.provider.capabilities().depends_on_editing_provider
            && (self.health.has("capture_folder_not_set") || self.health.has("document_not_front"))
        {
            return Ok(CameraStatus::Disconnected);
        }
        let status = self.provider.status()?;
        match &status {
            CameraStatus::Disconnected => {
                self.health
                    .set("camera_disconnected", "No camera connected");
                self.health.clear("camera");
            }
            CameraStatus::Connected { name, link } => {
                self.health.clear("camera_disconnected");
                self.health.set_ok(
                    "camera",
                    format!(
                        "{name} · {}",
                        match link {
                            Link::Cable => "cable",
                            Link::Wireless => "wireless",
                        }
                    ),
                );
            }
            CameraStatus::Connecting { .. }
            | CameraStatus::Busy { .. }
            | CameraStatus::NeedsAction { .. } => {}
        }
        Ok(status)
    }
    pub fn retry(&self) -> Result<CameraStatus> {
        let status = self.status()?;
        if let Some(events) = self.events {
            events.emit(AppEvent::CameraStatusChanged {
                status: status.clone(),
            });
        }
        Ok(status)
    }
    pub fn capture(&self) -> Result<CaptureId> {
        if self.provider.capabilities().depends_on_editing_provider
            && self.health.has("document_not_front")
        {
            return Err(Error::DocumentNotFront);
        }
        if self.provider.capabilities().depends_on_editing_provider
            && self.health.has("capture_folder_not_set")
        {
            return Err(Error::CaptureFolderNotSet);
        }
        self.provider.capture()
    }
}
