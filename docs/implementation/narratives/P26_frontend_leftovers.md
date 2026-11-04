# Phase P26_frontend_leftovers: the app's own session path shown in full, and two quick drags that never jump back

## What you can do now
In Settings, when you press "Use the app's own session", the line under it shows the full folder the app will use, such as `/Users/<you>/Pictures/Session Manager`, right away and before you save. It no longer shows the short `~/Pictures/Session Manager`. In a session you can drag one photo and then drag another before the first has finished saving. Both photos stay where you dropped them, and the screen refreshes once, after the last drag is saved.

## Try it yourself
1. Run this phase's checks (no camera, Capture One or Shopify needed):
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager/apps/desktop
   npx vitest run src/features/settings/__tests__/p26_frontend_leftovers.test.tsx src/features/workspace/__tests__/p26_double_drag.test.tsx
   ```
   Both should pass.
2. Start the app:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager/apps/desktop
   npm run tauri -- dev
   ```
3. Go to Settings. Press "Use another session…" and pick `~/Pictures/session test 1/session test 1.cosessiondb`. Then press "Use the app's own session" **without saving**. The line reads `Effective path: /Users/<you>/Pictures/Session Manager`, the same as after Save. Leave without saving if you want to keep your current choice.
4. Open a session with four or more Main photos. Drag photo 01 to the last slot. Straight away, before it settles, drag the new first photo to the second slot. Both should land where you dropped them and neither should jump back. A second later the order is still the one you made.

## Choices made for you
- After a burst of quick drags, the app refreshes the photo list once, when the last drag has been saved, even if nothing else changed in the meantime. If you'd rather it refresh only when Capture One reports a change, say "refresh only on events".

## Not yet
- The quick double drag was checked in the test suite only. Step 4 is the first time anyone tries it on screen.
- A very narrow case is left: if a new capture or preview arrives at the exact moment you drop a photo, the order can still flicker back for a moment before it corrects itself.

## Needs you
Nothing.
