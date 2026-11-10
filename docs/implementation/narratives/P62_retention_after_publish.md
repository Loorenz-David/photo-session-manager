# Phase P62_retention_after_publish: a few days after an item is published, its photo files leave this Mac and Shopify becomes its archive

## What you can do now
A few days after you publish an item (4 by default), Session Manager deletes that item's files from this Mac: its RAWs, the Capture One settings and cache files named after them, its duplicates, masters, exports and previews. This happens only if the item is still exactly as published and you have not pinned it. Before deleting anything, the app asks Shopify for the item's images and checks that every image it published is still there. If one is missing, or Shopify cannot be reached, nothing of that item is deleted and the Health page says `Not cleared: <SKU> — Shopify is missing N images`. A cleared item shows **Archived in Shopify** in the list. Opening it shows its photos as they are on Shopify, and you can no longer change anything that needs the photos; its details still save. A **Keep files on this Mac** checkbox keeps an item's files for good. When the disk is still too full after the earlier clean-up steps, the app also clears published items early, oldest publish first. It never touches an item that is unpublished, changed since publishing, or pinned.

## Try it yourself
1. Run this phase's checks. They use a stand-in Shopify and temporary folders, so nothing on your disk or in your store is touched:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager
   cargo test -p psm-testkit --test p62_retention_after_publish
   cd apps/desktop && npx vitest run src/features/workspace/__tests__/p62_archived_ui.test.tsx
   ```
   You should see `12 passed; 0 failed`, then the archived-view checks passing.
2. Start the app with your test store configured:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager/apps/desktop && npm run tauri -- dev
   ```
3. In **Settings → Storage**, set **Keep files of published items (days)** to `0` and press **Save days**.
4. Publish a test item to your test store. A few seconds after the publish finishes, its RAWs disappear from its folder in the Capture One session's `Capture` folder, along with their Capture One files. The app's log shows `storage.item_cleared item=<SKU>` and `storage.cleared rule=retention …` lines.
5. The session list now marks the item **Archived in Shopify**. Open it: its photos come from Shopify, main photos first, then condition photos. Reordering or reviewing is not possible, and an undo or publish attempt says "This item's files were cleared after publishing". **Edit details →** still opens the form, and the description still saves.
6. With the archived item open, take a shot or import a file. It lands in **Unassigned**, not in the archived item, and it is not deleted.
7. Open another published item, press **I** and tick **Keep files on this Mac**. It keeps its files however old it gets.
8. To see the safety check: publish an item, delete one of its images in the Shopify admin, and wait for the clean-up. Nothing is deleted, and Health shows `Not cleared: <SKU> — Shopify is missing 1 images`.
9. Set the days back to `4` afterwards.

## Choices made for you
- "Published" means the last publish finished successfully. An item whose latest republish failed is not cleared. The days count from the last completed publish. Say "count from the first publish" to change that.
- "Every published image" means each image of the main and condition photos sent in the last completed publish. An image Shopify returns without an address counts as missing.
- When Shopify cannot be read at all, Health says `Not cleared: <SKU> — Shopify could not be read` rather than giving a made-up count.
- A RAW that another, not-yet-cleared item still uses (through "Copy to session") is kept, with its Capture One files and duplicates, until that item is cleared too.
- An item's duplicates are the copies whose bytes are exactly those of one of its RAWs.
- Capture One files go only when they are named exactly after one of the item's RAWs. The Capture One catalogue itself is never edited, so those images show as missing there.
- The item becomes read-only right after the Shopify check and before its files are deleted, so no edit can slip in between.
- After a failed Shopify check, the app tries that item again after 15 minutes. It checks at most 5 items at a time and deletes 20 files per batch.
- If the app is quit or crashes in the middle of clearing an item, it finishes that item after the next start. It looks for leftovers at most every 5 minutes. (This was the first fix after review.)
- The clean-up only ever deletes the files an item had at the moment it was cleared. A shot or import made while an archived item is open goes to Unassigned, and the clean-up never deletes it. (This was the second fix after review. Before it, such a new photo could have been deleted within minutes.)
- A shot taken while an archived item is open is still saved on disk in that item's folder, though the app lists it under Unassigned. Say "put those files in the Unassigned folder" if you want them moved.
- Shopify images are downloaded once, at 1600 px on the long side, the first time you open a cleared item. If a download fails, a placeholder shows and the app tries again next time.
- "Read-only" covers everything that needs the photos: review, reorder, delete, move or copy (from or into the item), composition, undo/redo, publish, local export and the condition-photos field. Other details save on this Mac, but they cannot be published, because republishing needs a new photo session.
- The pin sits in the item details card (key I). A cleared item shows no pin. The "Archived in Shopify" label looks like the "Published" label with a dashed border.
- Saving the minimum free space no longer resets the retention days, and the reverse.

## Not yet
- None of this has run against your real Shopify store. The query that reads an item's images back from Shopify has only been tested against a stand-in. Steps 3–8, on your test store, are the first real run. Do them before using this on your work Mac.
- Opening a cleared item for the first time waits while all of its images download, one at a time.
- If you put a deleted RAW back into an archived item's folder yourself, for example from a backup, the clean-up removes it again within minutes. There is no "restore" yet.
- While anything is publishing, the whole clean-up waits.

## Needs you
Nothing.
