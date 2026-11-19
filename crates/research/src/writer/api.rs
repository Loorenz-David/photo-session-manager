//! The writer's app-facing surface (`master_plan_writer.md` §4 "API"). The wiring hands the same
//! `ResearchService` as `Arc<dyn WriterApi>` beside `Arc<dyn ResearchApi>`.

use psm_core::{Result, ports::store::StoreTx};
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;

use crate::{
    api::{Reason, RunView, StepView},
    domain::{RunState, RunTrigger, Stage},
    service::ResearchService,
    store,
    writer::{
        config::WriterSettings,
        defaults,
        note::NoteView,
        run::{FindingCount, RESULT_UNIT, WriterBlock},
    },
};

/// The answer to a request that may queue a run: whether it did, and why not.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Requested {
    pub queued: bool,
    pub reason: Option<Reason>,
}

/// One post-check hit as the panel shows it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HitView {
    /// The text's output name (`title`, `description`, …).
    pub text: String,
    /// A `HitCode` in snake case.
    pub code: String,
    pub detail: String,
}

/// One text the latest writer run produced.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextView {
    pub key: String,
    pub label: String,
    /// The text still had a post-check hit when it was written (it arrived `low`).
    pub flagged: bool,
    /// The hits left after the run (the "Check:" line).
    pub hits: Vec<HitView>,
    pub reason: String,
    /// The labels of the facts the writer says it used.
    pub facts: Vec<String>,
    /// What happened to the text in the form (`written`, `matches a text you dismissed`, …).
    pub outcome: String,
}

/// The post-check of one call of the run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttemptView {
    pub attempt: u32,
    pub hits: Vec<HitView>,
}

/// An open writer finding or question of the item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FindingView {
    pub id: i64,
    /// `finding` or `question`.
    pub kind: String,
    /// A `FindingType` name, or `question`.
    #[serde(rename = "type")]
    pub finding_type: String,
    pub keys: Vec<String>,
    pub text_field: String,
    pub text: String,
    /// `note` or `ask`.
    pub mode: String,
}

/// What the latest writer run received (WC2).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputsView {
    pub fields: usize,
    pub note: usize,
    pub listings: Vec<String>,
}

/// The writer's state for one session (08C §11; W3 `status`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriterStatus {
    /// Why "Write texts" would queue nothing now, if it would not.
    pub reason: Option<Reason>,
    /// The latest `write` run of the session.
    pub run: Option<RunView>,
    pub texts: Vec<TextView>,
    pub attempts: Vec<AttemptView>,
    /// The texts the run rewrote once after a hit.
    pub regenerated: Vec<String>,
    /// The item's open findings and questions.
    pub findings: Vec<FindingView>,
    pub steps: Vec<StepView>,
    pub cost_micro_usd: i64,
    pub inputs: InputsView,
    pub stale_main: bool,
    pub stale_condition: bool,
    pub can_write: bool,
}

pub trait WriterApi: Send + Sync {
    /// W1: the session's product note as the owner's decisions show it, and its staleness.
    fn note(&self, session_id: i64) -> Result<NoteView>;
    /// W1 (WC1.8): **Look again**: queue a `look` run when the photos changed.
    fn look_again(&self, session_id: i64) -> Result<Requested>;
    /// W2 (WC3): queue a writer run of the session, or say why not.
    fn request(&self, session_id: i64, origin: crate::writer::run::Origin) -> Result<Requested>;
    /// W2 (WC6): answer an open writer question or finding; the texts are written again.
    fn answer(&self, finding_id: i64, answer: &str) -> Result<Requested>;
    /// W2 (WC6): dismiss an open writer question or finding.
    fn dismiss(&self, finding_id: i64) -> Result<()>;
    /// W3 (08C §11): the latest writer run of the session, its texts, checks, findings and cost.
    fn status(&self, session_id: i64) -> Result<WriterStatus>;
    /// W4: refuses a `writer` settings value the writer cannot run with; the text names the field.
    fn settings_validate(&self, settings: &Json) -> Result<()> {
        WriterSettings::validate(settings).map_err(psm_core::Error::InvalidData)
    }
    /// W4: the shipped instruction of `language` (§9).
    fn instruction_default(&self, language: &str) -> String {
        defaults::instruction(language).to_owned()
    }
    /// W4 (08C §8 "Preview what is sent"): the exact request a write would send under the
    /// settings value for `language`, for the built-in example item; no provider call, no run.
    fn preview(&self, settings: &Json, language: &str) -> Result<Preview>;
    /// W4 (WC6 "Counts"): per finding type, how often it was answered and dismissed.
    fn counts(&self) -> Result<Vec<FindingCount>>;
}

/// A writer request as it is sent (W4 `preview`; built by `writer::run::request_of`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Preview {
    pub instructions: String,
    pub input: Json,
    pub schema: Json,
}

/// The text names in the order the writer writes them (`master_plan_writer.md` §4 Text keys).
const TEXT_ORDER: [&str; 5] = [
    "title",
    "description",
    "condition_report",
    "seo_title",
    "seo_description",
];

fn hits_of(value: &Json) -> Vec<HitView> {
    value
        .as_array()
        .into_iter()
        .flatten()
        .map(|h| HitView {
            text: h["text"].as_str().unwrap_or_default().to_owned(),
            code: h["code"].as_str().unwrap_or_default().to_owned(),
            detail: h["detail"].as_str().unwrap_or_default().to_owned(),
        })
        .collect()
}

fn strings(value: &Json) -> Vec<String> {
    value
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|v| v.as_str().map(str::to_owned))
        .collect()
}

impl ResearchService {
    /// W3 `WriterApi::status`: read from the latest `write` run of the session (its input
    /// snapshot, its `result` unit, its steps and cost), the item's open findings, the note's
    /// staleness and the prerequisites a "Write texts" press would meet now.
    pub fn writer_status(&self, session_id: i64) -> Result<WriterStatus> {
        let item = self.session_item(session_id)?;
        let runs = self
            .store
            .research_transaction(|tx| store::all_runs(tx.conn(), Some(session_id)))?;
        let research_busy = runs.iter().any(|r| {
            !matches!(r.trigger, RunTrigger::Browse | RunTrigger::Write)
                && matches!(r.state, RunState::Queued | RunState::Running)
        });
        let block = match item {
            None => Some(WriterBlock::NothingToWrite),
            Some(_) if research_busy => Some(WriterBlock::ResearchBusy),
            Some(item_id) => {
                self.writer_block(item_id, &self.settings(), &self.writer_settings())?
            }
        };
        let latest = runs.into_iter().rfind(|r| r.trigger == RunTrigger::Write);
        let (stale_main, stale_condition) = match item {
            Some(_) => {
                let note = self.assemble(session_id)?;
                (note.stale_main, note.stale_condition)
            }
            None => (false, false),
        };
        let mut status = WriterStatus {
            can_write: block.is_none(),
            reason: block.map(|b| b.reason()),
            run: None,
            texts: Vec::new(),
            attempts: Vec::new(),
            regenerated: Vec::new(),
            findings: Vec::new(),
            steps: Vec::new(),
            cost_micro_usd: 0,
            inputs: InputsView::default(),
            stale_main,
            stale_condition,
        };
        if let Some(item_id) = item {
            status.findings = self.open_findings(item_id)?;
        }
        let Some(run) = latest else {
            return Ok(status);
        };
        let input: Json = run
            .input_snapshot
            .as_deref()
            .and_then(|t| serde_json::from_str(t).ok())
            .unwrap_or(Json::Null);
        status.inputs = InputsView {
            fields: input["fields"].as_array().map_or(0, Vec::len),
            note: input["note"].as_array().map_or(0, Vec::len),
            listings: input["listings"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|l| l["title"].as_str().map(str::to_owned))
                .collect(),
        };
        let (result, steps) = self.store.research_transaction(|tx| {
            let conn = tx.conn();
            Ok((
                store::output(conn, run.id, Stage::Write.as_str(), RESULT_UNIT)?,
                store::steps(conn, run.id)?,
            ))
        })?;
        let result: Json = result
            .and_then(|row| serde_json::from_str(&row.output).ok())
            .unwrap_or(Json::Null);
        let labels = self.labels()?;
        let label = |key: &str| {
            labels
                .iter()
                .find(|(k, _)| k == key)
                .map_or_else(|| key.to_owned(), |(_, l)| l.clone())
        };
        let writes = result["writes"].as_array().cloned().unwrap_or_default();
        for name in TEXT_ORDER {
            let entry = &result["texts"][name];
            if !entry.is_object() {
                continue;
            }
            let key = entry["key"].as_str().unwrap_or(name).to_owned();
            let hits = hits_of(&entry["hits"]);
            let reason_text = entry["reason"].as_str().unwrap_or_default();
            status.texts.push(TextView {
                label: label(&key),
                flagged: !hits.is_empty(),
                hits,
                reason: reason_text.lines().next().unwrap_or_default().to_owned(),
                facts: strings(&entry["facts"]).iter().map(|f| label(f)).collect(),
                outcome: writes
                    .iter()
                    .find(|w| w["key"].as_str() == Some(key.as_str()))
                    .and_then(|w| w["outcome"].as_str())
                    .unwrap_or_default()
                    .to_owned(),
                key,
            });
        }
        status.attempts = result["hits"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|a| AttemptView {
                attempt: a["attempt"]
                    .as_u64()
                    .and_then(|n| u32::try_from(n).ok())
                    .unwrap_or(0),
                hits: hits_of(&a["hits"]),
            })
            .collect();
        status.regenerated = strings(&result["regenerated"]);
        status.steps = steps
            .into_iter()
            .map(|s| StepView {
                unread: false,
                seq: s.seq,
                stage: s.stage,
                kind: s.kind,
                summary: s.summary,
                detail: s.detail,
                source_url: s.source_url,
                source_title: s.source_title,
                photo_ids: s.photo_ids,
                field_keys: s.field_keys,
            })
            .collect();
        status.cost_micro_usd = run.cost_micro_usd;
        status.run = Some(RunView {
            id: run.id,
            state: run.state.as_str().to_owned(),
            trigger: run.trigger.as_str().to_owned(),
            stage_reached: run.stage_reached,
            error: run.error,
        });
        Ok(status)
    }

    /// The item's `open` writer findings and questions, oldest first.
    fn open_findings(&self, item_id: i64) -> Result<Vec<FindingView>> {
        self.store.research_transaction(|tx| {
            let mut stmt = tx
                .conn()
                .prepare(
                    "SELECT id, kind, type, keys, text_field, text, mode FROM writer_findings \
                     WHERE item_id=?1 AND state='open' ORDER BY id",
                )
                .map_err(store::db)?;
            stmt.query_map([item_id], |r| {
                let keys: String = r.get(3)?;
                Ok(FindingView {
                    id: r.get(0)?,
                    kind: r.get(1)?,
                    finding_type: r.get(2)?,
                    keys: serde_json::from_str(&keys).unwrap_or_default(),
                    text_field: r.get(4)?,
                    text: r.get(5)?,
                    mode: r.get(6)?,
                })
            })
            .map_err(store::db)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(store::db)
        })
    }

    /// Every active definition's `(key, label)`: the built-in fields and the integration's.
    fn labels(&self) -> Result<Vec<(String, String)>> {
        let integration = self.integration.clone();
        self.store.research_transaction(|tx| {
            let mut all = psm_core::services::attributes::builtin_definitions();
            if let Some(integration) = &integration {
                all.extend(tx.attribute_definitions(integration)?);
            }
            Ok(all.into_iter().map(|d| (d.key, d.label)).collect())
        })
    }
}
