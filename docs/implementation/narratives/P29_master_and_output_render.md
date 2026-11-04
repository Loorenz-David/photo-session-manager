# Phase P29_master_and_output_render: every exported photo is now the app's own crop of a full-quality master from Capture One

## What you can do now
When you press Export, Capture One no longer makes the final JPEGs. It makes one full-size, 16-bit, uncropped master TIFF per photo, and only when the app doesn't already have one for that photo's current edit. The app then makes each JPEG itself from that master: it turns, flips, straightens and crops the photo by its saved framing (square by default), shrinks it to 2048 pixels on the long side (never enlarges it), and saves it once at quality 90 with an sRGB colour profile. Camera, capture date and copyright are copied in; GPS is not. Finished JPEGs are kept, so exporting again without changes takes a moment and asks Capture One for nothing. Export now refuses, and names the photos ("Re-frame first: Main 02"), while any main or condition photo is flagged for re-framing. Settings has "Output size" and "Output JPEG quality" in place of "Web recipe".

## Try it yourself
1. Run this phase's checks (no camera, Capture One or Shopify needed):
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager
   cargo test -p psm-testkit --test p29_master_and_output_render
   cd apps/desktop && npx vitest run src/features/settings/__tests__/p29_output_format.test.tsx
   ```
   All should pass.
2. Start the app with Capture One open, open a session with a few main photos whose previews are done, and press **Export**:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager/apps/desktop
   npm run tauri -- dev
   ```
   Watch Capture One's `Output/_session_manager/` folder: one large TIFF per photo appears, then disappears once the app has taken it. Then check the export folder:
   ```
   ls ~/Pictures/Session\ Manager\ Export/*/
   sips -g pixelWidth -g pixelHeight -g profile "$(ls ~/Pictures/Session\ Manager\ Export/*/*_01_main.jpg | head -1)"
   ```
   You should see square JPEGs at 2048 pixels with the profile `sRGB IEC61966-2.1`, plus `item.json` and `manifest.json`. Open one in Preview and use Tools → Show Inspector: your camera model and capture date are there, and there is no GPS.
3. Press Export again straight away. It finishes almost at once and Capture One renders nothing.
4. Go to Settings, set **Output size (long edge, px)** to 1600, press Save, and export. The files are now 1600 pixels. Capture One renders the TIFFs once more, because masters are thrown away after a finished export. Export again and nothing is rendered.
5. In Capture One, `PSM Web` is still in your recipes. The app leaves it alone and no longer repairs it.
6. Rotate one photo 90° in Capture One and wait for its preview to refresh. Export now refuses with "Re-frame first: Main 0n" and renders nothing. Undoing the rotation does not clear this; the framing editor that confirms it comes later.

## Choices made for you
- The straighten uses a standard sharp (bicubic) resampling written in the app, not an extra library. Say "use a different straighten filter" if straightened photos look soft.
- The sRGB colour profile is built into the app instead of being copied from macOS. Say "use the macOS sRGB profile file" if any program misreads it.
- Once the app has copied Capture One's master TIFF, it deletes that TIFF from Capture One's output folder, so a 60-megapixel shoot doesn't leave 360 MB files behind. The RAW is never touched. Say "keep Capture One's master files" to stop this.
- If two exports need the same photo at the same time, Capture One renders its master once and both use it.
- For a split photo, the saved left and right JPEGs are kept side by side. Only versions from an older edit, framing or output size are cleaned away.
- If Capture One delivers a master that no export is waiting for any more, the app ignores it.
- A finished export keeps a master that another export still in progress needs.
- In refusal messages, photos are numbered the way export file names are ("Main 02", "Condition 01").
- If a camera only records a general date rather than a capture date, that date is written as the capture date.

## Not yet
- Publishing to Shopify already uses these new JPEGs, because its first step is the same export. Split photos in the gallery and damage report, the new upload names, and deleting masters after a publish come in the next phase.
- There is no framing editor yet. Every photo is exported with its default framing (square unless Settings says otherwise).
- No real 16-bit master has been rendered through Capture One yet. The pipeline was checked with simulated masters, and the cropping and metadata copy were also run once on a real Capture One TIFF from your ZV-E10. Step 2 is the first full real run. If your camera and date are missing from the JPEGs there, tell us, and the metadata copy will be adjusted (the pictures themselves are unaffected).
- Two rare edge cases are written down for the next phase. First, if Capture One's geometry changes on a photo in the few seconds while an export is cropping it, that one export can still use the old framing. Second, an unreadable master could leave Export unresponsive until you restart the app.

## Needs you
Nothing.
