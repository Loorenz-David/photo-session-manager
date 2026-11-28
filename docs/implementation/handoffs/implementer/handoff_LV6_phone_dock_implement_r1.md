---
plan: LV6_phone_dock
role: implement
round: 1
state: DONE
date: 2026-11-26
actor: fable
---
# Handoff — LV6_phone_dock, implement r1

Both units built and checkpointed, plus row LV6·C3(b) (the shorter lock swipe), which the orchestrator added to the
plan while this session was running (commit `67b7dcff`); the closing gate is green on the tree handed over. Nothing
needs the owner.
(The prompt's closing line names actor `codex`; this session ran on Claude Fable as the plan's `implementer: fable`
says, so the tracker and this file say `fable`.)

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## What was built
**For the photographer.** The phone's shot dock has bigger tiles (56 px, were 36) that slide sideways. One tap, short
or long, opens a taken photo's preview or makes an empty shot the current one; nothing magnifies, nothing is dragged
up. A photo whose preview the Mac has not made yet shows a plain grey tile with its number (in the full preview:
"Preparing preview…"), and the picture appears by itself; a picture that fails to load is tried again three times,
two seconds apart, and never shows as a broken image. A phone removed in Settings → Phone now reads: "This phone was
removed in Session Manager. Delete this app from the home screen, then scan the new QR code in Session Manager →
Settings → Phone and add it again." On the main screen, sliding up to lock live now takes half the distance (55 px,
was 110).

**In the code.**
- U1: `components/Dock.tsx` (pointer down / track / up: a tap is a touch within 8 px per axis; sideways scrolls;
  nearest tile within 32 px at release), `gestures.ts` (`DOCK = {tapPx, nearPx}`; `tileScale`, `TILE_PX`, hold
  thresholds and three haptics removed), `phone.css` (`.ph-dock-*`: 84 px strip, scroller the strip's height, 56 px
  tiles, 12 px radius, 8 px gap; label and magnifying rules removed).
- U1, row C3(b): `gestures.ts` `LOCK_PX` 110 → 55 (nothing else of the lock gesture changed).
- U2: `api.ts` (`PhoneImage.preview`, `previewUrl(token, image, attempt = 0): string | null`),
  `components/PhotoImage.tsx` (new; the only place a preview `<img>` is made), used in `Dock.tsx`, `Preview.tsx`,
  `MoveDrawer.tsx`; `phone.css` (`.ph-photo-wait`); `state.ts` (`TEXT.removed`); guide files.

## Tests → criterion rows
All in `apps/desktop/src/phone/__tests__/`. Boundary: `PhoneApp` rendered against the fake Mac (`fetch`, `WebSocket`).

| Row | Test | Asserts the row's shape? |
|---|---|---|
| LV6·C1(a) | `lv6_dock.test.tsx::lv6_c1a_tiles_are_56_px_with_12_px_corners_8_px_apart_in_an_84_px_strip` | yes: computed styles with the real `phone.css` loaded (6 tiles 56 × 56, radius 12; row gap 8; strip 84; tallest dock element 84) |
| LV6·C2(a) | `lv6_c2a_a_tap_on_an_extra_opens_its_preview` | yes ("IMAGE 04 · EXTRA"; vibrate calls exactly `[[15]]`; no POST) |
| LV6·C2(b) | `lv6_c2b_a_tap_on_a_condition_image_opens_its_preview` | yes |
| LV6·C2(c) | `lv6_c2c_a_long_press_is_a_tap_and_magnifies_nothing` | yes (600 ms hold: tile inline sizes/transforms unchanged, no label, nothing open; on release B's preview; vibrate calls exactly `[[15]]`) |
| LV6·C2(d) | `lv6_c2d_a_touch_that_moves_up_selects_nothing` | yes |
| LV6·C3(a) | `lv6_c3a_a_sideways_swipe_scrolls_and_the_next_tap_selects_the_tile_now_under_the_finger` | yes (`scrollLeft` +128; nothing selected; the tap posts `/current` for C, two pitches right of "02") |
| LV6·C3(b) | `lv6_dock.test.tsx::lv6_c3b_sliding_up_55_px_locks_live_and_50_px_does_not` | yes (50 px: "LIVE" while held, "TEMPLATE" and demand `live: false` after release; 55 px: "LIVE · LOCKED", still so 1 s after release, demand `live: true`; `lockProgress(-55)` = 1) |
| LV6·C4(a) | `lv6_images.test.tsx::lv6_c4a_a_photo_without_a_preview_shows_the_placeholder_and_asks_for_nothing` | yes |
| LV6·C4(b) | `lv6_c4b_the_picture_is_asked_for_by_its_preview_version_and_again_when_it_changes` | yes (whole URL asserted, stronger than "ends with") |
| LV6·C4(c) | `lv6_c4c_a_picture_that_fails_shows_the_placeholder_and_is_tried_three_more_times` | yes (no `<img>` after each error; nothing at 1999 ms, `&try=n` at 2000 ms, n = 1, 2, 3; then none for 10 s; `k2` without `try`) |
| LV6·C5(a) | `lv6_c5a_the_preview_says_preparing_until_the_photo_has_a_preview` | yes |
| LV6·C5(b) | `lv6_c5b_the_move_drawer_shows_the_placeholder_for_a_photo_without_a_preview` | yes |
| LV6·C6(a) | `lv6_c6a_a_phone_removed_before_it_starts_is_told_to_delete_the_app_and_add_it_again` | yes |
| LV6·C6(b) | `lv6_c6b_a_phone_removed_while_connected_is_told_the_same` | yes (socket closed with 4401) |
| Core mechanism `previewUrl` | `lv6_core_previewUrl_names_the_preview_version_and_the_retry_or_is_null_without_a_preview` | unit test, input → output (plan's Core mechanisms list; no criterion row) |

Reverse direction: the two LV6 files hold 15 tests = the 14 rows above + the core-mechanism test; no other test added.
**Red baseline: not captured** (tests were written alongside the code; not reconstructed).

**LV3 tests removed (the plan's six, exactly):** `lv3_c16a_tileScale_is_2_5_under_the_finger_and_1_from_95_px`,
`lv3_c16b_a_held_tile_is_magnified_and_names_itself`, `lv3_c18b_holding_200_ms_magnifies_with_a_tick_per_tile`,
`lv3_c19a_dragging_a_held_tile_up_past_50_px_opens_it_on_release`,
`lv3_c19b_bringing_the_tile_back_down_before_release_opens_nothing`,
`lv3_c19c_opening_an_empty_template_shot_selects_it`.
**LV3 tests kept, changed only for layout/fixture:** `lv3_c15a` (expected URL gains `&v=k0`), `lv3_c15b`, `lv3_c15c`,
`lv3_c17a`, `lv3_c17b` (URL), `lv3_c17c`, `lv3_c18a`, `lv3_c13a`, `lv3_c14f` (fixture `preview`).
**Re-texted:** `lv3_shell.test.tsx::lv3_c1c_a_removed_phone_forgets_its_token_and_says_so` (P4's text).
**Changed for the 55 px lock (C3(b), as the plan's Notes ask):** `lv3_geometry.test.ts::lv3_c8_lockProgress_and_overlayValue`
(`lockProgress` values for 55); in `lv3_main.test.tsx` the `lock()` helper (slides 55 px), `lv3_c8a` (fill 27 %, was
14 %), `lv3_c8b` (renamed `lv3_c8b_sliding_up_55_px_locks_live_and_it_stays_on_after_release`, slides 55 px) and
`lv3_c8c` (releases at 40 px, was 60: 60 px now locks).

## Assumptions
1. A tap is a touch that never went more than 8 px sideways or more than 8 px up/down (per axis); decided on release.
2. Sideways past 8 px scrolls from then on; a touch that moved and came back is still not a tap.
3. The nearest tile is measured at release; exactly 32 px away still counts.
4. Sizes are in `phone.css` only. Beyond P1: dock tags 10 px, check badge 14 px, divider 40 px, tile fill `#F1F1EF`.
5. Placeholder: plain `#F1F1EF` fill; tile tag and "captured" check stay; in the Preview it is opaque with
   "Preparing preview…" (no template under it), also after a load error.
6. Retry state lives in `PhotoImage` state keyed by photo, revision and preview token; a failed `<img>` is unmounted.
7. One unit test for `previewUrl` (listed Core mechanism).
8. C3(b): only the lock distance changed; the 14 px at which a slide starts and the lock control are as before, so
   the control fills twice as fast.

## Deviations
Test files outside the plan's **Files** lists, changed because the plan's own changes forced it (rows unchanged):
- `lv3_preview.test.tsx`: dock tap coordinates to the new layout (U1); fixture `preview: 'k0'` and one expected URL
  `&v=k0` (U2).
- `lv3_drawers.test.tsx`: two fixtures `preview: 'k0'`, one expected URL `&v=k0` (U2).
- `lv3_main.test.tsx`: three fixtures `preview: 'k0'` (U2).
- `lv3_main.test.tsx` and `lv3_geometry.test.ts` for C3(b) (named by the plan's Notes), and one phrase in
  `docs/guide/01_product.md`.
No production file outside the plan. No named mutations (flow §4). No architecture graph in this repo.

## Files changed
U1 (`a4df2ebd`): `apps/desktop/src/phone/components/Dock.tsx`, `gestures.ts`, `phone.css`,
`__tests__/helpers.ts`, `__tests__/lv3_dock.test.tsx`, `__tests__/lv3_preview.test.tsx`, `__tests__/lv6_dock.test.tsx` (new).
U2 (`b0728c90`): `apps/desktop/src/phone/api.ts`, `state.ts`, `phone.css`, `components/PhotoImage.tsx` (new),
`components/Dock.tsx`, `components/Preview.tsx`, `components/MoveDrawer.tsx`, `__tests__/helpers.ts`,
`__tests__/lv3_dock.test.tsx`, `__tests__/lv3_drawers.test.tsx`, `__tests__/lv3_main.test.tsx`,
`__tests__/lv3_preview.test.tsx`, `__tests__/lv3_shell.test.tsx`, `__tests__/lv6_images.test.tsx` (new),
`docs/guide/01_product.md`, `docs/guide/02_architecture.md`.
C3(b) (`da576109`): `apps/desktop/src/phone/gestures.ts`, `__tests__/lv6_dock.test.tsx`, `__tests__/lv3_main.test.tsx`,
`__tests__/lv3_geometry.test.ts`, `docs/guide/01_product.md`.
Documents only: `996ee63d` (a first version of this handoff, written before C3(b) was known; it wrongly said the
Tracker line and Review log were already written — a script step had failed on the plan file the orchestrator had
just changed) and the closing commit: `docs/implementation/plans/LV6_phone_dock.md` (Tracker line, Review log), this
handoff.
Not mine, landed on the branch during the session: `67b7dcff` (orchestrator: plan row C3(b), book 10J, changelog).
Build outputs, not committed (git-ignored): `apps/desktop/dist/`, the stub sidecar binary, `target/`.

## Checkpoint SHA
1. U1 — `a4df2ebd`
2. U2 — `b0728c90`
3. U1 row C3(b) — `da576109`
4. Documents — `996ee63d` (first handoff), then the commit that contains this version of the file together with the
   Tracker line and Review log (see `git log -1`).

## Stamp
Tree: `da576109`, `git status --porcelain` empty before and after. Each command once on this tree, in this order:
| Command | Result |
|---|---|
| `cd apps/desktop && npm run build` | built |
| `cargo test --workspace` | exit 0; 2760 passed, 0 failed |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo fmt --all -- --check` | exit 0 |
| `tools/sidecar/build.sh --stub --test` | exit 0; 100% tests passed out of 2 |
| `cd apps/desktop && npm run typecheck` | exit 0 |
| `npm test` | 149 files, 1058 tests passed |
**The gate was run twice.** First on `b0728c90` (all green: Rust 2760 passed, of which the behaviour suite 1640; UI
1057 passed). Then row C3(b) arrived and changed code, so the whole gate was taken again on `da576109` (the stamp
above), rather than citing a stamp from an older tree. The commits after `da576109` change documents only.
Unit commands (`npx vitest run src/phone`): 94 passed before the phase, 94 after U1 (−6, +6), 102 after U2, 103 after
C3(b); typecheck clean each time. No pre-existing failures. No command was refused.
Not checked: a real phone. Nothing was run on the iPhone or the Android phone, and no camera was used; how the 55 px
lock and the 56 px tiles feel in the hand is for the hardware check.

## Manual try-out
On this Mac, camera connected, phones paired, the app built from this branch:
1. Open a templated session. The phone's dock shows bigger tiles; slide it sideways and it scrolls.
2. Tap an empty shot: it gets the outline and the Mac's "Next" follows.
3. Take a picture: its tile is a grey square with its number, then the photo a few seconds later. Tap it: the preview
   opens (if it is not ready yet it says "Preparing preview…" and then shows the photo).
4. Press and hold a tile: nothing grows; on release its preview opens. Slide a tile upwards: nothing happens.
5. On the main screen hold the picture and slide up: live locks after a short swipe (about half of before).
6. Settings → Phone → Remove the Android phone: its screen says to delete the app from the home screen and scan the
   new QR code. Do that and add it again.
