# Phase LV1_camera_live_view: the camera's live picture and tap-to-focus reach the app

## What you can do now
Nothing new is on screen yet. This phase builds the plumbing under the live preview. When the camera is
connected, the camera helper pulls the camera's live picture by itself, about 25 times a second over USB, and never
sends the same frame twice. It sends the frames on their own channel, so they never slow down captures or the camera's
other messages. Inside the app, a hub keeps only the newest frame. It hands that frame only to screens that are showing
Live, so a slow viewer gets the latest picture instead of a backlog. Tap-to-focus is wired too: a point becomes "move the
camera's focus point there and half-press". The camera answers focused or not focused and returns its focus frame. If the
helper restarts for any reason, the live picture comes back by itself on the next camera session.

## Try it yourself
1. Build the camera helper and run its own tests (the stub build needs no camera):
   ```
   tools/sidecar/build.sh --stub --test
   ```
   Both test groups pass.
2. Run the live-view tests of the app:
   ```
   cargo test -p psm-testkit --test suite lv1_
   ```
   Every test passes. Among them: frames arrive in order, a repeated frame is dropped, a slow viewer only gets the
   newest frame, a helper that stops also stops live view, and a restarted helper's frames arrive.
3. Optional, with the camera and the SDK build of the helper: start the app from a terminal with `PSM_LOG=debug` and
   connect the camera. The log shows `liveview.started`, and every 10 seconds a `live.source … frames_10s=` line near
   250 on USB.

## Choices made for you
- Live view starts by itself whenever the camera is connected and has live view enabled. The app never turns it on and
  never changes its quality. To change this, say "let the app switch live view on or off".
- A tap moves the camera's focus point and half-presses. The app never changes the camera's focus area. If the focus
  area cannot be moved, the app says so and changes nothing. To change this, say "let a tap change the focus area".
- A capture waits for a focus that is still running (at most about a second), so the photo is focused where you tapped.
  To change this, say "capture immediately, even during a focus".
- Focus keeps working while live view itself is off, as long as the camera is connected. To change this, say "focus only
  while live view runs".
- If the helper ever sends a broken frame, live view stops for that helper until it restarts, rather than showing a
  garbled picture. To change this, say "skip bad frames and keep going".
- The review sent the work back twice. Both fixes were about the helper restarting:
  - The first fix made the frame connection survive a helper restart.
  - The second fix lets a newly started helper take over the frame connection even when live view was off at the moment
    of the restart. That happens when you switch Capture One sessions or save Wi-Fi details. Before this fix, the next
    camera session would have shown no live picture at all.

  Nothing for you to decide here.

## Not yet
- No screen shows the live picture yet. The phone viewer comes in the next phases, and the Mac's live screen waits for
  its own design.
- The "current shot" (the photo you just took, shown next to Live) comes next.
- Nothing has been tried on the real camera yet. The hardware check is its own later phase.

## Needs you
Nothing.
