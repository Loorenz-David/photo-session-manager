use crate::{commands::items::CommandError, state::AppState};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Manager, State};

/// The `integration` block as shown and as patched (09A MC-23/MC-24). On read the client
/// secret is masked (`••••` + last 4); sending the masked value back keeps the stored secret.
#[derive(Serialize, Deserialize, Type, Clone)]
pub struct IntegrationSettingsDto {
    pub kind: String,
    pub shop_domain: String,
    pub api_version: String,
    pub client_id: String,
    pub client_secret: String,
    pub condition_photos_key: String,
}

/// The output setting (10D MC-28): the long edge in pixels (512–8192) and the JPEG quality
/// (60–100) of every exported or uploaded file; JPEG, 4:4:4 and sRGB are fixed.
#[derive(Serialize, Deserialize, Type, Clone)]
pub struct OutputFormatDto {
    pub long_edge: u32,
    pub quality: u8,
}
#[derive(Serialize, Deserialize, Type, Clone)]
pub struct CameraSettingsDto {
    pub receiver_port: u16,
    pub passive_port_start: u16,
    pub passive_port_end: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_take_back: Option<bool>,
}

/// `settings.json` as shown in the settings screen (ch12 §Settings). `preview_delay_s` is the
/// stored value; `effective_preview_delay_s` is the value clamped to 10–30 s (MC-17).
#[derive(Serialize, Deserialize, Type)]
pub struct SettingsDto {
    pub camera: CameraSettingsDto,
    pub document_id: Option<String>,
    pub effective_document_id: String,
    pub default_document_id: String,
    pub preview_recipe: String,
    pub output_format: OutputFormatDto,
    pub style_name: String,
    pub default_shape: String,
    pub master_recipe: String,
    pub export_folder: String,
    pub preview_delay_s: u32,
    pub effective_preview_delay_s: u32,
    pub theme: String,
    pub sku_prefix: Option<String>,
    pub integration: Option<IntegrationSettingsDto>,
}

/// A partial update: absent fields stay as stored.
#[derive(Serialize, Deserialize, Type, Default)]
pub struct SettingsPatchDto {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub camera: Option<CameraSettingsDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview_recipe: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_format: Option<OutputFormatDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_shape: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub master_recipe: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub export_folder: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview_delay_s: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sku_prefix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub integration: Option<IntegrationSettingsDto>,
}

fn to_dto(value: serde_json::Value) -> Result<SettingsDto, CommandError> {
    serde_json::from_value(value).map_err(|e| psm_core::Error::InvalidData(e.to_string()).into())
}

#[tauri::command]
#[specta::specta]
pub async fn settings_get(state: State<'_, AppState>) -> Result<SettingsDto, CommandError> {
    let app = state.handles().map_err(CommandError::from)?;
    tauri::async_runtime::spawn_blocking(move || -> Result<SettingsDto, CommandError> {
        to_dto(app.settings_get()?)
    })
    .await
    .map_err(|e| CommandError::from(psm_core::Error::InvalidData(e.to_string())))?
}

#[tauri::command]
#[specta::specta]
pub async fn settings_set(
    _state: State<'_, AppState>,
    app_handle: AppHandle,
    patch: SettingsPatchDto,
) -> Result<SettingsDto, CommandError> {
    tauri::async_runtime::spawn_blocking(move || {
        let patch = serde_json::to_value(&patch)
            .map_err(|e| psm_core::Error::InvalidData(e.to_string()))?;
        let state = app_handle.state::<AppState>();
        to_dto(state.settings_set(&patch)?)
    })
    .await
    .map_err(|e| CommandError::from(psm_core::Error::InvalidData(e.to_string())))?
}

/// Settings → Storage (10F §4): free space on the disk holding the capture root, the space each
/// kind of file uses (computed in the background; `computed` false until the first pass), the
/// minimum free space and, when levels 1–3 cannot reach it, the warning shown in the header.
#[derive(Serialize, Deserialize, Type)]
pub struct StorageDto {
    pub computed: bool,
    pub disk: String,
    pub free_bytes: Option<f64>,
    pub raws_bytes: f64,
    pub duplicates_bytes: f64,
    pub preview_exports_bytes: f64,
    pub preview_cache_bytes: f64,
    pub masters_bytes: f64,
    pub min_free_gb: u32,
    pub warning: Option<String>,
}

fn storage_dto(app: &psm_integrations::AppHandles) -> StorageDto {
    let report = app.storage.report();
    StorageDto {
        computed: report.is_some(),
        disk: report.as_ref().map(|r| r.disk.clone()).unwrap_or_default(),
        free_bytes: report.as_ref().and_then(|r| r.free_bytes).map(|b| b as f64),
        raws_bytes: report.as_ref().map_or(0.0, |r| r.raws_bytes as f64),
        duplicates_bytes: report.as_ref().map_or(0.0, |r| r.duplicates_bytes as f64),
        preview_exports_bytes: report
            .as_ref()
            .map_or(0.0, |r| r.preview_exports_bytes as f64),
        preview_cache_bytes: report
            .as_ref()
            .map_or(0.0, |r| r.preview_cache_bytes as f64),
        masters_bytes: report.as_ref().map_or(0.0, |r| r.masters_bytes as f64),
        min_free_gb: app.settings.storage.min_free_gb,
        warning: app.storage.warning(),
    }
}

#[tauri::command]
#[specta::specta]
pub async fn storage_get(state: State<'_, AppState>) -> Result<StorageDto, CommandError> {
    let app = state.handles().map_err(CommandError::from)?;
    Ok(storage_dto(&app))
}

/// Saves the minimum free space (GB); the background job applies it at its next tick.
#[tauri::command]
#[specta::specta]
pub async fn storage_min_free_set(
    _state: State<'_, AppState>,
    app_handle: AppHandle,
    min_free_gb: u32,
) -> Result<StorageDto, CommandError> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app_handle.state::<AppState>();
        state.settings_set(&serde_json::json!({"storage": {"min_free_gb": min_free_gb}}))?;
        let app = state.handles()?;
        Ok(storage_dto(&app))
    })
    .await
    .map_err(|e| CommandError::from(psm_core::Error::InvalidData(e.to_string())))?
}
