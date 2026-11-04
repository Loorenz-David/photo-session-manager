# Phase P27_retire_shape_on_arrival: the app stops cropping in Capture One, and gets a "Default shape" setting and a full-size master recipe

## What you can do now
New photos, whether shot or imported, are no longer cropped in Capture One. The app only asks Capture One for its usual preview, and the crop tool on your variants stays as you left it. Settings has a "Default shape" choice (Original, Square (1:1), 4:5, 3:2, 2:3, 16:9, Free) where "Shape on arrival" used to be, plus a "Master recipe" field. Your old setting carries over: "Square" becomes "Square (1:1)". When the app starts, it updates its preview recipe in Capture One to 2560 px, quality 85, with crops ignored. It also creates a new disabled "PSM Master" recipe (16-bit uncompressed TIFF, full size, crops ignored), ready for the full-size renders that come later. Export still works exactly as before, through "PSM Web".

## Try it yourself
1. Run this phase's checks (no camera, Capture One or Shopify needed):
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager
   cargo test -p psm-testkit --test p27_retire_shape_on_arrival
   cd apps/desktop
   npx vitest run src/features/settings/__tests__/p27_default_shape.test.tsx
   ```
   All should pass.
2. With Capture One open, start the app:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager/apps/desktop
   npm run tauri -- dev
   ```
3. Go to Settings. "Default shape" reads "Square (1:1)" and "Master recipe" reads "PSM Master". There is no "Shape on arrival" any more. Press Save once. Your settings file no longer mentions `crop_on_arrival`:
   ```
   grep -E 'default_shape|crop_on_arrival' ~/Library/Application\ Support/com.loorenz.sessionmanager/settings.json
   ```
   Only the `default_shape` line should print.
4. In Capture One, open the process recipes. "PSM Preview" now reads 2560 px, quality 85, crop ignored. A new "PSM Master" (TIFF 16-bit, uncompressed, sRGB, full size, disabled) is there, and "PSM Web" is unchanged. Health shows "Recipes: Ready".
5. Shoot one frame. Its preview appears after the usual delay at the full frame, with no square crop. In Capture One the variant's crop tool is untouched. If you set a crop by hand, the app's next preview still shows the full frame.
6. Export a session: the files still come out through "PSM Web" as before.

## Choices made for you
- An old setting of "Leave as shot" becomes "Original" (the whole frame). If you'd rather it become Square, say "map off to square".
- Old crop names that have no matching shape (Unconstrained, 5x7, 3x4 and the like) become "Square (1:1)". If you want a different fallback, name it.
- The app sets up the master recipe before the web recipe. Nothing visible depends on the order.

## Not yet
- The square framing now comes back through the app's own framing in the next phases. Until then, new previews show the full frame.
- Previews made before this phase (1600 px, Capture One's crop applied) stay as they are until that photo is next adjusted. A later phase refreshes them.
- The new recipe settings were checked live in Capture One on temporary recipes, which were then deleted. Your own "PSM Preview" and "PSM Master" are updated the first time you start the app, in step 2.
- "PSM Web" and export through Capture One stay until full-size renders replace them in a later phase.

## Needs you
Nothing.
