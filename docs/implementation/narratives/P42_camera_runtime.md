# Phase P42_camera_runtime: the groundwork for talking to the camera directly

## What you can do now
Nothing new to see yet. This phase lays the foundation for driving the Sony camera without Capture One in the middle. The app can now
start, watch over and stop the small camera helper program. It never runs two copies of it, even while you change settings. If the
helper crashes, the app restarts it, waiting a little longer after each crash. The camera passwords will live in the Mac's Keychain and
never end up in a log or on the helper's command line. The camera indicator in the header now knows five states: connected,
connecting, busy, needs your attention, and disconnected. Today it still shows only connected or disconnected, because the camera
still reaches the app through Capture One. All of this was checked with a stand-in helper program; the real one arrives in later
phases.

## Try it yourself
1. Run this phase's checks (no camera, Capture One or Keychain needed):
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager
   cargo test -p psm-testkit --test p42_camera_runtime
   cargo test -p psm-core camera_status
   ```
   The first should pass 12 checks (about 25 seconds: it waits out the real 3-second quit timeout), the second 12.
2. Start the app with Capture One open and the camera tethered as usual:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager/apps/desktop
   npm run tauri -- dev
   ```
   The camera indicator in the header is green and shows the camera's name, as before.
3. Unplug the camera and click the indicator. It turns red and says "Camera disconnected". Plug it back in and click again: it turns
   green.
4. Open Settings, change the SKU prefix and save. The app keeps running, and the Health page still shows the camera line.

## Choices made for you
- The camera over Capture One's tether always counts as connected "by cable". To change this, say "show the Capture One camera
  without a connection type".
- A warning that the camera helper is down appears only after three crashes in a row, not after the first, because one crash is
  recovered within a second. To change this, say "warn me on the first helper crash".
- The helper's crash counter starts over once a copy has stayed up for a minute. To change this, say "reset the helper's crash count
  after <n> seconds".
- When several versions of camera support are installed, the app uses the one whose version name sorts last. This becomes a proper
  choice when camera support can be installed from Settings.
- The helper's last 50 error lines are kept in memory for diagnosis, with any password replaced by "[REDACTED]". To change this, say
  "keep more helper log lines".
- When another program holds the camera, the message reads "<program> is holding the camera" for now. A precise message with what to
  do comes with the phase that takes the camera back from other programs.

## Not yet
- The real camera helper, the Wi-Fi photo receiver, the list of expected photos and the direct camera connection come in the next
  phases. Until then the camera still works through Capture One, exactly as before.
- The camera indicator shows the new states (connecting, busy, needs attention) without their own colours. The final wording and
  colours come with the camera settings and health screens.
- Two weak spots for later phases to fix. If the helper program cannot start at all, the app does not yet retry it or tell you so.
  If reading the Keychain fails once, the helper would keep running without the Wi-Fi password until the app restarts. Neither can
  happen today, because no helper is installed yet.

## Needs you
Nothing.
