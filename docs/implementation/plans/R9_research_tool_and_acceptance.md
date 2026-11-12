---
phase: R9_research_tool_and_acceptance
title: The dev-Mac tool and the acceptance measurement — from an exported research folder the tool reports what was accepted, edited or cleared and can replay a stage with another model; then the finished agent is measured on the owner's 42 pieces (owner-attended, real keys) so the owner can confirm the model for each stage
difficulty: high
projection: no
depends_on: [R7_research_wiring_and_export, R8_research_real_adapters, R1_research_benchmark]
blocked_on: U2's measurement run is owner-attended and needs the keys of master_plan_research.md §7.6
---
# R9 — Tool and acceptance: report, replay, and the finished agent measured on the 42 pieces

**Tracker:** NOT_STARTED · 2026-11-12 · planner · —

(This line is the milestone's tracker row. Sessions change only this line of this file: state, date, actor, one-line note. The gate for `depends_on` is the Tracker line of each plan named there. `master_plan.md` §4 is not used for research milestones.)

## Goal
(U1) `psm-research report` and `replay` (08A §10A, MC-A11 Replay identity). (U2) `psm-research accept`, which runs the real engine and adapters over the test pieces in a scratch data folder, and the measurement of 08A §11 recorded in the evidence doc. The runner builds both units' code and automated rows; **the measurement run itself (U2 tasks 3–6) is done by the orchestrator with the owner**, as P47 and P52 were. **Spending ceiling USD 40** plus USD 2 for one manual replay. **Not here:** any change to the engine; the shipped defaults (R10).

## How to work this plan (flow.md §10)
This milestone is one runner phase made of 2 units. Work the units in order. After each unit run its **Unit test commands**, then make a `CHECKPOINT (not approved):` commit of that unit's paths. One review, one fix budget and one narrative cover the whole milestone. Row ids keep the number of the superseded plan they were written in (`A07·C1(a)`); `docs/implementation/research_row_mapping.md` says where every row went. Name each test after its row (`a07_c1a_<slug>`). Reviewer notes go to `docs/implementation/backlog_research/R9_research_tool_and_acceptance.md`.

| Unit | What it builds | Criteria | Rows | Changes app files |
|---|---|---|---|---|
| U1 | `psm-research report` and `replay` | 5 | 9 | no |
| U2 | Acceptance measurement on the real engine | 5 | 6 | no |
| **Total** | | **10** | **15** | |

## Read first (whole milestone)
`docs/process/flow.md` (**§10 is the authority for this track**) · `docs/implementation/master_plan.md` §3, §6, §7 · `docs/implementation/master_plan_research.md` (all: layout §0, standing rules §3, sequencing and tracks §5, registry §6, commands §7.2, the storage decision §6.4) · book `01`, `14` §Research agent, `08A`, `08B` · then each unit's own Read first. Name key and sameness are `psm_core::mechanisms::{name_key, sameness}` (R2·U1); every other research rule is `psm_research::rules`.

## Carried from earlier reviews (orchestrator, 2026-11-12; wins over older wording below)
- **U2, offline calls in the MA12 check (R4 review).** A `not_sent` call keeps its recorded request but has no outbound rows (nothing left the Mac). The acceptance check of MA12 applies to sent requests only (ledger wording amended, 08B changelog 5).

## Unit 1 — `psm-research report` and `replay`

**Unit id:** `R9·U1` · **Replaces:** A22 (all rows) · **Changes app files:** no

### Goal
Grow R1·U1's tool into the dev-Mac tool of 08A §10A. `report <export>` prints, per stage and overall, suggestions
accepted unchanged, edited, cleared, not determined, attribution right or wrong against the photographer's final
value, cost and time. `replay <export> --run <id> --stage <stage> [--model …] [--effort …] [--instructions <file>]`
rebuilds each recorded request of that stage, substitutes only what the command line names, sends it with the
tool's own key from the environment, and shows the difference to the recorded answer. It never touches the work
Mac or the app's data. **Not in this phase:** running the engine over the test pieces (R9·U2).

### What changed from the superseded plan (this wins over any older wording below)
- Tests are in the tool crate (`crates/research-tool/tests/tool/u3_report_and_replay.rs`). The export fixture is produced in the test by `psm_research`'s `export` from a scripted run; the test includes the research crate's harness file by path (`#[path = "../../../research/tests/engine/harness.rs"] mod harness;`) and the tool crate dev-depends on `psm-research` and `psm-testkit`.

### Offline part / keyed part
Everything in the criteria is offline (an HTTP seam stub). A real replay is one manual step at the end, with
`OPENAI_API_KEY` from `~/.psm_research.env`, capped at USD 2; if the key file is absent the step is skipped and
said so. R1·U1 must be APPROVED only because the tool crate and its recording reader come from it; if this phase is
reached before R1·U1 has run, the orchestrator runs R1·U1 first.

### Read first
`master_plan_research.md` §3, §6.1, §7.2 · book `14` (MA9, MA16), `08A` §10A, `08B` **MC-A11** (Export layout,
Exclusions, **Replay identity**), MC-A9 (record fields) · R1·U1's plan (tool, recording layout, temporary helpers) ·
code `crates/research-tool/src/*`, `crates/research/src/export.rs` (the file shapes),
`crates/research/src/rules/{sites.rs, cost.rs, sameness.rs}`.

### Files
Edited/new: `crates/research-tool/src/{main.rs, lib.rs, report.rs, replay.rs, export_reader.rs}` (R1·U1's private host and cost helpers removed in favour of `psm_core` / `psm_research::rules`), `crates/research-tool/Cargo.toml`, `crates/research-tool/tests/tool/{main.rs, u3_report_and_replay.rs}`.

### Core mechanisms (the only functions with unit tests)
None.

### Ordered tasks
1. `export_reader`: read an unzipped (or zipped) export; the same reader reads R1·U1's bench recordings.
2. `report`: per stage and overall — from `writes.json` and `feedback.json`: accepted unchanged, edited, cleared,
   kept previous, not acted on, not determined; for the attribution keys of the run's settings snapshot: right
   (the final value is the same as the suggestion under `S`), wrong (edited or cleared), open; cost (sum of the
   calls' `cost_micro_usd`) and time per stage.
3. `replay`: for each call of the chosen stage, rebuild the recorded request body, resolving every
   `{"image_sha256"}` from `images/`; substitute **only** the model, effort or instructions named on the command
   line; send; print the recorded and the new output side by side. A call with a withheld image, a missing file or
   an expired record is reported `not replayable: <reason>` and is never sent, not even partially. A stage 3 call
   made through a model-side page tool is reported `not replayable: the page content cannot be reproduced`.
4. The key comes only from the environment; the tool has no Keychain code. Without a key `replay` exits before any
   request.
5. Replace R1·U1's private host matching and micro-USD sum with `psm_research::rules::{sites, cost}`.

### Criteria (5 criteria, 9 rows, 1 `[rule]`)
Boundary: the tool's library functions and command line over an export produced in the test by R7·U2's `export` from
a scripted run (so the fixture is the real export shape, built by production code).
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| A22·C1(a) | `report` | an export of one run that wrote five fields; the photographer accepted two, edited one, cleared one, left one | counts: accepted unchanged 2, edited 1, cleared 1, not acted on 1; the per-stage cost equals the sum of that stage's call costs | MA9 |
| A22·C1(b) | `report`, attribution | designer suggested A and accepted; manufacturer suggested B and edited to C | designer right, manufacturer wrong | MA9 |
| A22·C2(a) | `replay`, no substitution | a recorded observe call with two images | the body the HTTP stub receives is byte-equal to the body rebuilt from the record with the two images restored | MA16 |
| A22·C2(b) | `replay --model m2` | the same call | the received body differs from C2(a)'s only in the model field | MA16 |
| A22·C2(c) | `replay --instructions file` | the same call | the received body differs only in the instructions field | MA16 |
| A22·C3(a) | not replayable | a call whose image was withheld (photo deleted); a call whose image file is missing from `images/`; a run marked `call record expired` | each is reported `not replayable:` with its reason; the stub received no request for them | MA9 (R-IA26) |
| A22·C4(a) | key | `replay` with no key in the environment | exits non-zero before any request; the stub received nothing | MA16 |
| A22·C4(b) | key | a successful replay's printed output and any file it writes | the key value appears in none | MA16 |
| A22·C5(a) | bench recordings | `report` pointed at a folder in R1·U1's recording layout (built in the test with R1·U1's `recording` writer) | it reads the calls and prints cost and time per call group without error | `[rule]` 08A §10A |

### Delegated choices
Output format of `report` (plain text tables; `--json` if cheap) · how the side-by-side difference is printed ·
flag names beyond those in the goal.

### Notes
- The tool's dependency on `psm-core` is one-way; nothing in the app depends on the tool.

### Unit test commands (run after the unit, then make the `CHECKPOINT (not approved):` commit)
`cargo test -p psm-research-tool u3_report_and_replay::` · `cargo clippy -p psm-research-tool --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cargo check --workspace`

## Unit 2 — Acceptance measurement on the real engine

**Unit id:** `R9·U2` · **Replaces:** A23 (all rows) · **Changes app files:** no

### Goal
Run the real engine with the real adapters over the owner's test pieces, in a scratch data folder, and record the
operational measurement of 08A §11: the share of attribution suggestions at medium or high that are correct, the
share left not determined, cost and time per piece. Then compare models per stage by replaying recorded stages
(R9·U1), so the owner can confirm or change the per-stage picks of R1·U2. **Not in this phase:** any change to the
engine (defects found become findings for the orchestrator), the shipped defaults (R10·U1).

### What changed from the superseded plan (this wins over any older wording below)
- Tests are in the tool crate (`crates/research-tool/tests/tool/u4_accept.rs`).
- Read `run_phase.sh` launches "this phase" as "this milestone": the implementer builds tasks 1–2 and the automated rows, then the milestone is reviewed; tasks 3–6 are the orchestrator's.

### How this phase runs
Owner-attended, like P47 and P52: the orchestrator runs it with the owner available. **If launched by
`run_phase.sh`, the implementer builds only the `accept` subcommand and its two automated rows (tasks 1–2), then
reports `STATE: DONE`; the measurement run (tasks 3–6) is done by the orchestrator** and recorded in the evidence
doc and this plan's Review log. **Spending ceiling: USD 40** (`--ceiling-usd 40`; the engine's own caps
`max_cost_per_run` 0.50 and `max_cost_per_day` are set in the scratch settings so that 42 runs fit: day cap 25).

### Read first
`master_plan_research.md` (all) · book `08A` §6, §9, §11 (operational measurement), §12A, `08B` MC-A4, MC-A8 ·
evidence doc (R1·U1, R1·U2 sections, Owner picks) · the test file and `answer_key_corrections.json` (**scoring rules**:
hedged pieces 2, 26, 37, 38 count as correct at medium or low and as wrong at high; disagreements are review
candidates, listed with the agent's evidence) · R9·U1's plan · code `crates/integrations/src/lib.rs` (`App::open`
with `PSM_DATA_DIR`), `crates/testkit/src/test_app.rs` (how a session with photos and previews is built through
the services), `crates/research-tool/src/*`.

### Files
`crates/research-tool/src/{accept.rs, score.rs, main.rs, lib.rs}`, `crates/research-tool/Cargo.toml` (`psm-integrations`, `psm-infra`, `psm-research`), `crates/research-tool/tests/tool/{main.rs, u4_accept.rs}`, `docs/intention/evidence/research_providers.md` (section "Acceptance measurement", written by the orchestrator's run). Outside the repo: `~/psm-research-bench/a23/`.

### Core mechanisms (the only functions with unit tests)
None.

### Ordered tasks
1. `accept --pieces <file> --data <scratch dir> --ceiling-usd <n> [--only <n,…>]`: open the app headless on the
   scratch folder (the real `App`, the real research adapters, keys from the environment handed to an in-memory
   secrets store, never the Keychain). The shop is a **stand-in `CommerceAdapter` inside the tool**: definitions
   and layouts from the shop definitions fixture, `public_listing` from the pieces file (the shop's host, each
   published piece's URL and handle, so its own listing is excluded, MC-A3; nothing for the unpublished pieces),
   and every mutating method refusing (H1: this tool can write nothing to a shop). For each piece create an item
   and a session through the services and add its images as photos whose adjusted preview is the downloaded image
   (first image = cover; `condition_report_images` as Condition photos). The scratch settings keep the default
   attribution keys and carry the nine trusted sites in the owner's order. Let the engine run each session
   (manual trigger), then export (R7·U2).
2. `score` (extends R1·U2's): compare each run's written values with the key; attribution fields scored with the
   hedged rule; corrected-key fields in their own bucket; fields the key does not hold are not scored.
3. Run over all pieces with the picked adapters and the per-stage models of the owner's R1·U2 pick.
4. Record in the evidence doc: per group (published / unpublished) — attribution at medium or high: correct /
   wrong / hedged-correct; attribution at low; not determined; visible and condition fields correct / wrong / not
   determined; USD and seconds per piece (median, maximum); runs that ended `stopped_at_cap` or `failed`; every
   disagreement with its evidence (steps and sources).
5. Per-stage model comparison by `replay` on a sample of 12 pieces (4 per test group) for each candidate model.
6. Findings: anything that contradicts a contract (a value above `low` without the required evidence would be MA2
   broken) is reported to the orchestrator as a defect with the run id; it is not fixed here.

### Criteria (5 criteria, 6 rows, 3 `[rule]`, 3 `[manual]`)
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| A23·C1(a) | `score`, hedged rule | a hedged piece whose named designer is suggested at medium; at low; at high | correct; correct; wrong | `[rule]` corrections file `scoring_rules.hedged_attribution` |
| A23·C1(b) | `score` | an attribution the key leaves empty, suggested at medium; left not determined | wrong (listed as a review candidate with its sources); correct | MA2 |
| A23·C2(a) | `accept` (HTTP seam stub replaying R1·U1/R1·U2 captures; `FakeTransport` for the shop) | two pieces from a small pieces file | two items, two sessions, each with its photos; one run per session reaches a final state; one export folder exists per run; nothing is written outside the scratch folder | MA9 |
| A23·C3(a) | evidence doc `[manual]` | all pieces of the test file | the tables of task 4 | `[rule]` 08A §11 |
| A23·C4(a) | evidence doc `[manual]` | the 12-piece sample, each candidate model per stage | per stage and model: agreement with the key, USD and seconds per piece | `[rule]` 08A §9 |
| A23·C5(a) | Review log `[manual]` | the run | no stored attribution above `low` lacks a read label, a shop `match` or two trusted `match` sites in its steps (checked over every run's `writes.json` and steps); spend within the ceiling | MA2, HA6 |

### Delegated choices
How the shop's layout is obtained for item types the test file names (the shop's presets if any, else every
researchable definition) · sampling of the 12 pieces (stated) · table layout.

### Notes
- The photos of the test pieces are finished shop images, not the studio's previews; results on live captures may
  differ. The narrative says so in plain words.
- The owner's answer goes into the evidence doc under **Owner picks**; R10·U1 reads it.

### Unit test commands (run after the unit, then make the `CHECKPOINT (not approved):` commit)
`cargo test -p psm-research-tool u4_accept::` · `cargo clippy -p psm-research-tool --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cargo check --workspace`

## Closing gate (once, after the last unit; replaces the template's closing gate)
`cargo test -p psm-research-tool` · `cargo clippy -p psm-research-tool --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cargo check --workspace`. **No app suite** (only the tool crate changed).

## Review mode
The reviewer runs `cargo test -p psm-research-tool`, `cargo clippy -p psm-research-tool --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cargo check --workspace`; re-runs no real call; after the orchestrator's measurement, checks the evidence tables against the exports in `~/psm-research-bench/a23/`. The narrative's **Needs you** carries one card: the model and effort per stage to ship as defaults (and the page limit, if the measurement shows it). Notes: `docs/implementation/backlog_research/R9_research_tool_and_acceptance.md`.

## Manual try-out (the narrative is built from this)
1. Export research from the app (R7) and run `cargo run -p psm-research-tool -- report <the zip>`: read which stage loses the most suggestions to edits.
2. Read "Acceptance measurement" in the evidence doc: of the designer and maker suggestions the agent was fairly sure about, how many were right; how many pieces it left open; what a piece costs.
3. Open the disagreement list and look at three pieces you know well.
4. Answer the model pick in **Needs you**.

## Review log
