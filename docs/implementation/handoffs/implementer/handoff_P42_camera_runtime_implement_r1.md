---
plan: P42_camera_runtime
role: implement
round: 1
state: BLOCKED
date: 2026-11-06
actor: codex
---

# P42 camera runtime implement r1

## What was built

The camera indicator still reports the Capture One tether and can now render connected, connecting, busy, needs-action and disconnected states. The camera service can work independently of Capture One for the future direct adapter. A process-wide camera runtime owns one helper across settings saves, shuts it down on app exit, restarts crashed helpers on the contracted schedule, and keeps camera credentials in Keychain or a test secret store. Hardware-free fake-helper tests cover that lifecycle.

## Tests → criterion rows

| Rows | Boundary tests | Result |
|---|---|---|
| C1(a), C1(b), C1(c) | `p42_camera_runtime::{c1a_runtime_save_keeps_child,c1b_theme_save_keeps_child,c1c_restart_reaps_old_child}` | pass |
| C2(a), C2(b) | `p42_camera_runtime::{c2a_shutdown_is_graceful_and_terminal,c2b_shutdown_kills_ignoring_child_after_grace}` | pass |
| C3(a) | `p42_camera_runtime::c3a_restarts_at_bounded_offsets` | pass: 0, 1, 3, 7, 15, 31, 61 s |
| C3(b) | `p42_camera_runtime::c3b_health_clears_on_line_without_resetting_backoff` | pass against MC-31 and C3(a)'s 8 s after fourth exit; the row's 16 s is contradictory |
| C4(a), C4(b) | `p42_camera_runtime::{c4a_credentials_are_only_on_first_stdin_line,c4b_stderr_is_redacted}` | pass |
| C5(a)–(g) | `camera_status::tests::mc31_c5a_* ` through `mc31_c5g_*` | seven pass |
| C5(h)–(l) | `camera_status::tests::{mc31_c5h_needs_action_over_busy,mc31_c5i_busy_over_connected,mc31_c5j_connected_over_connecting,mc31_c5k_connecting_requires_report,mc31_health_list}` | five pass |
| C6(a) | `p42_camera_runtime::c6a_direct_provider_ignores_editor_health` | pass |
| C6(b) | Existing `p04_capture_one_adapter::{c1b_failed_set_restores_and_blocks_camera,c1c_readback_mismatch_restores_and_blocks_camera}` | pass |
| C7(a), C7(b) | `p42_camera_runtime::{c7a_camera_health_copied_on_rebuild,c7b_runtime_identity_survives_rebuild_and_reopen}` | pass |
| C8(a) | `psm_desktop_lib::p42_tests::c8a_camera_commands_emit_retry_through_hub` | pass |

## Assumptions

- The newest installed support folder is selected by lexical version-folder order, as delegated.
- P42 stages helper files under `camera/staging/sdk-<run_id>`; P46 will choose the capture-root location.
- The helper log is a 50-line in-memory ring.
- A generic holder message is used until P48 supplies the holder-specific guidance.
- The fake sidecar executable is found through Cargo's test environment or the workspace target directory, then copied before App construction. The testkit library cannot use the integration test's compile-time `CARGO_BIN_EXE_fake_sidecar` value.

## Deviations

- **Blocking plan contradiction:** MC-31 and C3(a) define consecutive delays of 1, 2, 4, 8, 16 seconds. C3(b) requires 16 seconds after the fourth exit, where that same schedule requires 8. The implementation and test use 8. The plan should correct C3(b) to 8 seconds; no owner choice is needed because the ratified contract fixes the schedule.
- `Cargo.lock` changed for the declared dependency additions. It was not listed in the phase perimeter.
- The tracker row remains `NOT_STARTED` because the acceptance row cannot be satisfied as written; it was not falsely marked `IMPLEMENTED`.

## Files changed

Checkpoint commit, 47 paths:

- `Cargo.lock`
- `apps/desktop/src-tauri/src/commands/camera.rs`
- `apps/desktop/src-tauri/src/lib.rs`
- `apps/desktop/src/features/compose/__tests__/harness.tsx`
- `apps/desktop/src/features/sessions/__tests__/p07_shell_sessions_ui.test.tsx`
- `apps/desktop/src/features/shell/CameraDot.tsx`
- `apps/desktop/src/features/shell/__tests__/p19_navigation.test.tsx`
- `apps/desktop/src/features/workspace/__tests__/p08_workspace_ui.test.tsx`
- `apps/desktop/src/features/workspace/__tests__/p19_drop.test.tsx`
- `apps/desktop/src/features/workspace/__tests__/p20_drop_continuity.test.tsx`
- `apps/desktop/src/features/workspace/__tests__/p22_workspace_tidy.test.tsx`
- `apps/desktop/src/features/workspace/__tests__/p26_double_drag.test.tsx`
- `apps/desktop/src/services/bindings.ts`
- `crates/core/src/domain/camera.rs`
- `crates/core/src/domain/editing.rs`
- `crates/core/src/domain/mod.rs`
- `crates/core/src/events.rs`
- `crates/core/src/mechanisms/camera_status.rs`
- `crates/core/src/mechanisms/mod.rs`
- `crates/core/src/ports/camera.rs`
- `crates/core/src/ports/mod.rs`
- `crates/core/src/ports/secrets.rs`
- `crates/core/src/services/camera.rs`
- `crates/infra/Cargo.toml`
- `crates/infra/src/keychain.rs`
- `crates/infra/src/lib.rs`
- `crates/infra/src/logging.rs`
- `crates/infra/src/paths.rs`
- `crates/integrations/Cargo.toml`
- `crates/integrations/src/camera/mod.rs`
- `crates/integrations/src/camera/runtime.rs`
- `crates/integrations/src/camera/sidecar/mod.rs`
- `crates/integrations/src/camera/sidecar/supervisor.rs`
- `crates/integrations/src/camera/support/mod.rs`
- `crates/integrations/src/camera/via_capture_one.rs`
- `crates/integrations/src/lib.rs`
- `crates/testkit/Cargo.toml`
- `crates/testkit/src/bin/fake_sidecar.rs`
- `crates/testkit/src/fake_direct_camera.rs`
- `crates/testkit/src/fake_secrets.rs`
- `crates/testkit/src/lib.rs`
- `crates/testkit/src/test_app.rs`
- `crates/testkit/tests/p04_capture_one_adapter.rs`
- `crates/testkit/tests/p24_capture_one_provisioning.rs`
- `crates/testkit/tests/p25_tidy_up_after_provisioning.rs`
- `crates/testkit/tests/p42_camera_runtime.rs`
- `docs/implementation/plans/P42_camera_runtime.md`

This handoff is written after the checkpoint for the runner to commit.

## Checkpoint SHA

`cd76ef3a8fc22b2cfd86b4fbc8b2226d079e4814` — `CHECKPOINT (not approved): P42 camera runtime pending backoff criterion correction`.

## Stamp

- `cargo test --workspace` — exit 0; all Rust unit, integration and doc tests pass, including 12 P42 runtime tests, 12 P42 status mechanism tests and the desktop command test.
- `cargo clippy --workspace --all-targets -- -D warnings` — exit 0.
- `cargo fmt --all -- --check` — exit 0.
- `cd apps/desktop && npm run typecheck` — exit 0.
- `cd apps/desktop && npm test` — exit 0, 50 files / 369 tests.
- `git diff --cached --check` — exit 0 before commit.
- No live camera or Capture One probe was run; the direct adapter is not present in P42.

## Manual try-out

1. Start Session Manager with Capture One open and the ZV-E10 tethered as before. The header camera dot should be green and show the camera name.
2. Unplug the camera and click the dot. It should turn red and say “Camera disconnected.”
3. Save a different SKU prefix in Settings. The app should keep running, and Health should retain the camera row.

## ⚠ OWNER DECISIONS REQUIRED (0)

None.
