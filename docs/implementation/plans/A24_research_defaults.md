---
phase: A24_research_defaults
title: Research works out of the box — the models, efforts and prices the owner picked are the shipped defaults, so with the keys entered the first approved photo starts a run with no further setup
difficulty: normal
projection: no
depends_on: [A23_research_acceptance_benchmark]
blocked_on: the owner's pick of model and effort per stage (evidence doc, "Owner picks", after A23)
---
# A24 — Shipped defaults and reachability (08A §9 "Defaults are chosen by the first benchmark"; charter rule 10; MA5, MA7)

## Goal
Write the owner's picks into the settings defaults: per stage the model and effort, the provider ids, the price
rows for exactly those models and services (with their fallback call price), the attribution keys, and `max_pages`
if the measurement changed it. Prove that the shipped defaults reach a run. **Not in this phase:** anything else.

## Task 0 (gate)
Read `docs/intention/evidence/research_providers.md` §Owner picks. If the per-stage models are not recorded there,
report `STATE: BLOCKED` (reason: `owner pick missing: per-stage models`) and change nothing.

## Read first
`master_plan_research.md` §3 (HA5 in code: literals belong in `settings_file.rs` defaults and adapters, never in
core), §6.6 · book `08A` §6 (attribution keys), §9, `08B` MC-A6 (prerequisite: price table), MC-A8 (price rows,
fallback) · evidence doc (Owner picks; the pricing pages recorded by A01/A23 with their dates) · code
`crates/infra/src/settings_file.rs` (`ResearchSettings::default`), `crates/core/src/services/research/runs.rs`.

## Dependencies and gate
Tracker row and gate: `master_plan_research.md` §4. Notes: `backlog_research.md`.

## Files expected to change
`crates/infra/src/settings_file.rs`, `crates/testkit/tests/suite/research/a24_research_defaults.rs` (+ `mod` line),
`master_plan_research.md` §6.6 (the defaults recorded) and own tracker row.

## Core mechanisms
None.

## Ordered tasks
1. Defaults from the Owner picks: `stages.<stage>.{model, effort}`, `providers.*`, `prices` (one row per unit the
   picked adapters report, from the dated pricing record; `fallback_call_usd` per provider and model),
   `attribution_keys` (the app's keys of the shop's Designer and Manufacturer fields, checked against the
   definitions fixture), `max_pages`.
2. The reachability test (C1) and the consistency test (C2).

## Criteria (2 criteria, 4 rows)
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | a fresh app (suite): default settings, the keys set in `FakeSecrets`, a fake shop, fake providers registered under the default provider ids | a session gets its first Main photo with a preview; the quiet period passes | a run is created and starts with **no** settings edit; `status` names no prerequisite | MA7 (rule 10) |
| C1(b) | same, without the keys | the same photo | no run; `status` names `NoModelKey`; export of the session works | MA7 |
| C2(a) | `ResearchSettings::default()` | the default value | every stage has a non-empty model and effort; for every (provider, model) pair the defaults use, the price table has a row for each unit that adapter reports and a positive `fallback_call_usd`; `max_cost_per_run` and `max_cost_per_day` parse as micro-USD; `record_days` ≥ 2 | MA5, MA15 |
| C2(b) | same | `attribution_keys` | not empty; each key is a definition key in the shop definitions fixture | MA2 |

## Delegated choices
None beyond formatting. The values are the owner's picks; the test asserts their contract, never their spelling
(charter rule 13), so a later change of model is a settings edit, not a red suite.

## Manual try-out
1. Fresh scratch folder: `PSM_DATA_DIR=$HOME/psm-research-dev-2 npm run tauri -- dev`; connect the test shop; enter
   the two keys in Settings → Research and your trusted sites.
2. Create a session and shoot one photo. Without touching any other research setting, the form's research panel
   starts showing steps within a few seconds.

## Notes
- Prices change; the row is dated in the evidence doc, and a wrong price only mis-estimates cost (the cap still
  stops a run by the estimate). The orchestrator re-checks the pricing page when a provider announces a change.
- Review mode: this phase's module, `research::a13…::`, clippy, fmt.

## Review log
