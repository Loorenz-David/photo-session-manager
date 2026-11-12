---
phase: A22_research_report_and_replay_tool
title: The dev-Mac tool — from an exported research folder it reports, stage by stage, what was accepted, edited or cleared and whether attributions were right, and it can replay one stage with another model or instructions to see the difference
difficulty: normal
projection: yes
depends_on: [A21_research_retention_and_export, A01_research_probes_and_search_benchmark]
---
# A22 — `psm-research report` and `replay` (08A §10A "On the dev Mac"; 08B MC-A11 Replay identity; MA9, MA16)

## Goal
Grow A01's tool into the dev-Mac tool of 08A §10A. `report <export>` prints, per stage and overall, suggestions
accepted unchanged, edited, cleared, not determined, attribution right or wrong against the photographer's final
value, cost and time. `replay <export> --run <id> --stage <stage> [--model …] [--effort …] [--instructions <file>]`
rebuilds each recorded request of that stage, substitutes only what the command line names, sends it with the
tool's own key from the environment, and shows the difference to the recorded answer. It never touches the work
Mac or the app's data. **Not in this phase:** running the engine over the test pieces (A23).

## Offline part / keyed part
Everything in the criteria is offline (an HTTP seam stub). A real replay is one manual step at the end, with
`OPENAI_API_KEY` from `~/.psm_research.env`, capped at USD 2; if the key file is absent the step is skipped and
said so. A01 must be APPROVED only because the tool crate and its recording reader come from it; if this phase is
reached before A01 has run, the orchestrator runs A01 first.

## Read first
`master_plan_research.md` §3, §6.1, §7.2 · book `14` (MA9, MA16), `08A` §10A, `08B` **MC-A11** (Export layout,
Exclusions, **Replay identity**), MC-A9 (record fields) · A01's plan (tool, recording layout, temporary helpers) ·
code `crates/research-tool/src/*`, `crates/core/src/services/research/export.rs` (the file shapes),
`crates/core/src/mechanisms/research/{sites.rs, cost.rs, sameness.rs}`.

## Dependencies and gate
Tracker row and gate: `master_plan_research.md` §4. Notes: `backlog_research.md`.

## Files expected to change
`crates/research-tool/src/{main.rs, lib.rs, report.rs, replay.rs, export_reader.rs}` and the removal of A01's
private host and cost helpers in favour of `psm_core`; `crates/research-tool/Cargo.toml` (`psm-core`);
`crates/testkit/tests/suite/research/a22_research_report_and_replay_tool.rs` (+ `mod` line); own tracker row.

## Core mechanisms
None.

## Ordered tasks
1. `export_reader`: read an unzipped (or zipped) export; the same reader reads A01's bench recordings.
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
5. Replace A01's private host matching and micro-USD sum with `psm_core::mechanisms::research::{sites, cost}`.

## Criteria (5 criteria, 9 rows, 1 `[rule]`)
Boundary: the tool's library functions and command line over an export produced in the test by A21's `export` from
a scripted run (so the fixture is the real export shape, built by production code).
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | `report` | an export of one run that wrote five fields; the photographer accepted two, edited one, cleared one, left one | counts: accepted unchanged 2, edited 1, cleared 1, not acted on 1; the per-stage cost equals the sum of that stage's call costs | MA9 |
| C1(b) | `report`, attribution | designer suggested A and accepted; manufacturer suggested B and edited to C | designer right, manufacturer wrong | MA9 |
| C2(a) | `replay`, no substitution | a recorded observe call with two images | the body the HTTP stub receives is byte-equal to the body rebuilt from the record with the two images restored | MA16 |
| C2(b) | `replay --model m2` | the same call | the received body differs from C2(a)'s only in the model field | MA16 |
| C2(c) | `replay --instructions file` | the same call | the received body differs only in the instructions field | MA16 |
| C3(a) | not replayable | a call whose image was withheld (photo deleted); a call whose image file is missing from `images/`; a run marked `call record expired` | each is reported `not replayable:` with its reason; the stub received no request for them | MA9 (R-IA26) |
| C4(a) | key | `replay` with no key in the environment | exits non-zero before any request; the stub received nothing | MA16 |
| C4(b) | key | a successful replay's printed output and any file it writes | the key value appears in none | MA16 |
| C5(a) | bench recordings | `report` pointed at a folder in A01's recording layout (built in the test with A01's `recording` writer) | it reads the calls and prints cost and time per call group without error | `[rule]` 08A §10A |

## Delegated choices
Output format of `report` (plain text tables; `--json` if cheap) · how the side-by-side difference is printed ·
flag names beyond those in the goal.

## Manual try-out
1. Export research from the app (A21) and copy the zip to the dev Mac.
2. `cargo run -p psm-research-tool -- report ~/Desktop/research-export-….zip` — read which stage loses the most
   suggestions to edits.
3. `set -a; . ~/.psm_research.env; set +a; cargo run -p psm-research-tool -- replay <export> --run <id> --stage observe --model <another model> --ceiling-usd 2`
   — see the recorded answer beside the new one.

## Notes
- The tool's dependency on `psm-core` is one-way; nothing in the app depends on the tool.
- Review mode: this phase's module, `research::a01…::`, `research::a21…::`, clippy, fmt. The reviewer re-runs no real call.

## Review log
