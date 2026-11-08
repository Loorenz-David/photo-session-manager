---
plan: P50_camera_settings_and_support
role: implement
round: 1
state: DONE
date: 2026-11-08
actor: codex
---

# What was built

The app can install the downloaded Sony SDK runtime from a ZIP, check its seven files against a compiled SHA-256 manifest, sign the copied runtime and camera helper, and use the direct camera adapter after installation. It reports a damaged installation in Health and returns to Capture One camera mode. A later app build refreshes its copied helper after a hash comparison.

Camera Access Authentication credentials and the generated FTP password go to the Keychain. The FTP password is returned once and replaces the receiver's old password on the same port. Keychain reads run off the startup path: the helper starts immediately for USB, then receives the credentials on the next safe spawn. Unknown or changed Wi-Fi fingerprints emit the pending event with the visible fingerprint; confirm arms the helper's next connection to pin it, and reject leaves it pending.

P47's spike corrections are included: the SDK helper has no absolute `LC_RPATH`, and an empty helper staging folder is removed on clean shutdown. The compiled SDK build and stub build both succeed; the SDK build was inspected with `otool` and has only `@executable_path` as `LC_RPATH`. No Sony binary is tracked.

# Tests → criterion rows

| Rows | Tests at boundary |
|---|---|
| C1(a), C1(b) | `p50_camera_settings_and_support::c1a_archive_install_has_signed_unquarantined_runtime_and_sidecar`, `c1b_install_starts_its_sidecar_and_receiver` |
| C2(a), C2(b) | `p50_camera_settings_and_support::c2a_missing_file_names_it_and_leaves_no_install`, `c2b_wrong_hash_names_the_file`; also `support::manifest::tests::c2a_missing_and_c2b_hash_mismatch` on the named mechanism |
| C2(c), C2(d), C2(e) | `p50_camera_settings_and_support::c2c_unsupported_sdk_version_is_refused`, `c2d_stub_sdk_version_is_refused`, `c2e_text_archive_is_refused_by_name` |
| C3(a) | `p50_camera_settings_and_support::c3a_update_refreshes_copy_once` |
| C4(a), C4(b) | `p50_camera_settings_and_support::c4a_damaged_support_falls_back_with_health`, `c4b_no_support_is_normal_capture_one_mode` |
| C5(a), C5(b), C5(c) | `p50_camera_settings_and_support::c5a_camera_credentials_are_keychain_only_and_redacted`, `c5b_wifi_credentials_change_restarts_with_new_first_line`, `c5c_usb_change_waits_for_next_spawn` |
| C6(a), C6(b), C6(c) | `mechanisms::secrets::tests::c6a_password_shape_and_rejection`, `p50_camera_settings_and_support::c6b_generated_password_reopens_listener_and_is_stored_only_in_keychain`, `c6c_regeneration_invalidates_old_password` |
| C7(a)–C7(e) | `p50_camera_settings_and_support::c7a_unknown_fingerprint_raises_event_status_and_health`, `c7b_confirmation_sends_accept_and_pin_clears_notice`, `c7c_rejection_keeps_pending_and_sends_nothing`, `c7d_mismatch_confirmation_replaces`, `c7e_auth_refusal_never_sends_a_retry` |
| C8(a), C8(b) | `p50_tests::c8a_support_and_secret_commands_have_their_wire_shapes`, `c8b_pending_fingerprint_reaches_tauri_event` |

The 20 new testkit boundary tests, 2 desktop command/event tests, and 2 unit tests cover all 23 rows. Existing regression tests also pass. The first full run exposed one incorrect expected string in the new seeded password test; the assertion was corrected, and the subsequent workspace run passed.

# Assumptions

- The bundled helper is beside the app executable in a packaged build; development resolves the existing `binaries/` path.
- ZIP entries are buffered for verification. Every archive name is checked for absolute and parent traversal before a manifest path is used.
- The bundled helper and installed copy have the same executable name and ad-hoc signing options, which allows their final SHA-256 values to match. The test signs both with P47's required entitlement.
- For a first Keychain read, USB starts with an empty credentials line. When the values arrive, the helper restarts only if no USB session is open; a USB session uses them on its next spawn.
- A helper staging directory is removed only when empty on clean shutdown. Files left by a transfer remain untouched.

# Deviations

- P47's Review-log requirements required `native/camera-sidecar/CMakeLists.txt` (remove the absolute SDK runtime path), `crates/core/src/ports/secrets.rs` (prompt-free cache seam), and `crates/testkit/src/fake_secrets.rs` outside the plan's expected file list.
- New command error variants required `apps/desktop/src-tauri/src/commands/items.rs`; new dependencies required the Cargo manifests and `Cargo.lock`.
- `CameraDot.tsx` did not need an edit: the newly generated `needs_action.link` member is optional and the component compiles unchanged.
- The fake sidecar's version behaviour is copied into the install's temporary folder only for its version probe and removed before installation. Production bundles do not contain that test file.
- No live camera or Capture One run was made in this phase; tests use the fake helper and loopback FTP.

# Files changed

`Cargo.lock`; `apps/desktop/src-tauri/Cargo.toml`, `src/commands/{camera.rs,items.rs}`, `src/lib.rs`; `apps/desktop/src/services/bindings.ts`; `crates/core/src/{error.rs,events.rs,mechanisms/{mod.rs,secrets.rs},ports/secrets.rs,services/camera.rs}`; `crates/integrations/Cargo.toml`, `src/camera/{direct/adapter.rs,runtime.rs,sidecar/{protocol.rs,supervisor.rs},support/{mod.rs,manifest.rs,sony.rs}}`; `crates/testkit/Cargo.toml`, `src/{bin/fake_sidecar.rs,fake_secrets.rs,fixtures/sidecar_scripts/fingerprint_unknown.txt,test_app.rs}`, `tests/p50_camera_settings_and_support.rs`; `native/camera-sidecar/{CMakeLists.txt,src/{main.cpp,camera_connection.cpp},manifest/sony-2.02.00.json}`; `docs/implementation/{master_plan.md,plans/P50_camera_settings_and_support.md,handoffs/implementer/handoff_P50_camera_settings_and_support_implement_r1.md}`.

# Checkpoint SHA

This handoff is in the checkpoint commit; resolve its SHA with `git log -1 --format=%H -- docs/implementation/handoffs/implementer/handoff_P50_camera_settings_and_support_implement_r1.md` after the commit. A commit cannot contain its own SHA in its contents.

# Stamp

| Command | Result |
|---|---|
| `cargo test --workspace` | Passed, all workspace crates including 20 P50 testkit tests, 2 desktop tests, and 2 mechanism unit tests. |
| `cargo clippy --workspace --all-targets -- -D warnings` | Passed. |
| `cargo fmt --all -- --check` | Passed. |
| `cd apps/desktop && npm run typecheck` | Passed. |
| `cd apps/desktop && npm test` | Passed, 50 files / 369 tests. |
| `tools/sidecar/build.sh --stub --test` | Passed, 21 C++ cases. |
| `tools/sidecar/build.sh --sdk <lab SDK>` and `otool -l target/camera-sidecar/sdk/camera-sidecar` | Build passed; sole `LC_RPATH` is `@executable_path`. The stub helper was restored to the development binaries path afterward. |
| `git diff --check` | Passed. |

# Manual try-out

1. Before P51 adds the Settings controls, use the Tauri dev tools to invoke `camera_support_install` with the downloaded Sony SDK ZIP path. The response names version `2.02.00`, and `~/Library/Application Support/com.loorenz.sessionmanager/camera/sony/2.02.00/` contains the helper and runtime libraries.
2. Invoke `camera_ftp_password_generate`. Copy the returned password now; `camera_support_get` and `settings_get` will not reveal it later. Enter it with username `sessionmanager` on the camera when ready.
3. Invoke `camera_secrets_set` with the camera's Access Authentication user and password. Inspect `settings.json` to see that neither value is present; Keychain Access shows the three `com.loorenz.sessionmanager.camera` items.
4. P51 will expose the fingerprint comparison and confirmation in Settings → Camera. This phase's fake-helper tests exercise both first pin and changed-key paths without a camera.

# ⚠ OWNER DECISIONS REQUIRED (0)

None.
