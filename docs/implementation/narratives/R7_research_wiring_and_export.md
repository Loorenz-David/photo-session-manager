# Phase R7_research_wiring_and_export: research switched on in the app, its records kept for a set time and exportable as one zip

## What you can do now
The research agent is now connected to the app. When a session gets its first Main photo, a research run is queued. It runs on its own background worker, so a slow research run never holds up previews, renders or publishing. A run that was going when you quit continues at the next launch. Without a shop or keys, shooting, reviewing and exporting work exactly as before.

Settings → Research now saves. A wrong value, such as `0,5` as a cost limit, is refused with a sentence naming the field. Trusted sites keep their order. The detailed record of each research call is deleted after "Keep research records (days)", together with images no remaining record uses. Runs, steps and your accept/edit choices are kept. **Export research** in Settings writes the chosen runs to a folder you pick, as a folder plus a zip. It never includes a key, a RAW or a photo you deleted. If a key turns up in any file, nothing is written and the file is named.

Several smaller fixes are in too. Undo or redo that settles the last conflict now lets the waiting publish go ahead at once. "Accept all (n)" counts only what it will accept. The form now asks the backend which fields are conflicts. "Waiting for Capture One" now uses the agent's own wait time.

## Try it yourself
1. Start the app on scratch data:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager-agentic/apps/desktop
   PSM_DATA_DIR=$HOME/psm-research-dev npm run tauri -- dev
   ```
2. Open a product form and its research panel. It says why research is not running, for example "Add the model key in Settings → Research". Real research providers come in the next research phase.
3. Open **Settings → Research**. Add a trusted site, move it up, close and reopen Settings: the order is kept. Type `0,5` as the per-run limit and save: the refusal names the field.
4. In the same section, under **Export research**, keep today's dates, click **Export research** and pick the Desktop. The section shows "Exported to …research-export-….zip". It holds no runs until real research has run.
5. Shoot, review and export a session as usual. Nothing behaves differently.
6. To see the new behaviour checked with made-up data:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager-agentic
   cargo test -p psm-testkit --test suite research::r7_
   cargo test -p psm-research --test engine export::
   ```

## Choices made for you
- The export date range uses your Mac's local dates, both days included. Say so if you want a different range.
- Export leaves both the folder and the zip at the destination. Say so if you want only the zip.
- Records are cleaned when the app starts (also after a settings save that restarts the app's background work) and then every hour.
- A run you export by item includes runs that were queued but never started. Their file shows only the queued run.
- The export check looks for the three research keys, the camera's three passwords and the shop's client secret. Say so if another secret should be on that list.
- Research notices photo changes by listening to the app's own change signals. A form edit does not count as a photo change, so it does not restart the wait before research starts.
- Research settings take effect without restarting the app's background work. The one exception is a change of provider.

## Not yet
- Real research: the model, search and page-reader connections, and the keys, arrive in the next research phase. Until then no real run happens.
- The report and replay tool for exported research comes later.
- No automatic export folder.

## Needs you
Nothing.
