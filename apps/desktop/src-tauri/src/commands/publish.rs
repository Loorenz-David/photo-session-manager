use crate::{
    commands::items::{CommandError, CommandId},
    state::AppState,
};
use psm_core::{
    Error,
    domain::publish::{Notification, PublishActivity, PublishPreview},
    ports::store::Store,
};
use serde::Serialize;
use specta::Type;
use tauri::State;

#[derive(Serialize, Type)]
pub struct PreviewItemDto {
    #[specta(type = specta_typescript::Number)]
    pub session_id: i64,
    #[specta(type = Vec<specta_typescript::Number>)]
    pub main: Vec<i64>,
    #[specta(type = Vec<specta_typescript::Number>)]
    pub condition: Vec<i64>,
    pub main_thumbnails: Vec<Option<String>>,
    pub condition_thumbnails: Vec<Option<String>>,
    #[specta(type = specta_typescript::Number)]
    pub details_filled: usize,
    #[specta(type = specta_typescript::Number)]
    pub details_total: usize,
    #[specta(type = specta_typescript::Number)]
    pub unreviewed: usize,
    pub announced_removals: Vec<String>,
    pub create_item: bool,
}
#[derive(Serialize, Type)]
pub struct PreviewDto {
    pub items: Vec<PreviewItemDto>,
    pub summary: String,
}
impl From<PublishPreview> for PreviewDto {
    fn from(preview: PublishPreview) -> Self {
        Self {
            items: preview
                .items
                .into_iter()
                .map(|p| PreviewItemDto {
                    session_id: p.session_id,
                    main: p.main,
                    condition: p.condition,
                    main_thumbnails: p.main_thumbnails,
                    condition_thumbnails: p.condition_thumbnails,
                    details_filled: p.details_filled,
                    details_total: p.details_total,
                    unreviewed: p.unreviewed,
                    announced_removals: p.announced_removals,
                    create_item: p.create_item,
                })
                .collect(),
            summary: preview.summary,
        }
    }
}
#[derive(Serialize, Type)]
pub struct NotificationDto {
    #[specta(type = specta_typescript::Number)]
    pub id: i64,
    #[specta(type = specta_typescript::Number)]
    pub item_id: i64,
    #[specta(type = specta_typescript::Number)]
    pub session_id: i64,
    #[specta(type = specta_typescript::Number)]
    pub run_id: i64,
    pub kind: String,
    pub text: String,
    #[specta(type = specta_typescript::Number)]
    pub created_at: i64,
}
impl From<Notification> for NotificationDto {
    fn from(n: Notification) -> Self {
        Self {
            id: n.id,
            item_id: n.item_id,
            session_id: n.session_id,
            run_id: n.run_id,
            kind: n.kind,
            text: n.text,
            created_at: n.created_at,
        }
    }
}

#[tauri::command]
#[specta::specta]
pub async fn publish_preview(
    state: State<'_, AppState>,
    session_ids: Vec<CommandId>,
) -> Result<PreviewDto, CommandError> {
    let app = state.handles().map_err(CommandError::from)?;
    tauri::async_runtime::spawn_blocking(move || -> Result<PreviewDto, CommandError> {
        let service = app
            .publish
            .as_ref()
            .ok_or(CommandError::from(Error::NoIntegration))?;
        service
            .preview(&session_ids.into_iter().map(|id| id.0).collect::<Vec<_>>())
            .map(Into::into)
            .map_err(Into::into)
    })
    .await
    .map_err(|e| CommandError::from(psm_core::Error::InvalidData(e.to_string())))?
}
#[tauri::command]
#[specta::specta]
pub async fn publish_confirm(
    state: State<'_, AppState>,
    session_ids: Vec<CommandId>,
    summary_text: String,
) -> Result<CommandId, CommandError> {
    let app = state.handles().map_err(CommandError::from)?;
    tauri::async_runtime::spawn_blocking(move || -> Result<CommandId, CommandError> {
        let service = app
            .publish
            .as_ref()
            .ok_or(CommandError::from(Error::NoIntegration))?;
        service
            .confirm(
                &session_ids.into_iter().map(|id| id.0).collect::<Vec<_>>(),
                &summary_text,
            )
            .map(CommandId)
            .map_err(Into::into)
    })
    .await
    .map_err(|e| CommandError::from(psm_core::Error::InvalidData(e.to_string())))?
}
#[tauri::command]
#[specta::specta]
pub async fn publish_retry(
    state: State<'_, AppState>,
    item_id: CommandId,
) -> Result<(), CommandError> {
    let app = state.handles().map_err(CommandError::from)?;
    tauri::async_runtime::spawn_blocking(move || -> Result<(), CommandError> {
        app.publish
            .as_ref()
            .ok_or(CommandError::from(Error::NoIntegration))?
            .retry(item_id.0)
            .map_err(Into::into)
    })
    .await
    .map_err(|e| CommandError::from(psm_core::Error::InvalidData(e.to_string())))?
}
/// P64: what the publish progress popover shows (step in words, elapsed time, last error).
#[derive(Serialize, Type)]
pub struct PublishActivityDto {
    #[specta(type = specta_typescript::Number)]
    pub item_id: i64,
    pub step: String,
    pub text: String,
    pub percent: u8,
    #[specta(type = specta_typescript::Number)]
    pub elapsed_ms: i64,
    pub error: Option<String>,
    /// `retrying` or `retry_available`.
    pub retry: Option<String>,
    pub can_cancel: bool,
}
impl From<PublishActivity> for PublishActivityDto {
    fn from(a: PublishActivity) -> Self {
        Self {
            item_id: a.item_id,
            step: serde_json::to_value(a.step)
                .ok()
                .and_then(|v| v.as_str().map(str::to_owned))
                .unwrap_or_default(),
            text: a.text,
            percent: a.percent,
            elapsed_ms: a.elapsed_ms,
            error: a.error,
            retry: a.retry,
            can_cancel: a.can_cancel,
        }
    }
}
#[tauri::command]
#[specta::specta]
pub async fn publish_activity(
    state: State<'_, AppState>,
    session_id: CommandId,
) -> Result<Option<PublishActivityDto>, CommandError> {
    let app = state.handles().map_err(CommandError::from)?;
    tauri::async_runtime::spawn_blocking(
        move || -> Result<Option<PublishActivityDto>, CommandError> {
            Ok(app
                .publish
                .as_ref()
                .ok_or(CommandError::from(Error::NoIntegration))?
                .activity(session_id.0)?
                .map(Into::into))
        },
    )
    .await
    .map_err(|e| CommandError::from(psm_core::Error::InvalidData(e.to_string())))?
}
/// Changelog 44 (P64): Cancel publish, after the photographer confirmed it in the popover.
#[tauri::command]
#[specta::specta]
pub async fn publish_cancel(
    state: State<'_, AppState>,
    item_id: CommandId,
) -> Result<(), CommandError> {
    let app = state.handles().map_err(CommandError::from)?;
    tauri::async_runtime::spawn_blocking(move || -> Result<(), CommandError> {
        app.publish
            .as_ref()
            .ok_or(CommandError::from(Error::NoIntegration))?
            .cancel(item_id.0)
            .map_err(Into::into)
    })
    .await
    .map_err(|e| CommandError::from(psm_core::Error::InvalidData(e.to_string())))?
}
#[tauri::command]
#[specta::specta]
pub async fn notifications_list(
    state: State<'_, AppState>,
    session_id: Option<CommandId>,
) -> Result<Vec<NotificationDto>, CommandError> {
    let app = state.handles().map_err(CommandError::from)?;
    tauri::async_runtime::spawn_blocking(move || -> Result<Vec<NotificationDto>, CommandError> {
        app.store
            .notifications(session_id.map(|id| id.0))
            .map(|rows| rows.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    })
    .await
    .map_err(|e| CommandError::from(psm_core::Error::InvalidData(e.to_string())))?
}
