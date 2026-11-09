# Phase P60_take_back_and_self_healing: taking the camera back runs cleanly, and photos you delete by hand no longer break the app

## What you can do now
When you plug the camera in with Capture One open, Session Manager quits Capture One. It now keeps it closed until the cable session is up, then reopens it once, so the "Capture One did not quit" failure you saw in the evening check should not happen again. If a Capture One window (for example its sign-in window) stops it from quitting, the camera card now says `Capture One has a window open — close it so Session Manager can take the camera`. Close the window and the take-back finishes on its own. If macOS's camera service keeps hold of the cable after Capture One has quit, the card now says so and tells you to restart the Mac, instead of blaming Capture One. Photos whose RAW you deleted in Finder now drop out of the session, are never sent to Capture One again, never hold up the take-back, and are counted on the Health page as "N photos' files were deleted outside Session Manager". Put a file back and its photo returns. Capture One reopening no longer re-renders previews that are already up to date. If its `PSM Preview` recipe is deleted or switched to TIFF, the app repairs it and renders once more.

## Try it yourself
1. Run this phase's checks. They use a stand-in Capture One and camera, so nothing touches your real ones:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager
   cargo test -p psm-testkit --test p60_take_back_and_self_healing
   cd apps/desktop && npx vitest run src/features/shell/__tests__/p60_header_sentences.test.tsx
   ```
   You should see `8 passed; 0 failed` and then `2 passed`.
2. Start the app:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager/apps/desktop && npm run tauri -- dev
   ```
3. With Capture One open, plug in the camera. Capture One should close, the dot should read Cable, and only then should Capture One reopen, about 7–8 seconds in all.
4. In Finder, delete one RAW from the session's capture folder, then click back into Session Manager. The photo should disappear from the session. Health should show a "Deleted files" item reading "1 photos' files were deleted outside Session Manager", and Capture One should not keep trying to render it. Undo the delete in Finder (⌘Z), click back into Session Manager, and the photo should be back.
5. Open Capture One's sign-in or preferences window, then plug in the camera. The camera card should ask you to close the window. Close it, and the take-back should finish without a click.
6. In Capture One, change the `PSM Preview` recipe's format to TIFF, then take a photo. Its preview should still arrive, and the recipe should be back on JPEG.

## Choices made for you
- A photo whose file is gone still counts for export and publish. If it was a main or condition photo, its item stops at export or publish with a "file missing" error, and you won't see that photo in the session to change it. Restoring the file fixes it. Say "leave photos with deleted files out of export and publish" to change this.
- "Missing" uses the photo's existing file-missing mark. Nothing new is stored, and the photo's classification and history stay as they were.
- A file that's there but can't be read (for example, a permissions problem) counts as present, and the problem goes to the log.
- The app looks for deleted files at startup, whenever you switch back to the app, when Capture One opens or reopens, and when a render runs into a missing file. There is no clock-driven background scan. Say "check every minute" if you want one.
- If a file comes back with different bytes, its photo returns marked "RAW changed", as for any edited RAW.
- While the app takes the camera back, it waits up to 30 seconds for anything it was already asking Capture One to finish before it quits Capture One.
- Previews are not sent to Capture One while it is closed and waiting for the camera helper. They are tried again shortly after and never count as failures.
- With a Capture One window blocking the quit, the app asks Capture One to quit again every 2 seconds, waiting up to 20 seconds each time.
- The stuck-camera-service message appears 10 seconds after Capture One quit if the cable is still claimed and no camera app is running. It stays until you unplug the camera or the cable session starts.
- Before each batch of renders, only that render's own recipe is checked and repaired. A broken `PSM Master` does not hold up previews.
- A preview that comes out in a format other than JPEG is never used. The recipe is repaired and the render sent once more. If the second one is wrong too, that preview fails. Masters are not checked, since they are TIFF on purpose.
- The Health page names the new item "Deleted files" and uses the sentence exactly as planned, including "1 photos'". Say "write 1 photo's" to fix the grammar.

## Not yet
- None of this has run on your Mac with the real camera and Capture One yet. Steps 3–6 are for the next hands-on run.
- The rest of the storage work comes in the next two phases: cleaning up duplicates and old preview exports, a storage budget, and clearing published items' RAWs. That clearing must not show up as "deleted outside Session Manager". This is noted for the phase that does it.
- While a Capture One window stays open, the automatic take-back keeps retrying, and previews mostly wait until you close the window.
- If a take-back you clicked fails because a Capture One window is open, the message can stay up after you close the window, until you try again or unplug the camera.
- The check for deleted files runs on every switch back to the app. With a very large library that may add a short delay. This will be timed on your work Mac.

## Needs you
Nothing.
