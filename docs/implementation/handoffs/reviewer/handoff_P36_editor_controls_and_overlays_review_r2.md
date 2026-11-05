---
plan: P36_editor_controls_and_overlays
role: review
round: 2
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-05
actor: opus (pipeline-reviewer)
---

# P36 re-review r2: APPROVED_WITH_NOTES

Fix 1 changed only the two test files that the r1 findings allowed. Both blocking findings are resolved as their corrections said, and the suite, lint and typecheck are green. No blocking item remains, so triage finds 0 MUST. The verdict carries the r1 backlog notes, which are already in `backlog.md`. Production was settled in r1 and is unchanged.

## ⚠ OWNER DECISIONS REQUIRED (0)

None.

## Review history
- r1 (CHANGES_REQUESTED) settled production on all 22 rows, H1–H8, the assumptions and the r1 perimeter (`5123fc8`). It raised two test-only B2 findings: F1 (C7(a)/(b)) and F2 (C1(b)). This round re-verifies only that seam.

## Gate
- The tracker read `IMPLEMENTED` with the note "fix 1: …". Checkpoint `9cbfd24` (`CHECKPOINT (not approved): P36 fix 1`) is in `git log`. HEAD is `351c16c`, which adds only the re-review prompt. `git status --porcelain` was clean at entry.

## Perimeter check
`git show --stat 9cbfd24` lists 5 paths:
- `apps/desktop/src/features/compose/__tests__/p36_straighten.test.tsx`: allowed by F1.
- `apps/desktop/src/features/compose/__tests__/p36_strip_overlays.test.tsx`: allowed by F2.
- `docs/implementation/master_plan.md`: the P36 tracker row only.
- `docs/implementation/plans/P36_editor_controls_and_overlays.md`: Review log entry appended.
- `docs/implementation/handoffs/implementer/handoff_P36_editor_controls_and_overlays_fix_r2.md`: new.

No production file is in the list and nothing is outside the perimeter. `git diff 9cbfd24 HEAD` touches only the re-review prompt.

## Stamp
The re-review prompt directs one run. Tree: `351c16c`, clean; code identical to `9cbfd24`.
- `cargo test --workspace`: exit 0, 532 passed, 0 failed.
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0.
- `cargo fmt --all -- --check`: exit 0.
- `cd apps/desktop && npm run typecheck`: exit 0.
- `cd apps/desktop && npm test`: exit 0, 46 files, 339/339.
- The P33 C4(d) flake (backlog) did not occur in this run.

## Findings resolved
- **F1 (C7(a), C7(b)): resolved as corrected.**
  - C7(a) now runs `act(() => shape.focus())` before `fireEvent.change`. C7(b) runs `act(() => slider().focus())` before `pointerDown`. Both press `x` at `document.activeElement`.
  - `isTyping` (`lib/keyboard.ts`) returns true for `INPUT`, `TEXTAREA` and `SELECT`. A control that kept focus would therefore swallow `x`, no second save would happen, and `toHaveBeenCalledTimes(2)` would go red.
  - Changed-input probe, in a scratch file that was then deleted: `x` at a focused select with no change gives 0 saves and focus stays on the select. `x` at a held slider with no release gives 0 saves and focus stays on the slider. After `change`, the select no longer holds focus. So each test depends on the production blur.
  - The slider has two blur paths: `onPointerUp` `currentTarget.blur()`, and the window `pointerup` listener's `release`. Removing only one of them changes nothing at the boundary (an equivalent mutant), so this is not a finding.
- **F2 (C1(b)): resolved as corrected.** The panel is unfolded from photo 1, the test clicks tile 2 in `.photo-panel`, and it asserts preview src `asset:///previews/2/adjusted-1.jpg`. If the `PhotoPanel` `onOpen={pick}` wiring were missing, the src would stay on photo 1 and the test would fail. The rest of the row (Fold panel, Return restores both prior states) is unchanged.
- **Divergences declared by the fix:** none. **Assumptions added:** none.

## Changed seam, read adversarially
- Both test edits model a real keyboard: focus moves on click in WebKit, and keys are delivered at the focused element. No row's input or expected value was weakened. C7(a) still asserts the `4:5` shape save and then the `turned` save. C7(b) still asserts `straighten = 10` and then `turned`.
- Nothing wrong was seen in passing.

## Triage (flow.md §5)
| Item | Class | Reason |
|---|---|---|
| none | — | No blocking item remains after fix 1. |

0 MUST, 0 DROP. The verdict is `APPROVED_WITH_NOTES` because the r1 backlog notes stay open.

## Remaining findings
None blocking. The open notes are r1's four backlog rows (plan lesson; layout not checked in WebKit plus the dead selector; typed degrees dropped when blur comes from a stage drag; the resolution marker). This round adds no new note.

## Carry-forward dispositions
| Note | Destination |
|---|---|
| P36 plan lesson (C4(a) 0.816, C6(d) sequence, C1(a) `data-open`) | coordinator: plan-authoring lesson, `backlog.md` |
| P36 layout in jsdom only, dead `.compose-straighten-grid + path` selector | owner try-out, steps 3–8; next phase touching `compose.css` |
| P36 typed degrees dropped on blur from a stage pointerdown | backlog, next compose phase |
| P36 → P32/P34 rows | done this round: P32→P34 focus, P34 always-centre guide and P34 (test) C2(b) marked `✓ resolved by P36` |

## Lessons for the plans
- r1's two lessons stand: a focus-handoff row names the dispatch site and focuses the control first, and a row that follows another row's state names its starting photo. The fix needed no further change, which suggests the corrections were precise enough to apply as written.

## Write perimeter
- `docs/implementation/handoffs/reviewer/handoff_P36_editor_controls_and_overlays_review_r2.md` (this file, new).
- `docs/implementation/master_plan.md`: the P36 tracker row only (→ `APPROVED`).
- `docs/implementation/plans/P36_editor_controls_and_overlays.md`: Review log entry appended.
- `docs/implementation/narratives/P36_editor_controls_and_overlays.md` (new).
- `docs/implementation/backlog.md`: 4 existing rows prefixed with a resolution marker (P32→P34 focus, P34 guide, P34 (test) C2(b), P36→P32/P34). No row text was otherwise changed.
- **Probe file (created, run, deleted):** `apps/desktop/src/features/compose/__tests__/zz_review_probe_p36_r2.test.tsx`, 3 tests, green. `git status --porcelain` showed no trace of it afterwards. No production file was edited at any point. No database or state side effects.
- No commits (the runner commits).
