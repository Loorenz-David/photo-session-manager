# Phase T3_template_hand_changes: your changes win, and there is a way back

## What you can do now
In a session that has a shooting template, anything you change by hand stays as you left it. That covers moving a photo, classifying it differently, re-framing it, or moving it to another session. The app never moves it back on its own. Each photo that differs from the template gets a small amber dot. A quiet line above Main says how many photos differ ("2 photos differ from the template") and has a **Back to template** button. That button puts only the differing photos back, with their container's split and zoom, and it is one step you can undo. You can also drag a photo onto a container: an empty container takes it, and two placed photos swap. Sessions without a template work as before.

## Try it yourself
1. Start the app on scratch data:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager-agentic/apps/desktop
   PSM_DATA_DIR=$HOME/psm-templates-dev npm run tauri -- dev
   ```
2. Open a dining-chair session where every container is filled and there are a few Condition photos.
3. Move four Condition photos to Main. A line says "4 photos differ from the template" and those photos get a dot. Shoot another photo: nothing moves back.
4. In another chair session, drag the split photo from place 3 onto place 2. The two swap, both get a dot, and the line says 2 photos differ.
5. Press **Back to template**. Both photos return and the line goes away. Press ⌘Z: your swap is back. Press ⌘⇧Z: the template order again.
6. Open Compose and re-frame the split photo. It gets a dot. **Back to template** gives it the split and zoom again.
7. Drop a Condition photo onto an empty container. It is placed there.
8. Drag one extra Main photo (one after the containers) onto another extra, then press ⌘Z. The two go back to their old order.

## Choices made for you
- When you drop a placed photo onto another filled container, the two photos swap and both get a dot. Say "drop should push the others along instead" to change it.
- You can't drop a Condition photo onto a filled container yet. You get a short "That container is filled" message. The next phase turns this into a retake.
- A photo you put into a container by hand keeps its own framing. The container's split and zoom only come with **Back to template**.
- A re-framed photo counts as differing only when its container has split or zoom settings.
- A photo you denied is not counted as differing, and **Back to template** never brings it back. Undo does.
- **Back to template** touches only the photos that differ, and it uses the template as it is right now.
- Some photos show as differing, and **Back to template** files them by the time they were shot:
  - photos that were in the session before its template existed;
  - photos moved in from another session;
  - photos shot into Unassigned and then assigned. One tap files these.
- If undo moves a photo back from Condition, it keeps the container it was last in.
- Main is always ordered the same way: photos in containers, in container order, then the extra photos in the order you gave them. This holds after a drag, a swap, an undo and **Back to template**.
- A swap also swaps the two photos' places in the export order.
- If a photo goes back to a container without split or zoom settings, or to Condition, it keeps its framing.
- The warning line also updates after a classify or re-frame, not only when new photos arrive.
- **Changed during review:** undoing a drag between two extra Main photos left them where you had dragged them, and the undo had no visible effect. Now undo puts both photos back exactly as they were before the drag. That includes whether the photo counts as placed by hand, so a photo you drag and then undo loses its dot. To have it behave differently, say "undo of a drag should keep the photo marked".

## Not yet
- **Retakes** come in the next phase: dropping a new shot onto a filled container to replace a bad one. So does **Shift the rest back** after a denied shot.
- One known edge for the next phase. If you drag a photo from a container to behind the extra photos, nothing visibly moves, but the photo gets a dot. ⌘Z does not remove that dot. **Back to template** does.
- The agent guide for developers is updated in the next phase.
- No hardware try-out was done in the build sessions. Your try-out above is the first run with the camera.

## Needs you
Nothing.
