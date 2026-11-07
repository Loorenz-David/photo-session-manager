---
plan: P46_direct_camera_adapter
role: implement
round: 1
state: DONE
date: 2026-11-07
actor: claude (opus, pipeline-implementer)
---
# Handoff — P46 direct camera adapter, implement r1

Gate: intention RATIFIED; P45_sidecar_and_protocol APPROVED; P46 was PROJECTED (not APPROVED). Passed.

## What was built
- **Session Manager can drive the camera itself.** When a camera support folder is installed, the app now uses the new direct camera adapter
  instead of asking Capture One about the camera. Without a support folder nothing changes: the Capture One camera adapter is used as before.
  The choice is made once per app build (`App::build` → `support::installed(paths)`), and the adapter is created once per app process
  (`CameraRuntime::direct_adapter()`), so its view of the camera survives a settings save.
- **The camera dot follows the camera helper's reports.** `Connecting` (with the link it is looking on), `Connected { name, link }` (the model the
  camera reports, e.g. `ILCE-7RM5 · cable`), `Busy` (taking a photo, receiving a photo over the cable, switching between cable and Wi-Fi — held through
  the whole switch), `NeedsAction` (password refused, fingerprint to confirm or changed, another program holding the USB connection with its name,
  store destination not confirmed, photos not received) and `Disconnected` (also at once when the helper dies). Every change is emitted as
  `camera_status_changed`; the Health page items the adapter owns are re-published from its own state on every change and at every settings save.
- **Shots are recorded the moment the camera reports them**, app-initiated (with the capture id `camera_capture` now answers) and camera-initiated
  (the camera's own shutter), stamped with the camera helper's time, in the session active at that moment.
- **Photos over the cable become deliveries.** A completed Save-to-PC download inside this run's staging folder is re-read, hashed and handed to the
  capture ledger as a cable delivery; Sony's re-send suffix (`DSC01234_1.ARW`) is stripped first. A failed, empty, outside-staging or missing
  download is never promoted and never deleted; the helper log says `download ignored: <name> (<reason>)`. **First copy wins** end to end: the same
  photo by cable and by FTP becomes one photo and one `.duplicates/` file, in either order.
- **Watchdog glue.** Once per second the ledger's watchdog runs with "transfer active" = an FTP upload in flight or a cable download expected. On the
  first raise the app asks the camera for its FTP state (`ftp-status`); the camera's answer is shown at the end of the "photos taken but not
  received" item (` · camera FTP: key=value, …`). Dismissing the item (`camera_files_not_received_dismiss`) marks those shots dismissed.
- **Reconnect, Wi-Fi only.** `camera_reconnect` sends the new helper command `reconnect`, which closes the Wi-Fi session and reopens it once (C++
  `TransportManager::reconnect()`, at the next safe point); on a cable session the app refuses (`reconnect_on_cable_refused`) and so does the helper
  (`cmd.rejected {reason:"reconnect only on wifi"}`).
- **With the direct adapter, new photos are attributed without Capture One**: the arrival path takes the capture folder from the settings
  (`<document>/Capture`), the same formula as the FTP receiver (R-I28).
- **Test harness.** The fake helper gained an inject channel (`TestApp::inject`, `inject_all`, `inject_exit`, `sdk_download`), `TestApp::capture()`,
  `camera_status()`, `staging_dir()`, `SCRIPT_EPOCH_MS` (scripted apps start their clock on the scripts' day), and 9 new scripts.

## Tests → criterion rows
Behaviour tests in `crates/testkit/tests/p46_direct_camera_adapter.rs` unless noted.
| Row | Test |
|---|---|
| C1(a) | `c1a_status_from_usb_session` (health read before any `CameraService::status()` call, so the adapter's publish is what is asserted) |
| C1(b) | `c1b_status_from_wifi_session` |
| C1(c) | `c1c_session_lost_connecting_then_sidecar_exit_disconnected` |
| C1(d) | `c1d_switching_is_busy_across_the_switch` (also: no `Connecting { Cable }` emitted during the switch) |
| C2(a) | `c2a_capture_returns_id_and_records_exposure` |
| C2(b) | `c2b_camera_initiated_shot` |
| C2(c) | `c2c_capture_refused_without_session` |
| C2(d) | unit `camera::direct::filenames::tests::c2d_sony_resend_suffix` (Core mechanism) |
| C3(a) | `c3a_save_to_pc_completion_becomes_cable_delivery` |
| C3(b) | `c3b_busy_transfer_until_completion_or_10s` ((ii) ticks 9 999 ms → still `Transfer`, then 1 ms → `Connected`) |
| C3(c) | `c3c_zero_or_missing_download_never_promoted` |
| C3(d) | `c3d_resend_suffix_by_cable_keys_to_original` |
| C4(a) | `c4a_store_destination_failed_keeps_session` |
| C4(b) | `c4b_verified_clears_store_destination` |
| C4(c) | unit `camera::direct::adapter::tests::c4c_connect_failed_texts` + boundary `c4c_authentication_refused_then_cleared_by_wifi_session` |
| C4(d) | `c4d_fingerprint_unknown_until_pinned` |
| C5(a) | `c5a_first_copy_wins_cable_then_ftp` (real receiver, real ledger) |
| C5(b) | `c5b_first_copy_wins_ftp_then_cable` |
| C6(a) | `c6a_watchdog_raise_sends_ftp_status_once` |
| C6(b) | `c6b_transfer_in_progress_defers_the_raise` |
| C6(c) | `c6c_dismiss_clears_the_notice` + desktop `p46_tests::c6c_dismiss_command_clears_the_notice` (mock webview: command `Ok`, `health_get` no longer lists the code) |
| C6(d) | `c6d_reconnect_only_on_wifi` + C++ `TEST_CASE("reconnect reopens the wifi session once")` run by `tools/sidecar/build.sh --stub --test` |
| C6(e) | `c6e_camera_ftp_report_reaches_health` |
| C7(a) | `c7a_holder_reported` + unit `camera::direct::adapter::tests::c7a_holder_shapes` |
| C8(a) | `c8a_adapter_selection` |
| C8(b) | desktop `p46_tests::c8b_camera_capture_answers_the_capture_id` (mock webview) |
| C8(c) | `c8c_direct_arrival_capture_root_from_settings` (see Deviations; checked to fail with the capture-root closure removed) |

## Assumptions
1. **`HeldByOtherProgram` clears only on a USB `connection.ready`.** The event table lists the clear under `ConnectionReady` with the reason "the
   interface is ours now"; planner reading (4) says "our own `connection.ready usb`", and R-I37 says a `NeedsAction` "wins even while connected on the
   other link". A Wi-Fi session therefore keeps the "held" notice.
2. **The C++ reconnect waits for the safe point** (busy `none`) before closing the Wi-Fi session (10E MC-33: "An SDK reconnect at a safe point"); a
   request is dropped if the session closes or is lost before it runs. On USB `TransportManager::reconnect()` ignores the request (main.cpp rejects first).
3. **`capture()` is also refused while the helper is not running** (`CameraNotConnected`); the `Exited` reset already clears the session, so this only
   covers the moment before the first tick.
4. **`bind` keeps P42's copy of camera-family items** from the previous health service, then the direct adapter re-publishes its owned codes on top.
   So the ledger's and runtime's own items carry across a save and the adapter's codes come from its inputs (§6.11 Runtime (a)).
5. **The watchdog glue runs only while the direct adapter is the bound provider**, the same as the receiver and the sidecar, which run only with a
   support folder.
6. **`CameraRuntime::provider()` returns `Option`**: there is no provider before the first bind.
7. **`CameraService::dismiss_files_not_received` without a ledger** answers `provider` ("camera ledger not bound"); the command takes the ledger from
   `camera_runtime.ledger()` (delegated choice: `with_ledger(&LedgerService)`).
8. **FTP note truncation**: the whole short form is at most 160 characters (159 + `…`).
9. **Staging comparison is canonical** (macOS `/var` ↔ `/private/var`): the staging folder and the download's folder are resolved before
   `starts_with`; the `--save-dir` argument is unchanged.
10. **`SessionClosed`/`SessionLost` also tell the sink `transfer_in_progress(Cable, false)`** when a download was expected (the ledger's is a no-op).
11. **Test harness**: `TestApp::inject_all` (several lines in one write, for the rows that say "inject (one write)"); the fake's inject tail reads
    lines appended since the process started and runs them once the script ends (20 ms cadence); test settles are bounded at 10 s (P45's), not 3 s.
12. **Delegated choices taken**: `Weak<CameraRuntime>` for `send_command`; one `Mutex<AdapterState>`; the arrival capture-root closure injected by
    `App::build` (`ArrivalService::with_capture_root`); `uuid::Uuid::new_v4()` in the adapter; helper-log wording other than `download ignored: …`
    unchanged (none added).

## Deviations
1. **C8(c) input**: both halves reopen the app (`kill_and_restart`) after `ops.clear()`. `ArrivalService` caches the capture root from its build-time
   startup scan (`services/arrival.rs`, cache rule unchanged per the plan). So with the row's literal input, (ii) cannot see `document_info` and (i)
   passes even without the direct closure. Reopening gives a fresh cache: the Capture One build asks `document_info` exactly once and the direct
   build asks nothing (probed). The amended (i) fails when the closure is removed (checked, then restored).
2. **Two test lines outside the plan's perimeter**, both direct consequences of the plan:
   - `crates/testkit/tests/p45_sidecar_and_protocol.rs` C2(a): `"20 tests, 0 failed"` → `"21 tests, 0 failed"`, because of the plan's new C++ `TEST_CASE`.
   - `crates/testkit/tests/p42_camera_runtime.rs` C7(a): `with_camera()` → `with_fake_document()` (plus a comment). From this phase a support folder
     selects the direct adapter, so the Capture One camera's health that C7(a) copies exists only without one (the plan's regression note).
3. C6(d)'s C++ half runs through `tools/sidecar/build.sh --stub --test`, as the plan says. P45's own `c2a`/`c6*` tests run that script from
   `cargo test`, so the new case is also exercised by the workspace suite through P45's harness.

## Files changed
Production and test perimeter (checkpoint `64aa4f8`, 38 paths = the plan's 34 + the 2 test lines above + tracker + plan Review log):
- new: `crates/integrations/src/camera/direct/{mod.rs, adapter.rs, filenames.rs}`
- integrations: `crates/integrations/src/camera/{mod.rs, runtime.rs}`, `crates/integrations/src/camera/sidecar/protocol.rs`,
  `crates/integrations/src/camera/support/mod.rs`, `crates/integrations/src/lib.rs`
- core: `crates/core/src/ports/camera.rs`, `crates/core/src/services/{camera.rs, camera_ledger.rs, arrival.rs}`, `crates/core/src/error.rs`
- desktop: `apps/desktop/src-tauri/src/commands/{camera.rs, items.rs}`, `apps/desktop/src-tauri/src/lib.rs`, `apps/desktop/src/services/{bindings.ts, camera.ts}`
- native: `native/camera-sidecar/src/{main.cpp, transport_manager.cpp}`, `native/camera-sidecar/include/crlab/transport.hpp`,
  `native/camera-sidecar/tests/test_transport_manager.cpp`
- testkit: `crates/testkit/src/test_app.rs`, `crates/testkit/src/bin/fake_sidecar.rs`, `crates/testkit/src/fixtures/sidecar_scripts/{ready_usb,
  ready_wifi, ready_usb_store_failed, capture_usb, camera_shutter, claimed, fingerprint_unknown, auth_refused, discovering}.txt`
- tests: `crates/testkit/tests/p46_direct_camera_adapter.rs` (new); `crates/testkit/tests/{p45_sidecar_and_protocol.rs, p42_camera_runtime.rs}` (one line each)
- docs: `docs/implementation/master_plan.md` (P46 tracker row only), `docs/implementation/plans/P46_direct_camera_adapter.md` (Review log entry)
- not committed by me: this handoff (the runner commits it). No tool-recorded state; no migration; no archgraph delta (not used in this project).
- Build outputs (gitignored, not committed): `target/camera-sidecar/stub/`, `apps/desktop/src-tauri/binaries/camera-sidecar-aarch64-apple-darwin`.

## Checkpoint SHA
`64aa4f8` — `CHECKPOINT (not approved): P46_direct_camera_adapter implement r1`

## Stamp
| Command | Result |
|---|---|
| `cargo test --workspace --no-fail-fast` | exit 0 — 723 passed, 0 failed (p46 25/25; desktop `p46_tests` 2/2; direct unit 4/4) |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo fmt --all -- --check` | exit 0 |
| `cd apps/desktop && npm run typecheck` | exit 0 |
| `cd apps/desktop && npm test` | 50 files, 369/369 passed |
| `tools/sidecar/build.sh --stub --test` | `21 tests, 0 failed`; `[ OK ] reconnect reopens the wifi session once` |
| `cargo test -p psm-desktop export_bindings` | bindings regenerated (committed file = regenerated) |
Pre-existing failures: none. The first workspace run stopped on P42 `c7a` (Deviation 2, fixed); the rerun above is green.

## Manual try-out
Hardware-free (the camera itself is P47's job):
1. Start the app with no camera support folder (`PSM_DATA_DIR=/tmp/psm-dev npm run tauri -- dev` in `apps/desktop`): everything is as before, and
   the camera adapter is still Capture One's.
2. Open Health: the camera row shows as before.
3. Nothing else is visible yet. The new behaviour is reachable only with a hand-installed support folder and the real helper, which P47 sets up.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
