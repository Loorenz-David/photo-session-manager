# Phase P43_ftp_receiver: the Wi-Fi photo receiver

## What you can do now
Nothing new to see yet. The app now has its own small file server, built in, that the Sony camera's "Auto FTP" upload sends photos to
over Wi-Fi. A photo counts as received only when its upload finished cleanly. If the Wi-Fi drops halfway, the part already sent is kept
out of sight, never shows up as a photo, and the camera can carry on from where it stopped. Two photos can arrive at once. The receiver
refuses to delete or rename anything. It accepts only devices on your own private network, and only with the camera password. When you
change its port, it moves to the new port at once without restarting the camera helper. All of this was checked with a stand-in client on
this Mac; the receiver stays switched off until camera support is installed in a later phase.

## Try it yourself
1. Run this phase's checks (no camera, Capture One or Wi-Fi needed):
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager
   cargo test -p psm-testkit --test p43_ftp_receiver
   ```
   It should pass 26 checks in about 25 seconds. Rarely, a run stalls for over a minute and one check fails. That is the Mac being slow
   to start the stand-in camera helper, not the receiver. Run it again.
2. Start the app (Capture One can stay closed):
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager/apps/desktop
   npm run tauri -- dev
   ```
3. In a second Terminal window, check that nothing is listening yet:
   ```
   nc -z 127.0.0.1 2121 && echo open || echo closed
   ```
   It prints `closed`. This is expected: the receiver starts only once camera support and its password exist.
4. Open Settings, change the SKU prefix and save. The app does not hang, and Settings shows nothing new yet.

## Choices made for you
- The camera logs in with the user name "sessionmanager". To change this, say "use a different camera FTP user name".
- The receiver runs only when camera support is installed and a camera password exists. Until a later phase creates that password,
  nothing listens.
- Only devices on your private network may connect. The Mac itself is refused outside the automated checks.
- A half-finished upload that sits idle for 10 minutes is given up (the partial file stays for a resume). To change this, say "change the
  upload idle timeout to <n> minutes".
- There is no cap on how many connections the camera may open at once. To change this, say "limit camera connections to 16".
- Dates in the file list the camera reads back are in UTC.
- What the review fixes changed. First fix: the receiver now tells the camera the Mac's real network address for each transfer. Before,
  it would have answered with an empty address (0.0.0.0) that a real camera cannot reach. The first fix also corrected a test helper that
  imitates a dropped Wi-Fi upload, so those checks stopped failing at random. Second fix: one check now waits up to 15 seconds instead of
  3 for the stand-in camera helper to start, because the Mac starts many of them at once during the checks.

## Not yet
- Moving received photos into the session folder, spotting duplicates and the list of expected photos come in the next phase. For now a
  received photo stays in a hidden staging folder.
- The camera password, the camera setup instructions, the Camera section in Settings and the receiver's lines on the Health page come
  later.
- Encrypted transfers (FTPS) are later, as you decided.
- Weak spots for later phases: if the camera sends any command while a photo is uploading, that upload is cancelled. One rare network
  error can stop the receiver until the app restarts. Changing the camera password will need a receiver restart. None of this can
  happen today, because the receiver is still off.

## Needs you
Nothing.
