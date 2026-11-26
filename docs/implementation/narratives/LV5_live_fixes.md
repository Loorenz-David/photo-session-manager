# Phase LV5_live_fixes: the fixes from the hardware check

## What you can do now
- **Fast shooting.** When you shoot quickly over Wi-Fi, the "Next" outline on the Mac and on the phone no longer jumps
  back onto a shot whose photo is still arriving. Each photo lands in the shot that was current when you pressed the
  shutter.
- **Mac changes reach the phones.** Deny, Main/Condition, reorder, move, copy, delete, Undo and Redo on the Mac show on
  every connected phone within a second.
- **Wi-Fi stalls.** A weak Wi-Fi moment no longer drops the phone's connection. Live freezes for a few seconds and
  continues with the newest picture.
- **Focus Area.** If the camera's Focus Area is set to something that cannot move (Wide, Zone, Center), the phone says
  so instead of pretending to focus.
- **Exposure readout.** The phone's live view shows the camera's exposure.
- **Settings → Phone** shows which phone is connected right now.

## Try it yourself
1. Start Session Manager from this branch with the camera on Wi-Fi and both phones paired:
   ```
   cd apps/desktop
   npm run build
   npm run tauri dev
   ```
2. Open a session that has a shooting template. From a phone, take four pictures quickly, about one a second. Watch
   the Mac's "Next" outline. It never jumps onto a shot whose photo is still arriving, and each photo lands in its own
   shot.
3. On the Mac, Deny a photo, move one to Condition, then Undo. Each change shows on both phones within a second.
4. Hold the phone on live and walk to the far end of the Wi-Fi. The picture may freeze for a few seconds, then
   continues with the newest frame. It does not fall back to the template.
5. Set the camera's Focus Area to Wide and tap the phone's live view. The phone says "Set the camera's Focus Area to a
   movable spot (e.g. Flexible Spot) to focus from the phone", and the camera's setting is unchanged. Set it back to
   Flexible Spot: tapping focuses again.
6. On live, the phone shows the camera's exposure, for example "1/80 · f/8 · ISO 2000".
7. Open Settings → Phone with a phone connected. Its dot is green and it reads "Connected now". Close the app on the
   phone: within about 20 seconds it no longer reads connected.

## Choices made for you
- **A photo on its way.** A shot whose photo is still on its way counts as taken until its photo is in the session.
  If the photo never appears, it stops counting 5 minutes after the shutter. To change this, say "a shot frees up
  sooner if its photo doesn't arrive".
- **Wi-Fi stalls.** The phone keeps its connection through a Wi-Fi stall of up to 15 seconds. After that, the Mac lets
  it go and the phone reconnects by itself. To change this, say how long a stall the phone should survive.
- **Which Focus Areas move.** Only Flexible Spot (any size, also with tracking) counts as movable for focusing from
  the phone. Wide, Zone, Center and their tracking variants get the "set a movable Focus Area" message, and the camera
  is left as it is. To change this, say which Focus Areas should also accept taps.
- **Settings → Phone** re-reads the list every 5 seconds while it is open. To change this, say how often.
- **A missing exposure value** shows as "–" in the phone's readout, for example with no lens attached or in a special
  mode. To change this, say what it should show instead.
- **New photos for the phone.** The Mac now tells the phone when a photo's preview is ready. The phone starts using
  this in the next phase.
- **What the review round changed.** The one fix round changed nothing in how the app behaves. It made the automatic
  check for fast shooting stricter: after every shutter press, it now checks where "Next" points, including that it
  stays on Free shot while earlier photos are still arriving. Before, that check could not have caught the original
  problem.

## Not yet
- **The phone's bottom strip** still uses the old gestures, and a tile may still show a broken picture. Both are fixed
  in the next phase.
- **The rest of the hardware check** resumes after the next phase.
- **The camera's own focus frame** on the phone, and HTTPS for the phone, are not built.

## Needs you
Nothing.
