---
plan: R7_research_wiring_and_export
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-12
actor: opus (pipeline-reviewer, Opus 5.5)
---
# R7_research_wiring_and_export — review round 1: APPROVED_WITH_NOTES

All 17 plan rows (U1 4, U2 13) and the carried rows (MA13 one-run, settings save, redo release, capped run,
Product step) hold at their boundaries. Each test observes its row's outcome and would fail on a wrong one. The
remaining carried items (`external_id`, `wait_ends_at`, `conflict` flag, Accept all count) were read in the code and
are correct. No B1–B4. The perimeter equals the declared files, and the review-mode checks are green. The
implementer's full-suite stamp stands. Five notes went to the research backlog.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Evidence
| What | Scope | Command | Tree | Result |
|---|---|---|---|---|
| Gate | — | plan Tracker line; `git log` | `937b541` | IMPLEMENTED; `4a1de73` `b4a0464` `ceb956f` present |
| Perimeter | — | `git show --stat` per checkpoint | same | = handoff's U1 (31) / U2 (18) / closing (2) lists; extras are the declared Deviations |
| R7 app rows | L1 | `cargo test -p psm-testkit --test suite research::r7_` | clean `937b541` | 8/8 |
| Touched app areas | L2 | `cargo test -p psm-testkit --test suite -- p16_operations:: p21_backend_robustness:: p09_local_export:: p01_storage_identity:: p61_storage_cleanup_and_budget::` | same | 59/59 |
| Engine | L2 | `cargo test -p psm-research --test engine` (whole binary: export + carried `stages::`/`writes::` rows) | same | 135/135 |
| Variation | L1 | `TZ=America/Los_Angeles` and `TZ=Pacific/Kiritimati` `cargo test -p psm-research --test engine export::a21_c2a` | same | 1/1 each |
| UI | L2 | `npx vitest run src/features/settings src/features/form` | same | 20 files, 108/108 |
| Typecheck, clippy, fmt | — | `npm run typecheck`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo fmt --all -- --check` | same | clean |
| Full suite | L4 | consumed: implementer stamp (handoff §Stamp, tree `b4a0464`; `ceb956f` docs only) | match | not re-run |

## Verified correct
- **A13·C1(c)**: the wired app with fakes only (no shop, no key) produces the exact `p09` export names, last progress
  (3,3,Done), no runs, no calls and no fake requests. The core workflow is untouched (H6).
- **A13·C2(a)**: classify Main → `HistoryChanged` → observer note → research turn → `session_changed` → one
  queued `auto` run for (session, item). It fails if the observer or the wiring is missing.
- **A13·C8(c)**: `requeue_running_jobs` runs at launch only (`lib.rs:567`), before the research thread spawns. After
  the restart there is one due `research` job, and the run ends `finished`.
- **A13·C8(d)**: a research job blocked in the fake call on its own thread; publish reaches `Done` while the run is
  still `running`. The main runner excludes `research`.
- **MA13 (carried)**: the turn lock plus `start_run … WHERE state='queued' AND NOT EXISTS running`. The test asserts
  that every provider call sees exactly one running run. The guard also covers a settings reopen, while the old
  detached research thread and the new one coexist.
- **Settings (carried)**: the patch is validated before the merge; `settings_changed` runs after apply; the refusal
  text is shown and the file is unchanged. Site commands save the engine's returned value.
- **Release (carried)**: core services return `Released` and emit `SessionChanged` + `PublishProgress{Prepare}`
  after commit, for attribute actions, undo and redo. The store reads in the command layer are gone.
- **Capped run / Product step / external_id / wait_ends_at / conflict / Accept all (carried)**: read in the code;
  correct. The capped-run and Product-step rows are tested in the engine.
- **A21·C1(a)–(d)**: the `≤` boundary matches the plan. Orphan SHAs are the expired calls' SHAs minus those of
  remaining calls (request and response). Only `research_calls` rows are deleted. Purge runs at launch
  (`build_research`) and then hourly in the turn; the C1(d) test shows a call kept past its own expiry until the hour.
- **A21·C2–C6**: local-date and item selection; exact tree, with zip entries byte-equal to the files; images equal
  to the stored copies; call texts equal to the record; feedback action/suggested/final;
  `call record expired`; `{"withheld":"photo deleted"}` with the image absent; a secret hit names the file without
  the value and leaves the destination empty; only `.json`/`.jpg`, none equal to a RAW.
- **A21·C7(a)**: the UI test sends a range and a folder to `exportResearch` and shows the returned path.
- **H1–H8**: walked against the diff, no violation. Only research copies are removed or copied (H2). `lib.rs`
  constructs and connects, and the commands are thin (H8). No secret is committed or logged, and no test touches a
  live service.
- **Orphans**: none. Every new test maps to a plan row or a carried row, and the R2 desktop test rename is declared.
- **Assumptions**: none contradicts 08B MC-A6/A9/A11. The owner narrative restates them in plain words.

## Findings
None blocking (no B1–B4).

## Notes written to backlog
`docs/implementation/backlog_research/R7_research_wiring_and_export.md` (the plan's named notes file; the prompt's
generic `backlog.md` is superseded for research milestones by flow.md §10 and the plan):
1. The export secret check scans JSON-escaped text: a secret containing `"`/`\` would slip through. No current key
   format is affected.
2. The C2(a) test separates local from UTC dates only off UTC (it would not on a UTC CI runner).
3. The backend's `wait_ends_at` value has no test row.
4. (H8 leaning) The event → "change" filter lives in the wiring, not the engine.
5. A settings reopen leaves the old research thread running until its provider call ends. The start guard keeps
   one run.

## Lessons for the plans
- Carried items that change a backend value the UI trusts (`wait_ends_at`) should carry a row at the service
  boundary, not only a UI fixture.
- A date-selection row should name its `TZ` (or pick instants that straddle midnight in UTC too), so it
  discriminates on any runner.

## Write perimeter
`docs/implementation/plans/R7_research_wiring_and_export.md` (Tracker line, Review log entry),
`docs/implementation/backlog_research/R7_research_wiring_and_export.md` (new),
`docs/implementation/narratives/R7_research_wiring_and_export.md` (new), this handoff. No production or test file
touched, no probes, no database or archgraph state changed. Commits: none (the runner commits).
