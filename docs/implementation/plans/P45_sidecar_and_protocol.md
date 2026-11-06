---
phase: P45_sidecar_and_protocol
title: Camera sidecar and protocol — the lab's C++ sidecar imported under native/camera-sidecar without any Sony file, built with CMake (stub backend from build.rs, mock-camera tests), the R-I19/R-I20/R-I23 sidecar changes, and the Rust protocol client tested against the fake sidecar replaying lab event lines
difficulty: high
projection: yes
depends_on: [P44_capture_ledger]
---
# P45 — Camera sidecar import, build, and the protocol client (MC-31)

## Goal
Bring the lab's `crlab` into this repo as `native/camera-sidecar/` (10E §Shape, §Camera support packages, chapter 11, decision R45/D8): every file
of the lab's `sdk/` **except** `sdk/tests/sdk_shim/` (the hand-written SDK stand-in headers, never copied into this public repo) and nothing from
`vendor/`; a CMake project that builds the **stub** backend on any Mac and the SDK backend only where `--sdk <root>` points at an installed Sony SDK;
`apps/desktop/src-tauri/build.rs` builds the stub when the `externalBin` file is absent, so `cargo test` and `tauri build` work on a clean machine;
the lab's 17 mock-camera tests plus two new ones run through `ctest`. The sidecar changes the inventory asks for (MC-31 inventory additions
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
§6.1 (`native/camera-sidecar/`, `tools/sidecar/`), §6.2, §6.9 (fixtures), **§6.11** (Sidecar: `SidecarSpec`, `Supervisor`, `SidecarEvent`, `Command`),
§7.1 (cmake 4.4.5, clang 17, the SDK's location), §7.2 (sidecar build commands).
Lab repo (read-only) at `221aa49`, `/Users/davidloorenz/Desktop/Developer/Camera-Connection-Tests/`:
- `docs/session-manager-handoff.md` §2 (what to take), **§4** (process, event envelope with 13 real lines, commands, meanings), §5 (building: `CRSDK_ROOT`
  glob, symbol probes `sdk_symbols.def`, runtime layout, `externalBin`), §12 (don'ts).
- `sdk/CMakeLists.txt` (whole; `CRLAB_SDK_SHIM` **dropped**, `CRSDK_ROOT` default **dropped**), `sdk/include/crlab/{camera.hpp, json_log.hpp, sha256.hpp,
  transport.hpp, usb_monitor.hpp}`, `sdk/src/{camera_capture.cpp, camera_connection.cpp, camera_discovery.cpp, camera_properties.cpp, diagnostics.cpp,
  json_log.cpp, main.cpp, sdk_adapter.cpp, sdk_internal.hpp, sdk_stub.cpp, sdk_symbols.def, sha256_macos.cpp, transport_manager.cpp,
  usb_monitor_macos.cpp, usb_watch_main.cpp}`, `sdk/tests/{mini_test.hpp, test_transport_manager.cpp}` (17 `TEST_CASE`s).
- Emit sites the fixtures cite: `main.cpp:471–506` (`cmd.received`, `cmd.rejected`, `cmd.error`, `cmd.done`, `transport.status`), `:525`
  (`sdk.unavailable`), `:250` (`crlab.version`); `transport_manager.cpp:81–87` (`transport.state`), `:104` (`usb.arrived`/`usb.removed`), `:167`
  (`usb.fallback_to_wifi`), `:249` (`usb.detected_by_discovery`); `usb_monitor_macos.cpp:183` (`usb.claimed_by {owners, hint}`); `camera_capture.cpp:
  173–178` (`sdk.download.completed {filename, size_bytes, success, transport, type?, provisional_capture_id?, ms_since_capture_request?}`), `:183`
  (`sdk.download.hashed`); `camera_connection.cpp:270–308` (`ssh.fingerprint {status ∈ not_pinned | dir_create_failed | match | MISMATCH | unknown |
  pinned | accepted_not_saved, success?}`), `:421` (`fetch_failed`); the env reads to remove: `main.cpp:129–131` (`CAMERA_SSH_USER`, `CAMERA_SSH_PASSWORD`,
  `CRLAB_FINGERPRINT_DIR` stays); `main.cpp:140–160` (`StdinQueue`), `:430–509` (`cmd_auto`); `transport_manager.cpp:92–96` (`desired_transport`,
  `usb_retry_at_`), `:160–172` (fallback), `camera_connection.cpp:548` (`restore_store_destination` on disconnect — **removed**), `camera_capture.cpp:
  206–262` (`configure_save`: `SetSaveInfo` + store destination verified ≤ 3 s — the model for the connect sequence), `:264–285`
  (`set_store_destination`), `usb_monitor_macos.cpp` (`matching`: add `idProduct`).
Code at `41bef14` (+ P42 names): `apps/desktop/src-tauri/{tauri.conf.json, Info.plist, build.rs, Cargo.toml}`; `.gitignore`; P42's `Supervisor`,
`CameraRuntime`, `fake_sidecar.rs`, `TestApp::with_camera`, `fake_sidecar_recording`.

## Files expected to change
Native (new, copied then edited): `native/camera-sidecar/CMakeLists.txt`, `native/camera-sidecar/include/crlab/*.hpp` (5), `native/camera-sidecar/src/*`
(15 incl. `sdk_internal.hpp`, `sdk_symbols.def`), `native/camera-sidecar/tests/{mini_test.hpp, test_transport_manager.cpp}`, `native/camera-sidecar/README.md`
(what it is, how to build with and without the SDK, what must never be committed). Tools (new): `tools/sidecar/build.sh`, `tools/sidecar/manifest.sh`
(writes `native/camera-sidecar/manifest/sony-<version>.json`; run by the owner/orchestrator on the build machine before P50; the file holds hashes only).
Repo (changed): `.gitignore` (`apps/desktop/src-tauri/binaries/`, `native/camera-sidecar/build/`, `target/camera-sidecar/`, `*.fingerprint`, `vendor/`,
`CrSDK*`), `apps/desktop/src-tauri/{tauri.conf.json, Info.plist, build.rs}`.
Integrations (new): `crates/integrations/src/camera/sidecar/{protocol.rs, client.rs}`, `crates/integrations/src/camera/sidecar/fixtures/
{lab_auto_switching.jsonl, emit_sites.jsonl}`. Integrations (changed): `crates/integrations/src/camera/sidecar/mod.rs`, `crates/integrations/src/camera/
runtime.rs` (the client replaces raw line handling; the `SidecarSpec` args `auto --save-dir <capture root>/.incoming/sdk-<run id> --usb-product-id 0x0E0C`).
Testkit (changed): `crates/testkit/src/bin/fake_sidecar.rs` (scripted replay), `crates/testkit/src/test_app.rs` (`with_camera_script`, `sidecar_events()`),
`crates/testkit/src/fixtures/sidecar_scripts/{lab_lines.txt, capture.txt}`. Tests (new): `crates/testkit/tests/p45_sidecar_and_protocol.rs`.
Estimated: 28 new paths (23 native, 2 tools, 2 Rust sources, 2 fixtures — counting folders' files), 7 changed, 1 new test — 36 paths.

## Core mechanisms (unit tests allowed)
`protocol.rs::{parse_line, Command::encode, percent_encode, iso_to_ms}`: inline tests named `mc31_<slug>` (rows C3, C4). Everything else is a
behaviour row (the fake sidecar, the build script).

## The sidecar changes (C++; each marked **mock-testable** or **SDK-only**)
1. **Credentials over stdin (R-I21, mock-testable in the stub? no — `auto` exits 4 in the stub before stdin; observed at P47):** `cmd_auto` reads the
   **first** stdin line; if it starts with `credentials ` it sets `base.ssh_user`/`ssh_password` from the percent-decoded `user=`/`password=` fields and
   emits `cmd.received {cmd: "credentials"}` with **no other field**; the env variables `CAMERA_SSH_USER`/`CAMERA_SSH_PASSWORD` are no longer read anywhere
   (grep-clean). `CRLAB_FINGERPRINT_DIR` stays an env variable (not secret).
2. **`usb.initial { present }` (R-I23, SDK-free, observed at P47):** emitted once right after `mon.start` succeeds, from `mon.present_count() > 0`.
3. **`usb-retry` command (R-I23, mock-testable):** `TransportManager::usb_retry()` sets `usb_failures_ = 0`, `usb_retry_at_ = 0`, `failures_ = 0`,
   `next_attempt_ms_ = now` and emits `usb.retry_requested`; `cmd_auto` accepts `usb-retry` whatever the state.
4. **IOKit match names the camera (R-I23, SDK-free):** `--usb-product-id 0x0E0C` (repeatable) → `UsbMonitor::start(vendor 0x054C, product ids)`; the
   `matching()` dictionary gets `idProduct` inside `IOPropertyMatch` beside `idVendor`; `usb_exclusive_owners` filters likewise.
5. **Connect sequence (R-I19, SDK-only):** `--save-dir <dir>`; after every successful connect (`SdkCameraAdapter::connect` success, both transports):
   `SetSaveInfo(dir)`, then `set_store_destination(usb ? "pc+camera" : "camera")` (verified ≤ 3 s, `configure_save`'s read-back loop), **then**
   `connection.ready { transport, model, store_destination: "verified" | "failed", store_value: "0x…" }`. `restore_store_destination` is deleted with
   its call at `camera_connection.cpp:548` and the `store_dest_changed` bookkeeping; the write is persistent. **"pc" alone is never written** (the
   `dest_value` mapping for `"pc"` is removed).
6. **Quit during a USB session (R-I19, SDK-only):** `request_stop` on `ConnectedUsb` first runs `set_store_destination("camera")` with a 1 s verify
   bound, then disconnects.
7. **Busy `Transfer` (R-I20/R-I22, mock-testable):** `capture.exposed` on USB sets `tm.set_busy(BusyReason::FileTransfer)`; `sdk.download.completed`
   or 10 s clears it; `request_stop()` while busy waits for the safe point up to 10 s (`TransportManager::stop_wait_ms`) instead of disconnecting at
   once. A transport switch away from USB happens only on cable loss, so busy never defers it.
8. Everything else stays as the lab left it (no idle disconnect, no reconnect on errors, `CrReconnecting_OFF`).

## The fake sidecar (scripted replay)
`behaviour.json.script` names a text file; each line is one of: a JSON object → printed to stdout as is after substituting `{capture_id}` and
`{cmd_id}` with the values of the last `capture` command received; `sleep <ms>`; `await <cmd>` → block until a stdin line whose first word is
`<cmd>` arrives (recording it as always); `exit <code>`; `stderr <text>`. With no script the fake behaves as in P42. `TestApp::with_camera_script(text)`
writes the script and `behaviour.json` before `with_camera()`'s spawn. `TestApp::sidecar_events() -> Vec<SidecarEvent>` returns what the client
delivered so far (the runtime keeps the last 1 000 for tests and the Health page's helper log).

## Ordered tasks
0. **Import.** Copy the lab's `sdk/` into `native/camera-sidecar/` **excluding** `sdk/tests/sdk_shim/` and `sdk/build/`; `cp -R` only — no Python; add
   `README.md`; `.gitignore` entries; verify with `git status` that nothing under `native/` is a Sony file (C1).
1. **CMake:** rename the project/binary (`camera_sidecar` / `camera-sidecar`), drop `CRLAB_SDK_SHIM`, make `CRSDK_ROOT` required only with
   `-DCAMERA_SIDECAR_WITH_SDK=ON`, keep the stub default, keep the POST_BUILD copy of Sony dylibs **only** in the SDK build and only into the build dir
   (never into the repo or the app bundle: the user installs them as a support package), keep `ctest`.
2. **`tools/sidecar/build.sh`** (`--stub | --sdk <root>`, `--test`): configures into `target/camera-sidecar/{stub,sdk}`, builds, optionally runs
   `ctest --output-on-failure`, copies the binary to `apps/desktop/src-tauri/binaries/camera-sidecar-aarch64-apple-darwin`. `tools/sidecar/manifest.sh`.
3. **`build.rs`:** before `tauri_build::build()`, if the externalBin file is missing run `tools/sidecar/build.sh --stub` (fail the build with a plain
   message if cmake is missing); `cargo:rerun-if-changed=native/camera-sidecar/src`, `…/include`, `…/CMakeLists.txt`. `tauri.conf.json`:
   `bundle.externalBin: ["binaries/camera-sidecar"]`. `Info.plist`: `NSLocalNetworkUsageDescription` (`Session Manager talks to your camera over the
   local network to control it and receive photos.`).
4. **C++ changes** 1–7 above + two new `TEST_CASE`s (C6).
5. **Rust protocol:** `protocol.rs` (`SidecarEvent`, `parse_line`, `iso_to_ms`, `Command::encode`), `client.rs` (`SidecarClient { supervisor,
   pending: HashMap<cmd_id, Command>, events: Sender<SidecarEvent> }`, `send(Command) -> CmdId`, `poll() -> Vec<SidecarEvent>` (parses supervisor lines;
   unparsable lines are logged once each and skipped), `outcome(cmd_id) -> Option<CommandOutcome {Done, Rejected {reason}, Error {error}}>`); the runtime
   uses the client and passes `--save-dir`, `--usb-product-id`; fixtures.
6. Fake sidecar script mode; `TestApp` helpers; tests `p45_sidecar_and_protocol.rs`; suite, clippy, fmt, typecheck.

## Criteria (7 criteria, 18 rows; 1 `[rule]` row)
`R` = repo root; `N` = `R/native/camera-sidecar`. `build()` = `tools/sidecar/build.sh --stub --test` run by the test with `R` as cwd (its cmake build
dir is cached under `target/camera-sidecar/stub`, so the second run is seconds).

| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | no vendor file in the tree | walk `N` recursively | no path matches `CameraRemote_SDK.h`, `Cr*.h`, `*.dylib`, `*.a`, `*.framework`, `sdk_shim`, `vendor`, `.env*`, `*.fingerprint`, `*.ARW`; the lab's five `include/crlab/*.hpp` and `src/sdk_stub.cpp` **are** present | R45 ("no vendor SDK file, nor the lab's hand-written SDK stand-in headers, ever enters the public repo"); D8; M39 ("the repo") |
| C1(b) | ignore rules | read `R/.gitignore` | contains `apps/desktop/src-tauri/binaries/`, `native/camera-sidecar/build/`, `*.fingerprint`, `vendor/`, `.env` (already) | R45; master plan §6.1 |
| C1(c) | no secret env read | grep `N/src` for `CAMERA_SSH_USER`, `CAMERA_SSH_PASSWORD` | zero matches; `CRLAB_FINGERPRINT_DIR` still read once | R-I21 ("not through its environment") |
| C2(a) | stub build and C++ tests | `build()` | exit 0; `R/apps/desktop/src-tauri/binaries/camera-sidecar-aarch64-apple-darwin` exists and is executable; the ctest summary line reports `19` tests passed (17 lab + C6(a), C6(b)) and 0 failed | 10E §Camera support packages ("CI builds the stub sidecar"); 11 |
| C2(b) | the stub refuses camera work loudly | run the binary with `version`; then with `auto --save-dir /tmp/x` | `version` prints one JSON line with `"event":"crlab.version"`, `"sdk_compiled_in":false`; `auto` prints a line with `"event":"sdk.unavailable"` and exits with code 4 within 2 s | 10E ("CI builds the stub sidecar"); lab handoff §5.1 |
| C3(a) | the lab's real lines parse (unit) | the 13 lines of `fixtures/lab_auto_switching.jsonl` (handoff §4.2 verbatim) through `parse_line` | the variant sequence `[TransportState{from:"DISCONNECTED",to:"DISCOVERING"}, ConnectionReady{transport:Wifi, connect_ms:914}, CmdReceived{cmd:"capture",cmd_id:"4db6be91"}, CameraBusy{reason:"exposure",safe_point:false}, CaptureRequested{capture_id:"53d7…",transport:Wifi,af:true}, CaptureAf{..}, CaptureAcknowledged{success:true,sdk_error:"0x00000000"}, CaptureExposed{capture_id:Some("53d7…"),transport:Wifi,association:"latest_release_provisional"}, CmdDone{cmd_id:"4db6be91"}, UsbArrived{detail:"pid=0x0E0C ILCE-7RM5"}, SessionClosed{transport:Wifi,reason:"switching wifi -> usb"}, SessionLost{transport:Usb,reason:"usb device removed",busy:"none"}, SwitchDeferred{to:"usb",busy:"pixel_shift"}]`; the second line's `timestamp_ms == 1_793_888_519_760` (`2026-11-05T14:21:59.760Z`) | MC-31 §Process ("stdout one JSON event per line"); lab handoff §4.2 (real samples) |
| C3(b) | emit-site lines parse (unit) | `fixtures/emit_sites.jsonl`: one line per cited site — `usb.claimed_by {owners:"pid 512, ptpcamerad", hint:…}` (`usb_monitor_macos.cpp:183`), `usb.fallback_to_wifi {usb_failures:3, usb_retry:"on next USB re-plug"}` (`transport_manager.cpp:167`), `sdk.download.completed {filename:"/…/.incoming/sdk-r1/DSC00007.ARW", size_bytes:51234567, success:true, transport:"usb", type:"0x00000001", provisional_capture_id:"c1", ms_since_capture_request:1960}` (`camera_capture.cpp:173–178`), `sdk.download.hashed {filename, sha256, hash_ms, success}` (`:183`), `ssh.fingerprint {status:"unknown", success:false, note:…}` and `{status:"MISMATCH", success:false}` (`camera_connection.cpp:283, 295`), `capture.uncertain {operation:"bracketing"}`, `capture.exposure_missing {capture_id}`, `cmd.rejected {cmd:"capture", reason:"not connected (DISCOVERING)"}` (`main.cpp:480`), `cmd.error {cmd:"x", error:"unknown command"}` (`:504`), `sdk.unavailable {error}` (`:525`), `connect.failed {transport:"wifi", sdk_error:"0x00008208", message:…}`, `usb.initial {present:true}` and `connection.ready {transport:"usb", model:"ILCE-7RM5", store_destination:"failed", store_value:"0x0002"}` (this phase's own emits) | each parses to its variant with the listed fields; `UsbClaimedBy.owners == "pid 512, ptpcamerad"`; `SdkDownloadCompleted.size_bytes == 51234567`; `ConnectionReady.store_destination == Some(Failed)` | MC-31 (event meanings); rule 17 (shapes cited to the emit sites) |
| C3(c) | unknown is ignored, malformed is an error (unit) | `{"timestamp":"…","source":"sdk","event":"camera.newthing","x":1}`; a `connection.ready` line with an extra field `"foo":"bar"`; `not json`; `{"event":"connection.ready"}` (no timestamp) | `Unknown { event: "camera.newthing" }`; `ConnectionReady` as normal; `Err(ProtocolError::Json)`; `Err(ProtocolError::MissingTimestamp)` | MC-31 §Process ("unknown fields and events ignored") |
| C4(a) | credentials encoding (unit) | `Command::Credentials { user: "u name", password: "p/w=&é" }.encode()` | `credentials user=u%20name password=p%2Fw%3D%26%C3%A9` (RFC 3986 unreserved kept) | R-I21 ("percent-encoded") |
| C4(b) | capture encoding (unit) | `Command::Capture { id: "11111111-…", af: true, cmd_id: "c1" }.encode()` | `capture id=11111111-… af=1 cmd_id=c1` | MC-31 §Capture ("Half-press, AF wait up to 3 s, release"); lab handoff §4.3 |
| C4(c) | the other commands (unit) | `UsbRetry{cmd_id:"c2"}`, `FtpStatus{cmd_id:"c3"}`, `Props{cmd_id:"c4"}`, `Status{cmd_id:"c5"}`, `Quit` | `usb-retry cmd_id=c2`, `ftp-status cmd_id=c3`, `props cmd_id=c4`, `status cmd_id=c5`, `quit` | R-I23 (`usb-retry`); R-I29 (`ftp-status`); MC-31 §Process (`quit`) |
| C5(a) | events flow from the fake to the client in order | `a = TestApp::with_camera_script(lab_lines.txt)` (the 13 lines, no sleeps); `a.tick(0)` until 13 events | `a.sidecar_events()` equals C3(a)'s sequence; the fake's `stdin.log` first line is the credentials line (P42 C4(a) still holds) | MC-31 §Process; R-I21 |
| C5(b) | command correlation | script `capture.txt`: `await capture`, then `cmd.received {cmd:"capture",cmd_id:"{cmd_id}"}`, `capture.requested {capture_id:"{capture_id}"}`, `capture.acknowledged {…}`, `capture.exposed {capture_id:"{capture_id}", association:"latest_release_provisional"}`, `cmd.done {cmd:"capture",cmd_id:"{cmd_id}"}`; `id = client.send(Command::Capture{..})`; `a.tick(0)` until `cmd.done` | `rec().stdin` contains exactly `capture id=<uuid> af=1 cmd_id=<id>`; `client.outcome(id) == Some(Done)`; the `CaptureExposed` event carries that uuid | MC-31 §Capture; lab handoff §4.3 ("Every command is wrapped in `cmd.received` … `cmd.done` carrying your `cmd_id`") |
| C5(c) | rejected command | script: `await capture`, `cmd.rejected {cmd:"capture", reason:"not connected (DISCOVERING)"}`, `cmd.done {…}` | `client.outcome(id) == Some(Rejected { reason: "not connected (DISCOVERING)" })` | lab handoff §4.3 ("Commands sent while disconnected get `cmd.rejected`") |
| C5(d) | a malformed line does not stop the stream | script: `{bad`, then a valid `transport.state` line | `sidecar_events()` holds the `TransportState`; the runtime's helper log holds one `unparsable sidecar line` entry | MC-31 §Process ("unknown … ignored") |
| C6(a) | `usb_retry()` after fallback (C++ `TEST_CASE("usb-retry gives usb a fresh chance without a re-plug")`, run by C2(a)) | rig with `fail_usb_connects = 3`, USB plugged → fallback (`usb.fallback_to_wifi`), `run(5000)`; `tm->usb_retry()`; `run(5000)` | after the retry the mock records a new `connect:usb` and the session is on USB (`cam.active_sessions == 1`, state `ConnectedUsb`); without the retry, `run(60000)` records no further `connect:usb` | R-I23 (`usb-retry` "clears the three-failure fallback"); MC-31 §Transport policy ("no periodic USB probing") |
| C6(b) | stop waits for a safe point (C++ `TEST_CASE("stop is deferred while busy, bounded by 10 s")`) | connected on USB; `set_busy(FileTransfer)`; `request_stop()`; `run(3000)` → still connected; `set_busy(None)`; `run(500)` → closed. Second rig: busy never cleared; `run(10_500)` → closed | as stated; `log.count("session.closed") == 1` in both | R-I20 ("That busy state only defers a voluntary stop"); R-I22 |
| C7(a) | packaging wiring | read `apps/desktop/src-tauri/tauri.conf.json` and `Info.plist` | `bundle.externalBin == ["binaries/camera-sidecar"]`; `Info.plist` has `NSLocalNetworkUsageDescription` with a non-empty string | `[rule]` ch11 (`externalBin`; `NSLocalNetworkUsageDescription`); R-I36 spike S4 |

## Delegated choices
Whether `native/camera-sidecar/README.md` repeats the lab's build notes or links `docs/intention/evidence/sony_camera_lab.md`; the C++ percent-decoder
location (`json_log.cpp` or a new `util.cpp`); the `stop_wait_ms` field's exposure in `TransportConfig`; whether `build.rs` shells out to the script or
runs cmake itself; the fixtures' file names beyond the two above; `SidecarEvent::Unknown` keeping the raw JSON; the helper-log capacity.

## Manual try-out
(Hardware-free.) 1. In Terminal: `tools/sidecar/build.sh --stub --test` — ends with "100% tests passed, 0 tests failed out of 19". 2. `apps/desktop/
src-tauri/binaries/camera-sidecar-aarch64-apple-darwin version` prints one JSON line saying the SDK is not compiled in. 3. Start the app: nothing
changes on screen; `git status` shows no `binaries/` or build output (ignored).

## Notes
- **Hard rules (owner):** **never commit a Sony SDK file, the lab's `sdk_shim` headers, a `.env`, a credential or an SSH fingerprint** — C1 is the check,
  the reviewer re-runs `git ls-files native/` by eye; no Python (the lab's Python is not copied); never change the camera's network/FTP settings (the
  sidecar writes only the store destination, R-I19, which the photographer agreed to in setup — P51 says so), never delete a camera file or kill any
  process; live view out of scope; the suite never needs the camera. Implemented by Claude agents (owner 2026-11-06, `PSM_IMPLEMENTER=claude`).
- **SDK-only code is compiled here only if the implementer runs `tools/sidecar/build.sh --sdk /Users/davidloorenz/Desktop/Developer/Camera-Connection-Tests/
  vendor/sony-sdk`** (the SDK is on this Mac in the lab's checkout). The implementer **should** run that build once to prove the R-I19/R-I20 edits compile
  (recorded in the handoff; nothing of it is committed), but the suite never depends on it. If the build cannot run, the edits ship compile-checked
  against nothing and P47 is where they are first exercised — the handoff must say which.
- **Planner readings (reversible):** (1) the binary keeps the lab's internal `crlab` namespace and event names (renaming them would invalidate the
  real fixtures); only the executable and CMake project are renamed. (2) `connection.ready` gains `model` so the status can show the camera's name. (3)
  `usb.retry_requested` is a new event acknowledging the command. (4) The product id list defaults to the A7R V's `0x0E0C`; another body means another
  id (the ZV-E10 is out of scope).
- **Hazards:** `tauri-build` validates `bundle.externalBin` at compile time (the file must exist with the target-triple suffix) — hence `build.rs` runs
  the stub build **before** `tauri_build::build()`; if that assumption is wrong, keep the step anyway so `tauri dev` has a sidecar. The stub's `auto`
  exits 4 immediately: with the stub installed as a "support package" the runtime would restart it on backoff and show `camera_sidecar_unavailable` —
  correct, and P50 refuses to install a stub (`version` must report an SDK). cmake's first configure takes ~20–40 s: C2(a) is the slow test of the
  suite; cache under `target/`. The `StdinQueue` pushes `quit` on EOF — the Rust side must keep stdin open for the sidecar's lifetime. Percent-decoding
  in C++ must handle `%` at end-of-string without UB.
- **Projection** is on: every fixture line in C3(b) is externally owned (the sidecar's C++ emit sites) — the projectionist re-reads each cited site and
  confirms the field names and value forms; and checks that the 13 real lines are copied verbatim from the handoff.
- Registry additions made by the planner in master plan §6.1, §6.2, §6.9, §6.11, §7.2. No intention-book change.
- Estimated size: ~36 paths; ~3 600 lines copied C++ (unchanged), ~250 lines of C++ edits and tests, ~900 lines Rust (protocol, client, fake, tests), ~150
  lines shell/CMake.

## Review log


- **2026-11-06, carried from P42 projection r0** (planner): the fake's `behaviour.json` and the script live in the recording dir the fake derives from its
  own location (`<data dir>/camera/sony/test/rec/`, L2); `with_camera_script` writes both **before** the spawn (L1); `Recording.argv` is `Vec<Vec<String>>`
  from `argv.<pid>.json` (L19) and `exits` records voluntary exits (L9); every "`a.tick(0)` until …" in C5 is P42's `settle(a, pred)` (L7); the credentials
  line is built from the runtime's cached credentials (L17). C1(c)'s grep must also cover the new percent-decoder: no `getenv` of a credential anywhere.
