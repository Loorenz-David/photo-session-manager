---
plan: R4_research_engine
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-12
actor: opus
---

# R4 research engine — review r1

**Verdict: APPROVED_WITH_NOTES.** All 99 criterion rows (39 criteria, 6 units) hold at their boundaries. The production code follows 08B MC-A1, MC-A3, MC-A4, MC-A5, MC-A6, MC-A7, MC-A8 and MC-A9, and the names and sentences in `master_plan_research.md` §6.6. There are no blocking findings (B1–B4). Ten notes went to `docs/implementation/backlog_research/R4_research_engine.md`, the folder the plan names for reviewer notes (not `backlog.md`).

## ⚠ OWNER DECISIONS REQUIRED (0)

None. Nothing needs the owner.

## Verified correct

**Gate.** The Tracker line read IMPLEMENTED. Checkpoints exist: U1 `d536832`, U2 `28411b2`, U3 `cb16665`, U4 `d891bf8`, U5 `e4f24c9`, U6 `620b2ac`, closing `6d53a3f`.

**Perimeter.** `git diff --stat d536832^ 620b2ac` lists exactly the handoff's declared files: `Cargo.lock`, `crates/research/Cargo.toml`, `src/{adapters/mod, api_impl, config, decide, domain, fakes, files, gateway, images, job, lib, ports, rules/mod, rules/run_rules, runs, schema, service, shop, stages, store, writes}.rs`, `tests/engine/{api, gateway, harness, images_and_storage, lifecycle, main, resume, stages}.rs`. The closing commit holds only the plan's Tracker line and Review log, plus the implementer handoff. The declared cross-unit deviations (`config.rs` in U2, enums in `domain.rs` in U1, `writes.rs` in U3) are in the commits that say so. No app file changed.

**Checks.** Each was run once, in the plan's Review mode, on the clean tree at `873e928`. That tree equals `6d53a3f` plus the review prompt.

| Command | Result |
|---|---|
| `cargo test -p psm-research` | lib 120 passed · engine 122 passed (34 s) · doc 1 passed; 0 failed |
| `cargo clippy -p psm-research --all-targets -- -D warnings` | clean |
| `cargo fmt --all -- --check` | exit 0, no output |
| `cargo check --workspace` | clean |
| variation: `TZ=Pacific/Chatham cargo test -p psm-research --test engine lifecycle::a13_c8a` | passed. The day limit lifts at local midnight in a UTC+12:45 zone too. |

There was no app suite, because no app file changed (flow §10).

**Per unit (production code read first, then the row's test).**
- **U1 (9 rows).**
  - `ResearchImage` has private fields and no constructor outside `images.rs`. The `compile_fail` doc test guards the type.
  - Eligibility means: the photo is in the run's session, that session belongs to the run's item, and the photo is not `deleted`. `Classification` has only four variants, so this equals MC-A5's main/condition/unreviewed list.
  - The source is `adjusted_preview_path`, used only while `rendered_gen ≥ adjusted_gen` and while the file exists. The original preview is never read.
  - There is one stored copy per (run, photo, generation, revision, long edge).
  - The engine tables and both partial unique indexes match the plan.
- **U2 (18 rows).**
  - Step 1 commits before `send` and holds no transaction across it. That transaction re-reads eligibility, applies the stop rule against the run's snapshot, and inserts the `sent` row and the outbound rows.
  - The reply, its cost and the run's cost commit in one transaction.
  - An offline call is marked `not_sent`, costs 0 and loses its outbound rows. `next_attempt` skips `not_sent` calls.
  - Retries wait 1 s, then 2 s, on the injected waiter.
  - The key class fails the run with `key rejected by <provider>`.
  - Redaction covers the request, the response, the error text and the warn log. One error line is written per call.
  - Listing images are stored and referenced, and they never get outbound rows.
- **U3 (24 rows).**
  - Unit identities match the MC-A7 table for all seven stages.
  - Reading follows MC-A3 order and stops once `max_pages` pages are *obtained*. The own listing is never read. A page's site comes from its final URL.
  - Decide uses R3's `decide_*`. The model's claimed confidence is never read.
  - Writes, `concluded`/`not_determined` steps and the stage 7 output commit in one transaction through `writes::suggest_in`/`lower_in`. Those two functions are a pure extraction of R3's bodies (diff read).
  - The endings `SkipToDecide`, `EndAtCap` and `RunFailed`, and a refusal, behave as task 5 says.
- **U4 (20 rows).**
  - The order of `first_failing` and the sentences equal §6.6. No run row is created while one of the first five prerequisites fails.
  - `may_start` checks the quiet period (manual runs skip it), readiness, the day cap and one running run.
  - The start transaction writes the settings, form and input snapshots and the `research` job row.
  - The `ResearchJob` handler sets no `max_attempts`, so an offline run is retried indefinitely with back-off capped at 60 s. It cannot be left stranded.
  - Auto-off cancels queued auto and follow-up runs.
- **U5 (17 rows).**
  - A committed unit is never re-run.
  - `close_open_calls` turns `sent` into `unknown` and charges the fallback.
  - Reuse happens only in auto and follow-up runs, matches by fingerprint, requires the files to be present, and never applies to Decide.
  - `follow_up_needed` compares the three identity sets regardless of order. MC-A6's follow-up row applies.
- **U6 (11 rows).**
  - The derived counts are computed when asked, never stored.
  - `run` equals `run_again`.
  - Keys go only through `SecretStore`. `secrets_state` returns booleans only.
  - The site methods return the new settings value, and the refusal text is verbatim.
  - `validate` refuses `0,5`, `0.0000001` and `record_days` 1.

**Tests.** Every row's test asserts the row's exact outcome at its boundary and would fail on a wrong outcome:
- The resume rows compare full end states (outputs, steps and attribute rows) with a reference run on a second harness.
- A12·C7(b) would turn red if `execute` read live settings, because the hook empties the live trusted list.
- A11·C5(a) asserts the 0/1 000/3 000 ms start times.

No orphan tests. The inline `run_rules` tests are the plan's named core mechanisms.

**Hard constraints H1–H8 and the secrets rules.**
- Nothing is written to an external platform.
- No RAW or master file is read. Pixel work goes only through R2's `render_research` on the adjusted preview (H3 as amended).
- H4 is covered by U5.
- The crate depends on `psm-core` and `psm-infra` only: no Tauri, no HTTP, no `psm-integrations` (H8, §3 isolation).
- Nothing was added to core (H7).
- The tests use `FakeSecrets` and the fakes; no live service is contacted.

**Assumptions.** None of the 26 contradicts the intention.
- Assumption 16 (saving keys queues no run) sits on a tension inside 08B: MC-A6 says eligibility is evaluated "on every save of a research setting or key", while MC-A7 says a setting "never starts" a run. It is noted for the book.
- Assumption 1 (default `attribution_keys` = `["beyo.designer"]`) is what row A13·C1(d) forces. The shop fixture has no manufacturer field (checked: 50 definitions, `beyo.designer` is the only designer or maker key).
- Assumption 8 (`not_sent` outbound rows deleted) follows MA3's intent. MA12's literal wording is noted for R9.

The narrative carries all 26 in plain words.

## Findings

No blocking findings (B1–B4). No should-fix item rises to a block under flow §5.

## Notes written to backlog

All in `docs/implementation/backlog_research/R4_research_engine.md`:
1. Saving keys, or switching Automatic back on, queues no run for a session with a Main photo and no run (book tension MC-A6/MC-A7; R7·U1).
2. A `not_sent` call has a recorded request but no outbound rows (MA12 wording; R9·U2 checker).
3. A run stopped at the cap lowers earlier suggestions it never got to re-check (08B MC-A8/MC-A1).
4. "n fields filled" shows 0 after a follow-up that confirms the form (R6·U1).
5. "Waiting for Capture One" shows before the 120 s source wait ends (R6·U1).
6. One running run app-wide relies on a single caller of `tick` (R7·U1).
7. The retry budget is one try shorter after a resume (`unknown` counts as attempt 1).
8. The own listing is read from any integration's link (R7·U1).
9. `a11_c4a` hard-codes `research_calls`' column count (over-engineering test).
10. The registry's default `attribution_keys` differs from the shipped default (§6.6, R10·U1).

## Carry-forward dispositions

| Note | Destination |
|---|---|
| 1 | orchestrator → book 08B (MC-A6/MC-A7 wording), then R7·U1 |
| 2 | R9·U2 (acceptance checker) / book 14 MA12 wording |
| 3 | orchestrator → book 08B (MC-A8 stop rule × MC-A1 lowering) |
| 4, 5 | R6·U1 (panel) |
| 6, 8 | R7·U1 (wiring) |
| 7 | R8 (adapters, MC-A9 reading of attempts) |
| 9 | next touch of `tests/engine/gateway.rs` (R7·U2) |
| 10 | `master_plan_research.md` §6.6 / R10·U1 |

## Lessons for the plans

- A14·C1(a) names a kill "before its record is committed", but the boundary it prescribes (a fake's `before_call` hook) is reached only after the record commits. That kill point is covered by construction, because nothing persists before step 1 commits. The row should name the point a hook can reach.
- A registry literal (§6.6 `attribution_keys`) disagreed with a row's fixture demand (A13·C1(d)). Registry defaults should be checked against the fixtures their rows name before a plan is final.

## Mutation-probe declaration

No mutation probes (flow §4). No production or test file was edited, even temporarily. The only extra run was the `TZ` variation above.

## Write perimeter

- `docs/implementation/plans/R4_research_engine.md`: the Tracker line, and the Review log entry "review r1".
- `docs/implementation/backlog_research/R4_research_engine.md` (new).
- `docs/implementation/narratives/R4_research_engine.md` (new).
- This handoff.

There is no archgraph state. Nothing was committed; the runner commits these files.
