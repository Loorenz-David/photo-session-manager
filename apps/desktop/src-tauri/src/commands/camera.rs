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
            CameraStatus::NeedsAction {
                reason,
                holder,
                message,
            } => Self::NeedsAction {
                reason: reason_name(&reason).into(),
                // The same sentence as the health detail (P48 L9).
                message: message.unwrap_or_else(|| status_message(&reason, holder.as_deref())),
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
/// The id `camera_capture` answers (MC-31 §Capture; the exposure row carries it).
#[derive(Clone, Serialize, Type)]
pub struct CaptureResponse {
    pub capture_id: String,
}
#[tauri::command]
#[specta::specta]
pub async fn camera_capture(state: State<'_, AppState>) -> Result<CaptureResponse, CommandError> {
    let app = state.handles().map_err(CommandError::from)?;
    let (camera, health) = providers(&app)?;
    tauri::async_runtime::spawn_blocking(move || {
        CameraService::new(camera.as_ref(), health.as_ref()).capture()
    })
    .await
    .map_err(|e| CommandError::from(psm_core::Error::InvalidData(e.to_string())))?
    .map(|id| CaptureResponse { capture_id: id.0 })
    .map_err(Into::into)
}
/// R-I29: reopen the camera's Wi-Fi session (`reconnect_on_cable_refused` on USB).
#[tauri::command]
#[specta::specta]
pub async fn camera_reconnect(state: State<'_, AppState>) -> Result<(), CommandError> {
    let app = state.handles().map_err(CommandError::from)?;
    let (camera, health) = providers(&app)?;
    tauri::async_runtime::spawn_blocking(move || {
        CameraService::new(camera.as_ref(), health.as_ref()).reconnect()
    })
    .await
    .map_err(|e| CommandError::from(psm_core::Error::InvalidData(e.to_string())))?
    .map_err(Into::into)
}
/// R-I29: the photographer dismisses "photos taken but not received".
#[tauri::command]
#[specta::specta]
pub async fn camera_files_not_received_dismiss(
    state: State<'_, AppState>,
) -> Result<(), CommandError> {
    let app = state.handles().map_err(CommandError::from)?;
    let (camera, health) = providers(&app)?;
    let runtime = app.camera_runtime.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let ledger = runtime
            .ledger()
            .ok_or_else(|| psm_core::Error::Provider("camera ledger not bound".into()))?;
        CameraService::new(camera.as_ref(), health.as_ref())
            .with_ledger(ledger.as_ref())
            .dismiss_files_not_received()
    })
    .await
    .map_err(|e| CommandError::from(psm_core::Error::InvalidData(e.to_string())))?
    .map_err(Into::into)
}

/// What "Take the camera back" did (10E MC-34 R-I32).
#[derive(Clone, Serialize, Type)]
pub struct TakeBackResponse {
    pub completed_steps: u8,
    pub camera_held: bool,
}
/// R-I32: runs only on the photographer's click (errors `take_back_blocked{reason}`,
/// `take_back_failed{step, cause}`).
#[tauri::command]
#[specta::specta]
pub async fn camera_take_back(
    state: State<'_, AppState>,
) -> Result<TakeBackResponse, CommandError> {
    let app = state.handles().map_err(CommandError::from)?;
    let runtime = app.camera_runtime.clone();
    tauri::async_runtime::spawn_blocking(move || runtime.take_back()?.run())
        .await
        .map_err(|e| CommandError::from(psm_core::Error::InvalidData(e.to_string())))?
        .map(|r| TakeBackResponse {
            completed_steps: r.completed_steps,
            camera_held: r.camera_held,
        })
        .map_err(Into::into)
}
/// "Test camera connection" (MC-34 §Setup check): who holds the camera now and every known program
/// running that could take it.
#[derive(Clone, Serialize, Type)]
pub struct TestConnectionResponse {
    pub holder: Option<String>,
    pub programs: Vec<String>,
    pub sentence: Option<String>,
}
#[tauri::command]
#[specta::specta]
pub async fn camera_test_connection(
    state: State<'_, AppState>,
) -> Result<TestConnectionResponse, CommandError> {
    let app = state.handles().map_err(CommandError::from)?;
    let runtime = app.camera_runtime.clone();
    tauri::async_runtime::spawn_blocking(move || {
        runtime.take_back().map(|service| {
            let report = service.holder_report();
            TestConnectionResponse {
                holder: report.holder,
                programs: report.programs,
                sentence: report.sentence,
            }
        })
    })
    .await
    .map_err(|e| CommandError::from(psm_core::Error::InvalidData(e.to_string())))?
    .map_err(Into::into)
}
