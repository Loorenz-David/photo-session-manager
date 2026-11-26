# Backlog — LV2_current_shot_and_phone_server (reviewer notes; none blocks)

- **LV2 reviewer r1 (note): a phone Delete sends no change event.** `routes::delete_photo` calls
  `ReviewService::delete`, which records an undo step (`history_changed`) but emits no `SessionChanged`/`PhotoChanged`.
  The Mac's workspace reloads only on `session_changed`/`photo_added`/`photo_changed`, and the phone sockets forward
  only those. Result: after a Delete on the phone, the Mac keeps showing the photo in its old place until some other
  change refreshes it, and a second phone gets no `session` message. Retake, Move and Current all go through
  `HandAction::finish` / `CurrentShots::set` and do emit. Why: the Mac view is stale after a phone action; LV3's dock on a
  second phone too. Destination: **LV3** (emit `SessionChanged` + `PhotoChanged` after the phone delete, in a core
  wrapper, not in the handler; add the event to LV2·C21(e)'s outcome). Plan lesson: "Delete = `ReviewService::delete`"
  named the service but not the event the other surfaces depend on.
- **LV2 reviewer r1 (note): deleting the active session does not announce the switch.** `session_delete::delete` builds
  `SessionsService` without `with_events`, so `store_target(None, …)` emits no `ActiveSessionChanged`. Phones are not
  sent `{type: "session", session_id: null}` and the server's `Feed.active` keeps the deleted id until the next request
  re-reads it. The held current shot is not cleared either (harmless: the next activation clears it). Why: a phone keeps
  showing a session that no longer exists; its POSTs answer 409 `session_changed`. Destination: **LV3** (one line:
  `.with_events(app.events.as_ref())` in `crates/integrations/src/session_delete.rs`).
- **LV2 reviewer r1 (note, assumption 12): after a settings save that rebuilds the App, open phone sockets get no
  `session` or camera-status messages** until the phone makes an HTTP request or reconnects (`ensure_fed` observes the
  new App's event hub only then). Why: a phone left on the dock would not see new photos arrive after, e.g., a camera
  settings save. Destination: **LV3** (observe the new hub at rebuild, e.g. from `apply_settings` / `transfer_target_to`,
  or have the phone app re-read `GET /session` on a timer).
- **LV2 reviewer r1 (note): the revoke wiring is tested in two halves.** LV2·C28(c) calls `PhonePairings::revoke` and
  then `PhoneServer::close_pairing` itself; the desktop command `phone_pairing_revoke` that does both in production has
  no test of its own (read: correct). Why: if the command lost its `close_pairing` call, a removed phone would keep its
  live socket until it reconnected, and no test would notice. Destination: LV4 hardware check (Remove a connected phone)
  or the next touch of `commands/phone.rs`.
- **LV2 reviewer r1 (note): error bodies carry the internal error text.** `wire::error_body` puts `error.to_string()`
  into `message` for every status, including 500 `internal` and 404 (`NotFound(<local file path>)` for a missing
  preview). Only paired phones see it. Why: local paths and internal messages reach the phone. Destination: LV3 (a
  generic message for 500 and for file 404s).
- **LV2 reviewer r1 (note, environment): `psm-desktop` does not compile without `apps/desktop/dist/`** (the
  `bundle.resources` mapping). The guide (`05_working.md`) says so, but `master_plan_live.md` §8's closing gate lists
  `cargo test --workspace` first and `npm run build` last. On a fresh checkout or CI the first cargo command fails.
  Destination: orchestrator (§8: run `npm run build` before the cargo commands), LV4 packaging.
- **LV2 reviewer r1 (note, carried from LV1): the LV1 notes whose destination was "LV2" were not in LV2's plan and are
  still open**: `psm-lv-*.sock` files left in `$TMPDIR` by test apps; the fake sidecar's frame-channel connect timing
  (FC1) and reconnect-after-failed-write; the lost frames-flowing restart row (C13(e)); SIGINT on diagnostic sidecar
  commands; dead `previous_connections` / `connection_count()`. LV2 touched neither `frames.rs`, the fake sidecar nor
  the sidecar. Destination: orchestrator (fold into the next plan that touches `frames.rs`, the fake sidecar or the
  sidecar; otherwise LV4).
- **Orchestrator (2026-11-26):** notes 1, 2, 3 and 5 (phone Delete events, session delete announce, rebuild observe,
  generic error bodies) are folded into `LV3_phone_app.md` as **Unit 0** (rows LV3·C26–C29). The environment note:
  `master_plan_live.md` §8 closing gate now runs `npm run build` first. The revoke-command note and the LV1 carry-overs
  (sockets left in `$TMPDIR`, fake connect/reconnect fidelity, C13(e), SIGINT on diagnostics, dead
  `previous_connections`) stay open for LV4 or the next plan touching those files.
