//! `CallGateway::call`: the one door every provider call of a run passes through (MC-A5 order per
//! call, MC-A8 stop rule and cost, MC-A9 record, retries, error classes and redaction).
//!
//! Per attempt: (1) one transaction re-reads each photo's eligibility, applies the stop rule,
//! inserts the call record (`sent`) and one outbound row per photo, and commits; (2) only then the
//! request is sent, with no transaction held; (3) one transaction records the reply, its usage and
//! cost and adds the cost to the run, or the failure and what follows from it.

use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Instant,
};

use psm_core::{Result, ports::clock::Clock};
use psm_infra::sqlite::store::SqliteStore;
use serde_json::{Map, Value as Json, json};

use crate::{
    config::ResearchConfig,
    domain::{CallFailure, CallState, ErrorClass, RunState, Stage},
    files::ResearchFiles,
    images::{ResearchImage, eligible},
    ports::{ModelReply, ModelRequest, PageReply, Providers, SearchReply},
    rules::{
        calls::{CallAction, Redacted, action, classify, redact},
        cost::{NextCall, call_cost, next_call},
    },
    store::{self, CallEnd, NewCall},
};

/// The secret values the redaction scan looks for (research keys and the integration's secrets).
pub type SecretValues = Arc<dyn Fn() -> Vec<String> + Send + Sync>;
/// Waits between attempts (a sleep, or the manual clock in tests).
pub type Waiter = Arc<dyn Fn(u64) + Send + Sync>;

/// What a unit asks a provider.
#[derive(Debug, Clone)]
pub enum CallRequest {
    Model(ModelRequest),
    Search(ResearchImage),
    Read { url: String },
}

/// A provider's answer; a page's images are stored and named by SHA-256.
#[derive(Debug, Clone, PartialEq)]
pub enum Answer {
    Model(ModelReply),
    Search(SearchReply),
    Page {
        reply: Box<PageReply>,
        image_sha256: Vec<String>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum GatewayOutcome {
    Answered(Answer),
    /// A refusal (its text), or a call left with no photo although it needs one.
    NotDetermined(String),
    /// Stop rule: skip the remaining reading units and go to Decide.
    SkipToDecide,
    /// Stop rule: the cap is reached before Decide.
    EndAtCap,
    /// No request byte left: the run is held in `running` (MC-A6 Offline).
    HoldOffline,
    /// The run is now `failed`.
    RunFailed {
        stage: Stage,
        class: ErrorClass,
        text: String,
    },
}

/// The outcome of a call and the photos dropped before it (no longer eligible: "photo removed
/// since").
#[derive(Debug, Clone, PartialEq)]
pub struct Called {
    pub outcome: GatewayOutcome,
    pub dropped: Vec<i64>,
}

/// The text the step detail uses for a photo dropped before a call.
pub const PHOTO_REMOVED: &str = "photo removed since";

pub struct CallGateway {
    store: Arc<SqliteStore>,
    clock: Arc<dyn Clock>,
    providers: Providers,
    secrets: SecretValues,
    files: ResearchFiles,
    wait: Waiter,
    offline: Arc<AtomicBool>,
}

fn redacted_json(value: &Json, secrets: &[&str]) -> Redacted {
    redact(&value.to_string(), secrets)
}

fn image_ref(sha: &str) -> Json {
    json!({ "image_sha256": sha })
}

fn usage_json(usage: Option<&[(String, i64)]>) -> Option<String> {
    usage.map(|u| {
        let map: Map<String, Json> = u.iter().map(|(k, v)| (k.clone(), json!(v))).collect();
        Json::Object(map).to_string()
    })
}

impl CallRequest {
    fn images(&self) -> Vec<ResearchImage> {
        match self {
            CallRequest::Model(m) => m.images.clone(),
            CallRequest::Search(image) => vec![image.clone()],
            CallRequest::Read { .. } => Vec::new(),
        }
    }

    fn keep_images(&mut self, keep: &[i64]) {
        if let CallRequest::Model(m) = self {
            m.images.retain(|i| keep.contains(&i.photo_id()));
        }
    }

    /// The request as recorded: every image replaced by `{"image_sha256": …}`.
    fn record(&self) -> Json {
        match self {
            CallRequest::Model(m) => json!({
                "model": m.model,
                "effort": m.effort,
                "language": m.language,
                "instructions": m.instructions,
                "input": m.input,
                "images": m.images.iter().map(|i| image_ref(i.sha256())).collect::<Vec<_>>(),
                "listing_images": m.listing_images.iter().map(|s| image_ref(s)).collect::<Vec<_>>(),
            }),
            CallRequest::Search(image) => json!({ "image": image_ref(image.sha256()) }),
            CallRequest::Read { url } => json!({ "url": url }),
        }
    }
}

/// The parts of the run a call needs, read in step 1.
struct Plan {
    call_id: i64,
    attempt: i64,
    cost: i64,
}

impl CallGateway {
    pub fn new(
        store: Arc<SqliteStore>,
        clock: Arc<dyn Clock>,
        providers: Providers,
        secrets: SecretValues,
        files: ResearchFiles,
        wait: Waiter,
    ) -> Self {
        Self {
            store,
            clock,
            providers,
            secrets,
            files,
            wait,
            offline: Arc::new(AtomicBool::new(false)),
        }
    }

    /// True after a call failed before any byte was sent, until a call is sent again.
    pub fn offline(&self) -> bool {
        self.offline.load(Ordering::SeqCst)
    }

    pub fn providers(&self) -> &Providers {
        &self.providers
    }

    fn provider_and_model(&self, request: &CallRequest) -> (String, String, String) {
        match request {
            CallRequest::Model(m) => (
                self.providers
                    .model
                    .as_ref()
                    .map(|p| p.id())
                    .unwrap_or_default(),
                m.model.clone(),
                m.effort.clone(),
            ),
            CallRequest::Search(_) => (
                self.providers
                    .search
                    .as_ref()
                    .map(|p| p.id())
                    .unwrap_or_default(),
                String::new(),
                String::new(),
            ),
            CallRequest::Read { .. } => (
                self.providers
                    .reader
                    .as_ref()
                    .map(|p| p.id())
                    .unwrap_or_default(),
                String::new(),
                String::new(),
            ),
        }
    }

    /// One provider call of a unit, with its attempts.
    pub fn call(
        &self,
        run_id: i64,
        stage: Stage,
        unit_key: &str,
        mut request: CallRequest,
    ) -> Result<Called> {
        let secret_values = (self.secrets)();
        let secrets: Vec<&str> = secret_values.iter().map(String::as_str).collect();
        let mut dropped: Vec<i64> = Vec::new();
        let mut hit_logged = false;
        let (provider, model, effort) = self.provider_and_model(&request);
        loop {
            // Step 1: one transaction.
            let needs_photo = !request.images().is_empty();
            let images = request.images();
            let now = self.clock.now_ms();
            let step1 = self.store.research_transaction(|tx| {
                let conn = tx.conn();
                let Some(run) = store::run(conn, run_id)? else {
                    return Ok(Err(GatewayOutcome::NotDetermined("run not found".into())));
                };
                let mut keep = Vec::new();
                for image in &images {
                    if eligible(tx, &run, image.photo_id())? {
                        keep.push(image.photo_id());
                    } else if !dropped.contains(&image.photo_id()) {
                        dropped.push(image.photo_id());
                    }
                }
                let config = ResearchConfig::from_snapshot(run.settings_snapshot.as_deref())?;
                let prices = config.price_table();
                let decide_model = config.stage(Stage::Decide).model;
                let reserve = self
                    .providers
                    .model
                    .as_ref()
                    .and_then(|p| prices.fallback(&p.id(), &decide_model))
                    .unwrap_or(0);
                match next_call(run.cost_micro_usd, config.max_cost_per_run, reserve, stage) {
                    NextCall::Call => {}
                    NextCall::SkipToDecide => return Ok(Err(GatewayOutcome::SkipToDecide)),
                    NextCall::EndAtCap => return Ok(Err(GatewayOutcome::EndAtCap)),
                }
                if needs_photo && keep.is_empty() {
                    return Ok(Err(GatewayOutcome::NotDetermined(PHOTO_REMOVED.into())));
                }
                let mut kept = request.clone();
                kept.keep_images(&keep);
                let recorded = redacted_json(&kept.record(), &secrets);
                let attempt =
                    store::next_attempt(conn, run_id, stage.as_str(), unit_key, &provider)?;
                let call_id = store::insert_call(
                    conn,
                    &NewCall {
                        run_id,
                        stage: stage.as_str(),
                        unit_key,
                        attempt,
                        provider: &provider,
                        model: &model,
                        effort: &effort,
                        request: &recorded.text,
                        started_at: now,
                        redaction_hit: recorded.hit,
                    },
                )?;
                for image in images.iter().filter(|i| keep.contains(&i.photo_id())) {
                    store::insert_outbound(
                        conn,
                        run_id,
                        call_id,
                        image.photo_id(),
                        &provider,
                        now,
                        image.sha256(),
                    )?;
                }
                Ok(Ok((
                    Plan {
                        call_id,
                        attempt,
                        cost: 0,
                    },
                    keep,
                    recorded.hit,
                    config,
                )))
            })?;
            let (mut plan, keep, request_hit, config) = match step1 {
                Ok(planned) => planned,
                Err(outcome) => return Ok(Called { outcome, dropped }),
            };
            if request_hit && !hit_logged {
                hit_logged = true;
                log_redaction(run_id, stage);
            }
            request.keep_images(&keep);
            // Step 2: send, no transaction held.
            let started = Instant::now();
            let mono_start = self.clock.mono();
            let reply = self.send(&request);
            let duration_ms = self
                .clock
                .mono()
                .checked_duration_since(mono_start)
                .map(|d| d.as_millis() as i64)
                .unwrap_or_else(|| started.elapsed().as_millis() as i64);
            let finished_at = self.clock.now_ms();
            let prices = config.price_table();
            // Step 3: record.
            match reply {
                Ok(mut answer) => {
                    self.offline.store(false, Ordering::SeqCst);
                    let (response, usage) = self.response_of(&mut answer)?;
                    let response = redacted_json(&response, &secrets);
                    plan.cost = call_cost(usage.as_deref(), true, &provider, &model, &prices);
                    let usage_text = usage_json(usage.as_deref()).map(|u| redact(&u, &secrets));
                    let hit = response.hit || usage_text.as_ref().is_some_and(|u| u.hit);
                    if hit && !hit_logged {
                        log_redaction(run_id, stage);
                    }
                    self.store.research_transaction(|tx| {
                        store::end_call(
                            tx.conn(),
                            plan.call_id,
                            &CallEnd {
                                state: CallState::Answered.as_str(),
                                response: Some(&response.text),
                                usage: usage_text.as_ref().map(|u| u.text.as_str()),
                                cost_micro_usd: plan.cost,
                                finished_at,
                                duration_ms,
                                error_class: None,
                                error_text: None,
                                redaction_hit: hit,
                            },
                        )?;
                        store::add_run_cost(tx.conn(), run_id, plan.cost)
                    })?;
                    return Ok(Called {
                        outcome: GatewayOutcome::Answered(answer),
                        dropped,
                    });
                }
                Err(failure) => {
                    let class = classify(&failure);
                    let next = action(class, u32::try_from(plan.attempt).unwrap_or(u32::MAX));
                    let text = redact(&failure.text, &secrets);
                    if text.hit && !hit_logged {
                        hit_logged = true;
                        log_redaction(run_id, stage);
                    }
                    tracing::warn!(
                        run_id,
                        stage = stage.as_str(),
                        class = class.as_str(),
                        attempt = plan.attempt,
                        error = %text.text,
                        "research call failed"
                    );
                    let sent = class != ErrorClass::Offline;
                    if sent {
                        self.offline.store(false, Ordering::SeqCst);
                    } else {
                        self.offline.store(true, Ordering::SeqCst);
                    }
                    plan.cost =
                        call_cost(failure.usage.as_deref(), sent, &provider, &model, &prices);
                    let usage_text = usage_json(failure.usage.as_deref());
                    let run_error = match class {
                        ErrorClass::Key => format!("key rejected by {provider}"),
                        _ => format!("{} failed: {}", stage.as_str(), text.text),
                    };
                    let fail_run = next == CallAction::FailRun;
                    self.store.research_transaction(|tx| {
                        let conn = tx.conn();
                        store::end_call(
                            conn,
                            plan.call_id,
                            &CallEnd {
                                state: if sent {
                                    CallState::Failed.as_str()
                                } else {
                                    CallState::NotSent.as_str()
                                },
                                response: None,
                                usage: usage_text.as_deref(),
                                cost_micro_usd: plan.cost,
                                finished_at,
                                duration_ms,
                                error_class: Some(class.as_str()),
                                error_text: Some(&text.text),
                                redaction_hit: text.hit,
                            },
                        )?;
                        if !sent {
                            store::delete_outbound(conn, plan.call_id)?;
                        }
                        store::add_run_cost(conn, run_id, plan.cost)?;
                        if fail_run {
                            store::set_stage_reached(conn, run_id, stage.as_str())?;
                            store::set_run_state(
                                conn,
                                run_id,
                                RunState::Failed,
                                Some(&run_error),
                                Some(finished_at),
                            )?;
                        }
                        Ok(())
                    })?;
                    match next {
                        CallAction::HoldOffline => {
                            return Ok(Called {
                                outcome: GatewayOutcome::HoldOffline,
                                dropped,
                            });
                        }
                        CallAction::NotDetermined => {
                            return Ok(Called {
                                outcome: GatewayOutcome::NotDetermined(text.text),
                                dropped,
                            });
                        }
                        CallAction::FailRun => {
                            return Ok(Called {
                                outcome: GatewayOutcome::RunFailed {
                                    stage,
                                    class,
                                    text: run_error,
                                },
                                dropped,
                            });
                        }
                        CallAction::Retry { wait_ms } => (self.wait)(wait_ms),
                    }
                }
            }
        }
    }

    fn send(&self, request: &CallRequest) -> std::result::Result<Answer, CallFailure> {
        let missing = |port: &str| CallFailure {
            sent: true,
            status: Some(400),
            kind: crate::domain::FailureKind::Status,
            quota: false,
            text: format!("no {port} adapter is configured"),
            usage: None,
        };
        match request {
            CallRequest::Model(m) => match &self.providers.model {
                Some(p) => p.call(m).map(Answer::Model),
                None => Err(missing("model")),
            },
            CallRequest::Search(image) => match &self.providers.search {
                Some(p) => p.search(image).map(Answer::Search),
                None => Err(missing("visual search")),
            },
            CallRequest::Read { url } => match &self.providers.reader {
                Some(p) => p.read(url).map(|reply| Answer::Page {
                    reply: Box::new(reply),
                    image_sha256: Vec::new(),
                }),
                None => Err(missing("page reader")),
            },
        }
    }

    /// The response body to record and the usage; a page's images are stored here.
    #[allow(clippy::type_complexity)]
    fn response_of(&self, answer: &mut Answer) -> Result<(Json, Option<Vec<(String, i64)>>)> {
        Ok(match answer {
            Answer::Model(reply) => (
                json!({ "body": reply.exchange.response }),
                reply.usage.clone(),
            ),
            Answer::Search(reply) => (
                json!({ "body": reply.exchange.response }),
                reply.usage.clone(),
            ),
            Answer::Page {
                reply,
                image_sha256,
            } => {
                let mut shas = Vec::new();
                for bytes in &reply.images {
                    shas.push(self.files.put(bytes)?);
                }
                *image_sha256 = shas.clone();
                (
                    json!({
                        "body": reply.exchange.response,
                        "listing_images": shas.iter().map(|s| image_ref(s)).collect::<Vec<_>>(),
                    }),
                    reply.usage.clone(),
                )
            }
        })
    }

    /// Resume (MC-A7): every call of the run still `sent` with no response (left by a kill) is
    /// closed as `unknown` and charged its fallback (MC-A8); the unit re-issues it as the next
    /// attempt.
    pub fn close_open_calls(&self, run_id: i64) -> Result<usize> {
        let now = self.clock.now_ms();
        self.store.research_transaction(|tx| {
            let conn = tx.conn();
            let Some(run) = store::run(conn, run_id)? else {
                return Ok(0);
            };
            let prices =
                ResearchConfig::from_snapshot(run.settings_snapshot.as_deref())?.price_table();
            let open = store::open_calls(conn, run_id)?;
            for call in &open {
                let cost = call_cost(None, true, &call.provider, &call.model, &prices);
                store::end_call(
                    conn,
                    call.id,
                    &CallEnd {
                        state: CallState::Unknown.as_str(),
                        response: None,
                        usage: None,
                        cost_micro_usd: cost,
                        finished_at: now,
                        duration_ms: 0,
                        error_class: None,
                        error_text: Some("no response was recorded before the app stopped"),
                        redaction_hit: false,
                    },
                )?;
                store::add_run_cost(conn, run_id, cost)?;
            }
            Ok(open.len())
        })
    }

    /// Every value the secrets port holds, as the redaction scan sees them (MC-A9); the export
    /// check reads the same list (MC-A11).
    pub fn secret_values(&self) -> Vec<String> {
        (self.secrets)()
    }

    /// The SHA-256s of the listing images a page call stored (from its record).
    pub fn page_images(response: &str) -> Vec<String> {
        serde_json::from_str::<Json>(response)
            .ok()
            .and_then(|r| r.get("listing_images").cloned())
            .and_then(|l| l.as_array().cloned())
            .unwrap_or_default()
            .iter()
            .filter_map(|i| {
                i.get("image_sha256")
                    .and_then(Json::as_str)
                    .map(str::to_owned)
            })
            .collect()
    }
}

fn log_redaction(run_id: i64, stage: Stage) {
    tracing::error!(
        run_id,
        stage = stage.as_str(),
        "a secret value was found in a research call record and redacted"
    );
}
