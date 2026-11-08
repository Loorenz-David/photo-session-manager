---
plan: P54_camera_cable_and_take_back
role: implement
round: 1
state: DONE
date: 2026-11-08
actor: codex
---
# P54 implementation handoff

## What was built
"Take the camera back" can run while a preview is deferred, while a render Capture One is actually processing still blocks it. After Capture One quits with the cable attached, the app gives its camera helper up to 15 seconds to acquire USB before relaunching Capture One. The helper reaches Wi-Fi after three quick failed USB attempts. The camera dot shows `Connecting · Wi-Fi` during Wi-Fi discovery and retry; Retry sends a new attempt to the helper. The take-back buttons in the camera popover and Health screen show the first blocking reason and stay disabled until it clears.

The observed F3 cause was twofold: the adapter cleared `connecting` on each retryable `DISCONNECTED` event, often in the same batch as discovery, and `camera_retry` only re-emitted the existing status without contacting the helper. The helper now keeps the connecting link through retryable gaps, and the command sends `usb-retry`.

## Tests → criterion rows
| Row | Boundary test and exact assertion |
|---|---|
| C1(a) | `c1a_deferred_variant_does_not_block_take_back_and_retries_when_it_appears`: a missing variant defers the preview, take-back completes through quit and relaunch, and the preview starts after the variant appears. |
| C1(b) | `c1b_issued_render_still_blocks_take_back`: an issued preview yields `a preview is rendering`. |
| C2(a) | `c2a_relaunch_waits_for_our_usb_session_and_is_bounded`: no launch before USB ready; release on USB ready or at the 15 s bound. |
| C2(b) | `c2b_relaunches_immediately_without_cable_or_when_usb_is_ours`: both cases launch without advancing the clock. |
| C3(a) | C++ `P54 C3(a) busy USB claims fall back to Wi-Fi then return at a safe point`: exactly three failed USB connects, one fallback, `CONNECTED_WIFI` within 10 s, and a later safe USB switch; also tests an empty USB enumeration and a long general backoff. |
| C4(a) | `c4a_retryable_wifi_discovery_remains_connecting_until_ready`: discovery, connect, retryable disconnect stays `Connecting(Wireless)` until ready. `p54_camera_cable.test.tsx` asserts the dot label `Connecting · Wi-Fi`. |
| C4(b) | Desktop `p54_tests::c4b_retry_command_sends_reconnect_and_returns_connecting`: `camera_retry` returns `connecting/wireless` and sends `usb-retry` to the fake helper. The UI test asserts the dot changes on Retry. |
| C5(a) | `c5a_read_only_availability_reports_export_and_then_clears`: the query answers `an export is running` with no editor action, then `None`; the UI test checks the disabled button, reason, and later enabled button. |

## Assumptions
- A retryable helper `DISCONNECTED` remains a Connecting state through its backoff. The terminal `ERROR` and helper exit end that state.
- The helper may report a claimed cable as either a failed USB connect or no USB endpoint. Both consume the three-attempt fallback budget.
- The 15-second wait starts at each later relaunch attempt; the one-shot app-start order remains unchanged.
- The read-only take-back availability query refreshes once per second while either action is shown.
- The deferred `variant_not_found` preview belongs to the preview service's retry state. Pending adapter renders are excluded from the take-back count; issued jobs and starts in progress remain included.

## Deviations
- The Health screen has a second take-back button, so `HealthPage.tsx` and its P51 test were changed to apply the same guard. The P51 camera-dot test changed its Connecting label and waits for the availability query before clicking. These are existing assertions whose visible outcome changed.
- P45's fixed `21 tests` assertion was changed to `0 failed`, as the plan permits for a new C++ case. Generated Tauri bindings were refreshed.
- The optional local Sony SDK directory in the plan was absent. The SDK build and live camera checks were not run; P52 run 2 is the hardware acceptance.

## Files changed
- `crates/core/src/ports/editing.rs`
- `crates/integrations/src/camera/direct/{adapter,start_order,take_back}.rs`, `crates/integrations/src/camera/runtime.rs`
- `crates/integrations/src/editing/capture_one/{adapter,gate}.rs`
- `crates/testkit/tests/{p45_sidecar_and_protocol,p54_camera_cable_and_take_back}.rs`
- `native/camera-sidecar/src/transport_manager.cpp`, `native/camera-sidecar/tests/test_transport_manager.cpp`
- `apps/desktop/src-tauri/src/{commands/camera,lib}.rs`
- `apps/desktop/src/features/shell/{CameraCard,CameraDot}.tsx`, `apps/desktop/src/features/health/HealthPage.tsx`
- `apps/desktop/src/features/shell/__tests__/{p51_camera_dot,p54_camera_cable}.test.tsx`, `apps/desktop/src/features/health/__tests__/p51_health_camera.test.tsx`
- `apps/desktop/src/services/{camera,bindings}.ts`
- `docs/implementation/{master_plan.md,plans/P54_camera_cable_and_take_back.md,handoffs/implementer/handoff_P54_camera_cable_and_take_back_implement_r1.md}`

## Checkpoint SHA
The commit containing this handoff is the checkpoint; resolve its SHA with `git log -1 --format=%H -- docs/implementation/handoffs/implementer/handoff_P54_camera_cable_and_take_back_implement_r1.md`. A literal SHA here would change the commit's SHA.

## Stamp
| Command | Result |
|---|---|
| `cargo test --workspace` | green, 834 passed, 0 failed; P54 6/6 and desktop command 1/1 |
| `cargo clippy --workspace --all-targets -- -D warnings` | green |
| `cargo fmt --all -- --check` | green |
| `npm run typecheck` (`apps/desktop`) | green |
| `npm test` (`apps/desktop`) | green, 407 passed |
| `tools/sidecar/build.sh --stub --test` | green, 22 C++ tests, 0 failed |
| `cargo test -p psm-desktop c4b_retry_command --lib` | green after the test's final retryable-disconnect fixture correction |
| `git diff --check` | green |

## Manual try-out
1. Open the app and switch the camera on over Wi-Fi. Confirm the dot first reads `Connecting · Wi-Fi`, then shows the connected camera. Disconnect briefly and click Retry; the dot should change to Connecting immediately.
2. With Capture One running and the cable attached, let the system camera service take the cable. Confirm Wi-Fi shooting and Auto FTP delivery continue. The take-back button should say why it is disabled while an export or active render is running, then enable when that work ends.
3. Click "Take the camera back" once. Confirm Capture One quits, the helper acquires USB within about 20 seconds, Capture One reopens, and the next shot transfers over USB. Quit Capture One once more while the cable is attached; confirm it stays closed until the helper has USB or 15 seconds pass.
4. Re-plug the cable while another program holds it. Confirm the helper returns to Wi-Fi within ten seconds after its first failed USB attempt; after releasing the holder, use a re-plug or take-back and confirm it returns to USB at a safe point.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
