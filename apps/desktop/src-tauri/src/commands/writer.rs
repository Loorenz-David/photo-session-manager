//! Thin writer commands over the engine the app builds (W1; `master_plan_writer.md` §4).

use std::{collections::BTreeMap, sync::Arc};

use psm_core::ports::clock::Clock as _;
use psm_research::writer::{
    api::{HitView, Requested, WriterApi, WriterStatus},
    config::{Example, Instruction, LANGUAGES, Lengths, SafeguardModes, WriterSettings},
    note::NoteView,
    run::Origin,
};
use serde::{Deserialize, Serialize};
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
    /// A24: when the latest run ended (UTC ms).
    #[specta(type = Option<specta_typescript::Number>)]
    pub finished_at: Option<i64>,
    /// A24: the requested texts the latest run did not offer, and why.
    pub not_offered: Vec<WriterNotOfferedDto>,
}

/// A24: a requested text the latest writer run did not put in the form (`outcome` in the
/// trace's words).
#[derive(Serialize, Type)]
pub struct WriterNotOfferedDto {
    pub key: String,
    pub label: String,
    pub outcome: String,
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
            finished_at: s.finished_at,
            not_offered: s
                .not_offered
                .into_iter()
                .map(|n| WriterNotOfferedDto {
                    key: n.key,
                    label: n.label,
                    outcome: n.outcome,
                })
                .collect(),
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

/// An earlier instruction text, newest first.
#[derive(Serialize, Deserialize, Type, Clone)]
pub struct InstructionVersionDto {
    #[specta(type = specta_typescript::Number)]
    pub saved_at: i64,
    pub text: String,
}

/// One language's instruction: the saved text (`null` = the shipped one), the text the writer
/// sends, whether that is the shipped one, and the earlier texts (at most 20, newest first).
/// On save only `text` is read: the earlier texts are the stored ones.
#[derive(Serialize, Deserialize, Type, Clone)]
pub struct WriterInstructionDto {
    pub text: Option<String>,
    pub effective_instruction: String,
    pub is_default: bool,
    pub versions: Vec<InstructionVersionDto>,
}

/// One style example (never copied by the writer).
#[derive(Serialize, Deserialize, Type, Clone)]
pub struct WriterExampleDto {
    pub name: String,
    pub fields: String,
    pub title: String,
    pub description: String,
    pub condition_report: String,
    pub seo_title: String,
    pub seo_description: String,
}

#[derive(Serialize, Deserialize, Type, Clone)]
pub struct WriterLengthsDto {
    pub title_max: u32,
    pub description_min_words: u32,
    pub description_max_words: u32,
    pub condition_max_words: u32,
    pub seo_title_max: u32,
    pub seo_description_min: u32,
    pub seo_description_max: u32,
}

/// One mode (`off`, `note` or `ask`) per safeguard type.
#[derive(Serialize, Deserialize, Type, Clone)]
pub struct WriterSafeguardDto {
    pub field_vs_photos: String,
    pub field_vs_field: String,
    pub field_vs_listings: String,
    pub missing_in_text: String,
    pub text_vs_facts: String,
}

/// How often the photographer answered and dismissed one finding type.
#[derive(Serialize, Deserialize, Type, Clone)]
pub struct WriterCountDto {
    pub answered: u32,
    pub dismissed: u32,
}

/// W4: Settings → Writer, every field present (`master_plan_writer.md` §4 "Settings"). Saved
/// whole: the save replaces the `writer` slot (`counts`, `effective_instruction`, `is_default`
/// and `versions` are read-only and ignored on save). `examples` holds each language's list as
/// the writer sends it; a list equal to the shipped one is stored as the shipped default.
#[derive(Serialize, Deserialize, Type, Clone)]
pub struct WriterSettingsDto {
    pub auto: bool,
    pub language: String,
    pub model: String,
    pub effort: String,
    pub instructions: BTreeMap<String, WriterInstructionDto>,
    pub examples: BTreeMap<String, Vec<WriterExampleDto>>,
    pub banned_extra: Vec<String>,
    pub lengths: WriterLengthsDto,
    pub visible_keys: Option<Vec<String>>,
    pub safeguard: WriterSafeguardDto,
    pub max_questions: u32,
    pub shop_name: String,
    pub condition_report_key: String,
    pub damage_terms: BTreeMap<String, BTreeMap<String, Vec<String>>>,
    /// A23: the four switches (default off).
    pub grade_word: bool,
    pub stability_sentence: bool,
    pub title_formula: bool,
    pub dimensions_line: bool,
    pub counts: BTreeMap<String, WriterCountDto>,
}

fn example_dto(e: Example) -> WriterExampleDto {
    WriterExampleDto {
        name: e.name,
        fields: e.fields,
        title: e.title,
        description: e.description,
        condition_report: e.condition_report,
        seo_title: e.seo_title,
        seo_description: e.seo_description,
    }
}

fn example_of(e: WriterExampleDto) -> Example {
    Example {
        name: e.name,
        fields: e.fields,
        title: e.title,
        description: e.description,
        condition_report: e.condition_report,
        seo_title: e.seo_title,
        seo_description: e.seo_description,
    }
}

fn invalid(error: impl ToString) -> CommandError {
    CommandError::from(psm_core::Error::InvalidData(error.to_string()))
}

/// The settings as shown: the slot read tolerantly, each language's instruction resolved, the
/// counts per finding type.
fn settings_dto(
    engine: &dyn WriterApi,
    settings: WriterSettings,
) -> Result<WriterSettingsDto, CommandError> {
    let mut languages: Vec<String> = LANGUAGES.iter().map(|l| (*l).to_owned()).collect();
    for language in settings.instructions.keys().chain(settings.examples.keys()) {
        if !languages.contains(language) {
            languages.push(language.clone());
        }
    }
    let instructions = languages
        .iter()
        .map(|language| {
            let stored = settings
                .instructions
                .get(language)
                .cloned()
                .unwrap_or_default();
            let shipped = engine.instruction_default(language);
            let dto = WriterInstructionDto {
                is_default: stored.text.is_none(),
                effective_instruction: stored.text.clone().unwrap_or(shipped),
                text: stored.text,
                versions: stored
                    .versions
                    .into_iter()
                    .map(|v| InstructionVersionDto {
                        saved_at: v.saved_at,
                        text: v.text,
                    })
                    .collect(),
            };
            (language.clone(), dto)
        })
        .collect();
    let examples = languages
        .iter()
        .map(|language| {
            let list = settings
                .resolved_examples(language)
                .into_iter()
                .map(example_dto)
                .collect();
            (language.clone(), list)
        })
        .collect();
    let counts = engine
        .counts()?
        .into_iter()
        .map(|c| {
            (
                c.finding_type,
                WriterCountDto {
                    answered: u32::try_from(c.answered).unwrap_or(u32::MAX),
                    dismissed: u32::try_from(c.dismissed).unwrap_or(u32::MAX),
                },
            )
        })
        .collect();
    let l = settings.lengths;
    let g = settings.safeguard;
    Ok(WriterSettingsDto {
        auto: settings.auto,
        language: settings.language,
        model: settings.model,
        effort: settings.effort,
        instructions,
        examples,
        banned_extra: settings.banned_extra,
        lengths: WriterLengthsDto {
            title_max: l.title_max,
            description_min_words: l.description_min_words,
            description_max_words: l.description_max_words,
            condition_max_words: l.condition_max_words,
            seo_title_max: l.seo_title_max,
            seo_description_min: l.seo_description_min,
            seo_description_max: l.seo_description_max,
        },
        visible_keys: settings.visible_keys,
        safeguard: WriterSafeguardDto {
            field_vs_photos: g.field_vs_photos,
            field_vs_field: g.field_vs_field,
            field_vs_listings: g.field_vs_listings,
            missing_in_text: g.missing_in_text,
            text_vs_facts: g.text_vs_facts,
        },
        max_questions: settings.max_questions,
        shop_name: settings.shop_name,
        condition_report_key: settings.condition_report_key,
        damage_terms: settings.damage_terms,
        grade_word: settings.grade_word,
        stability_sentence: settings.stability_sentence,
        title_formula: settings.title_formula,
        dimensions_line: settings.dimensions_line,
        counts,
    })
}

/// The slot a shown value stands for (before the versions rule is applied).
fn settings_of(dto: WriterSettingsDto) -> WriterSettings {
    let l = dto.lengths;
    let g = dto.safeguard;
    WriterSettings {
        auto: dto.auto,
        language: dto.language,
        model: dto.model,
        effort: dto.effort,
        instructions: dto
            .instructions
            .into_iter()
            .map(|(language, i)| {
                (
                    language,
                    Instruction {
                        text: i.text,
                        versions: Vec::new(),
                    },
                )
            })
            .collect(),
        examples: dto
            .examples
            .into_iter()
            .map(|(language, list)| (language, Some(list.into_iter().map(example_of).collect())))
            .collect(),
        banned_extra: dto.banned_extra,
        lengths: Lengths {
            title_max: l.title_max,
            description_min_words: l.description_min_words,
            description_max_words: l.description_max_words,
            condition_max_words: l.condition_max_words,
            seo_title_max: l.seo_title_max,
            seo_description_min: l.seo_description_min,
            seo_description_max: l.seo_description_max,
        },
        visible_keys: dto.visible_keys,
        safeguard: SafeguardModes {
            field_vs_photos: g.field_vs_photos,
            field_vs_field: g.field_vs_field,
            field_vs_listings: g.field_vs_listings,
            missing_in_text: g.missing_in_text,
            text_vs_facts: g.text_vs_facts,
        },
        max_questions: dto.max_questions,
        shop_name: dto.shop_name,
        condition_report_key: dto.condition_report_key,
        damage_terms: dto.damage_terms,
        grade_word: dto.grade_word,
        stability_sentence: dto.stability_sentence,
        title_formula: dto.title_formula,
        dimensions_line: dto.dimensions_line,
    }
}

/// The stored `writer` slot, read as the engine reads it.
fn stored(state: &State<'_, AppState>) -> Result<WriterSettings, CommandError> {
    let paths = state.handles().map_err(CommandError::from)?.paths;
    let file = psm_infra::settings_file::Settings::open(&paths).map_err(CommandError::from)?;
    Ok(WriterSettings::from_slot(file.writer.as_ref()))
}

/// Saves the whole slot (never a part of it: a partial `writer` patch keeps stale fields) and
/// returns it as shown.
fn save_slot(
    state: &State<'_, AppState>,
    engine: &dyn WriterApi,
    settings: &WriterSettings,
) -> Result<WriterSettingsDto, CommandError> {
    let slot = serde_json::to_value(settings).map_err(invalid)?;
    engine
        .settings_validate(&slot)
        .map_err(CommandError::from)?;
    let saved = state.settings_set(&serde_json::json!({ "writer": slot }))?;
    settings_dto(engine, WriterSettings::from_slot(saved.get("writer")))
}

fn now_ms(state: &State<'_, AppState>) -> Result<i64, CommandError> {
    Ok(state.handles().map_err(CommandError::from)?.clock.now_ms())
}

/// W4: Settings → Writer as saved, every field present, with the safeguard counts.
#[tauri::command]
#[specta::specta]
pub async fn writer_settings_get(
    state: State<'_, AppState>,
) -> Result<WriterSettingsDto, CommandError> {
    let engine = engine(&state)?;
    settings_dto(engine.as_ref(), stored(&state)?)
}

/// W4: saves Settings → Writer whole. An instruction whose text changes keeps the text it
/// replaces among the earlier versions; a refused value names its field and saves nothing.
#[tauri::command]
#[specta::specta]
pub async fn writer_settings_save(
    state: State<'_, AppState>,
    value: WriterSettingsDto,
) -> Result<WriterSettingsDto, CommandError> {
    let engine = engine(&state)?;
    let incoming = settings_of(value);
    let shown = serde_json::to_value(&incoming).map_err(invalid)?;
    engine
        .settings_validate(&shown)
        .map_err(CommandError::from)?;
    let next = stored(&state)?.saved_over(incoming, now_ms(&state)?);
    save_slot(&state, engine.as_ref(), &next)
}

/// W4: **Reset to default**: the language's instruction returns to the shipped one; the saved
/// text is kept among the earlier versions.
#[tauri::command]
#[specta::specta]
pub async fn writer_instruction_reset(
    state: State<'_, AppState>,
    language: String,
) -> Result<WriterSettingsDto, CommandError> {
    let engine = engine(&state)?;
    if !LANGUAGES.contains(&language.as_str()) {
        return Err(invalid("language must be English or Swedish"));
    }
    let mut next = stored(&state)?;
    next.reset_instruction(&language, now_ms(&state)?);
    save_slot(&state, engine.as_ref(), &next)
}

/// A23 **Export…**: writes the saved writer settings (only the `writer` slot, without earlier
/// instruction versions) to `path` as JSON. Never any other settings slot.
#[tauri::command]
#[specta::specta]
pub async fn writer_settings_export(
    state: State<'_, AppState>,
    path: String,
) -> Result<(), CommandError> {
    let file = stored(&state)?.export_value();
    let text = serde_json::to_string_pretty(&file).map_err(invalid)?;
    std::fs::write(&path, text + "\n")
        .map_err(|e| invalid(format!("cannot write the file: {e}")))?;
    tracing::info!(target: "writer", "writer.settings_exported");
    Ok(())
}

/// A23 **Import…**: reads an exported file at `path`, validates it as Save does, and saves it as
/// the writer settings; each instruction it changes keeps the replaced text among the earlier
/// versions. A refused file names the reason and saves nothing.
#[tauri::command]
#[specta::specta]
pub async fn writer_settings_import(
    state: State<'_, AppState>,
    path: String,
) -> Result<WriterSettingsDto, CommandError> {
    let engine = engine(&state)?;
    let text = std::fs::read_to_string(&path)
        .map_err(|e| invalid(format!("cannot read the file: {e}")))?;
    let file: serde_json::Value = serde_json::from_str(&text)
        .map_err(|_| invalid("this is not a Session Manager writer settings file"))?;
    let incoming = WriterSettings::import_value(&file).map_err(invalid)?;
    let next = stored(&state)?.saved_over(incoming, now_ms(&state)?);
    tracing::info!(target: "writer", "writer.settings_imported");
    save_slot(&state, engine.as_ref(), &next)
}

/// The exact request a write would send (instructions, input and schema; the last two as JSON
/// text).
#[derive(Serialize, Type)]
pub struct WriterPreviewDto {
    pub instructions: String,
    pub input: String,
    pub schema: String,
}

/// W4: **Preview what is sent**: the request a write would send now in `language`, for the
/// built-in example item, under the saved settings. Nothing is sent and no run is made.
#[tauri::command]
#[specta::specta]
pub async fn writer_preview(
    state: State<'_, AppState>,
    language: String,
) -> Result<WriterPreviewDto, CommandError> {
    let engine = engine(&state)?;
    let slot = serde_json::to_value(stored(&state)?).map_err(invalid)?;
    let preview = engine.preview(&slot, &language)?;
    Ok(WriterPreviewDto {
        instructions: preview.instructions,
        input: preview.input.to_string(),
        schema: preview.schema.to_string(),
    })
}
