---
plan: live-fixes
role: plan
round: 1
state: DONE
date: 2026-11-26
actor: opus (Claude Opus 5.5, pipeline-planner)
---
# Handoff — plan the live preview fixes (LV5, LV6) from the paused hardware check

**Summary.** Gate passed: book 10J is RATIFIED (D18) and LV-D19 is an owner answer recorded in changelog D19. I traced
every finding to its cause in the code and in the app log, then wrote two milestone plans. **LV5_live_fixes**
(default implementer) covers F4, F3, the server half of F7, F2, the Settings → Phone list, F5 and F1: 4 units,
12 criteria, 49 rows. **LV6_phone_dock** (`implementer: fable`) covers the LV-D19 dock, the phone half of F7 and the
removed text: 2 units, 6 criteria, 13 rows. `master_plan_live.md` gets dated v1.1 amendments: CS4a, FC4a, HB5a, PH5a,
PH6a and PH8, plus §2/§3/§4.3/§4.6/§4.7/§6/§7/§9. In `phases_live.txt`, LV5 and LV6 now come before LV4. No code was
changed and nothing was committed.

## ⚠ OWNER DECISIONS REQUIRED (0)
None. Nothing needs the owner. The plans include five "choices made for you" (dock tile size, what counts as a tap,
which Focus Areas count as movable, how long a Wi-Fi stall is tolerated, how the Settings list refreshes). Each goes
to the narratives so the owner can change it by saying so.

## Write perimeter (complete)
| File | Change |
|---|---|
| `docs/implementation/plans/LV5_live_fixes.md` | new (Tracker `NOT_STARTED`, empty Review log) |
| `docs/implementation/plans/LV6_phone_dock.md` | new (Tracker `NOT_STARTED`, empty Review log) |
| `docs/implementation/master_plan_live.md` | v1.1 amendments, each marked in place; replaced text kept and marked superseded |
| `docs/implementation/phases_live.txt` | order `LV1, LV2, LV3, LV5, LV6, LV4`; header comments |
| `docs/implementation/handoffs/reviewer/handoff_live_fix_plan_r1.md` | this file |
Not touched: code, the book, `STATUS.md`, `STATUS_live.md`, other roles' prompts/handoffs, the LV1–LV4 plan files
(including LV4's `depends_on`, see Notes). No archgraph in this repo. No commit.

## Counts (derived by `grep` from the plan files)
| Milestone | Units | Criteria | Rows | Max criteria per unit |
|---|---|---|---|---|
| LV5_live_fixes | 4 | 12 | 49 (U1 9, U2 18, U3 6, U4 16) | 4 |
| LV6_phone_dock | 2 | 6 | 13 (U1 6, U2 7) | 3 |
Master plan §2 totals recomputed: 6 milestones, 22 units, 113 criteria, 329 rows. The v1 figures (15/91/261) had
missed LV3's U0. LV3's row is corrected to 5/29/85, the figures its plan file states.

## The two milestones in plain words
- **LV5: fixes in the camera helper, the app's core and the phone server.**
  - The next shot never points at a shot whose photo is still arriving.
  - Every Mac review action shows on the phones at once: Deny, Main/Condition, restore, reorder, move, copy, Keep as
    published, Undo/Redo, drop on a container, Back to template, Shift back.
  - A Wi-Fi stall makes the phone skip live frames instead of dropping its connection. The Mac gives up only after
    15 s with nothing getting through.
  - With the camera's Focus Area on Wide (or another non-movable area), a tap gets the "set a movable Focus Area"
    message. The camera is never changed.
  - The camera's exposure (shutter · aperture · ISO) reaches the phone.
  - The session view tells the phone when a photo's preview is ready.
  - Settings → Phone shows which phone is connected now.
  - *Owner can try after it:* fast shooting from the phone with Next watched on the Mac; Mac actions showing on the
    phones; walking to the edge of the Wi-Fi on live; Focus Area Wide; the exposure readout; the Settings list. The
    dock still has the old gestures, and a tile can still show a broken image until LV6.
- **LV6: the phone's dock as the owner asked (LV-D19).**
  - Bigger tiles (56 px) that scroll sideways.
  - A single tap opens a taken shot's preview or makes an empty shot current. No hold-to-magnify, no drag-up.
  - A photo's tile shows a grey numbered square until its preview is ready, then the picture. It is never a broken
    image.
  - A removed phone is told to delete the app and add it again from the new QR code.
  - *Owner can try after it:* the dock (tap, scroll, holding does nothing special), new photos filling in, Remove and
    reinstall. Then LV4 resumes.

## Findings → cause verified → where planned
| Finding | Cause found (evidence) | Contract | Rows |
|---|---|---|---|
| F4 | Two problems combine. `hold_times` (`crates/infra/src/sqlite/store.rs:915`) drops an exposure as soon as it has a delivery, at filing time. The photo row appears only when the arrival scan takes the file in. In that gap the container counts as empty. Log: `ftp.upload filed name=DSC07532.ARW` 14:07:01.78 → exposure 61 → `set container=10 by=advance` 14:07:02.224 → photo row 14:07:02.82 (`filed_to_row_ms=1058`). Same pattern at 14:07:07.375. | CS4a | LV5·C1–C4 |
| F3 | `ReviewService::{classify, reorder, delete, keep_published, move_photos, copy_photos}` emit nothing. `HistoryService::reverse` emits `PhotoChanged`/`SessionChanged` only when `changed_photos` is filled, which only Composition and Template steps do (Classify does not). Template hand actions already emit through `HandAction::finish`. | PH8 | LV5·C5–C7 |
| F7 (server) | `previews.rs` already emits `PhotoChanged` when a preview is written. The phone's URL carries only the composition `revision`, so it never changes when the preview appears (31 × 404). | PH6a | LV5·C8 |
| F2 | `ws.rs` `serve` puts every write inside `timeout(write_timeout = 2 s)` and closes 4408 on a slow frame write. While a write is awaited, revoke and reader-closed go unserved. | PH5a | LV5·C9 |
| Settings list | `PhoneSection.tsx` reads `phonePairingsList` only on mount and after its own actions. | — (UI) | LV5·C10 |
| F5 | The focus job refuses only when `!af_position_writable` (`live_runner.cpp` `focus()`). Under Wide the a7R V reports the position as writable: `point_ms=-1`, then AF "focused" (14:13:50–14:14:09). The idle focus frames are empty under Wide (`frames=[]` 14:13:40). The Focus Area value is already read by `focus_state()`; enum values are at `CrDeviceProperty.h:1456–1485`. | FC4a | LV5·C11, C12(f) |
| F1 | The helper sends each value as the `props` object (`display` inside). `protocol.rs` uses `req_str` on all three fields → "unparsable" (exact line at 14:06:58.216844Z). The display formats come from `camera_properties.cpp` `decode`. | HB5a | LV5·C12(a–e) |
| LV-D19 (1) | `Dock.tsx` holds/magnify/drag-up; `gestures.ts` `tileScale`. | 10J LV-D19 | LV6·C1–C3 |
| F7 (phone) | `api.ts` `previewUrl` has `&r=<revision>` only and no retry. | PH6a | LV6·C4–C5 |
| LV-D19 (2) | `state.ts` `TEXT.removed` serves both the 401 and the socket's 4401. | §6 (amended) | LV6·C6 |

## Notes for the orchestrator
- **A rule-17 slip from LV1, now corrected:** `lv1_frame_channel.rs::lv1_c15b_exposure_mapping` fed the hub an invented
  exposure shape (`"8.0"`, `"100"`). The real helper never sends that shape, which is why F1 passed the suite and
  failed on hardware. LV5·C12(b) replaces its input with the real line. Plan lesson: an external-shape fixture is
  taken from a captured line, never from the registry's prose. §4.1 said "(the decoded strings `props` already
  produces)", and that was wrong; it is now marked superseded.
- **LV4's plan file** still says `depends_on: [LV3_phone_app]`. I left it unchanged because it was outside my
  perimeter. Master plan §9 records that LV4 resumes after LV6, and which rows to re-run (C18, the exposure readout,
  the stall rows, the dock look). Update it when resuming.
- **Not folded (outside these files):** the open LV1/LV2 backlog carry-overs (`frames.rs` accept errors, fake-sidecar
  frame connect fidelity, C13(e), SIGINT on diagnostic commands, dead `previous_connections`, revoke command tested in
  halves). LV5 does not touch `frames.rs`, the fake sidecar's frame code, or `main.cpp`. Also not planned: the
  sidecar's once-a-second `liveview.exposure` emission caused by the flipping `writable` flag. It is log noise (the
  phone's `camera` message goes out only when the readout changes) and is noted in LV5·U4.
- **Settled choices** (also listed as "Points recorded by planning" in each plan):
  - Movable areas = the Flexible Spot family, with or without tracking. Zone counts as not movable, the conservative
    reading of 10J's "Flexible Spot / Tracking".
  - `stall_close` defaults to 15 s (the check's stalls lasted up to 8 s).
  - Dock tiles are 56 px with an 8 px gap; the strip is 84 px.
  - A tap is any touch that moved ≤ 8 px, however long it is held.
  - The image retries 3 times, 2 s apart.
  - The Settings list re-reads every 5 s.
  - A missing exposure value shows as "–".
- **F6** (an installed app is dead after re-pairing) needs no code: the owner accepted reinstalling, and LV6·C6 gives
  the text that says so.
- **Possible gap, still with the owner** (STATUS_live): the phone does not draw the camera's own focus frame. It is
  not planned here, and no card was raised because it was already listed under "Needs the owner".
- Projection stays off for both milestones (reason in master plan §3 v1.1). The two externally owned shapes are
  grounded: the quoted log line and the installed SDK enum.
