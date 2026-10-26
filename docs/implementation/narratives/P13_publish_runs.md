# Phase P13_publish_runs: the app can now publish a session to your Shopify product

## What you can do now
The publishing engine is built. When you confirm a publish, the app first takes a snapshot of what you confirmed: the Main photos in their order, the condition photos, the item's valid details, and any shop photos it is about to remove.

From then on it works through that snapshot, one step at a time:
1. It finds the product by its exact SKU, or creates a draft product if there is none.
2. It checks that no RAW file changed and exports the photos.
3. It uploads each photo once.
4. It makes the product's gallery match your Main order exactly.

If the app is closed or crashes midway, it picks up where it stopped when you open it again. It never uploads a photo twice. A failed step can be retried without redoing the earlier steps.

Publishing runs in its own lane, so new shots and previews keep arriving while an upload is under way.

You can already see one part on screen. With the shop connected, the New session search also lists products that exist only in the shop, marked "in the shop". Picking one links it to a new local item. A typed SKU that the shop already uses is flagged, and the suggested next SKU skips numbers the shop already holds.

## Try it yourself
1. Run this phase's checks. They use a pretend shop, so no network or store is needed:
   ```
   export PATH="$HOME/.cargo/bin:$PATH"
   cd ~/Desktop/Developer/Photo-Session-Manager
   cargo test -p psm-testkit --test p13_publish_runs
   ```
   The last line should read `test result: ok. 36 passed`.
2. See a whole publish run, a restart in the middle of an upload, and a capture arriving during an upload:
   ```
   cargo test -p psm-testkit --test p13_publish_runs c7a_done_records_publish_and_notification -- --exact
   cargo test -p psm-testkit --test p13_publish_runs c8b_restart_requeues_running_upload_job_and_completes -- --exact
   cargo test -p psm-testkit --test p13_publish_runs capture_arrival_is_recorded_while_publish_media_poll_is_held -- --exact
   ```
3. Optional, with your test store. Put the store's `shop_domain`, `client_id` and `client_secret` into the `integration` block of `~/Library/Application Support/com.loorenz.sessionmanager/settings.json`. Then start the app, open New session and type the title of a product that exists only in the shop. It appears marked "in the shop", and choosing it opens a session for that product.

## Choices made for you
- Each publish step is its own small job, so a crash loses at most the step in progress. Nothing to change.
- The export made for a publish is kept aside and only replaces your local item folder once the publish has finished. To change this, say "replace the local export as soon as it is rendered".
- The "Published" note records the time as a raw number for now. The publish screen in a later phase will show it as a normal date and time. Nothing to change.
- Changed in the fix: a product that already holds a finished video or 3D model no longer makes the upload wait forever. If you confirmed removing it, it is removed. If you did not, the publish stops with a message that the product gained media. Nothing to change.
- Changed in the fix: uploads used to run in the same lane as new captures and previews, so shooting the next item while one was publishing would have shown nothing until the upload ended. They now run separately. Nothing to change.

## Not yet
- There is no Publish button yet. The popover, the progress on each row, failure messages and batch publish come in phase P15.
- Item details, title, tags, description and the damage report are not written to the shop yet. That comes in the next phase.
- If the Shopify part of the settings file is incomplete, for example a missing secret, the app will not start. This gets fixed with the settings work in the last phase. Until then, fill in all three fields or set `integration` back to `null`.

## Needs you
Nothing.
