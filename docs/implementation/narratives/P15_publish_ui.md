# Phase P15_publish_ui: publishing from the app, with a review before anything is sent, live progress, and Retry when it fails

## What you can do now
When a shop is set up, the session header shows Publish instead of Export. Publish opens a review that shows what will be sent: the main and condition thumbnails in order, how many details are filled in, a note about unreviewed photos (they are not published), how many shop photos will be removed, and the exact summary. Pressing "Publish N images" sends the publish and records that same summary as what you confirmed. While it runs, the session row and the header show "Publishing n%", then "Published". If a publish fails, the row says which step failed and offers Retry. In the session itself you get a toast that stays until you act on it, the reason, the line "Nothing was overwritten", and a badge on each photo: "On Shopify", "Uploading" or "Not uploaded". To publish several items, tick their rows and press "Review & publish N". You then go through each form with "Confirm & next", and "Publish N confirmed" shows the same review for the confirmed items only. The bell counts sessions whose last publish failed. Its drawer lists the publish history, with Retry and Open on each entry.

## Try it yourself
1. Run this phase's screen checks. They use pretend data, so no shop or network is needed:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager/apps/desktop
   npx vitest run src/features/publish/__tests__/p15_publish_ui.test.tsx
   ```
   The summary should read `Tests  11 passed (11)`.
2. Run the checks for the new data behind those screens (failure reason, photo badges, failed entries in the history):
   ```
   export PATH="$HOME/.cargo/bin:$PATH"
   cd ~/Desktop/Developer/Photo-Session-Manager
   cargo test -p psm-testkit --test p15_publish_backend
   ```
   The last line should read `test result: ok. 3 passed`.
3. Optional, with your test store. Put the store's `shop_domain`, `client_id` and `client_secret` into the `integration` block of `~/Library/Application Support/com.loorenz.sessionmanager/settings.json`, then start the app:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager/apps/desktop
   npm run tauri -- dev
   ```
   Open a Ready session and click Publish. Check the review, then click "Publish N images". The row shows "Publishing n%", then "Published".
4. Still with the store: switch the Mac's Wi-Fi off and publish another session. You get "Publish failed", a toast that stays, and the reason. Switch Wi-Fi back on and press Retry. It finishes.
5. Tick three Ready rows and click "Review & publish 3". Press "Confirm & next" on the first two. Each gets a ✓ in the strip at the top. Click "Publish 2 confirmed". The review opens on top of the form. Cancel takes you back with both ✓ still there. Open it again and press "Publish N images". The two sessions publish and the third does not.

## Choices made for you
- Unreviewed photos are left out of a publish, and the review says so. This follows your earlier decision. To change it, say "publish unreviewed photos in Main".
- During a failed publish, a photo the shop has no record of yet shows "Not uploaded". Nothing to change.
- Every failed attempt gets its own line in the history, so repeated failures stay visible. To keep only the latest one, say "one history line per failed item".
- When a Retry succeeds, the sticky failure toast goes away by itself. Nothing to change.
- Open on a history entry takes you to that session's workspace. To land somewhere else, say where.
- The bell counts sessions whose latest publish failed, not the number of failure messages. Nothing to change.
- In batch publish, the confirmed items are sent in the order you confirmed them, and each form's "n/N filled" count is refreshed when you confirm it. Nothing to change.
- While a publish request is being sent, pressing Escape or clicking beside the review does nothing, so it cannot end up half closed. Nothing to change.
- Changed in the fixes: rows and the open session now update live while a publish runs. Before, they only changed when you left the page and came back.
- Changed in the fixes: a Retry that fails again now shows the toast and the reason again.
- Changed in the fixes: batch publish now shows the review before sending, and the review opens on top of the form, so its button works. In an earlier fix the review sat behind the form, and clicking it closed the queue without publishing.

## Not yet
- There is no settings or health screen yet, so a shop is still set up by editing the settings file. That comes in the last phase.
- Published entries in the history show their time as a raw number, not a date and time.
- The photo badge always says "On Shopify", whatever shop is set up. That only matters once a second shop type exists.
- In the batch review, Cancel can still be pressed after "Publish N images" has been sent. The publish goes ahead anyway. This is on the tidy-up list.

## Needs you
Nothing.
