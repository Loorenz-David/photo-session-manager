---
plan: T4_template_retake_and_shift
role: review
round: 2
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-16
actor: claude opus 5.5 (pipeline-reviewer)
---
# T4 — Retakes and bad shots · re-review r2

**Verdict: APPROVED_WITH_NOTES.** Fix 1 resolves the one blocking finding, T4-R1-B1, exactly as its correction described. The perimeter is clean and every check is green. No blocking item remains, so there was nothing to triage. One new note went to the backlog: a guide row is stale (flow §7A). Tracker is now APPROVED and the narrative is written.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Review history
- **r1** (CHANGES_REQUESTED) settled everything except one item: the gate, the perimeter of U1–U3 and the closing checkpoint, all 14 rows and their tests, the retake, Deny keeping the container empty, the mechanics of `shift_back`, the retirement of `container_filled`, the guide paths, and H1–H8.
- It left one blocking finding. T4-R1-B1: `can_shift_back` was true when nothing would move (a denied Condition shot with a later Condition photo).
- Fix 1 (`3b4f8fe` + `50a3585`, Sonnet 5.5) addressed only that finding. This round's depth went to the changed function and the callers that depend on it.

## Perimeter check
- **Gate.** The plan's Tracker line read `IMPLEMENTED` with the fix note. Checkpoint `3b4f8fe` exists, and so does its SHA follow-up `50a3585`. The tree was clean at `a0e17ce` when the review started.
- **`git show --stat 3b4f8fe`:**
  - `crates/core/src/services/template_placement.rs`: allowed.
  - `crates/testkit/tests/suite/t4_template_retake_shift.rs`: allowed.
  - `docs/implementation/plans/T4_template_retake_and_shift.md`: the fix's own Tracker line and Review log entry.
  - `handoff_…_fix_r2.md`: the fix's own handoff.
- **`git show --stat 50a3585`:** the fix handoff only (the SHA line).
- **Result:** no file outside the allowed set changed. The handoff declares no probes, and none are visible.

## Findings resolved
### T4-R1-B1: resolved as corrected
**What changed.**
- `can_shift_back(tx, session_id, template, now, indexes)` now collects every Deleted photo. If none of them has a shot index (that is, all are released), it returns false.
- Otherwise it recomputes the indexes with those shots left out (`shot_indexes_without`; `shot_indexes` delegates to it with an empty set). It returns true only if some non-Deleted photo with `place_by_hand = 0` gets a different expected `(class, slot)` from `expected_place` under the two index maps.
- `shift_back` calls this function inside its transaction, before any write, with the same `at`. A false answer gives `Err(NothingToShift)` with no write and no step.

**Read adversarially, and found sound.**
- **Same set.** The check simulates releasing exactly the photos that `shift_back` then releases: every Deleted photo not yet released. Photos already released are filtered out of both maps by `shot_keys().released`.
- **Same mover.** `place_template_photos` moves every non-Deleted, template-placed photo to `expected_place(index)`. It has no occupancy check, so a true answer means at least one photo actually moves. That holds for any photo that sat at its expected place before, and every arrival re-places all template photos.
- **Holds.** Both maps use the same `hold_times(session, now)`. Index shifts caused by holds are the same before and after, so they never cause a false true.
- **Presets** are not compared. A different container always means a different slot, so nothing is lost. The fix declares this judgment call.
- **Condition order.** Removing one shot keeps the relative order of the remaining shots, so a Condition-only change can never count as a move. That is correct, because the button would reorder nothing.
- **Mixed case.** One denied photo inside the containers plus one beyond them gives true, because a later container photo moves. That matches 10G §6.4 and P10.

**Test.** `t4_c2e2_denied_condition_shot_with_later_condition_photo_has_nothing_to_shift`. In S0, deny F4: `can_shift_back` is false, `shift_back` returns `NothingToShift`, and the undo label is still the denial's.
- The fixture's only reason for the outcome is the predicate under test. F5 is template-placed and later than F4, so the old rule returned true here (r1's probe saw exactly this). The test can therefore fail on the defect.
- It asserts exact outcomes at the row (e) boundary, which is the service plus the view.
- There is no implementation coupling.

**Full case table for "nothing to shift"** (r1's lesson):
| Case | Test |
|---|---|
| Nothing later | `t4_c2e` |
| Only hand-placed photos later | `t4_c2f` |
| Later photos all beyond the last container | `t4_c2e2` |

Rows C2(a), C2(c), C2(f) and A7 keep their outcomes under the new rule.

**Checks.** Tree `a0e17ce`, which differs from `3b4f8fe` only in docs. One cargo command at a time, none in the background.
| Check | Result |
|---|---|
| `cargo test -p psm-testkit --test suite -- t4_template_retake_shift:: t3_template_hand:: t2_template_placement::` | 52/52 (12 + 19 + 21), 7.6 s |
| `cargo test -p psm-core` | 96/96 |
| `cargo clippy -p psm-core -p psm-infra -p psm-desktop -p psm-testkit --all-targets -- -D warnings` | ok |
| `cargo fmt --all -- --check` | ok |
| `npm run typecheck` (apps/desktop) | ok |

The full suite was not re-run. The change is local to one private service function and its two callers, with no store, migration, binding or shared-code change. That is within flow §4A for a reviewer.

## Triage
No blocking items remain after fix 1. There is nothing to classify as MUST or DROP.

| Item | Class | Why |
|---|---|---|
| (none blocking) | — | — |
| N-r2-1: guide row stale (below) | note, not blocking | flow §7A: a stale guide row is a note |

## Remaining findings
None blocking.

**Note written to `docs/implementation/backlog.md`.**
- **N-r2-1 (flow §7A).** The "Retake and shift (T4)" bullet in `docs/guide/02_architecture.md` still says that `can_shift_back` "says a later template-placed photo exists". The plan's "Points recorded by planning" sentence has the same old wording.
  - Code after the fix: true only when releasing the denied photos would change a template-placed photo's expected place.
  - Why the fix left it: the guide was outside the fix's file list. The fix handoff disclosed it.
  - Destination: orchestrator / next guide touch.

**Carried forward unchanged from r1** (already in the backlog): the 10G §4.2 wording, the untested page path of Shift the rest back, the first-run failure of `psm-desktop`, and T3 N2/N6.

## Carry-forward dispositions
| Note | Destination |
|---|---|
| N-r2-1 (guide + plan point wording of `can_shift_back`) | orchestrator: guide fold (flow §7A) |
| r1 note 1 (10G §4.2) | orchestrator: book fold |
| r1 note 2 (page path untested) | backlog: next workspace phase |
| r1 note 3 (`psm-desktop` first run) | backlog: process/tooling |
| r1 note 4 (T3 N2/N6) | backlog: next phase that touches reorder in templated sessions |

## Lessons for the plans
- When a fix narrows a rule that the plan's "Points recorded by planning" and the guide also state, give the fix prompt's file list those two lines as well. Otherwise the approved phase ships with its narrative source and the guide one rule behind the code. The narrative written here uses the corrected rule.

## Mutation-probe declaration
None. No production or test file was touched. No database or state outside the test runs' temporary directories was changed.

## Files written by this review
- `docs/implementation/handoffs/reviewer/handoff_T4_template_retake_and_shift_review_r2.md` (this file)
- `docs/implementation/plans/T4_template_retake_and_shift.md` (Tracker line → APPROVED; Review log entry "re-review r2")
- `docs/implementation/backlog.md` (one T4 r2 note appended)
- `docs/implementation/narratives/T4_template_retake_and_shift.md` (new)
