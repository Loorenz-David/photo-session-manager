# Phase P22_frontend_tidy_up: readable publish times, safe typing in Settings, steady drops, form fixes

## What you can do now
The bell's notifications now show a readable local date and time ("Published · 29 Oct 2026, 14:03") instead of a long number, and that includes entries from before this phase. In Settings, clicking into the client secret clears the dots so your typing replaces them. If you click away without typing, the dots come back and the stored secret is kept. ⌘[ no longer takes you back while you are typing in a field. In a session, the photo you drag already shows the number and Cover label of the slot under your pointer, so it lands looking exactly as it did while you held it. A new photo arriving mid-drop no longer makes the dropped photo jump back. In the detail form, two quick "Looks right" clicks on different fields are both saved, and a failed one tells you why. A value the app can't read (like `12,5` in a number field) now stays in the box so you can correct it there. Pressing Escape right after editing the Title updates the workspace card. You can now clear a barcode from the form header. Back buttons on every screen take their destination from one place.

## Try it yourself
1. Run this phase's checks (no camera, Capture One or shop needed):
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager/apps/desktop
   npx vitest run src/features/form/__tests__/p22_form_tidy.test.tsx src/features/shell/__tests__/p22_shell_tidy.test.tsx src/features/workspace/__tests__/p22_workspace_tidy.test.tsx
   ```
   It should read `Tests  10 passed (10)`.
2. Start the app:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager/apps/desktop
   npm run tauri -- dev
   ```
3. Click the bell. Every "Published" entry shows a date and time, and no entry shows a long number.
4. Open Settings and click into Client secret: it empties. Click elsewhere: the dots come back. While your cursor is in any Settings field, press ⌘[ : nothing happens. Click outside the fields and press ⌘[ : you are back at Sessions.
5. Open a session with three or more Main photos, one Condition photo and, if you have one, a photo still to review. Drag photo 01 over slot 3 and hold: the floating copy reads "03" with no Cover label. Let go: it lands looking the same, with no flip. Drag 03 over slot 1 and hold: it reads "01" with Cover. Drag a Main photo over a photo still to review and hold: it reads the last Main number, and that is where it lands. Drag a Main photo over Condition: it reads the next Condition number.
6. Open a photo's form. If two fields have AI suggestions, click "Looks right" on one and then straight away on the other: both are accepted. Type `12,5` in a number field and leave it: the box keeps `12,5` and shows the Invalid marker. Change it to `12.5`: it saves and the marker goes. Edit the Title and press Escape at once: the workspace card shows the new title.
7. In the form header, click the pencil, clear the barcode and save: the barcode disappears and is still gone after you reopen the session.

## Choices made for you
- Dates use your Mac's medium date and short time format ("29 Oct 2026, 14:03"). Say "show relative times like 'today 14:03'" to change it.
- In the drag fix, the floating number and the drop now use one shared calculation, so they can never disagree. This also covers holding a Main photo over a photo still to review, which first showed "00" and now shows the last Main slot, where it actually lands. Nothing to change.
- A session page's own name is "Workspace". No screen shows it today, because no screen has a session as its parent. Say so if you want it to read differently.
- When you click "Clear" or "Accept all" on an AI suggestion, the small popup closes right away and the action finishes in the background. Say "keep the popup open until it's done" to change it.
- If you close the form twice quickly, only the first close counts. Nothing to change.

## Not yet
- Nobody has watched the drag, the glide or the Escape-after-Title on a real screen. The automated tests can't show animation, so steps 5 and 6 are the only proof that the floating copy and the landed photo match.
- If you drag a photo that is still to review and drop it back on its own spot, it is approved to Main. Whether dragging into Main should count as approving is still open.
- If you make two drops faster than the app can save the first, the second can briefly snap back.
- During a shop outage, the item search can show older shop results for a search you typed before.

## Needs you
Nothing.
