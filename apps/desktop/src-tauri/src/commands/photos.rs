use crate::{
    commands::{
        items::{CommandError, CommandId},
        sessions::SessionResponse,
    },
    state::AppState,
};
use psm_core::{
    domain::photo::Classification,
    ports::clock::SystemClock,
    services::photos::PhotosService,
    services::review::{ReviewResult, ReviewService},
};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::State;

#[derive(Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum PhotoClassification {
    Unreviewed,
    Main,
    Condition,
    Deleted,
}

impl From<PhotoClassification> for Classification {
    fn from(value: PhotoClassification) -> Self {
        match value {
            PhotoClassification::Unreviewed => Self::Unreviewed,
            PhotoClassification::Main => Self::Main,
            PhotoClassification::Condition => Self::Condition,
            PhotoClassification::Deleted => Self::Deleted,
        }
    }
}

#[derive(Serialize, Type)]
pub struct ReviewResponse {
    pub sessions: Vec<SessionResponse>,
    #[specta(type = Option<specta_typescript::Number>)]
    pub active_photo_id: Option<i64>,
}

impl From<ReviewResult> for ReviewResponse {
    fn from(value: ReviewResult) -> Self {
        Self {
            sessions: value.sessions.into_iter().map(Into::into).collect(),
            active_photo_id: value.active_photo_id,
        }
    }
}

fn with_review(
    app: &psm_integrations::AppHandles,
    op: impl FnOnce(&ReviewService<'_>) -> psm_core::Result<ReviewResult>,
) -> Result<ReviewResponse, CommandError> {
    op(&ReviewService::new(app.store.as_ref(), &SystemClock))
        .map(Into::into)
        .map_err(Into::into)
}

#[tauri::command]
#[specta::specta]
pub async fn photos_classify(
    state: State<'_, AppState>,
    photo_id: CommandId,
    classification: PhotoClassification,
) -> Result<ReviewResponse, CommandError> {
    let app = state.handles().map_err(CommandError::from)?;
    tauri::async_runtime::spawn_blocking(move || -> Result<ReviewResponse, CommandError> {
        with_review(&app, |service| {
            service.classify(photo_id.0, classification.into())
        })
    })
    .await
    .map_err(|e| CommandError::from(psm_core::Error::InvalidData(e.to_string())))?
}

#[tauri::command]
#[specta::specta]
pub async fn photos_reorder(
    state: State<'_, AppState>,
    photo_id: CommandId,
    position: CommandId,
) -> Result<ReviewResponse, CommandError> {
    let app = state.handles().map_err(CommandError::from)?;
    tauri::async_runtime::spawn_blocking(move || -> Result<ReviewResponse, CommandError> {
        with_review(&app, |service| service.reorder(photo_id.0, position.0))
    })
    .await
    .map_err(|e| CommandError::from(psm_core::Error::InvalidData(e.to_string())))?
}

#[tauri::command]
#[specta::specta]
pub async fn photos_delete(
    state: State<'_, AppState>,
    photo_id: CommandId,
) -> Result<ReviewResponse, CommandError> {
    let app = state.handles().map_err(CommandError::from)?;
    tauri::async_runtime::spawn_blocking(move || -> Result<ReviewResponse, CommandError> {
        with_review(&app, |service| service.delete(photo_id.0))
    })
    .await
    .map_err(|e| CommandError::from(psm_core::Error::InvalidData(e.to_string())))?
}

#[tauri::command]
#[specta::specta]
pub async fn photos_undo(
    state: State<'_, AppState>,
    session_id: CommandId,
) -> Result<ReviewResponse, CommandError> {
    let app = state.handles().map_err(CommandError::from)?;
    tauri::async_runtime::spawn_blocking(move || -> Result<ReviewResponse, CommandError> {
        with_review(&app, |service| service.undo(session_id.0))
    })
    .await
    .map_err(|e| CommandError::from(psm_core::Error::InvalidData(e.to_string())))?
}

#[tauri::command]
#[specta::specta]
pub async fn photos_move(
    state: State<'_, AppState>,
    photo_ids: Vec<CommandId>,
    target_session_id: CommandId,
) -> Result<ReviewResponse, CommandError> {
    let app = state.handles().map_err(CommandError::from)?;
    tauri::async_runtime::spawn_blocking(move || -> Result<ReviewResponse, CommandError> {
        with_review(&app, |service| {
            service.move_photos(
                &photo_ids.into_iter().map(|id| id.0).collect::<Vec<_>>(),
                target_session_id.0,
            )
        })
    })
    .await
    .map_err(|e| CommandError::from(psm_core::Error::InvalidData(e.to_string())))?
}

#[tauri::command]
#[specta::specta]
pub async fn photos_copy(
    state: State<'_, AppState>,
    photo_ids: Vec<CommandId>,
    target_session_id: CommandId,
) -> Result<ReviewResponse, CommandError> {
    let app = state.handles().map_err(CommandError::from)?;
    tauri::async_runtime::spawn_blocking(move || -> Result<ReviewResponse, CommandError> {
        with_review(&app, |service| {
            service.copy_photos(
                &photo_ids.into_iter().map(|id| id.0).collect::<Vec<_>>(),
                target_session_id.0,
            )
        })
    })
    .await
    .map_err(|e| CommandError::from(psm_core::Error::InvalidData(e.to_string())))?
}

#[tauri::command]
#[specta::specta]
pub async fn photos_import(
    state: State<'_, AppState>,
    paths: Vec<String>,
) -> Result<(), CommandError> {
    let app = state.handles().map_err(CommandError::from)?;
    tauri::async_runtime::spawn_blocking(move || -> Result<(), CommandError> {
        PhotosService {
            arrival: app.arrival.as_ref(),
            editing: app.editing.as_ref(),
            store: app.store.as_ref(),
            clock: &SystemClock,
        }
        .import(&paths)
        .map_err(Into::into)
    })
    .await
    .map_err(|e| CommandError::from(psm_core::Error::InvalidData(e.to_string())))?
}

#[tauri::command]
#[specta::specta]
pub async fn photos_assign_unassigned(
    state: State<'_, AppState>,
    item_id: CommandId,
) -> Result<ReviewResponse, CommandError> {
    let app = state.handles().map_err(CommandError::from)?;
    tauri::async_runtime::spawn_blocking(move || -> Result<ReviewResponse, CommandError> {
        PhotosService {
            arrival: app.arrival.as_ref(),
            editing: app.editing.as_ref(),
            store: app.store.as_ref(),
            clock: &SystemClock,
        }
        .assign_unassigned(item_id.0)
        .map(Into::into)
        .map_err(Into::into)
    })
    .await
    .map_err(|e| CommandError::from(psm_core::Error::InvalidData(e.to_string())))?
}
