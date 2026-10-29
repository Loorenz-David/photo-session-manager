# Phase P23_shape_on_arrival: every new photo gets its shape (Square by default) once, and tiles show the whole photo

## What you can do now
Every photo that reaches a session for the first time, whether you shot it or imported it, gets its crop shape set in Capture One once. The default shape is Square. The app does this only while the photo still has its full uncropped frame. If you change the crop yourself in Capture One, the app leaves it alone from then on, through restarts and re-imports too. In Settings, "Shape on arrival" lists Capture One's own shape names, plus "Leave as shot" to turn it off. Tiles and the large stage now show the whole photo in its real proportions instead of trimming the edges. If Capture One is closed or the chosen name doesn't exist, Settings → Health says so in a red row, and the waiting photos get their shape once the problem is fixed.

## Try it yourself
1. Run this phase's checks (no camera, Capture One or shop needed):
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager
   cargo test -p psm-testkit --test p23_shape_on_arrival
   cd apps/desktop && npx vitest run src/features/settings/__tests__/p23_shape_on_arrival.test.tsx
   ```
   The first should read `11 passed`, the second all passed.
2. Start Capture One with your session open, then start the app:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager/apps/desktop
   npm run tauri -- dev
   ```
3. Open a session and shoot one frame. Within a few seconds the tile shows the whole photo with no trimmed edges. After the preview delay, the tile and the stage show a square photo, and in Capture One the crop tool on that photo is set to Square.
4. In Capture One, change that photo's crop by hand (any shape, any size). Relaunch the app: your crop is still exactly as you set it.
5. Drop a photo file from the Finder into the session: it becomes square too. Drop the same file again: nothing changes in Capture One.
6. Settings → "Shape on arrival": choose "Leave as shot", Save, shoot a frame: it stays as shot. Choose "Square" again.
7. Quit Capture One and import a file into the session. Wait about three minutes: Settings → Health shows "Shape on arrival — Shape not applied to 1 photo". Start Capture One and press "Retry checks": the row disappears and the photo becomes square.
8. Publish a session with square photos: the images in the shop are square.

## Choices made for you
- On the first try, the app read every new photo as already cropped by hand and skipped it. This happened because Capture One reports "no shape set" differently from what our notes said, and it also reports the full-frame size in an unexpected form. The fix reads both the way Capture One actually reports them. I confirmed on your Mac that a test photo now turns square, that a second attempt leaves it alone, and that it can be put back to full frame. Nothing to change.
- The Settings list of shapes comes straight from Capture One. If the list can't be read, the box is left empty and no error pops up. Say "show a message when the list can't be read" to change it.
- The shape picker is a plain dropdown, with "Leave as shot" first. Say so if you want it styled like the other controls.
- The app counts a photo as untouched when its crop is still the full frame with no shape set, or with the free shape ("Unconstrained") at full size. Say "only treat photos with no shape at all as untouched" to change it.
- Opening the Health page also re-tries photos that didn't get their shape, not only the "Retry checks" button. Say "only retry on the button" to change it.

## Not yet
- The whole flow (shoot, arrival, square tile, square shop image) has not been watched end to end. Only the shaping step itself was run against Capture One. Steps 3–8 are the proof.
- The Health message for an unknown shape name mentions Capture One by name even though it is built in the app's editor-neutral core. That only matters if a second editing app is ever added.
- Our reference notes on Capture One still describe the "no shape set" value wrongly. They are queued for correction.

## Needs you
Nothing.
