# Phase P09_local_export: finish a session without a shop, as a folder of web-ready photos

## What you can do now
When no shop is connected, the session header shows an **Export** button. Clicking it first checks that none of the session's RAW files has changed since it was shot. If one has, the export stops before anything is rendered and names that photo. Otherwise Capture One renders every Main and Condition photo through the web recipe, in your order, and the app puts them in a folder named after the SKU: `<SKU>_01_main.jpg`, `<SKU>_02_main.jpg` and so on, then `<SKU>_01_condition.jpg`, plus an `item.json` with the SKU, title, type and the two file lists. Unreviewed and deleted photos are left out. A toast counts the progress and ends with "Exported N files" and an **Open folder** button. Exporting again replaces the whole folder, so a photo you deleted since the last export leaves no old file behind. If something fails, the previous folder stays exactly as it was.

## Try it yourself
1. Run this phase's checks. They use a pretend Capture One, so neither Capture One nor the camera is needed:
   ```
   export PATH="$HOME/.cargo/bin:$PATH"
   cd ~/Desktop/Developer/Photo-Session-Manager
   cargo test -p psm-testkit --test p09_local_export
   ```
   The last line should read `test result: ok. 8 passed`.
2. Open your Capture One session (the "PSM Web" recipe must exist), tether the camera, and start the app:
   ```
   export PATH="$HOME/.nvm/versions/node/v22.22.3/bin:$PATH"
   cd ~/Desktop/Developer/Photo-Session-Manager/apps/desktop
   npm run tauri -- dev
   ```
3. Open a session with a few photos and review every photo: approve it, mark it as condition, or delete it. The right-hand button in the header reads **Export**.
4. Click **Export**. The toast counts "Exporting n of N", then shows "Exported N files". Click **Open folder**. Finder shows `~/Pictures/Session Manager Export/<SKU>/` with the numbered main and condition JPEGs and `item.json`.
5. Back in the app, delete one main photo and click **Export** again. Check what is left, hidden files included:
   ```
   ls -A ~/Pictures/Session\ Manager\ Export/
   ls ~/Pictures/Session\ Manager\ Export/<SKU>/
   ```
   The first list shows only SKU folders. The second has one main file fewer, and its numbers have no gap.

## Choices made for you
- If you delete a photo while an export is running, it is left out and the remaining files are renumbered without a gap. To change this, say "a photo deleted during an export should fail the export instead".
- A photo you move between Main and Condition, or reorder, while an export is running keeps the place it had when you clicked Export. The next export picks up the change. To change this, say "re-read the order at the end of an export".
- `item.json` is written after all photos are in, just before the folder is swapped in. Nothing to change.
- The toast updates once at the start and once after each photo, and the count counts photos only, not `item.json`. Nothing to change.
- **Open folder** opens the folder in Finder with the system `open` command. Nothing to change.
- The app decides between Export and Publish from the settings file: no shop configured means Export. Until the settings screen exists, a configured shop shows a Publish button that does nothing yet. Nothing to change.
- While an export is running, its Export button is greyed out, and a second request returns the export already running. Nothing to change.
- A session with no main or condition photo still exports, and the folder holds only `item.json`. To change this, say "disable Export when there is nothing to export".
- If the app quits during an export, that export is dropped and you click Export again. A hidden `.<SKU>.partial-…` folder can stay behind in the export folder. It is noted for clean-up in the publish phase. Nothing to change now.
- The folder name comes from the product's current SKU. An export made under an earlier SKU stays where it is. Nothing to change.

## Not yet
- `item.json` lists no attribute values yet. Its attributes section is empty until the next phase adds the product details.
- Publishing to the shop, which reuses this export as its first step, comes later.
- No real export through Capture One was run during this phase. Steps 2–5 above are that check.
- Clicking Export within a few seconds of the last shot can fail with "RAW changed", because the app has not yet recorded that photo's fingerprint. Clicking again a moment later works. This is noted for fixing before publishing.

## Needs you
Nothing.
