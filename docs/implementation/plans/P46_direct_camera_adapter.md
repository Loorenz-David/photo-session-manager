---
phase: P46_direct_camera_adapter
title: Direct camera adapter — the Sony-SDK-sidecar CameraProvider over the protocol client: status from events, capture with ids, exposures and Save-to-PC deliveries into the ledger, store-destination outcome, busy and transport policy as reported, the watchdog's ftp-status and Wi-Fi-only reconnect, first copy wins end to end, adapter selection
difficulty: high
projection: yes
depends_on: [P45_sidecar_and_protocol]
---
# P46 — Direct camera adapter (MC-31 behaviour, MC-33 integration)

## Goal
`DirectCameraAdapter` (`crates/integrations/src/camera/direct/`), the `CameraProvider` that drives the sidecar through P45's client and turns its
events into the neutral model: `CameraStatus` (via P42's `derive_status`), `exposure observed` → `CaptureSink::exposure_observed` (app- and
camera-initiated), `sdk.download.completed` → re-read, hash, `Delivery { via: Cable }` → the ledger (R-I20), `connection.ready.store_destination:
failed` → `NeedsAction { StoreDestinationUnverified }` (R-I19), `camera.busy`/`transport.state` → `Busy` (R-I22), `capture()` → the `capture` command
and its `CaptureId`, the Sony filename normalisation (`_N`, R-I26), `usb.claimed_by` → `NeedsAction { HeldByOtherProgram, holder }` (the naming of
programs is P48), the watchdog's first raise → `ftp-status`, reconnect only on Wi-Fi (R-I29), transfer-in-progress for the watchdog; the runtime selects
this adapter whenever a camera support package is installed and keeps `CameraViaCaptureOne` otherwise. **First copy wins** is proven end to end: a
Save-to-PC copy and the same bytes by FTP through the real receiver become one photo and one `.duplicates/` file, in either order (M37, M38).
**Taken from P44 projection r0 (L18):** R-I28's last sentence — with the direct runtime bound, `ArrivalService` takes the capture root from the
settings formula (the receiver's, P43 C6(b)), never from Capture One — is this phase's Task 2 and row C8(c).
**Not in this phase:** naming the programs behind `ptpcamerad`, the start order, the launch gate and "Take the camera back" (P48); the collection switch
and style (P49); settings, Keychain UI, fingerprint confirmation and support install (P50/P51); anything against the camera.

## Read first
Book: `10E_direct_camera_adapter.md` §What the photographer gets, §Shape, §Port changes (R-I37, R-I33), MC-31 (incl. **R-I19, R-I20, R-I22, R-I23**),
MC-33 (**R-I26** normalisation by the adapter, **R-I29** `ftp-status`, reconnect on Wi-Fi only), §Invariants M37–M39 · `14_measurement_ledger.md` M37–M39 ·
evidence `sony_camera_lab.md` C3 (ack ≠ exposure ≠ file), C4, C9, §V6 · process `flow.md` §4, §6, §9 · master plan §3, §6.4, §6.6, §6.7 (`camera_capture`
→ `{capture_id}`, `camera_reconnect`, `camera_files_not_received_dismiss`, `camera_status_changed`), **§6.11** (Domain, Ports, Mechanisms `camera_status`,
Services `LedgerService`, Runtime, Sidecar `SidecarEvent`, Direct adapter, Support `installed`), §7.5.
Code at `41bef14` (+ P42–P45 by their §6.11 names): `crates/core/src/services/camera.rs` (P42 version), `crates/integrations/src/camera/
via_capture_one.rs`, `crates/integrations/src/camera/runtime.rs` (`bind`, `provider` slot), `crates/integrations/src/camera/sidecar/{protocol.rs,
client.rs}`, `crates/core/src/services/camera_ledger.rs` (`watchdog` → `WatchdogState.first_raise`), `crates/infra/src/ftp` (`ReceiverHandle::
uploads_in_progress`), `apps/desktop/src-tauri/src/commands/camera.rs`, `apps/desktop/src/services/camera.ts`, `crates/testkit/src/bin/fake_sidecar.rs`
(script mode), `crates/testkit/src/test_app.rs` (`with_camera_script`, `sidecar_events`, `ledger`, `ftp`).
Lab (read-only): `docs/session-manager-handoff.md` §4.4 (event meanings), §8 rows 5, 6, 13; `sdk/src/camera_connection.cpp:23–30` (`describe_error`:
`ssh user authentication failed (check Access Authen. user/password)` ↔ `AuthenticationRefused`, `ssh server authentication failed (fingerprint mismatch)`
↔ `FingerprintMismatch`; the texts the `connect.failed.message` carries), `sdk/src/transport_manager.cpp:302–304` (`connection.ready` emit),
`sdk/src/camera_capture.cpp:160–185` (`sdk.download.completed`).

## Files expected to change
Integrations (new): `crates/integrations/src/camera/direct/{mod.rs, adapter.rs, filenames.rs}`. Integrations (changed): `crates/integrations/src/camera/
{mod.rs, runtime.rs}` (adapter selection, the adapter's tick), `crates/integrations/src/camera/support/mod.rs` (`installed(paths) -> Option<InstalledSupport>`:
version folder + `camera-sidecar` present; the manifest check is P50's), `crates/integrations/src/lib.rs`.
Core (changed): `crates/core/src/ports/camera.rs` (`reconnect`, `request_ftp_status`), `crates/core/src/services/camera.rs` (`capture() -> CaptureId`,
`reconnect()`, `dismiss_files_not_received()`), `crates/core/src/error.rs` (`CameraNotConnected`, `ReconnectOnCableRefused`, `SidecarUnavailable`),
`crates/core/src/services/arrival.rs` (`capture_root` from the settings formula while the direct adapter is selected, L18).
Desktop (changed): `apps/desktop/src-tauri/src/commands/camera.rs` (`camera_capture -> CaptureResponse { capture_id }`, `camera_reconnect`,
`camera_files_not_received_dismiss`), `apps/desktop/src-tauri/src/lib.rs` (`collect_commands!`), `apps/desktop/src/services/bindings.ts` (regenerated),
`apps/desktop/src/services/camera.ts` (`cameraCapture` returns the id; callers ignore it), `apps/desktop/src-tauri/src/commands/items.rs` (error codes).
Testkit (changed): `crates/testkit/src/test_app.rs` (`with_camera_script`, `capture()`, `sdk_download(name, bytes)` writes into the staging folder and
appends the completion line through the script channel — see §Fixtures), `crates/testkit/src/fixtures/sidecar_scripts/*.txt` (new scripts). Tests (new):
`crates/testkit/tests/p46_direct_camera_adapter.rs`.
Estimated: 3 new source paths, 14 changed, 1 new test, new script fixtures — 21 paths.

## Core mechanisms (unit tests allowed)
`direct/filenames.rs::normalize_camera_filename` (rows C2(d)); `direct/adapter.rs::needs_action_from_connect_failed(message) -> Option<NeedsActionReason>`
(C4(c)). Everything else is a behaviour row through `TestApp`.

## The adapter contract
- **State** (`Mutex<AdapterState>`): `status_inputs: StatusInputs`, `expected_transfer: Option<(Instant, Option<CaptureId>)>`, `usb_fallen_back: bool`,
  `staging_dir`, `pending_captures: HashMap<CmdId, CaptureId>`. `tick(now)`: `client.poll()` → for each event the table below; after the batch,
  `derive_status` → if changed: `events.emit(CameraStatusChanged { status })` and `health` re-derived with `health_items` (set/clear every camera code).
- **Event table:** `TransportState { to }`: `DISCOVERING | CONNECTING_WIFI → connecting = Some(Wireless)`, `CONNECTING_USB → Some(Cable)`, `SWITCHING →
  busy = Some(Switching)`, `DISCONNECTED | RECOVERING | ERROR → connecting = None`, `CONNECTED_* → connecting = None` · `ConnectionReady { transport, model,
  store_destination }` → `session = Some((model or "Camera", link))`, `busy = None`, `store_destination == Some(Failed)` → add `StoreDestinationUnverified`,
  `Some(Verified)` → remove it; `HeldByOtherProgram` removed (the interface is ours now) · `SessionClosed | SessionLost` → `session = None`, `busy = None`,
  `expected_transfer = None` · `CameraBusy { reason: "exposure" }` → `busy = Some(Exposure)`; `"none"` → `None` · `CaptureExposed { capture_id, transport,
  association }` → `sink.exposure_observed(ExposureObserved { capture_id: capture_id.filter(|_| !association.starts_with("none")), at: timestamp_ms,
  initiated: if association starts with "none" { Camera } else { App }, provisional: association == "latest_release_provisional" })`; when `transport ==
  usb`: `expected_transfer = Some((now, capture_id))`, `busy = Some(Transfer)`, `sink.transfer_in_progress(Cable, true)` · `SdkDownloadCompleted {
  filename, size_bytes, success }`: `success && size_bytes > 0 && Path(filename).starts_with(staging_dir)` → `size = fs::metadata(filename).len()`,
  `sha256_file`, `sink.delivery(Delivery { camera_filename: normalize_camera_filename(basename), written_path, sha256, size, via: Cable, completed_at:
  timestamp_ms })`; then `expected_transfer = None`, `busy = None` (if it was `Transfer`), `transfer_in_progress(Cable, false)`; otherwise log and leave
  the file (never promoted, never deleted) · `expected_transfer` older than `TRANSFER_EXPECTED_MS` → cleared the same way (R-I20 "at most 10 s") ·
  `CaptureUncertain` → `sink.session_uncertain()` · `UsbClaimedBy { owners }` → add `HeldByOtherProgram` with `holder = process name after the last
  comma of the first owner` (`"pid 512, ptpcamerad"` → `"ptpcamerad"`) · `UsbFallbackToWifi` → `usb_fallen_back = true` (cleared by `UsbArrived` and by
  `usb.retry_requested`) · `SshFingerprint { status: "unknown" }` → add `FingerprintUnconfirmed`; `"MISMATCH"` → add `FingerprintMismatch`; `"match" |
  "pinned"` → remove both · `ConnectFailed { message }`: `needs_action_from_connect_failed` (contains `ssh user authentication failed` → `AuthenticationRefused`;
  contains `fingerprint mismatch` → `FingerprintMismatch`; else `None`) · `SdkUnavailable` → the helper is useless: `Error::SidecarUnavailable` on
  `capture()`, health `camera_sidecar_unavailable` · `CmdRejected`/`CmdDone` → `client.outcome`. `helper_running` = the supervisor has a live child.
- **`capture()`**: `session.is_none()` → `Err(CameraNotConnected)`; else `id = CaptureId(uuid)`, `client.send(Capture { id, af: true, cmd_id })`, return
  `Ok(id)` at once (`capture.acknowledged`/`exposed` are events, C3: "neither means a file exists"). `capabilities()` = `{ status, capture, transfer,
  exposure_events, settings_read, settings_write_store_destination: true, depends_on_editing_provider: false }`. `raw_extensions() = ["arw"]`.
- **`reconnect()`** (R-I29): allowed only while `session == Some((_, Wireless))` → `client.send(Command::Reconnect)` (new sidecar command `reconnect`:
  the transport manager closes and reopens the current Wi-Fi session at the next safe point — **C++ edit in this phase**, mock-testable: a `TEST_CASE`
  "reconnect reopens the wifi session once" added to the native tests); on USB → `Err(ReconnectOnCableRefused)`; no session → `Err(CameraNotConnected)`.
- **Watchdog glue** (runtime tick, every second of manual/real time): `transfer_active = receiver.uploads_in_progress() > 0 || expected_transfer.is_some()`;
  `state = ledger.watchdog(now, transfer_active)`; `state.first_raise` → `client.send(FtpStatus)`; `FilesNotReceived { count }` present in `status_inputs`
  iff `state.raised`. `CameraService::dismiss_files_not_received()` → `ledger.dismiss()`.
- **Selection** (runtime `bind`): `support::installed(paths)` → `Some` → the direct adapter (spawned sidecar, receiver) is the `provider`; `None` →
  `CameraViaCaptureOne` as before. `AppHandles.camera` reads the runtime's current provider.

## Fixtures (testkit)
Scripts under `fixtures/sidecar_scripts/`: `ready_usb.txt` (`transport.state → DISCOVERING`, `connection.ready {transport:"usb", model:"ILCE-7RM5",
store_destination:"verified", store_value:"0x0003"}`), `ready_wifi.txt`, `ready_usb_store_failed.txt`, `capture_usb.txt` (`await capture`, `cmd.received`,
`camera.busy {reason:"exposure"}`, `capture.requested`, `capture.acknowledged`, `capture.exposed {capture_id:"{capture_id}", transport:"usb", association:
"latest_release_provisional"}`, `camera.busy {reason:"none"}`, `cmd.done`), `camera_shutter.txt` (`capture.exposed {transport:"wifi", association:"none: no
SDK capture pending (camera-initiated?)"}` with no `capture_id`), `claimed.txt` (`usb.claimed_by {owners:"pid 512, ptpcamerad", hint:"…"}`, `connect.failed
{transport:"usb", sdk_error:"0x00008208", message:"connection error"}`), `fingerprint_unknown.txt`, `auth_refused.txt` (`connect.failed {…, message:"ssh user
authentication failed (check Access Authen. user/password)"}`). `TestApp::sdk_download(name, bytes)` writes `bytes` to `<staging>/<name>` and makes the fake
print `sdk.download.completed {filename:<path>, size_bytes:<len>, success:true, transport:"usb"}` (the fake reads an `inject` file the test appends to; the
timestamp is the fake's clock). `a.capture()` = `CameraService::new(provider, health).capture()`.

## Ordered tasks
0. **Registry first** (§6.11): errors, `CameraProvider::{reconnect, request_ftp_status}`, `Command::Reconnect` (+ C++ `reconnect` command and its test),
   `InstalledSupport`/`installed`, `CaptureResponse`.
1. `direct/filenames.rs` + unit tests; `direct/adapter.rs` per §The adapter contract; `needs_action_from_connect_failed`.
2. Runtime: adapter selection, the adapter's `tick`, the watchdog glue, `provider()` accessor used by `App::build` for `App.camera`/`AppHandles.camera`.
   **Capture root (P44 projection L18, R-I28):** while the direct adapter is selected, `ArrivalService::capture_root()` resolves `<settings document>/
   Capture` by the receiver's formula (P43 C6(b)) and never calls `EditingProvider::inspect_document`; with `CameraViaCaptureOne` it asks Capture One as
   today. The cache rule (`services/arrival.rs:57–68`) is unchanged. Row C8(c).
3. `CameraService`: `capture() -> Result<CaptureId>`, `reconnect`, `dismiss_files_not_received`; commands + bindings; `services/camera.ts`.
4. Testkit fixtures and helpers; tests `p46_direct_camera_adapter.rs`; suite, clippy, fmt, typecheck, `npm test`.

## Criteria (8 criteria, 26 rows; 0 `[rule]` rows)
`a = TestApp::with_camera_script(<script>)`; `st()` = `a.camera_status()` (`CameraService::status`); `ev()` = the `CameraStatusChanged` payloads in
`a.events`; `H` = health; `L = a.ledger()`; `s` the active session; `rec()` the fake's recording.

| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | status from a USB session | `ready_usb.txt`; `a.tick(0)` until the ready event | `st() == Connected { name: "ILCE-7RM5", link: Cable }`; `ev()` ends with `Connecting { link: Wireless }` (from `DISCOVERING`) then `Connected {..}`; `H` has ok `camera` with detail `ILCE-7RM5 · cable` and no `camera_disconnected` | R-I37; 10E §What the photographer gets (the dot shows camera and link) |
| C1(b) | Wi-Fi session | `ready_wifi.txt` | `Connected { name, link: Wireless }` | R-I37 |
| C1(c) | session lost → connecting; sidecar dead → disconnected | `ready_usb.txt` then injected `session.lost {transport:"usb", reason:"usb device removed", busy:"none"}` and `transport.state {to:"DISCOVERING"}`; then the fake exits (`inject exit 3`) and `a.tick(0)` | `Connecting { link: Wireless }`; after the exit `Disconnected`; `H` has `camera_disconnected` | R-I37 ("`Connecting` (sidecar running, no session) › `Disconnected`"); R-I23 (cable pull closes a USB session) |
| C1(d) | switching is busy | injected `transport.state {from:"CONNECTED_WIFI", to:"SWITCHING", reason:"usb arrived"}` after `ready_wifi.txt` | `Busy { reason: Switching }`; then `connection.ready usb` → `Connected { link: Cable }` | R-I22 ("the adapter reports `Switching` during a transport change") |
| C2(a) | `capture()` returns the id and records the exposure | `ready_usb.txt` + `capture_usb.txt`; `activate(s)`; `id = a.capture()`; `a.tick(0)` until `cmd.done` | `rec().stdin` has `capture id=<id> af=1 cmd_id=…`; `L.exposures(None)` has one row `{ capture_id: Some(id), initiated: app, session_id: s, status: expecting, exposed_at: <the event's timestamp_ms> }`; while the `camera.busy exposure` line is the last processed, `st() == Busy { Exposure }`; afterwards `Connected` | MC-31 §Capture (`capture()` returns a `capture_id`; "neither means a file exists"); MC-33 (exposure rows); C3 |
| C2(b) | camera-initiated shot | `ready_wifi.txt` + `camera_shutter.txt` | one exposure `{ capture_id: None, initiated: camera, provisional: false }` | MC-33 ("Camera-initiated exposures are shots, not orphans"); lab §8 row 13 |
| C2(c) | capture refused without a session | script `transport.state DISCOVERING` only; `a.capture()` | `Err(CameraNotConnected)`; nothing written to stdin but the credentials line; no exposure row | MC-31; master plan §6.6 |
| C2(d) | Sony re-send suffix (unit) | `normalize_camera_filename` on `DSC01234_1.ARW`, `DSC01234_12.ARW`, `DSC01234_123.ARW`, `DSC01234.ARW`, `IMG_1234.CR3`, `DSC01234__dup1.ARW` | `DSC01234.ARW`, `DSC01234.ARW`, `DSC01234_123.ARW` (three digits is not the suffix), `DSC01234.ARW`, `IMG_1234.CR3` (no trailing digits before `_`), `DSC01234__dup1.ARW` (core's suffix is not touched here) | R-I26 ("the adapter strips them … Sony: `_<1–2 digits>` after the stem's trailing number") |
| C3(a) | Save-to-PC completion becomes a cable delivery | `ready_usb.txt` + `capture_usb.txt`; `id = a.capture()` (exposure at `t_e`); `a.sdk_download("DSC00007.ARW", data)`; `a.tick(0)` | `L.deliveries()` has `{ camera_filename: "DSC00007.ARW", via: cable, sha256: sha256(data), size: len, association: fifo, outcome: photo, state: done }`; the file is at `root/<key(s)>/DSC00007.ARW`; the staging folder `root/.incoming/sdk-<run id>/` is empty; after `a.watcher_event` + `tick(1000)` one photo in `s` with `captured_at == t_e` | R-I20 ("A file is handed to the ledger only on `sdk.download.completed` with `size_bytes > 0`. The adapter then re-reads the size, hashes the file itself … and reports `via: Cable`"); M38 |
| C3(b) | busy `Transfer` between exposed and completion | as C3(a), sampling `st()` after `capture.exposed` and before `sdk_download` | `Busy { reason: Transfer }`; after the completion `Connected`; with no completion and `a.clock` advanced 10 s → `Connected` (cleared at `TRANSFER_EXPECTED_MS`) | R-I20 ("While a download is expected … at most 10 s … the sidecar holds busy `Transfer`") |
| C3(c) | zero or missing download is never promoted | `inject` `sdk.download.completed {filename:<staging>/X.ARW, size_bytes:0, success:true}`; then a line naming a file that does not exist | no delivery row; the staging folder is untouched; the helper log has one `download ignored` entry each | R-I20 ("A file in `.incoming/` without a completion event … is never promoted and never deleted") |
| C3(d) | a `_1` re-send by cable keys to the original | `sdk_download("DSC00008_1.ARW", data)` after a Wireless delivery of `DSC00008.ARW` with `data` | outcome `duplicate` (same `file_key` `DSC00008.ARW`, same hash) → `.duplicates/DSC00008_1.ARW` | R-I26; R-I27; M37 |
| C4(a) | store destination failed | `ready_usb_store_failed.txt` | `st() == NeedsAction { reason: StoreDestinationUnverified, holder: None }`; `H.has("camera_store_destination_unverified")`; the session is kept (`capture()` still sends the command) | R-I19 ("`failed` → `NeedsAction { StoreDestinationUnverified }`, and the session is kept"); M39 ("Every store-destination write is verified") |
| C4(b) | verified clears it | after C4(a): inject `session.lost` then `connection.ready {store_destination:"verified"}` | `Connected`; the health item is gone | R-I19 |
| C4(c) | connect failures map to reasons (unit + boundary) | `needs_action_from_connect_failed` on the two `describe_error` texts and on `connection error`; boundary: `auth_refused.txt` | `Some(AuthenticationRefused)`, `Some(FingerprintMismatch)`, `None`; `st() == NeedsAction { AuthenticationRefused }` with `H` detail `The camera refused the password` | 10E §What the photographer gets ("The camera refused the password"); R-I37; lab `camera_connection.cpp:23–30` |
| C4(d) | fingerprint unknown | `fingerprint_unknown.txt` (`ssh.fingerprint {status:"unknown", success:false}`) | `NeedsAction { FingerprintUnconfirmed }`; detail `Confirm the camera's fingerprint`; a later `{status:"pinned"}` clears it | MC-31 §Fingerprint; R-I37 |
| C5(a) | first copy wins: cable then FTP | `ready_usb.txt` + `capture_usb.txt`; `a.capture()`; `a.sdk_download("DSC04622.ARW", data)`; `a.ftp().stor("DSC04622.ARW", data)`; ticks + watcher | one photo in `s`; `root/<key>/DSC04622.ARW` (cable) and `root/.duplicates/DSC04622.ARW` (wireless) both read `data`; two delivery rows `photo` / `duplicate` | M37, M38 ("whichever path delivered it first"); C4 (H-14) |
| C5(b) | first copy wins: FTP then cable | the order reversed | the FTP copy is the photo, the cable copy the duplicate; still one photo | M37, M38 |
| C6(a) | watchdog raise sends `ftp-status` once | `ready_wifi.txt`; `a.capture()` (exposure); no file; `a.clock` + 61 s; `a.tick(1_000)` ×3 | `rec().stdin` has exactly one `ftp-status cmd_id=…`; `st() == NeedsAction { FilesNotReceived { count: 1 } }`; `H` detail `1 photos taken but not received for 60 s` | R-I29 ("When it is first raised, the adapter sends `ftp-status`") |
| C6(b) | transfer in progress defers the raise | as C6(a) but `a.ftp()` holds an open `STOR` (1 byte sent) while the clock passes 61 s | not raised; after the upload completes (a different file) and the next tick → raised | R-I29 ("iff … no upload or download is in progress") |
| C6(c) | dismiss | `CameraService::dismiss_files_not_received()` (and the command `camera_files_not_received_dismiss`) | status back to `Connected`; the exposure row is `dismissed` | R-I29 |
| C6(d) | reconnect only on Wi-Fi | `ready_wifi.txt`: `a.app.camera.reconnect()` → `Ok` and `rec().stdin` has `reconnect cmd_id=…`; `ready_usb.txt`: → `Err(ReconnectOnCableRefused)` and no line; no session → `Err(CameraNotConnected)` | as stated | R-I29 ("offered only while the SDK session is on Wi-Fi, because a reconnect on USB gives the camera to Capture One (V6)"); M39 |
| C7(a) | holder reported | `claimed.txt` | `st() == NeedsAction { reason: HeldByOtherProgram, holder: Some("ptpcamerad") }`; `H.has("camera_held")` | MC-34 §Detect; R-I37 (naming the programs is P48) |
| C8(a) | adapter selection | `TestApp::with_fake_document()` (no support folder) → `a.app.camera.capabilities()`; `TestApp::with_camera_script(ready_usb)` → the same | `depends_on_editing_provider: true` and `CameraStatus` from the fake document's `camera` (Capture One adapter); vs `depends_on_editing_provider: false`, `settings_write_store_destination: true` and the status of C1(a); `rec().pids.len() == 1` only in the second case | 10E §Port changes (capabilities of the direct adapter); 10E §Camera support packages (adapter active iff support installed) |
| C8(b) | `camera_capture` command (mock webview) | `with_camera_script(ready_usb + capture_usb)`; `camera_capture` | `{"capture_id": "<uuid>"}` and the exposure row has that id; `camera_reconnect` → error `{"code":"reconnect_on_cable_refused"}` | master plan §6.7 |
| C8(c) | with the direct runtime bound, the arrival path takes the capture root from the settings formula, not from Capture One | `with_camera_script(ready_usb + capture_usb)`; `activate(s)`; `a.fake.state`: `running = false`, `ops.clear()` (Capture One "not running", the P43 C6(b) pattern); `a.capture()`; `a.ftp().stor("DSC00020.ARW", data)`; until the row is `done`; `a.watcher_event(final_path)` + `a.tick(1_000)` | `photos(s).len() == 1` with `raw_path == final_path` (= `a.capture_root()/<key(s)>/DSC00020.ARW`); `a.fake.state.ops` contains no `document_info` (the arrival path never asked Capture One); with `TestApp::with_fake_document()` (no support folder) the same arrival still goes through `inspect_document` | R-I28 ("With the direct adapter, `ArrivalService` takes the capture root from the same formula as the receiver (MC-32 additions), so `final_path` and MC-2 compare like with like"); M38 |

## Delegated choices
How the fake receives `inject` lines (a file the test appends to and the fake tails, recommended); whether `AdapterState` is one `Mutex` or atomics;
the exact helper-log texts; the C++ `reconnect` implementation detail (`intentional_disconnect("reconnect requested")` + immediate retry on the same
transport); the uuid crate call site (`uuid::Uuid::new_v4()` is already a dependency of core); how `ArrivalService` learns the adapter selection for
C8(c) (recommended: `App::build` injects a `capture_root: Arc<dyn Fn() -> Result<PathBuf> + Send + Sync>` chosen from the runtime's selection, the same
closure shape the receiver already takes, so `ArrivalService` keeps no knowledge of adapters).

## Manual try-out
(Hardware-free unless the owner wants to: with the real sidecar built and a hand-installed support folder this phase already connects — that is P47's
job, do not do it here.) 1. Start the app without a support folder: everything is as before (Capture One camera adapter). 2. Health shows the camera
row as before. 3. No other visible change.

## Notes
- **Hard rules (owner):** never commit a Sony SDK file, the lab's `sdk_shim` headers, a `.env`, a credential or an SSH fingerprint; no Python; never change
  the camera's network/FTP settings (the store destination write is the one agreed setting, R-I19), delete a camera file (a zero-size or unmatched download
  is left in place) or kill any process; live view out of scope; the suite never needs the camera. Implemented by Claude agents (owner 2026-11-06, `PSM_IMPLEMENTER=claude`).
- **Planner readings (reversible):** (1) the camera's display name is `connection.ready.model` (P45 adds it), `Camera` when absent. (2) A `connect.failed`
  with an unrecognised message changes no `NeedsAction` (the transport manager keeps retrying). (3) `reconnect` is a new sidecar command, not the lab's
  watchdog hypothesis — "offered as a button, never done automatically" (MC-33) is P51's button over this command. (4) `HeldByOtherProgram` is cleared
  by our own `connection.ready usb` (we hold the interface then).
- **Hazards:** `timestamp_ms` of the fake's lines is the fake's wall clock while `a.clock` is manual — C2(a) compares `exposed_at` with the **event's**
  timestamp, not `a.clock`. The staging folder path must be the one passed as `--save-dir` (P45): `root/.incoming/sdk-<run id>/`; `is_candidate` ignores it.
  Health codes are set/cleared **from the derived inputs on every change**, never incrementally, or a cleared reason leaves a stale item. The receiver's
  `uploads_in_progress` is read on every watchdog tick. `CameraService::capture` for the direct adapter must not consult `document_not_front` (P42 C6(a)).
- **Projection** is on: the event table above is the adapter's whole semantics against externally owned event shapes; the projectionist checks each row's
  script against P45's fixtures and the C++ emit sites, and that C5(a)/(b) run the **real** receiver and ledger.
- Registry additions made by the planner in master plan §6.4, §6.6, §6.7, §6.11. No intention-book change.
- Estimated size: ~20 paths, ~1 700 lines (≈ 700 adapter + runtime + commands, ≈ 1 000 tests + scripts).

## Review log


- **2026-11-06, carried from P42 projection r0** (planner): from this phase the runtime's `bind` re-publishes health from the adapter's own
  `StatusInputs` through `health_items` and the adapter emits `camera_status_changed` on every derived change — replacing P42's health copy and
  retry-only emission (L4, L5; C1(a)'s `ev()` and the health assertions are the rows); the adapter's `tick` runs under the runtime's serialised tick
  (L18: two App loops during a rebuild never double-process an event); the `--save-dir` becomes R-I20's staging folder (L16); every "`a.tick(0)` until …"
  is P42's `settle(a, pred)` (L7); `rec()` reads follow P42's positive/negative settle rule. C2(a) compares `exposed_at` with the **event's** timestamp.
- **2026-11-06, carried from P44 projection r0 (L18)** (planner, orchestrator ruling): R-I28's last sentence had no owning phase; P44 does not take
  it. This phase owns it as Task 2's capture-root paragraph and row C8(c) (25 → 26 rows; `services/arrival.rs` added to the perimeter, 20 → 21 paths).
