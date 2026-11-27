---
phase: LV10_zoom_reframe_guides
title: Phone tools — pinch to zoom in the full-screen preview to judge detail and sharpness; reframe a photo on the phone exactly as the Mac's composition (Normal or Split, shape, quarter turns, flip, straighten, the frame), with two fingers to scale and straighten, a fine grid while straightening and the composition guides inside the frame, saved into the Mac's own composition; and the composition guides on the live view, chosen from the live screen's menu
difficulty: high
projection: no
depends_on: [LV9_preview_corrections]
implementer: fable
blocked_on: none
---
# LV10 — Zoom, reframe and guides on the phone (book 10J LV-D21 (5)–(7))

**Tracker:** NOT_STARTED (2026-11-27, planner)

(This line is the milestone's tracker row. Sessions change only this line of this file. The gate for `depends_on` is
the Tracker line of the plan named there.)

## Goal
Book `10J` **LV-D21** (owner 2026-11-27, changelog D21; built from the description, judged in use), new tools (5)–(7):
(5) **pinch to zoom** in the Preview (two fingers; pan while zoomed), one gesture module reused by (6); (6) **reframe on
the phone** from the Preview's drawer: the composition of book `10D` **MC-27** (Normal/Split, shape, quarter turns, flip,
straighten, rect), phone gestures, a fine grid while straightening, the composition guides inside the frame, Reset,
Done/Cancel, saved through a phone route into the Mac's own `CompositionService` (MC-27 validation; the Mac's outputs
follow the new revision as for its own edits, MC-28; undo on the Mac, MC-30); (7) the **composition guides on the live
view**, chosen from the live screen's menu, remembered on the phone, drawn by the UI only (H3).
(U1) The Mac's side: the composition on the phone wire and two save routes. (U2) The pinch module, zoom in the Preview,
and photos shown as composed. (U3) The reframe editor. (U4) Saving from the editor; guides on live.
**Not here:** the Mac's editor (unchanged; its pure geometry is reused, never edited); trimming handles on the phone (a
Free shape keeps its proportions on the phone, P8); Capture One's crop; any image processing (H3: the phone moves CSS
transforms over the preview the Mac already serves); copy/paste framing; the master and output render themselves
(MC-28, unchanged: a new revision makes outputs stale exactly as a Mac edit does).

## How to work this plan (flow.md §10, master_plan_live.md §3)
One runner phase of 4 units, in order; after each, its **Unit test commands**, then a `CHECKPOINT (not approved):`
commit of that unit's paths. Row ids `LV10·C<n>(<row>)`; tests named after their row (`lv10_c5a_<slug>`; Rust
`lv10_c1a_<slug>`). Open details: 10J LV-D21 and 10D first, then the design README's tokens and haptics (local, read,
never copy; no new prototype), then the smallest option; recorded under **Assumptions**. Reviewer notes:
`docs/implementation/backlog_live/LV10_zoom_reframe_guides.md`. `implementer: fable`. `difficulty: high` (two-finger
gestures, a geometry-bound editor, a save path into the composition service).
**Why the server part is a unit here and not its own milestone:** one field in `phone_view` (read from the stored
composition), two thin routes calling `CompositionService` through one new revision-checked function, three error codes
(4 criteria, no schema, no camera code); a separate default-implementer milestone would add a full review cycle. H8: the
revision rule and every validation are in `crates/core`; the routes parse, guard and call.

| Unit | What it builds | Criteria | Rows | Changes app files |
|---|---|---|---|---|
| U1 | The composition on the wire (PH6c); `POST …/composition`, `POST …/composition/reset` (PH4b) | 4 | 9 | yes (Rust) |
| U2 | `pinch.ts`; zoom in the Preview; photos shown as composed | 4 | 13 | yes (TypeScript, guide) |
| U3 | The reframe editor: gestures within MC-27, controls, grid, guides, Reset, Cancel | 5 | 18 | yes (TypeScript, guide) |
| U4 | Done → the Mac; refusals; guides on live | 3 | 13 | yes (TypeScript, guide) |
| **Total** | | **16** | **53** | |

## Read first (whole milestone)
`docs/process/flow.md` (§4, §4A, §6, §7A, §10) · `master_plan_live.md` §3, §4.5, **§5 PH4b, PH6c, PH11, PH12** (v1.3,
this milestone's contract), §6 (phone texts, v1.3) · book `10J` **LV-D21**, LV-D20 · book **`10D`** (§What the
photographer sees: the frame stays and the photo moves under it, the straightening grid, the Overlay guides; **MC-27**
whole; MC-28 §Cache key; §The result is shown) · `05A` MC-30 (undo) · plan `LV9_preview_corrections.md` (the Preview's
layout and photo drawer this milestone extends; the pending chain) · the Mac's pure composition modules this milestone
**imports, never copies or edits**: `apps/desktop/src/features/compose/{geometry.ts, refit.ts, overlays.ts, composed.ts}`
and the constants `OVERLAY_ORDER`, `STRAIGHTEN_GRID_PX` of `useComposePrefs.ts` · `crates/core/src/services/composition.rs`,
`crates/core/src/mechanisms/composition.rs` (`validate`, `default_rect`) · `apps/desktop/src/phone/{api.ts, state.ts,
gestures.ts, geometry.ts, PhoneApp.tsx, phone.css, components/{Preview.tsx, PhotoDrawer.tsx, PhotoImage.tsx, Stage.tsx,
ItemDrawer.tsx}}`, `__tests__/helpers.ts`.

## Points recorded by planning (each is one sentence in the narrative's "Choices made for you")
The book leaves these open; each is the smallest option consistent with LV-D21 and 10D.
- **P1 Zoom:** two fingers zoom about the point between them, from 1× to 6× (while the fingers are down it may stretch
  to 0.8× and 7× and springs back on release); one finger pans while zoomed, never past the picture's edge; while zoomed,
  swipes and hold-for-template are off (the strip, ⋯ and × still work). **Double-tap**: when zoomed, back to 1×; at 1×,
  3× about the tapped point. The zoom goes back to 1× when the shown photo changes.
- **P2 A second finger during a swipe or hold** ends it (the photo springs back, nothing is sent) and starts a pinch.
- **P3 The phone shows photos as composed** (10D §The result is shown), in the Preview and the strips, once the Mac has
  Capture One's full-frame preview; before that (the camera's own preview only) it shows the picture as today and
  Reframe waits ("Waiting for Capture One's full-frame preview", the Mac's own words).
- **P4 Reframe works like the Mac's editor:** the frame stays centred and the photo moves under it. One finger moves the
  photo; two fingers move it, zoom it (spread = closer) and straighten it (twist); the photo follows the fingers. A
  twist counts after the fingers have turned 3°, then follows them without a jump, in 0.1° steps, −45° to 45°.
- **P5 The frame never shows empty corners:** while you move, zoom or straighten, the photo stops where the frame would
  leave it (it never springs back); straightening shrinks the frame's share about its centre just enough, as the Mac
  does.
- **P6 The fine grid** (24 px squares, the Mac's) shows while two fingers straighten and while the straighten slider is
  held, and only then. **Guides** inside the frame: Off, Thirds, Grid, Golden ratio, Diagonals, remembered on the phone
  (Thirds the first time, as the Mac).
- **P7 Done saves, Cancel discards:** nothing reaches the Mac before Done; an unchanged Done sends nothing; Reset puts
  the Mac's default framing in the editor (the default shape, centred, as large as fits) and Done then saves it as the
  default. If the photo's framing changed on the Mac meanwhile, Done refuses and says so (nothing is overwritten).
- **P8 No trimming handles on the phone:** a Free shape keeps its current proportions on the phone (trim it on the Mac).
- **P9 The editor stays upright** (the phone held normally) even for a long shot; the frame fits the screen.
- **P10 Guides on live:** the live screen's menu has a Guides row (Off, Thirds, Grid, Golden ratio, Diagonals); Off the
  first time; remembered on the phone; drawn over the live picture and the template alike (both are framing); separate
  from the editor's choice.

## Shared for all units
- **Boundary (UI rows):** `PhoneApp` with the Mac faked at `fetch`/`WebSocket` (`helpers.ts`), mouse events of the
  pointer types with a `pointerId` (jsdom has no `PointerEvent`); **two-finger gestures use two `pointerId`s** (1 and 2)
  on the same element; fake timers.
- **Fixture K (composed):** `sortSession` (LV8 fixture S) with `default_shape: "1:1"` and photo 201's `composition` =
  `{kind: "normal", shape: "1:1", orientation: "named", quarter_turns: 0, flip: "none", straighten: 0, rect: {x:
  0.1666016, y: 0, w: 0.6667969, h: 1}, frame: {w: 2560, h: 1707}, state: "ok"}` (the default 1:1 rect, `1707/2560 =
  0.6667969`); the other photos `composition: null`. **Fixture K45:** K with 201's composition shape "4:5", rect `{x:
  0.2, y: 0, w: 0.5334375, h: 1}` (`0.8 · 1707 / 2560`).
- **Editor layout:** in the layout stub the editor's stage (`data-testid="reframe-stage"`) measures 390 × 520 at (0, 0);
  for a 1:1 rect the frame box is `frameBox` = left 24, top 89, 342 × 342, so `s = 342 / 1707` screen px per source px
  and the box centre is (195, 260).
- **Preview layout:** stage 390 × 610 (LV8), viewport centre (195, 305); a 1:1 picture is 390 × 390.
- **Imports from the Mac's code** (pure modules only; type-only imports of `@/services/compositions`): `geometry.ts`
  (`frameBox`, `viewTransform`, `transformCss`, `anchorIn`, `panBy`, `zoomAbout`, `clampRect`, `validate`, `defaultRect`,
  `turnedFrame`, `insideAfterStraighten`), `refit.ts` (`refit`), `overlays.ts` (`overlayLines`), `composed.ts`
  (`composedLayout`), `useComposePrefs.ts` (`OVERLAY_ORDER`, `STRAIGHTEN_GRID_PX`, constants only). No phone module
  imports anything that imports `@/services/bindings` or `@tauri-apps/*`.
- **Visible hooks:** the zoom layer `data-testid="preview-zoom"` with `transform: translate(<x>px, <y>px) scale(<s>)`; the
  composed picture's frame element `data-testid="composed-frame"` with `composedLayout`'s `frame` (width, height, left,
  top) and `rotate` as inline style; the editor `role="dialog"` `aria-label="Reframe"` `data-testid="reframe-editor"`; its
  picture `data-testid="reframe-picture"` with `transformCss(viewTransform(values, frame, stage))` as `transform`; its
  frame box `data-testid="reframe-frame"`; the grid `data-testid="reframe-grid"`; guide lines as SVG `<line>` in
  `data-testid="reframe-guides"` / `data-testid="live-guides"`.
- **Core mechanisms** (the only unit tests, flow §4): new `apps/desktop/src/phone/pinch.ts` (PH11) and
  `apps/desktop/src/phone/reframe.ts` (PH12); unit tests `lv10_core_<fn>_…` in `__tests__/lv10_core.test.ts`, input →
  output, one case per rule line.

## Unit 1 — The Mac's side: the composition on the phone wire, and saving it (LV-D21 (6); PH4b, PH6c)

**Unit id:** `LV10·U1` · **Rows:** LV10·C1–C4 · **Changes app files:** yes (Rust)

### Goal
- **PH6c (view):** `PhoneImage.composition` = `{kind, shape, orientation, quarter_turns, flip, straighten, rect: {x, y, w,
  h}, frame: {w, h}, state: "ok" | "needs_review"}` when the photo's stored composition has a rect and a frame **and** the
  file the preview route serves is Capture One's adjusted preview (`adjusted_preview_path` set and on disk: the picture is
  the full frame the composition is measured in, MC-27 §Source frame); else `null`. `PhoneSession.default_shape` = the
  setting (`"1:1"`, …). Enum spellings are core's serde (`normal`/`split`, `"1:1"`…`"free"`, `named`/`turned`,
  `none`/`horizontal`), so the phone hands them to the Mac's TypeScript geometry unchanged.
- **PH4b (routes):** `POST /api/phone/v1/photos/{id}/composition {session_id, revision, values: {kind, shape,
  orientation, quarter_turns, flip, straighten, rect}}` → `{revision}`; `POST /api/phone/v1/photos/{id}/composition/reset
  {session_id, revision}` → `{revision}`. Guard (PH4 `session_changed`), `session_photo` (404) as every photo route; then
  core `CompositionService::set_at_revision(photo_id, revision, values)` / `reset_at_revision(photo_id, revision)`: the
  stored revision ≠ `revision` → `Error::CompositionChanged` (409 `composition_changed`), nothing saved; else exactly
  `set` / `reset` (MC-27 validation → `CompositionInvalid` 400 `composition_invalid`; no frame → `FrameUnknown` 409
  `frame_unknown`; one undo step "Compose NN"; `PhotoChanged` + `SessionChanged` after the commit, PH8). Unchanged values
  → 200 with the same revision, no step, no event. A body that does not parse (an unknown enum spelling) → 400
  `invalid_value`.

### Read first
`crates/core/src/services/composition.rs`, `crates/core/src/services/phone_view.rs` (`image`), `crates/core/src/error.rs`,
`crates/integrations/src/phone/{routes.rs (move_photo, classify_photo), wire.rs (error_code)}`,
`crates/testkit/tests/suite/p28_composition_foundation.rs` (`ready`, `preview`, `values`: a photo with a 2560 × 1707
adjusted preview), `crates/testkit/tests/suite/lv2_phone_http.rs`, `crates/testkit/src/phone.rs`.

### Files
Edited: `crates/core/src/services/{composition.rs, phone_view.rs}`, `crates/core/src/error.rs` (`CompositionChanged`),
`crates/integrations/src/phone/{routes.rs, wire.rs}`, and where the desktop maps error codes if the compiler requires it
(`apps/desktop/src-tauri/src/commands/items.rs`). New: `crates/testkit/tests/suite/lv10_phone_composition.rs` (+ its
`mod` line in `crates/testkit/tests/suite/main.rs`). Guide: `docs/guide/02_architecture.md` (the phone's composition
routes beside "Phone classify"), `docs/guide/04_change_impact.md` (the phone imports the Mac's composition geometry: a
change to `features/compose/{geometry, refit, overlays, composed}.ts` reaches the phone).

### Core mechanisms
None new (`set_at_revision` is `set` behind a revision check; rows test it at the route).

### Ordered tasks
1. `CompositionChanged`; `set_at_revision`, `reset_at_revision` in `CompositionService`.
2. PH6c in `phone_view` (`image` already reads the composition) and `default_shape`.
3. Wire bodies, the two routes, the three error codes.
4. Tests for the rows; guide.

### Criteria (4 criteria, 9 rows)
Boundary: HTTP through `TestPhone` with a paired token on an app built as `p28`'s `ready()` (photo P with a 2560 × 1707
adjusted preview and the default 1:1 composition, revision 0), P classified Main in the active session S; the store, the
history and the recorded events after.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| LV10·C1(a) | the view | `GET /session` | P's image `composition` = `{kind: "normal", shape: "1:1", orientation: "named", quarter_turns: 0, flip: "none", straighten: 0, rect: <the stored rect>, frame: {w: 2560, h: 1707}, state: "ok"}`; `session.default_shape` = `"1:1"` | PH6c, MC-27 |
| LV10·C1(b) | no full-frame preview | a second photo Q of S whose frame came from the camera's preview (`Store::set_original_preview` then `Store::adopt_frame` with `FrameSource::Camera`, as `OriginalPreviewJob` does) and no adjusted preview; the test first asserts Q's stored composition has a rect and a frame | Q's image `composition` = `null` (the missing full-frame preview is the only reason) | PH6c, P3 |
| LV10·C2(a) | save | `POST /photos/P/composition {session_id: S, revision: 0, values: p28's values() (valid on that frame: a quarter turn, flip, 2.5°)}` | 200 `{revision: 1}`; the stored composition equals the values, `origin` user; history undo label "Compose NN" (or "Compose photo"); events = `PhotoChanged{P, S}` then `SessionChanged{S}`; one Mac undo (`HistoryService`) restores the default values | LV-D21 (6), MC-27, MC-30, PH8 |
| LV10·C2(b) | nothing changed | `POST` with the stored values and `revision: 0` | 200 `{revision: 0}`; no history step; no event | PH4b |
| LV10·C3(a) | invalid | `POST` with straighten 10 and the full-height default rect (corners outside after straightening) | 400 `composition_invalid`; the composition unchanged (revision 0) | MC-27 §Validation |
| LV10·C3(b) | changed on the Mac | the Mac saves a composition first (revision → 1); then the phone `POST`s with `revision: 0` | 409 `composition_changed`; the stored composition is the Mac's | PH4b, P7 |
| LV10·C3(c) | guard and ownership | `POST` with `session_id` of another session; separately a photo of another session (S active) | 409 `session_changed`; 404 `not_found`; nothing saved | PH4, Invariant PH |
| LV10·C4(a) | reset | after C2(a): `POST /photos/P/composition/reset {session_id: S, revision: 1}` | 200 `{revision: 2}`; the composition = normal, shape "1:1", no turn, flip or straighten, the default rect, `origin` default; one undo step | P7, 10D (⌘⌥R reset) |
| LV10·C4(b) | reset, stale | `POST …/composition/reset` with `revision: 0` after C2(a) | 409 `composition_changed`; nothing saved | PH4b |

### Delegated choices
Whether the revision check reads and saves in one store transaction or reads then calls `set` (the composition save is
already one transaction; a race window of one request is acceptable, the next Done sees the new revision) · the wire
struct names (`CompositionBody`, `ResetBody`).

### Notes
- `features` in `hello` is not changed (the phone and the server ship together).
- The route never renders anything: MC-28's outputs become stale by the revision, as for a Mac edit.

### Unit test commands
`cargo test -p psm-testkit --test suite lv10_phone_composition::` · `cargo check -p psm-integrations`

## Unit 2 — The pinch module, zoom in the Preview, photos shown as composed (LV-D21 (5); PH11; 10D §The result is shown)

**Unit id:** `LV10·U2` · **Rows:** LV10·C5–C8 · **Changes app files:** yes (TypeScript, guide)

### Goal
- New `pinch.ts` (PH11): the two-finger maths and the finger tracker both the Preview and the editor use.
- The Preview's touch state machine (LV8 PH9.1, LV9) gains `pinching` and `zoomed`: a second pointer in `pressing`,
  `swiping` or `holding` ends that gesture with no request (P2) and starts `pinching`; in `pinching` the zoom layer follows
  `pinchView`; releasing one finger leaves `panning` with the other (re-based) when zoomed, else waiting for it to lift
  (no swipe starts from a pinch's last finger); releasing all → `settleZoom` (200 ms ease). `zoomed` (scale > 1): one
  finger pans (`clampPan`), no swipe, no hold; double-tap → 1×; at 1× double-tap → 3× about the tap (`isDoubleTap`,
  `gestures.ts`). `pointercancel`, page hidden, window blur → the gesture ends and the zoom settles where it is. The zoom
  resets to 1× when the shown photo changes.
- **Composed display (P3):** `PhotoImage` draws a photo whose image has `composition` with `composedLayout(values,
  frame)` (the picture inside a `composed-frame` element with the layout's frame box and rotation, the picture element
  turned and flipped by `imageTransform`; a split shows its middle line), in the Preview and in the strips' tiles (the
  composed picture covers the 56 px tile); the Preview's viewport ratio is `composedLayout(...).ratioValue` (so a long
  composition turns the viewport, as today). A photo with `composition` null is drawn as today.

### Read first
`components/{Preview.tsx, PhotoImage.tsx, Stage.tsx (ImageFrame, pointerLocal)}`, `state.ts` (`previewRatio`),
`gestures.ts` (`isDoubleTap`, `SWIPE`), `geometry.ts` (`viewportBox`, `localPoint`), the Mac's
`features/compose/{composed.ts, ComposedPicture.tsx}`, master plan **PH11**.

### Files
New: `apps/desktop/src/phone/pinch.ts`, `__tests__/lv10_zoom.test.tsx`, `__tests__/lv10_core.test.ts`. Edited:
`api.ts` (PH6c types), `state.ts` (`previewRatio`), `gestures.ts` (`ZOOM`), `components/{Preview.tsx, PhotoImage.tsx}`,
`phone.css`, `__tests__/helpers.ts` (fixtures K, K45; two-pointer helpers if needed). Guide: `docs/guide/01_product.md`
(pinch zoom in the Preview; photos shown as composed).

### Core mechanisms
`pinch.ts` (PH11): `twoFinger(start: [Point, Point], now: [Point, Point]) -> {scale, rotation, anchor, mid}` ·
`pinchView(base: Zoom, start, now, box) -> Zoom` (the image point under the start midpoint stays under the current
midpoint) · `clampPan(zoom, box) -> Zoom` · `settleZoom(zoom, box) -> Zoom` · `track(state, event) -> state` (the finger
tracker: `idle | one | two`, events down/move/up/cancel with `pointerId`). `ZOOM = {max: 6, stretchMin: 0.8, stretchMax:
7, doubleTap: 3, settleMs: 200}` in `gestures.ts`.

### Ordered tasks
1. `pinch.ts`, `ZOOM` and their unit tests.
2. The Preview's `pinching`, `panning`, `zoomed`; double-tap; cancels; reset on photo change.
3. Composed display in `PhotoImage` and the Preview's ratio.
4. Tests for the rows; guide.

### Criteria (4 criteria, 13 rows)
Boundary: `PhoneApp` with fixture S (pictures not composed) for C5–C7, fixture K / K45 for C8; the Preview opened by a
tap on the dock; gestures on `preview-viewport`. "Spread ×2" = `press` (145, 305) id 1, `press` (245, 305) id 2, `slide`
id 1 to (95, 305), `slide` id 2 to (295, 305).
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| LV10·C5(a) | pinch in | Preview of 202; spread ×2; `lift` id 2, `lift` id 1 | `preview-zoom`'s `transform` = `translate(0px, 0px) scale(2)` during and after; no POST | LV-D21 (5), P1 |
| LV10·C5(b) | below 1× | Preview of 202; `press` (95, 305) id 1, `press` (295, 305) id 2, `slide` them to (145, 305) / (245, 305); release both; `advance(200)` | during: `scale(0.8)` (stretch limit); after the settle: `scale(1)`, translate 0 | P1 |
| LV10·C5(c) | above 6× | Preview of 202; three spreads ×2 in a row (fingers lifted between) | after the third release and `advance(200)`: `scale(6)` | P1 |
| LV10·C6(a) | pan while zoomed | after C5(a): `press` (195, 305), `slide` (95, 305), `lift` | `translate(-100px, 0px) scale(2)`; no swipe hint; no POST (100 px ≥ the 96 px swipe) | P1 |
| LV10·C6(b) | never past the edge | after C5(a): `press` (195, 305), `slide` (-300, 305), `lift` | `translate(-195px, 0px) scale(2)` (`(2 − 1) · 390 / 2`) | P1, PH11 |
| LV10·C6(c) | no hold while zoomed | Preview of 201 zoomed ×2; `press` (195, 305), `advance(300)`, `lift` | no "TEMPLATE" pill; `vibrate(10)` not called | P1 |
| LV10·C6(d) | double-tap | Preview of 202: two taps at (100, 305) 150 ms apart; then two taps anywhere | after the first pair: `translate(190px, 0px) scale(3)`; after the second (and `advance(200)`): `translate(0px, 0px) scale(1)` | P1 |
| LV10·C7(a) | second finger ends a swipe | Preview of 202: `press` (195, 305) id 1, `slide` (165, 305); `press` (300, 305) id 2; `slide` id 2 to (360, 305); release both | after the second press: no `swipe-hint`, the photo's `transform` has no translate; after the slide: `scale(1.4444…)` (195/135); no POST | P2, PH11 |
| LV10·C7(b) | cancels | Preview of 202 in a spread (fingers down at ×2): `cancel` id 1; separately a window `blur` | the zoom stays `scale(2)` (settled, within 1–6); a later `lift` of id 2 sends nothing; no POST | PH11, design §Reliability |
| LV10·C7(c) | another photo | Preview of 202 zoomed ×2; tap "C1" in `preview-strip` | the Preview shows 203 with `scale(1)` | P1 |
| LV10·C8(a) | composed Preview | fixture K45; Preview of 201 | `preview-viewport` is 390 × 487.5 (ratio 0.8); `composed-frame`'s style has `width: 187.4634%` and the rest of `composedLayout(201's values, frame).frame`, and `transform: rotate(0deg)` | P3, 10D, MC-27 |
| LV10·C8(b) | composed tile | fixture K45; the dock's tile of 201 | contains a `composed-frame` with the same frame style | P3 |
| LV10·C8(c) | not composed | fixture K45; the Preview and the tile of 202 (`composition` null) | no `composed-frame`; the picture drawn as LV9 draws it | P3 |

### Delegated choices
The settle animation's curve (≤ 200 ms) · whether the double-tap's 3× also animates · the composed tile's crop mark (10D
shows one on the Mac; optional here).

### Notes
- **Hazard (jsdom):** two fingers are two `pointerId`s on the same element; `setPointerCapture` is stubbed. A real
  browser sends `pointercancel` when it takes the touch (iOS system gestures): that path is C7(b).
- **Hazard:** `getBoundingClientRect` includes CSS transforms. The pinch maths takes the viewport's box at the gesture's
  start (as LV8 took homes at lift), never the zoomed layer's live rectangle.
- LV8's swipe, hold and overlay rows keep passing at 1× (the state machine only adds states).

### Unit test commands
`cd apps/desktop && npx vitest run src/phone` · `npm run typecheck`

## Unit 3 — The reframe editor (LV-D21 (6); PH12; 10D MC-27)

**Unit id:** `LV10·U3` · **Rows:** LV10·C9–C13 · **Changes app files:** yes (TypeScript, guide)

### Goal
New `components/ReframeEditor.tsx` over the Preview, opened by **Reframe** in the photo drawer (LV9) — enabled when the
photo's image has `composition`, else disabled with the line "Waiting for Capture One's full-frame preview". The editor
opens with the stored values, or for `state: "needs_review"` with `refit(values, {kind: "reframe"}, frame)` (as the
Mac). Layout: a top bar **Cancel** (left) · **Done** (right); the stage (`reframe-stage`) with the frame box
(`frameBox(stage, rect ratio)`, fixed) and the photo under it (`transformCss(viewTransform(values, frame, stage))`, turned
and flipped as `composedLayout`'s `imageTransform`), the area outside the frame dimmed; a split frame's middle line; the
**straighten** control (`<input type="range">` `aria-label="Straighten"`, −45…45, step 0.1, readout "2.5°"); a tool row:
**Normal | Split**; **Shape** (Original, 1:1, 4:5, 3:2, 2:3, 16:9, Free) with **Turn** beside it; **Rotate and flip**
(Rotate 90° left, Rotate 90° right, Flip horizontal; closes after a pick, 10D); **Guides** (Off, Thirds, Grid, Golden
ratio, Diagonals); **Reset**. Each control applies the Mac's `refit(values, change, frame)`. Gestures on the stage (PH12):
one finger pans (`panWithin`), two fingers pan + zoom about their midpoint + straighten (`twoFinger` → `rotateWithin`,
`zoomWithin`, `panWithin`, in that order, from the values at the gesture's start); every candidate is valid (P5). The
grid (`STRAIGHTEN_GRID_PX` squares over the stage) shows only while a two-finger twist is past the dead zone or the
straighten control is held. Guides: `overlayLines(kind, frame box, split)` inside the frame, remembered in
`localStorage["psm.phone.reframeGuides"]` (default `thirds`). **Reset** sets the values to `resetValues(frame,
default_shape)`. **Cancel** closes with no request. Done's request is U4. While open, nothing under it takes a touch; the
editor closes if its photo leaves the session or its `composition` becomes null.

### Read first
The Mac's `features/compose/{geometry.ts, refit.ts, overlays.ts, composed.ts, machine.ts (entryValues, settle, the
straighten and pan rules), ComposeFrame.tsx (the grid pattern), CompositionSection.tsx (labels)}`, `components/
{PhotoDrawer.tsx, Preview.tsx}`, `pinch.ts` (U2), master plan **PH12**.

### Files
New: `apps/desktop/src/phone/reframe.ts`, `components/ReframeEditor.tsx`, `__tests__/lv10_reframe.test.tsx`. Edited:
`components/PhotoDrawer.tsx` (Reframe), `PhoneApp.tsx`, `state.ts` (texts), `phone.css`, `__tests__/{helpers.ts (the
editor stage in the layout stub), lv10_core.test.ts}`. Guide: `docs/guide/01_product.md` (Reframe on the phone),
`docs/guide/02_architecture.md` (the phone editor reuses the Mac's composition geometry).

### Core mechanisms
`reframe.ts` (PH12): `panWithin(values, frame, delta) -> values` (delta in frame fractions; the furthest valid point along
the move, bisection 16 steps) · `zoomWithin(values, frame, anchor, factor) -> values` (`zoomAbout`, then the largest valid
factor between 1 and it, bisection 16 steps) · `rotateWithin(values, frame, degrees) -> values` (= `refit(values, {kind:
"straighten", degrees}, frame)`) · `straightenFromTwist(twist) -> number` (0 while |twist| ≤ 3°, else `twist − 3·sign`,
rounded to 0.1°) · `resetValues(frame, defaultShape) -> values` (normal, the default shape, named, no turn, no flip, 0°,
`defaultRect(normal, shape, named, turnedFrame(frame, 0))`). "Valid" = the Mac's `validate(values, frame) === null`.

### Ordered tasks
1. `reframe.ts` and its unit tests.
2. Reframe in the drawer; the editor's layout, picture and frame; Cancel.
3. Gestures (with `pinch.ts`'s tracker) and the constraints; the grid.
4. Controls, guides, Reset.
5. Tests for the rows; guide.

### Criteria (5 criteria, 18 rows)
Boundary: `PhoneApp` with fixture K; the Preview of 201 open, "Photo details", "Reframe"; gestures on `reframe-stage`
(Shared: 390 × 520, frame box left 24 top 89, 342 × 342, `s = 342/1707`). "Values" = what the editor shows (the picture's
`transform` and the frame box), checked against the named function's output; rect numbers within 1e-6.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| LV10·C9(a) | opens like the Mac | "Reframe" | `reframe-editor` open; `reframe-frame` at left 24, top 89, 342 × 342; `reframe-picture`'s `transform` = `transformCss(viewTransform(K's values, {w: 2560, h: 1707}, {left: 0, top: 0, width: 390, height: 520}))`; guides = Thirds (4 lines) | LV-D21 (6), 10D, P4 |
| LV10·C9(b) | waits for the preview | the Preview of 202 (`composition` null), "Photo details" | "Reframe" is disabled and the drawer reads "Waiting for Capture One's full-frame preview" | P3 |
| LV10·C9(c) | needs review | fixture K with 201's `state` "needs_review" | opens with `refit(K's values, {kind: "reframe"}, frame)` | MC-27 §Revisions |
| LV10·C10(a) | one finger | `press` (195, 260), `slide` (229.2, 260), `lift` | rect.x = 0.1666016 − 0.0666797 = 0.0999219; y, w, h unchanged | P4 |
| LV10·C10(b) | spread | `press` (145, 260) id 1, `press` (245, 260) id 2; `slide` id 1 → (95, 260), id 2 → (295, 260); release | rect = `zoomAbout(K's rect, {x: 0.5, y: 0.5}, 2, {w: 2560, h: 1707})` = `{x: 0.3333008, y: 0.25, w: 0.3333984, h: 0.5}` | P4 |
| LV10·C10(c) | twist | two fingers at (145, 260) / (245, 260) turned 10° clockwise about (195, 260); release | during the turn past 3°: `reframe-grid` present; after: straighten 7.0 and rect = `refit(K's values, {kind: "straighten", degrees: 7}, frame).rect`; no `reframe-grid` | P4, P6 |
| LV10·C10(d) | small twist | the same turned 2° | straighten 0; rect unchanged; `reframe-grid` never present | P4 |
| LV10·C11(a) | stops at the edge | `press` (195, 260), `slide` (495, 260), `lift` | rect.x = 0 (± 1e-6); w, h unchanged | P5, MC-27 |
| LV10·C11(b) | stops with straighten | after the straighten control set to 5: `press` (195, 260), `slide` (545, 610), `lift` | `validate(values, frame)` = null; the rect moved a further 1 % of the same move (in frame fractions) makes `validate` non-null (it stopped on the boundary, not short of it) | P5 |
| LV10·C11(c) | zoom out stops | after C10(b): two fingers from (95, 260) / (295, 260) to (170, 260) / (220, 260); release | rect = K's rect (the largest valid, centre kept) | P5 |
| LV10·C11(d) | cancelled mid-move | `press` (195, 260), `slide` (229.2, 260), `cancel` | the values stay C10(a)'s (not reverted); no request | PH12, design §Reliability |
| LV10·C12(a) | Split | "Split" | values = `refit(K's values, {kind: "kind", to: "split"}, frame)`; the frame box is 2:1 with a middle line | 10D Split, MC-27 |
| LV10·C12(b) | shape and turn | "Shape" → "4:5"; then "Turn" | values = `refit(K's values, {kind: "shape", shape: "4:5"}, frame)`, then `refit(that, {kind: "orientation", orientation: "turned"}, frame)` | MC-27 |
| LV10·C12(c) | rotate and flip | "Rotate and flip" → "Rotate 90° right"; again → "Flip horizontal" | `quarter_turns` 1 = `refit(…, {kind: "turn", direction: "right"}, …)`; then `flip` "horizontal" = `refit(…, {kind: "flip"}, …)`; the menu closed after each pick | 10D (rotate dropdown) |
| LV10·C12(d) | straighten control | `pointerdown` on "Straighten"; change to 2.5; `pointerup` | while held: `reframe-grid` present, readout "2.5°"; values = `refit(K's values, {kind: "straighten", degrees: 2.5}, frame)`; after: no grid | 10D, P6 |
| LV10·C12(e) | guides | "Guides" → "Golden ratio"; Cancel; open Reframe again | lines = `overlayLines("golden", frame box, false)` (4 lines); after reopening still Golden ratio; `localStorage["psm.phone.reframeGuides"]` = "golden" | 10D Overlay, P6 |
| LV10·C13(a) | Reset | after C10(b): "Reset" | values = `resetValues({w: 2560, h: 1707}, "1:1")` (= K's values) | P7, 10D |
| LV10·C13(b) | Cancel | after C10(b): "Cancel"; open Reframe again | no POST; the editor reopens with K's values | P7 |

### Delegated choices
The editor's look within the tokens (the dim outside the frame, the controls' icons and placement, the menus as sheets or
popovers) · haptics (the design's `select` on a control, `overlayTick` on each whole degree of straighten) · the
straighten control's look (a ruler or a plain slider; it stays an `<input type="range">` for its value and keyboard) ·
whether one finger on the frame's edge does anything (no handles, P8).

### Notes
- **Hazard:** the constraints run on every move, never at release only: a picture that leaves empty corners on screen and
  jumps back at release is the defect P5 forbids.
- **Hazard:** keep the composition maths in the Mac's modules; the phone adds only `reframe.ts`'s bounded moves. A copy
  of `refit` or `validate` in the phone would drift from the Mac (`composition_cases.json` proves only the Mac's copy).
- The Mac's editor is untouched; the phone never loads the master (10D performance rule: gestures move a CSS transform
  over the preview, never wait for Rust).

### Unit test commands
`cd apps/desktop && npx vitest run src/phone` · `npm run typecheck`

## Unit 4 — Saving from the editor; guides on live (LV-D21 (6), (7); PH4b, PH12)

**Unit id:** `LV10·U4` · **Rows:** LV10·C14–C16 · **Changes app files:** yes (TypeScript, guide)

### Goal
- **Done:** values equal to those the editor opened with → close, no request. After Reset with nothing changed since →
  `POST /photos/<id>/composition/reset {session_id, revision}`; otherwise `POST /photos/<id>/composition {session_id,
  revision, values}`, `revision` = the image's `revision` when the editor opened. Done is disabled while its request is
  out (one request per Done). 200 → the editor closes, toast "Framing saved", a session re-read (the Mac's `session`
  message also comes, PH8; the composed display shows the new framing). Refusals: `composition_changed` → the editor
  closes, toast "Framing changed on the Mac. Open Reframe again."; `composition_invalid` → the editor stays with its
  values, toast "That framing could not be saved."; `session_changed` → the editor closes, `refused` (toast "Session
  changed on the Mac", re-read); anything else → stays, "That did not work. Try again.".
- **Guides on live (7):** the Item drawer (the live screen's ⋯ menu) gains a **Guides** row: buttons Off, Thirds, Grid,
  Golden ratio, Diagonals (`aria-pressed` on the chosen one); stored in `localStorage["psm.phone.liveGuides"]` (one of
  `OVERLAY_ORDER`; default `off`). On the live screen's viewport an SVG (`live-guides`, `pointer-events: none`) draws
  `overlayLines(kind, {left: 0, top: 0, width, height} of the viewport, split of the current shot)`, over the live picture
  and the template alike; it turns with the viewport for a long shot.

### Read first
`components/{ReframeEditor.tsx, ItemDrawer.tsx, Stage.tsx}`, `PhoneApp.tsx` (`refused`, `readSession`, the Item drawer's
props), `api.ts`, `state.ts` (`TEXT`), master plan §6 (texts v1.3), **PH12**.

### Files
New: `__tests__/lv10_save_and_guides.test.tsx`. Edited: `api.ts` (`saveComposition`, `resetComposition`),
`components/{ReframeEditor.tsx, ItemDrawer.tsx, Stage.tsx}`, `PhoneApp.tsx`, `state.ts`, `phone.css`. Guide:
`docs/guide/01_product.md` (Done/Cancel and what the Mac does; guides on live).

### Core mechanisms
None new.

### Ordered tasks
1. `saveComposition`, `resetComposition`; Done's three cases; refusals.
2. The Guides row, its storage, the live SVG.
3. Tests for the rows; guide.

### Criteria (3 criteria, 13 rows)
Boundary: `PhoneApp` with fixture K; the editor opened as in U3. C16 on the live screen (`viewport`: 390 × 390 for a 1:1
shot, LV3).
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| LV10·C14(a) | Done saves | C10(a)'s pan; "Done" | POSTs = `[["/photos/201/composition", {session_id: 7, revision: 1, values: {kind: "normal", shape: "1:1", orientation: "named", quarter_turns: 0, flip: "none", straighten: 0, rect: {x: 0.0999219, y: 0, w: 0.6667969, h: 1}}}]]` (rect within 1e-6); the editor closes; toast "Framing saved"; a `GET /session` follows | LV-D21 (6), PH4b |
| LV10·C14(b) | unchanged | "Done" without a change | no POST; the editor closes; no toast | P7 |
| LV10·C14(c) | after Reset | fixture K with 201's shape "4:5" (K45); "Reset"; "Done" | POSTs = `[["/photos/201/composition/reset", {session_id: 7, revision: 1}]]` | P7, PH4b |
| LV10·C14(d) | one request | `mac.hold()`; C10(a)'s pan; "Done" twice | one POST; Done disabled until the answer | PH12 |
| LV10·C15(a) | changed on the Mac | the save answers 409 `composition_changed` | the editor closes; toast "Framing changed on the Mac. Open Reframe again." | P7 |
| LV10·C15(b) | refused framing | the save answers 400 `composition_invalid` | the editor stays open with C10(a)'s values; toast "That framing could not be saved." | MC-27 §Validation |
| LV10·C15(c) | session changed | the save answers 409 `session_changed` | the editor closes; toast "Session changed on the Mac"; a `GET /session` follows | PH4 guard |
| LV10·C15(d) | the photo leaves | editor open; `macChanges` without 201 | the editor and the Preview close; no POST | PH12 |
| LV10·C16(a) | the Guides row | the live screen; "Item" (menu) | the drawer has buttons "Off", "Thirds", "Grid", "Golden ratio", "Diagonals"; "Off" `aria-pressed` true; no `live-guides` lines on the viewport | LV-D21 (7), P10 |
| LV10·C16(b) | Thirds | "Thirds" | `live-guides` has 4 lines: vertical at x 130 and 260, horizontal at y 130 and 260 (390 × 390) | LV-D21 (7), 10D Overlay |
| LV10·C16(c) | remembered | after (b): unmount, `startPhone` again | still Thirds (4 lines); `localStorage["psm.phone.liveGuides"]` = "thirds" | P10 |
| LV10·C16(d) | split shot | a current shot with `split` true (ratio 2, viewport turned) and Thirds | 8 lines (`overlayLines("thirds", box, true)`), inside the viewport | 10D Overlay, LV-D16 |
| LV10·C16(e) | UI only, no touch | Thirds on; lock live (LV3) and tap (200, 200) on `viewport`, `advance(260)` (`TAP_FOCUS_MS`) | one POST `/focus` (the tap reaches the stage); `live-guides` has `pointer-events: none` | H3, LV-D14 kept |

### Delegated choices
The Guides row's look in the drawer (a segmented row or a list) · the guide lines' style on live (thin, light with a dark
edge so they read over any picture, as 10D's middle line) · whether "Framing saved" also vibrates (`HAPTIC.select`).

### Notes
- The editor's guides (`psm.phone.reframeGuides`) and the live guides (`psm.phone.liveGuides`) are separate choices
  (P10): framing on live and checking a crop are different moments.
- The Mac shows the phone's change at once (its session view re-reads on `SessionChanged`); ⌘Z there undoes it (one step
  "Compose NN").

### Unit test commands
`cd apps/desktop && npx vitest run src/phone` · `npm run typecheck`

## Guide files (flow §7A)
U1: `docs/guide/02_architecture.md` (PH4b routes, `set_at_revision`, PH6c), `docs/guide/04_change_impact.md` (the phone
imports the Mac's composition geometry). U2: `docs/guide/01_product.md` (pinch zoom; photos shown as composed). U3:
`docs/guide/01_product.md` (Reframe on the phone), `docs/guide/02_architecture.md` (`reframe.ts`, `pinch.ts`). U4:
`docs/guide/01_product.md` (Done/Cancel, guides on live).

## Closing gate (once, after the last unit)
`cd apps/desktop && npm run build` (first) · `cargo test --workspace` · `cargo clippy --workspace --all-targets -- -D
warnings` · `cargo fmt --all -- --check` · `tools/sidecar/build.sh --stub --test` (rebuilds only the stub the gate needs;
nothing else in `apps/desktop/src-tauri/binaries/` is rebuilt) · `cd apps/desktop && npm run typecheck` · `npm test`.
One command at a time.

## Review mode
The reviewer reads the units in order and runs, once: `cargo test -p psm-testkit --test suite lv10_phone_composition::` ·
`cd apps/desktop && npx vitest run src/phone` · `npm run typecheck` · `npm run build`. Only B1–B4 block. Also check, by
reading: the revision rule and validation live in `CompositionService` (H8), the routes only parse, guard and call; the
phone imports the Mac's `geometry`/`refit`/`overlays`/`composed` and copies none of them; nothing in `src/phone` imports
`@/services/bindings` or `@tauri-apps/*` (`grep`; and the built `dist/assets/phone-*.js` contains no `__TAURI`); every
gesture uses pointer capture and an explicit state with named cancels (`pointercancel`, hidden, blur, unmount); the
constraints run on every move; the grid shows only while straightening; no image is processed anywhere (H3). Notes go to
`docs/implementation/backlog_live/LV10_zoom_reframe_guides.md`.

## Manual try-out (the narrative is built from this)
On this Mac with the camera connected and the phone paired (the app built from this branch), a templated session open,
some photos taken and their previews ready:
1. Open a photo's preview and spread two fingers: it zooms; move it with one finger to check sharpness; double-tap to go
   back. Swiping does nothing while zoomed.
2. Tap ⋯ → Reframe. Move the photo with one finger, spread to zoom, twist two fingers to straighten: a fine grid shows
   while you turn, the frame never shows empty corners. Pick Guides → Golden ratio. Try Split, a shape, Rotate 90°.
3. Tap Cancel: nothing changed. Reframe again, change it, tap Done: "Framing saved"; the preview and the strip show the new
   framing; on the Mac the photo shows it too, and ⌘Z there undoes it.
4. On the live screen open the menu (≡), Guides → Thirds: the thirds lines show over live and over the template; close the
   app and open it again: still Thirds.

## Review log
