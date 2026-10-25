use crate::{
    commands::{
        items::{CommandError, CommandId, ItemDto},
        photos::PhotoClassification,
    },
    state::AppState,
};
use psm_core::{
    domain::{
        photo::Photo,
        session::{SessionDto, SessionView},
    },
    ports::clock::SystemClock,
    services::sessions::SessionsService,
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

/// One visible photo of the workspace (P08): what the stage, grid and strip need.
#[derive(Serialize, Type)]
pub struct PhotoResponse {
    #[specta(type = specta_typescript::Number)]
    pub id: i64,
    #[specta(type = specta_typescript::Number)]
    pub session_id: i64,
    pub classification: PhotoClassification,
    #[specta(type = Option<specta_typescript::Number>)]
    pub position: Option<i64>,
    #[specta(type = Option<specta_typescript::Number>)]
    pub captured_at: Option<i64>,
    pub original_preview_path: Option<String>,
    pub adjusted_preview_path: Option<String>,
    #[specta(type = Option<specta_typescript::Number>)]
    pub orientation: Option<i64>,
    pub source: String,
    pub integrity: String,
}

impl From<Photo> for PhotoResponse {
    fn from(photo: Photo) -> Self {
        use psm_core::domain::photo::{Classification, Integrity, PhotoSource};
        Self {
            id: photo.id,
            session_id: photo.session_id,
            classification: match photo.classification {
                Classification::Unreviewed => PhotoClassification::Unreviewed,
                Classification::Main => PhotoClassification::Main,
                Classification::Condition => PhotoClassification::Condition,
                Classification::Deleted => PhotoClassification::Deleted,
            },
            position: photo.position,
            captured_at: photo.captured_at,
            original_preview_path: photo.original_preview_path,
            adjusted_preview_path: photo.adjusted_preview_path,
            orientation: photo.orientation,
            source: match photo.source {
                PhotoSource::Capture => "capture",
                PhotoSource::Import => "import",
            }
            .into(),
            integrity: match photo.integrity {
                Integrity::Ok => "ok",
                Integrity::RawChanged => "raw_changed",
                Integrity::RawMissing => "raw_missing",
            }
            .into(),
        }
    }
}

/// `session_get`: the session, its item (null for Unassigned) and its visible photos.
#[derive(Serialize, Type)]
pub struct SessionViewResponse {
    pub session: SessionResponse,
    pub item: Option<ItemDto>,
    pub photos: Vec<PhotoResponse>,
}

impl From<SessionView> for SessionViewResponse {
    fn from(view: SessionView) -> Self {
        Self {
            session: view.session.into(),
            item: view.item.map(Into::into),
            photos: view.photos.into_iter().map(Into::into).collect(),
        }
    }
}

#[tauri::command]
#[specta::specta]
pub fn session_get(
    state: State<'_, AppState>,
    session_id: CommandId,
) -> Result<SessionViewResponse, CommandError> {
    let app = state
        .0
        .lock()
        .map_err(|e| CommandError::from(psm_core::Error::InvalidData(e.to_string())))?;
    SessionsService::new(app.store.as_ref(), &SystemClock)
        .view(session_id.0)
        .map(Into::into)
        .map_err(Into::into)
}

#[derive(serde::Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum SessionFilter {
    Open,
    Published,
    All,
}

impl From<SessionFilter> for psm_core::domain::session::ListFilter {
    fn from(value: SessionFilter) -> Self {
        use psm_core::domain::session::ListFilter;
        match value {
            SessionFilter::Open => ListFilter::Open,
            SessionFilter::Published => ListFilter::Published,
            SessionFilter::All => ListFilter::All,
        }
    }
}

#[derive(Serialize, Type)]
pub struct SessionListCountsResponse {
    #[specta(type = specta_typescript::Number)]
    pub open: i64,
    #[specta(type = specta_typescript::Number)]
    pub published: i64,
    #[specta(type = specta_typescript::Number)]
    pub all: i64,
}

#[derive(Serialize, Type)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum RowLabelResponse {
    Publishing(u8),
    PublishFailed,
    DescriptionFailed,
    Published,
    Ready,
    InProgress(#[specta(type = specta_typescript::Number)] i64),
}

#[derive(Serialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatusResponse {
    Publishing,
    Failed,
    Published,
    Ready,
    InProgress,
}

#[derive(Serialize, Type)]
pub struct SessionRowResponse {
    #[specta(type = specta_typescript::Number)]
    pub id: i64,
    pub sku: String,
    pub item_type: String,
    pub title: String,
    #[specta(type = specta_typescript::Number)]
    pub main_count: i64,
    #[specta(type = specta_typescript::Number)]
    pub condition_count: i64,
    #[specta(type = specta_typescript::Number)]
    pub unreviewed_count: i64,
    pub status: SessionStatusResponse,
    pub label: RowLabelResponse,
    pub progress: Option<u8>,
    #[specta(type = specta_typescript::Number)]
    pub last_activity_at: i64,
    pub cover_preview_path: Option<String>,
}
#[derive(Serialize, Type)]
pub struct SessionListGroupResponse {
    pub label: String,
    pub rows: Vec<SessionRowResponse>,
}
#[derive(Serialize, Type)]
pub struct SessionListResponse {
    pub groups: Vec<SessionListGroupResponse>,
    pub counts: SessionListCountsResponse,
    #[specta(type = specta_typescript::Number)]
    pub unassigned_count: i64,
}

impl From<psm_core::domain::session::SessionList> for SessionListResponse {
    fn from(list: psm_core::domain::session::SessionList) -> Self {
        use psm_core::domain::session::{RowLabel, SessionStatus};
        Self {
            groups: list
                .groups
                .into_iter()
                .map(|group| SessionListGroupResponse {
                    label: group.label,
                    rows: group
                        .rows
                        .into_iter()
                        .map(|row| SessionRowResponse {
                            id: row.id,
                            sku: row.sku,
                            item_type: row.item_type,
                            title: row.title,
                            main_count: row.main_count,
                            condition_count: row.condition_count,
                            unreviewed_count: row.unreviewed_count,
                            status: match row.status {
                                SessionStatus::Publishing => SessionStatusResponse::Publishing,
                                SessionStatus::Failed => SessionStatusResponse::Failed,
                                SessionStatus::Published => SessionStatusResponse::Published,
                                SessionStatus::Ready => SessionStatusResponse::Ready,
                                SessionStatus::InProgress => SessionStatusResponse::InProgress,
                            },
                            label: match row.label {
                                RowLabel::Publishing(p) => RowLabelResponse::Publishing(p),
                                RowLabel::PublishFailed => RowLabelResponse::PublishFailed,
                                RowLabel::DescriptionFailed => RowLabelResponse::DescriptionFailed,
                                RowLabel::Published => RowLabelResponse::Published,
                                RowLabel::Ready => RowLabelResponse::Ready,
                                RowLabel::InProgress(n) => RowLabelResponse::InProgress(n),
                            },
                            progress: row.progress,
                            last_activity_at: row.last_activity_at,
                            cover_preview_path: row.cover_preview_path,
                        })
                        .collect(),
                })
                .collect(),
            counts: SessionListCountsResponse {
                open: list.counts.open,
                published: list.counts.published,
                all: list.counts.all,
            },
            unassigned_count: list.unassigned_count,
        }
    }
}

#[tauri::command]
#[specta::specta]
pub fn sessions_list(
    state: State<'_, AppState>,
    filter: SessionFilter,
    query: Option<String>,
) -> Result<SessionListResponse, CommandError> {
    let app = state
        .0
        .lock()
        .map_err(|e| CommandError::from(psm_core::Error::InvalidData(e.to_string())))?;
    SessionsService::new(app.store.as_ref(), &SystemClock)
        .list(filter.into(), query)
        .map(Into::into)
        .map_err(Into::into)
}

#[tauri::command]
#[specta::specta]
pub fn session_active(state: State<'_, AppState>) -> Result<Option<SessionResponse>, CommandError> {
    let app = state
        .0
        .lock()
        .map_err(|e| CommandError::from(psm_core::Error::InvalidData(e.to_string())))?;
    let sessions = SessionsService::new(app.store.as_ref(), &SystemClock);
    match sessions.active()? {
        Some(id) => Ok(Some(sessions.get(id)?.into())),
        None => Ok(None),
    }
}

#[tauri::command]
#[specta::specta]
pub async fn session_activate(
    state: State<'_, AppState>,
    session_id: CommandId,
) -> Result<(), CommandError> {
    let (store, editing, health) = {
        let app = state
            .0
            .lock()
            .map_err(|e| CommandError::from(psm_core::Error::InvalidData(e.to_string())))?;
        (app.store.clone(), app.editing.clone(), app.health.clone())
    };
    tauri::async_runtime::spawn_blocking(move || {
        SessionsService::new(store.as_ref(), &SystemClock)
            .with_editing(editing.as_ref(), health.as_ref())
            .activate(session_id.0)
    })
    .await
    .map_err(|e| CommandError::from(psm_core::Error::InvalidData(e.to_string())))?
    .map_err(Into::into)
}

#[tauri::command]
#[specta::specta]
pub async fn session_deactivate(state: State<'_, AppState>) -> Result<(), CommandError> {
    let (store, editing, health) = {
        let app = state
            .0
            .lock()
            .map_err(|e| CommandError::from(psm_core::Error::InvalidData(e.to_string())))?;
        (app.store.clone(), app.editing.clone(), app.health.clone())
    };
    tauri::async_runtime::spawn_blocking(move || {
        SessionsService::new(store.as_ref(), &SystemClock)
            .with_editing(editing.as_ref(), health.as_ref())
            .deactivate()
    })
    .await
    .map_err(|e| CommandError::from(psm_core::Error::InvalidData(e.to_string())))?
    .map_err(Into::into)
}

#[cfg(test)]
mod list_tests {
    use super::*;
    use psm_core::domain::session::{
        ListCounts, ListGroup, RowLabel, SessionList, SessionRow, SessionStatus,
    };

    #[test]
    fn c5a_sessions_list_command_dto_fields() {
        let list = SessionList {
            groups: vec![ListGroup {
                label: "Today".into(),
                rows: vec![SessionRow {
                    id: 5,
                    sku: "OAK-2".into(),
                    item_type: "Furniture".into(),
                    title: "Oak sideboard".into(),
                    main_count: 2,
                    condition_count: 1,
                    unreviewed_count: 3,
                    status: SessionStatus::Publishing,
                    label: RowLabel::Publishing(42),
                    progress: Some(42),
                    last_activity_at: 123,
                    cover_preview_path: Some("/cover.jpg".into()),
                }],
            }],
            counts: ListCounts {
                open: 1,
                published: 0,
                all: 1,
            },
            unassigned_count: 2,
        };
        let response = SessionListResponse::from(list);
        assert_eq!(response.groups[0].label, "Today");
        let row = &response.groups[0].rows[0];
        assert_eq!(
            (
                row.id,
                row.sku.as_str(),
                row.item_type.as_str(),
                row.title.as_str()
            ),
            (5, "OAK-2", "Furniture", "Oak sideboard")
        );
        assert_eq!(
            (row.main_count, row.condition_count, row.unreviewed_count),
            (2, 1, 3)
        );
        assert!(matches!(row.status, SessionStatusResponse::Publishing));
        assert!(matches!(row.label, RowLabelResponse::Publishing(42)));
        assert_eq!(
            (
                row.progress,
                row.last_activity_at,
                row.cover_preview_path.as_deref()
            ),
            (Some(42), 123, Some("/cover.jpg"))
        );
        assert_eq!(
            (
                response.counts.open,
                response.counts.published,
                response.counts.all,
                response.unassigned_count
            ),
            (1, 0, 1, 2)
        );
    }
}
