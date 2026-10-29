# Phase P21_backend_robustness: the app always starts, saves never clash, and the shop and Health tell the truth

## What you can do now
The app now opens even if its settings file is damaged, half-filled or cannot be backed up. It falls back to safe defaults, keeps working without the shop (Export instead of Publish), and Health shows a red row that says what is wrong. Pressing "Save settings" repairs a damaged settings file, and two saves made right after each other both keep their changes. When the shop cannot be reached, the shop search reports an error instead of claiming "0 in the shop", so you won't make a duplicate item by mistake. Health now reads the shop's fields before it judges the condition-photos field. Offline it says "shop fields not read yet" instead of a false "not found". Publishing a session again no longer re-sends a description you did not change. The bell's entry no longer shows a long number. You can now clear an item's barcode (the button for that arrives in the next phase).

## Try it yourself
1. Run this phase's checks. No camera, Capture One or shop is needed:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager
   cargo test -p psm-testkit --test p21_backend_robustness
   cargo test -p psm-desktop p21_tests
   ```
   The first should read `13 passed`, the second `1 passed`.
2. Quit the app. Make a copy of your settings file, then open it in a text editor:
   ```
   cd ~/Library/Application\ Support/com.loorenz.sessionmanager
   cp settings.json settings.backup-copy.json
   open -e settings.json
   ```
   Delete the very last `}` and save. Start the app (`cd ~/Desktop/Developer/Photo-Session-Manager/apps/desktop && npm run tauri -- dev`). It opens on the Sessions list. Your settings are at their defaults for now, so re-pick your Capture One session and export folder in Settings. Settings → Health shows a red settings row naming `settings.json`. Go back to Settings, press "Save settings", then Health → "Retry checks": the row is gone and the file is valid again. If you prefer your old settings back, quit the app and run `cp settings.backup-copy.json settings.json`.
3. With a shop configured, quit the app and delete the `"client_secret"` line inside the `integration` part of the same file. Start the app. The session header shows Export instead of Publish, and Health shows a red settings row naming the missing field. In Settings the shop fields are now empty: type the shop domain, client ID and secret again, and save. Publish comes back without a restart.
4. In Settings, change the preview recipe and press Save, then change the export folder and press Save straight away. Restart the app: both changes are there.
5. Turn Wi-Fi off. Start a new session and type a title: the heading says "shop unavailable", not "0 in the shop". Open Health: the condition-photos row says "shop fields not read yet" if the app has never read your shop's fields. Otherwise it stays green. Turn Wi-Fi back on.
6. Publish one session twice without touching its description. Today's log in `~/Library/Application Support/com.loorenz.sessionmanager/logs/` shows only one description update. The bell's entry reads just "Published". The time next to it arrives in the next phase.

## Choices made for you
- A damaged settings file is replaced by the app's defaults in memory until you press Save. The damaged file is left on disk untouched so it can be inspected. Say "keep a copy of the damaged settings file" if you want the app to set it aside when it repairs it.
- If the shop's fields cannot be read during a Health check, the fields the app read earlier still count. Only when it has never read them does Health say "not read yet". Nothing to change.
- A "backup failed" row stays visible until a backup works again, even after you save settings. Nothing to change.
- Saving only the light/dark theme also clears a "settings file" warning once the file has been rewritten. Nothing to change.
- While the shop part of the settings is incomplete, the app will not save other settings until the shop details are complete again. Saving shows an error naming the missing field. Say "let me save other settings while the shop part is broken" to change this.

## Not yet
- The bell shows "Published" without the time until the next phase adds it.
- The next phase adds the control that clears the barcode on screen and makes the secret field empty itself when you click into it.
- None of this has been tried against your real shop or on the work Mac yet. Steps 3, 5 and 6 are the first real check.
- While the shop part is incomplete, the theme toggle and other settings saves fail until you re-enter the shop details (step 3).

## Needs you
Nothing.
