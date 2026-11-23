# Phase L2_labels_box: a Labels box that lists the price labels still to print

## What you can do now
The Sessions page has a fourth box, **Labels**, after Open / Published / All. Its number is how many priced products still have no printed price label in Pricing. Click it and you see Pricing's own list: missing price tags first, with **Not printed**, **Printed** and **All** beside it, each with its count. The search box searches Pricing by SKU, title or product type. Products you photographed on this Mac look and open like their session. Any other product shows Pricing's title and its small shop picture, and clicking it opens it as a shop-only product (one read from Shopify, nothing written). You can tick rows, tick a whole page, or select every matching product at once. Generating the labels comes in the next labels phase.

## Try it yourself
You need a Pricing key that may read labels (Settings → Pricing).
1. Start the app:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager-agentic/apps/desktop
   npm run tauri -- dev
   ```
2. Open **Sessions**. After Open / Published / All there is a **Labels** box with a number: the labels still to print.
3. Click **Labels**. You should see priced products with no printed label, newest first. Each row has a picture, SKU, price (for example "8 900 kr") and "Missing price tag". Under the toolbar you should see "Products created since <date>". The time filter, "To review" and the unassigned-photos card are gone while you are in the box.
4. Click **Printed**. The rows should read "Label printed · <date>". Type a SKU in the search box. The list and the counts should follow.
5. With more than 25 products, press **Next** and then **Previous**. The line between them should read "Page 2 of N", then "Page 1 of N".
6. Click a product you photographed on this Mac. Its session should open. Go back and click a product from the other Mac. It should open as a product whose photos are on Shopify.
7. Tick two rows. The bar should read "2 selected", with "Generate labels for 2" greyed out. Tick the header box, then press **Select all N**. The bar should read "All N selected". Press Escape to clear.
8. Print a label in Pricing for a product shown as "Missing price tag". Within about ten seconds the product should leave that list and the number should drop, with no sound.
9. To run the automated checks for this phase:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager-agentic
   cargo test -p psm-testkit --test suite l2_labels_list::
   cargo test -p psm-desktop
   cd apps/desktop && npx vitest run src/features/labels src/features/sessions
   ```

## Choices made for you
- The pager has only Previous and Next. "Page 2 of 5" is worked out from the filter's count. Say "add page numbers" if you want to jump.
- A Labels row whose product has the same SKU as a session on this Mac opens that session, even when that session is linked to another shop product. Say "match by shop link only" to change it.
- Opening a row that has no session here creates the session straight away (one read from Shopify, nothing written) and opens it as a shop-only product. Say "ask before creating" if you want a confirmation first.
- **Select all N** selects only products that have a SKU, so the bar may read "All 58 selected" under "Select all 60". Say "include products without SKU" to change it.
- The search box in the Labels box searches Pricing by SKU, title and product type. It keeps its own text, separate from the sessions search, and still holds it when you come back to the box.
- Each time you enter the box it starts on **Missing price tag**, on page 1. Say "remember my last filter" to change it.
- The list refreshes at most once every 2 seconds when labels change, and keeps your ticks and your page. Entering the box always loads straight away, even right after a refresh.
- **Try again** after an error reloads the first page and keeps your ticks.
- Leaving the box sets its number back to the count without your search.
- The label state shows twice on each row: as the small pill and in the Label column.
- A matched row's ⋯ menu treats label access as allowed, because the list could only load with it. An empty page reads "No products here".
- When a product is opened, the app uses the SKU from the list on screen to find a matching item. If you press Test connection while the box is open, a product whose only match is by SKU could show a "SKU already in use" message instead of opening. Reload the box and it works.
- If Pricing fails while the box's number is loading, the number you already had stays.
- "Generate labels for N" is on the bar but greyed out until label generation arrives.

## Not yet
- Generating, checking and downloading labels, and the working "Generate labels for N" button (next labels phase).
- Making labels in bulk from Open or Published.
- If you remove or replace your Pricing key while the Labels box is open, its rows stay on screen until you leave and re-enter the box. This is noted for the next labels phase.
- No live Pricing or Shopify call was made in this phase's checks. Steps 2 to 8 above are your first real test.

## Needs you
Nothing.
