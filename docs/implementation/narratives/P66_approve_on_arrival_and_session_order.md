# Phase P66_approve_on_arrival_and_session_order: photos arrive approved, Deny deletes the RAW after the undo window, and the sessions list keeps its creation order

## What you can do now
Every new photo now joins its session already approved, at the end of Main, in the order it arrived. You can reorder at once, and a well-shot session can be published without reviewing a single photo. Instead of approving, you mark the exceptions: **C** or swipe up sends a photo to Condition, **⌫** or swipe left (or "Deny" in the photo's ⋯ menu) denies it, and the next photo in order opens. A denied photo is hidden and never published. ⌘Z brings it back until the session's next publish completes or 24 hours have passed, whichever comes first. After that the app deletes its RAW and its own preview files, and the photo can no longer be restored. The sessions list stays in the order you created the sessions, newest on top. Opening, editing, shooting into or publishing a session no longer moves it.

## Try it yourself
1. Run this phase's checks. They use a stand-in Capture One and a stand-in shop, so nothing real is touched:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager
   cargo test -p psm-testkit --test p66_approve_on_arrival_and_session_order
   cd apps/desktop && npx vitest run src/features/workspace/__tests__/p66_approve_on_arrival_ui.test.tsx
   ```
   You should see `6 passed; 0 failed`, then 6 passing workspace and list checks.
2. Start the app with Capture One open on your session:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager/apps/desktop && npm run tauri -- dev
   ```
3. Open an item's session and make it the active one. Shoot three photos. They appear as Main 01, 02, 03 straight away. In Capture One each gets the green tag once its preview is ready.
4. With a photo on the stage, press **C**. It moves to Condition and the next photo opens.
5. Press **⌫** on another photo. "Photo denied · Undo ⌘Z" appears. Press ⌘Z and the photo is back in its place.
6. Use a test item whose RAWs you can lose. Deny one photo again and publish the session. A few seconds after the publish completes, that RAW is gone from the item's folder, and Capture One shows the variant as missing. ⌘Z no longer brings it back. The published product has only the other photos.
7. Go back to the sessions list. The order is the order you created the sessions, newest on top. The column reads "Created", and the time filter (sliders icon) filters by the day each session was created.

## Choices made for you
- Photos landing in Unassigned also arrive approved. Say "Unassigned photos should stay unreviewed" to change that.
- The green tag is written once Capture One has delivered the photo's first preview, usually seconds after the shot, not at the very moment it lands. Writing it earlier made the app drive Capture One even when Capture One was closed. Say "tag at the instant of arrival" to revisit.
- Photos you deleted before this change keep their RAW. Only photos denied from now on are deleted.
- Capture One's own files for a denied RAW (its settings file and cache) and the app's safety copies of duplicates are left in place. Only the RAW and the app's previews, masters and outputs are deleted. Say "also delete Capture One's files of denied photos" to change that.
- If you copied a photo to a second session, its RAW is deleted only once every copy has been denied and each copy's undo window has closed.
- The undo list still reads "Delete 03" for a deny. Every button, menu entry and message says "Deny". Say "rename it in the undo list too" to change that.
- Each row in the sessions list shows when the session was created, to match the order and the groups.
- A publish that was already running when you denied the photo also closes the undo window when it completes. The denied photo is still left out of that publish.
- "Keep files" on an item does not protect a denied photo. Deny is your explicit choice.
- The app checks for closed undo windows every two seconds, so the RAW goes within seconds of the publish completing.
- When you open a session that still has photos from before this change that were never reviewed, the first of those opens first, as before. Otherwise the first Main photo opens.

## Not yet
- Nothing here has been run against your real camera, Capture One or shop. Step 6 is the first real deletion. Try it on a test item first.
- Deleting a denied photo's files also removes from the undo list any group action that included it, such as moving several photos to Condition at once. The other photos in that action can then no longer be undone. This is recorded for a later fix.
- The design book still describes the old flow (Approve on arrival, Delete keeps the RAW). The orchestrator updates it.
- The full test run on this Mac was the implementer's. The review re-ran the new checks, the checks for every part this change touches, the formatting check and the type check, and all passed. The review's own run of the code-style check (clippy) stalled twice without doing any work, so it relies on the implementer's clean run on this exact code. The next full run of the checks will confirm it.

## Needs you
Nothing.
