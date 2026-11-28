---
phase: LV3_phone_app
title: The phone app (Shoot Companion) — scan the QR code and the phone becomes the shooting remote; hold for live, slide up to lock, tap to focus, slide sideways for the template overlay; capture with flash and haptics; the shot dock in shooting order; preview, retake, delete and move images; the item drawer; plain HTTP, installable on iPhone, Full screen on Android
difficulty: high
projection: no
depends_on: [LV2_current_shot_and_phone_server]
implementer: fable
blocked_on: none (owner card 1 answered 2026-11-26: Article number = the item's barcode, supplied by LV2's `article_number`)
---
# LV3 — The phone app

**Tracker:** APPROVED (2026-11-26, opus) — review r1: APPROVED_WITH_NOTES. All 85 rows hold at their boundaries; Review-mode checks green (build; suite stems `lv2_phone_ops`/`lv2_phone_ws`/`lv3_server_carry` 29 passed; phone UI 94 passed; typecheck); H1–H8 hold; no secure-context API, absolute URL or token leak under `src/phone/`; design thresholds match; no blocking item; six notes to the live backlog.

(This line is the milestone's tracker row. Sessions change only this line of this file. The gate for `depends_on` is
the Tracker line of the plan named there.)

## Goal
Book `10J` §The phone design, LV-D7, LV-D15…LV-D18, cards LV3, LV5…LV9, by `master_plan_live.md` §5 **PH5–PH7** and
§6, with the owner's design as the authority for look, gestures, thresholds, timings and haptics:
`claude_design/live_preview/design_handoff_shoot_companion/README.md` and its `prototype/Shoot Companion.dc.html`
(local, git-ignored, **high fidelity**: read and match; never copy design files into the repo). The book wins where they
differ: after the last template shot captures are **Condition** (card LV7; the Free shot tile adds Condition images in a
templated session), the phone's Move **swaps** (card LV8), × on the AF chip **recentres** the camera's focus point (card
LV9), split halves are **Left/Right** (LV-D16), the dock lists shots in **shooting order** with their final numbers
(card LV5), capture feedback reconciles with the camera's confirmation (C3).
(U1) Shell, pairing, socket and live drawing. (U2) Main screen gestures and capture. (U3) The shot dock. (U4) Preview,
Move drawer, Item drawer, toasts.
**Not here:** the Mac's live screen (card LV10), HTTPS, a service worker or wake lock (card LV3: no secure-context
API), item property inputs ("Fill item properties" opens an empty page, LV-D17), session completion/opening (LV-D18).

## How to work this plan (flow.md §10, master_plan_live.md §3)
One runner phase of 5 units (U0 first, then U1–U4), in order; after each, its **Unit test commands**, then a `CHECKPOINT (not approved):`
commit of that unit's paths. Row ids `LV3·C<n>(<row>)`; tests named after their row. Open details: the design README
and prototype first, then the smallest option consistent with 10J; recorded under **Assumptions**. Reviewer notes:
`docs/implementation/backlog_live/LV3_phone_app.md`.

| Unit | What it builds | Criteria | Rows | Changes app files |
|---|---|---|---|---|
| U0 | Server carry-overs from LV2's review (added by the orchestrator 2026-11-26) | 4 | 6 | yes (Rust) |
| U1 | Shell: pairing, connecting, socket, demand, frames, reconnect, Full screen, camera status | 7 | 18 | yes (TypeScript) |
| U2 | Main screen: hold, lock, overlay, focus, AF chip, geometry, viewport, capture, Next | 7 | 29 | yes (TypeScript) |
| U3 | Shot dock: order, tiles, magnification, select, scroll, drag-up | 5 | 13 | yes (TypeScript) |
| U4 | Preview, Move drawer, Item drawer, toasts | 6 | 19 | yes (TypeScript, guide) |
| **Total** | | **29** | **85** | |

## Read first (whole milestone)
`docs/process/flow.md` (§4, §4A, §6, §7A, §10) · `master_plan_live.md` (**§5 PH4–PH7**, §6, §4.5) · book `10J` (§The
phone design, LV-D15…LV-D18, cards LV3, LV5…LV10) · `docs/intention/evidence/live_preview_lab.md` (iPhone install,
Android Full screen, no wake lock on HTTP) · the design README (all) and the prototype's logic class (thresholds) ·
the LV2 plan (routes, wire, the placeholder `apps/desktop/src/phone/`) · `docs/intention/evidence/design_handoff.md`
(the desktop's rules; the phone follows its own design) · the lab page `~/Desktop/Developer/Camera-Connection-Tests/
camlab/pwa/static/index.html` (a working WebSocket JPEG drawer on iPhone and Android; read-only).

## Points recorded by planning (each is one sentence in the narrative's "Choices made for you")
- **The Free shot tile adds condition images** in a templated session (your answer LV7); a photo becomes an extra main
  image only by Move.
- **Next shot** jumps to the next empty template shot after the current one in shooting order, and to Free shot after
  the last.
- **When you switch to Live the phone shows the template until the first fresh camera frame arrives** (a fraction of a
  second), so it never shows an old picture as live.
- **Haptics work on Android only**; iPhone gets the flash and animations (the design's note).

## Shared for all units
- Code under `apps/desktop/src/phone/` (registry §4.5): `main.tsx`, `PhoneApp.tsx`, `api.ts` (HTTP: PH4 routes, Bearer
  token, error codes), `socket.ts` (WebSocket: PH5, PH7 demand and reconnect, frame header decode), `state.ts`
  (reducer: the design README §State Management adapted to the server's `PhoneSessionView`), `geometry.ts`, `gestures.ts`,
  `components/…`, `phone.css` (the design tokens as CSS variables). `phone.html` gets the iPhone tags
  (`apple-mobile-web-app-capable`, status bar style, `viewport-fit=cover`) and the manifest link is set by `main.tsx`
  (PH7).
- **Tests** at the component boundary: render `PhoneApp` (or the named component) with `api.ts` and `socket.ts` mocked
  (outcome = what shows, the exact request bodies, the demand messages sent, `navigator.vibrate` calls,
  `requestFullscreen` calls). A helper `apps/desktop/src/phone/__tests__/helpers.ts` (new, used by every phone test)
  stubs what jsdom lacks: `setPointerCapture`/`releasePointerCapture`, `matchMedia`, `navigator.vibrate`,
  `document.fullscreenEnabled`/`requestFullscreen`, `createImageBitmap` and the canvas drawer (a spy). Fake timers for
  every time rule.
- The live frame is drawn on a `<canvas>` from the binary JPEG (`createImageBitmap(new Blob([jpeg]))`), the template
  is an `<img>` of `/api/phone/v1/containers/<id>/example?pair=` with `filter: grayscale(1)`; overlays and black and
  white are drawing only (H3).
- Exact texts: `master_plan_live.md` §6 (phone texts, split labels) and the design README §2 and §7.

## Unit 0 — Server carry-overs from LV2's review (orchestrator, 2026-11-26)

**Unit id:** `LV3·U0` · **Rows:** LV3·C26–C29 · **Changes app files:** yes (Rust, small)

### Goal
Four notes the LV2 reviewer routed to this milestone (`docs/implementation/backlog_live/LV2_current_shot_and_phone_server.md`,
destination LV3), done first because the phone's Delete (U4) and dock (U3) depend on them. Server side only; the
handlers stay thin (H8): events are emitted in core/services, not in route handlers.

### Read first
The LV2 backlog notes above (each names the code path) · `master_plan_live.md` §5 PH4, PH5 · the LV2 plan rows
LV2·C21(e) and the `session` WebSocket message · `crates/integrations/src/phone/{routes.rs, ws.rs, wire.rs}`,
`crates/integrations/src/session_delete.rs`.

### Files
Edited (expected): `crates/core/src/services/review.rs` or a thin core wrapper used by the phone delete route,
`crates/integrations/src/phone/{routes.rs, ws.rs, wire.rs}`, `crates/integrations/src/session_delete.rs`, the code
path that rebuilds the App on a settings save (where the phone server must observe the new event hub), tests in
`crates/testkit/tests/suite/lv2_phone_ops.rs` and `lv2_phone_ws.rs` (or a new `lv3_server_carry.rs`).

### Ordered tasks
1. Phone Delete emits the same change events as a Mac deny/delete (`SessionChanged` + `PhotoChanged`), from core.
2. Deleting the active session on the Mac announces it (`SessionsService` built `with_events`), so phones get
   `{type: "session", session_id: null}`.
3. After a settings save rebuilds the App, open phone sockets keep receiving `session` and camera messages (observe the
   new event hub at rebuild; no phone reconnect needed).
4. Error bodies: a generic `message` for 500 `internal` and for a missing file (404); no internal text or local path.
5. Tests for the rows.

### Criteria (4 criteria, 6 rows)
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| LV3·C26(a) | `POST /photos/<id>/delete` | `{session_id}` on a Condition photo, a second phone socket open | 200; the event hub received `SessionChanged` and `PhotoChanged` for that session/photo; the second socket receives a `session` message | PH4, PH5, LV2 backlog note 1 |
| LV3·C26(b) | same | the same delete | the history step is recorded once (undo restores the photo) | PH4, §6 |
| LV3·C27 | desktop session delete | the active session deleted; a phone socket open | the socket receives `{type: "session", session_id: null}`; a following `GET /session` answers `session: null` | PH5, LV-D18, LV2 backlog note 2 |
| LV3·C28 | settings save that rebuilds the App | a phone socket open; then a photo arrives in the active session | the socket receives a `session` message without reconnecting | PH5, LV2 backlog note 3 |
| LV3·C29(a) | any route | an internal error (500) | body `message` is a fixed generic text; no error string from the store | PH4, LV2 backlog note 5 |
| LV3·C29(b) | `GET` a photo preview | the preview file missing on disk | 404; body carries no local path | PH4, LV2 backlog note 5 |

### Delegated choices
Where the delete wrapper lives (core service method vs. a small wrapper); how the server observes the rebuilt hub
(observe at rebuild, or re-subscribe on the next event-hub swap); the generic texts.

### Unit test commands
`cargo test -p psm-testkit --test suite -- lv2_phone_ops:: lv2_phone_ws::` (and the new stem if one is added) ·
`cargo check -p psm-integrations`. **Run `cd apps/desktop && npm run build` once before any cargo command** (psm-desktop
needs `apps/desktop/dist/`).

## Unit 1 — Shell: pairing, connecting, socket, demand, frames, reconnect, Full screen, camera status

**Unit id:** `LV3·U1` · **Rows:** LV3·C1–C7 · **Changes app files:** yes (TypeScript)

### Goal
PH7 (token, manifest link, demand, frame acceptance, reconnect, Full screen), the connecting overlay (design §1), the
camera status texts (10J §Status).

### Read first
Master plan §5 PH5, PH7, §6 · design README §1, §2 (Hint chip), §Reliability requirements · the LV2 placeholder
`apps/desktop/src/phone/{main.tsx, PhoneApp.tsx, api.ts}`.

### Files
New: `apps/desktop/src/phone/{socket.ts, state.ts, components/ConnectingOverlay.tsx, components/LiveCanvas.tsx,
components/FullscreenButton.tsx, phone.css}`, `apps/desktop/src/phone/__tests__/{helpers.ts, lv3_shell.test.tsx,
lv3_socket.test.ts}`. Edited: `apps/desktop/src/phone/{main.tsx, PhoneApp.tsx, api.ts}`, `apps/desktop/phone.html`,
`apps/desktop/package.json` and `package-lock.json` (`@fontsource/ibm-plex-mono`).

### Core mechanisms
`socket.ts`: `decodeFrame(buffer) -> {seq, gen, width, height, capturedAtMs, jpeg}` (PH5 header) and
`reconnectDelay(attempt)` (500, 1000, 2000, 4000, 4000, …).

### Ordered tasks
1. Token and manifest (PH7); the pairing screens (§6 texts).
2. `socket.ts`: connect `ws(s)://<location.host>/api/phone/v1/ws?pair=`, `hello`, messages into the state, demand with
   `gen`, frame acceptance (gen and seq), reconnect and visibility (PH7).
3. Connecting overlay; `LiveCanvas` (drawn only on accepted frames; hidden after a switch until the first frame of the
   new gen); camera status texts in the hint chip position; Full screen button.
4. Tests for the rows.

### Criteria (7 criteria, 18 rows)
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| LV3·C1(a) | start | location `/phone/?pair=T` | `localStorage["psm.phone.token"]` = T; `history.replaceState` to `/phone/`; a `<link rel="manifest">` with `href` `/phone/manifest.webmanifest?pair=T`; `hello` called with `Authorization: Bearer T` | PH7, LV-D7 |
| LV3·C1(b) | start | no stored token, no query | the text "Scan the QR code in Session Manager → Settings → Phone to connect this phone."; no request made | PH7, §6 |
| LV3·C1(c) | start | stored token; `hello` answers 401 | the token is removed; the text "This phone was removed in Session Manager. Scan a new QR code to connect it again." | PH1, PH7 |
| LV3·C2(a) | connecting overlay | `hello` and `session` not yet answered | "Connecting to session" and the indeterminate bar show; a press on the stage sends no demand | `[rule]` design §1 |
| LV3·C2(b) | same | both answered | the overlay's opacity goes to 0 and it leaves after 400 ms; the stage takes presses | `[rule]` design §1 |
| LV3·C3(a) | demand | connected; press and hold the stage, then release | `{type: "demand", gen: 1, live: true, visible: true}`, then `{type: "demand", gen: 2, live: false, visible: true}` | PH7, MC-LV2, M-LV2 |
| LV3·C3(b) | same | `visibilitychange` to hidden, then to visible | `{…, live: false, visible: false}` with a new gen, then `{…, visible: true}` with a new gen | PH7, MC-LV2 |
| LV3·C4(a) | frames | holding (gen 1); a binary frame with gen 0 | not drawn (the drawer spy not called) | PH7, MC-LV1 |
| LV3·C4(b) | same | frames gen 1 seq 10, then seq 9, then seq 10 | only the first drawn | PH7, MC-LV1, M-LV1 |
| LV3·C4(c) | same | press and hold; no frame yet; then a frame of the new gen | before it: the live layer hidden and the template at opacity 1; after it: the live layer shown and the template at the overlay value (35 %) | PH7, M-LV2 |
| LV3·C5(a) | reconnect | the socket closes while visible | new connections at +500, +1500, +3500, +7500, +11 500 ms | PH7, `decodeFrame`/`reconnectDelay` |
| LV3·C5(b) | same | closed while hidden; then visible | no attempt while hidden; one at once on visible; on open a demand with a new gen | PH7, 10J (reconnect after sleep) |
| LV3·C5(c) | `decodeFrame` (core mechanism) | the 39-byte message of LV2·C29(a) | `{seq, gen, width: 1024, height: 680, capturedAtMs, jpeg: 7 bytes}` equal to what was encoded | PH5 |
| LV3·C6(a) | Full screen | `fullscreenEnabled` true, no `fullscreenElement`, not standalone | a **Full screen** button; a tap calls `document.documentElement.requestFullscreen` once | PH7, LV-D7, card LV3 |
| LV3·C6(b) | same | `matchMedia('(display-mode: standalone)')` matches; or `fullscreenElement` set | no button | PH7 |
| LV3·C7(a) | camera status | `{type: "camera", status: "disconnected", …}`; hold the stage | "Camera not connected" | 10J §Status, §6 |
| LV3·C7(b) | same | `live: "unavailable"`, `reason: "not_supported"`; hold | "Live unavailable on this camera" | 10J §Status, §6 |
| LV3·C7(c) | same | `status: "busy"`; hold | "Camera busy" | 10J §Status, §6 |

### Delegated choices
Where the Full screen button sits (design-consistent; a small pill, not covering the stage's gestures) · the state
library (React reducer is enough) · font loading.

### Notes
- No service worker, no `navigator.wakeLock`, no `crypto.subtle`, no other secure-context API (card LV3).
- All URLs relative to `location`; the WebSocket scheme follows `location.protocol` (HTTPS later, card LV3).

### Unit test commands
`cd apps/desktop && npx vitest run src/phone` · `npm run typecheck`

## Unit 2 — Main screen: hold, lock, overlay, focus, AF chip, geometry, viewport, capture, Next

**Unit id:** `LV3·U2` · **Rows:** LV3·C8–C14 · **Changes app files:** yes (TypeScript)

### Goal
Design README §2 (layers, pills, reticle, AF chip, hint, flash, control bar) and §Interactions (main viewport gestures,
lock control, overlay bar, shutter), image-local coordinates when rotated, the tap → frame point mapping, capture
reconciliation, Next.

### Read first
Design README §Global layout, §2, §Main viewport gestures, §Lock control, §Overlay bar, §Shutter / capture, §Haptics ·
the prototype's logic class · master plan §5 PH4 (focus, capture, current), PH6 (`ratio`, `split`, `zoom`), §6 texts.

### Files
New: `apps/desktop/src/phone/{geometry.ts, gestures.ts, components/Stage.tsx, components/TemplateLayer.tsx,
components/LockControl.tsx, components/OverlayBar.tsx, components/ControlBar.tsx, components/Toast.tsx}`,
`apps/desktop/src/phone/__tests__/{lv3_main.test.tsx, lv3_geometry.test.ts}`. Edited: `apps/desktop/src/phone/
{PhoneApp.tsx, state.ts, phone.css}`.

### Core mechanisms
`geometry.ts`: `viewportBox(stage, ratio)`, `isRotated(ratio)` (> 1.2), `localPoint(client, rect, rotated)`,
`tapToFrame(local, viewport, frame, zoom)` (cover crop, then the zoom crop centred: the live picture and the template
are both drawn with `cover` and scaled by `1 / (1 − zoom/100)` about the centre, 10J "the live frame shows the area the
container will keep"), `ratioLabel(ratio, split)`. `gestures.ts`: the per-touch mode classifier (hold / lock / overlay,
decided once per touch), `overlayValue(start, dx, width)`, `isDoubleTap(a, b)`.

### Ordered tasks
1. Geometry and gesture mechanisms with their unit tests.
2. Stage layers (live canvas, template, split guide, reticle, mode pill, AF chip, overlay bar, lock control, hint,
   flash, exposure readout), rotation for long shots, the control bar.
3. Gestures wired to demand (U1), focus (`POST /focus`, `POST /focus/clear`), capture (`POST /capture`), Next
   (`POST /current`), with the design's haptics.
4. Capture reconciliation: flash and haptic at once; toast on `{type: "capture", state: "exposed"}` for that capture id
   once the session re-read shows where it landed; error texts.
5. Tests for the rows.

### Criteria (7 criteria, 29 rows)
Viewport 390 × 390 CSS px (shot ratio 1.0) unless stated; overlay 35 %; live frame 1024 × 680.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| LV3·C8(a) | stage gestures | not locked; pointer down, move dy −15, dx 2 | the lock control shows (progress 15/110) | `[rule]` design §Main viewport gestures |
| LV3·C8(b) | same | continue to dy −110 | locked: `vibrate([25,40,70])`; the pill "LIVE · LOCKED"; the last demand stays `live: true` after release | `[rule]` design; PH7 |
| LV3·C8(c) | same | instead release at dy −60 | not locked; the control fades; a demand `live: false` | `[rule]` design; PH7 |
| LV3·C8(d) | same | pointer down, move dx +20 (dy 3) | the overlay bar "TEMPLATE 42%" (round(35 + 20/390·130)); `vibrate(5)` exactly once (crossing 40) | `[rule]` design §Overlay bar |
| LV3·C8(e) | same | in Free shot, move dx +40 | the overlay unchanged; no overlay bar | `[rule]` design (not in free shot) |
| LV3·C9(a) | hold | not locked; pointer down | `vibrate(10)`; pill "LIVE"; demand `live: true` | `[rule]` design; PH7 |
| LV3·C9(b) | same | then `pointercancel` | pill "TEMPLATE"; demand `live: false` | design §Reliability; PH7 |
| LV3·C9(c) | same | holding; window `blur` | pill "TEMPLATE"; demand `live: false` | design §Reliability; PH7 |
| LV3·C10(a) | locked tap | locked; tap at the viewport centre | after 260 ms: `POST /focus {session_id, x: 0.5, y: 0.5}`; `vibrate(25)`; the reticle at the tap | LV-D14, HB6, design |
| LV3·C10(b) | same | two taps 200 ms apart, 20 px apart | no focus request; unlocked; `vibrate([20,30,20])`; a demand `live: false` | `[rule]` design (double tap) |
| LV3·C10(c) | same | not locked; a quick tap (no hold time) | no focus request | design (tap to focus only while locked) |
| LV3·C11(a) | AF chip | after C10(a), `{type: "focus", focus_id: <its id>, outcome: "focused"}` | the chip "AF LOCKED" with "×" | LV-D14, PH5 |
| LV3·C11(b) | same | tap "×" | `POST /focus/clear {session_id}`; `vibrate(10)`; chip and reticle gone | card LV9 |
| LV3·C11(c) | same | outcome `not_focused`; outcome `refused` (`focus_area_not_movable`) | toast "Could not focus there"; toast "Set the camera's Focus Area to a movable spot (e.g. Flexible Spot) to focus from the phone" | §6, LV-D14 |
| LV3·C12(a) | `tapToFrame` (core mechanism) | viewport 390 × 390, frame 1024 × 680, zoom 0, local (195, 195) | (0.5, 0.5) | HB6, PH7 |
| LV3·C12(b) | same | local (0, 195) | (0.16797, 0.5) ± 1e-4 | HB6 (cover crop) |
| LV3·C12(c) | same | zoom 20, local (0, 195) | (0.23438, 0.5) ± 1e-4 | 10J (presets' area), HB6 |
| LV3·C12(d) | `localPoint` + `tapToFrame` | rotated viewport, client rect {left 0, top 100, width 300, height 450}, frame 1024 × 680 scaled to 450 × 300, client (150, 325) | local (225, 150); frame point (0.5, 0.5) | design §Image viewport sizing |
| LV3·C13(a) | viewport | current shot ratio 1.0; then 1.5; then `null` with live frames 1024 × 680 | not rotated; rotated 90° with the menu, next and dock tiles rotated; rotated (aspect 1.506) | design §Image viewport sizing, 10J (landscape rotates) |
| LV3·C13(b) | split guide | a split current shot | a dashed vertical line at 50 % and the pills "Left · 1:1" at 25 %, "Right · 1:1" at 75 % | LV-D16, §6 |
| LV3·C13(c) | layers | template mode; holding (live drawn); Free shot | template opacity 1; 0.35; 0 | `[rule]` design §2 |
| LV3·C13(d) | exposure readout | `{type: "camera", exposure: {shutter: "1/125", aperture: "f/8", iso: "ISO 100"}}`; live showing | "1/125 · f/8 · ISO 100" | 10J §phone design (exposure readout), HB5 |
| LV3·C13(e) | Next | current = shot 01 (shoot order B=02, A=01, C=03 → the dock order 02, 01, 03), C empty; tap Next | `POST /current {session_id, container_id: <C>}`; on the last template shot the label reads "Free shot" and a tap sends `container_id: null`; disabled (opacity .3) in Free shot | 10J (current shot), design §2 Control bar |
| LV3·C14(a) | shutter | pointer down; pointer up | `vibrate(20)` and the inner disc at scale .86 on down; on up `POST /capture {session_id}`, the flash layer at opacity 1 then fading, `vibrate([80,40,120])` | design §Shutter, M-LV3 |
| LV3·C14(b) | same | pointer down, pointer leave | no capture request | design §2 Control bar |
| LV3·C14(c) | reconcile | capture `c1` answered; then `{type: "capture", capture_id: "c1", state: "exposed"}` and a session re-read showing the new photo in final 04 | toast "Shot 04 captured" | 10J (reconcile; C3), design §7 |
| LV3·C14(d) | same | the new photo bound to no shot (Condition) | toast "Condition image added" | card LV7, §6 |
| LV3·C14(e) | errors | `POST /capture` 409 `camera_not_connected`; `{state: "failed"}`; 409 `session_changed` | toast "Camera not connected"; "Not taken. Try again."; "Session changed on the Mac" and a session re-read | §6, PH4 |
| LV3·C14(f) | auto-advance display | after an exposure the server's `current` moves (session re-read) | the dock outline and the Next label follow the server's `current` (the phone never advances by itself) | MC-LV3 (one current shot, server-side) |

### Delegated choices
Component split · animation implementation (CSS transitions with the design's curves) · how the reticle's position is
kept in image-local coordinates when rotated.

### Notes
- The design's auto-advance (650 ms) is the server's (MC-LV3): the phone shows what `GET /session` says.
- Gesture mode is decided once per touch (design); hold the pointer capture on every gesture surface.

### Unit test commands
`cd apps/desktop && npx vitest run src/phone` · `npm run typecheck`

## Unit 3 — Shot dock: order, tiles, magnification, select, scroll, drag-up

**Unit id:** `LV3·U3` · **Rows:** LV3·C15–C19 · **Changes app files:** yes (TypeScript)

### Goal
Design README §3 and §Shot dock gestures, with the order of card LV5 (shooting order, final numbers) and card LV7 (the
Free shot tile adds Condition images).

### Read first
Design README §3, §Shot dock gestures, §Selecting a tile · master plan §5 PH6 (`shots`, `extras`, `condition`,
`current`).

### Files
New: `apps/desktop/src/phone/components/Dock.tsx`, `apps/desktop/src/phone/__tests__/lv3_dock.test.tsx`. Edited:
`apps/desktop/src/phone/{gestures.ts, state.ts, PhoneApp.tsx, phone.css}`.

### Core mechanisms
`gestures.ts`: `tileScale(fingerX, tileCenterX)` = `1 + 1.5 · max(0, 1 − |fingerX − tileCenterX| / 95)`.

### Ordered tasks
1. Tiles in order: template shots (shooting order, tag = final number), extras (tag = Main position, inverted), the
   Free shot tile, divider, condition (tags C1…); variants and the current outline.
2. Gestures: tap (< 200 ms, no movement) selects the nearest tile within 40 px; swipe (|dx| > 8 before 200 ms) scrolls;
   hold 200 ms magnifies; drag up past −50 px then release opens; tile centres snapshotted at pointer down.
3. Selecting: empty template → `POST /current`; a tile with an image → preview (U4); Free shot → `POST /current
   {container_id: null}`.
4. Tests for the rows.

### Criteria (5 criteria, 13 rows)
Session: shots B (final 02, filled, named "Back"), A (final 01, empty, named "Front"), C (final 03, empty), each ratio
1.0, in shooting order B, A, C; one extra (Main position 04); one condition image; current = A; item "Oak Lounge
Chair", SKU "BEY-123".
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| LV3·C15(a) | dock | the session above | tiles in order: "02" (photo, check badge), "01" (template, grey), "03" (template, grey), "04" (photo, inverted tag), Free shot (dashed "+"), divider, "C1" (inverted) | card LV5, card LV7, design §3 |
| LV3·C15(b) | same | — | the outline (2 px, offset 2) on tile "01" only | design §3, MC-LV3 |
| LV3·C15(c) | same | B is a split shot | its tile shows the 1.5 px white divider at 50 % | design §3, LV-D16 |
| LV3·C16(a) | `tileScale` (core mechanism) | finger at the centre; 47.5 px away; 95 px away; 120 px away | 2.5; 1.75; 1.0; 1.0 | design §3 |
| LV3·C16(b) | dock magnify labels | hold over "01" (name "Front"); over "04"; over Free shot; over "C1" | pills "01 · Front"; "04 · Extra image"; "Free shot · no template"; "Condition 01" | design §3 |
| LV3·C17(a) | select | tap "03" (empty template) | `POST /current {session_id, container_id: <C>}`; `vibrate(15)`; any focus marker cleared | card LV5, design §Selecting |
| LV3·C17(b) | same | tap "02" (filled) | the preview of B's photo opens; `vibrate(15)`; no request | design §Selecting |
| LV3·C17(c) | same | tap Free shot | `POST /current {session_id, container_id: null}` | card LV7 |
| LV3·C18(a) | swipe | pointer down, dx 12 within 150 ms | the strip's `scrollLeft` = start − 12; nothing selected | design §Shot dock gestures |
| LV3·C18(b) | hold | pointer down, still 200 ms; slide onto the next tile | `vibrate(12)` at 200 ms; `vibrate(6)` on the new tile | design §Shot dock gestures |
| LV3·C19(a) | drag up | hold on "02", drag to −60 px, release | `vibrate(18)` when crossing −50; the pill reads "Release to open" (inverted); on release the preview of B's photo opens | design §Shot dock gestures |
| LV3·C19(b) | same | hold, drag to −60, back to −30, release | nothing opens | design §Shot dock gestures |
| LV3·C19(c) | same | hold on "03" (empty template), drag up past −50, release | `POST /current {…, container_id: <C>}` (opening an empty template selects it) | design §Selecting |

### Delegated choices
Auto-scroll near the edges (design: 6 px per move within 36 px) · tile image loading.

### Unit test commands
`cd apps/desktop && npx vitest run src/phone` · `npm run typecheck`

## Unit 4 — Preview, Move drawer, Item drawer, toasts

**Unit id:** `LV3·U4` · **Rows:** LV3·C20–C25 · **Changes app files:** yes (TypeScript, guide)

### Goal
Design README §4 Preview, §5 Move drawer, §6 Item drawer, §7 Toast, §Preview gestures, §Preview actions, §Move
semantics (the server performs the move: PH4 `move` targets; card LV8 swap), LV-D17 (the empty "Fill item properties"
page).

### Read first
Design README §4–§7, §Preview gestures, §Preview actions, §Move semantics · master plan §5 PH4 (retake, move, delete),
PH6, §6 texts · book `10J` LV-D17.

### Files
New: `apps/desktop/src/phone/components/{Preview.tsx, MoveDrawer.tsx, ItemDrawer.tsx, PropertiesPage.tsx}`,
`apps/desktop/src/phone/__tests__/{lv3_preview.test.tsx, lv3_drawers.test.tsx}`. Edited: `apps/desktop/src/phone/
{PhoneApp.tsx, state.ts, components/Toast.tsx, phone.css}`; `docs/guide/{01_product.md, 02_architecture.md,
05_working.md}` (the phone app's screens, its code layout and tests, how to try it).

### Core mechanisms
None.

### Ordered tasks
1. Preview (header kicker/title/ratio chip per kind, image area with the template under a template-slot image, hold and
   overlay, actions).
2. Move drawer (main grid in Main order with states, New extra, the condition row and its tiles) → `POST /photos/<id>/
   move` with the PH4 target; the preview stays open on the image's new place after the re-read.
3. Item drawer (rows; "Article number" only when `article_number` is not null; "Fill item properties" → the empty
   full-page container with a back control).
4. Toast timing; tests for the rows; guide.

### Criteria (6 criteria, 19 rows)
Same session as U3 (B 02 "Back" filled; A 01 "Front"; C 03; all ratio 1.0; extra 04; condition C1; current A).
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| LV3·C20(a) | preview header | open B's photo | kicker "SHOT 02 · CAPTURED", title "Back", chip "1:1"; for a split shot the chip "SPLIT 2×1:1" | `[rule]` design §4 |
| LV3·C20(b) | same | open the extra; open C1 | "IMAGE 04 · EXTRA" / "Extra image"; "CONDITION 01" / "Condition image" | `[rule]` design §4 |
| LV3·C21(a) | preview hold | B's photo; hold | the photo at opacity 0.35 over the template; pill "TEMPLATE" | design §Preview gestures |
| LV3·C21(b) | same | hold and slide dx +20 (width 390) | bar "PHOTO 42%"; back on the main screen the template overlay is 42 % (shared value) | design §Preview gestures |
| LV3·C21(c) | same | the extra; hold | nothing changes; no hint shown | design §Preview gestures |
| LV3·C22(a) | preview actions | **Back** | the preview closes; no request | design §Preview actions |
| LV3·C22(b) | same | **Retake** on B's photo | `POST /retake {session_id, container_id: <B>}`; `vibrate([30,40,30])`; toast "Shot 02 reopened"; the preview closes | card LV6, MC-LV3, M-LV4 |
| LV3·C22(c) | same | **Delete** on C1 | `POST /photos/<C1>/delete {session_id}`; toast "Image deleted"; the preview closes | §6 (Delete = Deny) |
| LV3·C23(a) | Move drawer | open from B's photo | main tiles in Main order: "01" EMPTY, "02" THIS IMAGE (not tappable), "03" EMPTY, "04" SWAP, then "+ New extra" ADD AT END | `[rule]` design §5 |
| LV3·C23(b) | same | tap "04" | `POST /photos/<B photo>/move {session_id, target: {kind: "main_position", position: 4}}`; `vibrate([30,40,60])`; toast "Swapped with 04" | card LV8, PH4 |
| LV3·C23(c) | same | tap "03" | target `{kind: "container", container_id: <C>}`; toast "Moved to shot 03" | card LV8, PH4 |
| LV3·C23(d) | same | tap "+ New extra" | target `{kind: "extra_end"}`; toast "Added as extra image" | PH4, design §7 |
| LV3·C23(e) | same | expand Condition images; tap "C1" | target `{kind: "condition", position: 1}`; toast "Swapped with C1" | card LV8, PH4 |
| LV3·C23(f) | same | expanded; tap "+ Add" | target `{kind: "condition_end"}`; toast "Sent to condition images" | PH4, design §7 |
| LV3·C24(a) | Item drawer | tap the menu button; `article_number` null | `vibrate(12)`; title "Oak Lounge Chair"; "CONNECTED · <MAC NAME>"; rows SKU "BEY-123", Current shot "01/03 · 1:1", Angle "Front", Images "1 main · 1 condition"; **no** Article number row (an item without a barcode) | LV-D17, design §6 |
| LV3·C24(b) | same | `article_number` "A-77" | an "Article number" row "A-77" after SKU | LV-D17 |
| LV3·C24(c) | same | current Free | Current shot "FREE SHOT · 1:1"; no Angle value | design §6 |
| LV3·C24(d) | same | tap **Fill item properties** | a full-page empty container with a back control; back returns to the drawer | LV-D17 |
| LV3·C25(a) | toast | any toast | visible, then hidden after 1.4 s; locking or unlocking never shows a toast | `[rule]` design §7 |

### Delegated choices
Sheet animation implementation · whether the Move drawer's state labels come from the session view or are computed in
`state.ts` · the properties page's title text.

### Notes
- After a move the server re-read decides where the image is; the preview follows it (design: "keep the preview open
  on the image's new location").

### Unit test commands
`cd apps/desktop && npx vitest run src/phone` · `npm run typecheck`

## Closing gate (once, after the last unit; replaces the template's closing gate)
`cargo test --workspace` · `cargo clippy --workspace --all-targets -- -D warnings` · `cargo fmt --all -- --check` ·
`tools/sidecar/build.sh --stub --test` · `cd apps/desktop && npm run typecheck` · `npm test` · `npm run build`. (The
Rust gate runs once even though this milestone changes only TypeScript: the phone build is served by the Rust server;
U0 changes Rust, so the Rust gate runs. **Run `npm run build` first** (psm-desktop needs `apps/desktop/dist/`.) One
command at a time.

## Review mode
The reviewer reads the units in order and runs, once: U0's unit test commands · `cd apps/desktop && npx vitest run src/phone` · `npm run
typecheck` · `npm run build`. Only B1–B4 block. Also check, by reading: no secure-context API (`serviceWorker`,
`wakeLock`, `crypto.subtle`, `navigator.clipboard`, `getUserMedia`) anywhere under `src/phone/`; no absolute URL or
scheme; the token never logged or put in a toast; every threshold and timing equals the design README (hold 200 ms, tap
260 ms, double tap 300 ms / 50 px, slide 14 px, lock 110 px, overlay ×130, magnify 95 px, drag-up −50 px, toast 1.4 s,
flash .45 s); the book's differences (Condition after the last shot, swap, × recentres, Left/Right, shooting order) are
honoured. Compare with the prototype by opening it (`npx serve` on the design folder) beside `npm run dev` at
`http://localhost:1420/phone.html` in a phone-sized window; differences in look are notes unless they break a row.
Notes go to `docs/implementation/backlog_live/LV3_phone_app.md`.

## Manual try-out (the narrative is built from this)
On this Mac with the camera connected and the phone on the same Wi-Fi:
1. Start Session Manager built from this branch; open a session whose item type has a template.
2. Settings → Phone → **Pair a phone**; scan the QR code with the phone. The phone shows the template of the current
   shot. On iPhone: Share → Add to Home Screen, then open it from the home screen (full screen). On Android: tap **Full
   screen**.
3. Press and hold the picture: live appears; release: the template comes back. Slide sideways: the template shows
   through live at the percentage shown.
4. Slide up until the padlock closes: live stays on. Tap the chair's front leg: "AF LOCKED" appears and the camera
   focuses there. Tap × to send the focus back to the centre.
5. Press the round button: a flash, then "Shot 01 captured"; the dock's outline moves to the next shot.
6. Hold a filled tile in the dock and drag it up: its preview opens. **Retake**: the shot is reopened (on the Mac the
   photo turns red); press the shutter again to refill it.
7. **Move image** → pick another position: the two images swap.

## Review log

### 2026-11-26 — implement r1 (Claude Fable 5.2, `implementer: fable`)
**Built.** U0: a phone Delete announces `PhotoChanged` + `SessionChanged` from core (`ReviewService::delete_announced`);
deleting the active session announces `ActiveSessionChanged {None}` (`with_events`, plus the path where Capture One
could not be left); the phone server's event observer is carried from hub to hub at a rebuild
(`EventHub::observe_carried`, handed over by `transfer_target_to` in `apply_settings`); `500 internal` and `404
not_found` bodies carry fixed texts. U1–U4: the Shoot Companion under `apps/desktop/src/phone/` as the registry names
it (shell, socket, live canvas, main screen gestures, capture and focus, dock, preview, Move and Item drawers,
properties page, toast), the guide updated. Checkpoints in unit order: 0ee711f8 · fefafb8f · ce02e69f · 34fa2a4f ·
a43b75d5. Full detail, the row → test map and the try-out: `handoffs/implementer/handoff_LV3_phone_app_implement_r1.md`.

**Judgment calls (each also under Assumptions in the handoff).**
1. Tests fake the Mac at the transports (`fetch`, `WebSocket`) instead of mocking `api.ts`/`socket.ts`: the rows name
   request bodies, the `Authorization` header and demand messages, which are only observable below those modules.
2. Free shot shows live all the time (demand `live: true` while in Free shot): there is no template to show, and the
   design's pill reads "FREE SHOT · LIVE". An untemplated session is Free shot only.
3. Free shot's viewport is 1:1 (the design's session ratio; the row "FREE SHOT · 1:1"). A shot whose ratio is `null`
   uses the live frame's aspect, 3:2 before the first frame.
4. Whenever the phone removes its focus marker (×, choosing another shot, Retake) it also asks the camera to recentre
   (`POST /focus/clear`), by card LV9's reason; when the Mac moves the current shot the marker stays (the camera keeps
   its point).
5. The capture toast waits for the socket's `exposed` for the phone's own capture id and then for the first session
   re-read that lists a photo it did not list before; its place decides the text. An untemplated session's capture
   reads "Extra image NN added" (the design's text; §6's "does not occur" is about templated sessions).
6. "All template shots captured" (design §7) is not shown: the Mac moves the current shot at the exposure, before the
   photo lands, so the design's order of the two toasts cannot be kept; the pill and the dock outline show Free shot.
7. The camera's own focus frames (`focus_frames`) are not drawn: no row asks for it; the reticle at the tap and "AF
   LOCKED" on the camera's report are the design's.
8. No session open on the Mac: the text "Open a session in Session Manager to start shooting." A failed action other
   than the named codes: the toast "That did not work. Try again." and a session re-read. Neither text is in §6.
9. While the Mac does not answer `hello`/`session` the connecting overlay stays and both are asked again every 2 s;
   the session is re-read on socket open, on every `session` message and after each of the phone's own actions.
10. The Full screen button sits in its own row above the stage. The dock's scroller is 260 px tall, not the design's
    190: at 190 the "Release to open" pill of a tile dragged up was cut off (seen in a real browser).
11. Properties page title: "Item properties". Generic texts of U0: "Something went wrong in Session Manager." and
    "Not found.".
12. A double tap uses the prototype's strict `<` (300 ms, 50 px); the README writes `≤`.

**Deviations.**
- **LV3·C24(a) "Images 1 main · 1 condition".** The row's own fixture (U3's session: B's photo and one extra) has two
  Main photos, and the Mac's `counts.main` counts both (`phone_view`). The phone shows the Mac's counts; the test's
  fixture is truthful and asserts "2 main · 1 condition". A slip in the row's text, not a behaviour question.
- **Preview in U3.** U3's rows open a preview that is U4's file; U3's checkpoint carried a minimal preview (the photo,
  full screen) that U4 replaced with `Preview.tsx`.
- **Tracker actor** is `fable` (the prompt template says `codex`): this plan is `implementer: fable`.
- **No red baseline for U1–U4:** the UI tests were written with the code, not before it. U0's was captured (5 of the
  6 rows failed before the change; C26(b), one undo step, already held). C28's first red was confounded by the test
  server's short idle timeout, so it was re-checked by removing the hand-over once (red), then restoring it.

**For the reviewer.**
- jsdom in this repo has **no `PointerEvent`** (master_plan_live §3 calls the Pointer API "standard" in jsdom):
  `helpers.ts` sends mouse events of the pointer types with a `pointerId`, and supplies a layout (stage 390 × 610,
  viewport rect from its inline size and rotation, dock tiles 36 px on a 42 px pitch).
- Seen in a real browser (headless Chrome, 390 × 844, a throwaway mock of the phone server outside the repo that sent
  frames by demand): pairing, connecting, template, hold → live over the template at 35 %, the lock slide and lock,
  tap → reticle → "AF LOCKED", the overlay bar, the flash, dock magnify and drag-up, preview with hold and slide, the
  Move drawer with the condition group open, the Item drawer, the properties page, a long shot rotated. **Not seen:**
  a real phone, a real camera, haptics, iPhone's home-screen app, Android's Full screen: LV4.
- The LV2 backlog notes left for LV4 (the revoke command's wiring, the LV1 carry-overs) are untouched.

### 2026-11-26 — review r1 (opus, pipeline-reviewer): **APPROVED_WITH_NOTES**
- **Gate.** Tracker read `IMPLEMENTED`; checkpoints `0ee711f8` · `fefafb8f` · `ce02e69f` · `34fa2a4f` · `a43b75d5` ·
  close `9ba8180a` all in `git log`. Tree at review `1c00d149`, clean.
- **Perimeter.** `git diff --stat 0ee711f8^ 9ba8180a` = 46 paths = the handoff's 45 code/guide files + this plan file.
  The two paths outside the plan's file lists (`crates/testkit/src/phone.rs`, `crates/integrations/src/lib.rs`) are
  disclosed and needed (C28: the plan's "code path that rebuilds the App"; the test server that follows a rebuild).
- **Checks (Review mode, once each).** `npm run build` ok; `cargo test -p psm-testkit --test suite -- lv2_phone_ops::
  lv2_phone_ws:: lv3_server_carry::` 29 passed; `npx vitest run src/phone` 7 files, 94 passed; `npm run typecheck`
  clean. The implementer's closing-gate stamp (Rust 2723, clippy, fmt, sidecar stub, UI 1047) stands.
- **Verified correct.** U0: `ReviewService::delete_announced` emits `PhotoChanged` + `SessionChanged` from core (H8; the
  route only calls it); `session_delete` builds `SessionsService::with_events` and announces `ActiveSessionChanged
  {None}` only when Capture One could not be left (no double emission); `EventHub::observe_carried` is handed over in
  both rebuild paths (`apply_settings` Runtime and `reopen`), deduplicated by pointer; `error_body` fixes the 500/404
  texts. U1–U4: tests fake the Mac at `fetch`/`WebSocket`, so the asserted bodies, `Authorization` header and demand
  messages are the real ones. `decodeFrame`/`reconnectDelay` match PH5/PH7; frame acceptance by `gen` and `seq`;
  `liveShown` keeps the template at 1 until a frame of the current `gen` is drawn. `tapToFrame` re-derived by hand
  for C12(a–d) (0.16797, 0.23438, rotated (225, 150) → (0.5, 0.5)). Camera `status` strings match `ws::status_name`.
  The phone's extra positions (`shots.length + index + 1`) match the server's container-then-extras Main model in
  `move_to`, and the `container` target is converted to `final_no` by the route. Every threshold and timing equals the
  design README (hold 200 ms, tap 260 ms, double tap 300 ms / 50 px, slide 14 px, lock 110 px, overlay ×130, magnify
  95 px, drag-up −50 px, toast 1.4 s, flash .45 s after 70 ms). The book's differences hold: Condition after the last
  shot (hint text, "Condition image added"), Move swaps, × recentres (`POST /focus/clear`), Left/Right split labels,
  dock in shooting order with final numbers. No `serviceWorker`/`wakeLock`/`crypto`/`clipboard`/`getUserMedia`, no
  `console`, no absolute URL or scheme under `src/phone/`; the token never reaches a toast or a log. Every row's test
  observes the row's outcome at its boundary and fails on a wrong one (C24(a) asserts the truthful counts, see notes).
- **Assumptions.** None contradicts 10J. Free shot always live fits LV-D5 as revised (no sleep). Recentring on every
  marker removal follows card LV9's reasoning. Leaving out "All template shots captured" is allowed: §6 lists the
  toast texts, it does not require each one. All twelve go to the narrative.
- **H1–H8.** Hold (no external writes; Delete = Deny, bytes kept; frames drawn as received, grey by CSS; events from
  core; nothing device-specific in core).
- **Notes** (6, `docs/implementation/backlog_live/LV3_phone_app.md`): C24(a) row text vs fixture (plan lesson);
  capture toast depends on `exposed` before the re-read; old hub keeps the carried observer; jsdom has no
  `PointerEvent` (`master_plan_live` §3); overlay tick against the render's value; side-by-side look comparison not
  done by the reviewer (unattended, no browser).

