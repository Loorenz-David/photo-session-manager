---
phase: P34_composition_polish
title: Composition polish — the Z loupe, snapping to the centre and edges (⌥ suspends), copy and paste a composition and apply it to the selection, the unsaved dot, reduced motion, a larger default window, and the measured frame and render times for the owner
difficulty: normal
projection: no
depends_on: [P33_split_ui]
---
# P34 — Composition polish: the Z loupe, snapping to the centre and edges (⌥ suspends), copy and paste a composition and apply it to the selection, the unsaved dot, reduced motion, a larger default window, and the measured frame and render times for the owner

## Goal
The remaining editor items of 10D §What the photographer sees and R37, plus the performance report. **Z** held shows a 100 % loupe of the
preview under the pointer and changes nothing. **Snapping**: while dragging, the composition snaps to the frame's centre lines and to the
image edges when within a few screen pixels; **⌥** held suspends it. **Copy / paste**: ⌥⌘C copies the active photo's composition values,
⌥⌘V pastes them onto the active photo, and the grid menu's "Apply composition to selected" pastes them onto every selected photo through one
backend command `composition_apply` (values re-fitted per photo, each photo's own frame; a photo whose frame refuses the values keeps its
composition and is named in the toast). The **unsaved dot** marks a photo whose change is saving or waiting to save (and clears when the save
lands or fails). **Reduced motion** (`prefers-reduced-motion: reduce`) removes the glide, the browser slide and the Original/Adjusted
crossfade. The window opens **larger** by default (R37 "the window opens larger": 1440 × 900, minimum 1100 × 720). And the **measured targets**
of 10D: a built-in frame-time meter during drags and pinches (⌥⌘P toggles an overlay reading the last second's average and worst frame time)
and the master and output render times written to the log per photo; the implementer records the dev-Mac numbers in the handoff for the owner.
**Not in this phase:** colour and light tools; the phone remote; anything after 10D.

## Read first
Book: `10D_composition_and_framing.md` §What the photographer sees (bullet 5: "hold Z for a 100 % loupe that changes nothing. Snapping to the
centre and the image edges; ⌥ suspends it"; bullet 6: "a dot marks unsaved changes in between"), §Performance targets · `15` R37 ("the window
opens larger") · `06` §Original / Adjusted (the 700 ms crossfade that reduced motion removes) · evidence `design_handoff.md` §Tokens (motion) ·
`12` §Logs ("every provider call with duration") · master plan §6.3, §6.7 (`composition_apply`), §6.10, §7.2 (dev app) · process `flow.md` §4, §6,
§9 · plans P31–P33 (the machine gains `Snap` behaviour inside `panBy`; the saves hook exposes pending state), `P28_composition_foundation.md`
(`CompositionService::set`; the apply command mirrors it) · code: `apps/desktop/src-tauri/tauri.conf.json` (`windows :13`: `1100 × 720`, no
`minWidth`), `apps/desktop/src-tauri/src/commands/compositions.rs` (P28: `composition_set`), `crates/core/src/services/composition.rs` (P28),
`apps/desktop/src/features/workspace/StageImage.tsx` (`CROSSFADE_MS`, the `.crossfade` CSS), `WorkspacePage.tsx` (`reducedMotion :160` already read
for the drop animation), `features/compose/{machine.ts, useComposeGesture.ts, useCompositionSaves.ts, ComposeStage.tsx, compose.css}`,
`crates/infra/src/imaging/render.rs` and `crates/core/src/services/export.rs` (where the timings are logged), `tracing` usage (`logging.rs`).

## Files expected to change
Frontend (new): `apps/desktop/src/features/compose/{Loupe.tsx, snapping.ts, useClipboardComposition.ts, FrameMeter.tsx}`.
Frontend (changed): `apps/desktop/src/features/compose/{machine.ts, useComposeGesture.ts, useCompositionSaves.ts, ComposeStage.tsx, ComposeFrame.tsx,
useComposeKeys.ts, compose.css}`, `apps/desktop/src/features/workspace/{PhotoTile.tsx, PhotoMenu.tsx, WorkspacePage.tsx, StageImage.tsx,
workspace.css}`, `apps/desktop/src/services/compositions.ts` (`applyComposition`), `apps/desktop/src/styles/globals.css` (the reduced-motion rules).
Backend (small, in this fable phase because it exists only for the frontend action): `crates/core/src/services/composition.rs` (`apply`),
`apps/desktop/src-tauri/src/commands/compositions.rs` (`composition_apply`), `apps/desktop/src-tauri/src/lib.rs` (`collect_commands!`),
`apps/desktop/src/services/bindings.ts` (regenerated), `apps/desktop/src-tauri/tauri.conf.json` (window size).
Tests: `apps/desktop/src/features/compose/__tests__/{snapping.test.ts, p34_loupe_snap.test.tsx, p34_clipboard.test.tsx, p34_motion_dot.test.tsx}`,
`crates/testkit/tests/p34_composition_apply.rs` (new).

## Core mechanisms (unit tests allowed)
- `features/compose/snapping.ts` — `snap(rect, candidates, thresholdFrac) → { rect, guides }` (C1(a)).

## Ordered tasks
0. **Registry first (master plan §6.10).** Command `composition_apply(photo_ids: Vec<CommandId>, values: CompositionValuesDto) -> Result<ApplyResultDto
   { applied: Vec<i64>, refused: Vec<{ photo_id, reason }> }, CommandError>`: for each photo, `refit` is **not** the backend's job — the frontend
   sends per-photo re-fitted values? **No**: the backend cannot re-fit without the frontend's rule, and the frontend has each photo's frame
   (`composition.frame`), so the **frontend** re-fits per photo and the command takes `Vec<(photo_id, values)>`: `composition_apply(items:
   Vec<CompositionApplyItemDto { photo_id, values }>)`, validating and saving each through `CompositionService::set` in one call (no transaction
   across photos: each save is its own; a refusal does not undo the others), returning `applied` and `refused` with reasons. Keys while composing:
   `z` (hold) loupe, `⌥` (hold) suspends snapping, `⌥⌘c` copy, `⌥⌘v` paste, `⌥⌘p` toggles the meter. Menu action `'apply'` (`Apply composition to
   selected`, shown when the clipboard holds values and the selection is non-empty). CSS: `.compose-loupe`, `.compose-guide[data-axis]`,
   `.tile-unsaved`, `.frame-meter`. `SNAP_PX = 6` (screen pixels). Window: `tauri.conf.json` `width 1440`, `height 900`, `minWidth 1100`,
   `minHeight 720`.
1. **Loupe** (`Loupe.tsx`): while `z` is held (keydown without repeat → show; keyup/blur → hide) a `.compose-loupe` box (240 px) follows the
   pointer and shows the preview at 100 % (the 2560 px image at scale 1) centred on the image point under the pointer; it is pointer-transparent
   (`pointer-events: none`) and dispatches nothing to the machine; releasing changes nothing (`setComposition` not called).
2. **Snapping** (`snapping.ts` + the machine's drag rule): candidates for the rect's centre x/y = the frame's centre (`0.5`), and for its edges =
   the image edges (`x = 0`, `x + w = 1`, `y = 0`, `y + h = 1`); during `PointerMove` (drag only; not wheel pans, not zoom) the proposed rect is
   snapped when a candidate is within `SNAP_PX / (W_t·s)` (resp. `H_t·s`) fractions — the nearest candidate per axis wins; snapped axes draw a
   `.compose-guide[data-axis="x"|"y"]` line in the overlay; `altKey` on the move event suspends snapping (no snap, no guides); the save on release
   carries the snapped rect.
3. **Clipboard** (`useClipboardComposition.ts`): an in-memory clipboard (not the system clipboard) of `CompositionValues`; ⌥⌘C copies the active
   photo's `values`; ⌥⌘V pastes: `SetValues { values: refitForFrame(clip, frame), cause: 'paste' }` (re-fit = centre/area-preserving `fit` onto this
   photo's frame, keeping kind/shape/turns/flip/straighten); the grid action `apply` builds `items` for every selected photo with a frame (re-fit per
   photo) and calls `applyComposition(items)`; the toast reads `Applied to n photos` or `Applied to n photos; m refused: 03, 05` (labels).
4. **Unsaved dot**: `useCompositionSaves` exposes `pendingIds: Set<photoId>` (in flight or queued); `PhotoTileFace` shows `.tile-unsaved` for those
   photos (both in the browser overlay and the normal grid); the active photo's stage bar shows the same dot while composing.
5. **Reduced motion**: a shared `prefersReducedMotion()` (the P08 `matchMedia` read) and CSS `@media (prefers-reduced-motion: reduce)` rules in
   `globals.css` set `--dur-crossfade: 0ms`, `--dur-med: 0ms`, `--dur-slow: 0ms` for the editor's glide, the browser slide and the crossfade; the
   `data-gliding` attribute is still set and cleared (tests of P32 keep passing) but the transition is instant.
6. **Window**: `tauri.conf.json` as task 0.
7. **Meter and timings**: `FrameMeter.tsx` records `requestAnimationFrame` deltas while a gesture is active (machine phase ≠ idle), shows average and
   worst over the last second when toggled with ⌥⌘P; Rust: `tracing::info!(photo_id, slot, ms, "render_output done")` and `info!(raw, adjusted_gen,
   ms, "master rendered")` (the time from `process` to `master_ready`). The handoff's **Manual try-out** section reports the dev-Mac numbers: frame
   time on drag and pinch, master time, output time per photo for the ZV-E10 session; the a7R / work-Mac numbers are owed by the owner's run.
8. Tests (unit, component, one Rust behaviour file for `composition_apply`), `cargo test --workspace`, clippy, fmt, bindings regeneration,
   `npm run typecheck`, `npm test`.

## Criteria (7 criteria, 13 rows; 3 `[rule]` rows)
Fixture: P31's.

| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | `snapping.test.ts` | rect `(0.1704, 0, 0.6668, 1)` (centre `0.5038`, 3.7 frame px from the centre at `W_t·s = 977.81`), threshold `6/977.81`; rect `(0.004, 0, 0.6668, 1)` (left edge 3.9 px from `0`); rect `(0.1, 0, 0.6668, 1)` (nothing within 6 px) | first → `x = 0.1666` (centre `0.5`) with guide `x`; second → `x = 0` with guide `x`; third → unchanged, no guides | 10D ("Snapping to the centre and the image edges") |
| C2(a) | composing; `keydown z` (no repeat); `pointermove` to `(600, 300)`; `keyup z` | the loupe | `.compose-loupe` present while held, positioned at the pointer, containing an `img` with the preview URL at `width: 2560px` and translated so the image point under the pointer is centred (`translate(−(ix − 120), −(iy − 120))` where `ix, iy` = the pointer's image coordinates: `(600 − 11.09)/0.381956 = 1542`, `(300 − 24)/0.381956 = 722.6`); after `keyup` the loupe is gone; `setComposition` never called | `[rule]` 10D ("hold Z for a 100 % loupe that changes nothing") |
| C2(b) | composing; drag from `(500, 350)` to `(503.7, 350)` (centre 3.7 px right of the frame centre) and release; separately the same with `altKey: true` on the moves | snapping and ⌥ | first: during the move a `.compose-guide[data-axis="x"]` is shown and the save has `x = 0.166601562` (snapped to the centre); second: no guide, save `x = 0.166601562 + 3.7/977.81 = 0.170386` | 10D ("Snapping to the centre and the image edges; ⌥ suspends it") |
| C3(a) | composing on photo 1 with values `V` (a 4:5 rect, straighten 2); press ⌥⌘C; `]` to photo 2 (frame 1707×2560); press ⌥⌘V | copy and paste | one `setComposition(2, V')` with `V'` = `V` re-fitted to the portrait frame (shape, orientation, turns, flip, straighten equal to `V`; rect centred, area-preserving, inside) | 10D (editor conveniences — owner phasing "copy/paste composition") |
| C3(b) | the grid: ⌘-click tiles 2 and 3; right-click tile 2 → `Apply composition to selected` (clipboard holds `V`) | apply to the selection | one `applyComposition([{photo_id: 2, values: V'₂}, {photo_id: 3, values: V'₃}])`; the mock answers `{ applied: [2], refused: [{ photo_id: 3, reason: 'off_shape' }] }` → toast `Applied to 1 photo; 1 refused: 03`; tile 2's `.tile-frame` updates in place | 10D ("apply-to-selected" — owner phasing); MC-27 §Validation (a refusal keeps the previous composition) |
| C3(c) | `crates/testkit/tests/p34_composition_apply.rs`: three photos with frames; `composition_apply([(a, valid), (b, off-shape), (c, valid)])` | the command | returns `applied == [a, c]`, `refused == [(b, "off_shape")]`; `a` and `c` have `revision 1`, `b` unchanged (`revision 0`); each saved photo's session `change_seq` bumped once (MC-6) | MC-27 §Validation; MC-29 §Dirty |
| C4(a) | composing; `setComposition` returns a pending promise; drag and release | the unsaved dot | the active photo's tile (in the browser overlay) and the stage bar show `.tile-unsaved`; resolving the promise removes them; a rejected promise removes them too (and shows the P31 toast) | 10D ("a dot marks unsaved changes in between") |
| C5(a) | `matchMedia('(prefers-reduced-motion: reduce)')` mocked to match; a handle trim release (P32 C4(a)); the browser edge reveal; an adjusted render replacing the previous (`StageImage` with two layers) | reduced motion | the stage still sets and clears `data-gliding` but `.compose-stage`'s computed `transition-duration` is `0s`; `.compose-browser`'s transition `0s`; `.crossfade` `transition-duration` `0s` (the new layer is shown at once) | `[rule]` owner phasing ("reduced motion"); design handoff §Tokens (motion) |
| C5(b) | the same media query not matching | normal motion | `.crossfade` transition `0.7s` (ch06), `.compose-stage` glide `var(--dur-med)` | ch06 §Original / Adjusted (700 ms) |
| C6(a) | `apps/desktop/src-tauri/tauri.conf.json` read in a test (`p34` module in the desktop crate or a vitest reading the JSON) | the window | `app.windows[0]` has `width 1440`, `height 900`, `minWidth 1100`, `minHeight 720` | `[rule]` R37 ("the window opens larger") |
| C7(a) | composing; press ⌥⌘P; drag for 10 fake frames of 16 ms and 1 of 48 ms | the meter | `.frame-meter` present and reading `avg 19 ms · worst 48 ms` (format `avg <n> ms · worst <n> ms`, integers); ⌥⌘P again hides it | 10D §Performance targets ("Dragging and pinching keep the display's refresh rate … measured in the build") |
| C7(b) | `p34_composition_apply.rs` (or a P29 export row re-run): one export of one photo with the tracing subscriber capturing | the log lines | the log holds one `master rendered` line with `raw`, `adjusted_gen`, `ms` and one `render_output done` line with `photo_id`, `slot`, `ms` (integers ≥ 0) | 10D §Performance targets ("Output time per photo and master render time are recorded"); ch12 §Logs |
| C7(c) | the handoff's Manual try-out (not a test): the implementer runs the dev app with the meter on, drags and pinches on a 2560 px preview, exports a session of ZV-E10 photos | the report for the owner | the handoff states frame time (avg/worst) on drag and on pinch, the master time per photo and the output time per photo as measured on the dev Mac, and names the a7R / work-Mac run as owed | 10D §Performance targets (reported to the owner) |

## Delegated choices
Whether the clipboard also writes a JSON string to the system clipboard (not required); the loupe size (240 px) and border; the guide colour;
whether the meter counts frames via rAF deltas or `PerformanceObserver`; the toast wording beyond the fixed `Applied to …` prefix; the exact
`minWidth`/`minHeight` keys as Tauri 2 spells them (`minWidth`, `minHeight`).

## Manual try-out
1. In the editor hold Z: a loupe shows the pixels under the pointer at 100 %; release: nothing moved. Drag the photo slowly past the frame's
   centre: it clicks into place with a guide line; hold ⌥ to drag freely.
2. ⌥⌘C on a framed photo, `]` to the next, ⌥⌘V: the same framing lands, fitted to that photo. Select several tiles, right-click → "Apply
   composition to selected": all take it; any that cannot (a frame too different) is named in the toast and keeps its own.
3. Drag and watch the tile: a small dot shows until the save lands. Turn on "Reduce motion" in System Settings › Accessibility: glides, slides and
   the Original/Adjusted crossfade become instant.
4. Quit and relaunch: the window opens at 1440 × 900 and cannot be made smaller than 1100 × 720.
5. ⌥⌘P shows the frame meter; drag and pinch: read the average and worst frame time (the owner's targets: the display's refresh rate). The
   handoff lists the numbers measured here and the export timings from the log.

## Notes
- **Why `composition_apply` is backend work inside a fable phase**: it exists only to serve the grid action and is twenty lines over
  `CompositionService::set`; splitting it into a Codex phase would cost more than it saves. The frontend re-fits per photo because only it has
  the re-fit rule (P32); the backend validates every item exactly as a single save.
- **The in-memory clipboard** avoids a system-clipboard permission and the question of what pasting a composition into another app would mean.
- Measurements are reported, not asserted (10D: "measured in the build, not tests"); the owner's targets are read from the meter.
- Registry additions made by the planner in master plan §6.3, §6.7, §6.10. No intention-book change.
- Estimated size: ~18 files, ~1 100 lines (≈ 450 features, ≈ 100 backend, ≈ 550 tests).

## Review log
- 2026-11-05, codex implement r1 — Built the loupe, drag snapping with ⌥ bypass and guides, in-memory copy/paste and batch apply, pending dots, reduced-motion styles, larger window, frame meter, and render timing logs. Assumptions: clipboard stays in memory; paste uses the existing centre/area re-fit on the target frame; missing target frames are reported as refused. Deviations: `Stage.tsx` and `PhotoGrid.tsx` connect the shared pending/selection state, `harness.tsx` mocks the new service call, `machine.test.ts` adapts an existing nudge expectation to Alt bypass, and `crates/integrations/src/render_output.rs` logs actual output job elapsed time. The C3(c) test uses the application service because `psm-testkit` cannot depend on the Tauri desktop crate. The C2(b) fixture requests a save when snapping returns the unchanged entry rect; the existing machine correctly avoids an identical save. `await cua.getState();` was refused (`MCP tool call requires approval, but approval policy is never`), so the browser/motion check and live ZV-E10 timings remain unverified. Full Rust suite, clippy, fmt, UI suite (300 tests), and typecheck pass after the last code edit; phase remains IMPLEMENTING pending the blocked checks and plan correction.
