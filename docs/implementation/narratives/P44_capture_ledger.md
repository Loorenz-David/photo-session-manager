# Phase P44_capture_ledger: every camera file becomes exactly one photo, in the right session

## What you can do now
Nothing new to see on screen yet, but the app now decides where each photo the camera sends belongs. When a photo arrives over Wi-Fi,
the app moves it out of the hidden staging folder into the folder of the session that was open when you pressed the shutter. If no shot
is waiting for a file, it goes to the session open when the file arrived, and to "Unassigned" when no session is open. If the same photo
arrives twice (by cable and by Wi-Fi, or hours apart), the second copy goes to a hidden "duplicates" folder and never becomes a second
photo. If the camera reuses a file name for a different picture, the new one is kept as its own photo, named with "__dup1", "__dup2" and
so on. Nothing is ever overwritten or deleted. If the app crashes in the middle of moving a photo, it finishes the move the next time it
starts. Everything was checked with stand-in uploads on this Mac. No camera was used.

## Try it yourself
1. Run this phase's checks (no camera, Capture One or Wi-Fi needed):
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager
   cargo test -p psm-testkit --test p44_capture_ledger
   cargo test -p psm-core capture_ledger
   ```
   The first should pass 25 checks and the second 11.
2. Start the app once with an empty scratch folder, then quit it:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager/apps/desktop
   PSM_DATA_DIR=/tmp/psm-p44 npm run tauri -- dev
   ```
3. Look at the app's database:
   ```
   sqlite3 /tmp/psm-p44/session_manager.db ".tables"
   ```
   The list includes `camera_exposures` and `camera_deliveries` (the app's record of shots taken and files received). Both are empty.
4. Open the Health page. Nothing new appears there. The "photos taken but not received" notice needs a shot from the camera, which
   comes in a later phase.

## Choices made for you
- A shot counts as received once its file arrives. If the camera ever says how many files a shot makes, the shot waits for all of
  them. To change this, say "a shot is received only when all its files arrive".
- The folder name is read at the moment the app records where a photo will go, and from then on that item's folder name stays fixed,
  even if you edit its SKU before the photo shows up.
- A shot taken while the app was not sure which session was open is never matched to a file later. That file goes to the session
  open when it arrives.
- Only the app's own camera files follow the shutter-time rule. Photos you import by hand keep the import rules they had before.
- While a photo is uploading, the "not received" notice is hidden. When it reappears it counts as new, so the app may ask the camera
  for its upload status again. To change this, say "ask the camera for its upload status only once per missing batch".
- Dismissing the "not received" notice clears only the shots it was counting (older than one minute). A shot you took a few seconds
  ago stays expected.
- If the shots waiting for a file belong to different sessions, the photo goes to the oldest shot's session. The Health page then shows
  "Photo … may belong to <other session>", naming the first other session it finds. The newest such warning replaces the previous one.
- Until the camera connection is fully set up when the app starts, an incoming photo is refused. The camera then simply sends it again.
- If moving a file fails (for example on a disk that is not Mac-formatted), the photo is not lost. The camera sends it again, and the
  app reuses the record it already started.

## Not yet
- The camera helper that reports each shutter press comes in the next phases, so for now only Wi-Fi uploads reach this logic.
- Matching camera re-send names (like "DSC01234_1.ARW") to the original comes with the camera adapter.
- The timer that shows "N photos taken but not received" on the Health page, and the "check camera upload status" step, come later.
  The counting is ready.
- Dismissing the "may belong to another session" warning comes with the Health page update. Until then it clears when the app restarts.
- Small points for later: after a crash, if the app finds an identical copy already in place, that shot is not counted as missing.
  The app may also ask the camera for its upload status more than once during a long burst.

## Needs you
Nothing.
