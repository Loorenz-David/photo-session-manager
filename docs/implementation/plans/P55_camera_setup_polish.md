---
phase: P55_camera_setup_polish
title: Camera setup after P52 — "Add camera support" accepts Sony's download as it comes and says "missing" vs "changed" correctly, the setup steps include Auto FTP, the app has a Take photo button, and a crashed helper restarts at once
difficulty: normal
projection: no
depends_on: [P54_camera_cable_and_take_back]
---
# P55 — Setup polish (P52 findings F1, F4, F5; Auto FTP step)

Written by the orchestrator on 2026-11-08 from P52 run 1 (evidence `docs/intention/evidence/sony_camera_lab.md` §Acceptance (P52, 2026-11-08), rows A0, A3,
A5, A12).

## Goal
1. **F1: Sony's download as it comes.** The owner picked `CrSDK_v2.02.00_…_Mac.zip` and got "missing or changed file CrAdapter/libCr_PTP_IP.dylib". The seven
   runtime files are inside a nested zip (`RemoteCli.zip`, under `external/crsdk/`); `support/manifest.rs::verify` reads only top-level entries. After this
   phase the installer also looks inside nested `.zip` entries (one level), matching by path suffix as today; the error says **missing** when a file is
   absent and **changed** only when it is present with another hash.
2. **Auto FTP in the setup steps.** P52 A5 failed at first because the camera's *Auto FTP Trnsfr* was off. The camera setup steps shown in Settings → Camera
   include it (menu path from the lab's `camlab/experiments/camera_steps.py`, read-only).
3. **F4: Take photo.** 10E §What the photographer gets and A3 assume a capture button; there is none. The camera popover (the header dot) gets a
   **Take photo** button, enabled only while connected, that sends the helper's capture command and shows the error sentence on failure.
4. **F5: restart at once.** After the helper was killed, a new one started after 2 s and Wi-Fi was up after 5 s (target ~1 s). The first restart after an
   unexpected exit happens without delay; the existing backoff applies from the second consecutive failure.

**Not in this phase:** P53/P54 items, preview speed (F11).

## Read first
Book `10E_direct_camera_adapter.md` §What the photographer gets, MC-31 §Capture and §Process, §Support package (install, manifest); evidence §Acceptance (P52).
Code: `crates/integrations/src/camera/support/{manifest.rs, mod.rs, sony.rs}`, `native/camera-sidecar/manifest/sony-2.02.00.json`,
`crates/integrations/src/camera/sidecar/supervisor.rs`, `crates/integrations/src/camera/direct/adapter.rs` (capture command), `apps/desktop/src-tauri/src/commands/camera.rs`,
`apps/desktop/src/features/settings/CameraSection.tsx`, the header camera popover (find by `Take the camera back`), lab `camlab/experiments/camera_steps.py`.
Tests: `crates/testkit/tests/{p50_camera_settings_and_support.rs, p46_direct_camera_adapter.rs}`, p51 UI tests.

## Files expected to change (write perimeter)
The files above as needed, `crates/testkit/tests/p55_camera_setup_polish.rs` (new), UI tests for the popover and the setup steps, generated bindings.
Test archives are built in the test from made-up bytes (never Sony files). Estimated ≈ 14 paths, ≈ 450 lines.

## Core mechanisms (unit tests allowed)
`manifest::verify` (entry matching across a nested zip).

## Criteria (5 criteria, 7 rows)
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | install from a nested archive | a test zip whose only content is `RemoteCli.zip` containing the manifest's files (made-up bytes, test manifest) under `external/crsdk/` | installed; the seven files in the support folder with the expected hashes | `[rule]` 10E §Support package; P52 A0 |
| C1(b) | error wording | archive without `CrAdapter/libCr_PTP_IP.dylib` / with it but other bytes | error names the file with `missing` / with `changed` (exact sentences in the handoff; the UI shows them) | `[rule]` 10E §Support package |
| C2(a) | setup steps | Settings → Camera setup steps rendered | they include the Auto FTP step (`Auto FTP Trnsfr` → `On`) with its menu path | `[rule]` 10E §What the photographer gets; P52 A5 |
| C3(a) | Take photo | connected status; click Take photo | the camera capture command is called once; the helper receives the capture line (testkit fake); disabled when not connected | `[rule]` MC-31 §Capture; P52 A3 |
| C3(b) | Take photo failure | the command rejects | the error sentence is shown (toast) | `[rule]` MC-31 §Capture |
| C4(a) | first restart at once | the fake helper exits unexpectedly once | a new helper process within 300 ms; a second consecutive exit uses the existing backoff | `[rule]` MC-31 §Process; P52 A12 |
| C5(a) | existing suites | everything else | `cargo test --workspace`, `npm test` (known intermittent `p33_split_grid` aside), clippy, fmt, typecheck, `tools/sidecar/build.sh --stub --test` green | `[rule]` master plan §7.2 |

## Delegated choices
Nested-zip depth (one level is enough); whether the steps' Auto FTP line is a new step or part of the FTP step; the Take photo button's exact position in the
popover; the restart's "unexpected exit" definition (not a quit the app asked for).

## Manual try-out
P52 run 2: install support from Sony's download as it comes; Take photo over Wi-Fi and on the cable (A3); A12 again.

## Notes
- **Hard rules (owner):** never commit a Sony SDK file (test archives are made-up bytes), a `.env`, a credential or an SSH fingerprint; no Python; never change
  the camera's network/FTP settings, delete camera files or kill any process (tests kill only the fake helper they started); live view out of scope; the suite
  never needs the camera. Lab repo read-only. Default implementer.
- **Hazard:** reading a nested zip must not unpack everything to disk or memory beyond the matched files; keep the refusal of `..` names for nested entries.

## Review log
