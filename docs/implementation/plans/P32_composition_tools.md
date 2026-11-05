---
phase: P32_composition_tools
title: Composition tools — the left Tools panel with fold-open sections (Composition first, remembered, ⌘T), Shape menu, turn, rotate, flip, straighten, trim handles that glide back, the nudge and shape keys, [ ] navigation, Centre / Reset / Done, and the Re-frame confirmation
difficulty: high
projection: no
implementer: fable
depends_on: [P31_compose_mode]
---
# P32 — Composition tools: the left Tools panel with fold-open sections (Composition first, remembered, ⌘T), Shape menu, turn, rotate, flip, straighten, trim handles that glide back, the nudge and shape keys, [ ] navigation, Centre / Reset / Done, and the Re-frame confirmation

## Goal
The second frontend half of 10D's editor for Normal compositions. A **Tools panel** on the left is made of sections that fold open and closed
(remembered across launches) and is hidden and shown with ⌘T; **Composition** is its first section: Normal | Split (Split disabled until P33),
the Shape menu, Turn (X), rotate 90° left/right, flip, a straighten control, the output preview at its real proportions, and Centre, Reset
and Done. Corner and edge handles on the frame trim the composition (a locked shape keeps its ratio) and on release the frame glides back to
the centre of the stage. Keys, stage only, while composing: arrows nudge one screen pixel (Shift ×10), `.` centres, 1–6 and 0 pick Original,
1:1, 4:5, 3:2, 2:3, 16:9, Free, X turns the frame, `[` `]` move to the previous/next photo keeping the changes, ⌘⌥R resets to the default.
Every control runs through P31's machine as a `SetValues`/`Nudge` event (ignored during a pointer gesture) and saves through the same queue.
Re-fit rule (one pure function): a shape, turn, rotation, flip or straighten change keeps the composition's **centre and area** where the new
ratio and the frame allow, shrinking about the centre until the rect fits the frame and, with straighten, the rotated image. The **Re-frame**
confirmation: opening the editor on a `needs_review` photo shows a banner naming the cause (`review_reason geometry` → "Changed in Capture One:
straighten here instead"; `frame` → "Capture One changed this photo's frame — check the framing and press Return") and the stored fractions
re-fitted to the photo's current frame; Return (or Done, or any save) confirms and clears the flag; export/publish refusals surface their message as a toast. **Not in this
phase:** Split (P33); Z loupe, snapping, copy/paste, apply-to-selected, unsaved dot, reduced motion, window size, measurements (P34). Later
colour and light tools are more sections of the same panel (not built here; the panel is built to take them).

## Read first
Book: `10D_composition_and_framing.md` §What the photographer sees (bullet 3: the Tools panel and the Composition section in full; bullet 5: the
keys "arrows nudge one screen pixel (Shift ×10); `.` centres; 1–6 and 0 pick Original, 1:1, 4:5, 3:2, 2:3, 16:9, Free; X turns the frame; [ ]
previous/next photo keeping changes; … ⌘⌥R resets to the default"; "corner and edge handles trim the frame (a locked shape keeps its ratio) and on
release the frame glides back to the centre"), §MC-27 Values (`orientation`, `quarter_turns`, `flip`, `straighten` ranges), §Revisions ("the
editor shows the old rect mapped proportionally for confirmation with Return"; the notice "Changed in Capture One: straighten here instead" when a
geometry value moved; "publish confirmation and export refuse … (naming them)") ·
evidence `design_handoff.md` §Keyboard (⌘[ is Back — `[`/`]` without ⌘ are free), §Tokens (motion tokens `--dur-med`, `--ease-out`) · `06`
§Order (display sequence for `[ ]`) · master plan §6.7 (`composition_reset`), §6.9, §6.10 · process `flow.md` §4, §6, §9 · plans
`P31_compose_mode.md` (the machine's `SetValues`/`Nudge` rules 14–16; `useCompositionSaves`; `viewTransform`; the `Done` button moves into the
panel — the stage-bar copy may stay), `P28_composition_foundation.md` §The geometry contract (ratios, `insideAfterStraighten`, the validation the
backend applies to every save — the frontend's re-fit must produce rects that pass it) · code (P31 files): `features/compose/{geometry.ts,
machine.ts, useComposeGesture.ts, useCompositionSaves.ts, ComposeMode.tsx, ComposeStage.tsx, ComposeFrame.tsx, compose.css}`, `features/workspace/
{WorkspacePage.tsx, order.ts (displaySequence for [ ]), useWorkspaceState.ts}`, `lib/keyboard.ts` (`isTyping`; the `useBackKeys` ⌘[ handler must
not fire for a bare `[`: it already requires `metaKey`), `services/compositions.ts` (`resetComposition`), `services/export.ts` and the Header's
export action (`Header.tsx :useExportToasts`, the `ActionButton onError`) and `features/publish/PublishPopover.tsx` (where a `needs_review`
`ServiceError` surfaces) · tests to copy: P31's harness and `p31_gestures.test.tsx`.

## Files expected to change
Frontend (new): `apps/desktop/src/features/compose/{ToolsPanel.tsx, CompositionSection.tsx, OutputPreview.tsx, Handles.tsx, refit.ts,
useToolsPanelState.ts, useComposeKeys.ts}`.
Frontend (changed): `apps/desktop/src/features/compose/{geometry.ts (trim), machine.ts (SetValues/Nudge/Trim rules), useComposeGesture.ts,
ComposeMode.tsx, ComposeStage.tsx, ComposeFrame.tsx, compose.css}`, `apps/desktop/src/features/workspace/WorkspacePage.tsx` (the left column),
`apps/desktop/src/features/shell/Header.tsx` and `apps/desktop/src/features/publish/PublishPopover.tsx` (the `needs_review` message as a toast),
`apps/desktop/src/features/workspace/workspace.css` (three-column layout with the panel).
Tests: `apps/desktop/src/features/compose/__tests__/{refit.test.ts, p32_tools_panel.test.tsx, p32_controls.test.tsx, p32_keys.test.tsx,
p32_reframe.test.tsx}` (new); `machine.test.ts` gains the rows of C1(b). No Rust change.

## Core mechanisms (unit tests allowed)
- `features/compose/refit.ts` — `refit(values, change, frame) → CompositionValues` (centre- and area-preserving re-fit for shape, orientation,
  turns, flip, straighten changes; `trim`), proven by `refit.test.ts` (C1(a)).
- `features/compose/machine.ts` — the new events (`SetValues`, `Nudge`, `TrimStart/TrimMove/TrimEnd`), proven in `machine.test.ts` (C1(b)).

## The re-fit contract (copied into `refit.ts`)
Inputs: current `values` (with `rect`), the `frame`, one change. Output: new values whose rect passes P28's `validate`.
- `centre = (x + w/2, y + h/2)`, `area = (w·W_t)·(h·H_t)` in pixels of the turned frame **before** the change.
- **Shape or orientation change** (new ratio `r`, `r = None` for free): free → keep the rect; `original` → `(0, 0, 1, 1)`; else the rect of ratio
  `r` with the same area and centre: `w_px = sqrt(area·r)`, `h_px = w_px / r`, then `fit`.
- **Quarter turn** (rotate right = `turns + 1 mod 4`, left = `turns − 1`): the frame turns; the centre follows the content (right: `(cx, cy) →
  (1 − cy, cx)`; left: `(cx, cy) → (cy, 1 − cx)`); `w_px, h_px` from the shape ratio and area as above (a 1:1 shape keeps its size exactly), then `fit`.
- **Flip**: `x → 1 − x − w`, nothing else.
- **Straighten** `θ`: keep the rect; if `insideAfterStraighten` fails, shrink about the centre by bisection on a scale factor in `[MIN, 1]` (24
  steps) to the largest size that is inside; `θ` is clamped to `[−45, 45]` and rounded to 0.1°.
- **`fit`**: clamp the size so `w ≤ 1`, `h ≤ 1` and `min side ≥ MIN_RECT_FRACTION · min(W_t, H_t)` keeping the ratio; place at the centre; shift
  into `[0, 1]`; with `θ ≠ 0` shrink about the centre until `insideAfterStraighten` holds (same bisection).
- **Trim** (handles, during the gesture): `TrimMove { handle, point }` on `startRect`: corner handles (`nw, ne, sw, se`) move that corner; with a
  locked ratio the opposite corner stays fixed and the size follows the pointer's projection onto the diagonal; free → both edges move. Edge
  handles (`n, e, s, w`): locked → scale about the opposite edge's centre; free → move that edge only. Then clamp to the frame, the minimum and
  `insideAfterStraighten`. **On release** the frame "glides back": the stage's frame box is recomputed from the new rect (it always fills the
  stage), so the view re-scales — a CSS transition of `--dur-med` on the image transform and overlay (`data-gliding` while it runs).

## Ordered tasks
0. **Registry first (master plan §6.10).** Machine events `SetValues { values, cause }`, `Nudge { dx, dy }`, `TrimStart { handle, pointerId,
   point }`, `TrimMove`, `TrimEnd`, phase `trimming { handle, pointerId, startRect }`; effects unchanged. Keys: `useComposeKeys` (`ArrowLeft/Right/
   Up/Down` ±1 px, with Shift ±10; `.`; `1`–`6`, `0`; `x`; `[`, `]`; `⌘⌥r`; `Escape`/`Enter` stay in P31's binding; `⌘t` toggles the panel; every
   other key while composing is swallowed — `isTyping` targets excepted). Panel: `aside.tools-panel`, sections `.tools-section[data-open]`
   with a header button, `localStorage` keys `tools.hidden` (`'1'`) and `tools.sections` (JSON `{ composition: boolean }`). Composition section
   controls (labels are the contract): segmented `Normal` | `Split` (Split `aria-disabled` until P33), `Shape` (`<select id="compose-shape">`
   options `original → Original`, `1:1 → Square (1:1)`, `4:5`, `3:2`, `2:3`, `16:9`, `free → Free`), buttons `Turn frame` (X), `Rotate left`,
   `Rotate right`, `Flip`, `Straighten` (`<input type="range" id="compose-straighten" min=-45 max=45 step=0.1>` + a readout `n.n°`, double-click
   → 0), `Centre`, `Reset`, `Done`; `.output-preview` (the composed result at the rect's ratio). Handles: `.compose-handle[data-handle="nw|n|ne|e|
   se|s|sw|w"]`. Banner: `.compose-review` with text `Changed in Capture One: straighten here instead` when `composition.review_reason === 'geometry'`, else
   `Capture One changed this photo's frame — check the framing and press Return`.
1. **Re-fit** (`refit.ts`) per the contract; `trim` in `geometry.ts` or `refit.ts` (Delegated).
2. **Machine**: rules for `SetValues` (idle or wheeling → commit the wheel first; apply; `live = values = new`; `View`; `Save`), `Nudge` (same, `panBy`
   of ±1/(W_t·s) per px, clamped), `TrimStart/Move/End` (like dragging, rule 4 interruptions apply, Esc restores `startRect`), ignored while a
   pointer or pinch gesture is active (rule 14).
3. **Tools panel** (`ToolsPanel.tsx`, `useToolsPanelState.ts`): left column of the workspace **only while composing** (the normal workspace is
   unchanged); sections fold on header click and remember; ⌘T hides/shows the whole panel (remembered); Composition is the first and only
   section now; the panel is built as a list of `{ id, title, content }` so colour/light tools can be appended.
4. **Composition section** (`CompositionSection.tsx`): every control dispatches `SetValues` with `refit(values, change)`; the straighten range
   dispatches on `input` (live) and the save happens on `change`/pointerup through the machine (a slider drag is one gesture: `SetValues { cause:
   'straighten-live' }` writes the view only, `SetValues { cause: 'straighten-end' }` saves — Delegated: or treat the slider like a wheel gesture
   with the settle timer); `Centre` → the rect centred (`.`); `Reset` → `resetComposition(photoId)` then `Saved` with its result (the backend's
   default); `Done` → `KeyReturn`. `OutputPreview.tsx`: a box of the rect's ratio (max 220 px wide) showing the preview clipped to the rect
   (CSS `object-fit`/`transform` on the same image, no extra network).
5. **Handles** (`Handles.tsx` in the overlay): eight handles; pointer events go to the machine as `Trim*`; during a trim the overlay draws the
   trimmed rect inside the frame box (the image does not move); on `TrimEnd` the view re-fits (`viewTransform` with the new rect) with the glide
   transition.
6. **Keys** (`useComposeKeys.ts`) per task 0; `[`/`]` → `SwitchPhoto` to the previous/next photo of `displaySequence` (wrapping at the ends is
   **not** done: at the last photo `]` does nothing); ⌘⌥R → `Reset`.
7. **Re-frame** (`p32_reframe`): when the active photo's `composition.state === 'needs_review'` the editor opens with `entry = values = refit(stored
   values, { kind: 'reframe' }, frame)` (the stored fractions re-fitted to the composition's **current** frame — P28 stores the observed frame) and
   the banner; Return saves (`setComposition`, which clears the flag server-side) even when the values equal the stored ones? **No**: the stored
   values are off-shape for the new frame by construction when the ratio changed, so the re-fit differs and the save happens; when the fractions
   happen to still fit, Return sends the save anyway (`cause: 'confirm'` forces one `Save`) so the flag clears. Export and Publish surfaces: a
   `ServiceError` with `code === 'needs_review'` shows `toast.error(error.message)` (the backend's `Re-frame first: Main 02` text) — the Header's
   export `onError` and the publish popover's confirm error path.
8. Tests, `npm run typecheck`, `npm test`.

## Criteria (8 criteria, 25 rows; 4 `[rule]` rows)
Fixture: P31's (`1000 × 700` stage, 2560×1707 frame, default square). `values(x, y, w, h, extra)` builds `CompositionValues`.

| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | `refit.test.ts` | (1) shape `1:1 → 4:5` on the default square; (2) `1:1 → original`; (3) `1:1 → free`; (4) orientation `named → turned` on `4:5` from (1); (5) rotate right on the default square; (6) rotate right on a 4:5 rect `(0.233, 0, 0.533, 1)`; (7) flip on `(0.1, 0, 0.5, 0.75)`; (8) straighten 10 on the default square; (9) straighten 10 on the half-size centred square `(0.3333, 0.25, 0.3334, 0.5)`; (10) trim `se` corner by `(−100, −100)` px on the default square, locked; (11) trim `e` edge by `−100` px, free shape | (1) centre `(0.5, 0.5)`, area `1707² px`; `w_px = sqrt(1707²·0.8) = 1526.8`, `h_px = 1908.5 > 1707` → fit: `h = 1`, `w = 1365.6/2560 = 0.5334375`, `x = 0.23328125` (= P28 case 3); (2) `(0, 0, 1, 1)`; (3) rect unchanged; (4) ratio 1.25: `w_px = sqrt(1365.6·1707·1.25) = 1706.9`, `h_px = 1365.5` → fits: `w = 0.666760`, `h = 0.799941`, centred `x = 0.166620`, `y = 0.100030` (± 1e-5); (5) turned frame 1707×2560, centre `(1 − 0.5, 0.5) = (0.5, 0.5)`, 1:1 area kept: `w = 1`, `h = 1707/2560 = 0.666796875`, `y = 0.166601562`; (6) centre `(0.5, 0.5)` → `(0.5, 0.5)`; ratio 0.8, area `1365.6·1707`; `w_px = 1365.6`, `h_px = 1707` → in the 1707×2560 frame `w = 0.8`, `h = 0.666797`, `x = 0.1`, `y = 0.166602`; (7) `x = 1 − 0.1 − 0.5 = 0.4`; (8) shrunk about the centre to the largest inside rect: `h = 0.816 ± 0.005` (bisection; `w = h·1707/2560`), centre `(0.5, 0.5)`; (9) unchanged (already inside); (10) `se` corner: the `nw` corner fixed at `(0.166602, 0)`, size follows the pointer's diagonal projection: `−100 px` on both axes at `s = 0.381956` → `−261.8 px` of frame → `h = (1707 − 261.8)/1707 = 0.84664`, `w = 0.84664·1707/2560 = 0.56454`; (11) `e` edge −100 px → `w = (1707 − 261.8)/2560 = 0.56454`, `h = 1`, `x` unchanged | 10D §What the photographer sees (Shape menu, X, rotate, flip, straighten, handles with a locked shape); MC-27 §Validation (every result passes `validate`) |
| C1(b) | `machine.test.ts` | `SetValues` while `idle` → `Save`; `SetValues` while `wheeling` → two effects (`Save` of the wheel, `Save` of the set); `SetValues` while `dragging` → ignored; `Nudge{1, 0}` → `x −= 1/977.81`; `TrimStart/Move/End` → one `Save`; `TrimStart`, `TrimMove`, `KeyEsc` → `live == startRect`, no `Save`; `TrimStart`, `Blur` → `Save` | as listed | 10D (controls and keys act on the composition; one gesture at a time) |
| C2(a) | composing | the panel | `aside.tools-panel` present with a first `.tools-section` titled `Composition` and `data-open="true"`; click its header → `data-open="false"` and `localStorage['tools.sections']` contains `"composition":false`; unmount/remount the workspace (navigate away and back) → still closed; press ⌘T → the panel is hidden (`[hidden]` or absent) and `localStorage['tools.hidden'] === '1'`; ⌘T again → shown | `[rule]` 10D ("sections that fold open and closed … remembered across launches and hidden with ⌘T. Composition is its first section") |
| C2(b) | not composing | the normal workspace | no `aside.tools-panel`; the photo panel on the right as before (P31 C2(a) still holds) | `[rule]` 10D (the panel belongs to the editor) |
| C3(a) | composing | choose `4:5` in `#compose-shape` | one `setComposition` with `shape '4:5'`, `orientation 'named'`, rect `(0.23328125, 0, 0.5334375, 1)`; the overlay frame box becomes `4:5` (`522 × 652` at `(239, 24)`: the largest 0.8 box in `952 × 652`) | 10D (Shape menu); re-fit (1) |
| C3(b) | after (a) | click `Turn frame` (and separately press `x`) | `setComposition` with `orientation 'turned'`, `shape '4:5'`, rect `(0.166620, 0.100030, 0.666760, 0.799941)` ± 1e-5 (re-fit (4)); the frame box becomes `5:4` | 10D ("X turns the frame"); re-fit (4) |
| C3(c) | composing on the default square | click `Rotate right`; separately `Rotate left`; separately `Flip` on the rect `(0.1, 0, 0.5, 0.75)` (a free shape) | right: `quarter_turns 1`, rect `(0, 0.166601562, 1, 0.666796875)`; left: `quarter_turns 3`, same rect (a 1:1 shape re-fits identically); flip: `flip 'horizontal'`, `x = 0.4`; the `.compose-image`'s inner transform carries `rotate(90deg)` / `rotate(270deg)` / `scaleX(-1)` | 10D (rotate 90° left/right, flip); MC-27 §Values (`quarter_turns` 0–3 clockwise, `flip` horizontal) |
| C3(d) | composing | set `#compose-straighten` to `10` (fire `input`, then `change`); then double-click the control | straighten | during `input` the `.compose-image` transform ends in `rotate(10deg)` and no save yet; on `change` one `setComposition` with `straighten 10` and the rect shrunk to `h = 0.816 ± 0.005` centred (re-fit (8)); the readout shows `10.0°`; double-click → `straighten 0` saved, rect back to the largest inside (the default square, since the area is re-grown? **no**: area is preserved, so the rect stays at `h ≈ 0.816`; only the tilt goes) | 10D ("a straighten control"); MC-27 §Values (−45…45) |
| C3(e) | composing with the stage image loaded (a 2560×1707 `src`) | the output preview | `.output-preview` has `style.aspectRatio == "1707 / 1707"` (the rect's pixel ratio) for the default square and `"1365.6 / 1707"` after C3(a)'s 4:5; it contains an `img` whose `src` is the same adjusted preview URL (no second network resource) | `[rule]` 10D ("the output previews at their real proportions") |
| C4(a) | composing | `pointerdown` on `.compose-handle[data-handle="se"]` at the frame's bottom-right `(826, 676)`, `pointermove` to `(726, 576)`, `pointerup` | a locked corner trim | during the move the overlay's frame rect shrinks to `552 × 552` from the `nw` corner (the image transform unchanged); on release one `setComposition` with rect `(0.166602, 0, 0.56454, 0.84664)` ± 1e-5 and the stage re-fits: the frame box returns to `652 × 652` and `.compose-stage` carries `data-gliding="true"` until `transitionend` (fire it) | 10D ("corner and edge handles trim the frame (a locked shape keeps its ratio) and on release the frame glides back to the centre"); re-fit (10) |
| C4(b) | composing on a `free` shape rect `(0.166602, 0, 0.666797, 1)` | drag `.compose-handle[data-handle="e"]` by `−100` px | one `setComposition` with `w = 0.56454`, `h = 1`, `x` unchanged (free: one edge moves) | 10D (handles; free shape) |
| C4(c) | composing | `pointerdown` on the `se` handle, `pointermove`, press `Escape`, `pointerup` | Esc mid-trim | the overlay frame returns to `652 × 652`, no `setComposition` | interaction rule 5 extended to trims |
| C5(a) | composing | press `ArrowRight`; `Shift+ArrowRight`; `ArrowDown` (h = 1: no room) | nudges | three saves? **No** — each key is one `SetValues`: `ArrowRight` → `x += 1/977.81 = 0.0010227` (the photo moves right one screen pixel means the rect moves… the rect moves with the arrow: `x` **increases** by one px of frame — Delegated direction? **Fixed:** arrows move the **frame over the photo** in the arrow's direction, i.e. `x += dx/(W_t·s)`, `y += dy/(H_t·s)`); `Shift` → `+0.010227`; `ArrowDown` → `y` stays `0` (clamped); each a separate `setComposition` | 10D ("arrows nudge one screen pixel (Shift ×10)") |
| C5(b) | composing after a drag (`x = 0.115466`) | press `.` ; separately click `Centre` | one `setComposition` with `x = 0.166601562` (centred), `y = 0` | 10D ("`.` centres"; Centre button) |
| C5(c) | composing | press `1`, `2`, `3`, `4`, `5`, `6`, `0` in turn | saves with `shape` `original`, `1:1`, `4:5`, `3:2`, `2:3`, `16:9`, `free` in that order (each re-fitted from the previous) | 10D ("1–6 and 0 pick Original, 1:1, 4:5, 3:2, 2:3, 16:9, Free") |
| C5(d) | composing on photo 1 of `[1, 2, 5]` (display sequence) after a drag not yet released… **no**: after a drag released (save pending `p1`) | press `]`; then `]` again; then `[` three times | the editor shows photo 2, then 5, then 2, 1, and stays on 1 (no wrap); photo 1's pending save resolves and updates tile 1 only (P31 C7(a)); `getSessionView` not called | 10D ("[ ] previous/next photo keeping changes") |
| C5(e) | composing after a drag | press ⌘⌥R; separately click `Reset` | `resetComposition(1)` called once; the returned composition (the default) becomes the view (`translate(11.09px, 24px)`) | 10D ("⌘⌥R resets to the default"); P28 `composition_reset` |
| C6(a) | composing | press `a`, `c`, `Backspace`, ` `, `f`, `i`, `d`, `n`, `⌘[` | swallowed keys | no workspace or app action (`classifyPhoto`/`deletePhoto` not called, no dialog, `window.location.hash` unchanged); `.compose-stage` still present | 10D ("All workspace keys … are off while composing") |
| C6(b) | composing, focus inside the straighten `input[type=range]`; press `ArrowRight` | a key inside a control | the range changes by its step (the browser's own behaviour) and **no** nudge save is sent (`isTyping`-style guard for form controls) | design handoff §Keyboard (presses inside fields belong to the field) |
| C7(a) | photo 1 `state: 'needs_review'`, `review_reason: 'frame'`, `frame 1707×2560` (P28: the observed frame — Capture One turned the variant), stored rect `(0.166601562, 0, 0.666796875, 1)`; press `r` | opening on a photo whose frame changed | `.compose-review` banner with the text `Capture One changed this photo's frame — check the framing and press Return`; the view shows the re-fitted rect `(0, 0.166601562, 1, 0.666796875)` ± 1e-6 (the fractions re-fitted to the portrait frame: the largest square at the same centre); no save yet | MC-27 §Revisions ("the editor shows the old rect mapped proportionally for confirmation with Return"); M33 |
| C7(b) | after (a) | press `Enter` | one `setComposition(1, values with rect (0, 0.166601562, 1, 0.666796875))`; its answer `state 'ok'`, `review_reason null` removes the banner and the tile's `Re-frame` corner | MC-27 §Revisions ("confirmation with Return") |
| C7(c) | photo 1 `state: 'needs_review'`, `review_reason: 'geometry'`, frame 2560×1707 (unchanged size: a Capture One straighten), stored rect = the default square; press `r`; then `Enter` | a geometry change | the banner reads `Changed in Capture One: straighten here instead`; the view shows the stored rect unchanged (it still fits); `Enter` still sends exactly one `setComposition` with the stored values (`cause: 'confirm'`) and the `ok` answer clears the banner | MC-27 §Revisions (the notice when a geometry value moved; confirmation with Return) |
| C7(d) | after (a) | drag +50 and release | the save carries the re-fitted rect moved by the drag (`y = 0.166601562`, `x = 0` clamped — `w = 1` leaves no horizontal room; use a vertical drag of −50 instead: `y = 0.166601562 − 50/652 = 0.089916`); the banner disappears on the `ok` answer (any save confirms) | MC-27 §Revisions |
| C8(a) | the Header's Export button; `runExport` rejects with `ServiceError { code: 'needs_review', message: 'Re-frame first: Main 02' }` | a refused export | `toast.error` called with a text containing `Re-frame first: Main 02` | `[rule]` MC-27 §Revisions ("export refuse while … naming them") |
| C8(b) | the publish popover's confirm; `publishConfirm` rejects the same way | a refused publish | the same toast; the popover stays open | MC-27 §Revisions ("publish confirmation … refuse") |

## Delegated choices
Whether the slider is a native `input[type=range]` (default) or a custom drag; whether straighten-live goes through `SetValues{cause}` or a
wheel-like settle; the glide duration (`--dur-med` 280 ms default) and easing; the handle hit size (12 px suggested); the panel width (260 px
suggested) and the section header markup; the output preview's size cap; the exact banner wording beyond the fixed sentence; whether `Done`
stays in the stage bar as well as the panel; how `trim` splits between `geometry.ts` and `refit.ts`.

## Manual try-out
1. Open the editor (R). A Tools panel sits on the left with "Composition" open. Click its title: it folds; relaunch the app: still folded. ⌘T hides
   the whole panel; ⌘T again brings it back.
2. Pick 4:5 in Shape: the frame becomes upright 4:5 around the same centre; press X: it turns to 5:4; press 2 to go back to square. Rotate right:
   the photo turns, the frame stays; Flip mirrors it. Drag the straighten slider to about 5°: the photo tilts under the frame and the frame
   shrinks just enough to avoid empty corners; double-click the slider to level it.
3. Drag a corner handle inward: the frame shrinks while you hold; let go: it glides back to fill the stage, now showing a tighter crop. Press
   `.` to centre, arrows to nudge by a pixel (Shift for ten), ⌘⌥R to reset.
4. Press `]` and `[` to move through the photos without leaving the editor; your changes stay (watch the tiles via the edge reveal).
5. On a photo you straightened in Capture One (P28 step 3), open the editor: a banner says "Changed in Capture One: straighten here instead";
   press Return to confirm (and straighten here if you want); the amber "Re-frame" corner on its tile disappears and Export/Publish stop refusing.
   A photo rotated 90° in Capture One shows the other banner and its framing fitted to the turned frame.

## Notes
- **Area-preserving re-fit** keeps the photographer's zoom level across shape, turn and rotation changes (a shape change is not a reset);
  where the frame cannot hold that area at the new ratio, the rect shrinks to the largest that fits. Every output passes the backend's
  `validate` by construction (the frontend never sends a rect the backend would refuse, except through stale frames, which the Re-frame flow
  handles).
- **Arrows move the frame, not the photo** (C5(a)): with the frame fixed on screen the two are the same motion in opposite directions; the plan
  fixes "the frame moves in the arrow's direction" because it is the Capture One / Photoshop convention for a crop box.
- **`[`/`]` do not wrap**; the stage's ‹ › buttons keep their P08 behaviour outside the editor.
- **Why the straighten save happens on `change`**: a slider drag is one gesture; saving per `input` event would flood the queue (latest-wins
  would cope, but the backend would still receive dozens of saves).
- Registry additions made by the planner in master plan §6.1, §6.9, §6.10. No intention-book change, no backend change.
- Estimated size: ~16 files, ~2 000 lines (≈ 400 re-fit + machine, ≈ 650 panel/controls/handles/keys, ≈ 950 tests).

## Review log
### 2026-11-05 — implement r1 (Fable, `pipeline-implementer`)
**Built.** The second half of 10D's editor for Normal compositions. New: `features/compose/refit.ts` (`refit(values, change, frame)` for
`shape`, `orientation`, `turn`, `flip`, `straighten`, `centre`, `reframe` per §The re-fit contract — centre and area preserved, `fit` with the
24-step bisection, straighten clamped ±45 and rounded to 0.1°; `trim(startRect, handle, deltaPx, { turned, scale, ratio })` — locked corners
follow the pointer's projection onto the diagonal from the opposite corner, locked edges scale about the opposite edge's centre, free handles move
their edges, clamped to the frame and the minimum); `ToolsPanel.tsx` (a list of `{ id, title, content }` sections, `aside.tools-panel`,
`.tools-section[data-open]` with a header button); `useToolsPanelState.ts` (`tools.hidden` / `tools.sections` in `localStorage`);
`CompositionSection.tsx` (Normal | Split with Split `aria-disabled`, `#compose-shape`, Turn frame, Rotate left/right, Flip, `#compose-straighten`
+ `n.n°` readout with double-click → 0, Centre, Reset, Done, the output preview); `OutputPreview.tsx` (`.output-preview` of the rect's pixel ratio,
the same adjusted preview clipped by CSS); `Handles.tsx` (eight `.compose-handle[data-handle]` rects in the overlay); `useComposeKeys.ts` (arrows
± 1 / Shift × 10, `.`, 1–6 / 0, X, `[` `]`, ⌘⌥R, ⌘T, every other key swallowed; a `window` **capture** listener so the app's own maps — ⌘[ Back among
them — never see a swallowed key). Changed: `machine.ts` (phases `trimming { handle, pointerId, startRect, origin }` and `adjusting { startValues }`;
events `SetValues { values, cause: set | live | confirm }`, `TrimStart/TrimMove/TrimEnd`; `needsConfirm` in the state; entry re-fit for `needs_review`
photos; `KeyReturn` saves once to confirm), `useComposeGesture.ts` (handle pointer-down → `TrimStart`; `subscribe`/`getSnapshot` for React; the glide:
`data-gliding` on the stage until `transitionend`, 400 ms fallback), `ComposeStage.tsx` (renders from the machine's snapshot: trimmed box during a
trim, handles, `.compose-review` banner), `ComposeFrame.tsx` (children), `ComposeMode.tsx` (owns the Tools panel and the stage column; Reset →
`resetComposition` → `Saved`; `[`/`]` through `displaySequence`), `compose.css`, `workspace/WorkspacePage.tsx` (the editor replaces the stage column
and adds the left panel; `stageExtras` shared), `workspace.css` (`.tools-panel` 260 px), `shell/Header.tsx` and `publish/PublishPopover.tsx` (a
`needs_review` refusal shows the backend's message bare). No Rust, no bindings change.
**Tests.** `refit.test.ts` (C1(a): the 11 cases + clamp/round), `machine.test.ts` (+10 rows for C1(b)), `p32_tools_panel.test.tsx` (C2(a)–(b)),
`p32_controls.test.tsx` (C3(a)–(e), C4(a)–(c)), `p32_keys.test.tsx` (C5(a)–(e), C6(a)–(b)), `p32_reframe.test.tsx` (C7(a)–(d), C8(a)–(b)).
45 new tests; `npm test` 31 files / 268 passed; `npm run typecheck` clean; Rust suite, clippy and fmt unchanged and green (see the handoff stamp).
**Judgment calls (Assumptions).**
1. **`Nudge` keeps P31's pointer convention** (`Nudge{dx: 1}` → `x −= 1/(W_t·s)`, as C1(b) states) and the key layer negates it: ArrowRight → `Nudge{dx: −1}`
   so the frame moves right over the photo (`x += 1/977.81`, C5(a)). The two rows are both satisfied; the machine stays the gesture-level truth.
2. **ArrowDown with `h = 1` sends no save** (C5(a) "y stays 0 (clamped)"): the clamped rect equals the committed one and the machine saves only on a
   change (P31 rule 3 / assumption 15). The row's "each a separate `setComposition`" is read as "the arrows are not batched"; the test asserts 2 saves for
   the three presses and `y = 0` on every call.
3. **Straighten-live is a machine phase** (`adjusting { startValues }`, Delegated choice): React `onChange` (the native `input`) dispatches
   `SetValues{cause:'live'}` (view only), a native `change` listener dispatches `SetValues{cause:'set'}` (one save), double-click → straighten 0.
   Esc mid-slide restores the start values; Blur / a pointer down / a wheel commit it first, like a wheel gesture.
4. **Re-fit on a `free` rect during a quarter turn turns the rect with the content** (`(x, y, w, h) → (1 − y − h, x, h, w)` for right) — the contract's
   "centre follows the content" with the area preserved; `original` → `(0, 0, 1, 1)`. **Flip** also runs the straighten guard (identical result while
   `θ = 0`; with a tilt the mirrored rect may leave the rotated image and shrinks about its centre) — the contract's "nothing else" would otherwise send a
   rect the backend refuses. When even the minimum size at the rect's centre is outside the tilted image, the centre moves to the frame's centre before
   the bisection (a corner-hugging sliver at 45°).
5. **`TrimMove` carries `{ pointerId, point }`** (the handle lives in the phase); `PointerMove`/`PointerUp`/`PointerCancel`/`LostCapture` of the trimming
   pointer are treated as `TrimMove`/`TrimEnd`, so the P31 window listeners need no branching. A handle's pointer down is recognised by the stage listener
   (`closest('.compose-handle')`) because the native stage listener fires before React's synthetic handlers on the handle.
6. **The glide** is detected in the binding (phase `trimming → idle` with `live ≠ startRect`): `data-gliding="true"` on `.compose-stage`, cleared on the
   image's `transitionend` or after 400 ms; CSS transitions `--dur-med --ease-out` on the image transform/size and the overlay geometry.
7. **Controls render from the machine's published snapshot** (`useSyncExternalStore`; published after every event except the moves of a continuous
   drag, pinch or wheel), so the frame box, handles, output preview and inner turn/flip follow the editor and not the stored row; the image transform
   stays imperative (P31 assumption 9).
8. **Reset** calls `resetComposition` directly, patches the workspace row and hands the answer to the machine as `Saved` (rule 16 snaps the view); `entry`
   is left alone so Esc still reverts to how the editor came to the photo. A failure shows `Not reset: <message>`.
9. **Done stays in the stage bar** as well as the panel (the panel can be hidden with ⌘T); the panel copy carries `.compose-done.tools-done`.
10. **Keys**: ⌘ combinations other than ⌘T, ⌘⌥R and ⌘[ are left to the app/webview (swallowing ⌘Q or ⌘C in the webview would change nothing useful);
    ⌥ alone is left for P34's snap suspension; Tab keeps moving the focus (the panel's controls stay reachable); `event.code === 'KeyR'` also
    matches ⌘⌥R (Option changes `key` on macOS).
11. **Re-frame**: the banner reads `photo.composition.state` from the workspace row, so it disappears when the saved row (`state ok`) is patched in; the
    `needs_review` toasts check `error.code` duck-typed (p15's mock of `@/services/sessions` has no `ServiceError`).
12. **`[`/`]` on a photo whose save is still in flight** show the stored row until the answer lands (then rule 16 snaps to it) — the same semantics as
    P31 C7(a); C5(d) asserts the placement after `p1` resolves.
13. A side within 1e-12 of the whole frame is snapped to exactly 1 in the re-fit so `x`/`y` clamp to exactly 0 (`1707²/2560` is not a dyadic rational).
**Plan arithmetic noted (not adapted around; the contract values are asserted).** Re-fit (4) / C3(b): `sqrt(1365.6·1707·1.25) = 1707` exactly, so
`w = 0.666796875`, `h = 0.8`, `x = 0.1666015625`, `y = 0.1` (the plan's `0.666760 / 0.799941 / 0.166620 / 0.100030` carry a rounding slip beyond the
± 1e-5). Re-fit (8) / C3(d): the largest centred square inside the 10° tilted frame is `h = 1/(cos 10° + sin 10°) = 0.8632`, not `0.816` (bisection
confirms). Re-fit (10) / C4(a): `h = 1 − 100/652 = 0.846626`, `w = 0.564526` (plan `0.84664 / 0.56454`). C1(b) "TrimStart, Blur → Save": without a
`TrimMove` nothing changed and nothing is saved; the row is tested with a move in between. C7(d): under the machine (rule 2, unchanged by the plan)
a pointer moved **up** 50 px moves the photo up, so the frame sits lower: `y = 0.166601562 + 50/977.81 = 0.217737` (the plan's `− 50/652` divides by
the wrong side and has the wrong sign); `x = 0` clamped as the row says.
**Deviations.** (a) `ComposeFrame.tsx` changed (children slot for the handles; listed as changed in the plan — no deviation) and `trim` lives in
`refit.ts` (Delegated). (b) `useComposeGesture.ts` exports `LAYER_OPEN` for `useComposeKeys`. (c) Tracker actor is `fable`, not the prompt
template's `codex`. (d) No new files beyond the plan's list.
**Not done.** No live try-out in the Tauri window from this session (no interactive display); the owner's Manual try-out stands. Rust untouched.

### 2026-11-05 — review r1 (Opus, `pipeline-reviewer`): APPROVED_WITH_NOTES
**Gate.** Tracker `IMPLEMENTED`; checkpoint `c0cc169` in `git log`. Reviewed on clean `59cb210` (`c0cc169..59cb210` adds only the implement
handoff and the review prompt).
**Perimeter.** `git show --stat c0cc169` = the handoff's 25 paths exactly (7 new + 5 new tests, 11 changed, tracker row, Review log); no file
outside the plan's list; no Rust, no bindings.
**Suite (run once).** `cargo test --workspace` 529 passed / 0 failed; clippy `-D warnings` clean; `cargo fmt --check` exit 0; `npm run typecheck`
exit 0; `npm test` 31 files / 268 passed. No B4.
**Verified correct (code read, then its test).**
- C1(a) `refit.ts`: shape/orientation/turn/flip/straighten/centre/reframe per the contract; the 11 cases assert exact rects and `validate`. The
  plan's literals that disagree are plan slips, re-derived independently: (4) `1365.6·1.25 = 1707` so `w_px = 1707`, `h = 0.8`; (8) a centred square
  inside a 10° tilted 2560×1707 needs `s(cos θ + sin θ) ≤ 1707` → `h = 0.8632` (the test also shows a slightly larger rect is `rect_outside`); (10)
  `1 − (100/0.381956)/1707 = 1 − 100/652 = 0.846626`. Quarter-turn maps (`(cx, cy) → (1 − cy, cx)`, rect `(1 − y − h, x, h, w)`) are the clockwise
  rotation MC-27 defines; flip `x → 1 − x − w` keeps the same content.
- C1(b) machine: `SetValues` idle/wheeling/dragging, `Nudge` sign (P31 pointer convention; the key layer negates it), `Trim*` with the view frozen
  during the trim and re-fitted on end, Esc restores `startRect`, Blur commits; `adjusting` for the live slider; `needsConfirm` cleared by any `Save`,
  forced once by Return. React's value tracker drops the `onChange` that would follow the native `change` with the same value, so the slider ends
  `idle` (Esc after a slide still reverts to the entry).
- C2 panel: `aside.tools-panel` only in the compose branch of `WorkspacePage`; fold and hide persisted in `tools.sections` / `tools.hidden`; tests
  remount through the hash. C3(a)–(e), C4(a)–(c): each asserts the `setComposition` payload, the overlay frame box and the image transform; C4(a)
  checks the 552 × 552 box during the trim, one save, `data-gliding` until `transitionend`.
- C5(a)–(e), C6(a)–(b): `useComposeKeys` is a window **capture** listener, so ⌘[ and every non-editor key are stopped before the app's maps; the
  workspace keyboard (incl. ⌘Z) is already disabled while composing; `isTyping` targets keep their keys (C6(b) asserts `defaultPrevented false`).
  `[` `]` use `displaySequence` with no wrap.
- C7(a)–(d): entry re-fit for `needs_review` (`entryValues`, also on `SwitchPhoto`), banner text from `review_reason`, Return/any save confirms;
  C7(d)'s `y = 0.1666 + 50/977.81` follows P31 rule 2 (the photo follows the pointer; the plan divided by the wrong side). C8(a)–(b): `needs_review`
  refusals show the backend message bare; the popover stays open (same node).
**Assumptions.** None contradicts 10D/MC-27. Notable: ArrowDown with no room sends nothing (P31 "save only on change"); Flip also runs the straighten
guard (keeps the "never past the photo's edge" rule); Done stays in the stage bar too; ⌘ combinations other than ⌘T/⌘⌥R/⌘[ are left to the app.
**Hard constraints.** H1–H8 hold: frontend only, no external write, no pixel processing (CSS transforms only), the backend still validates every save.
**Notes (backlog, non-blocking).** (1) probe-confirmed: a re-fitting control used while a wheel is still settling re-fits from the pre-wheel rect,
discarding the scroll (→ P34); (2) focus left on the Shape select / straighten range turns every editor key, Return included, into a field key (→ P34);
(3) Reset bypasses the save queue, a queued save can land after it (→ P34); (4) the P31 layer-guard note now covers all P32 keys (→ P34); (5) not
verified live: panel layout, glide, output-preview clipping for turned/tilted photos.
**Probe declaration.** One scratch test `apps/desktop/src/features/compose/__tests__/zz_review_probe.test.tsx` created, run, deleted; no production
file touched; `git status` clean afterwards.
**Lessons for the plans.** Hand-computed literals in criterion rows were wrong four times (re-fit (4), (8), (10), C7(d)); rows that state a
contract formula and a derived number should carry the formula as the authority and the number as "≈". C1(b) "TrimStart, Blur → Save" is
unreachable without a move (nothing changed). `SetValues { values }` computed outside the machine cannot honour "commit the wheel first; apply";
P33/P34 rows that add controls should pass the change, not the result.
