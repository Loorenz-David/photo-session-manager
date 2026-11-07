# Phase P49_capture_one_alongside: Capture One follows your session, new photos get their preview and style, and Import works again

## What you can do now
When you open a session, Capture One's browser switches to that session's folder. When a new photo comes in from the camera's own connection,
or you import a file, Capture One jumps back to the session folder if it was showing another one. It does this every time, even while you
are working in Capture One, as you chose. If Capture One is closed, the app leaves it closed. A new photo's adjusted preview now waits for
Capture One to finish preparing the photo. The app checks every second for 15 seconds. After that it keeps trying at growing gaps (1, 2, 4,
8, 16 seconds, up to a minute) for as long as the photo exists, and it never marks the preview as failed for this reason. When Capture One
switches folders, waiting previews are retried at once. The style named in Settings is applied by the app to each newly arrived photo, once,
before its first preview. It is never applied again after a restart or after you retouch the photo. Import is fixed: it copies the file into
the session's capture folder and leaves your original where it was. Capture One's own import, which is broken in your version, is no
longer used.

## Try it yourself
1. Run this phase's checks. They use a stand-in Capture One, so no camera and no Capture One are needed:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager
   cargo test -p psm-testkit --test p49_capture_one_alongside
   ```
   You should see `21 passed`.
2. Open Capture One with its session document, then start the app:
   ```
   cd apps/desktop
   npm run tauri -- dev
   ```
   Open a session in the app. Capture One's browser shows that session's folder.
3. Drag a RAW file from Finder onto the workspace. Within a few seconds it appears in the app and in Capture One's browser, and its
   adjusted preview follows. The original stays where it was, and the copy sits in the session's folder under `Capture/`.
4. If Settings → Capture One names a style, open the new photo's Styles panel in Capture One. The style is there, once.
5. In Capture One, browse to another folder, then import another file. Capture One jumps back to the session folder at once.

## Choices made for you
- Import now copies the file into the session's capture folder. If a file with the same name is already there, the copy gets a `__dup1`
  suffix (then `__dup2`, and so on), and nothing is ever overwritten. "Use the original in place" is not possible, because Capture One would
  never see the file there.
- To decide whether to jump, the app asks Capture One which folder it is showing right now, instead of trusting the last folder the app set.
  That is what lets it jump back after you browse away.
- The app applies the style itself only to photos from the camera's own connection and to imports. Photos taken through Capture One's own
  tether still get the style from Capture One's next-capture settings, as before.
- What the fixes changed, in plain words: the first version would have styled almost any photo before its next preview, including old
  photos and photos you had already retouched. The first fix limited the style to newly arrived photos. It also broke something: whenever a
  style was set, photos taken through Capture One's tether never got their preview. The second fix repaired that, and those previews now
  arrive normally.
- A photo can receive the style only until its first preview is finished. If you set a style after that, existing photos are left alone.
  To change this, say "apply a newly set style to photos that have not been retouched yet".
- If Capture One refuses to switch folders, the existing notice "Capture One will not show this session's new photos" appears. Opening a
  session or restarting tries again.
- Photos taken through Capture One's tether do not wait for Capture One. Their preview starts straight away, as before.

## Not yet
- Nothing has run against the real Capture One or the camera. Jumping to another folder while you work, and a late photo still getting its
  preview, are checked in the hands-on camera acceptance phase.
- The screens that show this come in a later phase.
- Points for later:
  - After "Take the camera back", previews that went stale wait for their next retry, up to a minute, instead of retrying at once.
  - If applying the style keeps failing for a photo (for example, Capture One does not report the style afterwards), that photo's preview
    keeps retrying and does not arrive until the problem clears.
  - The way the app confirms that Capture One switched folders has not been tried on the real app. If it fails there, every switch would
    show the "will not show new photos" notice.
  - We still don't know why previews stopped retrying in your earlier hands-on test. The stand-in could not reproduce it.
  - Importing a file that is already in the session folder makes a `__dup1` copy beside it. A crash right after the copy leaves the copy
    without a photo in the app.
  - In a window of a few seconds (a style set while a photo's very first preview is still being made), the style could reach that photo.

## Needs you
Nothing.
