# Phase P25_tidy_up_after_provisioning: tidy-up after the Capture One setup

## What you can do now
You can export or publish right after your last shot. A photo that was still waiting for its fingerprint is now checked on the spot, so it no longer shows "RAW changed". If Capture One quietly refuses to open a session file, the app closes its own "PSM Open Window" helper window again and Health says the file couldn't be opened. Your other Capture One documents are never touched. When no session is open in the app, the red "Capture One document" row and the "camera disconnected" state go away on the next check, and when you close a session. A fresh setup starts with an empty Style, so Health starts green; a style you already saved is kept and applied. The log file now starts with an "app open" line, so the startup calls to Capture One are logged too. A misspelt shape name no longer makes every Health visit retry every waiting photo: they wait until Capture One lists that name. Publishing finds each new Shopify image by its file name, not by where it sits in the gallery.

## Try it yourself
1. Run this phase's checks (no camera, Capture One or Shopify needed):
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager
   cargo test -p psm-testkit --test p25_tidy_up_after_provisioning
   cargo test -p psm-testkit --test p12_shopify_adapter
   cargo test -p psm-desktop p25_tests
   ```
   All should report passed.
2. Quit the app. Move `~/Library/Application Support/com.loorenz.sessionmanager/settings.json` aside (keep it) and start the app:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager/apps/desktop
   npm run tauri -- dev
   ```
   Settings shows an empty Style field and Health has no "Style" row. Quit, put your file back and start again. Your settings return. If your file still says `PSM House` and you have no such style, the red "Style" row comes back: clear the field and Save.
3. Make a folder `~/Pictures/Scratch` with an empty text file named `Scratch.cosessiondb` inside. In Settings press "Use another session…", pick that file and Save. Health shows "Capture One document — Capture One could not open …/Scratch.cosessiondb…", and Capture One's Window menu lists no "PSM Open Window". Your other documents stay as they were. Then press "Use the app's own session", Save, and delete `~/Pictures/Scratch`.
4. Open a session in the app. In Capture One, click your Instagram session, then press Health → Retry checks: the app's session comes back to the front. Now close the session in the app, click Instagram in Capture One again and press Retry checks. Nothing switches, there is no red "Capture One document" row, and the camera shows connected.
5. Tether the camera, shoot one frame and press Export within a second or two. It exports, and "RAW changed" appears nowhere. Do the same with Publish on a session whose last shot was seconds ago.
6. Right after a launch, open the newest log file:
   ```
   open ~/Library/Application\ Support/com.loorenz.sessionmanager/logs/
   ```
   The first lines of today's `psm.<date>.log` are `app open`, then the startup calls to Capture One with their durations.
7. Quit the app, set `"crop_on_arrival": "Squircle"` in `settings.json`, start the app and shoot two frames. Health shows that 'Squircle' is not a Capture One crop ratio. Open Health three or four times: the log shows no new crop attempts. Pick a real shape in Settings, Save and press Retry checks: the two frames get shaped.
8. Publish a session with new photos. It works as before, and in the log each new image is created, then checked once before it counts as attached.

## Choices made for you
- A photo that hasn't been fingerprinted yet gets its fingerprint when you export or publish, instead of the app waiting for the background job. If you'd rather the app wait for the background job and say so, ask for "wait for the hash job instead".
- When Shopify renames a duplicate upload, the app recognises it only by the random-ID suffix Shopify was seen adding in testing. If Shopify ever renames files differently, say "accept any Shopify suffix".
- A relaunch still retries every failed shape once, even for a name Capture One doesn't know, so a corrected name is picked up straight away. Say "never retry unknown shape names on launch" to change it.
- The full path of the app's own session is now available to the screen. Showing it in Settings is the next phase's job.

## Not yet
- The "PSM Open Window" clean-up (step 3) was checked against a simulated Capture One, not your real one. Step 3 is the first real test.
- Settings still shows the short `~/Pictures/Session Manager` until you save. The next phase shows the full path.
- Leftover half-written export folders from interrupted exports are still not swept away.
- Opening the Health page still runs a full check, the same as "Retry checks". Only the pointless shape retries were removed.
- In a rare case (Shopify still processing your new image while showing another one as newest), one publish attempt can fail with "not found in the gallery". Publishing again picks the image up without duplicating it.

## Needs you
Nothing.
