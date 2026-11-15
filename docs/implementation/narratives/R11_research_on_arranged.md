# Phase R11_research_on_arranged: research starts when you say a session is arranged

## What you can do now
Research no longer starts while photos are still arriving. When you have finished sorting a session, press **Arranged — next** in the header. The session is marked "Arranged" in the list, its research starts, and the next session you still have to arrange opens, so new captures go there. If you change a photo afterwards, nothing starts by itself. Press the button again and research looks again, but only at what changed. When you publish a session you never arranged, publishing marks it first and starts its research. The publish window tells you research is still running, and you choose whether to wait or publish straight away. In Settings → Research you can now pick which shop fields mean designer and maker, which fields research must never touch, and which other web addresses belong to your shop. You can also fill in a table that turns the worst damage level into the Detailed condition grade, and reset the technical settings to their defaults while keeping your own lists. A similar piece in your own shop is no longer used as evidence for who designed or made an item.

## Try it yourself
1. Start the app on a scratch folder, with the test shop connected and both research keys entered:
   ```
   PSM_DATA_DIR=$HOME/psm-research-dev npm run tauri -- dev
   ```
2. Open Settings → Research. If you used this folder before the previous phase, press **Reset research settings to defaults** and confirm. Models and prices appear, and your trusted sites are still there.
3. Under **Designer and maker**, Designer and Manufacturer are ticked as a suggestion. Press Save. Under **Fields research leaves alone**, tick the dimensions, the weight and the damage details. Under **Extra shop addresses**, add your other shop addresses. Under **Detailed condition**, choose the field and fill in the table: None → Very Good, Minor → Very Good, Moderate → Good, Major → Worn.
4. Create two sessions and shoot a few photos in each. No research starts while the photos arrive. The form's research panel says "Research starts when you press Arranged — next."
5. In the first session, sort the photos, mark the condition photos and press **Arranged — next**. The second session opens. Open the first session's form: research steps appear, no width, weight or damage details are suggested, and Detailed condition follows the damage levels.
6. Change a photo in the first session. Nothing starts. Press **Arranged — next** there again. A new run starts and looks only at what changed.
7. In the second session, press **Publish** without arranging it. The window says research is still running for this item. Wait until the line goes away, or confirm straight away to publish without that research.

## Choices made for you
- **The "arranged" mark is kept on the session and never removed.** It is not something Undo takes back. To change this, ask for an "unmark" action.
- **"Next" means the nearest session created after the one you are in.** It skips sessions that are already arranged, published, archived or have no photos. When none is left after yours, it goes back to the oldest. To change this, name the order you want.
- **Publishing never waits for research.** Confirm stays enabled. If you confirm while research is running, that run's suggestions are not in the publish. To change this, ask for Confirm to wait.
- **The old "automatic research" switch and quiet period are gone,** not just hidden. Old saved values are ignored.
- **Reset keeps your own lists:** trusted sites, designer and maker fields, fields left alone, extra addresses, the condition table and the language. Everything else goes back to the defaults. To change this, say which ones should also reset.
- **The field pickers list only your shop's own fields,** not the app's built-in product fields. They have a search box. Without a shop they say "Connect a shop to choose fields".
- **The damage levels in the condition table come from your shop's condition fields,** in your shop's order. Choosing a different Detailed condition field clears the table.
- **A run that reaches its spending limit still fills in Detailed condition,** because working out that grade from the damage levels costs nothing. That run then shows as finished, not as stopped at the limit. Ask if you want it to stop instead.
- **What the review fix changed:** nothing you can see. It strengthened one automatic check, so the check now proves that pressing **Arranged — next** really starts research, not only that it marks the session.

## Not yet
- The condition table you chose (None → Very Good, Minor → Very Good, Moderate → Good, Major → Worn) arrives empty here. You enter it once in Settings, as in step 3. The next phase makes it the default.
- Picking photos by shoot template, reading pages locally, dropping non-matching pieces before comparing, and the second accuracy measurement all come in the next phase.

## Needs you
Nothing.
