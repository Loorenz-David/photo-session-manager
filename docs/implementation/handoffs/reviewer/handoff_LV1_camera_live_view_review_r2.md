---
plan: LV1_camera_live_view
role: review
round: 2
verdict: CHANGES_REQUESTED
state: DONE
date: 2026-11-26
actor: opus
---
# Handoff: LV1 camera live view, re-review r2

**Verdict: CHANGES_REQUESTED, with one MUST item (R2-B1).** Fix 1 resolved R1-B2 and R1-B3 and part of R1-B1. The
frame listener now outlives helper processes, so a restart while frames are flowing works. But the listener serves one
connection at a time, and while no live source is active it stops reading. A helper that exits while live view is off
therefore leaves a dead connection in place. The next camera session activates that dead connection, the source is
stopped as `SidecarExit`, and the new helper's frames never reach the hub for that whole session. I reproduced this 3/3
with a scratch test, against a 3/3 green control. The one automatic fix is spent (flow §5), so the orchestrator decides.

## ⚠ OWNER DECISIONS REQUIRED (0)
None. R2-B1 is a correction inside FC1's own contract. Whether to authorise fix 2 is the orchestrator's call
(`FIX2_AUTHORISED`), not an owner card.

## Review history
- **r1** settled everything except R1-B1, R1-B2 and R1-B3: the sidecar pacer, record, slot, focus queue and action
  lock; FC4–FC7 by reading; hub HB1–HB6; protocol and adapter mapping; H1–H8; the guide. This round does not re-verify
  those areas.
- **This round** covers the fix delta (`f7d03aa1`) and its seam: the frame listener's connection lifecycle across helper
  restarts, and the live runner's stop flags.

## Gate
- The tracker read `IMPLEMENTED (2026-11-26, codex)` with the fix-1 note.
- The fix checkpoint `f7d03aa1` ("CHECKPOINT (not approved): LV1 camera live view fix 1") is in `git log`. The
  orchestrator committed it because the fix sandbox could not write the worktree index; the reason is recorded in the
  fix handoff.
- The intention is RATIFIED, and 10J is RATIFIED.
- Tree at review: `2df3bd49`, clean.

## Perimeter check
- `git show --stat f7d03aa1` lists `runtime.rs`, `sidecar/frames.rs`, `fake_sidecar.rs`, `lv1_frame_channel.rs`,
  `docs/guide/05_working.md` and `live_runner.cpp`. All six are in the r1 findings' allowed lists. `liveview.hpp` and
  `main.cpp` were not needed and are untouched.
- The other changes in `git diff efff7ffc HEAD` are runner and orchestrator artifacts: the prompts, the fix handoff,
  the plan tracker and Review log lines, `STATUS_live.md`, `state/*.json` and `run_phase.sh` (the sandbox fix from the
  orchestrator's commit `e9f4ef6e`). None is a fix-session production edit.
- **Result: in perimeter.**

## Checks
Run once each, one cargo command at a time, on clean `2df3bd49`:

| Command | Result |
|---|---|
| `cargo test -p psm-testkit --test suite lv1_frame_channel::` | 24 passed |
| `cargo test -p psm-integrations camera::` | 23 passed |
| `cargo test -p psm-testkit --test suite -- lv1_live_hub:: p42_camera_runtime:: p45_sidecar_and_protocol:: p46_direct_camera_adapter::` (other users of the changed fake) | 74 passed |
| `tools/sidecar/build.sh --stub --test` | 2/2 CTest targets passed (live-view tests included) |
| `cargo clippy -p psm-integrations -p psm-testkit --all-targets -- -D warnings` | clean |
| `cargo fmt --all -- --check` | clean |
| `npm run typecheck` (apps/desktop) | clean |

No full suite was run: review mode, and the fix changed no shared store or service.

## Findings resolved
- **R1-B2: resolved as corrected.**
  - `LiveRunner::Impl` now has a separate `stream_stop`. `write_all`, `pull` and `sender` loop on it. On a write
    failure, `sender` emits `liveview.unavailable{frame_channel}` once (only if not shutting down), sets `stream_stop`
    and exits.
  - `focus()` still loops on `stop`. `running()` (`started && !stop`) stays true, so `main.cpp`'s `focus` branch keeps
    queueing jobs while the session is connected (FC1 "control line unaffected"; FC4).
  - `start()` clears both flags. `stop()` sets both, then closes the queue and slot and joins all three threads, so FC7
    is unchanged.
  - Checked in passing: a later session in the same process restarts the stream on the same broken fd and re-emits
    `frame_channel` unavailable once per session. There is still no live view for that process, as FC1 requires. This
    is not a regression.
- **R1-B3: resolved as corrected.** `lv1_c15b_exposure_mapping` asserts that the delivered events contain the literal
  `ExposureReadout{shutter: "1/125", aperture: "f/8", iso: "ISO 100"}`. The `exposure_readout` import is gone. A wrong
  shutter or ISO text now fails the test.
- **R1-B1: partly resolved; the remainder is R2-B1.**
  - `spawn_helper` binds the listener once, only when none exists, and `Exited` no longer rebinds. The socket path
    therefore exists before any respawn, and the original race (a connection landing on the closing listener, or no
    file) is gone.
  - The fake connects at process start, as FC1 says the real helper does, and C13(c) exercises that timing.
  - `helper_exited()` and every new accept clear the bad-record latch.
  - My control probe passes 3/3: restart while live view is running, then frames from the new helper reach the viewer.
  - The correction's other half ("let a new accepted connection replace the old one (new source)", FC1) is not
    delivered, as R2-B1 shows.

## Remaining findings

### R2-B1 (blocking, B1, route: production): after a helper restart while live view is off, the next camera session gets no live frames
- **What is wrong.**
  - `FrameListener`'s thread handles one connection at a time: `accept` → `read_stream` → back to `accept`.
  - Inside `read_stream`, while `state.active` is `None`, it sits in the wait loop (`sleep 5 ms`, re-check). It never
    reads the stream, so it cannot see the peer's EOF and cannot pick up a newer connection waiting in the backlog.
- **How the failure unfolds.**
  1. The old helper exits while no live source is active: the camera is not in a session, or live view was reported
     unavailable or disabled.
  2. The new helper connects at start, and its connection waits in the backlog.
  3. The new helper's first `liveview.started` triggers `live_start` → `start_source`. That sets `active` and calls
     `hub.source_started` with the *old* connection's source id.
  4. The parked thread wakes on the dead stream, reads EOF, sets `active = None` and calls
     `hub.source_stopped(SidecarExit)`.
  5. Only then does it accept the new helper's connection, with no active source.
  6. No frame is published until that camera session ends and a later `liveview.started` arrives.
- **Restart paths that reach it.** All are designed paths, and the real helper connects in `cmd_auto` before any
  camera exists (`main.cpp`):
  - a crash restart through `Supervisor` (`Exited`);
  - `restart_for_document`, when the owner opens another Capture One session;
  - `restart_if_not_usb`, when Wi-Fi credentials are saved in Settings or a USB session is lost with stale credentials.
    By construction this only fires without a USB session, so live view is normally off at that moment.
- **Regression status.** It is a regression for the crash and document paths: r1's code rebound the listener on both,
  which dropped the parked stream. It is pre-existing for `restart_if_not_usb`, which never rebound.
- **Evidence (reviewer probe).**
  - I appended two scratch tests to `lv1_frame_channel.rs`, each starting from `app()` + `viewing()`.
  - (a) No `liveview.started`; `inject_exit(1)`; wait for pid 2 and Connected; `liveview.started` epoch 1;
    `frames 1 1 3 20`; settle on any offered frame. **Failed 3/3**: events show both `ConnectionReady` lines and the
    second `LiveviewStarted`, and nothing was offered.
  - (b) The same, except that `liveview.started` is sent first and the test waits for `live_hub().state() != Off` plus
    100 ms before the exit. **Passed 3/3.**
  - A first version of (b) that did not wait collapsed into case (a) and also failed. That is the same defect: the exit
    was handled before the reader left its wait loop.
  - The file was restored with `git checkout --`, `shasum -c` reports OK, and `git status` is clean.
- **Authority.**
  - Master plan live §5 FC1: "The app accepts one connection per sidecar process; a new connection replaces the old
    (new source)".
  - 10J MC-LV2: "Live frames are pulled whenever the camera session is up".
  - Row LV1·C13(c): a restarted helper's frames are offered.
- **Correction.** A newer connection must win, and a dead one must not survive the wait. Two options:
  - While waiting for activation, the reader also notices EOF on its stream and a pending new connection. For example,
    a non-blocking `accept` (or a zero-timeout `peek`/`read`) inside the wait loop, so that a newer connection drops
    the parked one.
  - Or the runtime signals the listener on every helper replacement, so the parked stream is dropped. This means
    `Exited`, and also the two `quit_and_wait` restart paths, which do not produce `Exited`.

  Either way, cover it by extending C13(c), or adding its sibling row, with the restart happening while live view is
  off (probe (a) above). That is the boundary at which the real helper fails.
- **Files the fix may touch:**
  - `crates/integrations/src/camera/sidecar/frames.rs`
  - `crates/integrations/src/camera/runtime.rs`
  - `crates/testkit/tests/suite/lv1_frame_channel.rs`
- **Triage: MUST.** It is a user-visible wrong outcome: a blank live preview on the Mac (and later on phones) for a
  whole camera session after a routine action such as switching the Capture One session or saving Wi-Fi credentials.
  It is not speculative: the probe reproduces it through product paths every time.

## Triage (flow §5)
| Item | Classification | Reason |
|---|---|---|
| R2-B1 restart while live view is off → no live frames for the next camera session | **MUST** | User-visible wrong outcome (MC-LV2, FC1); deterministic; regression on two restart paths |
| Note: `previous_connections` / `connection_count()` now dead | DROP → backlog | Refactor |
| Note: the accept thread never restarts after a non-`WouldBlock` accept error | DROP → backlog | Speculative robustness |
| Note: the fake reconnects after a failed write (the real helper never does) | DROP → backlog | Test-fake fidelity; no row affected today |

## Lessons for the plans
- The r1 correction offered two shapes: "persistent listener with replace-on-new-connection" and "rebind before
  respawn". The fix took the first shape's half that is easy to see (persistence) and missed its load-bearing half
  (replace). Future correction text should list the restart states to cover: frames flowing, live view off, after a bad
  record.
- C13(c) enumerates one restart state (frames flowing). FC1's "one connection per process, new replaces old" implies at
  least two (live view on at exit, live view off at exit). Restart rows should enumerate the source state at exit.

## Notes written to backlog
There are three new notes in `docs/implementation/backlog_live/LV1_camera_live_view.md`, with a pointer line in
`docs/implementation/backlog.md`:
- the dead rebind argument;
- the accept thread that never restarts after an error;
- the fake reconnecting after a failed write.

## Carry-forward
- R2-B1 goes to the orchestrator, to authorise fix 2 or raise an owner card.
- All r1 notes and their destinations stand unchanged.

## Write perimeter (this session)
- `docs/implementation/plans/LV1_camera_live_view.md`: the Tracker line and one Review log entry.
- `docs/implementation/handoffs/reviewer/handoff_LV1_camera_live_view_review_r2.md` (this file).
- `docs/implementation/backlog_live/LV1_camera_live_view.md`: three appended notes.
- `docs/implementation/backlog.md`: one appended pointer line.
- **Probe declaration.**
  - I temporarily appended two test functions to `crates/testkit/tests/suite/lv1_frame_channel.rs`, edited one of them
    once, and ran each 3–4 times.
  - I restored the file with `git checkout --`, and `shasum -c /tmp/lv1r2_probe.sha` reports OK.
  - No production file was edited.
- **Other side effects.**
  - The stub sidecar build refreshed ignored build outputs.
  - Test runs may leave `psm-lv-*.sock` files in `$TMPDIR` (the r1 note).
  - Logs are in `/tmp/lv1r2_*.log`.
- No commits; the runner commits these files.
- No narrative was written, because the phase is not approved.
