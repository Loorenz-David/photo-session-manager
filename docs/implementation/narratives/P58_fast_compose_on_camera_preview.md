# Phase P58_fast_compose_on_camera_preview: reframe a photo the moment it arrives, and the photo you are looking at is rendered first

## What you can do now
You no longer have to wait for Capture One before reframing a new photo. As soon as a photo from your a7R V arrives, Compose works on the camera's own full-size preview. The editor shows one quiet line, `Camera preview — Capture One's version is on its way`. When Capture One's version arrives, the picture swaps to it in place. Your framing stays where you put it; it only moves if it would otherwise stick out past the photo's edge. The stage and the grid tile show your framing on the camera preview until then. Exports and Shopify uploads still always use Capture One's own render, never the camera preview. Capture One now also gets its preview work in a sensible order: the photo you have open goes first, then the newest arrivals (newest first), then everything else oldest-first.

## Try it yourself
1. Run this phase's checks. They use stand-ins, so no camera or Capture One is needed:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager
   cargo test -p psm-testkit --test p58_fast_compose_on_camera_preview
   cd apps/desktop && npx vitest run src/features/compose/__tests__/p58_compose_on_camera_preview.test.tsx
   ```
   You should see `0 failed` and `6 passed`.
2. Start the app as usual with the camera on Wi-Fi and your Capture One session open:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager/apps/desktop && npm run tauri -- dev
   ```
3. Take a photo. As soon as it appears, open it and press R. The editor should open on the camera preview and show the line at the bottom.
4. Reframe it. A few seconds later the picture should refresh to Capture One's version. The line should disappear, and your frame should stay where you put it.
5. Take ten photos quickly, then open the last one. Its Capture One version should arrive before the older ones'.
6. Optional: with the ZV-E10, the camera preview is a tighter crop. Compose should stay off with `Waiting for Capture One's full-frame preview` until Capture One's version arrives.

## Choices made for you
- The camera preview is used only when it shows the whole photo: same shape as the camera's sensor within 1 %, and at least 1600 pixels on its long side. The a7R V's does; the ZV-E10's tighter crop does not. Say "use a different threshold" to change this.
- The sensor's shape is read from the size the camera writes into the RAW file for this purpose. A padded storage size would have wrongly turned the feature off for every a7R V photo.
- A "new arrival" is a photo whose first preview has not yet been sent to Capture One. The newest of those goes first.
- Capture One now gets preview requests one at a time, so the order above can take effect. Export renders are still sent at once, as before.
- The app tells Capture One which photo is on the stage or in the editor, and stops when you leave the workspace.
- If Capture One shows a photo the other way up from the camera's preview, a frame you have not touched is simply made again. A frame you set yourself is marked for you to check.
- The line sits at the bottom centre of the editor.
- Photos that arrived before this update keep waiting for Capture One, as before.
- What the fix round changed: a photo whose camera preview does not show the whole photo could become composable if an export happened to finish before its first preview. The editor would then have opened on the cropped camera picture. Compose now uses the camera preview only when the photo's frame was measured from that preview. The check for "newest arrivals first" now feeds real captures through the same path the camera uses.

## Not yet
- Nobody has tried this with the real camera and Capture One yet. The steps above are for the next hands-on run.
- Side effect of the fix: if you export a photo before Capture One's first preview of it is back, Compose turns off for that photo until the preview arrives, with the usual waiting sentence. If the editor is open on that photo at that moment, it closes. Opening the photo puts its preview next in line, so the wait is usually one render. Your framing and the export are not affected.
- Because previews now go to Capture One one at a time, a very long re-render (for example after a settings change) may take longer overall. The photo you are looking at still comes first.
- After an app restart, the "newest first" memory is gone. Photos still without a preview then wait in the normal oldest-first order.
- If Capture One's orientation setting disagrees with the camera's, every portrait you reframed early will ask to be checked again.

## Needs you
Nothing.
