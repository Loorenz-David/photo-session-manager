---
plan: P36_editor_controls_and_overlays
role: review
round: 1
verdict: CHANGES_REQUESTED
state: DONE
date: 2026-11-05
actor: opus (pipeline-reviewer)
---

# P36 review r1: CHANGES_REQUESTED

Production is correct on all 22 rows: the reviewer re-derived the values by hand and ran changed-input probes. Two blocking findings, both **test-only** (B2): the C7(a)/C7(b) tests and one sub-step of the C1(b) test cannot fail on a wrong outcome. The fix is confined to two test files. No production change is requested.

## ⚠ OWNER DECISIONS REQUIRED (0)

None.

## Gate and stamp
- Tracker read `IMPLEMENTED`; checkpoint `5123fc8` (`CHECKPOINT (not approved): P36 editor controls and overlays`) is in `git log`.
- `git show --stat 5123fc8` has 31 paths. They are the plan's 16 source paths (new: `overlays.ts`, `EditorToolbar.tsx`, `StraightenControl.tsx`, `useComposePrefs.ts`; changed: `ComposeMode`, `ComposeStage`, `ComposeFrame`, `CompositionSection`, `useComposeKeys`, `useComposeGesture`, `snapping`, `machine`, `compose.css`, `Stage`, `WorkspacePage`; deleted: `BrowserOverlay.tsx`), the 4 new and 8 adjusted tests named in the plan, and 3 closing artifacts (handoff, tracker row, Review log). Nothing is extra or missing.
- Reviewer stamp on the clean tree `bbf381c` (= `5123fc8` + review prompt): `cargo test --workspace` exit 0; `cargo clippy --workspace --all-targets -- -D warnings` exit 0; `cargo fmt --all -- --check` exit 0; `npm run typecheck` exit 0; `npm test` 46 files, 339/339.

## Verified correct
- **C1(a).** `ComposeMode` renders `<FoldedStrip>` as a flex sibling after `.stage-column` (`.workspace` is `display: flex`). The tile click goes through `pick`. Nothing reacts to the right edge any more: the `BrowserOverlay` file and the `.compose-browser` CSS are deleted.
- **C1(b).** `browserFolded` starts `true` at every mount. `f` is no longer in `SWALLOWED` and `useComposeKeys` maps it to the toggle. `WorkspacePage`'s `folded` is untouched, so both pre-Compose states come back on Return, and the test asserts both runs. The one exception is the tile-click sub-step: see F2.
- **C2(a).** Every line set was re-derived by hand: thirds, grid at k/6, golden at `1/φ²` and `1/φ`, the diagonals (two on a square, four on the 952×634.79 box, duplicates removed), split per half (8 lines), and `off` → `[]`. The test compares order-free at 0.01 px.
- **C2(b).** Cycle order and line counts (4/10/4/2/0) are asserted. Reads and writes are guarded. `bogus` → `thirds`. The value persists across a remount.
- **C3(a).** The grid shows while the field is focused or the slider is held. It is a screen-aligned `userSpaceOnUse` 24 px pattern over the whole stage. The switch is persisted (`'0'`) and still reads off after a remount.
- **C4(a).** `h = 0.8632` is correct: `1/(cos 10° + sin 10°)`, the P32 review's correction (`P32_composition_tools.md:238`). The plan's `0.816` is the old slip.
- **C4(b)/(c).** The steps (0.1, 1.1, 1.0), the clamp at 45, the rejection of `abc`, the comma, and Esc are asserted exactly. Esc in the field is guarded by `isTyping` in both key handlers (`useComposeKeys` capture listener, `useComposeGesture`). If Esc leaked, the editor would revert to the entry and add a third save, so the test bites.
- **C5(a).** All 30 entries are found by role, name and place, each with its exact tooltip text. **C5(b).** `aria-disabled`, hintable, inert, and enabled after Copy.
- **C6(a)–(h).** The tests assert exact values: Revert = `KeyEsc` → `x = 0.1666015625`; stage arrows call `stepPhoto` with no wrap; the pinned loupe sits at 616/316 and causes no save; Snapping off forces `altKey` on `PointerMove`, the only snap site (`machine.ts:258`). The C6(d) Revert reading is arithmetically required: a second +3.7 px drag from `0.162818` ends 7.4 px from centre, beyond the 6 px threshold. Paste is refit across 2560×1707 → 1707×2560. Tools panel, Frame meter and nudge (±1/±10 px, same sign as the arrow keys) are covered.
- **C8(a)/(b).** `snap` reports `centre | start | end | null`. `ComposeFrame` draws at `frameBox.left` (174) or at the centre or the right edge. The publish check compares the guide values.
- **Delegated choice "must not save identical values".** `machine.ts` `SetValues` saves only when `!sameValues`. Probe: focusing and then blurring the field with unchanged text → 0 saves.
- **Assumptions.** C4(a) 0.8632, the C6(d) Revert, the narrow C1(a) `data-open` reading, and commit on Enter/blur: none contradicts 10D, and each is the smallest reading.
- **H1–H8.** The diff is frontend-only: no backend path, no external platform write, no RAW access, no pixel processing, no UI automation. No secrets. The suite uses no live service.

## Findings

### F1 — B2 · route: production (test-only fix) · C7(a), C7(b)
**What is wrong.** Neither test can fail if the focus handoff is removed.
- C7(a) fires `fireEvent.change` on `#compose-shape` without focusing it, so `document.activeElement` is `<body>` throughout. It then fires `x` on `window`, so `isTyping(event.target)` never sees the select.
- C7(b) has the same defect: `fireEvent.pointerDown` does not focus the slider, and `x` is fired on `window`.
- Deleting `event.target.blur()` in `CompositionSection`, or the slider's `pointerup` blur in `StraightenControl`, leaves both tests green.
- Reviewer probe (scratch test, deleted): with the select focused and no change, `x` fired on `window` still turns the frame. Fired at the focused select, it does nothing. The production code is correct: with the control focused first and `x` dispatched at `document.activeElement`, the frame turns in both cases.

**Authority.** flow.md §5 B2; plan rows C7(a), C7(b) (the outcome is "the `x` key reached the editor", which only means something when the control had the keyboard).

**Correction.** In each test, focus the control before acting (`act(() => select.focus())`; `act(() => slider.focus())` before `pointerDown`). Dispatch `x` at `document.activeElement` instead of `window`, as a real keyboard does.

**Files the fix may touch.** `apps/desktop/src/features/compose/__tests__/p36_straighten.test.tsx`.

### F2 — B2 · route: production (test-only fix) · C1(b), the panel tile click
**What is wrong.** The test starts a fresh editor on photo 1, presses `f`, and clicks **tile 1** in the unfolded panel. It then asserts that the editor shows photo 1, which was already true. The row runs "from C1(a)", so the editor should be on photo 2 at that point. The `PhotoPanel` branch's `onOpen={pick}` is a separate prop from the strip's, and nothing in the suite observes it. If that wiring were missing, clicking a photo in the unfolded panel would do nothing and the suite would stay green. Reviewer probe: tile 2 in the unfolded panel does move the editor, so production is correct.

**Authority.** flow.md §5 B2; plan row C1(b) ("tile 1 → editor on photo 1", from C1(a)'s state on photo 2); 10D ("Clicking a photo in the strip moves the editor to it").

**Correction.** Make the panel click change the photo: move to photo 2 first (strip tile 2, or `]`) and then click tile 1 in the panel, or click tile 2 in the panel from photo 1. Assert the new preview src.

**Files the fix may touch.** `apps/desktop/src/features/compose/__tests__/p36_strip_overlays.test.tsx`.

## Notes written to backlog
1. Plan lesson: C4(a) repeated P32's corrected `0.816`; C6(d)'s sequence cannot centre-snap without a reset; C1(a)'s "nothing with `data-open`" collides with the Tools sections.
2. Layout was checked in jsdom only (strip, overlays, grid, toolbar wrapping). The dead `.compose-straighten-grid + path` selector.
3. By reading: a changed degrees value is dropped silently when the field loses focus to a stage pointerdown (the machine is already `dragging`).
4. P36 resolves the P32→P34 focus row and the P34 always-centre guide row in production, and C6(d) asserts the `0.162818` value the P34 C2(b) row asked for. Mark them resolved at approval.

## Lessons for the plans
- A focus-handoff row must say where the key is dispatched (`document.activeElement`) and that the control is focused first. jsdom's `fireEvent.change`/`pointerDown` do not move focus, and a key fired on `window` bypasses every field guard.
- A row that follows another row's state ("from C1(a)") is often run in a fresh fixture. State the starting photo in the row so a click that changes nothing cannot satisfy it.

## Write perimeter
- `docs/implementation/handoffs/reviewer/handoff_P36_editor_controls_and_overlays_review_r1.md` (this file, new).
- `docs/implementation/master_plan.md`: the P36 tracker row only (→ `CHANGES_REQUESTED`).
- `docs/implementation/plans/P36_editor_controls_and_overlays.md`: Review log entry appended.
- `docs/implementation/backlog.md`: 4 rows appended.
- **Probe file (applied and removed):** `apps/desktop/src/features/compose/__tests__/zz_review_probe_p36.test.tsx` was created, run (6/6 green) and deleted. `git status --porcelain` shows no trace of it. No production file was touched or edited at any point. No database or other state side effects.
- No commits (the runner commits).
