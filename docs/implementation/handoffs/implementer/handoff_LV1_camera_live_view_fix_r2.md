---
plan: LV1_camera_live_view
role: fix
round: 2
state: DONE
date: 2026-11-26
actor: codex
---
# Handoff — LV1 camera live view, fix r2

Implemented all three production findings and passed the targeted suite, sidecar tests, lint, formatting and desktop typecheck. The checkpoint commit is blocked because Git's index lock is under the linked worktree's parent `.git` directory, outside the writable workspace. No owner decision is needed; the required commit remains outstanding.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Finding → what changed → files

- **R1-B1:** Kept the frame listener bound across helper restarts. A newly accepted helper connection clears the bad-record latch and gets a new source identity; helper exit also clears the latch. The fake sidecar now connects as soon as it starts, so C13(c) exercises the same startup timing as the real helper. Updated the guide to describe that behavior. Files: `crates/integrations/src/camera/runtime.rs`, `crates/integrations/src/camera/sidecar/frames.rs`, `crates/testkit/src/bin/fake_sidecar.rs`, `crates/testkit/tests/suite/lv1_frame_channel.rs`, `docs/guide/05_working.md`.
- **R1-B2:** Split the frame-stream stop flag from the runner-wide stop flag. A frame-channel write failure now emits `liveview.unavailable{frame_channel}` and ends pull/sender while leaving the focus worker and focus command path active. File: `native/camera-sidecar/src/live_runner.cpp`.
- **R1-B3:** Replaced the production-derived expected value with the literal `ExposureReadout { shutter: "1/125", aperture: "f/8", iso: "ISO 100" }`. File: `crates/testkit/tests/suite/lv1_frame_channel.rs`.
- Updated the plan tracker to `IMPLEMENTED` (fix 1) and added this fix to its Review log. File: `docs/implementation/plans/LV1_camera_live_view.md`.

## Divergences
None. The persistent listener is the correction option named in R1-B1.

## Files changed
- `crates/integrations/src/camera/runtime.rs`
- `crates/integrations/src/camera/sidecar/frames.rs`
- `crates/testkit/src/bin/fake_sidecar.rs`
- `crates/testkit/tests/suite/lv1_frame_channel.rs`
- `docs/guide/05_working.md`
- `docs/implementation/plans/LV1_camera_live_view.md`
- `native/camera-sidecar/src/live_runner.cpp`
- `docs/implementation/handoffs/implementer/handoff_LV1_camera_live_view_fix_r2.md`

## Checkpoint SHA
`f7d03aa1` — made by the orchestrator (2026-11-26) with the exact command below; the session's sandbox could not write the
worktree's git index. State set to DONE by the orchestrator: the only blocker was the commit (runner fixed to add the
worktree's git common dir).

## Stamp
Checks were run sequentially against the completed implementation. The LV1 suite and camera integration module tests were each run once in the closeout; the sidecar stub tests were also run once in the closeout. The full workspace suite was not run because this fix changes no store, migration, or service shared by every phase.
The checks are green; state remains blocked only on the required checkpoint commit. The `IMPLEMENTED` tracker line is not backed by a committed checkpoint yet.

| Command | Result |
|---|---|
| `cargo test -p psm-testkit --test suite lv1_frame_channel::` | Passed, 24 tests |
| `cargo test -p psm-integrations camera::` | Passed, 23 tests |
| `tools/sidecar/build.sh --stub --test` | Passed, both CTest targets (31 transport tests, 17 live-view tests) |
| `cargo clippy -p psm-integrations -p psm-testkit --all-targets -- -D warnings` | Passed |
| `cargo fmt --all -- --check` | Passed after correcting the reported formatting differences |
| `npm run typecheck` (from `apps/desktop`) | Passed |
| `git diff --check` | Passed |

**Blocked checkpoint command:**

```text
git diff --check && git add crates/integrations/src/camera/runtime.rs crates/integrations/src/camera/sidecar/frames.rs crates/testkit/src/bin/fake_sidecar.rs crates/testkit/tests/suite/lv1_frame_channel.rs docs/guide/05_working.md docs/implementation/plans/LV1_camera_live_view.md native/camera-sidecar/src/live_runner.cpp && git commit -m "CHECKPOINT (not approved): LV1 camera live view fix 1" -- crates/integrations/src/camera/runtime.rs crates/integrations/src/camera/sidecar/frames.rs crates/testkit/src/bin/fake_sidecar.rs crates/testkit/tests/suite/lv1_frame_channel.rs docs/guide/05_working.md docs/implementation/plans/LV1_camera_live_view.md native/camera-sidecar/src/live_runner.cpp
fatal: Unable to create '/Users/davidloorenz/Desktop/Developer/Photo-Session-Manager/.git/worktrees/Photo-Session-Manager-live/index.lock': Operation not permitted
```

