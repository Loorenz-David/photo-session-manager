---
phase: P45_sidecar_and_protocol
title: Camera sidecar and protocol — the lab's C++ sidecar imported under native/camera-sidecar without any Sony file, built with CMake (stub backend from build.rs, mock-camera tests), the R-I19/R-I20/R-I23 sidecar changes, and the Rust protocol client tested against the fake sidecar replaying lab event lines
difficulty: high
projection: done
depends_on: [P44_capture_ledger]
---
# P45 — Camera sidecar import, build, and the protocol client (MC-31)

## Goal
Bring the lab's `crlab` into this repo as `native/camera-sidecar/` (10E §Shape, §Camera support packages, chapter 11, decision R45/D8): every file
of the lab's `sdk/` **except** `sdk/tests/sdk_shim/` (the hand-written SDK stand-in headers, never copied into this public repo) and nothing from
`vendor/`; a CMake project that builds the **stub** backend on any Mac and the SDK backend only where `--sdk <root>` points at an installed Sony SDK;
`apps/desktop/src-tauri/build.rs` builds the stub when the `externalBin` file is absent, so `cargo test` and `tauri build` work on a clean machine;
the lab's 17 mock-camera tests plus three new ones run through `ctest`. The sidecar changes the inventory asks for (MC-31 inventory additions
**R-I19** connect sequence, **R-I20** Save-to-PC staging, **R-I21** credentials over stdin, **R-I23** `usb.initial`, `usb-retry`, camera-specific
IOKit match, no restore on disconnect, quit-time Memory Card write, **R-I22** busy `Transfer` deferring only a voluntary stop). On the Rust side the
protocol: `SidecarEvent` parsing (unknown events and fields ignored), `Command` encoding, and `SidecarClient` over the P42 supervisor, proven against
the **fake sidecar** replaying the lab's real event lines (handoff §4.2) and lines shaped from the lab's emit sites.
**Not in this phase:** driving the camera (P46 interprets the events), the support-package install flow (P50), any run against the camera (the
SDK-only code paths are exercised at P47, owner-attended).

## Read first
Book: `10E_direct_camera_adapter.md` §Shape (camera sidecar), §Camera support packages (no vendor file in the repo; CI builds the stub), MC-31 (whole
incl. **R-I19, R-I20, R-I21, R-I22, R-I23**), §Owner answers D8 · `11_architecture_and_packaging.md` (one native sidecar, CMake, `externalBin`,
`NSLocalNetworkUsageDescription`) · `14_measurement_ledger.md` M39 · `15_decisions.md` R45 · evidence `sony_camera_lab.md` C2, C8, C9, C10, §V6 (store
destination verified over USB) · process `flow.md` §4, §6, §9 · master plan §3 (camera hard rules), §5 (P45 precondition: the lab repo readable; cmake),
§6.1 (`native/camera-sidecar/`, `tools/sidecar/`), §6.2, §6.9 (fixtures), **§6.11** (Sidecar: `SidecarSpec`, `Supervisor`, `SidecarEvent`, `Command`;
Runtime: `send_command`, `command_outcome`, `sidecar_events`), §7.1 (cmake 4.4.5, clang 17, the SDK's location), §7.2 (sidecar build commands).
Projection: `handoffs/reviewer/handoff_P45_sidecar_and_protocol_projection_r0.md` (decision ledger L1–L23, all folded below; its reality checks
verified the stub and SDK builds out of tree on this Mac).
Lab repo (read-only) at `221aa49`, `/Users/davidloorenz/Desktop/Developer/Camera-Connection-Tests/`:
- `docs/session-manager-handoff.md` §2 (what to take), **§4** (process, event envelope with 13 real lines, commands, meanings), §5 (building: `CRSDK_ROOT`
  glob, symbol probes `sdk_symbols.def`, runtime layout, `externalBin`), §12 (don'ts).
- `sdk/CMakeLists.txt` (whole; `CRLAB_SDK_SHIM` **dropped**, `CRSDK_ROOT` default **dropped**; the one `add_test(NAME transport_manager COMMAND crlab_tests)`
  at `:61`), `sdk/include/crlab/{camera.hpp, json_log.hpp, sha256.hpp, transport.hpp, usb_monitor.hpp}`, `sdk/src/{camera_capture.cpp, camera_connection.cpp,
  camera_discovery.cpp, camera_properties.cpp, diagnostics.cpp, json_log.cpp, main.cpp, sdk_adapter.cpp, sdk_internal.hpp, sdk_stub.cpp, sdk_symbols.def,
  sha256_macos.cpp, transport_manager.cpp, usb_monitor_macos.cpp, usb_watch_main.cpp}`, `sdk/tests/{mini_test.hpp, test_transport_manager.cpp}` (17
  `TEST_CASE`s; `mt_run_all` prints `<n> tests, <m> failed`).
- Emit sites the fixtures cite: `main.cpp:470` (`cmd.received {cmd, cmd_id}` — always first, `cmd_id` written even when empty), `:481` (`cmd.rejected
  {cmd, reason}`, no `cmd_id`), `:498` (`auto`'s `cmd.error {cmd, error}`, no `cmd_id`), `:500` (`cmd.done {cmd, cmd_id}`), `:473–475` (`transport.status`),
  `:525` (`sdk.unavailable`), `:250` (`crlab.version`); `transport_manager.cpp:81–87` (`transport.state {from, to, reason, transport}`, all four written),
  `:115` (`usb.arrived`/`usb.removed {detail}`), `:167–169` (`usb.fallback_to_wifi {usb_failures, usb_retry}`), `:249` (`usb.detected_by_discovery`),
  `:301–304` (`connection.ready {transport, connect_ms, total_ms, recovery_ms?}` — emitted by `TransportManager::try_connect`, **not** by the adapter);
  `usb_monitor_macos.cpp:183–185` (`usb.claimed_by {owners, hint}`; several owners joined with `"; "`); `camera_capture.cpp:173–178` (`sdk.download.completed
  {filename, size_bytes /* −1 when file_size fails */, success, transport, type?, provisional_capture_id?, ms_since_capture_request?}`), `:183–184`
  (`sdk.download.hashed`); `camera_connection.cpp:270–308` (`ssh.fingerprint {status ∈ not_pinned | dir_create_failed | match | MISMATCH | unknown |
  pinned | accepted_not_saved, success?}`), `:421` (`fetch_failed`), `:492–493` and `:511–512` (`connect.failed {transport, sdk_error, error, duration_ms,
  timeout?}` — the field is **`error`**, never `message`; `:23–30` `describe_error`); the env reads to remove: `main.cpp:128–130` (`CAMERA_SSH_USER`,
  `CAMERA_SSH_PASSWORD`; `CRLAB_FINGERPRINT_DIR` stays) and the usage text at `:56`; `main.cpp:139–163` (`StdinQueue`), `:430–509` (`cmd_auto`: adapter
  constructed from `base` at `:437`, first `tm.step()` at `:466` before stdin, busy `None` after `run_capture` at `:488`); `main.cpp:70`, `:91`
  (`parse_args` stores flags in a `std::map` — a repeated flag keeps the last value); `transport_manager.cpp:92–96` (`desired_transport`), `include/crlab/
  transport.hpp:158` (`usb_retry_at_`), `:10–13` (threading contract: inputs queued, adapter calls on the `step()` thread), `transport_manager.cpp:160–172`
  (fallback), `:194` and `:213` (`session_lost` and `intentional_disconnect` both call `adapter_.disconnect()`), `:317` (stop path), `camera_connection.cpp:
  143–150` (`capture.exposed` from `OnWarning`), `:548` (`restore_store_destination` on disconnect — **removed**), `camera_capture.cpp:139–146` (S1 unlock
  sleep ≈ 1 s), `:159–178` (`download_completed`), `:193` (`dest_value("pc")` → `CrStillImageStoreDestination_HostPC`, **removed**), `:206–262`
  (`configure_save`: `SetSaveInfo` + store destination verified ≤ 3 s — the model for the connect sequence), `:263–285` (`set_store_destination`, returns
  `bool`, 3000 ms hard-coded at `:275`), `usb_monitor_macos.cpp:69–77` (`on_added` counts `present` before any filter), `:135` (`UsbMonitor(vendor_id)` —
  the vendor is a **constructor** argument), `matching()` (add `idProduct`).
Code at `635930b` (+ P42 names): `apps/desktop/src-tauri/{tauri.conf.json, Info.plist, build.rs, Cargo.toml}`; `.gitignore`; P42's `Supervisor`
(`crates/integrations/src/camera/sidecar/supervisor.rs`: stdin is written only at spawn for `first_line`, `:63–67`, and for `quit`, `:141–146`),
`CameraRuntime` (`runtime.rs`: staging today `<data>/camera/staging/sdk-<run>` at `:272–286`; `capture_root_fn` = `canon(<document>/Capture)` at
`:178–181`; `helper_log()` at `:200`, capacity 50, stderr only), `fake_sidecar.rs` (prints `fake.started` at every start, `:49`), `TestApp::{with_camera,
with_camera_behaviour, camera_runtime (test_app.rs:273), fake_sidecar_recording, set_fake_sidecar_behaviour}`, the `Recording` struct (`stdin: Vec<(u32,
String)>`); `settle` is a **local** helper in `tests/p42_camera_runtime.rs:16–26` — this phase's test file defines its own.

## Files expected to change
Native (new, copied then edited): `native/camera-sidecar/CMakeLists.txt` (1), `native/camera-sidecar/include/crlab/*.hpp` (5), `native/camera-sidecar/src/*`
(15 incl. `sdk_internal.hpp`, `sdk_symbols.def`; +1 if the percent-decoder goes in a new `util.cpp`, see §Delegated choices), `native/camera-sidecar/tests/
{mini_test.hpp, test_transport_manager.cpp}` (2), `native/camera-sidecar/README.md` (1: what it is, how to build with and without the SDK, run
`tools/sidecar/build.sh` after any C++ edit, what must never be committed) — 24 native paths. Tools (new, 2): `tools/sidecar/build.sh`, `tools/sidecar/manifest.sh`
(§Tools below; run by the owner/orchestrator on the build machine before P50; the output holds hashes only).
Repo (changed, 4): `.gitignore` (`apps/desktop/src-tauri/binaries/`, `native/camera-sidecar/build/`, `target/camera-sidecar/`, `*.fingerprint`, `vendor/`,
`CrSDK*`), `apps/desktop/src-tauri/{tauri.conf.json, Info.plist, build.rs}`.
Integrations (new, 4): `crates/integrations/src/camera/sidecar/{protocol.rs, client.rs}`, `crates/integrations/src/camera/sidecar/fixtures/
{lab_auto_switching.jsonl, emit_sites.jsonl}`. Integrations (changed, 3): `crates/integrations/src/camera/sidecar/mod.rs`, `crates/integrations/src/camera/
sidecar/supervisor.rs` (`send_line(&str) -> io::Result<()>` — today the supervisor writes stdin only for `first_line` and `quit`), `crates/integrations/src/camera/
runtime.rs` (the client replaces raw line handling; `send_command`/`command_outcome`/`sidecar_events`; the `SidecarSpec` args `auto --save-dir <staging>
--usb-product-id 0x0E0C`, staging per §Staging).
Testkit (changed, 2): `crates/testkit/src/bin/fake_sidecar.rs` (scripted replay), `crates/testkit/src/test_app.rs` (`with_camera_script`, `sidecar_events()`).
Testkit (new, 2): `crates/testkit/src/fixtures/sidecar_scripts/{lab_lines.txt, capture.txt}`. Tests (new, 1): `crates/testkit/tests/p45_sidecar_and_protocol.rs`.
Derived: **33 new** (24 native + 2 tools + 4 integrations + 2 testkit scripts + 1 test), **9 changed** (4 repo + 3 integrations + 2 testkit) — **42 paths**
(43 with `util.cpp`). `serde_json` is already a dependency of psm-integrations; no `Cargo.toml` change.

## Core mechanisms (unit tests allowed)
`protocol.rs::{parse_line, Command::encode, percent_encode, iso_to_ms}`: inline tests named `mc31_<slug>` (rows C3, C4). The C++ percent-decoder in
`crlab_core` (row C6(c), a `TEST_CASE`). Everything else is a behaviour row (the fake sidecar, the build script, the manifest script).

## The sidecar changes (C++; each marked **mock-testable** or **SDK-only**)
1. **Credentials over stdin (R-I21; SDK-only in effect — the stub's `auto` exits 4 before stdin; observed at P47):** `cmd_auto` reads the **first** stdin
   line **synchronously** (`std::getline(std::cin, …)`, no timeout) **before** constructing `SdkCameraAdapter` (`main.cpp:437` copies `base` into the
   adapter, so a later assignment never reaches `base_`) and before `StdinQueue::start`. A line starting `credentials ` is percent-decoded into
   `base.ssh_user`/`base.ssh_password` from its `user=`/`password=` fields and `cmd.received {cmd: "credentials"}` is emitted with **no other field**
   (so `cmd_id` is absent on the wire). EOF before any line behaves as `quit` (exit 0, no connect). Any other first line is queued as the first ordinary
   command. An empty `user=` keeps the lab's "no SSH" behaviour (`camera_connection.cpp:411`). The env variables `CAMERA_SSH_USER`/`CAMERA_SSH_PASSWORD`
   are no longer read anywhere and leave the usage text (`main.cpp:56`) — grep-clean. `CRLAB_FINGERPRINT_DIR` stays an env variable (not secret). The
   percent-decoder is a pure function in `crlab_core` (`json_log.cpp` or a new `util.cpp` added to the `crlab_core` source list — **not** `main.cpp`'s
   anonymous namespace, which the test binary cannot link), so C6(c) tests it; a trailing `%`, `%4` or `%G1` is kept literally (no UB at end of string).
2. **`usb.initial { present }` (R-I23, SDK-free, observed at P47):** emitted once right after `mon.start` succeeds, from `mon.present_count() > 0`.
3. **`usb-retry` command (R-I23, mock-testable):** `TransportManager::usb_retry()` sets `usb_failures_ = 0`, `usb_retry_at_ = 0`, `failures_ = 0`,
   `next_attempt_ms_ = now` and emits `usb.retry_requested` (no fields beyond the envelope); `cmd_auto` accepts `usb-retry` whatever the state (before
   the `!connected` rejection). Whether `usb_retry()` queues an `Input` (the threading contract, `transport.hpp:10–13`) or mutates directly (`cmd_auto`
   runs on the step thread) is delegated; C6(a) observes the same either way.
4. **IOKit match names the camera (R-I23, SDK-free):** **one** flag `--usb-product-id <hex[,hex…]>` holding a comma-separated list of product ids
   (`parse_args` keeps only the last occurrence of a repeated flag, `main.cpp:91`), default `0x0E0C` when absent. The ids reach `UsbMonitor`'s
   **constructor** beside the vendor `0x054C` (`usb_monitor_macos.cpp:135`). How several ids match is delegated (§Delegated choices) under one constraint:
   the filter applies **before** `present` is counted in `on_added` (`:69–77`), so `usb.initial` agrees with arrivals; `usb_exclusive_owners` follows the same
   choice.
5. **Connect sequence (R-I19, SDK-only):** `--save-dir <dir>` reaches the adapter as a `ConnectOptions` member or an adapter field (delegated). After
   `session_.connect` succeeds inside `SdkCameraAdapter::connect` (both transports): `SetSaveInfo(dir)`, then `set_store_destination(usb ? "pc+camera" :
   "camera", 3000)` (verified ≤ 3 s, `configure_save`'s read-back loop). `set_store_destination` **returns the value it last read back** beside the
   verified flag (today `bool` only, `camera_capture.cpp:263–285`), and gains the timeout parameter (hard-coded 3000 at `:275`). `ConnectResult` gains
   two fields: `store_destination` (`"verified"` / `"failed"`, empty = not attempted) and `store_value` (`0x%08X`). **`connection.ready` is emitted by
   `TransportManager::try_connect` (`transport_manager.cpp:301–304`)**, which appends `model` (= `ep.model`) and, when non-empty, `store_destination` and
   `store_value` to the `ready` Fields. `MockCamera` leaves both empty, so the 17 lab tests are unchanged. `restore_store_destination` is deleted with its
   call at `camera_connection.cpp:548` and the `store_dest_changed` bookkeeping; the write is persistent. **"pc" alone is never written**: the `dest_value`
   mapping for `"pc"` (`camera_capture.cpp:193`) is removed (C1(d)).
6. **Quit during a USB session (R-I19, SDK-only):** new `virtual void CameraAdapter::before_voluntary_stop(Transport)` on `CameraAdapter` (default no-op,
   so `MockCamera` is untouched). `TransportManager::step` calls it in the stop path **after** the safe-point wait (change 7) and **before**
   `intentional_disconnect`, only when `active_ == Transport::Usb`. `SdkCameraAdapter` implements it as one `set_store_destination("camera", 1000)`
   (one attempt, ≤ 1 s). Nothing is written in `disconnect()`: `session_lost` calls the same `adapter_.disconnect()` (`:194`) and a write there would
   target a dead session for up to 3 s — the stall R-I19 removes.
7. **Busy `Transfer` and the deferred stop (R-I20/R-I22, mock-testable):** `CameraSession` gains `set_exposure_handler(fn(Transport))` and
   `set_download_handler(fn())`, modelled on `set_disconnect_handler` (`camera.hpp:109`), fired from `OnWarning`'s `capture.exposed` path
   (`camera_connection.cpp:143–150`) and from `download_completed` (`camera_capture.cpp:159–178`). In `cmd_auto`: an exposure on `Usb` → `tm.set_busy(
   BusyReason::FileTransfer)` and `transfer_deadline = now + 10000`; after `run_capture` the `None` is set **only if no USB exposure was seen during that
   command** (today `main.cpp:488` would overwrite the `FileTransfer` ≈ 700 ms after it is set); a download completed → `None` and the deadline cleared;
   the 100 ms loop clears on the deadline. Camera-initiated USB exposures take the same path. `request_stop()` while busy waits for the safe point up to
   10 s (`TransportManager::stop_wait_ms`, exposure in `TransportConfig` delegated) instead of disconnecting at once. **Deferred-stop semantics:** while a
   stop is pending, `step()` starts no connect or switch; the 10 s is measured from the step that drains the `Stop` input, with `>=`; a session lost
   meanwhile completes the stop at once. A transport switch away from USB happens only on cable loss, so busy never defers it.
8. Everything else stays as the lab left it (no idle disconnect, no reconnect on errors, `CrReconnecting_OFF`).

## `SidecarEvent` (exact variants; the registry §6.11 mirrors this table)
Envelope: every parsed event carries `timestamp_ms: i64` from `timestamp`. `iso_to_ms` accepts `YYYY-MM-DDTHH:MM:SS[.f{1,9}]Z` (fraction optional,
truncated to ms); a missing `timestamp` → `Err(ProtocolError::MissingTimestamp)`, any other form → `Err(ProtocolError::BadTimestamp)`, non-JSON or a known
event missing a **required** field → `Err(ProtocolError::Json)`. Required = written unconditionally at the cited emit site; everything else is `Option`.
Unknown fields are ignored; an unknown `event` → `Unknown { event }`. `Transport { Usb, Wifi, None }` from `"usb" | "wifi" | "none"`
(`transport_manager.cpp:13`). `StoreDestination { Verified, Failed }` from `"verified" | "failed"`.

| Variant | Fields (Rust types) | Site |
|---|---|---|
| `TransportState` | `{from: String, to: String, reason: String, transport: Transport}` | `transport_manager.cpp:81–87` |
| `ConnectionReady` | `{transport: Transport, connect_ms: Option<i64>, model: Option<String>, store_destination: Option<StoreDestination>, store_value: Option<String>}` | `:301–304` + change 5 |
| `SessionClosed` | `{transport: Transport, reason: String}` | `:216–220` |
| `SessionLost` | `{transport: Transport, reason: String, busy: String}` | `:180–186` |
| `CmdReceived` / `CmdDone` | `{cmd: String, cmd_id: Option<String>}` (absent or `""` → `None`) | `main.cpp:470`, `:500`; change 1 (`credentials` has no `cmd_id`) |
| `CmdRejected` | `{cmd: String, reason: String}` | `:481` |
| `CmdError` | `{cmd: String, error: String}` | `:498` |
| `CaptureRequested` | `{capture_id: String, transport: Transport, af: bool}` | `camera_capture.cpp:74` |
| `CaptureAf` | `{capture_id: String, af_result: String}` | handoff §4.2 line 6 |
| `CaptureAcknowledged` | `{capture_id: String, success: bool, sdk_error: String}` | `camera_capture.cpp:128` |
| `CaptureExposed` | `{capture_id: Option<String>, transport: Transport, association: String}` | `camera_connection.cpp:143–150` |
| `CaptureExposureMissing` | `{capture_id: String}` | `:140–142` |
| `CaptureUncertain` | `{operation: String}` | `transport_manager.cpp:189–192` |
| `CameraBusy` | `{reason: String, safe_point: bool}` | handoff §4.2 line 4 |
| `SwitchDeferred` | `{to: String, busy: String}` | §4.2 line 13 |
| `UsbArrived` / `UsbRemoved` | `{detail: String}` | `transport_manager.cpp:115` |
| `UsbInitial` | `{present: bool}` | change 2 |
| `UsbClaimedBy` | `{owners: String, hint: Option<String>}` | `usb_monitor_macos.cpp:183–185` |
| `UsbFallbackToWifi` | `{usb_failures: i64, usb_retry: String}` | `transport_manager.cpp:167–169` |
| `UsbRetryRequested` | unit | change 3 |
| `SdkDownloadCompleted` | `{filename: String, size_bytes: i64 /* −1 when file_size fails */, success: bool, transport: Transport, r#type: Option<String>, provisional_capture_id: Option<String>, ms_since_capture_request: Option<i64>}` | `camera_capture.cpp:173–178` |
| `SshFingerprint` | `{status: String, success: Option<bool>, fingerprint: Option<String>}` (P50 adds `fingerprint` in C++) | `camera_connection.cpp:270–308` |
| `CameraFtpState` | `{fields: serde_json::Map<String, Value>}` (every non-envelope member) | `main.cpp:496` |
| `SdkUnavailable` | `{error: String}` | `:525` |
| `ConnectFailed` | `{transport: Transport, sdk_error: String, error: String, timeout: Option<bool>}` (**`error`, not `message`**; the callback path, `:511–512`, writes `connection failed: <describe_error>` or `timed out waiting for OnConnected` plus `timeout`; the `Connect()`-return path, `:492–493`, writes `<describe_error>` and no `timeout`) | `camera_connection.cpp:492–493`, `:511–512` |
| `Unknown` | `{event: String}` (keeping the raw JSON is delegated) | — |

`sdk.download.hashed` and `usb.detected_by_discovery` have **no** variant (the adapter hashes files itself, R-I20): they parse to `Unknown`.

## The client and the runtime's test seam
`client.rs`: `SidecarClient { supervisor, pending: HashMap<String /* cmd_id */, Command>, events: Sender<SidecarEvent> }` — `send(Command) -> io::Result<CmdId>`
(writes `encode()` + `\n` through `Supervisor::send_line`), `poll() -> Vec<SidecarEvent>` (parses supervisor `Line`s; each unparsable line is logged once
to the helper log as `unparsable sidecar line: …` and skipped), `outcome(&str) -> Option<CommandOutcome { Done, Rejected { reason }, Error { error } }>`.
**Correlation** (the real `auto` emits `cmd.received {cmd, cmd_id}` first, then `cmd.rejected`/`cmd.error` **without** a `cmd_id`, then `cmd.done
{cmd_id}` — `main.cpp:470–500`, one command at a time): a `CmdRejected`/`CmdError` attaches to the most recent `CmdReceived` whose `cmd_id` has no
`CmdDone` yet; `CmdDone` yields `Done` only when nothing was attached (it never overwrites `Rejected`/`Error`); an absent or empty `cmd_id` is never
tracked. Whether `send` uses the `cmd_id` the `Command` carries or mints one and overwrites it is delegated under one constraint: **the returned `CmdId`
is the one written on the wire.** The `events` sender's receiver is P46's; in P45 the ring buffer alone is enough (delegated).
`CameraRuntime` (the client lives inside the runtime's `Mutex<Option<…>>`): `send_command(Command) -> Result<CmdId>`, `command_outcome(&str) ->
Option<CommandOutcome>`, `sidecar_events() -> Vec<SidecarEvent>` (the last 1 000 delivered events, also the Health page's helper log), reached in tests
through `TestApp::camera_runtime()`; `TestApp::sidecar_events()` is a one-line wrapper over it.

## Staging (`--save-dir`)
`<binding document>/Capture/.incoming/sdk-<run_id>`, computed at the **first** spawn from the binding's document and created with `create_dir_all`
(as today, `runtime.rs:272–286`; no canonicalize precondition — `capture_root_fn`'s `canon` can be `None`). It is never moved afterwards (R-I20 "new
for each sidecar process"; R-I18 no respawn on a settings save), so a later document on another volume makes P46's rename cross-device (`EXDEV` →
`Err`, the file left in staging, the card copy arrives by FTP) — noted in P46. P42's argv assertion (`p42_camera_runtime.rs:191–193`) still holds.

## Tools
`tools/sidecar/build.sh --stub | --sdk <root> [--test]`: configures into `target/camera-sidecar/{stub,sdk}`, builds, with `--test` runs `ctest
--output-on-failure --verbose` (the harness line `20 tests, 0 failed` appears only under `--verbose`; ctest registers the whole binary as **one** test,
so its own summary is `100% tests passed out of 1`), copies the binary to `apps/desktop/src-tauri/binaries/camera-sidecar-aarch64-apple-darwin`.
`tools/sidecar/manifest.sh <sdk root> <version> [<out file>]`: `find <sdk root> -name '*.dylib'`; a file whose parent directory is `CrAdapter` is listed
as `CrAdapter/<name>`, any other as its basename (the support-folder layout: `libCr_Core.dylib`, `libmonitor_protocol.dylib`, `libmonitor_protocol_pf.dylib`,
`CrAdapter/<file>`); sorted bytewise by `path`; `sha256` lowercase hex via `shasum -a 256`; output `{"version": "<version>", "files": [{"path", "sha256"}]}`
to `<out file>` (default `native/camera-sidecar/manifest/sony-<version>.json`). No Python; the output never holds a file's bytes.

## The fake sidecar (scripted replay)
`behaviour.json.script` names a text file; each line is one of: a JSON object → printed to stdout as is after substituting `{capture_id}` and
`{cmd_id}` with the values of the last `capture` command received; `sleep <ms>`; `await <cmd>` → block until a stdin line whose first word is
`<cmd>` arrives (recording it as always); `exit <code>`; `stderr <text>`. **In script mode the fake prints only script lines** (no `fake.started`),
and after the last line it continues as in P42 (records stdin, exits on `quit`). With no script the fake behaves as in P42. `TestApp::with_camera_script(text)`
writes the script and `behaviour.json` before `with_camera()`'s spawn. `TestApp::sidecar_events()` → `camera_runtime().sidecar_events()`.

## Ordered tasks
0. **Import.** Copy the lab's `sdk/` into `native/camera-sidecar/` **excluding** `sdk/tests/sdk_shim/` and `sdk/build/`; `cp -R` only — no Python; add
   `README.md`; `.gitignore` entries; verify with `git status` that nothing under `native/` is a Sony file (C1).
1. **CMake:** rename the project/binary (`camera_sidecar` / `camera-sidecar`), drop `CRLAB_SDK_SHIM`, make `CRSDK_ROOT` required only with
   `-DCAMERA_SIDECAR_WITH_SDK=ON`, keep the stub default, keep the POST_BUILD copy of Sony dylibs **only** in the SDK build and only into the build dir
   (never into the repo or the app bundle: the user installs them as a support package), keep `ctest` (one registered test, the whole binary).
2. **`tools/sidecar/build.sh`** and **`tools/sidecar/manifest.sh`** per §Tools.
3. **`build.rs`:** before `tauri_build::build()`, **only if the externalBin file is missing** run `tools/sidecar/build.sh --stub` (fail the build with a
   plain message if cmake is missing). `cargo:rerun-if-changed` paths are relative to `apps/desktop/src-tauri/`, so they are `../../../native/camera-sidecar/
   {src,include,CMakeLists.txt}` — or dropped (delegated), since a source rerun never rebuilds while the file exists; the README says to run
   `tools/sidecar/build.sh` after C++ edits (deleting the binary also retriggers: tauri-build emits `rerun-if-changed` for the externalBin source).
   `tauri.conf.json`: `bundle.externalBin: ["binaries/camera-sidecar"]`. `Info.plist`: `NSLocalNetworkUsageDescription` (`Session Manager talks to your
   camera over the local network to control it and receive photos.`).
4. **C++ changes** 1–7 above + three new `TEST_CASE`s (C6(a), C6(b), C6(c)).
5. **Rust protocol:** `protocol.rs` (`SidecarEvent` per the table, `parse_line`, `iso_to_ms`, `Command::encode`), `client.rs` and `Supervisor::send_line`
   per §The client; the runtime uses the client, exposes `send_command`/`command_outcome`/`sidecar_events`, passes `--save-dir` (§Staging) and
   `--usb-product-id 0x0E0C`; fixtures `lab_auto_switching.jsonl` (the 13 lines of handoff §4.2 verbatim) and `emit_sites.jsonl`.
6. Fake sidecar script mode; `TestApp` helpers; tests `p45_sidecar_and_protocol.rs` (one `static BUILD: OnceLock<std::process::Output>` shared by C2(a)
   and C2(b): cargo runs a file's tests in parallel threads and both need the same cmake build dir and the same `binaries/` file); suite, clippy, fmt,
   typecheck.

## Criteria (7 criteria, 21 rows; 4 `[rule]` rows)
`R` = repo root; `N` = `R/native/camera-sidecar`. `build()` = the `OnceLock`ed `tools/sidecar/build.sh --stub --test` run by the test with `R` as cwd (its
cmake build dir is cached under `target/camera-sidecar/stub`, so the second run is seconds). `rt` = `a.camera_runtime()`. Every "until …" is a `settle`.

| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | no vendor file in the tree | walk `N` recursively | no path matches `CameraRemote_SDK.h`, `Cr*.h`, `*.dylib`, `*.a`, `*.framework`, `sdk_shim`, `vendor`, `.env*`, `*.fingerprint`, `*.ARW`; the lab's five `include/crlab/*.hpp` and `src/sdk_stub.cpp` **are** present | R45 ("no vendor SDK file, nor the lab's hand-written SDK stand-in headers, ever enters the public repo"); D8; M39 ("the repo") |
| C1(b) | ignore rules | read `R/.gitignore` | contains `apps/desktop/src-tauri/binaries/`, `native/camera-sidecar/build/`, `*.fingerprint`, `vendor/`, `.env` (already) | R45; master plan §6.1 |
| C1(c) | no secret env read | grep `N/src` for `CAMERA_SSH_USER`, `CAMERA_SSH_PASSWORD` | zero matches (code and usage text); `CRLAB_FINGERPRINT_DIR` still read once; no `getenv` of a credential anywhere | R-I21 ("not through its environment") |
| C1(d) | *Host PC* alone is never written | grep `N/src` for `CrStillImageStoreDestination_HostPC` **not** followed by `AndMemoryCard` | zero matches (today's only use is `camera_capture.cpp:193`) | R-I19 ("The app never writes *Host PC* alone"); M39 ("none is *Host PC* alone") |
| C2(a) | stub build and C++ tests | `build()` | exit 0; `R/apps/desktop/src-tauri/binaries/camera-sidecar-aarch64-apple-darwin` exists and is executable; the output contains `20 tests, 0 failed` (17 lab + C6(a), C6(b), C6(c)) and `100% tests passed out of 1` | `[rule]` ch11 (packaging); 10E §Camera support packages ("CI builds the stub sidecar") |
| C2(b) | the stub refuses camera work loudly | after `build()`: run `R/apps/desktop/src-tauri/binaries/camera-sidecar-aarch64-apple-darwin` with `version`; then with `auto --save-dir /tmp/x` | `version` prints one JSON line with `"event":"crlab.version"`, `"sdk_compiled_in":false`; `auto` prints a line with `"event":"sdk.unavailable"` and exits with code 4 within 2 s | `[rule]` ch11; 10E ("CI builds the stub sidecar"); lab handoff §5.1 |
| C2(c) | manifest script over a synthetic tree | temp dir `T` with `T/lib/{libCr_Core.dylib, libmonitor_protocol.dylib, libmonitor_protocol_pf.dylib}`, `T/lib/CrAdapter/{libCr_PTP_IP.dylib, libCr_PTP_USB.dylib}`, `T/README.txt`, each file's bytes = its own relative path; `tools/sidecar/manifest.sh T 9.99.99 <out>` | exit 0; `<out>` parses as `{version: "9.99.99", files: [...]}` with exactly the paths `CrAdapter/libCr_PTP_IP.dylib`, `CrAdapter/libCr_PTP_USB.dylib`, `libCr_Core.dylib`, `libmonitor_protocol.dylib`, `libmonitor_protocol_pf.dylib` in that order, each `sha256` == the lowercase hex SHA-256 of the file's bytes (computed in the test); `README.txt` absent; nothing written under `N` | `[rule]` ch11 (packaging); R-I36 ("the list of runtime files and their SHA-256 … exact names taken from the manifest", consumed by P50) |
| C3(a) | the lab's real lines parse (unit) | the 13 lines of `fixtures/lab_auto_switching.jsonl` (handoff §4.2 verbatim) through `parse_line` | exactly `[TransportState{from:"DISCONNECTED", to:"DISCOVERING", reason:"looking for camera over wifi", transport:None}, ConnectionReady{transport:Wifi, connect_ms:Some(914), model:None, store_destination:None, store_value:None}, CmdReceived{cmd:"capture", cmd_id:Some("4db6be91")}, CameraBusy{reason:"exposure", safe_point:false}, CaptureRequested{capture_id:"53d7…", transport:Wifi, af:true}, CaptureAf{capture_id:"53d7…", af_result:"focused"}, CaptureAcknowledged{capture_id:"53d7…", success:true, sdk_error:"0x00000000"}, CaptureExposed{capture_id:Some("53d7…"), transport:Wifi, association:"latest_release_provisional"}, CmdDone{cmd:"capture", cmd_id:Some("4db6be91")}, UsbArrived{detail:"pid=0x0E0C ILCE-7RM5"}, SessionClosed{transport:Wifi, reason:"switching wifi -> usb"}, SessionLost{transport:Usb, reason:"usb device removed", busy:"none"}, SwitchDeferred{to:"usb", busy:"pixel_shift"}]` (whole-variant equality; `53d7…` is the literal shortened id); the second line's `timestamp_ms == 1_793_888_519_760` (`2026-11-05T14:21:59.760Z`) | MC-31 §Process ("stdout one JSON event per line"); lab handoff §4.2 (real samples) |
| C3(b) | emit-site lines parse (unit) | `fixtures/emit_sites.jsonl`, one line per cited site — `usb.claimed_by {owners:"pid 512, ptpcamerad", hint:"…"}` (`usb_monitor_macos.cpp:183–185`), `usb.fallback_to_wifi {usb_failures:3, usb_retry:"on next USB re-plug"}` (`transport_manager.cpp:167–169`), `sdk.download.completed {filename:"/…/.incoming/sdk-r1/DSC00007.ARW", size_bytes:51234567, success:true, transport:"usb", type:"0x00000001", provisional_capture_id:"c1", ms_since_capture_request:1960}` and `{filename:"/…/X.ARW", size_bytes:-1, success:false, transport:"usb"}` (`camera_capture.cpp:173–178`), `sdk.download.hashed {filename, sha256, hash_ms, success}` (`:183–184`), `ssh.fingerprint {status:"unknown", success:false, note:"…"}` and `{status:"MISMATCH", success:false}` (`camera_connection.cpp:283`, `:295–296`), `capture.uncertain {operation:"bracketing"}`, `capture.exposure_missing {capture_id:"c0"}`, `cmd.rejected {cmd:"capture", reason:"not connected (DISCOVERING)"}` (`main.cpp:481`), `cmd.error {cmd:"x", error:"unknown command"}` (`:498`), `sdk.unavailable {error:"…"}` (`:525`), `connect.failed {transport:"wifi", sdk_error:"0x00008208", error:"connection failed: connection error", duration_ms:5003, timeout:false}` (`camera_connection.cpp:511–512`) and `{transport:"usb", sdk_error:"0x00008208", error:"connection error", duration_ms:12}` (`:492–493`), `usb.initial {present:true}` and `connection.ready {transport:"usb", connect_ms:1200, total_ms:1500, model:"ILCE-7RM5", store_destination:"failed", store_value:"0x00000002"}` (this phase's own emits, changes 2 and 5) | whole-variant equality per line: `UsbClaimedBy{owners:"pid 512, ptpcamerad", hint:Some(..)}`; `UsbFallbackToWifi{usb_failures:3, usb_retry:"on next USB re-plug"}`; `SdkDownloadCompleted{filename, size_bytes:51234567, success:true, transport:Usb, r#type:Some("0x00000001"), provisional_capture_id:Some("c1"), ms_since_capture_request:Some(1960)}` and `{.., size_bytes:-1, success:false, transport:Usb, r#type:None, provisional_capture_id:None, ms_since_capture_request:None}`; `Unknown{event:"sdk.download.hashed"}`; `SshFingerprint{status:"unknown", success:Some(false), fingerprint:None}` and `{status:"MISMATCH", success:Some(false), fingerprint:None}`; `CaptureUncertain{operation:"bracketing"}`; `CaptureExposureMissing{capture_id:"c0"}`; `CmdRejected{cmd:"capture", reason:"not connected (DISCOVERING)"}`; `CmdError{cmd:"x", error:"unknown command"}`; `SdkUnavailable{error}`; `ConnectFailed{transport:Wifi, sdk_error:"0x00008208", error:"connection failed: connection error", timeout:Some(false)}` and `{transport:Usb, sdk_error:"0x00008208", error:"connection error", timeout:None}`; `UsbInitial{present:true}`; `ConnectionReady{transport:Usb, connect_ms:Some(1200), model:Some("ILCE-7RM5"), store_destination:Some(Failed), store_value:Some("0x00000002")}` | MC-31 (event meanings); rule 17 (shapes cited to the emit sites; `connect.failed` carries `error`, never `message`) |
| C3(c) | unknown is ignored, malformed is an error (unit) | `{"timestamp":"2026-11-06T00:00:00.000Z","source":"sdk","event":"camera.newthing","x":1}`; a `connection.ready` line with an extra field `"foo":"bar"`; `not json`; `{"source":"sdk","event":"connection.ready","transport":"wifi","connect_ms":1}` (only the timestamp missing); `{"timestamp":"2026-11-06T00:00:00Z","source":"sdk","event":"usb.initial","present":false}` (no fraction); `{"timestamp":"2026-11-06 00:00:00","source":"sdk","event":"usb.initial","present":false}`; `{"timestamp":"2026-11-06T00:00:00.000Z","source":"sdk","event":"connection.ready"}` (a known event missing its required `transport`) | `Unknown { event: "camera.newthing" }`; `ConnectionReady` as normal; `Err(ProtocolError::Json)`; `Err(ProtocolError::MissingTimestamp)`; `UsbInitial{present:false}` with `timestamp_ms == 1_793_923_200_000`; `Err(ProtocolError::BadTimestamp)`; `Err(ProtocolError::Json)` | MC-31 §Process ("unknown fields and events ignored"); lab handoff §4.2 (envelope: `timestamp` ISO-8601 UTC `Z`) |
| C4(a) | credentials encoding (unit) | `Command::Credentials { user: "u name", password: "p/w=&é" }.encode()` | `credentials user=u%20name password=p%2Fw%3D%26%C3%A9` (RFC 3986 unreserved kept) | R-I21 ("percent-encoded") |
| C4(b) | capture encoding (unit) | `Command::Capture { id: "11111111-…", af: true, cmd_id: "c1" }.encode()` | `capture id=11111111-… af=1 cmd_id=c1` | MC-31 §Capture ("Half-press, AF wait up to 3 s, release"); lab handoff §4.3 |
| C4(c) | the other commands (unit) | `UsbRetry{cmd_id:"c2"}`, `FtpStatus{cmd_id:"c3"}`, `Props{cmd_id:"c4"}`, `Status{cmd_id:"c5"}`, `Quit` | `usb-retry cmd_id=c2`, `ftp-status cmd_id=c3`, `props cmd_id=c4`, `status cmd_id=c5`, `quit` | R-I23 (`usb-retry`); R-I29 (`ftp-status`); MC-31 §Process (`quit`) |
| C5(a) | events flow from the fake to the client in order | `a = TestApp::with_camera_script(lab_lines.txt)` (the 13 lines, no sleeps); `a.tick(0)` until 13 events | `a.sidecar_events()` equals C3(a)'s sequence (13 entries, nothing before them — the fake prints no `fake.started` in script mode); the fake's `stdin.log` first line is the credentials line (P42 C4(a) still holds) | MC-31 §Process; R-I21 |
| C5(b) | command correlation | script `capture.txt`: `await capture`, then `cmd.received {cmd:"capture",cmd_id:"{cmd_id}"}`, `capture.requested {capture_id:"{capture_id}", transport:"wifi", af:true}`, `capture.acknowledged {capture_id:"{capture_id}", success:true, sdk_error:"0x00000000"}`, `capture.exposed {capture_id:"{capture_id}", transport:"wifi", association:"latest_release_provisional"}`, `cmd.done {cmd:"capture",cmd_id:"{cmd_id}"}`; `id = rt.send_command(Command::Capture{id: <uuid>, af: true, cmd_id: ..})?`; `a.tick(0)` until a `CmdDone` | `rec().stdin` holds the line `capture id=<uuid> af=1 cmd_id=<id>` exactly once (`<id>` = the returned `CmdId`); `rt.command_outcome(&id) == Some(Done)`; the `CaptureExposed` event carries `Some(<uuid>)` | MC-31 §Capture; lab handoff §4.3 ("Every command is wrapped in `cmd.received` … `cmd.done` carrying your `cmd_id`") |
| C5(c) | rejected command | script: `await capture`, `cmd.received {cmd:"capture",cmd_id:"{cmd_id}"}`, `cmd.rejected {cmd:"capture", reason:"not connected (DISCOVERING)"}` (no `cmd_id`, as `main.cpp:481`), `cmd.done {cmd:"capture",cmd_id:"{cmd_id}"}`; `id = rt.send_command(Command::Capture{..})?`; `a.tick(0)` until a `CmdDone` | `rt.command_outcome(&id) == Some(Rejected { reason: "not connected (DISCOVERING)" })` (the later `cmd.done` does not overwrite it) | lab handoff §4.3 ("Commands sent while disconnected get `cmd.rejected`"); MC-31 §Process |
| C5(d) | a malformed line does not stop the stream | script: `{bad`, then a valid `transport.state` line | `a.sidecar_events()` holds exactly the one `TransportState`; `rt.helper_log()` holds exactly one `unparsable sidecar line` entry | MC-31 §Process ("unknown … ignored") |
| C6(a) | `usb_retry()` after fallback (C++ `TEST_CASE("usb-retry gives usb a fresh chance without a re-plug")`, run by C2(a)) | rig with `fail_usb_connects = 3`, USB plugged, `run(15000)` → fallback (`usb.fallback_to_wifi`, 3 `connect:usb` recorded, on Wi-Fi); `tm->usb_retry()`; `run(5000)` | after the retry: `usb.retry_requested` logged, the mock records a 4th `connect:usb`, state `ConnectedUsb`, `cam.active_sessions == 1`; control rig without the retry: `run(60000)` records no further `connect:usb` (still 3) | R-I23 (`usb-retry` "clears the three-failure fallback"); MC-31 §Transport policy ("no periodic USB probing") |
| C6(b) | stop waits for a safe point (C++ `TEST_CASE("stop is deferred while busy, bounded by 10 s")`) | connected on USB; `set_busy(FileTransfer)`; `request_stop()`; `run(3000)` → still connected (no `session.closed`); `set_busy(None)`; `run(500)` → closed. Second rig: busy never cleared; `run(9_900)` → still connected; `run(600)` → closed (deadline measured from the step that drained the stop, `>=`) | as stated; `log.count("session.closed") == 1` in both rigs | R-I20 ("That busy state only defers a voluntary stop"); R-I22 |
| C6(c) | credentials percent-decoding (C++ `TEST_CASE("credentials percent-decoding reverses the app's encoding")`, in `crlab_core`) | decode `u%20name`, `p%2Fw%3D%26%C3%A9`, `abc%`, `abc%4`, `abc%G1` | `u name`, `p/w=&é`, `abc%`, `abc%4`, `abc%G1` (malformed escapes kept literally, no UB at end of string) | R-I21 ("percent-encoded"; a decode slip shows as `AuthenticationRefused`, never retried — MC-31 §Fingerprint) |
| C7(a) | packaging wiring | read `apps/desktop/src-tauri/tauri.conf.json` and `Info.plist` | `bundle.externalBin == ["binaries/camera-sidecar"]`; `Info.plist` has `NSLocalNetworkUsageDescription` with a non-empty string | `[rule]` ch11 (`externalBin`; `NSLocalNetworkUsageDescription`); R-I36 spike S4 |

## Delegated choices
Whether `native/camera-sidecar/README.md` repeats the lab's build notes or links `docs/intention/evidence/sony_camera_lab.md`; the C++ percent-decoder's
file **within `crlab_core`** (`json_log.cpp` or a new `util.cpp` added to the `crlab_core` source list; never `main.cpp`); how `save_dir` reaches
`SdkCameraAdapter` (a `ConnectOptions` member or an adapter field); the `stop_wait_ms` field's exposure in `TransportConfig`; whether `usb_retry()` is a
queued `Input` or a direct mutation on the step thread (L18; C6(a) is the same either way); **the IOKit product-id matching** (L15): one `IOPropertyMatch`
dictionary per id, or a vendor match plus a product filter applied before `present` is counted — `usb_exclusive_owners` follows the same choice; whether
`build.rs` shells out to the script or runs cmake itself, and whether it keeps the `../../../native/camera-sidecar/…` rerun paths or drops them (L19);
whether `send_command` uses the `Command`'s `cmd_id` or mints one (the returned id is the wire id, L12); the receiver of `events: Sender<SidecarEvent>`
(none in P45 — the ring buffer suffices, L23); the fixtures' file names beyond the two above; `SidecarEvent::Unknown` keeping the raw JSON; the helper-log
capacity.

## Manual try-out
(Hardware-free.) 1. In Terminal: `tools/sidecar/build.sh --stub --test` — the output contains `20 tests, 0 failed` and `100% tests passed out of 1`.
2. `apps/desktop/src-tauri/binaries/camera-sidecar-aarch64-apple-darwin version` prints one JSON line saying the SDK is not compiled in. 3. Start the app:
nothing changes on screen; `git status` shows no `binaries/` or build output (ignored).

## Notes
- **Hard rules (owner):** **never commit a Sony SDK file, the lab's `sdk_shim` headers, a `.env`, a credential or an SSH fingerprint** — C1 is the check,
  the reviewer re-runs `git ls-files native/` by eye; no Python (the lab's Python is not copied); never change the camera's network/FTP settings (the
  sidecar writes only the store destination, R-I19, which the photographer agreed to in setup — P51 says so), never delete a camera file or kill any
  process; live view out of scope; the suite never needs the camera. Implemented by Claude agents (owner 2026-11-06, `PSM_IMPLEMENTER=claude`).
- **SDK-only code is compiled here only if the implementer runs `tools/sidecar/build.sh --sdk /Users/davidloorenz/Desktop/Developer/Camera-Connection-Tests/
  vendor/sony-sdk`** (the SDK is on this Mac in the lab's checkout; the projection built the unmodified lab out of tree against it: 61 probes present,
  `sdk_version:"2.02.00"`, 0 warnings). The implementer **should** run that build once to prove the R-I19/R-I20 edits compile (recorded in the handoff;
  nothing of it is committed), but the suite never depends on it. If the build cannot run, the edits ship compile-checked against nothing and P47 is
  where they are first exercised — the handoff must say which.
- **Planner readings (reversible):** (1) the binary keeps the lab's internal `crlab` namespace and event names (renaming them would invalidate the
  real fixtures); only the executable and CMake project are renamed. (2) `connection.ready` gains `model` so the status can show the camera's name. (3)
  `usb.retry_requested` is a new event acknowledging the command (variant `UsbRetryRequested`; P46 clears `usb_fallen_back` on it). (4) The product id
  list defaults to the A7R V's `0x0E0C`; another body means another id (the ZV-E10 is out of scope).
- **Hazards:** `tauri-build` 2.7.4 validates `bundle.externalBin` at compile time (the file must exist with the target-triple suffix; `ResourcePathNotFound`
  otherwise — verified by the projection) — hence `build.rs` runs the stub build **before** `tauri_build::build()`. The stub's `auto` exits 4 immediately
  (7 ms, before stdin): with the stub installed as a "support package" the runtime would restart it on backoff and show `camera_sidecar_unavailable` —
  correct, and P50 refuses to install a stub (`version` must report an SDK). cmake's first configure takes ~20–40 s: C2(a) is the slow test of the
  suite; cache under `target/`. The `StdinQueue` pushes `quit` on EOF — the Rust side must keep stdin open for the sidecar's lifetime. An in-tree build
  under `N/build/` would trip C1(a)'s `*.a` pattern: `build.sh` builds under `target/` only. `tauri.conf.json` declares `minimumSystemVersion: "13.0"`
  while the lab CMake pins `CMAKE_OSX_DEPLOYMENT_TARGET 14.1` (backlog, not blocking: irrelevant on the dev and work Macs).
- **Projection** done (r0, 2026-11-06): every C3(b) line was re-read against its emit site; `connect.failed`'s field is `error`; the 13 real lines hold
  verbatim; the stub and SDK builds were run out of tree. Nothing remains for the implementer to re-validate about the fixtures.
- Registry additions made by the planner in master plan §6.1, §6.2, §6.9, §6.11, §7.2; §6.11 re-aligned after projection r0. No intention-book change.
- Estimated size: 42 paths; ~3 600 lines copied C++ (unchanged), ~350 lines of C++ edits and tests, ~1 000 lines Rust (protocol, client, fake, tests), ~200
  lines shell/CMake.

## Review log


- **2026-11-06, carried from P42 projection r0** (planner): the fake's `behaviour.json` and the script live in the recording dir the fake derives from its
  own location (`<data dir>/camera/sony/test/rec/`, L2); `with_camera_script` writes both **before** the spawn (L1); `Recording.argv` is `Vec<Vec<String>>`
  from `argv.<pid>.json` (L19) and `exits` records voluntary exits (L9); every "`a.tick(0)` until …" in C5 is P42's `settle(a, pred)` (L7); the credentials
  line is built from the runtime's cached credentials (L17). C1(c)'s grep must also cover the new percent-decoder: no `getenv` of a credential anywhere.
- **2026-11-06, orchestrator amendments after projection r0 (L1–L23 folded; 0 owner cards)** (planner). L1 change 5 re-homed: the adapter does the
  writes and returns `store_destination`/`store_value` in `ConnectResult`; `try_connect` emits them with `model`. L2 change 6: `CameraAdapter::
  before_voluntary_stop(Transport)` in the stop path, nothing in `disconnect()`. L3 change 7: exposure/download handlers on `CameraSession`, `None` after
  `run_capture` only without a USB exposure, 10 s deadline owned by the loop. L4 change 1: the first stdin line is read synchronously before the adapter
  is constructed. L5 decoder in `crlab_core` + C6(c) (ctest count 20). L6 C2(a) asserts `20 tests, 0 failed` and `100% tests passed out of 1` under
  `--verbose`. L7 one `OnceLock` build for C2. L8 the `SidecarEvent` table (per-variant required/optional, `CaptureAf`, `UsbFallbackToWifi` fields,
  `UsbRetryRequested`, optional `cmd_id`, signed `size_bytes`, `Transport::None`); `CmdError {cmd, error}` added for the C3(b) line and the client's
  `Error` outcome; `sdk.download.hashed` parses to `Unknown`. L9 `ConnectFailed.error` (never `message`), both emit paths in C3(b); P46 edited to match.
  L10 no `fake.started` in script mode; `iso_to_ms` fraction optional, `BadTimestamp` (C3(c) rows). L11 C5(c) script carries `cmd.received`; correlation
  rule in §The client. L12 `CameraRuntime::{send_command, command_outcome, sidecar_events}` via `TestApp::camera_runtime()`; minting delegated. L13
  `supervisor.rs` (`send_line`) in the perimeter. L14 §Staging. L15 one comma-separated `--usb-product-id`, matching delegated. L16 C3(c) one-fault
  fixture. L17 deferred-stop semantics (C6(b) rigs). L18, L23 delegated explicitly. L19 build.rs rerun paths. L20 `manifest.sh` spec + C2(c). L21
  `[rule] ch11` on C2(a)/C2(b). L22 C1(d). Lab citations re-anchored (`:115`, `:481`, `:498`, `:128–130`, `:139–163`, `transport.hpp:158`, constructor
  vendor). Code anchor `41bef14` → `635930b`. Counts: 18 → 21 rows (C1(d), C2(c), C6(c)); 1 → 4 `[rule]` rows; 36 → 42 paths. Frontmatter
  `projection: done`.
- **2026-11-06, implement r1** (implementer, Claude Opus; handoff `handoffs/implementer/handoff_P45_sidecar_and_protocol_implement_r1.md`).
  Built: the lab's `sdk/` (at `221aa49`, minus `tests/sdk_shim/`; the lab has no `sdk/build/`) copied with `cp -R` into `native/camera-sidecar/`
  (23 files + README); CMake renamed (`camera_sidecar` / `camera-sidecar`, tests `camera_sidecar_tests`, ctest `transport_manager`),
  `CRLAB_SDK_SHIM` and the `CRSDK_ROOT` default dropped, `-DCAMERA_SIDECAR_WITH_SDK=ON` requires `CRSDK_ROOT`; `tools/sidecar/build.sh` and
  `manifest.sh`; `build.rs` runs the stub build when `binaries/camera-sidecar-<TARGET>` is missing (rerun paths dropped); `externalBin`;
  `NSLocalNetworkUsageDescription`. C++ changes 1–7 as specified; three new `TEST_CASE`s; `tools/sidecar/build.sh --stub --test` →
  `20 tests, 0 failed`, `100% tests passed out of 1`. **SDK build run once** (`--sdk …/Camera-Connection-Tests/vendor/sony-sdk`): configures
  (60 probes present, `DCVersion` absent), compiles with 0 warnings, `version` → `sdk_version:"2.02.00"`, `auto` with an empty stdin → exit 0
  without connecting; nothing of it committed, the stub rebuilt afterwards. Rust: `protocol.rs` (`SidecarEvent` per the table, `Stamped`,
  `parse_line`, `iso_to_ms`, `percent_encode`, `Command::encode`), `client.rs` (`SidecarClient`, `CommandOutcome`, correlation rule),
  `Supervisor::send_line`, runtime seam (`send_command`, `command_outcome`, `sidecar_events` ring of 1 000), staging
  `<document>/Capture/.incoming/sdk-<run_id>`, `--usb-product-id 0x0E0C`; fake sidecar script mode; `TestApp::{with_camera_script,
  sidecar_events}`. Assumptions and deviations: see the handoff (notably `Stamped` wrapper for `timestamp_ms`; `poll(now, side)` signature;
  no `events: Sender` field yet; the `StoreDestHostPC` probe line removed from `sdk_symbols.def` for C1(d)).
