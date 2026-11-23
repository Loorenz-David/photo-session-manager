# Phase H4_remote_workspace: archived and shop-only products open in the normal workspace

## What you can do now
A product whose files were cleared after publishing, or a shop product never photographed on this Mac, now opens in the usual workspace. You see the stage and the Main and Condition groups, filled with its Shopify pictures one after another, and a line that says "Photos are on Shopify. Editing a photo brings them back to this Mac first." Opening it never moves the camera's folder: new photos keep going wherever they went before. You can edit its Details and publish them. Only the details go to Shopify, and its pictures there stay as they are. The first time you try to change a photo (drag, classify, deny, turn, frame, move, copy, assign), the app asks "Bring back N photos from Shopify to edit them? About X MB." After Bring back and its progress line, the product is an ordinary session, and the camera's folder follows it like any other.

## Try it yourself
1. Open Terminal and run this step's tests, one after the other. Each should end with `test result: ok`, or with `passed` for the last one:

```
cd ~/Desktop/Developer/Photo-Session-Manager-agentic
cargo test -p psm-testkit --test suite h4_remote_workspace::
cargo test -p psm-testkit --test suite h4_details_only_publish::
cargo test -p psm-desktop h4_
cd apps/desktop && npx vitest run src/features/workspace/__tests__/h4_remote_workspace.test.tsx src/features/workspace/__tests__/h4_bring_back.test.tsx src/features/shell/__tests__/h4_capture_target.test.tsx
```

2. In the installed app (once you install this build), open a product labelled "Archived in Shopify" from Sessions. The usual workspace opens, the pictures appear one after another, and the line about Shopify shows under the header.
3. Take a photo with the camera. It does not land in this product. It goes where photos went before you opened it.
4. Press Details, change the title, then Publish. The confirmation says "Details only: the photos stay as they are on Shopify." and the button reads "Publish details". On Shopify the title changes and the pictures stay the same.
5. Drag a picture to another place. The app asks "Bring back N photos from Shopify to edit them? About X MB." Press Bring back and watch "Bringing back k of N photos…". Afterwards you can reorder, turn, re-frame and open the photos in Capture One. The next Publish uploads only what you changed.
6. Press N, search for a product photographed on the other Mac, and open it. You get the same workspace with its Shopify pictures (after a brief "Checking Shopify…"), and the same Bring back.
7. In the session's ⋯ menu, "Refresh from Shopify" reads the pictures again after someone changed them in Shopify's admin.
8. Open such a product from a conversation with "Open session". The bar at the top has no "Shoot here instead".

## Choices made for you
- **Opening an archived or shop-only product never moves the camera's folder.** Only bringing its photos back does that. Say "let me shoot into an archived product" if you want otherwise.
- **If the app can't read Shopify the first time you open a shop product,** it says so with Try again, and in the meantime the product behaves like any session.
- **If the app can't even tell where a product's photos are** (a rare read error), the product opens as an ordinary, empty session, and the camera's folder still doesn't move. Say "explain that on screen" if you want a message for it.
- **Publishing a remote product sends its details only.** Shopify's gallery and damage-report pictures are never touched. A shop product you never opened is read from Shopify first, once, so its pictures are never mistaken for removals.
- **Looking at and selecting pictures, the Details form, the writer and conversations need no bring back.** Every photo change asks first. "Bring back now" in the line under the header asks the same question without an edit (that's the way to get the photos into Capture One).
- **"Refresh from Shopify" is in the ⋯ menu of the product you have open,** not on the rows of the sessions list. Say "put it on the rows too" if you want it there.
- **In the sessions list,** remote products count their Shopify pictures and keep "Archived in Shopify". While their publish is running, failed or cancelled, they show that instead.
- **The progress line starts at "Bringing back 0 of N photos…"** until the first picture arrives.
- **"Shoot here instead" is not offered while you visit an archived or shop-only product from a conversation,** because it would point the camera at a product whose photos aren't back yet.

## Not yet
- Importing files (the Import button, or dropping files from Finder) while an archived product is open puts them into the session the camera is filing to, not into that product. It's noted for a later step.
- Getting the original RAW back is not part of this. What comes back is the published JPEG.
- This hasn't been tried against your real Shopify store yet. The orchestrator's live check comes next: one archived test product brought back and republished with one re-framed photo, plus one clearing check and one full-size download.

## Needs you
Nothing.
