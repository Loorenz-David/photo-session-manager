---
plan: LV2_current_shot_and_phone_server
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-26
actor: opus
---
# Handoff — LV2 the current shot and the phone server, review r1

**Verdict: APPROVED_WITH_NOTES.** All 101 rows of the five units are made true by the production code and each has a
test at its boundary that would fail on a wrong outcome. The checks of the plan's Review mode are green. H1–H8 hold. No
B1–B4 finding. Seven notes go to `docs/implementation/backlog_live/LV2_current_shot_and_phone_server.md`; the most
useful one for LV3 is that a phone Delete sends no change event. Tracker → `APPROVED`; narrative written.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Gate
- Tracker line read `IMPLEMENTED (2026-11-26, opus)`.
- Checkpoints U1 `7a685d76`, U2 `18cdd4f2`, U3 `84d94253`, U4 `d3db9091`, U5 `1463728a`, close `b5cc3336`: all in
  `git log`. Tree at review: `7979f449`, `git status --porcelain` clean.

## Perimeter (step 1)
`git show --stat` of each checkpoint equals the handoff's per-unit "Files changed" lists, including the declared
deviations: `t1_settings_drawer.test.tsx`; `phone_view.rs` helpers and `template_placement::container_final_no`;
`live_view.rs::viewer_count`; desktop `commands/{sessions,templates,settings}.rs`; `test_app.rs`;
`p01_storage_identity.rs` in the close commit. The U1 checkpoint dropped five t4 tests and U4 restored them, as declared.
I compared them with the parent of U1. Each restored test is unchanged and passes. No extra or missing path.

## Checks run (step 2, Review mode, once each, one cargo command at a time)
| Command | Result |
|---|---|
| `cargo test -p psm-testkit --test suite -- lv2_current_shot:: lv2_phone_ops:: lv2_phone_pairing:: lv2_phone_http:: lv2_phone_ws:: t2_template_placement:: t3_template_hand:: t4_template_retake_shift::` | 143 passed, 0 failed (14.9 s) |
| `cargo test -p psm-desktop` | 126 passed (incl. `lv2_c15a`, `lv2_c15b`) |
| `cd apps/desktop && npx vitest run src/features/settings` | 129 passed |
| `npm run typecheck` | clean |
| `cargo clippy -p psm-core -p psm-infra -p psm-integrations -p psm-testkit -p psm-desktop --all-targets -- -D warnings` | clean |
| `cargo fmt --all -- --check` | clean |
The implementer's full-suite stamp stands (tree = U5 + close, unchanged since).

## Verified correct
**U1 — the current shot (C1–C8).**
- `CurrentShots` matches CS1: in memory only and shared with the camera runtime, so it survives a settings rebuild.
  `resolve` advances a held shot that is taken or has left the template. A held `Free` is returned as it is. Another
  session's value is its default, and nothing held changes.
- `taken` = Main photos whose place-row slot is a container's final number, plus the bindings of hold-occupied slots
  (CS2/CS4).
- `next_untaken` is CS4 exactly: after `from`, then wrap, else `None`. `None` or an unknown `from` starts at the
  beginning.
- `slot_owners` zips the time-sorted owners with `order`'s output. That is correct because `order` only permutes
  entries among the time-sorted positions. `order`'s own results are unchanged (t2 rule-1 tests green).
- CS5: `LedgerService::exposure_observed` holds the ledger worker lock. `bind_exposure` resolves, inserts the row with
  its binding and runs `match_late_exposure`, all in one store transaction. A late match takes the file's binding with
  no second advance (C7(b)). Lock order is store transaction → held value everywhere, so there is no inversion with
  arrival.
- CS6: `place_arrival` binds a photo only when it has no slot owner and no place-row binding, once (C3(c)).
- CS3 placement: claimed and hand slots; a photo already in its container stays; a binding to a removed container →
  Condition (C8(d)).
- CS7: `set` refusals in the order SessionChanged → NoTemplate → NotInTemplate → ShotFilled. `retake` denies, releases,
  makes one "Retake shot <n>" step, then holds the container. `ActiveSessionChanged` is emitted by `store_target` on a
  real change, and `CurrentShotObserver` clears the held value (C6(a)).
- CS8: the drop retake binds the place row; `shift_plan` follows the rank formula; Back to template uses bindings; the
  refill is gone (C8(e)). The Mac's `next_shot` comes from `session_view_with_current`.
- Every rewritten t2/t3/t4 test is listed in the handoff with the 10G rule it encoded and now asserts the MC-LV3
  outcome. Test counts per file are unchanged.

**U2 — phone operations (C9–C12).**
- `phone_view`: shots in shooting order with ratio/split/zoom/photo, extras, Condition, counts, current;
  `article_number` = barcode; `null` without an active session.
- `move_to` covers the five targets as swaps:
  - Main↔Main, Main↔Condition and Condition↔Condition swap.
  - `ExtraEnd` and `ConditionEnd` append.
  - Every photo that moves gets `place_by_hand = 1`. Compositions are untouched. One "Move image" step.
  - In an untemplated session, `Container` answers `no_template`.
- `shot_ratio` follows the PH6 table.

**U3 — pairing and settings (C13–C16).**
- Token: 32 bytes from `getrandom`, unpadded base64url of 43 characters. Only its lowercase hex SHA-256 is stored. The
  C13(a) test scans every text column of the database for it.
- `verify` requires `revoked_at IS NULL` and updates `last_seen_at`.
- The `phone` setting is in the allowlist and is nulled in the rebuild comparison, so a save is `Simple`. A simple save
  refreshes `App.settings`, so the next QR code uses the new port.
- The port rule reads the stored FTP control port and passive range.
- The desktop commands are thin; `phone_pairing_revoke` revokes and then closes that phone's sockets.

**U4 — HTTP (C17–C23).**
- Peer filter (loopback plus MC-32 private ranges) wraps every route.
- Auth runs before any body parse or service call.
- Static files reject every non-`Normal` path component and check the canonical path lies under the root. C18(e)
  discriminates: the `secret` file really is at `web/assets/../../secret`.
- Every POST runs `guard` (active session) before acting. Move and Delete also check that the photo belongs to the
  session.
- Handlers decide nothing (H8): the container-id → final-number lookup and the preview/example choice live in core.
- `phone.request` lines name the route template and the pairing id, never the token. No tracing layer logs URIs.

**U5 — WebSocket (C24–C29).**
- The token is checked before the upgrade (401).
- `offer`, `event` and `withdraw` only put into the slot or queue and call `notify_one`; nothing under the hub mutex
  awaits or writes.
- The writer drops a frame whose generation is not the current one.
- Every write has `write_timeout`. Pings run, and the socket closes 4408 when idle, 4408 on a slow write, 4409 on
  queue overflow (64) and 4401 on revoke.
- Unsubscribe happens on every exit path.
- The frame header matches PH5 byte for byte (C29).

**Tests.** I read each test against its row. Every one observes the row's outcome at the row's boundary and would fail on
a wrong outcome. Examples:
- C1(b) would read `Shot(B)` if holds were not counted.
- C7(b) would bind A without the late match.
- C25(a) would receive frames if demand were ignored.
- C26(a) would get more than 2 frames, or a stale last frame, without the `LatestSlot`.

No test asserts internals, and no orphan test exists (every test traces to a row or to a listed core mechanism).

**Assumptions (step 4).** None contradicts the intention.
- Assumption 4 (a held `Free` stays `Free` until the next shutter) is CS1 read literally. It is consistent with MC-LV3
  ("refill gone") and with C8(e).
- Assumption 12 (sockets miss App events after a rebuild until the next request) is a real limit. It is recorded as a
  note for LV3.
- The others are plain choices and are listed in the narrative.

**Constraints H1–H8 (step 5).**
- H1: no external write.
- H2: Retake and Delete deny as before; the RAW goes only when the undo window closes.
- H3: JPEG bytes and example files are sent unchanged.
- H4: the binding is written in the insert's transaction and only once.
- H5: no UI automation.
- H6: nothing depends on commerce.
- H7: no Sony or Capture One concept in core; the pairing table is the app's own.
- H8: handlers and commands are thin.
- No secret or token in a log line or a column. No live service in any test.

## Findings
None blocking (B1–B4).

## Notes written to backlog
`docs/implementation/backlog_live/LV2_current_shot_and_phone_server.md` (the plan's named notes file):
1. A phone Delete emits no `SessionChanged`/`PhotoChanged`. The Mac workspace and other phones stay stale.
   Destination: LV3.
2. Deleting the active session emits no `ActiveSessionChanged` (`session_delete.rs` has no `with_events`).
   Destination: LV3.
3. After a rebuilding settings save, sockets get no session or camera messages until the phone's next request.
   Destination: LV3.
4. The revoke command's two calls are tested only in halves. Destination: LV4 / next `commands/phone.rs` touch.
5. Error bodies carry internal text and local paths to the phone. Destination: LV3.
6. `psm-desktop` needs `apps/desktop/dist/`, but §8's gate runs cargo first. Destination: orchestrator.
7. The LV1 notes addressed to LV2 were not in LV2's plan and are still open. Destination: orchestrator.

### Carry-forward dispositions
| Note | Destination |
|---|---|
| 1 phone Delete event | LV3 plan (add to C21(e)'s outcome) |
| 2 session delete without `ActiveSessionChanged` | LV3 plan |
| 3 sockets after an App rebuild | LV3 plan |
| 4 revoke wiring untested end to end | LV4 hardware check |
| 5 internal error text to the phone | LV3 plan |
| 6 `dist/` before cargo | orchestrator, `master_plan_live.md` §8 |
| 7 LV1 notes addressed to LV2 | orchestrator: next plan touching `frames.rs` / fake sidecar / sidecar, else LV4 |

## Lessons for the plans
- A row that reuses an existing service from a new surface ("phone Delete = `ReviewService::delete`") should also name
  the events the other surfaces depend on. The Mac learns of changes only through events, so a service whose callers
  relied on the response body is silent to every other screen.
- When a milestone's predecessor leaves notes addressed to it, the plan should list them, even as "not taken up" with a
  reason. Otherwise they lapse silently (note 7).
- The closing-gate order in `master_plan_live.md` §8 should put `npm run build` before the cargo commands now that the
  desktop crate depends on `dist/`.

## Write perimeter
- `docs/implementation/handoffs/reviewer/handoff_LV2_current_shot_and_phone_server_review_r1.md` (this file, new)
- `docs/implementation/backlog_live/LV2_current_shot_and_phone_server.md` (new)
- `docs/implementation/narratives/LV2_current_shot_and_phone_server.md` (new)
- `docs/implementation/plans/LV2_current_shot_and_phone_server.md` (Tracker line; Review log entry appended)
- No production or test file touched; no probe applied (flow §4: no mutation regime). Build artefacts only under
  `target/` from the checks above. No database or settings state outside test temp dirs.
