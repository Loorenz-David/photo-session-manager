---
phase: LV2_current_shot_and_phone_server
title: The current shot and the phone server — every shutter (phone, Mac or the camera's own button) goes to the shot that was current when the camera fired, even when its file arrives late; Retake denies the old photo at once and reopens the shot; the Mac's "Next" is the current shot; Settings → Phone pairs a phone with a QR code; the app's own server answers paired phones with the session, capture, focus, shot changes, moves and live frames
difficulty: high
projection: no
depends_on: [LV1_camera_live_view]
blocked_on: none (owner card 1 answered 2026-11-26: `article_number` = the item's barcode)
---
# LV2 — Current shot and the phone server

**Tracker:** IMPLEMENTED (2026-11-26, opus) — 5 units, 101 rows: 79 behaviour tests (lv2_current_shot 31, lv2_phone_ops 13, lv2_phone_pairing 7, lv2_phone_http 27, lv2_phone_ws 10 incl. chained rows), 2 desktop command tests, 5 UI tests, core-mechanism unit tests; closing gate green (workspace suite 1598 after the p01 schema tripwire was updated). See the implement r1 handoff.

(This line is the milestone's tracker row. Sessions change only this line of this file. The gate for `depends_on` is
the Tracker line of the plan named there.)

## Goal
Book `10J` MC-LV3, LV-D7, LV-D12, LV-D18, card LV3, LV8; `10G` §4–§6 as 10J amends them; `11` §Phone remote; by
`master_plan_live.md` §5 **CS** and **PH**. (U1) The current shot in placement. (U2) The phone's operations and its
session view. (U3) Pairing, the `phone` setting, Settings → Phone. (U4) The HTTP server, its routes and the placeholder
phone page. (U5) The WebSocket: frames by demand, events, viewer lifecycle.
**Not here:** the Shoot Companion UI (LV3: U4 ships a placeholder page), HTTPS, session completion/opening from the
phone (LV-D18 later), the Mac's live screen (card LV10).

## How to work this plan (flow.md §10, master_plan_live.md §3)
One runner phase of 5 units, in order; after each, its **Unit test commands**, then a `CHECKPOINT (not approved):`
commit of that unit's paths. Row ids `LV2·C<n>(<row>)`; tests named after their row. Open details: the smallest
option consistent with 10J, 10G, 10E MC-33 and §5, recorded under **Assumptions**. Reviewer notes:
`docs/implementation/backlog_live/LV2_current_shot_and_phone_server.md`.

| Unit | What it builds | Criteria | Rows | Changes app files |
|---|---|---|---|---|
| U1 | The current shot: binding, advance, tap, retake, reset, placement by binding | 8 | 31 | yes (core, infra) |
| U2 | Phone operations: session view, Move (swap), Delete, change events | 4 | 14 | yes (core) |
| U3 | Pairing, the `phone` setting, Tauri commands, Settings → Phone | 4 | 14 | yes |
| U4 | The HTTP server: auth, static files, manifest, routes, placeholder page | 7 | 27 | yes |
| U5 | The WebSocket: demand, frames, events, lifecycle | 6 | 15 | yes |
| **Total** | | **29** | **101** | |

## Read first (whole milestone)
`docs/process/flow.md` (§4, §4A, §6, §7A, §10) · `master_plan_live.md` (all; **§5 CS, PH**, §4, §6, §8) · book `10J`
(MC-LV3, LV-D7, LV-D12, LV-D17, LV-D18, cards LV3, LV5, LV6, LV7, LV8) · `10G` §2, §4, §5, §6 (with the 10J note at the
top of §4) · `10E` MC-33 (association order, FIFO window, R-I25, R-I28) · `11` §Phone remote ·
`docs/guide/02_architecture.md` §Templates and §Camera, `docs/guide/04_change_impact.md` · the LV1 plan (what exists).

## Points recorded by planning (each is one sentence in the narrative's "Choices made for you")
- **P1 After a Condition shot the current shot goes back to the first empty template shot**, if any (10J's rule;
  the phone prototype stayed on Free shot).
- **P2 A photo that arrives with no shutter report** (Capture One tether, or a file more than 5 minutes after its
  shutter) **takes the shot that is current when it arrives.**
- **P3 "Shift the rest back" stays on the Mac**; only the automatic refill of an emptied container is gone (Retake and
  tapping a shot replace it).
- **P4 The phone server is on by default on port 8686 but answers only phones you paired**; change or switch it off
  in Settings → Phone.
- **P5 Every phone action names its session**; if you switched session on the Mac in between, the phone says
  "Session changed on the Mac" and does nothing.

## Unit 1 — The current shot: binding, advance, tap, retake, reset, placement by binding

**Unit id:** `LV2·U1` · **Rows:** LV2·C1–C8 · **Changes app files:** yes (core, infra)

### Goal
CS1–CS8 exactly; the schema of §4.4 (the two binding column pairs); `AppEvent::ActiveSessionChanged`; the Mac's
`next_shot` = the current shot; the existing template tests that encode 10G §4 rule 2 / §6 refill rewritten to MC-LV3.

### Read first
Master plan §5 CS, §4.2, §4.4 · `crates/core/src/services/template_placement.rs` (all) · `crates/core/src/mechanisms/
{shot_order.rs, template.rs, capture_ledger.rs}` · `crates/core/src/services/camera_ledger.rs` (`exposure_observed`,
`decide_and_place`) · `crates/core/src/services/arrival.rs` (where `place_arrival` is called) ·
`crates/core/src/services/review.rs` (`delete`) · `crates/core/src/services/history.rs` (template steps) ·
`crates/core/src/services/sessions.rs` (`store_target`) · `crates/core/src/domain/{template.rs, camera.rs}` ·
`crates/infra/src/sqlite/{store.rs (shot_keys, hold_times, photo_place_*, insert_exposure, match_late_exposure),
ensure.rs}` · `crates/integrations/src/lib.rs` (where `ArrivalService` and `LedgerService` are built) ·
`apps/desktop/src-tauri/src/lib.rs` (`TauriEvents`) · `crates/testkit/tests/suite/{t2_template_placement.rs,
t3_template_hand.rs, t4_template_retake_shift.rs, p44_capture_ledger.rs}`.

### Files
New: `crates/core/src/mechanisms/current_shot.rs`, `crates/testkit/tests/suite/lv2_current_shot.rs`. Edited:
`crates/core/src/services/{current_shot.rs, template_placement.rs, camera_ledger.rs, arrival.rs, sessions.rs,
history.rs (only if the retake step needs a new kind), mod.rs}`, `crates/core/src/mechanisms/{shot_order.rs, mod.rs}`,
`crates/core/src/domain/{template.rs, camera.rs}`, `crates/core/src/{events.rs, error.rs, ports/store.rs}`,
`crates/infra/src/sqlite/{ensure.rs, store.rs}`, `crates/integrations/src/lib.rs`, `crates/integrations/src/camera/
runtime.rs` (ledger gets `current_shots`), `apps/desktop/src-tauri/src/lib.rs` (event name), `apps/desktop/src-tauri/
src/commands/items.rs` (codes), `crates/testkit/tests/suite/{t2_template_placement.rs, t3_template_hand.rs,
t4_template_retake_shift.rs, main.rs}` (rewritten rows listed in the handoff).

### Core mechanisms
`mechanisms::current_shot::next_untaken` (CS4), `mechanisms::shot_order::slot_owners` (CS2).

### Ordered tasks
1. Schema (`ensure_live_columns`) and domain (`ShotBinding` on `NewExposure`, `Exposure`, `PlaceRow`); Store reads and
   writes for them.
2. `slot_owners` (CS2) beside `order` (whose existing results stay identical); `next_untaken` (CS4).
3. `CurrentShots` (CS1, CS4, CS7 `set`), held by the camera runtime (LV1 made the holder); `ActiveSessionChanged`
   emitted by `SessionsService` when the active session changes and observed by `CurrentShots` (clear).
4. Binding at exposure (CS5, in `LedgerService::exposure_observed`, ledger worker lock) and at arrival (CS6, in
   `place_arrival`); `current_shot.bound` / `current_shot.set` log lines; `SessionChanged` on every change.
5. Placement by binding (CS3) in `place_template_photos`, `differs`, `can_shift_back`, `shift_back`, `back_to_template`,
   `drop_on_container` (CS8); remove `gaps`, `gap_for_arrival`, `take_shot_key`, index-based `next_shot` and
   `expected_place`; `TemplateView.next_shot` per CS8.
6. `retake` (CS7).
7. Rewrite the t2/t3/t4 tests that assert the replaced rules; tests for the rows.

### Criteria (8 criteria, 31 rows)
Boundary: the wired TestApp (outcome = photos' classification, Main slot and composition, `camera_exposures` binding
columns, `photo_places` rows, `CurrentShots::current`, `TemplateView.next_shot`, undo results, emitted events, log
lines). Fixture **T3**: item type "Chair" with a template of three containers — **A** (final 1, shoot 2, presets 1:1
zoom 20), **B** (final 2, shoot 1, split), **C** (final 3, shoot 3, no presets) — so the shooting order is B, A, C.
Exposures enter through the app's `CaptureSink::exposure_observed`, files through `CaptureSink::delivery` and the
arrival scan, as `t2_template_placement.rs` does (camera counters `DSC00100`…).
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| LV2·C1(a) | `current`, `session_view` | a new active T3 session, nothing shot | `Shot(B)`; `next_shot` = 1 (B's shoot_no) | CS1, CS4, MC-LV3 |
| LV2·C1(b) | same | an exposure bound to B reported, no file (within the window); then a relaunch | after relaunch `current` = `Shot(A)` (B is held by the reported shutter) | CS4, MC-LV3 |
| LV2·C1(c) | same | photos in A, B and C | `Free`; `next_shot` = none | CS4 |
| LV2·C1(d) | exposure + arrival | an active session of a type with no template; one exposure and its file | exposure row `shot_bound = 0`; the photo is Main at the end of the list (P66); `current` none | CS1, MC-LV3 (untemplated) |
| LV2·C1(e) | `session_view` of a non-active T3 session | the active session's current is `Shot(C)` by a tap | the other session's `next_shot` = 1 (its default); the active one's current still `Shot(C)` | CS8 |
| LV2·C2(a) | `exposure_observed` | new T3 session; one exposure | the row has `shot_bound = 1`, `shot_container_id` = B; `current` = `Shot(A)`; one `SessionChanged` for the session; log `current_shot.bound … container=<B>` | CS5, MC-LV3, M-LV4 |
| LV2·C2(b) | same | `set(Shot(C))` (B and A empty); one exposure | bound to C; `current` = `Shot(B)` (wraps to the first not taken) | CS4, CS5 |
| LV2·C2(c) | same | photos in B and A; current `Shot(C)`; one exposure | bound to C; `current` = `Free` | CS4, CS5 |
| LV2·C2(d) | same | photos in B and C, A empty; `set(Free)`; one exposure, then its file | row `shot_container_id` NULL, `shot_bound = 1`; the photo is Condition, last; `current` = `Shot(A)` | CS4, CS5, P1, 10G §4 rule 4 |
| LV2·C3(a) | late files | photo DSC00100 in B (its exposure bound B); exposures e1 (bound A) and e2 (bound C) reported 2 s apart; file DSC00102 delivered first (the ledger's FIFO matches it to e1), then DSC00101 | right after DSC00102 arrives it is in C (its counter follows 100 by two); at the end DSC00101 is in A with A's presets and DSC00102 in C | CS2, CS3, CS5, M-LV4, 10G §4 rule 1 |
| LV2·C3(b) | late file after a tap | exposure e1 bound B; then `set(Shot(C))`; e2 bound C; e2's file arrives, then e1's | e1's photo in B, e2's in C; no other photo moved when e1's arrived | CS3, MC-LV3, M-LV4 |
| LV2·C3(c) | binding written once | e1 bound B; its file arrives; then a later arrival runs placement again | e1's exposure row unchanged; its photo still in B; `current` advanced only once | CS5, CS6 |
| LV2·C4(a) | `CurrentShots::set` | B, A, C empty; `set(Shot(C))`; one exposure | bound to C | CS7, M-LV4 |
| LV2·C4(b) | same | photo in B; `set(Shot(B))` | `ShotFilled`; `current` unchanged | CS7 |
| LV2·C4(c) | same | `set` for a T3 session that is not the active one | `SessionChanged`; nothing held changes | CS7 |
| LV2·C4(d) | same | a container id of another type's template | `NotInTemplate` | CS7 |
| LV2·C4(e) | same | an untemplated active session; `set(Free)` | `NoTemplate` | CS7 |
| LV2·C5(a) | `retake` | photo P in A (template-placed); `retake(A)` | P is Deleted with `photo_denied` set and `photo_places.released = 1`; `current` = `Shot(A)`; one undo step "Retake shot 1"; `SessionChanged` | CS7, card LV6, M-LV4 |
| LV2·C5(b) | same, then capture | after (a), one exposure and its file | bound to A; the new photo is in A with A's presets (1:1, zoom 20); `current` = the next not-taken container after A in shooting order (C if empty, else B if empty, else `Free`) | CS5, CS7, M-LV4 |
| LV2·C5(c) | undo | after (a), undo the step | P is Main in A again (released 0, `photo_denied` cleared as the Mac's undo of a deny does); `current` read → advanced past A | CS7, 10G §6 rule 5 |
| LV2·C5(d) | `retake` | `retake(C)` with C empty | `NotInTemplate`; nothing changes | CS7 |
| LV2·C6(a) | session switch | active A-session with `set(Shot(C))`; switch to another session and back (through `SessionsService`) | `current` for the first session = its default (`Shot(B)`), not C; `ActiveSessionChanged` emitted with each new id | CS1, CS7, MC-LV3 |
| LV2·C6(b) | relaunch | `set(Shot(C))`; restart the TestApp | `current` = default `Shot(B)` | CS1, MC-LV3 |
| LV2·C7(a) | arrival with no exposure | a T3 session; a photo arrives through the arrival scan with no delivery (Capture One tether path) | its place row `shot_bound = 1`, `shot_container_id` B; the photo in B; `current` = `Shot(A)` | CS6, P2 |
| LV2·C7(b) | file before its exposure | a delivery completes first (no exposure yet: bound at arrival to B, current → A), then its exposure event arrives and `match_late_exposure` matches it | the exposure row gets `shot_container_id` B; `current` still `Shot(A)` (no second advance) | CS5, CS6 |
| LV2·C8(a) | `drop_on_container` (Mac) | photo X (no container, e.g. Condition) dropped on A holding P | X Main in A, `place_by_hand = 0`, place-row binding A, A's presets; P Deleted and released; one undo step "Retake container 1" | CS8, 10G §6 rule 3, card LV8 |
| LV2·C8(b) | `shift_back` | photos bound B, A, C in place; deny A's photo (Mac Deny); Shift the rest back | C's photo moves to A (rank 2) with A's presets; C empty; the denied photo released; one undo step restores all | CS8, 10G §6 rule 4 |
| LV2·C8(c) | `back_to_template` | A's photo dragged to Condition by hand | Back to template returns it to A | CS8, 10G §6 rule 2 |
| LV2·C8(d) | container removed | A's photo bound A; container A removed from the template; next arrival runs placement | that photo is Condition (last); its binding columns unchanged | CS3 |
| LV2·C8(e) | refill gone | photos in B and A, `current` = `Shot(C)`; deny A's photo (Mac); one exposure | bound to C (not A); `current` then = `Shot(A)` | CS8 (10G §6 refill replaced), MC-LV3 |
| LV2·C8(f) | unbound photos | a T3 session with photos placed before this change (rows `shot_bound = 0`, no bound exposure) | a new arrival leaves them where they are; they count as taken | CS3 |

### Delegated choices
Store method names · where `CurrentShots` observes `ActiveSessionChanged` (an `EventHub` observer or a direct call
from the sessions path) as long as C6 holds · the undo step kind for `retake` (reuse the template step of the drop
retake if it restores the denial) · how existing t2/t4 tests are rewritten (each rewritten test keeps its scenario and
asserts the MC-LV3 outcome).

### Notes
- Hazard: `exposure_observed` runs under the ledger worker lock; `current()` reads the store (taken set) — do it in the
  same transaction that inserts the exposure, so a concurrent arrival cannot slip between.
- Hazard: `shot_order::order` results must not change (t2 tests of rule 1 keep passing); `slot_owners` is additive.
- Hazard (10E MC-33): a file beyond the 300 s FIFO window matches no exposure and is bound at arrival (P2).
- Hazard (H2): a retake denies; the RAW is deleted only when the undo window closes, as for any Deny.

### Unit test commands
`cargo test -p psm-core mechanisms::current_shot mechanisms::shot_order` (if cargo refuses two filters: run them one
after the other) · `cargo test -p psm-testkit --test suite -- lv2_current_shot:: t1_templates:: t2_template_placement:: t3_template_hand:: t4_template_retake_shift:: p44_capture_ledger:: p05_capture_arrival:: p66_approve_on_arrival_and_session_order::` ·
`cargo clippy -p psm-core -p psm-infra -p psm-integrations --all-targets -- -D warnings`

## Unit 2 — Phone operations: session view, Move (swap), Delete, change events

**Unit id:** `LV2·U2` · **Rows:** LV2·C9–C12 · **Changes app files:** yes (core)

### Goal
PH6 (`phone_view`, `shot_ratio`), `template_placement::move_to` (card LV8: the phone swaps; design README §Move
semantics), phone Delete = Deny, `SessionChanged` on current-shot changes.

### Read first
Master plan §5 PH6, §6 (Delete, Move) · design `claude_design/live_preview/design_handoff_shoot_companion/README.md`
§Move semantics, §5 Move image drawer (local, git-ignored; cite, never copy) · `crates/core/src/services/
{template_placement.rs, review.rs, previews.rs}` · `crates/core/src/domain/{photo.rs, composition.rs, item.rs}` ·
`crates/core/src/mechanisms/template.rs`.

### Files
New: `crates/core/src/services/phone_view.rs`, `crates/testkit/tests/suite/lv2_phone_ops.rs`. Edited:
`crates/core/src/services/{template_placement.rs, current_shot.rs, mod.rs}`, `crates/core/src/mechanisms/template.rs`,
`crates/testkit/tests/suite/main.rs`.

### Core mechanisms
`mechanisms::template::shot_ratio`.

### Ordered tasks
1. `shot_ratio` (PH6 table) and `phone_view` (PH6 shape; `article_number` = the item's `barcode` (`crates/core/src/domain/item.rs`, read from the Shopify variant barcode), owner card 1 answered 2026-11-26).
2. `move_to` with the five targets: to a Main position (container or extra position) → swap with whatever is there
   (nothing → move); `ExtraEnd` → Main after the containers' range; `Condition(j)` → swap with condition j (a Main
   source gives its Main place to that condition photo); `ConditionEnd` → append. A template photo touched is placed by
   hand (10G §6 rule 1). One transaction, one undo step ("Move image"). Untemplated sessions: Main positions only.
3. Delete for the phone = `ReviewService::delete`.
4. Tests for the rows.

### Criteria (4 criteria, 14 rows)
Boundary: `phone_view`, `move_to`, `ReviewService::delete` through the wired TestApp with fixture T3 (U1) plus an
untemplated session (outcome = the returned view, photos' classification/position/slot, `place_by_hand`, undo,
events).
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| LV2·C9(a) | `phone_view` | active T3 session; item title "Oak Lounge Chair", SKU "BEY-123"; photo in B; one Condition photo; `current` = `Shot(A)` | `shots` in order B, A, C with `final_no` 2, 1, 3, `ratio` 2.0, 1.0, 1.0 (C has no shape: the default `1:1`), `split` true/false/false, `zoom` 0/20/0, `photo` only on B; `extras` []; `condition` one; `counts` {main 1, condition 1}; `current` {shot, A}; item name and SKU as given | PH6, LV-D15, LV-D17 |
| LV2·C9(b) | same | an item with barcode "7350001234567"; an item with no barcode | `article_number` = "7350001234567"; `article_number` = null (owner card 1, 2026-11-26: the item's barcode) | PH6, LV-D17 |
| LV2·C9(c) | same | active untemplated session with Main P1, P2 and Condition Q | `templated` false; `shots` []; `extras` [P1, P2]; `condition` [Q]; `current` null | PH6, 10J §phone design (untemplated) |
| LV2·C9(d) | same | no active session | `session` null | PH6, LV-D18 |
| LV2·C10(a) | `move_to(P, Container(C))` | P in A, Q in C | P in C, Q in A, both `place_by_hand = 1`; one undo step reverses both | card LV8, 10G §6 rule 1 |
| LV2·C10(b) | same | P in A, C empty | P in C; A empty | card LV8 |
| LV2·C10(c) | `move_to(P, ExtraEnd)` | P in A | P Main without slot, after every container in Main order; A empty | card LV8, 10J (extra Main only by Move) |
| LV2·C10(d) | `move_to(R, Condition(1))` | condition [S, R] | condition [R, S] | design §Move semantics |
| LV2·C10(e) | `move_to(P, Condition(1))` | P in A; condition [S] | S Main in A; P Condition 1 | design §Move semantics, card LV8 |
| LV2·C10(f) | `move_to(P, ConditionEnd)` | P in A; condition [S] | condition [S, P]; A empty | design §Move semantics |
| LV2·C10(g) | `move_to(P1, MainPosition(3))` | untemplated Main [P1, P2, P3] | Main [P3, P2, P1] | card LV8 |
| LV2·C11(a) | `ReviewService::delete` from the phone path | an extra Main photo; a Condition photo | each Deleted with `photo_denied` set; undo restores each to its place | §6 (phone Delete = Deny), 10J §phone design |
| LV2·C12(a) | events | `set(Shot(C))`; then one exposure (advance); then `retake(A)` | three `SessionChanged{session}` events, one per change, in that order | CS5, CS7, PH5 |
| LV2·C12(b) | `mechanisms::template::shot_ratio` (core mechanism) | split; `1:1`; `4:5`; `3:2`; `2:3`; `16:9`; `original`; `free`; no shape with default `1:1` | 2.0; 1.0; 0.8; 1.5; 0.6667; 1.7778 (±1e-4); null; null; 1.0 | PH6 |

### Delegated choices
Undo labels · whether `move_to` reuses `drop_on_container`'s swap branch.

### Notes
- Hazard (card LV8): the Mac's drop keeps replace-by-drop; `move_to` is phone-only and never replaces.

### Unit test commands
`cargo test -p psm-testkit --test suite -- lv2_phone_ops:: lv2_current_shot:: t3_template_hand::` · `cargo test -p psm-core mechanisms::template` ·
`cargo clippy -p psm-core --all-targets -- -D warnings`

## Unit 3 — Pairing, the `phone` setting, Tauri commands, Settings → Phone

**Unit id:** `LV2·U3` · **Rows:** LV2·C13–C16 · **Changes app files:** yes

### Goal
PH1 (pairing), the `phone` setting (§4.5, §6 port rule), the five Tauri commands, Settings → Phone with the QR codes.

### Read first
Master plan §5 PH1, §4.4–4.5, §6 (port, QR) · `crates/infra/src/settings_file.rs` · `crates/core/src/services/
settings.rs` · `crates/integrations/src/lib.rs` (settings prepare and the rebuild comparison) · `apps/desktop/src-tauri/
src/commands/{settings.rs, writer.rs}` (command pattern) · `apps/desktop/src-tauri/src/lib.rs` (`specta_builder`,
command tests) · `apps/desktop/src/features/settings/{SettingsNav.tsx, SettingsPage.tsx, CameraSection.tsx}` and their
`__tests__` · `crates/infra/src/net.rs`.

### Files
New: `crates/integrations/src/phone/{mod.rs, pairing.rs, host.rs}`, `apps/desktop/src-tauri/src/commands/phone.rs`,
`apps/desktop/src/services/phone.ts`, `apps/desktop/src/features/settings/PhoneSection.tsx`,
`apps/desktop/src/features/settings/__tests__/lv2_phone_settings.test.tsx`, `crates/testkit/tests/suite/
lv2_phone_pairing.rs`. Edited: `crates/infra/src/{settings_file.rs, sqlite/ensure.rs, sqlite/store.rs}`,
`crates/core/src/{ports/store.rs, services/settings.rs, domain/live.rs}`, `crates/integrations/{Cargo.toml, src/lib.rs}`,
`apps/desktop/src-tauri/src/{commands/mod.rs, commands/settings.rs, lib.rs}`, `apps/desktop/src/services/bindings.ts`
(generated), `apps/desktop/src/features/settings/{SettingsNav.tsx, SettingsPage.tsx}`,
`crates/testkit/tests/suite/main.rs`.

### Core mechanisms
None.

### Ordered tasks
1. `phone_pairings` (`ensure`), Store methods, `PhonePairings` (`create`, `verify`, `revoke`, `rename`, `list`).
2. `Settings.phone` (default `{enabled: true, port: 8686}`), allowlisted, validated (§6 port rule), nulled in the
   rebuild comparison.
3. `host.rs`: Mac name (`scutil --get LocalHostName` + `.local`), first private IPv4, the two URLs, QR SVGs (`qrcode`).
4. Commands of §4.5 with DTOs; regenerate bindings; `services/phone.ts`.
5. `PhoneSection`: "Pair a phone" → name field (optional) → the QR of the `.local` URL, the URL as text, "Show the
   address instead" (the address QR and text), and one line: "Open the camera on the phone and scan this code. On
   iPhone, then Share → Add to Home Screen; on Android, add a shortcut and tap Full screen in the app." The list of
   pairings (name, last seen, connected dot) with rename and **Remove** (confirm). The server line: "Phone server on
   port 8686" or the error. Port field and on/off switch.
6. Tests for the rows.

### Criteria (4 criteria, 14 rows)
Boundary: `PhonePairings` and settings through the wired TestApp (outcome = returned values, `phone_pairings` rows,
the settings file, whether the App was rebuilt); the desktop command tests; the UI component with `services/phone.ts`
mocked.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| LV2·C13(a) | `PhonePairings::create` | no pairings | a 43-character token of `[A-Za-z0-9_-]`; the row's `token_sha256` = lowercase hex SHA-256 of it; name "Phone 1"; the token string is in no column of the database | PH1, LV-D12 |
| LV2·C13(b) | `verify` | that token; then "x" | the pairing (id, name) with `last_seen_at` set; then none | PH1 |
| LV2·C13(c) | `revoke` | two pairings; revoke the first | the first token verifies none; the second still verifies | PH1, LV-D12 |
| LV2·C13(d) | `create`, `rename`, `list` | create twice; rename the second "Android" | names "Phone 1", "Android"; `list` items carry no token | PH1 |
| LV2·C14(a) | `settings_set` | `{"phone": {"port": 8790}}` | saved; the App is not rebuilt (the prepared settings are the simple kind, as for `theme`) | §4.5, 10E R-I18 |
| LV2·C14(b) | same | `{"phone": {"port": 2121}}`; `{"phone": {"port": 80}}`; `{"phone": {"port": 50005}}` | each refused `phone_port_invalid`; the file unchanged | §6 port rule |
| LV2·C14(c) | defaults | a fresh settings file | `phone` = `{enabled: true, port: 8686}` | charter rule 10 (defaults reach it), P4 |
| LV2·C15(a) | desktop command `phone_pairing_create` | port 8686; host name "Studio-Mac"; address 192.168.1.20 (injected) | `url_name` = `http://Studio-Mac.local:8686/phone/?pair=<token>`, `url_address` = `http://192.168.1.20:8686/phone/?pair=<token>`; both SVGs start with `<svg` | PH1, §6 QR, 11 §Phone remote |
| LV2·C15(b) | `phone_pairings_list`; bindings | after (a) | no field holds the token; `bindings.ts` contains the five command names | PH1, §4.5 |
| LV2·C16(a) | `PhoneSection` | click **Pair a phone**, then **Create** | `phonePairingCreate` called once; the QR SVG and the `.local` URL show; **Show the address instead** shows the address QR and URL | `[rule]` §4.5, design of Settings |
| LV2·C16(b) | same | a list of two pairings; **Remove** on the second, confirm | `phonePairingRevoke(id2)` once; after the re-read only the first shows | `[rule]` PH1 |
| LV2·C16(c) | same | `phoneServerStatus` answers `{running: false, error: "port 8686 in use"}` | "Phone server could not start: port 8686 in use" shows | `[rule]` PH2 |
| LV2·C16(d) | same | rename the first to "iPhone" | `phonePairingRename(id1, "iPhone")` once | `[rule]` PH1 |
| LV2·C16(e) | same | `connected: true` on a pairing | its connected dot shows | `[rule]` PH5 |

### Delegated choices
How `host.rs` is injectable for C15(a) (a trait or a function parameter) · the Settings page layout (existing
sections' look) · the confirm dialog component.

### Notes
- The token is shown once (in the QR and URL of the create result); a lost QR means a new pairing.

### Unit test commands
`cargo test -p psm-testkit --test suite -- lv2_phone_pairing:: p68_setup_polish_address_password_shop::` · `cargo test -p psm-desktop` ·
`cd apps/desktop && npx vitest run src/features/settings` · `npm run typecheck` ·
`cargo clippy -p psm-integrations -p psm-infra -p psm-desktop --all-targets -- -D warnings`

## Unit 4 — The HTTP server: auth, static files, manifest, routes, placeholder page

**Unit id:** `LV2·U4` · **Rows:** LV2·C17–C23 · **Changes app files:** yes

### Goal
PH2, PH3, PH4; the web root (§4.5); the phone entry build (two Vite inputs) with a **placeholder** phone page; the
desktop starts the server once and rebinds it on a port change.

### Read first
Master plan §5 PH2–PH4, PH6, §4.3, §4.5, §6, §8 · `crates/infra/src/nudge.rs` (server on its own thread) ·
`crates/infra/src/ftp/peers.rs` · `crates/integrations/src/lib.rs` (`AppHandles`) · `apps/desktop/src-tauri/src/
{lib.rs, state.rs}` · `apps/desktop/src-tauri/tauri.conf.json` · `apps/desktop/{vite.config.ts, index.html,
package.json}` · `crates/core/src/services/{camera.rs, previews.rs}` (capture; which preview file the session view
shows) · `apps/desktop/src-tauri/src/commands/templates.rs` (`template_session_view`: where example files live).

### Files
New: `crates/integrations/src/phone/{server.rs, auth.rs, routes.rs, wire.rs, web_root.rs}`, `crates/testkit/src/phone.rs`
(`TestPhone`), `crates/testkit/tests/suite/lv2_phone_http.rs`, `apps/desktop/phone.html`,
`apps/desktop/src/phone/{main.tsx, PhoneApp.tsx, api.ts}` (placeholder: reads the token per PH7, calls `hello` and
`session`, shows "Connected to <item name>" or the pairing text), `apps/desktop/public/phone/icon.png` (copy of
`src-tauri/icons/icon.png` if it is 512 px, else resized by hand once). Edited: `crates/infra/Cargo.toml` (axum `ws`),
`crates/integrations/{Cargo.toml, src/phone/mod.rs, src/lib.rs}`, `crates/testkit/{Cargo.toml, src/lib.rs}`,
`apps/desktop/src-tauri/src/lib.rs` (start, rebind, shutdown), `apps/desktop/src-tauri/tauri.conf.json`
(`bundle.resources`), `apps/desktop/vite.config.ts`, `crates/testkit/tests/suite/main.rs`.

### Core mechanisms
`auth::token_of(headers, query)` · `web_root::resolve(packaged_resources, env, debug_default)` · `wire::error_code`.

### Ordered tasks
1. `PhoneServer` (PH2): thread + tokio runtime, bind `0.0.0.0:<port>`, peer filter, `stop`, `rebind`, status; started in
   the desktop setup after the first App with `handles = state.handles()`; rebind on a saved port change; stopped on
   exit.
2. Auth (PH3), static files and manifest (PH4; no directory traversal), the API routes of PH4 with the session guard,
   `phone.request` log lines (never the token).
3. Web root (§4.5); `vite.config.ts` two inputs; `tauri.conf.json` resources; the placeholder page.
4. `TestPhone` and tests for the rows; check `npm run build` puts `dist/phone.html` (record in the handoff).

### Criteria (7 criteria, 27 rows)
Boundary: HTTP requests from the test (`reqwest`) to `TestPhone` (the real `PhoneServer` on `127.0.0.1:0` over the
TestApp's handles, a temp web root holding `phone.html` "<p>phone</p>" and `assets/app.js`), the fake sidecar's stdin,
the store. Fixture T3 active with a photo in B unless stated.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| LV2·C17(a) | `GET /api/phone/v1/hello` | no token | 401, `{error: {code: "pairing_unknown", …}}` | PH3 |
| LV2·C17(b) | same | `Authorization: Bearer <valid>` | 200 `{api: 1, pairing: {id, name: "Phone 1"}, mac_name, features: ["live","capture","focus","current_shot","move"]}` | PH3, PH4, LV-D18 |
| LV2·C17(c) | same | `?pair=<valid>` | 200 | PH3 |
| LV2·C17(d) | same | a revoked token | 401 | PH1, PH3 |
| LV2·C18(a) | `GET /phone/` | no token | 200, `text/html`, body `<p>phone</p>` | PH4 |
| LV2·C18(b) | `GET /assets/app.js` | no token | 200, a JavaScript content type, the file's bytes | PH4 |
| LV2·C18(c) | `GET /phone/manifest.webmanifest?pair=<valid>` | — | JSON equal to PH4's manifest with `start_url` `/phone/?pair=<valid>` and `display` `standalone` | PH4, LV-D7, card LV3 |
| LV2·C18(d) | same | no token | 401 | PH3 |
| LV2·C18(e) | `GET /assets/..%2F..%2Fsecret` | a file `secret` beside the web root | 404 | PH4 |
| LV2·C19(a) | `GET /api/phone/v1/session` | valid token | the JSON of `phone_view` for the active session (session id, shots order B, A, C, `current` {shot, A}) | PH4, PH6, LV-D18 |
| LV2·C19(b) | same | the Mac switches the active session, then GET | the new session's id | PH4, LV-D18 |
| LV2·C19(c) | same | no active session | `{session: null}` | PH6 |
| LV2·C20(a) | `POST /capture` | `{session_id: <active>}`, camera connected (fake) | 200 `{capture_id}`; the fake's stdin has `capture id=<capture_id> af=1` | PH4, M-LV3, 10E MC-31 |
| LV2·C20(b) | same | `{session_id: <another session>}` | 409 `session_changed`; nothing on the fake's stdin | PH4 (session guard), P5 |
| LV2·C20(c) | same | camera not connected | 409 with the desktop's code for `CameraNotConnected`; nothing on stdin | PH4 |
| LV2·C21(a) | `POST /current` | `{session_id, container_id: C}`; then an exposure | 200; the exposure is bound to C | PH4, CS7, M-LV4 |
| LV2·C21(b) | same | `container_id: B` (filled) | 409 `shot_filled` | PH4, CS7 |
| LV2·C21(c) | `POST /retake` | `{session_id, container_id: B}` | 200; B's photo Deleted; `current` = `Shot(B)` | PH4, CS7, card LV6 |
| LV2·C21(d) | `POST /photos/<B photo>/move` | `{session_id, target: {kind: "extra_end"}}` | 200; the photo Main without slot after the containers | PH4, card LV8 |
| LV2·C21(e) | `POST /photos/<id>/delete` | `{session_id}` on a Condition photo | 200; Deleted | PH4, §6 |
| LV2·C21(f) | the four routes above | each with a non-active `session_id` | each 409 `session_changed`; nothing changed | PH4, P5 |
| LV2·C22(a) | `POST /focus` | `{session_id, x: 0.5, y: 0.5}` | 200 `{focus_id}`; the fake's stdin `focus x=320 y=240 af=1 cmd_id=<focus_id>` | PH4, HB6, LV-D14 |
| LV2·C22(b) | `POST /focus/clear` | `{session_id}` | 200; stdin `focus x=320 y=240 af=0 cmd_id=…` | PH4, card LV9 |
| LV2·C22(c) | `POST /focus` | `x: 2` | 400 `invalid_value`; nothing on stdin | PH4, HB6 |
| LV2·C23(a) | `GET /photos/<B photo>/preview?pair=` | — | 200, the bytes of the preview file the session view shows for it | PH4 |
| LV2·C23(b) | `GET /containers/<A>/example?pair=` | A has an example file | 200, the stored file's bytes | PH4, H3 |
| LV2·C23(c) | `GET /photos/<id>/preview?pair=` | a photo of a non-active session | 404 | PH4 (the phone follows the active session) |

### Delegated choices
Router layout · JSON body limits · the content-type table · how `TestPhone` reaches a session switch (the sessions
service as the Mac's command does).

### Notes
- Hazard (H8): handlers hold no rules; anything that decides belongs to `current_shot.rs`, `phone_view.rs`,
  `template_placement.rs`, `review.rs`, `live_view.rs`, `camera.rs`.
- Hazard (PH2): the server must survive App rebuilds: never hold an `AppHandles` across requests.
- The placeholder page is replaced in LV3; keep it to the token handling and two calls.

### Unit test commands
`cargo test -p psm-testkit --test suite lv2_phone_http::` · `cargo test -p psm-integrations phone::` ·
`cargo test -p psm-desktop` · `cd apps/desktop && npm run build` · `npm run typecheck` ·
`cargo clippy -p psm-integrations -p psm-testkit -p psm-desktop --all-targets -- -D warnings`

## Unit 5 — The WebSocket: demand, frames, events, lifecycle

**Unit id:** `LV2·U5` · **Rows:** LV2·C24–C29 · **Changes app files:** yes

### Goal
PH5: the viewer as a `LiveOutlet` (frames by demand through a `LatestSlot`, events through a bounded queue), the frame
header, pings and timeouts, revoke closes, `session`/`camera`/`focus`/`focus_frames`/`capture` messages; `live.viewer`
log lines; the desktop guide.

### Read first
Master plan §5 PH5, HB3–HB5 · `crates/core/src/services/live_view.rs` (LV1) · `crates/integrations/src/lib.rs`
(`EventHub::observe`) · axum 0.8 `extract::ws` (the installed source under `~/.cargo/registry/src/*/axum-0.8.9/src/
extract/ws.rs`).

### Files
New: `crates/integrations/src/phone/ws.rs`, `crates/testkit/tests/suite/lv2_phone_ws.rs`. Edited:
`crates/integrations/src/phone/{server.rs, wire.rs, mod.rs}`, `crates/testkit/{Cargo.toml, src/phone.rs}`,
`crates/testkit/tests/suite/main.rs`; `docs/guide/{01_product.md, 02_architecture.md, 03_rules_and_principles.md,
04_change_impact.md, 05_working.md}`.

### Core mechanisms
`wire::frame_header` (PH5 layout).

### Ordered tasks
1. `/api/phone/v1/ws`: auth before upgrade; `hello`; subscribe a viewer outlet to the hub; `demand` → `set_demand` →
   `demand_ack`; writer task: takes from the viewer's `LatestSlot` (frames) and its event queue (≤ 64), drops a frame
   whose `gen` ≠ the viewer's current gen, writes binary/text with `write_timeout`; pings every `ping`, closes after
   `idle_close`; close → unsubscribe.
2. `session` messages from `EventHub` (`ActiveSessionChanged`, and `SessionChanged`/`PhotoAdded`/`PhotoChanged` of the
   active session; `rev` increases); `camera` messages on status/live/exposure change; `focus`, `focus_frames`,
   `capture` from hub events.
3. Revoke closes the pairing's sockets with 4401 (PH1).
4. Tests for the rows; guide (flow §7A): `01_product.md` (the phone app, pairing, the current shot), `02_architecture.md`
   (current shot in placement, the phone server, the hub's outlets), `03_rules_and_principles.md` (MC-LV3 rules in the
   template section; the phone's no-secure-context rule; tokens never logged), `04_change_impact.md` (rows: the phone
   wire ↔ `wire.rs` ↔ `apps/desktop/src/phone/api.ts`/`socket.ts`; `CurrentShots` ↔ ledger and arrival; the
   `phone_pairings` table), `05_working.md` (how to pair and try the phone in dev).

### Criteria (6 criteria, 15 rows)
Boundary: WebSocket clients (`tokio-tungstenite`) connected to `TestPhone`; frames published into the TestApp's
`live_hub()` by the test (a started source and `CameraFrame`s), hub events reported by the test or through the fake
sidecar (outcome = the messages each client receives, close codes, the hub's viewer count).
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| LV2·C24(a) | WS upgrade | no token; a revoked token | HTTP 401, no upgrade | PH3, PH5 |
| LV2·C24(b) | same | valid token | first message `{type: "hello", viewer_id}`; the hub has one more viewer | PH5 |
| LV2·C25(a) | demand | connected, no demand sent; 10 frames published | no binary message within 300 ms after the last publish | PH5, HB3, MC-LV2, M-LV2 |
| LV2·C25(b) | same | send `{type: "demand", gen: 3, live: true, visible: true}`; after `demand_ack` 3, publish 5 frames | 5 binary messages, each header `gen` 3, `seq` strictly increasing, the first `seq` greater than the hub's newest seq at the ack | PH5, HB3, M-LV2 |
| LV2·C25(c) | same | then `{gen: 4, live: false, visible: true}`; after `demand_ack` 4, 10 frames | no binary message after the ack | PH5, HB3, MC-LV2 |
| LV2·C25(d) | same | `{gen: 5, live: true, visible: false}`; after the ack, 10 frames | no binary message | PH5, HB3 |
| LV2·C26(a) | two viewers | both wanting; client A reads nothing while 50 frames are published; client B reads all | B received 50 binary messages; A, when it starts reading, receives at most 2 binary messages and the last one's `seq` is the 50th frame's | PH5, HB4, M-LV1 |
| LV2·C27(a) | events | viewer X wanting, viewer Y not wanting; hub reports `Focus{"k", Focused}` | both get `{type: "focus", focus_id: "k", outcome: "focused", reason: null}` | PH5, HB5 |
| LV2·C27(b) | same | a photo arrives in the active session | both get `{type: "session", session_id: <active>, rev}` with `rev` above the previous | PH5, LV-D18 |
| LV2·C27(c) | same | the hub reports `Capture{capture_id: "c1", state: Exposed}` | `{type: "capture", capture_id: "c1", state: "exposed"}` | PH5, HB5, 10J §phone design (reconcile) |
| LV2·C27(d) | same | the hub's state becomes `Unavailable{"not_supported"}` | `{type: "camera", …, live: "unavailable", reason: "not_supported", …}` | PH5, 10J §Status |
| LV2·C28(a) | lifecycle | a client closes | within 1 s the hub's viewer count is back to what it was | PH5, MC-LV2 |
| LV2·C28(b) | same | `PhoneTimings {ping: 200 ms, idle_close: 600 ms}`; a client that stops reading (sends no pong) | the server closes it with 4408 within 1.5 s; the viewer is gone | PH5, MC-LV2 (viewer disappears) |
| LV2·C28(c) | same | the pairing is revoked while connected | the socket closes with 4401 within 1 s | PH1, PH5 |
| LV2·C29(a) | frame bytes | a published frame (1024×680, 7 JPEG bytes) to a wanting viewer | one binary message of 39 bytes: `PSMF`, version 1, flags 0, reserved 0, `seq`, `gen`, width 1024, height 680, `captured_at_ms` (little-endian, PH5 offsets), then the 7 bytes unchanged | PH5, H3 |

### Delegated choices
Tokio task layout · the event-queue type · how `connected` (U3's list) is computed from open viewers.

### Notes
- Hazard (HB): `offer`/`event` run under the hub mutex: they only `put` into the viewer's slot/queue and wake its
  writer (`tokio::sync::Notify::notify_one`); never `.await`, never write to the socket there.
- Hazard: a stuck socket must not hold memory: one frame in the slot, ≤ 64 events, then close (4409).

### Unit test commands
`cargo test -p psm-testkit --test suite -- lv2_phone_ws:: lv2_phone_http::` · `cargo test -p psm-integrations phone::` ·
`cargo clippy -p psm-integrations -p psm-testkit --all-targets -- -D warnings`

## Closing gate (once, after the last unit; replaces the template's closing gate)
`cargo test --workspace` · `cargo clippy --workspace --all-targets -- -D warnings` · `cargo fmt --all -- --check` ·
`tools/sidecar/build.sh --stub --test` · `cd apps/desktop && npm run typecheck` · `npm test` · `npm run build`. One
command at a time. A failure is fixed and only the failing files re-run.

## Review mode
The reviewer reads the units in order and runs, once: `cargo test -p psm-testkit --test suite -- lv2_current_shot::
lv2_phone_ops:: lv2_phone_pairing:: lv2_phone_http:: lv2_phone_ws:: t2_template_placement:: t3_template_hand::
t4_template_retake_shift::` · `cargo test -p psm-desktop` · `cd apps/desktop && npx vitest run src/features/settings`
· clippy on the touched crates, fmt, typecheck. The implementer's full-suite stamp stands. Only B1–B4 block. Also
check, by reading: every rewritten t2/t3/t4 test is listed with the 10G rule it encoded and now asserts MC-LV3; the
exposure binding happens inside the ledger worker lock and the same transaction as the insert; no handler in
`src/phone/` decides anything a service should (H8); no token in any log line or column; no POST acts without the
session guard. Notes go to `docs/implementation/backlog_live/LV2_current_shot_and_phone_server.md`.

## Manual try-out (the narrative is built from this)
On this Mac, with the app built from this branch (`npm run tauri dev` from `apps/desktop`, after `npm run build` once
so the phone page exists):
1. Open a session whose item type has a shooting template. The container outlined "Next" is the first in shooting
   order.
2. Shoot with the camera's own button: the photo lands in that container and "Next" moves to the next empty one.
3. Shoot two more quickly over Wi-Fi: each lands in the container that was "Next" when you pressed, even if the
   files arrive out of order.
4. Settings → Phone → **Pair a phone**: a QR code appears. Scan it with the phone's camera: the phone opens a simple
   page saying "Connected to <item>".
5. Settings → Phone → **Remove** that phone: reload the page on the phone; it says it was removed.

## Review log
- 2026-11-26, opus (implementer; the runner moved to Claude Opus after Codex Sol/Terra were out of usage; a Codex
  session had left uncommitted U1 work, which this session reviewed and reworked), implement r1. **Built:** U1 the
  current shot (`services/current_shot.rs` `CurrentShots`, `mechanisms/current_shot.rs::next_untaken`,
  `shot_order::slot_owners`; binding at exposure inside the ledger lock and the insert transaction, binding at arrival,
  placement by binding, `retake`, Mac "Next" = current shot via `session_view_with_current`, `ActiveSessionChanged`);
  U2 `phone_view`, `shot_ratio`, `move_to` (swap), Delete = Deny; U3 `phone_pairings` + `PhonePairings`,
  `settings.phone` (port rule), host/QR, five commands, Settings → Phone; U4 `PhoneServer` (own thread/runtime, peers,
  auth, static files without traversal, manifest, the PH4 routes with the session guard), web root, two Vite inputs,
  `bundle.resources`, placeholder phone page, `TestPhone`; U5 the WebSocket viewer (LatestSlot frames by demand
  generation, ≤ 64 events, pings/idle 4408, slow write 4408, overflow 4409, revoke 4401, `session`/`camera`/`focus`/
  `focus_frames`/`capture`/`demand_ack`), guide files. Checkpoints: U1 `7a685d76`, U2 `18cdd4f2`, U3 `84d94253`,
  U4 `d3db9091`, U5 `1463728a`.
  **Judgment calls** (all in the handoff's Assumptions): "hand wins" applies to a photo coming into a container, a
  template photo already there stays (keeps t2 B1); `ActiveSessionChanged` only on a real change; `current()` of a
  non-active session = its default, held value untouched; a held `Free` is not re-read (CS1 advances only a held
  shot); `phone_view` takes `now`; image ratio/revision from the photo's composition; untemplated `move_to` refuses
  `Container`; preview = developed, else original, else thumbnail; POSTs answer `{"ok":true}`; one `camera` snapshot
  after `hello`; the listener's send buffer is fixed at 256 KB (HB4: measured — a non-reading client otherwise let
  macOS buffer more than one 1 MB frame); each App's EventHub is observed when a request or socket resolves it.
  **Rewritten t2/t3/t4 tests** (rule encoded → MC-LV3 outcome) are listed in the handoff. **Deviations:** files
  outside the unit lists (t1_settings_drawer test, core helpers in `phone_view.rs`/`template_placement.rs` for H8,
  `live_view.rs::viewer_count`, desktop `commands/{sessions,templates,settings}.rs`, `test_app.rs`); the U1 checkpoint
  dropped five t4 shift tests by an editing mistake, restored unchanged in U4. **Observations:** C3(b) needs an
  earlier file of the counter run (the test starts with a Condition shot DSC00100); `psm-desktop` does not build
  without `apps/desktop/dist/` (the `bundle.resources` mapping; verified), so `npm run build` must run before the
  cargo part of the closing gate on a fresh checkout; `npm run build` writes `dist/phone.html` and `dist/phone/icon.png`.
  **Closing gate:** `cargo test --workspace` red once at `p01_storage_identity::c1a_open_creates_all_domain_columns`
  (the schema tripwire: the new `phone_pairings` table and binding columns); its expected schema updated, the file
  re-run green (20/20). Clippy, fmt, sidecar stub (2/2), typecheck, `npm test` (139 files / 953 tests), `npm run build`
  green.
