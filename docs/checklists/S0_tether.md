# S0 — Tether proof of concept (photographer, no code)

Goal: 20 shots over USB-C into Capture One with 0 lost, and the shot-to-preview time measured.

## Setup
1. Quit Sony Imaging Edge and any other camera app. They block the tether.
2. Camera: ZV-E10 → Menu → Setup → USB Connection = **PC Remote**. USB-C cable to the Mac.
3. Capture One Pro → File → New Session… Name it `Studio`. Note the folder path:
   `__________________________________________` (needed in `.env` as `CO_SESSION_PATH`).
4. Capture tool tab → Camera tool should show "Sony ZV-E10". If not: unplug, replug, restart Capture One.

## Test
5. Shoot 20 frames at the normal pace of a product shoot. Count on the camera: 20.
6. Count files in `Studio/Capture/`: ____ (must be 20).
7. For 5 of the frames, time from shutter press to the preview appearing in the Capture One viewer:
   ____ s, ____ s, ____ s, ____ s, ____ s.
8. Disconnect and reconnect the cable mid-session, shoot 2 more frames. Did they arrive? ____

## Record
| Item | Value |
|---|---|
| Date | |
| Capture One version | 16.8.6.25 |
| Frames shot / arrived | 20 / |
| Median shot-to-preview | s |
| Reconnect test | pass / fail |
| Problems | |

Pass = 20/20 and reconnect works. Report the table in chat; it goes into the intention's
operational measurements.
