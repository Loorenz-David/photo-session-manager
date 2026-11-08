# Phase P51_camera_setup_ui: you can see the camera's state and set it up from the app

## What you can do now
- **The camera dot in the header** has five states. Green shows the camera's name and "Cable" or "Wi-Fi". Amber pulses while it
  connects, and stays amber with "Taking a photo", "Receiving a photo" or "Switching connection" while the camera is busy. Red shows
  one plain sentence when something needs you, and red also means no camera. Clicking the dot opens a card with the whole
  sentence and the one useful action. **Take the camera back** appears when Capture One holds the camera. **Retry** appears
  otherwise, and on Wi-Fi, when photos were taken but not received, **Reconnect** and **Dismiss**.
- **Health** lists every camera item in plain words. "Photos not received" has a Dismiss button. "Capture One holds the camera"
  has Take the camera back. The camera helper's last log lines open behind a small disclosure.
- **Settings → Camera** has four parts. *Camera support*: add the Sony archive you downloaded; the installed version appears.
  *Transfer from the camera*: the FTP user, a password shown once with Copy and the camera menu steps, and the receiver ports.
  *Wi-Fi control*: the camera login, which goes to the Keychain and is never shown back, and the fingerprint card. *Check*: Test
  camera connection tells you who holds the camera or could take it. One line says the app sets where the camera saves photos:
  card and computer on the cable, card only on Wi-Fi.

## Try it yourself
1. Run this phase's checks. They use stand-ins, so no camera, Sony archive or Capture One is needed:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager/apps/desktop
   npx vitest run src/features/shell/__tests__/p51_camera_dot.test.tsx src/features/health/__tests__/p51_health_camera.test.tsx src/features/settings/__tests__/p51_camera_section.test.tsx
   ```
   You should see `32 passed`.
2. Start the app:
   ```
   npm run tauri -- dev
   ```
3. Open Settings → Camera. Under *Camera support*, add the Sony archive you have. Its version appears, and *Transfer from the
   camera* and *Wi-Fi control* appear below.
4. Press **Generate password**. It is shown once, with Copy and the camera menu steps. Quit the app (⌘Q) and start it again:
   the password is no longer shown.
5. Enter the camera's Access Authentication user and password and save. Both fields empty themselves, and the settings file has
   no trace of either.
6. (Camera, optional; the hands-on phase covers it too.) Switch the camera on with the cable in: the dot shows the camera and
   "Cable". Pull the cable: "Wi-Fi". On the first Wi-Fi connect, Settings → Camera shows the fingerprint. Compare it with the
   camera's menu (MENU › Network › Network Option › Access Authen. Info), then press Confirm.

## Choices made for you
- The helper's log lines load when you open the disclosure. They do not refresh by themselves. Say "refresh the helper log
  live" to change it.
- Copy shows "Copied" for 1.5 seconds.
- The camera steps show a placeholder where your Mac's address goes, because the app does not know the address yet. Say "show
  the Mac's address in the camera steps" to have it filled in.
- The dot's text is cut at 48 characters; the card always has the whole sentence.
- The reject button on the fingerprint card is labelled "Not mine".
- What the fix changed, in plain words: in the first version, if you opened Settings after the camera had already asked about
  its fingerprint, the card showed the words "Check the camera fingerprint" where the fingerprint belongs. It still offered
  Confirm, so you could trust a fingerprint you never saw. Now that card says the fingerprint will appear when the camera
  reports it, and Confirm and Not mine appear only next to the real value. The camera asks again at each connection attempt,
  so the value arrives within about 30 seconds.

## Not yet
- Nothing has run against the real camera in this phase. The hands-on acceptance phase covers it.
- Take the camera back is always clickable. If it cannot run right now, it tells you why only after the click, instead
  of being greyed out with the reason beforehand.
- When saving settings fails for a reason that is not the camera ports (for example, a JPEG quality of 50), the error still
  talks about ports.
- The Camera section sits inside the Capture One block, so Capture One's "Default shape" setting appears below it.
- If the camera's fingerprint changes while Settings is already open, the card's title says "Confirm camera fingerprint" rather
  than "Fingerprint changed". The header dot still says the fingerprint changed, and confirming correctly replaces the old one.
- After you press "Not mine", the camera keeps trying to connect, so the card comes back at its next attempt.

## Needs you
Nothing.
