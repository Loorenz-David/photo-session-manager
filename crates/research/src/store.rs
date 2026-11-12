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

/// Every run, oldest first; `session` narrows to one session.
pub fn runs(conn: &Connection, session: Option<i64>) -> Result<Vec<RunRow>> {
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
) -> Result<()> {
    conn.execute(
        "UPDATE research_runs SET state='running',started_at=?1,settings_snapshot=?2,\
         form_snapshot=?3,input_snapshot=?4 WHERE id=?5",
        params![
            started_at,
            settings_snapshot,
            form_snapshot,
            input_snapshot,
            id
        ],
    )
    .map_err(db)?;
    Ok(())
}

pub fn add_run_cost(conn: &Connection, id: i64, micro_usd: i64) -> Result<()> {
    conn.execute(
        "UPDATE research_runs SET cost_micro_usd=cost_micro_usd+?1 WHERE id=?2",
        params![micro_usd, id],
    )
    .map_err(db)?;
    Ok(())
}
