# Phase T2_template_placement: photos file themselves into the template's containers

## What you can do now
When a session's item type has a shooting template, each photo you shoot goes into its own container. The order of the containers follows the template, not the order the files arrive. The app goes by when each shot was taken. If a file arrives late, the photos still end up in the right containers. A container with Split and Zoom applies them to its photo by itself. Photos taken after the last container's shot arrive under Condition. The session's Main area shows the containers in their final order. An empty container is a faded box with its number. A small **Template** button opens the template: the black-and-white examples, their numbers and their presets. Anything you change by hand stays as you left it. Sessions whose type has no template work exactly as before.

## Try it yourself
1. Start the app on scratch data:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager-agentic/apps/desktop
   PSM_DATA_DIR=$HOME/psm-templates-dev npm run tauri -- dev
   ```
2. In Settings → Shooting templates, make sure there is a "Dining chair" template with about five containers. Give the third one Split and Zoom 20, and set a shooting order that differs from the final order.
3. Make a new session for an item of type Dining chair and open it. The Main area shows five faded, numbered boxes and a small **Template** button. Press it to see the examples, their numbers, "Shot n" and the presets.
4. Shoot the angles in your shooting order. Each photo lands in its own container. The photo for the third container is split and zoomed.
5. Keep shooting after the last angle. Those photos arrive under Condition.
6. Move a Condition photo to Main. It stays there when the next photo arrives.
7. When all containers are filled, drag one Main photo to another place. It stays where you put it, also after the next photo arrives.
8. Open a session of a type without a template. Everything is as before.

## Choices made for you
- Shots are ordered by the time the shutter fired. Among camera files shot within five minutes of each other, the camera's own file number decides. Files from Capture One's tether and imported files go by the file's time.
- If the camera reported a shot whose file has not arrived yet, its container stays empty for up to five minutes. After that, the next photo that arrives closes the gap.
- If the second shot arrives before the first, it sits in the first's container for a few seconds. Once the first arrives, both are right.
- If you change a template while a session is being shot, the change applies from the next photo that arrives. Photos you placed by hand stay.
- Photos that were in the session before its template existed stay where they are, and they count as shots.
- A template with no containers works like having no template.
- Extra Main photos come after the containers.
- Each photo is placed in one step with its presets. If placing a photo fails, the photo is still kept and simply not placed.
- An empty container is a dashed, faded box with its number. The Template button is small and outlined, with an info icon. The folded strip stays as it was.
- **Changed during review:** dragging a Main photo to another place in a templated session used to be undone. The photo snapped back behind the template's photo, while Undo still listed the move. Now a photo you drag keeps the place you gave it. The other photos fill the remaining places in container order, also when more photos arrive. One limit for now: if you drag while an earlier container is still empty, the screen can show the photo one container too early until that container fills. The export uses the order you made. The next template phase reworks dragging onto containers. To have it behave differently, say "a dragged photo should always stay right behind the photo I dropped it on".

## Not yet
- No warning or mark yet when photos differ from the template, and no **Back to template**. Both come in the next phase.
- Dropping a photo onto a container, retakes, and **Shift the rest back** after a bad shot come in the next two phases.
- Undoing an older Classify in a templated session can leave Main out of container order until the next photo arrives. The next phase closes that.
- No hardware try-out was done in the build sessions. Your try-out above is the first run with the camera.

## Needs you
Nothing.
