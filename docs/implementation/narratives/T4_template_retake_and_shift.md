# Phase T4_template_retake_and_shift: retakes and bad shots

## What you can do now
In a session with a shooting template, a bad shot no longer ruins the order. Say you reshot a bad angle at the end and the new photo landed under Condition. Drag it onto the bad angle's container: it takes that container with the container's split and zoom, and no warning follows. The old photo turns red as a denied photo, and ⌘Z brings it back. If you deny a photo instead, its container stays empty and nothing else moves. If you reshot it straight away, the photos after it are each one container too far. A **Shift the rest back** button then appears and moves them back, all in one step you can undo. Your whole story, from a new template to a wrong drag fixed with Back to template, now runs as one automatic test.

## Try it yourself
1. Start the app on scratch data:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager-agentic/apps/desktop
   PSM_DATA_DIR=$HOME/psm-templates-dev npm run tauri -- dev
   ```
2. Open a new dining-chair session and shoot the chair. Pick one angle that came out badly and press Deny on it. Its container is empty and the other photos stay where they are.
3. Reshoot that angle at once, then shoot the next angles. Every photo after the gap sits one container too far. Press **Shift the rest back**: they move back one. Press ⌘Z: the gap is back. Press ⌘Z again: the denied photo is back.
4. In another chair session, deny one angle and reshoot it at the end instead. The new photo lands under Condition. Drag it onto the bad angle's container. The old photo turns red, the new one sits in the container with its split and zoom, and no "differs" warning shows. ⌘Z undoes the whole retake.
5. Deny a photo under Condition, after the last container. **Shift the rest back** does not appear, because nothing would move.
6. Run through your whole story once: template, new chair, shoot out of order, Condition after the last angle, four Condition photos to Main, a wrong drag, and Back to template.

## Choices made for you
- A retake is a photo outside the containers (a Condition photo or an extra Main photo) dropped onto a filled container. It takes the container and its split, shape and zoom, and counts as the shot it replaced, so no warning follows. Two photos that both sit in containers still swap. Say "a retake should need a confirmation" to change it.
- The replaced photo is denied like any other denied photo: it turns red, you can undo it, and its file is deleted when the undo window closes.
- A denied photo keeps its container empty and still counts as a shot, so later photos stay put until you press **Shift the rest back**.
- **Shift the rest back** moves only photos the template placed. Photos you placed by hand stay where you put them.
- Undo order: the first ⌘Z after Shift the rest back brings the gap back, and the next ⌘Z brings the denied photo back.
- **Changed during review:** the first build showed **Shift the rest back** whenever any later photo existed. So after you denied a Condition photo, the button appeared, and pressing it moved nothing but still took up an undo step. Now the button shows only when pressing it would actually move a photo. To change this, say "show Shift the rest back after every deny".

## Not yet
- The app does not look at the pictures. It does not spot a bad shot or recover the order on its own. You deny and drag.
- Two small edges remain from the previous phase. Dragging a photo from a container to behind the extra photos moves nothing visibly, yet gives the photo a dot, and ⌘Z does not remove that dot (**Back to template** does). That kind of drag can also give the photo a container number the template doesn't have.
- No hardware try-out was done in the build sessions. Your try-out above is the first run with the camera.

## Needs you
Nothing.
