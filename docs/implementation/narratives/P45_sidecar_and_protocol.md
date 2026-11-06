# Phase P45_sidecar_and_protocol: the camera helper joins the app, and the app learns to talk to it

## What you can do now
Nothing new appears on screen yet. The small camera helper program from your Sony camera lab is now part of Session Manager's own
source code, and none of Sony's files and none of the lab's home-made stand-ins for them are in your public repository. On any Mac
the helper builds as a safe "stub" that refuses camera work and says so. On a Mac that has Sony's kit (yours does, in the lab
folder), it builds as the real helper. The helper now behaves the way the book asks. Your camera password reaches it privately at
start-up, never through the system environment. Every new connection sets where photos are stored and checks that the setting took:
"computer and memory card" over the cable, "memory card" over Wi-Fi, never "computer only". When you quit during a cable session,
the camera is set back to "memory card". A quit waits up to 10 seconds for a photo that is still downloading over the cable. It
watches only your camera's USB plug, not every Sony device, and it has a new "try the cable again" command. On the app side,
Session Manager now reads everything the helper reports and pairs each command it sends with its answer: done, refused, or failed.

## Try it yourself
1. Build the stub helper and run its own checks (no camera needed):
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager
   tools/sidecar/build.sh --stub --test
   ```
   Near the end you should see `20 tests, 0 failed` and `100% tests passed out of 1`.
2. Ask the helper what it is:
   ```
   apps/desktop/src-tauri/binaries/camera-sidecar-aarch64-apple-darwin version
   ```
   It prints one line that includes `"sdk_compiled_in":false`, meaning this is the stub.
3. Ask the stub to start camera work and watch it refuse:
   ```
   apps/desktop/src-tauri/binaries/camera-sidecar-aarch64-apple-darwin auto --save-dir /tmp/x; echo $?
   ```
   It prints a line with `sdk.unavailable`, then `4`.
4. Optional, using Sony's kit in your lab folder:
   ```
   tools/sidecar/build.sh --sdk ~/Desktop/Developer/Camera-Connection-Tests/vendor/sony-sdk
   target/camera-sidecar/sdk/camera-sidecar version
   tools/sidecar/build.sh --stub
   ```
   The middle command prints `"sdk_version":"2.02.00"`. The last one puts the stub back, so the app keeps using it until the phase
   that installs the camera support package.
5. Run this phase's checks:
   ```
   cargo test -p psm-testkit --test p45_sidecar_and_protocol
   cargo test -p psm-integrations mc31
   ```
   The first should pass 15 checks and the second 7.
6. Start the app and confirm nothing changed on screen. Then run `git status`: no helper binary or build folder is listed.
   ```
   cd apps/desktop && npm run tauri -- dev
   ```

## Choices made for you
- A finished download ends the "photo still downloading" wait only if such a wait was running, so it never cuts short a shot that
  is still being taken. To change this, say "any finished download ends the busy state".
- If the cable is pulled while a quit is waiting for a download, the helper stops at once and treats it as a lost connection.
- If the camera does not report its storage setting at all, the connection still goes ahead and is marked "not verified", with no
  value shown.
- The camera's USB plug is recognised by its product number (the a7R V's). A device that reports no product number is still counted
  as the camera, so a missing detail never hides it. To change this, say "ignore Sony devices that report no product number".
- The "computer only" storage choice was removed from the helper's diagnostic commands too, not just from what the app sends.
- When the app sends a command without its own reference, the app makes up a short one. The answer is always matched to the
  reference actually sent.
- The "helper unavailable" notice now clears only when the helper sends a line the app can read. Garbage output no longer counts as
  "the helper is alive".
- The helper's notes keep the last 50 lines, and the app keeps the last 1,000 helper events.
- Building the app on a clean Mac builds the stub helper by itself. After changing the helper's C++ code, run the build script
  yourself. The app does not rebuild a helper that is already there.
- The support-package manifest tool lists every Sony library under the folder you give it. Point it at the folder that holds
  `libCr_Core.dylib`, not at the top of Sony's download, or it will also list the sample app's libraries.

## Not yet
- The app does not act on the helper's reports yet. Showing camera status, cable/Wi-Fi switching and capture handling come in the
  next phase.
- Nothing has run against the camera. The real helper's connect, storage-setting and quit steps are first tried in the hands-on
  camera check, with you present.
- Installing Sony's runtime as a support package comes later.
- Small points for later: with two quick cable shots, the first finished download ends the wait for the second. A storage-setting
  word the app does not know would hide the whole "connected" report. Running this phase's checks makes the next app build slightly
  slower. One older screen test failed once by chance during this review and passed on every re-run.

## Needs you
Nothing.
