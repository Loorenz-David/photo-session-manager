---
phase: R10_research_defaults
title: Research works out of the box — the models, efforts and prices the owner picked are the shipped defaults, so with the keys entered the first approved photo starts a run with no further setup
difficulty: normal
projection: no
depends_on: [R9_research_tool_and_acceptance]
blocked_on: none (owner picked the models 2026-11-15)
---
# R10 — Defaults: research works out of the box

**Tracker:** IMPLEMENTED · 2026-11-15 · codex · U1 shipped defaults (owner's models, prices with cache_write_token, auto off, attribution keys empty) + 4 row tests; research crate 294 tests, clippy, fmt, workspace check green

(This line is the milestone's tracker row. Sessions change only this line of this file: state, date, actor, one-line note. The gate for `depends_on` is the Tracker line of each plan named there. `master_plan.md` §4 is not used for research milestones.)

## Goal
Write the owner's picks into `psm_research::config::ResearchSettings::default()` and prove the shipped defaults reach a run (charter rule 10). One unit.

## How to work this plan (flow.md §10)
This milestone is one runner phase made of 1 unit. Work the units in order. After each unit run its **Unit test commands**, then make a `CHECKPOINT (not approved):` commit of that unit's paths. One review, one fix budget and one narrative cover the whole milestone. Row ids keep the number of the superseded plan they were written in (`A07·C1(a)`); `docs/implementation/research_row_mapping.md` says where every row went. Name each test after its row (`a07_c1a_<slug>`). Reviewer notes go to `docs/implementation/backlog_research/R10_research_defaults.md`.

| Unit | What it builds | Criteria | Rows | Changes app files |
|---|---|---|---|---|
| U1 | Shipped defaults and the reachability check | 2 | 4 | no |
| **Total** | | **2** | **4** | |

## Read first (whole milestone)
`docs/process/flow.md` (**§10 is the authority for this track**) · `docs/implementation/master_plan.md` §3, §6, §7 · `docs/implementation/master_plan_research.md` (all: layout §0, standing rules §3, sequencing and tracks §5, registry §6, commands §7.2, the storage decision §6.4) · book `01`, `14` §Research agent, `08A`, `08B` · then each unit's own Read first. Name key and sameness are `psm_core::mechanisms::{name_key, sameness}` (R2·U1); every other research rule is `psm_research::rules`.

## Carried from earlier reviews (orchestrator, 2026-11-12; wins over older wording below)
- **U1, default attribution keys (R4 review).** The shipped default `attribution_keys` is `["beyo.designer"]` (the shop's Designer field; the shop has no manufacturer field), not the registry's older `["custom.designer", "custom.manufacturer"]`; `master_plan_research.md` §6.6 is updated. Keep it unless the owner's pick says otherwise.
- **U1, `cache_write_token` price (R8, 08B changelog 11).** OpenAI bills cache-write tokens as their own unit; the shipped
  price table carries a `cache_write_token` row for every model default, or calls that report it are charged the
  fallback.
- **U1, which fields are designer and maker (R9 review).** The measurement uses `custom.designer` and
  `custom.manufacturer` (the 68-metafield fixture, which the answer key matches); the engine's default is
  `["beyo.designer"]` (R4). Before writing the shipped default, confirm against the live shop which keys hold designer
  and maker, so the measured behaviour is the shipped one; ask the owner if the shop has both namespaces.
- **U1, designer and maker are a setting the owner points (owner 2026-11-13; replaces the "confirm the keys" note
  above).** The owner's shop names them "Designer" and "Manufacturer" (`custom.designer`, `custom.manufacturer`); the
  dev store's designer is `beyo.designer` and it has no manufacturer, so no fixed key can be the default. Like the
  condition-photos field, Settings → Research gets a picker of the linked shop's product fields for "designer and
  maker" (one or more), stored in `attribution_keys`. The shipped default is empty; when the shop is linked and the
  setting is still empty, the picker preselects the fields named Designer or Manufacturer (name compared without
  case) for the owner to save or change. Rows: picker lists the shop's fields; saving two keys → both treated as
  attribution fields; an empty setting → the engine's default `["beyo.designer"]` is gone and no field is treated
  as attribution until chosen (the settings screen says so).
- **U1, the shop's own addresses (owner 2026-11-13).** The owner's shop answers on `beyovintage.se`, `www.beyovintage.se`,
  `beyovintage.com`, `www.beyovintage.com`, `beyovintage.myshopify.com`, `68c5b4-6a.myshopify.com`. Shop hosts today
  are the primary domain and the platform host (08B changelog 11(c)). Without `read_markets` (declined for now), add a
  settings list of extra shop addresses (empty by default), and check that `www.` of the primary is already covered.
- **U1, scope after the owner's 2026-11-15 decisions (orchestrator; wins over every note above and the rows below).**
  (1) Models: the evidence doc's "Owner pick of models" (observe Sol low, read Luna low, compare Terra low,
  condition Terra high, labels Sol low, decide Terra low). Task 0 reads that section, not "Owner picks".
  (2) Prices for exactly those models, each with a `cache_write_token` row, from the evidence doc's dated price table.
  (3) `auto` defaults to **false**: runs will start from the "Arranged — next" button (R11, 08B changelog 13); until
  then the manual "Run research" starts them. (4) `attribution_keys` defaults to **empty**; the engine's
  `["beyo.designer"]` goes; an empty list means no field is treated as attribution. (5) The designer/maker picker,
  the extra shop addresses and every other settings screen change are **R11, not here**; this phase changes no app
  file. Rows changed accordingly: **A24·C1(a)** — fresh app, keys set, a session with a Main photo with a preview:
  the quiet period passing creates **no** run; a manual run request starts one with no settings edit and `status`
  names no prerequisite. **A24·C1(b)** unchanged except the trigger is the manual request. **A24·C2(b)** —
  `attribution_keys` is empty by default (the row asserts only that; the rules are unchanged).

## Unit 1 — Shipped defaults and the reachability check

**Unit id:** `R10·U1` · **Replaces:** A24 (all rows) · **Changes app files:** no

### Goal
Write the owner's picks into the settings defaults: per stage the model and effort, the provider ids, the price
rows for exactly those models and services (with their fallback call price), the attribution keys, and `max_pages`
if the measurement changed it. Prove that the shipped defaults reach a run. **Not in this phase:** anything else.

### What changed from the superseded plan (this wins over any older wording below)
- `ResearchSettings` is `psm_research::config` (R4·U4), so the defaults are in `crates/research/src/config.rs`, not in `crates/infra/src/settings_file.rs` (HA5 still holds: the literals are in the research crate's settings defaults and adapters, never in `psm-core`).
- The tests are research-crate tests (`crates/research/tests/engine/defaults.rs`); A24·C1's "fresh app" is the research harness built with `ResearchSettings::default()` untouched (`Harness::with_default_settings()`), the keys in `FakeSecrets`, the fake shop, and the crate's fakes registered under the default provider ids; the trigger is a first Main photo followed by `session_changed`, as in R4·U4.

### Task 0
Read `docs/intention/evidence/research_providers.md` §Owner picks. If the per-stage models are not recorded there,
report `STATE: BLOCKED` (reason: `owner pick missing: per-stage models`) and change nothing.

### Read first
`master_plan_research.md` §3 (HA5 in code: literals belong in the research crate's settings defaults and adapters, never in
core), §6.6 · book `08A` §6 (attribution keys), §9, `08B` MC-A6 (prerequisite: price table), MC-A8 (price rows,
fallback) · evidence doc (Owner picks; the pricing pages recorded by R1·U1/R9·U2 with their dates) · code
`crates/research/src/config.rs` (`ResearchSettings::default`), `crates/research/src/runs.rs`.

### Files
Edited: `crates/research/src/config.rs`, `master_plan_research.md` §6.6 (the defaults recorded). New: `crates/research/tests/engine/defaults.rs` (+ `mod` line in `tests/engine/main.rs`).

### Core mechanisms (the only functions with unit tests)
None.

### Ordered tasks
1. Defaults from the Owner picks: `stages.<stage>.{model, effort}`, `providers.*`, `prices` (one row per unit the
   picked adapters report, from the dated pricing record; `fallback_call_usd` per provider and model),
   `attribution_keys` (the app's keys of the shop's Designer and Manufacturer fields, checked against the
   definitions fixture), `max_pages`.
2. The reachability test (C1) and the consistency test (C2).

### Criteria (2 criteria, 4 rows)
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| A24·C1(a) | a fresh app (suite): default settings, the keys set in `FakeSecrets`, a fake shop, fake providers registered under the default provider ids | a session gets its first Main photo with a preview; the quiet period passes | a run is created and starts with **no** settings edit; `status` names no prerequisite | MA7 (rule 10) |
| A24·C1(b) | same, without the keys | the same photo | no run; `status` names `NoModelKey`; export of the session works | MA7 |
| A24·C2(a) | `ResearchSettings::default()` | the default value | every stage has a non-empty model and effort; for every (provider, model) pair the defaults use, the price table has a row for each unit that adapter reports and a positive `fallback_call_usd`; `max_cost_per_run` and `max_cost_per_day` parse as micro-USD; `record_days` ≥ 2 | MA5, MA15 |
| A24·C2(b) | same | `attribution_keys` | not empty; each key is a definition key in the shop definitions fixture | MA2 |

### Delegated choices
None beyond formatting. The values are the owner's picks; the test asserts their contract, never their spelling
(charter rule 13), so a later change of model is a settings edit, not a red suite.

### Notes
- Prices change; the row is dated in the evidence doc, and a wrong price only mis-estimates cost (the cap still
  stops a run by the estimate). The orchestrator re-checks the pricing page when a provider announces a change.

### Unit test commands (run after the unit, then make the `CHECKPOINT (not approved):` commit)
`cargo test -p psm-research --test engine defaults::` · `cargo clippy -p psm-research --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cargo check --workspace`

## Closing gate (once, after the last unit; replaces the template's closing gate)
`cargo test -p psm-research` · `cargo clippy -p psm-research --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cargo check --workspace`. **No app suite** (only `crates/research` changed).

## Review mode
The reviewer reads the units in order and runs, once: the milestone's research-crate tests (`cargo test -p psm-research --test engine defaults::`), `cargo clippy -p psm-research --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cargo check --workspace`. **No app suite** (no app file changed; flow §10). Only B1–B4 block. Notes go to `docs/implementation/backlog_research/R10_research_defaults.md`.

## Manual try-out (the narrative is built from this)
1. Fresh scratch folder: `PSM_DATA_DIR=$HOME/psm-research-dev-2 npm run tauri -- dev`; connect the test shop; enter the two keys in Settings → Research and your trusted sites.
2. Create a session and shoot one photo. Without touching any other research setting, the form's research panel starts showing steps within a few seconds.

## Review log

### 2026-11-15 · implement r1 (Claude Opus 5.5, launched as `pipeline-implementer`)
- **Built (R10·U1).** `ResearchSettings::default()` (`crates/research/src/config.rs`) now ships the owner's pick of
  2026-11-15: observe `gpt-6.1-sol` low, read `gpt-6-luna` low, compare `gpt-6.1-terra` low, labels `gpt-6.1-sol` low,
  condition `gpt-6.1-terra` high, decide `gpt-6.1-terra` low (find calls no model); the price table for exactly
  those three models (`input_token`, `cached_input_token`, `cache_write_token`, `output_token`, `web_search_call`,
  fallback 0.05 per call), `serpapi_lens` `image_search_call` (USD 25 per 1,000, fallback 0.025) and `page_http`
  `page_fetch` (0, fallback 0.000001); `auto` false; `attribution_keys` empty. `master_plan_research.md` §6.6 records
  the values. Tests `crates/research/tests/engine/defaults.rs`: `a24_c1a_…`, `a24_c1b_…`, `a24_c2a_…`, `a24_c2b_…`.
- **Assumptions.** (1) "status names no prerequisite" with `auto` off = the status reason is `AutoOff` with an empty
  sentence (MC-A6 puts `auto` off last and the panel shows no sentence for it); the test asserts exactly that.
  (2) C2(a) "every stage has a model" = every stage that calls the model; Find is the visual search and keeps an empty
  model. (3) Lens is priced at the cheapest paid SerpApi plan (USD 25 / 1,000), not 0 as in the measurement, so the
  caps count searches. (4) `page_http` gets a 1 µUSD fallback to satisfy "positive fallback" for a free download.
  (5) Model fallback 0.05 per call = the measurement tool's worst case. (6) The fresh app is built inside the test
  file (settings slot removed, fakes wrapped under the default ids) instead of a new `Harness::with_default_settings()`,
  so `harness.rs` stays untouched.
- **Deviations.** `tests/engine/lifecycle.rs`: removed `a13_c1d_default_attribution_keys_are_the_shops` (R4·U4,
  A13·C1(d)), which asserted the non-empty `beyo.designer` default the owner's 2026-11-15 decision removes (A24·C2(b)
  now asserts the empty default); its unused imports went with it. `max_pages` unchanged (10).

