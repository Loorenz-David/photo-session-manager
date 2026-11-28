---
plan: LV9_preview_corrections
role: implement
round: 1
state: DONE
date: 2026-11-27
actor: fable
---
# Handoff — LV9_preview_corrections, implement r1

All four units are built and committed; the closing gate is green on the tree handed over (`7b9d7df1`, clean; the
close commit `1e4a1fbf` adds only the plan file's Tracker line and Review log). 44 criterion rows → 53 tests. **Nothing
was seen on a phone or in a browser**: every row is proven in jsdom or at the Rust boundary, and the looks (the cue's
strength, the template picture under a lifted photo, the Preview's bar and drawer) are CSS written by reasoning.

## ⚠ OWNER DECISIONS REQUIRED (0)
None. Eight judgment calls are listed under Assumptions for the narrative's "Choices made for you".

## What was built
In product terms first.
1. **The cue from the shutter.** The moment the camera reports a shutter, the shot it was taken for shows the quiet
   light-over-grey cue in place of its template picture (number kept), in the live screen's strip and in the Preview's
   strip. A capture going to the Condition images (or to the end of Main in a session without a template) shows a
   waiting tile at the end of that group. When the photo exists, its own tile keeps the cue until its picture is
   there. Waiting tiles take no tap, never lift, are no drop target, and one appearing never cancels a drag. The cue is
   a little stronger than LV7's; a picture that failed to load after its retries is plain grey.
2. **Nothing blinks when photos move.** A drop and a swipe show in both strips at once; several quick ones stack and
   the strips keep showing them until the Mac has answered every one; a refusal or 8 s without an answer puts the
   strips back to the Mac's view. A moved photo keeps its picture; a picture is fetched again only when its file
   changed on the Mac.
3. **The template picture stays.** While a template shot's photo is lifted, its place shows that shot's grey template
   picture with its number; extras and Condition images leave a gap as before.
4. **The minimal Preview.** Only the photo, the strip, ⋯ (bottom left) and × (bottom right). ⋯ opens a drawer with what
   the header showed, the hold hint and Retake for a template shot's photo. × closes. Move image is gone.

In code:
- **U1** `crates/core/src/services/phone_view.rs`: `PhoneShot.incoming`, `PhoneSession.incoming: PhoneIncoming {extras,
  condition}`, read once at `now` inside `phone_view`'s transaction from `current_shot::shot_plan` (`held`; new
  `ShotPlan::held_free`) or, untemplated, `StoreTx::hold_times`. No route, event, schema or camera change (H8).
- **U2** `api.ts` (types), `state.ts` (`DockTile` kind `incoming`, keys `wextra<n>` / `wcondition<n>`, `isWaiting`,
  `INCOMING_LABEL`), `components/Dock.tsx` (waiting look, `data-incoming`, no tap, not in homes),
  `components/PhotoImage.tsx` (`data-gave-up`), `phone.css` (the cue's grey and peak, the gave-up rule).
- **U3** `state.ts` (tile key `p<photo_id>` for every tile that shows a photo), `api.ts::previewUrl` (no `r=`),
  `PhotoImage` (key `photo_id:preview`), `arrange.ts` (`Removal`, `applyDrop` kind `remove`), `PhoneApp.tsx` (the
  pending chain: `Chain`, `ask`, `answered`, `refusedAhead`, `shownNow`; drops and swipes), `Dock.tsx` (one flat keyed
  list; the lift home `data-testid="lift-home"`), `phone.css`, `helpers.ts` (`stripFrame`, `stripFrames`).
- **U4** `components/Preview.tsx` (root `aria-label` = kicker; no header, resting pill, hint or button block; a bar
  with "Photo details" and "Close"; `enabled`), new `components/PhotoDrawer.tsx`, `state.ts` (`detailsOpen`, action
  `details`; `moveTiles`, `MoveTile`, `backLabel`, action `move` removed), `PhoneApp.tsx`, `phone.css`;
  `components/MoveDrawer.tsx` removed.

## Tests → criterion rows
One line per row. Rust: `crates/testkit/tests/suite/lv9_phone_incoming.rs`. UI: `apps/desktop/src/phone/__tests__/`.
"Shape" = the test asserts what the row's outcome cell says.

| Row | Test | Shape |
|---|---|---|
| C1(a) | `lv9_c1a_a_reported_shutter_marks_its_shot_incoming` | as the row |
| C1(b) | `lv9_c1b_a_filed_file_is_still_on_its_way` | as the row |
| C1(c) | `lv9_c1c_the_photo_ends_incoming` | as the row |
| C2(a) | `lv9_c2a_a_free_shot_on_its_way_counts_for_condition` | as the row |
| C2(b) | `lv9_c2b_an_untemplated_shutter_counts_for_main` | as the row |
| C2(c) | `lv9_c2c_a_file_that_never_arrives_stops_being_incoming` | as the row |
| C3(a) | `lv9_c3a_the_session_route_carries_incoming` (through `TestPhone`) | as the row |
| C3(b) | `lv9_c3b_a_denied_photo_is_not_on_its_way` | as the row |
| C4(a) | `lv9_c4a_a_shot_on_its_way_shows_the_cue_and_keeps_its_number` | as the row |
| C4(b) | `lv9_c4b_the_previews_strip_shows_the_same_cue` | as the row |
| C4(c) | `lv9_c4c_a_shot_on_its_way_takes_no_tap` | as the row |
| C4(d) | `lv9_c4d_the_cue_stays_from_the_shutter_until_the_picture` | as the row |
| C5(a) | `lv9_c5a_a_group_shows_one_waiting_tile_per_photo_on_its_way`, `lv9_c5a_an_untemplated_session_waits_at_the_end_of_main` | as the row (two fixtures, two tests) |
| C5(b) | `lv9_c5b_a_waiting_tile_never_lifts_and_takes_no_tap` | as the row (lifted read after 600 ms and after release) |
| C5(c) | `lv9_c5c_a_waiting_tile_is_not_a_drop_target` | as the row; see note 2 |
| C5(d) | `lv9_c5d_a_photo_coming_on_its_way_does_not_cancel_a_drag` | as the row |
| C6(a) | `lv9_c6a_a_picture_that_failed_every_try_stops_shimmering` | as the row (the CSS rule is read from `phone.css`) |
| C6(b) | `lv9_c6b_a_new_picture_after_giving_up_is_asked_for` | as the row |
| C7(a) | `lv9_c7a_a_swap_keeps_both_pictures` | as the row (element identity, after the drop and after the re-read) |
| C7(b) | `lv9_c7b_filling_an_empty_shot_keeps_the_picture` | as the row |
| C7(c) | `lv9_c7c_a_move_on_the_mac_keeps_every_picture` | as the row |
| C7(d) | `lv9_c7d_a_new_revision_alone_fetches_nothing` | as the row |
| C8(a) | `lv9_c8a_a_denied_photo_leaves_both_strips_at_once` | as the row ("no further change" = one frame per strip) |
| C8(b) | `lv9_c8b_a_photo_made_main_shows_at_the_end_of_main_at_once`, `lv9_c8b_in_the_previews_strip_too` | the row's fixture shows the live strip only; see note 1 |
| C8(c) | `lv9_c8c_a_retaken_shot_is_empty_at_once` | as the row (both strips) |
| C8(d) | `lv9_c8d_a_refused_swipe_puts_the_photo_back_in_the_strips` | as the row |
| C8(e) | `lv9_c8e_swipes_stack_while_the_mac_answers` | as the row |
| C9(a) | `lv9_c9a_a_drop_reaches_the_strip_in_one_step` | as the row (frames) |
| C9(b) | `lv9_c9b_a_slow_read_never_flashes_back` | as the row (answer at 7.9 s, read at 8.5 s) |
| C9(c) | `lv9_c9c_a_drop_then_a_swipe_stack_in_two_steps` | as the row |
| C9(d) | `lv9_c9d_a_change_from_the_mac_is_one_step` | as the row |
| C10(a) | `lv9_c10a_a_lifted_template_shots_photo_leaves_its_template_picture` | as the row, except "at B's home": jsdom has no layout, the place is not asserted |
| C10(b) | `lv9_c10b_a_shot_without_an_example_leaves_its_number` | as the row |
| C10(c) | `lv9_c10c_a_lifted_extra_leaves_a_gap` | as the row |
| C10(d) | `lv9_c10d_the_template_picture_goes_when_the_gesture_ends`, `lv9_c10d_and_when_the_phone_takes_the_touch_away` | as the row (drop; cancel) |
| C11(a) | `lv9_c11a_the_preview_is_the_photo_its_strip_and_two_buttons` | as the row (the buttons are asserted as the exact list) |
| C11(b) | `lv9_c11b_the_close_icon_closes_the_preview` | as the row |
| C11(c) | `lv9_c11c_hold_and_swipes_keep_working` | as the row |
| C12(a) | `lv9_c12a_the_drawer_reads_what_the_header_did_and_offers_retake`, `lv9_c12a_an_extra_has_no_hint_and_no_retake` | as the row |
| C12(b) | `lv9_c12b_retake_from_the_drawer_reopens_the_shot` | as the row |
| C12(c) | `lv9_c12c_its_close_button_closes_only_the_drawer`, `lv9_c12c_so_does_the_backdrop` | as the row |
| C13(a) | `lv9_c13a_the_photo_and_the_strip_take_no_touch_under_the_drawer` | as the row |
| C13(b) | `lv9_c13b_the_drawer_closes_with_its_preview` | as the row |
| C13(c) | `lv9_c13c_move_image_is_nowhere` | as the row (page text on the live screen, the Item drawer, three Previews and their drawers; the file check) |
| core `arrangementKey` | `lv9_core_arrangementkey_ignores_photos_on_their_way` | plan's Core mechanisms |
| core `applyDrop` `remove` | `lv9_core_applydrop_remove_a_template_shots_photo_leaves_its_shot_empty`, `…_an_extra_leaves_the_extras`, `…_a_condition_image_leaves_the_condition_images` | plan's Core mechanisms |

Every test in the four new UI files and the new Rust file is in this table (45 UI + 8 Rust = 53); none is unmapped.

**Red baseline.** U3: the 19 `lv9_one_step` tests and the 3 `applyDrop` core cases were written before the change;
21 of 23 in those two files failed (green before: `lv9_core_arrangementkey…`, which is U2's, and `lv9_c10c`, an
absence row that was already true). U4: 11 of 11 `lv9_preview` tests failed before the change. **U1 and U2: no red
baseline was taken** (tests written after the code).

Notes:
1. **C8(b).** The row says the *strips'* keys after a right swipe on 207. 207 is the last photo, so the Preview
   closes (LV8·C6(c)) and only the live strip is left to read. The second test makes the same swipe from 203, where
   the Preview stays open, and reads both strips.
2. **C5(c)** holds by PH9.3's span alone (the waiting tile stands past the last Condition tile). What keeps waiting
   tiles out of insert places and swap targets is the homes filter in `Dock.liftAt`; that row does not discriminate
   it.

## Tests changed or removed (earlier phases)
| File | Test | Change | Reason |
|---|---|---|---|
| `lv3_main.test.tsx` | fixture `withFourth` | `incoming: false` on a hand-built shot | U2: the type requires the field |
| `lv7_loading_cue.test.tsx` | `lv7_c2a` | the "moving" filter skips `animation: none` | U2: the gave-up rule C6(a) requires; **not in the plan's list** |
| `lv3_dock.test.tsx` | `lv3_c15a`, `lv3_c17b` | expected URL loses `&r=1` | U3 F2 (plan: 2) |
| `lv3_drawers.test.tsx` | `lv3_c23b_the_preview_stays_open…` | expected URL loses `&r=1` (then removed in U4) | U3 F2 (plan: 1) |
| `lv3_preview.test.tsx` | `lv3_c21a` | expected URL loses `&r=1` | U3 F2 (plan: 1) |
| `lv6_images.test.tsx` | `lv6_core_previewUrl…` (2), `lv6_c4b` (2), `lv6_c4c` (2), `lv6_c5a` (1) | expected URL loses `&r=3` | U3 F2 (plan: 7) |
| `lv7_loading_cue.test.tsx` | `lv7_c1b` | expected URL loses `&r=3` | U3 F2 (plan: 1) |
| `lv8_drag_reliability.test.tsx` | constants `AS_GIVEN`, `DROPPED`, `OTHERWISE` | `s12`, `s11` → `p201`, `p204` | U3 F1 (a filled shot's key is its photo's) |
| `lv8_drag.test.tsx` | `lv8_c9b`, `lv8_c10a`, `lv8_c11b` | `s12` → `p201`, `s11` → `p204` | U3 F1 |
| `lv3_preview.test.tsx` | `lv3_c20a` (two), `lv3_c20b`, `lv3_c21a`, `lv3_c21b`, `lv3_c21c`, `lv3_c22a`, `lv3_c22b` | kicker read from the Preview's `aria-label`; title, chip and hint read in the photo drawer; Back → "Close"; Retake clicked in the drawer; the resting pill assertions go | U4 (plan) |
| `lv3_drawers.test.tsx` | `lv3_c23a`, `c23b` (two), `c23c`, `c23d`, `c23e`, `c23f` | removed, with the helpers only they used | U4, P7 (plan) |
| `lv8_swipe.test.tsx` | `kicker` helper; `lv8_c5a`, `lv8_c5b` (no resting pill); `lv8_c6f` (Back → "Close"); `lv8_c8e` removed | | U4 (plan; C8(e) superseded by C11(a), C12(a)) |
| `lv6_images.test.tsx` | `lv6_c5b_the_move_drawer_shows_the_placeholder…` | removed | U4, P7; **not in the plan's list** |
| `lv6_dock.test.tsx`, `lv8_drag.test.tsx`, `lv8_drag_reliability.test.tsx` | `kicker` helper | reads the Preview's `aria-label` | U4; **files not in the plan's list** |
| `lv7_loading_cue.test.tsx` | `lv7_c3a` (last line) | reads the Preview's `aria-label` | U4; **not in the plan's list** |

`lv8_c15d` was **not** changed and is green: PH9.5a (c) keeps its outcome.

## Assumptions
Each is the smallest option I found consistent with 10J LV-D21 and LV-D20.
1. **What counts as "on its way" to a group.** Only shutters bound to no container count for the Condition waiting
   tiles (a binding to a container that left the template counts as none). A shutter bound to a shot that already
   holds a photo shows nothing; an unbound shutter in a templated session shows nothing until its file arrives.
2. **The cue's grey is a step darker.** White is only about 6 % lighter than the fill grey (#f1f1ef), so a peak
   "8–12 % above `--ph-fill`" cannot be drawn on it. The placeholder's grey is now the chip border's (#e2e2e0) and the
   light is white at 80 %: about 10 % lighter than the grey it crosses. Every placeholder is that grey, the Preview's
   "Preparing preview…" included.
3. **A waiting shot is no drop target** (left out of the homes), besides taking no tap and never lifting.
4. **The template picture under a lifted photo goes the moment the gesture ends**; during a cancelled drag's
   spring-back (under 0.2 s) the place is a gap.
5. **A swipe asks the Mac by where the Mac says the photo is** (plan note); only what the strips show and the next
   photo follow the pending arrangement. A swipe on a photo that an earlier, unanswered change already moved may be
   refused by the Mac: the strips go back and the phone says so.
6. **Retake from the drawer keeps LV3's order**: the Mac is asked, then the toast, and the drawer and Preview close.
7. **The drawer shows ACTIONS only when it has one** (a template shot's photo). The bar sits under the strip with the
   design's 52 px round buttons; ⋯ gives the menu's short buzz; the swipe hint ("DENY", "MAIN" …) stays as feedback
   during the gesture, like the "TEMPLATE" pill.
8. **While the "Move this photo to Main…?" question is open, the Preview's picture takes no touch** (before, only its
   strip was off).
Also: the waiting tile has no glyph and no number; no preloading of the next photo's picture in the Preview; the
pending chain is a ref plus one state in `PhoneApp`.

## Deviations
1. `lv7_c2a` changed, not in the plan's list (see the table): the plan's own C6(a) rule made it red.
2. `lv6_c5b` removed and four `kicker` reads changed outside the plan's list (see the table): they depended on the
   Move drawer and on the header's kicker element.
3. `lv8_c15d` kept as is (the plan calls C15(d) superseded; its outcome still holds under PH9.5a (c)).
4. `lv9_core.test.ts` was created in U2, not U3 (its `arrangementKey` case is U2's).
5. `crates/core/src/services/current_shot.rs` edited (the plan allows it "only if a read helper is shared"):
   `ShotPlan` gained `held_free`, counted in the existing hold loop.
6. `docs/guide/02_architecture.md` also edited in U4 (it named `MoveDrawer` and `moveTiles`).
7. Closing-gate output was written to files under `.pipeline/runs/` (git-ignored) instead of the terminal; every
   command ran in the foreground, one at a time, none piped.
8. Actor `fable`, not the prompt's `codex`.

## Files changed
Write perimeter of this session (`git diff --stat e39e6f2f 1e4a1fbf`: 33 paths) plus this handoff.
- Rust: `crates/core/src/services/phone_view.rs`, `crates/core/src/services/current_shot.rs`,
  `crates/testkit/tests/suite/lv9_phone_incoming.rs` (new), `crates/testkit/tests/suite/main.rs`.
- Phone: `apps/desktop/src/phone/{api.ts, arrange.ts, state.ts, PhoneApp.tsx, phone.css}`,
  `components/{Dock.tsx, PhotoImage.tsx, Preview.tsx}`, `components/PhotoDrawer.tsx` (new),
  `components/MoveDrawer.tsx` (removed).
- Phone tests: `__tests__/{lv9_core.test.ts, lv9_incoming.test.tsx, lv9_one_step.test.tsx, lv9_preview.test.tsx}`
  (new), `__tests__/helpers.ts`, and the changed earlier files of the table above (`lv3_dock`, `lv3_drawers`,
  `lv3_main`, `lv3_preview`, `lv6_dock`, `lv6_images`, `lv7_loading_cue`, `lv8_drag`, `lv8_drag_reliability`,
  `lv8_swipe`).
- Docs: `docs/guide/01_product.md`, `docs/guide/02_architecture.md`, `docs/guide/04_change_impact.md`,
  `docs/implementation/plans/LV9_preview_corrections.md` (Tracker line, Review log entry).
- This handoff (in no checkpoint commit; left for the runner).
- Not committed, git-ignored: `.pipeline/runs/lv9_implement_*.log` (gate output), `apps/desktop/dist/` and the stub
  sidecar binary rebuilt by the gate.

No mutation probes; no file touched and reverted. No archgraph in this repo.

## Checkpoint SHA
Branch `feature/live-preview`, parent `e39e6f2f`. In order:
| Unit | SHA |
|---|---|
| U1 | `75c1b755` |
| U2 | `59cefe03` |
| U3 | `7ad59d7f` |
| U4 | `7b9d7df1` |
| close (plan file: Tracker line, Review log) | `1e4a1fbf` |

Tree clean after `1e4a1fbf` except this handoff.

## Stamp
The closing gate, once, in the plan's order, on `7b9d7df1` with `git status --porcelain` empty. The close commit after
it changes no code.
| Command | Result |
|---|---|
| `cd apps/desktop && npm run build` | built; `dist/phone.html` written |
| `cargo test --workspace` | exit 0; 2782 passed, 0 failed, 0 ignored (LV8's stamp 2774 + this phase's 8); 5 min 19 s |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo fmt --all -- --check` | exit 0 |
| `tools/sidecar/build.sh --stub --test` | exit 0; 2 of 2 passed |
| `cd apps/desktop && npm run typecheck` | exit 0 |
| `npm test` | exit 0; 158 files, 1172 passed, 0 failed (LV8: 154 files, 1136 tests; +4 files, +45 tests, −9 removed) |

No pre-existing failure showed this time (the recorded `p33_split_grid` flake passed). Unit test commands ran after
each unit: `cargo test -p psm-testkit --test suite lv9_phone_incoming::` 8 passed, `cargo check -p psm-integrations`
clean (U1); `npx vitest run src/phone` and `npm run typecheck` green after U2 (16 files, 193), U3 (17 files, 215) and
U4 (18 files, 217).

**Not verified:** anything on a real phone or in a real browser. In particular: how strong the cue reads on the
darker grey; that the template picture sits exactly on the lifted tile's place (it is placed from the tile's
`offsetLeft` / `offsetTop` in the row); the bar and the drawer on a 390 × 844 screen; that a moved tile's picture is
not decoded again by Safari or Chrome (the tests prove the `<img>` element and its `src` are kept).

## Manual try-out
On this Mac with the camera connected (Wi-Fi shows it best) and the phone paired to an app built from this branch, a
templated session open:
1. Take a picture from the phone. At once the shot you took shows a light moving over grey in the strip; open any
   photo's preview and its strip shows the same. A few seconds later the picture appears.
2. Take two pictures after the last template shot: two waiting tiles appear after the Condition images, then become
   the photos.
3. Press and hold a template shot's photo until it lifts: its place shows the shot's grey template picture with its
   number. Drag it onto another shot and let go: the two swap in one step, no blink, no old picture.
4. In a preview, swipe three Condition images left quickly, one after the other: each leaves the strip at once and
   nothing comes back.
5. The preview shows only the photo, the strip, ⋯ at the bottom left and × at the bottom right. Tap ⋯: the photo's
   details and Retake. Tap ×: back to live.
6. On the Mac, check each change in the session view (⌘Z brings back a denied photo).

## For the coordinator
- Candidate backlog note: the **Next** button still treats a shot whose photo is on its way as empty; choosing it is
  refused by the Mac as before LV9 (`state.ts::nextStep`). Outside this plan's rows.
- A photo arriving in a waiting shot gets a new tile element (key `s<container>` → `p<photo>`), so the sweep starts
  over once at that moment.
- LV10 builds on: the photo drawer's actions column (`PhotoDrawer`), the Preview's `enabled` prop and state machine,
  tile keys by photo, the pending chain (`ask` / `answered` / `refusedAhead`).
