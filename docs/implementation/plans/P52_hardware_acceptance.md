---
phase: P52_hardware_acceptance
title: Hardware acceptance (owner-attended, camera needed) — the 10E acceptance table on the packaged build with the support package installed through the app: Wi-Fi and USB connect, capture, RAW over USB and by Auto FTP, switching, never two sessions, receiver restart, cut upload, sidecar killed, another program holds the camera, Capture One sees new photos, no re-send, settings save on USB, Take the camera back
difficulty: normal
projection: no
depends_on: [P51_camera_setup_ui]
---
# P52 — Hardware acceptance (10E §Hardware acceptance; owner-attended)

## Goal
Run 10E's acceptance table (18 rows, lab handoff §10 adapted plus the inventory's three rows) on the **packaged build** with the Sony support package
installed **through the app** (P50/P51), the FTP user and password entered on the camera by the owner, and Capture One running — on the dev Mac first;
the work Mac follows when the owner moves (the Capture One camera adapter is retired only after that, by a later tidy-up). Every row's measurement is
written into `evidence/sony_camera_lab.md §Acceptance (P52, <date>)`; a failed row becomes a finding for the orchestrator (a fix phase or an owner card),
never a silent pass.
**Not in this phase:** code (findings route to later phases), FTPS, live view, a second camera.

## Read first
Book: `10E_direct_camera_adapter.md` **§Hardware acceptance** (the table), §What the photographer gets, MC-31…MC-34, §Invariants M37–M40 · `14_measurement_
ledger.md` M37–M40 · evidence `sony_camera_lab.md` (§Answers for the expected timings; §V6; **§Spike (P47)**) · master plan §3 (`[manual]` rows; camera hard
rules), §7.1 (camera row), §7.2 (SDK build, app build) · the narratives of P42–P51 (what the owner can already do) · lab `docs/session-manager-handoff.md`
§10 (the original expected values) and `camlab/experiments/camera_steps.py` (menu paths, read-only).

## Files expected to change
`docs/intention/evidence/sony_camera_lab.md` (§Acceptance: one row per test with the measured values and pass/fail), this plan's Review log,
`docs/implementation/backlog.md` (findings that are notes), `docs/implementation/STATUS.md` (by the orchestrator). **No code.**

## Core mechanisms
None.

## Ordered tasks (the orchestrator with the owner)
0. **If launched by `run_phase.sh`:** report `STATE: BLOCKED`, reason `owner-attended phase (P52): run by the orchestrator with the owner`, change nothing.
1. Build: `tools/sidecar/build.sh --sdk <lab>/vendor/sony-sdk`, then `npm run tauri -- build --bundles app`; install the `.app` in `/Applications`
   (the owner's usual way). Start it. In Settings → Camera: **Add camera support** from the owner's SDK archive; **Generate password**; the owner enters
   user `sessionmanager` and the password on the camera (menu steps on screen); enter the Access Authentication login; the first Wi-Fi connect shows the
   fingerprint card — compare and confirm. Health: every camera row green. (This replaces P47's hand install; if the P47 folder is still there, delete it
   first — it is ours, not the camera's.)
2. Run the table below in order, each row at most twice (a flaky second run is a finding too). Time with the app's log (`PSM_LOG=debug`) and the helper
   log on the Health page; count photos in the session view and files in the capture folder (`ls` only).
3. Record every row in the evidence; findings to the Review log with the failing row and the observed value; the orchestrator decides routes.

## Criteria (18 criteria, 18 rows; all `[manual]`)
"Pass" values are 10E's; the measured value is recorded whatever it is.

| Row | Test | Pass | Trace |
|---|---|---|---|
| A1 | Wi-Fi connect (SSH, pinned fingerprint): camera on, cable out, app started | `Connected · Wi-Fi` ≤ ~6 s from start; no auth retry | `[manual]` 10E §Hardware acceptance; MC-31 §Fingerprint |
| A2 | USB connect: cable in at start | `Connected · Cable` ≤ ~4 s from start (≤ ~8 s incl. the 2.5 s settle on a plug-in) | `[manual]` 10E; MC-31 §Transport policy |
| A3 | Capture from the app's button | `capture.acknowledged` ≤ 50 ms; `capture.exposed` ≤ ~0.6 s after the command (AF lock) | `[manual]` 10E; MC-31 §Capture |
| A4 | RAW over USB (Save-to-PC) | file on the Mac ≤ ~2.5 s after the command, intact (ARW header, SHA-256 equals the card copy read later), **one photo in the right session**; the FTP copy lands in `.duplicates/` and is byte-identical | `[manual]` M37, M38; R-I20; R-I27 |
| A5 | RAW by Auto FTP during a Wi-Fi session (10 shots, some on the camera's own shutter) | every exposure arrives during the session, ~4–4.5 s after exposure, intact, one photo each in the right session; camera-initiated shots included | `[manual]` M37, M38; MC-32; MC-33 |
| A6 | Cable in during a Wi-Fi session | USB session ≤ ~8 s after the cable event, no camera interaction; the dot reads `Cable` | `[manual]` 10E; MC-31 §Transport policy |
| A7 | Cable out during a USB session | Wi-Fi session ≤ ~6 s after the cable event | `[manual]` 10E; R-I23 |
| A8 | Plug-in during a protected operation (a capture in progress) | switch deferred (`switch.deferred` in the helper log); done within ~1.5 s after busy clears | `[manual]` 10E; R-I22; M39 ("no transport switch starts while the camera is busy") |
| A9 | Never two sessions | over the whole run the helper log never shows a second `connection.ready` without a `session.closed`/`session.lost` between; `ps` shows one `camera-sidecar` throughout | `[manual]` M39 |
| A10 | FTP receiver restarted (change the receiver port in Settings during a Wi-Fi session, then back) | the camera re-sends the pending shot within seconds; one photo | `[manual]` 10E; R-I18; MC-32 |
| A11 | Upload cut mid-transfer (switch Wi-Fi off on the Mac for 2 s during a transfer) | recorded failed (`426` in the receiver log); the full file is re-sent; no partial file visible under a candidate name; one photo | `[manual]` M37 (F-1); MC-32 §1–2 |
| A12 | Sidecar killed (`kill <pid of camera-sidecar>` by the orchestrator — **the only kill in the whole programme, of our own helper, for this test**) | the restarted sidecar reconnects in ~1 s, no `SessionAlreadyOpened`; the dot recovers | `[manual]` 10E; MC-31 §Process |
| A13 | Another program holds the USB connection (Imaging Edge Desktop opened first, then the camera plugged in) | the header names it with the fix; shooting continues over Wi-Fi (A5 style); USB resumes once it is quit and the cable re-plugged | `[manual]` MC-34 §Detect, §Guide; M39 |
| A14 | New photo while Capture One shows another collection | Capture One jumps to the session folder (R46), shows the photo, and the adjusted preview arrives in the app | `[manual]` 10E §Capture One alongside; R46; R-I35; M24 |
| A15 | No FTP re-send of a completed file: a 20-shot Wi-Fi session | FTP deliveries = exposures; no `_1` names; nothing new in `.duplicates/` | `[manual]` 10E (inventory row); R-I24 §Namespace |
| A16 | Settings saved while on USB with Capture One running (change the SKU prefix) | USB session unbroken (no `session.closed`); the receiver still accepts uploads (a Wi-Fi-less check: `nc -z <mac> 2121`); one sidecar process | `[manual]` R-I18; M39 |
| A17 | Take the camera back: cable re-plug with Capture One running (the system camera service wins, V6) | the header shows the Capture One sentence; **one click**; USB session within ~20 s; Capture One back with its document and the session folder; opening Health meanwhile does not relaunch Capture One early; no photo lost or doubled | `[manual]` MC-34 §Take it back; R-I32; M39, M40 |
| A18 | Store destination | after A2 the camera menu shows `Still Img. Save Dest. = Host PC + Memory Card`; after A7 (Wi-Fi) `Memory Card`; after quitting the app on USB `Memory Card`; the app never set `Host PC` alone (menu read by the owner each time) | `[manual]` R-I19; M39 ("every store-destination write is verified, and none is Host PC alone") |

## Delegated choices
The order of rows (A13 and A17 last, since they involve other programs); whether A11 cuts Wi-Fi on the Mac or on the camera (the Mac is safer: the camera's
settings are untouched); how the orchestrator reads the card copy's SHA-256 for A4 (`shasum` on a copy made from the card reader — never from the camera's
card over the app).

## Manual try-out
This phase **is** the try-out; the narrative is the evidence section in plain words plus the list of findings.

## Notes
- **Hard rules (owner):** nothing of the Sony SDK, the fingerprints or the Keychain goes into the repo; no Python; the camera's network/FTP settings are
  changed only by the owner's hand (step 1, once); **no camera file is deleted** (the `.duplicates/` folder stays; the card is never formatted); no process
  other than our own helper (A12) is killed, and Capture One is quit only through "Take the camera back" on the owner's click (A17); live view out of scope.
- **Owner-attended.** `run_phase.sh` must not dispatch this phase (task 0). After it passes on the work Mac, a later tidy-up retires `CameraViaCaptureOne`
  (10E §Port changes) — not part of this phase.
- Findings: a failed row → `CHANGES_REQUESTED`-equivalent for the orchestrator: a fix phase planned from the row and the observed value (the usual route), or
  an owner card when the fix is a product choice.

## Review log

