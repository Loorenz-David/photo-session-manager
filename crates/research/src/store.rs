//! The engine's SQL (`master_plan_research.md` §6.4). Every function takes the connection of a
//! `SqliteStore::research_transaction`, so engine rows commit together with app rows.

use psm_core::{Error, Result};
use rusqlite::{Connection, OptionalExtension, params};

use crate::domain::{RunState, RunTrigger};

pub(crate) fn db(error: rusqlite::Error) -> Error {
    Error::InvalidData(error.to_string())
}

/// The engine's five tables and the two partial unique indexes (idempotent).
pub const TABLES: &str = "
CREATE TABLE IF NOT EXISTS research_runs (
    id INTEGER PRIMARY KEY,
    item_id INTEGER NOT NULL,
    session_id INTEGER NOT NULL,
    trigger TEXT NOT NULL,
    state TEXT NOT NULL,
    stage_reached TEXT,
    queued_at INTEGER NOT NULL,
    started_at INTEGER,
    finished_at INTEGER,
    cost_micro_usd INTEGER NOT NULL DEFAULT 0,
    settings_snapshot TEXT,
    form_snapshot TEXT,
    input_snapshot TEXT,
    error TEXT
);
CREATE UNIQUE INDEX IF NOT EXISTS research_runs_one_running
    ON research_runs(session_id) WHERE state='running';
CREATE UNIQUE INDEX IF NOT EXISTS research_runs_one_queued
    ON research_runs(session_id) WHERE state='queued';
CREATE TABLE IF NOT EXISTS research_steps (
    id INTEGER PRIMARY KEY,
    run_id INTEGER NOT NULL,
    seq INTEGER NOT NULL,
    stage TEXT NOT NULL,
    kind TEXT NOT NULL,
    summary TEXT NOT NULL,
    detail TEXT NOT NULL DEFAULT '',
    source_url TEXT,
    source_title TEXT,
    photo_ids TEXT NOT NULL,
    field_keys TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    UNIQUE(run_id, seq)
);
CREATE TABLE IF NOT EXISTS research_calls (
    id INTEGER PRIMARY KEY,
    run_id INTEGER NOT NULL,
    stage TEXT NOT NULL,
    unit_key TEXT NOT NULL,
    attempt INTEGER NOT NULL,
    provider TEXT NOT NULL,
    model TEXT NOT NULL,
    effort TEXT NOT NULL,
    state TEXT NOT NULL,
    request TEXT NOT NULL,
    response TEXT,
    usage TEXT,
    cost_micro_usd INTEGER NOT NULL DEFAULT 0,
    started_at INTEGER NOT NULL,
    finished_at INTEGER,
    duration_ms INTEGER,
    error_class TEXT,
    error_text TEXT,
    redaction_hit INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS research_calls_run ON research_calls(run_id, stage, unit_key);
CREATE TABLE IF NOT EXISTS research_outbound (
    id INTEGER PRIMARY KEY,
    run_id INTEGER NOT NULL,
    call_id INTEGER NOT NULL,
    photo_id INTEGER NOT NULL,
    provider TEXT NOT NULL,
    sent_at INTEGER NOT NULL,
    sha256 TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS research_stage_outputs (
    run_id INTEGER NOT NULL,
    stage TEXT NOT NULL,
    unit_key TEXT NOT NULL,
    fingerprint TEXT NOT NULL,
    output TEXT NOT NULL,
    reused_from_run_id INTEGER,
    committed_at INTEGER NOT NULL,
    UNIQUE(run_id, stage, unit_key)
);
";

pub fn create_tables(conn: &Connection) -> Result<()> {
    conn.execute_batch(TABLES).map_err(db)
}

/// One `research_runs` row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunRow {
    pub id: i64,
    pub item_id: i64,
    pub session_id: i64,
    pub trigger: RunTrigger,
    pub state: RunState,
    pub stage_reached: Option<String>,
    pub queued_at: i64,
    pub started_at: Option<i64>,
    pub finished_at: Option<i64>,
    pub cost_micro_usd: i64,
    pub settings_snapshot: Option<String>,
    pub form_snapshot: Option<String>,
    pub input_snapshot: Option<String>,
    pub error: Option<String>,
}

const RUN_COLUMNS: &str = "id,item_id,session_id,trigger,state,stage_reached,queued_at,started_at,\
    finished_at,cost_micro_usd,settings_snapshot,form_snapshot,input_snapshot,error";

fn run_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<RunRow> {
    let trigger: String = r.get(3)?;
    let state: String = r.get(4)?;
    Ok(RunRow {
        id: r.get(0)?,
        item_id: r.get(1)?,
        session_id: r.get(2)?,
        trigger: RunTrigger::parse(&trigger).unwrap_or(RunTrigger::Manual),
        state: RunState::parse(&state).unwrap_or(RunState::Failed),
        stage_reached: r.get(5)?,
        queued_at: r.get(6)?,
        started_at: r.get(7)?,
        finished_at: r.get(8)?,
        cost_micro_usd: r.get(9)?,
        settings_snapshot: r.get(10)?,
        form_snapshot: r.get(11)?,
        input_snapshot: r.get(12)?,
        error: r.get(13)?,
    })
}

/// A run row to insert.
#[derive(Debug, Clone)]
pub struct NewRun {
    pub item_id: i64,
    pub session_id: i64,
    pub trigger: RunTrigger,
    pub state: RunState,
    pub queued_at: i64,
    pub started_at: Option<i64>,
    pub settings_snapshot: Option<String>,
    pub form_snapshot: Option<String>,
    pub input_snapshot: Option<String>,
}

pub fn insert_run(conn: &Connection, run: &NewRun) -> Result<i64> {
    conn.execute(
        "INSERT INTO research_runs(item_id,session_id,trigger,state,queued_at,started_at,\
         settings_snapshot,form_snapshot,input_snapshot) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
        params![
            run.item_id,
            run.session_id,
            run.trigger.as_str(),
            run.state.as_str(),
            run.queued_at,
            run.started_at,
            run.settings_snapshot,
            run.form_snapshot,
            run.input_snapshot,
        ],
    )
    .map_err(db)?;
    Ok(conn.last_insert_rowid())
}

pub fn run(conn: &Connection, id: i64) -> Result<Option<RunRow>> {
    conn.query_row(
        &format!("SELECT {RUN_COLUMNS} FROM research_runs WHERE id=?1"),
        [id],
        run_row,
    )
    .optional()
    .map_err(db)
}

/// Every research run, oldest first; `session` narrows to one session. Browse runs (A12: one
/// call from the "Search this photo" window) are not research runs and are left out; see
/// [`all_runs`].
pub fn runs(conn: &Connection, session: Option<i64>) -> Result<Vec<RunRow>> {
    let mut stmt = conn
        .prepare(&format!(
            "SELECT {RUN_COLUMNS} FROM research_runs WHERE (?1 IS NULL OR session_id=?1) \
             AND trigger<>'browse' ORDER BY id"
        ))
        .map_err(db)?;
    let rows = stmt
        .query_map([session], run_row)
        .map_err(db)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(db)?;
    Ok(rows)
}

/// Every run, browse runs included (the export), oldest first.
pub fn all_runs(conn: &Connection, session: Option<i64>) -> Result<Vec<RunRow>> {
    let mut stmt = conn
        .prepare(&format!(
            "SELECT {RUN_COLUMNS} FROM research_runs WHERE (?1 IS NULL OR session_id=?1) ORDER BY id"
        ))
        .map_err(db)?;
    let rows = stmt
        .query_map([session], run_row)
        .map_err(db)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(db)?;
    Ok(rows)
}

/// Every run in `state`, app-wide, oldest first.
pub fn runs_in_state(conn: &Connection, state: RunState) -> Result<Vec<RunRow>> {
    let mut stmt = conn
        .prepare(&format!(
            "SELECT {RUN_COLUMNS} FROM research_runs WHERE state=?1 ORDER BY id"
        ))
        .map_err(db)?;
    let rows = stmt
        .query_map([state.as_str()], run_row)
        .map_err(db)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(db)?;
    Ok(rows)
}

pub fn set_run_state(
    conn: &Connection,
    id: i64,
    state: RunState,
    error: Option<&str>,
    finished_at: Option<i64>,
) -> Result<()> {
    conn.execute(
        "UPDATE research_runs SET state=?1,error=COALESCE(?2,error),\
         finished_at=COALESCE(?3,finished_at) WHERE id=?4",
        params![state.as_str(), error, finished_at, id],
    )
    .map_err(db)?;
    Ok(())
}

pub fn set_stage_reached(conn: &Connection, id: i64, stage: &str) -> Result<()> {
    conn.execute(
        "UPDATE research_runs SET stage_reached=?1 WHERE id=?2",
        params![stage, id],
    )
    .map_err(db)?;
    Ok(())
}

pub fn set_trigger(conn: &Connection, id: i64, trigger: RunTrigger) -> Result<()> {
    conn.execute(
        "UPDATE research_runs SET trigger=?1 WHERE id=?2",
        params![trigger.as_str(), id],
    )
    .map_err(db)?;
    Ok(())
}

/// The start transaction's update (MC-A6): `running`, `started_at` and the three snapshots.
pub fn start_run(
    conn: &Connection,
    id: i64,
    started_at: i64,
    settings_snapshot: &str,
    form_snapshot: &str,
    input_snapshot: &str,
) -> Result<bool> {
    // MC-A6 (MA13): one running run app-wide, re-checked inside the start transaction, so two
    // starts that both read "nothing running" cannot both commit.
    let changed = conn
        .execute(
            "UPDATE research_runs SET state='running',started_at=?1,settings_snapshot=?2,\
             form_snapshot=?3,input_snapshot=?4 WHERE id=?5 AND state='queued' \
             AND NOT EXISTS (SELECT 1 FROM research_runs WHERE state='running')",
            params![
                started_at,
                settings_snapshot,
                form_snapshot,
                input_snapshot,
                id
            ],
        )
        .map_err(db)?;
    Ok(changed == 1)
}

pub fn add_run_cost(conn: &Connection, id: i64, micro_usd: i64) -> Result<()> {
    conn.execute(
        "UPDATE research_runs SET cost_micro_usd=cost_micro_usd+?1 WHERE id=?2",
        params![micro_usd, id],
    )
    .map_err(db)?;
    Ok(())
}

/// One `research_calls` row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallRow {
    pub id: i64,
    pub run_id: i64,
    pub stage: String,
    pub unit_key: String,
    pub attempt: i64,
    pub provider: String,
    pub model: String,
    pub effort: String,
    pub state: String,
    pub request: String,
    pub response: Option<String>,
    pub usage: Option<String>,
    pub cost_micro_usd: i64,
    pub started_at: i64,
    pub finished_at: Option<i64>,
    pub duration_ms: Option<i64>,
    pub error_class: Option<String>,
    pub error_text: Option<String>,
    pub redaction_hit: bool,
}

const CALL_COLUMNS: &str = "id,run_id,stage,unit_key,attempt,provider,model,effort,state,request,\
    response,usage,cost_micro_usd,started_at,finished_at,duration_ms,error_class,error_text,\
    redaction_hit";

fn call_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<CallRow> {
    Ok(CallRow {
        id: r.get(0)?,
        run_id: r.get(1)?,
        stage: r.get(2)?,
        unit_key: r.get(3)?,
        attempt: r.get(4)?,
        provider: r.get(5)?,
        model: r.get(6)?,
        effort: r.get(7)?,
        state: r.get(8)?,
        request: r.get(9)?,
        response: r.get(10)?,
        usage: r.get(11)?,
        cost_micro_usd: r.get(12)?,
        started_at: r.get(13)?,
        finished_at: r.get(14)?,
        duration_ms: r.get(15)?,
        error_class: r.get(16)?,
        error_text: r.get(17)?,
        redaction_hit: r.get::<_, i64>(18)? != 0,
    })
}

/// The calls of a run (every run when `None`), in insert order.
pub fn calls(conn: &Connection, run_id: Option<i64>) -> Result<Vec<CallRow>> {
    let mut stmt = conn
        .prepare(&format!(
            "SELECT {CALL_COLUMNS} FROM research_calls WHERE (?1 IS NULL OR run_id=?1) ORDER BY id"
        ))
        .map_err(db)?;
    let rows = stmt
        .query_map([run_id], call_row)
        .map_err(db)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(db)?;
    Ok(rows)
}

/// A call record in state `sent`.
pub struct NewCall<'a> {
    pub run_id: i64,
    pub stage: &'a str,
    pub unit_key: &'a str,
    pub attempt: i64,
    pub provider: &'a str,
    pub model: &'a str,
    pub effort: &'a str,
    pub request: &'a str,
    pub started_at: i64,
    pub redaction_hit: bool,
}

pub fn insert_call(conn: &Connection, call: &NewCall<'_>) -> Result<i64> {
    conn.execute(
        "INSERT INTO research_calls(run_id,stage,unit_key,attempt,provider,model,effort,state,\
         request,started_at,redaction_hit) VALUES (?1,?2,?3,?4,?5,?6,?7,'sent',?8,?9,?10)",
        params![
            call.run_id,
            call.stage,
            call.unit_key,
            call.attempt,
            call.provider,
            call.model,
            call.effort,
            call.request,
            call.started_at,
            i64::from(call.redaction_hit),
        ],
    )
    .map_err(db)?;
    Ok(conn.last_insert_rowid())
}

/// How a call record ends.
pub struct CallEnd<'a> {
    pub state: &'a str,
    pub response: Option<&'a str>,
    pub usage: Option<&'a str>,
    pub cost_micro_usd: i64,
    pub finished_at: i64,
    pub duration_ms: i64,
    pub error_class: Option<&'a str>,
    pub error_text: Option<&'a str>,
    pub redaction_hit: bool,
}

pub fn end_call(conn: &Connection, id: i64, end: &CallEnd<'_>) -> Result<()> {
    conn.execute(
        "UPDATE research_calls SET state=?1,response=?2,usage=?3,cost_micro_usd=?4,finished_at=?5,\
         duration_ms=?6,error_class=?7,error_text=?8,redaction_hit=MAX(redaction_hit,?9) WHERE id=?10",
        params![
            end.state,
            end.response,
            end.usage,
            end.cost_micro_usd,
            end.finished_at,
            end.duration_ms,
            end.error_class,
            end.error_text,
            i64::from(end.redaction_hit),
            id,
        ],
    )
    .map_err(db)?;
    Ok(())
}

/// The attempt number the next call of a unit gets on one provider: calls never sent do not count.
pub fn next_attempt(
    conn: &Connection,
    run_id: i64,
    stage: &str,
    unit_key: &str,
    provider: &str,
) -> Result<i64> {
    let used: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM research_calls WHERE run_id=?1 AND stage=?2 AND unit_key=?3 \
             AND provider=?4 AND state!='not_sent'",
            params![run_id, stage, unit_key, provider],
            |r| r.get(0),
        )
        .map_err(db)?;
    Ok(used + 1)
}

/// The calls of a run still `sent` with no response (left by a kill).
pub fn open_calls(conn: &Connection, run_id: i64) -> Result<Vec<CallRow>> {
    Ok(calls(conn, Some(run_id))?
        .into_iter()
        .filter(|c| c.state == "sent")
        .collect())
}

/// `(started_at, cost)` of every call, for today's spend (MC-A8).
pub fn call_costs(conn: &Connection) -> Result<Vec<(i64, i64)>> {
    let mut stmt = conn
        .prepare("SELECT started_at,cost_micro_usd FROM research_calls")
        .map_err(db)?;
    let rows = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .map_err(db)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(db)?;
    Ok(rows)
}

/// One `research_outbound` row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutboundRow {
    pub id: i64,
    pub run_id: i64,
    pub call_id: i64,
    pub photo_id: i64,
    pub provider: String,
    pub sent_at: i64,
    pub sha256: String,
}

pub fn insert_outbound(
    conn: &Connection,
    run_id: i64,
    call_id: i64,
    photo_id: i64,
    provider: &str,
    sent_at: i64,
    sha256: &str,
) -> Result<()> {
    conn.execute(
        "INSERT INTO research_outbound(run_id,call_id,photo_id,provider,sent_at,sha256) \
         VALUES (?1,?2,?3,?4,?5,?6)",
        params![run_id, call_id, photo_id, provider, sent_at, sha256],
    )
    .map_err(db)?;
    Ok(())
}

/// A call that never left: its outbound rows are not proof of anything sent.
pub fn delete_outbound(conn: &Connection, call_id: i64) -> Result<()> {
    conn.execute("DELETE FROM research_outbound WHERE call_id=?1", [call_id])
        .map_err(db)?;
    Ok(())
}

pub fn outbound(conn: &Connection, run_id: Option<i64>) -> Result<Vec<OutboundRow>> {
    let mut stmt = conn
        .prepare(
            "SELECT id,run_id,call_id,photo_id,provider,sent_at,sha256 FROM research_outbound \
             WHERE (?1 IS NULL OR run_id=?1) ORDER BY id",
        )
        .map_err(db)?;
    let rows = stmt
        .query_map([run_id], |r| {
            Ok(OutboundRow {
                id: r.get(0)?,
                run_id: r.get(1)?,
                call_id: r.get(2)?,
                photo_id: r.get(3)?,
                provider: r.get(4)?,
                sent_at: r.get(5)?,
                sha256: r.get(6)?,
            })
        })
        .map_err(db)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(db)?;
    Ok(rows)
}

/// A trace step to commit; `seq` is taken in the committing transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewStep {
    pub stage: String,
    pub kind: crate::domain::StepKind,
    pub summary: String,
    pub detail: String,
    pub source_url: Option<String>,
    pub source_title: Option<String>,
    pub photo_ids: Vec<i64>,
    pub field_keys: Vec<String>,
}

/// One `research_steps` row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepRow {
    pub id: i64,
    pub run_id: i64,
    pub seq: i64,
    pub stage: String,
    pub kind: String,
    pub summary: String,
    pub detail: String,
    pub source_url: Option<String>,
    pub source_title: Option<String>,
    pub photo_ids: Vec<i64>,
    pub field_keys: Vec<String>,
    pub created_at: i64,
}

/// Inserts a step at `max(seq of the run) + 1` (gapless, in commit order) and returns its seq.
pub fn insert_step(conn: &Connection, run_id: i64, step: &NewStep, at: i64) -> Result<i64> {
    let seq: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(seq),0)+1 FROM research_steps WHERE run_id=?1",
            [run_id],
            |r| r.get(0),
        )
        .map_err(db)?;
    let photo_ids =
        serde_json::to_string(&step.photo_ids).map_err(|e| Error::InvalidData(e.to_string()))?;
    let field_keys =
        serde_json::to_string(&step.field_keys).map_err(|e| Error::InvalidData(e.to_string()))?;
    conn.execute(
        "INSERT INTO research_steps(run_id,seq,stage,kind,summary,detail,source_url,source_title,\
         photo_ids,field_keys,created_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
        params![
            run_id,
            seq,
            step.stage,
            step.kind.as_str(),
            step.summary,
            step.detail,
            step.source_url,
            step.source_title,
            photo_ids,
            field_keys,
            at,
        ],
    )
    .map_err(db)?;
    Ok(seq)
}

pub fn steps(conn: &Connection, run_id: i64) -> Result<Vec<StepRow>> {
    let mut stmt = conn
        .prepare(
            "SELECT id,run_id,seq,stage,kind,summary,detail,source_url,source_title,photo_ids,\
             field_keys,created_at FROM research_steps WHERE run_id=?1 ORDER BY seq",
        )
        .map_err(db)?;
    let rows = stmt
        .query_map([run_id], |r| {
            let photo_ids: String = r.get(9)?;
            let field_keys: String = r.get(10)?;
            Ok(StepRow {
                id: r.get(0)?,
                run_id: r.get(1)?,
                seq: r.get(2)?,
                stage: r.get(3)?,
                kind: r.get(4)?,
                summary: r.get(5)?,
                detail: r.get(6)?,
                source_url: r.get(7)?,
                source_title: r.get(8)?,
                photo_ids: serde_json::from_str(&photo_ids).unwrap_or_default(),
                field_keys: serde_json::from_str(&field_keys).unwrap_or_default(),
                created_at: r.get(11)?,
            })
        })
        .map_err(db)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(db)?;
    Ok(rows)
}

/// One committed unit (`research_stage_outputs`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputRow {
    pub run_id: i64,
    pub stage: String,
    pub unit_key: String,
    pub fingerprint: String,
    pub output: String,
    pub reused_from_run_id: Option<i64>,
    pub committed_at: i64,
}

fn output_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<OutputRow> {
    Ok(OutputRow {
        run_id: r.get(0)?,
        stage: r.get(1)?,
        unit_key: r.get(2)?,
        fingerprint: r.get(3)?,
        output: r.get(4)?,
        reused_from_run_id: r.get(5)?,
        committed_at: r.get(6)?,
    })
}

const OUTPUT_COLUMNS: &str =
    "run_id,stage,unit_key,fingerprint,output,reused_from_run_id,committed_at";

pub fn insert_output(conn: &Connection, row: &OutputRow) -> Result<()> {
    conn.execute(
        "INSERT INTO research_stage_outputs(run_id,stage,unit_key,fingerprint,output,\
         reused_from_run_id,committed_at) VALUES (?1,?2,?3,?4,?5,?6,?7)",
        params![
            row.run_id,
            row.stage,
            row.unit_key,
            row.fingerprint,
            row.output,
            row.reused_from_run_id,
            row.committed_at,
        ],
    )
    .map_err(db)?;
    Ok(())
}

pub fn output(
    conn: &Connection,
    run_id: i64,
    stage: &str,
    unit_key: &str,
) -> Result<Option<OutputRow>> {
    conn.query_row(
        &format!(
            "SELECT {OUTPUT_COLUMNS} FROM research_stage_outputs \
             WHERE run_id=?1 AND stage=?2 AND unit_key=?3"
        ),
        params![run_id, stage, unit_key],
        output_row,
    )
    .optional()
    .map_err(db)
}

/// The committed units of a run, in commit order.
pub fn outputs(conn: &Connection, run_id: i64) -> Result<Vec<OutputRow>> {
    let mut stmt = conn
        .prepare(&format!(
            "SELECT {OUTPUT_COLUMNS} FROM research_stage_outputs WHERE run_id=?1 ORDER BY rowid"
        ))
        .map_err(db)?;
    let rows = stmt
        .query_map([run_id], output_row)
        .map_err(db)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(db)?;
    Ok(rows)
}

/// MC-A11 retention: the call rows started at or before `cutoff` (`started_at + record_days ×
/// 86 400 000 ≤ now` ⇔ `started_at ≤ now − record_days × 86 400 000`).
pub fn calls_started_by(conn: &Connection, cutoff: i64) -> Result<Vec<CallRow>> {
    let mut stmt = conn
        .prepare(&format!(
            "SELECT {CALL_COLUMNS} FROM research_calls WHERE started_at<=?1 ORDER BY id"
        ))
        .map_err(db)?;
    stmt.query_map([cutoff], call_row)
        .map_err(db)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(db)
}

pub fn delete_call(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM research_calls WHERE id=?1", [id])
        .map_err(db)?;
    Ok(())
}

/// One `research_feedback` row as the export writes it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeedbackRow {
    pub id: i64,
    pub key: String,
    pub suggested_value: String,
    pub confidence: String,
    pub final_value: Option<String>,
    pub action: String,
    pub at: i64,
}

/// The item's feedback rows at or after `since` (UTC ms), oldest first.
pub fn feedback_since(conn: &Connection, item_id: i64, since: i64) -> Result<Vec<FeedbackRow>> {
    let mut stmt = conn
        .prepare(
            "SELECT id,key,suggested_value,confidence,final_value,action,at FROM research_feedback \
             WHERE item_id=?1 AND at>=?2 ORDER BY at,id",
        )
        .map_err(db)?;
    stmt.query_map(params![item_id, since], |r| {
        Ok(FeedbackRow {
            id: r.get(0)?,
            key: r.get(1)?,
            suggested_value: r.get(2)?,
            confidence: r.get(3)?,
            final_value: r.get(4)?,
            action: r.get(5)?,
            at: r.get(6)?,
        })
    })
    .map_err(db)?
    .collect::<rusqlite::Result<Vec<_>>>()
    .map_err(db)
}

/// Whether the photo row is gone or `deleted` (MC-A11 exclusions at export time).
pub fn photo_deleted(conn: &Connection, photo_id: i64) -> Result<bool> {
    use rusqlite::OptionalExtension;
    let class: Option<String> = conn
        .query_row(
            "SELECT classification FROM photos WHERE id=?1",
            [photo_id],
            |r| r.get(0),
        )
        .optional()
        .map_err(db)?;
    Ok(class.is_none_or(|c| c == "deleted"))
}

/// The item's folder key (`items.folder_key`).
pub fn folder_key(conn: &Connection, item_id: i64) -> Result<String> {
    conn.query_row("SELECT folder_key FROM items WHERE id=?1", [item_id], |r| {
        r.get(0)
    })
    .map_err(db)
}
