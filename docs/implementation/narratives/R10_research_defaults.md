# Phase R10_research_defaults: research works out of the box with the models you picked

## What you can do now
On a fresh install, research needs nothing set up except the two keys. As shipped, the app uses the models you picked on 15 November. Photo look uses Sol (low effort), reading listings uses Luna (low), comparing uses Terra (low), labels use Sol (low), condition uses Terra (high) and the final decision uses Terra (low). It also comes with a price list for those models, the photo search and page downloads, so the cost caps work from the first run. Research no longer starts by itself: you start it with "Run research" on the item's research panel, and later with the "Arranged — next" button. No shop field counts as "designer" or "maker" until you choose one.

## Try it yourself
1. Start the app on an empty scratch folder:
   ```
   PSM_DATA_DIR=$HOME/psm-research-dev-2 npm run tauri -- dev
   ```
2. Connect the test shop. In Settings → Research, enter the model key and the visual-search key. Add your trusted sites if you like.
3. Still in Settings → Research, check that each step already has a model and an effort. The photo search step has none, because it is not a model. Automatic research is off.
4. Create a session, shoot one photo and mark it Main. Nothing starts by itself.
5. Press "Run research" on the form's research panel. Steps appear within a few seconds, and you did not have to touch any other research setting.
6. To see the safety net, remove the model key and press "Run research" again. The panel says "Add the model key in Settings → Research" and no run starts. Exporting the session still works.

## Choices made for you
- **Photo search is charged USD 0.025 per search.** That is SerpApi's cheapest paid plan, so the cost caps count searches. Your measurement counted them as free because it ran on the free plan. To change it, say "price photo search at 0" or give the plan you are on.
- **When a call's cost can't be read, it counts as USD 0.05** for a model call, USD 0.025 for a photo search and a tiny USD 0.000001 for a page download, which is otherwise free. The model figure is the worst case seen in your measurement. To change it, give a different figure per call.
- **The photo search step has no model or effort setting,** because it is a search service, not a model.
- **While automatic research is off, the panel shows no warning sentence.** "Off" is treated as a normal state, not as something missing. If you want a hint there, say so.
- **Up to 10 pages are read per run, as before,** because your measurement didn't suggest changing it.
- **Prices are the ones on the providers' pricing pages on 12 November.** If a provider changes them, ask for the defaults to be updated. Your own saved settings always win over the defaults.

## Not yet
- The picker to choose which shop fields mean "designer" and "maker" comes in the next phase. Until then, no field is treated that way.
- The "Arranged — next" button that starts research also comes in the next phase, along with the list of extra shop addresses.
- The new defaults only reach a scratch folder where research settings were never saved. A folder where you already saved Settings → Research keeps its old values: automatic research on, no prices, no models. Research there stays blocked with a "price missing" message until you fill the values in. Use a fresh folder (as in the steps above), or ask for a "reset research settings" button.

## Needs you
Nothing.
