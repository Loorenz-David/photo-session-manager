# Phase P38_editor_polish: editor polish — a split line you can see, icons that show on and off, and two small dropdowns instead of drawers

## What you can do now
In Split, the line down the middle is now a white line with a dark edge, at least 2 pixels wide, so you can see it on a black cloth and on
a white wall alike. The same line appears on the big photo and on the tiles in the strip. Loupe, Snapping and the straightening grid now
show their state at a glance: a filled accent-coloured icon when on, a plain outline when off. Rotate and flip is one small icon that opens
an ordinary menu (Rotate 90° left, Rotate 90° right, Flip horizontal), which closes after you pick. The nudge arrows open from one arrows
icon as a small pad that stays open while you click its arrows and closes when you click anywhere else. Both icons sit on the degrees row
beside the grid icon, with no words and no drawers. Nothing the editor saves has changed.

## Try it yourself
1. Run this phase's checks (no camera or Capture One needed):
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager/apps/desktop
   npx vitest run p38 p37 p32
   ```
   All should pass (11 of them are this phase's own checks).
2. Start the app and open a session with photos:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager/apps/desktop
   npm run tauri -- dev
   ```
3. **Split line.** Open a dark photo (a black cloth or a night shot), press R, then S: the middle line is white with a dark edge and easy
   to see. Press Return and look at the big photo and its tile in the strip: the line is visible there too. Repeat on a very light photo.
4. **On and off.** Press R. Click the Loupe icon above the photo: it fills with the accent colour. Click again: outline. Snapping starts
   filled (on). Click it: outline. Do the same with the grid icon beside the degrees field.
5. **Rotate menu.** In Composition, the degrees row ends with three icons: grid, rotate, arrows. Click the rotate icon: a small menu with
   "Rotate 90° left", "Rotate 90° right" and "Flip horizontal". Pick one: the photo turns or mirrors and the menu is gone. Open it again
   and click elsewhere: it closes.
6. **Nudge pad.** Click the arrows icon: a small pad opens (↑ above ← ↓ →). Click → a few times and ⇧-click once: the frame moves and the
   pad stays open. Press the arrow keys on your keyboard: they still nudge while the pad is open. Click the photo: the pad closes. Open it,
   nudge once, then press Esc: the pad closes and your nudge stays.
7. **Look closely.** These checks ran in a test browser that does not draw the screen. Check by eye that the line is clearly visible, that
   the filled icons really look filled, and that the menu and the pad open neatly below their icons.

## Choices made for you
- The dark edge is a soft black (60 % opacity) one pixel around the white line. To change it, say "make the split line's edge darker" (or
  lighter, or a different colour).
- Tiles in the strip get the same 2-pixel line with the dark edge, even though the book only asks for 2 pixels in the editor and on the big
  photo. To change it, say "keep the thin line on tiles".
- "Filled" means the icon's shape is filled with the accent colour and its button border turns accent too. To change it, say "only fill
  the icon, keep the border plain".
- The rotate menu entries show an icon and their words; "no words" applies to the icon row, not to the menu. To change it, say "icons only
  in the rotate menu".
- Esc closes the nudge pad without undoing your nudges, like any dropdown. A second Esc (pad already closed) still reverts the photo as
  before. To change it, say "Esc in the nudge pad should also revert".
- Opening the nudge pad does not take the keyboard away, so the arrow keys keep nudging. While the rotate menu is open, the editor's keys
  are off, because the menu owns the keyboard as any menu does. To change either, say so.
- Neither dropdown is remembered: both start closed every time the editor opens.

## Not yet
- The undo/redo history for the editor (the next phases).
- Colour and light tools.
- The look on the real screen (line contrast, filled icons, where the menu and the pad open) has only been checked by the description
  above, not by eye. Step 7 is the first visual check.

## Needs you
Nothing.
