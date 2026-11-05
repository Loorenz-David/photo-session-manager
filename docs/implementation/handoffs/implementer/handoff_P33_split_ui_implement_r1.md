---
plan: P33_split_ui
role: implement
round: 1
state: IMPLEMENTED
date: 2026-11-05
actor: fable (pipeline-implementer, Claude Fable 5.1)
---
# P33_split_ui — implement r1

Gate at launch: intention `RATIFIED` (2026-10-24); tracker `P32_composition_tools` APPROVED, `P30_outputs_in_export_and_publish` APPROVED,
`P33_split_ui` NOT_STARTED. Launched on `0ec6f31`.

⚠ OWNER DECISIONS REQUIRED (0): none.

## What was built
**Split in the editor.** Pressing **S**, or clicking **Split** in the Composition section's Normal | Split control, turns the composition into
10D's Split: the frame becomes 2:1 (the largest 2:1 box in the stage), a 1 px hairline runs down its exact middle, and the Composition section
shows two square previews, **Left** and **Right**, each the adjusted preview clipped to its half of the frame. The re-fit keeps the composition's
centre and area (a Split is a presentation of the same framing, not a reset); the stored shape survives — pressing S again (or **Normal**)
restores a Normal rect of the kept shape at the same centre and area (the default square comes back exactly). While split, the Shape menu and X
only store the shape and orientation for Normal (the frame stays 2:1); drags, zooms, nudges, `[` `]`, Esc revert and Return work unchanged;
the handles keep the ratio locked at 2. Split is for Main and Condition photos: on an unreviewed photo the control is disabled and S does nothing.

**Split in the grid.** Tiles are numbered per output: a split photo reads the two places it fills (`01–02`), the next photo `03`, and so on in
Main (then the unreviewed photos) and in Condition; a split tile also shows a hairline at the middle of its composition rect. The folded strip
and the drag overlay follow (the overlay shows `03–04` when a split tile hovers slot 2, `05` when a normal tile hovers after two split photos);
drops still reorder by photo slot. The stage bar reads `Main 4–5 of 6` for a split photo. The ⋯ and right-click menus offer **Split** on a normal
photo and **Normal** on a split one (hint S); on a selection every selected photo whose kind differs gets its own save, one after the other, and
each tile renumbers in place when its answer lands. No backend change: export and publish of split outputs are P30's.

## Tests → criterion rows
| Row | Test (file › name) | Boundary / outcome asserted |
|---|---|---|
| C1(a) | `compose/__tests__/refit.test.ts` › P33 C1(a) (1)–(4) + "while split a shape or orientation change …" | `refit` kind change: (1) split rect `(0.028474, 0.146447, 0.943052, 0.707107)` exactly and ± 1e-3 vs the plan; (2) back to the default square (1e-6); (3) original → `(0,0,1,1)`; (4) portrait `(0, 0.333301, 1, 0.333398)`; every result passes `validate` |
| C1(b) | `workspace/__tests__/outputs.test.ts` › 4 cases | `outputNumbers` main `a{1,2} b{3,3} c{4,5} u{6,6}`, condition `d{1,2} e{3,3}`; `outputLabel(1,2) == "01–02"`, `(3,3) == "03"`; `outputTotal` |
| C2(a) | `p33_split_editor.test.tsx` › C2(a) ×2 (S; the Split control) | one `setComposition` `{kind split, shape 1:1}` rect = contract value (1e-9) and plan ± 1e-3; frame box `952 × 476 at (24, 112)`; `.compose-middle` `x1 = x2 = 500 ± 0.5`; `.compose-frame[data-kind=split]`; a repeated Split click saves nothing |
| C2(b) | › C2(b) | second S: `kind normal`, rect = default square (1e-6), no `.compose-middle`, square box back |
| C2(c) | › C2(c) | 4:5 then x while split: saves with `shape 4:5` / `orientation turned`, rect **equal** to the split rect, box stays 2:1; then S → `kind normal` 5:4 rect of the same centre/area (`w_px = 1908.5`), box `815 × 652` |
| C2(d) | › C2(d) | se handle (−100, −50): overlay box `852 × 426` from the nw corner while held; the save is 2:1 within 1 px, `x`, `y` unchanged, `w = 0.943·852/952` |
| C3(a) | › C3(a) | `.split-previews` with 2 `.split-preview` (`data-side` left/right, `aspectRatio "1 / 1"`, `aria-label` Left/Right, captions), each an `img` with the stage's URL (3 uses in the document, no new resource); `[data-half=left]` `x0 = x`, `x1 = x + w/2`; the right half `x0 = x + w/2`, `x1 = x + w`; the clip frame's width/height/left |
| C4(a) | `p33_split_grid.test.tsx` › C4(a) | Main `.tile-number` `01–02, 03, 04–05, 06`; Condition `01–02, 03`; `.tile-split` on tiles 1, 3, 5 with `left == 50%` (= `(x + w/2)·100%`), `top`, `height` of the rect; none on 2, 4, 6; aria-labels `Photo 01–02`, `Photo 04–05`, `Photo 06, to review`; Cover on tile 1 only |
| C4(b) | › C4(b) | `f`: strip order `[1..6]`, no `.tile-number`, `.tile-split` exactly on 1, 3, 5 |
| C4(c) | › C4(c) | held overlay reads `03–04` (tile 3 over slot 2, with hairline) then drop → `reorderPhoto(3, 2)`; `05` (tile 2 over slot 3) → `reorderPhoto(2, 3)`; `classifyPhoto` not called |
| C4(d) | › C4(d) | `.stage-place` `Main 4–5 of 6`, `Main 1–2 of 6`, `Main 3 of 6`, `Condition 3 of 3`, `Condition 1–2 of 3` |
| C5(a) | › C5(a) | ⋯ of tile 2 → Split: `setComposition(2, {kind split, …, rect = contract value})` once; Main renumbers `01–02, 03–04, 05–06, 07`, stage `Main 3–4 of 7`; tile 1's menu has `Normal S` and no `Split S`; tile 4 (unreviewed) has Split disabled |
| C5(b) | › C5(b) | ⌘-click 2 and 3, right-click 2 → Split: exactly one call, for photo 2; both tiles read split labels |
| C5(c) | › C5(c) | tile 6 → Split: `setComposition(6, kind split)`; Condition `01–02, 03–04`; Main unchanged |
| C6(a) | `p33_split_editor.test.tsx` › C6(a) | composing on split photo 1: `]`/`[` switch photos (2:1 ↔ square box, hairline on/off), no save; Esc idle no save; drag then Esc → one save with the entry values (`kind split`, the stored rect), placement back; Return exits, still 2 calls |

Red check (not a mutation ledger): the 14 changed production files stashed, the test files kept → 19 of the 23 new tests fail (the 4
`outputs.test.ts` cases pass because `outputs.ts` is a new untracked file); stash popped, `git status` as before.

## Assumptions
1. **Split is for Main and Condition photos (10D)**: the Split control is `aria-disabled` and S is a no-op on an unreviewed photo; the menu's Split
   is disabled there; `outputNumbers` counts an unreviewed photo as one place whatever its kind (plan: "each unreviewed counting one place as today").
2. The menu's Split / Normal entry carries the hint `S` (menuitem names `Split S` / `Normal S`), like `Compose… R`; the label is the plan's.
3. While composing, Split / Normal from a menu on the **stage photo** runs through the machine (`SetValues { cause: 'kind' }`), so the editor
   follows and the next gesture cannot overwrite it; `WorkspacePage.setKind` skips the stage photo while composing. A selection containing the
   stage photo runs both: the machine for it, the workspace for the others.
4. Saves from the grid run one after the other (`for … await`), each with its own `Not saved: <message>` toast on failure; a photo already of the
   asked kind, without a frame, or unreviewed (for Split) is skipped.
5. Preview clip surface: `data-half`, `data-x0`, `data-x1` on the clipped `.output-preview-frame` inside each `.split-preview` (which carries
   `aria-label` Left / Right and `aspect-ratio 1 / 1`, caption below); size cap 110 px per box; the P32 CSS clip technique (sized frame div).
6. The editor's middle hairline is an SVG `<line class="compose-middle">` (1 px, `--hair-strong`, `crispEdges`) on the box the overlay draws
   (the trimmed box during a trim); the tile's is a 1 px white span the height of the rect.
7. `Stage.placement` writes `first–last` with an en dash and the total as the group's outputs (Delegated: yes).
8. The overlay's projected numbers: same-group hover → the group with the dragged photo moved to the P22 `projectedSlot`; cross-group → the
   target group plus the (reclassified) photo at its end; `outputNumbers` of that list.
9. `cause: 'kind'` is handled exactly like `'set'` in the machine (apply, View, Save when the values differ) — the plan names the cause, not a new rule.

## Deviations
- Files changed beyond the plan's list (each a one-line or implied seam): `compose/machine.ts` (the `'kind'` cause of task 0), `compose/ComposeStage.tsx`
  (passes the kind to `ComposeFrame`), `compose/ComposeMode.tsx` (S and the menus' Split / Normal on the stage photo go through the machine — assumption 3).
- `workspace/FoldedStrip.tsx` is listed as changed but needed no change (it renders `PhotoGrid`, which carries the numbering and the hairline).
- The grid test imports the P31/P32 harness from `features/compose/__tests__/harness.tsx` instead of copying it.
- Tracker actor is `fable` (the prompt template says `codex`).
- Plan arithmetic asserted at the contract's exact values (plan literals also asserted ± 1e-3 where given): C1(a)(1) `w = 0.943052, h = 0.707107,
  x = 0.028474, y = 0.146447`; C1(a)(4) `h = 0.333398438, y = 0.333300781`; C2(c)'s 5:4 rect `w_px = 1908.5` → `w = 0.745508, h = 0.894454`.

## Files changed (the full write perimeter; checkpoint `d47b6ce`, 22 paths)
New: `apps/desktop/src/features/compose/SplitPreviews.tsx`, `apps/desktop/src/features/workspace/outputs.ts`, `apps/desktop/src/features/compose/
__tests__/p33_split_editor.test.tsx`, `apps/desktop/src/features/workspace/__tests__/outputs.test.ts`, `apps/desktop/src/features/workspace/__tests__/
p33_split_grid.test.tsx`.
Changed: `apps/desktop/src/features/compose/{refit.ts, machine.ts, CompositionSection.tsx, ComposeFrame.tsx, ComposeStage.tsx, ComposeMode.tsx,
useComposeKeys.ts, compose.css, __tests__/refit.test.ts}`, `apps/desktop/src/features/workspace/{PhotoTile.tsx, PhotoGrid.tsx, PhotoMenu.tsx,
WorkspacePage.tsx, Stage.tsx, workspace.css}`, `docs/implementation/master_plan.md` (P33 tracker row only), `docs/implementation/plans/P33_split_ui.md`
(Review log entry).
Not committed by this session (the runner's): this handoff. No Rust, no bindings, no settings, no other phase's row or prompt. Mutation-probe files:
none (no named-mutation regime; the red check stashed and restored the 14 changed production files through `git stash`, leaving no residue).

## Checkpoint SHA
`d47b6ce` — `CHECKPOINT (not approved): P33_split_ui implement r1 — Split in the editor and the grid`; `git status --porcelain` empty after it
(before this handoff was written).

## Stamp (tree = `d47b6ce`)
| Command | Result |
|---|---|
| `cargo test --workspace` | 529 passed / 0 failed (summed over every test binary; Rust untouched) |
| `cargo clippy --workspace --all-targets -- -D warnings` | Finished, no warnings |
| `cargo fmt --all -- --check` | exit 0 |
| `cd apps/desktop && npm run typecheck` | exit 0 (Node 22 via `PATH=/Users/davidloorenz/.nvm/versions/node/v22.22.3/bin:$PATH`, master plan §7.1) |
| `cd apps/desktop && npm test` | 34 files / 291 passed (268 before + 23 new) |
Pre-existing failures: none.

## Manual try-out
1. Open a session, select a Main photo with a preview and press **R**. Press **S**: the frame becomes a wide 2:1 box with a hairline down its exact
   middle; the Composition section shows two squares labelled Left and Right. Drag, pinch and nudge as before; pull a corner handle: the frame stays
   2:1. Pick 4:5 in Shape and press X: nothing moves (the frame is still 2:1). Press **S** again: a 5:4 (4:5 turned) frame comes back at the same
   centre and size. Press 2 then S, S: the square returns exactly.
2. Press Return. The tile shows a thin line down the middle of its frame and reads `01–02`; the next photo reads `03`; the stage bar says
   "Main 1–2 of n". Press F: the strip shows the same line on the split tile, no numbers.
3. Right-click a normal photo → **Split**; right-click a split photo → **Normal**. ⌘-click two photos and right-click one of them → Split: both
   become split (the already-split one is left alone). A Condition photo splits the same way and Condition renumbers `01–02, 03…`. Drag a split tile
   over another slot: the floating tile shows the two places it would take.
4. Export: a split photo gives two square files, Left first (P30). Publish: two media, adjacent, Left first; the cover is the left half when the split
   photo is Main 01.

⚠ OWNER DECISIONS REQUIRED (0): none.
