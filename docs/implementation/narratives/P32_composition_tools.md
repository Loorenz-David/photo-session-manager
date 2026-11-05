# Phase P32_composition_tools: the editor gets its Tools panel, shapes, turning, straightening, trim handles and keys, and asks you to confirm photos Capture One changed

## What you can do now
When you open the editor, a Tools panel sits on the left with a "Composition" section. In it you can pick a shape (Original, Square, 4:5, 3:2, 2:3, 16:9 or Free), turn the frame between portrait and landscape, rotate the photo 90° either way, flip it, and straighten it with a slider. A small preview shows the result at its real proportions, and Centre, Reset and Done buttons sit below it. When you change the shape or turn the photo, the app keeps the same middle and about the same zoom. The frame only shrinks when it would otherwise run past the photo's edge. You can drag the frame's corners and edges to tighten the crop. When you let go, the frame glides back to fill the screen. Keys work too: the arrows nudge one pixel (ten with Shift), "." centres, 1–6 and 0 pick the shape, X turns the frame, [ and ] move to the previous or next photo, and ⌘⌥R resets. If a photo was rotated or straightened in Capture One, the editor opens with a banner saying what changed. Press Return to confirm. Until you do, Export and Publish refuse and name that photo in a message.

## Try it yourself
1. Run this phase's checks (no camera or Capture One needed):
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager/apps/desktop
   npx vitest run src/features/compose
   ```
   All 136 should pass.
2. Start the app and open a session with a few previewed photos:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager/apps/desktop
   npm run tauri -- dev
   ```
3. Select a photo and press R. The Tools panel is on the left with "Composition" open. Click its title: the section folds. Quit and start the app again: it is still folded. Press ⌘T to hide the whole panel, and ⌘T again to bring it back.
4. Pick 4:5 in Shape: the frame becomes an upright 4:5 around the same middle. **Click the photo once**, then press X: the frame turns to 5:4. Press 2 to go back to square. Click Rotate right: the photo turns and the frame stays. Click Flip: the photo mirrors. Drag the straighten slider to about 5°: the photo tilts and the frame shrinks just enough to avoid empty corners. Double-click the slider to level it. The small preview follows each change.
5. Drag a corner handle inward: the frame shrinks while you hold it. Let go: it glides back to fill the screen, now with a tighter crop. Press "." to centre, use the arrows to nudge by a pixel (hold Shift for ten), and press ⌘⌥R to reset.
6. Press ] and [ to move between photos without leaving the editor. At the last photo, ] does nothing. Move the pointer to the right edge to see that the tiles kept your changes.
7. In Capture One, straighten a photo of this session a little, then open it in the editor here. A banner says "Changed in Capture One: straighten here instead". Press Return: the amber "Re-frame" corner on its tile disappears. Before you confirm, Export (or Publish) shows a message naming the photo.

## Choices made for you
- The arrow keys move the frame over the photo in the arrow's direction, as a crop box does in Capture One and Photoshop. Say "arrows move the photo instead" to swap it.
- An arrow press that cannot move the frame (it already touches that edge) saves nothing.
- Straightening only ever shrinks the frame. Levelling the photo again keeps the smaller frame, so press 2, 1 or Reset to make it big again. Say "level should grow the frame back" to change this.
- Flipping a tilted photo can shrink the frame slightly, so the crop never shows empty corners.
- Rotating a Free-shaped frame turns the frame together with the photo.
- The Done button is both in the panel and above the photo, so it stays reachable when you hide the panel.
- ⌘ shortcuts other than ⌘T, ⌘⌥R and ⌘[ keep working as usual while you compose (⌘Q quits, for example). ⌘[ no longer leaves the workspace while you compose.
- Reset goes straight to the app's default framing. Esc afterwards still returns the photo to how it was when you opened the editor.
- When Export or Publish refuses because of a changed photo, the message shows only the app's own words, for example "Re-frame first: Main 02".

## Not yet
- After you use the Shape menu or the straighten slider, that control keeps the keyboard. Keys, Return included, do nothing until you click the photo once. This is noted for the next editor phase.
- If you scroll or zoom the photo and pick a shape within about a second, the scroll can be lost and the frame jumps back to where it was before. Wait a moment after scrolling, or drag instead. This is noted for the next editor phase.
- Pressing Reset right after two quick moves can, rarely, be undone by the move that was still saving.
- While the Item details card or a header pop-up is open, all editor keys stay off until you close it.
- Split, the Z loupe, snapping, copy and paste, applying to several photos, the unsaved-changes dot and measurements are still to come.
- So far the panel, the glide and the small preview have only been checked by the test suite, not on screen. Steps 3–7 are the first real look.

## Needs you
Nothing.
