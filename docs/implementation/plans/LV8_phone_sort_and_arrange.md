---
phase: LV8_phone_sort_and_arrange
title: Sort and arrange on the phone — in the full-screen preview a swipe left denies, right makes Main (asking first for a template shot's photo), up sends to Condition, and the next photo shows; the preview carries the same photo strip as the live screen; in both strips press and hold lifts a photo to drag it — swap between template shots, insert and shift among extra Main photos and among Condition photos, to the end of the other group across groups — with the drop shown at once and taken back if the Mac refuses
difficulty: high
projection: no
depends_on: [LV7_tile_loading_cue]
implementer: fable
blocked_on: none
---
# LV8 — Sort and arrange on the phone (book 10J LV-D20)

**Tracker:** IMPLEMENTED (2026-11-27, fable) — 4 units checkpointed (`09ba8340`, `9cbb7f7c`, `0348cc8a`, `c7824ab9`): the Mac's classify route and insert moves; the Preview's swipes, confirmation and strip; lift-drag-drop in both strips; cancels and the pending drop. 70 row tests (14 Rust, 56 UI) and 20 core unit tests; `cargo test --workspace` 2774 passed, clippy, fmt, sidecar, typecheck and build green; `npm test` 1135 passed, 1 failed (the known `p33_split_grid` C4(d) flake, green alone).

(This line is the milestone's tracker row. Sessions change only this line of this file. The gate for `depends_on` is
the Tracker line of the plan named there.)

## Goal
Book `10J` **LV-D20** (owner 2026-11-27, changelog D20; built from the description, no design prototype): (1) **swipe to
classify** in the full-screen Preview — left = Deny, right = Main, up = Condition, then the next photo; (2) the Preview
shows **the live screen's photo strip**; (3) **press and hold to lift, drag, release to drop** in both strips — swap
between template shots, insert and shift among extra Main photos and among Condition photos; (4) Remove is the left
swipe; Retake stays. Reliability is the owner's first requirement ("usually i run into buggy drag and drop
behaviours"): the drag is a named state machine, every way it can end is a row, and the Mac's answer is the truth.
(U1) The Mac's side: a classify route and insert-and-shift moves. (U2) The Preview: swipes, the confirmation, the
strip, no Delete button. (U3) Lift, drag and drop. (U4) What cancels a drag; what the strip shows while the Mac answers.
**Not here:** the Mac's own review screens (unchanged; the Mac's drop onto a filled container still replaces, card
LV8); an undo button on the phone (Deny is undone on the Mac, ⌘Z, as for Retake); the Move drawer (kept as LV3 built
it); the main screen's gestures (hold for live, lock, overlay, focus: unchanged); camera code.

## How to work this plan (flow.md §10, master_plan_live.md §3)
One runner phase of 4 units, in order; after each, its **Unit test commands**, then a `CHECKPOINT (not approved):`
commit of that unit's paths. Row ids `LV8·C<n>(<row>)`; tests named after their row (`lv8_c3a_<slug>`; Rust
`lv8_c1a_<slug>`). Open details: 10J LV-D20 first, then the design README's tokens, timings and haptics
(`claude_design/live_preview/design_handoff_shoot_companion/README.md`, local, git-ignored; read, never copy), then
the smallest option; recorded under **Assumptions**. Reviewer notes: `docs/implementation/backlog_live/
LV8_phone_sort_and_arrange.md`. `implementer: fable` (phone interactions, owner 2026-11-26/27).
**Why the server part is a unit here and not its own milestone:** it is one core function and two move variants beside
`move_to`, one route and two DTO variants (4 criteria, no camera code, no schema); `HandAction::finish` already makes
the PH8 announcements; a separate default-implementer milestone would add a full review cycle for about a hundred
lines. U1 keeps H8: the rules are in `crates/core`, the route only parses, guards and calls.

| Unit | What it builds | Criteria | Rows | Changes app files |
|---|---|---|---|---|
| U1 | The Mac's side: `POST /photos/<id>/classify`; move targets `extra_insert`, `condition_insert` | 4 | 14 | yes (Rust) |
| U2 | The Preview: swipe left/right/up, the confirmation, the strip, no Delete | 4 | 22 | yes (TypeScript) |
| U3 | The strips: press-hold-lift, drag, drop (within and across groups), auto-scroll, the Preview's strip | 5 | 22 | yes (TypeScript, guide) |
| U4 | Reliability: cancels, the drop shown at once, rollback | 2 | 12 | yes (TypeScript) |
| **Total** | | **15** | **70** | |

## Read first (whole milestone)
`docs/process/flow.md` (§4, §4A, §6, §7A, §10) · `master_plan_live.md` §3, §4.2 (`template_placement`), **§5 CS7–CS8,
PH4, PH4a, PH6, PH7, PH8, PH9** (v1.2, this milestone's contract), §6 (phone texts, as amended v1.2) · book `10J`
**LV-D20**, LV-D19, card LV8, §The phone design · book `06` §Classification and §Order ("a photo moving between
groups goes to the end of the target") · `10G` §6 rule 1 · the design README §Preview gestures, §Preview actions,
§Move semantics, §Haptics, §Reliability requirements · plans `LV3_phone_app.md` (U3, U4 rows), `LV6_phone_dock.md`
(U1: the tap rule P2), `LV7_tile_loading_cue.md` · `apps/desktop/src/phone/{api.ts, state.ts, gestures.ts,
PhoneApp.tsx, phone.css, components/{Dock.tsx, Preview.tsx, MoveDrawer.tsx, PhotoImage.tsx, Stage.tsx}}`,
`apps/desktop/src/phone/__tests__/{helpers.ts, lv3_preview.test.tsx, lv6_dock.test.tsx, lv6_images.test.tsx}`.

## Points recorded by planning (each is one sentence in the narrative's "Choices made for you")
The book leaves these open; each is the smallest option consistent with LV-D20, 06 and card LV8.
- **P1 Lift:** a photo tile lifts after the finger has rested **400 ms** within 8 px; a shorter still touch is a tap
  (opens or selects, LV-D19 (1)); an empty shot or the Free shot tile never lifts (a long press on it is still a tap).
  Lifted, the tile grows to 1.15× with a shadow and Android vibrates 12 ms (the design's "magnify start").
- **P2 Swipe or hold in the Preview:** a move of more than 14 px within **200 ms** of touching the photo is a swipe; a
  template shot's photo held still for 200 ms shows its template as before (the hold now starts after 200 ms, not at
  the touch, so a swipe never flashes the template); once the hold has started, sideways sets the overlay and no swipe
  happens. The 200 ms is the design's own hold-versus-swipe time (dock §Shot dock gestures).
- **P3 A swipe counts at 96 px** along its direction; released shorter, the photo springs back and nothing happens.
  Downwards does nothing. On a sideways-held (long) shot the directions follow the picture, as every Preview gesture.
- **P4 After a swipe the next photo shows at once:** the next one in the strip's order (template shots in shooting
  order, extra Main photos, Condition photos); after the last one the Preview closes.
- **P5 Right on an extra Main photo and up on a Condition photo do nothing:** the photo springs back with the hint
  "ALREADY MAIN" / "ALREADY CONDITION".
- **P6 A template shot's photo sent to Main** (right swipe after "Move", or dragged into the extra Main photos) leaves
  its shot empty and makes it the current shot, as Deny and Condition do (LV-D20 says so for those two; one rule for
  all three).
- **P7 Across groups** (LV-D20 names only within-group drags): a photo dropped **on a template shot** swaps with it, or
  fills it when empty (card LV8, the Move drawer's rule); dropped **into the other group** it goes to that group's end
  (book 06); a template shot's photo dropped among the extras asks the same "Move this photo to Main…" question;
  dropped on the Free shot tile, nothing. A group with no photos has no place to drop into (use the swipe).
- **P8 The drop shows at once;** if the Mac refuses, the strip goes back and the phone says why; if the Mac has not
  answered in **8 s**, the strip goes back and asks the Mac again (the move may still have landed; the Mac's answer
  shows it).
- **P9 A drag ends with nothing changed** on: a second finger, the phone taking the touch away (a call, a system
  gesture), leaving the app, the Mac changing the photos while you drag, or letting go more than 48 px above or below
  the strip.
- **P10 At the strip's ends** (40 px) a dragged photo scrolls the strip, 8 px every 16 ms.
- **P11 The Preview's strip** marks the photo shown (not the current shot); a tap on a photo there shows it; a tap on
  an empty shot or the Free shot makes it current and closes the Preview.
- **P12 Toasts:** a swipe says what happened ("Image deleted", "Shot 02 reopened", "Moved to main images", "Sent to
  condition images"); a successful drop says nothing (the strip shows it); a refused drop says why.

## Shared for all units
- **Boundary (UI rows):** `PhoneApp` rendered with the Mac faked at `fetch`/`WebSocket` (`helpers.ts`), mouse events of
  the pointer types with a `pointerId` (jsdom has no `PointerEvent`; LV3 lesson, master plan §3), fake timers. The
  helpers gain `cancel(target, x, y)` (`pointercancel`) and an optional pointer id on `press`/`slide`/`lift` (for the
  second finger); the layout stub gains the scroller's `scrollWidth` and `clientWidth` (auto-scroll clamps to them).
- **Fixture S (U2):** `chairSession()` plus a second condition image `207`: shots B (final 02, photo 201), A (01,
  empty, current), C (03, empty); extra 202 (Main 04); condition 203 (C1), 207 (C2). The strip reads "02 01 03 04 ·
  C1 C2". Preview viewport 390 × 610 stage, ratio 1, centre (195, 305).
- **Fixture D (U3, U4):** shots in shooting order B (final 02, photo 201), A (01, photo 204), C (03, empty); extras
  202, 205, 206 (Main 04, 05, 06); condition 203, 207, 208 (C1–C3); current **Free**; `phoneEnv({stage: {width: 800,
  height: 610}})` unless a row says 390. Strip y 700–784, tile centre y 742. Tile centres x (the stub's 64 px pitch,
  +19 after the divider): B 44 · A 108 · C 172 · 202 236 · 205 300 · 206 364 · Free 428 · 203 511 · 207 575 · 208 639.
  "Lift X" = `press` at X's centre, `advance(400)`.
- **Visible hooks the rows assert** (CSS draws them; master plan PH9): the lifted tile has `data-lifted` and a
  `transform` with `scale(1.15)`; a swap target has `data-drop="swap"`; the tiles of a whole-group target have
  `data-drop="group"`; tiles slid aside for an insert have `translateX(±64px)` in their `transform`; the Preview's
  strip has `data-testid="preview-strip"` (the live screen's stays `dock`).
- **Core mechanisms** (the only unit tests, flow §4): new `apps/desktop/src/phone/arrange.ts` — `dropTarget(homes,
  source, fingerX, inBand)` (PH9.3), `applyDrop(view, photoId, drop)` (the arrangement the strip shows while the Mac
  answers), `arrangementKey(view)` (PH9.4), `nextPhoto(view, photoId)` (P4), `swipeAction(subject, direction)` (PH9.1).
  Unit tests `lv8_core_<fn>_…` in `__tests__/lv8_core.test.ts`, input → output, one case per rule line.

## Unit 1 — The Mac's side: classify, and insert-and-shift moves (LV-D20 (1), (3); PH4a)

**Unit id:** `LV8·U1` · **Rows:** LV8·C1–C4 · **Changes app files:** yes (Rust)

### Goal
`template_placement::classify_to(h, photo_id, ClassifyTo::{Main, Condition})` (core) and two new `MoveTarget`
variants, `ExtraInsert(i64)` and `ConditionInsert(i64)`, inside `move_to`'s phone model (containers by final number,
then extras; Condition in order). Route `POST /api/phone/v1/photos/{id}/classify {session_id, to: "main"|"condition"}`
→ `{ok: true}`; move DTO kinds `extra_insert {index}`, `condition_insert {index}` (PH4a).
- **classify to Main:** a Condition photo → Main at the end (as `ExtraEnd`); a photo in a template container → an extra
  at the end, and that container becomes current (P6); an extra Main photo (or any Main photo of an untemplated
  session) → nothing changes (200, no step, no event).
- **classify to Condition:** a Main photo (in a container or an extra) → Condition at the end (as `ConditionEnd`); from
  a container, that container becomes current; a Condition photo → nothing changes.
- **The current shot after a take-out** is set like Retake's (`CurrentShots::set_after_take_out(s, c)`, log
  `current_shot.set … by=classify`), **after** the move committed and **before** the announcement, so the phone's
  re-read sees both (CS1 moves it on if `c` is taken by a shutter on the way). One undo step "Move image"; undo does
  not restore the current shot (it is in memory, CS1).
- **Insert:** the source must already be in that group (an extra, or a Main photo of an untemplated session, for
  `extra_insert`; a Condition photo for `condition_insert`), else `invalid_value`; `index` (1-based, the photo's place
  after the move) in `1..=group length`, else `invalid_value`; the photo is taken out and put back at `index`, the
  others keep their order; same place → nothing changes. Positions only: place rows are untouched (as LV2·C10(d)'s
  swap). **Not** `ReviewService::reorder`: it marks the photo placed by hand in slot = position (the Mac's container
  semantics) and would put an extra into a container.
- Any other photo state (Deleted, Unreviewed) → `invalid_value`. The session guard (PH4) and `session_photo` (404) as
  the move route.

### Read first
`crates/core/src/services/template_placement.rs` (`move_to`, `MoveTarget`, `HandAction::finish`, `retake`),
`crates/core/src/services/current_shot.rs` (`set_after_retake`), `crates/integrations/src/phone/{routes.rs
(move_photo, delete_photo), wire.rs (MoveBody, MoveTargetDto)}`, `crates/testkit/tests/suite/{lv2_phone_ops.rs (t3,
session, set, arrive, place, by_hand, group), lv2_phone_http.rs (the request fixture)}`, `crates/testkit/src/phone.rs`.

### Files
Edited: `crates/core/src/services/template_placement.rs`, `crates/core/src/services/current_shot.rs`,
`crates/integrations/src/phone/{routes.rs, wire.rs}`. New: `crates/testkit/tests/suite/lv8_phone_sort.rs` (+ its `mod`
line in `crates/testkit/tests/suite/main.rs`). Guide: `docs/guide/02_architecture.md` (the phone's classify and insert
moves, beside "Phone Move").

### Core mechanisms
None new (the variants live in `move_to`; rows test them at the route).

### Ordered tasks
1. `MoveTarget::ExtraInsert`, `ConditionInsert` in `move_to`'s model; `classify_to` (the move part shares `move_to`'s
   transaction code; no second copy of the model).
2. `set_after_take_out`; the order: commit → current → `finish`.
3. Wire and route (thin: parse, guard, `session_photo`, call).
4. Tests for the rows; guide.

### Criteria (4 criteria, 14 rows)
Boundary: HTTP through `TestPhone` with a paired token (as `lv2_phone_http.rs`), then the store (`place`, `group`,
`by_hand` as `lv2_phone_ops.rs`), the phone view (`current`), the history and the recorded events. **Fixture U1:** T3 of
`lv2_phone_ops.rs` (A final 1 shoot 2, B final 2 shoot 1, C final 3 shoot 3); templated session S active; P in A, Q
in C (`p_in_a_q_in_c`); three arrivals K1, K2, K3, each with the current shot set to Free first (as LV2·C10(d)), so
they are Condition in that order; three more arrivals made the same way and moved with `MoveTarget::ExtraEnd` to give
extras E1, E2, E3 (Main 3, 4, 5). After the fixture the current shot is B (CS4: after a Free binding, the first
untaken shot in shooting order); events cleared after the fixture.
Untemplated session U (item type without a template) with Main U1, U2, U3 and Condition V1 (classified on the Mac).
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| LV8·C1(a) | classify → main | K2, `{session_id: S, to: "main"}` | 200 `{ok: true}`; Main = P, Q, E1, E2, E3, K2 (positions 1–6, K2 no slot, placed by hand); Condition = K1, K3 (1, 2); history undo label "Move image"; events = `PhotoChanged` for K2 and K3, then one `SessionChanged{S}` | LV-D20 (1), 06 Order, PH8 |
| LV8·C1(b) | same | P (in A) | Main = Q, E1, E2, E3, P (P no slot, by hand); no photo in A; the phone view's `current` = `{kind: "shot", container_id: A}`; one undo restores P to A (slot 1) | LV-D20 (1), P6, CS7 |
| LV8·C1(c) | same | E2 | 200; places, positions and history unchanged; no event | P5, LV-D20 (1) |
| LV8·C2(a) | classify → condition | Q (in C) | Condition = K1, K2, K3, Q; no photo in C; `current` = `{kind: "shot", container_id: C}`; Q by hand | LV-D20 (1) |
| LV8·C2(b) | same | E1 | Condition = K1, K2, K3, E1; Main = P, Q, E2, E3; `current` = `{kind: "shot", container_id: B}` before and after | LV-D20 (1), 06 |
| LV8·C2(c) | same | K1 | 200; nothing changes; no event | P5 |
| LV8·C2(d) | untemplated | V1 → main; then U1 → condition | Main = U1, U2, U3, V1 after the first; Main = U2, U3, V1 and Condition = U1 after the second; `current` null throughout | 06, P66 |
| LV8·C3(a) | move `extra_insert` | E3, `{kind: "extra_insert", index: 1}` | Main = P, Q, E3, E1, E2; slots of P (1) and Q (3) unchanged; no place row changed; one undo step "Move image"; `PhotoChanged` for E1, E2, E3 | LV-D20 (3) |
| LV8·C3(b) | move `condition_insert` | K1, `{kind: "condition_insert", index: 3}` | Condition = K2, K3, K1 | LV-D20 (3) |
| LV8·C3(c) | untemplated | U3, `extra_insert` index 1 | Main = U3, U1, U2 | LV-D20 (3) |
| LV8·C3(d) | same place | E2, `extra_insert` index 2 | 200; nothing changes; no step; no event | PH4a |
| LV8·C4(a) | refused | K1 `extra_insert` 1; P `extra_insert` 1; E1 `condition_insert` 1; E1 `extra_insert` 0; E1 `extra_insert` 4 | each 400 `invalid_value`; nothing changes | PH4a |
| LV8·C4(b) | session guard | classify K1 with `session_id` = another session | 409 `session_changed`; nothing changes | PH4, Invariant PH |
| LV8·C4(c) | not this session's photo | classify a photo of another session (S active) | 404 `not_found`; nothing changes | PH4 (`session_photo`) |

### Delegated choices
Whether `classify_to` is `move_to` with a target plus the current step, or `move_to` refactored around a shared inner
function · the `ClassifyBody` field type (`enum` with serde `rename_all = "lowercase"` or a string checked by hand).

### Notes
- `features` in `hello` is not changed (the phone and the server ship together).
- C1(a) also leaves the current shot B as it was (not asserted separately: C2(b) is the row that forbids a change).

### Unit test commands
`cargo test -p psm-testkit --test suite lv8_phone_sort::` · `cargo check -p psm-integrations`

## Unit 2 — The Preview: swipe to classify, the confirmation, the strip (LV-D20 (1), (2), (4))

**Unit id:** `LV8·U2` · **Rows:** LV8·C5–C8 · **Changes app files:** yes (TypeScript)

### Goal
`Preview.tsx` gets the swipe of PH9.1 on its image (state machine `idle → pressing → swiping | holding → committed |
cancelled`, P2, P3) with the photo following the finger along the decided axis and a hint ("DENY" in red, "MAIN",
"CONDITION", or "ALREADY MAIN" / "ALREADY CONDITION") whose opacity grows with the distance. On commit `PhoneApp`
sends the request (`retake` for a template shot's photo, `delete` for an extra or condition image, `classify` for Main
and Condition), shows the toast, vibrates, and **at once** shows the next photo (P4: `nextPhoto` of the view at
commit; the preview state changes before any re-read can close it as orphaned). Right on a template shot's photo opens
the **confirmation sheet** (new `components/ConfirmSheet.tsx`): "Move this photo to Main, outside its template
order?" with **Move** and **Cancel** (the backdrop is Cancel). The Preview shows the strip (`Dock` with
`testId="preview-strip"`) between the picture and the buttons, marking the photo shown (P11). The **Delete** button
goes (LV-D20 (4)); Back, Retake (template shots) and Move image stay.

### Read first
`components/Preview.tsx` (hold, overlay slide), `PhoneApp.tsx` (`retakeOrDelete`, `move`, `refused`, the orphaned
effect), `state.ts` (`previewSubject`, `TEXT`), `gestures.ts` (`SLIDE_PX`, `HAPTIC`), `geometry.ts` (`localPoint`),
`__tests__/lv3_preview.test.tsx`.

### Files
New: `apps/desktop/src/phone/arrange.ts` (`nextPhoto`, `swipeAction`; U3 adds the rest), `components/ConfirmSheet.tsx`,
`__tests__/lv8_swipe.test.tsx`, `__tests__/lv8_core.test.ts`. Edited: `api.ts` (`classifyPhoto`; `MoveTarget` +
`extra_insert`, `condition_insert`), `state.ts` (texts), `gestures.ts` (`SWIPE`), `PhoneApp.tsx`,
`components/{Preview.tsx, Dock.tsx (a `testId` and a "marked photo" prop)}`, `phone.css`, `__tests__/helpers.ts`.
**LV3/LV6 tests changed (each listed in the handoff with the reason):**
- `lv3_c22c_delete_denies_a_condition_image` **removed** (LV-D20 (4): no Delete button; its outcome is LV8·C6(a)).
- `lv3_c21a_…` and `lv3_c21b_…`: the hold starts after 200 ms (P2): `advance(200)` after `press` before the template
  shows or the slide starts; their rows unchanged.
- `lv6_c5a_…`: `previewRequests(preview)` scoped to the viewport (the Preview now contains the strip's tiles); row
  unchanged.

### Core mechanisms
`nextPhoto(view, photoId) -> number | null` (P4) · `swipeAction(subject, direction) -> "retake" | "delete" | "main" |
"main_confirm" | "condition" | "none"` (PH9.1).

### Ordered tasks
1. `SWIPE = {holdMs: 200, startPx: 14, commitPx: 96, hintPx: 24}` in `gestures.ts`; `arrange.ts` `nextPhoto`,
   `swipeAction` and their unit tests.
2. The Preview's touch state machine; the hint; the confirmation sheet.
3. `PhoneApp`: the swipe actions, toasts, haptics, the next photo; failures through `refused`.
4. The strip in the Preview (tap rules P11); no Delete button.
5. Tests for the rows; the changed LV3/LV6 tests.

### Criteria (4 criteria, 22 rows)
Boundary: `PhoneApp` with fixture S; the Preview opened by a tap on the dock (as `lv3_preview.test.tsx`); swipes on
`preview-viewport` from (195, 305). "Swipe left 120" = `press` (195, 305), `slide` (75, 305), `lift` (75, 305) within
the same fake-timer instant (0 ms elapsed); right/up likewise (315, 305) / (195, 185).
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| LV8·C5(a) | swipe, not hold | Preview of 201; `press`, `advance(199)`, `slide` to (165, 305) | the pill is not "TEMPLATE"; the photo's opacity 1; its `transform` contains `translateX(-30px)`; the hint reads "DENY"; then `slide` back to (190, 305) and `lift`: no POST, the `transform` has no translate, the kicker still "SHOT 02 · CAPTURED", `vibrate` not called | P2, PH9.1 |
| LV8·C5(b) | hold, not swipe | Preview of 201; `press` (100, 305), `advance(200)` still; `slide` (120, 308); `lift` | at 200 ms the pill "TEMPLATE", opacity 0.35, `vibrate(10)`; the slide shows "PHOTO 42%"; no hint; no POST | P2, LV3·C21 kept |
| LV8·C5(c) | short swipe | Preview of 202; `press`, `slide` to (100, 305) (95 px), `lift` | no POST; the photo's `transform` back to none; still "IMAGE 04 · EXTRA"; no toast | P3 |
| LV8·C5(d) | downwards | Preview of 202; `press`, `slide` to (195, 425), `lift` | no hint; no translate; no POST | P3 |
| LV8·C5(e) | interrupted | Preview of 202: (1) `press`, `slide` to (75, 305), `cancel`; (2) the same, then the page hidden (`setVisibility('hidden')`) before `lift` | in both: no POST; the `transform` back to none; still on 202 | PH9.1, design §Reliability |
| LV8·C5(f) | rotated | B's shot with ratio 1.7778 (the Preview turns, `isRotated`); Preview of 201; `press` (195, 305), `slide` (195, 185), `lift` | one POST `/retake {session_id: 7, container_id: B}` (screen-up = the picture's left) | P3, 10J (gestures follow the image) |
| LV8·C6(a) | left on Condition | Preview of 203; swipe left 120 | POSTs = `[["/photos/203/delete", {session_id: 7}]]`; `vibrate([30,40,30])`; toast "Image deleted"; at once the kicker "CONDITION 02" (207); a `GET /session` follows | LV-D20 (1), (4) |
| LV8·C6(b) | left on a template shot | Preview of 201; swipe left 120 | POSTs = `[["/retake", {session_id: 7, container_id: B}]]`; toast "Shot 02 reopened"; at once "IMAGE 04 · EXTRA" (202) | LV-D20 (1) (Retake semantics, MC-LV3) |
| LV8·C6(c) | right on the last | Preview of 207; swipe right 120 | POSTs = `[["/photos/207/classify", {session_id: 7, to: "main"}]]`; `vibrate([30,40,60])`; toast "Moved to main images"; the Preview closes (no photo after it) | LV-D20 (1), P4 |
| LV8·C6(d) | up on an extra | Preview of 202; swipe up 120 | `[["/photos/202/classify", {session_id: 7, to: "condition"}]]`; toast "Sent to condition images"; at once "CONDITION 01" (203) | LV-D20 (1) |
| LV8·C6(e) | up on a template shot | Preview of 201; swipe up 120 | `[["/photos/201/classify", {session_id: 7, to: "condition"}]]`; at once 202 | LV-D20 (1) |
| LV8·C6(f) | nothing to do | Preview of 202: `press`, `slide` to (315, 305) (hint), `lift`; Preview of 203: swipe up 120 | hints "ALREADY MAIN" / "ALREADY CONDITION" while sliding; no POST; still on 202 / 203; no toast | P5 |
| LV8·C6(g) | the Mac refuses | Preview of 203; `/photos/203/delete` answers 409 `session_changed`; swipe left 120 | toast "Session changed on the Mac"; a `GET /session` follows; the Preview on 207 (it does not go back) | PH4 guard, §6 texts |
| LV8·C7(a) | right on a template shot | Preview of 201; swipe right 120 | no POST; a dialog with the text "Move this photo to Main, outside its template order?" and buttons "Move" and "Cancel"; the photo's `transform` none; still 201 | LV-D20 (1) |
| LV8·C7(b) | Move | then "Move" | POSTs = `[["/photos/201/classify", {session_id: 7, to: "main"}]]`; `vibrate([30,40,60])`; toast "Moved to main images"; the dialog gone; at once 202 | LV-D20 (1), P6 |
| LV8·C7(c) | Cancel | again from (a): "Cancel"; separately a tap on the backdrop | no POST; the dialog gone; still 201 | LV-D20 (1) |
| LV8·C8(a) | the strip | Preview of 201 | `preview-strip` reads "02010304C1C2" (the dock's tiles, same order); the tile of 201 is outlined, no other | LV-D20 (2), P11 |
| LV8·C8(b) | tap a photo in it | Preview of 201; tap "04", then "C1" in `preview-strip` | the Preview shows 202 ("IMAGE 04 · EXTRA"), then 203; no POST; the outlined tile follows | LV-D20 (2), P11 |
| LV8·C8(c) | tap an empty shot in it | Preview of 201; tap "03" | POSTs = `[["/current", {session_id: 7, container_id: C}]]`; the Preview closes | P11, MC-LV3 |
| LV8·C8(d) | the strip follows a swipe | Preview of 201; swipe left 120 | the outlined tile is "04" | P4, P11 |
| LV8·C8(e) | buttons | Previews of 202, 203, 201 | 202 and 203: no "Delete" button, "Move image" and "Back to shot 01" present; 201: "Retake" present | LV-D20 (4) |
| LV8·C8(f) | the live dock under it | Preview of 201 open; tap at the live dock's "C1" position | nothing happens (the live screen's dock takes no touch while a Preview is open); no POST | LV6 kept |

### Delegated choices
The swipe's look within the tokens (a slight rotation with the horizontal translate, the red tint for Deny, the hint's
place and type) · the confirmation sheet's look (the design's sheet: `.3s cubic-bezier(.2,.8,.2,1)`, scrim
`rgba(0,0,0,.4)`) · a short fly-out of the swiped picture, **only** as a layer over the next photo (the switch to the
next photo is never delayed by it) · whether the strip in the Preview sits directly under the picture or above the
buttons (both between them) · scrolling the Preview's strip so the marked tile is visible.

### Notes
- **Hazard (found in planning):** the existing orphaned-preview effect closes the Preview when its photo leaves the
  session. After a Deny the re-read removes the photo; if the next photo were shown only after an animation, the
  Preview would close first. Rows C6(a)–(e) assert the next photo "at once" (`advance(0)`).
- The hold-for-template now starts after 200 ms; the design's "Hold start `vibrate(10)`" moves with it.
- The Move drawer, Back and Retake buttons behave as LV3 built them.

### Unit test commands
`cd apps/desktop && npx vitest run src/phone` · `npm run typecheck`

## Unit 3 — Lift, drag and drop in both strips (LV-D20 (3); PH9.2, PH9.3)

**Unit id:** `LV8·U3` · **Rows:** LV8·C9–C13 · **Changes app files:** yes (TypeScript, guide)

### Goal
`Dock.tsx` (both strips) runs PH9.2's state machine: `idle → pressing → lifted → dragging → dropping | cancelled → idle`
(and `pressing → scrolling | moved | tap`). The tile **homes** (content coordinates) are measured **once, at lift**;
targets are computed from them (PH9.3, `dropTarget`), never from the tiles' current on-screen rectangles, so tiles
sliding aside can never change the target under the finger. While dragging: the lifted tile follows the finger; a swap
target is marked; for an insert the other tiles of the group slide one pitch aside (150 ms); a whole-group target is
marked; `vibrate(6)` each time the target changes to another non-empty target. On release with a target: the request
(PH9.3 table), `vibrate([30,40,60])`, and the arrangement shown at once (`applyDrop`, U4 handles the answer); a
template shot's photo dropped among the extras opens U2's confirmation instead. Auto-scroll at the ends (P10). Taps and
sideways scrolling work as LV6 built them.

### Read first
`components/Dock.tsx`, `gestures.ts` (`DOCK`), `state.ts` (`dockTiles`), `PhoneApp.tsx` (`select`, `move`),
`__tests__/{helpers.ts (layout stub), lv6_dock.test.tsx}`, master plan **PH9.2, PH9.3**.

### Files
New: `__tests__/lv8_drag.test.tsx`. Edited: `arrange.ts` (`dropTarget`, `applyDrop`), `gestures.ts` (`DRAG`, `HAPTIC.lift
= 12`, `HAPTIC.dragTick = 6`), `components/Dock.tsx`, `PhoneApp.tsx`, `phone.css` (`touch-action: none` and
`-webkit-touch-callout: none` on the strips, the lifted look, the slide-aside transition), `__tests__/helpers.ts`,
`__tests__/lv8_core.test.ts`. Guide: `docs/guide/01_product.md` (the phone's Preview swipes, strip, lift-and-drag).
**LV6 test removed:** `lv6_c2c_a_long_press_is_a_tap_and_magnifies_nothing` (LV-D20 amends LV-D19 (1): a long press on
a photo tile now lifts; its outcome is LV8·C9(b)). `lv6_c2d_…` stays (its 300 ms press is under the 400 ms lift).

### Core mechanisms
`dropTarget(homes, source, fingerX, inBand) -> Drop | null` (PH9.3) · `applyDrop(view, photoId, drop) -> PhoneSession`
(each drop kind: swap filled/empty, insert, to group end; `current` untouched).

### Ordered tasks
1. `DRAG = {liftMs: 400, liftScale: 1.15, bandPx: 48, edgePx: 40, scrollStepPx: 8, scrollTickMs: 16, pendingMs:
   8000}`; `dropTarget`, `applyDrop` and their unit tests.
2. The state machine in `Dock.tsx` with pointer capture; homes at lift; the visuals; the auto-scroll timer (a 16 ms
   interval, so fake timers drive it).
3. `PhoneApp`: the drop → request (move or classify; the confirmation for a template photo to the extras).
4. Remove `lv6_c2c`; tests for the rows; guide.

### Criteria (5 criteria, 22 rows)
Boundary: `PhoneApp` with fixture D; the live screen's `dock` unless a row names the Preview's strip.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| LV8·C9(a) | a still tap | `press` B, `advance(399)`, `lift` | the Preview of 201 opens; no tile ever had `data-lifted`; `vibrate` calls `[[15]]` | LV-D19 (1) kept, P1 |
| LV8·C9(b) | lift, put down | Lift B; then `lift` at (44, 742) | at 400 ms B has `data-lifted` and `scale(1.15)`, `vibrate` `[[12]]`; after release: no POST, no Preview, no `data-lifted`, B's `transform` without scale | LV-D20 (3), P1 |
| LV8·C9(c) | empty shot | `press` C, `advance(600)`, `lift` | never `data-lifted`; POSTs = `[["/current", {session_id: 7, container_id: C}]]` | P1, LV6 P2 |
| LV8·C9(d) | scroll first | `press` 202, `advance(100)`, `slide` to (227, 742), `advance(500)`, `lift` | never `data-lifted`; the strip's `scrollLeft` 9; no POST | LV-D20 (3) (scroll without a hold) |
| LV8·C10(a) | swap, filled | Lift B, `slide` (108, 742), `lift` | while over A: A has `data-drop="swap"`; POSTs = `[["/photos/201/move", {session_id: 7, target: {kind: "container", container_id: A}}]]`; `vibrate` `[[12],[6],[30,40,60]]` | LV-D20 (3), card LV8 |
| LV8·C10(b) | swap, empty | Lift B, `slide` (172, 742), `lift` | `[["/photos/201/move", {…, target: {kind: "container", container_id: C}}]]` | card LV8 |
| LV8·C10(c) | insert among extras | Lift 202, `slide` (330, 742); `slide` (380, 742); `lift` | at 330: 205's `transform` has `translateX(-64px)`, 206's has none; at 380: both have it; POSTs = `[["/photos/202/move", {session_id: 7, target: {kind: "extra_insert", index: 3}}]]` | LV-D20 (3) |
| LV8·C10(d) | insert among Condition | Lift 208, `slide` (540, 742), `lift` | 207 has `translateX(64px)`, 203 none; `[["/photos/208/move", {…, target: {kind: "condition_insert", index: 2}}]]` | LV-D20 (3) |
| LV8·C10(e) | own place | Lift 205, `slide` (310, 742), `lift` | no POST; no tile slid; `vibrate` `[[12]]` | PH9.3 |
| LV8·C11(a) | extra onto a shot | Lift 202, `slide` (172, 742), `lift` | `[["/photos/202/move", {…, target: {kind: "container", container_id: C}}]]` | P7, card LV8 |
| LV8·C11(b) | Condition onto a filled shot | Lift 203 (511), `slide` (108, 742), `lift` | `[["/photos/203/move", {…, target: {kind: "container", container_id: A}}]]` | P7, card LV8 |
| LV8·C11(c) | extra into Condition | Lift 202, `slide` (575, 742), `lift` | while there 203, 207, 208 have `data-drop="group"`; `[["/photos/202/classify", {session_id: 7, to: "condition"}]]` | P7, 06 Order |
| LV8·C11(d) | Condition into the extras | Lift 203, `slide` (300, 742), `lift` | `[["/photos/203/classify", {session_id: 7, to: "main"}]]`; no dialog | P7, 06 Order |
| LV8·C11(e) | template photo into Condition | Lift B, `slide` (575, 742), `lift` | `[["/photos/201/classify", {session_id: 7, to: "condition"}]]` | P7, LV-D20 (1) |
| LV8·C11(f) | template photo into the extras | Lift B, `slide` (300, 742), `lift`; then "Move"; separately "Cancel" | no POST on release; U2's dialog text; "Move" → `[["/photos/201/classify", {…, to: "main"}]]`; "Cancel" → no POST | P7, LV-D20 (1) |
| LV8·C11(g) | onto Free shot | Lift 202, `slide` (428, 742), `lift` | no POST; `[30,40,60]` never vibrated | P7 |
| LV8·C12(a) | auto-scroll right | stage 390; Lift 202, `slide` (370, 742), `advance(160)`; `slide` (200, 742), `advance(160)` | `scrollLeft` 80 after the first 160 ms; still 80 after the second | P10 |
| LV8·C12(b) | auto-scroll left, clamped | from (a)'s end: `slide` (20, 742), `advance(160)`, `advance(160)` | `scrollLeft` 0 after the first; 0 after the second | P10 |
| LV8·C12(c) | the target follows the scroll | stage 390; Lift 202, `slide` (370, 742), `advance(160)`, `slide` (300, 742), `lift` | `[["/photos/202/move", {…, target: {kind: "extra_insert", index: 3}}]]` (content x 380, PH9.3) | P10, PH9.3 |
| LV8·C12(d) | no hold, no auto-scroll | stage 390; `press` 202, `slide` (20, 742) at once, `advance(160)`, `lift` | `scrollLeft` 216 after the slide (LV6's scroll: start − dx) and still 216 after 160 ms in the edge zone; never `data-lifted`; no POST | LV6·C3 kept, P10 |
| LV8·C13(a) | the Preview's strip | open the Preview of 201 (tap B); in `preview-strip` Lift 202, `slide` (172, 742), `lift` | `[["/photos/202/move", {…, target: {kind: "container", container_id: C}}]]`; the Preview still "SHOT 02 · CAPTURED" | LV-D20 (2), (3) |
| LV8·C13(b) | the Preview's strip, its own photo out | in the Preview of 201, Lift B in `preview-strip`, `slide` (575, 742), `lift`; the Mac's view then has 201 as the fourth Condition image | `[["/photos/201/classify", {…, to: "condition"}]]`; once the re-read answers, the Preview shows "CONDITION 04" (it follows its photo by id) | LV-D20 (2), design §Move semantics |

### Delegated choices
The lifted look within the tokens (shadow, the 1.15 scale's origin, z-order) · the marking of a swap target and of a
whole-group target · whether the source's home shows an empty outline while dragging · the lifted tile's offset under
the finger (where it was grabbed, or centred).

### Notes
- **Hazard:** in a real browser `getBoundingClientRect` includes CSS transforms; measuring targets from tiles that are
  sliding aside makes the target flip back and forth under a still finger. Homes are taken once at lift (PH9.3).
- **Hazard:** no HTML5 drag-and-drop (`draggable`, `dragstart`): unreliable on iOS. Pointer events with
  `setPointerCapture` only; images keep `draggable={false}`.
- LV8·C12(d) restates LV6's unclamped jsdom scroll (LV6 backlog note); a real browser stops at the strip's end.

### Unit test commands
`cd apps/desktop && npx vitest run src/phone` · `npm run typecheck`

## Unit 4 — Reliability: what cancels a drag, what shows while the Mac answers (PH9.4, PH9.5)

**Unit id:** `LV8·U4` · **Rows:** LV8·C14–C15 · **Changes app files:** yes (TypeScript)

### Goal
PH9.4: in `pressing`, `lifted` or `dragging`, each of these cancels (no request; the tile goes home; tiles slide back;
state `idle`): `pointercancel`; a second pointer down on the strip (its id differs); the page hidden; window `blur`;
a session view whose `arrangementKey` differs from the one at lift; the strip unmounting (its Preview closed); release with no target (more than `bandPx` above or below the strip, the Free tile, its own place). A view
with the **same** key (a preview token, a revision, the current shot) does not cancel. PH9.5: after a drop the strip
shows `applyDrop(view at drop, drop)` until (a) the read of the session **started after the Mac answered** returns
(then the Mac's view), or (b) the Mac refuses (then the last view from the Mac, the `refused` toast, a re-read), or (c)
`pendingMs` (8 s) pass without an answer (then the last view from the Mac and a re-read). While a drop is pending no
tile lifts (a long press is a tap).

### Read first
`PhoneApp.tsx` (`readSession`'s newer-answer rule, `refused`, the blur/visibility effect), `components/Dock.tsx` (U3),
master plan **PH9.4, PH9.5**, `__tests__/helpers.ts` (`FakeMac.hold/release/answer`, `setVisibility`, `macChanges`).

### Files
New: `__tests__/lv8_drag_reliability.test.tsx`. Edited: `arrange.ts` (`arrangementKey`), `components/Dock.tsx`,
`PhoneApp.tsx`, `__tests__/{helpers.ts, lv8_core.test.ts}`.

### Core mechanisms
`arrangementKey(view) -> string`: the shots' `container_id:photo_id|-` in shooting order, then the extras' ids, then the
Condition ids (nothing else).

### Ordered tasks
1. `arrangementKey` and its unit test.
2. The cancels (one function every cause calls).
3. The pending drop: the shown arrangement, the three ends, no lift while pending.
4. Tests for the rows.

### Criteria (2 criteria, 12 rows)
Boundary: `PhoneApp` with fixture D (stage 800); "drag 202 to 380" = Lift 202, `slide` (330, 742), `slide` (380, 742).
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| LV8·C14(a) | pointer cancel | drag 202 to 380; `cancel` | no `data-lifted`; no tile has a translate; a later `lift` sends nothing; `vibrate` has no `[30,40,60]` | PH9.4, design §Reliability |
| LV8·C14(b) | a second finger | drag 202 to 380; `press` with pointer id 2 at (600, 742); `lift` pointer 1 at (380, 742) | cancelled at the second press as (a); no POST | PH9.4 |
| LV8·C14(c) | leaving the app | drag 202 to 380; `setVisibility('hidden')`; separately a window `blur` | cancelled as (a) in both; no POST | PH9.4 |
| LV8·C14(d) | let go away from the strip | Lift 202, `slide` (380, 600), `lift` (380, 600) | while at y 600 no tile slid or marked; no POST | PH9.4, P9 |
| LV8·C14(e) | the Mac changes the photos | drag 202 to 380; `macChanges` with a fourth Condition photo 209 | cancelled at once (no `data-lifted`); the strip shows C1–C4; a later `lift` sends nothing | PH9.4 |
| LV8·C14(f) | the Mac changes only pictures | drag 202 to 380; `macChanges` with 205's `preview` "k1" | 202 still `data-lifted`; 205 and 206 still slid; `lift` → `[["/photos/202/move", {…, target: {kind: "extra_insert", index: 3}}]]` | PH9.4 |
| LV8·C15(a) | shown at once | `mac.hold()`; drag 202 to 380, `lift`; then `mac.release()` with the Mac's view now E order 205, 206, 202 | before the release the strip's keys read `… p205, p206, p202, free …`; after the POST is answered a `GET /session` is sent and its answer is shown (same order) | PH9.5 |
| LV8·C15(b) | refused, session changed | the move answers 409 `session_changed`; drag 202 to 380, `lift` | after the answer the strip shows 202, 205, 206 again; toast "Session changed on the Mac"; a `GET /session` follows | PH9.5, §6 texts |
| LV8·C15(c) | refused, other | the move answers 400 `invalid_value` | 202, 205, 206 again; toast "That did not work. Try again." | PH9.5 |
| LV8·C15(d) | no answer | `mac.hold()`; drag 202 to 380, `lift`; `advance(7999)`; `advance(1)` | at 7999 ms the strip still shows 205, 206, 202; at 8000 ms it shows 202, 205, 206 and a `GET /session` has been asked after the POST; no toast | PH9.5, P8 |
| LV8·C15(e) | no lift while pending | `mac.hold()`; drop as (a); `press` 205, `advance(600)`, `lift` | 205 never `data-lifted`; `vibrate` has no second `12`; the Preview of 205 opens (a tap) | PH9.5 |
| LV8·C15(f) | a later view wins | `mac.hold()`; drop as (a); `mac.release()`; the re-read answers 202, 206, 205 (the Mac did something else) | the strip shows 202, 206, 205 | PH9.5 (the Mac is the truth) |

### Delegated choices
Whether the pending drop lives in `PhoneApp` state or in `Dock` (it must survive the Preview opening and closing) · the
cancel's spring-back animation (≤ 200 ms).

### Notes
- A session message while a drop is pending triggers a read as always; its answer is stored but the strip keeps the
  pending arrangement until one of PH9.5's three ends (a read started **before** the answer may predate the move).
- The swipes (U2) are not optimistic in the strip: the Preview moves on at once and the strip shows the re-read.

### Unit test commands
`cd apps/desktop && npx vitest run src/phone` · `npm run typecheck`

## Guide files (flow §7A)
U1: `docs/guide/02_architecture.md` (the phone's classify route and insert moves; `set_after_take_out`; the
`by=classify` log). U3: `docs/guide/01_product.md` (the phone's Preview: swipe left/right/up, the confirmation, the
strip, no Delete; both strips: press and hold to lift, drag, drop: swap, insert, to the other group's end; the drop
shown at once and taken back when refused).

## Closing gate (once, after the last unit)
`cd apps/desktop && npm run build` (first) · `cargo test --workspace` · `cargo clippy --workspace --all-targets -- -D
warnings` · `cargo fmt --all -- --check` · `tools/sidecar/build.sh --stub --test` · `cd apps/desktop && npm run
typecheck` · `npm test`. One command at a time.

## Review mode
The reviewer reads the units in order and runs, once: `cargo test -p psm-testkit --test suite lv8_phone_sort::` · `cd
apps/desktop && npx vitest run src/phone` · `npm run typecheck` · `npm run build`. Only B1–B4 block. Also check, by
reading: targets come from homes measured at lift, never from live rectangles (PH9.3); no `draggable`/`dragstart` and
every strip handler uses pointer capture; every cancel cause of PH9.4 reaches the same cancel; the pending drop has
exactly PH9.5's three ends; the next photo after a swipe is set before any re-read can orphan the Preview;
`classify_to` sets the current shot before the announcement; the route is thin (H8); the removed and changed LV3/LV6
tests are exactly those listed. Notes go to `docs/implementation/backlog_live/LV8_phone_sort_and_arrange.md`.

## Manual try-out (the narrative is built from this)
On this Mac with the camera connected and a phone paired (the app built from this branch), a templated session open:
1. Take five or six pictures, some past the last template shot (they become Condition images).
2. Tap a Condition image in the strip: its preview opens with the strip under it. Swipe it to the right: it becomes a
   Main photo and the next image shows. Swipe the next one up: nothing happens ("ALREADY CONDITION").
3. Swipe a template shot's photo to the right: the phone asks "Move this photo to Main, outside its template order?";
   Cancel keeps it; Move makes it an extra Main photo and that shot is current again. Swipe another to the left: it is
   denied (red on the Mac, ⌘Z there brings it back) and its shot is current.
4. Back on the live screen, press and hold a template shot's photo until it grows, drag it onto another shot, let go:
   the two swap at once. Press and hold an extra photo and drag it along the extras: the others slide aside; let go.
5. Start a drag and put a second finger down, or slide the finger far above the strip and let go: nothing changes.
6. Drag a photo to the strip's right end and hold there: the strip scrolls.
7. On the Mac, check each change in the session view.

## Review log

### 2026-11-27 — implement r1 (fable)
**Built, by unit (checkpoints in order: U1 `09ba8340`, U2 `9cbb7f7c`, U3 `0348cc8a`, U4 `c7824ab9`).**
- **U1 (Rust).** `template_placement::classify_to(h, photo_id, ClassifyTo::{Main, Condition})` and
  `MoveTarget::{ExtraInsert, ConditionInsert}`. `move_to`'s transaction became the private `move_photo(h, photo_id,
  choose)`: it reads the phone's model once, asks `choose` where the photo goes given where it is (a container, an
  extra, Condition; `None` = nothing changes) and writes; `move_to` passes its target, `classify_to` passes PH4a's
  table. No second copy of the model. `CurrentShots::set_after_take_out` (log `by=classify`) is called after the
  commit and before `HandAction::finish`. Wire: `ClassifyBody`, `ClassifyToDto` (serde, lowercase),
  `MoveTargetDto::{ExtraInsert, ConditionInsert}`; route `classify_photo` (parse, guard, `session_photo`, call).
- **U2 (phone).** `arrange.ts` (`swipeAction`, `nextPhoto`); `Preview.tsx` runs PH9.1's touch (`pressing → swiping |
  holding`), the hint and a short fly-out; `ConfirmSheet.tsx`; `PhoneApp` sends the swipe's request and sets the next
  photo before it; the Preview carries `Dock` as `preview-strip`; the Delete button is gone.
- **U3 (phone).** `arrange.ts` (`dropTarget`, `applyDrop`); `Dock.tsx` is PH9.2's state machine on pointer events with
  pointer capture; homes measured once in `liftAt`; the auto-scroll is a 16 ms interval that exists only while the
  finger is in an end zone; `PhoneApp.drop` maps a `Drop` to move or classify.
- **U4 (phone).** `arrangementKey`; every cancel cause calls `Dock`'s `finish(false)`; the pending drop is `PhoneApp`
  state with three ends (`endPending` is called from the read that started after the answer, from the refusal, and
  from the 8 s timer; also on unmount).

**Assumptions (judgment calls; each is one sentence for the narrative).**
1. *Swipe on a photo that has no template* (an extra, a Condition image) may start at any moment of the touch: the
   200 ms window exists only to tell a swipe from the hold, and those photos have no hold. A template shot's photo
   follows PH9.1 exactly (swipe within 200 ms, else hold).
2. "A move of more than 14 px" is the straight-line distance from the touch.
3. The hint shows from 24 px and grows to full strength at 96 px (then it swells slightly: "let go now"); DENY is red,
   MAIN / CONDITION dark, ALREADY … dashed, and for those the photo follows the finger at 35 % only.
4. The toast and the re-read come when the Mac has answered (as Retake's always did); the haptic and the next photo
   come at the swipe. A refused swipe therefore shows only the refusal.
5. A second finger on the picture cancels a swipe (PH9.1 lists pointercancel, hidden and blur; added for the same
   reason as PH9.4's).
6. The swipe moves the picture element itself; while a photo has no preview yet (grey "Preparing preview…") the hint
   shows but the grey box does not follow the finger (`PhotoImage.tsx` is not in this milestone's files).
7. The Preview's strip sits directly under the picture and scrolls just far enough to keep the marked photo in sight;
   a tap on the photo already shown does nothing; a tap on an empty shot that is already current closes the Preview
   without a request. An extra's or Condition image's Back button takes the whole row (no Delete beside it).
8. The confirmation is a bottom sheet named by its question (`role="dialog"`); it closes by itself if the Mac's view no
   longer has that photo in a template shot.
9. The focus marker is cleared whenever a swipe or drop takes a photo out of its template shot (the current shot
   changes, as with Retake).
10. *Lifted look:* the tile follows the finger sideways without limit and up/down by at most 48 px (the drop band),
    dims beyond it, is held where it was touched; its home shows a gap; a swap target gets a solid ring, the tiles of
    a whole-group target (and of the extras when a template shot's photo would ask first) a dashed ring. While a photo
    is lifted or springing home the scroller reaches 48 px above and below the strip so the tile is not cut; idle it is
    exactly the strip (LV6·C1(a) unchanged).
11. The auto-scroll runs wherever the finger is vertically (PH9.2 names no band for it).
12. `[30,40,60]` is given at the drop and at the confirmation's "Move", never at the release that opens the
    confirmation.
13. `applyDrop` leaves `counts` and `current` as the Mac gave them (only the strips read the pending arrangement).
14. The 8 s timer starts at the drop and is the only bound: it also ends a wait whose move succeeded but whose
    re-read never came back.
15. *Second finger:* counted per strip. It cancels and starts nothing itself; a new first finger (a primary pointer)
    forgets any finger whose release was never seen, so a lost release cannot leave a strip dead.
16. The arrangement key is stored at lift (PH9.2), so a change of the photos ends a lifted or dragged photo; a touch
    still only pressing has no stored key and stays a tap or lift on what is then on screen.
17. A strip that stops taking touches under a finger (a drawer or the confirmation opening) cancels as well.
18. U1: `set_after_take_out` runs only when the photo's session is the active one (the route guards that already).
    An unknown `to` is a body that does not parse: 400 `invalid_value`.

**Deviations.**
1. **One more LV3 test changed than the plan lists:** `lv3_c23b_the_preview_stays_open_on_the_image_in_its_new_place`
   (`lv3_drawers.test.tsx`) ended with "a Delete button is shown" for an extra; LV-D20 (4) / LV8·C8(e) remove that
   button, so the line now asserts an extra has no Retake. The plan's list (`lv3_c22c` removed, `lv3_c21a/b`,
   `lv6_c5a`, `lv6_c2c` removed) is otherwise exactly what changed.
2. **`cargo test --workspace` ran detached, once**, with its output in a file, against the wrapper's "never in the
   background": this harness ends a foreground command after 10 minutes and I expected the first run after a rebuild
   to be longer (flow §4A: "about 9 minutes warm, more right after a rebuild"). It took 5 min 4 s, so it would have
   fitted. No other cargo command ran while it did; I waited for it.
3. `docs/guide/02_architecture.md` also describes the phone client (`arrange.ts`, the strip's state machine), added
   with U3 (flow §7A); the plan names that file only for U1.
4. `phone.css` changed in U4 too (one stacking rule added in U3 was removed); `__tests__/helpers.ts` gained the two
   fixtures (`sortSession`, `dragSession`) beside the helpers the plan names.
5. Actor `fable` (the plan's `implementer: fable`), not the prompt's `codex`.

**Observations for the reviewer.**
- Red baseline: captured for U4 only (before its code: `lv8_c14b`, `c14c`, `c14e`, `c15d`, `c15e` failed, 7 passed on
  U3's behaviour). U1–U3's tests were written with their code; no red baseline exists for them.
- `lv8_c15a` holds the read that follows the Mac's answer (the fake answers the move and holds what comes next), so
  the strip keeping the drop between the answer and that read is asserted, not only the end state.
- U1's fixture leaves "Move image" steps in the history (the three extras are made by `ExtraEnd`), so a label alone
  cannot show a new step: `lv8_c1a` and `lv8_c3a` undo once and check what came back; the "nothing changes" rows undo
  once and check the fixture's last step came back instead. (`lv8_c1a`'s undo lines were added after the full Rust
  run; only `lv8_phone_sort::` was re-run for them, 14 passed.)
- Events in U1 rows are the `PhotoChanged` / `SessionChanged` sequence; the `HistoryChanged` every recorded step emits
  is left out of the comparison for the rows that change something and counted for the rows that must change nothing.
- Nothing was seen on a phone or in a browser in this session: every row is proven in jsdom on markup, inline styles
  and requests. The lifted look, the 48 px reach of the scroller, the hint and the sheet are CSS by reasoning; iOS
  long-press behaviour on the strip (callout, selection) relies on the rules LV6 already had
  (`touch-action: none`, `-webkit-touch-callout: none`, `user-select: none`, images without pointer events). The
  manual try-out is the first real check.
- Candidate note: a photo with no preview yet does not follow the finger in a swipe (assumption 6); making it follow
  needs `PhotoImage` to pass its style to the placeholder.

**Closing gate (once, in the plan's order; tree = `c7824ab9` plus the two test-only edits of the closing commit).**
`npm run build` exit 0 · `cargo test --workspace` exit 0, 2774 passed, 0 failed (LV6's stamp 2760 + this phase's 14) ·
`cargo clippy --workspace --all-targets -- -D warnings` exit 0 · `cargo fmt --all -- --check` exit 0 ·
`tools/sidecar/build.sh --stub --test` exit 0 (2 of 2) · `npm run typecheck` exit 0 · `npm test` exit 1: 1135 passed,
**1 failed: `src/features/workspace/__tests__/p33_split_grid.test.tsx` C4(d)**, the known intermittent suite flake of
`backlog.md` (P35 → P33: its 1 s `waitFor` runs out under full-suite load; here at 1049 ms). No file under
`src/features` changed in this milestone; that file re-run alone: 7 of 7 passed. Phone tests: 14 files, 181 passed.
