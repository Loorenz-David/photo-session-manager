use crate::{commands::items::CommandError, state::AppState};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::State;

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

/// `settings.json` as shown in the settings screen (ch12 §Settings). `preview_delay_s` is the
/// stored value; `effective_preview_delay_s` is the value clamped to 10–30 s (MC-17).
#[derive(Serialize, Deserialize, Type)]
pub struct SettingsDto {
    pub document_id: Option<String>,
    pub preview_recipe: String,
    pub web_recipe: String,
    pub style_name: String,
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
    pub document_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview_recipe: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub web_recipe: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style_name: Option<String>,
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
pub fn settings_get(state: State<'_, AppState>) -> Result<SettingsDto, CommandError> {
    let app = state
        .0
        .lock()
        .map_err(|e| psm_core::Error::InvalidData(e.to_string()))?;
    to_dto(app.settings_get()?)
}

#[tauri::command]
#[specta::specta]
pub fn settings_set(
    state: State<'_, AppState>,
    patch: SettingsPatchDto,
) -> Result<SettingsDto, CommandError> {
    let mut app = state
        .0
        .lock()
        .map_err(|e| psm_core::Error::InvalidData(e.to_string()))?;
    let patch =
        serde_json::to_value(&patch).map_err(|e| psm_core::Error::InvalidData(e.to_string()))?;
    to_dto(app.settings_set(&patch)?)
}
