# Phase W6_restoration_choices: "Restoration done" as fixed choices the writer must state

## What you can do now
Every item type's Condition step now ends with **Restoration done**, a field where you tick what the workshop did. It has 14 choices: Cleaned, Sanded, Oiled, Waxed, Lacquered, Joints re-glued, Veneer repaired, New foam, New webbing, New springs, New papercord, New fabric, New leather, Rewired. It shows up even in item types whose Condition step you arranged earlier. When you press **Write texts**, the writer gets your ticks as facts. The check then makes sure the **condition report** names every step you ticked, in English or Swedish. If a step appears only in the description, the text is flagged and names the missing step. The field stays in Session Manager. It is saved with the item and goes into the export, but it is never sent to Shopify, and research never fills it. The free-text Restoration field is unchanged, for anything extra.

## Try it yourself
1. Start the app on scratch data with the dev store connected:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager-agentic/apps/desktop
   PSM_DATA_DIR=$HOME/psm-research-dev npm run tauri -- dev
   ```
2. Open a session and go to its Condition step. "Restoration done" is the last field. Tick Sanded and Oiled.
3. Press **Write texts**. The condition report should say the frame was sanded and oiled. If the writer leaves a step out of the condition report, AI review flags the text with a line like `the restoration step "Oiled" is not in the condition report`.
4. Publish a linked item. In Shopify's admin the product has no new field.
5. Export the session and open the item's `item.json`. It holds `"local.restoration": ["Sanded", "Oiled"]`.
6. To run the automated checks for this phase:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager-agentic
   cargo test -p psm-testkit --test suite -- research::w6_restoration::
   cargo test -p psm-research --lib -- writer::postcheck rules::write_table
   ```

## Choices made for you
- "Restoration done" lives only in Session Manager: it is saved with the item and in its export, never sent to Shopify, so the shop needs no new field. You picked this answer earlier.
- The 14 choices are fixed in this build. Say "add <name> to the restoration choices" to have more added in a later build.
- Research never fills this field, because it is the workshop's knowledge and not something the photos show.
- Only the condition report counts for the check. A step mentioned only in the description is still flagged, as you decided.
- Each step counts as stated when the condition report uses one of its listed words, for example "sanded" or "slipad" for Sanded, and "new fabric", "reupholstered", "nyklädd" or "nytt tyg" for New fabric. Say "also accept <word> for <step>" to widen a list.
- The words are found anywhere inside the text, so a report that says "soiled" also counts as saying "oiled". Say "match restoration words as whole words" if you want that tightened.
- The choices show in the usual multi-choice dropdown, with no new design.
- Item types with no saved Condition layout also show the field last, after the shop's fields.
- The photo note and the writer now also see this field, like every other Condition field.

## Not yet
- The writer's built-in instruction still asks for "the restoration in one sentence" in the condition report. It does not yet say "name each step exactly". So the first draft may miss a step and need the automatic rewrite, which costs one more call. A wording change is noted for later.
- You cannot edit or add choices in the app yet.
- No live writer call or live publish was made in this phase's checks. Your first real check is steps 2–5 above.

## Needs you
Nothing.
