---
phase: LV4_hardware_check
title: Hardware check (owner-attended, camera and phones needed) — live view on the phone over USB and Wi-Fi, no stale frame on a switch, several phones, reconnect after sleep, iPhone and Android install; captures from phone, Mac and camera button land in the right shot even when files arrive late; retake; Condition after the last shot; tap-to-focus and sharp photos; × recentres; no camera setting changed; clean stop; frame rate and delay measured and targets set
difficulty: normal
projection: no
depends_on: [LV3_phone_app]
blocked_on: none
---
# LV4 — Hardware check (book 10J M-LV1…M-LV6; owner-attended)

**Tracker:** NOT_STARTED (2026-11-26, planner)

(This line is the milestone's tracker row. Sessions change only this line of this file.)

## Goal
Run the checks below on the **packaged app** built from this branch, with the a7R V (owner's settings: Manual exposure,
AF-S, Flexible Spot S), Capture One as usual, the studio Wi-Fi, the owner's iPhone and Android phone. Every row's
measurement is written into the evidence files (below); a failed row becomes a finding for the orchestrator (a fix
milestone or an owner card), never a silent pass. The owner sets the **M-LV6 targets** at this check from the measured
values.
**Not in this milestone:** code (findings route to later milestones), HTTPS, the Mac's live screen.

## How this milestone runs
Owner-attended, as P52 did: the orchestrator and the owner run it together; `run_phase.sh` must not dispatch it.
Three units of rows (≤ 8 each) only to keep the record readable; one Review log entry per run.

## Read first
Book `10J` (§The working loop, M-LV1…M-LV6, LV-S1…LV-S6, §Not covered) · evidence
`docs/intention/evidence/sony_camera_lab.md` §Live view — Experiment 07, §Focus from the live picture — Experiment 10 ·
`docs/intention/evidence/live_preview_lab.md` (Experiments 08, 09, 11; §Not covered yet) · the lab procedures
(read-only) `~/Desktop/Developer/Camera-Connection-Tests/experiments/{07-live-view.md, 07-clock.html, 08-phone-app.md,
10-focus-from-live.md, 11-wifi-phones.md}` · the narratives of LV1–LV3 · `master_plan_live.md` §4.6 (log lines), §8.

## Files expected to change
`docs/intention/evidence/live_preview_lab.md` (new section `## Hardware check (LV4, <date>)`, one table row per check with
measured values and pass/fail) · `docs/intention/evidence/sony_camera_lab.md` (new section `## Live view in the app (LV4,
<date>)` for the camera-side rows LV4·C9, C15–C19) · this plan's Review log (summary, findings, the owner's M-LV6
targets) · `docs/implementation/backlog.md` (notes) · `docs/implementation/STATUS_live.md` (by the orchestrator). **No
code.**

## Core mechanisms
None.

## Ordered tasks (the orchestrator with the owner)
0. **If launched by `run_phase.sh`:** report `STATE: BLOCKED`, reason `owner-attended phase (LV4): run by the
   orchestrator with the owner`, change nothing.
1. **Build:** `tools/sidecar/build.sh --sdk <lab>/vendor/sony-sdk`; `cd apps/desktop && npm ci && npm run tauri --
   build --bundles app`; check `Session Manager.app/Contents/Resources/phone-web/phone.html` exists (master plan §8);
   install in `/Applications` the owner's usual way; start with `PSM_LOG=debug`. Camera support installed (Settings →
   Camera, all green).
2. **Set up:** a session whose item type has a shooting template of at least 4 containers with a shooting order
   different from the final order and one split container; Settings → Phone → pair the iPhone and the Android phone
   (names "iPhone", "Android"). Open the lab's clock page (`07-clock.html`) on the Mac for the delay rows.
3. Run the rows in order, each at most twice (a flaky second run is a finding too). Read values from the app log lines
   (`live.source`, `live.viewer`, `focus.result`, `current_shot.bound`, `phone.viewer`), the helper log on the Health
   page, the session view, the camera's menu and screen.
4. Record every row; findings to the Review log with the row and the observed value; the orchestrator routes them.

## Criteria (20 criteria, 20 rows; all `[manual]`)
"Pass" values come from 10J and the lab evidence; the measured value is recorded whatever it is.

### Unit 1 — Live delivery (LV4·C1–C8)
| Row | Check | Pass | Trace |
|---|---|---|---|
| LV4·C1(a) | Live on the iPhone over **USB** (camera on the cable): hold, then lock, 60 s; film the clock page through the camera | live appears on hold; `live.viewer … sent_10s` ≈ 250 (25 fps); glass → phone delay recorded (Experiment 08: ≈ 145 ms) | `[manual]` M-LV6, M-LV1 |
| LV4·C2(a) | Live on the Android phone over **Wi-Fi** (cable out), 60 s locked; one Auto FTP RAW upload during it | ≈ 20 fps outside the upload, the dip during it (lab: ~13 fps) shown without the phone calling live "stalled"; delay recorded | `[manual]` M-LV6, M-LV1, 10J §Shared Wi-Fi |
| LV4·C3(a) | **No stale frame on a switch:** template view 20 s, then hold, filming the clock | the first picture shown after the press shows a clock time later than the press (never the picture from the previous peek) | `[manual]` M-LV2 |
| LV4·C4(a) | **Not on Live → nothing sent:** both phones on the template view 30 s | `live.viewer … live=false sent_10s=0` for both, three lines each | `[manual]` M-LV2, MC-LV2 |
| LV4·C5(a) | **Several viewers:** iPhone and Android both locked on live, 60 s; then walk the Android to the edge of the Wi-Fi for 20 s | each near the camera's rate while both are close; the far phone skips frames (`skipped_10s` rises) while the near phone's `sent_10s` does not drop | `[manual]` M-LV1, LV-D12, 10J (several phones at acceptance) |
| LV4·C6(a) | **Reconnect after sleep:** lock the iPhone 60 s while on live, unlock | live (or the template) is back within 3 s without a reload; the log shows `phone.viewer closed` then `opened` | `[manual]` MC-LV2, PH7, 10J (reconnect after sleep) |
| LV4·C7(a) | **iPhone reachability and install:** open by the `.local` QR; if it fails, by the address QR; Add to Home Screen; open from the home screen | which address worked recorded; the home-screen app opens **full screen and paired** (no QR again) | `[manual]` LV-D7, PH7, 10J (iPhone reachability at acceptance) |
| LV4·C8(a) | **Android:** home-screen shortcut, then **Full screen** | browser bars hidden after one tap; the button is gone while full screen | `[manual]` LV-D7, card LV3 |

### Unit 2 — Capture and the current shot (LV4·C9–C14)
| Row | Check | Pass | Trace |
|---|---|---|---|
| LV4·C9(a) | **Capture from the phone, USB, live on:** 4 shots | each yields exactly one photo in the active session, in the shot that was current; button → `capture.exposed` time recorded (lab 0.40–0.77 s); live freezes ≤ ~0.5 s and resumes | `[manual]` M-LV3 |
| LV4·C10(a) | **Wi-Fi, mixed buttons, late files:** 8 shots quickly: phone, Mac, the camera's own button, alternating | every exposure's photo lands in the container that was current at its shutter (`current_shot.bound` lines vs the session view), even those whose file arrived after later shots; no duplicate, none lost | `[manual]` M-LV4, M-LV3, MC-LV3 |
| LV4·C11(a) | **Tap a shot:** tap an empty later shot on the phone, shoot with the camera button | the photo lands in the tapped shot; the Mac's "Next" outline showed it before the shot | `[manual]` M-LV4, card LV5 |
| LV4·C12(a) | **Retake:** Retake a filled shot on the phone; check the Mac; undo on the Mac; Retake again; shoot | the old photo turns red at once on the Mac; undo brings it back; after the second Retake the new photo fills that shot with its presets and the current shot moves on | `[manual]` M-LV4, card LV6 |
| LV4·C13(a) | **After the last shot:** fill every container, shoot 2 more | both are Condition, appended; the phone's dock shows them as C1, C2 | `[manual]` M-LV4, card LV7 |
| LV4·C14(a) | **Revoke:** remove the Android in Settings → Phone while it shows live | its live stops within 1 s and it shows the "removed" text; the iPhone is unaffected | `[manual]` PH1, LV-D12 |

### Unit 3 — Focus, camera safety, clean stop, targets (LV4·C15–C20)
| Row | Check | Pass | Trace |
|---|---|---|---|
| LV4·C15(a) | **Tap-to-focus over Wi-Fi from the phone:** 10 taps alternating near / far objects | focused ≥ 9/10; tap → focused recorded (lab 0.38–0.83 s); the focus frame follows the taps on the phone and on the camera's screen | `[manual]` LV-D14, 10J (Wi-Fi focus at acceptance) |
| LV4·C16(a) | **Sharp where tapped:** tap the near object, shoot; tap the far object, shoot | at 100 % the tapped object is sharp and the other blurred in each photo | `[manual]` LV-D14, FC5, 10J (photo sharp where tapped) |
| LV4·C17(a) | **× recentres:** after a tap, press × | the camera's focus point is back at the centre (camera screen); the phone shows no AF chip | `[manual]` card LV9 |
| LV4·C18(a) | **Focus area not movable:** set the camera's Focus Area to Wide (owner, on the camera), tap on the phone; then set it back | the phone shows "Set the camera's Focus Area to a movable spot …"; the camera's Focus Area is still Wide afterwards (the app changed nothing) | `[manual]` LV-D14, FC4, FC6 |
| LV4·C19(a) | **No capture setting changed (M-LV5):** a photo with live view on and one after quitting live (app closed, camera alone), same scene | file type, quality, store destination and exposure mode identical (camera menu, EXIF); the helper log shows no property write other than the store destination and `AF_Area_Position` | `[manual]` M-LV5, FC6, LV-D11 |
| LV4·C20(a) | **Clean stop and M-LV6 targets:** quit the app during live view on USB; restart; also pull and re-plug the cable during live view | the next connect succeeds first time (no half-open session, Experiment 10); live resumes after the re-plug; then the owner sets the targets for USB/Wi-Fi × Mac→phone frame rate and glass→phone delay from C1/C2, written in the Review log | `[manual]` FC7, M-LV6, 10E MC-31 |

## Delegated choices
Row order within a unit (C14 and C20 last in their units); which objects are "near" and "far" (the lab used a tape
measure and the table); whether C10 uses 8 or more shots.

## Manual try-out
This milestone **is** the try-out; the narrative is the evidence section in plain words plus the list of findings and
the owner's M-LV6 targets.

## Notes
- **Hard rules (owner):** never change the camera's network, FTP or exposure settings from the app; the owner changes
  the Focus Area by hand for C18 and sets it back; no camera file is deleted; no process other than our own helper is
  killed (none is needed here); nothing of the Sony SDK goes into the repo.
- If the iPhone cannot reach the Mac by name (Experiment 11), the address QR is the fallback; record which worked.
- Findings: a failed row → a fix milestone planned from the row and the observed value (the usual route), or an owner
  card when the fix is a product choice.

## Review log
