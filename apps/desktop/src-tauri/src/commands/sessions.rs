use crate::{
    commands::items::{CommandError, CommandId},
    state::AppState,
};
use psm_core::{
    domain::session::SessionDto, ports::clock::SystemClock, services::sessions::SessionsService,
};
use serde::Serialize;
use specta::Type;
use tauri::State;

#[derive(Serialize, Type)]
pub struct SessionResponse {
    #[specta(type = specta_typescript::Number)]
    pub id: i64,
    #[specta(type = Option<specta_typescript::Number>)]
    pub item_id: Option<i64>,
    #[specta(type = Option<specta_typescript::Number>)]
    pub published_at: Option<i64>,
    #[specta(type = specta_typescript::Number)]
    pub published_seq: i64,
    #[specta(type = specta_typescript::Number)]
    pub change_seq: i64,
    pub dirty: bool,
    #[specta(type = specta_typescript::Number)]
    pub last_activity_at: i64,
    #[specta(type = specta_typescript::Number)]
    pub created_at: i64,
    #[specta(type = specta_typescript::Number)]
    pub photo_count: i64,
    #[specta(type = specta_typescript::Number)]
    pub unreviewed_count: i64,
    #[specta(type = specta_typescript::Number)]
    pub main_count: i64,
    #[specta(type = specta_typescript::Number)]
    pub condition_count: i64,
}

impl From<SessionDto> for SessionResponse {
    fn from(dto: SessionDto) -> Self {
        Self {
            id: dto.session.id,
            item_id: dto.session.item_id,
            published_at: dto.session.published_at,
            published_seq: dto.session.published_seq,
            change_seq: dto.session.change_seq,
            dirty: dto.dirty,
            last_activity_at: dto.session.last_activity_at,
            created_at: dto.session.created_at,
            photo_count: dto.photo_count,
            unreviewed_count: dto.unreviewed_count,
            main_count: dto.main_count,
            condition_count: dto.condition_count,
        }
    }
}

#[tauri::command]
#[specta::specta]
pub fn session_create(
    state: State<'_, AppState>,
    item_id: CommandId,
) -> Result<SessionResponse, CommandError> {
    let app = state
        .0
        .lock()
        .map_err(|e| CommandError::from(psm_core::Error::InvalidData(e.to_string())))?;
    SessionsService::new(app.store.as_ref(), &SystemClock)
        .create(item_id.0)
        .map(Into::into)
        .map_err(Into::into)
}

#[tauri::command]
#[specta::specta]
pub fn session_get(
    state: State<'_, AppState>,
    session_id: CommandId,
) -> Result<SessionResponse, CommandError> {
    let app = state
        .0
        .lock()
        .map_err(|e| CommandError::from(psm_core::Error::InvalidData(e.to_string())))?;
    SessionsService::new(app.store.as_ref(), &SystemClock)
        .get(session_id.0)
        .map(Into::into)
        .map_err(Into::into)
}
