# Phase P16_operations: a settings screen, a health screen, automatic backups, daily logs that never show your secret, and the finished app

## What you can do now
A gear icon in the header opens Settings. There you set the Capture One session folder, the recipes and style, the export folder and the preview refresh delay, and the screen shows the delay actually in effect (10 to 30 seconds). You also enter the Shopify shop, client ID, client secret and condition-photos field. The secret is shown only as four dots plus its last four characters, and saving keeps it. Changes take effect as soon as you save, with no restart. From Settings you reach Health. It lists what works and what doesn't (Capture One document, hooks, recipes, capture folder, camera, whether a Shopify token can be obtained, whether the condition-photos field exists), each with a red or green dot and a Retry checks button. The app copies its database to a backup folder every time it starts and after each published session, and keeps the newest 30. Each day gets its own log file, with every Capture One and Shopify call and its duration. Your client secret and access tokens are replaced by `[REDACTED]`. The finished, signed `Session Manager.app` builds, and finding a photo in a Capture One folder now works on the real app.

## Try it yourself
1. Run this phase's checks (no camera, Capture One or shop needed):
   ```
   export PATH="$HOME/.cargo/bin:$PATH"
   cd ~/Desktop/Developer/Photo-Session-Manager
   cargo test -p psm-testkit --test p16_operations
   cd apps/desktop && npx vitest run src/features/settings/__tests__/p16_settings.test.tsx
   ```
   The Rust line should read `test result: ok. 11 passed` and the screen checks `Tests  2 passed (2)`.
2. Start the app:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager/apps/desktop
   npm run tauri -- dev
   ```
   Click the gear icon. Set the preview refresh delay to 20 and check that "In effect" shows 20 s. Choose another export folder. Fill in your shop domain, client ID and client secret, then click Save settings. A "Settings saved" toast appears. Open Settings again and the secret shows as four dots and its last four characters.
3. Click Health. With correct credentials, the Shopify token row is green. The condition-photos row turns green once the app has read your shop's fields, which it does in the background a few seconds after launch or after a save. If it is still red, wait a moment and click Retry checks to run everything again.
4. Look at the backups and logs:
   ```
   ls ~/Library/Application\ Support/com.loorenz.sessionmanager/backup/
   tail ~/Library/Application\ Support/com.loorenz.sessionmanager/logs/psm.$(date +%F).log
   ```
   You should see one backup per launch (and one per published session), and log lines with `op=` and `duration_ms=`. Searching the logs for your secret finds nothing: `grep -r '<your secret>' ~/Library/Application\ Support/com.loorenz.sessionmanager/logs/`.
5. Build the finished app and open it:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager/apps/desktop
   npm run tauri -- build --bundles app
   open "src-tauri/target/release/bundle/macos/Session Manager.app"
   ```
   To use it on the work Mac, copy the app there and open it with right-click → Open the first time.

## Choices made for you
- Saving settings restarts the app's background work over the new settings, so everything applies at once. To get "apply at next launch" instead, say so.
- Saving settings does not make a backup; only launching the app and publishing do. To back up on every save too, say "back up when settings change".
- The light/dark switch stays on the header and is not repeated in Settings. To add it to the form, say "theme in settings".
- When the condition-photos field is missing, Health says "Condition photos field … not found in the shop", and "… is not a file list" when the field has the wrong type. Nothing to change.
- Startup time was measured from launch to the first window. The very first launch after a build took 16 seconds while macOS checked the new app; later launches took about 0.6 seconds. Nothing to change.
- A backup is made after each published session, so a batch of five sessions makes five backups. With 30 kept, a big batch can push older launch backups out. To back up once per batch instead, say "one backup per publish batch".
- Health is its own page, reached from Settings, and Settings is reached from the gear. Nothing to change.
- The old "Capture One" line on Health only repeated the document line, so it is gone. Nothing to change.

## Not yet
- The time from shot to on-screen preview (original and adjusted) is not measured yet, because the camera was not tethered during this phase. It needs one session with the camera attached.
- Right after launch or a save, Health can briefly show the condition-photos field as missing, until the background read of your shop's fields finishes. It also shows it as missing if that read failed.
- If Shopify is slow, the Health check can make the window wait until the shop answers.
- Keychain storage for the secret, automatic updates and the phone remote come after the MVP.

## Needs you
Nothing.
