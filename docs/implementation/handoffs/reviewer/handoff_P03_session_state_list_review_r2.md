---
plan: P03_session_state_list
role: review
round: 2
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-10-25
actor: opus
---
# P03 re-review, round 2: APPROVED_WITH_NOTES

F1, the only round-1 finding (B2, C1(l) could not fail), is resolved exactly as its correction said. The fix touched only the allowed file. Suite, lint and typecheck are green on a clean tree. No blocking items remain, so triage has nothing to classify. One new note went to the backlog.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Review history
Round 1 settled all production code (MC-7 status, label and progress; latest-item SQL; MC-4 grouping; list filters, search and counts; command mapping; H1–H8) and every test except C1(l). This round covers only the fix seam. Settled areas were not re-verified.

## Perimeter check
- Gate: the tracker row was `IMPLEMENTED` with a fix-1 note, and checkpoint `9d0d9b6` exists.
- `git show --stat 9d0d9b6` lists 3 files:
  - `crates/testkit/tests/p03_session_state_list.rs`, +1 line. This is the only file F1 allowed.
  - `docs/implementation/plans/P03_session_state_list.md`, with the Review log entry appended.
  - `docs/implementation/master_plan.md`, with the P03 tracker row updated.

  The last two are the implementer's standard closing writes, not production code. No file falls outside the perimeter.
- Commits after the checkpoint (`33329ae`, `7b8c919`) contain only the fix handoff and the re-review prompt, both committed by the runner.
- Stamp on tree `7b8c919`, with `git status --porcelain` empty before the run:

  | Command | Exit | Result |
  |---|---|---|
  | `cargo test --workspace` | 0 | 95 tests; the P03 target ran 28, including `c1l_latest_confirmed_wins` ok |
  | `cargo clippy --workspace --all-targets -- -D warnings` | 0 | |
  | `cargo fmt --all -- --check` | 0 | |
  | `npm run typecheck` | 0 | |
  | `npm test` | 0 | no frontend test files |

## Findings resolved
- **F1 (B2, route production): resolved.**
  - **Change.** `c1l_latest_confirmed_wins` now also asserts `row(&a, s).label == RowLabel::Published`. The input is unchanged: a failed item at upload confirmed at t=1, a done item at t=2, and `published_at` set.
  - **Why it can now fail.** If the older failed item were chosen as latest, `row_label` would return `PublishFailed`, because a failed non-describe step outranks Published (MC-7 label precedence, R-I7). The r1 reversed-input probe already showed this: `status=Published label=PublishFailed`. That evidence is consumed here, not re-run.
  - **Discrimination.** The new assertion catches choosing the oldest item, ordering ascending, or taking the first-inserted row. It also cannot pass by asserting the fixture.
  - **Over-engineering test.** Passes: the assertion is on the row's public outcome.
  - **Divergence.** None declared, none found.

## Triage
No blocking items remain after fix 1, so there is no triage table to write and no MUST item.

## Remaining findings
None blocking. One note, written to `docs/implementation/backlog.md`:
- **P03 → P13.** In C1(l), the run with the later `confirmed_at` also has the higher run id. An ordering by run id alone would therefore pass too. The production SQL orders by `pr.confirmed_at DESC, pr.id DESC, pi.id DESC` (read in `crates/infra/src/sqlite/store.rs:338`), which is correct today. P13 should include a run confirmed out of creation order. This is DROP-class (more tests, speculative), not blocking: the row's test now fails on a wrong outcome.

## Carry-forward dispositions
| Note | Destination |
|---|---|
| Unassigned count includes deleted photos (r1) | P07 |
| Flat count fields in the row DTO (r1) | P07 |
| About four queries per session (r1) | backlog, no phase |
| Future `last_activity_at` never groups as Today (r1) | backlog, no phase |
| C1(l) id order equals `confirmed_at` order (r2) | P13 |

## Lessons for the plans
None new. The r1 lesson stands: a row that proves a selection rule needs an input where the candidates produce different outcomes at the asserted boundary. Where order is by timestamp, that input should also make creation order disagree with timestamp order.

## Write perimeter
- `docs/implementation/handoffs/reviewer/handoff_P03_session_state_list_review_r2.md`: this file, new.
- `docs/implementation/plans/P03_session_state_list.md`: Review log entry appended.
- `docs/implementation/master_plan.md`: P03 tracker row set to `APPROVED`.
- `docs/implementation/backlog.md`: 1 row appended.
- `docs/implementation/narratives/P03_session_state_list.md`: new owner narrative.
- No probes, no production or test files touched, no scratch files. Build artefacts are under `target/` only.
- No commits.
