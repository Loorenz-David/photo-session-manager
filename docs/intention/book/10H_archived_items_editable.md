# 10H — Archived and shop-only items: seen, opened and edited like any session

**Status: RATIFIED** (owner David Loorenz, 2026-11-23; chapter 16 entry A32). Shaper Claude. Branch `feature/agentic`.
Amends 10F: the **Rules** bullet "After clearing, the item is read-only" and **Q3** ("reorder/retouch would need a new
photo session") are reversed. 10F's clearing itself, its verification before clearing, the Keep files pin and M41 stay.

## 1. Outcome

Published items are cleared after N days (10F rule 3), and from then on Shopify holds their pictures. Today such an
item shows a broken "?" in the sessions list, has a read-only page instead of a workspace, and its photos can no
longer be rearranged, rotated, framed or edited in Capture One. The owner (2026-11-23):

> "the session listing container doesn't show the first picture i should be capable of seen this (we can batch fetch
> the products that are rendering on the listing optimally and efficiently). when i enter into the session i should
> have the same session interface as if the picture where still existing … i should be able to edit the
> re-arrangement, rotations and framing, so as editing the pictures in capture one … the program should allow to
> download the picture from shopify to be edited and then i can publish those changes. i should also be capable of
> editing the details of the product … we will add pagination so that i don't consume the query shopify points on a
> single load … on the search bar the products that are shown should load their main first picture always."

The photographer gets:

1. **A picture on every row.** The sessions list shows each item's first main picture, also for archived items (from
   Shopify, fetched small and kept on this Mac).
2. **Pages.** The sessions list shows **25 sessions per page** with page numbers.
3. **A picture on every search result.** The product search (New session, Assign product) shows each result's first
   main picture: this Mac's own, or the shop's.
4. **The normal workspace.** Opening an archived item, from the list or from the search, shows the same workspace as
   any session: Main, Condition, Details, Publish. Its photos are the Shopify pictures.
5. **Editing by bringing the photos back.** The first photo edit (rearrange, classify, rotate, framing, Capture One)
   brings the item's published pictures back from Shopify at full size into its session folder and Capture One. From
   then on the item is an ordinary session, and **Publish** sends only what changed.
6. **Products from the other Mac too.** A shop product with no session on this Mac (photographed on the other Mac),
   opened from the search, gets the same: its Shopify pictures in the workspace, editable the same way.
7. **Details always editable,** without bringing anything back (as today, 10F Q3's "attributes stay editable").

## 2. Owner decisions (2026-11-23)

1. Build it as planned (the plan above, answered with the three cards below).
2. **Card O1, products from the other Mac:** "Yes, same flow."
3. **Card O2, edit source:** "Yes, JPEG is fine." What comes back is the **published JPEG**, not the RAW: framing can
   only crop tighter or rotate (the area cut away at publish is gone), and Capture One edits a JPEG. The Keep files pin
   (10F Q2) stays the way to keep RAWs of items expected to need heavy edits.
4. **Card O3, page size:** "25 per page" (page numbers, not Show more or endless scroll).

## 3. Vocabulary

- **Remote item:** an item whose pictures are only on Shopify. Two kinds: **archived** (10F cleared its files on this
  Mac; `item_retention.cleared_at` set) and **shop-only** (linked to a shop product but never photographed on this Mac:
  adopted from the search, or photographed on the other Mac).
- **Remote picture:** one Shopify image of a remote item with its role (gallery = main, condition), position, Shopify
  media id and CDN URL. Archived items already record these in `archived_images` (10F); shop-only items get them when
  first opened (§6).
- **Bring back:** the step that turns a remote item into an ordinary session by downloading its remote pictures at full
  size and adding them as the session's photos (§7).
- **Shopify points:** the Admin GraphQL cost budget. **CDN image downloads cost no points**; only GraphQL queries and
  mutations do.

## 4. Pictures in the sessions list (MC-H1)

- A row's picture is, in this order: the session's first main photo's local preview **if that file exists**; else, for
  a remote item, its **remote cover** (gallery position 1) from the cover cache; else a neutral placeholder. A preview
  path whose file is gone (as 10F leaves them today) is never handed to the screen.
- **Cover cache:** `<data>/previews/remote/<item_id>/cover-<sha256(media_id)[..12]>.jpg`, downloaded with the CDN size
  parameter at **320 px** wide. One file per item; a new cover media id replaces it.
- **Fetching:** only for the rows on the page being shown, in the background, at most **4 downloads at a time**, never
  delaying the list. Each finished cover updates its row (an event, not a reload of the list).
- **Stale URLs:** if a cover download answers 404/403, the app collects the page's failed items and refreshes their
  media URLs in **one** batched GraphQL query (`nodes(ids:)` over the media ids, at most 25 per query), updates the
  stored URLs and retries once. A media id Shopify no longer has leaves the placeholder and a Health note.
- The hover card and the workspace sessions panel use the same picture.

## 5. Pages and search pictures (MC-H2)

- **Sessions list:** 25 per page, page numbers under the list. The filter, the search box and the time groups apply to
  all sessions first, then the page is cut, so a search finds items on any page. Changing filter or search goes to
  page 1. Other callers that need every session (header, pickers, publish queue, review queue, visit bar) keep the
  unpaged list.
- **Search results** (New session, Assign product):
  - Local results: the item's row picture (§4).
  - Shop results: the product's first picture, which the shop search already returns (`featuredMedia`) and which
    today's code drops. It is fetched through the same cache at **160 px** (`<data>/previews/remote/search/`, cleared
    when it passes 200 MB, oldest first).
  - The first-letter tile stays only while a picture loads or when there is none.

## 6. Opening a remote item: the normal workspace (MC-H3)

- Opening an archived or shop-only item shows the standard workspace (stage, Main and Condition, Details, Publish,
  conversations), not the read-only page. A short line under the header says where the photos are: "Photos are on
  Shopify. Editing a photo brings them back to this Mac first." The "Archived in Shopify" label stays on the row.
  A shop-only item counts and filters as **Published** in the sessions list (amended 2026-11-24, 09C §14.3, A34).
- **Shop-only items, first open:** one GraphQL read of the product's media and condition files (the existing
  readback, one query) records its remote pictures. Later opens use the record. The record is refreshed when the
  photographer presses **Refresh from Shopify** (in the session's ⋯ menu) or when a picture's URL fails (§4 rule).
- **Showing the pictures:** the stage and grids show the remote pictures at the existing 1600 px (10F's archived
  download, kept in `<data>/previews/archived/<item_id>/`). They download in the background, **4 at a time**, the grid
  showing each as it arrives. Opening never waits for all of them (today's download inside the command goes).
- **Activating** a remote item never moves the camera's capture folder or Capture One's session collection until
  bring back. New photos keep going where they went.
- **Allowed without bringing back:** Details (every attribute but media lists, as today), the writer, conversations,
  Publish of details only (no photo changes), and looking at the photos.
- **Starting a photo edit** (rearrange, classify, hide, rotate, framing, a Capture One style or adjustment, Retouch)
  first asks: "Bring back N photos from Shopify to edit them? About X MB." → **Bring back** / Cancel. There is no
  silent download.

## 7. Bring back (MC-H4)

1. The app downloads each remote picture at **full size** (the CDN URL without a size parameter) to the item's session
   folder `<capture root>/<folder_key>/`, named `<SKU>-main-01.jpg` … and `<SKU>-condition-01.jpg` …, written to
   `.part` and renamed. It shows progress ("Bringing back 4 of 10 photos…"). A failed download is retried twice; if
   any still fails, nothing is imported and the item stays remote, with the reason and Try again.
2. Only when all are on disk: in one transaction the app adds them as the session's photos, main then condition, in
   the published order, and retires the old cleared photo rows (hidden, not deleted, so history keeps them). The item's
   retention record gets `cleared_at` cleared and `restored_at` set.
3. **JPEG photos.** A brought-back photo is a JPEG, not a RAW. Its original preview is the JPEG itself (downscaled
   like any preview), its frame is the JPEG's pixel size, its orientation its EXIF orientation (none means upright).
   Composition starts with no crop and no turn, so the photo looks exactly as published. Every photo operation that
   works on a RAW works on it.
4. **Capture One:** the app makes the item's folder Capture One's capture folder and collection, as on any activation,
   and syncs the folder so the JPEGs appear. Capture One's adjustments apply to them like to RAWs.
5. **Publishing afterwards sends only changes.** Each brought-back photo is linked to the Shopify media it came from,
   as already uploaded, with its current composition and edit state as the uploaded state. So:
   - an untouched photo is not uploaded again;
   - a photo whose composition, rotation or Capture One edit changed replaces its media (the existing superseded path:
     new upload, old media removed);
   - a new order reorders the gallery (the existing reorder path);
   - a hidden photo removes its media on publish, as for any published item.
   For a shop-only item, the media it brought back count as the app's own from then on, so the publisher may reorder,
   replace and remove them (today they count as foreign and block an update).
6. Bring back **never writes to Shopify**. Only Publish, confirmed by the photographer, does.
7. After it, the item follows 10F like any published item: N days after its next publish (and not dirty) its files are
   cleared again, unless Keep files is on.

## 8. Shopify points (MC-H5)

- Picture downloads go to Shopify's CDN and cost no points.
- GraphQL use added by this chapter: the batched URL refresh (§4, one query per up to 25 media), the shop-only
  first-open read (§6, one query per item) and Refresh from Shopify. Nothing per row, nothing per page load.
- The adapter reads `extensions.cost.throttleStatus` on every answer. Background work (cover refresh, shop-only
  reads, 10F's verification) waits while `currentlyAvailable` is below 25% of `maximumAvailable`. A press by the
  photographer is never delayed.

## 9. Invariants

- **M-H1:** no Shopify write outside a confirmed Publish (bring back, refresh, covers and search pictures only read).
- **M-H2:** bring back imports all of an item's remote pictures or none, and never deletes anything on Shopify or in
  Capture One's catalogue.
- **M-H3:** a publish after bring back uploads only photos whose composition, rotation or edit changed since they
  came back, plus new ones.
- **M-H4:** the screen is never given a local image path whose file is missing.
- **M-H5:** delete session still never touches Shopify or Pricing.

## 10. Out of scope

- Getting the RAW back (it is not on Shopify).
- Archived items in Capture One's catalogue: the old missing entries stay (10F Q4); the brought-back JPEGs are new
  entries.
- Pages in the pickers and other lists (only the sessions list is paged).
