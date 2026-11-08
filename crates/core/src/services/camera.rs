use crate::{
    Error, Result,
    domain::camera::secrets,
    domain::camera::{CameraStatus, CaptureId, Link},
    events::AppEvent,
    mechanisms::secrets::ftp_password,
    ports::secrets::SecretStore,
    ports::{camera::CameraProvider, events::EventSink},
    services::{camera_ledger::LedgerService, health::HealthService},
};

pub struct CameraService<'a> {
    provider: &'a dyn CameraProvider,
    health: &'a HealthService,
    events: Option<&'a dyn EventSink>,
    ledger: Option<&'a LedgerService>,
}
impl<'a> CameraService<'a> {
    pub fn set_secrets(
        secrets_store: &dyn SecretStore,
        user: &str,
        password: &str,
        changed: impl FnOnce(),
    ) -> Result<()> {
        secrets_store.set(secrets::ACCESS_AUTH_USER, user)?;
        secrets_store.set(secrets::ACCESS_AUTH_PASSWORD, password)?;
        changed();
        Ok(())
    }
    pub fn generate_ftp_password(
        secrets_store: &dyn SecretStore,
        rng: impl FnMut() -> u8,
        changed: impl FnOnce(&str),
    ) -> Result<String> {
        let password = ftp_password(rng);
        secrets_store.set(secrets::FTP_PASSWORD, &password)?;
        changed(&password);
        Ok(password)
    }
    pub fn confirm_fingerprint(confirm: impl FnOnce() -> Result<()>) -> Result<()> {
        confirm()
    }
    pub fn reject_fingerprint(reject: impl FnOnce() -> Result<()>) -> Result<()> {
        reject()
    }
    pub fn new(provider: &'a dyn CameraProvider, health: &'a HealthService) -> Self {
        Self {
            provider,
            health,
            events: None,
            ledger: None,
        }
    }
    pub fn with_events(mut self, events: &'a dyn EventSink) -> Self {
        self.events = Some(events);
        self
    }
    /// The capture ledger of the camera runtime's current binding (P46: the dismiss path).
    pub fn with_ledger(mut self, ledger: &'a LedgerService) -> Self {
        self.ledger = Some(ledger);
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
    /// R-I29: reopen the camera session; the provider refuses it off Wi-Fi.
    pub fn reconnect(&self) -> Result<()> {
        self.provider.reconnect()
    }
    /// R-I29: the photographer dismissed "photos taken but not received": those exposures
    /// become `dismissed` and the notice clears.
    pub fn dismiss_files_not_received(&self) -> Result<()> {
        self.ledger
            .ok_or_else(|| Error::Provider("camera ledger not bound".into()))?
            .dismiss()
    }
}
