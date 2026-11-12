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
    api::{Reason, ResearchStatus, RunView, StepView},
    config::{ResearchConfig, ResearchSettings},
    domain::{Prerequisite, RunState, RunTrigger, Stage},
    images::{eligible, source},
    job::KIND,
    rules::{
        cost::{day_spend, parse_micro_usd},
        run_rules::{
            InputSnapshot, MayStart, PrerequisiteInput, StartInput, first_failing,
            follow_up_needed, may_start, start_order,
        },
        write_table::never_written,
    },
    service::{ResearchService, form_snapshot},
    shop::ShopFacts,
    store::{self, NewRun, RunRow},
};

/// What the service keeps in memory: the instant of each session's last change (MC-A6 quiet
/// period; until one is seen, the launch instant counts).
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
    fn session_item(&self, session_id: i64) -> Result<Option<i64>> {
        self.store
            .research_transaction(|tx| Ok(tx.session(session_id)?.item_id))
    }

    /// The session's eligible photos (`main`, `condition`, `unreviewed`) for a run of `item_id`.
    fn session_photos(&self, session_id: i64, item_id: i64) -> Result<Vec<SessionPhoto>> {
        let probe = RunRow {
            id: 0,
            item_id,
            session_id,
            trigger: RunTrigger::Auto,
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

    /// The photo-derived input identities of the session now (photos with a source only).
    pub(crate) fn input_now(&self, session_id: i64, item_id: i64) -> Result<InputSnapshot> {
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
        Ok(InputSnapshot {
            observe: ids(&|c| matches!(c, Classification::Main | Classification::Unreviewed)),
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

    fn day_limit_reached(&self, settings: &ResearchSettings) -> Result<bool> {
        let cap = parse_micro_usd(&settings.max_cost_per_day).unwrap_or(0);
        Ok(self.today_spend()? >= cap)
    }

    /// The first `(provider, model)` a run can call with no `fallback_call_usd`.
    fn missing_price(&self, settings: &ResearchSettings) -> Option<(String, String)> {
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

    fn key_set(&self, account: &str) -> bool {
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
            day_limit_reached: self.day_limit_reached(settings)?,
            offline: self.gateway.offline(),
            auto: settings.auto,
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

    fn insert_queued(&self, item_id: i64, session_id: i64, trigger: RunTrigger) -> Result<i64> {
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
    /// research source became available, in the session (MC-A6 event table, `auto` trigger).
    /// Never call it inside another service's transaction.
    pub fn session_changed(&self, session_id: i64) {
        if let Err(error) = self.session_changed_inner(session_id) {
            tracing::warn!(session_id, %error, "research: session change not evaluated");
        }
    }

    fn session_changed_inner(&self, session_id: i64) -> Result<()> {
        let Some(item_id) = self.session_item(session_id)? else {
            return Ok(());
        };
        self.state
            .lock()
            .map_err(|e| Error::InvalidData(e.to_string()))?
            .last_change
            .insert(session_id, self.clock.mono());
        let settings = self.settings();
        let input = self.prerequisite_input(item_id, &settings)?;
        if first_failing(&input).is_some_and(|p| p.blocks_creation()) || !settings.auto {
            return Ok(());
        }
        let runs = self
            .store
            .research_transaction(|tx| store::runs(tx.conn(), Some(session_id)))?;
        if runs.is_empty() {
            let has_main = self
                .session_photos(session_id, item_id)?
                .iter()
                .any(|p| p.classification == Classification::Main);
            if has_main {
                self.insert_queued(item_id, session_id, RunTrigger::Auto)?;
            }
            return Ok(());
        }
        self.follow_up(session_id, item_id, &runs)
    }

    /// A research setting or key was saved: switching `auto` off cancels queued `auto` and
    /// `follow_up` runs (R-IA25); a running run finishes.
    pub fn settings_changed(&self) {
        if let Err(error) = self.settings_changed_inner() {
            tracing::warn!(%error, "research: settings change not evaluated");
        }
    }

    fn settings_changed_inner(&self) -> Result<()> {
        if self.settings().auto {
            return Ok(());
        }
        let now = self.clock.now_ms();
        let cancelled = self.store.research_transaction(|tx| {
            let mut sessions = Vec::new();
            for run in store::runs_in_state(tx.conn(), RunState::Queued)? {
                if run.trigger != RunTrigger::Manual {
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
                self.insert_queued(item_id, session_id, RunTrigger::Manual)?;
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
        let any_running = !self
            .store
            .research_transaction(|tx| store::runs_in_state(tx.conn(), RunState::Running))?
            .is_empty();
        let photos = self.session_photos(run.session_id, run.item_id)?;
        Ok(may_start(&StartInput {
            trigger: run.trigger,
            blocked: first_failing(&input).is_some_and(|p| p.blocks_creation()),
            any_running,
            day_limit_reached: input.day_limit_reached,
            since_change_ms: self.since_change_ms(run.session_id)?,
            quiet_ms: settings.quiet_seconds.saturating_mul(1_000),
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
        let config = ResearchConfig::from_settings(settings, &facts);
        let form = form_snapshot(
            &self.store,
            self.clock.as_ref(),
            self.integration.as_deref(),
            run.item_id,
        )?;
        let input = self.input_now(run.session_id, run.item_id)?;
        let text = |value: serde_json::Result<String>| {
            value.map_err(|e| Error::InvalidData(e.to_string()))
        };
        let (config, form, input) = (
            text(serde_json::to_string(&config))?,
            text(serde_json::to_string(&form))?,
            text(serde_json::to_string(&input))?,
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
        })
    }

    /// MC-A6's follow-up row for a session that has runs: when the photo-derived inputs differ
    /// from the latest started run's snapshot (MC-A7 trigger set), a queued run is kept (its
    /// start moves with the quiet period); otherwise one `follow_up` run is queued, also beside
    /// a running run.
    fn follow_up(&self, session_id: i64, item_id: i64, runs: &[RunRow]) -> Result<()> {
        if runs.iter().any(|r| r.state == RunState::Queued) {
            return Ok(());
        }
        let latest: InputSnapshot = runs
            .iter()
            .rev()
            .find_map(|r| r.input_snapshot.as_deref())
            .and_then(|text| serde_json::from_str(text).ok())
            .unwrap_or_default();
        let now = self.input_now(session_id, item_id)?;
        if follow_up_needed(&latest, &now) {
            self.insert_queued(item_id, session_id, RunTrigger::FollowUp)?;
        }
        Ok(())
    }

    /// The store, as the port the app's services take.
    pub fn store(&self) -> &dyn Store {
        self.store.as_ref()
    }
}
