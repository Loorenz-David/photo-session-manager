# Phase P40_undo_history_compositions_and_attributes: undo history — framing changes and product details edits can be taken back and redone

## What you can do now
The undo history now also records your framing changes and your product details edits, in the same list as your review actions, newest
first. Each saved framing is one action: a drag you finish, a control you change, Reset, Paste framing, Split/Normal, or Esc going back.
"Apply composition to selected" is one action for all the photos it changed. Taking a framing back saves the old framing as a new version,
so the tile redraws and the next export or publish makes fresh images. In the details form, saving a field, Accept all and Clear are each
one action. Taking one back restores the field exactly as it was, including an AI suggestion's confidence and sources, or empties a field
that was empty before. If Capture One has changed a photo since you framed it (the amber "Re-frame" state), taking that framing back is
refused with a notice and the next ⌘Z goes on to the action before it.

## Try it yourself
1. Run this phase's checks (no camera, Capture One or Shopify needed):
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager
   cargo test -p psm-testkit --test p40_undo_history_values
   cargo test -p psm-desktop p40_tests
   ```
   Both should pass (18 and 2 checks).
2. Start the app:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager/apps/desktop
   npm run tauri -- dev
   ```
3. Open a photo in the editor (R), drag the framing, press Return, then press ⌘Z: the framing goes back to where it was and the tile
   redraws. Press ⌘Z again: the action before it is taken back (an approval, for example).
4. Copy a framing from one photo (⌥⌘C), select three photos, and choose Apply composition to selected from the ⋯ menu. Press ⌘Z once: all
   three get their old framing back.
5. Open Details (D), type a colour, close the form and press ⌘Z. Open Details again: the field is empty, because the whole word you typed
   was one action.
6. With Capture One running, straighten a photo whose framing you changed and wait for "Re-frame". Press ⌘Z: the app says the photo
   changed in Capture One and to re-frame it first, and the next ⌘Z takes back the action before.

## Choices made for you
- Typing into one field makes one action, even with pauses, until you do something else (another field, a photo action, an undo). To
  change this, say "each pause while typing should be its own undo".
- Clear and Accept all are always their own actions. An AI suggestion is never an action, because you did not do it. To change this, say
  "AI suggestions should be undoable too".
- Taking a framing back always saves a new version, so exported images and shop images refresh at the next export or publish, as after
  any framing change.
- Taking back a details edit refreshes the session's "last activity" time but does not mark a published session as changed, since
  details edits never do.
- Apply composition to selected is one action even when some photos were refused. Only the photos that changed are in it, and undoing it
  counts as one change per photo, as applying it did.
- Split/Normal on several photos from the grid is still one action per photo until the next phase routes it through Apply. To change
  this, say "grid Split/Normal on a selection should be one action".
- Clearing the title is its own action, "Clear Title".
- Redoing a copy whose original photo has since been removed is refused with "that photo is gone", and that entry leaves the list. This
  fixes the rare stuck redo noted last phase.

## Not yet
- Redo (⌘⇧Z), the Undo and Redo icons, the Edit menu entries and the "Undone: …" notice come in the next phase. Today only ⌘Z reaches
  the history.
- One case to settle before then. Capture One changes a photo, you re-frame it, then you press ⌘Z. Your old framing comes back as
  finished, and the photo does not return to the amber "Re-frame" state, so that old framing would be exported without you confirming it
  against the changed photo. It still fits inside the image. It is noted for the next phase. If you want undo to bring back the
  "Re-frame" state instead, say "undoing a re-frame should ask me to re-frame again".

## Needs you
Nothing.
