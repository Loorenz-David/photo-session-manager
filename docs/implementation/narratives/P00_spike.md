# Phase P00_spike: The app's foundation, and proof it can talk to Capture One

## What you can do now
Session Manager now exists as a real Mac app with a single Health page. It shows where it keeps its
data, whether its Capture One hooks are ready, whether it is listening for Capture One's signals,
and the name of the document open in Capture One. Behind that page, the foundation the whole
product stands on works: when you process or capture a photo in Capture One, the app records the
event, even if the app was closed at the time (it picks it up at the next start). Capture One never
waits on Session Manager. There are no photo or session features yet.

## Try it yourself
1. Open Capture One with your session document (the app asks Capture One for it, and would
   start Capture One itself if it were closed).
2. Start the app from the project folder. The first line selects Node 22, which the frontend tools need:
   ```
   export PATH="$HOME/.nvm/versions/node/v22.22.3/bin:$PATH"
   cd ~/Desktop/Developer/Photo-Session-Manager/apps/desktop
   npm ci
   npm run tauri -- dev
   ```
   A window titled "Session Manager" opens on the Health page.
3. Press **Test Capture One**. The button reads "Testing…". If macOS asks whether Session
   Manager may control Capture One, choose **Allow**. The Capture One row then shows your
   document's name and "checked" with the time.
4. Quit the app (⌘Q) and start it again with `npm run tauri -- dev`. You should **not** be asked
   again, and the document name should appear again.
5. Build the stand-alone app and open it:
   ```
   npm run tauri -- build --bundles app
   open "src-tauri/target/release/bundle/macos/Session Manager.app"
   ```
   The first time, right-click it in Finder and choose Open if macOS refuses. Repeat step 3 there
   too.
6. Look at what the app created:
   ```
   ls -l ~/Library/Application\ Support/com.loorenz.sessionmanager/
   ```
   You should see `session_manager.db`, `settings.json` (readable only by you), and the folders
   `hooks`, `spool`, `logs`, `previews` and `backup`.

## Choices made for you
- The "checked" time on the Health page uses your Mac's local time. Say "show it in UTC" to change it.
- The Health page checks Capture One by itself when it opens, and again each time you press the
  button. If Capture One is closed, that check starts it. Say "only check when I press the button"
  to change it.
- The app keeps a note of every signal from Capture One it has already recorded, so a crash at the
  wrong moment can never record the same photo twice. Nothing to change. Later this note moves to
  a tidier place.
- If macOS refuses the fast folder-watching service, the app falls back to checking the folder
  ten times a second. Say "fail loudly instead" to change it.
- The app icon is a placeholder. Send an image to replace it.
- The frontend tools run on Node 22, because the newer Node on this Mac fails to install them.
  Nothing to change.

## Not yet
- Nobody has yet seen the macOS permission prompt for controlling Capture One with the corrected
  check. Steps 3–4 above are that check. Signals from Capture One themselves already reached the
  app in the earlier live test.
- The app has not been tried on the work Mac (no second Mac was available).
- Watching your real capture folder for new photos comes with capture arrival, later.
- Everything about items, sessions, photos, previews, exporting and Shopify comes in the next phases.

## Needs you
Nothing. (The one thing still waiting on you is the permission check in steps 3–4 above.)
