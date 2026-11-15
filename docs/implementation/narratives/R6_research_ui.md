# Phase R6_research_ui: the research agent's screens — the panel, Settings → Research, and conflicts in the publish review

## What you can do now
The screens for the research agent are built, but the agent itself is connected in the next research phase. Until then most of what is new stays quiet or refuses politely.
- **Research panel.** In a product form, the research panel lists what research did, step by step: Looked, Searched, Read (with a link to the page), Compared, Concluded. While a run is going you see "Researching…" and new steps appear by themselves. At the end it says, for example, "Finished · 3 fields filled · 2 to review", or the stage where research stopped and the error, with **Run again**. When research cannot run, it says why, for example "Day limit reached". The form header shows "2 to review" when two AI suggestions wait for you, and clicking it opens the panel.
- **Settings → Research.** A new section for the three keys (typed in, never shown again, marked "set" or "not set"), your trusted sites (your shop first and fixed, then your own list with add, remove, move up and down), model and image size per step, cost limits, prices, language, automatic research, quiet period, how long records are kept, and the fields that name a designer or maker (a warning appears under one your shop does not have).
- **Conflicts.** A *conflict* is an AI suggestion that replaced a preset default or a value from Shopify. The publish review now says "2 low-confidence AI suggestions will not be published" under the item concerned. Each conflict is listed with the old value, where it came from, the suggestion and its confidence, and **Keep previous / Use suggestion / Edit**. Publish stays available, and only that item waits. In the form, a conflict field is marked and its AI popover offers the same three actions. What you type there is saved once, when you press Enter or leave the field, never halfway through typing. A waiting session reads "Waiting for you · 2 conflicts" in the list. It switches to publishing by itself when you settle the last conflict.

## Try it yourself
1. Start the app on scratch data:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager-agentic/apps/desktop
   PSM_DATA_DIR=$HOME/psm-research-dev npm run tauri -- dev
   ```
2. Open **Settings** and scroll to **Research**. The keys read "not set". If a shop is connected, its address is first under Trusted sites, marked "always trusted", with no remove button. Saving keys, adding a site or saving the section shows a refusal for now. That is expected until research is connected.
3. Open a session's product form. Everyday editing works as before. With no AI suggestions in it, there is no "to review" label yet.
4. To see every new screen behaviour checked with made-up data, run the screen tests. Every line should pass:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager-agentic/apps/desktop
   npx vitest run src/features/form/__tests__/r6_research_panel.test.tsx src/features/form/__tests__/r6_conflicts_in_form.test.tsx src/features/settings/__tests__/r6_research_settings.test.tsx src/features/publish/__tests__/r6_research_publish.test.tsx
   ```

## Choices made for you
- "n to review" in the form header counts every unchecked AI suggestion in the form, including low-confidence ones. Say so if low-confidence ones should not count.
- A run that only confirmed values already there reads "no fields changed" instead of "0 fields filled". Say so if you prefer other wording.
- "Waiting for Capture One" appears only two minutes after the session's latest photo was taken. For photos imported from disk it can appear earlier than the agent's own wait. Say so if that matters to you.
- The form decides which fields are conflicts with its own copy of the "is this the same value" rule. In rare cases, such as the same size in different units, it could mark a field the agent does not treat as a conflict. If you then choose an action, it is refused. Say so if the app should ask the agent instead (recommended later).
- Edit in the publish review is a small text box. In the form, Edit puts you in the field itself. If you leave an Edit unchanged, nothing is saved.
- After a waiting publish has gone ahead, undoing your choice brings the conflict back in the form. The publish keeps the value you chose, and the popover says so. Say so if you want something stronger than wording.
- The confirmation you approve records the summary plus the "will not be published" and "waits until you resolve" lines. With several sessions, each line starts with "Session n:".
- The Research settings save as one block with their own Save button. Trusted sites save on each add, remove or move.

## Not yet
- Research is not connected, so the panel shows no steps and no button. Settings → Research cannot save keys, sites or settings yet. Both come with the next research phase.
- Undo or Redo that settles the last conflict does not refresh the session list at once. Only the direct actions do.
- In the publish review, Edit works well for text fields. For a list, a size or a number, settle the conflict in the form instead.
- "Accept all" in the panel shows a count that includes conflicts, which it then skips.
- Cancelling a run, researching one field again and showing the cost are later extras.

## Needs you
Nothing.
