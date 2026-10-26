use crate::state::AppState;
use serde::Serialize;
use specta::Type;
use tauri::State;

#[derive(Serialize, Type)]
pub struct HealthItemDto {
    pub code: String,
    pub ok: bool,
    pub detail: String,
}

#[derive(Serialize, Type)]
pub struct HealthReportDto {
    pub items: Vec<HealthItemDto>,
}

#[derive(Serialize, Type)]
pub struct AppPathsDto {
    pub data_dir: String,
    pub database: String,
    pub settings: String,
}

#[tauri::command]
#[specta::specta]
pub fn health_get(state: State<'_, AppState>) -> Result<HealthReportDto, String> {
    let app = state.0.lock().map_err(|e| e.to_string())?;
    let report = app.health_check().map_err(|e| e.to_string())?;
    Ok(HealthReportDto {
        items: report
            .items
            .into_iter()
            .map(|i| HealthItemDto {
                code: i.code,
                ok: i.ok,
                detail: i.detail,
            })
            .collect(),
    })
}

#[tauri::command]
#[specta::specta]
pub fn app_paths_get(state: State<'_, AppState>) -> Result<AppPathsDto, String> {
    let app = state.0.lock().map_err(|e| e.to_string())?;
    Ok(AppPathsDto {
        data_dir: app.paths.root.to_string_lossy().into_owned(),
        database: app.paths.db().to_string_lossy().into_owned(),
        settings: app.paths.settings().to_string_lossy().into_owned(),
    })
}
