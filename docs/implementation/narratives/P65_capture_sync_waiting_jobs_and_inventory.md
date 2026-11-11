# Phase P65_capture_sync_waiting_jobs_and_inventory: photos reach Capture One in seconds, publishes wait for Capture One instead of failing, and new products get stock at a location you choose

## What you can do now
When a photo lands in an item's folder, the app now asks Capture One to look at that folder right away. Before, Capture One took 17–80 seconds to notice a photo on its own. The style and preview should follow within a few seconds. If you press Publish before Capture One has loaded the photos, the publish no longer fails. The row reads **Waiting for Capture One…**, the publish retries on its own, and it continues as soon as the photos are there. It gives up only after 10 minutes, with a plain sentence such as "Capture One has not loaded DSC05162.ARW after 10 minutes". Cancel publish works while it waits. A product the app creates in Shopify now has stock tracking on, with the quantity you chose at the location you chose. The form remembers your last choice, and you can change it in Settings.

## Try it yourself
1. Run this phase's checks. They use a stand-in Capture One and a stand-in Shopify, so nothing real is touched:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager
   cargo test -p psm-testkit --test p65_capture_sync_waiting_jobs_and_inventory
   cd apps/desktop && npx vitest run src/features/form/__tests__/p65_stock_and_waiting.test.tsx
   ```
   You should see `8 passed; 0 failed`, then 5 passing form and row checks.
2. In your Shopify custom app (Shopify admin → Settings → Apps → Develop apps → your app → Configuration), add the **read_locations** and **write_inventory** permissions, then install the update. Without them, products are still created, just without stock tracking.
3. Start the app with Capture One open on your session:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager/apps/desktop && npm run tauri -- dev
   ```
4. Activate an item and take a photo over Wi-Fi. The style and preview should appear within a few seconds. In the day's log, look for `capture_one.variant_seen … after_ms=`. That number is how long Capture One took to see the photo:
   ```
   grep -E "capture_one.sync|capture_one.variant_seen" ~/Library/Application\ Support/com.loorenz.sessionmanager/logs/psm.$(date +%F).log
   ```
5. Take two or three more photos and press Publish immediately. The row reads **Waiting for Capture One…** (hover to see the same in the popover), then the publish continues and completes with every photo once. Try **Cancel publish** during the wait on another session. It ends as **Publish cancelled**.
6. Open the form of a new item and go to the Product step. Under **Stock**, pick a **Location** and a **Quantity**, then publish. In Shopify, the product shows inventory tracked with that quantity at that location. Open the form of the next new item: it starts with the same choice. **Settings → Shopify → Stock for new products** shows it and lets you change it.

## Choices made for you
- "Waiting" is a label on a publish that is still running, not a separate status. In filters, the session still counts as publishing. Say "give waiting publishes their own filter" to change that.
- Retries happen after 1, 2, 4 and 8 seconds, then every 15 seconds. The 10 minutes count from the first time a photo had to wait. Say "wait longer" or "retry faster" to change that.
- Only Capture One not being ready makes a publish wait: a photo not loaded yet, Capture One busy or timing out, or Capture One closed. Any other error fails at once, as before.
- Location and Quantity sit at the end of the form's Product step and are shown for items that are not linked to a Shopify product yet. If the publish then finds a product with the same SKU in your shop, the stock you picked is not sent.
- Your stock choice is locked in when you confirm the publish. Changing the form afterwards does not affect a publish already under way.
- Your choice becomes the new default only after a product was actually created with stock tracking. A product created without tracking leaves the default unchanged.
- Choosing "No location" in the form means that item is created without tracking, even when a default exists.
- After creating a product, the app checks Shopify that the stock was set. A mismatch is written to the log and does not fail the publish.
- The app reads your shop's locations once per start, and again after you save settings.
- If Shopify refuses the stock fields, the product is created again without them rather than failing the publish.

## Not yet
- Nothing here has run against real Capture One or your real shop. The Capture One folder sync has not been tried live. The stock fields follow Shopify's API as known but could not be checked against Shopify's documentation in these sessions. Your first new product is their first real test. If it comes out without tracking, the log says why ("item created without stock tracking").
- If Shopify returns some other error on that first create attempt (not a missing permission), the product is still created without tracking. Only the log mentions it.
- If the shop's locations cannot be read (offline, Shopify error), the Stock section is hidden in the form, but the remembered default is still used for the new product.
- The full test suite ran once on a busy Mac with one timing-only failure that passed on its own afterwards. One clean full run is still owed.

## Needs you
Nothing.
