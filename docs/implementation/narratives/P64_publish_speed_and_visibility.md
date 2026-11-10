# Phase P64_publish_speed_and_visibility: publishing is fast, explains itself, can be cancelled, and includes photos you have not reviewed yet

## What you can do now
Publishing no longer stalls on condition photos. Before, the app looked through every file in your shop to find each photo it might already have uploaded. Now it asks Shopify for that one file by name. Photos are reserved together, uploaded four at a time, and added to the product in groups instead of one by one. Hover over or click the **Publishing n%** bar, in the session list or the session header, to see what is happening: the current step (for example `Uploading main photos 3 of 8`), the percentage, the elapsed time, and the last error with whether it is being retried. **Cancel publish** in that popover stops the publish after the request in progress. Photos you have not reviewed are now published after your approved main photos, and a session is **Ready · Publish** without every photo being reviewed.

## Try it yourself
1. Run this phase's checks. They use a stand-in Shopify, so nothing in your store is touched:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager
   cargo test -p psm-testkit --test p64_publish_speed_and_visibility
   cd apps/desktop && npx vitest run src/features/publish/__tests__/p64_publish_progress.test.tsx
   ```
   You should see `8 passed; 0 failed`, then 5 passing popover checks.
2. Start the app with your test store configured:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager/apps/desktop && npm run tauri -- dev
   ```
3. In a session with about 8 photos, approve some as main, leave 2–3 unreviewed and mark 3 as condition photos. The row reads **Ready · Publish**, and the publish window says the unreviewed photos "will be published after the main photos".
4. Publish. Hover over the **Publishing n%** bar. The popover names each step as it happens. The whole publish should take well under a minute.
5. In the Shopify admin, the product shows your approved photos first, then the unreviewed ones. The condition photos are in the damage report field.
6. Publish another session and press **Cancel publish**, then **Stop publish**. The row changes to **Publish cancelled · Publish**. Press **Publish** again. It continues from what was already uploaded and ends with the full gallery and no duplicates.

## Choices made for you
- Unreviewed photos have no position of their own, so they follow the main photos in the order the workspace shows them, which is their arrival order. Say "order unreviewed photos by …" to change that.
- A session is Ready once it has at least one main or unreviewed photo and every photo that is not deleted, condition photos included, has its adjusted preview. Say "condition photos don't need a preview" to drop that.
- Uploads are added to the product in groups of four. If you cancel at "3 of 8", the 4th photo is usually already on Shopify, so the next publish uploads 4 photos, not 5. Uploading it again would create a duplicate.
- Photos that finished uploading but were not yet added to the product when you cancelled are uploaded again next time. Nothing for them exists on Shopify yet, so this cannot create a duplicate.
- If you cancel just as the last step is finishing, the publish completes as Published.
- A cancelled publish is not a separate status. The session counts as Ready (or In progress) in filters, and only its row reads "Publish cancelled". Say "give cancelled publishes their own filter" if you want one.
- The cancel request and the elapsed timer are kept in memory. If you quit the app right after pressing Cancel, the confirmed publish resumes on the next start. The elapsed time restarts when you press Retry.
- Up to 50 photos are reserved per request, and 4 upload at once. The popover opens when you click or hover over the bar.
- The grouped requests to Shopify (several photos per reservation, per product update and per file creation) follow Shopify's documentation but have not yet run against a real store. Your first publish is their first real test.
- If the app is killed in the split second after Shopify creates a condition photo but before the app records it, and Shopify's file search has not yet indexed that photo, the photo may be uploaded once more. The extra copy stays in Shopify's Files list only, never on the product. Looking up files by name is what made publishing fast.

## Not yet
- The popover is only on the **Publishing** bar. Once a publish has failed, the row shows **Publish failed · Retry** without the popover, so the "Stopped · press Retry" message does not appear there. The error shows in the popover again once you press Retry.
- The popover opens on hover but does not close when the mouse leaves. Click elsewhere to close it.
- The full test suite was not run end to end after this change because this Mac was very slow. Every publishing-related suite passed, and the camera, composition and storage suites were not touched. One full run on a quiet Mac is still owed.

## Needs you
Nothing.
