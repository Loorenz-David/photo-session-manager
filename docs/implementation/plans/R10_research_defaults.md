---
phase: R10_research_defaults
title: Research works out of the box — the models, efforts and prices the owner picked are the shipped defaults, so with the keys entered the first approved photo starts a run with no further setup
difficulty: normal
projection: no
depends_on: [R9_research_tool_and_acceptance]
blocked_on: the owner's pick of model and effort per stage (evidence doc, "Owner picks", after R9's measurement)
---
# R10 — Defaults: research works out of the box

**Tracker:** NOT_STARTED · 2026-11-12 · planner · —

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
