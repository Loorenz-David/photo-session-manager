---
plan: T3_template_hand_changes
role: review
round: 2
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-16
actor: claude opus 5.5 (pipeline-reviewer)
---
# T3 — Your changes win, and there is a way back · re-review r2

**Verdict: APPROVED_WITH_NOTES.** Fix 1 resolves T3-R1-B1 the way its correction asked. The perimeter is clean and every check is green. No blocking item remains. One new note (N6, extends r1's N2) went to the backlog for T4.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Review history
- **r1 (CHANGES_REQUESTED)** settled the perimeter, all 24 rows and their tests, the review-mode reads, H1–H8 and the assumptions. It raised one blocking finding: T3-R1-B1, undo of a reorder among extra Main photos was a no-op. Notes N1–N5 went to the backlog.
- **Fix 1** was done by Sonnet, checkpoint `4e1ba0c`.
- This round covers only the delta.

## Perimeter check
- **Gate.** The Tracker read `IMPLEMENTED` with a fix 1 note. Checkpoint `4e1ba0c` exists (`CHECKPOINT (not approved): T3 fix 1 …`).
- **`git show --stat 4e1ba0c`** lists five paths:
  - `crates/core/src/services/history.rs`
  - `crates/core/src/services/review.rs`
  - `crates/testkit/tests/suite/t3_template_hand.rs`
  - the plan file (Tracker and Review log)
  - the fix handoff

  All of them are inside the finding's allowed list. The store port and impl were allowed but not needed, and were not touched.
- **`git diff --stat 4e1ba0c HEAD`** shows only the fix handoff's SHA line and the r2 prompt.
- The tree was clean at the start of this review.

## Findings resolved
### T3-R1-B1 · resolved
- **What changed.**
  - `StepKind::Reorder` now carries `place_before` and `place_after`, each as (slot, place_by_hand).
  - `ReviewService::reorder` reads them from the place row right before and right after the drag's `photo_place_mark`.
  - `apply` in `history.rs` writes `place_before` on undo and `place_after` on redo, using `photo_place_row` + `photo_place_put`. `photo_place_put` can clear a slot, which `photo_place_mark` could not.
  - When no row exists and the value to write is the default, nothing is written. In an untemplated session `photo_place_mark` returns early, so before = after = (None, false) and no row is created.
- **Checked against the correction.**
  - The correction asked that undo leave the place row as it was before the drag (an extra with no slot has none again) and that redo write what the drag wrote. The code does exactly that.
  - Restoring `place_by_hand` too is implied by "as it was before the drag", so I accept the divergence as sound.
  - It also improves one case. A container photo dragged into another container and then undone now returns to its own slot unmarked. Under r1 it kept the mark.
- **The test.** `t3_c1g_undo_of_a_reorder_among_extra_main_photos_restores_the_order` uses the r1 probe input and asserts exact positions:
  - after undo: F4 4, F5 5
  - after redo: F5 4, F4 5

  On the r1 code the undo assertion fails, because the r1 probe observed F4 5, F5 4. It traces to T3·C1(g) and the U1 goal ("undo of any older step … Main in container order"), so it is not an orphan.
- **Persistence.** `Step` derives only `Debug, Clone` and is never serialized, so the new fields have no persistence impact.
- **Checks.** Run once each, one cargo command at a time, on `ee4a4ec` (= `4e1ba0c` + docs):

  | Check | Result |
  |---|---|
  | `cargo test -p psm-testkit --test suite t3_template_hand::` | 20/20 |
  | `cargo test -p psm-testkit --test suite p39_undo_history_core::` | 24/24 |
  | `cargo test -p psm-testkit --test suite t2_template_placement::` | 21/21 |
  | `cargo test -p psm-testkit --test suite p02_review_order::` | 25/25 |
  | `cargo test -p psm-core` | 96/96 |
  | `cargo clippy -p psm-core -p psm-infra -p psm-desktop -p psm-testkit --all-targets -- -D warnings` | ok |
  | `cargo fmt --all -- --check` | ok |
  | `npm run typecheck` | ok |

  r1's full-suite stamp stands: no shared code changed beyond the Reorder step, and there was no store, migration or binding change.

## Triage
No blocking item remains, so there is nothing to classify (MUST/DROP). The verdict is `APPROVED_WITH_NOTES` because of the note below.

| Item | Class | Destination |
|---|---|---|
| (none blocking) | — | — |
| N6 (note, seen in passing) | not blocking (pre-existing; not a regression of fix 1) | backlog → T4 |

## Remaining findings
### N6 · note · extends r1 N2 → T4
- **What is wrong.** In `apply` (`StepKind::Reorder`) the place-row restore sits inside `if old != next`. Take a container photo dragged behind the extras (N2's no-op drag; the implementer's disclosed "known edge"):
  - The drag writes slot = target and `place_by_hand = 1`.
  - `renumber_main` then returns the photo to its old position, because a slot sorts before the slotless extras.
  - On undo, `position::reorder` finds nothing to change, so the restore is skipped. The photo stays marked and differing after ⌘Z.
- **Not a regression.** r1's code had the same guard.
- **Workaround.** Back to template clears it.
- **Correction for T4.** Write the place row whether or not the positions changed.
- Written to `docs/implementation/backlog.md`.

r1's notes N1–N5 and their carry-forward dispositions stand unchanged:
- N1 → coordinator (P7)
- N2, N5 → T4
- N3, N4 → backlog

### Carry-forward dispositions
| Note | Destination |
|---|---|
| N1 | coordinator: fold into `master_plan_templates.md` P7 before T4 |
| N2, N6 | T4 (reorder / retake paths) |
| N3, N4 | backlog (test debt) |
| N5 | T4 (guide) |

## Lessons for the plans
- A step's restore written under a "did the positions change" guard misses steps whose effect was only on the place row. When T4 reworks reorder, its rows should include the no-visible-move drag and its undo.

## Mutation-probe declaration
None. I edited no production or test file, and wrote no scratch tests. No database or state outside the test runs' temp directories was touched.

## Write perimeter
- `docs/implementation/handoffs/reviewer/handoff_T3_template_hand_changes_review_r2.md` (this file)
- `docs/implementation/plans/T3_template_hand_changes.md` (Tracker line → APPROVED, Review log entry r2)
- `docs/implementation/backlog.md` (N6 appended under the T3 notes)
- `docs/implementation/narratives/T3_template_hand_changes.md` (new: the owner narrative)

No code, no archgraph. Nothing committed.
