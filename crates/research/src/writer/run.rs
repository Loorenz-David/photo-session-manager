//! W2: the writer run (`master_plan_writer.md` §5 WC3, WC4, WC5 "Regenerate once", WC7).
//!
//! Unit 3: when a `write` run is queued ([`request_decision`], `ResearchService::writer_request`)
//! and what stops it from being created ([`WriterBlock`]). Unit 4: what it does once started
//! ([`execute`]).

use std::collections::BTreeMap;

use psm_core::{
    Result,
    domain::attribute::{AiConfidence, AttributeGroup, CoreType},
    mechanisms::values::is_severity_set,
    ports::store::StoreTx,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value as Json, json};

use crate::{
    api::Reason,
    config::ResearchSettings,
    domain::{RunState, RunTrigger, Stage, StepKind},
    gateway::{Answer, CallRequest},
    ports::ModelRequest,
    rules::fingerprint::fingerprint,
    service::{MODEL_ACCOUNT, ResearchService, form_snapshot},
    stages::{Asked, Ctx, ExecuteOutcome, Flow, Work, display, step},
    store::{self, NewRun, NewStep, OutputRow},
    writer::{
        api::{Preview, Requested},
        config::{WriterSettings, WriterSnapshot},
        defaults,
        input::{requestable_texts, text_keys},
        note::{PRODUCT_TITLE, decision, reviewed},
        postcheck::{CheckContext, Hit, HitCode, SeverityField, check},
        safeguard::{
            Candidate, FilterInput, Filtered, FindingState, FindingType, KeyState, Survivor, filter,
        },
        schema,
        writes::{outcome_text, write_text},
    },
    writes::WriteOutcome,
};

/// Why a writer run was asked for (`master_plan_writer.md` §4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Origin {
    /// "Next to review →" after the AI review.
    NextToReview,
    /// Leaving a session whose AI review shows "All checked ✓".
    AllChecked,
    /// The "Write texts" button.
    WriteTexts,
    /// An answer to a writer question or finding (internal).
    Answer,
}

impl Origin {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NextToReview => "next_to_review",
            Self::AllChecked => "all_checked",
            Self::WriteTexts => "write_texts",
            Self::Answer => "answer",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        [
            Self::NextToReview,
            Self::AllChecked,
            Self::WriteTexts,
            Self::Answer,
        ]
        .into_iter()
        .find(|o| o.as_str() == text)
    }

    /// The origins of automatic writing (WC3.4).
    pub fn automatic(self) -> bool {
        matches!(self, Self::NextToReview | Self::AllChecked)
    }
}

/// Why the writer does not run (`master_plan_writer.md` §4 `writer::WriterBlock`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WriterBlock {
    NoIntegration,
    NoModelKey,
    PriceMissing { provider: String, model: String },
    NothingToWrite,
    ResearchBusy,
    DayLimit,
    Offline,
}

impl WriterBlock {
    pub fn code(&self) -> &'static str {
        match self {
            Self::NoIntegration => "NoIntegration",
            Self::NoModelKey => "NoModelKey",
            Self::PriceMissing { .. } => "PriceMissing",
            Self::NothingToWrite => "NothingToWrite",
            Self::ResearchBusy => "ResearchBusy",
            Self::DayLimit => "DayLimit",
            Self::Offline => "Offline",
        }
    }

    pub fn sentence(&self) -> String {
        match self {
            Self::NoIntegration => "Connect a shop to write texts".to_owned(),
            Self::NoModelKey => "Add the model key in Settings → Research".to_owned(),
            Self::PriceMissing { provider, model } => {
                format!("No price is set for {provider} {model}")
                    .trim_end()
                    .to_owned()
            }
            Self::NothingToWrite => "This item has no text the writer can write".to_owned(),
            Self::ResearchBusy => {
                "Research is still working on this session; write texts when it finishes".to_owned()
            }
            Self::DayLimit => "Day limit reached".to_owned(),
            Self::Offline => "Offline".to_owned(),
        }
    }

    pub fn reason(&self) -> Reason {
        Reason {
            code: self.code().to_owned(),
            sentence: self.sentence(),
        }
    }
}

/// The facts WC3 decides from, read in the request's transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestInput {
    /// The session has an item (not the Unassigned session).
    pub has_item: bool,
    /// A run of the session with trigger ∉ {`browse`, `write`} is queued or running.
    pub research_busy: bool,
    /// The first failing creation-blocking prerequisite (WC3.3).
    pub block: Option<WriterBlock>,
    pub origin: Origin,
    /// `writer.auto`.
    pub auto: bool,
    /// The states of the session's `write` runs.
    pub write_states: Vec<RunState>,
}

/// What a request does (WC3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RequestDecision {
    Refused(WriterBlock),
    /// Nothing, no reason shown (an automatic origin that is off or already used).
    Nothing,
    /// A `write` run is already queued: nothing new, `queued: true`.
    AlreadyQueued,
    /// Insert one `queued` `write` run.
    Insert,
}

/// WC3 items 1–6 in order (the pure part of `request`).
pub fn request_decision(input: &RequestInput) -> RequestDecision {
    if !input.has_item {
        return RequestDecision::Refused(WriterBlock::NothingToWrite);
    }
    if input.research_busy {
        return RequestDecision::Refused(WriterBlock::ResearchBusy);
    }
    if let Some(block) = &input.block {
        return RequestDecision::Refused(block.clone());
    }
    if input.origin.automatic()
        && (!input.auto
            || input.write_states.iter().any(|s| {
                matches!(
                    s,
                    RunState::Queued
                        | RunState::Running
                        | RunState::Finished
                        | RunState::StoppedAtCap
                )
            }))
    {
        return RequestDecision::Nothing;
    }
    if input.write_states.contains(&RunState::Queued) {
        return RequestDecision::AlreadyQueued;
    }
    RequestDecision::Insert
}

/// Where a queued writer run keeps its origin until the start transaction snapshots it.
pub(crate) const ORIGIN_KEY: &str = "writer_origin";

impl ResearchService {
    /// WC3.3: the first failing creation-blocking prerequisite of a writer run of the item.
    pub(crate) fn writer_block(
        &self,
        item_id: i64,
        settings: &ResearchSettings,
        writer: &WriterSettings,
    ) -> Result<Option<WriterBlock>> {
        if self.shop.is_none() {
            return Ok(Some(WriterBlock::NoIntegration));
        }
        if self.providers.model.is_none() || !self.key_set(MODEL_ACCOUNT) {
            return Ok(Some(WriterBlock::NoModelKey));
        }
        let provider = settings.providers.resolved().model;
        if !settings
            .fallbacks()
            .iter()
            .any(|f| f.provider == provider && f.model == writer.model)
        {
            return Ok(Some(WriterBlock::PriceMissing {
                provider,
                model: writer.model.clone(),
            }));
        }
        let form = form_snapshot(
            &self.store,
            self.clock.as_ref(),
            self.integration.as_deref(),
            item_id,
        )?;
        if requestable_texts(&form, writer).is_empty() {
            return Ok(Some(WriterBlock::NothingToWrite));
        }
        Ok(None)
    }

    /// `writer::run::request` (WC3): queues a `write` run of the session, or says why not.
    pub fn writer_request(&self, session_id: i64, origin: Origin) -> Result<Requested> {
        let item = self.session_item(session_id)?;
        let writer = self.writer_settings();
        let block = match item {
            Some(item_id) => self.writer_block(item_id, &self.settings(), &writer)?,
            None => None,
        };
        let now = self.clock.now_ms();
        let (decision, queued) = self.store.research_transaction(|tx| {
            let conn = tx.conn();
            let runs = match item {
                Some(_) => store::all_runs(conn, Some(session_id))?,
                None => Vec::new(),
            };
            let decision = request_decision(&RequestInput {
                has_item: item.is_some(),
                research_busy: runs.iter().any(|r| {
                    !matches!(r.trigger, RunTrigger::Browse | RunTrigger::Write)
                        && matches!(r.state, RunState::Queued | RunState::Running)
                }),
                block: block.clone(),
                origin,
                auto: writer.auto,
                write_states: runs
                    .iter()
                    .filter(|r| r.trigger == RunTrigger::Write)
                    .map(|r| r.state)
                    .collect(),
            });
            let queued_before = runs
                .iter()
                .any(|r| r.trigger == RunTrigger::Write && r.state == RunState::Queued);
            let queued = match &decision {
                RequestDecision::Insert => {
                    store::insert_run(
                        conn,
                        &NewRun {
                            item_id: item.unwrap_or_default(),
                            session_id,
                            trigger: RunTrigger::Write,
                            state: RunState::Queued,
                            queued_at: now,
                            started_at: None,
                            settings_snapshot: Some(
                                json!({ ORIGIN_KEY: origin.as_str() }).to_string(),
                            ),
                            form_snapshot: None,
                            input_snapshot: None,
                        },
                    )?;
                    true
                }
                RequestDecision::AlreadyQueued => true,
                RequestDecision::Nothing => queued_before,
                RequestDecision::Refused(_) => false,
            };
            Ok((decision, queued))
        })?;
        let reason = match &decision {
            RequestDecision::Refused(block) => Some(block.clone()),
            _ => None,
        };
        if decision == RequestDecision::Insert {
            self.changed(session_id);
        }
        tracing::info!(
            target: "psm::writer",
            "writer.requested session={session_id} origin={} queued={queued} reason={}",
            origin.as_str(),
            reason.as_ref().map_or("", WriterBlock::code)
        );
        Ok(Requested {
            queued,
            reason: reason.map(|r| r.reason()),
        })
    }
}

/// How often the photographer answered and dismissed one finding type (WC6 "Counts").
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FindingCount {
    #[serde(rename = "type")]
    pub finding_type: String,
    pub answered: i64,
    pub dismissed: i64,
}

impl ResearchService {
    /// Resolves an `open` finding or question; returns its session.
    fn resolve_finding(
        &self,
        finding_id: i64,
        state: FindingState,
        answer: Option<&str>,
    ) -> Result<i64> {
        let now = self.clock.now_ms();
        let session = self.store.research_transaction(|tx| {
            use rusqlite::OptionalExtension;
            let conn = tx.conn();
            let session: Option<i64> = conn
                .query_row(
                    "SELECT session_id FROM writer_findings WHERE id=?1 AND state='open'",
                    [finding_id],
                    |r| r.get(0),
                )
                .optional()
                .map_err(store::db)?;
            let Some(session) = session else {
                return Err(psm_core::Error::InvalidData(
                    "this question is no longer open".into(),
                ));
            };
            conn.execute(
                "UPDATE writer_findings SET state=?1, answer=?2, resolved_at=?3 WHERE id=?4",
                rusqlite::params![state.as_str(), answer, now, finding_id],
            )
            .map_err(store::db)?;
            Ok(session)
        })?;
        self.changed(session);
        Ok(session)
    }

    /// WC6 "Answer": the finding becomes `answered` with the answer, then the session's texts are
    /// written again (`request(session, Answer)`), the answer among the writer's input.
    pub fn writer_answer(&self, finding_id: i64, answer: &str) -> Result<Requested> {
        let answer = answer.trim();
        if answer.is_empty() {
            return Err(psm_core::Error::InvalidData("write an answer first".into()));
        }
        let session = self.resolve_finding(finding_id, FindingState::Answered, Some(answer))?;
        self.writer_request(session, Origin::Answer)
    }

    /// WC6 "Dismiss": the finding becomes `dismissed`; nothing is written again.
    pub fn writer_dismiss(&self, finding_id: i64) -> Result<()> {
        self.resolve_finding(finding_id, FindingState::Dismissed, None)
            .map(|_| ())
    }

    /// WC6 "Counts": per finding type, how many were answered and how many dismissed.
    pub fn writer_counts(&self) -> Result<Vec<FindingCount>> {
        let rows: Vec<(String, String, i64)> = self.store.research_transaction(|tx| {
            let mut stmt = tx
                .conn()
                .prepare(
                    "SELECT type, state, COUNT(*) FROM writer_findings \
                     WHERE state IN ('answered','dismissed') GROUP BY type, state",
                )
                .map_err(store::db)?;
            stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
                .map_err(store::db)?
                .collect::<rusqlite::Result<Vec<_>>>()
                .map_err(store::db)
        })?;
        let count = |t: &str, state: &str| {
            rows.iter()
                .find(|(rt, rs, _)| rt == t && rs == state)
                .map_or(0, |(_, _, n)| *n)
        };
        Ok(FindingType::ALL
            .iter()
            .map(|t| FindingCount {
                finding_type: t.as_str().to_owned(),
                answered: count(t.as_str(), "answered"),
                dismissed: count(t.as_str(), "dismissed"),
            })
            .collect())
    }
}

/// The unit keys of a writer run (`master_plan_writer.md` §4 Storage).
pub const TEXTS_UNIT: &str = "texts";
pub const RETRY_UNIT: &str = "retry";
pub const RESULT_UNIT: &str = "result";
/// The hit added to a text whose rewrite the run cost limit stopped (WC5, WC7).
pub const NOT_RECHECKED: &str = "not checked again: run cost limit";

/// The instructions a writer call sends: the instruction text, the title formula, the dimensions
/// line and the examples (each "Example <name>" with its fields and texts).
pub fn instructions_text(snapshot: &WriterSnapshot) -> String {
    let language = snapshot.settings.language.as_str();
    let mut out = vec![
        snapshot.instruction.clone(),
        defaults::title_formula(language).to_owned(),
        format!("Dimensions line: {}", defaults::dimensions_line(language)),
    ];
    for example in &snapshot.examples {
        out.push(format!(
            "Example {}\nFields: {}\nTitle: {}\nDescription:\n{}\nCondition report: {}\n\
             Search title: {}\nSearch description: {}",
            example.name,
            example.fields,
            example.title,
            example.description,
            example.condition_report,
            example.seo_title,
            example.seo_description,
        ));
    }
    out.join("\n\n")
}

/// A writer request as it is sent (WC7; 08C §8 "Preview what is sent"): the instructions, the
/// input and the strict output schema. A run's call and the preview are both built here.
pub fn request_of(snapshot: &WriterSnapshot, input: &Json) -> Preview {
    Preview {
        instructions: instructions_text(snapshot),
        input: input.clone(),
        schema: schema::schema_for_write(&schema::requested(input)),
    }
}

/// The built-in example item of the preview: example A's fields line (§9.4) split into
/// labelled fields, each a fact the owner typed.
pub const PREVIEW_FIELDS: [(&str, &str, &str); 12] = [
    ("example.type", "Type", "dining chair"),
    ("example.designer", "Designer", "Børge Mogensen"),
    ("example.manufacturer", "Manufacturer", "FDB Møbler"),
    ("example.model", "Model", "J39"),
    ("example.era", "Era", "1960s"),
    ("example.country", "Country", "Denmark"),
    ("example.materials", "Materials", "solid beech, papercord"),
    (
        "example.dimensions",
        "Dimensions",
        "W 49 D 46 H 78, seat 45",
    ),
    ("example.condition", "Condition", "light scratches"),
    ("example.stability", "Stability", "stable"),
    (
        "example.restoration",
        "Restoration",
        "sanded, oiled, seat rewoven",
    ),
    ("example.photo_note", "Photo note", "photo 4 shows scratch"),
];

/// The preview's input for `settings` (W4 task 4): the example item's fields, nothing else read
/// from storage, every text the writer can write requested.
pub fn preview_input(settings: &WriterSettings) -> Json {
    let fields = PREVIEW_FIELDS
        .iter()
        .map(|(key, label, value)| {
            json!({
                "key": key,
                "label": label,
                "value": value,
                "decision": crate::writer::note::Decision::Typed.code(),
                "fact": true,
            })
        })
        .collect();
    crate::writer::input::assemble(
        crate::writer::input::InputParts {
            fields,
            texts: text_keys(settings)
                .into_iter()
                .map(|(name, _)| name.to_owned())
                .collect(),
            ..Default::default()
        },
        settings,
    )
}

impl ResearchService {
    /// W4 `WriterApi::preview`: the exact request a write would send under the settings `slot`
    /// for `language`, for the built-in example item. No provider call, no run row.
    pub fn writer_preview(&self, slot: &Json, language: &str) -> Result<Preview> {
        WriterSettings::validate(slot).map_err(psm_core::Error::InvalidData)?;
        let mut settings = WriterSettings::from_slot(Some(slot));
        if !crate::writer::config::LANGUAGES.contains(&language) {
            return Err(psm_core::Error::InvalidData(
                "language must be English or Swedish".to_owned(),
            ));
        }
        language.clone_into(&mut settings.language);
        let snapshot = WriterSnapshot::resolve(&settings, "preview");
        Ok(request_of(&snapshot, &preview_input(&settings)))
    }
}

/// One text of an answer: its value (the description's paragraphs joined by a blank line), the
/// facts it says it used and its reason.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Drafted {
    pub value: String,
    pub facts: Vec<String>,
    pub reason: String,
}

/// The requested texts an answer holds, and the names it holds that were not requested.
fn texts_of(answer: &Json, requested: &[String]) -> (BTreeMap<String, Drafted>, Vec<String>) {
    let mut texts = BTreeMap::new();
    let mut extra = Vec::new();
    for (name, entry) in answer["texts"].as_object().into_iter().flatten() {
        if !requested.contains(name) {
            extra.push(name.clone());
            continue;
        }
        let value = match entry["paragraphs"].as_array() {
            Some(paragraphs) => paragraphs
                .iter()
                .filter_map(Json::as_str)
                .map(str::trim)
                .filter(|p| !p.is_empty())
                .collect::<Vec<_>>()
                .join("\n\n"),
            None => entry["text"].as_str().unwrap_or_default().trim().to_owned(),
        };
        texts.insert(
            name.clone(),
            Drafted {
                value,
                facts: entry["facts"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(|f| f.as_str().map(str::to_owned))
                    .collect(),
                reason: entry["reason"]
                    .as_str()
                    .unwrap_or_default()
                    .trim()
                    .to_owned(),
            },
        );
    }
    (texts, extra)
}

fn check_list(requested: &[String], texts: &BTreeMap<String, Drafted>) -> Vec<(String, String)> {
    requested
        .iter()
        .filter_map(|n| texts.get(n).map(|d| (n.clone(), d.value.clone())))
        .collect()
}

/// What the post-check of a writer run is read from (WC5): the input the call carried, the
/// form snapshot, the run's settings and the attribution names stored now.
fn check_context(ctx: &Ctx<'_>, input: &Json, settings: &WriterSettings) -> Result<CheckContext> {
    let fields: Vec<&Json> = input["fields"].as_array().into_iter().flatten().collect();
    let facts: Vec<String> = fields
        .iter()
        .filter(|f| f["fact"].as_bool() == Some(true))
        .filter_map(|f| f["value"].as_str().map(str::to_owned))
        .collect();
    let notes: Vec<String> = input["note"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|o| o["text"].as_str().map(str::to_owned))
        .collect();
    let mut severity = Vec::new();
    for field in &fields {
        let key = field["key"].as_str().unwrap_or_default();
        let Some(snapshot) = ctx.form.fields.iter().find(|f| f.definition.key == key) else {
            continue;
        };
        if snapshot.step == AttributeGroup::Condition
            && snapshot.definition.core_type == CoreType::Choice
            && is_severity_set(&snapshot.definition.choices)
        {
            severity.push(SeverityField {
                key: key.to_owned(),
                label: snapshot.definition.label.clone(),
                value: field["value"].as_str().unwrap_or_default().to_owned(),
                fact: field["fact"].as_bool() == Some(true),
            });
        }
    }
    let detailed_key = &ctx.config.detailed_condition.key;
    let detailed = fields
        .iter()
        .find(|f| !detailed_key.is_empty() && f["key"].as_str() == Some(detailed_key))
        .filter(|f| f["fact"].as_bool() == Some(true))
        .and_then(|f| f["value"].as_str().map(str::to_owned));
    // WC5.11: the chosen restoration steps, when the field is visible and its value a fact.
    let restoration_key = psm_core::services::attributes::LOCAL_RESTORATION;
    let restoration: Vec<String> = fields
        .iter()
        .find(|f| f["key"].as_str() == Some(restoration_key))
        .filter(|f| f["fact"].as_bool() == Some(true))
        .and_then(|_| {
            ctx.form
                .fields
                .iter()
                .find(|f| f.definition.key == restoration_key)
        })
        .and_then(|f| f.value.as_deref())
        .and_then(|v| serde_json::from_str::<Vec<String>>(v).ok())
        .unwrap_or_default();
    let keys = ctx.config.attribution_keys.clone();
    let integration = ctx.svc.integration.clone();
    let attribution_names = ctx.svc.store.research_transaction(|tx| {
        let mut names: Vec<String> = Vec::new();
        let mut definitions = psm_core::services::attributes::builtin_definitions();
        if let Some(integration) = &integration {
            definitions.extend(tx.attribute_definitions(integration)?);
        }
        for definition in definitions.iter().filter(|d| keys.contains(&d.key)) {
            names.extend(definition.choices.iter().cloned());
        }
        for key in &keys {
            let mut stmt = tx
                .conn()
                .prepare("SELECT value FROM attribute_values WHERE key=?1")
                .map_err(store::db)?;
            let values = stmt
                .query_map([key], |r| r.get::<_, String>(0))
                .map_err(store::db)?
                .collect::<rusqlite::Result<Vec<_>>>()
                .map_err(store::db)?;
            for value in values {
                if let Ok(json) = serde_json::from_str::<Json>(&value) {
                    match json {
                        Json::Array(items) => names.extend(items.iter().map(display)),
                        other => names.push(display(&other)),
                    }
                }
            }
        }
        Ok(names)
    })?;
    Ok(CheckContext {
        language: settings.language.clone(),
        lengths: settings.lengths.clone(),
        banned_extra: settings.banned_extra.clone(),
        facts,
        notes,
        severity,
        damage_terms: settings.damage_terms.clone(),
        detailed,
        attribution_names,
        restoration,
    })
}

/// One writer call as a unit's work: the answer committed as the unit's output.
fn call(
    ctx: &Ctx<'_>,
    unit_key: &str,
    instructions: &str,
    input: &Json,
    language: &str,
) -> Result<Work> {
    let stage = ctx.config.stage(Stage::Write);
    // HW4: a writer request never carries an image.
    let request = CallRequest::Model(ModelRequest {
        stage: Stage::Write,
        unit_key: unit_key.to_owned(),
        model: stage.model,
        effort: stage.effort,
        language: language.to_owned(),
        instructions: instructions.to_owned(),
        input: input.clone(),
        images: Vec::new(),
        listing_images: Vec::new(),
    });
    let (asked, _) = ctx.ask(Stage::Write, unit_key, request)?;
    Ok(match asked {
        Asked::Answer(Answer::Model(reply)) => Work::Commit {
            output: json!({ "answer": reply.output }),
            steps: Vec::new(),
        },
        Asked::Answer(_) => Work::Commit {
            output: json!({ "answer": null }),
            steps: Vec::new(),
        },
        Asked::NotDetermined(text) => Work::Commit {
            output: json!({ "answer": null, "refusal": text }),
            steps: Vec::new(),
        },
        Asked::Work(work) => work,
    })
}

fn hits_json(hits: &[Hit]) -> Json {
    serde_json::to_value(hits).unwrap_or(Json::Null)
}

/// The run's end after its `result` unit committed.
fn finish(ctx: &Ctx<'_>, output: &Json) -> Result<ExecuteOutcome> {
    let (state, outcome) = if output["ending"].as_str() == Some(RunState::StoppedAtCap.as_str()) {
        (RunState::StoppedAtCap, ExecuteOutcome::StoppedAtCap)
    } else {
        (RunState::Finished, ExecuteOutcome::Finished)
    };
    crate::decide::end(ctx, state)?;
    Ok(outcome)
}

/// Runs a started `write` run (WC4–WC7): unit `texts` (the call), unit `retry` (the rewrite of
/// the texts with post-check hits, at most once), unit `result` (the writes, one step per text,
/// the findings and the output, in one transaction). A committed unit never runs again.
pub(crate) fn execute(ctx: &Ctx<'_>) -> Result<ExecuteOutcome> {
    let committed = ctx.svc.store.research_transaction(|tx| {
        store::output(tx.conn(), ctx.run.id, Stage::Write.as_str(), RESULT_UNIT)
    })?;
    if let Some(row) = committed {
        return finish(
            ctx,
            &serde_json::from_str(&row.output).unwrap_or(Json::Null),
        );
    }
    let Some(snapshot) = ctx.config.writer.clone() else {
        let now = ctx.svc.clock.now_ms();
        ctx.svc.store.research_transaction(|tx| {
            store::set_run_state(
                tx.conn(),
                ctx.run.id,
                RunState::Failed,
                Some("the run has no writer settings"),
                Some(now),
            )
        })?;
        ctx.svc.changed(ctx.run.session_id);
        return Ok(ExecuteOutcome::Failed);
    };
    let settings = &snapshot.settings;
    let input: Json = ctx
        .run
        .input_snapshot
        .as_deref()
        .and_then(|t| serde_json::from_str(t).ok())
        .unwrap_or(Json::Null);
    let requested = schema::requested(&input);
    // W4: the request is built by the function the preview uses.
    let instructions = request_of(&snapshot, &input).instructions;
    let stage = ctx.config.stage(Stage::Write);
    let language = settings.language.clone();
    let identity = json!({
        "input": input,
        "instructions": instructions,
        "model": stage.model,
        "effort": stage.effort,
    });
    let first = match ctx.unit(Stage::Write, TEXTS_UNIT, identity, |ctx| {
        call(ctx, TEXTS_UNIT, &instructions, &input, &language)
    })? {
        Flow::Done(done) => done.output,
        Flow::Skip => {
            crate::decide::end(ctx, RunState::StoppedAtCap)?;
            return Ok(ExecuteOutcome::StoppedAtCap);
        }
        Flow::Halt(outcome) => return Ok(outcome),
    };
    let first_answer = first["answer"].clone();
    let (mut texts, extra) = texts_of(&first_answer, &requested);
    let check_ctx = check_context(ctx, &input, settings)?;
    let first_hits = check(&check_list(&requested, &texts), &check_ctx);
    log_post_check(ctx.run.id, 1, &first_hits);
    let mut attempts = vec![json!({ "attempt": 1, "hits": hits_json(&first_hits) })];
    let mut hits = first_hits.clone();
    let mut regenerated: Vec<String> = Vec::new();
    let mut stopped = false;
    if !first_hits.is_empty() {
        let rewrite: Vec<String> = requested
            .iter()
            .filter(|n| first_hits.iter().any(|h| &h.text == *n))
            .cloned()
            .collect();
        let mut retry_input = input.clone();
        retry_input["texts"] = json!(rewrite);
        retry_input["previous"] = Json::Object(
            rewrite
                .iter()
                .filter_map(|n| texts.get(n).map(|d| (n.clone(), json!(d.value))))
                .collect(),
        );
        retry_input["hits"] = hits_json(&first_hits);
        let named: Vec<String> = first_hits
            .iter()
            .map(|h| format!("{}: {}", h.text, h.detail))
            .collect();
        let retry_instructions = format!(
            "{instructions}\n\nRewrite only the texts listed in `texts`. The earlier versions are \
             in `previous` and failed these checks: {}.",
            named.join("; ")
        );
        let identity = json!({
            "input": retry_input,
            "instructions": retry_instructions,
            "model": stage.model,
            "effort": stage.effort,
        });
        match ctx.unit(Stage::Write, RETRY_UNIT, identity, |ctx| {
            call(
                ctx,
                RETRY_UNIT,
                &retry_instructions,
                &retry_input,
                &language,
            )
        })? {
            Flow::Done(done) => {
                let (rewritten, _) = texts_of(&done.output["answer"], &rewrite);
                for (name, drafted) in rewritten {
                    regenerated.push(name.clone());
                    texts.insert(name, drafted);
                }
                hits = check(&check_list(&requested, &texts), &check_ctx);
                log_post_check(ctx.run.id, 2, &hits);
                attempts.push(json!({ "attempt": 2, "hits": hits_json(&hits) }));
            }
            Flow::Skip => {
                stopped = true;
                for name in &rewrite {
                    hits.push(Hit {
                        text: name.clone(),
                        code: HitCode::NotRechecked,
                        detail: NOT_RECHECKED.to_owned(),
                        key: None,
                    });
                }
            }
            Flow::Halt(outcome) => return Ok(outcome),
        }
    }
    let keys = text_keys(settings);
    let key_of = |name: &str| {
        keys.iter()
            .find(|(n, _)| *n == name)
            .map_or_else(|| name.to_owned(), |(_, k)| k.clone())
    };
    let label = |key: &str| {
        ctx.form
            .fields
            .iter()
            .find(|f| f.definition.key == key)
            .map_or_else(|| key.to_owned(), |f| f.definition.label.clone())
    };
    let allowed: Vec<String> = requested.iter().map(|n| key_of(n)).collect();
    let ending = if stopped {
        RunState::StoppedAtCap
    } else {
        RunState::Finished
    };
    let now = ctx.svc.clock.now_ms();
    let integration = ctx.svc.integration.clone();
    let item_id = ctx.run.item_id;
    let identity = json!({ "texts": texts, "hits": hits_json(&hits) });
    let fingerprint = fingerprint(Stage::Write.number(), &identity)
        .map_err(|e| psm_core::Error::InvalidData(e.to_string()))?;
    let candidates = candidates(&first_answer, &hits, &key_of("condition_report"), &allowed);
    let layout: Vec<String> = ctx
        .form
        .fields
        .iter()
        .map(|f| f.definition.key.clone())
        .collect();
    let session_id = ctx.run.session_id;
    let (written, flagged, findings, questions) = ctx.svc.store.research_transaction(|tx| {
        let mut writes: Vec<Json> = Vec::new();
        let mut texts_out = Map::new();
        let mut steps: Vec<NewStep> = Vec::new();
        let (mut written, mut flagged) = (0, 0);
        // WC6 rule 2: the text keys written or unchanged by this run.
        let mut written_keys: Vec<String> = Vec::new();
        for name in &requested {
            let key = key_of(name);
            let field = label(&key);
            let Some(drafted) = texts.get(name).filter(|d| !d.value.is_empty()) else {
                let mut nd = step(
                    Stage::Write,
                    StepKind::NotDetermined,
                    format!("{field}: no text"),
                );
                nd.detail = "the writer gave no text".to_owned();
                nd.field_keys = vec![key.clone()];
                steps.push(nd);
                writes.push(json!({ "key": key, "outcome": "no text" }));
                continue;
            };
            let own: Vec<&Hit> = hits.iter().filter(|h| &h.text == name).collect();
            let facts: Vec<String> = drafted.facts.iter().map(|f| label(f)).collect();
            let mut reason = format!("{}\nFacts: {}", drafted.reason, facts.join(", "));
            if !own.is_empty() {
                reason.push_str(&format!(
                    "\nCheck: {}",
                    own.iter()
                        .map(|h| h.detail.as_str())
                        .collect::<Vec<_>>()
                        .join("; ")
                ));
            }
            let confidence = if own.is_empty() {
                AiConfidence::Medium
            } else {
                AiConfidence::Low
            };
            let outcome = write_text(
                tx,
                now,
                integration.as_deref(),
                item_id,
                &key,
                &json!(drafted.value),
                confidence,
                &reason,
                &allowed,
            )?;
            texts_out.insert(
                name.clone(),
                json!({
                    "key": key,
                    "value": drafted.value,
                    "facts": drafted.facts,
                    "reason": reason,
                    "confidence": confidence.as_str(),
                    "hits": own,
                }),
            );
            writes.push(json!({ "key": key, "outcome": outcome_text(&outcome) }));
            match outcome {
                WriteOutcome::Written(_) | WriteOutcome::Unchanged => {
                    written += 1;
                    written_keys.push(key.clone());
                    if !own.is_empty() {
                        flagged += 1;
                    }
                    let mut concluded = step(
                        Stage::Write,
                        StepKind::Concluded,
                        format!("{field}: written ({})", confidence.as_str()),
                    );
                    concluded.detail = reason.clone();
                    concluded.field_keys = vec![key.clone()];
                    steps.push(concluded);
                }
                other => {
                    let text = outcome_text(&other);
                    let mut nd = step(
                        Stage::Write,
                        StepKind::NotDetermined,
                        format!("{field}: {text}"),
                    );
                    nd.detail = text;
                    nd.field_keys = vec![key.clone()];
                    steps.push(nd);
                }
            }
        }
        for name in &extra {
            writes.push(json!({ "key": key_of(name), "outcome": "not requested" }));
        }
        // WC6: the filter reads the keys' states after this run's writes, in this transaction.
        let filtered = store_findings(
            tx,
            &FindingsTarget {
                item_id,
                session_id,
                run_id: ctx.run.id,
                now,
            },
            &candidates,
            &layout,
            &written_keys,
            settings,
        )?;
        if !filtered.dropped_questions.is_empty() {
            let mut capped = step(
                Stage::Write,
                StepKind::NotDetermined,
                format!(
                    "{} not asked (at most {} per write)",
                    filtered.dropped_questions.len(),
                    settings.max_questions
                ),
            );
            capped.detail = filtered.dropped_questions.join("\n");
            steps.push(capped);
        }
        let (stored_findings, stored_questions): (Vec<&Survivor>, Vec<&Survivor>) =
            filtered.stored.iter().partition(|s| s.kind == "finding");
        let output = json!({
            "texts": texts_out,
            "hits": attempts,
            "writes": writes,
            "findings": stored_findings,
            "questions": stored_questions,
            "dropped_questions": filtered.dropped_questions,
            "regenerated": regenerated,
            "ending": ending.as_str(),
        });
        let conn = tx.conn();
        store::insert_output(
            conn,
            &OutputRow {
                run_id: ctx.run.id,
                stage: Stage::Write.as_str().to_owned(),
                unit_key: RESULT_UNIT.to_owned(),
                fingerprint: fingerprint.clone(),
                output: output.to_string(),
                reused_from_run_id: None,
                committed_at: now,
            },
        )?;
        for s in &steps {
            store::insert_step(conn, ctx.run.id, s, now)?;
        }
        store::set_stage_reached(conn, ctx.run.id, Stage::Write.as_str())?;
        Ok((
            written,
            flagged,
            stored_findings.len(),
            stored_questions.len(),
        ))
    })?;
    ctx.svc.changed(ctx.run.session_id);
    let cost = ctx
        .svc
        .store
        .research_transaction(|tx| store::run(tx.conn(), ctx.run.id))?
        .map_or(0, |r| r.cost_micro_usd);
    tracing::info!(
        target: "psm::writer",
        "writer.finished run={} written={written} flagged={flagged} regenerated={} \
         findings={findings} questions={questions} cost_micro_usd={cost}",
        ctx.run.id,
        regenerated.len()
    );
    finish(ctx, &json!({ "ending": ending.as_str() }))
}

/// The findings and questions of a run before the filter (WC5, WC6): the model's findings and
/// questions of its first answer, and the post-check's damage and names hits that are left.
fn candidates(answer: &Json, hits: &[Hit], report_key: &str, texts: &[String]) -> Vec<Candidate> {
    let strings = |value: &Json| -> Vec<String> {
        value
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|k| k.as_str().map(str::to_owned))
            .collect()
    };
    let mut out = Vec::new();
    for finding in answer["findings"].as_array().into_iter().flatten() {
        let Some(finding_type) = finding["type"].as_str().and_then(FindingType::parse) else {
            continue;
        };
        let keys = strings(&finding["keys"]);
        let text_field = match finding_type {
            FindingType::MissingInText => report_key.to_owned(),
            FindingType::TextVsFacts => keys
                .iter()
                .find(|k| texts.contains(k))
                .cloned()
                .unwrap_or_default(),
            _ => String::new(),
        };
        out.push(Candidate {
            finding_type: Some(finding_type),
            keys,
            text_field,
            text: finding["text"].as_str().unwrap_or_default().to_owned(),
        });
    }
    let text_key = |name: &str| -> String {
        match name {
            "condition_report" => report_key.to_owned(),
            other => text_keys(&WriterSettings::default())
                .into_iter()
                .find(|(n, _)| *n == other)
                .map_or_else(|| other.to_owned(), |(_, k)| k),
        }
    };
    for hit in hits {
        let (finding_type, keys, text_field) = match hit.code {
            HitCode::Damage => (
                FindingType::MissingInText,
                hit.key.clone().into_iter().collect(),
                report_key.to_owned(),
            ),
            HitCode::Names | HitCode::Years | HitCode::Models => {
                let key = text_key(&hit.text);
                (FindingType::TextVsFacts, vec![key.clone()], key)
            }
            _ => continue,
        };
        out.push(Candidate {
            finding_type: Some(finding_type),
            keys,
            text_field,
            text: hit.detail.clone(),
        });
    }
    for question in answer["questions"].as_array().into_iter().flatten() {
        out.push(Candidate {
            finding_type: None,
            keys: strings(&question["keys"]),
            text_field: String::new(),
            text: question["text"].as_str().unwrap_or_default().to_owned(),
        });
    }
    out
}

/// Where a run's findings are stored.
struct FindingsTarget {
    item_id: i64,
    session_id: i64,
    run_id: i64,
    now: i64,
}

/// WC6 storage, inside the run's `result` transaction: the filter over the keys' states now,
/// every earlier `open` row of the item `superseded`, one `open` row per survivor.
fn store_findings(
    tx: &mut psm_infra::sqlite::store::SqliteTx<'_>,
    target: &FindingsTarget,
    candidates: &[Candidate],
    layout: &[String],
    written: &[String],
    settings: &WriterSettings,
) -> Result<Filtered> {
    let mut keys: Vec<String> = Vec::new();
    for candidate in candidates {
        for key in candidate.keys.iter().chain([&candidate.text_field]) {
            if !key.is_empty() && !keys.contains(key) {
                keys.push(key.clone());
            }
        }
    }
    let rows = tx.attribute_values(target.item_id)?;
    let item = tx.item(target.item_id)?;
    let mut states = BTreeMap::new();
    for key in keys {
        let row = rows.iter().find(|r| r.key == key);
        let title_base = if key == PRODUCT_TITLE && row.is_none() {
            tx.item_title_base(
                target.item_id,
                psm_core::services::attributes::LINK_INTEGRATION,
            )?
        } else {
            None
        };
        let stored = crate::writer::writes::stored_state(
            row,
            target.item_id,
            &key,
            &item.title,
            title_base.as_deref(),
        );
        let feedback = tx.feedback_by_key(target.item_id, &key)?;
        states.insert(
            key,
            KeyState {
                reviewed: reviewed(stored.as_ref(), &feedback),
                decision: decision(stored.as_ref(), &feedback),
            },
        );
    }
    let conn = tx.conn();
    let resolved: Vec<String> = {
        let mut stmt = conn
            .prepare(
                "SELECT identity FROM writer_findings WHERE item_id=?1 \
                 AND state IN ('answered','dismissed')",
            )
            .map_err(store::db)?;
        stmt.query_map([target.item_id], |r| r.get(0))
            .map_err(store::db)?
            .collect::<rusqlite::Result<Vec<String>>>()
            .map_err(store::db)?
    };
    let filtered = filter(
        candidates,
        &FilterInput {
            modes: &settings.safeguard,
            layout,
            states: &states,
            resolved: &resolved,
            max_questions: settings.max_questions,
            written,
        },
    );
    conn.execute(
        "UPDATE writer_findings SET state='superseded' WHERE item_id=?1 AND state='open'",
        [target.item_id],
    )
    .map_err(store::db)?;
    for survivor in &filtered.stored {
        conn.execute(
            "INSERT INTO writer_findings(item_id,session_id,run_id,kind,type,keys,text_field,\
             identity,text,mode,state,created_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,'open',?11)",
            rusqlite::params![
                target.item_id,
                target.session_id,
                target.run_id,
                survivor.kind,
                survivor.finding_type,
                serde_json::to_string(&survivor.keys)
                    .map_err(|e| psm_core::Error::InvalidData(e.to_string()))?,
                survivor.text_field,
                survivor.identity,
                survivor.text,
                survivor.mode.as_str(),
                target.now,
            ],
        )
        .map_err(store::db)?;
    }
    Ok(filtered)
}

fn log_post_check(run_id: i64, attempt: u32, hits: &[Hit]) {
    tracing::info!(
        target: "psm::writer",
        "writer.post_check run={run_id} attempt={attempt} hits={}",
        hits.len()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(origin: Origin) -> RequestInput {
        RequestInput {
            has_item: true,
            research_busy: false,
            block: None,
            origin,
            auto: true,
            write_states: Vec::new(),
        }
    }

    #[test]
    fn wc3_the_order_of_the_request_rules() {
        use RequestDecision::*;
        assert_eq!(request_decision(&input(Origin::WriteTexts)), Insert);
        let unassigned = RequestInput {
            has_item: false,
            research_busy: true,
            ..input(Origin::WriteTexts)
        };
        assert_eq!(
            request_decision(&unassigned),
            Refused(WriterBlock::NothingToWrite)
        );
        let busy = RequestInput {
            research_busy: true,
            block: Some(WriterBlock::NoModelKey),
            ..input(Origin::WriteTexts)
        };
        assert_eq!(request_decision(&busy), Refused(WriterBlock::ResearchBusy));
        let blocked = RequestInput {
            block: Some(WriterBlock::NoModelKey),
            ..input(Origin::NextToReview)
        };
        assert_eq!(request_decision(&blocked), Refused(WriterBlock::NoModelKey));
        let off = RequestInput {
            auto: false,
            ..input(Origin::NextToReview)
        };
        assert_eq!(request_decision(&off), Nothing);
        assert_eq!(
            request_decision(&RequestInput {
                auto: false,
                ..input(Origin::WriteTexts)
            }),
            Insert
        );
        for state in [
            RunState::Queued,
            RunState::Running,
            RunState::Finished,
            RunState::StoppedAtCap,
        ] {
            let used = RequestInput {
                write_states: vec![state],
                ..input(Origin::AllChecked)
            };
            assert_eq!(request_decision(&used), Nothing, "{state:?}");
        }
        for state in [RunState::Failed, RunState::Cancelled] {
            let retried = RequestInput {
                write_states: vec![state],
                ..input(Origin::AllChecked)
            };
            assert_eq!(request_decision(&retried), Insert, "{state:?}");
        }
        let queued = RequestInput {
            write_states: vec![RunState::Running, RunState::Queued],
            ..input(Origin::WriteTexts)
        };
        assert_eq!(request_decision(&queued), AlreadyQueued);
        let beside_running = RequestInput {
            write_states: vec![RunState::Running],
            ..input(Origin::Answer)
        };
        assert_eq!(request_decision(&beside_running), Insert);
    }
}
