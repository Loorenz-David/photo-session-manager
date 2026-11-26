---
plan: LV1_camera_live_view
role: fix
round: 3
state: BLOCKED
date: 2026-11-26
actor: codex
---
# Handoff — LV1 camera live view, fix 2

R2-B1 is corrected: a helper restart while live view is off no longer leaves the reader parked on the old frame connection. The focused LV1 tests and all non-workspace closing checks passed. The workspace gate failed in an unrelated P50 support-install test, so the phase tracker remains `CHANGES_REQUESTED` pending resolution of that gate.

## ⚠ OWNER DECISIONS REQUIRED (0)
None. The remaining gate failure is in P50 support installation and is outside this correction's perimeter; no owner decision was made in this fix cycle.

## Finding → what changed → files

### R2-B1 (production): restarting while live view is off left the reader on a dead/parked connection
- `FrameListener::read_stream` now checks the nonblocking listener during its no-active-source wait. A newer sidecar connection replaces the parked stream, increments the connection identity and clears the malformed-record latch. If activation races with replacement, the hub is started with the new source identity.
- C13(c) now starts with live view off, restarts the helper, then activates live view and verifies frames from the restarted helper are offered with increasing sequence numbers.
- Files: `crates/integrations/src/camera/sidecar/frames.rs`, `crates/testkit/tests/suite/lv1_frame_channel.rs`, and the C13(c) criterion and fix log in `docs/implementation/plans/LV1_camera_live_view.md`.

## Divergences
None. The nonblocking `accept` option is one of the correction clause's stated alternatives. An initial attempt to use `UnixStream::peek` did not compile because Unix socket peek is unstable on this toolchain; it was removed, and the accepted-connection replacement handles the reproduced restart path.

## Files changed
- `crates/integrations/src/camera/sidecar/frames.rs`
- `crates/testkit/tests/suite/lv1_frame_channel.rs`
- `docs/implementation/plans/LV1_camera_live_view.md`
- `docs/implementation/handoffs/implementer/handoff_LV1_camera_live_view_fix_r3.md`

## Checkpoint SHA
The single checkpoint commit includes this handoff, so its SHA is reported in the final session message.

## Stamp
- `cargo test -p psm-testkit --test suite lv1_frame_channel::` — 24 passed.
- `cargo test -p psm-integrations camera::` — 23 passed.
- `cargo test --workspace` — failed: 123 passed, 1 failed in `psm-desktop`; `p50_tests::c8a_support_and_secret_commands_have_their_wire_shapes` returned `support_version_unsupported: unavailable` during support installation. This test is outside the LV1 fix perimeter.
- Retried only the failing test with `cargo test -p psm-desktop --lib p50_tests::c8a_support_and_secret_commands_have_their_wire_shapes`; Cargo reported `Blocking waiting for file lock on build directory`. The waiting retry was stopped without changing files. The retry outcome is unknown.
- `cargo clippy --workspace --all-targets -- -D warnings` — passed.
- `cargo fmt --all -- --check` — passed.
- `tools/sidecar/build.sh --stub --test` — passed, 2 CTest targets (31 transport and 17 live-view tests).
- `cd apps/desktop && npm run typecheck` — passed.
- `cd apps/desktop && npm test` — passed, 138 files / 948 tests.
- `git diff --check` — pending checkpoint close.
