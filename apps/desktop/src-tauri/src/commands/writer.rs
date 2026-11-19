//! Thin writer commands over the engine the app builds (W1; `master_plan_writer.md` §4).

use std::{collections::BTreeMap, sync::Arc};

use psm_research::writer::{
    api::{HitView, Requested, WriterApi, WriterStatus},
    note::NoteView,
    run::Origin,
};
use serde::Serialize;
use specta::Type;
use tauri::State;

use crate::{
    commands::{
        items::{CommandError, CommandId},
        research::{ReasonDto, RunViewDto, StepViewDto},
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

/// One post-check hit.
#[derive(Serialize, Type)]
pub struct WriterHitDto {
    pub text: String,
    pub code: String,
    pub detail: String,
}

fn hits(list: Vec<HitView>) -> Vec<WriterHitDto> {
    list.into_iter()
        .map(|h| WriterHitDto {
            text: h.text,
            code: h.code,
            detail: h.detail,
        })
        .collect()
}

/// One text of the latest writer run: its reason, facts and what the check found.
#[derive(Serialize, Type)]
pub struct WriterTextDto {
    pub key: String,
    pub label: String,
    pub flagged: bool,
    pub hits: Vec<WriterHitDto>,
    pub reason: String,
    pub facts: Vec<String>,
    pub outcome: String,
}

/// The post-check of one call.
#[derive(Serialize, Type)]
pub struct WriterAttemptDto {
    pub attempt: u32,
    pub hits: Vec<WriterHitDto>,
}

/// An open writer finding (`kind` `finding`) or question (`kind` `question`); `mode` `ask` waits
/// for an answer, `note` can be dismissed.
#[derive(Serialize, Type)]
pub struct WriterFindingDto {
    #[specta(type = specta_typescript::Number)]
    pub id: i64,
    pub kind: String,
    #[serde(rename = "type")]
    pub finding_type: String,
    pub keys: Vec<String>,
    pub text_field: String,
    pub text: String,
    pub mode: String,
}

/// What the latest writer run received.
#[derive(Serialize, Type)]
pub struct WriterInputsDto {
    pub fields: u32,
    pub note: u32,
    pub listings: Vec<String>,
}

/// W3 (08C §11): the writer's state for a session.
#[derive(Serialize, Type)]
pub struct WriterStatusDto {
    pub reason: Option<ReasonDto>,
    pub run: Option<RunViewDto>,
    pub texts: Vec<WriterTextDto>,
    pub attempts: Vec<WriterAttemptDto>,
    pub regenerated: Vec<String>,
    pub findings: Vec<WriterFindingDto>,
    pub steps: Vec<StepViewDto>,
    #[specta(type = specta_typescript::Number)]
    pub cost_micro_usd: i64,
    pub inputs: WriterInputsDto,
    pub stale_main: bool,
    pub stale_condition: bool,
    pub can_write: bool,
}

fn count(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

impl From<WriterStatus> for WriterStatusDto {
    fn from(s: WriterStatus) -> Self {
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
            texts: s
                .texts
                .into_iter()
                .map(|t| WriterTextDto {
                    key: t.key,
                    label: t.label,
                    flagged: t.flagged,
                    hits: hits(t.hits),
                    reason: t.reason,
                    facts: t.facts,
                    outcome: t.outcome,
                })
                .collect(),
            attempts: s
                .attempts
                .into_iter()
                .map(|a| WriterAttemptDto {
                    attempt: a.attempt,
                    hits: hits(a.hits),
                })
                .collect(),
            regenerated: s.regenerated,
            findings: s
                .findings
                .into_iter()
                .map(|f| WriterFindingDto {
                    id: f.id,
                    kind: f.kind,
                    finding_type: f.finding_type,
                    keys: f.keys,
                    text_field: f.text_field,
                    text: f.text,
                    mode: f.mode,
                })
                .collect(),
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
            cost_micro_usd: s.cost_micro_usd,
            inputs: WriterInputsDto {
                fields: count(s.inputs.fields),
                note: count(s.inputs.note),
                listings: s.inputs.listings,
            },
            stale_main: s.stale_main,
            stale_condition: s.stale_condition,
            can_write: s.can_write,
        }
    }
}

/// W3: the writer's state for the session (the AI review tab's writer block, the research panel).
#[tauri::command]
#[specta::specta]
pub async fn writer_status(
    state: State<'_, AppState>,
    session_id: CommandId,
) -> Result<WriterStatusDto, CommandError> {
    engine(&state)?
        .status(session_id.0)
        .map(Into::into)
        .map_err(Into::into)
}

/// W3 (WC3): asks the writer to write the session's texts. `origin` is `next_to_review`,
/// `all_checked` or `write_texts`.
#[tauri::command]
#[specta::specta]
pub async fn writer_request(
    state: State<'_, AppState>,
    session_id: CommandId,
    origin: String,
) -> Result<RequestedDto, CommandError> {
    let origin = Origin::parse(&origin)
        .filter(|o| *o != Origin::Answer)
        .ok_or_else(|| CommandError {
            code: "invalid_value".into(),
            message: format!("unknown origin {origin}"),
            data: BTreeMap::new(),
        })?;
    engine(&state)?
        .request(session_id.0, origin)
        .map(Into::into)
        .map_err(Into::into)
}

/// W3 (WC6): answers an open writer question or finding; the texts are written again.
#[tauri::command]
#[specta::specta]
pub async fn writer_finding_answer(
    state: State<'_, AppState>,
    finding_id: CommandId,
    answer: String,
) -> Result<RequestedDto, CommandError> {
    engine(&state)?
        .answer(finding_id.0, &answer)
        .map(Into::into)
        .map_err(Into::into)
}

/// W3 (WC6): dismisses an open writer note or question; nothing is written again.
#[tauri::command]
#[specta::specta]
pub async fn writer_finding_dismiss(
    state: State<'_, AppState>,
    finding_id: CommandId,
) -> Result<(), CommandError> {
    engine(&state)?.dismiss(finding_id.0).map_err(Into::into)
}
