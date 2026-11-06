---
plan: P43_ftp_receiver
role: implement
round: 1
state: DONE
date: 2026-11-06
actor: codex
---

# P43 FTP receiver implement r1

## What was built

Session Manager can now receive the camera's wireless FTP uploads into hidden staging files. It accepts completed files for the later capture ledger, keeps interrupted files for resume, refuses deletes and renames, and answers the camera's virtual folder and listing commands. The receiver uses a Keychain-backed password and private-network peer policy. Camera port settings are validated and a changed port reopens the listener without restarting the helper. The camera setup screen and ledger are later phases; without installed camera support and an FTP password the app does not listen.

## ⚠ OWNER DECISIONS REQUIRED (0)

None.

## Tests → criterion rows

| Row | Test at named boundary | Result |
|---|---|---|
| C1(a) | `p43_ftp_receiver::c1a_completed_upload_integrity` | pass |
| C1(b) | `p43_ftp_receiver::c1b_virtual_subdirectory` | pass |
| C1(c) | `p43_ftp_receiver::c1c_two_concurrent_uploads` | pass |
| C2(a) | `p43_ftp_receiver::c2a_reset_is_failure` | pass |
| C2(b) | `p43_ftp_receiver::c2b_rest_resume` | pass |
| C2(c) | `p43_ftp_receiver::c2c_rest_past_length` | pass |
| C2(d) | `p43_ftp_receiver::c2d_new_stor_truncates` | pass |
| C2(e) | `p43_ftp_receiver::c2e_same_name_in_flight` | pass |
| C2(f) | `p43_ftp_receiver::c2f_control_close_aborts` | pass |
| C2(g) | `p43_ftp_receiver::c2g_appe_at_staged_length` | pass |
| C3(a) | `p43_ftp_receiver::c3a_delete_and_rename_refused` | pass |
| C3(b) | `p43_ftp_receiver::c3b_rnto_alone` | pass |
| C3(c) | `p43_ftp_receiver::c3c_hidden_names_refused` | pass |
| C3(d) | `p43_ftp_receiver::c3d_traversal_refused` + `ftp::names::tests::names_contain_staging_and_traversal` | pass |
| C4(a) | `p43_ftp_receiver::c4a_virtual_listings` | pass |
| C4(b) | `p43_ftp_receiver::c4b_size_and_mdtm` | pass |
| C4(c) | `p43_ftp_receiver::c4c_old_names_expire` | pass |
| C5(a) | `ftp::peers::tests::allowed_networks` | pass |
| C5(b) | `p43_ftp_receiver::c5b_public_policy_refuses_loopback` | pass |
| C5(c) | `p43_ftp_receiver::c5c_auth_and_secret_redaction` | pass; includes tracing output |
| C5(d) | `p43_ftp_receiver::c5d_passive_only_and_range` | pass |
| C6(a) | `p43_ftp_receiver::c6a_missing_capture_root` | pass |
| C6(b) | `p43_ftp_receiver::c6b_capture_root_from_settings` | pass |
| C7(a) | `p43_ftp_receiver::c7a_camera_settings_validation` | pass |
| C7(b) | `p43_ftp_receiver::c7b_port_reopens_in_place` | pass |
| C7(c) | `p43_ftp_receiver::c7c_unrelated_save_keeps_connection` | pass |
| C8(a) | `p43_ftp_receiver::c8a_shutdown_keeps_staging` | pass |

## Assumptions

- SHA-256 runs on Tokio's blocking pool after socket close. The completion timestamp is taken before hashing.
- `LIST` uses UTC month, day and time; `FEAT` advertises EPSV, REST and UTF8.
- The data idle timeout is 600 seconds, and no explicit control-connection cap is set.
- The interim `NoopSink` logs at debug. `receiver_generation` is `u64`.
- Tests bind loopback with ephemeral control and passive ports. Production binds `0.0.0.0` and filters peers to the contracted private ranges. The test client finds the changed control port by temporarily binding `127.0.0.1:0`.
- `CompletedNames` is an infra-side source separate from core `CaptureSink`, so the ledger in P44 can provide persistent listing names without making FTP listing a core camera port.

## Deviations

- `Cargo.lock` changed for the declared `ipnet` dependency; it was not listed in Files expected to change.
- Eight existing frontend test fixtures were updated with the additive `camera` settings block. These paths were outside the listed phase perimeter but required by the regenerated `SettingsDto` to keep typecheck green.
- `ftp/names.rs` is named by Ordered task 1 and Core mechanisms but omitted from the Files expected to change list.
- No live-camera or Capture One probe was run. The phase's contract is hardware-free; socket tests use loopback.

## Files changed

Checkpoint commit: 33 paths. The handoff file is an additional uncommitted runner-owned artifact.

- `Cargo.lock`
- `apps/desktop/src-tauri/src/commands/settings.rs`
- `apps/desktop/src/features/pending/__tests__/p18_pending_state.test.tsx`
- `apps/desktop/src/features/settings/__tests__/p16_settings.test.tsx`
- `apps/desktop/src/features/settings/__tests__/p24_capture_one_provisioning.test.tsx`
- `apps/desktop/src/features/settings/__tests__/p26_frontend_leftovers.test.tsx`
- `apps/desktop/src/features/settings/__tests__/p27_default_shape.test.tsx`
- `apps/desktop/src/features/settings/__tests__/p29_output_format.test.tsx`
- `apps/desktop/src/features/shell/__tests__/p19_navigation.test.tsx`
- `apps/desktop/src/features/shell/__tests__/p22_shell_tidy.test.tsx`
- `apps/desktop/src/services/bindings.ts`
- `crates/core/src/domain/camera.rs`
- `crates/core/src/ports/camera.rs`
- `crates/core/src/services/settings.rs`
- `crates/infra/Cargo.toml`
- `crates/infra/src/ftp/mod.rs`
- `crates/infra/src/ftp/names.rs`
- `crates/infra/src/ftp/peers.rs`
- `crates/infra/src/ftp/server.rs`
- `crates/infra/src/ftp/session.rs`
- `crates/infra/src/lib.rs`
- `crates/infra/src/settings_file.rs`
- `crates/integrations/src/camera/mod.rs`
- `crates/integrations/src/camera/runtime.rs`
- `crates/integrations/src/lib.rs`
- `crates/testkit/Cargo.toml`
- `crates/testkit/src/collecting_sink.rs`
- `crates/testkit/src/ftp_client.rs`
- `crates/testkit/src/lib.rs`
- `crates/testkit/src/test_app.rs`
- `crates/testkit/tests/p43_ftp_receiver.rs`
- `docs/implementation/master_plan.md`
- `docs/implementation/plans/P43_ftp_receiver.md`
- `docs/implementation/handoffs/implementer/handoff_P43_ftp_receiver_implement_r1.md` (runner-owned handoff)

## Checkpoint SHA

`e3569bed1c7daa6e486210c13f335a1eb159420e` — `CHECKPOINT (not approved): P43 FTP receiver`. `git status --porcelain` was empty immediately after the commit, before writing this handoff.

## Stamp

- `cargo test --workspace` — exit 0 on final code; 26 P43 socket tests and 2 named mechanism unit tests pass, as do earlier Rust tests and doc tests.
- `cargo clippy --workspace --all-targets -- -D warnings` — exit 0.
- `cargo fmt --all -- --check` — exit 0.
- `cd apps/desktop && npm run typecheck` — exit 0.
- `cd apps/desktop && npm test` — final exit 0, 50 files and 369 tests. An unchanged P33 split-grid assertion failed once in an intermediate full run; it passed alone and in the final full run.
- `git diff --cached --check` — exit 0 before the commit.

## Manual try-out

1. Start the app with Capture One closed. Camera support is not installed yet, so `ftp 127.0.0.1 2121` in Terminal should not connect; loopback is also refused by the production receiver.
2. Save a non-camera setting, such as the SKU prefix. The app should stay responsive.
3. Open Health. There should be no new FTP row; the Camera settings screen arrives in P51.
