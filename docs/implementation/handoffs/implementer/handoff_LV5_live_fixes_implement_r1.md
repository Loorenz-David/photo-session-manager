---
plan: LV5_live_fixes
role: implement
round: 1
state: DONE
date: 2026-11-26
actor: codex
---

# LV5 live fixes — implementer handoff

## What was built

- The current shot holds a shutter's container while its file has been filed but the photo has not yet been taken in. Later shutters advance to fresh containers, and late photo intake does not move the current shot backward.
- Mac review, template, and undo/redo actions now announce committed session changes to connected phones. A photo view includes an opaque preview version after its preview exists, and preview completion announces the session.
- A slow phone reader skips intermediate live frames while its socket writer remains active; a sustained stall or revoked pairing closes the viewer. Settings → Phone refreshes the connection list while mounted.
- The camera helper refuses phone focus in nonmovable Focus Areas without writing camera settings. Real helper exposure display fields are parsed and mapped to the phone's readout.

## Tests → criterion rows

| Rows | Behaviour tests | Result |
|---|---|---|
| LV5·C1(a–c), C2(a–b), C3(a–b), C4(a–b) | `lv5_current_shot` (9 tests) | Pass after fake-sidecar readiness fix. |
| LV5·C5(a–g), C6(a–e), C7(a–c), C8(a–c) | `lv5_mac_actions` (18 tests) | Pass. |
| LV5·C9(a–d) | `lv5_phone_socket` (4 tests) | Pass; C9(a–c) assert viewer continuity/removal and frame delivery, but do not capture and assert the specified `phone.viewer closed` trace lines. |
| LV5·C10(a–b) | `lv5_phone_list.test.tsx` (2 tests) | Pass. |
| LV5·C11(a–j) | `native/camera-sidecar/tests/test_liveview.cpp` (10 cases) | Pass in stub build. |
| LV5·C12(a–b), C12(d–f) | `lv5_camera_readouts` (5 tests), plus the revised `lv1_frame_channel` exposure case | Pass. |
| LV5·C12(c) | `camera::direct::live` readout unit test | Pass. |

The U1 tests assert placement and current-shot state; they do not assert the precise `current_shot.set` log lines named in C1(a), C2(a–b), and C3(a–b). The C9 tests assert frame delivery and viewer lifecycle, without the `phone.viewer closed` log lines. The test kit has no log-capture facility. C3(a–b) use the planned ordering and file/intake gaps; their exact clock offsets remain for review.

The amended P40 C7(a) test, `c7a_attribute_undo_announces_session`, passed with its module (18/18): set and undo retain their exact two history events; undo and redo each announce the touched session, with no photo event.

## Assumptions

- A filed file still holding a shot is found through its final path against the session's photos; the hold ends after the existing FIFO window. The test kit exposes separate file and intake steps for this gap.
- The preview version is an opaque token derived from preview path and modification time. The phone UI can consume it in LV6.
- The phone's in-flight socket send remains pinned across frame, ping, revoke, and stall checks. The server attempts a short close before unsubscribing.
- A readable Focus Area number in the documented movable set permits position setting; absent Focus Area falls back to position writability. The helper uses a header-only predicate, adds no `focus_area` field to the result log, and writes no new camera property.
- The missing exposure display is represented by the literal en dash `–`.
- For criteria that name a log trace, the existing test kit cannot capture log lines. The corresponding current-shot state, exposure bindings, photo places, socket delivery and viewer removal are the automated boundary proof; the log clause remains a review limitation.

## Deviations

- The orchestrator amended this plan's perimeter for the P40 test and ruled that PH8 supersedes its old attribute-undo event expectation. The resumed closing gate is green.
- The exact trace assertions for C1(a), C2(a–b), C3(a–b), and C9(a–c) are absent as described above; the existing harness cannot capture logs.

## Files changed

- Unit 1: `crates/infra/src/sqlite/store.rs`; `crates/testkit/src/test_app.rs`; `crates/testkit/tests/suite/lv5_current_shot.rs`; `crates/testkit/tests/suite/main.rs`; `docs/guide/02_architecture.md`; `docs/guide/03_rules_and_principles.md`.
- Unit 2: `crates/core/src/services/history.rs`, `phone_view.rs`, `review.rs`; `crates/testkit/tests/suite/lv5_mac_actions.rs`; `docs/guide/02_architecture.md`; `docs/guide/04_change_impact.md`.
- Unit 3: `apps/desktop/src/features/settings/PhoneSection.tsx`, `__tests__/lv5_phone_list.test.tsx`; `crates/integrations/src/phone/server.rs`, `ws.rs`; `crates/testkit/src/phone.rs`; `crates/testkit/tests/suite/lv5_phone_socket.rs`; `docs/guide/02_architecture.md`.
- Unit 4: `native/camera-sidecar/include/crlab/liveview.hpp`, `src/live_runner.cpp`, `tests/test_liveview.cpp`; `crates/integrations/src/camera/direct/live.rs`, `sidecar/protocol.rs`; `crates/testkit/tests/suite/lv5_camera_readouts.rs`, `lv1_frame_channel.rs`, `lv2_phone_ws.rs`; `docs/guide/03_rules_and_principles.md`, `04_change_impact.md`.
- Closing: `crates/testkit/tests/suite/lv5_current_shot.rs` (fake-sidecar readiness); `crates/testkit/tests/suite/p40_undo_history_values.rs` (orchestrator-authorized expectation); `docs/implementation/plans/LV5_live_fixes.md` (tracker and Review log); this handoff.

## Checkpoint SHA

1. U1 — `b487c8de` — current-shot hold.
2. U2 — `f82f0886` — Mac announcements and preview versions.
3. U3 — `78018009` — phone stalls and Settings refresh.
4. U4 — `49ee1c8e` — Focus Area and exposure readout.

The earlier closing documentation and readiness fix are in `8f12723c`. This continuation's test and documentation are in its final `CHECKPOINT (not approved):` commit; its SHA is recorded by Git history and the runner.

## Stamp

| Command | Result |
|---|---|
| Unit 1 plan commands: focused `lv5_current_shot`, core tests, clippy | Pass before checkpoint. |
| Unit 2 plan commands: focused `lv5_mac_actions`/related suite and clippy | Pass before checkpoint. |
| Unit 3 plan commands: focused socket suite, Settings Vitest, typecheck, clippy | Pass before checkpoint. |
| Unit 4 plan commands: `tools/sidecar/build.sh --stub --test`, focused camera unit and suite, clippy | Pass before checkpoint. |
| `cd apps/desktop && npm run build` | Pass. |
| `cargo test --workspace` | Pass on continuation after the authorized P40 expectation update. |
| `cargo test -p psm-testkit --test suite lv5_current_shot::` | Pass: 9 tests after readiness fix. |
| `cargo test -p psm-testkit --test suite p40_undo_history_values::` | Pass: 18 tests, including undo and redo event expectations. |
| `cargo clippy --workspace --all-targets -- -D warnings` | Pass. |
| `cargo fmt --all -- --check` | Continuation's first check found formatting in the P40 test; `cargo fmt --all` applied, check then passed. |
| `tools/sidecar/build.sh --stub --test` | Pass. |
| `cd apps/desktop && npm run typecheck` | Pass. |
| `cd apps/desktop && npm test` | Pass: 147 files, 1,049 tests. |

## Manual try-out

With this branch's app and SDK helper built, connect the camera over Wi-Fi and pair two phones. In a templated session, take four photos about one second apart: the Mac's Next outline should wait for each arriving photo, and each shot should receive its own photo. Deny, move to Condition, and Undo a photo on the Mac; both phones should refresh within a second. While viewing live on a phone, briefly walk to weak Wi-Fi: the image may freeze, then resume at the newest frame without leaving live view. Set the camera Focus Area to Wide and tap live view: the phone should explain that a movable area is needed; then switch to Flexible Spot and tap again to focus. Check that live exposure reads like `1/80 · f/8 · ISO 2000`. In Settings → Phone, a connected phone should read “Connected now”; after closing its app, the connection state should clear within about 20 seconds. The phone dock gestures and tile image issue belong to LV6.

## ⚠ OWNER DECISIONS REQUIRED (0)

None. The blocking test/perimeter conflict is a plan correction for the orchestrator, not an owner product decision.
