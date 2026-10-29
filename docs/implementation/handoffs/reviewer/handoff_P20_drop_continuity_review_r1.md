---
plan: P20_drop_continuity
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-10-29
actor: opus
---
# Review: P20_drop_continuity, round 1

**Verdict: APPROVED_WITH_NOTES.** No B1–B4 finding. Two notes went to the backlog.

## Gate
Tracker showed `IMPLEMENTED`. Checkpoint `63e5899` (`CHECKPOINT (not approved): implement P20 drop continuity`) is in `git log`.

## Verified correct
- **Perimeter.** `git show --stat 63e5899` lists exactly the 7 declared paths: `WorkspacePage.tsx`, `PhotoTile.tsx`, `workspace.css`, the new `__tests__/p20_drop_continuity.test.tsx`, the master plan (P20 row only), the plan's Review log, and the implementer handoff.
- **Stamp, re-run by the reviewer** (master plan commands): `cargo test --workspace` 407 passed, 0 failed · `cargo clippy --workspace --all-targets -- -D warnings` exit 0 · `cargo fmt --all -- --check` exit 0 · `npm test` 12 files / 114 passed · `npm run typecheck` exit 0.
- **C1(a).** Production: the overlay is `<div class="photo-tile tile-{grid|strip} drag-overlay" style="width:100%;height:100%">` with `PhotoTileFace`. The CSS rule `.drag-overlay { width: 120px; …; rotate(2.5deg) }` is gone (now `width/height: 100%`). dnd-kit's `PositionedOverlay` sizes the wrapper from `initialRect` (the active node's measured rect). Source: inline `opacity: isDragging ? 0 : 1`, and the CSS `[data-dragging]` rule is `0`. Test: the wrapper is `100px` (the measured tile), the overlay is `100%`/not `120px`, and the source's computed opacity is `0`. jsdom loads no CSS, so these come from the inline styles; a fixed inline width or an opacity of .35 would fail the test. Holds.
- **C1(b).** Production: `dropAnimation = { duration: 200, easing: var(--ease-out) value, keyframes: translate3d only, sideEffects: defaultDropAnimationSideEffects({ styles: { active: { opacity: '0' } } }) }`, and `null` under `prefers-reduced-motion`. Test: captures the real prop through a forwarding wrapper of `DragOverlay`; checks the duration range, feeds a transform with `scaleX/Y` 1.5 → 0.5 and asserts exact translate-only frames, runs the side effect (opacity `0`) and its cleanup (restored), and gets `null` under reduced motion. A scale keyframe, a duration outside 150–250 ms, or missing side effects would each fail it. By reading dnd-kit 6.3.1 `AnimationManager` / `createDefaultDropAnimation`: the side effect runs in a layout effect after React committed `opacity: 1`. It saves `1` as the original and restores it on `onfinish`. The tile is hidden for the whole glide and then appears at full opacity with no fade. Holds.
- **C1(c).** Test: dropping photo 1 on slot 3 shows `[2,5,1]` before `reorderPhoto` resolves; `reorderPhoto(1, 3)`; the order holds after it resolves; the landed tile has no `data-dragging` and opacity `1`. P19's three drop tests pass unchanged. Holds.
- **Easing** uses the design token `--ease-out: cubic-bezier(.22, 1, .36, 1)` (`apps/desktop/src/styles/tokens.css:57`), as task 3 asks.
- **Hard constraints H1–H8:** not touched (frontend presentation only; no service, command or platform concept added). No secrets, no live services.
- **Assumptions:** (1) no lift shadow or tilt, which the plan delegates ("shadow only is fine"); (2) reduced motion is read on each workspace render. Neither contradicts the intention. Both are in the narrative.

## Findings
None blocking.

## Notes written to backlog
1. **P20: the glide shows the pre-drop number and Cover badge, and the `flushSync` is inert.** dnd-kit's drag-end handler queues `dispatch({type: DragEnd})` *before* it calls `onDragEnd`. The `flushSync(() => drop(...))` render therefore already has `active = null`, and `AnimationManager` clones the last during-drag overlay. A temporary reviewer test (copy of the P20 harness with a stub `animate`, deleted afterwards) observed: slot 3 drop → glide `01`, landed `03`; slot 2 drop → glide `01`, landed `02`. The handoff's claim ("its final number and badges match the landed tile") and the code comment are wrong. The visual effect is a badge switch at landing, which I judge acceptable: the number changes when the photo takes its slot, and plan task 1's "pixel-identical" concerns size, radius and lift. The narrative offers it to the owner as a choice.
2. **P20 (not verified live):** jsdom runs no drop animation (the probe recorded 0 `animate` calls). The glide is verified only by reading and by calling the prop directly (C1(b)). The owner's try-out is the proof.

## Lessons for the plans
- For visual phases on dnd-kit, a criterion that the glide's contents match the landed tile needs a row at the `AnimationManager` clone (the overlay DOM present right after release). This row would have caught the inert `flushSync`. Only add it if the owner cares about the badges.

## Write perimeter
- `docs/implementation/master_plan.md`: P20 tracker row only (→ `APPROVED`)
- `docs/implementation/plans/P20_drop_continuity.md`: Review log entry only
- `docs/implementation/backlog.md`: 2 rows appended
- `docs/implementation/narratives/P20_drop_continuity.md`: new
- `docs/implementation/handoffs/reviewer/handoff_P20_drop_continuity_review_r1.md`: this file
- Temporary probe `apps/desktop/src/features/workspace/__tests__/zz_review_probe.test.tsx`: created, run and deleted. `git status` was clean afterwards. No production file was edited, and no database or app state was touched.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
