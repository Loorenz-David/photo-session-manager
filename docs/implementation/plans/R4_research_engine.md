---
phase: R4_research_engine
title: Research engine — a complete research run works end to end against stand-in providers; every call is recorded before it is sent, deleted photos never leave, the seven stages fill the form with suggestions and a step trail, runs start and stop by the rules, and a crash loses and repeats nothing
difficulty: high
projection: no
depends_on: [R3_research_rules]
---
# R4 — Engine on fakes: ports, gateway, stages, run lifecycle, resume

**Tracker:** NOT_STARTED · 2026-11-12 · planner · —

(This line is the milestone's tracker row. Sessions change only this line of this file: state, date, actor, one-line note. The gate for `depends_on` is the Tracker line of each plan named there. `master_plan.md` §4 is not used for research milestones.)

## Goal
The whole engine inside `crates/research`, proven against the crate's own fakes: provider ports and the `ResearchImage` guard, the engine's tables, the call gateway, stages 1–7 with Decide, the run lifecycle and its job handler, kill-and-resume, reuse and follow-ups, and the `ResearchApi` implementation. No real provider, no app file. **Not here:** wiring into the app (R7), real adapters (R8), the photographer's actions and the publish hold (R5).

## How to work this plan (flow.md §10)
This milestone is one runner phase made of 6 units. Work the units in order. After each unit run its **Unit test commands**, then make a `CHECKPOINT (not approved):` commit of that unit's paths. One review, one fix budget and one narrative cover the whole milestone. Row ids keep the number of the superseded plan they were written in (`A07·C1(a)`); `docs/implementation/research_row_mapping.md` says where every row went. Name each test after its row (`a07_c1a_<slug>`). Reviewer notes go to `docs/implementation/backlog_research/R4_research_engine.md`.

| Unit | What it builds | Criteria | Rows | Changes app files |
|---|---|---|---|---|
| U1 | Ports, the research image guard, the engine tables, fakes | 4 | 9 | no |
| U2 | The call gateway | 7 | 18 | no |
| U3 | The seven stages, Decide and its writes | 8 | 24 | no |
| U4 | Run lifecycle: prerequisites, triggers, quiet period, start order, day limit, offline, the job | 8 | 20 | no |
| U5 | Kill-and-resume, reuse, follow-up runs | 6 | 17 | no |
| U6 | The research API: status, Run again, keys, trusted sites, settings validation | 6 | 11 | no |
| **Total** | | **39** | **99** | |

## Read first (whole milestone)
`docs/process/flow.md` (**§10 is the authority for this track**) · `docs/implementation/master_plan.md` §3, §6, §7 · `docs/implementation/master_plan_research.md` (all: layout §0, standing rules §3, sequencing and tracks §5, registry §6, commands §7.2, the storage decision §6.4) · book `01`, `14` §Research agent, `08A`, `08B` · then each unit's own Read first. Name key and sameness are `psm_core::mechanisms::{name_key, sameness}` (R2·U1); every other research rule is `psm_research::rules`.

## Unit 1 — Ports, the research image guard, the engine tables, fakes

**Unit id:** `R4·U1` · **Replaces:** A10 criteria C3–C5; A11 criterion C8 · **Changes app files:** no

### Goal
The three provider ports and their neutral shapes; the `ResearchImage` value that only `ResearchImages` can make from an eligible photo (MC-A5 type guard); the stored copies; the engine's five tables in the app database; and the crate's fakes and test harness that every later unit scripts. **Not in this unit:** recording calls and outbound rows (U2), any real adapter (R8), any run.

### What changed from the superseded plan (this wins over any older wording below)
- The renderer (`Imaging::render_research`) exists since R2·U3 (rows A10·C1, A10·C2 are there). This unit builds, in the research crate: the ports, `ResearchImage`, `ResearchImages` (eligibility + one stored copy per identity, calling the renderer), the file store, and the fakes.
- Fakes are `psm_research::fakes::{FakeModel, FakeVisualSearch, FakePageReader}` (a public module of the research crate, no testkit dependency), not testkit files. The harness is `crates/research/tests/engine/harness.rs` (`Harness::new()`, `Harness::add_main_photo_with_preview`).
- **Engine tables** (new here, from the superseded gateway plan's first task; row A11·C8(a)): `psm_research::schema::ensure(conn)`, idempotent `CREATE TABLE IF NOT EXISTS` for `research_runs`, `research_steps`, `research_calls`, `research_outbound`, `research_stage_outputs` with the two partial unique indexes, in the **app database file**, run through `SqliteStore::research_transaction` by whoever opens the store for research (the harness now, the app in R7·U1). The SQL for these tables lives in `crates/research/src/store.rs` and runs inside `research_transaction`, so it shares the app's transactions (`master_plan_research.md` §6.4).
- The file store is `crates/research/src/files.rs` (`research/images/<sha[0..2]>/<sha>.jpg` under the data dir); `AppPaths` is not edited (the folder is created on first use).

### Read first
`master_plan.md` §6.1, §6.3 (data dir layout), §6.4 (`Imaging`), §6.10 (composition values, pixel rule), §7.5 ·
`master_plan_research.md` §3 (HA5 in code), §6.1, §6.6, §7.5 · book `01` (**H3 as amended**, H7), `14` (MA3, MA12),
`08A` §4, §5 (Photos used), `08B` **MC-A5** (Eligible photo, Source, Type guard), `10D` MC-27 (pixel rule), MC-28
(render contract) · code `crates/core/src/ports/images.rs` (`Imaging::render_output`), `crates/infra/src/imaging/`
(`render.rs`, `linear.rs`, `metadata.rs`), `crates/infra/src/media_store.rs`, `crates/core/src/ports/store.rs`
(`preview_state`, `RawPreviewState.adjusted_preview_path`, `adjusted_gen`), `crates/core/src/domain/composition.rs`,
`crates/testkit/src/{test_app.rs, fixtures/}` (`SyntheticJpeg`, `complete_render_with`).

### Files
New: `crates/research/src/{ports.rs, images.rs, files.rs, fakes.rs, schema.rs, store.rs}`, `crates/research/tests/engine/images_and_storage.rs`. Edited: `crates/research/src/{lib.rs, domain.rs}`, `crates/research/Cargo.toml` (`psm-infra`, `rusqlite` at the workspace's version), `crates/research/tests/engine/{main.rs, harness.rs}`.

### Core mechanisms (the only functions with unit tests)
None new. The pixel rule is `psm_core::mechanisms::composition` (existing, already tested).

### Ordered tasks
1. Ports and shapes as in `master_plan_research.md` §6.6 (`psm_research::ports`). All strings naming a model, an effort or a provider are opaque. `Exchange` is what an adapter hands back for the record; `CallFailure` is R3·U4's type.
2. `ResearchImage {photo_id, sha256, long_edge, path, bytes_len, source_gen, composition_rev}` with **private fields and no public constructor**; `ResearchImages::render(photo_id, run_id, long_edge) -> Result<ResearchImage, NoSource>` is the only maker.
3. `ResearchImages` (`images.rs`): read the photo row; it is **eligible** iff its session is the run's session, that session's item is the run's item, and its classification is `main`, `condition` or `unreviewed`; take the adjusted preview of its current `adjusted_gen` (none → `NoSource`; the original camera preview is never used); call `Imaging::render_research` (R2·U3) with the current composition values; store the copy at `research/images/<sha[0..2]>/<sha>.jpg` (`files.rs`). Within one run the same `(photo, source generation, composition revision, long edge)` returns the stored copy.
4. `schema::ensure` and `store.rs` (the engine's five tables; see the changes above). Store functions take the connection of a `SqliteStore::research_transaction`.
5. Fakes (`fakes.rs`): `FakeModel`, `FakeVisualSearch`, `FakePageReader` — replies scripted per stage and unit, a log of every request received (each image by SHA-256 and long edge), a queue of `CallFailure` faults, and a `before_call` hook (a closure run when a call arrives, before the reply) used by later units to observe the database and to stop a run at a chosen call.
6. The harness (`tests/engine/harness.rs`): `Harness::new()` (a `TestApp` with a fake shop, `schema::ensure` run, the fakes, `FakeSecrets` holding distinctive key values that appear in no expected text) and `Harness::add_main_photo_with_preview(session)`.

### Criteria (4 criteria, 9 rows)
Boundary: `ResearchImages::render` on the harness (real store, real infra imaging, temp data dir) and, for A11·C8(a), the database after `schema::ensure`.
Boundary: `CallGateway::call` on the research test harness (`Harness::new()`) with a run row inserted in state `running`; outcome =
the rows of `research_calls`, `research_outbound`, `research_runs`, and the fakes' request logs.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| A10·C3(a) | eligibility | a photo classified `deleted` | `NoSource`/refused; no file written | MA3 |
| A10·C3(b) | eligibility | a photo moved to another session after the run started | refused; no file written | MA12 |
| A10·C3(c) | eligibility | `main`, `condition`, `unreviewed` photos of the run's session | each rendered | MC-A5 |
| A10·C3(d) | source | an eligible photo with an original camera preview and no adjusted preview | `NoSource`; no file written | MC-A5 (card 3) |
| A10·C3(e) | type guard | code outside the port's implementation that builds a `ResearchImage` from a path | does not compile (one `compile_fail` doc test on the type) | MA12 |
| A10·C4(a) | one copy | the same photo rendered twice in one run at 1024 | the same SHA-256 and path; one file | MC-A5 |
| A10·C4(b) | new copy | the photo recomposed (revision + 1), rendered again | a different SHA-256; both files present | MC-A5 |
| A10·C5(a) | `FakeModel` | a scripted reply for (observe, photo 7) and a queued transient fault; two calls with a rendered image | the first call returns the fault, the second the scripted reply; the log holds two requests, each naming the image's SHA-256 and long edge 1024 | MA12 (the record R4·U2's rows read what was sent from) |
| A11·C8(a) | migration | insert a second `running` run for one session; a second `queued` run for one session | each insert is refused by the database | MA13 |

### Delegated choices
Whether the eligibility check lives in a core function called by the infra renderer or in a small core service ·
the JPEG encoder call (`jpeg-encoder` as P29) · chroma subsampling · the hook's exact signature. Not delegated: no
public constructor, no path or byte constructor, never the original preview, never the master.

### Notes
- H3: this is the only new pixel code of the research work, and it is the one the owner allowed (card 3). It must
  not decode a RAW or a master, and nothing but a research copy may be produced by it.
- Rule 17: "no EXIF" is observed with `kamadak-exif` (a dev-dependency of the testkit); the projection confirms
  what that reader returns for a JPEG without an APP1 segment before C2(a) is implemented.
- Rule 11: the guard is a property of the port's type, not of a caller's discipline; R4·U2's gateway accepts only
  `ResearchImage` values.

### Unit test commands (run after the unit, then make the `CHECKPOINT (not approved):` commit)
`cargo test -p psm-research --test engine images_and_storage::` · `cargo test -p psm-research --doc` (the one `compile_fail` guard) · `cargo clippy -p psm-research --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cargo check --workspace`

## Unit 2 — The call gateway

**Unit id:** `R4·U2` · **Replaces:** A11 criteria C1–C7 · **Changes app files:** no

### Goal
One function, `CallGateway::call`, through which every provider call of a run passes. In one transaction it
re-reads photo eligibility, applies the stop rule, inserts the call record (`sent`) and one outbound row per photo;
only then it sends; then it records the reply, the usage and the cost, or the failure and the next action. Add the
research run tables. **Not in this phase:** who creates and starts runs (R4·U4), the stages (R4·U3), real adapters.
(`depends_on` names R5·U2 only for the migration order, `master_plan_research.md` §6.1.)

### What changed from the superseded plan (this wins over any older wording below)
- Task 1 of the older list (the tables) is done in U1 by `schema::ensure`; there is no numbered migration.
- `secret_values()` is given to the engine as a function (`Arc<dyn Fn() -> Vec<String>>`): the harness supplies the `FakeSecrets` values plus a shop secret; the app supplies the real ones in R7·U1.
- Run rows for these tests are inserted through `psm_research::store` (it has a production caller in U4).

### Read first
`master_plan_research.md` §3, §6.3–§6.6, §7.4, §7.5 · book `14` (MA3, MA5, MA12, MA15, MA16), `08A` §2 (HA3, HA6),
§10, §10A, `08B` **MC-A5** (Order per call), **MC-A9** (all), **MC-A8** (Call cost, Stop rule, One call in flight),
MC-A6 (Offline) · code `crates/research/src/rules/{cost.rs, calls.rs}` (R3·U4), `crates/research/src/ports.rs` and
`crates/research/src/fakes.rs` (R4·U1), `crates/core/src/ports/store.rs` (`transaction`),
`crates/infra/src/sqlite/store.rs`, `crates/testkit/src/{fake_secrets.rs, logs.rs, isolation.rs}`.

### Files
New: `crates/research/src/gateway.rs`, `crates/research/tests/engine/gateway.rs`. Edited: `crates/research/src/{lib.rs, domain.rs, store.rs}`, `crates/research/tests/engine/{main.rs, harness.rs}`.

### Core mechanisms (the only functions with unit tests)
None new (R3·U4's functions are called, not re-tested).

### Ordered tasks
1. (Done in U1: the engine's tables.)
2. `CallGateway::call(run_id, stage, unit_key, request) -> GatewayOutcome {Answered(reply), NotDetermined(text),
   SkipToDecide, EndAtCap, HoldOffline, RunFailed {stage, class, text}}` for the three ports.
   Step 1, one transaction: for each `ResearchImage` re-read the photo and drop it when no longer eligible (the
   caller gets the dropped ids for the step detail `photo removed since`); `next_call` (R3·U4) with the run's cost,
   cap and reserve — if not `Call`, return without a record; insert `research_calls` (state `sent`, attempt, the
   request with each image as `{"image_sha256": …}`) and one `research_outbound` row per remaining photo (provider,
   `sent_at`, SHA-256); commit. A call left with no photo although it needs one is not sent.
   Step 2: send. Step 3, one transaction: state `answered` with response, usage, `cost_micro_usd`, times; add the
   cost to `research_runs.cost_micro_usd`. On failure: `classify` and `action` (R3·U4) — `HoldOffline` marks the call
   `not_sent`, cost 0, no attempt counted; `Retry` records `failed`, charges per MC-A8 and re-issues after the wait
   as the next attempt (the wait is driven by the clock the service is given); `FailRun` sets the run `failed` with
   stage and error (`key rejected by <provider>` for class key); `NotDetermined` returns the refusal text.
3. Redaction layer 2: before every insert or update of a call record and before every research log line, `redact`
   with `ResearchService::secret_values()` (the research keys from the secrets port and the integration's secret
   and token); a hit sets `redaction_hit` and logs an error without the value.
4. Third-party images a page reply carries are stored in the research files and referenced by the call record;
   they are never outbound rows.

### Criteria (7 criteria, 18 rows)
Boundary: `CallGateway::call` on the research test harness (`Harness::new()`) with a run row inserted in state `running`; outcome =
the rows of `research_calls`, `research_outbound`, `research_runs`, and the fakes' request logs.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| A11·C1(a) | order | a model call with two photos; the fake's `before_call` hook reads the database | inside the hook: one call row in state `sent` and two outbound rows already committed | MA12 |
| A11·C1(b) | outbound = sent | the same call | the SHA-256 values of the outbound rows equal the images the fake received, exactly | MA3 |
| A11·C2(a) | delete before the gate | two photos; one is classified `deleted` before the call | the fake receives one image; one outbound row; the outcome names the dropped photo | MA12 |
| A11·C2(b) | delete after the gate | the hook deletes a photo while the call is in flight; a second call follows with the same two images | the first call's two outbound rows stand; the second call sends one image and has one outbound row | MA12 |
| A11·C2(c) | nothing left | the only photo is deleted before the call | no request reaches the fake; no call row; no outbound row | MA3 |
| A11·C3(a) | record | an answered call with usage | state `answered`; the stored request has `{"image_sha256"}` in place of each image; response, usage, cost, `started_at`, `finished_at`, duration present | MA9 |
| A11·C3(b) | listing images | a page reply carrying two images | both stored as files and referenced by the call; no outbound row | MC-A5 |
| A11·C4(a) | redaction | the fake's reply body and a fault's error text both contain the model key value (plain, and URL-encoded) | the stored response and error text read `<redacted>`; `redaction_hit` is true; no stored column contains the value | MA16 |
| A11·C4(b) | log | the same call, with the log captured (`in_own_process`) | no log line contains the key value; one error line reports the hit | MA16 |
| A11·C5(a) | attempts | faults: transient, transient, then a reply | three call rows, attempts 1, 2, 3; the second is issued 1 s and the third 2 s later on the manual clock; outcome `Answered` | MC-A9 (R-IA20) |
| A11·C5(b) | attempts | three transient faults | outcome `RunFailed`; the run is `failed`, its error names the stage and the failure | MC-A9 |
| A11·C5(c) | key | a fault with status 401 | one call row; run `failed` with `key rejected by <provider>`; no second request | MC-A9 |
| A11·C5(d) | refusal | a refusal reply | outcome `NotDetermined` with the refusal text; one request; the run stays `running` | MC-A9 |
| A11·C5(e) | offline | a fault with `sent: false` | call state `not_sent`, cost 0; outcome `HoldOffline`; the next call for the same unit has attempt 1 | MA15, MC-A6 |
| A11·C6(a) | cost | a reply reporting usage, with the run's price snapshot | the call's `cost_micro_usd` equals `call_cost` for that usage; the run's cost grew by the same amount in the same commit | MA15 |
| A11·C6(b) | cost | a timeout after send (no usage) | the failed call is charged `fallback_call_usd` | MA15 (R-IA19) |
| A11·C7(a) | stop rule | run cost + reserve = cap; a stage 3 call | outcome `SkipToDecide`; no call row; no request | MA5 |
| A11·C7(b) | stop rule | run cost = cap; a stage 7 call | outcome `EndAtCap`; no call row; no request | MA5 |

### Delegated choices
Internal split of `gateway.rs` · whether waits between attempts are a sleep on an injected clock or a returned
"retry at" the caller honours (the tests need the manual clock to drive them; no real-time wait in a test, flow §4A)
· column encodings of request and response. Not delegated: the order commit → send, and that eligibility and the
stop rule are read in that first transaction.

### Notes
- C4 uses `FakeSecrets`' distinctive key values, which appear in no expected text (`master_plan.md` §7.5 practice).
- Hazard: a retry inside the gateway must not hold a database transaction across the send.

### Unit test commands (run after the unit, then make the `CHECKPOINT (not approved):` commit)
`cargo test -p psm-research --test engine gateway::` · `cargo clippy -p psm-research --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cargo check --workspace`

## Unit 3 — The seven stages, Decide and its writes

**Unit id:** `R4·U3` · **Replaces:** A12 (all rows) · **Changes app files:** no

### Goal
`ResearchService::execute(run_id)`: run the stages of 08A §5 for a run that is already `running`, against the
ports, through the gateway; persist each unit's output with its fingerprint; write the trace steps; decide every
field with R3·U3's functions and write through R3·U5's `writes::suggest` / `writes::lower`; end `finished`, `stopped_at_cap` or
`failed`. All provider replies in this phase are the fakes' scripted replies (our shapes). **Not in this phase:**
creating, queueing and starting runs, the job, settings on disk (R4·U4); resume after a kill, reuse and follow-ups
(R4·U5); real adapters (R8·U1–R8·U3).

### What changed from the superseded plan (this wins over any older wording below)
- Shop facts reach the engine through `psm_research::shop::CommerceShopFacts`, a small reader over core's `CommerceAdapter::public_listing` (frozen in R2·U2); tests set them with `FakeCommerce::set_public_listing`. No port method is added here, and there is no `unit_names` method: unit names come from the definitions (`validations.unit_names`).
- Writes go through `psm_research::writes` (R3·U5).

### Read first
`master_plan_research.md` (all of §6, §7.5) · book `01`, `14` (MA1, MA2, MA5, MA6, MA14), `08A` §5 (all), §6, §8, §9,
§10, `08B` **MC-A7** (Units, Storage, Commit rule), **MC-A3** (reading order, page limit, site of a page, own
listing), **MC-A4**, **MC-A1**, **MC-A2**, **MC-A8** (Stop rule), MC-A5 (image sizes per stage) · code
`crates/research/src/rules/*` (R3·U1–R3·U5), `crates/research/src/gateway.rs` (R4·U2), `crates/research/src/ports.rs` (R4·U1),
`services/attributes.rs` (`writes::suggest`, `writes::lower`, `form_get`, presets), `crates/core/src/ports/commerce.rs`,
`crates/research/src/fakes.rs`, `crates/testkit/src/fake_commerce.rs` (`set_public_listing`, R2·U2).

### Files
New: `crates/research/src/{service.rs, stages.rs, decide.rs, shop.rs}`, `crates/research/tests/engine/stages.rs`. Edited: `crates/research/src/{lib.rs, domain.rs, store.rs}`, `crates/research/tests/engine/{main.rs, harness.rs}`.

### Core mechanisms (the only functions with unit tests)
None new.

### Ordered tasks
1. **Snapshots.** `execute` works only from the run's `settings_snapshot` (`ResearchConfig`: per-stage model, effort,
   image long edge; `max_pages`; caps; price table; trusted sites; shop hosts; own listing; attribution keys;
   language; versions) and `form_snapshot` (item type; the ordered layout fields with key, label, core type,
   choices, description, current value and provenance). This phase's tests write both when they insert the run;
   R4·U4 fills them at start.
2. **Stage units** (MC-A7 table) with `unit_key` and `fingerprint` (R3·U2): 1 observe — one per eligible `main` or
   `unreviewed` photo, output includes the visible reads with `sure` and `shows_marking`; 2 find — one, the cover
   (main position 1); 3 read — one per page in MC-A3 reading order until `max_pages` pages are **obtained** or the
   stop rule says skip: the page reader, then a model call that returns `{stated, value}` per researched key
   (mapped by R3·U1); 4 compare — one per page that has images, the cover against the page's stored images, verdict
   `match` / `similar` / `different`; 5 labels — one per photo with `shows_marking`; 6 condition — one, over the
   `condition` photos (none → no call, every condition field not determined); 7 decide.
3. **Commit rule.** When a unit's last call returns, one transaction commits its `research_stage_outputs` row, its
   steps (`seq` = max of the run + 1, gapless) and its calls' responses. Step kinds: `looked` (1, 5, 6),
   `searched` (2), `read` (3; source URL and title; an unobtainable page is a `read` step `could not be read` and
   does not count; the own listing is a step `this item's own listing` and is not read), `compared` (4),
   `concluded` and `not_determined` (7). Each step names the photo ids it used and the field keys it led to.
4. **Decide.** For every layout field: `field_class`; build the evidence items from the committed outputs (site of
   a page from the reader's final URL when it reports one, else the candidate URL); R3·U3's decision; then one model
   call (stage 7) writes the reason for each decided field in `research.language`; one transaction applies the
   writes through `writes::suggest` (value, **computed** confidence, reason, `sources`) or `writes::lower` (a field decided
   not determined whose row is `ai` + `suggested`), and commits the `concluded` step per written key and one
   `not_determined` step listing every other field with its reason (including `not researched` and `matches a
   value you cleared`). The stage 7 output stores per key the value, confidence, reason, sources and the MC-A1
   outcome.
5. **Endings.** `SkipToDecide` from the gateway → skip the remaining reading units, go to Decide, end
   `stopped_at_cap` with the writes; `EndAtCap` before Decide → `stopped_at_cap`, no write; `RunFailed` → `failed`
   with stage and error, earlier values untouched; a refusal in a unit → that unit's output is "not determined"
   with the refusal text, the run continues; otherwise `finished` with `stage_reached` 7.

### Criteria (8 criteria, 24 rows)
Boundary: `ResearchService::execute` on the research test harness (`Harness::new()`) with scripted fakes and a `running` run row;
outcome = `attribute_values`, `research_steps`, `research_stage_outputs`, `research_runs`, the fakes' request logs.
Shop host `beyo.example`; trusted `[e1.example, e2.example]`; layout: `designer` (text, an attribution key),
`wood` (multi-choice), `width` (dimension), `scratches` (severity choice, Condition step), `title` (Product step).
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| A12·C1(a) | a full run | two main photos, one condition photo; search returns pages on e1, e2 and an untrusted site; e1 and e2 state designer A and are `match`; observe reads wood `Teak`, sure; condition reads `Minor`, sure | `designer` = A, medium, sources = the two pages; `wood` = `["Teak"]`, medium; `scratches` = `Minor`, medium; all `ai` + `suggested`; run `finished`, stage 7 | MA2 |
| A12·C1(b) | steps | the same run | `seq` is 1…n with no gap; kinds in order: `looked` ×2, `searched`, `read` ×3, `compared` (one per page with images), `looked` (condition), `concluded` per written key, one `not_determined`; every written key appears in some step's field keys | MA6 |
| A12·C1(c) | stage outputs | the same run | one row per unit of the MC-A7 table, each with the fingerprint R3·U2 computes for its identity; the stage 7 row lists per key value, confidence, reason, sources, outcome | MA14 |
| A12·C2(a) | reading order | candidates ranked: untrusted 1, e2 2, e1 3, shop (another product) 4 | the page reader is called for shop, e1, e2, untrusted, in that order | MC-A3 |
| A12·C2(b) | page limit | `max_pages` 2; four candidates, the first unobtainable | three reader calls; the first is a `read` step `could not be read`; two pages obtained; the fourth never requested | MC-A3 (R-IA10) |
| A12·C2(c) | own listing | a candidate that is the item's own listing (shop host, the item's path tail) | never read; a step `this item's own listing`; it is no evidence (designer not determined when it was the only page) | MA11 (R-IA9) |
| A12·C2(d) | redirect | the reader reports a final URL on e1 for a candidate on an untrusted host | the page counts as e1 | MC-A3 |
| A12·C3(a) | units and images | two main, one unreviewed, two condition photos; one main photo's observe output has `shows_marking` | observe ×3 at long edge 1024; labels ×1 at 2048; condition ×1 with two images at 2048; find and compare use the cover at 1024 (the model fake's and the search fake's logs) | MC-A7, MC-A5 |
| A12·C3(b) | label | the label unit reads designer A | `designer` = A, high, sources empty | MA2 |
| A12·C3(c) | no condition photo | no photo classified `condition` | no stage 6 call; `scratches` not determined; nothing written for it | MC-A4 |
| A12·C4(a) | not determined | no page states a designer, no label | no `designer` row is created; the `not_determined` step lists it with a reason | MA6 |
| A12·C4(b) | protected values | before the run: `designer` is `human`; `wood` is `ai` + `accepted` | both rows byte-identical after the run; the last step says why | MA1 |
| A12·C4(c) | cleared | `designer` was cleared with value A (feedback row); the run decides A | not written; the step says `matches a value you cleared` | MA1 |
| A12·C4(d) | lowering | `designer` holds `ai` + `suggested` A, medium, from before; this run finds no support | the row is A, `low`, reason starting `No longer supported:` | MA2 |
| A12·C4(e) | never written | `title` (Product step) | untouched; named `not researched` | MA10 |
| A12·C4(f) | measured | `width`: e1 and e2 `match` pages state 120 cm; a photo read offers 118 cm | 120 cm at medium (the photo read is not used) | MC-A4 |
| A12·C5(a) | cap, reserve | cap 0.50, reserve = the stage 7 fallback; scripted usage makes cost + reserve reach the cap after the second page | the remaining read and compare units are skipped; Decide runs; run `stopped_at_cap`; the values supported so far are written | MA5 (card 5) |
| A12·C5(b) | cap reached | the cost equals the cap before Decide | run `stopped_at_cap`; no attribute row written | MA5 |
| A12·C5(c) | bound | C5(a)'s run | run cost ≤ cap + the cost of the one call that crossed it | MA5 |
| A12·C6(a) | failure | the page reader returns three transient faults for the second page | run `failed`, error names stage `read`; rows written by an earlier run are unchanged; no stage 7 output | MC-A9 |
| A12·C6(b) | refusal | the model refuses one observe unit | that unit's output is "not determined" with the refusal text; the run finishes; other fields are written | MC-A9 |
| A12·C7(a) | language and choices | `language` = `Swedish`; `wood` choices `[Oak, Teak]` | every model request carries the language; the read and observe requests carry the choices; the stored `wood` value is the definition's own string | MC-A2 |
| A12·C7(b) | snapshot | the trusted list in the live settings is changed while the run executes (from the fake's hook) | reading order and site classes follow the run's snapshot | MC-A3 |
| A12·C8(a) | confidence is computed | the stage 7 reply claims `"confidence": "high"` for a designer supported by one `similar` page | stored confidence `low` | MA11 |

### Delegated choices
The per-stage output JSON shapes and the request field names (ours) · `unit_key` spellings · step summary wording
except the quoted texts · how reasons are requested (one stage 7 call for all decided fields). Not delegated: one
call per unit kind as in the MC-A7 table, calls strictly sequential, every provider call through the gateway.

### Notes
- Rule 2 companion: each row's fixture gives its outcome exactly one cause (C1(a): exactly two trusted `match`
  pages for the designer; C3(b): a label and no listing).
- The fakes' reply shapes are this repo's port shapes. No row here depends on what OpenAI or a search vendor
  returns; those rows are in R8·U1–R8·U3.
- Rule 10 is served in R10·U1 (shipped defaults reach a run).

### Unit test commands (run after the unit, then make the `CHECKPOINT (not approved):` commit)
`cargo test -p psm-research --test engine stages::` · `cargo clippy -p psm-research --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cargo check --workspace`

## Unit 4 — Run lifecycle: prerequisites, triggers, quiet period, start order, day limit, offline, the job

**Unit id:** `R4·U4` · **Replaces:** A13, all rows except C1(c), C2(a), C8(c), C8(d) (those are in R7·U1) · **Changes app files:** no

### Goal
The life of a run around U3's `execute` (MC-A6): when a run row is created, when it may start, in which order, what the status says when it cannot, and the job handler that carries the work; the `research` settings shape with its defaults. **Not in this unit:** wiring into the app (R7·U1), follow-up triggers and resume inside a run (U5), commands and UI.

### What changed from the superseded plan (this wins over any older wording below)
- Everything is in the research crate: `ResearchSettings` (serde, with the defaults of `master_plan_research.md` §6.6) is `psm_research::config`, read from the `research` slot of the settings file (R2·U2); `ResearchJob` implements `psm_infra::jobs::JobHandler` in `crates/research/src/job.rs`.
- The app is **not** wired here (no `App` field, no thread, no call site in a core service): tests make photo changes through the core services and then call `ResearchService::session_changed(session_id)` themselves, which is the call R7·U1 adds to the app; they drive the job with a `JobRunner::new(store).only_kind("research")` built in the harness (`Harness::tick`).
- The four rows that prove the app's own call sites, launch path and second runner are in R7·U1: A13·C1(c), A13·C2(a), A13·C8(c), A13·C8(d).
- `psm_research::adapters::build(config, secrets) -> Providers` is created here as the one place adapters are chosen by the ids in `research.providers.*`; it knows no adapter yet (an unknown or empty id gives none, which is prerequisite `NoModelKey` / `NoSearchKey`'s neighbour: the status says the provider is not set). R8 adds entries.

### Read first
`master_plan_research.md` §3, §6.3, §6.5, §6.6 (settings, panel sentences), §7.5 · book `01` (H4, H6), `14` (MA5,
MA7, MA13), `08A` §2 (HA4), §5 (Start, Quiet period), §9, §10, `08B` **MC-A6** (all), MC-A8 (the day), MC-A7
(Resume: only "the job is re-queued" is used here), `05A` MC-4 (monotonic clock, local dates), MC-6 (`change_seq`)
· code `crates/infra/src/jobs.rs`, `crates/integrations/src/lib.rs` (`App`, `AppHandles`, runners at 694–696,
background threads), `crates/infra/src/settings_file.rs` (the `research` slot), `crates/core/src/services/{settings.rs, arrival.rs,
review.rs, previews.rs}` (where a session's photos change), `crates/core/src/ports/{store.rs
(requeue_running_jobs, job_due), clock.rs, secrets.rs}`, `crates/testkit/src/{test_app.rs, clock.rs}`.

### Files
New: `crates/research/src/{runs.rs, config.rs, job.rs, rules/run_rules.rs, adapters/mod.rs}`, `crates/research/tests/engine/lifecycle.rs`. Edited: `crates/research/src/{lib.rs, service.rs, store.rs, rules/mod.rs}`, `crates/research/tests/engine/{main.rs, harness.rs}`.

### Core mechanisms (the only functions with unit tests)
`run_rules::{first_failing, may_start, start_order}`.

### Ordered tasks
1. `ResearchSettings` (keys and defaults in `master_plan_research.md` §6.6, **including the two default attribution
   keys**: a run must never class the designer or manufacturer field as a visible field) with tolerant reading of older files;
   `ResearchConfig::from(settings, shop facts)` builds the snapshot.
2. `first_failing(&PrerequisiteInput)`: MC-A6's order — no integration; no model key; no visual-search key; a
   configured provider or model missing from the price table; no layout field research may write; day limit
   reached; offline; `auto` off. While any of the first five fails, no run row is created.
   `ResearchService::status(session_id)` returns the first failing prerequisite (or the waiting reason), the latest
   run and its steps (R6·U1 turns it into a command).
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
5. `ResearchJob` (`JobHandler`, in the research crate) calls `execute`; `HoldOffline` → the job is retried with the jobs
   back-off capped at 60 s (`max_backoff_ms`), no attempt counted, status reason `Offline`. The runner is a `JobRunner::new(store).only_kind("research")` (built by the harness here, by the app in R7·U1). At launch `requeue_running_jobs` re-queues the job of a `running` run.
6. Emit `AppEvent::ResearchChanged {session_id}` (R2·U2) through the event sink on every run state change and committed step.

### Criteria (8 criteria, 20 rows)
Boundary: the harness; photo changes through the core services followed by `ResearchService::session_changed` (the call the app makes from R7·U1 on), settings changes followed by `settings_changed`, `Harness::tick`, `ResearchService::{status, run_again}`; outcome = `research_runs` rows, the status, the fakes' request logs.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| A13·C1(a) | prerequisites | each of the eight conditions failing alone, in turn | `status` names exactly that one (sentence of §6.6); for the first five, no run row exists after a first Main photo | MA7, MC-A6 |
| A13·C1(b) | order | no model key **and** day limit reached; offline **and** `auto` off | `NoModelKey`; `Offline` | MC-A6 |
| A13·C1(d) | default settings | `ResearchSettings::default()` and the shop definitions fixture (`sv1_definitions_p1.json`) | `attribution_keys` is not empty; each key is a definition key of the fixture; `field_class` of each is attribution | MA2 (HA2) |
| A13·C2(b) | once | a second and a third photo arrive while it is queued | still one run row | MA13 |
| A13·C2(c) | Unassigned | photos arrive in the Unassigned session | no run row | MC-A6 (R-IA23) |
| A13·C3(a) | Run again | no run queued or running | one `queued`, trigger `manual` | MC-A6 |
| A13·C3(b) | Run again | a run queued with trigger `auto` | the same row, trigger now `manual` | MC-A6 |
| A13·C3(c) | Run again | a run is running | one more row `queued`, `manual`; pressed twice: still one queued | MA13 |
| A13·C4(a) | `auto` switched off | one run queued (`auto`), another session's run running | the queued run is `cancelled`; the running one finishes | MC-A6 (R-IA25) |
| A13·C4(b) | `auto` off | then a new session gets a Main photo; then Run again | no run row; then one `queued` `manual` that starts | MC-A6 |
| A13·C5(a) | quiet period | `quiet_seconds` 5; a photo arrives at t; another change at t + 3 s | not started at t + 7.9 s; started at t + 8 s (manual clock) | MA13 |
| A13·C5(b) | launch | a queued run exists at launch | not started before launch + `quiet_seconds` | MA13 |
| A13·C5(c) | manual | Run again inside the quiet period | starts at the next tick | MC-A6 |
| A13·C6(a) | readiness | two eligible photos, one without an adjusted preview; `source_wait_seconds` 120 | not started at 119 s; started at 120 s, observing one photo | MC-A6 (R-IA14) |
| A13·C6(b) | readiness | no eligible photo has an adjusted preview after the wait | still `queued`; status `Waiting for Capture One`; no provider request | MC-A6 |
| A13·C7(a) | one at a time | two sessions with queued runs | never two rows `running`; the second starts after the first ends | MA13 |
| A13·C7(b) | order | queued: `auto` at t, `manual` at t + 1 | the `manual` run starts first | MC-A6 (R-IA15) |
| A13·C7(c) | start transaction | a run starts; a trusted site is then removed | the run's `settings_snapshot` still holds it; `form_snapshot` lists the layout fields with their values at start | MC-A6 |
| A13·C8(a) | day limit | today's calls already sum to the day cap; a run is queued | stays `queued`; status `Day limit reached`; it starts after the manual clock passes local midnight | MA13 (R-IA16) |
| A13·C8(b) | offline | the first call fails with `sent: false`, twice, then succeeds | the run stays `running`; status `Offline` meanwhile; retries no more than 60 s apart; attempts not counted | MC-A6 |

### Delegated choices
Where `session_changed` is called from (the smallest set of call sites that covers the listed changes; list them
in the handoff) · how "offline" is known for `first_failing` (the last call's outcome; no separate network probe) ·
thread wake-up interval of the research runner. Not delegated: the prerequisite order and sentences.

### Notes
- Rule 10: the settings defaults leave models and prices empty until R10·U1, so a real run cannot start by default
  yet; R10·U1 carries the reachability row.
- Hazard (P70): no real-time waits in tests; every wait above is on the manual clock.
- Hazard: `session_changed` must not run inside another service's transaction.

### Unit test commands (run after the unit, then make the `CHECKPOINT (not approved):` commit)
`cargo test -p psm-research --test engine lifecycle::` · `cargo test -p psm-research --lib rules::run_rules` · `cargo clippy -p psm-research --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cargo check --workspace`

## Unit 5 — Kill-and-resume, reuse, follow-up runs

**Unit id:** `R4·U5` · **Replaces:** A14 (all rows) · **Changes app files:** no

### Goal
Make a run resumable at any instant with the same end state as an uninterrupted run; copy unchanged units from the
session's latest earlier run in `auto` and `follow_up` runs; queue a follow-up when, and only when, the
photo-derived inputs changed. **Not in this phase:** anything in the UI; retention (R7·U2; here "the file is
present" is the only completeness test).

### What changed from the superseded plan (this wins over any older wording below)
- A kill is `Harness::restart()`: drop the service, the runner and the store, reopen the same data folder, run `schema::ensure`, call `requeue_running_jobs` as the app's launch does.

### Read first
`master_plan_research.md` §6.4, §6.5, §7.5 · book `01` (H4), `14` (MA4, MA14), `08A` §5 (New photos after a run),
§10, `08B` **MC-A7** (all), MC-A6 (the event table's follow-up row; "a run is running" column), MC-A8 (an `unknown`
call is charged the fallback) · code `crates/research/src/{stages.rs, decide.rs, runs.rs, gateway.rs}`,
`crates/research/src/rules/{fingerprint.rs, run_rules.rs}`, `crates/research/src/fakes.rs` (`before_call` hook),
the harness's `restart()`.

### Files
Edited: `crates/research/src/{stages.rs, decide.rs, runs.rs, gateway.rs, store.rs, domain.rs, rules/run_rules.rs}`. New: `crates/research/tests/engine/resume.rs`; `crates/research/tests/engine/{main.rs, harness.rs}` edited.

### Core mechanisms (the only functions with unit tests)
`run_rules::follow_up_needed`.

### Ordered tasks
1. **Resume.** `execute` continues at the first stage with an uncommitted unit; a committed unit never runs again;
   a call row still `sent` with no response is closed as `unknown` (charged the fallback) and re-issued as the next
   attempt; stage 7 re-applies its writes through `writes::suggest`, where the same-content check makes a repeat a no-op.
2. **Reuse.** In an `auto` or `follow_up` run, a unit whose fingerprint equals a committed unit's in the latest
   earlier run of the session is copied with `reused_from_run_id`, with no call and no step, provided every file
   its output references is still present. A `manual` run reuses nothing.
3. **Input snapshot and trigger set.** At start the run stores `input_snapshot`: the set of stage 1 unit photo
   identities (photo id, composition revision), the cover identity, the stage 6 Condition identity.
   `follow_up_needed(latest, now)` is true iff one of the three differs. `session_changed` (R4·U4) queues a
   `follow_up` then, per MC-A6's table: none queued or running → create; one queued → keep it (its start moves with
   the quiet period); one running → create unless one is queued.

### Criteria (6 criteria, 17 rows)
Boundary: the research test harness (`Harness::new()`); a kill is `Harness::restart()` from the fake's `before_call` hook or between
ticks; "equals the uninterrupted run" compares, against a reference run of the same script on a second harness:
the stage outputs (stage, unit key, fingerprint, output), the steps (seq, kind, summary, field keys) and the
`attribute_values` rows (all columns but `updated_at`).
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| A14·C1(a) | kill before a unit's call | kill when the second observe call arrives (before its record is committed) | equals the uninterrupted run; the first observe unit is requested once in total | MA14 |
| A14·C1(b) | kill with a call in flight | kill inside the hook of the first read call (record `sent`, no response) | that call row becomes `unknown`, charged the fallback; a second row with the next attempt number answers; outputs, steps and values equal the uninterrupted run (cost differs by exactly the fallback) | MA14, MA15 |
| A14·C1(c) | kill between units | kill after the find unit committed, before any read | equals the uninterrupted run; the search fake has one request in total | MA4 |
| A14·C1(d) | kill before Decide | kill after stage 6 committed | equals the uninterrupted run | MA4 |
| A14·C1(e) | kill inside Decide | kill when the stage 7 call arrives | equals the uninterrupted run; each written key has exactly one `concluded` step | MA4 |
| A14·C1(f) | kill after the writes | kill after the stage 7 commit, before the run row is set `finished` | run `finished`; no step and no attribute row changed by the resume | MA14 |
| A14·C2(a) | repeat of stage 7 | `execute` called again on a finished run's stage 7 | no attribute row's `updated_at` changes; no step is added | MA14 |
| A14·C3(a) | follow-up, new Main photo | a finished run; a third main photo is added (no marking) | the follow-up calls the model for one observe unit and for stage 7 only; every other unit row carries `reused_from_run_id`; no step repeats an earlier run's `looked` for the old photos | MA14 |
| A14·C3(b) | follow-up, new Condition photo | a condition photo is added | calls: stage 6 and stage 7 only | MA14 |
| A14·C3(c) | follow-up, new cover | main position 1 changes by reorder | calls: find, read, compare and stage 7; observe units are reused | MA14 (R-IA17) |
| A14·C3(d) | follow-up, recompose | one main photo is re-framed | calls: that photo's observe unit and stage 7 (plus find, read and compare iff it is the cover) | MA14 |
| A14·C3(e) | missing file | the stored copy a reusable unit references is deleted from disk | that unit is run again, not copied | MC-A7 |
| A14·C4(a) | manual run | Run again on an unchanged session | every unit is called again; no row carries `reused_from_run_id` | MC-A7 (R-IA18) |
| A14·C5(a) | triggers | after a finished run, each alone: a main photo added; a main photo classified `deleted`; a `deleted` photo restored to main; a photo re-framed; main → condition; condition → main; a new cover by reorder | one `follow_up` run queued, seven times | MA14 (R-IA17) |
| A14·C5(b) | non-triggers | each alone: a Capture One retouch (new `adjusted_gen` only); a stage's model changed in settings; two non-cover main photos swapped | no run queued, three times | MC-A7 |
| A14·C6(a) | changes during a run | three photo changes while a run is `running` | exactly one `follow_up` queued; it starts after the run ends and the quiet period has passed | MA13 |
| A14·C6(b) | `follow_up_needed` | two snapshots differing only in the order the photo identities are listed | false | MC-A7 |

### Delegated choices
How the reference run is built in the tests (a helper running the same script on a fresh `TestApp`) · where
`input_snapshot` is computed · whether a reused unit's steps are shown from the earlier run in the panel (they are
not copied; R6·U1 shows the latest run's steps and says "reused" for the rest — refine there).

### Notes
- C1 enumerates the kill points the commit rule creates (charter rule 2): before a call record, with a call in
  flight, between units, between stages 6 and 7, inside stage 7, after its commit.
- C5(b)'s first case: a retouch changes the adjusted preview but not the photo identity of MC-A7's stage 1 unit
  (photo id, composition revision); the contract excludes it on purpose.

### Unit test commands (run after the unit, then make the `CHECKPOINT (not approved):` commit)
`cargo test -p psm-research --test engine resume::` · `cargo test -p psm-research --lib rules::run_rules` · `cargo clippy -p psm-research --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cargo check --workspace`

## Unit 6 — The research API: status, Run again, keys, trusted sites, settings validation

**Unit id:** `R4·U6` · **Replaces:** A18 criteria C1 and C6; A19 criteria C1–C4 · **Changes app files:** no

### Goal
The implementation of `ResearchApi` (frozen in R2·U2) over the engine: status with its derived counts, Run again, keys into the secrets port, trusted-site editing, validation of research settings. These are the backend rows of the superseded panel and settings plans; their boundary is now the trait, not a Tauri command.

### Read first
`master_plan_research.md` §6.6 (`ResearchApi`, accounts, settings keys, sentences) · book `08A` §6 (Trusted sites), §8 (derived values), `08B` MC-A3 (Trusted entry), MC-A6, MC-A8 (decimal strings), MC-A10 ("n to review"), MC-A11 (`record_days` minimum 2) · code `crates/research/src/{api.rs, service.rs, runs.rs, config.rs, rules/sites.rs}`, `crates/core/src/ports/secrets.rs`.

### Files
New: `crates/research/src/api_impl.rs`, `crates/research/tests/engine/api.rs`. Edited: `crates/research/src/{lib.rs, service.rs, config.rs}`, `crates/research/tests/engine/{main.rs, harness.rs}`.

### Core mechanisms (the only functions with unit tests)
None new.

### Ordered tasks
1. `impl ResearchApi for ResearchService`. `status(session_id)`: the first failing prerequisite or waiting reason with its sentence, the latest run, its steps, and the derived counts (`fields_filled` = keys the latest run wrote; `to_review` = live `ai` + `suggested` rows whose key is in the current layout, at every confidence; never stored). `run(session_id)` = `run_again`.
2. `secrets_set` writes the given keys through the secrets port under the accounts of §6.6; `secrets_state` returns only which are set. No method ever returns a key.
3. `site_add`, `site_remove`, `sites_reorder`: `TrustedSites` (R3·U2) with the shop hosts from `CommerceShopFacts`; they return the new `research` settings value for the caller to save (the app saves it in R7·U1; the harness saves it to its settings file), or the refusal text.
4. `validate_settings(patch)`: refuses a cap or price that is not a decimal string with at most 6 fractional digits, `record_days` below 2, a negative `quiet_seconds`, a long edge that is not a positive integer.
5. `settings_changed()` after every successful save or key change (U4).

### Criteria (6 criteria, 11 rows)
Boundary: the `ResearchApi` methods on the harness. These rows were command-level rows in the superseded panel and settings plans; the Tauri commands are thin pass-throughs frozen in R2·U2 and connected in R7·U1.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| A18·C1(a) | `ResearchApi::status` | a finished run that wrote three fields, one accepted since, one of the remaining two `low` | `fields_filled` 3; `to_review` 2 | MC-A10 |
| A18·C1(b) | same | a field with a suggestion is removed from the layout | `to_review` drops by one | MC-A10 |
| A18·C1(c) | same | no model key | reason `NoModelKey` with its sentence; `can_run` false | MA7 |
| A18·C6(a) | `ResearchApi::run` | called on a session with a finished run | one `queued` run with trigger `manual` (it adds nothing to `run_again`) | MC-A6 |
| A19·C1(a) | `ResearchApi::secrets_set` / `secrets_state` (`FakeSecrets`) | set a model key | the secrets port holds it under `research-model-key`; `secrets_state` says model set, search not set; `settings.json` on disk does not contain the value | MA16 |
| A19·C1(b) | prerequisites | the keys set, a session with a Main photo | the status no longer names `NoModelKey` (eligibility re-evaluated by the save) | MC-A6 |
| A19·C2(a) | `ResearchApi::site_add` | `https://www.pamono.com/x` | the list ends with `pamono.com`; saved in `research.trusted_sites` | MC-A3 |
| A19·C2(b) | same | `shop.etsy.com` while `etsy.com` is listed; the shop's own domain | refused with `already covered by etsy.com`; refused with `already covered by <shop host>` | MC-A3 |
| A19·C2(c) | reorder, remove | three entries, the third moved first; one removed | the saved order is the new order; a run started afterwards reads pages in it, a run started before keeps its snapshot | MC-A3 |
| A19·C3(a) | `ResearchApi::validate_settings` | `max_cost_per_run` = `0.50`; `0,5`; `0.0000001`; `record_days` = 1 | saved; refused; refused; refused | MA15, MC-A11 |
| A19·C4(a) | `auto` | switch Automatic research off while a run is queued (`auto`) | that run is `cancelled` | MC-A6 (R-IA25) |

### Delegated choices
How the sites methods hand back the new settings value · wording of validation refusals. Not delegated: the refusal text `already covered by …` and the sentences of §6.6.

### Notes
- `FakeSecrets` holds distinctive key values that appear in no expected text.

### Unit test commands (run after the unit, then make the `CHECKPOINT (not approved):` commit)
`cargo test -p psm-research --test engine api::` · `cargo clippy -p psm-research --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cargo check --workspace`

## Closing gate (once, after the last unit; replaces the template's closing gate)
`cargo test -p psm-research` · `cargo clippy -p psm-research --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cargo check --workspace`. **No app suite** (no file outside `crates/research` changed, except `Cargo.lock`).

## Review mode
The reviewer reads the units in order and runs, once: the milestone's research-crate tests (`cargo test -p psm-research`), `cargo clippy -p psm-research --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cargo check --workspace`. **No app suite** (no app file changed; flow §10). Only B1–B4 block. Notes go to `docs/implementation/backlog_research/R4_research_engine.md`.

## Manual try-out (the narrative is built from this)
Nothing visible in the app yet (the engine is wired in R7). A run can be read as a story: `cargo test -p psm-research --test engine stages::` and `… resume::`.

## Review log
