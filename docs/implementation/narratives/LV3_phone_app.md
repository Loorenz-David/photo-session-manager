# Phase LV3_phone_app: the phone becomes your shooting remote

## What you can do now
Scan the QR code from Settings → Phone and the phone opens on whatever session is open on the Mac. It shows the
current shot's template in black and white. Hold the picture to see live; let go to see the template again. Slide up
to lock live on, double-tap to unlock. Slide sideways to set how much of the template shows through. While live is
locked, tap the picture to focus there: "AF LOCKED" appears when the camera confirms, and × sends the focus back to
the centre. The round button takes the picture with a flash, and a message such as "Shot 04 captured" appears once
the photo reaches the Mac. The strip at the bottom shows your shots in shooting order. From it you can open a photo
to Retake, Delete or Move it, or pick the next shot. The menu button shows the item's details.

## Try it yourself
1. Build the phone app once, then start Session Manager from this branch:
   ```
   cd apps/desktop
   npm run build
   npm run tauri dev
   ```
   Connect the camera and open a session whose item type has a shooting template.
2. On the Mac, go to Settings → Phone → **Pair a phone**, then scan the QR code with the phone (same Wi-Fi). The phone
   shows the template of the current shot. On iPhone: Share → Add to Home Screen, then open it from the home screen.
   On Android: tap **Full screen**.
3. Press and hold the picture: live appears over the template. Let go: the template comes back. Hold and slide
   sideways: a bar shows "TEMPLATE nn%".
4. Hold and slide up until the padlock closes: "LIVE · LOCKED". Tap a detail of the item: a square marks the spot,
   then "AF LOCKED". Tap ×: the marker goes away and the camera's focus point returns to the centre. Double-tap the
   picture: unlocked.
5. Press the round button: a flash, then "Shot 01 captured" a moment later, and the outline in the bottom strip moves
   to the next shot.
6. In the bottom strip, hold a filled tile, drag it up until it says "Release to open", and let go. Tap **Retake**:
   "Shot 01 reopened". On the Mac the old photo turns red and that shot is current again.
7. Open a photo, tap **Move image** and pick a position that already has a photo: the two swap, and the preview stays
   on your photo in its new place.
8. Tap the menu button (three dots): the item's name, SKU, current shot and image counts. **Fill item properties**
   opens an empty page; the arrow takes you back.
9. Open a condition image and tap **Delete**: "Image deleted". The Mac updates by itself, and Undo on the Mac brings
   the photo back.

## Choices made for you
- After the last template shot, the Free shot tile shows live all the time, because it has no template to show. To
  change this, say "in Free shot, show live only while I hold".
- The Free shot's picture is square. A shot with no fixed shape takes the camera's own shape (3:2 before the first
  live picture). To change this, say "make the Free shot follow the camera's shape".
- Whenever the phone removes its focus marker (×, picking another shot, Retake), the camera's focus point also goes
  back to the centre. When the Mac changes the shot, the marker stays. To change this, say "only × should recentre the
  focus".
- The "Shot 04 captured" message waits until the photo has reached the Mac, a few seconds after the flash over Wi-Fi.
  In a session without a template it reads "Extra image NN added". To change this, say "show the message as soon as
  the camera fires".
- The message "All template shots captured" is not shown, because the Mac moves on before the last photo arrives. The
  strip and the label show Free shot instead. To change this, say "show a message when the last template shot is
  done".
- The camera's own focus frames are not drawn; only your tap marker and "AF LOCKED" are. To change this, say "draw the
  camera's focus frames on live".
- Two messages were added: "Open a session in Session Manager to start shooting." when no session is open, and "That
  did not work. Try again." when an action fails for another reason. To change the wording, say which text you want.
- While the Mac does not answer, "Connecting to session" stays and the phone asks again every 2 seconds. To change
  this, say how often it should retry.
- The **Full screen** button sits in its own row above the picture, so it never covers a gesture. The bottom strip
  leaves extra room above it so a tile dragged up is never cut off. To change this, say where the button should go.
- The empty properties page is titled "Item properties". Errors from the Mac reach the phone only as "Something went
  wrong in Session Manager." or "Not found.", never with file paths. To change the wording, say which text you want.
- A double tap counts when the two taps are less than 300 ms and 50 px apart, as in the design's prototype. To change
  this, say how quick a double tap should be.
- The tests imitate the Mac at the network level, so the requests and messages they check are the real ones. Nothing
  for you to decide here.

## Not yet
- Not tried on a real phone, a real camera or real haptics, and not on the iPhone home-screen app or Android's Full
  screen. The phone was only seen in a desktop browser against a stand-in Mac. The hardware check of the next phase
  covers this.
- No secure connection (HTTPS), so no "keep the screen awake" and no offline start. The phone follows your Auto-Lock
  setting.
- "Fill item properties" is an empty page for now. Finishing or opening sessions from the phone, and the Mac's own
  live view, come later.
- Removing a phone while it is connected was checked only in two halves by the tests. The hardware check tries it
  end to end.

## Needs you
Nothing.
