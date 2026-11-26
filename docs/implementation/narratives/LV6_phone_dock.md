# Phase LV6_phone_dock: the phone's shot strip without hold gestures, pictures that never show as broken, and a shorter lock swipe

## What you can do now
The strip of shots at the bottom of the phone has bigger tiles that you slide sideways. One tap opens a taken photo's
preview, or makes an empty shot the current one. A long press is still just a tap, nothing grows, and sliding a tile
upwards does nothing. A photo shows as a plain grey tile with its number until the Mac has made its preview. Then the
picture appears by itself. It never shows as a broken image. On the main screen, sliding up to lock live now takes half
the distance it did before. A phone you remove in Settings → Phone now says to delete the app from the home screen and
add it again from the new QR code.

## Try it yourself
1. Start Session Manager from this branch with the camera connected and both phones paired:
   ```
   cd apps/desktop
   npm run build
   npm run tauri dev
   ```
2. Open a session that has a shooting template. The phone's bottom strip shows bigger tiles. Slide the strip sideways:
   it scrolls.
3. Tap an empty shot. It gets the outline, and the Mac's "Next" follows.
4. Take a picture. Its tile is a grey square with its number, and the photo appears a few seconds later. Tap it: its
   preview opens. If the preview isn't ready yet, it says "Preparing preview…" and then shows the photo.
5. Press and hold a tile. Nothing grows, and when you let go its preview opens. Slide a tile upwards: nothing happens.
6. On the main screen, hold the picture and slide up. Live locks after a short swipe, about half of the old one.
7. In Settings → Phone, Remove the Android phone. Its screen reads: "This phone was removed in Session Manager. Delete
   this app from the home screen, then scan the new QR code in Session Manager → Settings → Phone and add it again."
   Do that and add it again. It works.

## Choices made for you
- **Tile size.** Tiles are 56 px (they were 36), 8 px apart, with rounder corners. The strip is 84 px tall, so the
  picture above it is a little shorter. To change this, say the size you want.
- **What counts as a tap.** A touch is a tap if the finger moves less than 8 px in any direction, however long it
  stays down. A sideways move past that scrolls the strip, and an up or down move selects nothing. To change this, say
  how much movement should still count as a tap.
- **Which tile a tap picks.** A tap picks the tile whose centre is nearest the finger, if it is no more than half a
  tile-step away. A tap past the last tile, or on the divider before the Condition images, picks nothing. To change this, say so.
- **Small details.** The number tags in the strip are slightly larger to stay readable, and so are the "captured"
  check and the divider before the Condition images. The empty tile colour is the design's light grey. To change any
  of these, name the detail.
- **Waiting for a picture.** A dock or move tile that is waiting shows plain grey with its number and the "captured"
  check. The full preview shows "Preparing preview…" on grey. Holding the preview to see the template shows nothing
  until the photo is there. To change this, say "show the template while the preview is being prepared".
- **Retries.** A picture that fails to load is tried again three times, two seconds apart. After that, the tile stays
  grey until the Mac updates the photo. Opening its preview tries again. To change this, say how often it should keep
  trying.
- **Lock swipe.** Only the lock distance changed, to 55 px. The lock indicator appears at the same point as before, so
  it now fills twice as fast. To change this, say the distance you want.

## Not yet
- Nothing here was tried on a real phone. How the bigger tiles, the scrolling and the shorter lock swipe feel in your
  hand is for the rest of the hardware check, which resumes next.
- The camera's own focus frame on the phone, and HTTPS for the phone, are not built.

## Needs you
Nothing.
