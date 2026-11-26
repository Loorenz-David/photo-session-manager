---
plan: LV3_phone_app
role: implement
round: 1
state: DONE
date: 2026-11-26
actor: fable
---
# Handoff — LV3 the phone app (Shoot Companion), implement r1

Actor: Claude Fable 5.2 (`implementer: fable`). All five units built in order, one checkpoint each, closing gate green.
The tracker line says `fable` (the prompt template's `codex` does not apply to this plan).

## ⚠ OWNER DECISIONS REQUIRED (0)
None. Twelve judgment calls are listed under **Assumptions**; each can be changed by saying so.

## What was built
- **The phone is the shooting remote.** Scan the QR code in Settings → Phone and the phone opens on whatever session
  is open on the Mac. It shows the current shot's template in black and white. Hold the picture for live; release for
  the template. Slide up to lock live; double-tap to unlock. Slide sideways to set how much of the template shows
  through live (starts at 35 %). While locked, tap to focus there: the camera's answer shows "AF LOCKED", × sends the
  focus point back to the centre. Long shots turn the view 90°; a split shot shows a centre line and Left/Right; the
  exposure is read-only on live.
- **Shutter.** The round button flashes and (on Android) vibrates at once; the toast ("Shot 04 captured", "Condition
  image added") comes when the Mac shows where the photo landed. "Camera not connected", "Not taken. Try again." and
  "Session changed on the Mac" when that is what happened. The phone never moves the current shot itself.
- **Shot dock.** Template shots in shooting order with their final numbers, extra Main images, the Free shot tile,
  then Condition images. Tap selects an empty shot or opens an image; a quick swipe scrolls; hold magnifies; hold and
  drag up, then release, opens. **Next** goes to the next empty shot in shooting order, then to Free shot.
- **Preview, Move, Item.** A captured image full screen with Back, Retake (template shots) or Delete (extras,
  condition), and Move image: any Main position (a filled one swaps), a new extra, or the Condition group. The menu
  button opens the item's information (name, SKU, article number only when there is a barcode, current shot, angle,
  image counts) and "Fill item properties", an empty page for now.
- **Connection.** Plain HTTP, no secure-context API. The token is kept on the phone and removed from the address; a
  removed phone says so. The socket reconnects by itself (0.5, 1, 2, 4, 4… s; at once when the screen comes back). A
  **Full screen** button where the browser offers it (Android); none in the installed iPhone app.
- **Server carry-overs (U0).** A Delete on the phone now refreshes the Mac and other phones. Deleting the active
  session tells phones there is none. Open phone sockets keep hearing of changes after a settings save that rebuilds
  the App. Error answers no longer carry internal error text or local paths.
- **Guide** (`docs/guide/01_product.md`, `02_architecture.md`, `05_working.md`): the phone app's screens, code
  layout, tests, how to try it.

## Tests → criterion rows
85 rows. One line per row: the test that discharges it, and whether its assertion is the row's shape. Rust:
`crates/testkit/tests/suite/lv3_server_carry.rs`. UI: `apps/desktop/src/phone/__tests__/`.

| Row | Test | Shape |
|---|---|---|
| C26(a) | `lv3_c26a_phone_delete_emits_the_change_events_and_reaches_a_second_phone` | as the row |
| C26(b) | `lv3_c26b_phone_delete_is_one_undo_step_that_restores_the_photo` | as the row (one undo restores; the undo label is then what it was before) |
| C27 | `lv3_c27_deleting_the_active_session_tells_the_phone_there_is_none` | as the row |
| C28 | `lv3_c28_an_open_socket_hears_of_a_photo_after_the_app_was_rebuilt` | as the row (asserts the App's event hub changed) |
| C29(a) | `lv3_c29a_an_internal_error_answers_a_fixed_generic_message` | as the row (the store fails: pairings table renamed) |
| C29(b) | `lv3_c29b_a_missing_preview_file_is_404_without_its_path` | as the row |
| C1(a) | `lv3_shell` `lv3_c1a_…` | as the row |
| C1(b) | `lv3_c1b_…` | as the row |
| C1(c) | `lv3_c1c_…` | as the row |
| C2(a) | `lv3_c2a_…` | as the row |
| C2(b) | `lv3_c2b_…` | as the row (opacity 0, present at 399 ms, gone at 400 ms, then a press demands) |
| C3(a) | `lv3_c3a_…` | exact two messages |
| C3(b) | `lv3_c3b_…` | exact two messages |
| C4(a) | `lv3_c4a_…` | as the row |
| C4(b) | `lv3_c4b_…` | as the row |
| C4(c) | `lv3_c4c_…` | as the row |
| C5(a) | `lv3_socket` `lv3_c5a_…` (+ `lv3_c5_reconnectDelay_…`, core mechanism) | exact five times |
| C5(b) | `lv3_c5b_…` | as the row |
| C5(c) | `lv3_c5c_decodeFrame_…` | as the row |
| C6(a) | `lv3_c6a_…` | as the row |
| C6(b) | `lv3_c6b_no_button_in_the_installed_app`, `lv3_c6b_no_button_when_already_full_screen` | one test per input |
| C7(a–c) | `lv3_c7a_…`, `lv3_c7b_…`, `lv3_c7c_…` | as the rows |
| C8(a) | `lv3_main` `lv3_c8a_…` | as the row (fill 14 % = 15/110) |
| C8(b) | `lv3_c8b_…` | as the row |
| C8(c) | `lv3_c8c_…` | as the row |
| C8(d) | `lv3_c8d_…` | as the row, plus the value shows through live (0.42) |
| C8(e) | `lv3_c8e_…` | as the row ("unchanged" read as 35 % back on a template shot) |
| C9(a–c) | `lv3_c9a_…`, `lv3_c9b_…`, `lv3_c9c_…` | as the rows |
| C10(a) | `lv3_c10a_a_tap_while_locked_…`, `lv3_c10a_the_focus_point_is_the_tapped_point_…` | the row, and a second input off-centre (a centre tap cannot tell x from y) |
| C10(b) | `lv3_c10b_…` | as the row |
| C10(c) | `lv3_c10c_…` | as the row |
| C11(a) | `lv3_c11a_…` | as the row |
| C11(b) | `lv3_c11b_…` | as the row |
| C11(c) | `lv3_c11c_not_focused_…`, `lv3_c11c_a_focus_area_that_cannot_move_…` | one test per outcome |
| C12(a–d) | `lv3_geometry` `lv3_c12a_…` … `lv3_c12d_…` | as the rows |
| C13(a) | `lv3_c13a_a_long_shot_turns_the_viewport_and_the_buttons`; dock half `lv3_dock` `lv3_c13a_the_dock_tiles_turn_…`; mechanism `lv3_c13a_viewportBox_…` | as the row |
| C13(b) | `lv3_c13b_…` (mechanism `lv3_c13b_c20a_ratioLabel_…`) | as the row |
| C13(c) | `lv3_c13c_…` | as the row |
| C13(d) | `lv3_c13d_…` | as the row |
| C13(e) | `lv3_c13e_…` | as the row |
| C14(a) | `lv3_c14a_…` | as the row |
| C14(b) | `lv3_c14b_…` | as the row |
| C14(c) | `lv3_c14c_…` | as the row (no toast while the photo is not there) |
| C14(d) | `lv3_c14d_…` | as the row |
| C14(e) | `lv3_c14e_no_camera_…`, `lv3_c14e_a_capture_the_camera_did_not_take_…`, `lv3_c14e_a_session_changed_…` | one test per input |
| C14(f) | `lv3_c14f_next_follows_…`; dock half `lv3_dock` `lv3_c14f_the_outline_follows_…` | as the row |
| C15(a–c) | `lv3_dock` `lv3_c15a_…`, `lv3_c15b_…`, `lv3_c15c_…` | as the rows |
| C16(a) | `lv3_c16a_tileScale_…` | as the row |
| C16(b) | `lv3_c16b_…` | as the row |
| C17(a–c) | `lv3_c17a_…`, `lv3_c17b_…`, `lv3_c17c_…` | as the rows |
| C18(a–b) | `lv3_c18a_…`, `lv3_c18b_…` | as the rows |
| C19(a–c) | `lv3_c19a_…`, `lv3_c19b_…`, `lv3_c19c_…` | as the rows |
| C20(a) | `lv3_preview` `lv3_c20a_a_template_shots_photo_…`, `lv3_c20a_a_split_shots_chip_…` | as the row |
| C20(b) | `lv3_c20b_…` | as the row |
| C21(a–c) | `lv3_c21a_…`, `lv3_c21b_…`, `lv3_c21c_…` | as the rows |
| C22(a–c) | `lv3_c22a_…`, `lv3_c22b_…`, `lv3_c22c_…` | as the rows |
| C23(a) | `lv3_drawers` `lv3_c23a_…` | as the row |
| C23(b) | `lv3_c23b_a_position_that_has_an_image_swaps`, `lv3_c23b_the_preview_stays_open_on_the_image_in_its_new_place` | the row; the second is task 2's "the preview stays open on the image's new place" |
| C23(c–f) | `lv3_c23c_…`, `lv3_c23d_…`, `lv3_c23e_…`, `lv3_c23f_…` | as the rows |
| C24(a) | `lv3_c24a_…` | **differs:** asserts "2 main · 1 condition" (see Deviations) |
| C24(b–d) | `lv3_c24b_…`, `lv3_c24c_…`, `lv3_c24d_…` | as the rows |
| C25(a) | `lv3_c25a_…` | as the row |

Core-mechanism unit tests (the plan's list): `lv3_geometry.test.ts` (`tapToFrame`, `localPoint`, `viewportBox`,
`isRotated`, `ratioLabel`, `touchMode`, `lockProgress`, `overlayValue`, `isDoubleTap`), `lv3_socket.test.ts`
(`decodeFrame`, `reconnectDelay`), `lv3_dock.test.tsx` (`tileScale`). Every test maps to a row above.

**Red baseline.** U0: captured before the production change: 5 failed (`c26a`, `c27`, `c28`, `c29a`, `c29b`), 1 passed
(`c26b`: one undo step already held). C28's first red was confounded (the test server's 600 ms idle close shut the
socket during the rebuild); with the app's timings it was re-checked by removing the hand-over in
`transfer_target_to` once (red), then restoring the file. **U1–U4: not captured**; the UI tests were written with the
code.

## Assumptions
1. **Tests fake the Mac at `fetch` and `WebSocket`**, not at `api.ts`/`socket.ts`, so the asserted bodies, header and
   demand messages are the real ones.
2. **Free shot shows live all the time** (no template to show; the design's "FREE SHOT · LIVE"). An untemplated
   session is Free shot only.
3. **Free shot's picture is square (1:1).** A shot with no fixed shape uses the live frame's shape (3:2 before the
   first frame).
4. **When the phone removes its focus marker (×, choosing another shot, Retake) the camera's focus point also goes
   back to the centre.** When the Mac moves the current shot the marker stays.
5. **The capture toast waits for the photo to appear** in the session (after the camera's "exposed"). Over Wi-Fi that
   is a few seconds after the flash. An untemplated capture reads "Extra image NN added".
6. **"All template shots captured" is not shown** (the Mac moves the current shot before the photo lands).
7. **The camera's own focus frames are not drawn** (no row asks for it).
8. **Two texts not in the plan's list:** "Open a session in Session Manager to start shooting." (no session open) and
   "That did not work. Try again." (an action failed for another reason).
9. **Connecting:** the overlay stays and the phone asks again every 2 s until the Mac answers.
10. **Full screen button** in its own row above the picture. **Dock scroller 260 px** (design 190 px cut off "Release
    to open").
11. **Texts:** properties page "Item properties"; server errors "Something went wrong in Session Manager." and "Not
    found.".
12. **Double tap** strictly under 300 ms and 50 px (the prototype's rule).

## Deviations
- **C24(a)** shows the Mac's counts: the row's fixture has two Main photos, so the test asserts "2 main · 1
  condition", not the row's "1 main".
- **U3 carried a minimal preview** (U3's rows open one; `Preview.tsx` is U4's file and replaced it).
- **Tracker actor `fable`**, not the template's `codex`.
- **`master_plan_live.md` §3** says jsdom has the Pointer API; this repo's jsdom has no `PointerEvent`. The test
  helper sends mouse events of the pointer types with a `pointerId`. For the coordinator to fold into §3.
- Nothing outside the plan's file perimeter except `crates/testkit/src/phone.rs` (a `TestPhone` that follows the App
  across a rebuild, needed by C28) and `crates/integrations/src/lib.rs` (the event hub, the plan's "code path that
  rebuilds the App").

## Files changed
Write perimeter of this session (45 code and guide files, the plan file, this handoff). No tool-recorded state (no
architecture graph in this repo). No mutation probes left behind: the one temporary edit (`lib.rs`, C28 check) was
restored from a copy before the U0 commit.
- **Rust:** `crates/core/src/services/review.rs` · `crates/integrations/src/{lib.rs, session_delete.rs}` ·
  `crates/integrations/src/phone/{routes.rs, wire.rs, ws.rs}` · `crates/testkit/src/phone.rs` ·
  `crates/testkit/tests/suite/{main.rs, lv3_server_carry.rs (new)}`.
- **Phone app:** `apps/desktop/phone.html` · `apps/desktop/package.json`, `package-lock.json`
  (`@fontsource/ibm-plex-mono`) · `apps/desktop/src/phone/{main.tsx, PhoneApp.tsx, api.ts}` (edited) ·
  `{socket.ts, state.ts, geometry.ts, gestures.ts, phone.css}` (new) · `components/{ConnectingOverlay, ControlBar,
  Dock, FullscreenButton, ItemDrawer, LiveCanvas, LockControl, MoveDrawer, OverlayBar, Preview, PropertiesPage, Stage,
  TemplateLayer, Toast}.tsx` (new) · `__tests__/{helpers.ts, lv3_shell.test.tsx, lv3_socket.test.ts,
  lv3_main.test.tsx, lv3_geometry.test.ts, lv3_dock.test.tsx, lv3_preview.test.tsx, lv3_drawers.test.tsx}` (new).
- **Docs:** `docs/guide/{01_product.md, 02_architecture.md, 05_working.md}` ·
  `docs/implementation/plans/LV3_phone_app.md` (Tracker line, Review log) · this handoff (not committed by me: the
  runner commits handoffs).
- Outside the repo, not part of the product: a throwaway mock server and browser driver in `/tmp/psm_phone_mock/`.

## Checkpoint SHA
| Unit | SHA |
|---|---|
| U0 server carry-overs | `0ee711f8` |
| U1 shell | `fefafb8f` |
| U2 main screen | `ce02e69f` |
| U3 shot dock | `34fa2a4f` |
| U4 preview, drawers, guide | `a43b75d5` |
| Close (tracker, review log; documents only) | `9ba8180a` |

## Stamp
Closing gate, once, one command after the other, on `a43b75d5` with a clean tree (`git status --porcelain` empty before
and after). The close commit `9ba8180a` changes only the plan file.

| Command | Result |
|---|---|
| `cd apps/desktop && npm run build` | built |
| `cargo test --workspace` | exit 0; 2723 passed, 0 failed (behaviour suite 1604 passed) |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo fmt --all -- --check` | exit 0 |
| `tools/sidecar/build.sh --stub --test` | exit 0; 100 % tests passed out of 2 |
| `cd apps/desktop && npm run typecheck` | exit 0 |
| `npm test` | exit 0; 146 files, 1047 tests passed (phone: 7 files, 94 tests) |

No pre-existing failures. Per-unit commands were run after each unit (U0: `lv2_phone_ops:: lv2_phone_ws::
lv2_phone_http:: lv3_server_carry::` 56 passed, `cargo check -p psm-integrations`; U1–U4: `npx vitest run src/phone`,
`npm run typecheck`).

**Not verified by any test or by me:** a real phone, a real camera, haptics, the iPhone home-screen app, Android's
Full screen, focus accuracy on the camera. Seen only in headless Chrome against a mock (see the Review log). LV4.

## Manual try-out
On this Mac, camera connected, phone on the same Wi-Fi:
1. `cd apps/desktop && npm run build`, then start Session Manager from this branch. Open a session whose item type
   has a template.
2. Settings → Phone → **Pair a phone**; scan the QR code. The phone shows the current shot's template. iPhone: Share →
   Add to Home Screen, open it from there. Android: tap **Full screen**.
3. Press and hold the picture: live appears over the template; release: the template is back. Slide sideways: the
   bar shows "TEMPLATE nn%".
4. Slide up until the padlock closes: "LIVE · LOCKED". Tap the piece: a square marks the spot, then "AF LOCKED".
   Tap ×: the marker goes and the camera's focus point returns to the centre. Double-tap: unlocked.
5. Press the round button: a flash; when the photo reaches the Mac, "Shot 01 captured" and the dock's outline moves
   to the next shot.
6. Hold a filled tile in the dock and drag it up until "Release to open"; release. **Retake**: "Shot 01 reopened",
   the photo turns red on the Mac, the shot is current again.
7. Open an image, **Move image**, pick a position that has an image: the two swap and the preview stays on the image.
8. Menu button (three dots): the item's information. **Fill item properties**: an empty page; the arrow goes back.
9. Open a condition image on the phone and tap **Delete**: "Image deleted", and the Mac's view updates by itself
   (undo on the Mac brings it back).
