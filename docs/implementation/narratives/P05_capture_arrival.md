# Phase P05_capture_arrival: new shots become photos in the right session on their own

## What you can do now
When you press the shutter, the app records each new frame as one photo in the right session. It picks the frame up three ways: from Capture One's "capture done" signal, from watching the capture folder, and from a sweep of the folder each time the app starts. All three can see the same frame and you still get exactly one photo. A frame is taken only once the file has stopped growing, so a half-written file never counts. Frames in `_Unassigned` go to Unassigned, frames in an item's folder go to that item's session, and anything else goes to the session that is active. For each new photo, the app copies the camera's own preview JPEG out of the RAW without opening the RAW for writing, records when the shot was taken, and queues the fingerprint and the edited preview. Behind the scenes there is also "import a file" and "move everything in Unassigned to this item". They have no buttons yet.

## Try it yourself
1. Run this phase's checks. They use a pretend Capture One, so neither Capture One nor the camera is needed:
   ```
   export PATH="$HOME/.cargo/bin:$PATH"
   cd ~/Desktop/Developer/Photo-Session-Manager
   cargo test -p psm-testkit --test p05_capture_arrival
   ```
   The last line should read `test result: ok. 33 passed`.
2. Open your Capture One session (the one you set up after the last phase), tether the camera, and start the app:
   ```
   export PATH="$HOME/.nvm/versions/node/v22.22.3/bin:$PATH"
   cd ~/Desktop/Developer/Photo-Session-Manager/apps/desktop
   npm run tauri -- dev
   ```
   **Be aware:** on this first start, the sweep turns every RAW already in the capture folder into a photo. That includes the four test shots lying loose in the session's capture folder: they go to Unassigned, because no session is active yet.
3. Press the shutter once or twice. Then, in a second Terminal window, list the photos:
   ```
   sqlite3 ~/Library/Application\ Support/com.loorenz.sessionmanager/session_manager.db \
     'select session_id, raw_path, classification, original_preview_path from photos;'
   ```
   Each frame appears once, in session 1 (Unassigned), as `unreviewed`, with a preview path filled in a moment later.
4. See where each frame was reported from:
   ```
   sqlite3 ~/Library/Application\ Support/com.loorenz.sessionmanager/session_manager.db \
     'select source, payload from capture_events;'
   ```
   A frame can show up as both `hook` and `watcher`, and step 3 still lists it only once.
5. Open one of the previews (paste a path from step 3): `open "<path>"`. It is the camera's own JPEG.
6. Quit the app and start it again (same line as step 2). Repeat step 3: there are no new rows, and no frame appears twice.

## Choices made for you
- If you move photos out of Unassigned or into another product, they stay where you put them after a restart. This is what the fix changed. Before, the startup sweep made a second copy of any frame that had moved away from the folder it sits in, and it did the same for loose frames whenever a different session was active. Now the app recognises a RAW it has already seen, whichever session it is in. Nothing to change.
- Importing a file on purpose may still add the same file to a second session. Nothing to change.
- Import accepts any file, not only RAWs. Say "only accept RAW files on import" to change this.
- If Capture One is closed when the app starts, the app waits and picks up the missed frames once it can reach Capture One again. Nothing to change.
- A file that is still changing after two minutes is given up on and recorded as an error, with no photo made. Say "wait longer for slow files" to change this.
- The "edited preview finished" signals from Capture One are only logged for now. The next phase acts on them.
- Starting the app takes about a second longer once photos exist, because the sweep checks twice that each file has stopped changing. This is noted to be trimmed later.
- Nobody pressed the real shutter in this phase. The checks use a pretend Capture One plus one real Sony RAW, read without changing it. Step 3 above is the first real run.

## Not yet
- No screen shows the new photos yet. That comes with the screen phases.
- There are no buttons for importing a file or moving Unassigned to an item yet.
- Edited previews from Capture One are requested but not collected yet. That comes next, with live previews.

## Needs you
Nothing.
