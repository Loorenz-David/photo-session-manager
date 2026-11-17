//! Thin research commands over the engine the app builds (R7·U1).

use std::{collections::BTreeMap, path::PathBuf, sync::Arc};

use psm_research::api::{
    ExportSelection, ResearchApi, ResearchStatus, SecretsInput, SessionResearch,
};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::State;

use crate::{
    commands::items::{CommandError, CommandId},
    state::AppState,
};

fn unavailable() -> CommandError {
    CommandError {
        code: "research_unavailable".into(),
        message: "Research is not available".into(),
        data: BTreeMap::new(),
    }
}

fn engine(state: &State<'_, AppState>) -> Result<Arc<dyn ResearchApi>, CommandError> {
    state
        .handles()
        .map_err(CommandError::from)?
        .research
        .ok_or_else(unavailable)
}

#[derive(Serialize, Type)]
pub struct ResearchStatusDto {
    pub reason: Option<ReasonDto>,
    pub run: Option<RunViewDto>,
    pub steps: Vec<StepViewDto>,
    pub fields_filled: u32,
    pub to_review: u32,
    pub can_run: bool,
    /// With the `WaitingForEditor` reason: when the wait for Capture One's previews ends (ms
    /// since the epoch); the panel says "Waiting for Capture One" from then on.
    #[specta(type = Option<specta_typescript::Number>)]
    pub wait_ends_at: Option<i64>,
    /// A6: the latest run is queued: its place in the line and the session being researched.
    pub queue: Option<QueueViewDto>,
}
#[derive(Serialize, Type)]
pub struct QueueViewDto {
    pub position: u32,
    #[specta(type = Option<specta_typescript::Number>)]
    pub running_session_id: Option<i64>,
    pub running_name: Option<String>,
    pub manual: bool,
}
/// A6: one session's research state (`researching`, `waiting`, `to_review`, `done`, `failed`,
/// `blocked`) for the sessions list and the session header.
#[derive(Serialize, Type)]
pub struct SessionResearchDto {
    #[specta(type = specta_typescript::Number)]
    pub session_id: i64,
    pub state: String,
    pub position: Option<u32>,
    pub to_review: u32,
    pub reason: Option<ReasonDto>,
}
fn count(n: usize) -> u32 {
    n.try_into().unwrap_or(u32::MAX)
}
impl From<SessionResearch> for SessionResearchDto {
    fn from(s: SessionResearch) -> Self {
        Self {
            session_id: s.session_id,
            state: s.state,
            position: s.position.map(count),
            to_review: count(s.to_review),
            reason: s.reason.map(|r| ReasonDto {
                code: r.code,
                sentence: r.sentence,
            }),
        }
    }
}
#[derive(Serialize, Type)]
pub struct ReasonDto {
    pub code: String,
    pub sentence: String,
}
#[derive(Serialize, Type)]
pub struct RunViewDto {
    #[specta(type = specta_typescript::Number)]
    pub id: i64,
    pub state: String,
    pub trigger: String,
    pub stage_reached: Option<String>,
    pub error: Option<String>,
}
#[derive(Serialize, Type)]
pub struct StepViewDto {
    #[specta(type = specta_typescript::Number)]
    pub seq: i64,
    pub stage: String,
    pub kind: String,
    pub summary: String,
    pub detail: String,
    pub source_url: Option<String>,
    pub source_title: Option<String>,
    #[specta(type = Vec<specta_typescript::Number>)]
    pub photo_ids: Vec<i64>,
    pub field_keys: Vec<String>,
    pub unread: bool,
}
impl From<ResearchStatus> for ResearchStatusDto {
    fn from(s: ResearchStatus) -> Self {
        Self {
            reason: s.reason.map(|r| ReasonDto {
                code: r.code,
                sentence: r.sentence,
            }),
            run: s.run.map(|r| RunViewDto {
                id: r.id,
                state: r.state,
                trigger: r.trigger,
                stage_reached: r.stage_reached,
                error: r.error,
            }),
            steps: s
                .steps
                .into_iter()
                .map(|v| StepViewDto {
                    seq: v.seq,
                    stage: v.stage,
                    kind: v.kind,
                    summary: v.summary,
                    detail: v.detail,
                    source_url: v.source_url,
                    source_title: v.source_title,
                    photo_ids: v.photo_ids,
                    field_keys: v.field_keys,
                    unread: v.unread,
                })
                .collect(),
            fields_filled: count(s.fields_filled),
            to_review: count(s.to_review),
            can_run: s.can_run,
            wait_ends_at: s.wait_ends_at,
            queue: s.queue.map(|q| QueueViewDto {
                position: count(q.position),
                running_session_id: q.running_session_id,
                running_name: q.running_name,
                manual: q.manual,
            }),
        }
    }
}
#[derive(Deserialize, Type)]
pub struct SecretsInputDto {
    pub model_key: Option<String>,
    pub search_key: Option<String>,
    pub reader_key: Option<String>,
}
#[derive(Serialize, Type)]
pub struct SecretsStateDto {
    pub model: bool,
    pub search: bool,
    pub reader: bool,
}
#[derive(Deserialize, Type)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ExportSelectionDto {
    Range {
        #[specta(type = specta_typescript::Number)]
        from: i64,
        #[specta(type = specta_typescript::Number)]
        to: i64,
    },
    Items {
        #[specta(type = Vec<specta_typescript::Number>)]
        ids: Vec<i64>,
    },
}

#[tauri::command]
#[specta::specta]
pub async fn research_status(
    state: State<'_, AppState>,
    session_id: CommandId,
) -> Result<ResearchStatusDto, CommandError> {
    engine(&state)?
        .status(session_id.0)
        .map(Into::into)
        .map_err(Into::into)
}
/// A6: the research state of every session that has one (or of `session_id` only), in one call.
#[tauri::command]
#[specta::specta]
pub async fn research_overview(
    state: State<'_, AppState>,
    session_id: Option<CommandId>,
) -> Result<Vec<SessionResearchDto>, CommandError> {
    engine(&state)?
        .overview(session_id.map(|s| s.0))
        .map(|rows| rows.into_iter().map(Into::into).collect())
        .map_err(Into::into)
}
#[tauri::command]
#[specta::specta]
pub async fn research_run(
    state: State<'_, AppState>,
    session_id: CommandId,
) -> Result<(), CommandError> {
    engine(&state)?.run(session_id.0).map_err(Into::into)
}
#[tauri::command]
#[specta::specta]
pub async fn research_secrets_set(
    state: State<'_, AppState>,
    input: SecretsInputDto,
) -> Result<(), CommandError> {
    engine(&state)?
        .secrets_set(SecretsInput {
            model_key: input.model_key,
            search_key: input.search_key,
            reader_key: input.reader_key,
        })
        .map_err(Into::into)
}
#[tauri::command]
#[specta::specta]
pub async fn research_secrets_state(
    state: State<'_, AppState>,
) -> Result<SecretsStateDto, CommandError> {
    engine(&state)?
        .secrets_state()
        .map(|s| SecretsStateDto {
            model: s.model,
            search: s.search,
            reader: s.reader,
        })
        .map_err(Into::into)
}
/// Saves the `research` value a site command returned (R7·U1 task 4) and returns it as text.
fn save_research(
    state: &State<'_, AppState>,
    value: serde_json::Value,
) -> Result<String, CommandError> {
    let saved = state.settings_set(&serde_json::json!({ "research": value }))?;
    Ok(saved["research"].to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn research_settings_reset(state: State<'_, AppState>) -> Result<String, CommandError> {
    let value = engine(&state)?.settings_reset()?;
    save_research(&state, value)
}

#[tauri::command]
#[specta::specta]
pub async fn research_site_add(
    state: State<'_, AppState>,
    site: String,
) -> Result<String, CommandError> {
    let value = engine(&state)?.site_add(&site)?;
    save_research(&state, value)
}
#[tauri::command]
#[specta::specta]
pub async fn research_site_remove(
    state: State<'_, AppState>,
    site: String,
) -> Result<String, CommandError> {
    let value = engine(&state)?.site_remove(&site)?;
    save_research(&state, value)
}
#[tauri::command]
#[specta::specta]
pub async fn research_sites_reorder(
    state: State<'_, AppState>,
    sites: Vec<String>,
) -> Result<String, CommandError> {
    let value = engine(&state)?.sites_reorder(&sites)?;
    save_research(&state, value)
}
#[tauri::command]
#[specta::specta]
pub async fn research_export(
    state: State<'_, AppState>,
    selection: ExportSelectionDto,
    dest: String,
) -> Result<String, CommandError> {
    let selection = match selection {
        ExportSelectionDto::Range { from, to } => ExportSelection::Range { from, to },
        ExportSelectionDto::Items { ids } => ExportSelection::Items(ids),
    };
    engine(&state)?
        .export(selection, &PathBuf::from(dest))
        .map(|p| p.to_string_lossy().into_owned())
        .map_err(Into::into)
}
