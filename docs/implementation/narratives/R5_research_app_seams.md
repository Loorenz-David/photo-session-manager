# Phase R5_research_app_seams: what you do with an AI suggestion is remembered, and publishing waits for you only where a suggestion replaced your value

## What you can do now
Nothing new appears on screen yet, and the research agent does not write suggestions in the app yet. Underneath, the two places where the app must treat suggestions carefully are now ready.

**In the form:**
- **Looks right** now accepts a suggestion. The field stays marked as an AI value you approved; before, it became your own edit. On a suggestion you already accepted, the popover only offers Clear.
- A suggestion that replaced a preset default or a value from Shopify is a *conflict*. It gets three actions: Keep previous, Use suggestion, and Edit.
- Every choice you make on a suggestion is written down, so later research runs can learn from it. For example, research never suggests again a value you cleared or kept away.
- **Undo and Redo never erase a newer suggestion.** If research changed a field after your step, the step is skipped and you see "that field was changed by research".

**When you publish:**
- Low-confidence guesses that replaced nothing are left out of the publish.
- An item with a conflict waits ("Waiting for you") and sends nothing to Shopify. The other items you confirmed with it go ahead.
- When you resolve the last conflict, the item starts publishing with the values you chose. Cancel on a waiting item takes effect at once, and a waiting item stays waiting after a restart.
- If the suggestions change while the publish confirmation is open, Confirm refuses with "AI suggestions changed while you were confirming".

## Try it yourself
1. Check that everyday editing and publishing still work as before. Start the app on scratch data with the test shop connected:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager-agentic/apps/desktop
   PSM_DATA_DIR=$HOME/psm-research-dev npm run tauri -- dev
   ```
   Open an item's form, edit a field, then Undo and Redo. Publish an item that has no suggestions. Both should behave exactly as before.
2. See the new rules for your actions on suggestions, Undo and Redo. Every line should say `ok`:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager-agentic
   cargo test -p psm-testkit --test suite research::r5_feedback_and_undo::
   ```
3. See publishing with left-out guesses, a waiting item, release at the last resolution, cancel and restart:
   ```
   cargo test -p psm-testkit --test suite research::r5_publish_hold::
   ```

## Choices made for you
- The new undo steps are named "Accept <field>" and "Keep previous <field>". Use suggestion counts as Accept, and Edit on a conflict as "Edit <field>". Say so if you want other names.
- What you choose is recorded only for your own edits, not for values the app fills in itself. Say so if app-filled values should count too.
- An edit that is recorded as your choice, or that settles a conflict, is its own undo step. It is never merged with the typing before it. Say so if you'd rather it merged.
- "Accept all" skips a suggestion that replaced a value when the app no longer knows that field's type, and leaves it for you to decide. Say so if Accept all should take those too.
- If an item has more than one waiting publish, settling a conflict counts for the most recent one. Say so if it should count for another.
- Redo of a step that settles a conflict uses the value that step recorded. Say so if Redo should use the field's current value instead.
- Cancel on a waiting item is done in one step. If the item was released at the same moment, the normal cancel runs instead. Say so if you want the cancel refused in that case.
- An action that doesn't fit the field's state is refused, and nothing is saved. Examples: accepting something that isn't a suggestion, or Keep previous on a field without a conflict. Say so if it should quietly do nothing instead.

## Not yet
- No buttons for conflicts, no "waiting" line in the publish popup and no research panel yet. Those come in the next research phase.
- Research is not connected to the app yet, so nothing writes suggestions in normal use. That comes after the screens.
- When you settle the last conflict, the session list still shows "Waiting for you" until it next refreshes. The screens phase will make it update at once.
- After a waiting item has started publishing, undoing the choice that released it shows the conflict in the form again, but the publish keeps the value you chose. The screens phase should make this clearer.

## Needs you
Nothing.
