---
plan: P51_camera_setup_ui
role: implement
round: 1
state: BLOCKED
date: 2026-11-08
actor: codex
---
# P51 camera setup UI — implement round 1

## What was built

Settings → Camera now lets the photographer add Sony support, generate and copy the one-time FTP password, follow the camera menu steps, edit receiver ports, save the camera login, compare a pending fingerprint, and check who holds the camera. The header dot shows five camera states and opens a card with the relevant action. Health names the camera issues, offers Dismiss and Take the camera back, and shows the redacted helper log in a disclosure. A thin Tauri command exposes the runtime's existing in-memory helper log; bindings were regenerated.

The phase is **blocked at the closing frontend suite gate**. The tracker remains NOT_STARTED; this checkpoint is not an IMPLEMENTED claim.

## Tests → criterion rows

| Rows | Boundary tests | Result |
|---|---|---|
| C1(a–f), C2(a–d) | `p51_camera_dot.test.tsx` | 13 cases pass; connected labels, all dot states, event update, card text/actions and outcomes |
| C3(a–c) | `p51_health_camera.test.tsx` | 3 cases pass; all camera labels/details, actions and helper disclosure |
| C4(a–c), C5(a–c), C6(a–c), C7(a), C8(a–b) | `p51_camera_section.test.tsx` | 15 cases pass; support, password, ports, login, fingerprint, connection check and conditional sections |
| C8(c) | Whole frontend suite | **Fails**: 394 pass, 6 fail, 2 unhandled errors in the final run |

C8(c)'s inherited P07 camera-dot assertions require the old title `Camera connected: <name>`, old hover help, and click-to-retry. P51 requires the exact title `Camera connected by cable|Wi-Fi` and click-to-open-card. The P19 mock omits `cameraSupportGet`, so mounting Settings raises two unhandled errors. Updating `p07_shell_sessions_ui.test.tsx` and `p19_navigation.test.tsx` would cross the plan's listed file perimeter; changing product behavior would contradict P51's exact contract. One P33 split-grid case failed in both full runs and passed when run alone (7/7); its cause was not established.

## Assumptions

- Camera helper log loads when the disclosure opens, with no polling.
- Copy changes to `Copied` for 1.5 seconds.
- The FTP instructions show `<this Mac's address on your network>` because no address command exists.
- A fingerprint status without a pending event shows a comparison prompt; the status DTO has no fingerprint value. The pending event shows the value.
- Camera menu steps follow the external `Camera-Connection-Tests/camlab/experiments/camera_steps.py`, adapted to `sessionmanager` and the app's password card.

## Deviations

- C8(c) and the suite-green closing gate are unmet. The required old-test changes lie outside the listed file perimeter. The master tracker was therefore not moved to IMPLEMENTED.
- No hardware capture was attempted; the automated suite remains hardware-free.
- No architecture graph exists in this repository; no graph delta was made.

## Files changed

Checkpoint: `docs/implementation/plans/P51_camera_setup_ui.md`; `apps/desktop/src-tauri/src/commands/camera.rs`, `src/lib.rs`; desktop `features/health/{HealthPage,HelperLog,__tests__/p51_health_camera.test}.tsx`; `features/settings/{SettingsPage,CameraSection,cameraSteps,__tests__/p51_camera_section.test}.tsx` (cameraSteps is `.ts`); `features/shell/{CameraDot,CameraCard,__tests__/p51_camera_dot.test}.tsx`; `services/{bindings,camera,events,settings}.ts`; `styles/globals.css`.

This handoff file is written after the checkpoint so it can cite the SHA; the runner owns its artifact commit. No probe or mutation files were changed.

## Checkpoint SHA

`67975d90cb53e731c1c9028f285f58de83f8decd` — `CHECKPOINT (not approved): build P51 camera setup UI and record suite blocker`.

## Stamp

| Command | Result |
|---|---|
| `cargo test -p psm-desktop export_bindings` | PASS; regenerated `bindings.ts` |
| `npx vitest run` on the three P51 test files | PASS; 31/31 after the final camera-step copy edit |
| `npm test -- --reporter=dot` | FAIL; 394/400 tests pass, 6 fail, 2 unhandled errors; P07/P19 conflicts and P33 intermittent case above |
| `npx vitest run src/features/workspace/__tests__/p33_split_grid.test.tsx --reporter=dot` | PASS; 7/7 in isolation |
| `npm run typecheck` | PASS |
| `cargo test --workspace` | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo fmt --all -- --check` | PASS |
| `git diff --check` | PASS |

## Manual try-out

1. Open Settings → Camera. Add a Sony SDK 2.02.00 ZIP archive; the installed version and transfer fields appear.
2. Generate an FTP password. Copy it and follow the camera menu steps. Close and reopen Settings: the password is gone.
3. Enter the camera's Access Authentication login and save it. The fields clear. Use Test camera connection to see who holds the camera.
4. Optionally with the camera connected, observe the header dot and open its card. If Capture One holds it, use Take the camera back. Check Health for camera items and the Camera helper log.

## ⚠ OWNER DECISIONS REQUIRED (0)

None. The orchestrator can resolve the plan perimeter and inherited test conflict before the next run.
