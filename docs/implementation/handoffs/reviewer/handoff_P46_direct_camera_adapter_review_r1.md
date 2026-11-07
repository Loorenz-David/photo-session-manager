---
plan: P46_direct_camera_adapter
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-07
actor: opus (pipeline-reviewer)
---
# Review: P46 direct camera adapter, round 1

**Verdict: APPROVED_WITH_NOTES.** No B1–B4 finding. The production code satisfies all 27 rows. Every row's test observes its outcome at
the row's boundary and would fail on a wrong outcome. The suite, lint and typecheck are green on the reviewed tree, and no assumption
contradicts 10E. 6 notes went to the backlog.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Gate
- Tracker row was `IMPLEMENTED`. Checkpoint `64aa4f8` (`CHECKPOINT (not approved): P46_direct_camera_adapter implement r1`) is in `git log`.
- Reviewed tree: `05cf4f8`. It equals `64aa4f8` plus runner files only (the implementer handoff and this review's prompt). No code drift.
  The tree was clean before my edits.

## Verified correct
- **Perimeter.** `git show --stat 64aa4f8` lists 38 paths, which equals the handoff's declaration:
  - the plan's 34 paths: 3 new integrations files, 5 changed integrations, 5 core, 5 desktop, 4 native, 2 testkit, 9 scripts, 1 new test;
  - 2 disclosed test lines: `p45` C2(a) `20 → 21 tests` (the plan's new C++ case), and `p42` C7(a) on `with_fake_document()` (from this
    phase a support folder selects the direct adapter, as the plan's regression note says; the row's meaning, the Capture One camera's
    health copied across a rebuild, is kept);
  - the tracker and the plan's Review log.

  No path is missing and none is undeclared.
- **Suite (one reviewer run, code tree `64aa4f8`).** All five commands exited 0:
  - `cargo test --workspace`: 723 passed, 0 failed. That includes `p46_direct_camera_adapter` 25/25, desktop `p46_tests` 2/2, the direct
    unit tests, and P45 `c2a`, which runs `tools/sidecar/build.sh --stub --test` and asserts `21 tests, 0 failed`.
  - `cargo clippy --workspace --all-targets -- -D warnings`.
  - `cargo fmt --all -- --check`.
  - `npm run typecheck`.
  - `npm test`: 50 files, 369/369.
- **Adapter, read against plan §The adapter contract and 10E R-I19…R-I23, R-I37:**
  - `TransportState`:
    - DISCOVERING takes its link from the `over usb`/`over wifi` suffix; any other reason changes nothing.
    - `CONNECTING_*` sets the link, and `CONNECTED_*` clears `connecting`.
    - `SWITCHING` sets busy `Switching`.
    - `DISCONNECTED`/`RECOVERING`/`ERROR` clear `connecting` and `busy`.
  - `ConnectionReady`:
    - sets the session to (model or `Camera`, link) and clears `connecting` and `busy`;
    - `Failed` adds `StoreDestinationUnverified`, and `Verified` removes it;
    - on USB it removes `HeldByOtherProgram`; on Wi-Fi it removes `AuthenticationRefused` and `FingerprintMismatch`;
    - with transport `none` it changes nothing.
  - `SessionClosed` keeps `busy` only when it is `Switching`. `SessionLost` clears it. Both drop the expected transfer and tell the sink
    `transfer_in_progress(Cable, false)` (assumption 10; the ledger's handler is a no-op).
  - `CameraBusy`:
    - `exposure` → `Exposure`, `file_transfer` → `Transfer`, `none` → no busy;
    - the other reasons change nothing.
  - `CaptureExposed`:
    - `capture_id` is filtered out when the association starts with `none`, and `initiated` is decided by the same test;
    - `provisional` is true when the association is `latest_release_provisional`, and `at` is the envelope stamp;
    - on USB it sets the expected transfer and busy `Transfer`, and calls `transfer_in_progress(Cable, true)`.
  - `UsbClaimedBy` → `holder_from_owners`.
  - `SshFingerprint`: `unknown` adds `FingerprintUnconfirmed`, `MISMATCH` adds `FingerprintMismatch`, `match`/`pinned` remove both.
  - `ConnectFailed` → `needs_action_from_connect_failed`.
  - `SdkUnavailable` → the flag, plus the health set in `perform` after the runtime's per-batch clear. The sidecar exits 4 right after this
    line (`main.cpp:594–596`), so the item stays put across restarts.
  - Supervisor `Started` clears `sdk_unavailable` and the two refusals. `Exited` resets session, connecting, busy and the expected transfer.
  - `capture()` refuses `SidecarUnavailable` first, then `CameraNotConnected` (no session, or the helper is not running). It never consults
    Capture One health: `CameraService` gates only when `depends_on_editing_provider`.
  - `reconnect()`: Wi-Fi sends `Reconnect`, cable answers `ReconnectOnCableRefused`, no session answers `CameraNotConnected`.
  - Capabilities and `raw_extensions` are as planned.
- **Save-to-PC.**
  - Refusal order: `!success` → `size 0` → outside staging → missing. The staging folder is the canonical `--save-dir` of the first spawn.
    A refused download only pushes `download ignored: <base> (<reason>)` to the helper log, and the file is never touched (H2).
  - An accepted download: `fs::metadata` size, then `sha256_file`, then `Delivery { camera_filename: normalize_camera_filename(base),
    via: Cable, completed_at: envelope }`.
  - The `≥ TRANSFER_EXPECTED_MS` deadline is checked after each batch.
  - `normalize_camera_filename` strips only `_<1–2 digits>` after a digit-ending stem.
- **Runtime.**
  - One `client.poll` per tick.
  - The supervisor lock is released before the adapter step, and ticks are serialised by `tick_serial`. A hash never blocks `capture()`,
    and the step holds no lock the sink needs.
  - `bound_direct` compares data pointers. `bind` force-publishes the owned codes after P42's copy.
  - Owned codes are set or cleared from `health_items` on every change, never `camera_files_not_received`, `camera_photo_session_uncertain`
    or `camera_sidecar_unavailable`.
  - Watchdog glue:
    - runs once per mono second with `uploads_in_progress() > 0 || transfer_expected()`, read on every call;
    - on `first_raise` it calls `provider().request_ftp_status()`;
    - the `FilesNotReceived` input mirrors `LedgerShared.raised` on every tick.
  - The ledger stays the single writer of `camera_files_not_received`. The ` · camera FTP: …` suffix lives in `LedgerShared.ftp_note`
    (survives a save) and is cleared by `dismiss()` and by a non-raising run.
- **Selection and capture root.** `App::build` decides `support::installed(&paths)` up front. The same `direct_adapter()` `Arc` goes to
  `camera`, `ArrivalService`, the binding and the handles, and it is created before `bind`'s first spawn, so the staging folder is set. The
  arrival closure is `canon(effective_document_id/Capture)`, the same document as the receiver's `capture_root_fn` (binding
  `capture_root = effective_document_id`, `lib.rs:753`). The cache rule is unchanged.
- **C++.**
  - `reconnect()` queues an input. `drain_inputs` arms it only when a Wi-Fi session is open.
  - `step()` waits for busy `none`, then calls `intentional_disconnect("reconnect requested")` and `try_connect(Wifi)`.
  - The flag is reset on session loss and on intentional disconnect.
  - `main.cpp` rejects on USB with `reconnect only on wifi`, after the existing `not connected` rejection.
- **Commands.**
  - `camera_capture` returns `CaptureResponse { capture_id }`.
  - `camera_reconnect` and `camera_files_not_received_dismiss` are thin. Dismiss takes the ledger from `camera_runtime.ledger()`.
  - Error codes are mapped in `items.rs`, the bindings are regenerated, and `services/camera.ts` returns the id (no existing caller).
- **Rows → tests** (each test read against the production path above; "fails if" = the wrong outcome it would catch):

  | Row | Test | Fails if |
  |---|---|---|
  | C1(a) | `c1a_status_from_usb_session` | the DISCOVERING link were missing (`ev()` tail), a wrong name/link were set, or health were stale (health is read before any `CameraService::status()`) |
  | C1(b) | `c1b_status_from_wifi_session` | the link were wrong |
  | C1(c) | `c1c_…` | RECOVERING→DISCOVERING `over wifi` were not `Connecting { Wireless }`, the `Exited` reset were missing, or capture were accepted after the exit; `pids == 1` proves no replay |
  | C1(d) | `c1d_…` | `session.closed` dropped `Switching`, or any intermediate line emitted `Connecting { Cable }` |
  | C2(a) | `c2a_…` | the wire line were not sent exactly once, the id or `exposed_at` came from the wrong source, or the `Exposure`/`Transfer` busy states were wrong |
  | C2(b) | `c2b_…` | a camera-initiated shot kept an id, were marked `app`, or set busy on Wi-Fi |
  | C2(c) | `c2c_…` | capture were sent with no session (stdin compared whole) |
  | C2(d) | unit `c2d_sony_resend_suffix` | any of the six input → output pairs differed |
  | C3(a) | `c3a_…` | the delivery had the wrong via, hash, size, association, outcome or `completed_at`, or were left in staging, or the photo's `captured_at` were wrong |
  | C3(b) | `c3b_…` | the completion did not end `Transfer`, or the deadline were `>` instead of `≥` (9 999 ms still busy, then 1 ms clears it) |
  | C3(c) | `c3c_…` | a zero-size or missing file were promoted, or the helper-log lines differed (compared exactly) |
  | C3(d) | `c3d_…` | `_1` were not normalised (the delivery would be a new photo, not a duplicate under `.duplicates/DSC00008.ARW`) |
  | C4(a) | `c4a_…` | `failed` did not set the reason, or the session were dropped (`camera` ok item; capture sent) |
  | C4(b) | `c4b_…` | `verified` did not clear it |
  | C4(c) | unit + `c4c_authentication_refused_then_cleared_by_wifi_session` | a text mapping were wrong, or a Wi-Fi ready did not clear the refusal |
  | C4(d) | `c4d_…` | `pinned` did not clear `FingerprintUnconfirmed` |
  | C5(a)/(b) | `c5a_…`/`c5b_…` | the first copy were not the photo in either order (real receiver, real ledger) |
  | C6(a) | `c6a_…` | `ftp-status` were sent zero times or more than once, or the status or detail were wrong |
  | C6(b) | `c6b_…` | an upload in flight did not defer the raise |
  | C6(c) | `c6c_dismiss_clears_the_notice` + desktop `c6c_dismiss_command_clears_the_notice` | dismiss did not mark `dismissed`, or did not clear the status or the item |
  | C6(d) | `c6d_…` + C++ case | the per-link answers were wrong (stdin counted per link); the C++ case is executed by P45 `c2a` in the workspace suite |
  | C6(e) | `c6e_…` | the suffix format or ordering differed, or the note survived a dismiss |
  | C7(a) | `c7a_holder_reported` + unit | the holder were parsed wrongly |
  | C8(a) | `c8a_adapter_selection` | the selection were inverted (capabilities, status source, pid count) |
  | C8(b) | desktop `c8b_…` | the command dropped the id, the exposure row did not carry it, or reconnect on cable did not answer `reconnect_on_cable_refused` |
  | C8(c) | `c8c_…` | the direct build asked `document_info`, or the Capture One build did not |

  On C8(c): the disclosed input change, reopening with `restart(false)` after `ops.clear()`, means no startup scan fills the cache, and
  that is what lets the row fail. The fake records `document_info` before it can error (`fake_capture_one.rs:366`).
- **Assumptions 1–12.** None contradicts 10E.
  - 1 follows R-I37: a `NeedsAction` "wins even while connected on the other link", and planner reading 4.
  - 2 follows MC-33: "An SDK reconnect at a safe point".
  - 3–12 are within the plan's contract or its Delegated choices.

  All are reported to the owner in the narrative.
- **H1–H8.**
  - H1, H6: no platform write.
  - H2: a refused download is left in place, nothing is deleted, and the rename into the session folder is the ledger's (P44).
  - H3: no pixels.
  - H4: the delivery path is P44's pending-then-done ledger.
  - H5: no UI automation.
  - H7: core gains only neutral errors, default port methods and an opaque note string; the port method name is from §6.11.
  - H8: commands are thin.
- **Secrets and live services.** Test credentials are fakes. No live service in the suite (the fake sidecar and the fake Capture One).

## Findings
None blocking. No should-fix.

## Notes written to backlog
1. **P46 → coordinator / book 10E.** Every Save-to-PC download is reported `via: Cable`, as the plan and R-I20 say. R-I19 says a Wi-Fi
   download in staging is `via: Wireless`. Nothing branches on `via` today.
2. **P46 → P48.** `ftp-status` re-fires after each transfer gap, because the open P44 note still holds (the watchdog resets `raised`
   while a transfer runs).
3. **P46 → P50/P51.** Two texts for `camera_sidecar_unavailable`. The P42 failed-spawn note is still open.
4. **P46 → P48.** Assumption 1 consequence: the held notice persists on Wi-Fi after the holder quits, until a USB session opens.
5. **P46 → planner (lesson).** C8(c) as written could not fail, because of the cached capture root.
6. **P46 → P47.** An unknown `store_destination` still drops `connection.ready` (P45 note). From this phase that line drives Connected.

## Carry-forward dispositions
| Open note | Destination |
|---|---|
| `via` on a Wi-Fi Save-to-PC download | coordinator (book wording) |
| `ftp-status` re-fires / notice flicker across transfers (P44 + P46) | P48 |
| held notice clear trigger | P48 |
| `camera_sidecar_unavailable` texts; failed first spawn never retried (P42) | P50/P51 |
| unknown `store_destination` drops `connection.ready` (P45) | P47 (before the spike relies on it) |
| `SidecarClient.outcomes` unbounded (P45) | P48 (unchanged by P46) |
| accept-loop break, FTP idle timeouts (P43) | P48 (unchanged by P46) |

## Lessons for the plans
- A row that asserts "provider X is never asked" through a service with a cache must state that the cache starts fresh (C8(c)).
- 10E R-I19 and R-I20 disagree on `via` for a Wi-Fi Save-to-PC download. The plan followed R-I20.

## Write perimeter and probes
- Written: `docs/implementation/master_plan.md` (P46 tracker row only), `docs/implementation/plans/P46_direct_camera_adapter.md` (Review
  log entry), `docs/implementation/backlog.md` (6 rows appended), `docs/implementation/narratives/P46_direct_camera_adapter.md` (new), and
  this handoff.
- No production or test file was edited, not even temporarily, and no mutation probe was run (flow §4).
- Side effects: build outputs under `target/`, `apps/desktop/src-tauri/target/` and the gitignored stub sidecar binary rebuilt by P45 `c2a`.
  Test temp directories only. Nothing committed (the runner commits).
