# Phase P19_navigation_and_drop: a Back button in the top-left corner everywhere, and dropped photos that stay where you drop them

## What you can do now
Every page except the sessions list now has one Back button in the top-left corner, labelled with the page it returns to. In an open session and in Settings it reads "← Sessions". In Health it reads "← Settings". The extra "Sessions" and "Settings" buttons on the right of Health and Settings are gone. ⌘[ goes back on every page that has a Back button. Escape also goes back on Settings and Health, but not while you are typing in a field. Inside a session, Escape still only closes whatever is open on top, and ⌘[ waits until a dialog or menu is closed. When you drag a photo to a new place in the grid and let go, the grid shows the new order immediately instead of waiting for the app to save it. A photo dragged into the other group appears at the end of that group straight away. If saving the new order fails, the photos go back to their saved order and you get a message.

## Try it yourself
1. Run this phase's checks (no camera, Capture One or shop needed):
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager/apps/desktop
   npx vitest run src/features/shell/__tests__/p19_navigation.test.tsx src/features/workspace/__tests__/p19_drop.test.tsx
   ```
   It should read `Tests  9 passed (9)`.
2. Start the app:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager/apps/desktop
   npm run tauri -- dev
   ```
3. Click the gear. The top-left corner shows "← Sessions" with "Settings" beside it, and "Health" is on the right. Click "← Sessions" to return to the list. Open Settings again and press ⌘[, then open it once more and press Escape. Both bring you back to the list.
4. In Settings, click "Health". The top-left corner shows "← Settings", and the only button on the right is "Retry checks". Press ⌘[ to return to Settings.
5. Open a session with three or more Main photos. Drag the first photo onto the third slot and let go. **Watch the floating preview.** It should glide into the third slot, not fly back to the first slot before jumping. Then drag a Main photo onto the Condition group: it should land at the end of Condition.
6. Still in the session, press Escape: the selection clears and you stay in the session. Press ⌘[ to return to the list. The list's header has no Back button.

## Choices made for you
- Health and Settings keep their narrow centred content column. Only the header spans the full window. Say "make Settings full width" to change it.
- Ctrl+[ works the same as ⌘[. Say "only ⌘[" if you'd rather not.
- Pressing Escape or ⌘[ on Settings leaves the page without saving fields you edited but did not save. Say "warn me before leaving unsaved settings" if you want a prompt.
- Back keys are ignored while a dialog, popover or menu is open, so Escape closes that first. Nothing to change.
- Keeping or condition-marking a photo with the review keys still waits for the save before the photo moves, because those keys also move you on to the next photo. Only drags (and the photo menu's "Move to Main" / "Move to Condition") are immediate. Say "make the review keys immediate too" to change it.
- The drop animation itself was left as it was. With the new order shown at once, the animation should now glide to the new slot, but nobody has seen it on screen yet (step 5). If the preview still flies back, say "turn off the drop animation". The preview will then simply disappear at the drop point.

## Not yet
- No one has watched the drop on a real screen yet. Step 5 is the proof that the photo no longer flies back.
- If a new preview arrives in the instant just after you drop a photo, the tile may flick back and forward once.
- ⌘[ also works while you are typing in a field, so on Settings it can leave the page while you are still editing.
- Health and Settings still have no visual design of their own beyond the shared header.

## Needs you
Nothing.
