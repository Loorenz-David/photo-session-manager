---
phase: A23_research_acceptance_benchmark
title: The finished research agent measured on your 42 pieces (owner-attended, real keys) — how often a medium or high attribution is correct, how much is left undetermined, what a piece costs and how long it takes, and a second look at which model suits each stage
difficulty: high
projection: no
depends_on: [A15_research_model_adapter, A16_research_visual_search_adapter, A17_research_page_reader_and_shop_hosts, A18_research_panel_ui, A19_research_settings_ui, A20_research_publish_ui, A21_research_retention_and_export, A22_research_report_and_replay_tool]
blocked_on: owner-attended; the keys of master_plan_research.md §7.6
---
# A23 — Acceptance measurement on the real engine (08A §11 operational measurement, §9 defaults, §12A)

## Goal
Run the real engine with the real adapters over the owner's test pieces, in a scratch data folder, and record the
operational measurement of 08A §11: the share of attribution suggestions at medium or high that are correct, the
share left not determined, cost and time per piece. Then compare models per stage by replaying recorded stages
(A22), so the owner can confirm or change the per-stage picks of A02. **Not in this phase:** any change to the
engine (defects found become findings for the orchestrator), the shipped defaults (A24).

## How this phase runs
Owner-attended, like P47 and P52: the orchestrator runs it with the owner available. **If launched by
`run_phase.sh`, the implementer builds only the `accept` subcommand and its two automated rows (tasks 1–2), then
reports `STATE: DONE`; the measurement run (tasks 3–6) is done by the orchestrator** and recorded in the evidence
doc and this plan's Review log. **Spending ceiling: USD 40** (`--ceiling-usd 40`; the engine's own caps
`max_cost_per_run` 0.50 and `max_cost_per_day` are set in the scratch settings so that 42 runs fit: day cap 25).

## Read first
`master_plan_research.md` (all) · book `08A` §6, §9, §11 (operational measurement), §12A, `08B` MC-A4, MC-A8 ·
evidence doc (A01, A02 sections, Owner picks) · the test file and `answer_key_corrections.json` (**scoring rules**:
hedged pieces 2, 26, 37, 38 count as correct at medium or low and as wrong at high; disagreements are review
candidates, listed with the agent's evidence) · A22's plan · code `crates/integrations/src/lib.rs` (`App::open`
with `PSM_DATA_DIR`), `crates/testkit/src/test_app.rs` (how a session with photos and previews is built through
the services), `crates/research-tool/src/*`.

## Dependencies and gate
Tracker row and gate: `master_plan_research.md` §4. Notes: `backlog_research.md`.

## Files expected to change
`crates/research-tool/src/{accept.rs, score.rs, main.rs, lib.rs}`, `crates/research-tool/Cargo.toml`
(`psm-integrations`, `psm-infra`), `crates/testkit/tests/suite/research/a23_research_acceptance_benchmark.rs`
(+ `mod` line), `docs/intention/evidence/research_providers.md` (section "Acceptance measurement"), own tracker
row, this plan's Review log. Outside the repo: `~/psm-research-bench/a23/` (the scratch data folder and exports).

## Core mechanisms
None.

## Ordered tasks
1. `accept --pieces <file> --data <scratch dir> --ceiling-usd <n> [--only <n,…>]`: open the app headless on the
   scratch folder (the real `App`, the real research adapters, keys from the environment handed to an in-memory
   secrets store, never the Keychain). The shop is a **stand-in `CommerceAdapter` inside the tool**: definitions
   and layouts from the shop definitions fixture, `public_listing` from the pieces file (the shop's host, each
   published piece's URL and handle, so its own listing is excluded, MC-A3; nothing for the unpublished pieces),
   and every mutating method refusing (H1: this tool can write nothing to a shop). For each piece create an item
   and a session through the services and add its images as photos whose adjusted preview is the downloaded image
   (first image = cover; `condition_report_images` as Condition photos). The scratch settings keep the default
   attribution keys and carry the nine trusted sites in the owner's order. Let the engine run each session
   (manual trigger), then export (A21).
2. `score` (extends A02's): compare each run's written values with the key; attribution fields scored with the
   hedged rule; corrected-key fields in their own bucket; fields the key does not hold are not scored.
3. Run over all pieces with the picked adapters and the per-stage models of the owner's A02 pick.
4. Record in the evidence doc: per group (published / unpublished) — attribution at medium or high: correct /
   wrong / hedged-correct; attribution at low; not determined; visible and condition fields correct / wrong / not
   determined; USD and seconds per piece (median, maximum); runs that ended `stopped_at_cap` or `failed`; every
   disagreement with its evidence (steps and sources).
5. Per-stage model comparison by `replay` on a sample of 12 pieces (4 per test group) for each candidate model.
6. Findings: anything that contradicts a contract (a value above `low` without the required evidence would be MA2
   broken) is reported to the orchestrator as a defect with the run id; it is not fixed here.

## Criteria (5 criteria, 6 rows, 3 `[rule]`; C3–C5 are `[manual]`)
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | `score`, hedged rule | a hedged piece whose named designer is suggested at medium; at low; at high | correct; correct; wrong | `[rule]` corrections file `scoring_rules.hedged_attribution` |
| C1(b) | `score` | an attribution the key leaves empty, suggested at medium; left not determined | wrong (listed as a review candidate with its sources); correct | MA2 |
| C2(a) | `accept` (HTTP seam stub replaying A01/A02 captures; `FakeTransport` for the shop) | two pieces from a small pieces file | two items, two sessions, each with its photos; one run per session reaches a final state; one export folder exists per run; nothing is written outside the scratch folder | MA9 |
| C3(a) | evidence doc `[manual]` | all pieces of the test file | the tables of task 4 | `[rule]` 08A §11 |
| C4(a) | evidence doc `[manual]` | the 12-piece sample, each candidate model per stage | per stage and model: agreement with the key, USD and seconds per piece | `[rule]` 08A §9 |
| C5(a) | Review log `[manual]` | the run | no stored attribution above `low` lacks a read label, a shop `match` or two trusted `match` sites in its steps (checked over every run's `writes.json` and steps); spend within the ceiling | MA2, HA6 |

## Delegated choices
How the shop's layout is obtained for item types the test file names (the shop's presets if any, else every
researchable definition) · sampling of the 12 pieces (stated) · table layout.

## Manual try-out
1. Read "Acceptance measurement" in the evidence doc: of the designer and maker suggestions the agent was fairly
   sure about, how many were right; how many pieces it left open; what a piece costs.
2. Open the disagreement list and look at three pieces you know well: is the agent or the shop data right?
3. The narrative's **Needs you** carries one card: the model and effort per stage to ship as defaults (numbers in
   the card), and, if the measurement shows it, the page limit.

## Notes
- The photos of the test pieces are finished shop images, not the studio's previews; results on live captures may
  differ. The narrative says so in plain words.
- The owner's answer goes into the evidence doc under **Owner picks**; A24 reads it.
- Review mode: the reviewer runs this phase's suite module, clippy, fmt; checks the evidence tables against the
  exports in `~/psm-research-bench/a23/`; re-runs no real call.

## Review log
