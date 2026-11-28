---
phase: LV6_phone_dock
title: The phone's shot dock without hold gestures — larger tiles that scroll sideways, a single tap opens a taken shot's preview or makes an empty shot current; tile pictures appear as soon as their previews are ready and never stay broken; a removed phone is told to delete the app and install it again from the new QR code
difficulty: normal
projection: no
depends_on: [LV5_live_fixes]
implementer: fable
blocked_on: none
---
# LV6 — The phone's dock (LV-D19) and the phone halves of the fixes

**Tracker:** APPROVED (2026-11-26, opus) — review r1 APPROVED_WITH_NOTES: perimeter = the declared paths of `a4df2ebd`, `b0728c90`, `da576109`; phone tests 103 passed, typecheck and build green (reviewer run on `b815e3b2`); all 14 rows have a boundary test that fails on a wrong outcome; the six removed LV3 tests are exactly the plan's; H1–H8 hold; notes in `backlog_live/LV6_phone_dock.md`.

(This line is the milestone's tracker row. Sessions change only this line of this file. The gate for `depends_on` is
the Tracker line of the plan named there.)

## Goal
Book `10J` **LV-D19** (owner, at the hardware check, changelog D19): (1) the shot dock loses hold-to-magnify and
hold-and-drag-up; larger tiles; sideways scroll; a single tap on a filled tile opens its Preview, on an empty template
tile makes it the current shot; Free shot and Condition tiles as before; (2) a removed phone reinstalls, and the
removed screen says so. Plus the phone half of finding **F7** (tile pictures stuck as broken images): the phone uses
the session view's `preview` token (`master_plan_live.md` §5 **PH6a**, served by LV5·U2) and retries a failed image.
Everything else of the design stays as LV3 built it (the design README remains the authority for look, except where
LV-D19 changes the dock).
(U1) The dock: sizes, single tap, scroll. (U2) Pictures that are never broken; the removed text.
**Not here:** server code (LV5), the main screen's hold-for-live and the preview's hold-for-template (unchanged: LV-D19
removes only the dock's holds), the exposure readout and the Focus Area message (LV5 fixed the server; the phone
already draws `camera.exposure` (LV3·C13(d)) and maps `focus` `refused` to its Focus Area text (LV3·C11(c)), so no phone
change is needed for them), drawing the camera's own focus frame (STATUS_live "Needs the owner").

## How to work this plan (flow.md §10, master_plan_live.md §3)
One runner phase of 2 units, in order; after each, its **Unit test commands**, then a `CHECKPOINT (not approved):`
commit of that unit's paths. Row ids `LV6·C<n>(<row>)`; tests named after their row (`lv6_c2a_<slug>`). Open details:
the design README and prototype first, then the smallest option consistent with 10J LV-D19; recorded under
**Assumptions**. Reviewer notes: `docs/implementation/backlog_live/LV6_phone_dock.md`. `implementer: fable` (phone UI,
owner 2026-11-26; no camera code here).

| Unit | What it builds | Criteria | Rows | Changes app files |
|---|---|---|---|---|
| U1 | The dock: larger tiles, single tap, sideways scroll; the hold gestures removed | 3 | 6 | yes (TypeScript) |
| U2 | Pictures never broken (`preview` token, retry); the removed text | 3 | 7 | yes (TypeScript, guide) |
| **Total** | | **6** | **13** | |

## Read first (whole milestone)
`docs/process/flow.md` (§4, §4A, §6, §7A, §10) · `master_plan_live.md` (§3, **§5 PH6a**, PH7, **§6 phone texts as
amended 2026-11-26**) · book `10J` (LV-D15, **LV-D19**, §The phone design) · the design README
`claude_design/live_preview/design_handoff_shoot_companion/README.md` §3 Shot dock, §Shot dock gestures, §Selecting a
tile, §Reliability requirements (local, git-ignored; read, never copy) · the LV3 plan (Unit 3 rows, the "Shared for
all units" section) · `apps/desktop/src/phone/{api.ts, state.ts, gestures.ts, PhoneApp.tsx, phone.css,
components/{Dock.tsx, Preview.tsx, MoveDrawer.tsx}}` · `apps/desktop/src/phone/__tests__/{helpers.ts,
lv3_dock.test.tsx, lv3_shell.test.tsx}`.

## Points recorded by planning (each is one sentence in the narrative's "Choices made for you")
- **P1 Dock tiles are 56 px** (were 36), 8 px apart, with rounder corners (12 px); the dock is 84 px tall, so the
  picture above it is 22 px shorter.
- **P2 A tap is any touch that did not move more than 8 px**, however long it is held; a touch that moves sideways
  scrolls, one that moves up or down selects nothing.
- **P3 A photo whose preview is not ready yet shows a plain grey tile with its number**, and its picture appears by
  itself when the Mac has made it; a picture that fails to load is tried again three times, two seconds apart.
- **P4 The removed screen reads:** "This phone was removed in Session Manager. Delete this app from the home screen,
  then scan the new QR code in Session Manager → Settings → Phone and add it again."

## Unit 1 — The dock: larger tiles, single tap, sideways scroll (LV-D19 (1))

**Unit id:** `LV6·U1` · **Rows:** LV6·C1–C3 · **Changes app files:** yes (TypeScript)

### Goal
`Dock.tsx` keeps the tile order, variants, tags, the current outline, the split divider, the long-shot rotation and the
pointer-driven sideways scroll of LV3·U3; it drops the 200 ms hold (magnify, label pill, hover ticks, edge
auto-scroll) and the drag-up-to-open. Sizes per P1. A tap per P2 selects the tile nearest the finger whose centre is
within 32 px (half the 64 px pitch): an empty template tile → `POST /current {session_id, container_id}`; a tile with a
photo (taken shot, extra, condition) → its Preview; Free shot → `POST /current {session_id, container_id: null}`.
The tall pointer-transparent scroller that let magnified tiles grow over the stage (`ph-dock-scroller`, 260 px) goes:
the scroller is the strip's height.

### Read first
The design README §3, §Shot dock gestures, §Selecting a tile · LV3 plan Unit 3 (rows C15–C19) and Review log
judgment call 10 (the 260 px scroller) · `Dock.tsx`, `gestures.ts` (`tileScale`, `TILE_PX`, `DOCK`), `phone.css`
(`.ph-dock-*`), `helpers.ts` (the dock layout stub: 36 px tiles on a 42 px pitch).

### Files
New: `apps/desktop/src/phone/__tests__/lv6_dock.test.tsx`. Edited: `apps/desktop/src/phone/{components/Dock.tsx,
gestures.ts, phone.css}`, `apps/desktop/src/phone/__tests__/{helpers.ts (dock layout: 56 px tiles, 64 px pitch, strip
84 px), lv3_dock.test.tsx}`.
**LV3 tests removed (they encode the gestures LV-D19 removed), each listed in the handoff:**
- `lv3_c16a_tileScale_is_2_5_under_the_finger_and_1_from_95_px` (LV3·C16(a), the magnification curve)
- `lv3_c16b_a_held_tile_is_magnified_and_names_itself` (LV3·C16(b))
- `lv3_c18b_holding_200_ms_magnifies_with_a_tick_per_tile` (LV3·C18(b))
- `lv3_c19a_dragging_a_held_tile_up_past_50_px_opens_it_on_release` (LV3·C19(a))
- `lv3_c19b_bringing_the_tile_back_down_before_release_opens_nothing` (LV3·C19(b))
- `lv3_c19c_opening_an_empty_template_shot_selects_it` (LV3·C19(c)) **LV3 tests kept** (updated only for the new layout
stub, their rows unchanged): `lv3_c15a`, `lv3_c15b`, `lv3_c15c`, `lv3_c17a`, `lv3_c17b`, `lv3_c17c`, `lv3_c18a`,
`lv3_c13a`, `lv3_c14f`.

### Core mechanisms
None (`tileScale` is removed with the magnifier).

### Ordered tasks
1. Remove the magnifier, the label pill, the hover/drag haptics and the drag-up from `Dock.tsx`; remove `tileScale`
   and the unused `DOCK` thresholds from `gestures.ts`.
2. Sizes (P1) in `phone.css`/`gestures.ts`; the scroller is the strip's height.
3. Tap per P2 and the selection rules above; keep `vibrate(15)` on a selection (LV3·C17) and the focus-marker clearing
   on choosing a shot.
4. Remove the six LV3 tests named above; update `helpers.ts`; tests for the rows.

### Criteria (3 criteria, 7 rows)
Session as LV3·U3: shots B (final 02, filled, "Back"), A (final 01, empty, "Front"), C (final 03, empty), each ratio
1.0, in shooting order B, A, C; one extra (Main position 04); one condition image; current = A. Boundary: `PhoneApp`
rendered with the Mac faked at `fetch`/`WebSocket` (as LV3's tests), mouse events of the pointer types with a
`pointerId` (jsdom has no `PointerEvent`), fake timers.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| LV6·C1(a) | dock layout | the session above | every tile's inline/computed size 56 × 56 px and radius 12 px; adjacent tiles 8 px apart; the strip 84 px tall; no element of the dock taller than the strip | `[rule]` LV-D19 (1), P1 |
| LV6·C2(a) | single tap | tap the extra "04" | its Preview opens ("IMAGE 04 · EXTRA"); `vibrate(15)`; no request | LV-D19 (1), design §Selecting |
| LV6·C2(b) | same | tap "C1" | its Preview opens ("CONDITION 01"); no request | LV-D19 (1) |
| LV6·C2(c) | long press | press "02" (filled), hold still 600 ms, release | the Preview of B's photo opens; no tile changes size; no label pill; `navigator.vibrate` called only with 15 (never 12, 6 or 18) | LV-D19 (1) (no hold-to-magnify), P2 |
| LV6·C2(d) | vertical move | press "02", move up 60 px, release | nothing opens; no request; no vibration | LV-D19 (1) (no drag-up), P2 |
| LV6·C3(a) | scroll then tap | swipe left by 128 px (pointer down, move dx −128, up), then tap at the same screen x as the first press | the strip's `scrollLeft` grew by 128; the swipe selected nothing; the tap selects the tile two pitches to the right of the one first under the finger | LV-D19 (1) (scrollable), design §Shot dock gestures |
| LV6·C3(b) | slide up to lock (main screen) | hold the stage, slide up **55 px** and release; separately slide up 50 px and release | at 55 px live is locked ("LIVE · LOCKED") and stays on after release; at 50 px it is not locked and returns to the template on release; `lockProgress(-55)` = 1 (core mechanism `gestures.ts` `LOCK_PX` = 55) | LV-D19 (3) (owner 2026-11-26: lock swipe half as long) |

### Delegated choices
Whether the tap is decided on `pointerup` (movement ≤ 8 px) or on `click` · tile corner and shadow details beyond P1
(design tokens) · whether tags grow with the tiles (design: mono tags; keep readable at 56 px).

### Notes
- The main screen (hold for live, slide up to lock, overlay slide) and the Preview (hold for template) keep their
  gestures; LV-D19 changes only the dock, **except LV-D19 (3): the slide-up-to-lock distance halves, 110 → 55 px**
  (`apps/desktop/src/phone/gestures.ts` `LOCK_PX`; update `lv3_geometry.test.ts` and `lv3_main.test.tsx` rows LV3·C8(b)
  and the lock-progress row to 55, and say so in the handoff). Added by the orchestrator while U1 was in progress.
- The design's "never move" rule for the dock and control bar still holds (fixed height, now 84 px).

### Unit test commands
`cd apps/desktop && npx vitest run src/phone` · `npm run typecheck`

## Unit 2 — Pictures that are never broken; the removed text (F7 phone half, LV-D19 (2))

**Unit id:** `LV6·U2` · **Rows:** LV6·C4–C6 · **Changes app files:** yes (TypeScript, guide)

### Goal
PH6a: each image in the session view carries `preview: <string>|null`. One shared image component (new
`components/PhotoImage.tsx`, used by the dock, the Preview and the Move drawer) requests
`/api/phone/v1/photos/<id>/preview?pair=<token>&r=<revision>&v=<preview>` only when `preview` is a string; while it is
null it shows the placeholder (neutral fill `#F1F1EF`, the tile's tag; in the Preview the text "Preparing preview…").
On an image `error` it never shows the browser's broken-image glyph: it shows the placeholder and retries after 2 s
with `&try=<n>` added, at most three times; a new `preview` value (or `revision`) starts over. The removed text per P4
replaces §6's old text (the same `TEXT.removed` serves the 401 at start and the socket's 4401).

### Read first
Master plan §5 **PH6a**, PH7, §6 (phone texts as amended) · `api.ts` (`previewUrl`, `PhoneImage`), `state.ts`
(`TEXT`), `PhoneApp.tsx` (removed screen), `socket.ts` (4401), `components/{Dock.tsx, Preview.tsx, MoveDrawer.tsx}` ·
`__tests__/lv3_shell.test.tsx` (`lv3_c1c_…`).

### Files
New: `apps/desktop/src/phone/components/PhotoImage.tsx`, `apps/desktop/src/phone/__tests__/lv6_images.test.tsx`.
Edited: `apps/desktop/src/phone/{api.ts (PhoneImage.preview; previewUrl), state.ts (TEXT.removed),
components/{Dock.tsx, Preview.tsx, MoveDrawer.tsx}, phone.css}`, `apps/desktop/src/phone/__tests__/lv3_shell.test.tsx`
(`lv3_c1c_a_removed_phone_forgets_its_token_and_says_so`: its expected text becomes P4's; listed in the handoff);
`docs/guide/{01_product.md (the dock: tap and scroll, no holds; reinstall after Remove), 02_architecture.md (the
phone's image rule)}`.

### Core mechanisms
`api.ts` `previewUrl(token, image, attempt)` → the URL above, or `null` when `image.preview` is null.

### Ordered tasks
1. `PhoneImage.preview` in `api.ts`; `previewUrl` per the rule; `PhotoImage` with placeholder and retry.
2. Use `PhotoImage` in the dock, the Preview and the Move drawer.
3. `TEXT.removed` per P4.
4. Tests for the rows; update `lv3_c1c`; guide.

### Criteria (3 criteria, 7 rows)
Boundary: `PhoneApp` (or the named component) with the Mac faked at `fetch`/`WebSocket`, fake timers; images'
`error` dispatched on the `<img>` element. Session as U1, B's photo with `revision 3`.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| LV6·C4(a) | dock tile | B's photo with `preview: null` | tile "02" has no `<img>` whose `src` contains `/preview`; it shows the placeholder and its tag "02" | PH6a, F7, P3 |
| LV6·C4(b) | same | `preview: "k1"`; then a `session` message and a re-read answering `preview: "k2"` | the tile's `src` ends with `&r=3&v=k1`; after the re-read it ends with `&r=3&v=k2` | PH6a, F7 |
| LV6·C4(c) | same | `preview: "k1"`; the `<img>` fires `error`; advance 2 s three times, each followed by another `error`; then a re-read answering `preview: "k2"` | after each `error` the placeholder shows and the `<img>` is not displayed; requests with `&try=1`, `&try=2`, `&try=3` follow 2 s apart, then none; after the `k2` re-read a request with `&v=k2` and no `try` | F7, P3 |
| LV6·C5(a) | Preview | open B's photo with `preview: null`; then a re-read answering `preview: "k1"` | the text "Preparing preview…" and no preview request; then the photo with `&v=k1` and the text gone | PH6a, P3 |
| LV6·C5(b) | Move drawer | open it from the extra; B's photo has `preview: null` | tile "02" shows the placeholder, no preview request | PH6a, P3 |
| LV6·C6(a) | removed, at start | stored token; `hello` answers 401 | the token is removed; the text "This phone was removed in Session Manager. Delete this app from the home screen, then scan the new QR code in Session Manager → Settings → Phone and add it again." | LV-D19 (2), PH7, §6 |
| LV6·C6(b) | removed, while connected | connected; the socket closes with 4401 | the same text; the token removed | LV-D19 (2), PH1, PH7 |

### Delegated choices
The placeholder's exact look within the design tokens · whether the retry counter lives in `PhotoImage` state or a
ref · whether the Preview's placeholder shows the template under it.

### Notes
- No secure-context API, no absolute URL (PH7, card LV3): `previewUrl` stays relative.
- The token never appears in a toast or log (LV3 review check).

### Unit test commands
`cd apps/desktop && npx vitest run src/phone` · `npm run typecheck`

## Guide files (flow §7A)
U1–U2: `docs/guide/01_product.md` (the phone's dock: larger tiles, tap and scroll, no holds; after Remove the phone
app is deleted and installed again from the new QR code), `docs/guide/02_architecture.md` (the phone's image rule:
`preview` token, placeholder, retry).

## Closing gate (once, after the last unit)
`cd apps/desktop && npm run build` (first) · `cargo test --workspace` · `cargo clippy --workspace --all-targets -- -D
warnings` · `cargo fmt --all -- --check` · `tools/sidecar/build.sh --stub --test` · `cd apps/desktop && npm run
typecheck` · `npm test`. (The Rust gate runs although only TypeScript changes: the owner asked for the full gate once
per milestone, and the phone build is served by the Rust server.) One command at a time.

## Review mode
The reviewer reads the units in order and runs, once: `cd apps/desktop && npx vitest run src/phone` · `npm run
typecheck` · `npm run build`. Only B1–B4 block. Also check, by reading: no hold, magnify or drag-up code left in
`Dock.tsx`/`gestures.ts`; the six removed LV3 tests are exactly those listed and every kept LV3 dock test still
passes; no element shows the broken-image glyph path (every preview `<img>` goes through `PhotoImage`); no
secure-context API or absolute URL under `src/phone/`; the removed text equals §6 as amended. Compare with the
prototype only for what LV-D19 did not change. Notes go to `docs/implementation/backlog_live/LV6_phone_dock.md`.

## Manual try-out (the narrative is built from this)
On this Mac with the camera connected and both phones paired (the app built from this branch, LV5 included):
1. Open a templated session; the phone's dock shows bigger tiles. Slide the dock sideways: it scrolls.
2. Tap an empty shot: it becomes the current shot (outlined; the Mac's "Next" follows).
3. Take a picture: its tile shows a grey square with its number, then the photo a few seconds later, never a broken
   image. Tap it: its preview opens.
4. Press and hold a tile: nothing grows; on release its preview opens. Slide a tile upwards: nothing happens.
5. Settings → Phone → Remove the Android phone: its screen says to delete the app from the home screen and scan the
   new QR code. Do that and add it again: it works.

## Review log
- 2026-11-26, orchestrator: owner request during the run — slide-up-to-lock distance halved (110 → 55 px), row LV6·C3(b) added to U1 (LV-D19 (3)). If the implement session finished U1 before this row existed, the review treats it as a missing row (fix cycle).

### 2026-11-26 — implement r1 (fable)
**Built.** U1 (`a4df2ebd`): `Dock.tsx` rewritten without the 200 ms hold, magnifier, label pill, hover/drag haptics,
edge auto-scroll and drag-up; `gestures.ts` loses `tileScale`, `TILE_PX`, the hold thresholds and the three unused
haptics (`magnify`, `hoverTick`, `dragOpen`); `DOCK = {tapPx: 8, nearPx: 32}`. `phone.css`: 56 px tiles, 12 px corners,
8 px gap, 84 px strip, the scroller is the strip's height; `.ph-dock-label*` and `.ph-dock-magnifying` removed.
U2 (`b0728c90`): `PhoneImage.preview`, `previewUrl(token, image, attempt)` (null without a preview), new
`components/PhotoImage.tsx` used by the dock, the Preview and the Move drawer, `.ph-photo-wait`, `TEXT.removed` per P4,
guide files. **Row LV6·C3(b)** (`da576109`; the orchestrator's entry above, added after U1 and U2 were committed):
`gestures.ts` `LOCK_PX` 110 → 55, test `lv6_c3b_…`, the LV3 lock tests updated. The closing gate was run once on
`b0728c90`, then taken again in full on `da576109` because code changed after it; both green. Details and the
row → test map: `handoffs/implementer/handoff_LV6_phone_dock_implement_r1.md`.

**Judgment calls.**
1. *Tap decided on `pointerup`* (delegated). A touch is a tap unless it went more than 8 px sideways **or** more than
   8 px up/down at any time (per axis, not the diagonal distance); a touch that came back to its start after leaving
   that range is still not a tap. Sideways past 8 px scrolls from then on (also after a vertical move).
2. *Nearest tile at release, measured then* (centres are read at `pointerup`, not snapshotted at `pointerdown`: nothing
   resizes any more). "Within 32 px" is inclusive.
3. *Sizes live in `phone.css` only* (the plan allowed `phone.css`/`gestures.ts`); no size constants duplicated in
   TypeScript. LV6·C1(a) therefore loads `phone.css` into jsdom and asserts computed styles (file read as
   `p23_tile_fit.test.tsx` does).
4. *Details beyond P1:* tags in the dock 10 px (were 8), check badge 14 px (was 12), divider 40 px tall (was 28), tile
   fill `#F1F1EF` (the design's neutral fill; was `#eee`), 14 px above and below the tiles.
5. *Placeholder* = a plain `#F1F1EF` fill; in a dock or Move tile the tile's own tag stays on top of it and the
   "captured" check stays; in the Preview it is opaque with "Preparing preview…" (the template is not shown under it;
   hold-for-template does nothing visible until the photo is there). The same placeholder (same text in the Preview)
   shows after a load error.
6. *Retry state in `PhotoImage` state*, keyed by `photo_id:revision:preview`; while failed the `<img>` is unmounted,
   so the broken-image glyph can never show. After the third retry fails nothing more is asked until `preview` or
   `revision` changes.
7. *One unit test for the Core mechanism* `previewUrl` (`lv6_core_previewUrl_…`), as flow §4 allows for listed
   mechanisms; it maps to the plan's Core mechanisms list, not to a criterion row.
8. *C3(b):* only the distance changed. The 14 px at which a slide starts, the lock control and its haptic are as LV3
   built them, so the lock control now fills twice as fast. `lv6_c3b_…` lives in `lv6_dock.test.tsx` (U1's file).

**Deviations (files outside the plan's lists, test-only, forced by the plan's own changes).**
- `__tests__/lv3_preview.test.tsx` (U1): its dock tap coordinates were the old layout's (34/160/261 → 44/236/383).
  With the new stub two of its taps hit other tiles. Rows unchanged.
- `__tests__/lv3_preview.test.tsx`, `lv3_drawers.test.tsx`, `lv3_main.test.tsx` (U2): their inline image fixtures gain
  `preview: 'k0'` (the type now requires PH6a's field, and a null preview shows no picture), and two expected preview
  URLs gain `&v=k0`. `helpers.ts` and `lv3_dock.test.tsx` (in the plan's list) get the same. Rows unchanged.
- `lv3_drawers.test.tsx` still taps the dock at x 34: inside tile "02" in the new layout too, so it was left alone.
- C3(b), as the Notes ask: `lv3_geometry.test.ts::lv3_c8_lockProgress_and_overlayValue` (values for 55);
  `lv3_main.test.tsx`: the `lock()` helper slides 55 px, `lv3_c8a` expects the fill at 27 % (15 of 55), `lv3_c8b` is
  renamed `lv3_c8b_sliding_up_55_px_locks_live_and_it_stays_on_after_release` and slides 55 px, and `lv3_c8c` releases
  at 40 px instead of 60 (60 px now locks). `docs/guide/01_product.md` names the 55 px.

**Observations for the reviewer.**
- No red baseline was recorded before the production edits (tests and code were written together); the six removed
  LV3 tests and the kept ones are exactly those the plan lists.
- `lv3_c1c_…` and `lv6_c6a_…` now assert the same outcome (the plan asks for both); `lv3_c8b` and `lv6_c3b` overlap on
  the 55 px lock.
- In jsdom the fixture's dock content (427 px) is wider than the 390 px stage, but jsdom does not clamp `scrollLeft`;
  C3(a)'s 128 px swipe is exact there, a real browser stops at the end of the strip.
- The plan's header table and frontmatter title still count 13 rows; with C3(b) there are 14.
- Plan-fold notes for the orchestrator: `master_plan_live.md` §4.7 could name the LV3 test files above; its §2 row
  for LV6 still says 13 rows; the design README's lock distance (110 px) is now superseded by LV-D19 (3).

### 2026-11-26 — review r1 (opus): APPROVED_WITH_NOTES
**Verified correct.**
- *Perimeter.* `a4df2ebd`, `b0728c90`, `da576109` change exactly the files the handoff declares (the plan's lists plus
  the test-only deviations it names: `lv3_preview`, `lv3_drawers`, `lv3_main`, `lv3_geometry`); no production file
  outside the plan; the document commits touch only the handoff and this file.
- *Checks (Review mode, once, on `b815e3b2`).* `npx vitest run src/phone` 9 files, 103 passed · `npm run typecheck` exit
  0 · `npm run build` built. The implementer's full-gate stamp on `da576109` stands (later commits are documents).
- *Rows.* All 14 read code-then-test. C1(a) loads the real `phone.css` and asserts computed sizes, radius, gap and the
  84 px ceiling. C2(a)–(d) tap through `PhoneApp`'s dock with exact preview kickers, `vibrate` calls equal to `[[15]]`
  (or none) and empty POST lists; C2(c) would fail on the LV3 hold (its magnify and hover haptics break the exact `[[15]]`).
  C3(a): the layout stub subtracts `scrollLeft`, so the second tap at x 44 lands on "03" and the test asserts the exact
  `/current` body. C3(b) asserts both the 50 px and the 55 px outcomes and the demand sent. C4(a)–(c), C5(a)–(b) assert
  whole URLs, the absence of `<img>` while waiting or failed, the 1999/2000 ms retry edges, `try=1..3` then none, and
  the restart on `k2`. C6(a)/(b) assert the exact §6 text (as amended) and the token removed for 401 and 4401.
- *By reading (Review mode).* No hold, magnify, label or drag-up code left in `Dock.tsx`/`gestures.ts` (`tileScale`,
  `TILE_PX`, hold thresholds and the three haptics gone; one comment names the old design). The six removed LV3 tests
  are exactly the plan's; the nine kept ones exist and pass. `previewUrl` is called only by `PhotoImage`, which is the
  only preview `<img>` (dock, Preview, Move drawer); a failed image is unmounted, so no broken-image glyph. No
  secure-context API or absolute URL under `src/phone/`. `TEXT.removed` equals master_plan_live §6 v1.1 verbatim. The
  Mac side sends `preview: Option<String>` (`phone_view.rs`, LV5·U2).
- *Assumptions 1–8:* none contradicts 10J LV-D19 or PH6a; reported to the owner in the narrative.
- *H1–H8:* phone UI only; no external write, no pixel processing (the grey look is the existing CSS display filter), no
  business logic moved into the UI. Guide files updated for the dock, the image rule and the 55 px lock.

**Notes** (none blocks; `docs/implementation/backlog_live/LV6_phone_dock.md`): a picture that failed four times stays
grey until its preview changes; the Preview's placeholder hides the template on hold; the dock layout stub encodes the
pitch by hand; duplicated removed-text and 55 px tests; plan/master-plan row counts (13 vs 14) and the design README's
110 px are stale.
