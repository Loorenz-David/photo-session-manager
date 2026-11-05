# Phase P34_composition_polish: a zoom loupe, snapping, copy and paste of framing, an unsaved dot, calmer motion, a bigger window and a speed meter

## What you can do now
In the editor, hold Z and a round loupe follows the pointer, showing the photo at full size exactly where you point. It is turned, flipped and tilted the same way the photo is, and letting go of Z changes nothing. While you drag the photo, it clicks into place when its centre or an edge comes within a few pixels of the frame's centre or edge, and a guide line shows. Hold ⌥ to drag freely. ⌥⌘C copies a photo's framing and ⌥⌘V pastes it onto another, fitted to that photo's own shape. In the grid, "Apply composition to selected" pastes it onto every selected photo at once, and the message names any photo that could not take it (that photo keeps its own framing). A small dot marks a photo while its change is saving, and it goes away when the save lands or fails. With "Reduce motion" turned on in macOS, glides, slides and the Original/Adjusted fade become instant. The window now opens at 1440 × 900 and cannot be made smaller than 1100 × 720. ⌥⌘P shows a small meter with the average and worst frame time while you drag or pinch, and every master and output render writes how long it took to the daily log.

## Try it yourself
1. Run this phase's checks (no camera or Capture One needed):
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager/apps/desktop
   npx vitest run p34
   cd ~/Desktop/Developer/Photo-Session-Manager
   cargo test -p psm-testkit --test p34_composition_apply
   ```
   9 and 2 should pass.
2. Start the app and open a session with a few previewed photos:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager/apps/desktop
   npm run tauri -- dev
   ```
3. Select a photo, press R, rest the pointer on a detail (an eye, a label) and hold Z. The loupe should show that detail at its centre without moving the mouse. Let go: nothing moved. Now click "Rotate right" and "Flip" in the Composition section and tilt the photo a few degrees with the straighten control, and hold Z again over the same detail. The loupe should still show it, turned, flipped and tilted like the stage.
4. Drag the photo slowly past the frame's centre: it clicks into place and a guide line shows. Drag again holding ⌥: it moves freely.
5. ⌥⌘C on a framed photo, `]` to a photo of a different shape, ⌥⌘V: the same framing lands, fitted to that photo. Select several tiles, right-click → "Apply composition to selected". All of them take it, and any that cannot is named in the message.
6. Drag and watch the tile: a small dot shows until the save lands. Turn on System Settings › Accessibility › Display › Reduce motion: the glides, the browser slide and the Original/Adjusted fade become instant.
7. Quit and relaunch: the window opens at 1440 × 900 and will not shrink below 1100 × 720.
8. Measure speed (this was not measured for you, see below). Press ⌥⌘P, then drag and pinch for a few seconds and note the average and worst numbers. Your target is your display's refresh time (about 8 ms at 120 Hz, 17 ms at 60 Hz). Then export a session and read the render times:
   ```
   grep -E "master rendered|render_output done" ~/Library/Application\ Support/com.loorenz.sessionmanager/logs/psm.$(date +%F).log
   ```

## Choices made for you
- Copied framing is kept inside the app only, not on the macOS clipboard, so it is gone after quitting. Say "put copied framing on the system clipboard" to change this.
- Pasting keeps the copied framing's centre and size and fits it to the target photo's shape, the same rule the editor already uses when you change shape.
- A selected photo that has no framing yet is skipped by "Apply composition to selected" and named in the message.
- The speed meter only counts frames while you are dragging, pinching or scrolling the photo, and shows the last second.
- The loupe is a 240-point circle.
- After review, the loupe was corrected twice. It now matches the photo's turn, flip and tilt (at first it showed an unturned picture squeezed into the frame, so it showed the wrong pixels). It also opens right under a resting pointer (at first it opened in the top-left corner until you moved the mouse). The checks were tightened so both mistakes would now be caught.

## Not yet
- The frame and render times were not measured here: the session could not drive the app on screen. Step 8 is the first measurement. Tell the orchestrator the numbers, and do the same on the work Mac with the a7R when you can.
- The guide line is always drawn at the frame's centre, even when the photo snapped to an edge.
- After "Apply composition to selected", a photo that showed "Re-frame" keeps that badge until you reload, although the change is saved.
- Apply (and the grid's Split/Normal, and Reset) does not wait for a photo's pending save, so a very quick edit followed by Apply on the same photo could end on the older framing. This is noted, and the fix is to put all framing saves in one queue.
- Everything on screen was checked by tests only. Steps 3–8 are the first real look.

## Needs you
Nothing. (Step 8's numbers are something to report back, not a decision.)
