---
phase: P33_split_ui
title: Split in the editor and the grid — S and Normal | Split, the 2:1 frame with its exact middle, Left and Right previews, split from the grid selection, the "03–04" badge and hairline, output numbering in the grid
difficulty: high
projection: no
implementer: fable
depends_on: [P32_composition_tools, P30_outputs_in_export_and_publish]
---
# P33 — Split in the editor and the grid: S and Normal | Split, the 2:1 frame with its exact middle, Left and Right previews, split from the grid selection, the "03–04" badge and hairline, output numbering in the grid

## Goal
The frontend of 10D's **Split**: one 2:1 composition whose exact middle makes two square files, Left and Right, allowed for Main and
Condition photos. In the editor, S (or the Normal | Split control) toggles the kind: the frame becomes 2:1 with a hairline at its exact
middle, the Composition section shows two square **Left** and **Right** previews at their real proportions, and every gesture, key and control
of P31/P32 works on the 2:1 rect (the shape ratio is locked to 2 while split; the Shape menu keeps the Normal shape for toggling back). From
the grid, the ⋯ and right-click menus offer "Split" / "Normal" for the photo (or the selection); the tile of a split photo shows a middle
hairline and the two places it fills ("03–04"); later tiles are numbered after both outputs, in Main and in Condition, and the folded strip
and drag overlay follow. The backend already renders, exports and publishes split outputs (P29/P30); the first version is for work: no
polish beyond what the tile and the editor need. **Not in this phase:** P34's items; any backend change.

## Read first
Book: `10D_composition_and_framing.md` §What the photographer sees (bullet 4: "Split: one 2:1 composition whose exact middle makes two square
files, Left and Right. Allowed for Main and Condition photos. In the grid a split photo shows a middle hairline and the two places it fills
(03–04)"; bullet 3: "Normal | Split"; bullet 5: "S toggles Split"), §MC-27 Values (`kind`, "split is always 2:1"), §MC-29 Outputs ("positions
stay per photo; outputs expand in photo order") · `06` §Order (display order and numbering) · master plan §6.9, §6.10 · process `flow.md` §4, §6, §9 ·
plans `P31_compose_mode.md`, `P32_composition_tools.md` (the machine, `refit`, the Composition section — Split is the disabled control to enable),
`P28_composition_foundation.md` §The geometry contract (`shape_ratio` returns 2 for split; `default_rect` for split; `split_halves`) · code
(P31/P32 files): `features/compose/{geometry.ts, refit.ts, machine.ts, CompositionSection.tsx, OutputPreview.tsx, ComposeFrame.tsx, compose.css}`,
`features/workspace/{PhotoTile.tsx (`PhotoTileFace`, `TileComposition`), PhotoGrid.tsx (`number={index + 1}`), PhotoMenu.tsx (`entries`),
WorkspacePage.tsx (`onMenu`, the drag overlay's `projectedNumber`), FoldedStrip.tsx, Stage.tsx (`placement`), order.ts}`.

## Files expected to change
Frontend (new): `apps/desktop/src/features/compose/SplitPreviews.tsx`, `apps/desktop/src/features/workspace/outputs.ts` (output numbering helpers).
Frontend (changed): `apps/desktop/src/features/compose/{refit.ts, CompositionSection.tsx, ComposeFrame.tsx, compose.css, useComposeKeys.ts}`,
`apps/desktop/src/features/workspace/{PhotoTile.tsx, PhotoGrid.tsx, PhotoMenu.tsx, WorkspacePage.tsx, FoldedStrip.tsx, Stage.tsx, workspace.css}`.
Tests: `apps/desktop/src/features/compose/__tests__/p33_split_editor.test.tsx`, `apps/desktop/src/features/workspace/__tests__/
p33_split_grid.test.tsx`, `outputs.test.ts` (new); `refit.test.ts` gains the split rows.

## Core mechanisms (unit tests allowed)
- `features/workspace/outputs.ts` — `outputNumbers(group) → Map<photoId, { first, last }>` (display numbers per output; a split photo takes two
  consecutive numbers) and `outputLabel(first, last)` (`"03"` or `"03–04"`). Proven by `outputs.test.ts` (C1(b)).
- `features/compose/refit.ts` — the `kind` change (C1(a)), in `refit.test.ts`.

## Ordered tasks
0. **Registry first (master plan §6.10).** `PhotoMenuAction` gains `'split'` and `'normal'` (labels `Split`, `Normal`; the entry shown is the
   one that changes the photo's kind; disabled like `compose` when no rect exists). Machine: `SetValues { cause: 'kind' }` (no new event).
   CSS: `.compose-frame[data-kind="split"]`, `.compose-middle` (the hairline), `.split-previews`, `.split-preview[data-side="left"|"right"]`,
   `.tile-split` (the tile hairline), `.tile-number` text per `outputLabel`. Keys: `s` toggles the kind while composing.
1. **Re-fit for the kind change** (`refit.ts`): `normal → split`: ratio 2, centre and area preserved, then `fit` (the shape value is kept);
   `split → normal`: the ratio of the kept shape (free → keep the rect; original → whole frame), centre and area preserved, `fit`. Shape-menu
   changes while split update `shape` only (the frame stays 2:1; the new shape applies when toggling back); X (orientation) while split is a no-op
   on the frame (stored for the Normal shape).
2. **Editor**: the Normal | Split segmented control enabled; `s` key → `SetValues { values: refit(values, { kind: toggle }), cause: 'kind' }`;
   `ComposeFrame` draws a vertical hairline at the frame's exact middle while `kind === 'split'` (`x = frameBox.left + frameBox.width / 2`, 1 px,
   `--hair-strong`); handles keep the ratio locked at 2 (treated as a locked shape); the output preview becomes `SplitPreviews`: two square boxes
   (max 110 px each, side by side, labelled `Left` and `Right`) showing the left and right halves of the rect from the same preview image.
3. **Grid**: `outputNumbers` from `displayOrder` groups (Main: main photos then unreviewed photos, each unreviewed counting one place as today;
   Condition likewise); `PhotoGrid` passes `number={first}` and `last` to the tile; `PhotoTileFace` shows `outputLabel(first, last)` in
   `.tile-number` and, for a split photo, `.tile-split` (a hairline at the middle of the composition rect inside `TileComposition`); `aria-label`
   `Photo 03–04`; the Cover badge stays on the first main photo (its left half); the drag overlay's projected number uses the same helper
   (`projectedNumber` → `outputLabel` of the projected slot: dropping a split photo at slot 3 of a group shows `03–04`); `Stage.placement` reads
   `Main 3–4 of 8` for a split photo (`total` = outputs in the group).
4. **Menus**: `entries(photo)` adds `Split` (when `composition.kind === 'normal'`) or `Normal` (when `split`) after the toggle entry; `onMenu`
   applies `refit` and saves through `setComposition` for the photo or, when the photo is part of the selection, for every selected photo whose
   kind differs (each its own save, the queue serialises).
5. Tests, `npm run typecheck`, `npm test`.

## Criteria (6 criteria, 15 rows; 4 `[rule]` rows)
Fixture: P31's (2560×1707 frame, default square). The split default rect for that frame is `(0, 0.125073228, 1, 0.749853544)` (P28 C1 case 10).

| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | `refit.test.ts` | (1) default square → `kind split`; (2) that split rect → `kind normal` (shape `1:1`); (3) split rect → normal with shape `original`; (4) split on a portrait frame 1707×2560 from its default square `(0, 0.166601562, 1, 0.666796875)` | (1) centre `(0.5, 0.5)`, area `1707²`: `w_px = sqrt(1707²·2) = 2414` > 2560? no: `2414 ≤ 2560` → `w = 0.943, h = 1207/1707 = 0.7071`, centred `x = 0.0285`, `y = 0.1465` (± 1e-3; the area is kept, the frame is wide enough); (2) back to a square of area `2414·1207`: `w_px = 1707` → the default square exactly (`1e-6`); (3) `(0, 0, 1, 1)`; (4) `w_px = sqrt(1707²·2) = 2414 > 1707` → fit: `w = 1`, `h = 853.5/2560 = 0.333398`, centred `y = 0.333301` (= P28 case 11) | 10D ("one 2:1 composition"; "S toggles Split"); P32 re-fit contract |
| C1(b) | `outputs.test.ts` | Main group `[a (split), b, c (split), u (unreviewed)]`; Condition `[d (split), e]` | `outputNumbers(main)` = `a → {1, 2}`, `b → {3, 3}`, `c → {4, 5}`, `u → {6, 6}`; `outputLabel(1, 2) == "01–02"`, `outputLabel(3, 3) == "03"`; condition `d → {1, 2}`, `e → {3, 3}` | 10D ("the two places it fills (03–04)"); MC-29 §Outputs ("outputs expand in photo order") |
| C2(a) | composing on the default square | press `s`; separately click `Split` in the segmented control | one `setComposition` with `kind 'split'`, `shape '1:1'` (kept), rect `(0.0285, 0.1465, 0.943, 0.7071)` ± 1e-3; the frame box becomes 2:1 (`952 × 476` at `(24, 112)`: the largest 2:1 box in `952 × 652`); `.compose-middle` present at `x = 500 ± 0.5` (the frame's exact middle) | 10D ("S toggles Split"; "whose exact middle makes two square files") |
| C2(b) | after (a) | press `s` again | `kind 'normal'`, rect = the default square (`1e-6`); no `.compose-middle` | 10D (Normal \| Split) |
| C2(c) | split, composing | choose `4:5` in the Shape menu; press `x` | saves with `shape '4:5'` / `orientation 'turned'` but the rect **unchanged** (still 2:1, `w = 0.943`); the frame box stays 2:1; then `s` → `kind 'normal'` with a 5:4 rect of the same centre and area | MC-27 §Values ("split is always 2:1"; the shape and its turned form apply to Normal) |
| C2(d) | split, composing | drag the `se` handle by `(−100, −50)` px | the rect stays exactly 2:1 (`w·2560 == 2·h·1707` ± 1 px) and shrinks from the `nw` corner (locked) | 10D (handles; "a locked shape keeps its ratio"); MC-27 §Validation (a split exactly 2:1) |
| C3(a) | split, composing | the Composition section | `.split-previews` with two `.split-preview` boxes, `data-side` `left` and `right`, each `style.aspectRatio == "1 / 1"`, labelled `Left` and `Right`, each containing an `img` with the adjusted preview URL; the left box's inner transform shows the left half (its `object-position`/`transform` places the rect's left half: assert `data-half="left"` and the computed clip fractions `x`, `x + w/2` on the element's dataset) | `[rule]` 10D ("the output previews at their real proportions"; Left and Right) |
| C4(a) | the normal workspace; photo 1 (main, `kind split`), 2 (main), 3 (main, split), unreviewed 4; condition 5 (split), 6 | tiles | `.tile-number` texts in Main order: `01–02`, `03`, `04–05`, `06`; Condition: `01–02`, `03`; tiles 1, 3 and 5 contain `.tile-split` positioned at the middle of the composition rect (`left == (x + w/2)·100%`); `aria-label` of tile 1 is `Photo 01–02`; the `Cover` badge is on tile 1 only | `[rule]` 10D ("In the grid a split photo shows a middle hairline and the two places it fills (03–04)") |
| C4(b) | the same, folded (`f`) | the strip | the strip tiles keep order and the split tiles carry `.tile-split` (no numbers in the strip, as today) | `[rule]` 10D (grid and strip) |
| C4(c) | drag tile 3 (split) onto tile 2's slot (P22 harness: `releaseAt`), and separately tile 2 (normal) onto tile 3's slot | the drag overlay | while tile 3 hovers slot 2 its overlay `.tile-number` reads `03–04` (the outputs before the slot are tile 1's two, then its own two); while tile 2 hovers slot 3 the overlay reads `05` (`01–02`, `03–04`, then it); the drops call `reorderPhoto(3, 2)` and `reorderPhoto(2, 3)` — photo positions, unchanged semantics | MC-29 §Outputs ("Positions stay per photo"); P19/P22 drop behaviour |
| C4(d) | the stage bar with photo 3 (split) active | placement | `Main 4–5 of 6` (outputs: 2 + 1 + 2 + 1) | `[rule]` 10D (the places a split fills) |
| C5(a) | the ⋯ menu of tile 2 (normal) | click `Split` | one `setComposition(2, { kind: 'split', … rect = refit })`; the tile shows `03–04` after the answer (in-place update) and tile 3 moves to `05–06`; the menu of tile 1 (split) offers `Normal` and not `Split` | 10D ("split from grid selection" — owner phasing; the menu is the grid's control) |
| C5(b) | ⌘-click tiles 2 and 3 (selection); right-click tile 2 → `Split` | a selection | `setComposition` called for photo 2 only (photo 3 is already split: no call); both tiles read split labels | 10D (Split for the selection); the saves queue |
| C5(c) | tile 6 (condition, normal) → `Split` | a condition photo | `setComposition(6, kind split)`; condition numbering `01–02`, `03–04` | 10D ("Allowed for Main and Condition photos") |
| C6(a) | composing on photo 1 (split); press `[`/`]`; Esc; Return | the P31/P32 flows on a split photo | navigation, revert and finish work unchanged: Esc reverts to the entry values (`kind split`), Return exits; no save when nothing changed | MC-29 §Dirty; P31 rules 6, 13 |

## Delegated choices
How the split previews clip the halves (CSS `clip-path` on a transformed `img`, or `object-fit`/`object-position` on a sized box — the
`data-half` and dataset fractions are the assertion surface); the previews' size cap; the hairline colour/thickness in the tile; whether
`Stage.placement` writes `3–4` with an en dash (yes, the same as the tile label); the exact overlay projected-number arithmetic for a split photo
being dragged (two outputs: the label shows `first–last` of its projected slot).

## Manual try-out
1. Open the editor on a main photo and press S: the frame becomes a wide 2:1 with a hairline down its exact middle; the Composition section shows
   Left and Right squares. Drag and zoom as before; press S again to return to the square.
2. Leave the editor: the tile shows a hairline and reads `01–02`; the next photo reads `03`. The stage bar says "Main 1–2 of n".
3. Right-click a normal photo → Split; right-click a split photo → Normal; select two photos (⌘-click) → Split: both become split. Condition
   photos split the same way.
4. Export: a split photo gives two square files, Left first (P30). Publish: two media, adjacent, Left first; the cover is the left half when the
   split photo is main 01.

## Notes
- **The shape survives a split.** Toggling back restores the Normal shape at the same centre and area — a Split is a presentation of the same
  framing, not a reset.
- **Numbering is a display rule, not a position change**: positions stay per photo (MC-29); only the numbers and the stage bar count outputs,
  so a drag still calls `reorderPhoto(photoId, slot)` with the photo slot as before.
- Registry additions made by the planner in master plan §6.10. No intention-book change, no backend change.
- Estimated size: ~12 files, ~900 lines (≈ 300 editor + grid, ≈ 600 tests).

## Review log
### 2026-11-05 — implement r1 (Fable, `pipeline-implementer`)
**Built.** 10D's Split in the editor and the grid. New: `features/workspace/outputs.ts` (`outputCount`, `outputNumbers(group) → Map<photoId,
{ first, last }>`, `outputTotal`, `outputLabel(first, last)` → `"03"` / `"03–04"`); `features/compose/SplitPreviews.tsx` (two square boxes
`.split-preview[data-side]` labelled Left / Right, the same adjusted preview clipped to each half by the P32 CSS technique; `data-half`, `data-x0`,
`data-x1` on the clipped frame). Changed: `refit.ts` (`RefitChange { kind: 'kind', to }` — `normal → split` ratio 2 at the same centre and area then
`fit`, the shape kept; `split → normal` the kept shape's ratio, `free` keeps the rect, `original` the whole frame; while split a shape or
orientation change updates the field only and leaves the 2:1 rect; `reshaped` no longer treats `original` as the whole frame while split),
`machine.ts` (`SetValuesCause` gains `'kind'`, handled as `'set'`), `CompositionSection.tsx` (Normal | Split enabled, `canSplit`, the previews
switch to `SplitPreviews` while split), `ComposeFrame.tsx` (`kind` prop, `.compose-frame[data-kind]`, `.compose-middle` hairline at
`left + width/2`), `ComposeStage.tsx` (passes the kind), `useComposeKeys.ts` (`s` → `onToggleKind`), `ComposeMode.tsx` (`setKind`/`toggleKind`
through the machine with `cause 'kind'`; the menus' Split / Normal on the stage photo — alone or inside the selection — go through the machine,
the rest to the workspace), `compose.css`; `workspace/{outputs.ts, PhotoTile.tsx (last, outputLabel, .tile-split at (x + w/2)·100 %, aria-label
Photo 03–04), PhotoGrid.tsx (outputNumbers per group), PhotoMenu.tsx (PhotoMenuAction split | normal; the entry that changes the kind, hint S,
disabled like Compose… and for unreviewed photos), WorkspacePage.tsx (setKind: one save per named photo whose kind differs, in display order, each
answer patched in place; the overlay's projected label from outputNumbers of the projected list), Stage.tsx (placement counts outputs: Main 4–5
of 6), workspace.css}`. No Rust, no bindings change.
**Tests.** `refit.test.ts` (+5: C1(a)(1)–(4) and the shape-while-split rule), `workspace/__tests__/outputs.test.ts` (C1(b), 4), `compose/__tests__/
p33_split_editor.test.tsx` (C2(a) ×2, C2(b)–(d), C3(a), C6(a)), `workspace/__tests__/p33_split_grid.test.tsx` (C4(a)–(d), C5(a)–(c)). 23 new tests;
`npm test` 34 files / 291 passed; `npm run typecheck` clean; Rust suite, clippy and fmt unchanged and green (handoff stamp). Red check: with the 14
changed production files stashed and the tests kept, 19 of the 23 fail (the 4 `outputs.test.ts` cases pass because `outputs.ts` is a new file).
**Judgment calls (Assumptions).**
1. **Split is for Main and Condition photos (10D)**: on an unreviewed photo the Split control is `aria-disabled`, S does nothing and the menu's Split
   entry is disabled; `outputNumbers` counts an unreviewed photo as one place whatever its kind (the plan's "each unreviewed counting one place as
   today").
2. **The menu's Split / Normal entry carries the hint `S`** like `Compose… R` (menuitem name `Split S` / `Normal S`); the label is the plan's.
3. **While composing, the stage photo's kind change from a menu runs through the machine** (so the editor follows and the next gesture does not
   overwrite it); `WorkspacePage.setKind` skips the stage photo while composing. For a selection containing the stage photo both paths run: the
   machine for the stage photo, the workspace for the others.
4. **Saves from the grid run one after the other** (`for … await`), each with its own `Not saved: <message>` toast on failure; a photo already of
   the asked kind, without a frame, or unreviewed (for Split) is skipped.
5. **The preview's clip surface**: `data-half`, `data-x0 = x₀`, `data-x1 = x₀ + w/2` on the clipped `.output-preview-frame` inside each
   `.split-preview`; the box carries `aria-label` Left / Right and a caption below (the box itself is `aspect-ratio 1 / 1`); size cap 110 px.
6. **The 2:1 hairline in the editor is an SVG `<line>`** (`.compose-middle`, 1 px, `--hair-strong`, `crispEdges`) drawn on the box the overlay draws
   (the trimmed box during a trim); the tile hairline is a 1 px white span the height of the rect.
7. **`Stage.placement`** writes `first–last` with an en dash (Delegated: yes) and the total as the group's outputs.
8. The drag overlay's projected list for a same-group hover is the group with the dragged photo moved to the P22 `projectedSlot`; for a cross-group
   hover the target group plus the photo (reclassified) at its end — `outputNumbers` of that list gives `first–last`.
**Plan arithmetic (asserted at the contract's exact values, plan literals ± 1e-3 also asserted where given).** C1(a)(1): `w_px = 1707·√2 =
2414.21`, `w = 0.943052`, `h = 0.707107`, `x = 0.028474`, `y = 0.146447`. C1(a)(4): `h = 0.333398438`, `y = 0.333300781`. C2(c)'s "5:4 rect of the
same centre and area": `w_px = sqrt(1707²·1.25) = 1908.5`, `w = 0.745508`, `h = 0.894454`. C2(d): the plan's `(−100, −50)` on the 2:1 frame is
exactly along the diagonal, so the trim is `0.943·852/952` wide and still 2:1.
**Deviations.** (a) Files changed beyond the plan's list, each one line or a small seam the plan implies: `compose/machine.ts` (the `'kind'` cause the
plan's task 0 names), `compose/ComposeStage.tsx` (passes the kind to `ComposeFrame`), `compose/ComposeMode.tsx` (S and the menu's Split / Normal on the
stage photo must go through the machine — see assumption 3). (b) `workspace/FoldedStrip.tsx` is listed as changed but needed no change (it renders
`PhotoGrid`, which carries the numbering and the hairline). (c) The grid test imports the P31 harness from `features/compose/__tests__/harness.tsx`
(no copy). (d) Tracker actor is `fable`, not the prompt template's `codex`.
**Not done.** No live try-out in the Tauri window (no interactive display); the owner's Manual try-out stands. No backend change: export and publish
of split outputs are P30's.
