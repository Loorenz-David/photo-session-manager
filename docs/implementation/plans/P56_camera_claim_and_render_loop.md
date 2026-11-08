---
phase: P56_camera_claim_and_render_loop
title: Session Manager takes the cable first and never sits idle, and stops re-exporting the same previews from Capture One in an endless loop
difficulty: high
projection: no
depends_on: [P55_camera_setup_polish]
---
# P56 — Claim the cable at plug-in, never idle, end the export loop (P52 findings F12, F13, F14)

Written by the orchestrator on 2026-11-08 after a follow-up check with the owner (evidence `docs/intention/evidence/sony_camera_lab.md` §Acceptance (P52,
2026-11-08), "Provider test" and "Export loop"). The owner: "we need a more reliable alternative for Session Manager to take over the camera fast, the whole
point is that connection and disconnection happen fast and the flow is not disturbed."

## Goal
1. **F13: take the cable before the system camera service.** With every camera provider switched off in Capture One (Settings → Capture → Provider),
   `ptpcamerad` still started ~15 s after Capture One launched and took the cable **~3 s after the plug-in**. Our helper waits a 2.5 s settle after the
   USB hot-plug event before it tries (evidence §Answers, constraint on the hot-plug switch). There is no supported way to stop or seize from `ptpcamerad`
   (no setting; the IOUSBHost capture option needs root or an Apple-granted entitlement; killing it is forbidden and it respawns). Whoever opens the
   interface first keeps it (P47 S1(a): with our helper holding it, `ptpcamerad` could not take it). After this phase the helper starts its USB connect
   **at once** on the hot-plug arrival and retries at short intervals through the old settle window, instead of waiting it out, and, once it holds the
   cable, keeps it until the photographer asks otherwise.
2. **F14: never idle.** At 16:27 the helper (running for 3.5 h) held neither Wi-Fi nor the cable: no SSH socket, no USB interface, while the camera was
   reachable over Wi-Fi and plugged in, its cable held by `ptpcamerad`. After this phase, whenever the camera is reachable on any link, the helper holds a
   session on one of them or is actively trying, without limit in time; a link it cannot get is retried with the transport policy's backoff, never abandoned.
   (P54 fixed the first fallback; this row proves it over a long run of failures.)
3. **F12: the export loop.** Capture One showed an "exporting" window every few seconds. The app issued `find_image` → `recipe_exists` → `process` about every
   0.7 s for hours, re-exporting the same photos (DSC04767, shot in the morning, re-exported at 16:25) into `Output/_session_manager/`, where files named
   `session test 1NNNN.jpg`/`.tif` and leftover `.jpg.tmp` files sit beside `DSC0xxxx.jpg`. The app does not recognise the finished export and orders it again.
   On 2026-11-08 the loop also rendered the **master** (`PSM Master`, 78 MB TIFF) of the same photos again and again into `Output/`, where Capture One adds a counter (`DSC04979 14.tif`): 860 TIFFs, 64 GB in one afternoon, and the disk ran out (167 MB free). This also feeds F7 (take-back blocked by "a preview is rendering") and F11 (slow previews). After this phase a render whose output has been produced is
   recognised as complete and is not re-issued; a render that cannot complete stops after a bounded number of attempts with a health detail; a photo whose
   preview is current is never re-exported. The implementer finds the cause (completion matching by name? output naming from Capture One's variant name?
   `.tmp` rename?) from the code and the evidence, and writes it in the handoff.

**Not in this phase:** preview speed beyond ending the loop (F11, owner card), live view.

## Read first
Evidence §Acceptance (P52) incl. the two new paragraphs; §V6; §Spike S1(a). Book `10E_direct_camera_adapter.md` MC-31 §Transport policy, MC-34; `10A_capture_one_contracts.md`
(render/process contract), `10_imaging_providers.md`. Code: `native/camera-sidecar/src/{transport_manager.cpp, usb_monitor_macos.cpp, camera_connection.cpp}`,
`include/crlab/{transport.hpp, usb_monitor.hpp}`, `tests/test_transport_manager.cpp`; `crates/integrations/src/editing/capture_one/{adapter.rs, scripting.rs, jxa.rs}`
(renders: `jobs`, `pending`, the `variant_not_found` deferral, the spool `processing_done` files, output naming), the render/preview service in `crates/core/src/services/`,
P49/P53/P54 narratives. Tests: p46, p48, p49, p54 testkit files, the fake Capture One (`crates/testkit/src/fake_capture_one.rs`).

## Files expected to change (write perimeter)
The files above as needed, `crates/testkit/tests/p56_camera_claim_and_render_loop.rs` (new), new C++ cases, the fake Capture One only to model the observed
output naming. Existing assertions change only where the contract changes them (list each in the handoff). Estimated ≈ 15 paths, ≈ 600 lines.

## Core mechanisms (unit tests allowed)
The helper's transport policy (C++); render completion matching.

## Criteria (5 criteria, 8 rows)
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | helper: connect at plug-in | C++ transport test: Wi-Fi session idle, USB arrival event at t=0 | the first USB connect attempt starts ≤ 300 ms after the event (no 2.5 s wait); while the interface is not yet openable, attempts repeat at ≤ 500 ms intervals until 2.5 s, then the normal policy | `[rule]` MC-31 §Transport policy; P52 F13 |
| C1(b) | helper: busy still defers | USB arrival during a protected operation | the switch still waits for busy to clear (`switch.deferred`), then attempts at once | `[rule]` R-I22; M39 |
| C2(a) | helper: never idle | C++ test with fake time: USB present, every USB connect fails "held" for 30 simulated minutes; Wi-Fi available | the manager is connected over Wi-Fi within the policy's bound and stays connected; USB is retried on its backoff throughout; when USB succeeds and not busy it switches to USB | `[rule]` MC-31; MC-34 §Guide; P52 F14 |
| C2(b) | helper: never idle, Wi-Fi drops | as C2(a), then the Wi-Fi session is lost | reconnect attempts continue (no terminal idle state); a session again when Wi-Fi returns | `[rule]` MC-31 |
| C3(a) | render completion | fake Capture One produces the export under the name it really uses (model the observed `session test 1NNNN` / variant naming) for a requested photo | the render is recorded complete once; no second `process` for that photo while its preview is current (assert the fake received exactly one `process` over a long fake-time window) | `[rule]` 10A render contract; P52 F12 |
| C3(b) | render that cannot complete | fake Capture One never produces output | `process` is issued at most N times (N stated in the handoff, ≤ 3), then the photo shows a preview failure / health detail and no further `process` | `[rule]` 10A; P52 F12 |
| C3(c) | no re-export of current previews | app restart with all previews current | no `process` calls at startup | `[rule]` 10A; P52 F12 |
| C3(d) | master renders do not pile up | the same photo's master (`PSM Master`, TIFF) requested again while its master is current / while the disk has less than 5 GB free | no second `process` with the master recipe while the master is current; below 5 GB free no master render is issued and a health/header warning names the disk (the photographer's existing exports are never deleted by the app) | `[rule]` 10A; P52 F12 (64 GB of ` N.tif` copies in one afternoon) |

Plus the closing gate row: `cargo test --workspace`, clippy, fmt, `npm run typecheck`, `npm test` (known intermittent `p33_split_grid` aside), `tools/sidecar/build.sh --stub --test` green.

## Delegated choices
Retry interval inside the settle window (≤ 500 ms); N for C3(b); how completion is matched (by spool event, by output path, by variant id).

## Manual try-out
P52 run 2: plug in with Capture One running → our helper holds the cable (`UsbExclusiveOwner = camera-sidecar`), dot `Cable`, no take-back needed; leave
the app for an hour → the helper is on a link the whole time; Capture One shows no repeating export window.

## Notes
- **Hard rules (owner):** never commit a Sony SDK file, a `.env`, a credential or an SSH fingerprint; no Python; never change the camera's network/FTP settings,
  delete camera files (or exported previews) or kill/quit any process automatically; live view out of scope; the suite never needs the camera or Capture One.
  Lab repo read-only. Default implementer. If the SDK is present locally, prove the SDK build compiles, then restore the stub; never commit build output.
- **Hazard:** the 2.5 s settle exists because the SDK reports a lost USB session late (~17 s) and the switch is driven by the hot-plug event (evidence
  §Constraints); an early attempt may fail while the camera's interface is still coming up. Failed early attempts must not count towards
  `usb_failures_before_wifi_fallback` or drop the Wi-Fi session before the USB session is ready.
- Book: MC-31's "≤ ~8 s incl. the 2.5 s settle" stays an upper bound; this phase makes the plug-in path faster, not slower.

## Review log

- 2026-11-08 implement r1 (codex): **BLOCKED before production edits.** C1(a) requires a USB connect attempt within 300 ms while a Wi-Fi SDK session is open, but the Notes prohibit dropping that session before USB is ready and MC-31 permits only one SDK session. C2(a) requires USB retries throughout 30 minutes of held-interface failures and switching when a later attempt succeeds, while ratified MC-31 §Transport policy says that after three failed USB connects the sidecar stays on Wi-Fi until re-plug or holder-gone signal, with no periodic USB probing. Both are observable policy contradictions; the implementer cannot implement either rule verbatim without violating the other. Read-only diagnosis for F12: the real database has 170 completed preview jobs for one RAW and its adjustment generation is 173, but it has zero `variant_adjusted` hook rows for that RAW; `PreviewService::check_sidecars` is the remaining generation-raising path and treats a later `.cos` mtime as an edit, while Capture One processing can change that sidecar. This is a likely self-trigger loop, not yet proved by a boundary test. No code, test, tracker or owner-data file was changed. Two owner cards in the handoff request a contract amendment; no suite stamp was run because no implementation tree exists to close.
