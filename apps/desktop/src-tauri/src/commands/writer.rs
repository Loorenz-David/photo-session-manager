//! Thin writer commands over the engine the app builds (W1; `master_plan_writer.md` §4).

use std::{collections::BTreeMap, sync::Arc};

use psm_research::writer::{
    api::{Requested, WriterApi},
    note::NoteView,
};
use serde::Serialize;
use specta::Type;
use tauri::State;

use crate::{
    commands::{
        items::{CommandError, CommandId},
        research::ReasonDto,
    },
    state::AppState,
};

/// The research engine could not be built: the writer is not available either (HA4).
fn unavailable() -> CommandError {
    CommandError {
        code: "research_unavailable".into(),
        message: "Research is not available".into(),
        data: BTreeMap::new(),
    }
}

fn engine(state: &State<'_, AppState>) -> Result<Arc<dyn WriterApi>, CommandError> {
    state
        .handles()
        .map_err(CommandError::from)?
        .writer
        .ok_or_else(unavailable)
}

/// One key of an observation and the owner's standing on it (`accepted`, `corrected`,
/// `kept_previous`, `dismissed`, `typed`, `unchecked`, `shop_default`, `imported`, `empty`).
#[derive(Serialize, Type)]
pub struct KeyDecisionDto {
    pub key: String,
    pub decision: String,
}

/// One observation of the product note.
#[derive(Serialize, Type)]
pub struct ObservationDto {
    /// `p1.<i>` (main photos) or `c1.<i>` (condition photos).
    #[serde(rename = "ref")]
    pub reference: String,
    pub original: String,
    /// The text shown: the owner's decisions replace what research saw.
    pub text: String,
    pub keys: Vec<String>,
    /// "condition photo n", or "photo no longer in the condition photos".
    pub photo: Option<String>,
    /// The text states the owner's decision ("Your decision").
    pub owner: bool,
    pub decisions: Vec<KeyDecisionDto>,
    pub in_writer_input: bool,
}

#[derive(Serialize, Type)]
pub struct ResearchSawDto {
    pub key: String,
    pub text: String,
}

/// What research saw on the session's photos, and whether the photos changed since.
#[derive(Serialize, Type)]
pub struct NoteDto {
    pub observations: Vec<ObservationDto>,
    pub research_saw: Vec<ResearchSawDto>,
    pub stale_main: bool,
    pub stale_condition: bool,
    pub missing_main: bool,
}

impl From<NoteView> for NoteDto {
    fn from(view: NoteView) -> Self {
        Self {
            observations: view
                .observations
                .into_iter()
                .map(|o| ObservationDto {
                    reference: o.reference,
                    original: o.original,
                    text: o.text,
                    keys: o.keys,
                    photo: o.photo,
                    owner: o.owner,
                    decisions: o
                        .decisions
                        .into_iter()
                        .map(|d| KeyDecisionDto {
                            key: d.key,
                            decision: d.decision,
                        })
                        .collect(),
                    in_writer_input: o.in_writer_input,
                })
                .collect(),
            research_saw: view
                .research_saw
                .into_iter()
                .map(|s| ResearchSawDto {
                    key: s.key,
                    text: s.text,
                })
                .collect(),
            stale_main: view.stale_main,
            stale_condition: view.stale_condition,
            missing_main: view.missing_main,
        }
    }
}

/// Whether a request queued a run, and why not.
#[derive(Serialize, Type)]
pub struct RequestedDto {
    pub queued: bool,
    pub reason: Option<ReasonDto>,
}

impl From<Requested> for RequestedDto {
    fn from(r: Requested) -> Self {
        Self {
            queued: r.queued,
            reason: r.reason.map(|r| ReasonDto {
                code: r.code,
                sentence: r.sentence,
            }),
        }
    }
}

/// W1: the session's product note ("What research saw").
#[tauri::command]
#[specta::specta]
pub async fn writer_note(
    state: State<'_, AppState>,
    session_id: CommandId,
) -> Result<NoteDto, CommandError> {
    engine(&state)?
        .note(session_id.0)
        .map(Into::into)
        .map_err(Into::into)
}

/// W1: **Look again**: the photo looks run again over the changed photos.
#[tauri::command]
#[specta::specta]
pub async fn writer_look_again(
    state: State<'_, AppState>,
    session_id: CommandId,
) -> Result<RequestedDto, CommandError> {
    engine(&state)?
        .look_again(session_id.0)
        .map(Into::into)
        .map_err(Into::into)
}
