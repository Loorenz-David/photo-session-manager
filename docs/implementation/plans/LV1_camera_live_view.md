---
phase: LV1_camera_live_view
title: Live frames and focus reach the app — the camera helper pulls the camera's live picture (paced, never the same frame twice) and sends it on its own channel; focus from a point on its own thread with focused/not and the camera's focus frame; every stop ends the camera session cleanly; the app keeps only the newest frame and gives it only to viewers that show Live
difficulty: high
projection: no
depends_on: []
blocked_on: none
---
# LV1 — Camera live view to the hub

**Tracker:** CHANGES_REQUESTED (2026-11-26, codex) — fix 2 corrected R2-B1 and C13(c) now covers a restart while live view is off; closing workspace gate remains blocked by unrelated P50 test `p50_tests::c8a_support_and_secret_commands_have_their_wire_shapes` (`support_version_unsupported: unavailable`).

(This line is the milestone's tracker row. Sessions change only this line of this file.)

## Goal
Book `10J` §Architecture (sidecar, hub, focus), MC-LV1, MC-LV2 (hub side), LV-D14, by `master_plan_live.md` §5 **FC**
and **HB**. (U1) The sidecar's live view, frame channel, focus worker, action lock and clean stop, ported from the
camera lab. (U2) The live view hub and the focus API in core. (U3) The sidecar protocol additions, the frame reader,
the fake sidecar's frame channel, the direct adapter's capability, focus and event mapping.
**Not here:** the current shot (LV2), the phone server and app (LV2, LV3), the Mac's live screen and the Tauri
channel (card LV10), any live-view setting write (LV-S1: none needed), sleep (LV-D5 revised).

## How to work this plan (flow.md §10, master_plan_live.md §3)
One runner phase of 3 units, in order; after each, its **Unit test commands**, then a `CHECKPOINT (not approved):`
commit of that unit's paths. Row ids `LV1·C<n>(<row>)`; tests named after their row (`lv1_c6a_…`, C++ `TEST(lv1_c1a…)`).
Open details: the smallest option consistent with 10J, 10E MC-31 and §5, recorded under **Assumptions**. Reviewer
notes: `docs/implementation/backlog_live/LV1_camera_live_view.md`.

| Unit | What it builds | Criteria | Rows | Changes app files |
|---|---|---|---|---|
| U1 | Sidecar live view, frame record, focus worker, action lock, clean stop (C++) | 5 | 17 | yes (sidecar) |
| U2 | Live view hub, outlets, focus API (core) | 6 | 22 | yes (core) |
| U3 | Protocol, frame reader, fake sidecar, adapter capability, focus and event mapping | 6 | 22 | yes |
| **Total** | | **17** | **61** | |

## Read first (whole milestone)
`docs/process/flow.md` (§4, §4A, §6, §6A, §7A, §10) · `master_plan_live.md` (all; **§5 FC, HB**, §4.1–4.3, §8) · book
`10J` (§Architecture, MC-LV1, MC-LV2, LV-D2, LV-D14, LV-S1…LV-S6) · `10E` MC-31 (process, capture, R-I21, R-I23) ·
evidence `docs/intention/evidence/sony_camera_lab.md` §Live view — Experiment 07 and §Focus from the live picture —
Experiment 10, `docs/intention/evidence/live_preview_lab.md` · `docs/guide/03_rules_and_principles.md` §Camera hard
rules · `native/camera-sidecar/README.md` · the lab (read-only, never copied wholesale): `~/Desktop/Developer/
Camera-Connection-Tests/sdk/src/camera_liveview.cpp`, `sdk/src/main.cpp` (`cmd_liveview`, `--commands`, the stop
signal), `sdk/include/crlab/camera.hpp`, `sdk/src/sdk_symbols.def`, `experiments/07-live-view.md`,
`experiments/10-focus-from-live.md`.

## Points recorded by planning (each is one sentence in the narrative's "Choices made for you")
- **Live view starts by itself whenever the camera is connected** and the camera has live view enabled; the app never
  turns it on or changes its quality (on the a7R V nothing needs changing).
- **A tap moves the camera's focus point and half-presses; the app never changes the camera's focus area.** If the
  camera's focus area cannot be moved, the app says so and changes nothing.
- **A capture waits for a focus that is still running** (at most about a second), so the photo is focused where you
  tapped.

## Unit 1 — Sidecar live view, frame record, focus worker, action lock, clean stop

**Unit id:** `LV1·U1` · **Rows:** LV1·C1–C5 · **Changes app files:** yes (sidecar, C++)

### Goal
FC1 (the sidecar's side: connect, record, never on stdout), FC2 (pull loop and pacer), FC3 (sender), FC4 (focus
worker, focus frames, exposure readout), FC5 (action lock), FC6 (writes), FC7 (clean stop), ported from the lab.

### Read first
Master plan §5 FC, §4.1 · `native/camera-sidecar/src/main.cpp` (`cmd_auto`, `StdinQueue`, `parse_cmd`, `run_capture`)
· `native/camera-sidecar/src/transport_manager.cpp` (where a session becomes ready and where it disconnects) ·
`native/camera-sidecar/src/camera_capture.cpp` (half-press, release) · `native/camera-sidecar/src/camera_properties.cpp`
(the decoders) · `native/camera-sidecar/src/sdk_stub.cpp` · `native/camera-sidecar/tests/test_transport_manager.cpp`,
`tests/mini_test.hpp` · `native/camera-sidecar/CMakeLists.txt` · the lab files of the milestone's read-first list.

### Files
New: `native/camera-sidecar/include/crlab/liveview.hpp`, `native/camera-sidecar/src/live_view.cpp`,
`native/camera-sidecar/src/live_runner.cpp`, `native/camera-sidecar/tests/test_liveview.cpp` (and
`src/liveview.cpp` if the pure classes are not header-only). Edited: `native/camera-sidecar/include/crlab/camera.hpp`,
`native/camera-sidecar/src/{main.cpp, transport_manager.cpp, sdk_symbols.def, sdk_stub.cpp, sdk_internal.hpp}` (only
as the port needs), `native/camera-sidecar/CMakeLists.txt`, `native/camera-sidecar/README.md` (§What changed from the
lab: live view, focus, frame channel, signals).

### Core mechanisms
`LivePacer` (FC2), `FrameRecord::encode` and its JPEG size reader (FC1), `LatestSlot<T>` (FC3), `FocusQueue` and the
`focus` line parser (FC4). C++ tests only for these, in `tests/test_liveview.cpp`.

### Ordered tasks
1. Port the six `CameraSession` functions and the types of §4.1 into `src/live_view.cpp` / `camera.hpp`, with their
   symbol probes; the stub answers `Unsupported`/empty. Do **not** port the quality, focus-area or battery functions.
2. `liveview.hpp`: `LivePacer` (FC2 table, `est` rule), `FrameRecord` (FC1 layout), `LatestSlot<T>`, `FocusQueue`.
3. `LiveRunner` (`src/live_runner.cpp`): start after `connection.ready` (status check, `epoch`), pull thread (FC2),
   sender thread over the socket given by `--frame-socket` (FC1, FC3), focus worker with the 250 ms / 1000 ms idle reads
   (FC4), `liveview.started/stopped/unavailable/stats/focus_frame/exposure` and `focus.result` events (§4.1 names).
4. `cmd_auto`: parse `--frame-socket`; connect once (failure → `liveview.unavailable {reason: "frame_channel"}`);
   the `focus` command (FC4 parsing and rejections, then `cmd.done`); the action lock around `run_capture` (FC5);
   stop and join the runner before every disconnect, transport switch or session loss (FC7); SIGINT/SIGTERM handlers
   that only set a flag the loop turns into the `quit` path (FC7).
5. FC6: one write function taking the four intents; route the existing store-destination, S1 and release writes and the
   new `AF_Area_Position` write through it.
6. Tests for the rows; README.

### Criteria (5 criteria, 17 rows)
Boundary: the pure classes through `ctest` (stub build). Times are `now_ms` inputs, never the wall clock.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| LV1·C1(a) | `LivePacer::on_pull` | epoch 1, first `Ok` frame_no 7 at t=1000 | forward; next pull at 1030 (`est` 40 → max(15, 30)) | FC2, MC-LV1 |
| LV1·C1(b) | same | then `Ok` frame_no 7 at t=1031 | not forwarded; `repeats` 1; next pull 1036 | FC2, M-LV1 |
| LV1·C1(c) | same | `Ok` frame_no 0, bytes "AB", then frame_no 0 bytes "AC" | both forwarded | FC2 |
| LV1·C1(d) | same | `Ok` frame_no 0 bytes "AB" twice | second not forwarded (repeat); next pull +5 | FC2, MC-LV1 |
| LV1·C1(e) | same | `NotUpdated` at t | not forwarded; next pull t+5; `errors` unchanged | FC2 |
| LV1·C1(f) | same | `Error` with `sdk_error` 0 at t (first pull, size 0) | not forwarded; `errors` unchanged; next pull t+20 | FC2 (evidence: zero-size first pull) |
| LV1·C1(g) | same | `Error` with `sdk_error` 0x8000 at t | not forwarded; `errors` 1; next pull t+20 | FC2 |
| LV1·C1(h) | same | 17 new frames 50 ms apart, then 17 new frames 300 ms apart | after the first run next pull = last + 40; after the second = last + 190 (`est` clamped to 200) | FC2 |
| LV1·C2(a) | `LivePacer::reset(epoch 2)` | after epoch 1 ended at frame_no 900; `Ok` frame_no 1 | forwarded (a new epoch never compares with the old) | FC2, MC-LV1 |
| LV1·C3(a) | `FrameRecord::encode` | epoch 2, frame_no 7, 1024×680, JPEG of 5 bytes | exactly `50 53 4C 56 01 01 00 00 02 00 00 00 07 00 00 00 00 04 A8 02 05 00 00 00` then the 5 bytes | FC1 |
| LV1·C3(b) | same | JPEG of 0 bytes; JPEG of 4 194 305 bytes | both refused (nothing encoded) | FC1 |
| LV1·C3(c) | JPEG size reader | a minimal baseline JPEG 640×424; 10 random bytes | (640, 424); (0, 0) | FC1 |
| LV1·C4(a) | `LatestSlot<int>` | `put(1)`, `put(2)`, `take()` | `take` returns 2; a second `take` with 0 ms timeout returns nothing | FC3, M-LV1 |
| LV1·C4(b) | same | a consumer thread blocked in a slow write holds item 1; producer `put`s 2, 3, 4 | each `put` returns without waiting; the next `take` returns 4 | FC3, M-LV1 |
| LV1·C5(a) | `FocusQueue` | job A running; B pushed; C pushed | B reported superseded (its `cmd_id`); C is the next job | FC4 |
| LV1·C5(b) | `focus` line parser | `focus x=640 y=10 af=1 cmd_id=k`; `focus y=10 cmd_id=k`; `focus x=320 y=240 af=1 cmd_id=k` | `bad_point`; `bad_point`; job {320, 240, af true, "k"} | FC4 |
| LV1·C5(c) | action lock with two fake jobs | a focus job holds the lock 200 ms; a capture asks for it at +50 ms | the capture starts at ≥ +200 ms; a focus pushed during the capture starts after it ends | FC5, LV-D14 |

### Delegated choices
Header-only vs `src/liveview.cpp` · thread names · the FNV-1a implementation · how the runner hooks into the transport
manager's ready/disconnect points (as long as FC7 holds) · log wording on stderr.

### Notes
- Hazard (evidence Experiment 10): focus on the pull thread froze live view ~0.5 s; it must be the worker (FC4).
- Hazard (10E R-I23): a USB session is never left for a live-view error; pull errors only count and wait.
- Hazard (FC7): the SDK disconnect callback can arrive while the pull thread is in `GetLiveViewImage`; stop the thread
  first, then let the transport manager close.
- The automated suite cannot exercise the real pull loop; `LV4` does. Keep everything decidable in the pure classes.

### Unit test commands
`tools/sidecar/build.sh --stub --test`

## Unit 2 — Live view hub, outlets, focus API

**Unit id:** `LV1·U2` · **Rows:** LV1·C6–C11 · **Changes app files:** yes (core)

### Goal
HB1–HB4 (sources, publish, demand, slow viewers), HB5's fan-out of events, HB6's focus API, in
`crates/core/src/services/live_view.rs` and `crates/core/src/domain/live.rs`; `CameraCapabilities` and
`CameraProvider::focus`.

### Read first
Master plan §5 HB, §4.2 · `crates/core/src/ports/camera.rs` · `crates/core/src/domain/camera.rs` (`Link`,
`CameraCapabilities`) · `crates/core/src/error.rs` · `crates/core/src/services/mod.rs`, `crates/core/src/domain/mod.rs`
· `crates/testkit/src/fake_direct_camera.rs` (a test camera provider) · `crates/testkit/src/lib.rs`.

### Files
New: `crates/core/src/domain/live.rs`, `crates/core/src/services/live_view.rs`, `crates/testkit/src/live_outlet.rs`
(`RecordingOutlet`: records offers with gen, withdraws, events; its slot is a `LatestSlot`; `take()` for the test),
`crates/testkit/tests/suite/lv1_live_hub.rs`. Edited: `crates/core/src/{domain/mod.rs, services/mod.rs,
ports/camera.rs, error.rs}`, every `CameraCapabilities` literal (the direct adapter, the Capture One adapter, fakes),
`apps/desktop/src-tauri/src/commands/items.rs` (code `focus_unsupported`), `crates/testkit/src/lib.rs`,
`crates/testkit/tests/suite/main.rs`.

### Core mechanisms
`LatestSlot<T>` (Rust): `put` replaces, `take` empties.

### Ordered tasks
1. Domain types of §4.2 (`live.rs`); `CameraCapabilities.live_view`, `.focus_point` (false everywhere until U3);
   `CameraProvider::focus` with its failing default; `Error::FocusUnsupported`.
2. `LiveViewHub` exactly as HB1–HB5 (`source_started`, `source_stopped`, `publish`, `subscribe`, `set_demand`,
   `unsubscribe`, `report`, `newest`, `state`), `LiveOutlet`, `LatestSlot`.
3. `focus`, `focus_clear` (HB6).
4. `RecordingOutlet`; tests for the rows.

### Criteria (6 criteria, 22 rows)
Boundary: `LiveViewHub` and the focus functions through `RecordingOutlet`s and a test `CameraProvider` (outcome = what
each outlet was offered/withdrawn/told, the `PublishOutcome`, the provider calls received). Frames are `CameraFrame`s
built in the test (`source`, `frame_no`, JPEG bytes).
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| LV1·C6(a) | `publish` | started source S; frame_no 1, 2, 3; one viewer wanting (gen 1, switched before) | `Forwarded{seq}` 1, 2, 3; the outlet got seq 1, 2, 3 in order, each with gen 1 and its JPEG bytes unchanged | HB2, MC-LV1, M-LV1 |
| LV1·C6(b) | same | then frame_no 3 again, then 2 | both `Repeat`; nothing offered | HB2, MC-LV1 |
| LV1·C6(c) | same | `source_started` with source S′, frame_no 1 | `Forwarded{seq: 4}`; offered | HB1, HB2, MC-LV1 |
| LV1·C6(d) | same | no start at all; frame_no 1 | `Stale`; nothing offered; `newest()` none | HB2 |
| LV1·C6(e) | same | frame_no 0 twice (same source) | both `Forwarded` (the hub does not de-duplicate unnumbered frames; the sidecar did) | HB2, FC2 |
| LV1·C7(a) | `set_demand` | viewer `{live: false, visible: true}`; 5 frames | nothing offered | HB3, MC-LV2, M-LV2 |
| LV1·C7(b) | same | viewer `{live: true, visible: false}`; 5 frames | nothing offered | HB3, MC-LV2, M-LV2 |
| LV1·C7(c) | same | frame F1 published; then demand `{gen: 1, live: true, visible: true}`; then F2 | F1 never offered; F2 offered with gen 1 | HB3, M-LV2 |
| LV1·C7(d) | same | then demand `{gen: 2, live: false, visible: true}`; F3 | `withdraw` called once; F3 not offered | HB3, MC-LV2 |
| LV1·C7(e) | same | then demand `{gen: 3, live: true, visible: true}`; F4 | F4 offered with gen 3 and a seq above F2's | HB3, MC-LV1 |
| LV1·C8(a) | `LatestSlot` (core mechanism) | `put(a)`, `put(b)`, `take`, `take` | `b`, then nothing | HB4 |
| LV1·C8(b) | `publish` with two wanting viewers | viewer A never `take`s; viewer B `take`s after every publish; 10 frames | B took seq 1…10; A's slot holds only seq 10; every `publish` returned `Forwarded` | HB4, M-LV1 |
| LV1·C9(a) | `report` | three outlets (one wanting, one not wanting, one unsubscribed); `Focus{focus_id: "k", outcome: Focused}` | the two subscribed outlets got it once, in subscription order; the unsubscribed none | HB5 |
| LV1·C9(b) | same | `FocusFrames`, then `Exposure`, then `Capture{Exposed}` | each subscribed outlet got the three in that order | HB5 |
| LV1·C10(a) | `focus` | provider with `focus_point: false`; (0.5, 0.5) | `FocusUnsupported`; no provider call | HB6 |
| LV1·C10(b) | same | provider with `focus_point: true`; (0.25, 0.75) | provider received `({0.25, 0.75}, autofocus true)`; its id returned | HB6, LV-D14 |
| LV1·C10(c) | same | (1.5, 0.5); (NaN, 0.5); (0.5, −0.01) | each `InvalidValue`; no provider call | HB6 |
| LV1·C10(d) | `focus_clear` | provider with `focus_point: true` | provider received `({0.5, 0.5}, autofocus false)` | HB6, card LV9 |
| LV1·C11(a) | `source_stopped` | live with a wanting viewer; `source_stopped(SessionClosed)` | `state()` = `Off`; `newest()` none; every outlet got `State(Off)`; a frame published now is `Stale` | HB1 |
| LV1·C11(b) | `source_stopped` with an unavailable reason | `Unavailable{reason: "not_supported"}` | `state()` = `Unavailable{"not_supported"}`; outlets got that state | HB1, 10J §Status |
| LV1·C11(c) | `unsubscribe` | a wanting viewer unsubscribed; 3 frames | nothing offered to it; the other viewer got all three | HB3 |
| LV1·C11(d) | `subscribe` | a new viewer while frames flow, never sending demand | nothing offered (default demand is not wanting) | HB3, MC-LV2 |

### Delegated choices
Mutex vs `RwLock` inside the hub (no I/O under it either way) · `ViewerId` allocation · whether `LatestSlot` uses a
`Condvar` · the test provider's shape (a small struct in the stem or `fake_direct_camera.rs` extended).

### Notes
- `received` is `Instant` (monotonic): the "newer than the switch" rule never uses the `ManualClock`, which tests
  freeze.
- `offer`, `withdraw` and `event` are called under the hub mutex: they must only touch the outlet's own slot/queue.

### Unit test commands
`cargo test -p psm-testkit --test suite lv1_live_hub::` · `cargo test -p psm-core services::live_view` ·
`cargo clippy -p psm-core -p psm-testkit --all-targets -- -D warnings`

## Unit 3 — Protocol, frame reader, fake sidecar, adapter capability, focus and event mapping

**Unit id:** `LV1·U3` · **Rows:** LV1·C12–C17 · **Changes app files:** yes

### Goal
FC1 (the app's side: listener, reader, records), HB1's adapter calls, HB5's event mapping, HB6's grid mapping, the
`live_view`/`focus_point` capability of the direct adapter, the runtime's `live_hub()` / `current_shots()` holders
(the second is an empty `CurrentShots` until LV2), and the fake sidecar's frame channel.

### Read first
Master plan §5 FC1, HB1, HB5, HB6, §4.3 · `crates/integrations/src/camera/sidecar/{protocol.rs, client.rs,
supervisor.rs, mod.rs}` · `crates/integrations/src/camera/runtime.rs` (`SidecarSpec` construction near the `"auto"`
argument, `tick`, `shutdown`) · `crates/integrations/src/camera/direct/adapter.rs` (`apply`, `capture`, `capabilities`)
· `crates/integrations/src/camera/via_capture_one.rs` (`capabilities`) · `crates/testkit/src/bin/fake_sidecar.rs` ·
`crates/testkit/src/test_app.rs` (`with_camera_script…`, `camera_runtime`, `capture`, `inject`) ·
`crates/testkit/tests/suite/{p45_sidecar_and_protocol.rs, p46_direct_camera_adapter.rs}`.

### Files
New: `crates/integrations/src/camera/sidecar/frames.rs`, `crates/integrations/src/camera/direct/live.rs`,
`crates/testkit/tests/suite/lv1_frame_channel.rs`. Edited: `crates/integrations/src/camera/sidecar/{protocol.rs,
mod.rs}`, `crates/integrations/src/camera/runtime.rs`, `crates/integrations/src/camera/direct/{adapter.rs, mod.rs}`,
`crates/integrations/src/camera/via_capture_one.rs`, `crates/core/src/services/current_shot.rs` (new: the empty
`CurrentShots` type only; LV2 fills it), `crates/core/src/services/mod.rs`, `crates/testkit/src/bin/fake_sidecar.rs`,
`crates/testkit/src/test_app.rs` (`live_hub()`), `crates/testkit/tests/suite/main.rs`; `docs/guide/{02_architecture.md,
03_rules_and_principles.md, 04_change_impact.md, 05_working.md}`.

### Core mechanisms
`frames::parse_record` (FC1 header checks), `direct::live::{to_grid, frame_norm, exposure_readout}`.

### Ordered tasks
1. Protocol: `Command::Focus` encode; the seven events parsed (§4.3 names, fields of §4.1).
2. `frames.rs`: listener at `$TMPDIR/psm-lv-<run id>.sock` created before the spawn (stale file removed, mode 0600),
   `--frame-socket <path>` added to the spawn arguments, one accepted connection per process (a new one replaces the
   old), the reader thread `camera-frames` → `parse_record` → `hub.publish` with `source = (connection number << 32) |
   epoch`; bad record → close, `live.stop reason=bad_record`, `source_stopped`; socket file removed on sidecar exit and
   in `shutdown`. `live.source` every 10 s.
3. Runtime holders `live_hub()`, `current_shots()` created once with the runtime (R-I18).
4. Adapter: `capabilities()` → `live_view: true, focus_point: true` (Capture One adapter: false); `focus()` per HB6
   (`to_grid`, `Command::Focus`, fresh `cmd_id` = `FocusId`; not connected → `CameraNotConnected`); events →
   `source_started`/`source_stopped`/`report` per HB1 and HB5; `focus.request` / `focus.result` log lines.
5. Fake sidecar: parse `--frame-socket`; connect when the first frame directive runs; directives `frame <epoch>
   <frame_no> <w> <h> <len>` (JPEG-like bytes of `len`, deterministic content from the numbers) and `frames <epoch>
   <first> <count> <interval_ms>`; `badframe` writes a record with a wrong magic; substitution `{focus_cmd_id}` = the
   last `focus` line's `cmd_id` (as `{cmd_id}` for capture).
6. Tests for the rows; guide (flow §7A): `02_architecture.md` (camera: live view path, frame channel, hub, focus),
   `03_rules_and_principles.md` (camera hard rules: the four write intents, focus/capture threads, clean stop),
   `04_change_impact.md` (rows: a sidecar command/event ↔ `protocol.rs` ↔ fake sidecar; the frame record ↔
   `frames.rs` ↔ the fake's directives; `CameraCapabilities` fields ↔ every adapter literal), `05_working.md` (fake
   sidecar directives, `--frame-socket`).

### Criteria (6 criteria, 22 rows)
Boundary: protocol functions (C12); otherwise the wired TestApp with the fake sidecar (`TestApp::with_camera_script…`),
a `RecordingOutlet` subscribed to `live_hub()` with a wanting demand, the fake's recorded stdin and exits, the store.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| LV1·C12(a) | `parse_line` | `{"event":"liveview.started","epoch":1,"transport":"usb","status":"enabled","frame_channel":"connected",…envelope}` | `LiveviewStarted{epoch: 1, transport: Usb, status: "enabled", frame_channel: "connected"}` | FC2, §4.1 |
| LV1·C12(b) | same | `focus.result` with `cmd_id "k"`, `x 320`, `y 240`, `af "focused"`, `point_set true`, `total_ms 412` | `FocusResult` with exactly those values | FC4 |
| LV1·C12(c) | same | `liveview.focus_frame` with `frames [[1,2,320,640,240,480,38,38]]` | `LiveviewFocusFrame{frames: [[1,2,320,640,240,480,38,38]]}` | FC4 |
| LV1·C12(d) | `Command::encode` | `Focus{x: 320, y: 240, af: true, cmd_id: "k1"}`; `Focus{x: 0, y: 479, af: false, cmd_id: "k2"}` | `focus x=320 y=240 af=1 cmd_id=k1`; `focus x=0 y=479 af=0 cmd_id=k2` | FC4 |
| LV1·C12(e) | `parse_line` | `{"event":"liveview.zoom",…envelope}` | `Unknown` (ignored) | 10E MC-31 (unknown events ignored) |
| LV1·C13(a) | frame channel → hub | script: `connection.ready` (USB), `liveview.started` (epoch 1, enabled, connected), `frames 1 1 5 20` | the outlet got 5 frames, seq strictly increasing, each JPEG byte-identical to what the fake wrote, width/height as written | FC1, HB2, MC-LV1 |
| LV1·C13(b) | same | `frame 1 5 …`, `frame 1 5 …`, `frame 1 6 …` | frame 5 offered once, then 6 (the repeat dropped) | HB2, M-LV1 |
| LV1·C13(c) | same | with live view off: the fake exits 1; the restarted fake sends `liveview.started` epoch 1 and `frames 1 1 2 20` | both frames from the restarted helper are offered, with strictly increasing seq | FC1, HB1, MC-LV1, MC-LV2 |
| LV1·C13(d) | same | `badframe` | the connection is closed; the log has `live.stop reason=bad_record`; `state()` = `Off`; a later good `frame` on that process is not offered | FC1 |
| LV1·C14(a) | `TestApp::capture` while frames flow | script floods `frames 1 1 200 1`; the outlet never `take`s; the app captures; script answers `capture.exposed` for `{capture_id}` | the fake's stdin has exactly one `capture id=<id> af=1` line; one `camera_exposures` row exists for that capture id | FC5, HB4, M-LV1, M-LV3 |
| LV1·C14(b) | same | the same capture with no `liveview.started` at all | the identical line shape `capture id=<id> af=1`; one exposure row | M-LV3 (live view on or off) |
| LV1·C15(a) | adapter → hub event | `liveview.focus_frame [[1,2,320,640,240,480,38,38]]` | outlets got `FocusFrames([{cx: 0.5, cy: 0.5, w: 0.059375, h: 0.0791667 (±1e-6), state: Focused}])` | HB5, LV-D14 |
| LV1·C15(b) | same | `liveview.exposure {f_number: "8.0", shutter_speed: "1/125", iso: "100"}` | `Exposure{shutter: "1/125", aperture: "f/8", iso: "ISO 100"}` | HB5, 10J §phone design |
| LV1·C15(c) | same | after `focus(0.5, 0.5)`: `focus.result {cmd_id: "{focus_cmd_id}", af: "focused", point_set: true}` | outlets got `Focus{focus_id: <the returned FocusId>, outcome: Focused}` | HB5, HB6 |
| LV1·C15(d) | same | `focus.result {cmd_id: "{focus_cmd_id}", refused: "focus_area_not_movable", point_set: false, af: "skipped"}` | `Focus{outcome: Refused(AreaNotMovable)}`; the fake's stdin after the `focus` line has no further line from the app | HB5, FC4, FC6, M-LV5 |
| LV1·C16(a) | `live_view::focus` through the direct adapter | connected; (0.5, 0.5) | fake stdin `focus x=320 y=240 af=1 cmd_id=<FocusId>` | HB6, LV-D14 |
| LV1·C16(b) | same | (0, 1); (1, 0) | `focus x=0 y=479 af=1 …`; `focus x=639 y=0 af=1 …` | HB6 |
| LV1·C16(c) | `focus_clear` | connected | `focus x=320 y=240 af=0 cmd_id=…` | HB6, card LV9 |
| LV1·C16(d) | `focus` | sidecar running, no camera session | `CameraNotConnected`; nothing written to the fake's stdin | HB6 |
| LV1·C17(a) | runtime `shutdown` during live view | frames flowing | the fake's stdin ends with `quit`; `exits` records code 0 for it (no kill); the socket file no longer exists | FC7, 10E MC-31 |
| LV1·C17(b) | `capabilities()` | the direct adapter; the Capture One camera adapter | `live_view && focus_point` true; both false | 10E §Port changes (10J amendment) |
| LV1·C17(c) | adapter → hub | `liveview.unavailable {reason: "not_supported"}` then `frame 1 1 …` | `state()` = `Unavailable{"not_supported"}`; the frame is `Stale`, not offered | HB1, 10J §Status |

### Delegated choices
Where the listener object lives inside the runtime · the reader's buffer sizes · how the fake builds JPEG-like bytes ·
whether `frames` writes on a thread or inline in the script loop.

### Notes
- Hazard (P70): tests must not wait on wall-clock seconds; use `SidecarTimings`-style knobs if a wait is needed and
  the fake's deterministic directives.
- Hazard (10E R-I21): the `credentials` first line is unchanged; `--frame-socket` is an argument (not secret).
- Hazard: `$TMPDIR` paths in tests can be long; the run id keeps the name short (≤ 104 bytes total; assert it in
  `frames.rs` and fall back to `/tmp/psm-lv-<run id>.sock` with the same mode if longer).

### Unit test commands
`cargo test -p psm-testkit --test suite -- lv1_frame_channel:: lv1_live_hub:: p45_sidecar_and_protocol:: p46_direct_camera_adapter:: p42_camera_runtime::` ·
`cargo test -p psm-integrations camera::` · `cargo clippy -p psm-integrations -p psm-testkit --all-targets -- -D warnings`

## Closing gate (once, after the last unit; replaces the template's closing gate)
`cargo test --workspace` · `cargo clippy --workspace --all-targets -- -D warnings` · `cargo fmt --all -- --check` ·
`tools/sidecar/build.sh --stub --test` · `cd apps/desktop && npm run typecheck` · `npm test` (run `npm ci` first if
`node_modules` is missing). One command at a time. A failure is fixed and only the failing files re-run.

## Review mode
The reviewer reads the units in order and runs, once: `tools/sidecar/build.sh --stub --test` · `cargo test -p
psm-testkit --test suite -- lv1_live_hub:: lv1_frame_channel:: p45_sidecar_and_protocol:: p46_direct_camera_adapter::`
· clippy on the touched crates, fmt. The implementer's full-suite stamp stands. Only B1–B4 block. Also check, by
reading: every `SetDeviceProperty` in `native/camera-sidecar/src` goes through the one write function with the four
intents (FC6); no `set_live_view_quality` / `set_focus_area` port; focus and capture never run on the pull thread; every
stop path joins the runner before disconnecting (FC7); nothing Sony-specific (grid, property names) in `crates/core`.
Notes go to `docs/implementation/backlog_live/LV1_camera_live_view.md`.

## Manual try-out (the narrative is built from this)
No phone yet; this milestone is visible in the logs.
1. `tools/sidecar/build.sh --stub --test`: the helper builds and its tests pass (with the stub there is no camera).
2. Run the test suite part for live view: `cargo test -p psm-testkit --test suite lv1_` — every live-view test passes
   (frames arrive in order, a repeated frame is dropped, a slow viewer only gets the newest, a stopped helper stops
   live view).
3. With the camera (optional, needs the SDK build of the helper): start the app from a terminal with
   `PSM_LOG=debug`; connect the camera; the log shows `liveview.started` and every 10 s a `live.source … frames_10s=`
   line near 250 on USB.

## Review log
- 2026-11-26, codex, implement r1: Ported camera live pull, pacing, JPEG frame record, focus worker and clean stop; added the app live hub, viewer demand, binary frame reader, protocol, direct adapter mapping, fake sidecar, and guide. Assumed one bounded reader thread per runtime, deterministic fake JPEG-like bytes, `/tmp` fallback for long socket paths, and 10-second source counts from sidecar stat deltas. Deviations: also edited `sdk_adapter.cpp` and `camera_capture.cpp` for central camera writes and compile guards; corrected the Unit 1 runner during Unit 3; formatted Unit 2 Rust; updated three legacy camera test files for the added CTest target and process startup timing. Unit checkpoints: `503e9ff0`, `5ea89350`, `107682ce`. Closing gate passed once (workspace tests, clippy, fmt, stub sidecar tests, desktop typecheck and tests). No camera hardware or SDK build was run; that belongs to LV4.
- 2026-11-26, opus, review r1: **CHANGES_REQUESTED.** Perimeter = the declared files (`503e9ff0`, `5ea89350`, `107682ce`; the three extra camera test files and `sdk_adapter.cpp`/`camera_capture.cpp` are declared deviations). Review-mode checks green on clean `2e9a0e01`: stub sidecar 2 CTest targets (17 live-view tests), `--test suite` lv1/p45/p46 86 passed, clippy (core, integrations, testkit), fmt. Verified by reading: FC6 (every `SetDeviceProperty`/`SendCommand` goes through `write_camera` with the four intents; no quality/Focus Area write or port), FC4/FC5 (focus on its worker under `ActionLock`, capture on the main loop, neither on the pull thread), FC7 (runner stopped and joined in `before_disconnect` before every SDK disconnect: quit, EOF, signals, session loss, switch), HB1–HB6, nothing Sony-specific in core, H1–H8. Rows C1–C12, C13(a,b,d), C14, C15(a,c,d), C16, C17 have tests that observe the outcome and fail on a wrong one; C13(c)'s test passes only because the fake connects late (R1-B1). Blocking: **R1-B1** (route production) after an unexpected helper exit the supervisor respawns in the same `tick` *before* the runtime rebinds the frame socket; the real sidecar connects at start, so its connection lands on the closing listener (or on no file) and that process has no live view; reviewer probe (temporary scratch test, reverted, checksum identical): restarted fake connecting at start lost its frame in 2 of 3 runs. Authority FC1, MC-LV2, row C13(c). **R1-B2** (route production) `LiveRunner` sender write failure sets the shared stop flag, so the focus worker exits and `focus` is rejected "live view unavailable" for the rest of the session (FC1 "control line unaffected", FC4 rejection rules). **R1-B3** (route production, B2) `lv1_c15b_exposure_mapping` computes the expected readout with `exposure_readout` itself; only the aperture is literal, so a wrong shutter or ISO text passes. Notes: `docs/implementation/backlog_live/LV1_camera_live_view.md`. Lessons: the fake's "connect at the first frame directive" (U3 task 5) diverges from FC1's "connects once at start" and hid R1-B1; C2(a)'s input cannot distinguish a missing pacer reset (the sidecar compares `==`); the U2 command `cargo test -p psm-core services::live_view` selects no test.
- 2026-11-26, codex, fix 1: Resolved R1-B1 by keeping the runtime frame listener bound across helper restarts, clearing its malformed-record latch on a new helper connection/exit, and making the fake connect when it starts; C13(c) now exercises startup timing. Resolved R1-B2 by separating frame-stream shutdown from the runner-wide stop flag, so a failed frame-channel write ends pull/sender while leaving focus running. Resolved R1-B3 by asserting C15(b) against a literal `ExposureReadout`. No divergence or additional judgment call. Targeted tests and closing gates are recorded in `handoff_LV1_camera_live_view_fix_r2.md`.
- 2026-11-26, opus, re-review r2: **CHANGES_REQUESTED (1 MUST).** Perimeter: `f7d03aa1` touches exactly the six allowed files; later commits are runner/orchestrator docs only. Checks once on clean `2df3bd49`: `--test suite lv1_frame_channel::` 24 passed; `-p psm-integrations camera::` 23 passed; suite `lv1_live_hub:: p42_camera_runtime:: p45_sidecar_and_protocol:: p46_direct_camera_adapter::` 74 passed (the fake changed); stub sidecar 2 CTest targets passed; clippy (integrations, testkit) clean; fmt clean; desktop typecheck clean. **R1-B2 resolved** (`stream_stop` ends only pull/sender; the focus worker loops on `stop`, `running()` stays true, `focus` stays accepted; `liveview.unavailable{frame_channel}` emitted once; full `stop()` sets both flags). **R1-B3 resolved** (C15(b) asserts the literal `ExposureReadout{"1/125","f/8","ISO 100"}`). **R1-B1 partly resolved**: the listener now outlives helper processes, so a restart while frames flow works (C13(c) green, connects at start like the real helper). **R2-B1** (blocking, B1, route production, triage MUST): `FrameListener` serves one connection at a time and, while no source is active, parks in `read_stream`'s wait loop without reading, so it never sees the old helper's EOF and never accepts the newer connection. When a helper exits while live view is not running (camera not in session, or live view unavailable) and a new helper starts (crash restart; `restart_for_document`; `restart_if_not_usb` on Wi-Fi credentials or stale-credential session loss), the next `liveview.started` activates the dead stream, the reader hits EOF, sends `source_stopped(SidecarExit)`, clears `active`, then accepts the new helper with no active source: no live frames for that whole camera session. Reviewer probe (scratch test in `lv1_frame_channel.rs`, reverted, checksum identical): restart with live off, then `liveview.started` + `frames 1 1 3 20` → nothing offered, 3/3 runs; same steps with live running at the restart → frames offered, 3/3. Regression for crash and document restarts (r1 rebound there); pre-existing for `restart_if_not_usb`. Authority FC1 ("a new connection replaces the old (new source)"), MC-LV2, row C13(c). Notes: three, in `docs/implementation/backlog_live/LV1_camera_live_view.md`.

FIX2_AUTHORISED: R2-B1 is a user-visible wrong outcome (blank live picture for a whole camera session after a routine helper restart while live view is off: switching Capture One session, saving Wi-Fi credentials, crash restart), reproduced 3/3 by the reviewer through product paths; FC1 "a new connection replaces the old (new source)" and MC-LV2. Fix 2 implements the replace half of FC1 (a newer connection wins; a dead parked stream never survives the wait) and extends C13(c) with the restart-while-live-view-off case (the reviewer's probe (a)). Perimeter: `crates/integrations/src/camera/sidecar/frames.rs`, `crates/integrations/src/camera/runtime.rs`, `crates/testkit/tests/suite/lv1_frame_channel.rs`. (Orchestrator, 2026-11-26.)

- 2026-11-26, codex, fix 2: Resolved R2-B1 by polling the nonblocking frame listener while the current connection waits for activation, replacing the parked stream when the restarted helper has connected; if activation races with replacement, the hub receives the replacement source identity. Extended C13(c) to restart before any `liveview.started` event. Targeted suite C13 module (24/24), `psm-integrations camera::` (23/23), clippy, fmt, stub sidecar (2 targets), desktop typecheck and UI tests (138 files / 948 tests) passed. Closing `cargo test --workspace` failed at unrelated P50 `p50_tests::c8a_support_and_secret_commands_have_their_wire_shapes` with `support_version_unsupported: unavailable`; its targeted retry waited on the Cargo build lock and was stopped. Tracker remains CHANGES_REQUESTED pending that gate. See fix r3 handoff.
