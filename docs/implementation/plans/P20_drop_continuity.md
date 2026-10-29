---
phase: P20_drop_continuity
title: Drop continuity — the dragged photo is the one that lands, same size, no pop
difficulty: normal
projection: no
depends_on: [P19_navigation_and_drop]
---
# P20 — Drop continuity: the dragged photo is the one that lands, same size, no pop

## Goal
Owner, after P19 (2026-10-29): "the drop works, but it looks odd because when I drop the picture, the preview (dragged)
stays in the correct place but it becomes smaller to then pop up on that place. Can we make it smooth … the user
shouldn't experience it as if it were two components (only one); the same drag is the same that got placed."

Cause (orchestrator, from the code): the overlay (`WorkspacePage.tsx:280`, `.drag-overlay` in `workspace.css:77`) has a
fixed `width: 120px` and `rotate(2.5deg)`, so it differs in size and angle from the tile; the source tile stays
visible at `opacity: .35` (`workspace.css:69`) in its new slot during the drop animation, then jumps to full opacity
when the overlay disappears. Two elements of different sizes are on screen at the drop.

## Read first
`docs/implementation/plans/P19_navigation_and_drop.md` (C3 rows, the immediate reorder), `docs/process/flow.md`,
`docs/intention/evidence/design_handoff.md` (motion), installed `@dnd-kit/core` 6.3.1 `DragOverlay` /
`defaultDropAnimationSideEffects`.

## Files expected to change
`apps/desktop/src/features/workspace/{WorkspacePage.tsx, PhotoTile.tsx, workspace.css}`,
`apps/desktop/src/features/workspace/__tests__/p20_drop_continuity.test.tsx` (new).

## Ordered tasks
1. The overlay takes the active tile's own size (no fixed width: it fills the rect `DragOverlay` measures from the
   active node) and the same corner radius; any lift effect (shadow, slight scale or tilt) is applied only while
   dragging and animates back to none during the drop, so at the end of the drop the overlay is pixel-identical to the
   tile in its slot.
2. While dragging and during the drop animation the source tile is fully invisible (`opacity: 0`, via the
   `dropAnimation` side effects or the `data-dragging` style), so only one photo is ever visible; when the drop
   animation ends the tile appears at full opacity exactly where the overlay ended. No fade-in, no scale-in.
3. Drop animation: translate only (no scale keyframe), duration within 150–250 ms, easing from the design handoff if
   it defines one (else `cubic-bezier(.2,.8,.2,1)`); respect `prefers-reduced-motion` (no animation).
4. Tests.

## Criteria (1 criterion, 3 rows; 3 `[rule]` rows)
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | `WorkspacePage` render (vitest), drag start on a grid tile | 3 Main photos | the overlay element has no fixed pixel width from CSS (its computed width is not `120px`; it sizes from the measured tile) and the source tile's style resolves to `opacity: 0` while dragging | `[rule]` owner 2026-10-29 (one component) |
| C1(b) | the `dropAnimation` passed to `DragOverlay` | — | its keyframes contain no `scale` and no change of size; duration 150–250 ms; its side effects keep the active tile at opacity 0 until the animation ends | `[rule]` owner 2026-10-29 |
| C1(c) | drop on the third slot (vitest) | 3 Main photos | P19 C3 rows still pass (new order shown immediately); after the drop the tile in slot 3 has full opacity and no `data-dragging` | `[rule]` owner 2026-10-29, P19 C3 |

## Delegated choices
Exact lift effect (shadow only is fine), duration within the range, the mechanism for hiding the source tile.

## Manual try-out
Open a session with three or more Main photos; drag the first onto the third slot and let go: one photo glides into
the slot at its tile size and stays; nothing shrinks, fades or pops. Drag a Main photo into Condition: same.

## Notes
- Frontend only; no backend change. CustomizeMode (form) uses no overlay and is out of scope.
- Estimated size: 3 files + 1 test file, ~150 lines.

## Review log
- 2026-10-29 (orchestrator): written from the owner's P19 try-out feedback; small enough to need no planner session.
- 2026-10-29, implement r1 (codex): The overlay now fills dnd-kit's measured tile rectangle and uses the tile's own visual markup, radius and state styling. The optimistic destination render is flushed before dnd-kit clones the overlay, so its final number and badges match the landed tile. Source opacity is zero during drag and through drop via dnd-kit's side effect, then returns directly to one. Drop uses translation-only keyframes for 200 ms with the design token's easing; reduced motion disables it. Three boundary tests cover C1(a–c), and the P19 drag tests still pass. Assumptions: used no lift shadow or tilt so the overlay matches the settled tile; read the OS reduced-motion preference when the workspace renders. Deviations: none. Rust and frontend suites, Clippy, formatting and TypeScript pass; live visual try-out remains for the owner.
