# Phase P56_camera_claim_and_render_loop: Session Manager grabs the cable the moment you plug it in, never sits disconnected, and no longer re-exports the same photos from Capture One in an endless loop

## What you can do now
When you plug the cable in, the camera helper now leaves Wi-Fi straight away and tries the cable at once, then again every fifth of a second for two and a half seconds. Before, it waited two and a half seconds first, and in that gap the Mac's own camera service took the cable. If another program already holds the cable, the helper goes back to Wi-Fi at once. It then watches the cable twice a second and takes it within about a second of that program letting go. It does not drop your working Wi-Fi just to "check" the cable. If neither link works, the helper keeps trying both for as long as it takes, so it never gives up and sits idle. The export loop is gone too. The loop had two causes. First, Capture One saved its exports with a counter added to the name ("DSC04979 14.tif") but told the app the old name, so the app never saw an export finish and ordered it again. Second, Capture One rewrote its own settings files after each export, and the app mistook that for an edit. The app now recognises the numbered file and only counts a real change to a photo's adjustments as an edit. A photo whose preview never arrives is tried twice and then reported on the Health page. Full-size renders for export pause when the disk holding your session has less than 5 GB free.

## Try it yourself
1. Run this phase's checks. They use stand-ins, so no camera or Capture One is needed:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager
   cargo test -p psm-testkit --test p56_camera_claim_and_render_loop
   tools/sidecar/build.sh --stub --test
   ```
   You should see `4 passed` and `100% tests passed`.
2. Build the app with the real camera helper and install it as usual:
   ```
   tools/sidecar/build.sh --sdk /Users/davidloorenz/Desktop/Developer/Camera-Connection-Tests/vendor/sony-sdk
   ```
3. With Capture One running and the camera on Wi-Fi (the dot says `Wi-Fi`), plug the cable in. Within about a second the dot should say `Cable`. To see who holds the cable:
   ```
   ioreg -l -r -n "ILCE-7RM5" | grep UsbExclusiveOwner
   ```
   It should name `camera-sidecar`.
4. If another program got the cable first, the dot should go back to `Wi-Fi` within about a second and stay there. When that program lets go, it should switch to `Cable` within about a second.
5. Leave the app open for an hour with the cable in. The dot should always read `Cable` or `Wi-Fi`, never disconnected while the camera is on.
6. Work normally and switch between Capture One and Session Manager often. Capture One's export window should appear only after you actually edit a photo. The `Output/_session_manager` folder should not fill up with `name 1.jpg`, `name 2.jpg`, … copies.

## Choices made for you
- During the 2.5 seconds after a plug-in, the cable is retried every 0.2 seconds. Say "retry slower" if that is too eager.
- If the cable is still not usable after those 2.5 seconds and nobody else holds it, the helper stays on Wi-Fi until you re-plug or press Retry. It never drops Wi-Fi on its own to try again. Say "keep retrying the cable" if your camera turns out to need longer than 2.5 seconds after a plug-in.
- The helper itself watches who holds the cable, twice a second, and only for as long as it is not on the cable.
- When Wi-Fi is unreachable and the cable is held by someone else, the helper takes turns trying both, waiting longer between tries up to 30 seconds.
- An export counts as finished when the file Capture One reported was written after the request, or otherwise when Capture One's newest numbered copy of it was.
- A preview that never arrives is ordered twice (the first request and one retry), then reported on the Health page as "Previews — Capture One produced no preview for …". Say "try three times" to raise it.
- The app counts a Capture One settings rewrite as an edit only when the adjustments in it changed. For photos previewed before this update, the app takes their current settings as the starting point. So an edit you made while the app was closed, before installing this update, will not re-render once.
- The disk guard checks the disk your Capture One session is on, needs 5 GB free, and pauses only full-size renders (previews are small). If the free space cannot be read, nothing is paused. Say a different limit if 5 GB is wrong for you.
- The Health page has two new lines: "Previews" and "Disk space".

## Not yet
- Nobody has tried this with the real camera and Capture One yet. Plug-in speed, holder release and the hour-long run are part of the next hands-on camera run.
- On 8 November your Capture One preview recipe had changed to TIFF, saving to `Output`, with numbered names. The app only repairs the recipe when it starts. Restarting Session Manager should put it back to JPEG and overwrite.
- The `session test 1NNNN` files seen on 8 November are still unexplained. If Capture One names an export that way again, that photo will stop after two tries and show up on Health. The app will not recognise the file.
- If the Mac's camera service lets go of the cable and grabs it again over and over, each time costs a short Wi-Fi gap.
- A photo whose preview failed is tried again (twice) every time the app starts.
- The disk-space warning stays on the Health page after you free space, until you next export.
- Edits to local adjustment layers made while the app missed Capture One's notice may not trigger a new preview. This will be checked with a layered photo.

## Needs you
Nothing.
