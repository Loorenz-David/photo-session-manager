# Phase P14_publish_update: a publish now writes the item's details, tags, damage report and retouched photos to Shopify

## What you can do now
A publish now does the whole job on the product, not just the gallery. After the photos are in place, the app writes the item's details to the shop:
- **Details fields.** Only the fields you filled in, and only those that differ from the shop, are sent. The app then reads them back and stops with the field's name if the shop kept something else. A field you left empty is never cleared on the shop.
- **Title, SKU and barcode.** Each is written when it differs from the shop. A barcode the shop has and you don't is kept.
- **Description.** The description you typed is written as paragraphs.
- **Tags.** Tags are merged with the shop's own tags. A tag someone added in the Shopify admin stays. A tag you remove in the app, including the very last one, is removed from the shop.
- **Damage report.** The condition photos you kept ticked are uploaded as files and listed in the damage-report field, in condition order. Unticked photos are left out. If the shop has no damage-report field, the rest of the publish still finishes, and the notification says the condition photos were not published.
- **Retouches.** A photo you retouched after publishing is uploaded again as a new version. It takes the old one's place in the gallery and in the damage report.

The confirmation text is complete too. It gives the number of images, a note about unreviewed photos, how many shop photos will be removed, and whether the product will be created.

## Try it yourself
1. Run this phase's checks. They use a pretend shop, so no network or store is needed:
   ```
   export PATH="$HOME/.cargo/bin:$PATH"
   cd ~/Desktop/Developer/Photo-Session-Manager
   cargo test -p psm-testkit --test p14_publish_update
   ```
   The last line should read `test result: ok. 22 passed`.
2. See the tag behaviour: an admin tag survives a second publish, and clearing your last tag removes it from the shop:
   ```
   cargo test -p psm-testkit --test p14_publish_update c2d_republish_preserves_admin_tags_and_applies_local_removal -- --exact
   ```
3. See the damage report in condition order, and a retouched photo taking its old place in the gallery:
   ```
   cargo test -p psm-testkit --test p14_publish_update c4a_selected_condition_files_in_order -- --exact
   cargo test -p psm-testkit --test p14_publish_update c5b_retouch_main_replaces_in_same_gallery_position -- --exact
   ```

## Choices made for you
- The first time the app touches a product's tags, it treats every tag already on the shop as the shop's own and leaves it alone. Nothing to change.
- The app remembers the tags you last published. A tag you remove from that list is removed from the shop. Tags it never published are never touched. Nothing to change.
- Changed in the fix: removing the very last tag in the form used to clear the field without telling the shop, so that tag stayed on Shopify. Now the shop drops it too, and admin tags stay. Nothing to change.
- Changed in an earlier fix: the app now reads the damage-report field back from the shop in its own way. Before, every real publish with condition photos would have failed its final check. Nothing to change.
- The "condition photos not published" note only appears when you had condition photos ticked. To get it every time, say "always warn when the damage-report field is missing".
- Details go to the shop in batches of at most 25 fields per request, the limit the shop allows. Nothing to change.
- The app tracks retouches by remembering which version of each RAW it exported. Nothing to change.

## Not yet
- There is still no Publish button. The popover, progress, failure messages and batch publish come in the next phase.
- The description is sent again on every publish, even when it has not changed. The result on the shop is the same. It is on the list to tidy up.
- The app does not yet warn before a publish that the shop has no damage-report field. You learn it from the notification afterwards. A health check for this comes with the settings work in the last phase.
- AI-written descriptions come after the MVP.

## Needs you
Nothing.
