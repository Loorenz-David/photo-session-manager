---
plan: P48_camera_ownership
role: implement
round: 1
state: DONE
date: 2026-11-07
actor: claude
---
# Handoff — P48 camera ownership, implement round 1

## What was built
- **The camera says who holds it, and how to fix it.** When the camera's USB connection is taken by another program, the camera status and
  the Health page now carry one plain sentence: `Capture One is holding the camera's USB connection — Take the camera back` whenever
  Capture One runs; otherwise the known programs that could take the camera, with the first one's fix (Imaging Edge Desktop → quit it and
  stop it opening at login; Photos / Image Capture → "Connecting this camera opens: No application"; Lightroom → quit it); the system camera
  service alone → "close apps that use cameras"; an unknown holder → `<name> is using the camera: close it or stop it from using cameras`.
  Shooting over Wi-Fi continues meanwhile. The item clears as soon as our own USB session holds the camera again.
- **Start order.** With camera support installed, the app no longer launches Capture One at start until the USB session had its chance:
  USB ready, no camera on the cable, the Wi-Fi fallback, the camera helper unavailable or gone, or 15 s — whichever comes first. If Capture
  One is already running, nothing waits. This is decided once per app run.
- **One gate for every Capture One launch.** While "Take the camera back" runs, nothing in the app can start Capture One except the
  take-back's own relaunch step: the health check, Retry checks, a settings save, session activation — all of them stop quietly ("deferred")
  without writing false health items such as "document not open".
- **"Take the camera back" (backend).** On the photographer's click only: refuses with a reason while a file is arriving, a photo is still
  expected (< 10 s), an export or publish is running, a preview is rendering, or Capture One is not the holder; otherwise saves the Capture
  One document, asks Capture One to quit (never kills it; gives up after 20 s if a dialog stays open), retries the USB session (re-sending once a
  second while something claims it during the first 5 s; up to 15 s), then relaunches Capture One and reopens the document and the session's
  capture folder whatever the USB step gave. If the photographer opens Capture One during the quit/USB steps, it stops and says so.
- **Session switching never waits for Capture One** (direct camera only): activating a session stores it first (new photos go there), then
  Capture One follows; if it cannot, one health item says "Capture One will not show this session's new photos until it is reachable" and the
  next health check retries. With the Capture One camera adapter nothing changed.
- **No sleep while on the cable.** While a USB camera session is open, the Mac holds an idle-sleep assertion ("Session Manager: camera
  connected by cable"); released when the session ends, when the camera helper crashes, or when the app quits.
- **Commands:** `camera_take_back -> { completed_steps, camera_held }` (errors `take_back_blocked{reason}`, `take_back_failed{step, cause}`,
  step as a string) and `camera_test_connection -> { holder, programs, sentence }`. The buttons arrive with P51.

## Tests → criterion rows
Behaviour tests in `crates/testkit/tests/p48_camera_ownership.rs` unless noted.
| Row | Test |
|---|---|
| C1(a)–(d) | unit `camera::direct::ownership::tests::{c1a_capture_one_first, c1b_guides_for_other_programs, c1c_system_service_alone, c1d_unknown_owner}` |
| C1(e) | `c1e_status_and_health_name_the_holder_wifi_continues` |
| C1(f) | `c1f_cleared_when_we_hold_the_interface` |
| C2(a) | `c2a_launch_waits_for_usb_ready` |
| C2(b) | `c2b_no_camera_on_the_cable` |
| C2(c) | `c2c_fallback_to_wifi` |
| C2(d) | `c2d_sidecar_unavailable` |
| C2(e) | `c2e_the_15_s_bound_counts_from_the_first_wait` |
| C2(f) | `c2f_capture_one_already_running` |
| C2(g) | `c2g_satisfied_once` |
| C3(a)–(h) | `c3a_gate_defers_ensure_at_launch`, `c3b_front_document_deferred`, `c3c_make_document_deferred`, `c3d_open_document_deferred`, `c3e_ensure_recipe_deferred`, `c3f_ensure_capture_style_deferred_on_the_user_lane`, `c3g_a_health_check_while_held_never_launches`, `c3h_released` |
| C4(a)–(f) | boundary `c4a_blocked_while_a_file_arrives` … `c4f_blocked_when_capture_one_is_not_the_holder`; unit halves `camera::direct::take_back::tests::{c4_all_clear, c4a_…, c4b_…, c4c_…, c4d_…, c4e_…, c4f_…, c4_check_order}` |
| C5(a)–(f) | `c5a_the_sequence_succeeds`, `c5b_save_fails_nothing_else_happens`, `c5c_quit_never_finishes_never_killed`, `c5d_step_4_times_out_step_5_still_runs`, `c5e_the_photographer_started_capture_one_meanwhile`, `c5f_capture_one_is_quit_only_on_click` |
| C5(g) | desktop `p48_tests::c5g_camera_take_back_command` (mock webview, ticker thread per fixture rule iv) |
| C6(a), C6(b) | `c6a_activation_stores_first_capture_one_follows_or_not`, `c6b_retried_at_the_next_ensure` |
| C7(a) | `c7a_sleep_assertion_follows_the_usb_session` |
| C8(a) | desktop `p48_tests::c8a_camera_test_connection_command` |

## Assumptions
1. **No `EditingProvider::quit`.** The Files list mentions `ports/editing.rs (quit, renders_in_flight)`; the contract has the take-back call
   `scripting.quit()` on the ungated `App.scripting` (a `CaptureOneScripting`), so only `CaptureOneScripting::quit` was added. `ports/editing.rs`
   gained `renders_in_flight()` (default `0`; the Capture One adapter counts issued + queued renders), the recommended delegated choice.
2. **Where `AlreadyRunning` is set.** In `GatedScripting::editor_info` (any gated `editor_info` that reports running satisfies the start order).
   Only `ensure_provision` calls `editor_info` on the gated lanes, so this is the contract's "first ensure whose `editor_info` reports running".
3. **Start-order conditions also include a failed sidecar spawn** (R-I31 "the sidecar being unavailable") besides `sdk.unavailable` and an exit.
4. **`Lanes` struct.** The user lane, background lane, gate and start order travel together as `psm_integrations::Lanes` (replacing the lanes
   tuple); `App::{user_lane, launch_gate, start_order}` expose them. `AppBinding` gained `editing`, `scripting` (ungated), `provision`,
   `document_id`, `gate`. The runtime's `new_with_receiver_bind` takes the start order.
5. **Sleep release on a Wi-Fi `connection.ready`** too (only one SDK session exists at a time, so a Wi-Fi session means no USB session).
6. **Step 4 polling.** Our USB session is detected from the adapter's state (`connection.ready` on USB); `editor_info` is re-checked every 500 ms
   of clock time; an `editor_info` error counts as "unknown" (not as running, not as stopped) in steps 3 and 4. A failed `usb-retry` send is
   logged and the wait continues (step 5 runs whatever step 4 gives).
7. **`holder_is_capture_one`** = a `HeldByOtherProgram` entry is present whose holder is `Capture One` (even if a higher-ranked reason is shown);
   `holder_report` names the holder only while the derived status is `HeldByOtherProgram` (L19 literal).
8. **`completed_steps` is 5** on success (step 6 is the release), as C5(a) states.
9. **Activation with the direct adapter clears `editor_not_following` when Capture One follows** (in addition to `EditingStartup::run`'s clear).
10. **`PsProcessList`** keeps the first occurrence of each name, in listing order. **CoreFoundation binding:** an `extern` block linking
    CoreFoundation and IOKit in `sleep.rs` (no new dependency; `infra/Cargo.toml` unchanged).
11. **Fixture rule (i) also resets the fake's `launched` flag** in the test helper (`editor_stopped`): without it, after C2(a)'s launch the fake's
    `running_after_polls` counter turns Capture One "running" on the very next `editor_info`, so C2(g)'s health check would find it running
    and never launch. The fake's `quit` also clears `launched`.

## Deviations
- `crates/core/src/services/camera.rs` was listed as changed; no change was needed (its gating already branches on
  `depends_on_editing_provider`). `crates/infra/Cargo.toml` unchanged (extern-block option taken).
- The C2 rows observe the start order through `App::start_order()` (`is_waiting()`, `state()`), a public accessor added for the purpose, besides
  the ops/launch assertions the rows name.
- Tracker actor is `claude` (this session ran under `PSM_IMPLEMENTER=claude`), not `codex` as the prompt template says.

## Files changed
Integrations (new): `crates/integrations/src/editing/capture_one/launch_gate.rs`, `crates/integrations/src/editing/capture_one/jxa/quit.js`,
`crates/integrations/src/camera/direct/ownership.rs`, `crates/integrations/src/camera/direct/take_back.rs`,
`crates/integrations/src/camera/direct/start_order.rs`.
Integrations (changed): `crates/integrations/src/editing/capture_one/{gate.rs, scripting.rs, jxa.rs, adapter.rs, provision.rs, mod.rs}`,
`crates/integrations/src/camera/{runtime.rs, direct/adapter.rs, direct/mod.rs}`, `crates/integrations/src/lib.rs`.
Core (new): `crates/core/src/ports/system.rs`. Core (changed): `crates/core/src/ports/{editing.rs, mod.rs}`, `crates/core/src/domain/{editing.rs,
camera.rs}`, `crates/core/src/mechanisms/camera_status.rs`, `crates/core/src/services/{sessions.rs, editing_startup.rs}`, `crates/core/src/error.rs`.
Infra (new): `crates/infra/src/{processes.rs, sleep.rs}`. Infra (changed): `crates/infra/src/lib.rs`.
Desktop (changed): `apps/desktop/src-tauri/src/commands/{camera.rs, sessions.rs, items.rs}`, `apps/desktop/src-tauri/src/lib.rs`,
`apps/desktop/src/services/bindings.ts` (regenerated).
Testkit (new): `crates/testkit/src/{fake_processes.rs, fake_sleep.rs}`, `crates/testkit/src/fixtures/sidecar_scripts/{claimed_wifi.txt, take_back.txt}`.
Testkit (changed): `crates/testkit/src/{lib.rs, test_app.rs, fake_capture_one.rs}`.
Tests: `crates/testkit/tests/p46_direct_camera_adapter.rs` (the two declared sites), `crates/testkit/tests/p48_camera_ownership.rs` (new).
Docs: `docs/implementation/master_plan.md` (P48 tracker row only), `docs/implementation/plans/P48_camera_ownership.md` (Review log entry).
Not changed though listed: `crates/core/src/services/camera.rs`, `crates/infra/Cargo.toml`. No archgraph or other tool-recorded state.
This handoff is committed by the runner.

## Checkpoint SHA
4da7fe9 (committed by the orchestrator, 2026-11-07: the session lost its network after writing this handoff; tree verified green first)

## Stamp
2026-11-07 orchestrator

## Manual try-out
Hardware-free (the real take-back is P52's row).
1. With Capture One already open, start the app (`cd apps/desktop && npm run tauri -- dev`): no change — nothing waits, Capture One is not
   relaunched.
2. Open Health: the camera row behaves as before; "Retry checks" still works.
3. Open a session and switch to another: Capture One follows as before.
4. Nothing else is visible yet; the "Take the camera back" button and "Test camera connection" arrive with P51.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
