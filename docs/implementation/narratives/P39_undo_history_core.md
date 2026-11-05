# Phase P39_undo_history_core: undo history — up to 25 review actions per session can be taken back and redone

## What you can do now
The app now keeps a history of your review actions for each session: approving, sending to Condition, deleting, moving back to Main,
dragging to a new place, moving photos to another session (including Assign product) and copying them. It keeps up to 25 of them per
session. Today's ⌘Z already uses this history, so it no longer only brings back the last deleted photo: each press takes back your most
recent action of any kind, as one whole change, and puts the photos back exactly where they were. If an action can no longer be taken
back, for example because the session it would put photos into has been published since, nothing changes, the app tells you why ("Can't
undo Move to Lamp 2: that session is published"), and the next ⌘Z goes on to the action before it. The history survives a settings change
and is cleared when you quit. Redo (⌘⇧Z) is built and tested, but no button or key reaches it yet.

## Try it yourself
1. Run this phase's checks (no camera, Capture One or Shopify needed):
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager
   cargo test -p psm-testkit --test p39_undo_history_core
   cargo test -p psm-desktop p39_tests
   ```
   Both should pass (24 and 3 checks).
2. Start the app:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager/apps/desktop
   npm run tauri -- dev
   ```
3. Open a new session with a few unreviewed photos. Approve three with A, then press ⌘Z three times: each press takes back the newest
   approval and the photo shows as unreviewed again. A fourth ⌘Z says "Nothing to undo."
4. Delete a photo, drag another photo to a new place in the strip, then press ⌘Z: the drag is undone first. Press ⌘Z again: the deleted
   photo is back in its old place. With Capture One running, the colour tags follow within a few seconds (green, blue or none).
5. Move a photo to another session, come back to the first session and press ⌘Z: the photo is back where it was.
6. Change a setting (Settings → SKU prefix), come back to the session and press ⌘Z: the actions from before the change can still be taken
   back. Quit and start the app again: ⌘Z says "Nothing to undo", and your photos stay as you left them.

## Choices made for you
- An action is taken back only if its photos are still where it left them. If something that is not in the history changed a photo, that
  action is skipped with "that photo already changed". To change this, say "undo should still apply when the photo changed since".
- Taking back a copy deletes the copied photos from the other session, since they did not exist before. This is refused once that session
  has been published, or a publish of it has started, after the copy. To change this, say "let me undo a copy after publishing".
- Taking back a move into a session that is published and unchanged is refused the same way. To change this, say "let undo move photos
  back into a published session".
- Action names use the photo's number in Main or Condition ("Approve 03", "Reorder 03 → 01"). Moves and copies use the other session's
  product title, or its SKU when it has no title ("Move to Lamp 2", "Move 2 photos to Lamp 2"). A photo without a number is called
  "photo" ("Delete photo"). To change the wording, say how it should read.
- When photos from several sessions are moved together, each session gets its own entry in its own history. To change this, say "a move
  from several sessions should be one action".
- Taking back an approval or a delete makes that photo the open one. Taking back other actions leaves the open photo alone.
- The history survives a settings change and is empty after the app restarts. The old single "last delete" slot is no longer used; its
  column stays in the database, unused.
- If an undo fails because of a disk or database error, rather than because of the rules above, the action stays in the history for
  another try.
- Redoing a move puts each photo back at the number it had after the move. If the other session gained photos in between, the photo can
  land among them instead of at the end. To change this, say "redo should always put moved photos at the end".

## Not yet
- Redo (⌘⇧Z), the Undo and Redo icons in the header, the Edit menu entries and the "Undone: Approve 03" notice. The next phases wire these
  to the screen.
- Taking back framing changes and product details edits (next phase).
- A rare case where a redo of a copy fails over and over, when the photo it copied was itself a copy that was later taken back. It is noted
  for the next phase, which works on the same code. No screen can reach it today.

## Needs you
Nothing.
