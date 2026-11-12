# Phase P69_faster_export_and_publish: faster publishing — lighter masters made in advance, batches that overlap, and a timing line per item

## What you can do now
Capture One now makes the full-size master of each photo as an 8-bit TIFF instead of 16-bit. That is about half the file (≈ 180 MB instead of ≈ 360 MB per a7R V photo), and the published JPEG is still made once from it and compressed once. Once a session is Ready, the app quietly asks Capture One for its masters in the background, one photo at a time, only while Capture One has nothing else to do. When you then publish, those photos skip the Capture One wait. In a batch, Capture One now renders the next item's masters while the current item uploads, instead of asking for every item's masters at once. That all-at-once request is the likely cause of the two "export failed: timeout" items in your 9-item batch. Each published item writes one line to the app log with how long each step took, and the publish progress popover shows "Masters ready in advance: n of m".

## Try it yourself
1. Run this phase's checks. They use a stand-in Capture One and a stand-in shop, so nothing real is touched:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager
   cargo test -p psm-testkit --test p69_faster_export_and_publish
   cd apps/desktop && npx vitest run src/features/publish/__tests__/p69_masters_ready.test.tsx src/features/settings/__tests__/p69_prepare_masters_setting.test.tsx
   ```
   You should see `7 passed; 0 failed` (about a minute), then 3 passing checks.
2. Start the app:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager/apps/desktop && npm run tauri -- dev
   ```
3. Open Settings → Storage. "Prepare masters in advance" is ticked.
4. Shoot or open a session until its row says Ready, then leave the app alone for a minute. Masters appear one after the other in this folder:
   ```
   open ~/Library/Application\ Support/com.loorenz.sessionmanager/masters/
   ```
   The app log shows "master asked for in advance" and "master prepared in advance".
5. Publish that session. While it runs, the progress popover shows "Masters ready in advance: n of n". Afterwards, the log has one line starting `publish.timing item=<your SKU>` with the time spent on masters, outputs, upload and Shopify.
6. Select three Ready sessions that have not been prepared yet (for example right after turning the setting off and on again), choose Review & publish and confirm. While the first item uploads, Capture One is already rendering the second item's masters.
7. Untick "Prepare masters in advance". New Ready sessions then get their masters only when you publish them.

## Choices made for you
- Compression stays "Uncompressed". Earlier measurements showed ZIP took three to four times as long for a file only 15 % smaller, and Capture One offers nothing faster that is lossless. Say "try ZIP masters" to change it.
- "Capture One is idle" means: no photo on its way from the camera, no camera take-back, no render out, no preview still to be made, and no export or publish running for any session. Free space must also be above your storage minimum, so preparing and the space clean-up never fight. Say "keep preparing while another session publishes" to loosen it.
- The newest Ready session is prepared first. Within a session, the order is main photos by position, then condition photos.
- If Capture One is closed or refuses, the app tries again after 30 seconds. A master that fails for good isn't asked for again until the photo is edited, and publishing makes it as usual.
- A batch is the set of items you confirmed together. Items go at most one ahead: an item waits until the one before it has its outputs ready, and while it waits it shows the usual "preparing" text.
- In the timing line, "masters" is the wait for Capture One, "outputs" is the app's own framing and compression, and "shopify" is finding or creating the product plus the final update. "Masters reused" counts photos, not files. A retry starts the clock again.
- The "Masters ready in advance" line only shows while the item is publishing.
- A master made in advance is checked against the photo's frame just as during a publish. A photo whose frame no longer fits can therefore be flagged for review before you publish, rather than during the publish.

## Not yet
- Nothing here has been tried with the real Capture One or the camera yet. Steps 4 to 6 are the first real checks. They are also the moment to compare publish times against your 9-item batch.
- If you plug the camera cable in while a master is being made in the background, taking the camera back waits for that one master to finish. Meanwhile the header says "an export is running". This has been noted for a later fix: the wording, and pausing the background job sooner.
- The design book still describes the master as 16-bit and made only when publishing. It will be updated to match.

## Needs you
Nothing.
