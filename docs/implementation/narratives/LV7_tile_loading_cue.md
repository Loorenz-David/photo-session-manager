# Phase LV7_tile_loading_cue: a quiet shimmer on a phone tile while its picture is on the way

## What you can do now
When you take a picture, its tile in the phone's shot strip no longer sits there as plain grey. A faint band of light
crosses the grey slowly, about once every two seconds, until the photo appears. The tile's number and the "captured"
check stay on top and readable. Nothing waits on the shimmer: you can take the next picture at once, and tapping a
shimmering tile still opens its preview. The full preview's "Preparing preview…" and the waiting tiles in the Move
drawer shimmer the same way. With Reduce Motion on, the phone shows plain grey as before.

## Try it yourself
1. Start Session Manager from this branch with the camera connected and the phone paired:
   ```
   cd apps/desktop
   npm run build
   npm run tauri dev
   ```
2. Open a session that has a shooting template. Take a picture from the phone. Its tile shows a faint light moving
   across the grey for a few seconds, and then the photo.
3. Take the next picture straight away. Nothing stops you while the previous tile is still shimmering.
4. Tap a shimmering tile. The preview opens with "Preparing preview…" over the same faint sweep, then shows the photo.
5. On the iPhone, turn on Settings → Accessibility → Motion → Reduce Motion. A waiting tile is plain grey, with no
   movement.

## Choices made for you
- **How strong the shimmer is.** At its brightest the band is about 5 % lighter than the grey, which is deliberately
  faint. If it is too faint or too strong on the phone, say "make the shimmer stronger" or "fainter".
- **How fast it moves.** One pass every 1.8 seconds, at an even speed, slightly slanted. To change it, say the speed
  you want.
- **Where it shows.** Every waiting picture on the phone shimmers: the shot strip, the Move drawer and the full preview
  behind its "Preparing preview…" text. To keep it on the strip only, say so.
- **A picture that never arrives.** If the phone gives up loading a picture after its retries, that tile keeps
  shimmering until the photo changes. Before, it stayed plain grey. Telling "still coming" apart from "gave up" needs
  a small extra step that was not part of this request. If you see a tile that shimmers for a long time, say so and
  that tile can go back to plain grey.

## Not yet
- Nobody has seen the shimmer on a real phone yet. Step 2 above is the first look, and that is where the strength is
  judged.
- A tile that gave up loading is not told apart from one still arriving (see above).

## Needs you
Nothing.
