---
plan: P57_camera_auto_take_back_and_run2_fixes
role: implement
round: 1
state: DONE
date: 2026-11-09
actor: codex
---

# P57 implement r1 — automatic cable take-back and second camera-run fixes

## What was built

Plugging in the Sony camera while Capture One holds the cable now starts the existing take-back automatically, once per plug-in. Settings → Camera has a default-on switch to turn this off. When an export, transfer, or other take-back precondition is active, the camera header says why it is waiting and proceeds when the condition clears.

Camera helper messages now have a dedicated 20 ms app polling thread. **F16 cause:** the helper already flushed every JSON line, but the app polled it on the same background thread that could block in preview and Capture One work. A photo no longer waits for that work or a later helper line to make the app consume its event.

A late camera-initiated exposure can be linked to a photo already filed in the ledger, avoiding a false “files not received” notice. Output quality and size saves take the fast settings path, update future exports, and do not request existing previews again. Each export run keeps the format it started with. The Capture One launch gate is checked again after a queued scripting call gets its lane, preventing an early relaunch during take-back.

The helper reads FTP state when a session connects and every 60 seconds. An FTP connection error puts the Mac’s current LAN address and receiver port in the camera header; a connected status or received wireless upload clears it. After a helper restart, the last discovered Wi-Fi IP is tried directly before SDK discovery, with discovery on failure.

## Tests → criterion rows

| Row | Boundary proof |
|---|---|
| C1(a) | Rust P57 test: plug/claim makes one quit and one launch after a USB session; another claim in the same plug-in makes no second take-back. |
| C1(b) | Rust P57 test: a running export keeps Capture One open, shows the exact waiting sentence, then permits take-back. UI P57 test checks the header sentence. |
| C1(c) | Rust P57 test: setting off keeps the manual take-back available and does not quit Capture One. UI P57 test checks the setting switch. |
| C2(a) | Rust P57 test: one fake helper line, followed by silence, becomes connected camera status within 200 ms. |
| C3(a) | Rust P57 test: a filed `DSC05001.ARW` then a 12-second-late exposure is linked; no files-not-received notice at 61 seconds. |
| C4(a) | Rust P57 test: saving changed output quality with a render in flight returns within one second and issues no new preview process. |
| C5(a) | Rust P57 test: no launch while take-back waits for USB; the eventual launch has a connected USB session and the gate releases. |
| C6(a) | Rust P57 test: FTP error shows the exact address/port sentence and clears on an upload or connected status, even with a stale error value. |
| C7(a) | C++ P57 test: the stub-backed SDK adapter supplies a cached direct IP without discovery, a failed direct attempt falls back to discovery, and the transport manager retries even if that direct failure was marked fatal. |
| C8(a) | Full suite and tool stamps below. |

## Assumptions

- The SDK `capture.exposed` callback has no camera filename. For an exposure reported after its file arrived, the ledger associates the oldest unassociated, completed, nonduplicate delivery within its 300-second window. This follows MC-33’s FIFO fallback; out-of-order exposures with no filename cannot be matched by exact stem.
- The Mac LAN address is selected from the default-route interface with a UDP route lookup; if no route exists, the header names “this Mac’s LAN address” rather than showing an unusable address.
- FTP status value `2` means connected, matching the Sony lab fixture. A connected reading clears a stale error value.
- A failed cached direct-IP attempt falls back to SDK discovery, even if the direct attempt reports a fatal error. A precondition race before automatic take-back starts leaves the plug-in pending; another take-back failure consumes that plug-in’s single attempt.

## Deviations

- C3’s exact filename/stem association cannot be made for a late exposure because the SDK event carries neither value; the FIFO fallback and its limit are recorded above.
- The implementation also touched the export service, SQLite store, app worker, C++ test wiring, and testkit fixture to meet the named boundaries. Existing assertions changed only where the new default-on camera setting changed their contract: P43’s saved camera object, P54’s manual-only fixture, and P51’s settings UI expectation.
- The known intermittent `p33_split_grid` UI test timed out in two full runs while Cargo was active. It passed alone and the final full frontend run passed 413/413 after Cargo stopped. No P57 frontend failure remained.

## Files changed

- Desktop commands/UI: `apps/desktop/src-tauri/src/commands/{camera,settings}.rs`, `apps/desktop/src/features/health/HealthPage.tsx`, `apps/desktop/src/features/settings/{CameraSection,SettingsPage}.tsx`, settings tests `p51_camera_section.test.tsx` and `p57_auto_take_back.test.tsx`, `apps/desktop/src/services/bindings.ts`.
- Core and SQLite: `crates/core/src/domain/camera.rs`, `crates/core/src/mechanisms/camera_status.rs`, `crates/core/src/ports/store.rs`, `crates/core/src/services/{camera_ledger,export,settings}.rs`, `crates/infra/src/sqlite/store.rs`.
- Integrations: `crates/integrations/src/camera/direct/{adapter,take_back}.rs`, `crates/integrations/src/camera/runtime.rs`, `crates/integrations/src/camera/sidecar/protocol.rs`, `crates/integrations/src/editing/capture_one/gate.rs`, `crates/integrations/src/lib.rs`.
- Testkit: `crates/testkit/src/test_app.rs`, `crates/testkit/tests/{p43_ftp_receiver,p54_camera_cable_and_take_back,p57_camera_auto_take_back_and_run2_fixes}.rs`.
- Native helper: `native/camera-sidecar/CMakeLists.txt`, `include/crlab/{camera,transport}.hpp`, `src/{main,sdk_adapter,transport_manager}.cpp`, `tests/test_transport_manager.cpp`.
- Process: `docs/implementation/master_plan.md` (P57 → IMPLEMENTED) and this plan’s Review log. This handoff is left for the runner’s handoff commit per flow §8.

## Checkpoint SHA

`763b99771f4383f45a3ed57964ff472db65c9e32` — `CHECKPOINT (not approved): P57 camera auto take-back and run-two fixes`.

## Stamp

| Command | Result |
|---|---|
| `cargo test -p psm-testkit --test p57_camera_auto_take_back_and_run2_fixes` | 8/8 passed |
| `cargo test --workspace` | 849 tests passed, 0 failed, 56 test groups |
| `cd apps/desktop && npm test` | 413/413 passed, 57 files |
| `cd apps/desktop && npm run typecheck` | Passed |
| `cargo clippy --workspace --all-targets -- -D warnings` | Passed |
| `cargo fmt --all -- --check` and `git diff --check` | Passed |
| `tools/sidecar/build.sh --sdk /Users/davidloorenz/Desktop/Developer/Camera-Connection-Tests/vendor/sony-sdk` | SDK build passed; no SDK file staged |
| `tools/sidecar/build.sh --stub --test` | 28/28 native tests passed; stub restored |

No live camera capture was performed in this session.

## Manual try-out

1. Start the desktop app on the Mac with `cd apps/desktop && npm run tauri -- dev`. In Settings → Camera, leave “Take the cable back from Capture One automatically when the camera is plugged in” on.
2. Open the Capture One session, then plug in the awake camera. Capture One should close and reopen without a click; the header should reach Cable after our USB session connects. If an export is running, it should first say why take-back is waiting.
3. Take two photos over the cable. Each should appear without waiting for the next shot, and there should be no false “files not received” notice.
4. Change Output JPEG quality in Settings and save. Saving should leave “Saving…” promptly, and existing previews should stay as they are.
5. To check FTP guidance, point the camera’s FTP host at a wrong address, then restore it afterward. The header should show this Mac’s current LAN address and receiver port, and clear when FTP reconnects or a photo uploads.

## ⚠ OWNER DECISIONS REQUIRED (0)

None.
