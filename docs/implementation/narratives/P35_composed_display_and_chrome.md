# Phase P35_composed_display_and_chrome: the stage and the tiles show each photo as composed, a Compose button that explains itself, Split for any photo, a working Details button, and Sessions moved into the photo panel

## What you can do now
Outside the editor, the stage now shows each photo the way it will be exported. A square crop shows as a square. A turned,
flipped or straightened photo shows turned, flipped or straight. A split photo shows its two halves side by side with a thin
line between them. Press Space and the camera original appears with the same framing, so you compare like for like.

The tiles show the same composed pictures. A small crop mark sits on every grid tile whose framing leaves part of the photo out.

A Compose button now sits on the stage bar. When a photo cannot be composed yet, because Capture One is still making its
full-frame preview, the button, the menu, R and a double-click all say "Waiting for Capture One's full-frame preview" instead
of doing nothing.

You can split any photo, including one still to review. Its number stays one place until you approve it into Main or
Condition.

The Details button in the header opens the product details form, and so do D and the item card's "Edit details →". Sessions
is now an icon in the photo panel's header, and in the folded strip. After "Apply composition to selected", a "Re-frame"
badge clears at once.

## Try it yourself
1. Run this phase's checks (no camera or Capture One needed):
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager/apps/desktop
   npx vitest run p35 composed.test
   cd ~/Desktop/Developer/Photo-Session-Manager
   cargo test -p psm-testkit --test p35_split_any_photo
   ```
   19 and 1 should pass.
2. Start the app and open a session with framed photos:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager/apps/desktop
   npm run tauri -- dev
   ```
3. **Stage.** Look at the stage. A square crop should be a square, centred, with nothing cut off at the top and no empty band
   at the bottom. A split photo should show two halves with a thin line in the middle. Press Space to see the camera original
   with the same framing, and Space again to go back.
4. **Tiles.** Look at the tiles: most carry a small crop mark (with the default Square framing, most photos leave something out).
   A photo framed as Original has none. Press F: the strip shows the composed pictures too.
5. **Compose button.** Hover Compose on the stage bar: it reads "Compose (R)". On a photo whose full-frame preview is still
   coming (right after a new capture), the button is greyed, and R, a double-click or a click shows the waiting message.
6. **Split.** Right-click a photo still to review and choose Split. It splits.
7. **Details and Sessions.** Click Details in the header: the form opens. Hover the Sessions icon beside Import and Fold, then
   click it: the sessions panel opens and closes.
8. **Re-frame badge.** Copy a framing onto a photo showing "Re-frame" with "Apply composition to selected": the badge goes away
   at once.

## Choices made for you
- The crop mark shows whenever the framing leaves anything out or is straightened, so with the default Square nearly every tile
  has one. Say "mark only photos I re-framed" to change this.
- The crop mark is on grid tiles only, not on the small strip tiles. Say "crop marks in the strip too" to change this.
- The crop mark is a small crop icon in the tile's upper right corner, just below the "to review" dot.
- Space shows the camera original with the same framing as the adjusted photo. Say "show the whole original on Space" to change this.
- A photo that needs re-framing is still shown with its stored framing, and Compose stays available for it.
- The waiting message appears as one ordinary notification at a time, alongside the button's hover hint.
- The Sessions icon sits beside Import and Fold, because Export lives in the app header. It is in the folded strip too, so you
  can reach the sessions panel while the panel is folded.
- The header Details button also works while you are composing: the form opens over the editor.
- If you switch sessions while the Details form is open, it closes.
- The small output previews in the editor's tools keep their own drawing for now. Only the stage and the tiles use the new composed view.
- After review, one thing was corrected. At first the stage drew the composed photo a quarter of its height too high, cutting
  off the top and leaving an empty band below. An old layout rule for the stage picture was still in effect inside the new
  composed view. The fix switches that rule off there. I measured the result in the same browser engine the app uses, and the
  photo now sits exactly in its frame.

## Not yet
- The editor's right-hand browser, overlays, straightening grid, typed straighten and the editor's hints come in the next phase.
- Everything on screen was checked by tests and by a measurement in the app's browser engine, not by looking at the running app.
  Steps 3–8 are the first real look.
- One older automatic check (about the stage bar's place count, from the split phase) sometimes fails when all checks run
  together, and passes on its own. This was already the case before this phase. Nothing in the app is known to be wrong behind
  it. It is listed to be fixed so the checks are reliable.
- If you click Details in the first moment after opening a session, before it has loaded, the form opens only once the session arrives.

## Needs you
Nothing.
