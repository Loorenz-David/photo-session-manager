---
plan: R9_research_tool_and_acceptance
role: implement
round: 1
state: DONE
date: 2026-11-13
actor: opus (Claude Opus 5.5, pipeline-implementer)
---
# R9 implement r1: report, replay, accept

## What was built
**For the photographer / owner.** On the dev Mac, `psm-research report <export zip or folder>` reads an
Export research file from the app and prints, per stage and overall, how many suggestions were accepted
unchanged, edited, cleared, kept previous, left alone or not determined; for the designer and maker fields,
whether each suggestion was right or wrong against what the photographer kept; and the cost and time of
each stage. It also reads the benchmark's recording folders (cost and time per call group).
`psm-research replay <export> --run <id> --stage <stage> [--model m] [--effort e] [--instructions file]`
sends the recorded model calls of that stage again, changing only what you name, and shows the recorded and
the new answer side by side with the differences. A call whose photo was deleted, whose image is missing
or whose record expired is reported "not replayable" and never sent. The key is only `OPENAI_API_KEY` from
the environment; without it nothing is sent.
`psm-research accept --pieces <file> --data <scratch dir> --ceiling-usd 40` runs the finished research agent
on the owner's test pieces in a scratch data folder (real app, real adapters, keys from the environment, a
read-only stand-in shop that refuses every write), one run and one export per piece; `psm-research
score-accept --data <dir>` turns the exports into the numbers of 08A §11 (`<data>/scores.json`).
**Before the measurement (task 0):** the page reader now reads Swedish dealer pages in windows-1252 /
ISO-8859-1 correctly, treats a PDF or other non-HTML reply as "could not be read", ignores any environment
proxy for page reading, and refuses more private and special address ranges.

**Code.** U1: `crates/research-tool/src/{export_reader.rs, replay.rs}` (new), `report.rs` (export report and
bench report appended), `lib.rs` (`report`, `replay` commands; `run_with` takes the environment lookup and the
output writer), `answer_key.rs` / `budget.rs` (R1's private host match and micro-USD parse/sum now call
`psm_research::rules::{sites, cost}`); `crates/research/src/adapters/{http.rs, page_reader/http.rs}` (task 0).
U2: `crates/research-tool/src/accept.rs` (new: `StandInShop`, `MemorySecrets`, scratch settings, the run loop,
`score_data`), `score.rs` (`score_attribution`, `score_run`, `FieldScore`), `lib.rs` (`accept`, `score-accept`).
**The measurement run itself (U2 tasks 3–6) is not done here**: the plan gives it to the orchestrator with the
owner (real keys, ceiling USD 40).

## Tests → criterion rows
Test program `crates/research-tool/tests/tool/` (`cargo test -p psm-research-tool`); U1's fixture is a real
export written by `ResearchService::export` from runs on the research harness (included by path).

| Row | Test | Assertion shape |
|---|---|---|
| A22·C1(a) | `u3_report_and_replay::a22_c1a_report_counts_actions_and_stage_cost` | engine run writes 5 fields; accept 2, edit 1, clear 1, leave 1 through `AttributesService` → overall counts exactly (5; 2, 1, 1, 1; kept 0); per-stage cost = Σ that stage's `cost_micro_usd` from the store; overall cost = Σ all calls |
| A22·C1(b) | `…::a22_c1b_report_attribution_right_and_wrong` | per-key verdicts exactly `[(run, custom.designer, Right), (run, custom.maker, Wrong)]`; counts (1, 1, 0) |
| A22·C2(a) | `…::a22_c2a_replay_sends_the_recorded_body` | the stub's body is byte-equal to the body the real OpenAI adapter sent for the recorded two-image observe call; two images restored |
| A22·C2(b) | `…::a22_c2b_replay_model_substitutes_only_the_model` | received JSON == original JSON with `model` = `m2` (and ≠ original) |
| A22·C2(c) | `…::a22_c2c_replay_instructions_substitutes_only_the_instructions` | received JSON == original JSON with `instructions` = the file's text |
| A22·C3(a) | `…::a22_c3a_withheld_missing_and_expired_are_not_replayable` | three exports (photo deleted; image file removed; calls purged): exact `not replayable: <reason>` line each; stub received nothing |
| A22·C4(a) | `…::a22_c4a_replay_without_a_key_sends_nothing` | exit ≠ 0, nothing printed, stub received nothing |
| A22·C4(b) | `…::a22_c4b_the_key_is_in_no_output_and_no_file` | the key reached the request header; printed output, the `--out` file and the export contain it nowhere |
| A22·C5(a) | `…::a22_c5a_report_reads_bench_recordings` | folder built by R1's `Caller`; `report` exit 0; group lines `serpapi_lens 2 0.003000`, `openai m1 1 0.002250`, `total 3 0.005250` |
| task 0 (charset) | `crates/research/tests/engine/adapter_pages.rs::r9_t0a_a_windows_1252_page_reads_mork` | windows-1252 by header and ISO-8859-1 by `<meta charset>`: text contains "mörk", title "Byrå" |
| task 0 (non-HTML) | `…::r9_t0b_a_pdf_reply_is_not_obtained` | `application/pdf` → `obtained: false`, empty text, one `page_fetch` |
| task 0 (ranges) | `…::r9_t0c_added_address_ranges_are_refused` | table: 14 addresses in the new ranges refused, 9 neighbours pass |
| A23·C1(a) | `u4_accept::a23_c1a_hedged_attribution_rule` | piece 2 (hedged, real key) "Magnus Olesen" at medium / low / high → Correct, Correct, Wrong |
| A23·C1(b) | `u4_accept::a23_c1b_attribution_the_key_leaves_empty` | piece 27 (no attribution): designer suggested at medium → Wrong, review candidate, both sources, the naming step as evidence; nothing written → Correct |
| A23·C2(a) | `u4_accept::a23_c2a_accept_runs_two_pieces_in_the_scratch_folder` | pieces 1 and 31: items `A23-01`, `A23-31`; two sessions with 3 photos each (2 main, 1 condition); one run per session, final state; one export per run holding exactly that run; beside the scratch folder only the test's pieces file |
| A23·C3(a), C4(a), C5(a) | `[manual]` | the orchestrator's measurement (tasks 3–6) |

Every test in the two new test files discharges a row above; none is extra. Red baseline: not captured
(tests written alongside the code).

## Assumptions
1. **Report counting** (delegated output, rule needed): a suggestion = a `writes.json` entry with outcome
   written / unchanged / lowered; its action = the first feedback row for the key whose suggested value is
   the suggestion under `S`; it counts in every stage whose steps name its key (stage 7 if none);
   "not determined" leaves out "not researched". Right = the photographer acted and the final value is the
   suggestion under `S`.
2. **Replay scope:** model calls only; visual-search and plain page-download calls are listed "skipped"; a
   run marked "call record expired" is not replayable as a whole; only provider `openai` can be rebuilt.
3. **Accept's attribution keys are `custom.designer` and `custom.manufacturer`**, not the shipped default
   `beyo.designer`: the definitions fixture that matches the answer key (the owner's 68 metafields) has no
   `beyo.designer`, so the default would make the attribution measurement empty. The orchestrator can
   override with `--settings` if it reads the plan differently.
4. **Accept's models:** observe Sol low, read Terra low, condition Terra high (Owner picks); compare and
   labels Sol low, decide Terra low (not in the picks); SerpApi and page download priced 0; `auto` off.
5. **Layout:** every researchable definition of the fixture for every item type (the fixture has no presets).
6. All piece images are Main photos (first = cover); condition report images are Condition photos.

## Deviations
- `psm-testkit` is a regular dependency of the tool (definitions fixture, editor stand-in), beside the
  three the plan names.
- Task 5 edited `answer_key.rs` and `budget.rs` (outside U1's file list; that is where R1's helpers were).
- A23·C2(a)'s "`FakeTransport` for the shop": the stand-in shop has no transport; nothing to fake.
- The one manual replay (USD 2) was **not run**: `~/.psm_research.env` is present, but no export with a real
  OpenAI call exists yet; it fits after the measurement produces exports.
- Offline, accept's runs end `failed` (R1's observe capture does not fit the engine's strict schema; after
  three schema failures the engine fails the run, MC-A9). The row asks for a final state; it holds.

## Files changed (write perimeter)
U1 (`f69122a`): `Cargo.lock`, `crates/research-tool/Cargo.toml`, `crates/research-tool/src/{answer_key.rs,
budget.rs, lib.rs, report.rs, export_reader.rs, replay.rs}`, `crates/research-tool/tests/tool/{main.rs,
u3_report_and_replay.rs}`, `crates/research/src/adapters/http.rs`, `crates/research/src/adapters/page_reader/http.rs`,
`crates/research/tests/engine/adapter_pages.rs`.
U2 (`9c4f7b9`): `Cargo.lock`, `crates/research-tool/Cargo.toml`, `crates/research-tool/src/{lib.rs, score.rs,
accept.rs}`, `crates/research-tool/tests/tool/{main.rs, u4_accept.rs}`.
Closing commit: `docs/implementation/plans/R9_research_tool_and_acceptance.md` (Tracker line, Review log) and
this handoff. No mutation probes (flow §4). No architecture graph in this project. Nothing written outside
the repo (no `~/psm-research-bench/a23/` yet).

## Checkpoint SHA
U1 `f69122a` · U2 `9c4f7b9` · closing: the commit carrying this file
(`CHECKPOINT (not approved): R9 closing tracker, review log and handoff`).

## Stamp
Tree: `9c4f7b9` (code identical to the closing commit). Run once, one after the other:
- `cargo test -p psm-research-tool` → 26 passed, 0 failed.
- `cargo clippy -p psm-research-tool --all-targets -- -D warnings` → clean.
- `cargo fmt --all -- --check` → clean.
- `cargo check --workspace` → clean.
- Because task 0 changed the research crate: `cargo test -p psm-research` → 120 + 171 + 1 passed;
  `cargo clippy -p psm-research --all-targets -- -D warnings` → clean;
  `cargo test -p psm-testkit --test suite -- research::r8_shop_facts:: research::r7_wiring::` → 11 passed.
No app suite (flow §10: no app file changed). No pre-existing failures seen.

## Manual try-out
1. In the app on scratch data, Settings → Research → Export research (a date range), then on the dev Mac:
   `cargo run -p psm-research-tool -- report ~/Downloads/research-export-<stamp>.zip` — read which stage
   loses the most suggestions to edits; add `--json` for the raw numbers.
2. `set -a; source ~/.psm_research.env; set +a; cargo run -p psm-research-tool -- replay <the zip> --run <id> --stage observe --model gpt-6.1-terra`
   — each call shows "recorded" and "new" and the fields that differ (costs a few cents).
3. The measurement (orchestrator with the owner): `set -a; source ~/.psm_research.env; set +a; cargo run -p psm-research-tool -- accept --pieces docs/test-items/beyo_test_samples_1.json --data ~/psm-research-bench/a23 --ceiling-usd 40`
   (`--only 1,2,3` for a first few), then `cargo run -p psm-research-tool -- score-accept --data ~/psm-research-bench/a23`
   and copy the tables from `scores.json` into the evidence doc.
4. Model comparison (task 5): `replay` on 12 pieces' exports per candidate model, or a second `accept` into
   another scratch folder with `--settings` naming other stage models.

## ⚠ OWNER DECISIONS REQUIRED (0)
None. (Assumption 3, the attribution keys of the measurement, is the orchestrator's to confirm from 08A §6.)
