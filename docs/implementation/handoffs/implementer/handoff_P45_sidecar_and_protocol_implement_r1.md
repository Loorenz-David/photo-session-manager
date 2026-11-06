---
plan: P45_sidecar_and_protocol
role: implement
round: 1
state: DONE
date: 2026-11-06
actor: claude (opus, pipeline-implementer)
---
# Handoff — P45 camera sidecar and protocol, implement r1

## What was built
- **The camera helper is now part of Session Manager's source.** The Sony camera lab's C++ helper (`crlab`, lab commit `221aa49`) lives in
  `native/camera-sidecar/`. It contains no Sony file and none of the lab's hand-written SDK stand-in headers. On any Mac it builds as a
  **stub** that refuses camera work loudly (`sdk.unavailable`, exit 4). On a machine that has the Sony SDK, `tools/sidecar/build.sh --sdk <dir>`
  builds the real helper. Building the app on a clean machine builds the stub by itself (`build.rs`), and the app declares it as its bundled
  helper (`externalBin`) together with the macOS local-network permission text.
- **The helper does what the book's inventory asks** (MC-31 R-I19…R-I23):
  - the camera password arrives only as the first stdin line, percent-encoded, and is never read from the environment;
  - every new session sets the download folder and writes the store destination (USB *Host PC + Memory Card*, Wi-Fi *Memory Card*), reads it
    back within 3 s and reports `verified`/`failed` with the value read and the camera model in `connection.ready`;
  - nothing is restored on disconnect, and *Host PC* alone can no longer be written at all;
  - a quit during a USB session writes *Memory Card* once (≤ 1 s);
  - a USB exposure holds busy *file transfer* until its download completes (≤ 10 s), and a quit while busy waits for that safe point (≤ 10 s);
  - `usb.initial {present}` is reported once at start, the USB watch is limited to the camera's product id, and the new `usb-retry` command
    clears the three-failure USB fallback.
- **The app can talk to the helper.** It parses every event line into typed events (unknown events and fields are ignored, malformed lines are
  logged once and skipped), encodes commands, and matches each command to its outcome (done / rejected / error). The camera runtime exposes
  `send_command`, `command_outcome` and `sidecar_events` (the last 1 000 events). The helper now starts with
  `auto --save-dir <session>/Capture/.incoming/sdk-<run id> --usb-product-id 0x0E0C`.
- **Test harness:** the fake helper replays scripts (JSON lines, `sleep`, `await`, `exit`, `stderr`) through `TestApp::with_camera_script`, and
  `TestApp::sidecar_events()` reads the runtime's events.
- **SDK build proven on this Mac (not committed):** `tools/sidecar/build.sh --sdk /Users/davidloorenz/Desktop/Developer/Camera-Connection-Tests/vendor/sony-sdk`
  configured (60 symbol probes present, `DCVersion` absent as expected on 2.02), compiled every SDK-only edit (changes 1, 5, 6, 7) with
  **0 warnings**, `version` → `"sdk_compiled_in":true,"sdk_version":"2.02.00"`. `auto` with an empty stdin exited 0 after `sdk.init` without
  connecting (the "end of input before any line = quit" rule, observed on the real build). No camera was used. The stub was rebuilt
  afterwards, so `binaries/` holds the stub.

## Tests → criterion rows
| Row | Test | Boundary |
|---|---|---|
| C1(a) | `p45_sidecar_and_protocol::c1a_no_vendor_file_in_tree` | recursive walk of `native/camera-sidecar` |
| C1(b) | `c1b_ignore_rules` | `.gitignore` lines |
| C1(c) | `c1c_no_secret_env_read` | grep of `native/camera-sidecar/src` (+ every `env("…")`/`getenv("…")` name checked for credential words) |
| C1(d) | `c1d_host_pc_alone_never_written` | grep of `src` for `…_HostPC` not followed by `AndMemoryCard` |
| C2(a) | `c2a_stub_build_and_cpp_tests` | `tools/sidecar/build.sh --stub --test` (one `OnceLock` run): exit 0, executable binary, `20 tests, 0 failed`, `100% tests passed out of 1` |
| C2(b) | `c2b_stub_refuses_camera_work` | the built stub: `version` one line with `crlab.version`/`sdk_compiled_in:false`; `auto` → `sdk.unavailable`, exit 4 within 2 s |
| C2(c) | `c2c_manifest_over_synthetic_tree` | `tools/sidecar/manifest.sh` on a temp tree: exact path list and order, SHA-256 per file, README absent, nothing written under `native/` |
| C3(a) | `protocol::tests::mc31_lab_lines_parse` (unit) | `parse_line` over the 13 real lines (fixture extracted verbatim from handoff §4.2); `timestamp_ms == 1_793_888_519_760` |
| C3(b) | `protocol::tests::mc31_emit_site_lines_parse` (unit) | `parse_line` over `fixtures/emit_sites.jsonl`, whole-variant equality per line |
| C3(c) | `protocol::tests::mc31_unknown_ignored_malformed_rejected` (unit) | the seven inputs and outcomes of the row (`mc31_iso_to_ms` adds the timestamp forms) |
| C4(a) | `protocol::tests::mc31_credentials_encoding` (unit) | `Command::Credentials{..}.encode()` |
| C4(b) | `protocol::tests::mc31_capture_encoding` (unit) | `Command::Capture{..}.encode()` |
| C4(c) | `protocol::tests::mc31_other_commands_encoding` (unit) | `UsbRetry`/`FtpStatus`/`Props`/`Status`/`Quit` |
| C5(a) | `c5a_events_flow_in_order` | fake replaying `lab_lines.txt` → `a.sidecar_events()` equals the 13-event sequence; `stdin.log` first line is the credentials line |
| C5(b) | `c5b_command_correlation` | `rt.send_command(Capture)` → wire line once in `rec().stdin`, `command_outcome == Done`, `CaptureExposed` carries the uuid |
| C5(c) | `c5c_rejected_command` | `command_outcome == Rejected{"not connected (DISCOVERING)"}`, not overwritten by the later `cmd.done` |
| C5(d) | `c5d_malformed_line_does_not_stop_stream` | exactly one `TransportState` event; exactly one `unparsable sidecar line` helper-log entry |
| C6(a) | C++ `TEST_CASE("usb-retry gives usb a fresh chance without a re-plug")`; Rust `c6a_usb_retry_after_fallback` asserts it passed in C2(a)'s run | mock-camera rig via ctest |
| C6(b) | C++ `TEST_CASE("stop is deferred while busy, bounded by 10 s")`; Rust `c6b_stop_waits_for_safe_point` | mock-camera rigs (3 000/500 and 9 900/600) |
| C6(c) | C++ `TEST_CASE("credentials percent-decoding reverses the app's encoding")` (`crlab_core`); Rust `c6c_credentials_percent_decoding` | `percent_decode` |
| C7(a) | `c7a_packaging_wiring` | `tauri.conf.json` `bundle.externalBin`; `Info.plist` `NSLocalNetworkUsageDescription` non-empty |

Unit tests exist only for the plan's Core mechanisms (`parse_line`, `Command::encode`, `percent_encode`, `iso_to_ms`; the C++ decoder).

## Assumptions
(Smallest choices consistent with 10E MC-31 and the plan's Delegated choices.)
1. **Timestamp carrier:** `parse_line` returns `Stamped { timestamp_ms, event: SidecarEvent }`, and `SidecarEvent` keeps exactly the table's
   variants (`UsbRetryRequested` stays a unit variant, and the C3 rows compare whole variants). `poll()` and `sidecar_events()` return `Stamped`.
2. **`SidecarClient::poll(now, side)`:** the supervisor needs `now`. Started, Exited and stderr events and unparsable lines go to the `side`
   callback; an unparsable line arrives there as `Stderr("unparsable sidecar line: …")`, so it reaches the helper log through the redactor.
3. **No `events: Sender<SidecarEvent>` field yet** (L23: P45 has no receiver; the ring buffer is enough). P46 adds the sender with its receiver.
4. **`cmd_id`:** `send` uses the command's own `cmd_id`. When it is empty, `send` mints 8 hex characters and writes that id on the wire (the
   returned id is always the wire id). `encode` leaves `cmd_id=` out when it is empty and writes `af=0` when `af` is false.
5. **C++ choices:**
   - the percent-decoder lives in `json_log.cpp`, declared in `json_log.hpp` (no `util.cpp`, so the perimeter stays at 42);
   - `save_dir` and `usb_product_ids` are `ConnectOptions` members;
   - `stop_wait_ms` is a `TransportConfig` field (10 000);
   - `usb_retry()` is a queued `Input` (keeps the threading contract);
   - USB matching is a vendor match plus a product filter applied before `present` is counted. A registry entry that reports no
     `idProduct` (0) is kept, so a missing property never hides the camera. `usb_exclusive_owners` and `report_usb_claim` apply the same filter
     (an empty list means any Sony device, which is what the lab's diagnostic commands use).
6. **Deferred stop:** if IOKit reports the cable gone while a USB stop is waiting, the wait ends at once as a session loss (`session.lost`), the
   same as an SDK-reported loss ("a session lost meanwhile completes the stop at once").
7. **Busy Transfer:** a completed download returns busy to *none* only when a transfer was pending, so it never clears an `Exposure` busy in the
   middle of a capture. The 10 s timer lives in `cmd_auto`'s 100 ms loop.
8. **`store_value`** is left out of `connection.ready` when nothing could be read back (property not reported). In that case
   `store_destination` is `failed`.
9. **`build.rs`** shells out to `tools/sidecar/build.sh --stub`, checks for `cmake` first (plain panic message), names the file with `TARGET`,
   and drops the `../../../native/…` rerun paths (L19). After a C++ edit, run the script (the README says so).
10. **Health item:** `camera_sidecar_unavailable` is cleared by parsed events (a stdout made only of garbage no longer clears it). Output is still
    handled before exits within one tick, as P42 did.
11. README links the evidence instead of repeating the lab's notes. `Unknown` does not keep the raw JSON. Helper-log capacity stays at 50.
12. C2(b) runs `auto --save-dir <temp>/x` instead of `/tmp/x` (the stub touches neither). Each C6 row also has a Rust test that checks its C++
    case passed in C2(a)'s single build run.
13. Tracker actor written as `opus` (the P44 precedent for Claude implementers) rather than the template's `codex`.

## Deviations
- **`sdk_symbols.def`:** the probe line `sym StoreDestHostPC SCRSDK::CrStillImageStoreDestination_HostPC` was deleted, together with its use in
  `dest_value`'s `#if`. C1(d) greps all of `src/`, and that line names *Host PC* alone. Probes: 61 → 60.
- **Lab diagnostic commands:** `store-dest`/`save` no longer accept `pc`, and their usage text says `camera|pc+camera`. `configure_save`'s
  destination write is now the persistent `set_store_destination`.
- **Signatures differ slightly from the registry/plan text:** `Stamped` (assumption 1), `poll(now, side)` (2), no `events` field (3),
  `set_store_destination` returns `StoreDestinationResult { verified, read, value }` (the plan's "value read back beside the verified flag").
- **Fake sidecar race found and fixed:** a `capture` line that arrived before the script reached `await capture` was consumed and lost, so C5(b)
  and C5(c) failed intermittently. Lines taken before an `await` now count for it. Fixed inside the perimeter (`fake_sidecar.rs`).
- **P42 health ordering:** the first full run failed `p42::c3b` because a dying sidecar's last line cleared the health item after its exit had
  set it. Fixed in `runtime.rs` (output first, exits after, as before). P42 passes 12/12.
- **Manifest note for P50/owner (not a code change):** `manifest.sh` follows its spec and lists every `*.dylib` under the root. Run on the SDK's
  top folder, it would also list RemoteCli's four OpenCV dylibs. Point it at `CrSDK_v2.02.00_Mac/RemoteCli/external/crsdk` (the script's header
  and the README say so). Not run in this phase: no manifest is committed.
- The template prompt's actor `codex` replaced by `opus` (assumption 13).

## Files changed
Checkpoint `db3cdae`: 44 paths = the plan's 42 + tracker + plan Review log.
- Native (24 new): `native/camera-sidecar/{CMakeLists.txt, README.md}`, `include/crlab/{camera,json_log,sha256,transport,usb_monitor}.hpp`,
  `src/{camera_capture,camera_connection,camera_discovery,camera_properties,diagnostics,json_log,main,sdk_adapter,sdk_stub,sha256_macos,
  transport_manager,usb_monitor_macos,usb_watch_main}.cpp`, `src/{sdk_internal.hpp, sdk_symbols.def}`, `tests/{mini_test.hpp,
  test_transport_manager.cpp}`.
- Tools (2 new): `tools/sidecar/{build.sh, manifest.sh}`.
- Repo (4 changed): `.gitignore`, `apps/desktop/src-tauri/{tauri.conf.json, Info.plist, build.rs}`.
- Integrations (4 new): `crates/integrations/src/camera/sidecar/{protocol.rs, client.rs, fixtures/lab_auto_switching.jsonl, fixtures/emit_sites.jsonl}`.
  Integrations (3 changed): `sidecar/{mod.rs, supervisor.rs}`, `camera/runtime.rs`.
- Testkit (2 changed): `src/bin/fake_sidecar.rs`, `src/test_app.rs`. Testkit (3 new): `src/fixtures/sidecar_scripts/{lab_lines.txt, capture.txt}`,
  `tests/p45_sidecar_and_protocol.rs`.
- Docs (2 changed): `docs/implementation/master_plan.md` (P45 row only), `docs/implementation/plans/P45_sidecar_and_protocol.md` (Review log).
- Not committed (generated, ignored): `apps/desktop/src-tauri/binaries/camera-sidecar-aarch64-apple-darwin`, `target/camera-sidecar/{stub,sdk}`.
- This handoff: written after the checkpoint and committed by the runner.
- Not touched: `docs/implementation/STATUS.md` was already modified in the working tree before this session (the orchestrator's). No `Cargo.toml`
  or `Cargo.lock` change. No archgraph or other tool state.

## Checkpoint SHA
`db3cdae` — `CHECKPOINT (not approved): P45_sidecar_and_protocol implement r1`

## Stamp
| Command | Result |
|---|---|
| `tools/sidecar/build.sh --stub --test` | exit 0; `20 tests, 0 failed`; `100% tests passed out of 1`; 0 compiler warnings |
| `tools/sidecar/build.sh --sdk …/Camera-Connection-Tests/vendor/sony-sdk` | configure + build OK, 60 probes present, 0 warnings (not committed; stub rebuilt after) |
| `cargo test --workspace --no-fail-fast` | exit 0 — 693 passed, 0 failed (46 test binaries); `p45_sidecar_and_protocol` 15/15, `psm-integrations` mc31 7/7 |
| `cargo test -p psm-testkit --test p45_sidecar_and_protocol c5` ×6 (after the fake fix) | 6/6 green |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo fmt --all -- --check` | exit 0 |
| `cd apps/desktop && npm run typecheck` | exit 0 |
| `cd apps/desktop && npm test` | 50 files, 369/369 passed |
| clean-machine check: delete `binaries/camera-sidecar-aarch64-apple-darwin`, `cargo build -p psm-desktop` | `build.rs` rebuilt the stub (7.6 s); `version` → `sdk_compiled_in:false` |

Pre-existing failures: none observed.

## Manual try-out
Hardware-free; nothing changes on screen in this phase.
1. In Terminal at the repo root: `tools/sidecar/build.sh --stub --test`. The output ends with `20 tests, 0 failed`, `100% tests passed out of 1`
   and the path of the installed helper.
2. `apps/desktop/src-tauri/binaries/camera-sidecar-aarch64-apple-darwin version` prints one JSON line with `"sdk_compiled_in":false`.
3. `apps/desktop/src-tauri/binaries/camera-sidecar-aarch64-apple-darwin auto --save-dir /tmp/x; echo $?` prints a `sdk.unavailable` line and `4`.
4. Optional, on this Mac with the lab's SDK:
   `tools/sidecar/build.sh --sdk /Users/davidloorenz/Desktop/Developer/Camera-Connection-Tests/vendor/sony-sdk`,
   then `target/camera-sidecar/sdk/camera-sidecar version` prints `"sdk_version":"2.02.00"`.
   Run step 1 again afterwards, so the app keeps the stub until the support-package phase.
5. Start the app (`cd apps/desktop && npm run tauri -- dev`): nothing changes on screen. `git status` shows no `binaries/` or build output (both
   ignored).

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
