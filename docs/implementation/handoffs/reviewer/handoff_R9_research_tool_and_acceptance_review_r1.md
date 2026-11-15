---
plan: R9_research_tool_and_acceptance
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: OWNER_DECISIONS_PENDING
date: 2026-11-13
actor: opus (pipeline-reviewer, Opus 5.5)
---
# R9_research_tool_and_acceptance — review round 1: APPROVED_WITH_NOTES

This is the first review of the milestone: 2 units, 10 criteria, 15 rows, of which 12 are automated and 3 are `[manual]`. The authorities are flow.md §5 and §10, the plan's Review mode, and book 08A §6/§10A/§11 and 08B MC-A11 (Replay identity).
- Every automated row has a test at its boundary, and each test fails on a wrong outcome.
- No behaviour or assumption contradicts the book or the plan.
- The checks are green.

The three `[manual]` rows (A23·C3(a), C4(a), C5(a)) are the owner-attended measurement. The plan gives them to the orchestrator, and they are **still open**. After the measurement, the reviewer checks the evidence tables against `~/psm-research-bench/a23/`. Five non-blocking notes were written. One of them should be settled before task 5: `replay` has no spending ceiling.

## ⚠ OWNER DECISIONS REQUIRED (1)

**Card 1: model and effort per stage to ship as defaults**
**Question:** After the 42-piece measurement, which model and effort should each research stage ship with?
**Story:** Research is about to run on all 42 pieces with your current picks, and then compare other models on 12 of them. The tables will show, per stage and model, how often it agreed with your answer key, what a piece costs and how long it takes. Whatever you choose becomes what every new installation starts with.
**Branches:** Keep your picks if the tables agree → defaults ship as they are. · Change a stage → that stage ships the model the tables favour. · Decide later → no defaults ship; each user picks models by hand.
**Recommendation:** Decide when the tables are in. They exist to answer exactly this, and USD 40 is already set aside for them.
**On silence:** The gate holds; no default models ship.
**Trace:** 08A §9, §11; R9 plan U2 tasks 3–5 and Review mode; R10·U1 shipped defaults.

## Verified correct

**Gate.** Tracker `IMPLEMENTED`. `f69122a` (U1), `9c4f7b9` (U2) and closing `329a39d` are in `git log`. The working tree is clean, and there is no code diff `9c4f7b9..HEAD`.

**Perimeter.** `git show --stat` of each checkpoint equals the handoff's declared list. The paths outside the plan's Files lists are all disclosed or implied:
- `crates/research-tool/src/{answer_key.rs, budget.rs}`: task 5, where R1's helpers lived.
- `crates/research/src/adapters/{http.rs, page_reader/http.rs}` and `crates/research/tests/engine/adapter_pages.rs`: task 0, whose rows the plan names.
- `psm-testkit` as a regular dependency of the tool, for the definitions fixture and the editor stand-in.

**Checks.** Plan Review mode on code tree `9c4f7b9`, one cargo command at a time, foreground:

| Command | Result |
|---|---|
| `cargo test -p psm-research-tool` | 26 passed, 0 failed |
| `cargo clippy -p psm-research-tool --all-targets -- -D warnings` | clean |
| `cargo fmt --all -- --check` | exit 0 |
| `cargo check --workspace` | ok |

The implementer's research-crate stamp is consumed, because the tree is the same: `cargo test -p psm-research` 171+120+1, clippy clean, and app seams r7/r8 11/11. No app file changed, so there is no app suite (flow §10).

**Rows** (the production code read first, then its test):

| Row | Test | Why it discriminates |
|---|---|---|
| A22·C1(a) | `a22_c1a_report_counts_actions_and_stage_cost` | The real engine run writes 5 fields. The actions go through `AttributesService` (accept ×2, `set_value` Human, clear, none). The test asserts the exact tuple (2,1,1,1) and kept 0. Per-stage cost is checked against Σ `cost_micro_usd` per stage from the store. Costs are non-zero (the harness fallback is 0.001 USD/call), so stage mis-assignment changes the sums. |
| A22·C1(b) | `a22_c1b_report_attribution_right_and_wrong` | It asserts the exact verdict list `[(run, designer, Right), (run, maker, Wrong)]` and the counts (1,1,0). |
| A22·C2(a) | `a22_c2a_replay_sends_the_recorded_body` | The stub's body is byte-equal to the body the real `OpenAiModel` put on the wire through `CallGateway`, with 2 data-URL images. `replay::rebuild` was read against `adapter.rs::bodies`: same field order, data URL form and listing images. `export.rs` copies every referenced sha (listing images included) to `images/`. |
| A22·C2(b), C2(c) | `a22_c2b_…`, `a22_c2c_…` | Each asserts whole-JSON equality with exactly one field replaced, plus `≠ original`. |
| A22·C3(a) | `a22_c3a_…` | Three real exports: photo deleted (withheld), image file removed, and calls purged at +31 days. Each gives an exact `not replayable: <reason>` line, and the stub is empty each time. |
| A22·C4(a) | `a22_c4a_…` | Exit ≠ 0, empty output, stub empty. The key check runs before the export is even read (`lib.rs`). |
| A22·C4(b) | `a22_c4b_…` | The key reached the `authorization` header (so the check is live). It is absent from stdout, from the one `--out` file and from the export folder. |
| A22·C5(a) | `a22_c5a_…` | The folder is built by R1's `Caller`. The test asserts the exact group lines `serpapi_lens 2 0.003000`, `openai m1 1 0.002250` and `total 3 0.005250`. |
| task 0 charset | `r9_t0a_…` | windows-1252 by header and ISO-8859-1 by `<meta>`. The text contains "mörk" and the title is "Byrå". UTF-8-lossy decoding would fail both. |
| task 0 non-HTML | `r9_t0b_…` | An `application/pdf` reply with an HTML-looking body gives `obtained: false` and empty text. |
| task 0 ranges | `r9_t0c_…` | 14 addresses in the new ranges are refused and 9 adjacent public ones pass. The `.no_proxy()` change was read; it has no row. |
| A23·C1(a) | `a23_c1a_hedged_attribution_rule` | The real key gives piece 2 hedged, "Magnus Olesen". At medium/low/high the result is exactly (Correct, Correct, Wrong), which matches corrections `scoring_rules.hedged_attribution`. |
| A23·C1(b) | `a23_c1b_attribution_the_key_leaves_empty` | Piece 27 has no attribution. The suggestion at medium is Wrong, a review candidate, with both sources and the step evidence line. Nothing written is Correct. |
| A23·C2(a) | `a23_c2a_accept_runs_two_pieces_in_the_scratch_folder` | The real `App` runs with the real adapters over `FakeResearchHttp`. The test asserts the exact SKUs, the per-session photo counts (3,2,1), one run per session in `FINAL_STATES` (the row's own outcome is "a final state"), and one export per run containing exactly that run id. Nothing is written beside the scratch folder. The reviewer also confirmed that no `~/Pictures/Session Manager` was created by the run. |

**Structural reads.**
- `StandInShop`: every mutating `CommerceAdapter` method returns the H1 refusal. Reads are empty or not-found, and `public_listing` comes from the pieces file (MC-A3 own listing).
- The ceiling is checked before each piece as spent + one run's cap > ceiling, and the store sum carries across resumed invocations.
- `folder_key("A23-NN")` keeps the piece number that `score_data::piece_of` parses.
- `report` uses `psm_core::mechanisms::sameness` per definition.
- R1's private host and micro-USD helpers now call `psm_research::rules::{sites, cost}` (task 5).

**Assumptions.** None of them contradicts the book.
- #3 (attribution keys `custom.designer` / `custom.manufacturer`) matches 08A §6, "initially the shop's designer and manufacturer keys", for the fixture shop that the answer key matches.
- #1, #2 and #4–#6 lie within Delegated choices and are reported in the narrative.

**H1–H8 and the standing rules.**
- H1: the stand-in shop refuses all writes.
- H2: placeholder RAWs exist only under `<data>/pieces/`.
- H3: the adjusted previews are made by the app's thumbnail function, which is allowed.
- H4: `accept` skips pieces already in the scratch store.
- H5: fake scripting only, and no `osascript`.
- H6: n/a.
- H7: no `crates/core` file changed, and the HA5 literals are only in research adapters and the tool.
- H8: no Tauri.
- Keys: the environment feeds an in-memory `SecretStore` (never the Keychain), and the replay text and file are scanned.
- Rule 9: no suite calls a live service.

## Findings

None blocking (no B1–B4).

## Notes written to backlog
`docs/implementation/backlog_research/R9_research_tool_and_acceptance.md` (the plan's named file for reviewer notes):
1. **`replay` has no spending ceiling**, although `master_plan_research.md` §3 says "enforced by the tool". Settle this before task 5 by adding `--ceiling-usd` to replay, or by budgeting task 5 by invocation count.
2. **The carried R1 instruction was not done and not reported:** "select error fixtures by HTTP status" (`fixtures.rs` still picks by label). No test reads that fixture, so it is not visible.
3. **No offline test sees a value-writing `accept` run.** The R1 observe capture fails the strict schema, so runs end `failed`. `score-accept` is never run over a real export, so at the first real run, check that `scores.json` has 42 runs.
4. **`accept`'s ceiling covers one scratch folder.** The USD 40 total across `--settings` comparison folders is the orchestrator's to sum.
5. **For R10:** measured attribution keys `custom.*` vs the shipped default `beyo.designer`.

## Carry-forward dispositions

| Item | Destination |
|---|---|
| [manual] A23·C3(a), C4(a), C5(a); evidence tables vs exports | orchestrator's measurement, then a reviewer check |
| Note 1 (replay ceiling) | orchestrator, before U2 task 5 |
| Note 3 (check `scores.json` after the first real run) | orchestrator, U2 task 4 |
| Note 4 (ceiling across folders) | orchestrator, U2 tasks 3–5 |
| Note 2 (fixture pick by status) | next session that touches `fixtures.rs` / R10 |
| Note 5 (attribution keys) | R10·U1 |
| Card 1 (model pick) | owner, after the measurement tables |

## Lessons for the plans
- A row whose outcome is "reaches a final state" is satisfied by a run that fails at its first stage. When a fixture cannot drive the engine past observe, the row proves only the plumbing. A future plan could require "≥ 1 field written" with an engine-schema capture.
- A "Carried from earlier reviews" bullet without a row and without a file in the unit's Files list is easy to drop silently (the R1 fixture-pick item). Carried items should be given a row or a file entry.

## Write perimeter of this session
- `docs/implementation/plans/R9_research_tool_and_acceptance.md` (Tracker line, Review log entry)
- `docs/implementation/backlog_research/R9_research_tool_and_acceptance.md` (new)
- `docs/implementation/narratives/R9_research_tool_and_acceptance.md` (new)
- this handoff

No production or test file was edited. No probes were run. No commits were made.
