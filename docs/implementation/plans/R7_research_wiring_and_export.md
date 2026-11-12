---
phase: R7_research_wiring_and_export
title: Research switched on in the app — the engine is connected to the app's photos, settings, job queue and screens, so the first approved photo starts a run you can watch; the research record is kept for a set number of days and can be exported as one zipped folder with never a key, a RAW or a deleted photo in it
difficulty: high
projection: no
depends_on: [R4_research_engine, R6_research_ui]
---
# R7 — Wiring and export: the engine joins the app; the research record is kept and exported

**Tracker:** NOT_STARTED · 2026-11-12 · planner · —

(This line is the milestone's tracker row. Sessions change only this line of this file: state, date, actor, one-line note. The gate for `depends_on` is the Tracker line of each plan named there. `master_plan.md` §4 is not used for research milestones.)

## Goal
(U1) The one place the app learns about the engine: `App` builds the `ResearchService`, runs its job on a runner of its own, calls `session_changed` where a session's photos change, saves research settings through the engine's validation, and hands the service to the commands frozen in R2. (U2) Retention of call records and Export research (MC-A11), with the button in Settings. With the crate's fakes nothing real is called yet; real research needs R8's adapters and the keys. **Not here:** real adapters (R8), the tool (R9).

## How to work this plan (flow.md §10)
This milestone is one runner phase made of 2 units. Work the units in order. After each unit run its **Unit test commands**, then make a `CHECKPOINT (not approved):` commit of that unit's paths. One review, one fix budget and one narrative cover the whole milestone. Row ids keep the number of the superseded plan they were written in (`A07·C1(a)`); `docs/implementation/research_row_mapping.md` says where every row went. Name each test after its row (`a07_c1a_<slug>`). Reviewer notes go to `docs/implementation/backlog_research/R7_research_wiring_and_export.md`.

| Unit | What it builds | Criteria | Rows | Changes app files |
|---|---|---|---|---|
| U1 | Wiring the engine into the app | 3 | 4 | yes |
| U2 | Retention of call records and Export research | 7 | 13 | yes |
| **Total** | | **10** | **17** | |

## Read first (whole milestone)
`docs/process/flow.md` (**§10 is the authority for this track**) · `docs/implementation/master_plan.md` §3, §6, §7 · `docs/implementation/master_plan_research.md` (all: layout §0, standing rules §3, sequencing and tracks §5, registry §6, commands §7.2, the storage decision §6.4) · book `01`, `14` §Research agent, `08A`, `08B` · then each unit's own Read first. Name key and sameness are `psm_core::mechanisms::{name_key, sameness}` (R2·U1); every other research rule is `psm_research::rules`.

## Carried from earlier reviews (orchestrator, 2026-11-12; wins over older wording below)
- **U1, one running run app-wide (R4 review).** `ResearchService::tick` reads "nothing running" and starts a run in two separate transactions; the database enforces one running run per session only. The wiring must call `tick` from exactly one place (the job runner thread); commands only enqueue or wake it. Add a row: two concurrent wake-ups never leave two runs `running` (MA13).
- **U1, the item's own listing (R4 review).** `ResearchService::external_id` reads `item_links … ORDER BY integration LIMIT 1`; read the configured integration's link instead.
- **U1, a field in both Details and Product (R3 review).** `writes::layout_step` returns the first step (Details, Condition, Product) that lists the key, so a key placed in both the Details and the Product layout is written. MC-A1 says a `product`-step field is never written: fail closed, a key is not writable when **any** step that lists it is Product. Add a row with such a layout.
- **U2, the redaction test (R4 review).** `gateway::a11_c4a_secrets_are_redacted_in_the_record` hard-codes `research_calls`' 19 columns in a `UNION ALL`. When retention adds a column, change that test to a per-table text scan rather than adding the column to the list.
- **U1, a capped run lowers nothing (owner, 2026-11-12; 08B changelog 7).** `decide.rs` (the `not_determined` loop)
  lowers an earlier `ai` + `suggested` row to `low` "No longer supported" even when the run ends `stopped_at_cap`.
  Change it: a run whose ending is `stopped_at_cap` lowers nothing. Add a row in `crates/research/tests/engine/`: a
  first run writes a medium suggestion; a follow-up with a smaller cap stops before reading its pages → the row is
  unchanged (same value, confidence, reason, sources). The existing lowering row for a completed run still holds.

## Unit 1 — Wiring the engine into the app

**Unit id:** `R7·U1` · **Replaces:** A13 rows C1(c), C2(a), C8(c), C8(d) · **Changes app files:** yes

### Goal
Connect the engine built in R4 to the app, as thinly as possible, and prove the four things only the wired app can show: the core workflow is untouched without research, a first Main photo queues a run through the app's own services, a running run continues after the app is restarted, and research has its own job runner.

### Read first
`master_plan_research.md` §5 (seams), §6.6 · book `01` (H6, H8), `08A` §2 (HA4), `08B` MC-A6 (Quit and launch; the dedicated runner) · code `crates/integrations/src/lib.rs` (`App`, `AppHandles`, runners at lines 694–696, background threads, `settings_set`), `crates/core/src/services/{arrival.rs, review.rs, previews.rs, composition.rs, photos.rs}` (where a session's photos change), `crates/research/src/{service.rs, runs.rs, job.rs, schema.rs, shop.rs, adapters/mod.rs, api.rs}`, `apps/desktop/src-tauri/src/{state.rs, commands/research.rs}`, `crates/testkit/src/test_app.rs`.

### Files
Edited: `crates/integrations/{Cargo.toml, src/lib.rs}` (`psm-research` dependency; `schema::ensure`; the service with `CommerceShopFacts`, the secrets port, `secret_values`, `adapters::build`; `research_jobs = JobRunner::new(store).only_kind("research")` on its own thread; the main runner's `excluding_kinds` gains `"research"`; `App.research`, `AppHandles.research`), the smallest set of call sites that tell the engine a session's photos changed (an observer registered once is preferred over calls spread through core services; list the sites in the handoff), `settings_set` (a `research` patch is validated by `ResearchApi::validate_settings`, then `settings_changed`), `crates/testkit/src/test_app.rs` (`TestApp::with_research()`: the wired app with `psm_research::fakes` registered as providers, `FakeSecrets` filled, research settings and a price table set), test fixtures that list tables. New: `crates/testkit/tests/suite/research/r7_wiring.rs` (+ `mod` line).

### Core mechanisms (the only functions with unit tests)
None.

### Ordered tasks
1. `App::open…`: after the store opens, run `psm_research::schema::ensure`; build `ResearchService` from the settings' `research` slot, the store, the clock, the imaging, the secrets port, `CommerceShopFacts` over `App.commerce`, the event sink, and `adapters::build`; set `App.research = Some(…)`. The commands of R2·U2 now reach it.
2. The research runner and its thread; `requeue_running_jobs` at launch already re-queues a `running` run's job (MC-A6).
3. `session_changed(session_id)` on: a photo row created, a classification, position or composition change, a research source (adjusted preview) becoming available. Never inside another service's transaction.
4. Settings: a `research` patch passes `validate_settings`; a successful save calls `settings_changed`; the three site commands save the value the API returns.
5. `TestApp::with_research()` for this and later app-suite tests.

### Criteria (3 criteria, 4 rows)
Boundary: the wired `TestApp::with_research()` (the real `App` start-up path), photo changes through the app's existing services, `TestApp::tick`, `kill_and_restart`. In the superseded plan these four rows sat with the lifecycle rows; here their boundary is the app, which is what they are about.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| A13·C1(c) | core workflow | no integration and no key; a session is shot, reviewed and exported | the export completes as in `p09_local_export`; no research row, no provider request | MA7 (HA4) |
| A13·C2(a) | automatic start | `auto` on; a session gets its first Main photo | one run `queued`, trigger `auto` | MC-A6 |
| A13·C8(c) | quit and launch | `kill_and_restart` while a run is `running` | one job of kind `research` is due again; the run ends `finished` | MA4 |
| A13·C8(d) | own runner | a research job blocked in a fake call (hook) while a publish job is due | the publish job completes | MC-A6 |

### Delegated choices
Observer or call sites for `session_changed` (list them) · thread wake-up interval of the research runner · how `TestApp::tick` ticks the research runner.

### Notes
- Hazard (P70): no real-time waits in tests; every wait is on the manual clock.
- H8: `crates/integrations/src/lib.rs` only constructs and connects; no research rule is written there.
- `p01_storage_identity`'s table list grows by the engine's five tables once `schema::ensure` runs at open.

### Unit test commands (run after the unit, then make the `CHECKPOINT (not approved):` commit)
`cargo test -p psm-testkit --test suite research::r7_wiring::` · `cargo test -p psm-testkit --test suite p16_operations::` · `cargo test -p psm-testkit --test suite p21_backend_robustness::` · `cargo test -p psm-testkit --test suite p09_local_export::` · `cargo test -p psm-testkit --test suite p01_storage_identity::` · `cargo check --workspace`

## Unit 2 — Retention of call records and Export research

**Unit id:** `R7·U2` · **Replaces:** A21 (all rows) · **Changes app files:** yes

### Goal
Remove call records and the image files only they reference once they are older than `record_days`; keep runs,
steps, feedback, outbound rows and stage outputs. Settings → Research → **Export research**, for a date range or
chosen items, writes the MC-A11 folder and zips it, leaving out the images of photos that are deleted at export
time and aborting if any file contains a secret. **Not in this phase:** the report and replay tool (R9·U1); the
only-if-cheap automatic export folder (not built).

### What changed from the superseded plan (this wins over any older wording below)
- The logic is in the research crate (`crates/research/src/{retention.rs, export.rs}`) and its rows are tested there on the harness (`crates/research/tests/engine/export.rs`): A21·C1(a)–(c), C2, C3, C4, C5, C6.
- Two rows are about the app and are tested in the app: A21·C1(d) (the purge runs at launch and hourly without an explicit call: `crates/testkit/tests/suite/research/r7_export_schedule.rs`, on `TestApp::with_research()`) and A21·C7(a) (the Settings button, a UI test).
- `ResearchApi::export` is implemented here; the command `research_export` exists since R2·U2.

### Read first
`master_plan.md` §6.3 (data dir) · `master_plan_research.md` §3 (keys), §6.4, §6.6 · book `01` (H2), `14` (MA9,
MA16), `08A` §10A, `08B` **MC-A11** (all but Replay identity), MC-A9 (record fields), MC-A5 (stored copies), MC-A7
(reuse needs the files), `10F` (the retention mechanism's elapsed-time rule) · code
`crates/core/src/mechanisms/retention.rs`, `crates/core/src/services/{retention.rs, storage.rs}` (launch and hourly
checks), `crates/infra/src/{research_files.rs, export_fs.rs}`, `crates/integrations/src/lib.rs` (background
threads), `crates/testkit/Cargo.toml` (`zip`), `crates/research/src/gateway.rs` (`secret_values`),
`apps/desktop/src/features/settings/ResearchSection.tsx` (R6·U2).

### Files
New: `crates/research/src/{retention.rs, export.rs}`, `crates/research/tests/engine/export.rs`, `crates/testkit/tests/suite/research/r7_export_schedule.rs` (+ `mod` line), `apps/desktop/src/features/settings/__tests__/r7_research_export.test.tsx`. Edited: `crates/research/src/{lib.rs, api_impl.rs, store.rs, files.rs}`, `crates/research/Cargo.toml` (`zip`), `crates/integrations/src/lib.rs` (the hourly call), `apps/desktop/src/features/settings/ResearchSection.tsx` (the Export button, range and item choice), `apps/desktop/src/services/research.ts`.

### Core mechanisms (the only functions with unit tests)
None new (the expiry test is `now ≥ started_at + record_days × 86 400 000`, proven at the service in C1).

### Ordered tasks
1. `ResearchService::purge_expired(now_ms)`: delete each `research_calls` row whose `started_at + record_days ×
   86 400 000 ≤ now`, and every image file no remaining call references. Run at launch and hourly.
2. `ResearchService::export(selection, dest) -> ExportResult`: selection = runs whose `started_at` local date is in
   `[from, to]` inclusive, or all runs of chosen items. Write `research-export-<local YYYYMMDD-HHMMSS>/<item id>-
   <folder key>/run-<id>/{run.json, steps.json, writes.json, feedback.json, calls/<call id>.json, images/<sha256>.jpg}`
   (`run.json`: the run row with its settings and form snapshots; `writes.json`: the stage 7 output; `feedback.json`:
   the item's feedback rows at or after the run's start). A run whose calls expired gets `"call record expired"` in
   `run.json`.
3. Exclusions at export time: the image of a photo that is now `deleted`, or whose row is gone, is not written, and
   every request that referenced it carries `"withheld": "photo deleted"` in place of the image reference.
4. Secret check, fail closed: before zipping, scan every written file for every value of `secret_values()`, plain
   and URL-encoded. A hit deletes the partial folder, writes no zip, and returns the file's name (never the value).
5. Zip the folder; only `.json` and `.jpg` files exist in it.
6. Command `research_export({from, to} | {item_ids}, dest_folder) -> path`; the Settings button with a date range
   or an item choice and a folder picker.

### Criteria (7 criteria, 13 rows, 1 `[rule]`)
Boundary: `ResearchService::{purge_expired, export}` on the research harness after scripted runs (A21·C1(d) on the wired the research test harness (`Harness::new()`), A21·C7(a) a UI test); outcome = table rows, files on disk, the zip's entries.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| A21·C1(a) | retention | `record_days` 30; a call started exactly 30 × 86 400 000 ms ago; another 1 ms younger | the first is deleted, the second kept | MA16 |
| A21·C1(b) | retention, files | two expired calls share one image with a call that has not expired; one image is referenced only by expired calls | the shared file stays; the other is deleted | MC-A11 |
| A21·C1(c) | retention, kept | after the purge | the runs, steps, feedback rows, outbound rows and stage outputs are all still there | MA16 (R-IA26) |
| A21·C1(d) | schedule | launch with an expired call; then the manual clock advances one hour past another call's expiry | each is purged without an explicit call | MC-A11 |
| A21·C2(a) | selection by date | runs started at local 2026-11-10 23:59 and 2026-11-11 00:01; range 2026-11-11 to 2026-11-11 | only the second run is exported | MA9 |
| A21·C2(b) | selection by items | two items with runs; one chosen | every run of that item, none of the other | MA9 |
| A21·C3(a) | layout | one finished run with 6 calls and 3 sent images | exactly: `run.json`, `steps.json`, `writes.json`, `feedback.json`, six files in `calls/`, three in `images/` named by SHA-256, under `<item id>-<folder key>/run-<id>/`; the zip holds the same entries | MA9 |
| A21·C3(b) | content | the same export, after the photographer accepted one suggestion and edited another | `steps.json` equals the run's steps; each call file holds the recorded request and response byte for byte; `feedback.json` holds the two actions with suggested and final values | MA9 |
| A21·C3(c) | expired | a run whose calls were purged | `run.json` says `call record expired`; no `calls/` entries; steps and writes present | MA9 |
| A21·C4(a) | deleted photo | a photo sent in two calls is classified `deleted` before the export | its image file is absent from the export; both requests carry `"withheld": "photo deleted"`; other images present | MA16 |
| A21·C5(a) | secret check | a call record that contains the model key value (written straight through the store, bypassing the gateway) | the export returns the offending file's name; no folder and no zip remain at the destination; the returned text does not contain the value | MA16 |
| A21·C6(a) | file types | any export | every entry ends in `.json` or `.jpg`; no file is a RAW or a master, and no entry's bytes equal a RAW or master of the session | MA16 (H2) |
| A21·C7(a) | command and button (UI test) | choose a range and a folder, click Export research | `research_export` is called with them; the returned path is shown | `[rule]` 08A §10A |

### Delegated choices
Zip crate use (`zip`, already a dependency of integrations) · where the partial folder is built (a temp folder
beside the destination, renamed at the end) · JSON field names inside `run.json` (they are read by R9·U1; fix them
in `master_plan_research.md` §6 when written). Not delegated: fail closed on a secret.

### Notes
- Hazard: purge must not delete an image a newer run reuses (C1(b)); reuse (R4·U5) already refuses a unit whose file
  is gone.

### Unit test commands (run after the unit, then make the `CHECKPOINT (not approved):` commit)
`cargo test -p psm-research --test engine export::` · `cargo test -p psm-testkit --test suite research::r7_export_schedule::` · `cargo test -p psm-testkit --test suite p61_storage_cleanup_and_budget::` · `cd apps/desktop && npx vitest run src/features/settings` · `npm run typecheck`

## Closing gate (once, after the last unit; replaces the template's closing gate)
This milestone changed app files, so the full gate runs once: `cargo test --workspace` · `cargo clippy --workspace --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cd apps/desktop && npm run typecheck` · `npm test` · `cargo test -p psm-research`. One cargo command at a time.

## Review mode
The reviewer runs: `cargo test -p psm-testkit --test suite research::r7_wiring::`, `research::r7_export_schedule::`, `p16_operations::`, `p21_backend_robustness::`, `p09_local_export::`, `p01_storage_identity::`, `p61_storage_cleanup_and_budget::`, `cargo test -p psm-research --test engine export::`, `npx vitest run src/features/settings`, clippy, fmt, typecheck; the implementer's full-suite stamp stands. Notes: `docs/implementation/backlog_research/R7_research_wiring_and_export.md`.

## Manual try-out (the narrative is built from this)
On scratch data (`PSM_DATA_DIR=$HOME/psm-research-dev`), shop connected:
1. Open a form: the research panel now states why research is not running ("Add the model key in Settings → Research", or that no provider is set until R8).
2. Settings → Research: add a trusted site, reorder, reopen — the order is kept.
3. After R8 and with keys: shoot one photo, open the form, watch steps appear; then Settings → Research → Export research, choose today and the Desktop, unzip and open `steps.json`.
Everything else in the app behaves as before.

## Review log
