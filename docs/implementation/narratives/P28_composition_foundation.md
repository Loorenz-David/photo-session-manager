# Phase P28_composition_foundation: every photo now remembers its own framing, and the app notices when Capture One changes the picture underneath it

## What you can do now
Nothing changes on screen yet. Underneath, every photo now has a saved framing: its shape (Square by default, or whatever "Default shape" says), turns, flip, straighten angle and crop box. New photos get the default framing the moment they arrive, without the app asking Capture One for anything. As soon as the first full-frame preview comes back, the crop box is placed as the largest centred box of that shape. Each preview now also gets a small 640-pixel copy for the grid tiles, kept beside it. If you straighten, flip, apply keystone or rotate a photo in Capture One, the app flags that photo's framing "needs review" instead of quietly cropping something different. The framing editor that lets you confirm it comes in a later phase. The first time you open this version, it asks Capture One to redo every existing preview once at full frame, and no session becomes "needs publishing" because of it.

## Try it yourself
1. Run this phase's checks (no camera, Capture One or Shopify needed):
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager
   cargo test -p psm-testkit --test p28_composition_foundation
   cargo test -p psm-core composition
   ```
   All should pass.
2. With Capture One open, start the app and open a session you already have:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager/apps/desktop
   npm run tauri -- dev
   ```
   The tiles still show the whole photo. Within about a minute Capture One redoes each preview once. Nothing turns "needs publishing".
3. Shoot or import one frame and wait for its preview. Then look in the preview folder: next to each `adjusted-<n>.jpg` there is a `thumb-<n>.jpg` whose long side is 640 pixels.
   ```
   find ~/Library/Application\ Support/com.loorenz.sessionmanager/previews -name 'thumb-*.jpg' | head
   sips -g pixelWidth -g pixelHeight "$(find ~/Library/Application\ Support/com.loorenz.sessionmanager/previews -name 'thumb-*.jpg' | head -1)"
   ```
4. In Capture One, straighten that photo by about 3° and wait for the preview to refresh. Then ask the database:
   ```
   sqlite3 ~/Library/Application\ Support/com.loorenz.sessionmanager/session_manager.db "select photo_id, shape, state, review_reason from compositions order by photo_id desc limit 5"
   ```
   The photo reads `needs_review` with reason `geometry`. Undoing the straighten does not clear it; a person confirms in the editor later. Rotating a photo 90° in Capture One flags it with reason `frame`.
5. Go to Settings, choose Default shape `4:5`, press Save, and shoot a frame. Run the query from step 4 again: the new photo reads shape `4:5`. Photos you already had keep their framing.

## Choices made for you
- The 640-pixel tile copies are saved as JPEG quality 80. If tiles look soft or the folder grows too large, say "thumbnail quality <number>".
- When a refreshed preview is merely a different size but otherwise unchanged, the app keeps the frame size it first recorded. Preview sizes are never compared, so nothing changes for you. Say "always record the latest preview size" if you'd prefer that.
- Changing framing on a deleted photo is saved, but does not mark its session "needs publishing", because a deleted photo never goes to the shop. Say "count deleted photos too" to change that.

## Not yet
- No framing editor, no amber "Re-frame" corner on tiles, and no blocking of publish or export while a photo needs review. Those come in the next phases.
- Tiles still show the full preview; they start drawing the saved framing when the editor arrives.
- The "needs review" flag has been checked with a simulated Capture One. On your Mac it was checked only by reading one photo's straighten and keystone values (read-only). Step 4 is the first real end-to-end proof.
- Two edge cases are written down for the next phase. Editing "default_shape" by hand in the settings file to something the Settings screen doesn't offer (for example `Square` instead of `1:1`) would stop sessions from opening, so please change it only through Settings. And a RAW that is imported into a second session after its preview already exists gets no tile copy or framing until Capture One next re-renders it.

## Needs you
Nothing.
