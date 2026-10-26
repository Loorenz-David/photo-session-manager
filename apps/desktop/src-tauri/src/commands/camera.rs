use crate::{commands::items::CommandError, state::AppState};
use psm_core::{
    domain::editing::CameraStatus, events::AppEvent, ports::events::EventSink,
    services::camera::CameraService,
};
use serde::Serialize;
use specta::Type;
use tauri::{AppHandle, Emitter, State};

#[derive(Clone, Serialize, Type)]
#[serde(tag = "kind", content = "name", rename_all = "snake_case")]
pub enum CameraStatusResponse {
    Connected(String),
    Disconnected,
}
impl From<CameraStatus> for CameraStatusResponse {
    fn from(value: CameraStatus) -> Self {
        match value {
            CameraStatus::Connected(name) => Self::Connected(name),
            CameraStatus::Disconnected => Self::Disconnected,
        }
    }
}
struct TauriCameraEvents(AppHandle);
impl EventSink for TauriCameraEvents {
    fn emit(&self, event: AppEvent) {
        if let AppEvent::CameraStatusChanged { status } = event {
            let _ = self
                .0
                .emit("camera_status_changed", CameraStatusResponse::from(status));
        }
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
    app_handle: AppHandle,
) -> Result<CameraStatusResponse, CommandError> {
    let app = state.handles().map_err(CommandError::from)?;
    let (camera, health) = providers(&app)?;
    let status = tauri::async_runtime::spawn_blocking(move || {
        let events = TauriCameraEvents(app_handle);
        CameraService::new(camera.as_ref(), health.as_ref())
            .with_events(&events)
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
    .map_err(Into::into)
}
