//! The life of a run around `execute` (MC-A6): when a run row is created, when it may start and
//! in which order, the start transaction, and what the status says when research cannot run.

use std::{collections::HashMap, time::Instant};

use psm_core::{
    Error, Result,
    domain::photo::Classification,
    ports::store::{Store, StoreTx},
};
use rusqlite::params;

use crate::{
    api::{QueueView, Reason, ResearchStatus, RunView, SessionResearch, StepView},
    config::{ResearchConfig, ResearchSettings},
    domain::{Prerequisite, RunState, RunTrigger, Stage},
    images::{eligible, source},
    job::KIND,
    rules::{
        cost::{day_spend, parse_micro_usd},
        photo_plan::{PhotoCandidate, look_at},
        run_rules::{
            InputSnapshot, MayStart, PrerequisiteInput, StartInput, arranged_queues, first_failing,
            may_start, start_order,
        },
        write_table::never_written,
    },
    service::{ResearchService, form_snapshot},
    shop::ShopFacts,
    store::{self, NewRun, RunRow},
};

/// The error of a queued look or writer run cancelled by Research / Run again (WC1.9).
pub const STARTED_AGAIN: &str = "research started again";

/// The instant of each session's last change, used for the missing-source wait.
#[derive(Debug, Default)]
pub struct LiveState {
    pub last_change: HashMap<i64, Instant>,
}

/// The research photos of a session, as counted for start and snapshots.
#[derive(Debug, Clone)]
struct SessionPhoto {
    id: i64,
    classification: Classification,
    position: Option<i64>,
    revision: i64,
    has_source: bool,
}

impl ResearchService {
    /// The live `research` settings.
    pub fn settings(&self) -> ResearchSettings {
        ResearchSettings::from_slot((self.settings)().as_ref())
    }

    /// The item of a session (`None`: the Unassigned session).
    pub(crate) fn session_item(&self, session_id: i64) -> Result<Option<i64>> {
        self.store
            .research_transaction(|tx| Ok(tx.session(session_id)?.item_id))
    }

    /// The session's eligible photos (`main`, `condition`, `unreviewed`) for a run of `item_id`.
    fn session_photos(&self, session_id: i64, item_id: i64) -> Result<Vec<SessionPhoto>> {
        let probe = RunRow {
            id: 0,
            item_id,
            session_id,
            trigger: RunTrigger::Arranged,
            state: RunState::Queued,
            stage_reached: None,
            queued_at: 0,
            started_at: None,
            finished_at: None,
            cost_micro_usd: 0,
            settings_snapshot: None,
            form_snapshot: None,
            input_snapshot: None,
            error: None,
        };
        self.store.research_transaction(|tx| {
            let mut out = Vec::new();
            for photo in tx.photos(session_id)? {
                if !eligible(tx, &probe, photo.id)? {
                    continue;
                }
                out.push(SessionPhoto {
                    id: photo.id,
                    classification: photo.classification,
                    position: photo.position,
                    revision: tx.composition(photo.id)?.revision,
                    has_source: source(tx, photo.id)?.is_some(),
                });
            }
            Ok(out)
        })
    }

    /// The photo-derived input identities of the session now (photos with a source only). The
    /// stage 1 part is the photos stage 1 looks at (`rules::photo_plan::look_at` under `limit`):
    /// a change to a photo beyond the limit changes nothing.
    pub(crate) fn input_now(
        &self,
        session_id: i64,
        item_id: i64,
        limit: u32,
    ) -> Result<InputSnapshot> {
        let mut photos: Vec<SessionPhoto> = self
            .session_photos(session_id, item_id)?
            .into_iter()
            .filter(|p| p.has_source)
            .collect();
        photos.sort_by_key(|p| (p.position.unwrap_or(i64::MAX), p.id));
        let ids = |keep: &dyn Fn(Classification) -> bool| -> Vec<(i64, i64)> {
            let mut ids: Vec<(i64, i64)> = photos
                .iter()
                .filter(|p| keep(p.classification))
                .map(|p| (p.id, p.revision))
                .collect();
            ids.sort();
            ids
        };
        let candidates: Vec<PhotoCandidate> = photos
            .iter()
            .map(|p| PhotoCandidate {
                id: p.id,
                classification: p.classification,
                position: p.position,
                revision: p.revision,
            })
            .collect();
        let mut observe: Vec<(i64, i64)> = look_at(
            &candidates,
            usize::try_from(limit.max(1)).unwrap_or(usize::MAX),
        )
        .chosen
        .iter()
        .map(|c| (c.id, c.revision))
        .collect();
        observe.sort();
        Ok(InputSnapshot {
            observe,
            cover: photos
                .iter()
                .find(|p| p.classification == Classification::Main)
                .map(|p| (p.id, p.revision)),
            condition: ids(&|c| c == Classification::Condition),
        })
    }

    fn today_spend(&self) -> Result<i64> {
        let calls = self
            .store
            .research_transaction(|tx| store::call_costs(tx.conn()))?;
        Ok(day_spend(&calls, self.clock.now_ms(), &chrono::Local))
    }

    pub(crate) fn day_limit_reached(&self, settings: &ResearchSettings) -> Result<bool> {
        let cap = parse_micro_usd(&settings.max_cost_per_day).unwrap_or(0);
        Ok(self.today_spend()? >= cap)
    }

    /// The first `(provider, model)` a run can call with no `fallback_call_usd`.
    pub(crate) fn missing_price(&self, settings: &ResearchSettings) -> Option<(String, String)> {
        let fallbacks = settings.fallbacks();
        let has = |provider: &str, model: &str| {
            fallbacks
                .iter()
                .any(|f| f.provider == provider && f.model == model)
        };
        let mut pairs: Vec<(String, String)> = Vec::new();
        if let Some(model) = &self.providers.model {
            for stage in [
                Stage::Observe,
                Stage::Read,
                Stage::Compare,
                Stage::Labels,
                Stage::Condition,
                Stage::Decide,
            ] {
                pairs.push((model.id(), settings.stage(stage).model));
            }
        }
        if let Some(search) = &self.providers.search {
            pairs.push((search.id(), String::new()));
        }
        if let Some(reader) = &self.providers.reader {
            pairs.push((reader.id(), String::new()));
        }
        pairs.into_iter().find(|(p, m)| !has(p, m))
    }

    pub(crate) fn key_set(&self, account: &str) -> bool {
        self.secrets
            .get(account)
            .ok()
            .flatten()
            .is_some_and(|k| !k.trim().is_empty())
    }

    /// The facts MC-A6's prerequisites are read from, for an item.
    fn prerequisite_input(
        &self,
        item_id: i64,
        settings: &ResearchSettings,
    ) -> Result<PrerequisiteInput> {
        let form = form_snapshot(
            &self.store,
            self.clock.as_ref(),
            self.integration.as_deref(),
            item_id,
        )?;
        Ok(PrerequisiteInput {
            integration: self.shop.is_some(),
            model_key: self.providers.model.is_some()
                && self.key_set(crate::service::MODEL_ACCOUNT),
            search_key: self.providers.search.is_some()
                && self.key_set(crate::service::SEARCH_ACCOUNT),
            missing_price: self.missing_price(settings),
            writable_field: form
                .fields
                .iter()
                .any(|f| !never_written(&f.definition, Some(f.step))),
            chosen_field: form.fields.iter().any(|f| {
                !never_written(&f.definition, Some(f.step))
                    && settings.fill_keys.contains(&f.definition.key)
            }),
            day_limit_reached: self.day_limit_reached(settings)?,
            offline: self.gateway.offline(),
        })
    }

    /// The first failing prerequisite of a session (`None` for the Unassigned session too).
    pub fn first_failing(&self, session_id: i64) -> Result<Option<Prerequisite>> {
        let Some(item_id) = self.session_item(session_id)? else {
            return Ok(None);
        };
        let settings = self.settings();
        Ok(first_failing(&self.prerequisite_input(item_id, &settings)?))
    }

    pub(crate) fn insert_queued(
        &self,
        item_id: i64,
        session_id: i64,
        trigger: RunTrigger,
    ) -> Result<i64> {
        let now = self.clock.now_ms();
        let id = self.store.research_transaction(|tx| {
            store::insert_run(
                tx.conn(),
                &NewRun {
                    item_id,
                    session_id,
                    trigger,
                    state: RunState::Queued,
                    queued_at: now,
                    started_at: None,
                    settings_snapshot: None,
                    form_snapshot: None,
                    input_snapshot: None,
                },
            )
        })?;
        self.changed(session_id);
        Ok(id)
    }

    /// A photo row was created, or a classification, position or composition changed, or a
    /// research source became available. This only moves the source-wait clock.
    /// Never call it inside another service's transaction.
    pub fn session_changed(&self, session_id: i64) {
        if let Err(error) = self.session_changed_inner(session_id) {
            tracing::warn!(session_id, %error, "research: session change not evaluated");
        }
    }

    fn session_changed_inner(&self, session_id: i64) -> Result<()> {
        let Some(_) = self.session_item(session_id)? else {
            return Ok(());
        };
        self.state
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?
            .last_change
            .insert(session_id, self.clock.mono());
        Ok(())
    }

    /// A settings save does not start or cancel a research run.
    pub fn settings_changed(&self) {}

    /// Retire queued runs from builds where photo changes started them automatically.
    pub(crate) fn cancel_legacy_queued(&self) -> Result<()> {
        let now = self.clock.now_ms();
        let cancelled = self.store.research_transaction(|tx| {
            let mut sessions = Vec::new();
            for run in store::runs_in_state(tx.conn(), RunState::Queued)? {
                if matches!(run.trigger, RunTrigger::Auto | RunTrigger::FollowUp) {
                    store::set_run_state(tx.conn(), run.id, RunState::Cancelled, None, Some(now))?;
                    sessions.push(run.session_id);
                }
            }
            Ok(sessions)
        })?;
        for session_id in cancelled {
            self.changed(session_id);
        }
        Ok(())
    }

    /// A photographer has finished arranging this session. A19 (owner 2026-11-18): research
    /// starts by itself at most once per session ([`arranged_queues`]): queued only when no run of
    /// the session is queued, running, finished or stopped at the cost limit. A failed or
    /// cancelled run produced nothing, so arranging again retries it. Afterwards only **Run
    /// again** researches the session again; photo changes queue nothing.
    pub fn arranged(&self, session_id: i64) -> Result<bool> {
        let Some(item_id) = self.session_item(session_id)? else {
            return Err(Error::InvalidData(
                "the Unassigned session is never researched".into(),
            ));
        };
        let settings = self.settings();
        if first_failing(&self.prerequisite_input(item_id, &settings)?)
            .is_some_and(|p| p.blocks_creation())
        {
            return Ok(false);
        }
        if !self
            .session_photos(session_id, item_id)?
            .iter()
            .any(|p| p.classification == Classification::Main)
        {
            return Ok(false);
        }
        let runs = self
            .store
            .research_transaction(|tx| store::runs(tx.conn(), Some(session_id)))?;
        let needed = arranged_queues(&runs.iter().map(|r| r.state).collect::<Vec<_>>());
        if needed {
            self.queue_research(item_id, session_id, RunTrigger::Arranged)?;
        }
        Ok(needed)
    }

    /// Queues a research run. W1 (WC1.9): a queued look or writer run of the session (the
    /// per-session queued index allows one queued run) becomes `cancelled` with
    /// [`STARTED_AGAIN`], in the same transaction.
    fn queue_research(&self, item_id: i64, session_id: i64, trigger: RunTrigger) -> Result<i64> {
        let now = self.clock.now_ms();
        let id = self.store.research_transaction(|tx| {
            let conn = tx.conn();
            for run in store::all_runs(conn, Some(session_id))? {
                if run.state == RunState::Queued && matches!(run.trigger.as_str(), "look" | "write")
                {
                    store::set_run_state(
                        conn,
                        run.id,
                        RunState::Cancelled,
                        Some(STARTED_AGAIN),
                        Some(now),
                    )?;
                }
            }
            store::insert_run(
                conn,
                &NewRun {
                    item_id,
                    session_id,
                    trigger,
                    state: RunState::Queued,
                    queued_at: now,
                    started_at: None,
                    settings_snapshot: None,
                    form_snapshot: None,
                    input_snapshot: None,
                },
            )
        })?;
        self.changed(session_id);
        Ok(id)
    }

    /// **Run again** (MC-A6): a queued run becomes `manual`; otherwise one `manual` run is
    /// queued (also beside a running one).
    pub fn run_again(&self, session_id: i64) -> Result<()> {
        let Some(item_id) = self.session_item(session_id)? else {
            return Err(Error::InvalidData(
                "the Unassigned session is never researched".into(),
            ));
        };
        let settings = self.settings();
        if let Some(blocking) = first_failing(&self.prerequisite_input(item_id, &settings)?)
            .filter(Prerequisite::blocks_creation)
        {
            return Err(Error::InvalidData(blocking.sentence()));
        }
        let queued = self.store.research_transaction(|tx| {
            Ok(store::runs(tx.conn(), Some(session_id))?
                .into_iter()
                .find(|r| r.state == RunState::Queued))
        })?;
        match queued {
            Some(run) => {
                if run.trigger != RunTrigger::Manual {
                    self.store.research_transaction(|tx| {
                        store::set_trigger(tx.conn(), run.id, RunTrigger::Manual)
                    })?;
                    self.changed(session_id);
                }
            }
            None => {
                self.queue_research(item_id, session_id, RunTrigger::Manual)?;
            }
        }
        Ok(())
    }

    /// Since the session's last change on the monotonic clock (at launch: since the launch).
    fn since_change_ms(&self, session_id: i64) -> Result<i64> {
        let last = self
            .state
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?
            .last_change
            .get(&session_id)
            .copied()
            .unwrap_or(self.launched);
        Ok(i64::try_from(
            self.clock
                .mono()
                .saturating_duration_since(last)
                .as_millis(),
        )
        .unwrap_or(i64::MAX))
    }

    fn may_start_run(&self, run: &RunRow, settings: &ResearchSettings) -> Result<MayStart> {
        let input = self.prerequisite_input(run.item_id, settings)?;
        // W1: a look run needs only what its two photo looks call (WC1.8); W2: a writer run
        // what its call needs (WC3 "Start").
        let blocked = if run.trigger == RunTrigger::Look {
            self.look_block(settings).is_some()
        } else if run.trigger == RunTrigger::Write {
            self.writer_block(run.item_id, settings, &self.writer_settings())?
                .is_some()
        } else {
            first_failing(&input).is_some_and(|p| p.blocks_creation())
        };
        let any_running = !self
            .store
            .research_transaction(|tx| store::runs_in_state(tx.conn(), RunState::Running))?
            .is_empty();
        let photos = self.session_photos(run.session_id, run.item_id)?;
        Ok(may_start(&StartInput {
            trigger: run.trigger,
            blocked,
            any_running,
            day_limit_reached: input.day_limit_reached,
            since_change_ms: self.since_change_ms(run.session_id)?,
            source_wait_ms: settings.source_wait_seconds.saturating_mul(1_000),
            with_source: photos.iter().filter(|p| p.has_source).count(),
            without_source: photos.iter().filter(|p| !p.has_source).count(),
        }))
    }

    /// Starts the first startable queued run, if no run is running (MC-A6 start order). Returns
    /// the started run's id.
    pub fn tick(&self) -> Result<Option<i64>> {
        let (running, queued) = self.store.research_transaction(|tx| {
            Ok((
                store::runs_in_state(tx.conn(), RunState::Running)?,
                store::runs_in_state(tx.conn(), RunState::Queued)?,
            ))
        })?;
        if !running.is_empty() || queued.is_empty() {
            return Ok(None);
        }
        let settings = self.settings();
        let order = start_order(
            &queued
                .iter()
                .map(|r| (r.trigger, r.queued_at, r.id))
                .collect::<Vec<_>>(),
        );
        for id in order {
            let Some(run) = queued.iter().find(|r| r.id == id) else {
                continue;
            };
            if self.may_start_run(run, &settings)? == MayStart::Yes {
                return Ok(self.start(run, &settings)?.then_some(run.id));
            }
        }
        Ok(None)
    }

    /// The start transaction: `running`, `started_at`, the settings, form and input snapshots,
    /// and one `jobs` row of kind `research` whose target is the run. Nothing is written when
    /// another run started first (returns `false`).
    fn start(&self, run: &RunRow, settings: &ResearchSettings) -> Result<bool> {
        let facts = match &self.shop {
            Some(shop) => shop.facts(self.external_id(run.item_id)?.as_deref())?,
            None => ShopFacts::default(),
        };
        let mut config = ResearchConfig::from_settings(settings, &facts);
        let form = form_snapshot(
            &self.store,
            self.clock.as_ref(),
            self.integration.as_deref(),
            run.item_id,
        )?;
        let text = |value: serde_json::Result<String>| {
            value.map_err(|e| Error::InvalidData(e.to_string()))
        };
        let input = if run.trigger == RunTrigger::Write {
            // W2 (WC7): the writer's settings, the instruction and examples actually sent, the
            // origin; `stages.write`; the input the call carries (WC2).
            let writer = self.writer_settings();
            let origin = run
                .settings_snapshot
                .as_deref()
                .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
                .and_then(|s| {
                    s[crate::writer::run::ORIGIN_KEY]
                        .as_str()
                        .map(str::to_owned)
                })
                .unwrap_or_default();
            config.stages.insert(
                Stage::Write.as_str().to_owned(),
                crate::config::StageConfig {
                    model: writer.model.clone(),
                    effort: writer.effort.clone(),
                    image_long_edge: 0,
                },
            );
            // W4: resolved by the function the preview uses.
            config.writer = Some(crate::writer::config::WriterSnapshot::resolve(
                &writer, &origin,
            ));
            text(serde_json::to_string(
                &self.writer_input(run.session_id, &writer)?,
            ))?
        } else {
            let input = self.input_now(run.session_id, run.item_id, settings.observe_max_photos)?;
            text(serde_json::to_string(&input))?
        };
        let (config, form) = (
            text(serde_json::to_string(&config))?,
            text(serde_json::to_string(&form))?,
        );
        let now = self.clock.now_ms();
        let started = self.store.research_transaction(|tx| {
            if !store::start_run(tx.conn(), run.id, now, &config, &form, &input)? {
                return Ok(false);
            }
            tx.conn()
                .execute(
                    "INSERT INTO jobs(kind,target,payload,status,attempts,next_attempt_at,\
                     created_at,updated_at) VALUES (?1,?2,'{}','queued',0,?3,?3,?3)",
                    params![KIND, run.id.to_string(), now],
                )
                .map_err(store::db)?;
            Ok(true)
        })?;
        if started {
            self.changed(run.session_id);
            if run.trigger == RunTrigger::Write {
                tracing::info!(
                    target: "psm::writer",
                    "writer.started run={} session={}",
                    run.id,
                    run.session_id
                );
            }
        }
        Ok(started)
    }

    /// What the panel shows for a session: the first failing prerequisite or the waiting reason,
    /// the latest run and its steps.
    pub fn status(&self, session_id: i64) -> Result<ResearchStatus> {
        let Some(item_id) = self.session_item(session_id)? else {
            return Ok(ResearchStatus {
                reason: None,
                run: None,
                steps: Vec::new(),
                fields_filled: 0,
                to_review: 0,
                can_run: false,
                wait_ends_at: None,
                queue: None,
            });
        };
        let settings = self.settings();
        let input = self.prerequisite_input(item_id, &settings)?;
        let failing = first_failing(&input);
        let latest = self
            .store
            .research_transaction(|tx| Ok(store::runs(tx.conn(), Some(session_id))?.pop()))?;
        let mut reason = failing.clone();
        let mut wait_ends_at = None;
        if reason.is_none()
            && let Some(run) = latest.as_ref().filter(|r| r.state == RunState::Queued)
            && self.may_start_run(run, &settings)? == MayStart::WaitingForEditor
        {
            reason = Some(Prerequisite::WaitingForEditor);
            // MC-A6 readiness: an automatic run waits `source_wait_seconds` from the session's
            // last change; the panel says "Waiting for Capture One" once that wait has ended.
            let remaining = if run.trigger == RunTrigger::Manual {
                0
            } else {
                settings
                    .source_wait_seconds
                    .saturating_mul(1_000)
                    .saturating_sub(self.since_change_ms(session_id)?)
                    .max(0)
            };
            wait_ends_at = Some(self.clock.now_ms().saturating_add(remaining));
        }
        let steps = match &latest {
            Some(run) => self
                .store
                .research_transaction(|tx| store::steps(tx.conn(), run.id))?
                .into_iter()
                .map(|s| StepView {
                    unread: s.kind == crate::domain::StepKind::Read.as_str()
                        && s.summary == crate::stages::UNREAD,
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
                .collect(),
            None => Vec::new(),
        };
        let (fields_filled, to_review) = self.derived_counts(item_id, latest.as_ref())?;
        let queue = match latest.as_ref().filter(|r| r.state == RunState::Queued) {
            Some(run) => self.queue_of(run)?,
            None => None,
        };
        Ok(ResearchStatus {
            reason: reason.map(|r| Reason {
                code: r.code().to_owned(),
                sentence: r.sentence(),
            }),
            run: latest.map(|r| RunView {
                id: r.id,
                state: r.state.as_str().to_owned(),
                trigger: r.trigger.as_str().to_owned(),
                stage_reached: r.stage_reached,
                error: r.error,
            }),
            steps,
            fields_filled,
            to_review,
            can_run: !failing.as_ref().is_some_and(Prerequisite::blocks_creation),
            wait_ends_at,
            queue,
        })
    }

    /// The name the sessions list shows for a session (item type, or the title when there is
    /// none) with its SKU.
    fn session_name(&self, session_id: i64) -> Result<Option<String>> {
        self.store.research_transaction(|tx| {
            use rusqlite::OptionalExtension;
            tx.conn()
                .query_row(
                    "SELECT CASE WHEN i.item_type<>'' THEN i.item_type ELSE i.title END, i.sku \
                     FROM sessions s JOIN items i ON i.id=s.item_id WHERE s.id=?1",
                    [session_id],
                    |r| {
                        Ok(format!(
                            "{} · {}",
                            r.get::<_, String>(0)?,
                            r.get::<_, String>(1)?
                        ))
                    },
                )
                .optional()
                .map_err(store::db)
        })
    }

    /// A6: the queued runs app-wide in MC-A6 start order, and the running run's session.
    fn line(&self) -> Result<(Vec<RunRow>, Option<i64>)> {
        let (running, queued) = self.store.research_transaction(|tx| {
            Ok((
                store::runs_in_state(tx.conn(), RunState::Running)?,
                store::runs_in_state(tx.conn(), RunState::Queued)?,
            ))
        })?;
        let order = start_order(
            &queued
                .iter()
                .map(|r| (r.trigger, r.queued_at, r.id))
                .collect::<Vec<_>>(),
        );
        let mut line: Vec<RunRow> = Vec::with_capacity(queued.len());
        for id in order {
            if let Some(run) = queued.iter().find(|r| r.id == id) {
                line.push(run.clone());
            }
        }
        Ok((line, running.first().map(|r| r.session_id)))
    }

    /// A6: where a queued run stands in the line.
    fn queue_of(&self, run: &RunRow) -> Result<Option<QueueView>> {
        let (line, running) = self.line()?;
        let Some(index) = line.iter().position(|r| r.id == run.id) else {
            return Ok(None);
        };
        let running_name = match running {
            Some(session_id) => self.session_name(session_id)?,
            None => None,
        };
        Ok(Some(QueueView {
            position: index + 1,
            running_session_id: running,
            running_name,
            manual: run.trigger == RunTrigger::Manual,
        }))
    }

    /// A6: the research state of each unpublished session that has a run, or that is arranged
    /// and cannot be researched; `session` limits it to one session. One pass over the runs and
    /// the suggestions; the form layout is read only for items holding a suggestion.
    pub fn overview(&self, session: Option<i64>) -> Result<Vec<SessionResearch>> {
        /// `(session_id, item_id, state)` of each session's latest run.
        type Latest = (i64, i64, String);
        /// Live unchecked AI suggestions per item.
        type Suggested = HashMap<i64, i64>;
        /// `(session_id, item_id)` of arranged sessions with no run.
        type Unrun = Vec<(i64, i64)>;
        let (latest, suggested, unrun): (Vec<Latest>, Suggested, Unrun) =
            self.store.research_transaction(|tx| {
                let conn = tx.conn();
                let mut stmt = conn
                    .prepare(
                        "SELECT r.session_id, r.item_id, r.state FROM research_runs r \
                         JOIN sessions s ON s.id=r.session_id \
                         WHERE r.id=(SELECT MAX(id) FROM research_runs WHERE session_id=r.session_id \
                         AND trigger NOT IN ('browse','look','write')) \
                         AND s.published_at IS NULL AND (?1 IS NULL OR r.session_id=?1) \
                         ORDER BY r.session_id",
                    )
                    .map_err(store::db)?;
                let latest = stmt
                    .query_map([session], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
                    .map_err(store::db)?
                    .collect::<rusqlite::Result<Vec<Latest>>>()
                    .map_err(store::db)?;
                let mut stmt = conn
                    .prepare(
                        "SELECT item_id, COUNT(*) FROM attribute_values \
                         WHERE provenance='ai' AND ai_status='suggested' GROUP BY item_id",
                    )
                    .map_err(store::db)?;
                let suggested = stmt
                    .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
                    .map_err(store::db)?
                    .collect::<rusqlite::Result<HashMap<i64, i64>>>()
                    .map_err(store::db)?;
                let mut stmt = conn
                    .prepare(
                        "SELECT s.id, s.item_id FROM sessions s WHERE s.arranged_at IS NOT NULL \
                         AND s.item_id IS NOT NULL AND s.published_at IS NULL \
                         AND NOT EXISTS (SELECT 1 FROM research_runs r WHERE r.session_id=s.id \
                         AND r.trigger NOT IN ('browse','look','write')) \
                         AND (?1 IS NULL OR s.id=?1) ORDER BY s.id",
                    )
                    .map_err(store::db)?;
                let unrun = stmt
                    .query_map([session], |r| Ok((r.get(0)?, r.get(1)?)))
                    .map_err(store::db)?
                    .collect::<rusqlite::Result<Vec<(i64, i64)>>>()
                    .map_err(store::db)?;
                Ok((latest, suggested, unrun))
            })?;
        let (line, _) = self.line()?;
        let mut out = Vec::new();
        for (session_id, item_id, state) in latest {
            let to_review = if suggested.get(&item_id).copied().unwrap_or(0) > 0 {
                self.derived_counts(item_id, None)?.1
            } else {
                0
            };
            let position = line
                .iter()
                .position(|r| r.session_id == session_id)
                .map(|i| i + 1);
            let state = match state.as_str() {
                "running" => "researching",
                "queued" => "waiting",
                _ if to_review > 0 => "to_review",
                "finished" | "stopped_at_cap" => "done",
                "failed" => "failed",
                _ => continue,
            };
            out.push(SessionResearch {
                session_id,
                state: state.to_owned(),
                position: if state == "waiting" { position } else { None },
                to_review,
                reason: None,
            });
        }
        if !unrun.is_empty() {
            let settings = self.settings();
            for (session_id, item_id) in unrun {
                if let Some(blocking) = first_failing(&self.prerequisite_input(item_id, &settings)?)
                    .filter(Prerequisite::blocks_creation)
                {
                    out.push(SessionResearch {
                        session_id,
                        state: "blocked".to_owned(),
                        position: None,
                        to_review: 0,
                        reason: Some(Reason {
                            code: blocking.code().to_owned(),
                            sentence: blocking.sentence(),
                        }),
                    });
                }
            }
        }
        out.sort_by_key(|s| s.session_id);
        Ok(out)
    }

    /// The store, as the port the app's services take.
    pub fn store(&self) -> &dyn Store {
        self.store.as_ref()
    }
}
