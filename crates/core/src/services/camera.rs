use std::net::IpAddr;

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

/// Health code and header sentence when this Mac's address is not the one the camera last
/// uploaded to (P68). The address is shown in the UI only, never logged.
pub const ADDRESS_CHANGED: &str = "camera_address_changed";
pub fn address_changed_sentence(address: &str) -> String {
    format!("This Mac's address changed to {address} — set Host Name on the camera")
}

/// This Mac's address for the camera's Host Name (P68): the address the camera last uploaded to
/// while this Mac still has it, else the first private IPv4 address, else the first address.
/// `changed` when the camera uploaded before and that address is no longer this Mac's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FtpAddress {
    pub address: Option<IpAddr>,
    pub changed: bool,
}
impl FtpAddress {
    pub fn of(current: &[IpAddr], last_upload: Option<IpAddr>) -> Self {
        if let Some(last) = last_upload.filter(|last| current.contains(last)) {
            return Self {
                address: Some(last),
                changed: false,
            };
        }
        let rank = |ip: &IpAddr| match ip {
            IpAddr::V4(v4) if v4.is_private() => 0,
            IpAddr::V4(_) => 1,
            IpAddr::V6(_) => 2,
        };
        let address = current.iter().copied().min_by_key(rank);
        Self {
            address,
            changed: last_upload.is_some() && address.is_some(),
        }
    }
    /// The sentence for the header and Health, when the address changed.
    pub fn warning(&self) -> Option<String> {
        self.address
            .filter(|_| self.changed)
            .map(|address| address_changed_sentence(&address.to_string()))
    }
}

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
    /// "Show password" (P68): the FTP password stored in the Keychain, if any.
    pub fn ftp_password(secrets_store: &dyn SecretStore) -> Result<Option<String>> {
        secrets_store.get(secrets::FTP_PASSWORD)
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
