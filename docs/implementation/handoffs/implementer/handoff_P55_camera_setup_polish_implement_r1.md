---
plan: P55_camera_setup_polish
role: implement
round: 1
state: DONE
date: 2026-11-08
actor: codex
---
# P55 implementation handoff

## What was built
The photographer can add Sony camera support from the original downloaded archive. The installer finds the seven runtime files inside its `RemoteCli.zip`, verifies their hashes, and copies only those files into camera support. Settings now distinguishes an absent file from one whose bytes changed: **“The archive is missing CrAdapter/libCr_PTP_IP.dylib”** and **“The archive has changed CrAdapter/libCr_PTP_IP.dylib”** are the exact example sentences. The camera setup steps include `MENU › Network › [FTP Transfer] › [FTP Transfer Func.] › [Auto FTP Trnsfr] = [On]`. The camera popover has **Take photo**, enabled while connected; capture failures appear in a toast. After the helper's first unexpected exit, a replacement starts immediately. A second consecutive exit waits one second, then the existing backoff continues.

## Tests → criterion rows
| Row | Boundary proof |
|---|---|
| C1(a) | `p55_camera_setup_polish::c1a_sony_download_installs_seven_verified_files` installs a zip containing only `RemoteCli.zip` and reads all seven installed files against the made-up manifest bytes. |
| C1(b) | `p55_camera_setup_polish::c1b_missing_and_changed_are_distinct` expects `missing CrAdapter/libCr_PTP_IP.dylib` for an absent entry and `changed CrAdapter/libCr_PTP_IP.dylib` for wrong bytes. The P51 Settings install-error case asserts both toast sentences. |
| C2(a) | `p55_camera_setup_steps` renders Settings → Camera after generating the FTP password and asserts the Auto FTP Trnsfr menu path and On value. |
| C3(a) | `p55_camera_setup_polish.test.tsx` asserts Take photo is disabled when disconnected and calls `cameraCapture` once when connected. Existing `p46_direct_camera_adapter::c2a_capture_returns_id_and_records_exposure` asserts that a capture command reaches the fake helper exactly once. |
| C3(b) | `p55_camera_setup_polish.test.tsx` rejects the capture command and asserts the exact error sentence reaches `toast.error`. |
| C4(a) | `p55_camera_setup_polish::c4a_first_unexpected_exit_restarts_within_300ms_then_backs_off` sees a new fake helper within 300 ms and verifies no third spawn before the second exit's one-second delay. P42's existing offset cases were updated to the new schedule. |
| C5(a) | Full closing stamp below: Rust workspace, frontend, clippy, format, typecheck, and stub helper tests all green. |

## Assumptions
- One nested zip level is sufficient for Sony's download. The compressed inner zip is copied to a temporary seekable file once; unrelated members are never extracted.
- The Auto FTP line belongs with the existing FTP setup steps, after enabling FTP Function.
- Take photo appears first among the camera popover actions and stays disabled in every status other than Connected.
- An unexpected exit is a child exit while the supervisor is running. Explicit app quit stops the supervisor; the existing 60-second healthy-run reset remains.

## Deviations
- `crates/testkit/tests/p42_camera_runtime.rs` and `apps/desktop/src-tauri/src/lib.rs` were outside the plan's expected list but needed changes because their prior assertions encoded the superseded restart timing and installer error text.
- The Test-0 red baseline was not captured before the first production edit. No baseline is reconstructed here.
- Hardware acceptance remains P52 run 2; no live camera or Capture One action was used in this session.

## Files changed
- `crates/integrations/src/camera/support/{manifest,sony}.rs`
- `crates/integrations/src/camera/sidecar/supervisor.rs`
- `crates/testkit/tests/{p42_camera_runtime,p50_camera_settings_and_support,p55_camera_setup_polish}.rs`
- `apps/desktop/src-tauri/src/lib.rs`
- `apps/desktop/src/features/settings/CameraSection.tsx`, `apps/desktop/src/features/settings/cameraSteps.ts`
- `apps/desktop/src/features/settings/__tests__/{p51_camera_section,p55_camera_setup_steps}.test.tsx`
- `apps/desktop/src/features/shell/CameraCard.tsx`
- `apps/desktop/src/features/shell/__tests__/p55_camera_setup_polish.test.tsx`
- `docs/implementation/master_plan.md`
- `docs/implementation/plans/P55_camera_setup_polish.md`
- this handoff

## Checkpoint SHA
The commit containing this handoff is the checkpoint; resolve its SHA with `git log -1 --format=%H -- docs/implementation/handoffs/implementer/handoff_P55_camera_setup_polish_implement_r1.md`. A literal SHA in this committed file would change the commit's SHA.

## Stamp
| Command | Result |
|---|---|
| `cargo test --workspace` | green, 836 passed, 0 failed across 54 groups |
| `npm test` (`apps/desktop`) | green, 411 passed, 0 failed |
| `cargo clippy --workspace --all-targets -- -D warnings` | green |
| `cargo fmt --all -- --check` | green |
| `npm run typecheck` (`apps/desktop`) | green |
| `tools/sidecar/build.sh --stub --test` | green, 22 C++ cases, 0 failed |
| `git diff --check` | green |

## Manual try-out
1. In Settings → Camera, click Add camera support → Sony and choose the original `CrSDK_v2.02.00_…_Mac.zip`. Confirm the installed version appears. To inspect an error, choose a test archive missing a listed library; the message should say **missing**, while altered library bytes should say **changed**.
2. Generate the FTP password and read the setup list. On the camera, follow the `Auto FTP Trnsfr` step and set it to On.
3. With the camera connected over Wi-Fi, open the header camera dot and click **Take photo**. Confirm one RAW arrives in the active session. Repeat with the cable. The button should be disabled while disconnected.
4. In the owner-attended P52 run 2, repeat the helper-restart check (A12) and verify the replacement appears immediately, then the camera reconnects.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
