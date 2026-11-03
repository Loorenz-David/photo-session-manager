# Phase P24_capture_one_provisioning: the app sets up Capture One by itself

## What you can do now
You no longer have to prepare a Capture One session by hand. If you haven't picked one, the app creates its own session, "Session Manager", in your Pictures folder. It starts Capture One when it's closed, opens that session next to whatever you already have open (your Instagram session stays open), creates the two export recipes it needs and puts them back if they're changed, and applies your chosen style to new captures. When you open a session in the app, Capture One brings the app's session to the front, so shots can't land in another session. If something blocks the setup (Capture One not installed, not the Pro edition, macOS not allowing control, a style name that doesn't exist), Settings → Health says so in plain words, and the rest of the app keeps working. In Settings you can switch between "Session Manager (created by the app)" and "Use another session…".

## Try it yourself
1. Run this phase's checks (no camera or Capture One needed):
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager
   cargo test -p psm-testkit --test p24_capture_one_provisioning
   cd apps/desktop && npx vitest run
   ```
   Both should report all passed.
2. Quit Capture One, then start the app:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager/apps/desktop
   npm run tauri -- dev
   ```
3. Settings: the Capture One block should show "Session Manager (created by the app)" and the path `~/Pictures/Session Manager`. If it still shows `session test 1`, press "Use the app's own session" and Save. Within about half a minute Capture One opens by itself, showing a new "Session Manager" session beside your Instagram session. Capture One → Process Recipes lists `PSM Preview` and `PSM Web`, both unticked.
4. Settings → Health: "Capture One document", "Recipes" and "Installed hooks" are green. If you have no style called `PSM House`, "Style" is red with "Style 'PSM House' not found in Capture One". Clear the Style field and Save (or create that style), press Retry checks, and the red row goes away.
5. In Capture One, click your Instagram session so it's in front. Then open a session in the app: Capture One's "Session Manager" session comes to the front, and Instagram stays open behind it.
6. Tether the camera and shoot one frame. It lands in `~/Pictures/Session Manager/Capture/<the item's folder>` and appears as a tile.
7. Quit Capture One while a session is open in the app, then press Health → Retry checks. Capture One starts again with the app's session in front.
8. In Capture One, tick `PSM Web` and set its quality to 70. Press Health → Retry checks: it's unticked and back at 90. Your own recipes are never touched.
9. Settings → "Use another session…", pick `~/Pictures/session test 1/session test 1.cosessiondb`, and Save. Health stays green. "Use the app's own session" plus Save switches back.

## Choices made for you
- On the first try, running the app's automatic tests reached your real Capture One. The tests created a "Session Manager" session and pointed Capture One's hooks at a temporary folder. I put all of that back during the review. The fix makes those tests use a stand-in for Capture One, and I confirmed the tests now leave Capture One untouched. Nothing for you to change.
- To open a closed session without closing yours, the app first creates a small throwaway window called "PSM Open Window", then opens the real session in its place. On the first try this two-step move was split so the tests couldn't see it; the fix moved it into one script the tests now check. I ran it on your Mac: your Instagram session stayed open. Nothing for you to change.
- "Your Pictures folder" is found from your user account's home folder. Say so if you want the app's session somewhere else by default.
- If the chosen folder holds more than one Capture One session file, the app uses the one named after the folder, then the first by name. Say "always ask which file" to change it.
- If the Style field is empty, the app leaves Capture One's next-capture style as it is, so it doesn't clear yours. Say "clear the style when the field is empty" to change it.

## Not yet
- Shooting into the app's own session (step 6) hasn't been tried with the camera. Only the setup steps were run against Capture One.
- If Capture One quietly refuses to open a session file, Health correctly says it couldn't open it, but the "PSM Open Window" throwaway stays open in Capture One. You can close it by hand. This is queued for a fix.
- After a session fails to come to the front, the red "Capture One document" row and "camera disconnected" stay until a session opens successfully, even if nothing is wrong any more.
- Right after "Use the app's own session", Settings shows the short form `~/Pictures/Session Manager` instead of the full path until you save.

## Needs you
Nothing.
