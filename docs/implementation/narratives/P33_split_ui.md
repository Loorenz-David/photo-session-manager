# Phase P33_split_ui: Split a photo into two square pictures, Left and Right, from the editor or the grid

## What you can do now
In the editor, press S (or click "Split" in the Composition section) to turn the frame into a wide 2:1 box with a thin line down its exact middle. The two halves become two square pictures, Left and Right, and the Composition section shows both side by side. The framing stays centred on the same spot at about the same size, and pressing S again brings back your previous shape exactly. Dragging, zooming, nudging, the corner handles (which keep the box at 2:1), moving between photos, Esc and Return all work as before. Back in the grid, a split photo shows a thin middle line and counts as two places, so it reads "01–02" and the next photo reads "03". This works in Main and in Condition, in the drag preview and in the stage bar ("Main 4–5 of 6"). You can also right-click any photo, or a ⌘-selection of photos, and choose Split or Normal.

## Try it yourself
1. Run this phase's checks (no camera or Capture One needed):
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager/apps/desktop
   npx vitest run p33 outputs.test
   ```
   All 18 should pass.
2. Start the app and open a session with a few previewed photos:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager/apps/desktop
   npm run tauri -- dev
   ```
3. Select a Main photo, press R, then press S. The frame becomes a wide box with a line down its middle, and two small squares labelled Left and Right appear in the Tools panel. Pull a corner handle: the box stays twice as wide as it is tall. Pick 4:5 in Shape and press X: the box does not change. Press S again: a 5:4 frame (4:5 turned sideways) comes back around the same middle.
4. Press S once more to split again, then Return. The tile shows a thin line down its frame and reads "01–02" if it is the first photo. The next photo reads "03", and the stage bar says "Main 1–2 of …". Press F: the folded strip shows the line too, with no numbers.
5. Right-click a normal photo and choose Split, then right-click a split photo and choose Normal. ⌘-click two photos, right-click one of them and choose Split: both become split, and a photo that was already split is left alone. Do the same with a Condition photo: Condition renumbers "01–02, 03–04".
6. Drag a split tile to another place: the floating tile shows the two numbers it would take there.
7. Export the session: the split photo gives two square files, Left first. (Publishing works the same way: two pictures next to each other, Left first, and the cover is the left half when the split photo is first in Main.)

## Choices made for you
- Split is only for Main and Condition photos. On a photo still to review, the Split button and menu entry are greyed out and S does nothing. Such a photo counts as one place. Say "let me split photos before reviewing them" to change this.
- The menu's Split / Normal entry shows the S key beside it, like Compose shows R.
- If you use the menu's Split / Normal on the photo you are editing, the editor changes with it immediately.
- When several photos are split from the menu, they are saved one after another, and each tile updates as soon as its save finishes. If one fails, you see a "Not saved" message for that photo only.
- The Left and Right previews are at most 110 points wide each and use the same preview image as the stage.
- The middle line is 1 pixel thin in the editor and in the tile.
- The stage bar writes the two places with a dash, like the tile: "Main 3–4 of 8".
- While you drag a tile, its preview shows the numbers it would have where you are about to drop it.

## Not yet
- If you choose Split from the menu and reopen the menu before the save has finished, it still offers Split. Picking Split again saves the same thing twice. Picking Normal right then is ignored and the photo stays split. On a normal-speed save you won't notice. This is noted for the next editor phase.
- Splitting or unsplitting a photo while a publish is running can make that product fail to publish, and Retry keeps failing. Starting a new publish fixes it and nothing is lost. Wait for a publish to finish before changing Split. This was already known and is noted.
- Everything here was checked by the test suite only, not on screen. Steps 3–6 are the first real look.
- The Z loupe, snapping, copy and paste of framing, the unsaved-changes dot and measurements come in the next editor phase.

## Needs you
Nothing.
