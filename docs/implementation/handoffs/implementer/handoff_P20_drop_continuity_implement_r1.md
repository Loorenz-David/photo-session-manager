---
plan: P20_drop_continuity
role: implement
round: 1
state: DONE
date: 2026-10-29
actor: codex
---

## What was built

A dragged photo now stays at the size and shape of its grid tile or folded-strip tile. Its source tile is invisible while dragging and throughout the drop. The visible tile moves into place over 200 ms at a constant size, then the source is immediately fully visible. The overlay shares the tile's image, number, badges, border radius and active/selected styling; the destination's optimistic render is flushed before dnd-kit clones it for the glide. The OS reduced-motion preference disables the drop animation.

## Tests → criterion rows

| Row | Boundary test | Outcome asserted |
|---|---|---|
| C1(a) | `C1(a) the measured tile fills the overlay while its source is invisible` | Overlay parent uses the measured 100 px tile width, the child fills it rather than using 120 px, and the source has computed opacity zero during drag. |
| C1(b) | `C1(b) drop moves at constant size and hides the tile until the overlay finishes` | The actual `DragOverlay` prop has 200 ms translation-only frames, its side effect hides/restores the active tile, and reduced motion passes no animation. |
| C1(c) | `C1(c) the reorder is immediate and the landed tile is fully visible` | The third-slot reorder is visible before the service resolves, stays after reload, and the landed tile has opacity one and no `data-dragging`. |

P19's three immediate-drop tests also pass unchanged.

## Assumptions

- Used no lift shadow or tilt so the overlay visually matches the destination tile at the handoff.
- Read `prefers-reduced-motion` during workspace render; a newly opened workspace picks up a changed OS setting.

## Deviations

None. No files outside the plan perimeter were changed except the required tracker, plan Review log and handoff.

## Files changed

- `apps/desktop/src/features/workspace/WorkspacePage.tsx`
- `apps/desktop/src/features/workspace/PhotoTile.tsx`
- `apps/desktop/src/features/workspace/workspace.css`
- `apps/desktop/src/features/workspace/__tests__/p20_drop_continuity.test.tsx` (new)
- `docs/implementation/master_plan.md` (P20 tracker row only)
- `docs/implementation/plans/P20_drop_continuity.md` (Review log only)
- `docs/implementation/handoffs/implementer/handoff_P20_drop_continuity_implement_r1.md` (this file)

## Checkpoint SHA

The commit containing this handoff is the checkpoint: run `git rev-parse HEAD` after this session's commit. Its subject begins `CHECKPOINT (not approved):`.

## Stamp

| Command | Result |
|---|---|
| `cargo test --workspace` | Pass, exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | Pass, exit 0 |
| `cargo fmt --all -- --check` | Pass, exit 0 |
| `cd apps/desktop && npm test` | Pass, 12 files / 114 tests |
| `cd apps/desktop && npm run typecheck` | Pass, exit 0 |

## Manual try-out

1. Open a session with at least three Main photos. Drag the first photo onto the third slot and release. The same-size photo should glide into place without shrinking, fading or popping; its number updates with the order.
2. Drag a Main photo into Condition. It should land at the end of Condition with the same smooth handoff.
3. Fold the photo panel and drag in the strip. The small strip tile should retain its own dimensions.
4. Enable Reduce Motion in macOS Accessibility settings and repeat a drop. It should move directly into the destination without a drop animation.

## ⚠ OWNER DECISIONS REQUIRED (0)

None.
