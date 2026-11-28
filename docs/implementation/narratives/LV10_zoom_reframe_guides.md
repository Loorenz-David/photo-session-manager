# Phase LV10_zoom_reframe_guides: zoom in, reframe on the phone, and guides over live

## What you can do now
In the phone's full-screen view, spread two fingers to zoom into a photo (up to 6×) and check its detail and sharpness. One finger moves the zoomed photo, and a double tap goes back. While zoomed, swiping does nothing, so you can't sort a photo by accident.

Once the Mac has Capture One's full-frame preview of a photo, the phone shows that photo as it is framed: turned, flipped, straightened and cut to its frame. This applies in the full-screen view and in the strips.

⋯ → Reframe opens the same framing as the Mac's editor, made for fingers. One finger moves the photo under the frame. Two fingers zoom it and straighten it, and a fine grid shows while you straighten. The frame never shows empty corners. You also get Normal or Split, the shapes, Turn, Rotate 90°, Flip, the guides inside the frame, and Reset. Done saves into the Mac's own framing: the Mac shows it at once, its exports follow it, and ⌘Z on the Mac undoes it. Cancel changes nothing.

The live screen's menu now has a Guides row (Thirds, Grid, Golden ratio, Diagonals). The lines show over live and over the template, and the phone remembers your choice.

## Try it yourself
1. Start Session Manager from this branch with the camera connected and the phone paired:
   ```
   cd apps/desktop
   npm run build
   npm run tauri dev
   ```
2. Open a session with a shooting template and take a few photos. Wait until each tile shows the framed picture: that means Capture One's preview has arrived.
3. Open a photo full screen. Spread two fingers: it zooms. Move it with one finger: it stops at its edges. Double-tap: back to the whole photo. Double-tap again: 3× where you tapped. Try a swipe while zoomed: nothing happens.
4. Tap ⋯ → Reframe. Move the photo with one finger, spread two fingers to zoom, and twist them to straighten. Check that the fine grid shows only while you turn, and that the frame never shows an empty corner. Tap Guides → Golden ratio. Try Split, Shape → 4:5, Turn, and the round-arrow button → Rotate 90° right.
5. Tap Cancel: nothing changed. Open Reframe again, change something and tap Done. You should see "Framing saved", and the full-screen view and the strip show the new framing. On the Mac the photo shows it too, and ⌘Z there undoes it.
6. Open Reframe on the phone. Before tapping Done, change the same photo's framing on the Mac, then tap Done on the phone. The phone should say "Framing changed on the Mac. Open Reframe again.", and the Mac's framing is untouched.
7. On the live screen, open the menu, then Guides → Thirds. The lines show over the template and over live (hold for live). Close the app and open it again: Thirds is still on.

Things only your eyes and hands can judge (nothing here was seen on a real phone or in a browser yet):
- whether a framed photo looks right in the strip and full screen, especially one turned 90° or straightened
- whether the photo follows your fingers smoothly in Reframe, and the zoom feels natural
- whether the grid and the guide lines can be seen on both light and dark photos

## Choices made for you
- **Zoom limits:** 1× to 6×. While your fingers are down it stretches a little past both ends and settles back when you let go. A double tap goes to 3×. Say "a different maximum" to change it.
- **A second finger during a swipe or a hold** cancels it, with nothing sent, and starts a zoom.
- **Another photo always opens at 1×.**
- **Reframe waits for Capture One's preview.** Until the Mac has it, Reframe is greyed out and says "Waiting for Capture One's full-frame preview", because the camera's own preview is not the picture the framing is measured on.
- **Straightening by twisting starts after 3°**, so a zoom doesn't tilt the photo by accident. After that it follows your fingers in 0.1° steps.
- **The photo stops at the edge instead of springing back.** Right after straightening, the frame already touches the photo's edge, so the photo barely moves until you zoom in a little. Zooming out at an edge stops there, where the Mac's editor slides the frame inward instead. A diagonal move that reaches one edge stops both ways. Say "let it slide along the edge" if this feels stuck.
- **No trimming handles on the phone.** A Free shape keeps its proportions; trim it on the Mac.
- **The editor stays upright** even for a long shot.
- **Done with nothing changed sends nothing.** Done right after Reset asks the Mac for its own default framing.
- **A photo that Capture One changed ("needs review") opens already re-fitted, as on the Mac.** But Done without a change does not confirm it the way Return does on the Mac, so the photo still says "needs review" there. Say "Done should confirm a photo that needs review" to change it. That is a small change.
- **Sending back the framing a photo already has changes nothing on the Mac,** not even for a photo nobody has framed by hand yet.
- **If two saves of the same photo land in the same instant**, both may pass, and the second one wins. The plan accepted this.
- **"Framing changed on the Mac" also refreshes the phone,** so your next Reframe opens on the Mac's framing. Cancel does nothing while Done is waiting for the Mac.
- **The editor's look:** its menus are small sheets that close after a pick. Outside the frame is dimmed. Controls give a light buzz on Android, and "Framing saved" buzzes too.
- **The two guide choices are separate.** The live screen's guides start Off, and the guides inside Reframe start on Thirds, as on the Mac. The Guides row sits above the menu's actions, and the guides also show in the Free shot.
- **A framed photo of a split template shot** shows its own middle line, not the dashed template line over it.
- **Missing values:** if the Mac ever sends no framing or no default shape, the phone treats it as "none" and "1:1". The Mac always sends both.

## Not yet
- Confirming a "needs review" photo from the phone (see above).
- Trimming a Free frame with handles on the phone.
- Copying or pasting a framing between photos on the phone.

## Needs you
Nothing.
