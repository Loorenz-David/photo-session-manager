# Phase P08_workspace_ui: the session workspace for shooting, sorting and ordering

## What you can do now
Opening a session now shows the real workspace.

**Shooting and sorting**
- Each new shot appears on the stage. It shows the camera's own picture first, with a small spinner. Capture One's adjusted version fades in when it is ready.
- Press A to approve a photo into Main, C to put it in Condition, or Delete to hide it. You can also use the three round buttons or swipe. After each action the next photo still to review opens.
- A deleted photo can be brought back with ⌘Z or the toast's Undo button. It returns to the same place.
- Space switches between the camera original and the adjusted version.
- The arrow keys step through the photos.

**Ordering**
- The panel shows Main first, then the photos still to review, then Condition.
- Drag a photo to reorder it within its group.
- Drag a photo into the other group to reclassify it. It goes to the end of that group.
- The panel can fold into a thin strip.

**Moving and copying**
- The ⋯ menu and right-click offer Move to session…, Copy to session…, Assign to product… and Delete.
- Unassigned opens with the sessions list beside it. Drag photos onto a session to move them there. Published sessions do not accept drops.
- "Assign product" sends the whole Unassigned set to one item's session.

**Importing and arrivals**
- You can drop RAW files onto the workspace, or pick them with a file dialog.
- When shots land in Unassigned while you are on another screen, a toast with an Open button appears.

## Try it yourself
1. Run this phase's checks. They use pretend services, so neither Capture One nor the camera is needed:
   ```
   export PATH="$HOME/.cargo/bin:$HOME/.nvm/versions/node/v22.22.3/bin:$PATH"
   cd ~/Desktop/Developer/Photo-Session-Manager
   cargo test -p psm-testkit --test p08_workspace_ui
   cd apps/desktop && npm test
   ```
   The first command should end with `test result: ok. 1 passed`. The second should end with `Tests  43 passed (43)`.
2. Open your Capture One session, tether the camera, and start the app:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager/apps/desktop
   npm run tauri -- dev
   ```
3. Open a session and take a picture. Within about a second it should be on the stage with a spinner on the pill, and the adjusted version should fade in a few seconds later. Take a second picture. The stage should jump to the new one.
4. Press A, C and Delete on three photos. The next photo to review should open each time. Press ⌘Z: the deleted photo should come back where it was. Drag a Main photo to another slot, then drag a Condition photo into Main.
5. Press Space to compare the original and the adjusted version. Edit the photo in Capture One, then come back to the app: the preview should update with a fade.
6. Click Back. With no session open, take one picture. A toast "Photos arrived in Unassigned · Open" should appear. Click Open, then drag the photo onto a session in the panel.

## Choices made for you
- The fix changed one thing: a new shot taken in the open session now becomes the photo on the stage. A, C and Delete then act on that new shot. Before the fix, the first shot of a session left the stage empty, and later shots left the keys acting on the previous photo. Nothing to change.
- Imported files behave like new shots: the stage jumps to the photo that arrived last. Say so if you would rather stay on the photo you were looking at.
- Pressing A on a photo already in Main, or C on one already in Condition, changes nothing. It only opens the next photo to review.
- Dragging a photo into the other group reclassifies it but does not change which photo is on the stage.
- You can ⌘-click or ⇧-click several photos. Dragging and the menus then act on all of them.
- "Assign to product…" moves the photos into that item's session, creating the session if needed.
- "Assign product" in Unassigned only searches items you already have. It has no Create row.
- The item details card (press I) opens in the top right of the stage.
- Any workspace can open the sessions list with a Sessions button. Unassigned opens it by default.
- Dropped or picked files are all sent to the import. The app keeps only the RAW files.
- Photos still to review are numbered after Main in the panel. These numbers are for display only.
- There is no shutter button (S) in the app yet. Say so if you want one.

## Not yet
- Export, Publish and the form come in later phases. Those header buttons are not active here.
- The "Importing N files" toast counts every file you dropped, including any that are not RAW.
- When a new shot arrives, the stage may show "Ready for the first photo" for a split second before the new photo appears. Tell me if you see it during step 3.

## Needs you
Nothing.
