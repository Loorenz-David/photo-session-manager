---
plan: LV1_camera_live_view
role: review
round: 1
verdict: CHANGES_REQUESTED
state: DONE
date: 2026-11-26
actor: opus
---
# Handoff: LV1 camera live view, review r1

**Verdict: CHANGES_REQUESTED.** There are three blocking findings, all `route: production`. Two are B1: a restarted
helper loses its frame channel in a race, and a frame-channel write failure also stops tap-to-focus. One is B2: the
C15(b) test computes its expected value with the production function. Everything else checks out (list below), and the
checks are green.

## ⚠ OWNER DECISIONS REQUIRED (0)
None. All three findings are corrections inside the plan's own contracts.

## Gate
- The tracker read `IMPLEMENTED` (2026-11-26, codex).
- The checkpoints `503e9ff0`, `5ea89350` and `107682ce` (plus the handoff commit `fd5e4feb`) are all in `git log`.
- The intention is RATIFIED.
- Tree at review: `2e9a0e01`, clean.

## Verified correct
- **Perimeter.** `git show --stat` of the three unit checkpoints equals the handoff's "Files changed" list. These
  paths are outside the plan's file lists but declared under Deviations: `sdk_adapter.cpp`, `camera_capture.cpp`,
  `p42_camera_runtime.rs`, `p45_sidecar_and_protocol.rs`, `p46_direct_camera_adapter.rs`. `transport_manager.cpp` is
  untouched: FC7 is hooked through `SdkCameraAdapter::disconnect` → `before_disconnect` (a delegated choice).
- **Checks (review mode, run once, clean tree `2e9a0e01`).**
  - `tools/sidecar/build.sh --stub --test`: 2 CTest targets passed, including 17 live-view tests.
  - `cargo test -p psm-testkit --test suite -- lv1_live_hub:: lv1_frame_channel:: p45_sidecar_and_protocol:: p46_direct_camera_adapter::`: 86 passed.
  - `cargo clippy -p psm-core -p psm-integrations -p psm-testkit --all-targets -- -D warnings`: clean.
  - `cargo fmt --all -- --check`: clean.
  - The implementer's full-suite stamp stands; nothing contradicts it.
- **Sidecar (U1).** `LivePacer` matches the FC2 table: the `est` median of the last 16 intervals, clamped 20…200,
  next pull at `max(15, est−10)`. Repeats wait +5, NotUpdated +5, Error +20, and `errors` counts only for
  `sdk_error ≠ 0`. `FrameRecord::encode` follows the FC1 layout and the size bounds. The SOF reader works.
  `LatestSlot` replaces its item and never blocks the producer. `FocusQueue` has depth 1 and answers a superseded job.
  The `focus` parser rejects out-of-range or missing points. `ActionLock` lets a running focus finish, then gives the
  capture priority over a newly queued focus.
  - Tests C1(a–h), C2(a), C3(a–c), C4(a–b) and C5(a–c) assert the rows' exact values. C2(a) is non-discriminating by
    input: see the backlog plan lesson.
- **FC6 (by reading).** Every `SDK::SetDeviceProperty` and `SDK::SendCommand` in `native/camera-sidecar/src` is
  inside `CameraSession::Impl::write_camera(WriteIntent{StoreDestination, FocusPoint, HalfPress, ShutterRelease})`.
  `set_property_verified` is narrowed to the store destination. There is no `set_live_view_quality` or
  `set_focus_area` port. Live-view quality and Focus Area are only read.
- **FC4/FC5 (by reading).** Focus runs only on the runner's focus thread, under `action.lock_focus()`. A refusal for
  a non-movable focus area writes nothing. Capture holds `action.lock_capture()` on the main loop. Nothing runs on the
  pull thread except `GetLiveViewImage*`, which is called only after `connected()`.
- **FC7 (by reading).**
  - Quit, EOF, SIGINT and SIGTERM all lead to `tm.request_stop()`, then `intentional_disconnect`, then
    `adapter.disconnect()`. That calls `before_disconnect`, which runs `live->stop()`: the stop flag is set, the AF
    wait is cut (`cancel_focus` plus the condition variable), and all three threads are joined. Only then does
    `session_.disconnect()` run.
  - Session loss and transport switch go through the same `adapter_.disconnect()`.
  - The signal handlers only set a flag.
- **Hub (U2).** HB1–HB6 hold as written.
  - `seq` is one counter per process and never resets.
  - A viewer gets a frame only if it is wanting, the frame's `seq` is above its last, and the frame arrived after the
    switch.
  - Events reach every subscribed outlet in subscription order (BTreeMap by id).
  - `focus` checks the capability, then finite values in [0, 1]. `focus_clear` sends (0.5, 0.5) with autofocus off.
  - All 22 rows (C6–C11) have tests at the outlet/provider boundary that fail on a wrong outcome.
- **U3.**
  - Protocol rows C12(a–e) assert full values.
  - The frame reader checks magic, version, kind, reserved bytes and length (1…4 194 304). A bad record closes the
    stream, logs `live.stop reason=bad_record`, stops the source and latches for the process.
  - The adapter maps events per HB5: focus results including superseded/refused, focus frames, exposure, and capture
    reports. Focus is mapped per HB6: `to_grid` rounds and clamps, the `cmd_id` is fresh, and `CameraNotConnected`
    is returned without a session.
  - Capabilities: direct adapter true/true, Capture One adapter false/false.
  - On shutdown the app sends `quit`, the helper exits with code 0, and the socket file is removed.
  - Rows C13(a,b,d), C14(a–b), C15(a,c,d), C16(a–d) and C17(a–c) have tests that fail on a wrong outcome.
- **Nothing Sony-specific in `crates/core`.** No 639/479 grid or property names: the grid lives in
  `integrations/camera/direct/live.rs`.
- **H1–H8 hold.**
  - No file is moved or deleted (H2).
  - No pixel processing: the SOF header is parsed and JPEG bytes are passed through (H3).
  - The camera is driven only through the SDK (H5).
  - Core has no device concepts (H7).
  - The hub is in core with no Tauri or HTTP (H8).
  - The credentials first line is unchanged, `--frame-socket` is not a secret, and no test touches a live service.
- **Assumptions (handoff).** None contradicts 10J. One concerns per-process rebinding: the assumption is fine, but its
  timing is R1-B1. The others are within the delegated choices:
  - one reader thread and a 4 MiB bound;
  - the `/tmp` fallback;
  - deterministic fake bytes;
  - `live.source` computed from stats deltas;
  - the focus worker staying up while live view is disabled;
  - an empty `CurrentShots`.
- **Guide.** `02`, `03`, `04` and `05` are updated for the live path, the four write intents, the threads, the clean
  stop and the fake's directives.

## Findings

### R1-B1 (blocking, B1, route: production): a restarted helper's frame channel is lost to the rebind race
- **What is wrong.** The supervisor restarts the helper immediately after its first unexpected exit, and it does so
  inside `Supervisor::tick`, which returns `[Exited, Started]`. `CameraRuntime::tick` handles `Exited` *after* the
  new process already exists. Handling it means `old.stop()`, which joins the reader (up to ~30 ms) and removes the
  file, followed by `FrameListener::bind` on the same path.
- **Why it bites the real helper.** The real sidecar connects its frame socket at the start of `cmd_auto`. It
  therefore races the rebind:
  - If it connects to the closing listener, the connection is dropped. The first `send` fails, the runner emits
    `liveview.unavailable{frame_channel}` and stops, so live view is dead for the process.
  - If it connects in the window with no file, `frame_fd = -1`, and live view is unavailable for the whole process.
- **Why the test misses it.** The fake connects only at its first frame directive (U3 task 5), long after the rebind.
  That is why C13(c) is green.
- **Probe (temporary).** I added a scratch test in `lv1_frame_channel.rs` where each fake process connects at start
  by putting `frame 1 1 640 424 8` in its script. The restarted process's start-up frame never reached the viewer in
  2 of 3 runs. The file was reverted and its checksum verified identical.
- **Authority.** 10J MC-LV2 ("Live frames are pulled whenever the camera session is up"); master plan live §5 FC1
  ("The sidecar connects once at start … The app accepts one connection per sidecar process; a new connection
  replaces the old"); row LV1·C13(c).
- **Correction.** The listener the next helper process connects to must exist before that process is spawned. For
  example, keep one listener for the runtime's life, let a new accepted connection replace the old one (new
  `source`), and clear the bad-record latch on a new connection or helper exit. Alternatively, rebind before the
  supervisor can respawn. Make the fake connect at start (as FC1 says the sidecar does) so C13(c) observes the
  restart path the real helper takes.
- **Files the fix may touch:**
  - `crates/integrations/src/camera/sidecar/frames.rs`
  - `crates/integrations/src/camera/runtime.rs`
  - `crates/testkit/src/bin/fake_sidecar.rs`
  - `crates/testkit/tests/suite/lv1_frame_channel.rs`
  - `docs/guide/05_working.md` (if the fake's connect semantics change)

### R1-B2 (blocking, B1, route: production): a frame-channel write failure also stops tap-to-focus
- **What is wrong.** In `LiveRunner::Impl::sender()`, a failed `write_all` sets the runner-wide `stop = true` and
  calls `session.cancel_autofocus()`. The focus thread's `while (!stop)` then exits, and `running()` becomes false.
  From then on, every `focus` command in `cmd_auto` is answered `cmd.rejected {reason: "live view unavailable"}`
  until the session ends.
- **When it happens.** It happens whenever the app closes the frame connection, which is the designed reaction to a
  bad record (C13(d)). It also happens in R1-B1's race. At startup with no channel (`fd < 0`), by contrast, the focus
  worker does run. The behaviour is inconsistent.
- **Authority.** Master plan live §5 FC1 ("failure → `liveview.unavailable {reason: "frame_channel"}` and no live
  view for that process (control line unaffected)"); FC4 (a `focus` command is rejected only for `bad_point` or
  "not connected"; otherwise it is queued and answered).
- **Correction.** A frame-channel write failure ends only the pull and sender threads, after emitting
  `liveview.unavailable{frame_channel}` once. The focus worker keeps running, and `focus` stays accepted while the
  session is connected.
- **Files the fix may touch:**
  - `native/camera-sidecar/src/live_runner.cpp`
  - `native/camera-sidecar/include/crlab/liveview.hpp` (only if `LiveRunner`'s interface changes)
  - `native/camera-sidecar/src/main.cpp` (only if the `focus` branch's condition changes)

### R1-B3 (blocking, B2, route: production): C15(b)'s expected readout is computed by the production function
- **What is wrong.** `lv1_c15b_exposure_mapping` asserts
  `o.events().contains(&LiveEvent::Exposure(exposure_readout("1/125", "8.0", "100")))`. The expected value comes from
  the function under test. The only literal assertion is `aperture == "f/8"`. An adapter producing `iso: "100"` or a
  wrong shutter text would pass.
- **Authority.** Row LV1·C15(b): exact outcome `Exposure{shutter: "1/125", aperture: "f/8", iso: "ISO 100"}`. Flow
  §5 B2 ("asserts the fixture").
- **Correction.** Assert that the delivered event equals the literal
  `ExposureReadout{shutter: "1/125", aperture: "f/8", iso: "ISO 100"}`.
- **Files the fix may touch:** `crates/testkit/tests/suite/lv1_frame_channel.rs`.

## Lessons for the plans
- The fake sidecar's late frame-channel connect (U3 task 5) diverges from FC1's "connects once at start". It hid
  R1-B1. Later fakes should model the real connection timing.
- C2(a)'s input (frame 1 after 900) cannot catch a missing pacer reset when the sidecar compares `==`. A
  discriminating input is the same frame number after `reset`.
- U2's `cargo test -p psm-core services::live_view` selects no test.

## Notes written to backlog
These are in `docs/implementation/backlog_live/LV1_camera_live_view.md`, where the plan's Review mode routes them, with
a pointer line in `docs/implementation/backlog.md`. Eight notes:
- C13(d)'s weak "later frame" half.
- Leftover `psm-lv-*.sock` files in `$TMPDIR`.
- The store-destination write runs before the runner stops on a USB quit.
- The diagnostic sidecar commands ignore Ctrl-C.
- Two safe hub tightenings (a foreign source is `Stale`; extra `withdraw` calls) that LV2's outlet must expect.
- The three plan lessons above.

## Write perimeter (this session)
- `docs/implementation/plans/LV1_camera_live_view.md`: the Tracker line and the Review log entry.
- `docs/implementation/handoffs/reviewer/handoff_LV1_camera_live_view_review_r1.md` (this file).
- `docs/implementation/backlog_live/LV1_camera_live_view.md` (new) and one appended line in
  `docs/implementation/backlog.md`.
- **Probe declaration.** I temporarily appended one test function to
  `crates/testkit/tests/suite/lv1_frame_channel.rs`, ran it three times, then restored the file with
  `git checkout --`. `shasum -c` confirms it is byte-identical. No production file was edited.
- **Other side effects.** The stub sidecar build refreshed the ignored build outputs (`target/camera-sidecar/stub`,
  `apps/desktop/src-tauri/binaries/camera-sidecar-*`), and `git status` stayed clean. Test runs left frame-socket
  files in `$TMPDIR` (noted in the backlog). Logs are in `/tmp/lv1_*.log`.
- No commits; the runner commits these files.
