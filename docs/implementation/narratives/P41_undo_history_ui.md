# Phase P41_undo_history_ui: undo and redo everywhere in the app, with names you can see

## What you can do now
⌘Z takes back your newest action in the open session and ⌘⇧Z brings it forward again. This works in the photo grid, in the editor,
and with the details form or a picker open. The one exception is when the cursor is in a text field: there ⌘Z undoes your typing.
Every undo or redo shows a short notice naming the action, for example "Undone: Approve 03 · Redo ⌘⇧Z". When an action can no longer be
taken back, the notice says why. Two new arrow icons at the top of the session, left of the bell, do the same thing. Hover them to see
which action they would take back or bring forward; they grey out when there is nothing. The Edit menu in the menu bar now reads
"Undo Approve 03" / "Redo …" and does the same. The delete toast's Undo button uses the same history. Split or Normal on several selected
photos from the grid is now one action, so one ⌘Z puts them all back.

## Try it yourself
1. Run this phase's checks (no camera, Capture One or Shopify needed):
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager/apps/desktop
   npx vitest run src/features/history src/features/compose/__tests__/p41_editor_sync.test.tsx
   ```
   Both files should pass (14 checks).
2. Start the app:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager/apps/desktop
   npm run tauri -- dev
   ```
3. Open a session and approve three photos (A, A, A). Press ⌘Z three times: each approval comes back as unreviewed, with a notice
   each time. Press ⌘⇧Z: "Redone: Approve 01 · Undo ⌘Z". Hover the two arrows left of the bell and read their names.
4. Open the Edit menu in the menu bar: it names the same actions. Choose "Undo …". Then press ⌘Z **once** in the grid and check that
   **exactly one** action is undone, not two.
5. **Please check this one carefully.** Press D, click into Description, type a few letters, and press ⌘Z while the cursor is still in
   the field. Only your typing should disappear: no "Undone: …" notice, and no photo should change. Then click outside the field (the
   form stays open) and press ⌘Z: now the last saved edit is undone with a notice.
6. Press R to open the editor, drag the framing, press Return, and press ⌘Z without leaving the editor: the framing on the stage jumps
   back. ⌘⇧Z brings it forward.
7. Delete a photo and click Undo in its toast: the photo comes back and is selected. Delete another, wait for the toast to go, press
   ⌘Z: the same.
8. Select two photos, open ⋯ and choose Split: both split. One ⌘Z: both are Normal again.

## Choices made for you
- ⌘Z and ⌘⇧Z work with any dialog open, as long as the cursor is not in a text field. To change this, say "undo should not work while a
  dialog is open".
- The notice is a small toast at the bottom right, one at a time; each new one replaces the last. To change this, say "show undo notices
  somewhere else".
- The Undo and Redo arrows appear only inside a session, not on the Sessions page. To change this, say "show the undo arrows on the
  Sessions page too".
- The Edit menu entries show ⌘Z and ⇧⌘Z. If step 4 shows two actions undone by one keypress, say "drop the keys from the Edit menu"; the
  menu then shows no key and nothing else changes.
- When you undo a framing while the editor is open on that photo, the editor jumps to the restored framing. A photo whose save is still
  going through is left alone until the save finishes. To change this, say "the editor should not jump on undo".
- Split or Normal on one photo is one action, and on several selected photos it is also one action.
- After an undo, the photo it touched becomes the selected photo (for example, a restored deleted photo).

## Not yet
- Step 5 is the one thing the automated checks cannot show. The app now has its own Undo and Redo entries in the Edit menu, and on a real
  Mac it might send ⌘Z inside a text field to the session's history instead of to the field. If step 5 undoes a saved action (you see a
  notice, or a photo changes) instead of your typing, say "⌘Z in a text field undid an action" and it will be fixed in a small follow-up.
- When you choose Split for several photos and one of them is refused, the app does not say so yet; that photo simply stays Normal.
- Still open from the last phase: undoing a re-frame of a photo that Capture One changed brings the old framing back as finished instead
  of asking you to re-frame again. To change it, say "undoing a re-frame should ask me to re-frame again".

## Needs you
Nothing.
