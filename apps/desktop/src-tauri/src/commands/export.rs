use crate::{
    commands::items::{CommandError, CommandId},
    state::AppState,
};
use psm_core::{Error, services::export::ExportService};
use serde::Serialize;
use specta::Type;
use std::sync::Arc;
use tauri::State;

#[derive(Serialize, Type)]
pub struct ExportRunResponse {
    #[specta(type = specta_typescript::Number)]
    pub run_id: i64,
}

fn service(state: &State<'_, AppState>) -> Result<Arc<ExportService>, CommandError> {
    let app = state
        .0
        .lock()
        .map_err(|e| CommandError::from(Error::InvalidData(e.to_string())))?;
    Ok(app.export.clone())
}

/// Starts the local export of the session (ch07 §Completion without an integration). Progress and
/// the outcome arrive as `export_progress` events; a changed or missing RAW fails here, before any render.
#[tauri::command]
#[specta::specta]
pub async fn export_run(
    state: State<'_, AppState>,
    session_id: CommandId,
) -> Result<ExportRunResponse, CommandError> {
    let export = service(&state)?;
    tauri::async_runtime::spawn_blocking(move || export.run(session_id.0))
        .await
        .map_err(|e| CommandError::from(Error::InvalidData(e.to_string())))?
        .map(|run| ExportRunResponse { run_id: run.0 })
        .map_err(Into::into)
}

/// Shows the session's export folder (`<export folder>/<SKU>/`) in Finder.
#[tauri::command]
#[specta::specta]
pub fn export_open_folder(
    state: State<'_, AppState>,
    session_id: CommandId,
) -> Result<(), CommandError> {
    let folder = service(&state)?.folder(session_id.0)?;
    std::process::Command::new("/usr/bin/open")
        .arg(&folder)
        .spawn()
        .map_err(|e| CommandError::from(Error::Io(e)))?;
    Ok(())
}

/// The configured integration's kind (`settings.integration.kind`), or null when none is configured:
/// the session header then offers Export instead of Publish (H6).
#[tauri::command]
#[specta::specta]
pub fn integration_get(state: State<'_, AppState>) -> Result<Option<String>, CommandError> {
    let app = state
        .0
        .lock()
        .map_err(|e| CommandError::from(Error::InvalidData(e.to_string())))?;
    Ok(app.settings.integration.as_ref().and_then(|value| {
        (!value.is_null()).then(|| {
            value
                .get("kind")
                .and_then(|kind| kind.as_str())
                .unwrap_or("unknown")
                .to_owned()
        })
    }))
}
