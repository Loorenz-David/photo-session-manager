use crate::{
    Error, Result,
    domain::editing::CameraStatus,
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
        if self.health.has("capture_folder_not_set") {
            return Ok(CameraStatus::Disconnected);
        }
        let status = self.provider.status()?;
        match &status {
            CameraStatus::Disconnected => {
                self.health
                    .set("camera_disconnected", "No camera connected");
                self.health.clear("camera");
            }
            CameraStatus::Connected(name) => {
                self.health.clear("camera_disconnected");
                self.health.set_ok("camera", name);
            }
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
    pub fn capture(&self) -> Result<()> {
        if self.health.has("capture_folder_not_set") {
            return Err(Error::CaptureFolderNotSet);
        }
        self.provider.capture()
    }
}
