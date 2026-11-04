---
phase: P31_compose_mode
title: Compose mode — the editor's stage with the frame fixed and the photo moving under it, drag / pinch / wheel gestures through one explicit state machine, autosave and Esc revert, the right browser hidden with edge reveal, tiles drawing the composition
difficulty: high
projection: no
implementer: fable
depends_on: [P28_composition_foundation]
---
# P31 — Compose mode: the editor's stage with the frame fixed and the photo moving under it, drag / pinch / wheel gestures through one explicit state machine, autosave and Esc revert, the right browser hidden with edge reveal, tiles drawing the composition

## Goal
The first frontend half of 10D's editor (§What the photographer sees; MC-27 as the saved shape). **Compose** (R, double-click on the stage
photo, "Compose…" in the ⋯ and right-click menus) turns the stage into the editor: the frame of the photo's composition stays centred and
fills the stage; the 2560 px adjusted preview moves under it. Drag or two-finger swipe moves the photo; pinch, ⌘ + wheel or ctrl-wheel zooms
the composition around the pointer; the composition can never extend past the photo's edge nor shrink below a minimum. Every gesture runs
through **one explicit, pure state machine** (`machine.ts`: `idle`, `dragging`, `pinching`, `wheeling`), proven on a case table: a second
pointer, a wheel during a drag, an interrupted gesture (pointer cancel, lost capture, window blur, page hidden), Esc mid-gesture, a photo
switch mid-gesture and a save that settles late all have one defined outcome. During a gesture the values live outside React state and the
preview layer is moved by a CSS transform written **once per animation frame**; nothing waits for Rust. Changes save automatically when a
gesture ends and when leaving the photo (latest-wins per photo, ticketed, never applied to another photo); Esc reverts this photo to how it
was when the editor came to it; Return (or Done) finishes. While composing, the workspace keys (A, C, Delete, Space, F, I, D), the stage
swipes, the webview's page zoom and the two-finger back swipe are off. The photo browser on the right hides and slides in **over** the image
when the pointer reaches the window's right edge, hides again when the pointer leaves it, never during a drag; clicking a photo there moves
the editor to it and keeps the changes. Grid tiles show the 640 px thumbnail with the composition drawn over it in CSS, and an amber corner
with "Re-frame" when the composition needs review. The first time the editor opens, one line says "Crops made in Capture One are not used."
The TypeScript geometry mirrors the Rust module and passes **the same case table**. **Not in this phase:** the Tools panel, Shape menu,
turn/rotate/flip/straighten controls, handles, the nudge and shape keys, `[ ]`, ⌘⌥R, Centre/Reset buttons, the needs_review re-frame flow (P32);
Split (P33); loupe, snapping, copy/paste, unsaved dot, reduced motion, window size (P34). The editor opens only on a photo that has an adjusted
preview and a composition rect (the backend materialised one); otherwise Compose is unavailable (menu entry disabled, R does nothing).

## Read first
Book: `10D_composition_and_framing.md` §What the photographer sees (bullets 2, 3 (the panel is P32), 5 (the gesture list: drag/swipe, pinch
and ⌘-wheel, Esc, Return, "all workspace keys … stage swipes, the webview's page zoom and the two-finger back swipe are off"), 6 (autosave),
7 (the first-time line)), §MC-27 Values and Validation (what a save sends; the backend refuses invalid values), §Revisions (the amber corner,
"Re-frame"), §Performance targets (what the architecture must allow; measured in P34) · `06` §Original / Adjusted (the editor shows the adjusted
preview only) · `04` scenarios (S-C the stage) · evidence `design_handoff.md` §Keyboard (the workspace keys that must be off), §Tokens
(`--stage`, `--ai`, `--danger`, `--ok`; no amber token exists — add `--warn` in `tokens.css` per Delegated choices) · `01` (H8: logic independent
of Tauri — the machine and geometry are plain TS) · master plan §6.1 (`features/compose/`), §6.7 (`composition_set`, `PhotoResponse.composition`,
`thumbnail_path`), §6.9 (frontend tests), §6.10 (names), §7.2 · process `flow.md` §4 (unit tests only for the Core mechanisms list: `geometry.ts`,
`machine.ts`), §6, §9 · plan `P28_composition_foundation.md` **§The geometry contract** (the TS mirror implements it verbatim; `composition_cases.json`
is the shared table) and its DTO shapes · code (line numbers at `d763492`; P28 regenerates `bindings.ts`): `apps/desktop/src/features/workspace/
WorkspacePage.tsx` (`review :174`, `onMenu :188–199`, `useKeyboard({ d }) :233`, `useWorkspaceKeyboard(…, !dialogOpen) :234–252`, `<Stage …>
:297–306`, `<FoldedStrip>`/`<PhotoPanel> :320–322`, `DragOverlay :327`), `Stage.tsx` (`stage-body :60`, `<SwipeLayer><StageImage/></SwipeLayer>
:64–66`, `<TogglePill> :72`, `PhotoMenuButton :54`), `StageImage.tsx` (`stage-frame`, `stage-layer`, the crossfade layers; the editor renders
its own image element), `SwipeLayer.tsx` (what is **not** rendered while composing), `useKeyboard.ts` (`useWorkspaceKeyboard` — the `enabled`
flag), `apps/desktop/src/lib/keyboard.ts` (`useKeyboard`, `isTyping`, `useBackKeys :34–50` — Esc must not navigate back while composing: the
compose layer renders a `[role="dialog"]`-equivalent guard or `useBackKeys` is paused; see Delegated), `useWorkspaceState.ts` (`run :127–146`,
`setActiveId :70`, `reload :75`, `step :225`, the `photosRef :122`), `PhotoTile.tsx` (`Thumb :15–31`, `PhotoTileFace :45–59`, `tile-number :52`),
`PhotoGrid.tsx` (`number={index + 1} :42`), `PhotoMenu.tsx` (`PhotoMenuAction :6`, `entries :11–20`), `PhotoPanel.tsx`, `workspace.css`
(`.stage-body :20`, `.swipe-layer :25`, `.stage-frame :27`, `.stage-layer :28`, `.photo-panel :50`, `.photo-tile :64`, `.tile-grid :65`,
`.thumb-image :70`), `apps/desktop/src/services/photos.ts` (`Photo`, `getSessionView`), `apps/desktop/src/services/bindings.ts` (`PhotoResponse_Serialize`
with `thumbnail_path`, `composition`; `commands.compositionSet`, `compositionReset`, `CompositionDto`, `CompositionValuesDto`), `apps/desktop/src/App.tsx`
(`useKeyboard({ n })`, hash routes), `apps/desktop/src/styles/tokens.css` · tests to copy the harness from: `features/workspace/__tests__/
p22_workspace_tidy.test.tsx` (`:10–105` mocks, polyfills (`ResizeObserver`, `PointerEvent`), `photo()`, `openWorkspace`, `layout`, `releaseAt`),
`p20_drop_continuity.test.tsx` (fake `getBoundingClientRect` layouts, rAF handling), `p26_double_drag.test.tsx` (pending promises `p1`/`p2`).

## Files expected to change
Frontend (new): `apps/desktop/src/features/compose/{geometry.ts, machine.ts, useComposeGesture.ts, useCompositionSaves.ts, ComposeMode.tsx,
ComposeStage.tsx, ComposeFrame.tsx, BrowserOverlay.tsx, compose.css}`, `apps/desktop/src/services/compositions.ts`.
Frontend (changed): `apps/desktop/src/features/workspace/{WorkspacePage.tsx, Stage.tsx, PhotoMenu.tsx, PhotoTile.tsx, workspace.css}`,
`apps/desktop/src/styles/tokens.css` (`--warn`), `apps/desktop/src/lib/keyboard.ts` (only if `useBackKeys` needs the pause — Delegated).
Tests: `apps/desktop/src/features/compose/__tests__/{geometry.test.ts, machine.test.ts, p31_compose_mode.test.tsx, p31_gestures.test.tsx,
p31_browser_and_tiles.test.tsx}` (new). No Rust change, no bindings change.

## Core mechanisms (unit tests allowed)
- `features/compose/geometry.ts` — the TS mirror of P28's geometry contract (`turnedFrame`, `shapeRatio`, `defaultRect`, `validate`, `pixelRect`,
  `splitHalves`, `fingerprintCompatible`, `insideAfterStraighten`) plus the view maths (`frameBox`, `viewTransform`, `panBy`, `zoomAbout`, `clampRect`).
  Proven by `geometry.test.ts` over **the shared `composition_cases.json`** (read with `fs` from `crates/core/src/mechanisms/`) and by the hand-computed
  view cases of C1(b).
- `features/compose/machine.ts` — `reduce(state, event) → { state, effects }`, the interaction state machine. Proven by `machine.test.ts` (C1(c), C6).

## The interaction contract (copied into `machine.ts` doc comments)
State: `{ phase, values: CompositionValues (last committed), entry: CompositionValues (when the editor came to this photo), live: Rect (what
is on screen), photoId, frame: Frame, view: ViewGeometry }` with `phase ∈ idle | dragging { pointerId, startRect, origin: Point } | pinching
{ startRect, anchor: Point, viaGestureEvents: boolean } | wheeling { mode: pan | zoom, startRect, lastAt }`.
Events: `PointerDown { pointerId, isPrimary, button, point }`, `PointerMove { pointerId, point }`, `PointerUp { pointerId }`, `PointerCancel
{ pointerId }`, `LostCapture { pointerId }`, `Wheel { dx, dy, ctrlKey, metaKey, point, at }`, `GestureStart { point }`, `GestureChange { scale,
point }`, `GestureEnd`, `Tick { at }`, `KeyEsc`, `KeyReturn`, `SwitchPhoto { photoId, composition }`, `Blur`, `Hidden`, `Nudge { dx, dy }` (P32),
`SetValues { values }` (P32: panel controls), `Saved { photoId, composition }`.
Effects: `View { rect }` (write the transform — the binding coalesces to one write per animation frame), `Save { photoId, values }`, `Exit`,
`Switch { photoId }`, `PreventDefault`.
Rules (each has a row in `machine.test.ts`):
1. A drag starts only on `PointerDown` with `isPrimary && button === 0` while `idle` (or while `wheeling`: rule 10 first). Pointer events of any
   other `pointerId` while `dragging` have no effect.
2. `PointerMove` of the dragging pointer → `live = clampRect(panBy(startRect, (p − origin) / (W_t·s, H_t·s)))` → `View`. The pan direction: the
   photo follows the pointer (`x −= dx / (W_t·s)`, `y −= dy / (H_t·s)`).
3. `PointerUp` of the dragging pointer → `idle`; if `live ≠ values` then `values = live` and `Save`.
4. `PointerCancel`, `LostCapture` (of the dragging pointer), `Blur`, `Hidden` while `dragging` or `pinching` → exactly as rule 3 (the visible
   state is committed; an interrupted gesture never loses what the photographer saw).
5. `KeyEsc` while `dragging`/`pinching`/`wheeling` → `live = startRect`, `View`, `idle`, **no** `Save` (`values` unchanged).
6. `KeyEsc` while `idle` → `live = values = entry` (the photo's composition when the editor came to it), `View`, and `Save` **only if** it differs
   from the last saved values.
7. `Wheel` while `idle`: `ctrlKey || metaKey` → `wheeling { zoom }`, `live = clampRect(zoomAbout(startRect, point, exp(−dy · ZOOM_PER_PX)))`,
   `ZOOM_PER_PX = 0.005`; otherwise → `wheeling { pan }`, `live = clampRect(panBy(startRect, (−dx, −dy) / (W_t·s, H_t·s)))` (natural scrolling: the
   photo follows the fingers). Every `Wheel` yields `PreventDefault` (page zoom and back-swipe off). Subsequent `Wheel`s of the same mode extend
   the gesture (`lastAt = at`); a `Wheel` of the **other** mode first commits (rule 9) and starts the new gesture.
8. `Wheel` or `GestureChange` while `dragging` → `PreventDefault` only (one gesture at a time).
9. `Tick { at }` while `wheeling` and `at − lastAt ≥ WHEEL_SETTLE_MS` (`120`) → commit as rule 3 → `idle`.
10. `PointerDown` while `wheeling` → commit the wheel gesture (rule 3) **then** start the drag (rule 1): two `Save`s when both changed.
11. `GestureStart` while `idle` or `wheeling { zoom }` → `pinching { startRect, anchor: point, viaGestureEvents: true }` (a wheel zoom in flight is
    committed first); `GestureChange { scale }` → `live = clampRect(zoomAbout(startRect, anchor, scale))`, `View`; `GestureEnd` → commit (rule 3).
    While `pinching` with `viaGestureEvents`, `Wheel { ctrlKey }` → `PreventDefault` only (WebKit sends both; the gesture scale wins).
12. `SwitchPhoto` → if a gesture is active commit it first (rule 3 — one `Save` for the old photo), then `values = entry = live = the new photo's
    composition values`, `phase idle`, `Switch`.
13. `KeyReturn` → commit an active gesture (rule 3), then `Exit`.
14. `Nudge`/`SetValues` while a pointer or pinch gesture is active → ignored; while `idle` or `wheeling` → commit the wheel first, then apply
    (details P32).
15. Clamps (`clampRect`): the rect keeps its ratio; `x ∈ [0, 1 − w]`, `y ∈ [0, 1 − h]`; `zoomAbout` keeps the anchor's image point under the pointer
    and clamps the scale so that `w ≤ 1`, `h ≤ 1` (never past the edge) and `min(w·W_t, h·H_t) ≥ MIN_RECT_FRACTION · min(W_t, H_t)`; with
    `straighten ≠ 0` a move or zoom whose result fails `insideAfterStraighten` is **refused** (the previous `live` stays) — the image sticks at a
    tilted edge rather than sliding; smoothing is a P34 candidate.
16. `Saved { photoId, composition }` with `photoId ≠ state.photoId` → no change to the machine (the saves hook updates that photo's stored values);
    with `photoId == state.photoId` while `idle` and the composition's values ≠ `values` (a refused save returned the stored composition) →
    `values = live = stored`, `View`.

## The view maths (`geometry.ts`, with the P28 contract)
`frameBox(stage: Box, rectRatio) `: the largest box of ratio `rectRatio = (w·W_t)/(h·H_t)` centred in `stage` shrunk by `FRAME_MARGIN_PX = 24`
on every side. `scale s = frameBox.width / (w·W_t)`. The preview element shows the turned, flipped frame at `W_t·s × H_t·s`; its **centre** is at
`frameBox.left + (0.5 − x)·W_t·s`, `frameBox.top + (0.5 − y)·H_t·s`; the element's CSS transform is `translate(cx − W_t·s/2, cy − H_t·s/2)
rotate(θdeg)` with `transform-origin: center` (the straighten rotates the photo about its centre under the fixed frame), and the turn/flip are
applied to the inner `<img>` (`rotate(90·turns deg) scaleX(flip ? −1 : 1)` around its centre, the inner box sized to the turned frame).
`viewTransform(values, frame, stage) → { frameBox, scale, centre, rotateDeg }`; `panBy`, `zoomAbout` are the inverses used by the machine.

## Ordered tasks
0. **Registry first (master plan §6.10).** `services/compositions.ts`: `setComposition(photoId, values) → Promise<Composition>`,
   `resetComposition(photoId)`, types `Composition = CompositionDto`, `CompositionValues = CompositionValuesDto`, `Rect`, `Frame`. Constants in
   `geometry.ts`/`machine.ts`: `MIN_RECT_FRACTION = 0.05`, `FINGERPRINT_TOLERANCE = 0.002`, `FRAME_MARGIN_PX = 24`, `ZOOM_PER_PX = 0.005`,
   `WHEEL_SETTLE_MS = 120`, `EDGE_REVEAL_PX = 12`, `COMPOSE_HINT_KEY = 'compose.hintShown'`. `PhotoMenuAction` gains `'compose'` (label `Compose…`,
   hint `R`, disabled when the photo has no `adjusted_preview_path` or `composition.rect === null`). CSS classes: `.compose-stage`, `.compose-image`,
   `.compose-overlay` (SVG), `.compose-frame`, `.compose-done`, `.compose-browser[data-open]`, `.tile-composition`, `.tile-frame`, `.tile-review`.
1. **Geometry** (`geometry.ts`) per P28's contract and the view maths; `geometry.test.ts` loads `composition_cases.json` from
   `crates/core/src/mechanisms/` (path resolved from `import.meta.url`) and runs every `op`.
2. **Machine** (`machine.ts`) per the interaction contract; pure, no DOM, no React.
3. **Binding** (`useComposeGesture.ts`): attaches to the `.compose-stage` element: `pointerdown` (calls `setPointerCapture` when available),
   `pointermove`, `pointerup`, `pointercancel`, `lostpointercapture`, `wheel` (`{ passive: false }`, `preventDefault` per effect), `gesturestart`/
   `gesturechange`/`gestureend` (WebKit), `window` `blur`, `document` `visibilitychange`, `keydown` Escape/Enter (the compose keyboard, which also
   swallows every workspace key); a `Tick` every `WHEEL_SETTLE_MS / 2` while `wheeling` (a timer, not rAF); `View` effects set a ref and schedule
   **one** `requestAnimationFrame` that writes `transform` on the image element (a second `View` before the frame replaces the pending rect: one
   write per frame); `Save` effects go to `useCompositionSaves`; while composing a document-level `gesturestart` listener calls `preventDefault`
   (page zoom), and the stage has `touch-action: none; overscroll-behavior: none`.
4. **Saves** (`useCompositionSaves.ts`): per photo `{ inFlight: ticket | null, pending: values | null }`; `save(photoId, values)`: in flight →
   `pending = values` (latest wins); else send `setComposition` with a new ticket; on settle: ticket older than the photo's latest → ignore; success
   → `onSaved(photoId, composition)` (updates the workspace view's photo in place — `setView` on `useWorkspaceState` with the new `composition`, so
   tiles redraw without a reload) and send `pending` if any; failure → `toast.error('Not saved: ' + message)` and `onSaved(photoId, lastStored)` so
   the machine (rule 16) snaps the view back. Leaving compose mode flushes nothing extra (every gesture end already saved).
5. **Compose mode** (`ComposeMode.tsx` + `ComposeStage.tsx` + `ComposeFrame.tsx`): `WorkspacePage` holds `composing: boolean`; entry: `R` (via a
   `useKeyboard({ r })` enabled when not composing, no dialog, the active photo has a preview and a rect), `dblclick` on the stage image
   (`StageImage`'s `.stage-frame`), menu action `compose`; while composing: `useWorkspaceKeyboard(…, false)` and `useKeyboard({ d }, false)`, the
   `Stage` renders `<ComposeStage>` instead of `<SwipeLayer><StageImage/></SwipeLayer>` and the `TogglePill`/`ReviewButtons` (the editor shows the
   adjusted preview only), a `Done` button (`.compose-done`, label `Done`, hint Return) in the stage bar; the right column renders no `PhotoPanel`/
   `FoldedStrip`; `<BrowserOverlay>` is mounted; exit on `Exit` effect / Done click → `composing = false`, the normal stage returns with the same
   active photo. The overlay SVG dims everything outside the frame (a mask) and draws the frame border; no handles. On open, when
   `localStorage[COMPOSE_HINT_KEY] !== '1'` (guarded with try/catch) show the line `Crops made in Capture One are not used.` once (a toast or a
   caption; Delegated) and set the key.
6. **Browser overlay** (`BrowserOverlay.tsx`): absolutely positioned over the stage's right side, width = the photo panel's width, renders
   `<PhotoPanel>` (same tiles, no fold button); `data-open` becomes `"true"` when a `pointermove` on the workspace has `clientX ≥ window.innerWidth −
   EDGE_REVEAL_PX` **and** the machine is `idle`; `"false"` on `pointerleave` of the overlay; a tile click → `SwitchPhoto` (the machine commits first).
7. **Tiles** (`PhotoTile.tsx`): `Thumb` prefers `thumbnail_path`; `PhotoTileFace` adds `<TileComposition>` when `composition.rect` is non-null: a
   `.tile-composition` box with `style.aspectRatio = "<W_t> / <H_t>"` (turned frame) centred in the tile (`contain`), holding `.tile-frame` with
   `left/top/width/height` = `x·100% / y·100% / w·100% / h·100%` and a dimmed outside (`box-shadow` or an SVG mask); when `composition.state ===
   'needs_review'` a `.tile-review` corner (`--warn`) with the text `Re-frame`. The drag overlay and the folded strip render `PhotoTileFace` and follow.
8. Tests, `npm run typecheck`, `npm test`; the Rust suite is untouched.

## Criteria (8 criteria, 27 rows; 4 `[rule]` rows)
Test fixture: the p22 harness (`openWorkspace`, mocks) with photos carrying `thumbnail_path`, `composition` (frame 2560×1707, default square rect
`(0.166601562, 0, 0.666796875, 1)`, `state ok`, `revision 0`); the stage `.compose-stage` has a fixed `getBoundingClientRect` of `1000 × 700` at
`(0, 0)` (so the square frame box is `652 × 652` at `(174, 24)`, `s = 652 / 1707 = 0.381956`, `W_t·s = 977.81`, `H_t·s = 652`); `setComposition` is
`vi.fn()` resolving to the sent values with `revision + 1` unless the row says otherwise; `pointer(type, x, y, extra)` dispatches a `PointerEvent`
with `pointerId 1`, `isPrimary true`, `button 0`; rAF is captured with `vi.spyOn(window, 'requestAnimationFrame')` and flushed by hand.

| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | `geometry.test.ts` | every case of `crates/core/src/mechanisms/composition_cases.json` (P28 C1: 11 `default_rect`, 4 `pixel_rect`/`split_halves`, 13 `validate`, the `fingerprint` and `inside` cases) | every case passes with the same tolerances as the Rust test (`1e-9` fractions, exact integers, the same reason strings) | MC-27 §Pixel rule ("one function, used by every renderer and by the tests"); §Validation; §Default |
| C1(b) | `geometry.test.ts`, view maths | stage `1000×700`, default square on 2560×1707: `viewTransform`; `panBy(rect, (−50/977.81, 0))`; `zoomAbout(rect, frame centre, 1.6487)` (= `exp(100·0.005)`); `zoomAbout(rect, centre, 0.5)`; `zoomAbout(rect, centre, 20)` | `frameBox == {174, 24, 652, 652}`, `scale == 0.381956 ± 1e-6`, centre `(174 + (0.5 − 0.166601562)·977.81, 24 + 0.5·652) = (500.0, 350.0) ± 0.01`; pan → `x = 0.115466 ± 1e-6`, `y 0`; zoom in → `w = 0.404440`, `h = 0.606531`, `x = 0.297780`, `y = 0.196735` (± 1e-6; the frame centre stays under the pointer); zoom ×0.5 → clamped to the default square (`w = 0.666796875`, `h = 1`); zoom ×20 → clamped at `h = 0.05`, `w = 0.033340` (`85.35 px` short side) with the centre kept at `(0.5, 0.5)` | 10D §What the photographer sees ("zooms the composition around the pointer"; "can never extend past the photo's edge"); P28 contract (`MIN_RECT_FRACTION`) |
| C1(c) | `machine.test.ts` | one row per rule 1–16 of the interaction contract, driven as event sequences on an `idle` state with the fixture above (e.g. rule 1: `PointerDown{isPrimary:false}` → still `idle`; rule 4: `PointerDown`, `PointerMove(+50,0)`, `Blur` → `idle`, one `Save` with `x = 0.115466`; rule 5: …, `KeyEsc` → `idle`, `live == startRect`, no `Save`; rule 10: `Wheel{pan}`, `PointerDown`, `PointerMove`, `PointerUp` → two `Save`s; rule 11: `GestureStart`, `Wheel{ctrlKey}` → only `PreventDefault`; rule 12: `PointerDown`, `PointerMove`, `SwitchPhoto` → `Save` for the old photo then `Switch`; rule 16: `Saved{other photo}` → state unchanged) | each sequence ends in the phase, `live`, `values` and effect list the rule states — the test file lists all 16 | 10D §What the photographer sees (gestures, Esc, Return, autosave); owner 2026-11-04 ("robust and reliable interactions") |
| C2(a) | `<App>` workspace, photo 1 active | press `r`; separately `dblclick` on `.stage-frame`; separately the ⋯ menu entry `Compose…`; separately the right-click entry on tile 1 | each: `.compose-stage` present, `.compose-image` has `transform` `translate(11.09px, 24px) rotate(0deg)` (± 0.5 px: centre 500 − 977.81/2 = 11.09, 350 − 326 = 24), `.compose-overlay` present, `.compose-done` button present; no `.swipe-layer`, no `.toggle-pill`, no `.review-buttons`; no `aside.photo-panel` inside `main.workspace`'s layout (only inside `.compose-browser`) | `[rule]` 10D §What the photographer sees ("Compose (R, double-click, or Compose… in the ⋯ and right-click menus) turns the stage into the editor … the photo browser on the right hides") |
| C2(b) | composing | press `Enter`; separately click `Done` | `.compose-stage` gone, `.swipe-layer` back, the same photo active, `setComposition` **not** called (nothing changed) | 10D ("Return finishes"; "Opening and closing the editor without a change is not" a change — MC-29) |
| C2(c) | composing | press `a`, `c`, `Backspace`, ` ` (Space), `f`, `i`, `d` | `classifyPhoto`/`deletePhoto` not called; `.toggle-pill` absent and the mode unchanged after leaving; the panel not folded; no item card, no form dialog; `.compose-stage` still present | 10D ("All workspace keys (A, C, Delete, Space, F…) … are off while composing") |
| C2(d) | a photo with `adjusted_preview_path: null` (or `composition.rect: null`) active; press `r`; open the ⋯ menu | an unready photo | no `.compose-stage`; the menu entry `Compose…` has `aria-disabled="true"` | this plan's Goal (the editor needs a frame); MC-27 §Default (rect materialised late) |
| C2(e) | fresh `localStorage`; press `r`; leave; press `r` again | the first-time line | the text `Crops made in Capture One are not used.` appears on the first open and not on the second; `localStorage.getItem('compose.hintShown') === '1'` | `[rule]` 10D §What the photographer sees (last bullet) |
| C3(a) | composing; `pointer('pointerdown', 500, 350)`; `pointer('pointermove', 530, 350)`; `pointer('pointermove', 550, 350)`; flush rAF; `pointer('pointerup', 550, 350)` | a drag of +50 px | between the two moves `requestAnimationFrame` was called **once** (coalesced) and `.compose-image`'s `transform` after the flush is `translate(61.09px, 24px) rotate(0deg)` (± 0.5: the image followed by 50 px); after `pointerup` exactly one `setComposition(1, values)` with `rect.x = 0.115466 ± 1e-6`, `rect.y = 0`, `rect.w = 0.666796875`, `rect.h = 1`, `kind 'normal'`, `shape '1:1'`, `quarter_turns 0`, `flip 'none'`, `straighten 0` | 10D ("drag … moves the photo"; "gestures update a CSS transform once per frame and never wait for Rust"; autosave "when a gesture ends") |
| C3(b) | composing; drag from `(500, 350)` to `(900, 350)` (+400 px: `x` would go to −0.24) and release; then drag `(500, 350)` → `(100, 600)` | past the edge | first save `rect.x == 0`; second save `rect.x == 0.333203125` (`1 − w`) and `rect.y == 0` (`h = 1`: nothing to move vertically) | 10D ("The composition can never extend past the photo's edge") |
| C4(a) | composing; `wheel` on `.compose-stage` at `(500, 350)` with `ctrlKey: true, deltaY: −100`; advance timers 130 ms | a ctrl-wheel (trackpad pinch as WKWebView reports it) | `.compose-image` transform scale grew (the element is `translate(…) rotate(0deg)` but its width style is `W_t·s'` = `977.81·1.6487 ≈ 1612 px` ± 1); after the settle one `setComposition` with `rect == {0.297780, 0.196735, 0.404440, 0.606531}` ± 1e-6; the wheel event's `defaultPrevented === true` | 10D ("pinch, or ⌘ + mouse wheel, zooms the composition around the pointer"); C1(b) numbers |
| C4(b) | composing; `wheel` with `metaKey: true, deltaY: −100` | ⌘ + wheel | the same outcome as (a) | 10D ("⌘ + mouse wheel") |
| C4(c) | composing; `gesturestart` at `(500, 350)`; `gesturechange` with `scale 1.6487`; a `wheel` with `ctrlKey` during the gesture; `gestureend` | WebKit gesture events, with the duplicate wheel | one `setComposition` with the rect of (a) (the wheel was ignored: `defaultPrevented` true, no second zoom) | 10D ("pinch"); interaction rule 11 |
| C4(d) | composing; `wheel` `ctrlKey, deltaY: +2000` (zoom out) then settle; `wheel` `ctrlKey, deltaY: −3000` then settle | both clamps | first save `rect == default square` (not past the edge); second save `rect.h == 0.05`, `rect.w == 0.033340 ± 1e-6` | 10D ("can never extend past the photo's edge"); `MIN_RECT_FRACTION` |
| C5(a) | composing; `wheel` `deltaX: −50, deltaY: 0` (no modifier); settle | a two-finger swipe right | `.compose-image` moved right by 50 px before the settle; one `setComposition` with `rect.x = 0.115466 ± 1e-6`; `defaultPrevented === true` | 10D ("two-finger swipe moves the photo"; "stage swipes … the two-finger back swipe are off") |
| C5(b) | composing: dispatch `gesturestart` on `document.body` (outside the stage); leave compose mode; dispatch again | page zoom off only while composing | while composing `defaultPrevented === true`; after leaving `false` | 10D ("the webview's page zoom … off while composing") |
| C5(c) | composing; `pointerdown` + `pointermove(+50)` then `wheel` `ctrlKey, deltaY: −100` then `pointerup` | a wheel during a drag | the wheel's `defaultPrevented` is true and it changed nothing: one save with the drag's rect `x = 0.115466` and `w = 0.666796875` | interaction rule 8 (one gesture at a time) |
| C6(a) | composing; `pointerdown`, `pointermove(+50)`; press `Escape`; `pointerup` | Esc mid-drag | the transform returns to `translate(11.09px, 24px)`; **no** `setComposition` call; `.compose-stage` still present (Esc mid-gesture cancels the gesture, not the editor) | interaction rule 5; owner ("interrupted gestures") |
| C6(b) | composing; `pointerdown`, `pointermove(+50)`, then (1) `pointercancel`; (2) `window.dispatchEvent(new Event('blur'))`; (3) `document.visibilityState = 'hidden'` + `visibilitychange`; (4) `lostpointercapture` — one fresh editor each | interrupted drags | each: exactly one `setComposition` with `rect.x = 0.115466` (the visible state is committed), the machine back to idle (a following `pointermove` without `pointerdown` changes nothing) | interaction rule 4 |
| C6(c) | composing; `pointerdown` pointerId 1 at `(500, 350)`; `pointerdown` pointerId 2 (`isPrimary false`) at `(600, 350)`; `pointermove` pointerId 2 to `(700, 350)`; `pointerup` pointerId 2; `pointermove` pointerId 1 to `(550, 350)`; `pointerup` pointerId 1 | a second pointer | one save with `x = 0.115466` (only pointer 1 moved the photo) | interaction rule 1 |
| C6(d) | composing after a drag saved `x = 0.115466`; press `Escape` (idle) | Esc revert | one more `setComposition` with the **entry** values (`rect.x = 0.166601562`); the transform returns to the entry placement | 10D ("Esc reverts this photo to how it was when the editor came to it"); interaction rule 6 |
| C7(a) | composing on photo 1; `setComposition` for photo 1 returns a pending promise `p1`; drag (+50) and release (save 1 pending); open the browser (C8(a)) and click tile 2 (`SwitchPhoto`); drag photo 2 (+50) and release → `setComposition(2, …)` resolves at once; then resolve `p1` with `{…values, revision: 1}` | rapid switching while a save is pending | `setComposition` calls in order: `(1, x = 0.115466)`, `(2, x = 0.115466)`; after `p1` resolves, tile 1's `.tile-frame` `left` is `11.5466%` and photo 2's `.compose-image` transform is unchanged (the late response touched photo 1 only); `getSessionView` was **not** called again (in-place update) | 10D ("Clicking a photo there moves the editor to it and keeps the changes"); interaction rules 12, 16; P26 lesson (no reload races) |
| C7(b) | composing on photo 1; two drags released in quick succession: save A pending `pA`, save B pending `pB` (B is the later request); resolve `pB` with `revision 2` then `pA` with `revision 1` | out-of-order settles | the view and tile 1 show B's rect (`pA`'s stale answer is ignored); exactly two `setComposition` calls | latest-wins per photo (the saves contract) |
| C7(c) | composing; `setComposition` rejects with `ServiceError { code: 'composition_invalid', message: 'off_shape' }` on a drag release | a refused save | a toast `Not saved: off_shape` (sonner `toast.error` mock); the transform returns to the last stored rect (`translate(11.09px, 24px)`); the editor stays open | MC-27 §Validation ("Invalid → the save is refused and the previous composition stays") |
| C8(a) | composing; `pointermove` on `main.workspace` at `clientX = window.innerWidth − 5`; then `pointerleave` on `.compose-browser` | the edge reveal | `.compose-browser` gets `data-open="true"` and contains the tiles (`[data-photo-id]` for every photo); after `pointerleave` `data-open="false"`; the overlay is positioned over the stage (its `className` contains `compose-browser`, rendered inside `.stage-column`, not as a flex sibling) | `[rule]` 10D §What the photographer sees ("slides in over the image when the pointer reaches the window's right edge, hiding again when the pointer leaves it") |
| C8(b) | composing; `pointerdown` on the stage, then `pointermove` at `clientX = innerWidth − 5` (still down); `pointerup` | the edge during a drag | `data-open` stays `"false"` during the drag; a `pointermove` at the edge **after** the release opens it | 10D ("never during a drag") |
| C8(c) | tiles in the normal workspace (not composing) with photo 1 `composition.rect = (0.166601562, 0, 0.666796875, 1)`, frame 2560×1707, `thumbnail_path '/previews/1/thumb-1.jpg'`; photo 2 `rect: null`; photo 3 `state: 'needs_review'`, `quarter_turns: 1` | tile overlays | tile 1: `img.thumb-image` `src` ends with `thumb-1.jpg`; `.tile-composition` `style.aspectRatio == "2560 / 1707"`; `.tile-frame` `style.left == "16.6602%"`, `top "0%"`, `width "66.6797%"`, `height "100%"` (4 decimals); tile 2: no `.tile-composition`; tile 3: `.tile-composition` `aspectRatio "1707 / 2560"` (turned) and a `.tile-review` element with text `Re-frame` | `[rule]` MC-28 §Thumbnail ("grid tiles, which draw the composition over it with CSS"); MC-27 §Revisions ("the tile shows an amber corner and Re-frame") |

## Delegated choices
How `useBackKeys`' Esc-goes-back is suppressed while composing (a `[role="dialog"]` attribute on `.compose-stage`, which `useBackKeys` already
respects, or a paused flag); the `--warn` token value (an amber in the tokens' oklch style, light and dark); whether the first-time line is a
sonner toast or a caption under the stage bar; the mask technique of the overlay (SVG `<mask>` or four rects); whether the inner turn/flip
transform is one `<img>` or a wrapper; the `ComposeState` store (a `useRef` + `useSyncExternalStore`, or a small class) — anything but React state
during a gesture; the wheel settle timer granularity; how the `BrowserOverlay` width follows the panel's CSS variable; how `onSaved` writes into
`useWorkspaceState` (a new `patchPhoto(photoId, patch)` on the hook is the suggested seam).

## Manual try-out
1. Open a session, select a photo with a preview and press R (or double-click it): the stage becomes the editor — a square frame, the photo
   under it, the right panel gone. Drag the photo: it follows the pointer smoothly; let go: a moment later the tile in the browser (edge-reveal
   it: move the pointer to the right edge) shows the new framing. Pinch on the trackpad, or hold ⌘ and scroll: the composition zooms around the
   pointer; you cannot zoom out past the photo's edge nor zoom in to a sliver. Two-finger swipe moves the photo; nothing in the window zooms or
   navigates back.
2. Start a drag and press Esc before letting go: the photo snaps back to where the drag started. Press Esc again with nothing in progress: the
   photo returns to how it was when you opened the editor. Return (or Done) leaves the editor; A/C/Delete/Space do nothing while it is open.
3. Drag, then immediately click another photo in the edge-reveal browser and drag that one: both keep their framing (check the tiles).
4. The first time you opened the editor a line said "Crops made in Capture One are not used."; it does not come back.
5. Tiles show the photo with the frame drawn over it; a photo rotated in Capture One (P28 step 3) shows an amber corner reading "Re-frame".

## Notes
- **Why a pure machine.** Every bug the owner fears ("buggy interactions") is a state bug: a lost pointerup, a wheel landing mid-drag, a late
  save. A reducer with an enumerated event set makes each such sequence a unit row that runs in milliseconds; the DOM binding is then a thin
  translator that the component tests cover once per event type.
- **Why the frame stays and the photo moves.** 10D's words; also the arithmetic: the frame box is fixed per stage size and shape ratio, so a
  gesture changes only `x, y, w, h`, never the frame, and the transform is one `translate … rotate`.
- **Trackpad facts** (owner brief 2026-11-04): in WKWebView a pinch arrives as `wheel` events with `ctrlKey` and/or as `gesturestart/change/end`;
  a two-finger swipe arrives as `wheel` deltas with momentum. The machine handles both and prefers gesture events while one is open; the
  momentum tail is just more pan wheels until the settle.
- **Straighten is read, not written, here** (P32 adds the control); the transform already carries `rotate(θ)` so a straightened composition from
  the backend displays correctly.
- **Reduced motion is P34**; nothing animates in this phase except the browser slide (CSS transition), which P34 gates.
- Registry additions made by the planner in master plan §6.1, §6.9, §6.10. No intention-book change, no backend change.
- Estimated size: ~14 files, ~1 900 lines (≈ 450 geometry + machine, ≈ 500 components and hooks, ≈ 950 tests).

## Review log
