# Phase P50_camera_settings_and_support: the app can install Sony camera support, keep the camera passwords in your Keychain, and make the FTP password

## What you can do now
The app can now set up its own direct camera connection, behind the scenes (the screens come in the next phase).
- **Camera support.** You give the app the Sony Camera Remote SDK archive you downloaded. It unpacks the archive itself and checks every
  runtime file against a list of fingerprints built into the app. It then copies those files, together with its own camera helper, into
  its data folder and signs them so macOS lets them run. If a file is missing or does not match, nothing is installed and the app names
  that file. If the folder is damaged later, the app falls back to Capture One and says so in Health. If no camera support is installed,
  that is not an error: the camera keeps working through Capture One as before.
- **Camera passwords.** The camera's Access Authentication user and password go only to your Keychain. They never go into the
  settings file or the logs. Changing them restarts the camera helper only while the camera is on Wi-Fi. During a cable session the
  helper picks them up at its next start, so a shoot over cable is never interrupted.
- **FTP password.** The app makes a 32-character password for the camera's FTP upload, stores it in the Keychain, shows it to you once,
  and reopens its upload receiver with it at once. Making a new one replaces the old one.
- **Fingerprint.** The first time the camera connects over Wi-Fi, the app asks you to confirm its fingerprint, and it remembers it only
  after you confirm.
- Reading the Keychain never delays the camera's start. If macOS asks for your Keychain password at launch, the cable connection still
  starts straight away, and the passwords are passed on when the Keychain answers.

## Try it yourself
1. Run this phase's checks. They use a stand-in camera helper, so no camera, no Sony archive and no Keychain are needed:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager
   cargo test -p psm-testkit --test p50_camera_settings_and_support
   ```
   You should see `22 passed`. It takes about a minute.
2. The following steps need the controls that arrive in the next phase. Until then, the orchestrator can call the commands for you.
   Add camera support from the Sony archive you downloaded. The app answers with the version `2.02.00`, and its data folder gains
   `camera/sony/2.02.00/` with the Sony runtime and `camera-sidecar`.
3. Generate an FTP password. It is shown once. After a relaunch it is not shown again.
4. Enter the camera's Access Authentication user and password. Open the settings file: neither value is there. Keychain Access shows
   three `com.loorenz.sessionmanager.camera` items.

## Choices made for you
- When the app first reads the camera passwords from the Keychain, a cable connection that is already open starts without them.
  The helper restarts with them only once the cable session ends. To change this, say "wait for the camera passwords before starting
  the camera".
- The app compares its installed helper with the one it ships, and on an app update it refreshes the installed copy.
- An archive with a file name that starts at the disk's root is refused, even though Sony's archive never has one.
- What the two fixes changed, in plain words:
  - The first version read the camera passwords at launch, but when you were on a cable it never passed them to the helper, even
    after you unplugged. Switching to Wi-Fi would then have failed. The first fix passes them on as soon as the cable session ends.
  - That version also read your Keychain at every launch, even with no camera support installed. After every app update, that would
    have shown you a Keychain prompt for a camera feature you were not using, and the app's own checks did the same on your Mac. The
    second fix reads the Keychain only once camera support is installed.

## Not yet
- Nothing has run against the real camera or the real Sony archive in this phase. The hands-on acceptance phase covers it.
- The Settings → Camera screen, the camera dot's messages and the Health items come in the next phase.
- Points for later:
  - In a packaged app, the "is the installed helper up to date?" check never matches, so the app re-copies and re-signs the helper at
    every start. It costs about 20 ms. It is harmless, but it is not what was intended.
  - Confirming a fingerprint trusts whatever fingerprint the camera shows next, not exactly the one you looked at.
  - The fake camera may raise the "confirm fingerprint" question repeatedly. The next phase's screen must show it only once.
  - Two of this phase's checks (the delayed Keychain answer during a cable session, and "no Keychain read without camera support") pass
    even on the old, faulty code. The code is right, but these checks would not notice if it broke again.
  - One check of the camera passwords can fail about once in a few hundred runs by chance, because of random folder names.
  - Three of this phase's checks take about a minute each.

## Needs you
Nothing.
