//! Settings → Phone (`master_plan_live.md` §4.5, §5 PH1): pair a phone (QR code), list, rename and
//! remove pairings, and the phone server's state. Thin commands over `psm_integrations::phone`.
use std::sync::{Arc, OnceLock};

use psm_core::domain::live::PhoneSettings;
use psm_integrations::phone::{
    host::{HostInfo, SystemHost, pairing_links},
    pairing::PhonePairings,
    server::PhoneServer,
};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::State;

use crate::{
    commands::items::{CommandError, CommandId},
    state::AppState,
};

/// The phone remote's process state: where the Mac's name and address come from.
pub struct PhoneState {
    pub host: Arc<dyn HostInfo>,
}

impl PhoneState {
    /// This Mac's name and address.
    pub fn system() -> Self {
        Self {
            host: Arc::new(SystemHost),
        }
    }
    /// Given name and address (tests).
    #[cfg(test)]
    pub fn with_host(host: Arc<dyn HostInfo>) -> Self {
        Self { host }
    }
}

/// PH2: the app's one phone server, started in the desktop setup after the first App.
static SERVER: OnceLock<Arc<PhoneServer>> = OnceLock::new();

/// Keeps the server the setup started (once per process).
pub fn set_server(server: Arc<PhoneServer>) {
    let _ = SERVER.set(server);
}

/// The phone server, once started.
pub fn server() -> Option<Arc<PhoneServer>> {
    SERVER.get().cloned()
}

/// §4.5: a saved `phone` setting rebinds (port) or stops/starts (switch) the server.
pub fn settings_saved(settings: &serde_json::Value) {
    if let (Some(server), Some(phone)) = (
        server(),
        settings
            .get("phone")
            .and_then(|v| serde_json::from_value::<PhoneSettings>(v.clone()).ok()),
    ) {
        server.apply(phone);
    }
}

/// A pairing just made. The token is in the two URLs and QR codes only, shown once.
#[derive(Serialize, Deserialize, Type)]
pub struct PairingCreatedDto {
    #[specta(type = specta_typescript::Number)]
    pub id: i64,
    pub name: String,
    pub url_name: String,
    pub url_address: String,
    pub qr_name_svg: String,
    pub qr_address_svg: String,
}

/// One pairing in the list (never its token).
#[derive(Serialize, Deserialize, Type)]
pub struct PairingDto {
    #[specta(type = specta_typescript::Number)]
    pub id: i64,
    pub name: String,
    #[specta(type = specta_typescript::Number)]
    pub created_at: i64,
    #[specta(type = Option<specta_typescript::Number>)]
    pub last_seen_at: Option<i64>,
    /// A socket of this phone is open now.
    pub connected: bool,
}

/// The phone server: switched on (`phone.enabled`), listening, its port, and why it could not
/// start.
#[derive(Serialize, Deserialize, Type)]
pub struct PhoneServerStatusDto {
    pub enabled: bool,
    pub running: bool,
    pub port: u16,
    pub error: Option<String>,
}

fn blocking_error(e: impl std::fmt::Display) -> CommandError {
    CommandError::from(psm_core::Error::InvalidData(e.to_string()))
}

/// PH1: a new pairing (named, or "Phone <n>") with the URLs and QR codes that open the phone app.
#[tauri::command]
#[specta::specta]
pub async fn phone_pairing_create(
    state: State<'_, AppState>,
    phone: State<'_, PhoneState>,
    name: Option<String>,
) -> Result<PairingCreatedDto, CommandError> {
    let app = state.handles().map_err(CommandError::from)?;
    let host = phone.host.clone();
    tauri::async_runtime::spawn_blocking(move || -> Result<PairingCreatedDto, CommandError> {
        let created =
            PhonePairings::new(app.store.as_ref(), app.clock.as_ref()).create(name.as_deref())?;
        let links = pairing_links(host.as_ref(), app.settings.phone.port, &created.token);
        Ok(PairingCreatedDto {
            id: created.id,
            name: created.name,
            url_name: links.url_name,
            url_address: links.url_address,
            qr_name_svg: links.qr_name_svg,
            qr_address_svg: links.qr_address_svg,
        })
    })
    .await
    .map_err(blocking_error)?
}

/// PH1: the pairings not removed, oldest first.
#[tauri::command]
#[specta::specta]
pub async fn phone_pairings_list(
    state: State<'_, AppState>,
) -> Result<Vec<PairingDto>, CommandError> {
    let app = state.handles().map_err(CommandError::from)?;
    tauri::async_runtime::spawn_blocking(move || -> Result<Vec<PairingDto>, CommandError> {
        let connected = server().map(|s| s.connected_pairings()).unwrap_or_default();
        Ok(PhonePairings::new(app.store.as_ref(), app.clock.as_ref())
            .list()?
            .into_iter()
            .map(|p| PairingDto {
                connected: connected.contains(&p.id),
                id: p.id,
                name: p.name,
                created_at: p.created_at,
                last_seen_at: p.last_seen_at,
            })
            .collect())
    })
    .await
    .map_err(blocking_error)?
}

#[tauri::command]
#[specta::specta]
pub async fn phone_pairing_rename(
    state: State<'_, AppState>,
    id: CommandId,
    name: String,
) -> Result<(), CommandError> {
    let app = state.handles().map_err(CommandError::from)?;
    tauri::async_runtime::spawn_blocking(move || -> Result<(), CommandError> {
        Ok(PhonePairings::new(app.store.as_ref(), app.clock.as_ref()).rename(id.0, &name)?)
    })
    .await
    .map_err(blocking_error)?
}

/// PH1: Remove — the phone's token stops working.
#[tauri::command]
#[specta::specta]
pub async fn phone_pairing_revoke(
    state: State<'_, AppState>,
    id: CommandId,
) -> Result<(), CommandError> {
    let app = state.handles().map_err(CommandError::from)?;
    tauri::async_runtime::spawn_blocking(move || -> Result<(), CommandError> {
        PhonePairings::new(app.store.as_ref(), app.clock.as_ref()).revoke(id.0)?;
        // PH1: that phone's open sockets close (4401).
        if let Some(server) = server() {
            server.close_pairing(id.0);
        }
        Ok(())
    })
    .await
    .map_err(blocking_error)?
}

/// PH2: whether the phone server is switched on and listening, its port, and its bind error.
#[tauri::command]
#[specta::specta]
pub async fn phone_server_status(
    state: State<'_, AppState>,
) -> Result<PhoneServerStatusDto, CommandError> {
    let app = state.handles().map_err(CommandError::from)?;
    Ok(match server() {
        Some(server) => {
            let status = server.status();
            PhoneServerStatusDto {
                enabled: status.enabled,
                running: status.running,
                port: status.port,
                error: status.error,
            }
        }
        None => PhoneServerStatusDto {
            enabled: app.settings.phone.enabled,
            running: false,
            port: app.settings.phone.port,
            error: None,
        },
    })
}
