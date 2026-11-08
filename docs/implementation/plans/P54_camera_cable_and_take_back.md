---
phase: P54_camera_cable_and_take_back
title: Getting the cable back after P52 — "Take the camera back" is no longer blocked by previews that wait for Capture One, the app does not relaunch Capture One into the cable's way, shooting continues over Wi-Fi while another program holds the cable, and the dot says "Connecting"
difficulty: high
projection: no
depends_on: [P53_camera_upload_and_log]
---
# P54 — Cable, take-back, Wi-Fi fallback, Connecting (P52 findings F7, F10, F6, F3)

Written by the orchestrator on 2026-11-08 from P52 run 1 (evidence `docs/intention/evidence/sony_camera_lab.md` §Acceptance (P52, 2026-11-08), rows A1, A6, A17
and "Other observations").

## Goal
1. **F7: take-back refused forever.** The click answered "a preview is rendering" for as long as the owner tried. The precondition counts
   `renders_in_flight = jobs + pending` (`crates/integrations/src/editing/capture_one/adapter.rs`), and `pending` keeps renders deferred on
   `variant_not_found` (Capture One has no variant for the file yet) that are retried every ~1 s indefinitely. After this phase only a render Capture One is
   actually processing blocks take-back; deferred/pending renders do not, and they resume after Capture One is back.
2. **F10: the relaunch hands the cable to the system camera service.** When the photographer quits Capture One, the app relaunches it within ~0.5 s, and
   `ptpcamerad` takes the cable ~6 s later, every time, so our helper never gets it. After this phase, when Capture One is gone while the camera is on the cable
   and our helper does not hold the cable, the app waits for our USB session (the same start order as at launch, 10E MC-34 / V6 "SDK first", bounded by
   the existing start-order wait) before relaunching Capture One.
3. **F6: no Wi-Fi while the cable is held elsewhere.** With `ptpcamerad` holding the cable, the helper never fell back to Wi-Fi (~5 min). 10E MC-34 §Guide and
   A13: "shooting continues over Wi-Fi" while another program holds the cable. After this phase the helper reaches Wi-Fi after the configured USB failures
   (`usb_failures_before_wifi_fallback`, 3) within a bounded time, also when the USB attempts fail because the interface is held, and keeps watching for the
   cable (a re-plug or the holder quitting switches back to USB as today).
4. **F3: no "Connecting".** At a Wi-Fi cold start the dot read `Camera disconnected` for minutes, never `Connecting`, though SSH was up at +8 s; Retry did
   nothing visible. After this phase the dot shows `Connecting` (with the link) from the first discovery/connect state until ready or failure, and
   Retry restarts the attempt at once (the dot shows `Connecting` immediately). The implementer reads the cold-start path (`apply_status` mapping of
   `transport.state` reasons, the Keychain/credentials restart path from P50) and fixes whatever keeps the dot at `disconnected` while the helper is
   connecting; the cause found is written in the handoff.
5. **R-I32 (backlog P51 r1):** "Take the camera back" is disabled and says why (the first failing precondition) instead of explaining only after a click,
   using a read-only precondition query.

**Not in this phase:** receiver/log changes (P53), install/capture button/restart timing (P55), preview speed (F11). Never kill or quit `ptpcamerad` or any
other program automatically; Capture One is quit only by the take-back click (owner rule).

## Read first
Book `10E_direct_camera_adapter.md` MC-31 §Transport policy, MC-34 (§Detect, §Guide, §Take it back, start order), R-I22, R-I23, R-I32, M39, M40; evidence
§V6 and §Acceptance (P52). Code: `crates/integrations/src/camera/direct/{take_back.rs, start_order.rs, ownership.rs, adapter.rs}`,
`crates/integrations/src/editing/capture_one/{adapter.rs, gate.rs}` (renders, relaunch/ensure-running — find where Capture One is relaunched after it quits),
`crates/core/src/ports/editing.rs` (`renders_in_flight`), `crates/integrations/src/camera/runtime.rs`, `native/camera-sidecar/src/transport_manager.cpp` +
`include/crlab/transport.hpp` + `tests/test_transport_manager.cpp`, the header camera dot / popover in `apps/desktop/src/` (find by `camera_connecting`
and `Take the camera back`), `apps/desktop/src-tauri/src/commands/camera.rs`. Tests: `crates/testkit/tests/{p48_camera_ownership.rs, p49_*.rs, p51_*.rs}`,
the p51 UI tests.

## Files expected to change (write perimeter)
The files above as needed, plus `crates/testkit/tests/p54_camera_cable_and_take_back.rs` (new), new C++ cases in `native/camera-sidecar/tests/` (if P45's
fixed test count blocks a new `TEST_CASE`, change P45's check to "0 failed" — allowed in this phase), one new UI test file under the header/camera feature,
generated bindings. Existing assertions change only where the contract changes them (list each in the handoff). Estimated ≈ 20 paths, ≈ 650 lines.

## Core mechanisms (unit tests allowed)
Take-back preconditions (`take_back::preconditions`); the helper's transport policy (C++ `transport_manager`).

## Criteria (7 criteria, 8 rows)
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | take-back preconditions | Capture One holds the cable; one render deferred on `variant_not_found` (pending, not processing); no upload/export/publish | take-back proceeds (no "a preview is rendering"); after Capture One is back the deferred render is still pending/retried | `[rule]` MC-34 §Take it back; P52 A17 |
| C1(b) | take-back preconditions, unchanged case | one render Capture One is processing | refused with `a preview is rendering` | `[rule]` MC-34 |
| C2(a) | no early relaunch | camera on the cable, helper not on USB, Capture One quits (fake) | Capture One is not relaunched until our helper reports `connection.ready transport=usb`, then relaunched; if that does not come within the start-order wait, relaunched at its end | `[rule]` MC-34 start order; V6; P52 F10 |
| C2(b) | relaunch unchanged otherwise | no camera on the cable (or helper already on USB); Capture One quits | relaunched as today (no wait) | `[rule]` 10E §Capture One alongside |
| C3(a) | helper: Wi-Fi while the cable is held | C++ transport test: USB present, every USB connect fails "held/busy"; Wi-Fi available | after `usb_failures_before_wifi_fallback` failures the manager connects Wi-Fi (`usb.fallback_to_wifi`, `CONNECTED_WIFI`) within the policy's bounded time; a later successful USB attempt switches back when not busy | `[rule]` MC-34 §Guide; MC-31 §Transport policy; P52 A6 |
| C4(a) | dot shows Connecting | fake helper emits the Wi-Fi cold-start sequence (`transport.state` DISCOVERING/CONNECTING_WIFI with the reasons the real helper sends) | camera status `connecting` with link Wi-Fi from the first such event until `connection.ready`; the header dot label `Connecting · Wi-Fi` (UI test) | `[rule]` 10E §What the photographer gets; P52 A1 |
| C4(b) | Retry | status disconnected; `camera_retry` | the helper receives a reconnect at once and the status is `connecting` immediately | `[rule]` 10E §What the photographer gets |
| C5(a) | take-back disabled with reason | precondition `an export is running` true | the button is disabled and shows `an export is running`; enabled when all preconditions clear (UI test against the read-only query) | `[rule]` R-I32 |

Plus the closing gate row: `cargo test --workspace`, clippy, fmt, `npm run typecheck`, `npm test` (known intermittent `p33_split_grid` aside), `tools/sidecar/build.sh --stub --test` green.

## Delegated choices
How "processing" vs "pending" is exposed by the editing port (a second method or a changed count); where the relaunch wait lives (start-order module vs the
Capture One gate); the fallback timing constants (keep them ≤ 10 s from the first failed USB attempt to `CONNECTED_WIFI`); the query's command name.

## Manual try-out
P52 run 2: A6 (cable in with Capture One running: Wi-Fi keeps shooting), A17 (one click → USB session ≤ ~20 s, Capture One back), A1 (`Connecting · Wi-Fi`
then `Connected` ≤ ~6 s), quitting Capture One on the cable lets our helper take it first.

## Notes
- **Hard rules (owner):** never commit a Sony SDK file, a `.env`, a credential or an SSH fingerprint; no Python; never change the camera's network/FTP settings,
  delete camera files or kill/quit any process automatically (Capture One only through the take-back click); live view out of scope; the suite never needs
  the camera or Capture One. Lab repo read-only. Default implementer. If the SDK is present locally (`~/Desktop/Developer/Camera-Connection-Tests/vendor/sony-sdk`),
  prove the SDK build compiles with `tools/sidecar/build.sh --sdk <path>`, then put the stub back with `tools/sidecar/build.sh --stub`; never commit build output.
- **Hazards:** F3's cause is not known (no logs in P52 run 1); P53 adds the log lines — read P53's helper event names. Do not make take-back ignore renders
  that are really in Capture One's queue (a quit would lose them).

## Review log

- **2026-11-08 implement r1 (codex).** The take-back guard counts only Capture One renders actually issued or starting, leaving deferred previews to retry. The editor launch path waits for the helper's USB session after a later quit when the cable is present, including scripts that could implicitly launch the editor; the existing 15 s bound applies. USB retries now cap their per-attempt delay at 2 s before Wi-Fi fallback. Retry sends `usb-retry` and returns Connecting immediately; retryable transport gaps keep their link visible. The camera popover and Health screen query the read-only take-back preconditions each second and disable the action with the first reason. Assumptions: a retryable `DISCONNECTED` from the helper remains Connecting through its backoff; `ERROR` is terminal, and the query's one-second refresh is sufficient. The helper may report a claimed cable either as a failed connect or no USB endpoint. Deviations: the Health screen shares the same action and received the same guard; P51's two UI tests changed for the new initial disabled state and label; P45's fixed helper-test count changed to `0 failed` as the plan allows. The SDK source directory in the note was absent, so the stub was built and tested; no live camera/Capture One probe was run in this session. Closing stamp: `cargo test --workspace` 834/0, `npm test` 407/0, clippy, fmt, typecheck and stub helper 22/0 green. P52 run 2 remains the hardware check.
- **2026-11-08 review r1 (opus): CHANGES_REQUESTED.** Perimeter = the 24 declared paths (`e8a95cd`). Suite green (Rust 834/0, UI 407/407, helper stub 22/0, clippy, fmt, typecheck; reviewer run, code tree `e8a95cd`). Verified correct: C1(b) (`max(jobs, busy)` still counts issued renders and starts in progress; `c1b` asserts the exact reason), C1(a) (excluded `pending` entries always sit behind a `busy` mark except orphans; full take-back with a deferred preview, preview in flight after the variant appears), C3(a) (three USB failures incl. empty enumeration → `CONNECTED_WIFI` ≤ 10 s with a 2 s USB delay cap; safe switch back), C4(a) (retryable `DISCONNECTED` keeps Connecting; `c4a` fails on the old mapping; UI label exact), C4(b) (`usb-retry` reaches the fake helper; exact `connecting/wireless` JSON), C5(a) (read-only query, no scripting op; disabled + reason, then enabled); assumptions within 10E; H1–H8 hold (H8 drift in `camera_retry` → backlog). **F1 (B1, production):** the relaunch wait never sends `usb-retry`; after the three-failure fallback the helper retries USB only on a re-plug, so quitting Capture One on the cable waits 15 s and Capture One takes the cable again (Goals 2–3, Manual try-out, MC-34). Fix: one `usb-retry` when the wait starts. **F2 (B2, production):** `c2a` sets `StartOrder::camera_state` directly and `c2b`'s USB case has no `usb.initial`, so neither fails if the helper-event wiring is missing. Fix: drive C2 through fake-helper events and assert `usb-retry` before launch. 7 backlog notes (F7 test passes on the pre-P54 count, likely orphaned `pending`, which also leaves a preview stuck; F6 cause unproven; Connecting while the camera is off).
- **2026-11-08 fix r2 (codex):** F1: the shared later-relaunch wait sends one best-effort `usb-retry` through `CameraRuntime` before waiting for USB readiness; send failure is logged and the existing bounded wait remains. F2: C2(a) now feeds `usb.initial`, `session.closed`, `usb.claimed_by`, and `connection.ready` through the fake helper and asserts exactly one retry arrives before launch; the 15 s timeout case asserts its retry too. C2(b) supplies `usb.initial` true/false for the ready-USB and no-cable cases and confirms neither sends a retry. Judgment: keep retry dispatch registered by the runtime on `StartOrder`, using a weak runtime reference, so launch and other launch-capable scripts share the same wait trigger without changing their gates. Closing stamp: `cargo test --workspace` green (P54 6/6); clippy, fmt, typecheck, and helper stub 22/0 green. UI suite initially hit known intermittent `p33_split_grid` C4(d) (406/407); rerun green 407/407. No divergence or owner decisions.
- **2026-11-08 re-review r2 (opus): APPROVED_WITH_NOTES.** Perimeter = 6 paths in `7e31f09`, all within F1/F2's allowed files; the foreign commits in between are docs/tooling only. Reviewer run on tree `01a0dba` (clean): `cargo test --workspace` 834/0 (P54 6/6), clippy, fmt, typecheck, `npm test` 407/407, helper stub 22/0. **F1 resolved:** `wait_for_relaunch` sends one `usb-retry` (registered by the runtime through a weak reference; send errors logged) when the cable is present and USB not ready. By reading, the helper's `usb-retry` resets the fallback and the Wi-Fi→USB switch runs inside the 15 s wait (3 s dwell, busy-deferred, 1 s after disconnect). Take-back step 5 is permitted, so there is no second retry. No lock cycle. The app-start order is unchanged. **F2 resolved:** C2(a)/(b) are driven by fake-helper `usb.initial`, `session.closed`, `usb.claimed_by`, `connection.ready` events and assert `usb-retry` on helper stdin before any `launch` (one per wait; none in C2(b)). Each wiring gap fails a step. No blocking item remains, so no triage. Notes → backlog: the relaunch retry drops Wi-Fi for a gap when another program holds the cable (P52 run 2); `c2a` syncs on `needs_relaunch_wait()`. r1 notes carried forward (dispositions in the r2 handoff).
