# Phase P31_compose_mode: the photo editor opens on the stage, where you frame a photo by dragging and pinching it under a fixed square

## What you can do now
Press R on a photo, double-click it, or pick "Compose…" in its ⋯ or right-click menu, and the stage turns into the editor. A square frame stays in the middle and you move the photo underneath it. Drag or two-finger swipe to move it. Pinch, or hold ⌘ and scroll, to zoom around the pointer. You can never push the photo past its own edge. Each change saves by itself when you let go. Esc in the middle of a drag cancels that drag. Esc with nothing in progress puts the photo back the way it was when you opened the editor. Return or Done leaves the editor. While you compose, the photo list on the right is hidden: move the pointer to the right edge to slide it in, and click another photo to move the editor to it. Your changes are kept. Grid tiles now show each photo with its frame drawn on top, and a photo rotated in Capture One shows an amber "Re-frame" corner.

## Try it yourself
1. Run this phase's checks (no camera or Capture One needed):
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager/apps/desktop
   npx vitest run src/features/compose
   ```
   All 91 should pass.
2. Start the app and open a session with a few previewed photos:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager/apps/desktop
   npm run tauri -- dev
   ```
3. Select a photo and press R. Drag the photo: it follows the pointer smoothly. Let go, then move the pointer to the right edge. The tile shows the new framing. Pinch, or hold ⌘ and scroll: it zooms around the pointer, never past the photo's edge. Nothing else in the window zooms or swipes back.
4. Start a drag and press Esc before letting go: the photo snaps back. Press Esc again: the photo returns to how it was when you opened the editor. Press Return: you leave the editor. A, C, Delete and Space do nothing while it is open.
5. Drag twice quickly, two short drags back to back. The photo stays where the second drag left it and does not jump back for a moment.
6. Drag, then open the ⋯ menu above the stage and press Esc. Only the menu closes, and the photo keeps its new framing.
7. Drag a photo, then click another photo in the edge browser and drag that one too. Both tiles keep their framing.
8. The first time you open the editor, a line says "Crops made in Capture One are not used." It does not come back.

## Choices made for you
- When two saves overlap, the app waits for the first save to finish before it sends the second. The tile can show the first one for a moment, but the photo in the editor stays where you left it. This is what the fix changed. Before it, the photo in the editor could jump back to your first drag. Say "show only the final save on the tile" if the brief tile update bothers you.
- While a menu or a dialog is open over the editor, keys belong to that menu. Esc closes the menu and leaves your framing alone. This is the other thing the fix changed: before it, that Esc also threw the framing away.
- Opening and closing the editor without moving anything saves nothing, so a published product is not marked as changed.
- If the app refuses a save, you see "Not saved: …" and the photo goes back to the last framing that was stored.
- A window resize keeps the frame centred and the photo where it was.
- If you switch windows or the app is hidden in the middle of a gesture, what was on screen is saved, the same as letting go.
- ⌘[ still leaves the workspace while you compose, and your framing is saved first. Say "turn ⌘[ off while composing" to change this.
- The first-time line sits under the stage for that whole first session instead of appearing as a pop-up.

## Not yet
- If the Item details card is open while you compose, Esc and Return do nothing until you close it. Done and the card's ✕ still work, and nothing is lost. The Publish and Notifications pop-ups in the header have the same effect. This is written down for the next editor phase, which adds many more keys to the editor.
- No Tools panel, shapes other than square, turning, straightening, handles or split yet. Those come in the next phases.
- Space does not press a button that has focus while you compose.
- The editor has only been tested on screen in the test suite. Steps 3–7 are the first real try on this Mac.

## Needs you
Nothing.
