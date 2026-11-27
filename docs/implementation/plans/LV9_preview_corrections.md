---
phase: LV9_preview_corrections
title: The owner's corrections after trying sort and arrange on the phone — a shot whose photo is on its way shows the quiet loading cue in every strip from the shutter until its picture; photos that move or go away settle in one step with no blink and no old picture; a lifted template-shot photo leaves its template picture behind; and the full-screen preview is only the photo, its strip, a ⋯ button for the photo's details and Retake, and a close icon
difficulty: high
projection: no
depends_on: [LV8_phone_sort_and_arrange]
implementer: fable
blocked_on: none
---
# LV9 — Preview corrections (book 10J LV-D21 (1)–(4))

**Tracker:** NOT_STARTED (2026-11-27, planner)

(This line is the milestone's tracker row. Sessions change only this line of this file. The gate for `depends_on` is
the Tracker line of the plan named there.)

## Goal
Book `10J` **LV-D21** (owner 2026-11-27, changelog D21; built from the description, judged in use), corrections (1)–(4)
after the owner tried LV8 on the phone: (1) the loading cue on a tile whose photo is on its way, in **every** strip,
from the shutter until its picture; (2) no flicker when photos move or go away in either strip; (3) a lifted
template-shot photo leaves its **template picture** in its place; (4) the Preview without header and action buttons —
the photo, the strip, a **⋯** button (bottom left) opening a drawer with the photo's details and Retake, a **close**
icon (bottom right). Reliability of touch stays first (master plan §5 PH9): every change here keeps LV8's state
machines, cancels and the Mac-is-the-truth rule.
(U1) The Mac's side: the phone view says which shots and groups have a photo on its way. (U2) The waiting tiles. (U3)
Moves that settle in one visual step, and the template picture under a lifted photo. (U4) The minimal Preview.
**Not here:** pinch zoom, reframe and guides (`LV10_zoom_reframe_guides`); the Mac's own screens; the camera; the
current-shot rules (MC-LV3, CS1–CS8 unchanged: U1 only **reads** CS4a's "on the way" set).

## How to work this plan (flow.md §10, master_plan_live.md §3)
One runner phase of 4 units, in order; after each, its **Unit test commands**, then a `CHECKPOINT (not approved):`
commit of that unit's paths. Row ids `LV9·C<n>(<row>)`; tests named after their row (`lv9_c4a_<slug>`; Rust
`lv9_c1a_<slug>`). Open details: 10J LV-D21 first, then the design README's tokens, timings and haptics
(`claude_design/live_preview/design_handoff_shoot_companion/README.md`, local, git-ignored; read, never copy; no new
prototype — owner: "build from the description, judge in use"), then the smallest option; recorded under
**Assumptions**. Reviewer notes: `docs/implementation/backlog_live/LV9_preview_corrections.md`. `implementer: fable`
(phone UI, owner 2026-11-26/27).
**Why the server part is a unit here and not its own milestone:** one read-only addition to `phone_view` (two fields,
computed from CS4a's existing "on the way" set), no route, no schema, no camera code (3 criteria); a separate
default-implementer milestone would add a full review cycle for it. U1 keeps H8: the rule is in `crates/core`, the
route is unchanged.

| Unit | What it builds | Criteria | Rows | Changes app files |
|---|---|---|---|---|
| U1 | The phone view: a shot's `incoming`, the session's `incoming {extras, condition}` (PH6b) | 3 | 8 | yes (Rust) |
| U2 | Waiting tiles in both strips; a picture that gave up stops shimmering | 3 | 10 | yes (TypeScript, guide) |
| U3 | One visual step per move: tiles keyed by photo, previews without the revision, swipes shown at once, the pending chain; the template picture under a lifted photo | 4 | 17 | yes (TypeScript, guide) |
| U4 | The Preview: photo, strip, ⋯ drawer (details + Retake), close icon | 3 | 9 | yes (TypeScript, guide) |
| **Total** | | **13** | **44** | |

## Read first (whole milestone)
`docs/process/flow.md` (§4, §4A, §6, §7A, §10) · `master_plan_live.md` §3, §4.5, **§5 CS4, CS4a, PH6, PH6a, PH6b, PH7,
PH9, PH9.5a, PH10** (v1.3, this milestone's contract), §6 (phone texts, v1.3) · book `10J` **LV-D21**, LV-D20, LV-D19,
§The phone design · the design README §Shot dock, §Preview, §Haptics, §Reliability requirements · plans
`LV8_phone_sort_and_arrange.md` (whole; its assumptions 1–18 stand unless a row here changes one), `LV7_tile_loading_cue.md`,
`LV6_phone_dock.md` (U2: `PhotoImage`) · `backlog_live/LV7_tile_loading_cue.md`, `backlog_live/LV8_phone_sort_and_arrange.md`
(two notes are fixed here: the gave-up shimmer, the 8 s timer) · `apps/desktop/src/phone/{api.ts, state.ts, arrange.ts,
gestures.ts, PhoneApp.tsx, phone.css, components/{Dock.tsx, Preview.tsx, PhotoImage.tsx, ItemDrawer.tsx,
MoveDrawer.tsx, ConfirmSheet.tsx}}`, `apps/desktop/src/phone/__tests__/{helpers.ts, lv8_drag.test.tsx,
lv8_drag_reliability.test.tsx, lv8_swipe.test.tsx, lv7_loading_cue.test.tsx, lv6_images.test.tsx}`.

## What planning found (the causes; each unit fixes its own)
- **(1) No cue from the shutter.** The cue (LV7) rides on `PhotoImage`'s placeholder, which exists only once the photo
  **row** exists and has no preview yet. From the shutter until the row (≈ 1.4 s file + ≈ 1.1 s intake on USB, ≈ 4–5 s +
  ≈ 1.1 s on Wi-Fi; STATUS_live LV4 C1/C9, CS4a evidence) the phone view has **no field** for it: the shot's `photo` is
  null, so its tile shows the empty-shot template picture, and a Condition capture has no tile at all. Then, right after
  the row, the job `OriginalPreviewJob` (`crates/integrations/src/lib.rs:1658–1708`) extracts the camera's embedded
  JPEG, sets `original_preview_path` and emits `PhotoChanged`, so `preview` (PH6a: adjusted ?? original ?? thumbnail)
  becomes a string within a fraction of a second: the cue showed, if at all, for a blink. Its peak is also faint (≈ 5 %
  lighter than the grey, LV7 review). The Mac already knows the missing state: CS4a's "on the way" set
  (`StoreTx::hold_times`, bindings by `exposure_shot`; `current_shot::shot_plan(...).held` for containers).
- **(2) Flicker, four causes, by reading:** (F1) shot tiles are keyed by container (`s<container_id>`,
  `state.ts::dockTiles`), so a swap or a fill keeps the tile's element and swaps its `<img>` source: the tile keeps
  painting the previous photo until the new source is decoded, and `PhotoImage`'s load state resets. (F2) the preview URL
  carries `r=<composition revision>` (`api.ts::previewUrl`) and `PhotoImage` keys on it; a move into a container applies
  the container's presets (`template_placement::apply_frame`), which raises the revision, so an unchanged picture is
  fetched again. (F3) swipes are not shown ahead (LV8 PH9.5 note): the Preview's strip keeps the swiped photo until the
  re-read, then it vanishes and the rest jump — two steps, the second late. (F4) the 8 s bound of a pending drop is not
  stopped by a successful answer (LV8 assumption 14, backlog note): a slow follow-up read lets the strip flash back to the
  old arrangement and then forward. And a second drop or swipe while one is pending is computed from the Mac's view, not
  from what the strip shows, so the first one blinks back (PhoneApp `drop`: `applyDrop(view, …)`).
- **(3)** The lifted tile is the tile element itself, translated (`Dock.tsx`, LV8 assumption 10: "its home shows a
  gap"); nothing is drawn at its home.
- **(4)** `Preview.tsx` draws a header (kicker, title, ratio chip), a resting pill and hint over the photo, and a button
  block (Move image, Back, Retake).

## Points recorded by planning (each is one sentence in the narrative's "Choices made for you")
The book leaves these open; each is the smallest option consistent with LV-D21 and LV-D20.
- **P1 On its way** = the Mac's own rule (CS4a): from the moment the camera reports the shutter until the photo exists
  on the Mac, at most 5 minutes (the capture window; a file that never comes stops being "on its way" then). Then the
  photo's own tile shows the cue until its picture is there (LV7), so the cue runs from shutter to picture.
- **P2 Where:** a template shot whose photo is on its way shows the cue in place of its template picture, keeping its
  number; a capture on its way to the Condition group (or to the end of Main in a session without a template) shows a
  waiting tile at the end of that group. Waiting tiles cannot be tapped, lifted or dropped onto, and they never cancel a
  drag in progress.
- **P3 The cue is a little stronger:** its light peaks about 10 % above the grey (LV7 made about 5 %); still slow and
  quiet, still off with Reduce Motion. A picture that failed to load three times is plain grey (it is not on its way).
- **P4 One step per move:** a moved photo keeps its picture element (tiles follow their photos); a picture is fetched
  only when its file changed on the Mac; a swipe is shown in the strips at once, like a drop; several quick swipes or
  drops stack, and the strip shows them all until the Mac has answered every one.
- **P5 The template picture stays:** while a template shot's photo is lifted, its place shows the shot's template
  picture in grey with the shot's number, as an empty shot does; extras and Condition images leave a gap as before.
- **P6 The Preview** shows only the photo, its strip, **⋯** (bottom left) and **×** (bottom right). ⋯ opens the photo
  drawer: what the header showed (e.g. "SHOT 02 · CAPTURED", "Back", "1:1"), the hold hint for template shots, and
  Retake for a template shot's photo. × closes the Preview. While holding, the "TEMPLATE" pill and the overlay readout
  still show (feedback during the gesture only).
- **P7 Move image leaves the Preview** (LV-D21 (4) lists Retake as the drawer's action now): dragging in the strips and
  swiping reach every place the Move drawer offered; the drawer's code goes. Say "put Move image in the ⋯ drawer" to
  bring it back.

## Shared for all units
- **Boundary (UI rows):** `PhoneApp` rendered with the Mac faked at `fetch`/`WebSocket` (`helpers.ts`), mouse events of
  the pointer types with a `pointerId` (jsdom has no `PointerEvent`; master plan §3), fake timers; fixtures S
  (`sortSession`) and D (`dragSession`) as LV8 defines them. `chairSession` gains the PH6b defaults (`incoming: false`
  on each shot, `incoming: {extras: 0, condition: 0}` on the session), so earlier tests are unchanged.
- **Fixture W (U2):** `chairSession` with shot C (final 03, empty) `incoming: true` and `incoming: {extras: 0,
  condition: 2}`: the live dock reads "02 01 03 04 · C1" then two waiting tiles. **Fixture DW:** fixture D with
  `incoming: {extras: 0, condition: 1}`: one waiting tile after 208, its centre x 703 (the stub's 64 px pitch).
- **Strip frames (U3):** new helper `stripFrames(testId)` in `helpers.ts`: a `MutationObserver` on the strip (subtree,
  child list, attributes `src` and `data-dock-k`) that, after every batch of DOM changes, records one frame = the tiles'
  `data-dock-k` in order, each with its `<img>`'s `src` or `-`; consecutive equal frames collapse; it returns the frames
  recorded since it started. A blink (an old arrangement or an old picture shown again) is an extra frame.
- **Visible hooks the rows assert:** a waiting tile has `data-incoming` (the shot tile keeps its `aria-label`; a
  group waiting tile is `aria-label="Photo on its way"`); a gave-up placeholder has `data-gave-up`; a tile's
  `data-dock-k` is `p<photo_id>` when it shows a photo, else `s<container_id>` (empty shot), `free`, or `w<group><n>`
  (waiting); the home marker of a lifted template photo is `data-testid="lift-home"`; the Preview root has `aria-label` =
  the kicker text; its buttons `aria-label="Photo details"` (⋯) and `aria-label="Close"` (×); the drawer
  `data-testid="photo-drawer"`, `role="dialog"`.
- **Core mechanisms** (the only unit tests, flow §4): `apps/desktop/src/phone/arrange.ts` — `applyDrop` gains the kind
  `remove` (U3); `arrangementKey` keeps ignoring waiting tiles (U2). Unit tests `lv9_core_<fn>_…` in
  `__tests__/lv9_core.test.ts`, input → output, one case per rule line.

## Unit 1 — The Mac's side: what is on its way (LV-D21 (1); PH6b)

**Unit id:** `LV9·U1` · **Rows:** LV9·C1–C3 · **Changes app files:** yes (Rust)

### Goal
`phone_view` (core) adds, read at `now` from CS4a's "on the way" set (the reported shutters of the session whose photo
does not exist yet, within `FIFO_WINDOW_MS`): `PhoneShot.incoming: bool` — true iff the shot has **no photo** and an
exposure on the way is bound to its container; `PhoneSession.incoming: {extras: u32, condition: u32}` — the exposures on
the way that will become Condition (templated session, bound to no container) and Main at the end (untemplated session,
unbound; P66). Read-only; no route, no event changes (CS5 already emits `SessionChanged` at the binding, arrival
`PhotoAdded` when the row appears; the phone re-reads on both).

### Read first
`crates/core/src/services/phone_view.rs`, `crates/core/src/services/current_shot.rs` (`shot_plan`, `taken`, the `held`
set), `crates/core/src/ports/store.rs` (`hold_times`, `exposure_shot`), `crates/testkit/tests/suite/lv5_current_shot.rs`
(`fixture`, `expose`, `file`, `arrive`, `lv5_c1c` for the window), `crates/testkit/tests/suite/lv2_phone_http.rs` (the
paired `GET /session`).

### Files
Edited: `crates/core/src/services/phone_view.rs` (and `current_shot.rs` only if a read helper is shared). New:
`crates/testkit/tests/suite/lv9_phone_incoming.rs` (+ its `mod` line in `crates/testkit/tests/suite/main.rs`). Guide:
`docs/guide/02_architecture.md` (the phone view's `incoming`).

### Core mechanisms
None new (the rule is CS4a's set; rows test it at `phone_view`).

### Ordered tasks
1. One read of the on-the-way exposures with their bindings inside `phone_view`'s transaction (reuse `shot_plan`'s
   `held` for containers; count the bound-to-none and unbound ones).
2. The two fields, serialized as `incoming` (shot) and `incoming {extras, condition}` (session).
3. Tests for the rows; guide.

### Criteria (3 criteria, 8 rows)
Boundary: `phone_view(store, current_shots, default_shape, Some(s), now)` on the `lv5_current_shot.rs` fixture (template
"Chair": containers A final 1 shoot 2, B final 2 shoot 1, C final 3 shoot 3; session S active; current = B), using its
`expose`, `file`, `arrive` helpers and the test clock; C3(a) through `TestPhone` (`GET /api/phone/v1/session`).
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| LV9·C1(a) | shutter, no file | `expose(101)` (bound to B) | B: `photo` null, `incoming` true; A and C `incoming` false; session `incoming` = `{extras: 0, condition: 0}`; `current` = A | LV-D21 (1), CS4a |
| LV9·C1(b) | file filed, not taken in | then `file(101)` | B: `photo` null, `incoming` true (CS4a: on the way until the photo exists) | CS4a |
| LV9·C1(c) | photo exists | `expose(101)`, `arrive(101)` | B: `photo` = that photo, `incoming` false | LV-D21 (1), P1 |
| LV9·C2(a) | on its way to Condition | current set to Free (`CurrentShots::set(…, Current::Free, …)`); `expose(102)`; then `arrive(102)` | first: `incoming` = `{extras: 0, condition: 1}`, no shot `incoming`; after: `{extras: 0, condition: 0}` and `condition` lists the photo | LV-D21 (1), 10G §4 rule 4 |
| LV9·C2(b) | untemplated | session U (item type without a template) active; `expose(103)`; then `arrive(103)` | first: `incoming` = `{extras: 1, condition: 0}`, `shots` empty; after: `{extras: 0, condition: 0}` and `extras` lists the photo | P66, P2 |
| LV9·C2(c) | never arrives | `expose(104)`, `file(104)`, `clock.advance(301_000)` | B `incoming` false; session `incoming` `{0, 0}` | CS4a (`FIFO_WINDOW_MS`), P1 |
| LV9·C3(a) | the wire | after `expose(101)`, `GET /session` with a paired token | JSON `session.shots[0].incoming` = `true` (B is first in shooting order), `session.shots[1].incoming` = `false`, `session.incoming` = `{"extras": 0, "condition": 0}` | PH6b, Invariant PH |
| LV9·C3(b) | denied after arrival | `expose(101)`, `arrive(101)`, the photo denied on the Mac (`ReviewService::delete`) | B: `photo` null, `incoming` false (a denied photo is not on its way) | CS4a, P1 |

### Delegated choices
Whether the counts are computed beside `shot_plan` or by a small read helper in `current_shot.rs` · the Rust field types
(`u32` or `i64`; serialized as numbers).

### Notes
- A Capture One tether capture has no reported shutter (CS6 binds at arrival): it shows no cue before its file; nothing
  to do.
- `features` in `hello` is not changed (the phone and the server ship together).

### Unit test commands
`cargo test -p psm-testkit --test suite lv9_phone_incoming::` · `cargo check -p psm-integrations`

## Unit 2 — Waiting tiles in both strips (LV-D21 (1); PH6b, PH7)

**Unit id:** `LV9·U2` · **Rows:** LV9·C4–C6 · **Changes app files:** yes (TypeScript, guide)

### Goal
`api.ts` types gain PH6b's fields. `dockTiles` gives a template shot with `incoming` and no photo the waiting look
(`.ph-photo-wait` with LV7's sweep instead of its grey template picture; number tag kept; `data-incoming`), and appends
waiting tiles (kind `incoming`, `data-incoming`, `aria-label="Photo on its way"`, no tag): `incoming.extras` after the
extras (before Free shot), `incoming.condition` after the Condition images. Both strips (the live dock and the Preview's
strip) show them. A waiting tile takes no tap (no request, no haptic, no Preview), never lifts, is not a drop target and
counts in no insert index (PH9.3 homes exclude it), and is not in `arrangementKey` (so one appearing or going never
cancels a drag, PH9.4). The sweep's peak rises to about 10 % above the fill (P3). `PhotoImage` marks a placeholder whose
retries are used up `data-gave-up`; the stylesheet turns the sweep off for it (LV7 backlog note).

### Read first
`state.ts` (`dockTiles`, `DockTile`, `tileImage`), `components/Dock.tsx` (`liftAt`'s homes, `nearest`, `up`),
`components/PhotoImage.tsx`, `arrange.ts` (`arrangementKey`, `dropTarget`), `phone.css` (`.ph-photo-wait`),
`__tests__/{lv7_loading_cue.test.tsx, lv6_images.test.tsx}`.

### Files
New: `__tests__/lv9_incoming.test.tsx`. Edited: `api.ts` (types), `state.ts`, `arrange.ts` (only if `arrangementKey`
needs to skip the new kind), `components/{Dock.tsx, PhotoImage.tsx}`, `phone.css`, `__tests__/helpers.ts` (PH6b
defaults; fixtures W and DW). Guide: `docs/guide/01_product.md` (the phone's strip shows a photo on its way).

### Core mechanisms
None new.

### Ordered tasks
1. Types and `chairSession` defaults; `dockTiles` (the shot's waiting look; the group waiting tiles, keys `wextra<n>` /
   `wcondition<n>`).
2. `Dock`: waiting tiles skipped by tap, lift, homes and targets; the sweep's peak; `data-gave-up` and its CSS.
3. Tests for the rows; guide.

### Criteria (3 criteria, 10 rows)
Boundary: `PhoneApp` with fixture W unless a row says D/DW; the Preview of 201 opened by a tap on its dock tile where a
row names `preview-strip`.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| LV9·C4(a) | shot on its way | fixture W | the dock tile "03 · Container 3" has `data-incoming`, contains a `photo-wait` element with class `ph-photo-wait`, no `<img>`, and its tag reads "03" | LV-D21 (1), P2 |
| LV9·C4(b) | in the Preview's strip | fixture W; Preview of 201 open | `preview-strip`'s tile "03 · Container 3" has `data-incoming` and a `photo-wait` element | LV-D21 (1) ("every strip") |
| LV9·C4(c) | it takes no tap | fixture W; tap tile "03" in the dock | no POST; `vibrate` not called; no Preview | P2 (the shot is taken, CS4) |
| LV9·C4(d) | shutter to picture | fixture W; then `macChanges` with C's `photo` = `{photo_id: 209, revision: 1, ratio: 1, preview: null}`, `incoming` false; then `macChanges` with `preview` "k1" | after the first: tile "03" has no `data-incoming`, a `photo-wait` element still; after the second: its `<img>` `src` starts `/api/phone/v1/photos/209/preview?pair=T` and contains `v=k1`, and no `photo-wait` (U3 then drops `r=` from the URL) | P1, LV7·C1(b) kept |
| LV9·C5(a) | group waiting tiles | fixture W; separately an untemplated session (`templated` false, `shots` [], extras 202, `incoming {extras: 1, condition: 0}`) | W: the dock's tiles after "C1" are two with `aria-label="Photo on its way"` and `data-incoming`, also in `preview-strip`; untemplated: one such tile between the extra "01" and the Free shot tile | P2 |
| LV9·C5(b) | never lifts, never taps | fixture W; `press` on the first group waiting tile, `advance(600)`, `lift` | no tile ever has `data-lifted`; no POST; no Preview; `vibrate` not called | P2, PH9.2 |
| LV9·C5(c) | not a drop target | fixture DW; Lift 203 (x 511), `slide` (703, 742), `lift` | no POST (the waiting tile is outside span(Condition), PH9.3); no tile had `data-drop` while at 703 | P2, PH9.3 |
| LV9·C5(d) | arriving during a drag | fixture D; drag 202 to 380 (LV8: Lift 202, `slide` (330, 742), `slide` (380, 742)); `macChanges` with fixture DW's `incoming`; `lift` (380, 742) | 202 still `data-lifted` after the change; POSTs = `[["/photos/202/move", {session_id: 7, target: {kind: "extra_insert", index: 3}}]]` | P2, PH9.4 (key unchanged) |
| LV9·C6(a) | gave up | a dock tile whose `<img>` fails to load (`error` event) four times (`advance(2000)` between) | after the 4th failure its `photo-wait` has `data-gave-up`; before it, none had; `phone.css` has a rule for `.ph-photo-wait[data-gave-up]::before` with `animation: none` | P3, LV7 backlog note |
| LV9·C6(b) | a new picture after giving up | from (a), `macChanges` with that photo's `preview` "k2" | an `<img>` with `…&v=k2` is requested; no `data-gave-up` | LV6 P3 kept |

### Delegated choices
The waiting tile's look within the tokens (same fill and sweep as a photo's placeholder; whether it shows a faint
camera glyph) · the exact peak (between 8 % and 12 % above `--ph-fill`) · whether a waiting shot tile keeps the
current-shot outline (it is never current: CS4).

### Notes
- LV7's rows stay: the cue is still `.ph-photo-wait`'s sweep; only its peak and the gave-up rule change.
- Arrival of the photo itself changes `arrangementKey` (a new photo id), so it still cancels a drag in progress, as
  LV8 PH9.4 decided; only the waiting tile is outside the key.

### Unit test commands
`cd apps/desktop && npx vitest run src/phone` · `npm run typecheck`

## Unit 3 — One visual step per move; the template picture under a lifted photo (LV-D21 (2), (3); PH6a, PH7, PH9.5a)

**Unit id:** `LV9·U3` · **Rows:** LV9·C7–C10 · **Changes app files:** yes (TypeScript, guide)

### Goal
Fix F1–F4 (§What planning found) so every move, drop, swipe or change from the Mac reaches the screen in one visual
step, with no arrangement or picture shown again after it was replaced:
- **F1 Tiles follow their photos.** `dockTiles` keys a tile that shows a photo `p<photo_id>` in every group (template
  shots included); an empty shot `s<container_id>`, Free `free`, waiting `w…`. React then moves a photo's element and
  its `<img>`; nothing is re-requested. Everything keyed on tile keys (homes, `drag.key`, `aside`, `nearest`, the
  Preview strip's marked tile) works on the new keys.
- **F2 A picture is fetched only when its file changed.** `previewUrl` = `…/photos/<id>/preview?pair=<token>&v=<preview>`
  (+ `&try=<n>` on a retry); `PhotoImage`'s key = `photo_id:preview`. `revision` stays in the view (LV10 uses it). PH6a's
  token already changes whenever the served file changes (PH7 amended v1.3).
- **F3 Swipes are shown at once** in both strips, through the same pending mechanism as a drop: a Deny (template shot:
  Retake; extra or Condition: delete) shows the photo removed (`applyDrop` kind `remove`: a template shot becomes empty,
  an extra or Condition image leaves its group); Main / Condition shows it at the end of that group (`to_main` /
  `to_condition`).
- **F4 The pending chain (PH9.5a).** A drop or swipe while others are pending is applied to what the strips show (the
  pending arrangement), not to the Mac's view; the strips keep the pending arrangement until every outstanding request
  has been answered **and** a session read started after the last answer has returned (→ the Mac's view); any refusal →
  the last view from the Mac, `refused` (toast by code), a re-read; an outstanding request with no answer after
  `pendingMs` (8 s) → the last view from the Mac and a re-read, no toast; after the last answer, a follow-up read that
  has not returned within `pendingMs` → the last view from the Mac. A successful answer stops its 8 s bound. The
  Preview's next photo after a swipe is `nextPhoto` of what the strips show. While anything is pending nothing lifts
  (LV8); swipes stay allowed (sorting a stack quickly is the point).
- **(3) The template picture stays.** While a template shot's photo is lifted or dragged, its home shows the shot's
  template picture in grey (`exampleUrl`, `ph-grey`) with the shot's number tag, as an empty shot tile does (no picture
  when the shot has no example: the empty-shot look); the home marker (`data-testid="lift-home"`, no `data-dock-k`) is
  gone when the gesture ends (drop shown at once, cancel springs home). Extras and Condition images leave the gap LV8 drew.

### Read first
`state.ts` (`dockTiles`), `api.ts` (`previewUrl`), `components/{Dock.tsx, PhotoImage.tsx, Preview.tsx}`, `PhoneApp.tsx`
(`drop`, `swipe`, `waiting`, `endPending`, `readSession`, `arranged`), `arrange.ts` (`applyDrop`, `nextPhoto`), master
plan **PH9.5, PH9.5a**, `__tests__/{helpers.ts (FakeMac.hold/release/answer), lv8_drag_reliability.test.tsx}`.

### Files
New: `__tests__/lv9_one_step.test.tsx`, `__tests__/lv9_core.test.ts`. Edited: `state.ts`, `api.ts`, `arrange.ts`,
`PhoneApp.tsx`, `components/{Dock.tsx, PhotoImage.tsx}`, `phone.css`, `__tests__/helpers.ts` (`stripFrames`). Guide:
`docs/guide/02_architecture.md` (the strip's keys, the pending chain), `docs/guide/04_change_impact.md` (the preview
URL's token: the only reason to fetch a picture again).
**Earlier tests changed (each listed in the handoff with the reason):** every assertion of a preview URL with `&r=`
(F2: the URL no longer carries the revision): `lv3_dock.test.tsx` (2), `lv3_drawers.test.tsx` (1), `lv3_preview.test.tsx`
(1), `lv6_images.test.tsx` (7), `lv7_loading_cue.test.tsx` (1) — the expected URL loses `&r=<n>`, the row is unchanged;
any LV8 test that finds a tile by its `data-dock-k` or asserts the strip's keys (`lv8_drag_reliability` C15(a) reads
"… p205, p206, p202, free …": unchanged; a filled shot's key changes from `s<id>` to `p<photo>`); LV8·C15(d) and the
backlog note on the 8 s timer are superseded by C9(b) here.

### Core mechanisms
`applyDrop(view, photoId, {kind: "remove"})` — the photo leaves its place (a template shot keeps its slot, empty; an
extra or a Condition image leaves its group; `current` and `counts` untouched, as LV8 assumption 13). Unit tests
`lv9_core_applydrop_remove_…` (shot, extra, Condition).

### Ordered tasks
1. Keys (F1) and the URL (F2); the tests whose URL changes.
2. `applyDrop` `remove`; the pending chain in `PhoneApp` (one structure for drops and swipes; F3, F4).
3. The lift home (3).
4. `stripFrames`; tests for the rows; guide.

### Criteria (4 criteria, 17 rows)
Boundary: `PhoneApp` with fixture D (stage 800) for strip rows, fixture S for Preview rows (LV8's coordinates and
"Lift X", "drag 202 to 380", "swipe left 120"). "Frames" = `stripFrames` started just before the input.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| LV9·C7(a) | swap keeps pictures | fixture D; keep the `<img>` element shown in the tile of 201 (`p201`) and of 204 (`p204`); Lift B (x 44), `slide` (108, 742), `lift`; the Mac answers and the re-read gives B ↔ A swapped | after the drop, and again after the re-read: the element showing 201 is the same element as before, `src` unchanged, now in the second tile; likewise 204 in the first; the strip's keys start `p204, p201, s13` | LV-D21 (2), F1 |
| LV9·C7(b) | fill an empty shot | fixture D; Lift 202, `slide` (172, 742), `lift`; re-read with 202 in C | the element showing 202 before is the same after, `src` unchanged; keys start `p201, p204, p202` | F1 |
| LV9·C7(c) | a Mac-side move | fixture D; `macChanges` with extras 206, 202, 205 (moved on the Mac) | each extra's `<img>` element is the same object as before, `src` unchanged; keys in the new order | F1 |
| LV9·C7(d) | revision only | fixture D; `macChanges` with 201's `revision` 2 (same `preview`) | the `<img>` of 201 is the same element, `src` = `/api/phone/v1/photos/201/preview?pair=T&v=k0`; no other preview URL requested | F2, PH7 (v1.3) |
| LV9·C8(a) | Deny shown at once | fixture S, Preview of 203; `mac.hold()`; swipe left 120 | before any answer: in both `preview-strip` and `dock` no tile `p203`; the Preview shows 207; then `mac.release()` and the re-read without 203: no further change of either strip | LV-D21 (2), F3 |
| LV9·C8(b) | Main shown at once | fixture S, Preview of 207; `mac.hold()`; swipe right 120 | before any answer: the strips' keys end `… p202, p207, free, p203` (207 at the end of Main, before Free shot) | F3, LV-D20 (1) |
| LV9·C8(c) | Retake shown at once | fixture S, Preview of 201; `mac.hold()`; swipe left 120 | before any answer: the first tile is `s12` (B empty), showing its grey template picture | F3, MC-LV3 |
| LV9·C8(d) | a refused swipe | fixture S, Preview of 203; `/photos/203/delete` answers 409 `session_changed`; swipe left 120 | after the answer the strips show `p203` again; toast "Session changed on the Mac"; a `GET /session` follows; the Preview stays on 207 | PH9.5a, PH9.1 kept |
| LV9·C8(e) | swipes stack | fixture S, Preview of 203; `mac.hold()`; swipe left 120 (203), then swipe left 120 (207) | after the second, before any answer: no `p203`, no `p207` in either strip; the Preview has closed (no photo after 207); POSTs = deletes of 203 then 207, in that order | P4, PH9.5a |
| LV9·C9(a) | a drop in one step | fixture D; frames; `mac.hold()`; drag 202 to 380, `lift`; `mac.release()`; the re-read answers 205, 206, 202 | frames = exactly `[before, after]` (the drop's arrangement once; the Mac's equal view adds none) | LV-D21 (2), F3/F4 |
| LV9·C9(b) | a slow read never flashes back | fixture D; frames; drag 202 to 380, `lift`; the move answered at 7 900 ms; the follow-up read answered at 8 500 ms (205, 206, 202) | frames = exactly `[before, after]` | F4, PH9.5a |
| LV9·C9(c) | a drop then a swipe | fixture D; frames on `dock`; `mac.hold()`; drag 202 to 380, `lift`; tap 205 (a tap while pending, LV8·C15(e)) and swipe up 120 in its Preview; `mac.release()`; the re-read answers extras 206, 202 and Condition 203, 207, 208, 205 | frames = exactly `[before, after the drop, after the drop and the swipe]`; the Mac's equal view adds none | PH9.5a, P4 |
| LV9·C9(d) | a change from the Mac | fixture D; frames; `macChanges` without 206 (denied on the Mac) | frames = exactly `[before, after]` | LV-D21 (2) |
| LV9·C10(a) | template picture at home | fixture D; Lift B (x 44); `slide` (300, 742) | a `lift-home` element exists at B's home, contains an `<img>` with `src` `/api/phone/v1/containers/12/example?pair=T` and class `ph-grey`, and the tag "02"; it has no `data-dock-k` | LV-D21 (3), P5 |
| LV9·C10(b) | no example | fixture D with B's `example` false; Lift B | `lift-home` exists with the tag "02" and no `<img>` | P5 |
| LV9·C10(c) | an extra leaves a gap | fixture D; Lift 202 | no `lift-home` element | P5, LV8 assumption 10 kept |
| LV9·C10(d) | gone at the end | fixture D; Lift B, `slide` (108, 742), `lift` (drop); separately Lift B, `cancel` | after the drop and after the cancel: no `lift-home` element | P5, PH9.4 |

### Delegated choices
Where the pending chain lives (`PhoneApp` state with a list of outstanding requests, or a small reducer) · how the home
marker is drawn (a sibling layer positioned at the home, or the lifted photo drawn in a carry layer while the tile stays
as an empty shot — the existing hooks stay: the lifted element has `data-lifted` and `scale(1.15)` in its `transform`;
`data-dock-k` stays unique per tile) · preloading the next photo's picture in the Preview.

### Notes
- **Hazard:** a second **drag** cannot start while a drop is pending (LV8 PH9.5: a long press is a tap), so the stacked
  strip case is a drop then a swipe (C9(c)); stacked swipes are C8(e).
- **Hazard:** `stripFrames` sees the DOM, not paint: a picture swapped inside the same element would show as one frame.
  That is why C7 asserts element identity, and C9 asserts frames.
- The Preview's subject is still resolved from the Mac's view (LV3: it follows its photo by id); only the strips and
  `nextPhoto` use the pending arrangement.

### Unit test commands
`cd apps/desktop && npx vitest run src/phone` · `npm run typecheck`

## Unit 4 — The Preview: photo, strip, ⋯ and × (LV-D21 (4); PH10)

**Unit id:** `LV9·U4` · **Rows:** LV9·C11–C13 · **Changes app files:** yes (TypeScript, guide)

### Goal
`Preview.tsx` draws only: the photo (`ImageFrame`, its swipes and hold as LV8), the strip (LV8), a bottom bar with **⋯**
(left, `aria-label="Photo details"`) and **×** (right, `aria-label="Close"`). No header, no resting pill or hint over the
photo, no Move image / Back / Retake buttons. The root carries `aria-label` = the kicker ("SHOT 02 · CAPTURED",
"IMAGE 04 · EXTRA", "CONDITION 01"). While a hold shows the template, the "TEMPLATE" pill and the overlay readout show as
today. **⋯** opens the **photo drawer** (new `components/PhotoDrawer.tsx`, the ItemDrawer's sheet look): the kicker, the
title ("Back", "Extra image", "Condition image"), the ratio chip, the line "Hold to view template · slide sideways for
overlay" for a template shot's photo, and under ACTIONS **Retake** for a template shot's photo (LV3's `retakeShot`: POST
`/retake`, toast "Shot NN reopened", the drawer and the Preview close). The drawer closes on its ×, on the backdrop, and
when the Preview closes; while it is open the photo and the strip take no touch. **×** closes the Preview (as Back did).
The Move drawer goes (P7): `MoveDrawer.tsx`, `moveTiles`/`MoveTile`, the `move` action and state, `backLabel`, their CSS.

### Read first
`components/{Preview.tsx, ItemDrawer.tsx, MoveDrawer.tsx}`, `state.ts` (`previewHeader`, `backLabel`, `moveTiles`,
`reduce` `move`), `PhoneApp.tsx` (`retakeShot`, `move`, the Preview's props), `__tests__/{lv3_preview.test.tsx,
lv3_drawers.test.tsx, lv8_swipe.test.tsx}`.

### Files
New: `components/PhotoDrawer.tsx`, `__tests__/lv9_preview.test.tsx`. Edited: `components/Preview.tsx`, `PhoneApp.tsx`,
`state.ts`, `phone.css`. Removed: `components/MoveDrawer.tsx`. Guide: `docs/guide/01_product.md` (the Preview's layout;
Move image gone).
**Earlier tests changed or removed (each listed in the handoff with the reason):** the Move-drawer tests of
`lv3_drawers.test.tsx` (LV3 C23 rows: removed, P7; any row there that is not about the Move drawer stays); assertions of
the header's kicker or title text, the "Back to shot NN" button, the resting pill or the hint over the photo
(`lv3_preview.test.tsx`, `lv8_swipe.test.tsx` C5–C8, `lv3_drawers.test.tsx`): the kicker is read from the Preview's
`aria-label`, Back becomes ×, Retake moves into the drawer; LV8·C8(e) is superseded by C11(a) here.

### Core mechanisms
None.

### Ordered tasks
1. The Preview's layout and the root's `aria-label`; × .
2. `PhotoDrawer`; Retake from it; touches off while it is open.
3. Remove the Move drawer and what only it used.
4. Tests for the rows; the changed tests; guide.

### Criteria (3 criteria, 9 rows)
Boundary: `PhoneApp` with fixture S; the Preview opened by a tap on the dock.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| LV9·C11(a) | minimal at first glance | Preview of 201 | `preview` has `aria-label` "SHOT 02 · CAPTURED"; no visible text "SHOT 02 · CAPTURED", "Back", "1:1", "CAPTURED" or "Hold to view template · slide sideways for overlay" in it; no button named "Move image", "Retake" or "Back to shot 01"; buttons "Photo details" and "Close"; `preview-strip` present | LV-D21 (4), P6 |
| LV9·C11(b) | × | Preview of 202; click "Close" | the Preview is gone; no POST | P6 |
| LV9·C11(c) | gestures kept | Preview of 201: `press` (100, 305), `advance(200)`; then a new Preview of 203 and swipe left 120 | at 200 ms the pill "TEMPLATE" shows (LV8·C5(b)); the swipe POSTs `/photos/203/delete` and the Preview shows 207 (LV8·C6(a)) | LV-D21 (4) ("swipes and hold keep working") |
| LV9·C12(a) | the drawer | Preview of 201; click "Photo details"; separately Preview of 202 | `photo-drawer` open, reading "SHOT 02 · CAPTURED", "Back", "1:1", the hold hint, and a "Retake" button; for 202: "IMAGE 04 · EXTRA", "Extra image", "1:1", no hint, no "Retake" | LV-D21 (4), P6 |
| LV9·C12(b) | Retake from it | Preview of 201, drawer open; click "Retake" | POSTs = `[["/retake", {session_id: 7, container_id: 12}]]`; `vibrate([30,40,30])`; toast "Shot 02 reopened"; drawer and Preview gone | LV-D21 (4), MC-LV3, card LV6 |
| LV9·C12(c) | closing it | drawer open on 201: its ×; separately the backdrop | drawer closed (`aria-hidden` true); the Preview still on 201; no POST | P6 |
| LV9·C13(a) | no touch under it | drawer open on 203: swipe left 120 on `preview-viewport`; tap "C2" in `preview-strip` | no POST; the Preview still on 203 | design §Reliability |
| LV9·C13(b) | closes with its Preview | drawer open on 203; `macChanges` without 203 | the Preview and the drawer are gone | LV3 (orphaned preview), P6 |
| LV9·C13(c) | no Move drawer | the built phone | no element with text "Move image" anywhere in the app (live screen, Preview, drawers); `apps/desktop/src/phone/components/MoveDrawer.tsx` does not exist | P7 |

### Delegated choices
The ⋯ and × icons and the bottom bar's height within the tokens (44 px touch targets at least) · whether the bar sits
under the strip or overlays the photo's bottom edge (the strip and both buttons stay visible on a 390 × 844 phone) · the
drawer's motion (the design's sheet, `.3s cubic-bezier(.2,.8,.2,1)`).

### Notes
- LV10 adds "Reframe" to this drawer's actions; keep the actions list a column that takes more buttons.
- The Item drawer on the live screen is unchanged in LV9 (LV10 adds its Guides row).

### Unit test commands
`cd apps/desktop && npx vitest run src/phone` · `npm run typecheck`

## Guide files (flow §7A)
U1: `docs/guide/02_architecture.md` (the phone view's `incoming`, CS4a's set read by `phone_view`). U2:
`docs/guide/01_product.md` (a photo on its way shows the cue in both strips; gave-up pictures are plain grey). U3:
`docs/guide/02_architecture.md` (tiles keyed by photo; the pending chain for drops and swipes),
`docs/guide/04_change_impact.md` (the preview token is the only reason to fetch a picture again; `revision` is not in the
URL). U4: `docs/guide/01_product.md` (the Preview: photo, strip, ⋯ drawer with details and Retake, ×; Move image gone).

## Closing gate (once, after the last unit)
`cd apps/desktop && npm run build` (first) · `cargo test --workspace` · `cargo clippy --workspace --all-targets -- -D
warnings` · `cargo fmt --all -- --check` · `tools/sidecar/build.sh --stub --test` (rebuilds only the stub the gate needs;
nothing else in `apps/desktop/src-tauri/binaries/` is rebuilt) · `cd apps/desktop && npm run typecheck` · `npm test`.
One command at a time.

## Review mode
The reviewer reads the units in order and runs, once: `cargo test -p psm-testkit --test suite lv9_phone_incoming::` ·
`cd apps/desktop && npx vitest run src/phone` · `npm run typecheck` · `npm run build`. Only B1–B4 block. Also check, by
reading: `incoming` comes from CS4a's set in core, the route is unchanged (H8); waiting tiles are outside homes, targets
and `arrangementKey`; every tile with a photo is keyed by the photo; no preview URL carries the revision; drops and swipes
share one pending chain whose ends are exactly PH9.5a's; the lift home is drawn only for a template shot's photo and only
while lifted; the Preview has no header or button block; the removed and changed tests are exactly those listed. Notes go
to `docs/implementation/backlog_live/LV9_preview_corrections.md`.

## Manual try-out (the narrative is built from this)
On this Mac with the camera connected (over Wi-Fi, so files take a few seconds) and the phone paired (the app built from
this branch), a templated session open:
1. Take a picture from the phone: at once the shot you took shows a faint light moving over grey, in the live screen's
   strip; open any photo's preview — its strip shows the same; a few seconds later the picture appears.
2. Take two pictures after the last template shot: two waiting tiles appear after the Condition images, then become the
   photos.
3. Press and hold a template shot's photo until it lifts: its place shows the shot's grey template picture with its
   number. Drag it onto another shot and let go: the two swap in one step, no blink, no old picture.
4. In a preview, swipe three Condition images left quickly one after the other: each disappears from the strip at once,
   and nothing comes back.
5. The preview shows only the photo, the strip, ⋯ at the bottom left and × at the bottom right. Tap ⋯: the photo's
   details and Retake. Tap ×: back to live.
6. On the Mac, check each change in the session view (⌘Z brings back a denied photo).

## Review log
