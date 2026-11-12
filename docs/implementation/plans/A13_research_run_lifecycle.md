---
phase: A13_research_run_lifecycle
title: When research runs — automatically after the first approved photo once the session has been quiet, on "Run again", one run at a time, never past the day limit, waiting when offline or when Capture One has not delivered a preview, and picking up again after the app was closed
difficulty: high
projection: yes
depends_on: [A12_research_stages_and_decide]
---
# A13 — Run rows, prerequisites, triggers, quiet period, readiness, start order, day limit, offline, the job (08B MC-A6; MA5, MA7, MA13)

## Goal
The life of a run around A12's `execute`: when a run row is created, when it may start, in which order, what the
panel is told when it cannot, and how the work is carried by the job queue on its own runner. Add the `research`
settings to the settings file and build the service into `App`. **Not in this phase:** follow-up triggers and
resume inside a run (A14), commands and UI (A18, A19), real adapters (the app has none yet, so prerequisite
`NoModelKey` holds in a real run until A15–A19).

## Read first
`master_plan_research.md` §3, §6.3, §6.5, §6.6 (settings, panel sentences), §7.5 · book `01` (H4, H6), `14` (MA5,
MA7, MA13), `08A` §2 (HA4), §5 (Start, Quiet period), §9, §10, `08B` **MC-A6** (all), MC-A8 (the day), MC-A7
(Resume: only "the job is re-queued" is used here), `05A` MC-4 (monotonic clock, local dates), MC-6 (`change_seq`)
· code `crates/infra/src/jobs.rs`, `crates/integrations/src/lib.rs` (`App`, `AppHandles`, runners at 694–696,
background threads), `crates/infra/src/settings_file.rs`, `crates/core/src/services/{settings.rs, arrival.rs,
review.rs, previews.rs}` (where a session's photos change), `crates/core/src/ports/{store.rs
(requeue_running_jobs, job_due), clock.rs, secrets.rs}`, `crates/testkit/src/{test_app.rs, clock.rs}`.

## Dependencies and gate
Tracker row and gate: `master_plan_research.md` §4. Notes: `backlog_research.md`.

## Files expected to change
New: `crates/core/src/services/research/runs.rs`, `crates/core/src/mechanisms/research/run_rules.rs`,
`crates/integrations/src/research/{mod.rs, job.rs}`, `crates/testkit/tests/suite/research/
a13_research_run_lifecycle.rs` (+ `mod` line).
Edited: `crates/core/src/services/research/mod.rs`, `crates/core/src/events.rs` (`ResearchChanged`),
`crates/infra/src/settings_file.rs` (`ResearchSettings`), `crates/integrations/src/lib.rs` (service, runner
`research_jobs`, its thread, `excluding_kinds`), the services that change a session's photos (one call each to
`ResearchService::session_changed`), `crates/core/src/ports/store.rs` and `crates/infra/src/sqlite/store.rs`,
`crates/testkit/src/test_app.rs` (`research()`, `research_tick`), own tracker row.

## Core mechanisms (unit tests allowed)
`run_rules::{first_failing, may_start, start_order}`.

## Ordered tasks
1. `ResearchSettings` (keys and defaults in `master_plan_research.md` §6.6, **including the two default attribution
   keys**: a run must never class the designer or manufacturer field as a visible field) with tolerant reading of older files;
   `ResearchConfig::from(settings, shop facts)` builds the snapshot.
2. `first_failing(&PrerequisiteInput)`: MC-A6's order — no integration; no model key; no visual-search key; a
   configured provider or model missing from the price table; no layout field research may write; day limit
   reached; offline; `auto` off. While any of the first five fails, no run row is created.
   `ResearchService::status(session_id)` returns the first failing prerequisite (or the waiting reason), the latest
   run and its steps (A18 turns it into a command).
3. `session_changed(session_id)` (called on a photo row created, a classification, position or composition change,
   and a research source becoming available) and `settings_changed()` evaluate eligibility and apply MC-A6's event
   table for the `auto` trigger; `run_again(session_id)` applies the **Run again** row; switching `auto` off cancels
   queued `auto` and `follow_up` runs. The Unassigned session is never researched.
4. Start: `may_start` — quiet period on the monotonic clock since the session's last change (at launch: the launch
   instant; `manual` skips it); readiness (an eligible photo's source missing → wait up to `source_wait_seconds`,
   then start without it; no eligible photo has a source → stay `queued`, reason `WaitingForEditor`); today's spend
   below the day cap; no run `running` app-wide. `start_order`: `manual` first, then `queued_at`, then id. The
   start transaction sets `running`, `started_at`, the settings snapshot and the form snapshot, and enqueues one
   `jobs` row of kind `research` whose target is the run id.
5. `ResearchJob` (`JobHandler`, in integrations) calls `execute`; `HoldOffline` → the job is retried with the jobs
   back-off capped at 60 s (`max_backoff_ms`), no attempt counted, status reason `Offline`. Runner
   `App.research_jobs = JobRunner::new(store).only_kind("research")` on its own thread; the main runner excludes
   the kind. At launch `requeue_running_jobs` re-queues the job of a `running` run.
6. `AppEvent::ResearchChanged {session_id}` on every run state change and committed step.

## Criteria (8 criteria, 24 rows)
Boundary: `TestApp::with_research()`, photo and settings changes through the existing services, `research_tick`,
`ResearchService::{status, run_again}`; outcome = `research_runs` rows, the status, the fakes' request logs.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | prerequisites | each of the eight conditions failing alone, in turn | `status` names exactly that one (sentence of §6.6); for the first five, no run row exists after a first Main photo | MA7, MC-A6 |
| C1(b) | order | no model key **and** day limit reached; offline **and** `auto` off | `NoModelKey`; `Offline` | MC-A6 |
| C1(c) | core workflow | no integration and no key; a session is shot, reviewed and exported | the export completes as in `p09_local_export`; no research row, no provider request | MA7 (HA4) |
| C1(d) | default settings | `ResearchSettings::default()` and the shop definitions fixture (`sv1_definitions_p1.json`) | `attribution_keys` is not empty; each key is a definition key of the fixture; `field_class` of each is attribution | MA2 (HA2) |
| C2(a) | automatic start | `auto` on; a session gets its first Main photo | one run `queued`, trigger `auto` | MC-A6 |
| C2(b) | once | a second and a third photo arrive while it is queued | still one run row | MA13 |
| C2(c) | Unassigned | photos arrive in the Unassigned session | no run row | MC-A6 (R-IA23) |
| C3(a) | Run again | no run queued or running | one `queued`, trigger `manual` | MC-A6 |
| C3(b) | Run again | a run queued with trigger `auto` | the same row, trigger now `manual` | MC-A6 |
| C3(c) | Run again | a run is running | one more row `queued`, `manual`; pressed twice: still one queued | MA13 |
| C4(a) | `auto` switched off | one run queued (`auto`), another session's run running | the queued run is `cancelled`; the running one finishes | MC-A6 (R-IA25) |
| C4(b) | `auto` off | then a new session gets a Main photo; then Run again | no run row; then one `queued` `manual` that starts | MC-A6 |
| C5(a) | quiet period | `quiet_seconds` 5; a photo arrives at t; another change at t + 3 s | not started at t + 7.9 s; started at t + 8 s (manual clock) | MA13 |
| C5(b) | launch | a queued run exists at launch | not started before launch + `quiet_seconds` | MA13 |
| C5(c) | manual | Run again inside the quiet period | starts at the next tick | MC-A6 |
| C6(a) | readiness | two eligible photos, one without an adjusted preview; `source_wait_seconds` 120 | not started at 119 s; started at 120 s, observing one photo | MC-A6 (R-IA14) |
| C6(b) | readiness | no eligible photo has an adjusted preview after the wait | still `queued`; status `Waiting for Capture One`; no provider request | MC-A6 |
| C7(a) | one at a time | two sessions with queued runs | never two rows `running`; the second starts after the first ends | MA13 |
| C7(b) | order | queued: `auto` at t, `manual` at t + 1 | the `manual` run starts first | MC-A6 (R-IA15) |
| C7(c) | start transaction | a run starts; a trusted site is then removed | the run's `settings_snapshot` still holds it; `form_snapshot` lists the layout fields with their values at start | MC-A6 |
| C8(a) | day limit | today's calls already sum to the day cap; a run is queued | stays `queued`; status `Day limit reached`; it starts after the manual clock passes local midnight | MA13 (R-IA16) |
| C8(b) | offline | the first call fails with `sent: false`, twice, then succeeds | the run stays `running`; status `Offline` meanwhile; retries no more than 60 s apart; attempts not counted | MC-A6 |
| C8(c) | quit and launch | `kill_and_restart` while a run is `running` | one job of kind `research` is due again; the run ends `finished` | MA4 |
| C8(d) | own runner | a research job blocked in a fake call (hook) while a publish job is due | the publish job completes | MC-A6 |

## Delegated choices
Where `session_changed` is called from (the smallest set of call sites that covers the listed changes; list them
in the handoff) · how "offline" is known for `first_failing` (the last call's outcome; no separate network probe) ·
thread wake-up interval of the research runner. Not delegated: the prerequisite order and sentences.

## Manual try-out
On scratch data (`PSM_DATA_DIR=$HOME/psm-research-dev`), with a shop connected: shoot or import one photo. Nothing
visible changes yet; research stays off because no model key can be entered before A19. The app's other functions
behave as before, which is this phase's promise to check.

## Notes
- Rule 10: the settings defaults leave models and prices empty until A24, so a real run cannot start by default
  yet; A24 carries the reachability row.
- Hazard (P70): no real-time waits in tests; every wait above is on the manual clock.
- Hazard: `session_changed` must not run inside another service's transaction.
- Review mode: this phase's module, `research::a12…::`, `p16_operations::`, `p21_backend_robustness::`,
  `p09_local_export::`, clippy, fmt.

## Review log
