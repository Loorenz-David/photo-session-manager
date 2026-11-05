---
plan: P37_editor_chrome
role: implement
round: 1
state: DONE
date: 2026-11-05
actor: codex
---
# P37 editor chrome — implement r1

## What was built

The editor now has five icon controls above the photo, with a name, shortcut and one-line explanation on hover. The frame meter stays on ⌥⌘P without a button. The Tools panel hides from its own header and leaves a slim left rail to bring it back. Turn sits beside Shape; Rotate and flip and the arrow pad fold open only when needed; the grid switch is beside the degrees field. Their composition actions and remembered settings still work.

## Tests → criterion rows

`apps/desktop/src/features/compose/__tests__/p37_editor_chrome.test.tsx` has one boundary test for each row: C1(a) five icon toolbar controls and retired-button absence; C2(a) all 32 T2 hover cards at their controls; C3(a) frame meter key; C4(a) panel-to-rail; C4(b) rail-to-panel and ⌘T; C4(c) remembered rail and exit; C5(a) rotate drawer and saved rotated composition; C5(b) Turn placement and saved orientation; C6(a) grid switch and stored value; C7(a) pad shape and order; C7(b) nudge saved positions and vertical clamp; C8(a) drawer state across photo/panel/section changes and editor reopen. Existing P32 C3(c) and P36 C5(b)/C6(h) were adjusted only as the plan lists; P36 C5(a), C6(f), C6(g) were superseded and removed.

## Assumptions

- The left rail is 36 px wide, within the plan's 40 px limit.
- Drawer toggles show a short visible label beside their icons; the buttons inside are icon-only.
- The card uses the existing tooltip font and colors, with a heavier name and plain description.

## Deviations

None. No backend, key mapping, save rule or new storage key changed. The first full UI suite run hit the documented intermittent P33 C4(d) stage-bar test; it passed on the next full run. No red baseline was recorded before production edits.

## Files changed

- `apps/desktop/src/components/ui/tooltip.tsx`
- `apps/desktop/src/features/compose/EditorToolbar.tsx`
- `apps/desktop/src/features/compose/ToolsPanel.tsx`
- `apps/desktop/src/features/compose/CompositionSection.tsx`
- `apps/desktop/src/features/compose/StraightenControl.tsx`
- `apps/desktop/src/features/compose/ComposeMode.tsx`
- `apps/desktop/src/features/compose/compose.css`
- `apps/desktop/src/features/workspace/Stage.tsx`
- `apps/desktop/src/features/workspace/FoldedStrip.tsx`
- `apps/desktop/src/features/workspace/PhotoPanel.tsx`
- `apps/desktop/src/features/workspace/workspace.css`
- `apps/desktop/src/features/compose/__tests__/p37_editor_chrome.test.tsx`
- `apps/desktop/src/features/compose/__tests__/p36_controls.test.tsx`
- `apps/desktop/src/features/compose/__tests__/p32_controls.test.tsx`
- `docs/implementation/master_plan.md`
- `docs/implementation/plans/P37_editor_chrome.md`
- `docs/implementation/handoffs/implementer/handoff_P37_editor_chrome_implement_r1.md`

## Checkpoint SHA

The checkpoint is the commit containing this handoff (`git rev-parse HEAD` after the commit). A commit cannot contain its own SHA as text.

## Stamp

- `cargo test --workspace` — exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings` — exit 0.
- `cargo fmt --all -- --check` — exit 0.
- `npm run typecheck` — exit 0 after removing an unused import from the adjusted P36 test.
- `npm test` — 47 files, 348 tests passed. An earlier full run had the documented intermittent P33 C4(d) failure; rerun exit 0.
- `git diff --check` — exit 0.

## Manual try-out

1. Open a photo and press R. Above it, inspect the five icons and hover Loupe to read “Loupe · Z — 100 % view under the pointer.” Press ⌥⌘P twice to show and hide the frame meter.
2. Click the hide icon in the Tools header. The panel becomes a slim rail on the left; click its icon to restore the panel. ⌘T toggles the same state, which stays hidden after leaving and reopening the editor.
3. In Composition, click Turn beside Shape. Open Rotate and flip to use the three icons. Leave and reopen the editor; its drawer starts closed.
4. Click the grid icon beside degrees to turn the straightening grid off. Open Nudge and use the inverted T arrow pad; Shift makes a ten-pixel nudge. Switching photos keeps the pad open; reopening the editor closes it.

## ⚠ OWNER DECISIONS REQUIRED (0)

None.
