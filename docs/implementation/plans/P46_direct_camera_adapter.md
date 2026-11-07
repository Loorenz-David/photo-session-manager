---
phase: P46_direct_camera_adapter
title: Direct camera adapter — the Sony-SDK-sidecar CameraProvider over the protocol client: status from events, capture with ids, exposures and Save-to-PC deliveries into the ledger, store-destination outcome, busy and transport policy as reported, the watchdog's ftp-status and Wi-Fi-only reconnect, first copy wins end to end, adapter selection
difficulty: high
projection: done
depends_on: [P45_sidecar_and_protocol]
---
# P46 — Direct camera adapter (MC-31 behaviour, MC-33 integration)

## Goal
`DirectCameraAdapter` (`crates/integrations/src/camera/direct/`), the `CameraProvider` that drives the sidecar through P45's client and turns its
events into the neutral model: `CameraStatus` (via P42's `derive_status`), `exposure observed` → `CaptureSink::exposure_observed` (app- and
camera-initiated), `sdk.download.completed` → re-read, hash, `Delivery { via: Cable }` → the ledger (R-I20), `connection.ready.store_destination:
failed` → `NeedsAction { StoreDestinationUnverified }` (R-I19), `camera.busy`/`transport.state` → `Busy` (R-I22), `capture()` → the `capture` command
and its `CaptureId`, the Sony filename normalisation (`_N`, R-I26), `usb.claimed_by` → `NeedsAction { HeldByOtherProgram, holder }` (the naming of
programs is P48), the watchdog's first raise → `ftp-status` and the camera's FTP report on the health screen, reconnect only on Wi-Fi (R-I29),
transfer-in-progress for the watchdog; `App::build` selects this adapter whenever a camera support package is installed and keeps `CameraViaCaptureOne`
otherwise. **First copy wins** is proven end to end: a Save-to-PC copy and the same bytes by FTP through the real receiver become one photo and one
`.duplicates/` file, in either order (M37, M38).
**Taken from P44 projection r0 (L18):** R-I28's last sentence — with the direct runtime bound, `ArrivalService` takes the capture root from the
settings formula (the receiver's, P43 C6(b)), never from Capture One — is this phase's Task 2 and row C8(c).
**Not in this phase:** naming the programs behind `ptpcamerad`, the start order, the launch gate and "Take the camera back" (P48); the collection switch
and style (P49); settings, Keychain UI, fingerprint confirmation and support install (P50/P51); anything against the camera.

## Read first
Book: `10E_direct_camera_adapter.md` §What the photographer gets, §Shape, §Port changes (R-I37, R-I33), MC-31 (incl. **R-I18, R-I19, R-I20, R-I22, R-I23**),
MC-33 (**R-I26** normalisation by the adapter, **R-I29** `ftp-status` and "the health screen shows the result", reconnect on Wi-Fi only), §Invariants
M37–M39 · `14_measurement_ledger.md` M37–M39 · evidence `sony_camera_lab.md` C1 (`UsbExclusiveOwner` texts), C3 (ack ≠ exposure ≠ file), C4, C9, §V6 ·
process `flow.md` §4, §6, §9 · master plan §3, §6.4, §6.6 (the single writer of `camera_files_not_received`), §6.7 (`camera_capture` → `{capture_id}`,
`camera_reconnect`, `camera_files_not_received_dismiss`, `camera_status_changed`), **§6.11** (Domain, Ports, Mechanisms `camera_status`, Services
`LedgerService`/`LedgerShared`, Runtime, Sidecar `SidecarEvent`, Direct adapter, Support `installed`), §7.2 (the native test command), §7.5.
Code at `e907dfb` (P45 approved; names by their §6.11 entries): `crates/core/src/services/camera.rs` (`CameraService`: `status`, `retry`, `capture() ->
Result<CaptureId>` already), `crates/core/src/ports/camera.rs`, `crates/integrations/src/camera/via_capture_one.rs`, `crates/integrations/src/camera/
runtime.rs` (`bind`, `tick` — the one `client.poll` per tick, the ring buffer, `push_helper_log`, `uploads_in_progress`, `ledger()`, `send_command`),
`crates/integrations/src/camera/sidecar/{protocol.rs, client.rs, supervisor.rs}` (`parse_line` required fields; `Supervisor::tick` drains every available
line), `crates/integrations/src/camera/support/mod.rs` (`installed_sidecar` only), `crates/core/src/services/camera_ledger.rs` (`watchdog` →
`WatchdogState { pending_count, gaps, raised, first_raise }`, `dismiss`, `exposure_observed` drops `provisional`, duplicates named `base_name(camera_filename)`),
`crates/core/src/mechanisms/{camera_status.rs, capture_ledger.rs}` (`pending_count`: both bounds exclusive), `crates/infra/src/ftp/session.rs` (`completed_at
= clock.now_ms()`), `crates/integrations/src/lib.rs` (`App::build`: `camera` built at `:481`, `ArrivalService::new(.., camera, ..)` `:496–502`, `bind` last
`:728`), `apps/desktop/src-tauri/src/lib.rs` (`collect_commands!`; the mock-webview test pattern `#[cfg(test)] mod p42_tests`, `:538–606`),
`apps/desktop/src-tauri/src/commands/camera.rs`, `apps/desktop/src/services/camera.ts`, `crates/testkit/src/bin/fake_sidecar.rs` (script mode; no
inject channel yet), `crates/testkit/src/test_app.rs` (`with_camera_script`, `ManualClock::new(0)` at `:205`, `sidecar_events`, `ledger`, `ftp`,
`capture_root`), `crates/testkit/src/fixtures/sidecar_scripts/{capture.txt, lab_lines.txt}`.
Native (read-only unless Task 0 names it): `native/camera-sidecar/src/main.cpp:486–498, 538–549, 553–556, 562–564` (capture order, `ftp-status`, the
busy deadline), `src/transport_manager.cpp:85–92` (`transport.state` fields), `:147–152` (`camera.busy` is emitted at the next `step()`), `:205–223`
(`session.lost` then `RECOVERING`), `:226–233` (`session.closed`), `:244` (`DISCOVERING`, reason `looking for camera over usb|wifi`), `:280`, `:310–323`
(`connection.ready` with `model`, `store_destination`, `store_value`), `:375–376` (`SWITCHING` then `session.closed{reason:"switching wifi -> usb"}`),
`src/usb_monitor_macos.cpp:204–208` (`owners` joined with `"; "`), `src/camera_connection.cpp:23–30` (`describe_error`: `ssh user authentication failed
(check Access Authen. user/password)` ↔ `AuthenticationRefused`, `ssh server authentication failed (fingerprint mismatch)` ↔ `FingerprintMismatch`; the
texts the `connect.failed.error` field carries — prefixed `connection failed: ` on the callback path, `:511–512`; the field is **`error`**, never
`message`, P45 projection L9), `:142–149` (`capture.exposed` association strings), `:291–314` (`ssh.fingerprint` statuses `unknown`/`MISMATCH`/`match`/
`pinned`), `src/camera_capture.cpp:160–185` (`sdk.download.completed`), `src/camera_properties.cpp:84–105, 198–211` (`camera.ftp_state` fields:
`ftp_function`, `ftp_auto_transfer`, `ftp_transfer_target`, `ftp_connection_status`, `ftp_connection_error`, `ftp_auto_transfer_target`,
`ftp_auto_transfer_target_still`, `ftp_power_save`, raw values, plus `request_ftp_transfer_result_error` (hex) or `request_ftp_transfer_result`).
Lab (read-only): `docs/session-manager-handoff.md` §4.4 (event meanings), §8 rows 5, 6, 13.

## Files expected to change
Integrations (new): `crates/integrations/src/camera/direct/{mod.rs, adapter.rs, filenames.rs}`. Integrations (changed): `crates/integrations/src/camera/
{mod.rs, runtime.rs}` (the direct adapter's creation and lifetime, the adapter step inside `tick`, the watchdog glue, `provider()`),
`crates/integrations/src/camera/sidecar/protocol.rs` (`Command::Reconnect {cmd_id}` in `encode`/`cmd_id`/`with_cmd_id`), `crates/integrations/src/camera/
support/mod.rs` (`installed(paths) -> Option<InstalledSupport>`: version folder + `camera-sidecar` present; the manifest check is P50's), `crates/integrations/
src/lib.rs` (`App::build` selection, `ArrivalService` capture-root closure).
Core (changed): `crates/core/src/ports/camera.rs` (`reconnect`, `request_ftp_status` as **default methods** answering `Err(Error::Provider("not
supported"))`, so `via_capture_one.rs` and `fake_direct_camera.rs` are untouched and nothing is hidden: the default fails closed), `crates/core/src/services/
camera.rs` (`reconnect()`, `dismiss_files_not_received()`; `capture()` already returns `CaptureId`), `crates/core/src/services/camera_ledger.rs` (`LedgerShared.
ftp_note`, `set_ftp_note`, the note appended by `watchdog` — §The adapter contract, Watchdog glue), `crates/core/src/error.rs` (`CameraNotConnected`,
`ReconnectOnCableRefused`, `SidecarUnavailable`), `crates/core/src/services/arrival.rs` (`capture_root` from an injected closure while the direct adapter is
selected, L18).
Desktop (changed): `apps/desktop/src-tauri/src/commands/camera.rs` (`camera_capture -> CaptureResponse { capture_id }`, `camera_reconnect`,
`camera_files_not_received_dismiss`), `apps/desktop/src-tauri/src/lib.rs` (`collect_commands!`; the mock-webview tests of rows C8(b) and C6(c)'s command
half in a `#[cfg(test)] mod p46_tests`, pattern `p42_tests`), `apps/desktop/src/services/bindings.ts` (regenerated), `apps/desktop/src/services/camera.ts`
(`cameraCapture` returns the id; callers ignore it), `apps/desktop/src-tauri/src/commands/items.rs` (error codes).
Native (changed; the `reconnect` command, **mock-testable**, run by `tools/sidecar/build.sh --stub --test`, never by `cargo test`): `native/camera-sidecar/
src/main.cpp` (`reconnect` → `tm.reconnect()`; on USB `cmd.rejected {reason:"reconnect only on wifi"}`), `native/camera-sidecar/src/transport_manager.cpp`
and `native/camera-sidecar/include/crlab/transport.hpp` (`TransportManager::reconnect()`), `native/camera-sidecar/tests/test_transport_manager.cpp` (the
`TEST_CASE` "reconnect reopens the wifi session once").
Testkit (changed): `crates/testkit/src/test_app.rs` (`with_camera_script` starts the clock at `SCRIPT_EPOCH_MS`; `capture()`, `camera_status()`,
`inject(event, fields)`, `inject_exit(code)`, `sdk_download(name, bytes)`, `staging_dir()` — §Fixtures), `crates/testkit/src/bin/fake_sidecar.rs` (the
`inject.txt` tail), `crates/testkit/src/fixtures/sidecar_scripts/*.txt` (new scripts: `ready_usb.txt`, `ready_wifi.txt`, `ready_usb_store_failed.txt`,
`capture_usb.txt`, `camera_shutter.txt`, `claimed.txt`, `fingerprint_unknown.txt`, `auth_refused.txt`, `discovering.txt` — 9 files; `capture.txt` is reused
unchanged). Tests (new): `crates/testkit/tests/p46_direct_camera_adapter.rs`.
**Count (derived from the lists above):** new source 3 + changed integrations 5 + changed core 5 + changed desktop 5 + changed native 4 + changed testkit 2
+ new scripts 9 + new test 1 = **34 paths** (25 without the scripts).

## Core mechanisms (unit tests allowed)
`direct/filenames.rs::normalize_camera_filename` (rows C2(d)); `direct/adapter.rs::needs_action_from_connect_failed(error) -> Option<NeedsActionReason>`
(C4(c)); `direct/adapter.rs::holder_from_owners(owners) -> String` (the parse of §The adapter contract, Holder; exercised by C7(a) at the boundary, a unit
test is allowed for the three shapes). Everything else is a behaviour row through `TestApp`.

## The adapter contract
- **Plumbing (one poller, L19).** The runtime's `tick` keeps its single `client.poll()` per tick and the P45 ring buffer unchanged; it hands the batch,
  the supervisor's `Started`/`Exited` side events and `helper_running` (= the supervisor has a live child) to the adapter, which applies the table below
  and returns what to do (status to emit, health to publish, deliveries to hand to the sink, a helper-log line, an FTP note). The runtime performs those
  effects against the **current binding's** `events`, `health` and sink, so a settings save re-targets by construction (R-I18). Hashing a download runs
  **outside the supervisor lock** (`tick` releases the lock before `sha256_file`; `capture()` must not wait on a 51 MB hash). How the adapter reaches
  `send_command` (a `Weak<CameraRuntime>` or a command closure) is delegated.
- **Lifetime and selection (R-I18, L18).** The adapter is created **once per runtime** (`CameraRuntime::direct_adapter() -> Arc<DirectCameraAdapter>`,
  lazily, kept in the runtime) and reused by every later `App::build` and `bind`: its `StatusInputs` survive a settings save, because the sidecar does not
  re-send `connection.ready` for a session that stays open. `App::build` decides the selection **up front**: `support::installed(&paths)` → `Some` →
  `camera = camera_runtime.direct_adapter()`; `None` → `CameraViaCaptureOne` as today. The same `Arc` goes to `ArrivalService::new`, to `AppBinding.provider`
  and to `App.camera`/`AppHandles.camera`. `CameraRuntime::provider()` returns the current binding's provider (the watchdog glue calls
  `request_ftp_status` through it; there is no separate provider slot). The selection also chooses `ArrivalService`'s capture-root closure (Task 2).
- **State** (`Mutex<AdapterState>`): `status_inputs: StatusInputs`, `expected_transfer: Option<Instant>`, `sdk_unavailable: bool`, `staging_dir: PathBuf`
  (the `--save-dir` of the first spawn). No `usb_fallen_back`, no `pending_captures` (no consumer in P46 or P48; P48's start order triggers on the
  `UsbFallbackToWifi` **event**). After each batch: `derive_status` → if changed: `events.emit(CameraStatusChanged { status })`; health re-published from
  the derived inputs (below) whenever the inputs changed.
- **Owned health codes (L11).** The adapter owns, through the runtime's publish: `camera`, `camera_connecting`, `camera_busy`, `camera_disconnected` and
  every `NeedsActionReason::code()` **except `camera_files_not_received`** — that item has one writer, the ledger's `watchdog`/`dismiss` (its detail
  carries the gaps count and the FTP note). Publish = `health_items(&inputs)` minus the `camera_files_not_received` item; for each owned code: set/`set_ok`
  when present, else `clear`. Never touched by the adapter: `camera_files_not_received`, `camera_photo_session_uncertain` (ledger), `camera_sidecar_unavailable`
  (runtime; see `SdkUnavailable` below).
- **Event table:** `TransportState { to, reason }`: `DISCOVERING → connecting = Some(link from the reason's suffix: "over usb" → Cable, "over wifi" →
  Wireless; any other reason leaves connecting unchanged)` (L4), `CONNECTING_USB → Some(Cable)`, `CONNECTING_WIFI → Some(Wireless)`, `CONNECTED_* →
  connecting = None`, `SWITCHING → busy = Some(Switching)`, `DISCONNECTED | RECOVERING | ERROR → connecting = None` and `busy = None` · `ConnectionReady
  { transport, model, store_destination }` → `session = Some((model or "Camera", link))`, `connecting = None`, `busy = None`, `store_destination == Some(Failed)`
  → add `StoreDestinationUnverified`, `Some(Verified)` → remove it; remove `HeldByOtherProgram` (the interface is ours now); `transport == wifi` → remove
  `AuthenticationRefused` and `FingerprintMismatch` (L12) · `SessionClosed` → `session = None`, `expected_transfer = None`, `busy` kept **only if** it is
  `Switching`, else `None` (L3: a switch is `SWITCHING` → `session.closed` → `DISCOVERING` → `CONNECTING_*` → `CONNECTED_*` → `connection.ready`, and
  `Switching` survives all of them until `connection.ready`, `DISCONNECTED`/`RECOVERING`/`ERROR` or a sidecar exit) · `SessionLost` → `session = None`,
  `busy = None`, `expected_transfer = None` · `CameraBusy { reason }`: `"exposure"` → `Some(Exposure)`, `"file_transfer"` → `Some(Transfer)`, `"none"` →
  `None`; `"bracketing" | "pixel_shift" | "other"` → no change (they come only from the `busy reason=` stdin command the app never sends) (L2) ·
  `CaptureExposed { capture_id, transport, association }` → `sink.exposure_observed(ExposureObserved { capture_id: capture_id.filter(|_| !association.
  starts_with("none")), at: timestamp_ms, initiated: if association starts with "none" { Camera } else { App }, provisional: association ==
  "latest_release_provisional" })`; when `transport == usb`: `expected_transfer = Some(now)`, `busy = Some(Transfer)`, `sink.transfer_in_progress(Cable,
  true)` · `SdkDownloadCompleted { filename, size_bytes, success }`: `success && size_bytes > 0 && Path(filename).starts_with(staging_dir) && the file
  exists` → `size = fs::metadata(filename).len()`, `sha256_file`, `sink.delivery(Delivery { camera_filename: normalize_camera_filename(basename),
  written_path, sha256, size, via: Cable, completed_at: timestamp_ms })`; then `expected_transfer = None`, `busy = None` (if it was `Transfer`),
  `transfer_in_progress(Cable, false)`; otherwise the helper log gets **`download ignored: <basename> (<reason>)`** with reason `failed` (`!success`),
  `size 0` (`size_bytes <= 0`), `outside staging`, or `missing` (checked in that order), and the file is left where it is (never promoted, never deleted)
  (L17; the text is contract, written through the runtime's helper log) · `expected_transfer` with `now − t ≥ TRANSFER_EXPECTED_MS` → cleared the same way
  (R-I20 "at most 10 s"; L16: `≥`) · `CaptureUncertain` → `sink.session_uncertain()` · `UsbClaimedBy { owners }` → add `HeldByOtherProgram` with `holder =
  holder_from_owners(owners)` (below) · `UsbFallbackToWifi`, `UsbArrived`, `UsbRemoved`, `UsbInitial`, `UsbRetryRequested`, `SwitchDeferred`, `CaptureRequested`,
  `CaptureAf`, `CaptureAcknowledged`, `CaptureExposureMissing`, `Unknown` → no change · `SshFingerprint { status: "unknown" }` → add `FingerprintUnconfirmed`;
  `"MISMATCH"` → add `FingerprintMismatch`; `"match" | "pinned"` → remove both · `ConnectFailed { error, .. }`: `needs_action_from_connect_failed(error)`
  (contains `ssh user authentication failed` → add `AuthenticationRefused`; contains `fingerprint mismatch` → add `FingerprintMismatch`; else no change) ·
  `SdkUnavailable` → `sdk_unavailable = true` and health `camera_sidecar_unavailable` (detail `The camera helper could not start`) set **after** the
  runtime's per-batch clear of that code, so the set wins (L21) · `CameraFtpState { fields }` → FTP note (Watchdog glue, L13) · `CmdReceived`/`CmdDone`/
  `CmdRejected`/`CmdError` → the client's correlation only.
- **Supervisor events (L12).** `Started { .. }` → `sdk_unavailable = false`, remove `AuthenticationRefused` and `FingerprintMismatch` (a new process gets a
  fresh try; P50's credentials save respawns). `Exited { .. }` → `session = None`, `connecting = None`, `busy = None`, `expected_transfer = None`,
  `sink.transfer_in_progress(Cable, false)`; `helper_running` is false until the next `Started`, so the status is `Disconnected` through the backoff and
  `capture()` is refused instead of writing to a dead process.
- **Holder (L20).** `holder_from_owners(owners)`: first owner = the text before the first `"; "` (the whole text when absent); holder = the text after
  its last `,`, trimmed, or the whole trimmed first owner when it has no comma. `"pid 512, ptpcamerad"` → `ptpcamerad`; `"ptpcamerad"` → `ptpcamerad`;
  `"pid 512, ptpcamerad; pid 9, Other"` → `ptpcamerad` (`sony_camera_lab.md` C1 and §V6 show both single shapes).
- **`capture()`** (order, L21): `sdk_unavailable` → `Err(SidecarUnavailable)`; then `session.is_none()` → `Err(CameraNotConnected)`; else `id =
  CaptureId(uuid)`, `client.send(Capture { id, af: true, cmd_id })`, return `Ok(id)` at once (`capture.acknowledged`/`exposed` are events, C3: "neither
  means a file exists"). `capabilities()` = `{ status, capture, transfer, exposure_events, settings_read, settings_write_store_destination: true,
  depends_on_editing_provider: false }`. `raw_extensions() = ["arw"]`. `status()` = `derive_status(&inputs)`.
- **`reconnect()`** (R-I29): `session == Some((_, Wireless))` → `client.send(Command::Reconnect)` (new sidecar command `reconnect`: `TransportManager::
  reconnect()` closes the current Wi-Fi session with `intentional_disconnect("reconnect requested")` and retries the same transport at once — **C++ edit in
  this phase**, mock-testable: `TEST_CASE("reconnect reopens the wifi session once")` — exactly one `session.closed{reason:"reconnect requested"}`, one
  more `connection.ready{transport:"wifi"}`, one connect call; on a USB session `main.cpp` answers `cmd.rejected {reason:"reconnect only on wifi"}`);
  `Some((_, Cable))` → `Err(ReconnectOnCableRefused)`; `None` → `Err(CameraNotConnected)`.
- **`request_ftp_status()`**: `client.send(FtpStatus)`; `Ok(())`. The watchdog glue is its caller, through the `CameraProvider` port.
- **Watchdog glue** (runtime tick; L10, L13): once per mono second, `transfer_active = receiver.uploads_in_progress() > 0 || expected_transfer.is_some()`
  (read on every call); `state = ledger.watchdog(now_ms, transfer_active)`; `state.first_raise` → `provider().request_ftp_status()`. **On every tick**
  (not only the 1-s cadence) the adapter's `FilesNotReceived { count }` input mirrors the ledger: present with `state.pending_count` while the ledger's
  raised flag (`LedgerShared.raised`) is true, absent otherwise — so a `dismiss()` reaches the status at the next `a.tick(0)`. `CameraFtpState { fields }`
  → `ledger.set_ftp_note(Some(short form))` where the **short form** = `camera FTP: ` + every non-envelope field except `cmd_id`, sorted by key, as
  `key=value` (JSON scalars as `serde_json` prints them, strings without quotes), joined `, `, truncated to 160 chars with `…`. The ledger's `watchdog`
  appends ` · <short form>` to the `camera_files_not_received` detail while it raises; `dismiss()` and a non-raising `watchdog` clear the note. The ledger
  stays the item's single writer; the note lives in `LedgerShared` so it survives a settings save. `CameraService::dismiss_files_not_received()` →
  `ledger.dismiss()` (how `CameraService` reaches the ledger is delegated).

## Fixtures (testkit)
- **Time base (L1).** `SCRIPT_EPOCH_MS = 1_793_959_200_000` (= `2026-11-06T10:00:00.000Z`, the day P45's `capture.txt` is stamped), a `pub const` in
  `test_app.rs`. `TestApp::with_camera_script` builds `ManualClock::new(SCRIPT_EPOCH_MS)`; `with_camera()`/`with_camera_behaviour` keep `0`. **T0** = the
  clock at start = `SCRIPT_EPOCH_MS`. Every script line below is stamped on `2026-11-06T10:00:0x`; every injected line is stamped by the test with
  `a.clock.now_ms()` rendered `YYYY-MM-DDTHH:MM:SS.mmmZ`, never wall time. The exposure stamps are **t_e = T0 + 2 550 ms** for `capture_usb.txt` and
  **t_e = T0 + 550 ms** for `capture.txt`/`camera_shutter.txt`. FIFO (R-I26 rule 3: `exposed_at ≤ completed_at ≤ exposed_at + 300 s`) therefore holds
  iff a completion is stamped after t_e and within 300 s: every FIFO row runs **`a.tick(5_000)` (clock = T0 + 5 s) before its first completion**, and the
  FTP receiver stamps `completed_at = clock.now_ms()` (`session.rs:127`), so an upload within those bounds matches too. Watchdog rows advance the clock
  by 61 s: age = 61 s − 0.55 s > `WATCHDOG_MIN_AGE_MS` (`pending_count`'s bounds are exclusive).
- **Line shapes (L6).** Plan lines are abbreviated: every script or injected line carries `timestamp`, `source:"sdk"`, `event` and every field
  `parse_line` requires (`protocol.rs:182–310`, §6.11 Sidecar): `transport.state {from, to, reason, transport}`, `connection.ready {transport, model?,
  store_destination?, store_value?}`, `session.closed {transport, reason}`, `session.lost {transport, reason, busy}`, `camera.busy {reason, safe_point}`,
  `capture.exposed {transport, association, capture_id?}`, `sdk.download.completed {filename, size_bytes, success, transport}`, `usb.claimed_by {owners,
  hint?}`, `connect.failed {transport, sdk_error, error}`, `ssh.fingerprint {status, success?}`. An unparsable line never reaches the adapter (it goes to the
  helper log), so a missing field makes a row fail against correct code.
- **Batching (L5).** `Supervisor::tick` drains every available line, and the adapter derives once per batch, so a state a row samples between two lines
  needs a pause: scripts carry `sleep 300` after each line whose state a row samples; a test that injects a sequence and samples in the middle appends the
  lines in two writes with a `settle` between. `settle(a, pred)` is P42's loop (`a.tick(0)` / 10 ms sleep, 3 s bound); negative reads follow the
  positive/negative settle rule.
- **Scripts** under `fixtures/sidecar_scripts/` (`A + B` in a row = the two files' texts concatenated by the test, in that order):
  `discovering.txt`: `transport.state {from:"DISCONNECTED", to:"DISCOVERING", reason:"looking for camera over usb", transport:"none"}` (T0).
  `ready_usb.txt`: the `discovering.txt` line (T0), `sleep 300`, `transport.state {from:"DISCOVERING", to:"CONNECTING_USB", reason:"camera found",
  transport:"none"}` (T0 + 400 ms), `transport.state {from:"CONNECTING_USB", to:"CONNECTED_USB", reason:"connected", transport:"usb"}` (T0 + 900 ms),
  `connection.ready {transport:"usb", connect_ms:500, model:"ILCE-7RM5", store_destination:"verified", store_value:"0x0003"}` (T0 + 950 ms)
  (`transport_manager.cpp:244, 280, 310, 317–323`; `store_value` is not asserted).
  `ready_wifi.txt`: the same over Wi-Fi (`reason:"looking for camera over wifi"`, `CONNECTING_WIFI`, `CONNECTED_WIFI`, `connection.ready {transport:"wifi",
  model:"ILCE-7RM5", store_destination:"verified", store_value:"0x0001"}`).
  `ready_usb_store_failed.txt`: `ready_usb.txt` with `store_destination:"failed"`.
  `capture_usb.txt` (the real USB order, `main.cpp:538–549`, `transport_manager.cpp:147–152`): `await capture`, `cmd.received {cmd:"capture", cmd_id:
  "{cmd_id}"}` (T0 + 2 000 ms), `camera.busy {reason:"exposure", safe_point:false}` (T0 + 2 001), `sleep 300`, `capture.requested {capture_id:
  "{capture_id}", transport:"usb", af:true}` (T0 + 2 002), `capture.acknowledged {capture_id:"{capture_id}", success:true, sdk_error:"0x00000000"}`
  (T0 + 2 300), `capture.exposed {success:true, transport:"usb", capture_id:"{capture_id}", ms_since_release:269, association:"latest_release_provisional"}`
  (**t_e = T0 + 2 550**), `cmd.done {cmd:"capture", cmd_id:"{cmd_id}"}` (T0 + 3 300), `camera.busy {reason:"file_transfer", safe_point:false}` (T0 + 3 400).
  The real sidecar follows the download with `camera.busy {reason:"none", safe_point:true}`; the adapter already cleared `Transfer` on the completion, so
  that line is not scripted (it would be a no-op).
  `capture.txt` (P45, Wi-Fi, unchanged; t_e = T0 + 550 ms) is used by the Wi-Fi capture rows; `capture.txt + capture.txt` = two captures.
  `camera_shutter.txt`: `capture.exposed {success:true, transport:"wifi", association:"none: no SDK capture pending (camera-initiated?)"}` with no
  `capture_id` (T0 + 550 ms; `camera_connection.cpp:148`).
  `claimed.txt`: `usb.claimed_by {owners:"pid 512, ptpcamerad", hint:"another process holds the camera's USB interface; …"}` (T0), `connect.failed
  {transport:"usb", sdk_error:"0x00008208", error:"connection error", duration_ms:12}` (T0 + 12 ms).
  `fingerprint_unknown.txt`: `ssh.fingerprint {status:"unknown", success:false, note:"re-run with --accept-fingerprint after comparing with the camera
  screen"}` (T0; `camera_connection.cpp:303–304`).
  `auth_refused.txt`: `connect.failed {transport:"wifi", sdk_error:"0x00008208", error:"connection failed: ssh user authentication failed (check Access
  Authen. user/password)", duration_ms:900, timeout:false}` (T0; the callback path's shape, `camera_connection.cpp:511–512`; the adapter reads the text,
  the hex code is not asserted).
- **Inject channel (L14).** In script mode, once the script has ended, the fake tails `rec/inject.txt` (read every 20 ms between its stdin polls; new lines
  only) and runs each line as a script directive: a JSON object is printed verbatim, `exit <code>` exits with that code (recorded in `exits`). The fake has
  no other inject path. `TestApp::inject(event: &str, fields: serde_json::Value)` appends the JSON line `{timestamp: iso(a.clock.now_ms()), source:"sdk",
  event, ..fields}`; `TestApp::inject_exit(code)` appends `exit <code>`. `TestApp::sdk_download(name, bytes)` writes `bytes` to `a.staging_dir()/<name>` and
  injects `sdk.download.completed {filename: <that path>, size_bytes: bytes.len(), success: true, transport: "usb"}`. `a.staging_dir()` = `a.capture_root()
  /.incoming/sdk-<a.camera_runtime().run_id()>` (the `--save-dir`, P45 §Staging). `a.capture()` = `CameraService::new(a.app.camera, a.app.health).capture()`;
  `a.camera_status()` = `CameraService::new(..).status()` (listed in §7.5 since P42 but absent at HEAD; added here with its callers).

## Ordered tasks
0. **Registry first** (§6.11): errors, `CameraProvider::{reconnect, request_ftp_status}` default methods, `Command::Reconnect` (+ C++ `reconnect` command
   and its test), `InstalledSupport`/`installed`, `CaptureResponse`, `LedgerShared.ftp_note` + `LedgerService::set_ftp_note` + the `watchdog` suffix.
1. `direct/filenames.rs` + unit tests; `direct/adapter.rs` per §The adapter contract; `needs_action_from_connect_failed`, `holder_from_owners`.
2. Runtime: `direct_adapter()`, the adapter step inside `tick` (one poll), the watchdog glue, `provider()`; `App::build` selection (`support::installed`)
   for `camera`, `ArrivalService` and `AppBinding.provider`.
   **Capture root (P44 projection L18, R-I28):** while the direct adapter is selected, `ArrivalService::capture_root()` resolves `<settings document>/
   Capture` by the receiver's formula (P43 C6(b)) and never calls `EditingProvider::inspect_document`; with `CameraViaCaptureOne` it asks Capture One as
   today. The cache rule (`services/arrival.rs:57–68`) is unchanged. Row C8(c).
3. `CameraService`: `reconnect`, `dismiss_files_not_received`; commands + bindings; `services/camera.ts`.
4. Testkit fixtures and helpers (the fake's inject tail, `SCRIPT_EPOCH_MS`, the helpers of §Fixtures); tests `p46_direct_camera_adapter.rs` and the
   `p46_tests` module in the desktop crate; suite, clippy, fmt, typecheck, `npm test`, and `tools/sidecar/build.sh --stub --test` for the C++ half.

## Criteria (8 criteria, 27 rows; 0 `[rule]` rows)
`a = TestApp::with_camera_script(<script>)` (clock at T0); `st()` = `a.camera_status()`; `ev()` = the `CameraStatusChanged` payloads in `a.events`;
`H` = `a.app.health`; `L = a.ledger()`; `s` the active session; `rec()` = `a.fake_sidecar_recording()`; "until X" = `settle(a, || X)`.

| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | status from a USB session | `ready_usb.txt`; until `st()` is `Connected` | `st() == Connected { name: "ILCE-7RM5", link: Cable }`; `ev()` ends with `Connecting { link: Cable }` (from `DISCOVERING … over usb`, sampled thanks to the script's `sleep 300`) then `Connected { .. }`; `H` has ok `camera` with detail `ILCE-7RM5 · cable`, no `camera_disconnected`, no `camera_connecting` | R-I37; 10E §What the photographer gets (the dot shows camera and link) |
| C1(b) | Wi-Fi session | `ready_wifi.txt` | `Connected { name: "ILCE-7RM5", link: Wireless }` | R-I37 |
| C1(c) | session lost → connecting; sidecar dead → disconnected | `ready_usb.txt`, until `Connected`; inject (one write) `session.lost {transport:"usb", reason:"usb device removed", busy:"none"}`, `transport.state {from:"CONNECTED_USB", to:"RECOVERING", reason:"usb device removed", transport:"none"}`, `transport.state {from:"RECOVERING", to:"DISCOVERING", reason:"looking for camera over wifi", transport:"none"}` (`transport_manager.cpp:205–223, 244`); until `Connecting`; then `a.inject_exit(3)`; until `Disconnected` — **without advancing the clock** (the restart after 1 s of mono time replays the script) | `Connecting { link: Wireless }` with `H` `camera_connecting` detail `Connecting over Wi-Fi…`; after the exit `Disconnected`, `H` has `camera_disconnected` and no `camera`/`camera_connecting`; `a.capture()` → `Err(CameraNotConnected)` (the exit reset) | R-I37 ("`Connecting` (sidecar running, no session) › `Disconnected`"); R-I23 (cable pull closes a USB session) |
| C1(d) | switching is busy and survives the intermediate lines | `ready_wifi.txt`, until `Connected`; inject (one write) `usb.arrived {detail:"pid=0x0E0C ILCE-7RM5"}`, `transport.state {from:"CONNECTED_WIFI", to:"SWITCHING", reason:"usb available; preferring usb", transport:"wifi"}`, `session.closed {transport:"wifi", reason:"switching wifi -> usb"}`; until `Busy`; then inject (one write) `transport.state {from:"SWITCHING", to:"DISCOVERING", reason:"looking for camera over usb", transport:"none"}`, `… to:"CONNECTING_USB" …`, `… to:"CONNECTED_USB", transport:"usb"`, `connection.ready {transport:"usb", model:"ILCE-7RM5", store_destination:"verified", store_value:"0x0003"}` (`transport_manager.cpp:375–376, 226–233, 244, 280, 310, 323`) | after `session.closed`: `st() == Busy { reason: Switching }` and `H` `camera_busy` detail `Switching connection`; after `connection.ready`: `Connected { name: "ILCE-7RM5", link: Cable }`, no `camera_busy` | R-I22 ("the adapter reports `Switching` during a transport change") |
| C2(a) | `capture()` returns the id and records the exposure | `ready_usb.txt + capture_usb.txt`; until `Connected`; `activate(s)`; `id = a.capture()`; until `st() == Busy { Exposure }` (the script's `sleep 300` after the busy line); until a `CmdDone` in `a.sidecar_events()` | `rec().stdin` has `capture id=<id> af=1 cmd_id=…` exactly once; `L.exposures(None)` has one row `{ capture_id: Some(id), initiated: app, session_id: s, status: expecting, exposed_at: t_e }` (the **event's** timestamp, not `a.clock`); `Busy { reason: Exposure }` was sampled; after `cmd.done`: `st() == Busy { reason: Transfer }` (set by `capture.exposed {usb}`; the clears are C3(b)) | MC-31 §Capture (`capture()` returns a `capture_id`; "neither means a file exists"); MC-33 (exposure rows); R-I22; C3 |
| C2(b) | camera-initiated shot | `ready_wifi.txt + camera_shutter.txt`; `activate(s)` **before the first `a.tick`** (the fake prints regardless of ticks; the adapter processes the line at the first tick after activation, and `exposure_observed` reads `active_session_id` then); until one exposure row | one exposure `{ capture_id: None, initiated: camera, session_id: s, status: expecting, exposed_at: t_e }`; `st()` stays `Connected` (no busy on Wi-Fi) | MC-33 ("Camera-initiated exposures are shots, not orphans"); lab §8 row 13 |
| C2(c) | capture refused without a session | `discovering.txt`; until `Connecting`; `a.capture()` | `Err(CameraNotConnected)`; `rec().stdin` holds only the credentials line (negative settle); `L.exposures(None)` empty | MC-31; master plan §6.6 (`camera_not_connected`) |
| C2(d) | Sony re-send suffix (unit) | `normalize_camera_filename` on `DSC01234_1.ARW`, `DSC01234_12.ARW`, `DSC01234_123.ARW`, `DSC01234.ARW`, `IMG_1234.CR3`, `DSC01234__dup1.ARW` | `DSC01234.ARW`, `DSC01234.ARW`, `DSC01234_123.ARW` (three digits is not the suffix), `DSC01234.ARW`, `IMG_1234.CR3` (no trailing digits before `_`), `DSC01234__dup1.ARW` (core's suffix is not touched here) | R-I26 ("the adapter strips them … Sony: `_<1–2 digits>` after the stem's trailing number") |
| C3(a) | Save-to-PC completion becomes a cable delivery | `ready_usb.txt + capture_usb.txt`; `activate(s)`; `id = a.capture()`; until `CmdDone`; `a.tick(5_000)` (clock = T0 + 5 s > t_e); `a.sdk_download("DSC00007.ARW", data)`; until one delivery row | `L.deliveries()` has `{ camera_filename: "DSC00007.ARW", via: cable, sha256: sha256(data), size: len, association: fifo, outcome: photo, state: done, completed_at: T0 + 5 000 }`; the file is at `root/<key(s)>/DSC00007.ARW`; `a.staging_dir()` is empty; after `a.watcher_event(final_path)` + `a.tick(1_000)` one photo in `s` with `captured_at == t_e` | R-I20 ("A file is handed to the ledger only on `sdk.download.completed` with `size_bytes > 0`. The adapter then re-reads the size, hashes the file itself … and reports `via: Cable`"); R-I26 rule 3; M38 |
| C3(b) | busy `Transfer` until the completion or 10 s | as C3(a) up to `CmdDone`: `st() == Busy { Transfer }`; **(i)** `a.sdk_download(..)`; until `Connected` · **(ii)** a second app: as C3(a) up to `CmdDone`, no download, `a.tick(10_000)` (mono + 10 s, `now − t ≥ TRANSFER_EXPECTED_MS`) | (i) `Connected { .. }` after the completion · (ii) `Connected { .. }` after the tick, and `H` has no `camera_busy` in both | R-I20 ("While a download is expected … at most 10 s … the sidecar holds busy `Transfer`"); R-I22 |
| C3(c) | zero or missing download is never promoted | `ready_usb.txt`, until `Connected`; write an empty `a.staging_dir()/X.ARW`; `a.inject("sdk.download.completed", {filename: <that path>, size_bytes: 0, success: true, transport: "usb"})`; then `a.inject(.., {filename: a.staging_dir()/Y.ARW, size_bytes: 10, success: true, transport: "usb"})` with no such file; `a.tick(0)` until two new helper-log entries | `L.deliveries()` empty; `X.ARW` still in `a.staging_dir()` (empty, untouched); `a.camera_runtime().helper_log()` gained exactly `download ignored: X.ARW (size 0)` and `download ignored: Y.ARW (missing)` | R-I20 ("A file in `.incoming/` without a completion event … is never promoted and never deleted") |
| C3(d) | a `_1` re-send by cable keys to the original | `ready_usb.txt`; `activate(s)`; `a.tick(5_000)`; `a.ftp().stor("DSC00008.ARW", data)`; until `done`; `a.sdk_download("DSC00008_1.ARW", data)`; until the second row | second row: `camera_filename: "DSC00008.ARW"` (normalised), outcome `duplicate` (same `file_key`, same hash), final path `root/.duplicates/DSC00008.ARW` (the ledger names duplicates after the normalised `camera_filename`, `camera_ledger.rs:240, 248`), the file reads `data`; one photo after watcher + tick | R-I26; R-I27; M37 |
| C4(a) | store destination failed | `ready_usb_store_failed.txt`; until `NeedsAction` | `st() == NeedsAction { reason: StoreDestinationUnverified, holder: None }`; `H.has("camera_store_destination_unverified")` with detail `The camera did not confirm where photos are saved`; the session is kept: `H` has ok `camera` `ILCE-7RM5 · cable` and `a.capture()` is `Ok` with the `capture` line in `rec().stdin` | R-I19 ("`failed` → `NeedsAction { StoreDestinationUnverified }`, and the session is kept"); M39 ("Every store-destination write is verified") |
| C4(b) | verified clears it | after C4(a): inject `session.lost {transport:"usb", reason:"usb device removed", busy:"none"}` then `connection.ready {transport:"usb", model:"ILCE-7RM5", store_destination:"verified", store_value:"0x0003"}`; until `Connected` | `Connected { name: "ILCE-7RM5", link: Cable }`; `H` has no `camera_store_destination_unverified` | R-I19 |
| C4(c) | connect failures map to reasons (unit + boundary) | `needs_action_from_connect_failed` on the two `describe_error` texts (with and without the `connection failed: ` prefix) and on `connection error`; boundary: `auth_refused.txt`, until `NeedsAction` | `Some(AuthenticationRefused)` ×2, `Some(FingerprintMismatch)` ×2, `None`; `st() == NeedsAction { reason: AuthenticationRefused, holder: None }` with `H` `camera_auth_refused` detail `The camera refused the password`; then `a.inject("connection.ready", {transport:"wifi", model:"ILCE-7RM5", store_destination:"verified", store_value:"0x0001"})` → `Connected { .., Wireless }` and the item is gone (L12 clear rule) | 10E §What the photographer gets ("The camera refused the password"); R-I37; lab `camera_connection.cpp:23–30` |
| C4(d) | fingerprint unknown | `fingerprint_unknown.txt`; until `NeedsAction`; then `a.inject("ssh.fingerprint", {status:"pinned", success:true})`; until not `NeedsAction` | `NeedsAction { reason: FingerprintUnconfirmed, holder: None }` with detail `Confirm the camera's fingerprint`; after `pinned`: `Disconnected` (no session in this script) and no `camera_fingerprint_unconfirmed` | MC-31 §Fingerprint; R-I37 |
| C5(a) | first copy wins: cable then FTP | `ready_usb.txt + capture_usb.txt`; `activate(s)`; `a.capture()`; until `CmdDone`; `a.tick(5_000)`; `a.sdk_download("DSC04622.ARW", data)`; until `done`; `a.ftp().stor("DSC04622.ARW", data)`; until two rows; watcher + `tick(1_000)` | one photo in `s`; `root/<key>/DSC04622.ARW` (cable, `photo`) and `root/.duplicates/DSC04622.ARW` (wireless, `duplicate`) both read `data`; the real receiver and the real ledger (`a.ftp()`, `RoutingSink` → `LedgerService`) | M37, M38 ("whichever path delivered it first"); C4 (H-14) |
| C5(b) | first copy wins: FTP then cable | the order reversed (`stor` first, until `done`, then `sdk_download`) | the FTP copy is the photo (`via: wireless`, `root/<key>/`), the cable copy the duplicate (`.duplicates/`); still one photo | M37, M38 |
| C6(a) | watchdog raise sends `ftp-status` once | `ready_wifi.txt + capture.txt`; `activate(s)`; `a.capture()`; until one exposure row; no file; `a.clock.advance(61_000)`; `a.tick(1_000)` ×3 | `rec().stdin` has exactly one `ftp-status cmd_id=…` (negative settle after the third tick); `st() == NeedsAction { reason: FilesNotReceived { count: 1 }, holder: None }`; `H` `camera_files_not_received` detail exactly `1 photos taken but not received for 60 s` | R-I29 ("When it is first raised, the adapter sends `ftp-status`") |
| C6(b) | transfer in progress defers the raise | `ready_wifi.txt + capture.txt + capture.txt`; `activate(s)`; `a.capture()` twice (until two exposure rows); `f = a.ftp()`; `f.open_stor("DSC00030.ARW")` and send 1 byte (upload in flight, `uploads_in_progress() == 1`); `a.clock.advance(61_000)`; `a.tick(1_000)` ×2 | not raised: `st() == Connected { .. }`, no `ftp-status` in `rec().stdin`, no `camera_files_not_received`; then `f.finish_stor(..)` (completion at clock T0 + 63 s, FIFO-matched to one exposure, within 300 s) and `a.tick(1_000)` ×2 → `NeedsAction { FilesNotReceived { count: 1 } }` and exactly one `ftp-status` | R-I29 ("iff … no upload or download is in progress"); R-I26 rule 3 |
| C6(c) | dismiss | after C6(a): `CameraService::dismiss_files_not_received()` then `a.tick(0)`; **command half (mock webview, desktop `p46_tests`):** the same state through `with_camera_script`, then `camera_files_not_received_dismiss` | `st() == Connected { .., Wireless }`; `L.exposures(None)`'s row is `dismissed`; `H` has no `camera_files_not_received`; the command answers `Ok` and `health_get` no longer lists the code | R-I29 ("the photographer dismisses it (those exposures become `dismissed`)") |
| C6(d) | reconnect only on Wi-Fi | `ready_wifi.txt`, until `Connected`: `a.app.camera.reconnect()`; `ready_usb.txt`, until `Connected`: the same; `discovering.txt`: the same. **C++ half** (`tools/sidecar/build.sh --stub --test`): `TEST_CASE("reconnect reopens the wifi session once")` | Wi-Fi: `Ok(())` and `rec().stdin` has `reconnect cmd_id=…` exactly once; USB: `Err(ReconnectOnCableRefused)` and no `reconnect` line; no session: `Err(CameraNotConnected)`. C++: one `session.closed {reason:"reconnect requested"}`, a second `connection.ready {transport:"wifi"}`, one connect call, no USB attempt | R-I29 ("offered only while the SDK session is on Wi-Fi, because a reconnect on USB gives the camera to Capture One (V6)"); M39 |
| C6(e) | the camera's FTP report reaches the health screen | after C6(a): `a.inject("camera.ftp_state", {ftp_function: 1, ftp_connection_status: 2, request_ftp_transfer_result_error: "0x00000000"})` (field names `camera_properties.cpp:84–105, 205`; raw values); `a.tick(1_000)` | `H` `camera_files_not_received` detail exactly `1 photos taken but not received for 60 s · camera FTP: ftp_connection_status=2, ftp_function=1, request_ftp_transfer_result_error=0x00000000`; after `dismiss_files_not_received()` + `a.tick(1_000)` the item is gone and a later raise without a new `camera.ftp_state` has no ` · camera FTP:` suffix | R-I29 ("the adapter sends `ftp-status` and the health screen shows the result") |
| C7(a) | holder reported | `claimed.txt`; until `NeedsAction` | `st() == NeedsAction { reason: HeldByOtherProgram, holder: Some("ptpcamerad") }`; `H.has("camera_held")`; unit: `holder_from_owners` on `"pid 512, ptpcamerad"`, `"ptpcamerad"`, `"pid 512, ptpcamerad; pid 9, Other"` → `ptpcamerad` ×3 | MC-34 §Detect; R-I37 (naming the programs is P48); lab C1, §V6 (`UsbExclusiveOwner` shapes) |
| C8(a) | adapter selection | `TestApp::with_fake_document()` (no support folder) → `a.app.camera.capabilities()` and `st()`; `TestApp::with_camera_script(ready_usb.txt)` → the same | first: `depends_on_editing_provider: true` and `CameraStatus` from the fake document's `camera` (Capture One adapter), `rec().pids` empty; second: `depends_on_editing_provider: false`, `settings_write_store_destination: true`, the status of C1(a), `rec().pids.len() == 1` | 10E §Port changes (capabilities of the direct adapter); 10E §Camera support packages (adapter active iff support installed) |
| C8(b) | `camera_capture` command (mock webview, desktop `p46_tests`) | `with_camera_script(ready_usb + capture_usb)`, until `Connected`, `activate(s)`; `camera_capture`; then `camera_reconnect` | `{"capture_id": "<uuid>"}` and the exposure row has that id (after `CmdDone`); `camera_reconnect` → error `{"code":"reconnect_on_cable_refused"}` | MC-31 §Capture (`capture()` returns a `capture_id`); R-I29 |
| C8(c) | with the direct runtime bound, the arrival path takes the capture root from the settings formula, not from Capture One | **(i)** `with_camera_script(ready_usb + capture_usb)`; `activate(s)`; `a.fake.state`: `running = false`, `ops.clear()` (Capture One "not running", the P43 C6(b) pattern); `a.capture()`; `a.tick(5_000)`; `a.ftp().stor("DSC00020.ARW", data)`; until `done`; `a.watcher_event(final_path)` + `a.tick(1_000)` · **(ii)** `TestApp::with_fake_document()` (no support folder); `activate(s)`; `ops.clear()`; write `data` to `a.capture_root()/<key(s)>/DSC00020.ARW`; `a.watcher_event(path)` + `a.tick(1_000)` | (i) `photos(s).len() == 1` with `raw_path == final_path` (= `a.capture_root()/<key(s)>/DSC00020.ARW`); `a.fake.state.ops` contains no `document_info` · (ii) one photo with `raw_path == path` and `ops` contains `document_info` (the Capture One path still asks `inspect_document`) | R-I28 ("With the direct adapter, `ArrivalService` takes the capture root from the same formula as the receiver (MC-32 additions), so `final_path` and MC-2 compare like with like"); M38 |

## Delegated choices
How the adapter reaches `send_command` (`Weak<CameraRuntime>` or a command closure) and how the runtime hands it the batch; whether `AdapterState` is one
`Mutex` or atomics; how `CameraService` reaches the ledger for `dismiss_files_not_received` (`with_ledger(&LedgerService)` or a runtime call — L10); the
`TransportManager::reconnect()` detail (`intentional_disconnect("reconnect requested")` + immediate retry on the same transport); the uuid crate call site
(`uuid::Uuid::new_v4()` is already a dependency of core); how `ArrivalService` learns the adapter selection for C8(c) (recommended: `App::build` injects a
`capture_root: Arc<dyn Fn() -> Result<PathBuf> + Send + Sync>` chosen from `support::installed`, the same closure shape the receiver already takes, so
`ArrivalService` keeps no knowledge of adapters); the fake's inject tail cadence (20 ms suggested); the exact wording of helper-log lines other than
`download ignored: …`.

## Manual try-out
(Hardware-free unless the owner wants to: with the real sidecar built and a hand-installed support folder this phase already connects — that is P47's
job, do not do it here.) 1. Start the app without a support folder: everything is as before (Capture One camera adapter). 2. Health shows the camera
row as before. 3. No other visible change.

## Notes
- **Hard rules (owner):** never commit a Sony SDK file, the lab's `sdk_shim` headers, a `.env`, a credential or an SSH fingerprint; no Python; never change
  the camera's network/FTP settings (the store destination write is the one agreed setting, R-I19), delete a camera file (a zero-size or unmatched download
  is left in place) or kill any process; live view out of scope; the suite never needs the camera. Implemented by Claude agents (owner 2026-11-06, `PSM_IMPLEMENTER=claude`).
- **Planner readings (reversible):** (1) the camera's display name is `connection.ready.model` (P45 adds it), `Camera` when absent. (2) A `connect.failed`
  whose `error` text is unrecognised changes no `NeedsAction` (the transport manager keeps retrying). (3) `reconnect` is a new sidecar command, not the lab's
  watchdog hypothesis — "offered as a button, never done automatically" (MC-33) is P51's button over this command. (4) `HeldByOtherProgram` is cleared
  by our own `connection.ready usb` (we hold the interface then). (5) `DISCOVERING` names the link it is looking for (`Connecting { Cable }` with a cable in),
  read from the reason's suffix (L4). (6) `AuthenticationRefused`/`FingerprintMismatch` clear on a Wi-Fi `connection.ready` and on a new sidecar process
  (L12); a fingerprint `match`/`pinned` also clears the mismatch. (7) The FTP report is shown as a suffix of the `camera_files_not_received` detail (L13).
- **Hazards:** `timestamp_ms` of a script line is **whatever the script says**; of an injected line, what the test stamped (`a.clock.now_ms()`); it is never
  the fake's wall clock. `TestApp::with_camera_script` starts the clock at T0 = `SCRIPT_EPOCH_MS` so the scripts' stamps and the clock share one day (L1);
  rows compare `exposed_at` with the **event's** timestamp and advance the clock past t_e before any completion they want FIFO-matched. The staging folder
  path must be the one passed as `--save-dir` (P45 §Staging): `<binding document>/Capture/.incoming/sdk-<run id>/`, computed at the **first** spawn and
  never moved; `is_candidate` ignores it. A later settings save that moves the document to another volume makes the ledger's rename cross-device (`EXDEV`
  → `Err`, the file stays in staging, the card copy arrives by FTP) — P45 projection L14; not a row here. Health codes are set/cleared **from the derived
  inputs on every change**, never incrementally, or a cleared reason leaves a stale item; the adapter never writes `camera_files_not_received`. The receiver's
  `uploads_in_progress` is read on every watchdog tick. `CameraService::capture` for the direct adapter must not consult `document_not_front` (P42 C6(a)).
  The runtime clears `camera_sidecar_unavailable` on every non-empty batch; the adapter's `SdkUnavailable` set must run after it. After a sidecar exit the
  supervisor respawns after 1 s of mono time and the fake replays its script: rows that assert the post-exit state sample before advancing the clock.
- **Regression note (projection r0):** from this phase every earlier test built on `with_camera()`/`with_camera_script()` runs under the direct adapter and
  the runtime tick runs the watchdog glue. P44's manual `ledger().watchdog(…)` rows do not call `a.tick` and are unaffected; P45 `c5b` now also writes an
  exposure row (no active session), harmless. The L4 stamp is where this shows up.
- **Projection** done (r0, 2026-11-07): the event table is the adapter's whole semantics against externally owned event shapes; every script line above was
  checked against the C++ emit sites named in §Read first, and C5(a)/(b) run the **real** receiver and ledger.
- Registry additions made by the planner in master plan §6.4, §6.6, §6.7, §6.11. No intention-book change.
- Estimated size: 34 paths, ~1 900 lines (≈ 800 adapter + runtime + commands + C++, ≈ 1 100 tests + scripts).

## Review log


- **2026-11-06, carried from P42 projection r0** (planner): from this phase the runtime's `bind` re-publishes health from the adapter's own
  `StatusInputs` through `health_items` and the adapter emits `camera_status_changed` on every derived change — replacing P42's health copy and
  retry-only emission (L4, L5; C1(a)'s `ev()` and the health assertions are the rows); the adapter's `tick` runs under the runtime's serialised tick
  (L18: two App loops during a rebuild never double-process an event); the `--save-dir` becomes R-I20's staging folder (L16); every "`a.tick(0)` until …"
  is P42's `settle(a, pred)` (L7); `rec()` reads follow P42's positive/negative settle rule. C2(a) compares `exposed_at` with the **event's** timestamp.
- **2026-11-06, carried from P44 projection r0 (L18)** (planner, orchestrator ruling): R-I28's last sentence had no owning phase; P44 does not take
  it. This phase owns it as Task 2's capture-root paragraph and row C8(c) (25 → 26 rows; `services/arrival.rs` added to the perimeter, 20 → 21 paths).
- **2026-11-06, consistency edits from P45 projection r0** (planner, orchestrator ruling): `connect.failed` carries `error`, never `message` (L9) —
  `ConnectFailed { error, .. }`, `needs_action_from_connect_failed(error)`, the `claimed.txt`/`auth_refused.txt` fixtures rewritten to the two real emit
  shapes (`camera_connection.cpp:492–493`, `:511–512`), C4(c)'s unit input covers the `connection failed: ` prefix; `usb.retry_requested` is the variant
  `UsbRetryRequested` and `UsbFallbackToWifi` carries fields (L8); the staging path and the cross-device note (L14) in Hazards; code anchor `41bef14` →
  `635930b`. No row added or removed (26 rows, 21 paths unchanged).
- **2026-11-07, orchestrator amendments after projection r0 (L1–L22 folded; 0 owner cards)** (planner). L1: one time base — `with_camera_script` starts
  the clock at `SCRIPT_EPOCH_MS` (= the scripts' `2026-11-06T10:00:00Z`), injected lines are stamped by the test, FIFO rows advance the clock past t_e
  (fewest fixture edits: P45's `capture.txt` is reused unchanged). L2: `capture_usb.txt` in the real USB order with `file_transfer`; `bracketing`/
  `pixel_shift`/`other` → no change; C2(a) ends at `Busy { Transfer }`. L3: `Switching` survives `session.closed` and the intermediate `transport.state`
  lines; C1(d) injects the real sequence. L4: `DISCOVERING` link from the reason's suffix; C1(a) expects `Connecting { Cable }`. L5: `sleep 300` after
  sampled lines; two-write injection. L6: full field sets stated once in §Fixtures; C4(b)'s link is `usb`. L7: C3(d) → `.duplicates/DSC00008.ARW`. L8:
  `provisional` dropped from C2(b). L9: C6(b) uses two captures and the completion of a different file; C6(a)/(b) use `capture.txt`. L10: C6(c) after the next
  `a.tick(0)` (the `FilesNotReceived` input mirrors the ledger's raised flag on every tick); the ledger handle for `CameraService` delegated. L11: owned-code
  list; the ledger is the single writer of `camera_files_not_received`. L12: clear rules for `AuthenticationRefused`/`FingerprintMismatch`; the `Exited` reset;
  C1(c) asserts it. L13: the FTP report as a suffix of the `camera_files_not_received` detail through `LedgerShared.ftp_note` (new row **C6(e)**); the watchdog
  glue calls `request_ftp_status` through the port. L14: `protocol.rs`, `camera_ledger.rs`, the fake's inject tail, the four native paths and the desktop test
  module added; `CameraProvider::{reconnect, request_ftp_status}` are default methods (the two existing impls untouched, fail closed); 21 → **34 paths**
  (25 + 9 scripts). L15: C8(c)'s second half rewritten (a file in the session folder + `watcher_event`). L16: `≥ TRANSFER_EXPECTED_MS`, `a.tick(10_000)`. L17:
  `download ignored: <basename> (<reason>)` is contract text. L18: lifetime and selection sentence (`direct_adapter()` once per runtime; `App::build` selects
  up front). L19: one poller, the hash outside the supervisor lock. L20: holder parse rule (`holder_from_owners`). L21: `usb_fallen_back` and `pending_captures`
  dropped (P48 names neither); `capture()` order `SidecarUnavailable` → `CameraNotConnected`; the `SdkUnavailable` set runs after the runtime's clear. L22: C8(b)
  traces to MC-31 §Capture. Also: `TestApp::camera_status()` listed (§7.5 names it, HEAD lacks it); code anchor `635930b` → `e907dfb`; `projection: done`.
  26 → **27 rows**; §6.6 and §6.11 aligned (provider accessor, adapter lifetime, `LedgerShared.ftp_note`, the single writer).
- **2026-11-07, implement r1** (implementer, claude opus): built the plan's 34 paths. `DirectCameraAdapter` in `crates/integrations/src/camera/direct/`
  (event table, owned health codes, refusal clear rules, `Exited` reset, Save-to-PC deliveries with `normalize_camera_filename`, `download ignored: …`
  lines, busy `Transfer` to the completion or ≥ 10 s), created once per runtime (`CameraRuntime::direct_adapter()`, `Weak` back-reference for
  `send_command`); the runtime's `tick` keeps one `client.poll`, releases the supervisor lock before the adapter step (serialised by a separate tick
  lock), then runs the watchdog glue once per mono second and mirrors `LedgerShared.raised` into `FilesNotReceived` on every tick; `bind` keeps P42's
  copy and re-publishes the adapter's owned codes on top. `LedgerShared.ftp_note` + `set_ftp_note` + the ` · camera FTP: …` suffix; `CameraProvider::
  {reconnect, request_ftp_status}` default methods; `Command::Reconnect`; C++ `TransportManager::reconnect()` (at the next safe point) and `main.cpp`
  `reconnect` (`cmd.rejected {reason:"reconnect only on wifi"}` on USB), ctest 21/21; `support::installed`; `App::build` selects up front and injects
  the arrival capture-root closure; commands `camera_capture -> {capture_id}`, `camera_reconnect`, `camera_files_not_received_dismiss`; bindings and
  `services/camera.ts`. Tests: 25 in `p46_direct_camera_adapter.rs`, 2 desktop `p46_tests`, unit tests for the three Core mechanisms.
  **Assumptions** (handoff §Assumptions): `HeldByOtherProgram` cleared only by a **USB** `connection.ready` (planner reading 4, MC-34 "wins even while
  connected on the other link"); the C++ reconnect waits for busy `none`; `TestApp::inject_all` added for the plan's one-write injections; the fake's
  inject tail reads lines appended since the process started; settle bound 10 s. **Deviations:** C8(c) reopens the app (`kill_and_restart`) after
  `ops.clear()` in both halves — `ArrivalService` caches the root from its build-time scan, so the row as written could not fail (verified: the
  amended (i) fails with the closure removed); two lines outside the perimeter: `p45_sidecar_and_protocol.rs` C2(a) count 20 → 21 (the plan's new
  C++ test) and `p42_camera_runtime.rs` C7(a) on `with_fake_document()` (from this phase `with_camera()` selects the direct adapter, regression note).
  Stamp: `cargo test --workspace` 723/0, clippy `-D warnings`, fmt, `npm run typecheck`, `npm test` 369/369, `tools/sidecar/build.sh --stub --test` 21/0.
- **2026-11-07, review r1 — APPROVED_WITH_NOTES** (reviewer, opus). Gate: tracker `IMPLEMENTED`, checkpoint `64aa4f8` in `git log`; reviewed tree
  `05cf4f8` = `64aa4f8` + runner files only. **Perimeter:** `git show --stat 64aa4f8` = 38 paths = handoff list (plan's 34 + `p45` count line
  20→21 + `p42` C7(a) on `with_fake_document()` + tracker + this log); both extras are direct consequences of the plan (new C++ case; a support
  folder now selects the direct adapter). **Suite (reviewer run):** `cargo test --workspace` 723/0, clippy `-D warnings`, fmt, `npm run
  typecheck`, `npm test` 50 files 369/369, all exit 0. **Verified correct (by reading the production code, then the test):** `apply`/`apply_status`
  follow the event table row by row (DISCOVERING suffix rule, `Switching` survives `session.closed` and intermediate `transport.state`, store
  destination add/remove, held cleared on USB ready, refusals cleared on Wi-Fi ready and on `Started`); the `Exited` reset clears
  session/connecting/busy/expected transfer and `capture()` checks `sdk_unavailable` → session/helper_running in the planned order; downloads are
  refused in the order failed → size 0 → outside staging (canonical) → missing, logged as `download ignored: <base> (<reason>)`, and never moved or
  deleted; accepted ones are re-read, hashed and delivered with `normalize_camera_filename`; `≥ TRANSFER_EXPECTED_MS` deadline; `tick` releases the
  supervisor lock before the adapter step (serialised by `tick_serial`); `perform` sets `camera_sidecar_unavailable` after the per-batch clear;
  watchdog glue once per mono second with `uploads_in_progress() > 0 || transfer_expected()`, `ftp-status` through `provider()` on `first_raise`,
  `FilesNotReceived` mirrored from `LedgerShared.raised` on every tick; the ledger stays the single writer of `camera_files_not_received` (note
  suffix, cleared by `dismiss` and a non-raising run); `App::build` selects up front by `support::installed`, the same `Arc` reaches arrival,
  binding and handles, and the arrival closure uses `effective_document_id/Capture` = the receiver's `capture_root_fn`; C++ `reconnect` waits for
  busy `none`, closes with `reconnect requested` and retries Wi-Fi once; `main.cpp` rejects on USB. Every row C1(a)…C8(c) has a test at its
  boundary that would fail on a wrong outcome; C8(c)'s disclosed input change (reopen after `ops.clear()`, `restart(false)` so no startup scan)
  is what makes the row able to fail. **Assumptions** 1–12: none contradicts 10E (1 follows R-I37 "wins even while connected on the other
  link"; 2 follows MC-33 "at a safe point"). H1–H8, secrets and no-live-services: hold. **Notes** (backlog, 6): `via: Cable` for a Save-to-PC
  download on a Wi-Fi session vs R-I19's sentence; `ftp-status` re-fires after every transfer gap (P44 note still open) → P48; two texts for
  `camera_sidecar_unavailable` and the P42 failed-spawn note still open → P50/P51; held notice persists on Wi-Fi after the holder quits → P48;
  lesson: C8(c) as written could not fail (cached capture root); unknown `store_destination` still drops `connection.ready` → P47.
