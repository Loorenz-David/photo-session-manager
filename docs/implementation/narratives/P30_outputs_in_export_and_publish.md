# Phase P30_outputs_in_export_and_publish: publishing now uploads the app's own crops, two pictures per split photo, and replaces a photo on the shop when its framing changes

## What you can do now
Publishing to Shopify now uploads the same square JPEGs that Export makes, the app's own crops at 2048 pixels. A photo set to "split" becomes two pictures, left then right, side by side in the gallery and in the damage report. Later photos move down one, and in the export folder the split takes two numbers. Uploads are named after the item and the photo (`K-<photo>.jpg`, and `-L`/`-R` for the halves). If you change a photo's framing after publishing, the next publish uploads it again as `-r1`, `-r2` and so on, puts it in the same place and removes the old one. If you switch a split photo back to a single picture, the two halves leave the shop and one picture comes back in the same place. Publish refuses, and names the photos ("Re-frame first: Main 02"), while any photo is flagged for re-framing. After a publish finishes and the session has no unpublished changes, the large master files for that session are deleted.

## Try it yourself
1. Run this phase's checks (no camera, Capture One or Shopify needed):
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager
   cargo test -p psm-testkit --test p30_outputs_in_export_and_publish
   ```
   All 21 should pass.
2. With Shopify set up and Capture One open, start the app, open a session with a few main photos, and publish it to your test store:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager/apps/desktop
   npm run tauri -- dev
   ```
   The product's gallery shows the square pictures in order, and the cover is main 01. Capture One renders one TIFF per photo during the publish. Afterwards the session's master files are gone:
   ```
   ls ~/Library/Application\ Support/com.loorenz.sessionmanager/masters/
   ```
3. Publish again without changing anything. Nothing is uploaded and the run finishes at once.
4. Make one photo a split by hand (the framing editor comes later). Find its id, then set it:
   ```
   DB=~/Library/Application\ Support/com.loorenz.sessionmanager/session_manager.db
   sqlite3 "$DB" "select id, uid from photos order by id desc limit 5"
   sqlite3 "$DB" "update compositions set kind='split', revision=revision+1, x=0, y=0.1251, w=1, h=0.7499 where photo_id=<id>"
   ```
   Publish. That photo becomes two square pictures, left then right, next to each other, and the photos after it move down one.
5. Set it back the same way (`kind='normal'`, `revision=revision+1`) and publish. The two halves leave the gallery and one picture returns in the same place.
6. Rotate a photo 90° in Capture One and wait for its preview. Publish is refused with "Re-frame first: Main 0n", and so is Export.

## Choices made for you
- If a publish crashed or lost the connection after a picture had already reached the shop, and you then re-frame that photo, the next run now uploads the new framing as `-r1` and deletes the half-finished old picture. Before, the old framing could stay on the shop while the app believed it was current. In that situation the picture is always uploaded as `-r1`, even if the old upload never really arrived, because the app cannot tell. This is what the fix changed. Say "keep the plain name when nothing reached the shop" if the `-r1` bothers you.
- Upload names keep the item's folder key in the case you typed it (`K-…`, not `k-…`), with `-L`/`-R` in capitals. Pictures uploaded in lowercase before this phase are still recognised. Say "lowercase upload names" to change this.
- If a photo changes while a publish is uploading, the run notices that what it staged is out of date and makes the files again before uploading anything.
- If the export inside a publish fails, the publish now fails with the export's reason instead of waiting forever. Retry starts a fresh export.
- A photo that goes from single to split and back reuses its earlier record. The old picture is still recognised as the app's own and is removed from the shop.
- While a publish is waiting for the shop, a pending picture's record notes the framing it is about to upload.
- The refusal names photos the way export files are numbered ("Main 02", "Condition 01").

## Not yet
- No screen shows split photos yet. The publish window still shows one thumbnail per photo, and the framing editor comes in later phases. Until then, splits are set by hand as in step 4.
- The first publish after this update re-uploads, once, any photo whose framing was changed before the update (named `-r1`). After that, only real changes upload.
- Switching a photo between single and split in the middle of a publish makes Retry fail for that run. Starting a new publish works.
- Changing the output size in Settings between a crashed publish and its retry can leave the old-size picture on the shop. The framing is right. This is rare and written down.

## Needs you
Nothing.
