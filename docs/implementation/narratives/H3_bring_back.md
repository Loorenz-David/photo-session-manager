# Phase H3_bring_back: bringing a cleared or shop-only product's photos back from Shopify

## What you can do now
There is still nothing to click. The app now has the engine behind "Bring back", and the next step puts the button on screen. For a product whose files were cleared after publishing, or a shop product never photographed on this Mac, the app downloads every published picture from Shopify at full size into that product's capture folder. Only when all of them are on disk do they become the session's photos: main photos first, then condition photos, in the order Shopify shows them, each exactly as published. If any picture can't be downloaded, nothing changes. A publish afterwards sends only what you changed: a re-framed, turned or Capture One-edited photo replaces just its own picture on Shopify, a new order only reorders, a hidden photo's picture is removed, and untouched photos are left alone. JPEG photos now work like RAWs everywhere in the app: preview, frame, Compose, review, Capture One render and export.

## Try it yourself
1. Open Terminal and run this step's tests, one after the other. Each should end with `test result: ok`:

```
cd ~/Desktop/Developer/Photo-Session-Manager-agentic
cargo test -p psm-core --lib mechanisms::remote
cargo test -p psm-testkit --test suite h3_jpeg_photos::
cargo test -p psm-testkit --test suite h3_bring_back::
cargo test -p psm-testkit --test suite h3_publish_after_bring_back::
```

2. Open `docs/intention/evidence/capture_one_16.8.6.md` and read its last section, "VERIFY-JPEG". It records the check run on your Mac: Capture One lists, finds and renders a JPEG in a capture folder the same way it handles a RAW.
3. In Capture One's "Beyo Studio" session, the `_Unassigned` folder holds `psm-probe-verify-jpeg-2026-11-23.jpg`, a plain test picture used for that check, plus its two exports in `Output/_session_manager/`. The app never deletes it; you can remove it yourself if you like.

## Choices made for you
- **Nothing to see yet.** The next step adds the screen and the Bring back button. Say "show it earlier" only if you want a temporary button.
- **All or nothing.** If one picture can't be downloaded after three tries, nothing changes and the files that attempt downloaded are removed. Say "keep the pictures that did download" to change it.
- **The files are named after the product's folder** (`BEYO-12-main-01.jpg`, `BEYO-12-condition-01.jpg`). Bringing the same product back a second time adds `-b2`, so an earlier photo's file is never overwritten. Say so if you want another naming pattern.
- **Bringing back is not an edit.** A published product stays "Published", and it won't be cleared again before its next publish. Say "count a bring back as a change" to change it.
- **The product's earlier photo entries are kept but hidden** (their RAWs are gone), and that session's undo list starts empty after a bring back.
- **A brought-back photo starts exactly as published:** no crop, no turn, and no house style applied again. Say "apply the house style to brought-back photos" if you want it.
- **A picture Shopify sends back that isn't a JPEG counts as not downloaded.** The bring back fails and says so, rather than bringing in a photo the app can't show or frame. Say "convert PNGs" if your products hold PNG originals.
- **In a product with a shooting template, the brought-back photos keep the published order and count as placed by hand.** A photo shot afterwards fills a free spot or goes to Condition. It never replaces the published cover.
- **A failed bring back always says why on screen,** including "wait until the publish has finished" and "the capture folder is not known yet". Pressing it with no shop connected, for a product that isn't remote, or while one is already running gives a plain refusal with no progress line.
- **If Shopify has moved a picture,** the app asks once for its new address and tries three more times there. If Shopify no longer has it at all, it is put on Health as "no longer in the shop" and the bring back stops.
- **A file left over from an interrupted bring back is replaced** by the next attempt. No photo ever points to it.
- **A large JPEG's preview is scaled to 1600 px** with a high-quality filter. A smaller one is used as it is, never enlarged. Its EXIF rotation is respected.

## Not yet
- The Bring back button, its prompt ("About X MB"), the progress on screen, and opening the session afterwards come in the next step.
- Getting the original RAW back is not part of this.
- A real bring back from your Shopify store hasn't been tried yet. The orchestrator does that after the next step: one archived test product, then a publish with one re-framed photo. Shopify should show one replaced picture and the others untouched.

## Needs you
Nothing.
