# Phase R12_research_cost: research costs less: one look at all the photos, shorter page reading, no comparing against the wrong kind of piece

## What you can do now
Research now looks at an item's photos together in one call, not one call per photo. It sends up to 8 photos: the Main photos in their order first, then any unreviewed ones, and never the condition photos. It is told the first Main photo is the wide shot (what the piece is, and how many) and the others are the closer shots (wood, material, colour, surface). It gives one answer per field and says which photos show a label or stamp, and only those get a label read. The research panel shows one "Looked at n photos" step, and when photos were left out it says how many. When research reads a listing page, it now gets only the product part: the title, the page's description, what the page's product data says about creator, maker, material and date (never the "brand", which on these sites names the seller), and the text from the page's heading on. Menus and "more from this dealer" lists no longer reach it. It also says whether the page is about the same kind of piece. A page that clearly shows another kind (a table when your item is a chair) is not compared with your photo, and its step says so. Your detailed-condition table (None or Minor → Very Good, Moderate → Good, Major → Worn, never Excellent) now ships as the default.

## Try it yourself
1. Start the app on the scratch folder, with the test shop connected and both research keys entered, set up as in the last phase:
   ```
   cd apps/desktop
   PSM_DATA_DIR=$HOME/psm-research-dev npm run tauri -- dev
   ```
2. Shoot one piece with five or six photos, the wide one first. Sort them and press **Arranged — next**.
3. Open that item's form and its research panel. You see one "Looked at 6 photos" step (or however many you shot) instead of one step per photo.
4. If a listing page about a different kind of piece turned up, its read step says "lists another kind of piece (…): not compared".
5. Settings → Research → **Export research for today**. In the zip, that run's calls folder holds a single observe call. Its request lists the photos with "wide" and "closer". The run's cost is in `run.json`.
6. To see the condition table arrive filled in, start once on a fresh scratch folder (for example `PSM_DATA_DIR=$HOME/psm-research-dev2`) and open Settings → Research → Detailed condition. On your usual folder the table you entered by hand stays (reset keeps your own lists).

## Choices made for you
- **Reordering two of the closer photos does not start research again by itself.** A new first photo does. To change this, ask for any reorder to count as a change.
- **"N/A" in a damage field counts for nothing in the condition grade.** For example, None plus N/A gives Very Good. To change this, say what N/A should give.
- **A listing page with no readable text but a title or description is now read** on that title and description, where before it was skipped as unreadable. Ask if you want such pages skipped again.
- **The page's heading counts even when it is hidden.** If the hidden heading's text is not in the visible text, the page is read from its start. Nothing to decide unless you see a page read from the wrong place.
- **The kind of piece is only sent when the item has a type.** Without one, every page is compared as before.
- **The measurement's daily spending cap is USD 20,** the same as its ceiling (the first measurement used 25).
- **The step texts are fixed:** "Looked at n photos", "n photos were not looked at (the limit is 8)", "Could not read the photos". Ask for other wording if you like.

## Not yet
- The second 42-piece measurement has not been run. It needs you there, your keys and a USD 20 ceiling. It will say what a piece costs now and whether designer, maker and wood are right as often as before. If the photo look got less accurate, you decide whether to keep the one-call look.
- Choosing photos by a shoot template (which shot serves which field) does not exist yet. For now the first Main photo is the wide one and the rest are closer.
- If research's single photo call is refused, none of the photo-read fields (wood, colour and so on) are filled for that run. Before, only the refused photo was lost.

## Needs you
Nothing.
