---
plan: LV10_zoom_reframe_guides
role: implement
round: 1
state: DONE
date: 2026-11-27
actor: fable
---
# Handoff — LV10_zoom_reframe_guides, implement r1

All four units are built and committed; the closing gate is green on the code tree handed over (`764ba540`, clean; the
close commit `814990a9` adds only the plan file's Tracker line and Review log). 53 criterion rows → 54 row tests, plus
21 unit tests of the two core mechanisms. **Nothing was seen on a phone or in a browser**: every row is proven in jsdom
(two fingers = two pointer ids) or at the Rust boundary, and the looks (the composed picture's CSS, the zoom layer, the
editor, the grid, the guide lines) are CSS written by reasoning. No hardware capture was triggered.

## ⚠ OWNER DECISIONS REQUIRED (0)
None. Twelve judgment calls are listed under Assumptions for the narrative's "Choices made for you"; four observations
for the coordinator are at the end (one of them, a photo that needs review, is answered by book 10D and needs a
contract line, not an owner).

## What was built
In product terms first.
1. **Pinch to zoom in the full-screen preview.** Two fingers zoom the photo about the point between them, 1× to 6× (it
   stretches a little past both while the fingers are down and settles on release). Zoomed, one finger moves it, never
   past its edge; swipes and the hold for the template are off; the strip, ⋯ and × still work. A double tap goes to 3×
   about the tap, or back to 1×. A second finger during a swipe or a hold ends it with nothing sent and starts a pinch.
   Another photo always shows at 1×.
2. **Photos shown as framed.** Once the Mac has Capture One's full-frame preview of a photo, the preview and the strips
   show it as the Mac frames it (turned, flipped, straightened, cut to its frame; a split shows its middle line) and
   the preview takes the framing's shape. Before that the picture shows as it is.
3. **Reframe on the phone.** ⋯ → Reframe opens the Mac's composition as an editor: the frame stays centred and the
   photo moves under it. One finger moves; two fingers move, zoom and straighten (a twist counts after 3°). The photo
   stops where the frame would show an empty corner, while moving. A fine grid shows only while straightening. Controls:
   Normal | Split, Shape with Turn, Rotate and flip, Guides, Reset, a straighten slider; Cancel and Done. Reframe is off
   with "Waiting for Capture One's full-frame preview" until the Mac has that preview.
4. **Done saves into the Mac's own composition.** One request; "Framing saved"; the Mac shows it at once, its outputs
   follow as for its own edits, ⌘Z there undoes it. An unchanged Done sends nothing; Done right after Reset asks the
   Mac for its reset. If the framing changed on the Mac meanwhile, nothing is overwritten and the phone says so.
5. **Guides on the live view.** The live screen's menu has a Guides row (Off, Thirds, Grid, Golden ratio, Diagonals),
   Off the first time, remembered on the phone, drawn over live and template alike; they take no touch.

In code:
- **U1** `crates/core/src/error.rs` (`CompositionChanged`); `services/composition.rs` (`set_at_revision`,
  `reset_at_revision`, private `at_revision`); `services/phone_view.rs` (`PhoneComposition`, `PhoneImage.composition`,
  `PhoneSession.default_shape`, `full_frame_preview`); `integrations/src/phone/wire.rs` (`CompositionBody`, `ResetBody`,
  three codes in `error_code`); `routes.rs` (`save_composition`, `reset_composition`, two routes);
  `apps/desktop/src-tauri/src/commands/items.rs` (`composition_changed` in the exhaustive code map).
- **U2** new `apps/desktop/src/phone/pinch.ts`; `gestures.ts` (`ZOOM`); `api.ts` (`PhoneComposition`, the two optional
  fields); `state.ts` (`composedOf`, `previewRatio`); `components/Preview.tsx` (a `Gesture` per touch sequence with the
  tracker, phases `panning` / `waiting`, `zoomCss`, the `.ph-zoom` layer `preview-zoom`); `components/PhotoImage.tsx`
  (the composed picture, `composed-frame`); `phone.css` (`.ph-zoom`, `.ph-composed*`).
- **U3** new `reframe.ts`, `components/ReframeEditor.tsx`; `components/PhotoDrawer.tsx` (Reframe, the waiting line);
  `PhoneApp.tsx` (`reframing`, the editor over the Preview, the Preview and its strip disabled under it); `state.ts`
  (`TEXT.reframeWaiting`, `GUIDE_LABELS`, `GUIDE_KINDS`, `readGuides`, `writeGuides`, `REFRAME_GUIDES_KEY`); `phone.css`
  (`.ph-reframe*`, `.ph-guides`).
- **U4** `api.ts` (`saveComposition`, `resetComposition`, `CompositionValuesBody`); `PhoneApp.tsx` (`saveFraming`,
  `liveGuides`); `components/ItemDrawer.tsx` (the Guides group); `components/Stage.tsx` (`LiveGuides`; `ImageFrame`'s
  children may be a function of the viewport's size); `state.ts` (three toasts, `LIVE_GUIDES_KEY`); `phone.css`.

## Tests → criterion rows
Rust: `crates/testkit/tests/suite/lv10_phone_composition.rs` (HTTP through `TestPhone`). UI:
`apps/desktop/src/phone/__tests__/lv10_{zoom,reframe,save_and_guides}.test.tsx` (`PhoneApp`, the Mac faked at `fetch` /
`WebSocket`). Each test is named after its row.

| Row | Test | Asserts the row's exact outcome? |
|---|---|---|
| C1(a) | `lv10_c1a_the_view_carries_the_composition_and_the_default_shape` | yes: the whole `composition` JSON with the stored rect, `default_shape` "1:1" |
| C1(b) | `lv10_c1b_a_photo_without_the_full_frame_preview_has_no_composition` | yes: Q has a rect and a camera frame, no adjusted preview → `null`; P in the same view keeps its composition |
| C2(a) | `lv10_c2a_a_save_from_the_phone_is_the_macs_own_composition` | yes: 200 `{revision: 1}`, stored values + origin user, undo label "Compose 01", events `PhotoChanged{P,S}` then `SessionChanged{S}` exactly, one undo → default values |
| C2(b) | `lv10_c2b_unchanged_values_save_nothing` | yes: 200 `{revision: 0}`, same undo label as before, no event (see Assumption 1) |
| C3(a) | `lv10_c3a_an_invalid_framing_is_refused` | yes: 400 `composition_invalid`, composition unchanged at revision 0 |
| C3(b) | `lv10_c3b_a_framing_changed_on_the_mac_is_not_overwritten` | yes: 409 `composition_changed`, stored = the Mac's; the posted values are valid on their own |
| C3(c) | `lv10_c3c_the_guard_and_the_photos_session_refuse` | yes: 409 `session_changed`; 404 `not_found`; both compositions unchanged, no event |
| C4(a) | `lv10_c4a_reset_saves_the_default_framing` | yes: 200 `{revision: 2}`, default values + origin default, one undo → C2(a)'s values |
| C4(b) | `lv10_c4b_a_stale_reset_is_refused` | yes: 409 `composition_changed`, stored unchanged at revision 1 |
| C5(a) | `lv10_c5a_two_fingers_spread_zoom_the_picture_about_the_point_between_them` | yes: `translate(0px, 0px) scale(2)` during, after release and after 200 ms; no POST |
| C5(b) | `lv10_c5b_below_1x_it_stretches_a_little_and_springs_back` | yes: `scale(0.8)` during, `scale(1)` after |
| C5(c) | `lv10_c5c_above_6x_it_settles_at_6x` | yes: `scale(7)` while down on the third spread, `scale(6)` after |
| C6(a) | `lv10_c6a_one_finger_moves_a_zoomed_picture_and_never_swipes` | yes: `translate(-100px, 0px) scale(2)`, no hint, no POST, same photo |
| C6(b) | `lv10_c6b_the_picture_never_moves_past_its_edge` | yes: `translate(-195px, 0px) scale(2)` during and after |
| C6(c) | `lv10_c6c_a_zoomed_template_photo_is_not_held_for_its_template` | yes: no pill after 300 ms, no `vibrate(10)` |
| C6(d) | `lv10_c6d_a_double_tap_zooms_to_3x_about_the_tap_and_back_to_1x` | yes: `translate(190px, 0px) scale(3)`, then `translate(0px, 0px) scale(1)` |
| C7(a) | `lv10_c7a_a_second_finger_ends_a_swipe_and_starts_a_pinch` | yes: hint gone and the photo's transform empty after the second press; `scale(1.4444)`; no POST |
| C7(b) | `lv10_c7b_a_cancelled_finger_or_a_blur_ends_the_pinch_where_it_is`, `lv10_c7b_a_window_blur_ends_the_pinch_where_it_is` | yes (two tests, one per input): `scale(2)` stays, later moves and lifts change nothing, no POST |
| C7(c) | `lv10_c7c_another_photo_shows_at_1x` | yes: 203 shown with `scale(1)` |
| C8(a) | `lv10_c8a_the_preview_shows_a_composed_photo_as_composed` | yes: viewport 390 × 487.5; `composed-frame` width `187.4634%`, the layout's frame, `rotate(0deg)` |
| C8(b) | `lv10_c8b_a_strip_tile_shows_a_composed_photo_as_composed` | yes: the tile's `composed-frame` has the same frame style |
| C8(c) | `lv10_c8c_a_photo_without_a_composition_is_drawn_as_before` | yes: no `composed-frame`; an `<img>` with LV9's classes in the Preview and in the tile |
| C9(a) | `lv10_c9a_reframe_opens_like_the_macs_editor` | yes: dialog "Reframe"; frame 24 / 89 / 342 × 342; the picture's transform string equal to the Mac's for K; 4 Thirds lines |
| C9(b) | `lv10_c9b_reframe_waits_for_the_full_frame_preview` | yes: Reframe disabled, the waiting line shown, a click opens nothing |
| C9(c) | `lv10_c9c_a_photo_that_needs_review_opens_refitted_to_its_frame` | yes, on a fixture whose stored rect differs from its refit (Assumption 12) |
| C10(a) | `lv10_c10a_one_finger_moves_the_photo_under_the_frame` | at display precision (≈ 1e-5; see Deviations); 1e-6 in C14(a) |
| C10(b) | `lv10_c10b_two_fingers_spread_zoom_about_the_point_between_them` | at display precision; `zoomAbout`'s output pinned to the row's numbers at 1e-6 |
| C10(c) | `lv10_c10c_a_twist_straightens_after_3_degrees_and_shows_the_grid_meanwhile` | yes: grid during (24 px), none after, "7.0°", the Mac's refit for 7° |
| C10(d) | `lv10_c10d_a_small_twist_changes_nothing` | yes: the grid never showed, "0.0°", K shown |
| C11(a) | `lv10_c11a_the_photo_stops_at_its_edge` | at display precision, during the move and after; exact (1e-9) in `lv10_core_panWithin_stops_exactly_at_the_frames_edge` |
| C11(b) | `lv10_c11b_a_straightened_photo_stops_on_the_images_edge` | yes: shows `panWithin`'s output; that output is valid and 1 % further is not |
| C11(c) | `lv10_c11c_zooming_out_stops_at_the_largest_frame` | yes: K shown |
| C11(d) | `lv10_c11d_a_cancelled_move_keeps_what_it_showed` | yes: C10(a)'s framing stays; later events of that finger change nothing; no request |
| C12(a) | `lv10_c12a_split_makes_the_frame_two_squares` | yes: the Mac's refit; frame 342 × 171; the middle line; Split pressed |
| C12(b) | `lv10_c12b_shape_and_turn_refit_the_frame` | yes: the seven shapes in order; both refits |
| C12(c) | `lv10_c12c_rotate_and_flip_from_one_menu_that_closes_after_a_pick` | yes: the three entries; both refits; the picture's turn and flip; the menu closed after each pick |
| C12(d) | `lv10_c12d_the_straighten_control_shows_the_grid_while_it_is_held` | yes: grid while held, "2.5°", the refit, no grid after |
| C12(e) | `lv10_c12e_the_guides_are_chosen_and_remembered` | yes: the five names; the 4 golden lines; still golden after reopening; the storage key |
| C13(a) | `lv10_c13a_reset_puts_the_macs_default_framing_in_the_editor` | yes: `resetValues`' output shown |
| C13(b) | `lv10_c13b_cancel_discards_the_framing` | yes: no POST; reopens with K |
| C14(a) | `lv10_c14a_done_saves_the_framing_into_the_macs_composition` | yes: one POST, the exact body (rect at 1e-6), editor closed, toast, then `GET /session` |
| C14(b) | `lv10_c14b_done_without_a_change_sends_nothing` | yes: no request at all, editor closed, toast empty |
| C14(c) | `lv10_c14c_done_after_reset_asks_the_mac_for_its_own_reset` | yes: the exact reset POST |
| C14(d) | `lv10_c14d_one_request_per_done` | yes: one POST while held and after; Done disabled meanwhile |
| C15(a) | `lv10_c15a_a_framing_changed_on_the_mac_closes_the_editor_and_says_so` | yes |
| C15(b) | `lv10_c15b_a_refused_framing_stays_in_the_editor` | yes: open, C10(a)'s framing still shown, toast, Done usable again |
| C15(c) | `lv10_c15c_a_session_changed_on_the_mac_closes_the_editor_and_rereads` | yes: closed, toast, `GET /session` after the POST |
| C15(d) | `lv10_c15d_the_editor_closes_when_its_photo_leaves_the_session` | yes: editor and Preview gone, no POST |
| C16(a) | `lv10_c16a_the_item_drawer_has_a_guides_row_off_the_first_time` | yes: the five buttons in a group named "Guides", Off pressed, no lines |
| C16(b) | `lv10_c16b_thirds_draws_the_thirds_over_the_live_view` | yes: the four lines at 130 / 260, over the template and over live |
| C16(c) | `lv10_c16c_the_choice_is_remembered_on_the_phone` | yes: unmount, start again: the same lines, the key "thirds" |
| C16(d) | `lv10_c16d_a_split_shot_has_the_guides_in_each_half` | yes: 8 lines = `overlayLines('thirds', 610 × 305, true)`, all inside |
| C16(e) | `lv10_c16e_the_guides_take_no_touch` | yes: one `/focus` POST after the locked tap; `pointer-events: none` |

Core mechanisms (`__tests__/lv10_core.test.ts`, 21 tests): `twoFinger` 3 · `pinchView` 2 · `clampPan` 2 · `settleZoom` 1 ·
`track` 5 · `panWithin` 3 · `zoomWithin` 2 · `rotateWithin` 1 · `straightenFromTwist` 1 · `resetValues` 1.

**Red baseline: not captured.** Every unit's tests were written after its code. Seen red while working: C2(b) before the
unchanged-values rule; `lv7_c2a` (an inherited tripwire, satisfied by moving new CSS below the loading cue's
reduced-motion block); C10(a) and C11(a) with deliberately wrong expected rects (test file changed and restored, no
production file touched).

**No row, no test:** the codes 409 `frame_unknown` and 400 `invalid_value` (a body that does not parse). U1's Goal
names them; no criterion row does, so no test was added (the mappings are in `wire::error_code` and `routes::body`).

## Assumptions
One sentence each for the narrative; the full reasoning is in the plan's Review log.
1. Sending back the framing a photo already has changes nothing on the Mac, even for a photo nobody has framed yet (the
   Mac's own `set` would mark it "framed by hand"); needed for row C2(b).
2. The Mac checks "is this still the framing you opened" and saves in two steps; two saves landing in the same instant
   could both pass (the plan accepts it).
3. The phone treats a missing framing or default shape from the Mac as "none" and "1:1" (the Mac always sends both;
   this keeps earlier test fixtures unchanged).
4. A move stops exactly at the photo's own edge; at a straightened photo's slanted edge it stops within 1/65536 of the
   move. A diagonal move that reaches an edge stops both ways.
5. With two fingers the photo stays under the fingers while zooming and moving at once.
6. While the straighten slider is held, sliding back gives the earlier frame back.
7. The zoom settles in 0.2 s; a double tap eases too; a touch that moved less than 14 px on a zoomed photo is a tap;
   composed strip tiles have no crop mark.
8. A framed photo of a split template shot shows no dashed template line over it (a split framing shows its own line).
9. The editor's menus are small sheets over the tool row that close after a pick; outside the frame is dimmed white;
   controls give a light buzz on Android.
10. "Framing changed on the Mac" also re-reads the session, so the next Reframe opens on the Mac's framing; Cancel does
    nothing while Done is being answered.
11. The Guides row sits above ACTIONS in the menu; the guides also show in Free shot.
12. The "needs review" test uses a stored frame that no longer has its shape, so the re-fit is visible.

## Deviations
- `ImageFrame` (in `Stage.tsx`) accepts children as a function of the viewport's size; the Preview is unchanged.
- U4 has three commits: `1764d2d6` failed `npm run typecheck` (an unused test helper; I committed before reading the
  check's output), `21ffb636` fixes it, `764ba540` renames the drawer's group label to "Guides".
- Editor rows read a rect from the picture's transform (0.01 px, about 1e-5 of the frame), not at the plan's 1e-6; the
  1e-6 values are in `lv10_core` and in C14(a)'s request body.
- C7(b) is two tests.

## Files changed
Write perimeter of this session (`git diff --stat 062f6d38 814990a9`: 30 files). No archgraph, no tool-recorded state,
no mutation probes on production files. One test file was edited and restored (`lv10_reframe.test.tsx`, the wrong-value
check above).

Code: `crates/core/src/error.rs` · `crates/core/src/services/composition.rs` · `crates/core/src/services/phone_view.rs` ·
`crates/integrations/src/phone/routes.rs` · `crates/integrations/src/phone/wire.rs` ·
`apps/desktop/src-tauri/src/commands/items.rs` · `apps/desktop/src/phone/{PhoneApp.tsx, api.ts, gestures.ts, state.ts,
phone.css}` · new `apps/desktop/src/phone/{pinch.ts, reframe.ts}` · `apps/desktop/src/phone/components/{ItemDrawer.tsx,
PhotoDrawer.tsx, PhotoImage.tsx, Preview.tsx, Stage.tsx}` · new `apps/desktop/src/phone/components/ReframeEditor.tsx`.

Tests: new `crates/testkit/tests/suite/lv10_phone_composition.rs` (+ its `mod` line in `main.rs`) · new
`apps/desktop/src/phone/__tests__/{lv10_core.test.ts, lv10_zoom.test.tsx, lv10_reframe.test.tsx,
lv10_save_and_guides.test.tsx}` · `apps/desktop/src/phone/__tests__/helpers.ts` (fixtures K, K45; `REFRAME_STAGE` in the
layout stub).

Documents: `docs/guide/{01_product.md, 02_architecture.md, 04_change_impact.md}` ·
`docs/implementation/plans/LV10_zoom_reframe_guides.md` (Tracker line, Review log) · this handoff (not committed: the
runner commits handoffs).

The Mac's composition modules (`features/compose/*`) are imported, not edited. Nothing in `src/phone` imports
`@/services/bindings` or `@tauri-apps/*`.

## Checkpoint SHA
In order: U1 `05729f61` · U2 `f594e337` · U3 `92dfbef3` · U4 `1764d2d6`, `21ffb636`, `764ba540` · close `814990a9`
(plan file only). `git status --porcelain` is empty at `814990a9` except this handoff file.

## Stamp
Run once, one command after the other, on the clean tree `764ba540` (the close commit changes no code).

| Command | Result |
|---|---|
| `cd apps/desktop && npm run build` | ok; `dist/assets/phone-*.js` 67.96 kB; `__TAURI` and `tauri` occur 0 times in it and in the chunk it shares with the desktop entry |
| `cargo test --workspace` | exit 0; 2791 passed, 0 failed, 0 ignored (LV9's stamp 2782 + this phase's 9); 5 min 21 s |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo fmt --all -- --check` | exit 0 |
| `tools/sidecar/build.sh --stub --test` | exit 0; ctest 100 % passed out of 2 |
| `cd apps/desktop && npm run typecheck` | exit 0 |
| `npm test` | exit 0; 162 files, 1238 passed, 0 failed (LV9's 1172 + 66) |

No pre-existing failure. Failure-ID delta against LV9's stamp: none in either direction.

## Manual try-out
On this Mac with the camera connected and the phone paired (the app built from this branch: `cd apps/desktop && npm run
build`, then the app), a templated session open, some photos taken and their previews ready (a photo can be reframed
once Capture One's preview of it exists; until then Reframe is off and says why):
1. Open a photo's preview and spread two fingers: it zooms. Move it with one finger; it stops at its edges. Double-tap:
   back to full view. Double-tap again: 3× where you tapped. While zoomed, swiping does nothing.
2. Tap ⋯ → Reframe. Move the photo with one finger, spread to zoom, twist two fingers to straighten: a fine grid shows
   while you turn and the frame never shows an empty corner. Tap Guides → Golden ratio. Try Split, Shape → 4:5, Turn,
   the round arrow → Rotate 90° right.
3. Tap Cancel: nothing changed. Reframe again, change it, tap Done: "Framing saved"; the preview and the strip show the
   new framing; on the Mac the photo shows it too, and ⌘Z there undoes it.
4. Reframe a photo on the phone, and before Done change its framing on the Mac; then Done on the phone: "Framing
   changed on the Mac. Open Reframe again." and the Mac's framing is untouched.
5. On the live screen open the menu (≡), Guides → Thirds: the lines show over the template and over live; close the app
   and open it again: still Thirds.

What to look at closely, because no browser has drawn it yet: a framed photo in the strip tiles and in the preview
(especially one turned 90° or straightened), the picture following the fingers in the editor, and the grid's and the
guide lines' visibility on light and dark photos.

## For the coordinator
1. **A photo that needs review is not confirmed by an unchanged Done on the phone.** PH12 says "values equal to the open
   values → close, no request", and the editor opens such a photo already re-fitted, so Done sends nothing and the Mac
   keeps asking to re-frame. Book 10D (MC-27 §Revisions) has the Mac's editor confirm with Return. If the phone should
   confirm too: PH12 + "…unless the photo needs review" and one row; the change is one condition in
   `ReframeEditor.tsx::done`.
2. **How the bounded moves feel (PH12 verbatim), for the owner's try-out:** right after straightening, the frame touches
   the photo's edge, so the photo barely moves until it is zoomed in a little; zooming out with the frame at an edge
   stops there (the Mac's editor slides the frame inward instead); a diagonal move that reaches an edge stops both ways.
3. `frame_unknown` and `invalid_value` on the two new routes have no row (above).
4. Process: one commit of mine went in before I read its typecheck result (fixed in the next commit); the review's
   perimeter is `05729f61`…`764ba540` plus the plan file in `814990a9`.
