# Phase P54_camera_cable_and_take_back: Taking the camera back works, the app no longer reopens Capture One into the cable's way, Wi-Fi keeps shooting while the cable is busy, and the dot says "Connecting"

## What you can do now
"Take the camera back" is no longer refused for ever. Previews that are only waiting for Capture One to notice a new file no longer block it. Only a preview Capture One is actually making does. The button now tells you in advance why it can't be used: it is greyed out with the reason, for example "an export is running", and turns on by itself once nothing is in the way. The same applies on the Health screen. If you quit Capture One while the camera is on the cable, the app first tells the camera helper to try the cable again. It reopens Capture One only once the helper has the cable, or after 15 seconds at most. When another program holds the cable, the helper moves to Wi-Fi after three failed tries (within about 10 seconds) and keeps shooting. When the cable becomes free, it switches back. At a Wi-Fi start the dot now reads "Connecting · Wi-Fi" until the camera is ready. Retry immediately shows "Connecting" and tries again.

## Try it yourself
1. Run this phase's checks. They use stand-ins, so no camera or Capture One is needed:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager
   cargo test -p psm-testkit --test p54_camera_cable_and_take_back
   ```
   You should see `6 passed`.
2. With the camera (part of the next hands-on run): plug the cable in while Capture One is running. The camera should keep shooting over Wi-Fi.
3. With the camera: start an export, then open the camera popover. "Take the camera back" is greyed out and says "an export is running". When the export finishes, click it. Within about 20 seconds the app should have the cable and Capture One should be back.
4. With the camera on the cable and the app holding it, quit Capture One yourself. The app should get the cable before it reopens Capture One.
5. Start the app with the camera on Wi-Fi only. The dot should say "Connecting · Wi-Fi" and then "Connected" within a few seconds.

## Choices made for you
- After the review, one thing was fixed. When the app is about to reopen Capture One, it now tells the camera helper to try the cable again, once. Before, the helper only tried again after a re-plug, so the wait did nothing. One side effect: if a different program holds the cable, Wi-Fi drops briefly while the helper tries. Say "only retry the cable when Capture One was holding it" to narrow this.
- While the helper waits between Wi-Fi attempts, the dot stays "Connecting". This also means a camera that is switched off shows "Connecting" instead of "Camera disconnected". Say "show disconnected between attempts" to change it.
- A cable held by another program counts toward the switch to Wi-Fi, the same as a failed connection: three tries.
- The 15-second wait applies every time the app reopens Capture One, not only at app start. Name a different number to change it.
- The take-back button rechecks once a second while it is on screen.

## Not yet
- Nobody has tried this with the real camera yet. The next hands-on run checks the cable, take-back, Wi-Fi fallback and "Connecting" steps above.
- If two previews for the same photo collide, that photo's preview may never appear. It no longer blocks take-back. It is noted for the preview-speed work.
- When a program other than Capture One lets go of the cable, the helper stays on Wi-Fi until you re-plug the cable.
- Retry while the camera helper is not running shows an error instead of the status.

## Needs you
Nothing.
