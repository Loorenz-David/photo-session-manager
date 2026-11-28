# Phase LV2_current_shot_and_phone_server: the "current shot" decides where every photo goes, and a phone can be paired with the Mac

## What you can do now
In a session whose item type has a shooting template, one container is always the "current shot". The Mac outlines it
as "Next". Every shutter goes to the container that was current at the moment the camera fired, whether you pressed the
camera's own button, the Mac or, later, the phone. That holds even when Wi-Fi files arrive late or out of order. After
each shot, "Next" moves to the next empty container in shooting order, and when none is empty the shots go to
Condition. Retake removes the photo in a container at once (undoable) and makes that container current again. In
Settings → Phone you can pair a phone with a QR code, see which phones are connected, rename them and remove them. The
phone gets a simple page that says which item it is connected to. The full phone app comes next.

## Try it yourself
1. Build the phone page once, then start the app from this branch:
   ```
   cd apps/desktop
   npm run build
   npm run tauri dev
   ```
2. Open a session whose item type has a shooting template. The container outlined "Next" is the first one in shooting
   order.
3. Shoot with the camera's own button. The photo lands in that container, and "Next" moves to the next empty one.
4. Shoot two more quickly over Wi-Fi. Each lands in the container that was "Next" when you pressed, even if the files
   arrive out of order.
5. Open Settings → Phone. It says "Phone server on port 8686". Click **Pair a phone**, then **Create**, and a QR code
   appears. Scan it with the phone's camera (same Wi-Fi). The page on the phone says "Connected to" and the item's
   name. If the phone cannot open the address, click **Show the address instead** and scan that code.
6. Still in Settings → Phone, click **Remove** next to that phone and confirm. Reload the page on the phone: it now says
   the phone was removed.

## Choices made for you
- After a shot that went to Condition, the current shot goes back to the first empty template container. To change
  this, say "stay on Condition until I pick a shot".
- A photo that arrives with no shutter report takes the shot that is current when it arrives. This covers the Capture
  One tether, or a file more than 5 minutes after its shutter. To change this, say "send unreported files to
  Condition".
- "Shift the rest back" stays on the Mac. Only the automatic refill of an emptied container is gone: Retake, or tapping
  a shot on the phone, replaces it. To change this, say "bring back the automatic refill".
- If every container is full and you Deny one on the Mac, the next shot still goes to Condition. The shot after that
  fills the emptied container. Use Retake to refill it straight away. To change this, say "after a Deny, make the
  emptied container current".
- Re-opening the session you are already in keeps the shot you chose. Switching to another session and back, or
  restarting the app, starts again from the first empty container. To change this, say "reset the current shot every
  time I open a session".
- A photo already sitting in its container stays there, even if you also placed another photo there by hand. To
  change this, say "a hand-placed photo pushes the template photo out".
- The phone server is on by default on port 8686. It answers only phones you paired, and only from this Mac or your
  local network. You can change the port or switch it off in Settings → Phone with **Save phone server**. To change
  the default, say "phone server off by default".
- Every phone action names the session it was meant for. If you switched session on the Mac in between, the Mac
  refuses the action and nothing happens. The phone app of the next phase will then say "Session changed on the Mac".
  To change this, say "let the phone act on whatever session is open".
- A pairing's secret is shown only once, in the QR code. A lost QR code means pairing again. New phones are named
  "Phone 1", "Phone 2" and so on until you rename them. To change this, say "let me show a phone's QR code again".
- On a phone, Move swaps two photos and never replaces one, and Delete is the same as Deny on the Mac (undoable). To
  change this, say "Move on the phone should replace like the Mac's drop".
- The phone shows the developed preview of a photo, else the camera's own preview, else the thumbnail. To change this,
  say "show only developed previews on the phone".

## Not yet
- The real phone app (live picture, capture button, shot dock, Move and Delete) comes in the next phase. Today the
  phone shows only a "Connected to …" page.
- When a photo is deleted from the phone, the Mac's screen will not refresh by itself yet. The same goes for other
  connected phones. This is noted for the next phase, before the phone gets its Delete button.
- Deleting the open session on the Mac does not yet tell a connected phone. That is noted for the next phase too.
- Nothing has been tried with a real phone and camera together yet. That is the hardware check at the end.

## Needs you
Nothing.
