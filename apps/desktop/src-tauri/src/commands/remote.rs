//! H4 (book 10H §6, §7; master plan archive §4 Commands, §5 MC-H3.3, MC-H3.5, MC-H4.1): the
//! remote view of a session's item, Refresh from Shopify and Bring back. Thin: each command runs
//! one `RemoteItems` call on a blocking worker. `session_remote_get` never downloads (MC-H3.5).

use crate::{
    commands::items::{CommandError, CommandId},
    state::AppState,
};
use psm_core::domain::remote::{BringBackOutcome, BringBackProgress, RemoteView};
use serde::Serialize;
use specta::Type;
use tauri::State;

/// One remote picture: its role (`gallery` / `condition`), its position in that role, its media
/// id, its cached file on this Mac (`null` until downloaded) and whether the shop no longer has it.
#[derive(Serialize, Type)]
pub struct RemotePictureDto {
    pub role: String,
    #[specta(type = specta_typescript::Number)]
    pub position: i64,
    pub media_id: String,
    pub path: Option<String>,
    pub gone: bool,
}

/// A running bring back's progress.
#[derive(Serialize, Type)]
pub struct BringBackProgressDto {
    #[specta(type = specta_typescript::Number)]
    pub done: usize,
    #[specta(type = specta_typescript::Number)]
    pub total: usize,
}

/// Where a session's item's pictures are (`local`, `unknown`, `archived`, `shop_only`) and, for a
/// remote item, its pictures (MC-H3.5).
#[derive(Serialize, Type)]
pub struct RemoteViewDto {
    #[specta(type = specta_typescript::Number)]
    pub item_id: i64,
    #[specta(type = RemoteStateDto)]
    pub state: String,
    pub keep_files: bool,
    pub pictures: Vec<RemotePictureDto>,
    #[specta(type = specta_typescript::Number)]
    pub bring_back_bytes: i64,
    pub check_failed: bool,
    pub bring_back: Option<BringBackProgressDto>,
}

/// The wire form of `RemoteState` (for the generated bindings only).
#[derive(Serialize, Type)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub enum RemoteStateDto {
    Local,
    Unknown,
    Archived,
    ShopOnly,
}

impl From<RemoteView> for RemoteViewDto {
    fn from(view: RemoteView) -> Self {
        Self {
            item_id: view.item_id,
            state: view.state.as_str().to_owned(),
            keep_files: view.keep_files,
            pictures: view
                .pictures
                .into_iter()
                .map(|p| RemotePictureDto {
                    role: p.role,
                    position: p.position,
                    media_id: p.media_id,
                    path: p.path.map(|path| path.to_string_lossy().into_owned()),
                    gone: p.gone,
                })
                .collect(),
            bring_back_bytes: view.bring_back_bytes,
            check_failed: view.check_failed,
            bring_back: view
                .bring_back
                .map(|BringBackProgress { done, total }| BringBackProgressDto { done, total }),
        }
    }
}

/// What a finished bring back did: the new photos, and whether Capture One listed the folder.
#[derive(Serialize, Type)]
pub struct BringBackOutcomeDto {
    #[specta(type = Vec<specta_typescript::Number>)]
    pub photo_ids: Vec<i64>,
    pub synced: bool,
}

impl From<BringBackOutcome> for BringBackOutcomeDto {
    fn from(outcome: BringBackOutcome) -> Self {
        Self {
            photo_ids: outcome.photo_ids,
            synced: outcome.synced,
        }
    }
}

fn worker_error(error: tauri::Error) -> CommandError {
    CommandError::from(psm_core::Error::InvalidData(error.to_string()))
}

/// MC-H3.5: the session's remote view (`null` for the Unassigned session). Missing picture
/// files are queued for the background; nothing is downloaded here.
#[tauri::command]
#[specta::specta]
pub async fn session_remote_get(
    state: State<'_, AppState>,
    session_id: CommandId,
) -> Result<Option<RemoteViewDto>, CommandError> {
    let app = state.handles().map_err(CommandError::from)?;
    tauri::async_runtime::spawn_blocking(move || {
        app.remote
            .view(session_id.0)
            .map(|view| view.map(Into::into))
            .map_err(CommandError::from)
    })
    .await
    .map_err(worker_error)?
}

/// MC-H3.3, Refresh from Shopify (a press): the record re-read, then the view.
#[tauri::command]
#[specta::specta]
pub async fn session_remote_refresh(
    state: State<'_, AppState>,
    session_id: CommandId,
) -> Result<RemoteViewDto, CommandError> {
    let app = state.handles().map_err(CommandError::from)?;
    tauri::async_runtime::spawn_blocking(move || {
        app.remote.refresh(session_id.0)?;
        app.remote
            .view(session_id.0)?
            .map(Into::into)
            .ok_or_else(|| CommandError::from(psm_core::Error::NotRemote))
    })
    .await
    .map_err(worker_error)?
}

/// MC-H4, Bring back: the item's pictures downloaded at full size and made the session's photos.
#[tauri::command]
#[specta::specta]
pub async fn session_bring_back(
    state: State<'_, AppState>,
    session_id: CommandId,
) -> Result<BringBackOutcomeDto, CommandError> {
    let app = state.handles().map_err(CommandError::from)?;
    tauri::async_runtime::spawn_blocking(move || {
        app.remote
            .bring_back(session_id.0)
            .map(Into::into)
            .map_err(CommandError::from)
    })
    .await
    .map_err(worker_error)?
}
