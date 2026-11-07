# Phase P46_direct_camera_adapter: Session Manager drives the camera itself

## What you can do now
Nothing changes on screen yet. Under the hood, Session Manager can now run your camera itself, without asking Capture One, as soon as a
camera support folder is installed. Without that folder, everything works exactly as before. With it, the camera dot follows what the
camera helper reports. It shows the camera's own model and whether it is on the cable or on Wi-Fi, says "busy" while a photo is being
taken, downloaded or the link is switching, and asks you to act when the password is refused, the fingerprint needs confirming, another
program holds the cable, or the camera did not confirm where photos are saved. Every shot is recorded the moment the camera reports it,
whether you pressed the app's button or the camera's own shutter. A photo that comes over the cable is checked and filed into the
session. When the same photo also arrives over Wi-Fi, the first copy wins and the second goes to the duplicates folder. If photos were
taken but have not arrived after a minute, the app asks the camera for its Wi-Fi transfer state and shows the answer on the Health page.
Reconnecting is offered only on Wi-Fi.

## Try it yourself
1. Run this phase's checks (no camera needed). They play recorded camera conversations through a stand-in helper:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager
   cargo test -p psm-testkit --test p46_direct_camera_adapter
   ```
   You should see `25 passed`.
2. Run the camera helper's own checks, which now include the Wi-Fi reconnect:
   ```
   tools/sidecar/build.sh --stub --test
   ```
   Near the end you should see `21 tests, 0 failed` and a line with `reconnect reopens the wifi session once`.
3. Start the app with a scratch data folder (no camera support folder installed):
   ```
   cd apps/desktop
   PSM_DATA_DIR=/tmp/psm-dev npm run tauri -- dev
   ```
   Open Health: the camera row looks as it did before, because without a support folder the app still uses Capture One's camera.

## Choices made for you
- The "another program holds the camera" notice clears only when the app's own cable connection opens, not when Wi-Fi connects. To
  change this, say "clear the held notice when Wi-Fi connects too".
- A reconnect request waits until the camera is not busy (no shot or download in progress) before it drops and reopens Wi-Fi.
- Taking a photo is refused while the camera helper is not running, as well as when no camera session is open.
- After a settings save, camera notices from before are kept, and the camera's own notices are then rebuilt from what the helper last
  reported.
- The "photos not received" check and the Wi-Fi state request run only when the app drives the camera itself.
- Dismissing "photos not received" while the camera's photo log is not ready answers with an error instead of doing nothing.
- The camera's Wi-Fi transfer report on the Health page is cut to 160 characters.
- A cable download is accepted only if its file is inside this run's own download folder. The check also works when macOS spells the
  same folder two different ways.
- When the cable session closes or is lost, the app also stops waiting for a cable download.
- The stand-in helper used in the checks can take several recorded lines at once, and the checks wait up to 10 seconds for a result.
- Smaller technical choices were taken as the plan suggested: how the adapter sends commands, how the photo log is reached, and how
  the app's photo watcher learns the capture folder.

## Not yet
- Nothing has run against your camera. The first real connection, with a hand-installed support folder, is the hands-on camera check
  in the next phase, with you present.
- Naming the programs that grab the camera, "Take the camera back", the start order, settings, password and fingerprint screens, and
  installing the support folder all come in later phases. There is no reconnect button yet.
- Points for later: a cable download made during a Wi-Fi session is recorded as "cable". During a long shoot with uploads coming and
  going, the camera may be asked for its Wi-Fi state after each upload. The "helper unavailable" notice has two wordings. The "held by
  another program" notice can stay after that program quits, until the cable connects. A storage-setting word the app does not know
  would still hide the "connected" report.

## Needs you
Nothing.
