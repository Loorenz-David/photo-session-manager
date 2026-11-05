# Phase P36_editor_controls_and_overlays: a steady photo strip in the editor, alignment lines, typed straightening, a button and hint for every key, and the keyboard back after the menus

## What you can do now
In the editor, the photo browser on the right is now a thin strip that stays where it is. Nothing slides in from the edge any more.
Click a photo in the strip and the editor moves to it. Press F to open the full photo panel beside the editor, and F again to fold it.
When you leave the editor, the panel is back the way you had it.

The frame shows alignment lines: Thirds, Grid, Golden ratio or Diagonals, or none. Pick them in the Composition section or press O to
cycle through them. The app remembers your choice after a restart. In a split photo, each half gets its own lines.

Straighten now takes a typed number of degrees as well as the slider. ↑ and ↓ step by 0.1°, and ⇧ with an arrow steps by 1°. While you
straighten, a fine square grid covers the photo. It has its own on/off switch, which is remembered.

Every editor key now has a button in the Tools panel or above the photo. Hover any button and its hint gives the name and the key, for
example "Split (S)". After you use the Shape menu, the Overlay menu or the straighten slider, the keys work again at once, so there is
no need to click the photo first. When the photo snaps to an edge, the guide line now appears on that edge instead of in the middle.

## Try it yourself
1. Run this phase's checks (no camera or Capture One needed):
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager/apps/desktop
   npx vitest run p36 overlays
   ```
   21 should pass.
2. Start the app and open a session with photos:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager/apps/desktop
   npm run tauri -- dev
   ```
3. **Strip.** Press R. The right side becomes the thin strip of photos and stays put. Click another photo in the strip: the editor moves
   to it. Press F: the full panel opens beside the editor. Click a photo there: the editor moves to it. Press F again to fold. Press Return:
   the panel is as you left it before R.
4. **Lines.** Thirds lines sit inside the frame. Press O repeatedly: Grid, Golden ratio, Diagonals, Off, Thirds. Pick one in the Overlay
   menu, quit and relaunch: it is remembered. Split a photo: each half has its own lines.
5. **Straighten.** Click into the degrees field: a fine grid covers the photo. Type 2.5 and press Enter: the photo tilts 2.5°. Press ↑ and
   ↓ (0.1°) and ⇧↑ (1°). Turn "Straightening grid" off: no grid next time. Drag the slider: the grid shows while you hold it.
6. **Buttons and hints.** Hover every button in the Tools panel and above the photo: each hint gives its name and key. Click Loupe,
   Snapping, Copy framing, Paste framing, Tools panel, Frame meter, Revert, the nudge arrows and the ‹ › arrows: each does what its key does.
7. **Keyboard back.** Pick a shape in the Shape menu and press X right away: the frame turns. Drag the slider, let go, and press X: it turns
   again.
8. **Guide.** Drag the photo so its left edge meets the frame's left edge: the guide line appears on that edge, not in the middle.

## Choices made for you
- The editor always opens with the strip folded. F opens the full panel for that editing session only. Say "remember the open panel in the
  editor" to change this.
- The default lines are Thirds, as in Capture One and Photoshop. O cycles Off → Thirds → Grid → Golden ratio → Diagonals. Say "start with no
  lines" to change this.
- Grid is 6 × 6. Golden ratio draws lines at 38.2 % and 61.8 %. Diagonals are Photoshop's 45° corner lines, which on a square are just the
  two diagonals.
- In a split photo each half gets its own lines, because each half is its own picture. Say "one set of lines over the whole split" to
  change this.
- The straightening grid shows while you hold the slider or while the degrees field has the keyboard. It is fixed to the screen, with
  24-pixel squares, and covers the whole photo area. Say "a coarser grid" or "grid tilted with the photo" to change this.
- The degrees field accepts a comma as the decimal mark. Esc in the field cancels your typing and leaves the photo as it was.
- If you leave the degrees field without changing the number, nothing is saved.
- Typed degrees are saved when you press Enter or click away from the field.
- The arrow keys also have a small clickable nudge pad. Zoom has no buttons, because pinch and ⌘-scroll are gestures, not keys.
- The Loupe button keeps the loupe on until you click it again. Z still shows it while held. The Snapping switch is remembered.
- Esc while a hint is showing closes the hint first. A second Esc reverts as before.
- With straightening at 10°, the frame shrinks to about 86 % of the photo's height so it stays inside the tilted picture. The plan said
  82 %, which was an old arithmetic slip that an earlier phase had already fixed.
- In the review round, only the automatic checks were changed, not the app. The check for "keys work again after the Shape menu or the
  slider" now actually starts with the menu or slider holding the keyboard, as it does when you click it. The check for clicking a photo in
  the open panel now clicks a different photo, so it proves the editor really moves.

## Not yet
- Zoom buttons, colour and light tools, and one save queue shared by every editor action are later work.
- The strip, the lines, the straightening grid and the button row above the photo were checked by tests only, not by looking at the
  running app. Steps 3–8 are the first real look.
- If you type a new number of degrees and then start dragging the photo straight away, without pressing Enter, the typed number may be
  dropped. Press Enter first. This is noted to be fixed.
- One older automatic check from the split phase still sometimes fails when all checks run together. It did not fail in this round.

## Needs you
Nothing.
