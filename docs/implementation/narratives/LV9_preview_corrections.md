# Phase LV9_preview_corrections: the four corrections you asked for after sorting on the phone

## What you can do now
When you press the shutter, the shot you took shows the faint light moving over grey right away. It shows in the live screen's strip and in the full-screen view's strip, and it stays until the picture is there. Pictures that will become Condition photos (or extra Main photos in a session without a template) show a waiting tile at the end of their group.

Moving photos no longer blinks. A drag, a swipe or a change on the Mac shows in one step. A photo keeps its picture when it moves, and an old picture or an old order never flashes back. You can swipe several photos quickly one after another, and the strips show them all at once while the Mac catches up.

When you lift a template shot's photo, its place shows that shot's grey template picture with its number, not a gap.

The full-screen view is now just the photo, its strip, a ⋯ button at the bottom left and a × at the bottom right. ⋯ opens a drawer with what the header used to show and, for a template shot's photo, Retake. × goes back to live.

## Try it yourself
1. Start Session Manager from this branch with the camera connected (Wi-Fi shows the waiting best, because files take a few seconds) and the phone paired:
   ```
   cd apps/desktop
   npm run build
   npm run tauri dev
   ```
2. Open a session with a shooting template. Take a picture from the phone. The shot you took shows the light moving over grey at once. Open any photo full screen: its strip shows the same. A few seconds later the picture appears.
3. Take two pictures after the last template shot. Two waiting tiles appear after the Condition photos, then turn into the photos.
4. Press and hold a template shot's photo until it lifts. Its place shows the shot's grey template picture with its number. Drag it onto another shot and let go. The two swap in one step, with no blink and no old picture.
5. Open a Condition photo full screen and swipe left three times quickly. Each photo leaves the strip at once and nothing comes back.
6. Look at the full-screen view: only the photo, the strip, ⋯ and ×. Tap ⋯ on a template shot's photo to see its details and Retake. Tap × to go back to live.
7. On the Mac, check each change in the session view. ⌘Z brings back a denied photo.

Things only your eyes and hands can judge (nothing here was seen on a real phone or in a browser yet):
- whether the light over grey is strong enough to notice, and still quiet
- whether the grey template picture sits exactly where the lifted photo was
- whether the ⋯ and × buttons and the drawer fit well on your phone's screen
- whether a moved photo really shows no flash on the phone's browser

## Choices made for you
- **"On its way" means what the Mac already uses:** from the shutter until the photo exists on the Mac, for at most five minutes. A shutter taken for a shot that already has a photo shows no waiting tile. A picture taken through Capture One's own capture button only shows the light once its file has arrived. Say "show a waiting tile for those too" if you want more.
- **All waiting pictures are one step darker grey.** On the old, very light grey the moving light could hardly be seen, so the grey is now slightly darker and the light about 10 % brighter than it. This also applies to "Preparing preview…" in the full-screen view. Say "lighter grey" or "stronger light" to change it.
- **A picture that failed to load three times is plain grey** (no moving light), because it is not on its way any more.
- **You can't drop a photo onto a waiting shot.** It also takes no tap and can't be lifted.
- **The template picture under a lifted photo goes the moment you let go.** During a cancelled drag's short spring back (under 0.2 seconds) the place is briefly empty.
- **A swipe still asks the Mac based on where the Mac says the photo is.** If you swipe a photo that an earlier, unanswered move already moved, the Mac may refuse. The strips then go back and the phone tells you why.
- **Retake from the drawer waits for the Mac:** the drawer and the full-screen view close once the Mac has reopened the shot.
- **The drawer shows the ACTIONS section only when there is one** (a template shot's photo). The two buttons sit under the strip as round 52-pixel buttons. ⋯ gives a short buzz. The swipe labels ("DENY", "MAIN"…) and the "TEMPLATE" pill still appear while your finger is moving.
- **Move image is gone.** Dragging in the strips and swiping reach every place the Move drawer offered. Say "put Move image in the ⋯ drawer" to bring it back.
- **While the "Move this photo to Main…?" question is open, the full-screen photo ignores touches,** as its strip already did.

## Not yet
- Pinch to zoom, reframing on the phone and the composition guides come in the next phase.
- While a move or swipe is still waiting for the Mac, a photo that starts arriving shows its waiting tile only once the Mac has answered (a few seconds at most).
- The Next button can still pick a shot whose photo is already on its way. The Mac then refuses it, as before.

## Needs you
Nothing.
