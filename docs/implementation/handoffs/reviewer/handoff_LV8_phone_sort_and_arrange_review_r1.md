---
plan: LV8_phone_sort_and_arrange
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-27
actor: opus
---
# Review — LV8_phone_sort_and_arrange, round 1

**Gate.** The plan's Tracker line read `IMPLEMENTED (2026-11-27, fable)`. This track keeps its tracker row in the plan
file. The checkpoints `09ba8340` (U1), `9cbb7f7c` (U2), `0348cc8a` (U3), `c7824ab9` (U4) and `798834a9` (close) are all
in `git log` on `feature/live-preview`. Gate passed.

## Verified correct
- **Perimeter.** `git diff --stat 23e42fcc 798834a9` lists exactly the 27 paths the handoff declares:
  - Rust: `template_placement.rs`, `current_shot.rs`, `routes.rs`, `wire.rs`, the new `lv8_phone_sort.rs`, and
    `suite/main.rs`.
  - Phone: `arrange.ts` (new), `ConfirmSheet.tsx` (new), `Dock.tsx`, `Preview.tsx`, `PhoneApp.tsx`, `api.ts`,
    `state.ts`, `gestures.ts`, `phone.css`.
  - Tests: the 4 new `lv8_*` files, `helpers.ts`, `lv3_preview`, `lv3_drawers`, `lv6_dock`, `lv6_images`.
  - Docs: the 2 guide files and the plan.

  Nothing is missing and nothing extra. The tree after `798834a9` held only the runner's files.
- **Checks (the plan's Review mode, run once on clean `1b880ee0`):**

  | Check | Result |
  |---|---|
  | `cargo test -p psm-testkit --test suite lv8_phone_sort::` | 14 passed |
  | `cd apps/desktop && npx vitest run src/phone` | 14 files, 181 passed |
  | `npm run typecheck` | exit 0 |
  | `npm run build` | exit 0 (chunk-size warning only) |

  The implementer's closing-gate stamp stands: `cargo test --workspace` 2774, plus clippy, fmt and the sidecar. Its one
  `npm test` failure is `p33_split_grid` C4(d), a flake already recorded in the backlog (it passed when run alone).
  LV8 changed no file under `src/features`, so nothing contradicts the stamp and B4 does not apply.
- **U1, read against PH4a.**
  - `move_to` and `classify_to` share one transaction, `move_photo(h, id, choose)`, so there is no second copy of the
    model.
  - The classify table matches PH4a:
    - Main → extra and Condition → Condition return `None`: status 200, no step, no event.
    - A template container's photo goes to `ExtraEnd` or `ConditionEnd`.
    - A Deleted or Unreviewed photo is not in the model, so the request gets `invalid_value`.
  - Insert:
    - A source outside its own group is refused (the `origin` check). `index` must be in `1..=len`.
    - The same place changes nothing.
    - The photo is removed and inserted at its final index.
    - Place rows are not touched: only positions are written.
  - `set_after_take_out` runs only when a step exists, the photo left a template container empty, and its session is
    the active one. It runs after the commit and before `HandAction::finish` announces.
  - The route only parses, guards, calls `session_photo` and calls core (H8).
- **U1 tests.**
  - Every row compares the whole store state (`everything()` / `place_rows()`) and the exact announcements.
  - C1(a) and C3(a) undo once and check what came back. The "nothing changes" rows check that one undo takes back the
    fixture's last step. So a step left by the fixture cannot pass for the row's own step.
  - C4(a) checks all five refusals, each 400 `invalid_value`, with nothing changed and nothing announced.
- **U2–U4, read against PH9.**
  - `swipeAction` matches PH9.1's table, and `dropTarget` matches PH9.3's table:
    - A shot other than the source within 32 px is a swap.
    - Each group's span runs from 32 px before its first tile's centre to 32 px after its last.
    - The insert index is 1 + the other tiles whose centre is left of the finger.
    - Its own place, the Free tile, a gap or a finger outside the band gives `null`.
  - **Targets come from homes measured at lift.** `Dock.liftAt` measures every tile once, and `follow` reads only
    `lift.homes`.
  - **Pointer capture, no HTML5 drag and drop.** `setPointerCapture` is in the strip's single `onPointerDown`. There is
    no `draggable={true}`, `dragstart` or `onDrag` anywhere in `src/phone`.
  - **One cancel.** pointercancel, a second pointer, the page hidden, window blur, an `arrangementKey` change, the strip
    unmounting, a disabled strip and a release with no target all call `finish(false)`. `main_confirm` cancels the drag
    and opens the sheet.
  - **The pending drop has exactly PH9.5's three ends**, all through `endPending`:
    - (a) a session read numbered after the Mac's answer returns;
    - (b) the Mac refuses;
    - (c) the 8 s timer fires.

    Unmount also ends it. `liftAt` returns early while a drop is pending, so a long press stays a tap.
  - **The next photo before any re-read.** `swipe` dispatches `nextPhoto` before its request, so the orphaned-preview
    effect cannot close the Preview.
- **UI tests.**
  - Each of the 56 rows asserts the row's exact outcome: the POST list, vibration patterns, `data-lifted`, `data-drop`,
    translate values, the kicker, `scrollLeft` or the toast. Each would fail on a wrong outcome.
  - C5(a) would fail under the old hold-at-touch behaviour.
  - C15(a) holds the follow-up read, so it observes the drop still shown between the Mac's answer and that read.
  - The 20 `lv8_core` unit tests cover only the five functions the plan names under Core mechanisms.
  - No test counts calls or mocks an internal collaborator.
- **Changed LV3/LV6 tests** are exactly the plan's list: `lv3_c22c` and `lv6_c2c` removed, `lv3_c21a/b` with
  `advance(200)`, and `lv6_c5a` scoped to the viewport. There is one more, `lv3_c23b`, which the handoff declares
  (deviation 1). Its Delete line now asserts that an extra has no Retake, which LV-D20 (4) requires.
- **Assumptions 1–18,** read against 10J LV-D20, PH9 and card LV8: none contradicts them. All of them are in the
  narrative's "Choices made for you".
- **H1–H8 hold:**
  - The rules live in `crates/core` (H8), and no platform concept enters core (H7).
  - Deny still only hides a photo (H2).
  - Repeating a classify or a same-place insert changes nothing (H4).
  - There is no external write (H1), no pixel work (H3) and no UI automation (H5).
- **The guide is current.** `02_architecture.md` describes classify/insert, `set_after_take_out` and `arrange.ts`.
  `01_product.md` describes the swipes, the strip and lift-and-drag.

## Findings
None blocking (B1–B4). The judgment calls and residual risks are notes, listed below.

## Notes written to backlog
To `docs/implementation/backlog_live/LV8_phone_sort_and_arrange.md`. The plan's Review mode names this file; it is
this track's backlog, and LV7 used the same convention.
1. **The 8 s wait is not stopped by a successful answer.** If a move is answered just before 8 s, the strip can flash
   back for one read (assumption 14 against the wording of PH9.5 (c)). The end state is always the Mac's view.
2. **A photo with no preview yet does not follow a swipe.** The fix belongs in `PhotoImage`, outside LV8's files
   (assumption 6).
3. **Nothing has been seen on a real phone.** This covers the lift look, the sheet, iOS long-press and second-finger
   handling, and assumption 15's `isPrimary` path, which jsdom never exercises.
4. **`p33_split_grid` C4(d) flaked again** in the implementer's full run.
5. **Process:** `cargo test --workspace` was run detached (deviation 2). It overlapped nothing.

**Carry-forward:** note 1 goes to the owner's try-out and STATUS_live; if a drop is seen flashing back, it becomes a
one-row fix. Note 2 goes to any later phase that touches `PhotoImage`. Note 3 goes to the owner's hardware try-out.
Notes 4 and 5 stay with the existing backlog item and the runner.

**Lessons for the plans.** The reviewer prompt says to write notes to `docs/implementation/backlog.md`, but the live
track's plans say `backlog_live/<phase>.md`. The prompt template for the live track should name the second.

**Probe declaration.** No mutation probes were run and no file was touched and reverted (flow §4). I wrote only these
files:
- the plan's Tracker line and a new Review log entry in `docs/implementation/plans/LV8_phone_sort_and_arrange.md`
- `docs/implementation/backlog_live/LV8_phone_sort_and_arrange.md` (new)
- `docs/implementation/narratives/LV8_phone_sort_and_arrange.md` (new)
- this handoff

There is no archgraph delta and no commit.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
