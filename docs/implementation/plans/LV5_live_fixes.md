---
phase: LV5_live_fixes
title: Fixes from the hardware check — the next shot never aims at a shot whose photo is still arriving; Deny, Main/Condition, reorder, move, undo and the other Mac actions show on the phones at once; a Wi-Fi stall makes the phone skip live frames instead of dropping its connection; the camera's Focus Area set to Wide is reported instead of pretending to focus; the exposure readout reaches the phone; a photo's preview tells the phone when it is ready; Settings → Phone shows which phone is connected
difficulty: high
projection: no
depends_on: [LV3_phone_app]
blocked_on: none
---
# LV5 — Fixes from the hardware check (findings F1–F5, F7 server half, the Settings list)

**Tracker:** IMPLEMENTED (2026-11-26, codex) — four unit checkpoints; amended P40 event expectation; workspace suite, clippy, fmt, sidecar stub, typecheck and 1,049 UI tests green.

(This line is the milestone's tracker row. Sessions change only this line of this file. The gate for `depends_on` is
the Tracker line of the plan named there.)

## Goal
The owner-attended hardware check (`LV4_hardware_check.md`, paused; running record `docs/implementation/STATUS_live.md`
§"LV4 hardware check in progress"; app log `~/Library/Application Support/com.loorenz.sessionmanager/logs/
psm.2026-11-26.log`, read-only) found six defects in what LV1–LV3 built. Each is a contract the code did not meet, or a
contract that was wrong; `master_plan_live.md` §5 carries the dated amendments (CS4a, PH5a, PH6a, PH8, FC4a, HB5a).
(U1) **F4**: the current shot treats a container whose shutter is bound but whose file is not yet a photo as empty.
(U2) **F3**: Mac review actions announce nothing to the phones; **F7 server half**: the session view gives the phone no
way to know a preview became ready. (U3) **F2**: one slow frame write closes the phone's socket; the Settings → Phone
list is read once. (U4) **F5**: Focus Area Wide is not detected on the a7R V; **F1**: the real helper's
`liveview.exposure` line is refused by the Rust protocol.
**Not here:** the phone UI (LV6: the dock of LV-D19, the phone half of F7, the removed text), F6 (owner accepted
reinstalling, LV-D19 (2)), HTTPS, drawing the camera's own focus frame on the phone (STATUS_live "Needs the owner"), the
open LV1/LV2 backlog carry-overs (`backlog_live/`; none touches a file this milestone changes except `ws.rs`, whose LV2
notes were folded in LV3), the hardware rows LV4 has not run yet.

## How to work this plan (flow.md §10, master_plan_live.md §3)
One runner phase of 4 units, in order; after each, its **Unit test commands**, then a `CHECKPOINT (not approved):`
commit of that unit's paths. Row ids `LV5·C<n>(<row>)`; tests named after their row (`lv5_c3a_<slug>`). Open details:
the smallest option consistent with 10J and the amended §5, recorded under **Assumptions**. Reviewer notes:
`docs/implementation/backlog_live/LV5_live_fixes.md`. Default implementer (camera code: never `implementer: fable`).
**Run `cd apps/desktop && npm run build` once before the first cargo command** (`psm-desktop` needs
`apps/desktop/dist/`, §8).

| Unit | What it fixes | Criteria | Rows | Changes app files |
|---|---|---|---|---|
| U1 | F4: shutters whose files are on the way, until the photo exists | 4 | 9 | yes (core, infra, testkit) |
| U2 | F3: Mac actions reach the phones; F7: preview version in the session view | 4 | 18 | yes (core, integrations) |
| U3 | F2: a Wi-Fi stall skips frames; Settings → Phone list stays current | 2 | 6 | yes (integrations, desktop UI) |
| U4 | F5: Focus Area not movable is reported; F1: the exposure readout | 2 | 16 | yes (C++ sidecar, integrations) |
| **Total** | | **12** | **49** | |

## Read first (whole milestone)
`docs/process/flow.md` (§4, §4A, §5, §6, §7A, §10) · `master_plan_live.md` (§3, §4, **§5 CS, FC, HB, PH with their
2026-11-26 amendments**, §6, §8) · book `10J` (MC-LV3, LV-D14, LV-D19, §The phone design) · `STATUS_live.md` §"LV4
hardware check in progress" (findings F1–F7 with the observed values) · the LV2 and LV3 plans (rows this milestone keeps
or extends) · `docs/guide/02_architecture.md` §The current shot, §The phone server · `docs/guide/04_change_impact.md`.

## Points recorded by planning (each is one sentence in the narrative's "Choices made for you")
- **P1 A shot whose photo is still on its way counts as taken until its photo exists** in the session, or until 5
  minutes after its shutter (the camera ledger's matching window) if the photo never appears.
- **P2 A phone keeps its connection through a Wi-Fi stall of up to 15 seconds**; live frames pause and resume with the
  newest one; after 15 seconds with nothing getting through, the Mac lets the phone go and the phone reconnects by
  itself.
- **P3 Only Flexible Spot focus areas (any size, also with tracking) count as movable** for focus from the phone; Wide,
  Zone, Center and their tracking variants are reported with the "set a movable Focus Area" message and the camera is
  left as it is.
- **P4 Settings → Phone re-reads the list every 5 seconds while it is open**; a connected phone reads "Connected now".
- **P5 An exposure value the camera does not report** (no lens, a special mode) shows as "–" in the phone's readout.

## Unit 1 — The current shot: shutters whose files are on the way, until the photo exists (F4)

**Unit id:** `LV5·U1` · **Rows:** LV5·C1–C4 · **Changes app files:** yes (core, infra, testkit)

### Goal
`master_plan_live.md` §5 **CS4a**. Today an exposure stops holding its container the moment its file is **filed**
(`camera_deliveries` row with `exposure_id`, ledger `decide_and_place`) but the photo row appears only when the arrival
scan takes the file in, about 1.1 s later (`arrival.photo_added … filed_to_row_ms=1058`). In that gap neither the
hold query (`crates/infra/src/sqlite/store.rs` `hold_times`: `status='expecting' AND NOT EXISTS(delivery)`) nor the
photo set sees the container, so `advance_after_binding`/`resolve`/`default_shot` treat it as empty. Evidence (log
2026-11-26): exposure 60 bound container 10 at 14:06:57.99; `ftp.upload filed name=DSC07532.ARW` at 14:07:01.78;
exposure 61 bound 8 at 14:07:02.22 → `current_shot.set container=10 by=advance`; DSC07532's photo row at 14:07:02.82 →
`set container=none`. Same at 14:07:07.375 (`set container=8` while exposure 61's file, filed at ≈14:07:06.45, was not
yet a photo; corrected at 14:07:07.58).

### Read first
Master plan §5 CS2, CS4, **CS4a**, CS5, CS6 · `crates/core/src/services/current_shot.rs` (all) ·
`crates/core/src/mechanisms/shot_order.rs` (`slot_owners`, `order`) · `crates/core/src/ports/store.rs` (`hold_times`) ·
`crates/infra/src/sqlite/store.rs` (`hold_times`, `match_late_exposure`) · `crates/core/src/services/camera_ledger.rs`
(`decide_and_place`: where the delivery gets its `exposure_id` and the exposure its status) ·
`crates/testkit/src/test_app.rs` (`arrival_photo`, `inject`, `with_camera_script`) ·
`crates/testkit/tests/suite/lv2_current_shot.rs` (fixture T3 and helpers).

### Files
New: `crates/testkit/tests/suite/lv5_current_shot.rs`. Edited: `crates/infra/src/sqlite/store.rs` (the on-the-way
query), `crates/core/src/ports/store.rs` (only if a method is added or renamed), `crates/core/src/services/
current_shot.rs` (only if `shot_plan`/`taken` need the new set), `crates/testkit/src/test_app.rs` (a helper that
delivers a file without taking it in, and one that takes it in later; `arrival_photo` may be rebuilt on them),
`crates/testkit/tests/suite/main.rs`.

### Core mechanisms
None new (`next_untaken` and `slot_owners` unchanged).

### Ordered tasks
1. The on-the-way set of CS4a: an exposure of the session, `now − exposed_at ≤ FIFO_WINDOW_MS`, not a duplicate-only
   delivery, for which **no photo of the session has `raw_path` = the `final_path` of one of its non-duplicate
   deliveries** — whether its file is not delivered yet, pending, or filed and not yet taken in. It holds its slot in
   `slot_owners` exactly as a hold does today (so CS2's owner of that slot, and through it the binding, count as
   taken). Simplest consistent change: widen `hold_times`'s `NOT EXISTS` from "a delivery" to "a delivery whose file is
   a photo of the session"; the alternative (a separate set) is the implementer's choice if `order`'s results for the
   existing t2/LV2 fixtures would otherwise change.
2. Testkit: split `arrival_photo` into "deliver (filed)" and "take in (arrival scan)" so a test can act in the gap;
   both get callers in this unit.
3. Tests for the rows; re-run the LV2 current-shot and placement stems unchanged.

### Criteria (4 criteria, 9 rows)
Boundary: the wired TestApp (outcome = `camera_exposures` binding columns, `CurrentShots::current`,
`TemplateView.next_shot`, `phone_view`'s `current`, photos' places, `current_shot.*` log lines). Fixture **T3** of
LV2·U1 (containers A final 1 shoot 2, B final 2 shoot 1, C final 3 shoot 3; shooting order B, A, C). "Filed" = the
file delivered through `CaptureSink::delivery` (the ledger matches it to its exposure, as `ftp.upload filed` does);
"taken in" = the arrival scan creates its photo row. Exposures through `CaptureSink::exposure_observed` (the call the
direct adapter makes on `capture.exposed`) unless the row says the fake sidecar.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| LV5·C1(a) | `exposure_observed` in the gap | e1 bound B, its file filed and taken in (photo in B); e2 reported (bound A), its file **filed, not taken in**; then e3 reported | e3's row `shot_container_id` = C; then `current` = `Free` (not `Shot(A)`); log `current_shot.set … container=none by=advance` after e3's `current_shot.bound` | CS4a, MC-LV3, M-LV4 (log 14:07:02.224) |
| LV5·C1(b) | same | photos in B and A; current `Shot(C)`; e3 reported (bound C), its file filed, not taken in; then e4 reported | e4 bound to no container (`shot_bound = 1`, `shot_container_id` NULL); `current` = `Free` (not `Shot(C)`) | CS4a, CS4 (after a Free binding), M-LV4 (log 14:07:07.375) |
| LV5·C1(c) | end of the window | new T3 session; e1 reported (bound B), its file filed and **never** taken in; 301 s after e1's shutter, `set(Shot(B))` | `Ok`; `current` = `Shot(B)` (the hold ends with the FIFO window, as a hold without a file does) | CS4a, P1 |
| LV5·C2(a) | arrival of the pending file | after C1(a), e2's file taken in | e2's photo Main in A; `current` still `Free`; that arrival logs no `current_shot.set` line | CS4a, CS6, MC-LV3 (the current shot never moves back) |
| LV5·C2(b) | same | after C1(b), e3's file taken in, then e4's (filed and taken in) | e3's photo in C; e4's photo Condition, last; `current` still `Free`; neither arrival logs `current_shot.set` | CS4a, 10G §4 rule 4 |
| LV5·C3(a) | fast Wi-Fi sequence, fake sidecar | fixture **T4** (four containers W, X, Y, Z in shooting order; final numbers 2, 4, 1, 3), app with the fake sidecar; five `capture.exposed` lines (`TestApp::inject`) at 0, 1.2, 2.4, 3.6, 4.8 s on the test clock (shutters s1…s5, camera counters `DSC00101`…`DSC00105`); files filed at 4.0 s (s1's), 5.6 s (s3's), 6.0 s (s2's), 7.6 s (s4's), 8.8 s (s5's), each taken in 1.0 s after it is filed (so s5 fires while s1's file is filed and not taken in, and s3's file arrives before s2's) | bindings s1…s5 = W, X, Y, Z, none; at the end W, X, Y, Z each hold the photo of the exposure bound to it and s5's photo is Condition; no two exposures share a container; every `current_shot.set … by=advance` line names a container no earlier exposure is bound to, or `none` | CS4a, CS2, CS5, M-LV4 |
| LV5·C3(b) | same, with a tap | as (a), but at 0.5 s the phone path `set(Shot(Z))`; files filed at 4.0 s (s2's), 4.4 s (s1's), 7.0 s (s4's), 7.4 s (s3's), 8.8 s (s5's), each taken in 1.0 s after it is filed (s5 fires while s2's and s1's files are filed and not taken in) | bindings W, Z, X, Y, none (after Z the advance skips W, whose file is on the way); each photo in its exposure's container; s5's photo Condition; the same log condition as (a) | CS4a, CS7, M-LV4 |
| LV5·C4(a) | Mac "Next" and the phone view in the gap | e1's photo in B; e2 (bound A) filed, not taken in | `TemplateView.next_shot` = 3 (C's shoot_no); `phone_view(…).session.current` = `{kind: "shot", container_id: C}` | CS4a, CS8, PH6 |
| LV5·C4(b) | `set` in the gap | same state; `set(Shot(A))` | `ShotFilled`; `current` unchanged (`Shot(C)`) | CS4a, CS7 |

### Delegated choices
Store method name and query shape · the testkit helpers' names (each with callers here) · how C3 drives the test
clock between injected lines (as `p46_direct_camera_adapter.rs` and `lv1_frame_channel.rs` do).

### Notes
- Hazard: `shot_order::order` results of the existing t2 tests must not change (those fixtures take every file in at
  once, so the widened set is empty there). Re-run `t2_template_placement::` and `lv2_current_shot::` unchanged.
- Hazard (CS5): the binding stays inside the ledger worker lock and the insert's transaction; the on-the-way read is
  part of that transaction.
- Hazard (10E MC-33): a delivery with outcome `duplicate` never makes or ends a hold.

### Unit test commands
`cargo test -p psm-testkit --test suite -- lv5_current_shot:: lv2_current_shot:: t2_template_placement:: t3_template_hand:: t4_template_retake_shift:: p44_capture_ledger::` ·
`cargo test -p psm-core mechanisms::shot_order` ·
`cargo clippy -p psm-core -p psm-infra -p psm-testkit --all-targets -- -D warnings`

## Unit 2 — Mac actions reach the phones; the preview version (F3, F7 server half)

**Unit id:** `LV5·U2` · **Rows:** LV5·C5–C8 · **Changes app files:** yes (core, integrations)

### Goal
Master plan §5 **PH8** and **PH6a**. F3: `photos_delete` → `ReviewService::delete` emits nothing (LV3·U0 added
`delete_announced` only for the phone's own delete); `ReviewService::{classify, reorder, delete, keep_published,
move_photos, copy_photos}` emit no event, and `HistoryService::reverse` emits `PhotoChanged`/`SessionChanged` only for
step kinds that fill `changed_photos` (Composition, Template), not for Classify, Reorder, Move, Copy, KeepPublished.
The phone sockets forward `SessionChanged`/`PhotoAdded`/`PhotoChanged` of the active session as `{type: "session"}`
(`crates/integrations/src/phone/ws.rs` `AppEventsBridge`), so nothing reaches them. Events are emitted by the services,
never by Tauri commands or HTTP handlers (H8). F7: the preview route answered 404 ×31 before previews existed
(`phone.request route=/api/phone/v1/photos/{id}/preview status=404`, e.g. 14:06:58.623) and the phone's URL
(`…/preview?pair=…&r=<composition revision>`) never changes when the preview appears; `previews.rs` already emits
`PhotoChanged` when a preview is written, so the server half is the `preview` version token of PH6a.

### Read first
Master plan §5 PH5, **PH6a**, **PH8** · `crates/core/src/services/review.rs` (all, incl. `delete_announced`) ·
`crates/core/src/services/history.rs` (`History::emit`, `HistoryService::reverse`, `apply`) ·
`crates/core/src/services/template_placement.rs` (`HandAction::finish`: the pattern) ·
`crates/integrations/src/photo_files.rs` (`move_photos`, `after_history`) · `apps/desktop/src-tauri/src/commands/
{photos.rs, history.rs, templates.rs}` (how the Mac builds the services: unchanged by this unit) ·
`crates/core/src/services/phone_view.rs` (`image`, `preview_file`) · `crates/core/src/services/previews.rs` (where a
preview is written and `PhotoChanged` emitted) · `crates/integrations/src/phone/{ws.rs, routes.rs}` ·
`crates/testkit/src/phone.rs` (`TestPhone`) · `crates/testkit/tests/suite/{lv2_phone_ws.rs, lv3_server_carry.rs}`.

### Files
New: `crates/testkit/tests/suite/lv5_mac_actions.rs`. Edited: `crates/core/src/services/{review.rs, history.rs,
phone_view.rs}`, `crates/integrations/src/photo_files.rs` (only if `move_photos` needs to announce the second
session), `crates/integrations/src/phone/routes.rs` (the delete route calls the single announcing delete, if
`delete_announced` is folded), `crates/testkit/tests/suite/main.rs`. Tauri command files change only if a service
constructor signature changes (avoid: the services can reach the app's sink through `History`, which every Mac
command already passes).

### Core mechanisms
None.

### Ordered tasks
1. `ReviewService`: after each committed `classify`, `reorder`, `delete`, `keep_published`, `move_photos`,
   `copy_photos`: `PhotoChanged{photo, s}` per photo it changed, then `SessionChanged{s}` per session it changed (move:
   source and target; copy: target). One emission path; `delete_announced` folds into `delete` or stays a one-line
   alias (no double events on the phone delete).
2. `HistoryService::reverse`: after a committed undo or redo of **any** step kind, `SessionChanged{s}` for every
   session in `touched` (deduplicated), plus `PhotoChanged` for the photos the step names where they are known.
3. `phone_view` `image`: `preview` per PH6a (null while `preview_file(photo)` is `None` or that file is missing; else a
   token that changes when the served file's path or modification time changes; never a path).
4. Tests for the rows.

### Criteria (4 criteria, 18 rows)
Boundary: a `tokio-tungstenite` client of `TestPhone` (the real `PhoneServer` over the TestApp's handles), connected
before the action; the action performed through the service exactly as the Mac's command builds it
(`ReviewService::new(store, clock, history)`, `psm_integrations::photo_files::move_photos(&handles, …)`,
`HistoryService::new(store, clock, history).undo/redo`, `HandAction` as `template_drop`/`template_back`/
`template_shift_back` do). Outcome = within 1 s the client receives `{type: "session", session_id: <active>, rev}` with
`rev` above every earlier `rev` it received, **and** the TestApp's event hub recorded `SessionChanged{active}` after
the action. Fixture: active T3 session with photos in B and A, one Condition photo, one Deleted photo; a second session
for move/copy.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| LV5·C5(a) | Mac review action | `classify(<A's photo>, Condition)` | the `session` message and `SessionChanged` as stated | PH8, PH5, LV-D18 |
| LV5·C5(b) | same | `classify(<the Deleted photo>, Main)` (restore) | same | PH8 |
| LV5·C5(c) | same | the active session is an untemplated one with Main [P1, P2] (made active before the client connects); `reorder(P2, 1)` | same, for that session | PH8 |
| LV5·C5(d) | same | `delete(<A's photo>)` (the Mac's Deny; finding F3) | same; also `PhotoChanged{A's photo}` | PH8 (F3, log 13:5x "the phone interface didn't react") |
| LV5·C5(e) | same | `keep_published(<active>)` on a published session with one photo added after the publish | same | PH8 |
| LV5·C5(f) | same | `photo_files::move_photos(&handles, [<the Condition photo>], <second session>)` | same (the active session lost a photo) | PH8 |
| LV5·C5(g) | same | `copy_photos([<a photo of the second session>], <active>)` | same | PH8 |
| LV5·C6(a) | Mac undo / redo | after C5(a), `undo(active)` | same | PH8, 10G §6 rule 5 |
| LV5·C6(b) | same | then `redo(active)` | same | PH8 |
| LV5·C6(c) | same | after C5(c), `undo` | same | PH8 |
| LV5·C6(d) | same | after C5(d), `undo` | same; the photo is back in A | PH8 |
| LV5·C6(e) | same | after C5(f), `undo(active)` | same | PH8 |
| LV5·C7(a) | Mac template hand action | `drop_on_container` of the Condition photo onto A (filled) | same | PH8, CS8 |
| LV5·C7(b) | same | A's photo dragged to Condition by hand, then **Back to template** | same, for the Back action | PH8, CS8 |
| LV5·C7(c) | same | Deny A's photo (Mac), then **Shift the rest back** | same, for the Shift action | PH8, CS8 |
| LV5·C8(a) | `phone_view` `preview` | a photo whose preview file does not exist yet; then the preview step writes it | `preview` = null; then a non-empty string with no `/` and no `\` | PH6a, F7 |
| LV5·C8(b) | same | that preview re-rendered (the file rewritten with other bytes and a later modification time) | `preview` differs from (a)'s string | PH6a |
| LV5·C8(c) | socket | a phone socket open; the preview step writes a photo's preview in the active session | the socket receives a `session` message (the phone re-reads and gets the new `preview`) | PH5, PH6a, F7 |

### Delegated choices
How `ReviewService` reaches the event sink (through `History`'s sink, or a `with_events` set where the services are
built) · whether `delete_announced` remains as an alias · the token's form (e.g. modification time in ms and size) ·
how C8(a) obtains a photo without a preview file (before `complete_preview_export`, or its preview file removed).

### Notes
- Hazard: the Mac's own window also reloads on `session_changed` (`apps/desktop/src/services/events.ts`); that is
  today's behaviour after template hand actions, so a Mac action now triggers one extra re-read of its own session.
  Acceptable; do not suppress it.
- Hazard (H8): no `emit` in `apps/desktop/src-tauri/src/commands/` or `crates/integrations/src/phone/routes.rs`.
- Hazard: a `PhotoChanged`/`SessionChanged` of a non-active session sends phones nothing (unchanged
  `AppEventsBridge` filter).
- The token never carries a local path (LV3·C29: no local path reaches the phone).

### Unit test commands
`cargo test -p psm-testkit --test suite -- lv5_mac_actions:: lv2_phone_ops:: lv2_phone_ws:: lv3_server_carry:: t3_template_hand:: t4_template_retake_shift::` ·
`cargo clippy -p psm-core -p psm-integrations -p psm-testkit --all-targets -- -D warnings`

## Unit 3 — A Wi-Fi stall skips frames; the Settings → Phone list stays current (F2)

**Unit id:** `LV5·U3` · **Rows:** LV5·C9–C10 · **Changes app files:** yes (integrations, desktop UI)

### Goal
Master plan §5 **PH5a**. Today `serve` (`crates/integrations/src/phone/ws.rs`) wraps every write in
`tokio::time::timeout(write_timeout = 2 s)` and closes the socket (4408 "slow", log `reason=write_timeout`) when one
**frame** write takes longer, and while a write is awaited the loop does not serve revoke or reader-closed. Evidence:
the Android was dropped five times in 2 min on live over Wi-Fi (`phone.viewer closed … reason=write_timeout`,
13:51–13:53), the iPhone once (14:15:43); each reconnect showed the template for a moment. PH5a: a frame write is never
cut short (newer frames replace the waiting one in the viewer's `LatestSlot`, so the phone skips frames); the socket
closes only when a single write has made no progress for `stall_close` (default 15 s) or nothing is heard for
`idle_close`; revoke and reader-closed are served while a write is in flight. Settings → Phone: `PhoneSection.tsx`
reads `phonePairingsList` only on mount and after its own actions (the Android read "Never connected" while
connected).

### Read first
Master plan §5 PH1, PH5, **PH5a**, HB4, §4.3 (`PhoneTimings`), §4.6 · `crates/integrations/src/phone/{ws.rs, server.rs}`
· `crates/testkit/src/phone.rs` · `crates/testkit/tests/suite/lv2_phone_ws.rs` (C26, C28 and their timings) · axum 0.8
`extract::ws` (`~/.cargo/registry/src/*/axum-0.8.*/src/extract/ws.rs`: a `SplitSink` send that is not awaited to
the end keeps its message buffered; the next send or flush writes it first) ·
`apps/desktop/src/features/settings/PhoneSection.tsx` and `__tests__/lv2_phone_settings.test.tsx`.

### Files
New: `crates/testkit/tests/suite/lv5_phone_socket.rs`, `apps/desktop/src/features/settings/__tests__/
lv5_phone_list.test.tsx`. Edited: `crates/integrations/src/phone/{ws.rs, server.rs}` (`PhoneTimings`: `write_timeout`
→ `stall_close`), `crates/testkit/src/phone.rs`, `crates/testkit/tests/suite/lv2_phone_ws.rs` (its `PhoneTimings`
literals only), `crates/testkit/tests/suite/main.rs`, `apps/desktop/src/features/settings/PhoneSection.tsx`.

### Core mechanisms
None.

### Ordered tasks
1. `PhoneTimings {ping: 5 s, idle_close: 15 s, stall_close: 15 s}` (`write_timeout` removed; §4.3 amended).
2. The writer: one write in flight at a time, kept as a future the `select!` polls beside ping, report, revoke and
   reader-closed; a frame taken from the slot only when no write is in flight; a write in flight for `stall_close`
   → close 4408 `"stalled"`, log `phone.viewer closed … reason=stalled`; revoke during a write → drop the write, best
   effort 4401, unsubscribe within 1 s. Pings are sent between writes. Events queue as today (≤ 64, then 4409).
3. `PhoneSection`: re-read `phonePairingsList` every 5 s while mounted (cleared on unmount); a pairing with `connected`
   reads "Connected now" in place of its last-seen text.
4. Tests for the rows; update the LV2 tests' `PhoneTimings` literals (no row changes).

### Criteria (2 criteria, 6 rows)
Boundary: `tokio-tungstenite` clients of `TestPhone` with frames published into the TestApp's `live_hub()` (outcome =
messages received, close code, the hub's viewer count, `phone.viewer` log lines); the Settings component with
`services/phone.ts` mocked and fake timers.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| LV5·C9(a) | slow reader | `PhoneTimings {ping: 200 ms, idle_close: 5 s, stall_close: 3 s}`; a wanting client stops reading for 1.5 s while 40 frames of 1 MB are published 25 ms apart, then reads again; then one more frame is published | no close frame and no `phone.viewer closed` line; of the messages sent during the stall the client receives at most 2 binary messages and the last carries the 40th frame's `seq`; then it receives the frame published after (the old 500 ms `write_timeout` of `TestPhone` would have closed it) | PH5a, HB4, M-LV1, F2 |
| LV5·C9(b) | reader gone | same timings but `idle_close: 10 s`; a wanting client stops reading and never resumes; frames keep being published | the hub's viewer count is back to its value before the client within 4 s of the first write that could not finish; log `phone.viewer closed … reason=stalled` | PH5a, MC-LV2 (viewer disappears) |
| LV5·C9(c) | revoke during a stuck write | as (b); the pairing revoked 1 s into the stuck write | the viewer is gone within 1 s of the revoke; log `reason=revoked` | PH1, PH5a |
| LV5·C9(d) | defaults | `PhoneTimings::default()` | `stall_close` ≥ 10 s (the stalls at the hardware check lasted up to 8 s) and `ping` < `idle_close` | PH5a, charter rule 10 |
| LV5·C10(a) | `PhoneSection` | `phonePairingsList` answers `[{id 1, name "Android", connected: false, last_seen_at: null}]`, then from the next call `[{… connected: true, last_seen_at: <t>}]`; advance 5 s | the list was re-read; pairing 1's dot is "Connected" and its text "Connected now" | `[rule]` PH5, P4 |
| LV5·C10(b) | same | the section unmounted; advance 10 s | no further `phonePairingsList` call | `[rule]` P4 |

### Delegated choices
Task layout of the writer (one task with a pinned in-flight future, or a separate writer task fed by a channel of
depth 1) · how C9 measures "the first write that could not finish" (the test may take the time of the last publish
before the client's receive buffer filled, with the stated 4 s bound measured from the stall's start + `stall_close`).

### Notes
- Hazard (HB): `offer`/`event` still only `put`/push and `notify_one`; never `.await` under the hub mutex.
- Hazard: dropping an in-flight frame write mid-message corrupts the stream; never drop one except to close the
  socket (revoke, stalled).
- PH1's "closed within 1 s" on revoke now also holds while a write is stuck (C9(c)).

### Unit test commands
`cargo test -p psm-testkit --test suite -- lv5_phone_socket:: lv2_phone_ws:: lv2_phone_http::` ·
`cargo test -p psm-integrations phone::` · `cd apps/desktop && npx vitest run src/features/settings` ·
`npm run typecheck` · `cargo clippy -p psm-integrations -p psm-testkit --all-targets -- -D warnings`

## Unit 4 — Focus Area not movable is reported; the exposure readout (F5, F1)

**Unit id:** `LV5·U4` · **Rows:** LV5·C11–C12 · **Changes app files:** yes (C++ sidecar, integrations)

### Goal
Master plan §5 **FC4a** and **HB5a**.
F5: the focus worker (`native/camera-sidecar/src/live_runner.cpp` `focus()`) refuses only when `AF_Area_Position` is
not settable. On the a7R V with Focus Area **Wide** the position stays settable, the write is accepted, no focus frame
confirms it (`point_ms=-1`) and the half-press focuses where the camera chooses (`focus.result af=focused point_ms=-1
… total_ms=1406`, 14:13:50.56; four more to 14:14:09.88; the phone showed AF LOCKED). Under Wide the idle focus frame
list is empty and after AF the frames are type-2 phase points (`liveview.focus_frame frames=[]` 14:13:40.44,
`[[2,2,422,640,240,480,14,14],…]` 14:13:58.66), unlike Flexible Spot S (type 6/8, 38×38). FC4a decides by the **Focus
Area value** `focus_state()` already reads (`CrDeviceProperty_FocusArea`; values from the installed SDK header
`~/Desktop/Developer/Camera-Connection-Tests/vendor/sony-sdk/CrSDK_v2.02.00_Mac/RemoteCli/app/CRSDK/
CrDeviceProperty.h:1456–1485`: Wide = 1, Zone = 2, Center = 3, Flexible Spot S/M/L = 4/5/6, Expand Flexible Spot = 7,
Flexible Spot = 8, Tracking Wide/Zone/Center = 0x11/0x12/0x13, Tracking Flexible Spot S/M/L/Expand/Flexible =
0x14…0x18, Flexible Spot XS/XL/FreeSize1–3 = 0x19…0x1D, Tracking Flexible Spot XS/XL/FreeSize1–3 = 0x23…0x27; the lab
saw Flexible Spot S = 4 movable and the offered list 1–7, 26–29, evidence `Camera-Connection-Tests/results/
20261126T070321Z_10-focus-usb-3/summary.md`). The Focus Area is never written (LV-D14, FC6).
F1: the helper emits `liveview.exposure` with each value as the `props` object (the exact line, log
2026-11-26T14:06:58.216844Z: `{"timestamp":"2026-11-26T14:06:58.210Z","experiment":"adhoc","source":"sdk",
"event":"liveview.exposure","shutter_speed":{"code":"0x00000103","value":65616,"value_hex":"0x00010050",
"display":"1/80","writable":false,"readable":true},"f_number":{"code":"0x00000100","value":800,"value_hex":
"0x00000320","display":"F8.0","writable":false,"readable":true},"iso":{"code":"0x00000104","value":2000,
"value_hex":"0x000007D0","display":"ISO 2000","writable":false,"readable":true}}`), or the string `""` for a property
it could not read (`emit_exposure`'s initial value); `crates/integrations/src/camera/sidecar/protocol.rs` requires
three strings (`req_str`), so every line is "unparsable" and the phone's readout (LV3·C13(d)) never shows. The display
formats come from `native/camera-sidecar/src/camera_properties.cpp` `decode`: F-number `F%.1f` or `special`; shutter
`1/<den>`, `<x.y>"`, `BULB` or `special`; ISO `ISO <n>` or `AUTO`. The LV1 test `lv1_c15b_exposure_mapping`
(`crates/testkit/tests/suite/lv1_frame_channel.rs`) used an invented shape (`"8.0"`, `"100"`) the helper never sends
and is rewritten to the real one (row C12(b)).

### Read first
Master plan §5 FC4, **FC4a**, FC6, HB5, **HB5a**, §4.1 · `native/camera-sidecar/src/{live_runner.cpp, live_view.cpp,
camera_properties.cpp}` · `native/camera-sidecar/include/crlab/{liveview.hpp, camera.hpp}` (`FocusState`) ·
`native/camera-sidecar/tests/test_liveview.cpp` · the lab `~/Desktop/Developer/Camera-Connection-Tests/sdk/src/
camera_liveview.cpp` (`focus_state`, read-only) and `experiments/10-focus-from-live.md` ·
`crates/integrations/src/camera/sidecar/protocol.rs` (`LiveviewExposure`, the field readers) ·
`crates/integrations/src/camera/direct/live.rs` (`exposure_readout`) · `crates/testkit/tests/suite/
lv1_frame_channel.rs` (C15) · `crates/integrations/src/phone/ws.rs` (`camera` message).

### Files
New: `crates/testkit/tests/suite/lv5_camera_readouts.rs`. Edited: `native/camera-sidecar/include/crlab/liveview.hpp`
(or `src/liveview.cpp`: the pure `focus_area_movable`), `native/camera-sidecar/src/live_runner.cpp` (the focus job uses
it; may add `focus_area` to `focus.result` for the log), `native/camera-sidecar/tests/test_liveview.cpp`,
`crates/integrations/src/camera/sidecar/protocol.rs`, `crates/integrations/src/camera/direct/live.rs`,
`crates/testkit/tests/suite/{lv1_frame_channel.rs (C15(b) rewritten), main.rs}`.

### Core mechanisms
C++ `focus_area_movable(int64_t focus_area, bool area_read, bool position_writable) -> bool` (FC4a; pure, no SDK
types, numbers not enum names so the stub build tests it). Rust `camera::direct::live::exposure_readout(shutter_display,
f_number_display, iso_display) -> ExposureReadout` (HB5a).

### Ordered tasks
1. `focus_area_movable` per FC4a; the focus job refuses (`focus.result {refused: "focus_area_not_movable", point_set:
   false, af: "skipped"}`, nothing written) when it answers false. A movable area with an unconfirmed point
   (`point_ms = -1`, e.g. a tap near the edge where the camera clamps the point) still runs AF as today.
2. Protocol: each `liveview.exposure` field is a string (taken as its display) or an object with a string `display`
   (that string); anything else → the line is unparsable as today. `exposure_readout` per HB5a.
3. `tools/sidecar/build.sh --stub --test` after the C++ edit; tests for the rows; rewrite `lv1_c15b_exposure_mapping`
   to the real line.

### Criteria (2 criteria, 16 rows)
Boundary: C11 the pure C++ function (`tests/test_liveview.cpp`, stub build); C12 the protocol parser, the hub's
events through the fake sidecar (`TestApp::inject` of the exact JSON), a `TestPhone` socket, and `exposure_readout`.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| LV5·C11(a) | `focus_area_movable` (core mechanism) | area read 1 (Wide), position writable | false | FC4a, LV-D14, F5 (log 14:13:50) |
| LV5·C11(b) | same | 2 (Zone), writable | false | FC4a, P3 |
| LV5·C11(c) | same | 3 (Center), writable | false | FC4a |
| LV5·C11(d) | same | each of 4, 5, 6, 7, 8, writable | true for each | FC4a (lab: 4 movable) |
| LV5·C11(e) | same | each of 0x11, 0x12, 0x13, writable | false for each | FC4a, P3 |
| LV5·C11(f) | same | each of 0x14…0x18, writable | true for each | FC4a |
| LV5·C11(g) | same | each of 0x19…0x1D and 0x23…0x27, writable | true for each | FC4a |
| LV5·C11(h) | same | 0 (Unknown), writable | false | FC4a |
| LV5·C11(i) | same | 4, position **not** writable | false | FC4a, FC4 |
| LV5·C11(j) | same | area not read, position writable; area not read, not writable | true; false (the FC4 rule where the camera has no Focus Area property) | FC4a |
| LV5·C12(a) | protocol (sidecar line → `SidecarEvent`) | the exact line quoted in the Goal | `LiveviewExposure` with the displays "1/80", "F8.0", "ISO 2000" (not unparsable) | HB5a, FC4, F1 |
| LV5·C12(b) | fake sidecar → hub → phone | that line's fields injected as `liveview.exposure`; a `TestPhone` socket open | the hub reports `Exposure{shutter: "1/80", aperture: "f/8", iso: "ISO 2000"}`; the socket receives `{type: "camera", …, exposure: {shutter: "1/80", aperture: "f/8", iso: "ISO 2000"}}` (replaces LV1·C15(b)'s invented input) | HB5a, PH5, 10J §phone design (exposure readout) |
| LV5·C12(c) | `exposure_readout` (core mechanism) | ("1/80", "F8.0", "ISO 2000"); ("2.5\"", "F5.6", "AUTO"); ("BULB", "F11.0", "ISO 100"); ("special", "special", "") | ("1/80", "f/8", "ISO 2000"); ("2.5\"", "f/5.6", "ISO AUTO"); ("BULB", "f/11", "ISO 100"); ("–", "–", "–") | HB5a, P5 |
| LV5·C12(d) | protocol + mapping | a line with `"shutter_speed": ""` and the other two as objects with displays "F8.0", "ISO 100" | readout ("–", "f/8", "ISO 100") | HB5a, P5 |
| LV5·C12(e) | protocol | a line whose `iso` is the number `2000` | unparsable (the runtime's warning, as today); no `Exposure` event | HB5a |
| LV5·C12(f) | fake sidecar → phone | `focus.result {cmd_id, x, y, af: "skipped", point_set: false, refused: "focus_area_not_movable", point_ms: 0, af_ms: 0, total_ms: 3}` for a phone's focus id | the socket receives `{type: "focus", focus_id: <id>, outcome: "refused", reason: "focus_area_not_movable"}` (the message the phone maps to its Focus Area text, LV3·C11(c)) | FC4a, HB5, PH5, LV-D14 |

### Delegated choices
Header-only or `src/liveview.cpp` for `focus_area_movable` · whether `focus.result` gains a `focus_area` number (the
Rust protocol ignores unknown fields; if added, log it in `focus.result focus= outcome= ms=`) · the en dash "–" as a
literal or a constant.

### Notes
- Hazard (FC6, M-LV5): no new `write_camera` call; the refusal path writes nothing. Reviewer checks by reading.
- The sidecar emits `liveview.exposure` each second because the objects' `writable` flag flips (log 14:06:58 vs
  14:06:59); the phone's `camera` message is sent only when the readout changes, so this is log noise, not in scope.
- Rule 17 (external shapes): C11's values are the installed SDK's enum; C12's input is the helper's real line from the
  log; neither fixture is invented.

### Unit test commands
`tools/sidecar/build.sh --stub --test` · `cargo test -p psm-integrations camera::direct::live` ·
`cargo test -p psm-testkit --test suite -- lv5_camera_readouts:: lv1_frame_channel:: lv2_phone_ws::` ·
`cargo clippy -p psm-integrations -p psm-testkit --all-targets -- -D warnings`

## Guide files (flow §7A, in the unit that changes the area)
U1: `docs/guide/02_architecture.md` §The current shot (on-the-way = until the photo exists), `03_rules_and_principles.md`
(MC-LV3 rule wording). U2: `02_architecture.md` §The phone server (which services announce changes; `preview` token),
`04_change_impact.md` (row: a service that changes a session's photos ↔ `SessionChanged` ↔ phone `session`; `PhoneImage`
↔ `apps/desktop/src/phone/api.ts`). U3: `02_architecture.md` §The phone server (stall rule, `PhoneTimings`),
`05_working.md` (nothing if no command changed). U4: `03_rules_and_principles.md` (Focus Area read, never written;
movable set), `04_change_impact.md` (row: the sidecar's `props` display formats ↔ `protocol.rs` ↔ `exposure_readout`).

## Closing gate (once, after the last unit)
`cd apps/desktop && npm run build` (first) · `cargo test --workspace` · `cargo clippy --workspace --all-targets -- -D
warnings` · `cargo fmt --all -- --check` · `tools/sidecar/build.sh --stub --test` · `cd apps/desktop && npm run
typecheck` · `npm test`. One command at a time. A failure is fixed and only the failing files re-run.

## Review mode
The reviewer reads the units in order and runs, once: `cargo test -p psm-testkit --test suite -- lv5_current_shot::
lv5_mac_actions:: lv5_phone_socket:: lv5_camera_readouts:: lv2_current_shot:: lv2_phone_ws:: lv1_frame_channel::` ·
`tools/sidecar/build.sh --stub --test` · `cd apps/desktop && npx vitest run src/features/settings` · clippy on the
touched crates, fmt, typecheck. The implementer's full-suite stamp stands. Only B1–B4 block. Also check, by reading:
no `emit` added in a Tauri command or a phone route (H8); the focus refusal writes nothing and no new
`SetDeviceProperty` intent exists (FC6, M-LV5); an in-flight frame write is never dropped except to close; the
`preview` token carries no path; `lv1_c15b` now uses the real line. Notes go to
`docs/implementation/backlog_live/LV5_live_fixes.md`.

## Manual try-out (the narrative is built from this)
On this Mac with the app built from this branch (packaged with the SDK helper, as LV4's step 1, or `npm run tauri
dev`), the camera on Wi-Fi, both phones paired:
1. Open a templated session. From a phone, take four pictures quickly (about one a second). Watch the Mac's "Next"
   outline: it never jumps onto a shot whose photo is still arriving; each photo lands in its own shot.
2. On the Mac, Deny a photo, move one to Condition, then Undo: each change shows on both phones within a second.
3. Hold the phone on live and walk to the far end of the Wi-Fi: the picture may freeze for a few seconds, then
   continues with the newest frame, without falling back to the template.
4. Set the camera's Focus Area to Wide and tap on the phone's live view: the phone says "Set the camera's Focus Area
   to a movable spot (e.g. Flexible Spot) to focus from the phone"; the camera's setting is unchanged. Set it back to
   Flexible Spot: tapping focuses again.
5. On live, the phone shows the camera's exposure, e.g. "1/80 · f/8 · ISO 2000".
6. Settings → Phone with a phone connected: its dot is green and it reads "Connected now"; close the app on the phone:
   within about 20 seconds it no longer reads connected.
(The phone's dock still uses the old gestures and a tile may still show a broken image until LV6.)

## Review log

- **2026-11-26, implement r1 continuation (codex), IMPLEMENTED.** The orchestrator amended the perimeter to include `p40_undo_history_values.rs` and ruled that PH8 supersedes its old attribute-undo event expectation. The test now checks `SessionChanged` after both undo and redo, retains the two pre-existing `HistoryChanged` expectations for set and undo, checks redo history, and excludes `PhotoChanged`. Its module passed 18/18; the closing gate resumed at the previously failing workspace suite and passed through clippy, fmt (after applying formatter output), sidecar stub, typecheck and desktop tests. Assumption for the named `current_shot.set` and `phone.viewer closed` trace clauses: the existing test harness has no log capture; the tests prove the current-shot state, binding/placement, socket delivery and viewer lifecycle at their observable boundaries. No production deviation from the plan in this continuation. The four unit checkpoints remain `b487c8de`, `f82f0886`, `78018009`, `49ee1c8e`.
- **2026-11-26, implement r1 (codex), BLOCKED after four unit checkpoints.** Built the current-shot hold through a filed-but-not-intaken photo, Mac change announcements and preview versions, stall-tolerant phone socket writes and Settings connection refresh, plus Focus Area refusal and real exposure display parsing. Used the delegated choices of a header-only focus predicate, no extra `focus_area` log field, a literal en dash, and a path/mtime-derived opaque preview token. Full closing gate was run once in order. Desktop build, workspace clippy, format check (after formatting), sidecar stub tests, typecheck, and desktop tests passed. The full Rust suite had three failures: two LV5 fake-sidecar startup races were fixed and their module passed on rerun; `p40_undo_history_values::c7a_attribute_events_only_history` still fails because it forbids `SessionChanged` on attribute undo, while PH8 and Unit 2 require that event for every history step. That test file is outside this plan's perimeter, so the tracker remains `NOT_STARTED`. The plan needs a perimeter amendment for that test and an updated expectation before implementation can close. See `handoffs/implementer/handoff_LV5_live_fixes_implement_r1.md` for stamps, checkpoints, and test coverage limits.
- 2026-11-26, orchestrator (answer to the implementer's BLOCKED): **the perimeter now includes
  `crates/testkit/tests/suite/p40_undo_history_values.rs`.** Master plan §5 PH8 (2026-11-26, F3) supersedes P40 C7(a)'s
  "no `SessionChanged` from the attribute reversal": P40 recorded only that attribute writes emitted nothing at the time
  (P40 plan task 7, D10), not a rule that they must not. Update `c7a_attribute_events_only_history` to expect the same two
  `HistoryChanged` events, **no** `PhotoChanged`, and `SessionChanged { session_id: s }` after each of the undo and the
  redo it exercises (PH8: undo and redo of every step kind announce each touched session); keep the test's name or rename
  it to `c7a_attribute_undo_announces_session` and say so. Re-run that file, then the closing gate from the first failing
  command. Also close the trace gaps the handoff lists where the row's outcome names a log line only if the existing test
  harness can capture log lines; otherwise record the state assertion as the row's proof under Assumptions. Keep the four
  unit checkpoints.
