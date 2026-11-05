---
plan: P38_editor_polish
role: implement
round: 1
state: DONE
date: 2026-11-05
actor: codex
---
# P38 editor polish — implementer handoff

## What was built
The Split divider now has a white centre and dark edge in the editor, stage and tiles. Loupe, Snapping and Straightening grid show filled accent icons when active. Rotate and flip opens a three-item menu; Nudge opens a four-arrow pad that stays open during nudges and closes outside or on Escape. Both triggers sit beside the straightening degrees field. The inline drawers are gone.

## Tests → criterion rows
- C1(a) → `p38_editor_polish.test.tsx`: split lines at the rendered frame centre, order, widths and Normal absence.
- C2(a), C2(b), C2(c) → `p38_editor_polish.test.tsx`: each toggle's state, fill and persisted value where applicable.
- C3(a), C3(b) → `p38_editor_polish.test.tsx`: menu structure and rotation save plus close.
- C4(a), C4(b), C4(c), C4(d) → `p38_editor_polish.test.tsx`: pad structure, saved nudges while open, outside dismissal, Escape without revert.
- C5(a) → `p38_editor_polish.test.tsx`: three icon controls in the degrees row, no drawers.
- C6(a) → adjusted `p37_editor_chrome.test.tsx`: T3's 29 exact hover cards (27 array entries plus Show Tools panel and Fold panel).
- Updated `p32_controls.test.tsx` for the three menu selections and retained rotation/flip outcomes; `p36_controls.test.tsx` needed no edit and passed unchanged.

## Assumptions
- Used the plan's suggested translucent black edge and the existing colour, radius, border and shadow tokens for the dropdowns.
- Kept focus on the editor when the nudge pad opens. A document capture pointer handler dismisses the pad on outside pointerdown, since Radix's dismissal did not fire for the stage in the DOM boundary test. Clicks inside remain open.

## Deviations
None. `ComposedPicture.tsx` did not need an edit because its existing split marker is shared by stage and tiles; only its CSS changed.

## Files changed
- `apps/desktop/src/features/compose/ComposeFrame.tsx`
- `apps/desktop/src/features/compose/ComposeMode.tsx`
- `apps/desktop/src/features/compose/CompositionSection.tsx`
- `apps/desktop/src/features/compose/EditorToolbar.tsx`
- `apps/desktop/src/features/compose/StraightenControl.tsx`
- `apps/desktop/src/features/compose/compose.css`
- `apps/desktop/src/features/workspace/workspace.css`
- `apps/desktop/src/features/compose/__tests__/p32_controls.test.tsx`
- `apps/desktop/src/features/compose/__tests__/p37_editor_chrome.test.tsx`
- `apps/desktop/src/features/compose/__tests__/p38_editor_polish.test.tsx`
- `docs/implementation/master_plan.md`
- `docs/implementation/plans/P38_editor_polish.md`
- `docs/implementation/handoffs/implementer/handoff_P38_editor_polish_implement_r1.md`

## Checkpoint SHA
The SHA is the commit containing this handoff: `git log -1 --format=%H -- docs/implementation/handoffs/implementer/handoff_P38_editor_polish_implement_r1.md`. A self-referential SHA cannot be embedded in the same commit.

## Stamp
- `cd apps/desktop && npm run typecheck` — pass.
- `cd apps/desktop && npm test` — pass, 355/355 UI tests.
- `cargo test --workspace` — pass, all Rust tests.
- `cargo clippy --workspace --all-targets -- -D warnings` — pass.
- `cargo fmt --all -- --check` — pass.
- Pre-existing failures: none observed. The Rust suite and UI suite ran without camera or Capture One.

## Manual try-out
1. Open a dark photo, press R then S. Check the white split line with its dark edge. Press Return; check the stage and strip tile. Repeat on a light photo.
2. In Compose, toggle Loupe, Snapping and the grid icon beside Degrees. Active icons should fill with the accent colour; inactive icons should be outlines.
3. Open the rotate icon beside Degrees. Pick a turn or flip and confirm the menu closes. Open it and click elsewhere to close it.
4. Open the arrows icon. Click right several times and Shift-click once; the frame should move while the pad stays open. Click the photo to close it. Open it again, nudge once, then press Escape; the pad should close and the nudge should remain.
5. Check that the row has grid, rotate and arrows icons with hover cards and no inline drawers.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
