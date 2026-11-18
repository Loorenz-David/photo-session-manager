//! Owner correction A12 (2026-11-18): the research parts of the "Search this photo" window.
//!
//! - [`lens_image`]: the picture the window gives Google Lens (the photo as the app frames it,
//!   reduced and stripped of metadata; no provider call, no run).
//! - [`ResearchService::browse_matches`] / [`ResearchService::browse_search`]: the window's
//!   fallback list when Google's own page cannot be used: the visual matches research already
//!   paid for, or one new visual search of the pinned photo.
//! - [`ResearchService::browse_ask`]: "Ask AI…" on text the owner selected on a page. The answer
//!   becomes AI suggestions (never applied) with the page as their source.
//!
//! Every provider call goes through the gateway on a `browse` run of the session (MC-A9 record,
//! MC-A8 cost and the day limit), so it is in the export and counted in the day's spend, but it
//! is not one of the session's research runs (`store::runs` leaves browse runs out).

use std::path::Path;

use psm_core::{
    Error, Result,
    domain::{
        attribute::{AiConfidence, CoreType},
        photo::Classification,
    },
    ports::{images::Imaging, store::StoreTx},
};
use psm_infra::sqlite::store::SqliteStore;
use serde::{Deserialize, Serialize};
use serde_json::{Value as Json, json};

use crate::{
    config::ResearchConfig,
    domain::{RunState, RunTrigger, Stage, StepKind},
    files::ResearchFiles,
    gateway::{Answer, CallRequest, GatewayOutcome},
    ports::ModelRequest,
    rules::{sites::host, value_map::Stated, write_table::never_written},
    service::{MODEL_ACCOUNT, ResearchService, SEARCH_ACCOUNT, form_snapshot},
    shop::ShopFacts,
    stages::{descriptor, display, statement_of},
    store::{self, NewRun, NewStep},
    writes::{self, WriteOutcome},
};

/// The unit key of every browse call.
pub const BROWSE_UNIT: &str = "browse";
/// The longest selected text "Ask AI…" sends (the window cuts what it reports to this).
pub const MAX_SELECTION_CHARS: usize = 2_000;
/// The longest instruction the owner can give "Ask AI…".
pub const MAX_INSTRUCTION_CHARS: usize = 300;
/// The long edge of the picture given to Google Lens.
pub const LENS_LONG_EDGE: u32 = 1_600;
/// An original camera preview larger than this is not sent as it is.
const MAX_LENS_BYTES: u64 = 8 * 1024 * 1024;

/// The instructions of an "Ask AI…" call (ours; the owner's instruction is part of the input).
pub const ASK: &str = "The owner is identifying the piece in their photos and selected this text \
    on a web page. Follow the owner's instruction. For each listed field, quote what the selected \
    text states for it and give the value in the field's terms, or null when the text names \
    nothing the field allows. Use only the selected text; do not guess.";

/// One visual match as the window lists it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VisualMatch {
    pub url: String,
    pub title: String,
    /// The site's own name, when the search gave one.
    pub source: String,
    /// A small picture of the match, when the search gave one (a remote address).
    pub thumbnail: Option<String>,
}

/// The fallback list: where it comes from and its matches.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrowseMatches {
    /// `research` (an earlier research run's search, no new cost) or `search` (a new search of
    /// the pinned photo).
    pub origin: String,
    pub matches: Vec<VisualMatch>,
}

/// "Ask AI…": the selected text, the page it is on and the owner's instruction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AskRequest {
    pub session_id: i64,
    pub page_url: String,
    pub page_title: String,
    pub text: String,
    pub instruction: String,
}

/// One field "Ask AI…" suggested.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AskSuggestion {
    pub key: String,
    pub label: String,
    /// The value as the form shows it.
    pub value: String,
}

/// What "Ask AI…" did.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AskAnswer {
    pub suggested: Vec<AskSuggestion>,
    /// Fields the answer named but that were not written, with why (e.g. "already holds this
    /// value").
    pub not_written: Vec<(String, String)>,
    pub cost_micro_usd: i64,
}

/// The visual-match list of a recorded SerpApi Lens response (`{"body": {"search": {"body"}}}`
/// as the gateway records it, or the search body itself).
pub fn recorded_matches(response: &Json) -> Vec<VisualMatch> {
    let body = [
        &response["body"]["search"]["body"],
        &response["search"]["body"],
        response,
    ]
    .into_iter()
    .find(|b| b.get("visual_matches").is_some())
    .cloned()
    .unwrap_or(Json::Null);
    crate::adapters::visual_search::serpapi_lens::display_matches(&body)
}

fn invalid(text: impl Into<String>) -> Error {
    Error::InvalidValue(text.into())
}

/// The picture for Google Lens: the photo's adjusted preview framed as the app frames it,
/// reduced to [`LENS_LONG_EDGE`] and stripped of metadata; without an adjusted preview, the
/// camera's original preview as it is. The photo must be a visible photo of the session.
pub fn lens_image(
    store: &SqliteStore,
    imaging: &dyn Imaging,
    data_dir: &Path,
    session_id: i64,
    photo_id: i64,
) -> Result<Vec<u8>> {
    let found = store.research_transaction(|tx| {
        let Ok(photo) = tx.photo(photo_id) else {
            return Ok(None);
        };
        if photo.session_id != session_id || photo.classification == Classification::Deleted {
            return Ok(None);
        }
        let source = crate::images::source(tx, photo_id)?;
        let composition = tx.composition(photo_id)?;
        Ok(Some((photo, source, composition)))
    })?;
    let Some((photo, source, composition)) = found else {
        return Err(invalid("this photo is not one of the session's photos"));
    };
    if let Some((preview, _)) = source {
        let files = ResearchFiles::new(data_dir);
        let temp = files.temp_path()?;
        let rendered = imaging.render_research(
            &preview,
            &crate::images::values_of(&composition),
            LENS_LONG_EDGE,
            &temp,
        );
        let bytes = rendered.and_then(|_| Ok(std::fs::read(&temp)?));
        let _ = std::fs::remove_file(&temp);
        return bytes;
    }
    if let Some(original) = photo.original_preview_path.as_deref() {
        let path = Path::new(original);
        if let Ok(meta) = std::fs::metadata(path)
            && meta.is_file()
            && meta.len() <= MAX_LENS_BYTES
        {
            return Ok(std::fs::read(path)?);
        }
    }
    Err(invalid("this photo has no preview yet"))
}

/// A field "Ask AI…" may answer: in the layout, one research writes, of a kind research fills.
fn askable(core_type: CoreType) -> bool {
    !matches!(
        core_type,
        CoreType::Tags
            | CoreType::RichText
            | CoreType::Money
            | CoreType::Rating
            | CoreType::MediaList
            | CoreType::Reference
    )
}

fn checked_page_url(url: &str) -> Result<String> {
    let url = url.trim();
    if host(url).is_none() {
        return Err(invalid("the page is not an http or https address"));
    }
    Ok(url.to_owned())
}

impl ResearchService {
    fn session_item_of(&self, session_id: i64) -> Result<i64> {
        self.store
            .research_transaction(|tx| Ok(tx.session(session_id)?.item_id))?
            .ok_or_else(|| invalid("the Unassigned session has no item"))
    }

    /// A finished `browse` run holding the next call; the settings as a run would snapshot them.
    fn browse_run(&self, session_id: i64, item_id: i64, form: Option<&Json>) -> Result<i64> {
        let config = ResearchConfig::from_settings(&self.settings(), &ShopFacts::default());
        let config = serde_json::to_string(&config).map_err(|e| invalid(e.to_string()))?;
        let form = form.map(Json::to_string);
        let now = self.clock.now_ms();
        self.store.research_transaction(|tx| {
            store::insert_run(
                tx.conn(),
                &NewRun {
                    item_id,
                    session_id,
                    trigger: RunTrigger::Browse,
                    state: RunState::Finished,
                    queued_at: now,
                    started_at: Some(now),
                    settings_snapshot: Some(config),
                    form_snapshot: form,
                    input_snapshot: None,
                },
            )
        })
    }

    /// Ends a browse run: its steps, its end time, and `failed` with `error` when the call failed.
    fn end_browse_run(&self, run_id: i64, steps: &[NewStep], error: Option<&str>) -> Result<()> {
        let now = self.clock.now_ms();
        self.store.research_transaction(|tx| {
            let conn = tx.conn();
            for step in steps {
                store::insert_step(conn, run_id, step, now)?;
            }
            let state = if error.is_some() {
                RunState::Failed
            } else {
                RunState::Finished
            };
            store::set_run_state(conn, run_id, state, error, Some(now))
        })
    }

    fn run_cost(&self, run_id: i64) -> Result<i64> {
        Ok(self
            .store
            .research_transaction(|tx| store::run(tx.conn(), run_id))?
            .map(|r| r.cost_micro_usd)
            .unwrap_or(0))
    }

    /// The day limit and the key of a port, checked before a browse call.
    fn browse_may_call(&self, account: &str, has_port: bool, what: &str) -> Result<()> {
        if !has_port || !self.key_set(account) {
            return Err(invalid(format!(
                "Add the {what} key in Settings → Research"
            )));
        }
        if self.day_limit_reached(&self.settings())? {
            return Err(invalid("Day limit reached"));
        }
        Ok(())
    }

    /// The fallback list from research already done: the matches of the session's latest
    /// answered visual search (no new call, no cost). Empty when there is none.
    pub fn browse_matches(&self, session_id: i64) -> Result<Vec<VisualMatch>> {
        let response: Option<String> = self.store.research_transaction(|tx| {
            use rusqlite::OptionalExtension;
            tx.conn()
                .query_row(
                    "SELECT c.response FROM research_calls c JOIN research_runs r ON r.id=c.run_id \
                     WHERE r.session_id=?1 AND c.stage='find' AND c.state='answered' \
                     AND c.response IS NOT NULL ORDER BY c.id DESC LIMIT 1",
                    [session_id],
                    |r| r.get(0),
                )
                .optional()
                .map_err(store::db)
        })?;
        Ok(response
            .and_then(|text| serde_json::from_str::<Json>(&text).ok())
            .map(|json| recorded_matches(&json))
            .unwrap_or_default())
    }

    /// One new visual search of `photo_id` (the window's fallback when Google cannot be used and
    /// research has no matches yet): its matches. Counted like any research call.
    pub fn browse_search(&self, session_id: i64, photo_id: i64) -> Result<Vec<VisualMatch>> {
        let item_id = self.session_item_of(session_id)?;
        self.browse_may_call(
            SEARCH_ACCOUNT,
            self.providers.search.is_some(),
            "visual search",
        )?;
        let run_id = self.browse_run(session_id, item_id, None)?;
        let edge = self.settings().stage(Stage::Find).image_long_edge;
        let image = match self.images.render(photo_id, run_id, edge.max(512)) {
            Ok(image) => image,
            Err(why) => {
                let text = why.to_string();
                self.end_browse_run(run_id, &[], Some(&text))?;
                return Err(invalid(text));
            }
        };
        let called =
            self.gateway
                .call(run_id, Stage::Find, BROWSE_UNIT, CallRequest::Search(image))?;
        let mut searched = crate::stages::step(
            Stage::Find,
            StepKind::Searched,
            "Searched for the pinned photo",
        );
        searched.photo_ids = vec![photo_id];
        let outcome = match called.outcome {
            GatewayOutcome::Answered(Answer::Search(reply)) => {
                let mut matches = recorded_matches(&reply.exchange.response);
                if matches.is_empty() {
                    matches = reply
                        .candidates
                        .iter()
                        .map(|c| VisualMatch {
                            url: c.url.clone(),
                            title: c.title.clone(),
                            source: host(&c.url).unwrap_or_default(),
                            thumbnail: None,
                        })
                        .collect();
                }
                searched.summary = format!("Found {} candidate pages", matches.len());
                Ok(matches)
            }
            other => Err(outcome_text(&other)),
        };
        let error = outcome.as_ref().err().cloned();
        self.end_browse_run(run_id, &[searched], error.as_deref())?;
        tracing::info!(
            target: "psm::search",
            "search_window.serpapi_search session={session_id} photo={photo_id} result={} cost_micro_usd={}",
            match &outcome {
                Ok(m) => format!("ok matches={}", m.len()),
                Err(_) => "failed".to_owned(),
            },
            self.run_cost(run_id)?
        );
        outcome.map_err(invalid)
    }

    /// "Ask AI…": one model call on the selected text, the answer written as AI suggestions
    /// (MC-A1's write table; medium confidence; the page as the source). Nothing is applied.
    pub fn browse_ask(&self, request: &AskRequest) -> Result<AskAnswer> {
        let text = request.text.trim();
        if text.is_empty() {
            return Err(invalid("select some text on the page first"));
        }
        if text.chars().count() > MAX_SELECTION_CHARS {
            return Err(invalid(format!(
                "the selected text is longer than {MAX_SELECTION_CHARS} characters"
            )));
        }
        let instruction = request.instruction.trim();
        if instruction.chars().count() > MAX_INSTRUCTION_CHARS {
            return Err(invalid(format!(
                "the instruction is longer than {MAX_INSTRUCTION_CHARS} characters"
            )));
        }
        let page_url = checked_page_url(&request.page_url)?;
        let item_id = self.session_item_of(request.session_id)?;
        self.browse_may_call(MODEL_ACCOUNT, self.providers.model.is_some(), "model")?;
        let form = form_snapshot(
            &self.store,
            self.clock.as_ref(),
            self.integration.as_deref(),
            item_id,
        )?;
        let fields: Vec<_> = form
            .fields
            .iter()
            .filter(|f| {
                !never_written(&f.definition, Some(f.step)) && askable(f.definition.core_type)
            })
            .map(|f| f.definition.clone())
            .collect();
        if fields.is_empty() {
            return Err(invalid("this item type has no fields AI can fill"));
        }
        let form_json = serde_json::to_value(&form).map_err(|e| invalid(e.to_string()))?;
        let run_id = self.browse_run(request.session_id, item_id, Some(&form_json))?;
        let settings = self.settings();
        let stage = settings.stage(Stage::Read);
        let instruction = if instruction.is_empty() {
            "Fill whatever fields the text states."
        } else {
            instruction
        };
        let call = CallRequest::Model(ModelRequest {
            stage: Stage::Read,
            unit_key: BROWSE_UNIT.to_owned(),
            model: stage.model,
            effort: stage.effort,
            language: settings.language.clone(),
            instructions: ASK.to_owned(),
            input: json!({
                "fields": fields.iter().map(descriptor).collect::<Vec<_>>(),
                "page": { "url": page_url, "title": request.page_title },
                "selected_text": text,
                "instruction": instruction,
                "language": settings.language,
            }),
            images: Vec::new(),
            listing_images: Vec::new(),
        });
        let called = self.gateway.call(run_id, Stage::Read, BROWSE_UNIT, call)?;
        let host_name = host(&page_url).unwrap_or_default();
        let mut read = crate::stages::step(
            Stage::Read,
            StepKind::Read,
            format!("Read the text you selected on {host_name}"),
        );
        read.source_url = Some(page_url.clone());
        read.source_title = Some(request.page_title.clone()).filter(|t| !t.is_empty());
        read.detail = format!("Instruction: {instruction}");
        let output = match called.outcome {
            GatewayOutcome::Answered(Answer::Model(reply)) => reply.output,
            other => {
                let why = outcome_text(&other);
                self.end_browse_run(run_id, &[read], Some(&why))?;
                self.log_ask(request.session_id, run_id, "failed", 0)?;
                return Err(invalid(why));
            }
        };
        let mut steps = vec![read];
        let mut answer = AskAnswer {
            suggested: Vec::new(),
            not_written: Vec::new(),
            cost_micro_usd: 0,
        };
        for definition in &fields {
            let Some(entry) = output.get("keys").and_then(|k| k.get(&definition.key)) else {
                continue;
            };
            let Some(Stated::Mapped(value)) = statement_of(definition, entry) else {
                continue;
            };
            let quote: String = entry
                .get("stated")
                .and_then(Json::as_str)
                .unwrap_or("")
                .chars()
                .take(200)
                .collect();
            let reason = if quote.trim().is_empty() {
                format!("From the text you selected on {host_name}.")
            } else {
                format!(
                    "From the text you selected on {host_name}: “{}”",
                    quote.trim()
                )
            };
            let outcome = writes::suggest(
                self.store.as_ref(),
                self.clock.as_ref(),
                self.integration.as_deref(),
                item_id,
                &definition.key,
                &value,
                AiConfidence::Medium,
                Some(&reason),
                std::slice::from_ref(&page_url),
            )?;
            match outcome {
                WriteOutcome::Written(_) => {
                    let mut concluded = crate::stages::step(
                        Stage::Decide,
                        StepKind::Concluded,
                        format!("Suggested {}: {}", definition.label, display(&value)),
                    );
                    concluded.field_keys = vec![definition.key.clone()];
                    concluded.source_url = Some(page_url.clone());
                    steps.push(concluded);
                    answer.suggested.push(AskSuggestion {
                        key: definition.key.clone(),
                        label: definition.label.clone(),
                        value: display(&value),
                    });
                }
                other => answer
                    .not_written
                    .push((definition.label.clone(), other.text())),
            }
        }
        self.end_browse_run(run_id, &steps, None)?;
        answer.cost_micro_usd = self.run_cost(run_id)?;
        self.log_ask(
            request.session_id,
            run_id,
            &format!("ok suggested={}", answer.suggested.len()),
            answer.cost_micro_usd,
        )?;
        if !answer.suggested.is_empty() {
            self.changed(request.session_id);
        }
        Ok(answer)
    }

    fn log_ask(&self, session_id: i64, run_id: i64, result: &str, cost: i64) -> Result<()> {
        let cost = if cost == 0 {
            self.run_cost(run_id)?
        } else {
            cost
        };
        tracing::info!(
            target: "psm::search",
            "search_window.ask_ai session={session_id} run={run_id} result={result} cost_micro_usd={cost}"
        );
        Ok(())
    }
}

/// The owner-facing sentence of a browse call that gave no answer.
fn outcome_text(outcome: &GatewayOutcome) -> String {
    match outcome {
        GatewayOutcome::Answered(_) => "the answer was not of the expected kind".to_owned(),
        GatewayOutcome::NotDetermined(text) => text.clone(),
        GatewayOutcome::SkipToDecide | GatewayOutcome::EndAtCap => {
            "the cost limit per run is reached".to_owned()
        }
        GatewayOutcome::HoldOffline => "Offline — try again when the Mac is online".to_owned(),
        GatewayOutcome::RunFailed { text, .. } => text.clone(),
    }
}
