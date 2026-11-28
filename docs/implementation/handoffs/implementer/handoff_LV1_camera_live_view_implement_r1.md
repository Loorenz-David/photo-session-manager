---
plan: LV1_camera_live_view
role: implement
round: 1
state: DONE
date: 2026-11-26
actor: codex
---
# Handoff — LV1 camera live view, implement r1

Gate passed: intention RATIFIED; LV1 had no dependencies and its tracker was NOT_STARTED. The tracker is now IMPLEMENTED, pending independent review.

## What was built
- A connected camera can supply a live JPEG picture through the sidecar's separate binary frame socket. Pulling is paced, repeated frames are dropped, and a slow receiver keeps only the newest frame.
- The app's live-view hub gives frames only to viewers that currently request Live, keeps sequence numbers increasing across camera source changes, and sends focus, exposure and capture reports to subscribers.
- A focus request moves the camera's focus point on a separate worker, asks for autofocus when requested, and reports its result. Capture remains independent of live-frame delivery. The sidecar joins its workers before camera disconnect.
- The direct camera adapter parses the new events, maps focus frames and exposure readouts, advertises live-view and focus capability, and writes `live.source` counts every ten seconds. The fake sidecar can send binary frames for wired tests. The camera guide describes the flow and test directives.

## Tests → criterion rows
| Rows | Boundary and tests | Result |
|---|---|---|
| LV1·C1(a–h), C2(a), C3(a–c), C4(a–b), C5(a–c) | `native/camera-sidecar/tests/test_liveview.cpp`: pacer, frame record/JPEG dimensions, latest slot, focus queue/parser and action lock | 17 passed |
| LV1·C6(a–e), C7(a–e), C8(a–b), C9(a–b), C10(a–d), C11(a–d) | `crates/testkit/tests/suite/lv1_live_hub.rs`: hub outlet, demand, source, event and focus boundary | 22 passed |
| LV1·C12(a–e), C13(a–d), C14(a–b), C15(a–d), C16(a–d), C17(a–c) | `crates/testkit/tests/suite/lv1_frame_channel.rs`: protocol and wired `TestApp` with fake sidecar | 22 criterion tests passed; 2 core-mechanism tests also passed |
| Existing camera behavior | `p42_camera_runtime`, `p45_sidecar_and_protocol`, `p46_direct_camera_adapter`, and `psm-integrations camera::` | 98 selected suite tests and 23 integration unit tests passed |

## Assumptions
- The frame listener belongs to the camera runtime. It uses one reader thread and a bounded 4 MiB JPEG record; the socket falls back to `/tmp` if the normal temporary path is too long.
- A new socket is bound for each helper process. The connection counter survives rebinding so a restarted helper with the same epoch is a new source. A malformed record disables that process's frame stream.
- The fake sidecar's JPEG-like payload is deterministic from its epoch, frame number and byte index; it does not need to decode as an image for the frame-channel tests.
- `live.source` reports ten-second differences of the sidecar's cumulative stats. The focus worker remains available while live view is disabled if the camera session is connected.
- `CurrentShots` is an empty runtime holder for LV2, as the plan requires.

## Deviations
- Unit 1 also changed `native/camera-sidecar/src/sdk_adapter.cpp` and `src/camera_capture.cpp` to route camera writes through the four-intent function. Unit 3 corrected the Unit 1 runner's result fields and stop handling, and guarded the store-destination SDK symbol in `camera_capture.cpp`.
- `cargo fmt` in Unit 3 formatted Rust files committed in Unit 2. This is included in the Unit 3 checkpoint.
- Three existing camera test files changed outside the listed Unit 3 perimeter to keep their checks valid: `p45_sidecar_and_protocol.rs` expects the new second CTest target; `p42_camera_runtime.rs` gives an immediately exiting fake 100 ms so it can be observed after socket setup; `p46_direct_camera_adapter.rs` permits the supervisor's immediate first restart while the mock clock is fixed.
- No SDK build or real camera run was performed; the plan reserves hardware acceptance for LV4.

## Files changed
- Sidecar: `native/camera-sidecar/{CMakeLists.txt,README.md,include/crlab/{camera.hpp,liveview.hpp},src/{camera_capture.cpp,live_runner.cpp,live_view.cpp,main.cpp,sdk_adapter.cpp,sdk_internal.hpp,sdk_stub.cpp,sdk_symbols.def},tests/test_liveview.cpp}`.
- Core and Tauri: `crates/core/src/domain/{camera.rs,live.rs,mod.rs}`, `crates/core/src/{error.rs,ports/camera.rs,services/{current_shot.rs,live_view.rs,mod.rs}}`, `apps/desktop/src-tauri/src/commands/items.rs`.
- Integrations: `crates/integrations/src/camera/{runtime.rs,via_capture_one.rs,direct/{adapter.rs,live.rs,mod.rs},sidecar/{frames.rs,mod.rs,protocol.rs}}`.
- Testkit: `crates/testkit/src/{bin/fake_sidecar.rs,fake_direct_camera.rs,lib.rs,live_outlet.rs,test_app.rs}`, `crates/testkit/tests/suite/{lv1_frame_channel.rs,lv1_live_hub.rs,main.rs,p42_camera_runtime.rs,p45_sidecar_and_protocol.rs,p46_direct_camera_adapter.rs}`.
- Guide and phase record: `docs/guide/{02_architecture.md,03_rules_and_principles.md,04_change_impact.md,05_working.md}`, `docs/implementation/plans/LV1_camera_live_view.md`, and this handoff.

## Checkpoint SHA
1. Unit 1: `503e9ff0` — sidecar live view and focus.
2. Unit 2: `5ea89350` — live-view hub.
3. Unit 3: `107682ce` — frame channel, protocol, adapter and fake sidecar.

## Stamp
| Command | Result |
|---|---|
| `tools/sidecar/build.sh --stub --test` (Unit 1 and closing gate) | passed, 2 CTest targets, 31 transport and 17 live-view tests |
| `cargo test -p psm-testkit --test suite lv1_live_hub::` (Unit 2) | passed, 22 tests |
| `cargo test -p psm-core services::live_view` (Unit 2) | passed, 0 tests matched; mechanism behavior is tested at the outlet boundary |
| `cargo clippy -p psm-core -p psm-testkit --all-targets -- -D warnings` (Unit 2) | passed |
| `cargo test -p psm-testkit --test suite -- lv1_frame_channel:: lv1_live_hub:: p45_sidecar_and_protocol:: p46_direct_camera_adapter:: p42_camera_runtime::` (Unit 3) | passed, 98 tests |
| `cargo test -p psm-integrations camera::` (Unit 3) | passed, 23 tests |
| `cargo clippy -p psm-integrations -p psm-testkit --all-targets -- -D warnings` (Unit 3) | passed |
| `cargo test --workspace` (closing gate, run once) | passed, including 1,510 suite tests and all crate/doc tests |
| `cargo clippy --workspace --all-targets -- -D warnings` | passed |
| `cargo fmt --all -- --check` | passed |
| `tools/sidecar/build.sh --stub --test` | passed, 2 CTest targets |
| `cd apps/desktop && npm run typecheck` | passed |
| `cd apps/desktop && npm test` | passed, 138 files and 948 tests |

## Manual try-out
1. Run `tools/sidecar/build.sh --stub --test` to see the helper and its live-view mechanism tests pass without a camera.
2. Run `cargo test -p psm-testkit --test suite lv1_` to exercise frame ordering, repeat dropping, viewer demand, helper restart and focus mapping through the fake sidecar.
3. For optional hardware inspection after an SDK helper build, start the desktop app with `PSM_LOG=debug`, connect the camera, and look for `liveview.started`, a `live.source … frames_10s=` line every ten seconds, and `focus.request`/`focus.result` when focus is requested. No phone or Mac live screen exists in LV1.

## ⚠ OWNER DECISIONS REQUIRED (0)
none
