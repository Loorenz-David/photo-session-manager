# Phase P04_capture_one_adapter: Session Manager now talks to Capture One and the camera

## What you can do now
There are no new buttons on screen yet, but the app can now work Capture One for you. When it starts with your
Capture One session open, it checks the document is a session (not a catalog). It installs its three event
scripts and sets both render recipes to overwrite old files and write into a `_session_manager` subfolder. Then
it points Capture One's capture folder at the active session's folder, or at `Capture/_Unassigned` when no
session is active. When you switch sessions it saves the document, creates the folder, sets it and reads it back.
If the read-back doesn't match, it puts the old folder back, shows "capture folder not set", shows the camera as
disconnected and refuses to shoot, so a frame never lands silently in the wrong product's folder. When you mark
photos Main, Condition or Deleted, Capture One gets the matching colour tag (green, blue, red, none). The app
checks each tag by reading it back and keeps retrying in the background if Capture One is busy or closed. It can
also read the camera's name and fire the shutter, focusing first.

## Try it yourself
1. Run this phase's checks. They use a pretend Capture One, so neither Capture One nor the camera is needed:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager
   cargo test -p psm-testkit --test p04_capture_one_adapter
   ```
   The last line should read `test result: ok. 21 passed`.
2. Point the app at your real session. Start the app once so it creates its settings file, then quit it:
   ```
   export PATH="$HOME/.nvm/versions/node/v22.22.3/bin:$PATH"
   cd ~/Desktop/Developer/Photo-Session-Manager/apps/desktop
   npm run tauri -- dev
   ```
   Open the settings file, set `"document_id": "/Users/davidloorenz/Pictures/session test 1"` and save:
   ```
   open -e ~/Library/Application\ Support/com.loorenz.sessionmanager/settings.json
   ```
   In Capture One, make sure that session has two recipes named `PSM Preview` and `PSM Web` (disabled is fine).
   **Be aware:** this run changes your real session. Capture One's capture folder moves to `Capture/_Unassigned`
   (the folder is created), both recipes switch to "overwrite" with a `_session_manager` subfolder, and the
   app's three event scripts are installed in Capture One.
3. With Capture One open, start the app again (same `npm run tauri -- dev` line). The Health page should list the
   session as open, hooks installed, recipes ready, and the camera: its name if the ZV-E10 is tethered, or
   "camera_disconnected" if not.
4. Check that the event scripts are installed. This prints a path inside the app's `hooks` folder:
   ```
   osascript -l JavaScript -e 'Application("Capture One").captureDoneScript().toString()'
   ```
5. Rename `PSM Web` in Capture One, quit the app and start it again: Health shows `recipe_missing` with
   "PSM Web" in red. Rename it back afterwards.
6. Quit Capture One, then quit and restart the app. Capture One is **not** started, and Health shows the document
   as not open. Health is checked when the app starts, so restart the app after opening or closing Capture One.
   The "Test Capture One" button only refreshes what was found at startup.

## Choices made for you
- The app sends Capture One one command at a time, so two checks never collide. Nothing to change.
- The startup checks run in the background, so the window never freezes while Capture One is slow. Nothing to
  change.
- Health lists the things that are fine as well as the problems. Say "show only problems on Health" to change this.
- When a tag write timed out, the retry first reads the tag. If the write had gone through after all, it doesn't
  write again. Nothing to change.
- The "photo not found in Capture One" warning clears as soon as any tag write succeeds, even one for a different
  photo. It comes back at that photo's next retry, within a minute. This is noted to be tightened. Say "fix the
  not-found warning now" to bring it forward.
- If you switch to a session while Capture One is closed, the app says the document isn't open, and the session
  you had before stays active. Nothing to change.
- Nobody tried this phase against your real Capture One, only against the pretend one. Step 2 onwards above is
  the first real run.

## Not yet
- No screen has buttons yet for switching sessions, checking the camera or shooting. They come with the screen
  phases.
- New captures are not picked up as photos yet. That is the next phase.
- Edited previews are requested but not collected yet. That comes with live previews.
- Finding a photo while Capture One is browsing a different folder has not been tried on the real app yet.

## Needs you
Nothing.
