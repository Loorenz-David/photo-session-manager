# Phase R2_research_foundation: the groundwork for the research agent is in place, and the app works as before

## What you can do now
Nothing new is visible yet, on purpose. This milestone lays down the shared pieces that the research agent and the app-side work will both build on, so the two can now go ahead side by side. The app now has a fixed rule for when two values count as "the same". For example, "Hans J. Wegner" and "hans j wegner" match, 25.4 mm and 2.54 cm match, but "Børge" and "Borge" don't. It also has a place to keep the value research replaced, a "waiting for you" step for a publish (nothing uses it yet), the research buttons' connection points (for now they answer "Research is not available"), and the small, private copy of a photo that research will send out. That copy is framed like your final photo, is at most 1024 or 2048 pixels on its long side, and carries no camera model, serial number or GPS location. Your existing data gets the new empty fields the first time the app opens, and nothing else in it changes.

## Try it yourself
1. Open the app on scratch data, so your real sessions are not touched:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager-agentic/apps/desktop
   PSM_DATA_DIR=$HOME/psm-research-dev npm run tauri -- dev
   ```
2. Open an item's form. Values you typed, preset defaults and values marked "from Shopify" should look exactly as before.
3. Open a publish review and publish an ordinary item (on your development shop). It should go through as before.
4. Optional, run the new checks yourself:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager-agentic
   cargo test -p psm-testkit --test suite research::r2_foundation::
   ```
   You should see `10 passed; 0 failed`.

## Choices made for you
- Two standard Unicode libraries handle letter case and punctuation when names are compared, instead of hand-written tables. Say "use different libraries" to change that.
- Measurements are compared with exact whole-number arithmetic. A number too large to compare counts as "not the same". Say "treat huge numbers differently" if you want that changed.
- The new publish fields for conflicts start empty, and the research settings entry is left out of your settings file until research has something to store there. Say "always write the research settings entry" to change that.
- The research status shows its stage, state and trigger as plain text for now. The engine milestone will turn them into fixed lists. Say "fix the lists now" if you want them settled earlier.
- The research copy is made by the same framing code as your final photos. All embedded information is then removed without compressing the image a second time, and that includes the colour-profile tag, since the colours are standard sRGB anyway. Say "keep the colour profile" if you want it kept.
- When a photo is turned a quarter and cut in half, the cut is measured on the turned photo, the same way the framing tools already work. Say "measure cuts on the unturned photo" to change that.

## Not yet
- No research runs, writes values, or holds a publish. Those arrive in later milestones.
- The session list has no "Waiting for you" pill yet. It comes with the research screens, before anything can actually wait.
- The research buttons and settings aren't on screen yet.

## Needs you
Nothing.
