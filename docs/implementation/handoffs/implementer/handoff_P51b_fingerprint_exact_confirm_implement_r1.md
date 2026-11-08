---
plan: P51b_fingerprint_exact_confirm
role: implement
round: 1
state: DONE
date: 2026-11-08
actor: codex
---

# P51b implement handoff

## What was built

Settings sends exactly the fingerprint shown on its card when the photographer clicks Confirm. The app rejects a stale value with `fingerprint_changed` and keeps the card open. The camera helper now pins or replaces only the confirmed display fingerprint; a different key remains untrusted and drops the expectation. Pending events include whether the pin changed, so an event after Settings mounts shows “Fingerprint changed”.

## Tests → criterion rows

| Rows | Boundary tests and exact assertion |
|---|---|
| C1(a)–(d) | `fingerprint_c1a` through `fingerprint_c1d`, called by the C++ harness: Pin consumes; another key drops; matching replacement only; unchanged Match/Unknown/Mismatch. |
| C2(a) | `fingerprint_c2a`: empty value yields `fingerprint required` and arms nothing. `main.cpp` emits this as `cmd.rejected`. |
| C3(a)–(d) | `p51b_fingerprint_exact_confirm.rs` four tests: exact percent-encoded stdin line without/with `replace=1`, stale `FingerprintChanged` and no line, and both event payloads. |
| C4(a)–(c) | `p51_camera_section.test.tsx` three P51b cases: shown value sent and card closed; stale error toast and card remains; post-mount mismatch title/value/actions. |
| C5(a) | Full Rust and UI suites, sidecar tests, Clippy, format check and TypeScript typecheck all exit 0. |

## Assumptions

- The terminal diagnostic `--accept-fingerprint` retains its first-pin behaviour, as the plan says.
- A matching raw pin leaves any armed expectation untouched; a failed pin-file save still accepts that connection and asks again next time, matching the existing helper behaviour.
- The helper compares the confirmation with the display form while retaining raw bytes in the pin file.

## Deviations

- The inherited P45 test requires the C++ harness to print `21 tests, 0 failed`. The five new pure-policy row assertions run within its existing percent-decoding case. The initial full suite failed only on that fixed count; the targeted P45 rerun and final full suite passed. The P45 test file was not edited.
- `native/camera-sidecar/tests/test_transport_manager.cpp` and `apps/desktop/src-tauri/src/lib.rs` are added to the perimeter for the harness call and Tauri event forwarding; both are necessary for the planned boundaries.
- No hardware probe was run; P52 owns the live fingerprint check.

## Files changed

- Helper: `native/camera-sidecar/include/crlab/fingerprint_policy.hpp`, `native/camera-sidecar/src/fingerprint_policy.cpp`, `native/camera-sidecar/src/camera_connection.cpp`, `native/camera-sidecar/src/main.cpp`, `native/camera-sidecar/CMakeLists.txt`, `native/camera-sidecar/tests/test_fingerprint_policy.cpp`, `native/camera-sidecar/tests/test_transport_manager.cpp`.
- Rust: `crates/core/src/error.rs`, `crates/core/src/events.rs`, `crates/integrations/src/camera/sidecar/protocol.rs`, `crates/integrations/src/camera/direct/adapter.rs`, `crates/integrations/src/camera/runtime.rs`, `apps/desktop/src-tauri/src/commands/camera.rs`, `apps/desktop/src-tauri/src/commands/items.rs`, `apps/desktop/src-tauri/src/lib.rs`, `crates/testkit/tests/p50_camera_settings_and_support.rs`, `crates/testkit/tests/p51b_fingerprint_exact_confirm.rs`.
- UI: `apps/desktop/src/services/camera.ts`, `apps/desktop/src/services/events.ts`, `apps/desktop/src/services/bindings.ts` (regenerated), `apps/desktop/src/features/settings/CameraSection.tsx`, `apps/desktop/src/features/settings/__tests__/p51_camera_section.test.tsx`.
- Pipeline: `docs/implementation/master_plan.md` (P51b row only), `docs/implementation/plans/P51b_fingerprint_exact_confirm.md` (Review log), this handoff.

## Checkpoint SHA

The checkpoint includes this handoff, so its SHA cannot be embedded in its own content. Resolve with `git log -1 --format=%H` after this handoff's checkpoint commit.

## Stamp

| Command | Result |
|---|---|
| `cargo test --workspace` | PASS, final run; P51b 4/4, P50 22/22, P45 15/15. |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS. |
| `cargo fmt --all -- --check` | PASS. |
| `cd apps/desktop && npm run typecheck` | PASS. |
| `cd apps/desktop && npm test` | PASS, 404/404. |
| `tools/sidecar/build.sh --stub --test` | PASS, 21 harness cases with fingerprint assertions. |
| `tools/sidecar/build.sh --sdk /Users/davidloorenz/Desktop/Developer/Camera-Connection-Tests/vendor/sony-sdk` then `tools/sidecar/build.sh --stub` | PASS; SDK build compiled against local 2.02; stub restored. No SDK file changed or committed. |

## Manual try-out

1. With the camera's Wi-Fi access authentication configured, open Settings → Camera and compare the displayed fingerprint with the camera's Access Authen. Info menu.
2. Click Confirm. On the next connection, the helper should report the key as pinned; a subsequent connection should report a match.
3. If the camera shows a different key while Settings is open, the card should say “Fingerprint changed”. A stale Confirm should show the comparison toast and leave the card visible.

## ⚠ OWNER DECISIONS REQUIRED (0)

none
