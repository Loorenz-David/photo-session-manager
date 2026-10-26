# Phase P17_real_run_fixes: the problems from your first real run, fixed in the backend

## What you can do now
Saving the Capture One session and recipes in Settings now applies at once. Health turns green without a relaunch, and "Recipe missing" no longer appears for recipes that exist. Edited previews render for photos in any capture folder, whichever folder Capture One is showing. The capture folder no longer nests (`_Unassigned/_Unassigned/…`). It is always `Capture/_Unassigned` or `Capture/<folder key>`, so new frames show up in the right session. Every app action now runs in the background, so typing, searching and opening windows no longer wait on Capture One or Shopify. A preview that fails to render is retried after 1, 2, 4, 8 … seconds, up to once a minute, instead of every 2 seconds. When you start something in the app that needs Capture One, such as opening a session, it goes ahead of those background retries. Saving settings while a publish is uploading no longer starts the same upload a second time.

## Try it yourself
1. Run this phase's checks (no camera, Capture One or shop needed):
   ```
   export PATH="$HOME/.cargo/bin:$PATH"
   cd ~/Desktop/Developer/Photo-Session-Manager
   cargo test -p psm-testkit --test p17_real_run_fixes
   cargo test -p psm-desktop p17_tests
   ```
   The first should read `test result: ok. 15 passed` and the second `test result: ok. 2 passed`.
2. Open Capture One on `session test 1`, then start the app:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager/apps/desktop
   npm run tauri -- dev
   ```
   Open Settings, enter the session folder and both recipe names (PSM Preview, PSM Web), and save. Open Health. The document, hooks, recipes and capture folder rows are green, with no relaunch.
3. In Capture One the capture folder now reads `…/session test 1/Capture/_Unassigned`. Create a session for an item and the folder becomes `…/Capture/<folder key>`. Go back to Sessions and it is `_Unassigned` again. Open the session again and it is `…/Capture/<folder key>`, never `…/_Unassigned/<folder key>`.
4. Shoot 3 frames in the open session. Each one appears within about 2 seconds, and its edited preview a few seconds later, even while Capture One shows another folder.
5. While previews render, type in the sessions search and open a session. The window never freezes.
6. To see the slower retries, look at today's log:
   ```
   tail -50 ~/Library/Application\ Support/com.loorenz.sessionmanager/logs/psm.$(date +%F).log
   ```
   A render that keeps failing is retried 1, 2, 4, 8 … seconds apart, not every 2 seconds.

## Choices made for you
- When you save settings, the app builds the new setup first and only then stops the old one. If the new setup fails to start, your previous settings are put back and the app keeps working as before. Nothing to change.
- Background jobs that were already running when you save settings are left to finish, and are not started a second time. Only an actual launch picks up jobs left behind by a crash. This is the correction made after the first review: before it, a save during a publish could upload the same photo twice. Nothing to change.
- What you start in the app and the app's background work share one queue to Capture One, and what you start always goes first. Nothing to change.
- Capture One folders with no disk folder (for example "All Images", "Five Stars") are kept in the app's list of folders and ignored for photo lookup. Nothing to change.
- When the window regains focus, the check for stale previews runs in the background, so the window does not wait for it. Nothing to change.
- If the session's `Capture` folder is missing on disk, Health shows the capture folder as not set and names the path it expected. Nothing to change.

## Not yet
- Buttons and pages do not yet show that something is loading, and item search still asks Shopify on each keystroke. Both come in the next phase.
- The folders that nested during the first run (`Capture/_Unassigned/_Unassigned/…`) are left as they are. Frames inside them are not picked up. Move any you want into `Capture/_Unassigned`, or leave them.
- If you click Save twice very quickly, or save two settings sections at almost the same moment, the second save can undo the first.
- If you remove the Shopify connection while a publish is in progress and then add it back, that publish stays stalled until you relaunch the app.
- A render that times out starts its retries from 1 second again instead of continuing the longer wait.

## Needs you
Nothing.
