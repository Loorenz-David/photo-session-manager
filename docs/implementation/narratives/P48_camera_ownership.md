# Phase P48_camera_ownership: Session Manager keeps the camera, and says who has it when it cannot

## What you can do now
Nothing new appears on screen yet. The buttons come in a later phase. Underneath, the app now protects its hold on the camera's cable
connection. When another program takes the camera, the camera dot and the Health page name that program and say how to fix it, for
example "Capture One is holding the camera's USB connection — Take the camera back". Shooting over Wi-Fi carries on meanwhile. When the app
starts with camera support installed, it gives its own cable connection the first chance and only then opens Capture One. That wait lasts
at most 15 seconds and is skipped when Capture One is already open. "Take the camera back" is built and tested on stand-ins. On your
click it saves the Capture One document, asks Capture One to quit (it never forces it), reconnects the cable, then reopens Capture One
with the document and the session's folder. While that runs, nothing else in the app can open Capture One early. Switching sessions no
longer waits for Capture One, and the Mac does not go to sleep while the camera is on the cable.

## Try it yourself
1. Run this phase's checks. No camera is needed: they play recorded camera conversations against a stand-in Capture One:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager
   cargo test -p psm-testkit --test p48_camera_ownership
   ```
   You should see `32 passed`. It takes about a minute, because the take-back checks step through their waits.
2. Run the two checks that call the app's new commands the way the screen will:
   ```
   cargo test -p psm-desktop p48_tests
   ```
   You should see `2 passed`.
3. Start the app with Capture One already open:
   ```
   cd apps/desktop
   npm run tauri -- dev
   ```
   Nothing changes: the app does not wait, and Capture One is not reopened.
4. Open Health. The camera row behaves as before, and "Retry checks" still works.
5. Open a session, then switch to another. Capture One follows, as before.

## Choices made for you
- "Take the camera back" uses the plain Capture One connection to save and quit, and only that. To change this, say "give the editing layer
  its own quit".
- Any check that finds Capture One running counts as "Capture One was already open" for the start-up rule. To change this, say "only the
  very first check at start-up counts".
- A camera helper that cannot even be started also lets Capture One open right away, just like a helper that reports it is unavailable.
- The Mac may sleep again as soon as the camera switches to Wi-Fi, because only one camera connection exists at a time. To change this,
  say "keep the Mac awake on Wi-Fi too".
- If the app cannot tell whether Capture One is running while it waits for it to quit, it keeps waiting. After 20 seconds it reports
  "Capture One did not quit". If the app cannot send the reconnect request, it keeps waiting too, and Capture One still reopens.
- "Take the camera back" is offered only while the camera is held for Capture One, even if another notice is shown first. "Test camera
  connection" names a holder only while the dot shows "held by another program".
- A successful take-back reports five completed steps. Releasing the guard at the end is not counted.
- When Capture One follows a session switch again, the "Capture One will not show this session's new photos" notice clears right away.
- The program list keeps each program name once, in the order macOS lists them. The sleep guard talks to macOS directly and adds no new
  library.
- The stand-in Capture One used in the checks also forgets that it was opened when it is told to stop.

## Not yet
- Nothing has run against your camera. The real "Take the camera back" (re-plug the cable with Capture One open, then one click) is part
  of the hands-on camera acceptance phase.
- The "Take the camera back" and "Test camera connection" buttons, and their sentences on screen, come with the camera settings and
  header phase.
- Programs found by a deeper scan (anything using macOS camera services that is not on the fixed list) wait for the support-package
  spike's answer. The real process names of Imaging Edge Desktop and the Photos widget still need checking on the Mac.
- Points for later:
  - In a rare timing case, an app action that started just before your click could reopen Capture One during the take-back. The take-back
    would then stop and say Capture One "was started meanwhile", although you did nothing.
  - If Capture One reopens but refuses the session folder, the take-back reports a failure even though the camera is back on the cable.
  - Right after app start, the Health page may take up to 15 seconds to load while the app waits for the camera.
  - The "held by another program" notice still stays after that program quits, until the cable connection opens.

## Needs you
Nothing.
