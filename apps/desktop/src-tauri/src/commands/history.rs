use crate::state::EditMenu;
use crate::{
    commands::{
        items::{CommandError, CommandId},
        sessions::SessionResponse,
    },
    state::AppState,
};
use psm_core::{
    ports::clock::SystemClock,
    services::history::{Direction, HistoryOutcome, HistoryService},
};
use serde::Serialize;
use specta::Type;
use tauri::{State, Wry};

#[derive(Serialize, Type)]
pub struct HistoryStateDto {
    pub undo: Option<String>,
    pub redo: Option<String>,
}
#[derive(Serialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum HistoryDirectionDto {
    Undone,
    Redone,
}
#[derive(Serialize, Type)]
pub struct HistoryOutcomeDto {
    pub label: String,
    pub direction: HistoryDirectionDto,
    pub sessions: Vec<SessionResponse>,
    #[specta(type = Option<specta_typescript::Number>)]
    pub active_photo_id: Option<i64>,
}
impl From<HistoryOutcome> for HistoryOutcomeDto {
    fn from(out: HistoryOutcome) -> Self {
        Self {
            label: out.label,
            direction: match out.direction {
                Direction::Undone => HistoryDirectionDto::Undone,
                Direction::Redone => HistoryDirectionDto::Redone,
            },
            sessions: out.sessions.into_iter().map(Into::into).collect(),
            active_photo_id: out.active_photo_id,
        }
    }
}

#[tauri::command]
#[specta::specta]
pub async fn history_get(
    state: State<'_, AppState>,
    session_id: CommandId,
) -> Result<HistoryStateDto, CommandError> {
    let app = state.handles().map_err(CommandError::from)?;
    tauri::async_runtime::spawn_blocking(move || {
        let s = app.history.state(session_id.0);
        Ok(HistoryStateDto {
            undo: s.undo,
            redo: s.redo,
        })
    })
    .await
    .map_err(|e| CommandError::from(psm_core::Error::InvalidData(e.to_string())))?
}
#[tauri::command]
#[specta::specta]
pub async fn history_undo(
    state: State<'_, AppState>,
    session_id: CommandId,
) -> Result<HistoryOutcomeDto, CommandError> {
    let app = state.handles().map_err(CommandError::from)?;
    tauri::async_runtime::spawn_blocking(move || {
        HistoryService::new(app.store.as_ref(), &SystemClock, app.history.as_ref())
            .undo(session_id.0)
            .map(Into::into)
            .map_err(Into::into)
    })
    .await
    .map_err(|e| CommandError::from(psm_core::Error::InvalidData(e.to_string())))?
}
#[tauri::command]
#[specta::specta]
pub async fn history_redo(
    state: State<'_, AppState>,
    session_id: CommandId,
) -> Result<HistoryOutcomeDto, CommandError> {
    let app = state.handles().map_err(CommandError::from)?;
    tauri::async_runtime::spawn_blocking(move || {
        HistoryService::new(app.store.as_ref(), &SystemClock, app.history.as_ref())
            .redo(session_id.0)
            .map(Into::into)
            .map_err(Into::into)
    })
    .await
    .map_err(|e| CommandError::from(psm_core::Error::InvalidData(e.to_string())))?
}

#[tauri::command]
#[specta::specta]
pub async fn edit_menu_set(
    menu: State<'_, EditMenu<Wry>>,
    undo: Option<String>,
    redo: Option<String>,
) -> Result<(), CommandError> {
    menu.undo
        .set_text(
            undo.as_ref()
                .map(|s| format!("Undo {s}"))
                .unwrap_or_else(|| "Undo".into()),
        )
        .map_err(|e| CommandError::from(psm_core::Error::InvalidData(e.to_string())))?;
    menu.undo
        .set_enabled(undo.is_some())
        .map_err(|e| CommandError::from(psm_core::Error::InvalidData(e.to_string())))?;
    menu.redo
        .set_text(
            redo.as_ref()
                .map(|s| format!("Redo {s}"))
                .unwrap_or_else(|| "Redo".into()),
        )
        .map_err(|e| CommandError::from(psm_core::Error::InvalidData(e.to_string())))?;
    menu.redo
        .set_enabled(redo.is_some())
        .map_err(|e| CommandError::from(psm_core::Error::InvalidData(e.to_string())))?;
    Ok(())
}
