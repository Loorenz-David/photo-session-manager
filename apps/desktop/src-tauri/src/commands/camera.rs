use crate::{commands::items::CommandError, state::AppState};
use psm_core::{
    domain::camera::{BusyReason, CameraStatus, Link, NeedsActionReason},
    mechanisms::camera_status::status_message,
    services::camera::CameraService,
};
use serde::Serialize;
use specta::Type;
use std::path::PathBuf;
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
        link: Option<String>,
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
                link: None,
            },
            CameraStatus::Disconnected => Self::Disconnected,
        }
    }
}
impl CameraStatusResponse {
    pub fn with_link(mut self, link: Option<Link>) -> Self {
        if let Self::NeedsAction {
            link: destination, ..
        } = &mut self
        {
            *destination = link.map(link_name);
        }
        self
    }
}
#[derive(Clone, Serialize, Type)]
pub struct CameraSupportDto {
    pub make: String,
    pub version: String,
    pub sidecar_path: String,
}
impl From<psm_integrations::camera::support::InstalledSupport> for CameraSupportDto {
    fn from(value: psm_integrations::camera::support::InstalledSupport) -> Self {
        Self {
            make: value.make.into(),
            version: value.version,
            sidecar_path: value.sidecar_path.to_string_lossy().into_owned(),
        }
    }
}
#[derive(Clone, Serialize, Type)]
pub struct FtpPasswordDto {
    pub password: String,
}

#[tauri::command]
#[specta::specta]
pub async fn camera_support_get(
    state: State<'_, AppState>,
) -> Result<Option<CameraSupportDto>, CommandError> {
    let handles = state.handles()?;
    Ok(psm_integrations::camera::support::installed(&handles.paths).map(Into::into))
}
#[tauri::command]
#[specta::specta]
pub async fn camera_support_install(
    state: State<'_, AppState>,
    archive_path: String,
) -> Result<CameraSupportDto, CommandError> {
    let handles = state.handles()?;
    let bundled = psm_integrations::camera::support::bundled_for(&handles.paths)
        .ok_or(psm_core::Error::SupportNotInstalled)?;
    let installed = tauri::async_runtime::spawn_blocking(move || {
        psm_integrations::camera::support::sony::install(
            &handles.paths,
            &PathBuf::from(archive_path),
            &bundled,
        )
    })
    .await
    .map_err(|e| psm_core::Error::InvalidData(e.to_string()))??;
    state.mutate(|app| {
        app.camera_runtime.support_changed();
        app.reopen()
    })?;
    Ok(installed.into())
}
#[tauri::command]
#[specta::specta]
pub async fn camera_secrets_set(
    state: State<'_, AppState>,
    access_auth_user: String,
    access_auth_password: String,
) -> Result<(), CommandError> {
    let handles = state.handles()?;
    tauri::async_runtime::spawn_blocking(move || {
        let secrets = handles.camera_runtime.secret_store();
        psm_infra::logging::add_secret(&access_auth_user);
        psm_infra::logging::add_secret(&access_auth_password);
        CameraService::set_secrets(
            secrets.as_ref(),
            &access_auth_user,
            &access_auth_password,
            || handles.camera_runtime.secrets_changed(),
        )
    })
    .await
    .map_err(|e| psm_core::Error::InvalidData(e.to_string()))??;
    Ok(())
}
#[tauri::command]
#[specta::specta]
pub async fn camera_ftp_password_generate(
    state: State<'_, AppState>,
) -> Result<FtpPasswordDto, CommandError> {
    let handles = state.handles()?;
    let password = tauri::async_runtime::spawn_blocking(move || {
        let secrets = handles.camera_runtime.secret_store();
        let mut bytes = [0u8; 64];
        let mut pos = bytes.len();
        CameraService::generate_ftp_password(
            secrets.as_ref(),
            || {
                if pos >= bytes.len() {
                    getrandom::fill(&mut bytes).expect("system RNG");
                    pos = 0;
                }
                let byte = bytes[pos];
                pos += 1;
                byte
            },
            |password| {
                psm_infra::logging::add_secret(password);
                handles.camera_runtime.ftp_password_changed();
            },
        )
    })
    .await
    .map_err(|e| psm_core::Error::InvalidData(e.to_string()))??;
    psm_infra::logging::add_secret(&password);
    Ok(FtpPasswordDto { password })
}
#[tauri::command]
#[specta::specta]
pub async fn camera_fingerprint_confirm(
    state: State<'_, AppState>,
    fingerprint: String,
) -> Result<(), CommandError> {
    let handles = state.handles()?;
    CameraService::confirm_fingerprint(|| {
        handles
            .camera_runtime
            .direct_adapter()
            .confirm_fingerprint(&fingerprint)
    })
    .map_err(Into::into)
}
#[tauri::command]
#[specta::specta]
pub async fn camera_fingerprint_reject(state: State<'_, AppState>) -> Result<(), CommandError> {
    let handles = state.handles()?;
    CameraService::reject_fingerprint(|| {
        handles.camera_runtime.direct_adapter().reject_fingerprint()
    })
    .map_err(Into::into)
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
        NeedsActionReason::FtpUnreachable => "ftp_unreachable",
        NeedsActionReason::FilesNotReceived { .. } => "files_not_received",
        NeedsActionReason::SupportNotInstalled => "support_not_installed",
        NeedsActionReason::DocumentMissing => "document_missing",
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
    let runtime = app.camera_runtime.clone();
    tauri::async_runtime::spawn_blocking(move || {
        CameraService::new(camera.as_ref(), health.as_ref()).status()
    })
    .await
    .map_err(|e| CommandError::from(psm_core::Error::InvalidData(e.to_string())))?
    .map(|status| {
        CameraStatusResponse::from(status).with_link(runtime.direct_adapter().current_link())
    })
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
    let runtime = app.camera_runtime.clone();
    let direct = runtime.direct_adapter();
    let status = tauri::async_runtime::spawn_blocking(move || {
        let status = if camera.capabilities().depends_on_editing_provider {
            CameraService::new(camera.as_ref(), health.as_ref()).retry()?
        } else {
            direct.retry()?
        };
        psm_core::ports::events::EventSink::emit(
            events.as_ref(),
            psm_core::events::AppEvent::CameraStatusChanged {
                status: status.clone(),
            },
        );
        Ok::<_, psm_core::Error>(status)
    })
    .await
    .map_err(|e| CommandError::from(psm_core::Error::InvalidData(e.to_string())))?
    .map_err(CommandError::from)?;
    Ok(CameraStatusResponse::from(status).with_link(runtime.direct_adapter().current_link()))
}
#[derive(Clone, Serialize, Type)]
pub struct TakeBackAvailability {
    pub reason: Option<String>,
}
#[tauri::command]
#[specta::specta]
pub async fn camera_take_back_availability(
    state: State<'_, AppState>,
) -> Result<TakeBackAvailability, CommandError> {
    let app = state.handles().map_err(CommandError::from)?;
    let runtime = app.camera_runtime.clone();
    tauri::async_runtime::spawn_blocking(move || {
        Ok::<_, psm_core::Error>(TakeBackAvailability {
            reason: runtime.take_back()?.blocked_reason()?,
        })
    })
    .await
    .map_err(|e| CommandError::from(psm_core::Error::InvalidData(e.to_string())))?
    .map_err(Into::into)
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

/// Redacted recent helper output, kept in memory by the camera runtime.
#[tauri::command]
#[specta::specta]
pub async fn camera_helper_log(state: State<'_, AppState>) -> Result<Vec<String>, CommandError> {
    let app = state.handles().map_err(CommandError::from)?;
    Ok(app.camera_runtime.helper_log())
}
