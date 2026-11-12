---
phase: A02_research_stage_model_benchmark
title: Research benchmark, part 2 — three model candidates are compared on the stages that have an answer key (looking at the piece, reading a listing, reading condition photos), with cost and time per piece, so the owner can make a first pick of the model for each stage
difficulty: normal
projection: no
depends_on: [A01_research_probes_and_search_benchmark]
blocked_on: the owner's keys (master_plan_research.md §7.6); A01's recordings on this Mac
---
# A02 — First per-stage model comparison (08A §9, §12A.1)

## Goal
Compare the candidate models, stage by stage, on the three stages for which the test set holds the answer:
**observe** (visible fields from the cover photo), **read** (what a listing page states), **condition** (severity
fields from the condition photos). Record accuracy, cost and time per piece and model in the evidence doc. The
owner makes a first pick of model and effort per stage from it; A23 confirms it on the finished engine.
**Not in this phase:** compare (no labelled match set exists; measured in A23), labels (the key does not say which
pieces carry a label), decide (confidence is computed by code, the model only writes the reason); any app code.

## Precondition
As A01, with `OPENAI_API_KEY` only. `~/psm-research-bench/a01/` (A01's recordings: reduced copies, fetched pages) is
present on this Mac; otherwise `STATE: BLOCKED`. **Spending ceiling: USD 35**, call cap 700.

## Read first
`master_plan_research.md` · A01's plan (the tool, recording layout, budget) and its Review log · evidence doc
(A01's sections: usage fields, strict schema, image limits, candidate models) · book `08A` §5 (stages), §6, §9,
`08B` MC-A2 (what a listing states, choice mapping), MC-A4 (condition rule) · the test file and the corrections
file · `crates/integrations/src/commerce/shopify/fixtures/sv1_definitions_p1.json` (the definitions and their
choices for the scored keys).

## Dependencies and gate
Tracker row and gate: `master_plan_research.md` §4. Notes: `backlog_research.md`.

## Files expected to change
`crates/research-tool/src/{bench_models.rs, instructions/…, score.rs}` and small edits to `main.rs`, `lib.rs`,
`report.rs` · `crates/integrations/src/research/fixtures/openai/` (one strict-schema reply per scored stage; README
updated) · `crates/testkit/tests/suite/research/{mod.rs, a02_research_stage_model_benchmark.rs}` ·
`docs/intention/evidence/research_providers.md` (section "Benchmark 2") · own tracker row.

## Core mechanisms
None in `psm-core`. `score` is tested through C1.

## Ordered tasks
1. `score` and its tests (C1), and the resume rule (C2), before any real call.
2. Draft instructions and strict output schemas, version 0, for the three stages (`instructions/`). They are the
   starting point A15 takes over; keep each to what the stage's output in 08A §5 needs. Observe returns the visible
   fields with a `sure` flag and `shows_marking`; read returns, per researched key, `{stated, value}`; condition
   returns one severity per condition field with a `sure` flag. The definitions' own choice strings are given to
   the model (MC-A2).
3. Matrix (`psm-research bench-models`): three candidate models at the provider's default effort on all three
   stages; then the best model per stage at one lower and one higher effort. Inputs: observe — the cover copy of
   every piece; read — for every piece with attribution, up to three pages A01 fetched (the first naming result,
   and the first two non-naming results on trusted sites or the shop); condition — every piece with condition
   images, at most six images, long edge 2048.
4. Score against the key; write "Benchmark 2" (per stage and model: correct / wrong / not determined, schema
   failures, refusals, USD per piece, seconds per piece; corrected-key bucket apart; every disagreement listed).
5. Copy one sanitized reply per stage into the fixtures folder; closing gate once.

## Criteria (5 criteria, 9 rows, 2 `[rule]`; C3–C5 are `[manual]`)
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | `score` | choice key, key value `Teak`, answers `Teak`, `teak`, `Oak`, none | correct, correct (mapped by name key), wrong, not determined | MC-A2 |
| C1(b) | `score` | multi-choice key, key `["Teak","Oak"]`, answers `["Oak","Teak"]`, `["Teak"]` | correct, wrong | MC-A2 |
| C1(c) | `score` | text key, key `Hans J. Wegner`, answers `hans j wegner`, `Hans Wegner` | correct, wrong | MC-A2 |
| C1(d) | `score` | a field listed in the corrections file | scored against the corrected value in the corrected bucket; absent from the owner-verified bucket | `[rule]` corrections file |
| C1(e) | `score` | read stage, a page that does not name the key, answers none / a name | correct / wrong | MC-A2 |
| C2(a) | `bench-models` resume | a run stopped after 3 of 5 calls (HTTP seam stub), started again | 2 requests leave; 5 recordings exist; spend counted once | MA5 |
| C3(a) | evidence doc `[manual]` | the matrix | the tables of task 4 for every stage, model and effort run | `[rule]` 08A §9 |
| C4(a) | fixtures `[manual]` | one reply per stage | sanitized, listed in the README with date and model; secret scan clean | charter rule 17 |
| C5(a) | Review log `[manual]` | the run | calls and USD, within the cap and the ceiling | HA6 |

## Delegated choices
The wording of the version-0 instructions · which two extra effort levels · table layout. Not delegated: the
ceiling, the inputs per stage, the scoring rules.

## Manual try-out
1. Read "Benchmark 2" in the evidence doc: for each stage, which model answered correctly how often, and what a
   piece costs.
2. The narrative's **Needs you** carries one card: the model and effort per stage to start with (the numbers are in
   the card).

## Notes
- The owner's answer is recorded by the orchestrator in the evidence doc under **Owner picks**. It sets nothing in
  the app yet; A24 writes the defaults after A23.
- Review mode as A01: no real call is re-run.

## Review log
