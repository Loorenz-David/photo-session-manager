# Phase P06_live_previews: your Capture One edits show up in the app on their own

## What you can do now
Each new photo now gets an edited preview. This is a JPEG that Capture One renders with your adjustments, and the app keeps its own copy. When you change a photo in Capture One, the app notices and waits until you have stopped touching it for 15 seconds. Then it asks Capture One for a fresh render and swaps in the new file. The old file is deleted. If you switch back to the app window sooner, every photo that is out of date is re-rendered at once. Photos in the session you are working on go first. If the app was closed while you edited, it sees Capture One's saved settings files at startup and on focus, and renders those photos again too. A render that never arrives is asked for once more, then marked as failed. An edit made in Capture One counts as a retouch, so an already published product gets marked as needing an update. The app's own colour-tag writes are not counted as retouches.

## Try it yourself
1. Run this phase's checks. They use a pretend Capture One, so neither Capture One nor the camera is needed:
   ```
   export PATH="$HOME/.cargo/bin:$PATH"
   cd ~/Desktop/Developer/Photo-Session-Manager
   cargo test -p psm-testkit --test p06_live_previews
   ```
   The last line should read `test result: ok. 19 passed`.
2. Open your Capture One session, tether the camera, and start the app:
   ```
   export PATH="$HOME/.nvm/versions/node/v22.22.3/bin:$PATH"
   cd ~/Desktop/Developer/Photo-Session-Manager/apps/desktop
   npm run tauri -- dev
   ```
3. Shoot one frame. Wait a few seconds, then in a second Terminal window:
   ```
   sqlite3 ~/Library/Application\ Support/com.loorenz.sessionmanager/session_manager.db \
     'select id, adjusted_preview_path from photos order by id desc limit 3;'
   ```
   The newest photo has a path ending in `adjusted-1.jpg`. Open it with `open "<path>"`.
4. In Capture One, move the Exposure slider on that photo. Keep Capture One in front and wait about 15 seconds, then repeat step 3. The path now ends in `adjusted-2.jpg`, and `open` on the old `adjusted-1.jpg` path says the file does not exist.
5. Move the slider again and click straight back into the app window. Repeat step 3 right away: the number has already gone up.
6. See the edit count:
   ```
   sqlite3 ~/Library/Application\ Support/com.loorenz.sessionmanager/session_manager.db \
     'select raw_path, adjusted_gen, rendered_gen, retouch_gen from raw_state;'
   ```
   `retouch_gen` went up with your edits. Nothing is published yet, so no product changes state.

## Choices made for you
- The fix made two changes. If the same RAW is added to a second product after its edited preview already exists, the second product now gets that preview at once. Before, it stayed without one until your next edit, so that product could never count as ready. Nothing to change.
- The second fix change is internal. Knowledge of where Capture One keeps its settings files now lives only in the Capture One connection, not in the app's core. Nothing changes for you.
- The 15-second wait is checked by the app's background timer, so a render can start up to about a second late. Nothing to change.
- When you return to the window, out-of-date photos in your current session render first, in the order you sorted them, then by capture time. Nothing to change.
- When Capture One keeps several settings folders, the newest file counts. Nothing to change.
- If Capture One is closed or not answering when a render is needed, the app tries again every 2 seconds. This retry is noted to be slowed down later.
- Exported renders have a slot ready, but nothing uses it until the export phase.

## Not yet
- No screen shows the edited previews yet. The fade from the camera preview to the edited one comes with the screen phases.
- Export renders and their file names come with the export phase.
- A failed render is not shown anywhere yet. It is only recorded in the app's database.

## Needs you
Nothing.
