# 04 — Workflow and scenarios

## The core workflow (independent of any integration)
1. Create a session for an item: found on the connected integration, or created locally.
2. Capture photos (tethered through the camera provider) or import them.
3. Review: approve as main, move to condition, or delete (hide). Order main and condition.
4. Adjust: the editing provider applies the basic layer automatically; manual retouch any time.
5. Fill item information; accept or edit AI suggestions (agent post-MVP, chapter 08).
6. Complete: export the ordered files locally, and/or publish to the connected integration.

## Scenarios (acceptance stories; each maps to measurements in chapter 14)

**S-A. Opening the studio.** The photographer opens Session Manager. The header's camera dot is
green when Capture One reports a camera. If not, clicking the dot retries, and the error state
explains what to check (USB mode, Imaging Edge closed). The Sessions list shows open sessions
grouped by last activity; published ones are hidden behind the filter.

**S-B. New session for an existing item.** New session → type a SKU or title → the integration
returns matches with thumbnails → pick one. A session opens with the item's type and SKU in the
header and an empty stage reading "Ready for the first photo". The editing provider's capture
folder is switched for this session (decision D1).

**S-C. Shooting and sorting.** Each shutter press lands a RAW. Within about a second the photo
appears on the stage from the camera's embedded preview, marked unreviewed and "Processing".
Seconds later the adjusted version fades in. The photographer presses A, C or Delete, or swipes;
the next unreviewed photo opens. Deleting shows "Photo deleted · Undo ⌘Z" for 4.5 s; ⌘Z keeps
working until the next delete. Main 01 is the cover. Drag reorders in the grid or the folded strip.

**S-D. New item.** Searching "Brass floor lamp" finds nothing; the row "Create 'Brass floor lamp'"
creates a local item with a new SKU; the integration creates the product at first publish.

**S-E. Forgot to pick a session.** Three photos arrive with no session open. They land in
Unassigned, a toast offers "Open". In the Unassigned workspace the sessions panel is open; the
photographer drags photos onto a session, or assigns the whole set to an item.

**S-F. Filling the details.** D opens the form: Details, Condition, Product. Fields come from the
attribute schema the integration provided; values save locally as typed. Condition photos are
pickable in the Damage report field (all selected by default). Publishing never requires a
complete form.

**S-G. Publishing one.** Publish opens the popover: main thumbnails in order, condition
thumbnails, product-details count, and the note that unreviewed photos will publish in Main
(decision N2). "Publish 9 images" records the confirmation and runs in the background: Creating
product (new items) or Preparing images → Uploading n of N → Updating product → Published.

**S-H. Publishing many.** Tick rows → "Review & publish 4" opens the form as a queue; "Confirm &
next" walks the items; "Publish N confirmed" sends the confirmed ones.

**S-I. The Mac sleeps mid-publish.** Two of four published, the third has uploaded 2 of 4. Next
start, the run resumes from the upload step after checking what already exists. End state equals
an uninterrupted run; no duplicates.

**S-J. Failure.** Upload times out: the row shows "Publish failed", a sticky toast offers Retry, the
popover shows the reason and that nothing was overwritten, per-photo badges show "On Shopify" or
"Not uploaded". Retry re-runs only the failed step.

**S-K. Editing a published session.** Moving, reordering or deleting photos sets the session back
to Ready. Publishing again reconciles the integration to the session's current lists (decision N1).

**S-L. No integration configured.** Everything above works except publish: the header action
reads "Export", and completing a session writes the ordered files to the export folder (M7).

**S-M. Phone in hand (design pending).** The same app opened on a phone on the studio Wi-Fi,
mirroring the current session and offering approve, condition, delete, undo and capture.
Scope position in chapter 13.
