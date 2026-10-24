# Session Manager: design notes

A macOS desktop app for furniture photography: camera capture → Capture One adjustment → sort → fill product data (with an AI agent) → publish to Shopify. Desktop only. There is no phone layout; `screens/narrow/` shows the app at a 924px-wide window, which is the smallest size designed for.

Screenshots are in `screens/`. The prototype source is in `source/session-manager.html` (main) and `source/metafield-input.html` (the input generator). Read them for structure, values and behaviour; the markup uses a prototype runtime and isn't meant to be transplanted.

## Screens

1. **Sessions (home).** A list of product sessions grouped by last activity (Today, Yesterday, This week, Last week, then by month). Each row has a thumbnail, item type and SKU, photo counts, status, and time. The default filter is Open; Published is hidden until chosen. `01`–`08`
2. **Unassigned photos card.** Appears at the top of Sessions only when photos were taken with no product selected. Clicking it opens those photos in the workspace. `01`, `21`
3. **New session dialog.** Searches Shopify products by type or SKU. Typing a name that doesn't exist adds a "Create '…'" row, which creates the product on Shopify at first publish. `09`, `10`
4. **Workspace (session).** A large photo stage on the left and the photo panel on the right (Main images, then Condition). The panel folds to a 64px strip of thumbnails. `11`–`20`
5. **Unassigned workspace.** The same workspace with a "No product" tag. The sessions panel opens on the right automatically; drag photos onto a session to move them. "Assign product" replaces Publish. `21`
6. **Product details form.** A modal with three steps: Details, Condition, Product. The left column shows the photos that will be uploaded (Shopify-style grid with a large cover). The right column holds fields generated from Shopify metafield definitions. `22`–`31`
7. **Research panel.** A side panel inside the form showing the AI agent's timeline: what it looked at, searched and read, and which fields it set at each step. `24`, `25`
8. **Notifications drawer.** Publish history with progress, failures and Retry. `08`

## Flows

- **Capture.** Photos arrive automatically while the camera is connected. If a session is open they go to its Main images, marked unreviewed and "Processing" until Capture One returns the adjusted version. If no session is open they go to Unassigned, with a toast offering "Open".
- **Review.** Each photo is approved (stays in Main), moved to Condition, or deleted, using the round buttons, the keyboard, or swipes on the stage: right = approve, up = condition, left = delete. The threshold is 120px horizontal or 110px vertical. After an action the next unreviewed photo opens. On a photo already in Condition, the middle button changes to "Move to Main".
- **Reorder.** Drag in the grid or the folded strip. A gap opens where the photo will land and the others slide aside. Main position 01 is the Shopify cover.
- **Move or copy between sessions.** Drag a photo onto a session in the sessions panel to move it. The ⋯ menu offers "Move to session…" and "Copy to session…" (copy keeps the original).
- **Fill details.** Open the form from the header (D) or the publish popover. Values save locally as you type. Publishing never requires the form to be complete.
- **AI agent.** It starts when a session's main photo exists. It fills fields in the background and marks each one with its confidence and sources. See "AI agent" below.
- **Publish one.** Open the Publish popover in the session, or click "Ready · Publish" on a row in Sessions. Publishing runs in the background.
- **Publish many.** Tick rows, or tick the header box for all publishable sessions. "Review & publish N" opens the form as a queue. Walk each item with "Confirm & next"; "Publish N confirmed" sends what's done at any point.
- **Publish steps.** Creating Shopify product (new products only) or Preparing images → Uploading n of N → Updating Shopify product → AI agent is writing the description (skipped for a manual description) → Published.
- **Failure.** An upload failure shows red "Publish failed", a sticky toast with Retry, a popover with the reason and progress, and per-image badges inside the session (✓ on Shopify or "Not uploaded"). A description failure shows "Description failed" / "Description not written". Retrying it re-runs only the description step and doesn't re-upload.

## Keyboard

| Key | Where | Action |
|---|---|---|
| A | Workspace | Approve as main image (or move a Condition photo back to Main) |
| C | Workspace | Move to Condition |
| ⌫ / Delete | Workspace | Delete photo |
| ⌘Z | Anywhere | Undo the last delete |
| ← / → (↑ / ↓) | Workspace | Previous / next photo |
| Space | Workspace | Toggle Original / Adjusted |
| F | Workspace | Fold / unfold the photo panel |
| I | Workspace | Item details card |
| D | Workspace | Open the product details form |
| S | Anywhere | Capture (prototype stand-in for the camera shutter) |
| N | Sessions | New session |
| Esc | Anywhere | Close the topmost layer (AI popover → research panel → customize mode → form; popovers; selection) |
| ↑ / ↓ / Enter | New session, dropdowns | Move and pick; Enter picks the first match in a filtered dropdown |

## Undo timing

Delete is instant. A toast "Photo deleted · Undo ⌘Z" stays for **4.5 s**. ⌘Z keeps working after the toast closes, until the next delete; only the last delete is undoable. The photo returns to its original position and becomes the active photo.

Other toasts last 3.5–6 s; errors stay until dismissed.

## Original / Adjusted toggle

- One round pill at the bottom-left of the stage, or Space. It shows the camera original or the Capture One adjusted version.
- It crossfades over **700 ms**: both images are stacked and the adjusted one fades.
- While Capture One is still processing, the pill shows a spinner and the photo shows the original. The adjusted version fades in automatically when it arrives.
- The toggle applies to the whole session view, not per photo.

## Decisions the pictures don't show

- **Colour roles.** Near-black is the primary action. Red appears only for failure, Delete hover, a disconnected camera and the failure count. The quiet slate blue (`--ai*`) marks everything the AI agent touched, and nothing else.
- **Metafields drive the form.** Fields are generated from the shop's metafield definitions; the type → input mapping is in `source/session-manager.html` (`KIND`).
  - Single-line text with choices: pills when there are ≤ 4 short options, a searchable dropdown otherwise, cards when options carry descriptions (Detailed condition), and a severity scale for the N/A…Major set.
  - Lists with choices: multi-select chips or a multi-select dropdown.
  - Other types: `dimension` is a number plus unit, `number_integer` a stepper, `list.file_reference` picks from the session's condition photos (all by default), and `rating` stars.
  - Unknown types render as "Edited in Shopify admin".
  - The definitions used are in `reference/SHOPIFY_PRODUCT_METAFIELDS.md`.
- **Condition step.** It holds every `Condition.*` metafield, plus Damage report and Condition report.
- **Customize fields.** Per item type and per step, saved locally: reorder by drag, remove, add via a search of all shop metafields, and a default value per field (marked "Default" in the form). Title, Description and Tags can't be removed.
- **SKU and barcode.** Editable from the form header.
- **Tags.** Free entry plus suggestions from the shop's tags.
- **Description.** "AI agent writes it" is the default. The field is optional notes for the agent, and the agent writes after publish, never inside the form. "Write manually" is the fallback and is published as written.
- **AI agent rules.**
  - **Pills:** a field it filled shows a pill with 3/2/1 confidence bars. Low confidence reads "Check".
  - **Popover:** the pill opens the reason, sources, Looks right, Clear, and See research.
  - **Human edits win:** any human edit makes a field human-owned and the agent never overwrites it.
  - **Accept all:** skips Low-confidence fields.
  - **Research panel:** lists steps in order. Focusing a field dims the steps that didn't lead to it.
- **Publishing.** It runs in the background and never blocks the workspace. Sessions show live progress. A failed session keeps the images that did upload.
- **Dirty state.** Editing a published session (moving, reordering or deleting photos) puts it back to Ready, which is how re-publishing works.
- **Light/dark toggle.** It's only on the Sessions page, so it doesn't crowd the workspace.
- **Not for production.** The prototype controls (bottom-right pill), simulated capture timing, fake publish timing, mock products, mock agent research and links, and the made-up barcodes.
