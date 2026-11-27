# Phase LV8_phone_sort_and_arrange: sort and arrange a session's photos from the phone

## What you can do now
Open a photo full screen on the phone and sort it with a swipe:
- Swipe left to deny it. A template shot's photo is reopened, exactly as Retake does, and ⌘Z on the Mac brings it
  back.
- Swipe right to make it a Main photo. A photo in a template shot first asks "Move this photo to Main, outside its
  template order?"
- Swipe up to send it to Condition.

After each swipe the next photo shows straight away, so you can work through the whole session like a stack. The full-screen view now has the same photo strip as the live screen. Tap a photo in it to jump there, or tap an empty shot to make it the next one. In either strip, press and hold a photo for a moment until it grows, then drag it:
- Dropped on a template shot, it swaps with that shot's photo, or fills the shot if it is empty.
- Dragged along the extra Main photos or the Condition photos, the others slide aside and it goes where you let go.
- Dropped into the other group, it goes to that group's end.

The phone shows the result at once. If the Mac refuses, the photo goes back and the phone tells you why. The Delete
button is gone, because the left swipe does the same thing.

## Try it yourself
1. Start Session Manager from this branch with the camera connected and the phone paired:
   ```
   cd apps/desktop
   npm run build
   npm run tauri dev
   ```
2. Open a session with a shooting template. Take five or six pictures, some after the last template shot (those
   become Condition photos).
3. Tap a Condition photo in the strip. It opens full screen with the strip under it. Swipe it to the right: it becomes
   a Main photo and the next photo shows. Swipe the next one up: it springs back with "ALREADY CONDITION".
4. Open a template shot's photo and swipe it to the right. The phone asks "Move this photo to Main, outside its
   template order?" Cancel keeps it. Move makes it an extra Main photo, and that shot becomes the next one to take.
   Swipe another template shot's photo to the left. On the Mac it is denied (⌘Z there brings it back), and its shot
   is the next one.
5. Back on the live screen, press and hold a template shot's photo until it grows. Drag it onto another shot and let
   go: the two swap. Press and hold an extra Main photo and drag it along the other extras: they slide aside. Let go.
6. Start a drag, then put a second finger down. Or slide the photo far above the strip and let go. Either way,
   nothing changes.
7. Drag a photo to the right end of the strip and hold it there: the strip scrolls.
8. On the Mac, check each change in the session view.

Things only your hands can judge:
- whether the hold before a photo lifts and the length of a swipe feel right
- whether a lifted photo is ever cut off at the strip's edge
- whether an iPhone shows a pop-up or text selection on the long press
- whether the swipe labels are readable over a bright photo

## Choices made for you
- **How long to hold before a photo lifts:** 0.4 seconds with the finger still. A shorter touch is still a tap.
  Say "make the lift quicker" or "slower" to change it.
- **How far a swipe must go:** about a thumb's width. Shorter, the photo springs back and nothing happens. Say "shorter
  swipes" or "longer swipes".
- **Swipe or hold on a template shot's photo:** a move within the first 0.2 seconds is a swipe. Holding still that long
  shows the template, as before. On photos without a template you can start a swipe at any moment. Say so if you want
  the same 0.2-second rule everywhere.
- **The swipe labels:** "DENY" in red, "MAIN", "CONDITION", or "ALREADY MAIN" / "ALREADY CONDITION". A label appears
  once the finger has moved a little and grows clearer as it goes on. When the swipe would do nothing, the photo
  follows the finger only partly. Say if you want them bolder or quieter.
- **What happens after a swipe:** the next photo shows and the phone buzzes at once. The message ("Image deleted",
  "Moved to main images"…) appears when the Mac has answered. A refused swipe shows only the reason.
- **A second finger** on the full-screen photo cancels the swipe, just as it cancels a drag in the strip.
- **The photo strip in the full-screen view** sits directly under the photo. It scrolls just enough to keep the
  photo you are looking at in sight. Tapping the photo already shown does nothing. Tapping an empty shot that is
  already the next one closes the view without asking the Mac anything.
- **The confirmation** is a sheet from the bottom. It closes by itself if, meanwhile, the Mac has moved that photo out
  of its shot.
- **The focus marker** is cleared whenever a swipe or a drop takes a photo out of its shot, as Retake does.
- **How a lifted photo looks:** it follows your finger sideways freely and only a little up and down. It dims when
  you are too far from the strip to drop. Its old place shows a gap. The shot it would swap with gets a solid ring,
  and a whole group it would join gets dashed rings.
- **Auto-scroll at the strip's ends** works wherever your finger is up or down, as long as a photo is lifted.
- **The confirmation's buzz** comes when you press Move, not when you let go of the drag that opened it.
- **While the Mac answers a drop,** only the strip shows the new order. The photo counts and the next shot stay
  as the Mac last said. Nothing can be lifted until the Mac answers (a long press then is a tap).
- **If the Mac doesn't answer within 8 seconds,** the strip goes back and asks the Mac what is true. That timer runs
  from the drop and does not stop when the Mac answers. On a very slow connection, a drop answered just before
  8 seconds can briefly jump back before settling. Say "stop the timer when the Mac answers" if you see this.
- **When the Mac's photos change during a drag** (a photo added, moved or removed), the drag ends with nothing done.
  A new picture arriving for a photo does not end it.
- **A drawer or the confirmation opening** while you hold a photo ends that drag too.
- **On the Mac,** the current shot is changed only for the session that is open there. A request with an unknown
  group is refused as invalid.
- **A photo still "Preparing preview…"** shows the swipe label but the grey box does not move with your finger. The
  swipe still works. Say "make the grey box follow the swipe" to change it.

## Not yet
- Nobody has tried any of this on a real phone yet. Steps 3–7 above are the first real check of how it feels,
  especially on an iPhone.
- There is no undo button on the phone. A denied photo is brought back on the Mac with ⌘Z.
- The Mac's own review screens are unchanged. There, dropping onto a filled shot still replaces its photo.
- A group with no photos in it has no place to drag into. Use the swipe instead.

## Needs you
Nothing.
