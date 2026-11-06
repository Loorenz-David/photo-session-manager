use crate::{commands::items::CommandError, state::AppState};
use psm_core::{
    domain::camera::{BusyReason, CameraStatus, Link, NeedsActionReason},
    mechanisms::camera_status::status_message,
    services::camera::CameraService,
};
use serde::Serialize;
use specta::Type;
use tauri::State;

#[derive(Clone, Serialize, Type)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CameraStatusResponse {
    Connected {
        name: String,
        link: String,
    },
    Connecting {
        link: String,
    },
    Busy {
        reason: String,
    },
    NeedsAction {
        reason: String,
        holder: Option<String>,
        message: String,
    },
    Disconnected,
}
impl From<CameraStatus> for CameraStatusResponse {
    fn from(value: CameraStatus) -> Self {
        match value {
            CameraStatus::Connected { name, link } => Self::Connected {
                name,
                link: link_name(link),
            },
            CameraStatus::Connecting { link } => Self::Connecting {
                link: link_name(link),
            },
            CameraStatus::Busy { reason } => Self::Busy {
                reason: busy_name(reason).into(),
            },
            CameraStatus::NeedsAction { reason, holder } => Self::NeedsAction {
                reason: reason_name(&reason).into(),
                message: status_message(&reason, holder.as_deref()),
                holder,
            },
            CameraStatus::Disconnected => Self::Disconnected,
        }
    }
}
fn link_name(link: Link) -> String {
    link.label().into()
}
fn busy_name(reason: BusyReason) -> &'static str {
    match reason {
        BusyReason::Exposure => "exposure",
        BusyReason::Transfer => "transfer",
        BusyReason::Switching => "switching",
    }
}
fn reason_name(reason: &NeedsActionReason) -> &'static str {
    match reason {
        NeedsActionReason::AuthenticationRefused => "authentication_refused",
        NeedsActionReason::FingerprintMismatch => "fingerprint_mismatch",
        NeedsActionReason::FingerprintUnconfirmed => "fingerprint_unconfirmed",
        NeedsActionReason::PhoneAppConnected => "phone_app_connected",
        NeedsActionReason::HeldByOtherProgram => "held_by_other_program",
        NeedsActionReason::StoreDestinationUnverified => "store_destination_unverified",
        NeedsActionReason::FilesNotReceived { .. } => "files_not_received",
        NeedsActionReason::SupportNotInstalled => "support_not_installed",
    }
}
fn providers(
    app: &psm_integrations::AppHandles,
) -> Result<
    (
        std::sync::Arc<dyn psm_core::ports::camera::CameraProvider>,
        std::sync::Arc<psm_core::services::health::HealthService>,
    ),
    CommandError,
> {
    Ok((app.camera.clone(), app.health.clone()))
}
#[tauri::command]
#[specta::specta]
pub async fn camera_status(
    state: State<'_, AppState>,
) -> Result<CameraStatusResponse, CommandError> {
    let app = state.handles().map_err(CommandError::from)?;
    let (camera, health) = providers(&app)?;
    tauri::async_runtime::spawn_blocking(move || {
        CameraService::new(camera.as_ref(), health.as_ref()).status()
    })
    .await
    .map_err(|e| CommandError::from(psm_core::Error::InvalidData(e.to_string())))?
    .map(Into::into)
    .map_err(Into::into)
}
#[tauri::command]
#[specta::specta]
pub async fn camera_retry(
    state: State<'_, AppState>,
) -> Result<CameraStatusResponse, CommandError> {
    let app = state.handles().map_err(CommandError::from)?;
    let (camera, health) = providers(&app)?;
    let events = app.events.clone();
    let status = tauri::async_runtime::spawn_blocking(move || {
        CameraService::new(camera.as_ref(), health.as_ref())
            .with_events(events.as_ref())
            .retry()
    })
    .await
    .map_err(|e| CommandError::from(psm_core::Error::InvalidData(e.to_string())))?
    .map_err(CommandError::from)?;
    Ok(status.into())
}
#[tauri::command]
#[specta::specta]
pub async fn camera_capture(state: State<'_, AppState>) -> Result<(), CommandError> {
    let app = state.handles().map_err(CommandError::from)?;
    let (camera, health) = providers(&app)?;
    tauri::async_runtime::spawn_blocking(move || {
        CameraService::new(camera.as_ref(), health.as_ref()).capture()
    })
    .await
    .map_err(|e| CommandError::from(psm_core::Error::InvalidData(e.to_string())))?
    .map(|_| ())
    .map_err(Into::into)
}
