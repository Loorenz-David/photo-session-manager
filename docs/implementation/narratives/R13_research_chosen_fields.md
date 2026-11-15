# Phase R13_research_chosen_fields: research fills only the fields you tick, and the detailed condition no longer waits for every damage level

## What you can do now
Settings → Research has one list, **Fields research fills**, in place of "Fields research leaves alone". Research only asks about, suggests and spends money on the fields you tick there. A field you did not tick, including a new field added to your shop later, stays empty until you tick it. Files, reference, rich-text, price and rating fields are not offered at all, because research can never fill them. When nothing is ticked, the item's research panel says "Choose the fields research fills in Settings → Research" and no research starts. The screen warns you when your designer or maker field, or your detailed-condition field, is not ticked. Those settings only control *how* a ticked field is filled, never *whether*. The detailed condition is now worked out from the damage levels that have a value. Function, Odor and Stability, which photos can never answer, no longer stop it, and its reason says how many levels were empty.

## Try it yourself
1. Start the app on the scratch folder, with the test shop connected and both research keys entered:
   ```
   cd apps/desktop
   PSM_DATA_DIR=$HOME/psm-research-dev npm run tauri -- dev
   ```
2. Before changing any setting, open an item's form and its research panel. It says "Choose the fields research fills in Settings → Research", and pressing **Arranged — next** starts no research.
3. Open Settings → Research → **Fields research fills**. Leave the dimensions, weight and damage details unticked. Also leave the review, price and location fields unticked. The files and reference fields are not offered. Tick: Designer, Manufacturer, Wood type, Country, Years, Color, Quantity, the twelve condition levels, Condition report, Condition Report Description and Detailed condition. Save.
4. Untick Designer. A line under **Designer and maker** says research will not suggest it. Tick it again.
5. Shoot a piece with a few main photos and two condition photos, sort them, and press **Arranged — next**. In the form, only ticked fields get suggestions. Price, link, location and the Etsy description are untouched.
6. Detailed condition has a grade even though Function, Odor and Stability are empty. Its reason names the worst level and ends with a count such as "3 of 12 condition levels have no value."
7. Tick one more field afterwards. Nothing starts by itself. Press **Run research** on the item to fill it.

## Choices made for you
- **On a fresh install the list is empty, so research does nothing until you tick fields.** That is the point of "a new field stays empty until you add it". Say so if you want a starting selection instead.
- **Your old "leaves alone" list was not converted.** It is ignored and removed the next time you save the research settings, so you tick your fields once. Only scratch folders ever had that list.
- **A research run already going keeps the list it started with.** Changing the list affects the next run.
- **Changing the list does not start research by itself.** Use **Run research** on an item to fill a newly ticked field. Ask if you want a change to the list to re-run research.
- **When no condition field is ticked, the condition photos are not sent to the model at all.** That is the most expensive single call, so you save money.
- **An empty damage level does not lower the grade or its confidence.** The reason only says how many levels were empty. Say so if you want fewer answered levels to mean lower confidence.
- **A field you ticked earlier that research can never fill is still shown, marked "Research never fills this kind of field",** so you can untick it.
- **The measurement tool's settings for your shop now carry your 22 fields** (the list in step 3). Without that file the tool still measures every field, as before.

## Not yet
- The short check on four real pieces (at most USD 2) has not been run yet. It will show that nothing outside your 22 fields gets a value, and that the detailed condition matches your table for the worst level found. The orchestrator runs it next.
- Research still runs its search, page-reading and comparing steps even when none of your ticked fields needs them. Your list has fields of every kind, so for you those steps are always needed.

## Needs you
Nothing.
