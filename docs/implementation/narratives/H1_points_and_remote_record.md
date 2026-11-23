# Phase H1_points_and_remote_record: Shopify's points, and remembering which pictures the shop holds

## What you can do now
Nothing new shows on screen yet. This step gives the app the knowledge the next steps will show. The app now keeps track of how many of Shopify's request points are left after every answer, also after a "slow down" answer. Its background work waits while fewer than a quarter are left, and your own presses never wait. For a product whose pictures are only in the shop, the app now remembers which pictures the shop holds, in order, with each one's file size. That covers an item the clean-up cleared after publishing, and a shop product you picked from the search that was never photographed on this Mac. The app can also download a picture at full size, and it can tell "this picture is gone from the shop" apart from "the download failed".

## Try it yourself
There is nothing to click yet. If you want to see the new behaviour checked on this Mac, run the tests for this step:

1. Open Terminal and go to the project folder.
2. Run the two test groups for this step, one after the other:

```
cd ~/Desktop/Developer/Photo-Session-Manager-agentic
cargo test -p psm-testkit --test suite h1_shopify_reads::
cargo test -p psm-testkit --test suite h1_remote_record::
```

3. Each one should end with `test result: ok` (21 and 16 tests).

Please don't install a build with this step yet. The size fields the app now asks Shopify for are taken from Shopify's documentation and haven't been tried on your real shop. A wrong field name would make every clean-up check report "Shopify could not be read". The live check after the step that adds the screen covers this.

## Choices made for you
- **Nothing to see yet.** This step only prepares what the next ones show. To change it, say "show something already in this step".
- **Background work waits for Shopify's points, and your own presses never do.** This covers the first read of a shop product's pictures and the clean-up's check before it clears an item. Say "let the clean-up ignore the points" if you'd rather it never waited.
- **A shop product's picture list is read the first time it's needed, then remembered.** A product whose pictures Shopify is still processing is asked again later. Say "re-read it every time" if you want it always fresh.
- **The clean-up now also remembers each picture's size,** so a later "Bring back" can tell you how many MB it will download.
- **The wait is written to the log once, when it starts,** not every quarter second while it lasts. Say "log every check" if you want more detail.
- **While the clean-up waits for points, the items it skipped count as already freed for the low-disk warning.** It already does this for items left for its next round. So in that short window the warning can show a little later. Say "warn on low disk even while waiting" to change it.
- **When the background step has read some products and then has to wait, it reports that it worked.** It goes straight round again and only then waits. Nothing is lost either way.
- **Refresh from Shopify, when no shop is connected, answers "this item's photos are on this Mac, not in the shop".**
- **The messages say "the shop", never "Shopify",** in the app's core. For example: "a picture of this item is still processing in the shop".

## Not yet
- No picture from the shop appears anywhere yet: not in the sessions list, not in the search. That comes next.
- "Bring back" (downloading a cleared item's pictures to this Mac again) is a later step.
- The screen for a shop-only or cleared item, its Refresh button, and switching to it are later steps.

## Needs you
Nothing.
