# 09C — Price labels from Pricing: see what needs a label, generate and print it

**Status: RATIFIED** (owner David Loorenz, 2026-11-23; chapter 16 entry A33). Shaper Claude. Branch `feature/agentic`.
Builds on 09B (the Pricing connection, the key in the Keychain, the event pull every 10 s) and 10H (shop-only items
opened in the normal workspace, remote pictures). Source: the Pricing package
`pricing-app/docs/integrations/session-manager/` (`LABELS.md`, `API.md` › Price labels, `EVENTS_AND_WEBHOOKS.md`),
read 2026-11-23. Where this chapter and the package differ on Pricing's behaviour, the package wins and this chapter
is amended.

## 1. Outcome

Printing price labels is part of the photographer's flow. Beyo Pricing already designs label templates, renders them
and keeps track of which products have a printed label. The owner (2026-11-23):

> "the capability of being able to download the appropriate print templates for a single product / session or a
> selection of products … on the main session listing i will need one more box … which is "labels" while in there the
> search bar works differently it works using the pricing api … as default it always loads the items that it's label
> hasn't been printed … in the filtration bar i can select the ones that has been, i can use the searchbar to search
> for a specific item … we continue to use the session manager interface for products, they are still sessions … the
> user should be able to print (generate the label for the product) regardless of where it is … we use the three dot
> menu … the label filtration still has the multi select capability because the user can batch label generate."

The photographer gets:

1. **A Labels box** next to Open / Published / All in the sessions list, with the number of labels still to print.
   Inside it the list, its filters and its search come from Pricing; the rows look and open like sessions.
2. **Generate label** (or **Reprint label**) in the ⋯ menu of every session row, in every box, and in the session's
   own ⋯ menu.
3. **Batch labels** in the Labels box: tick rows, or **Select all N**, then **Generate labels for N**.
4. **A check before anything changes:** what will happen to each product, grouped by template, with a preview of the
   real label. Then **Generate PDF**.
5. **The PDF saved and opened in Preview**, ready for ⌘P at 100 %.
6. **A label marker** on every session row and on the session's page: "Label printed · 4 Nov", "Missing price tag",
   "Price changed: new label needed", "No price yet".
7. **Always in step:** labels printed from Pricing itself, and labels made stale by a price change, update the markers
   and counts within the regular 10-second check, without reloading.

## 2. Owner decisions (2026-11-23)

1. Build it (the plan presented 2026-11-23, answered with the cards below).
2. **L1, products with no session on this Mac:** shown as sessions; opening one uses 10H's shop-only flow.
3. **L2, default list:** "Has a price, not printed" (Pricing's `missingPriceTag`). The other filters are Not printed,
   Printed and All.
4. **L3, the PDF:** saved in a Labels folder and opened in Preview.
5. **L4, the check step:** the check list grouped by template **plus** the label preview.
6. **L5, the label marker:** everywhere (Open, Published and Labels rows, and the session's page).
7. **L6, batch:** in the Labels box only, with **Select all N** across pages. Bigger than 300 is split into several
   runs. Open and Published keep their publish selection; single labels go through ⋯.
8. **L7, a label made stale by a price change:** updated quietly (marker, list, count). No sound, no toast.
9. **L8, templates:** Pricing's choice always. Session Manager shows the template's name and size and never offers
   another.

## 3. Vocabulary and boundaries

- **Labels provider:** the outside system that owns label templates, label state and rendering (Beyo Pricing in the
  MVP, adapter beside the conversations adapter). Core says "labels provider", "label state", "label run"; nothing
  Pricing-specific in core.
- **Label state:** Printed / Not printed, as the provider says. **Missing price tag** = has a valid selling price and
  Not printed. **Price changed** = a Printed label went back to Not printed because the price, SKU or barcode changed
  in Shopify (`resetReason`).
- **Check (plan):** the provider's dry run of a set of products: template, price, label state, outcome
  (`markPrinted`, `reprint`, `noPrice`, `fail`) and plain-words issues. Changes nothing.
- **Run:** the one call that generates the PDFs and marks products Printed. Only a run changes anything.
- **Labels folder:** `~/Documents/Session Manager Labels/`, where generated PDFs are saved.
- **Who owns what.** The provider owns templates, template choice, label state and rendering. Session Manager never
  stores label state as its own truth (it may hold a display copy, refreshed as §8 says), never creates or edits
  templates, never marks a label printed itself, and never changes prices.

## 4. Connection (extends 09B §4)

- The same Pricing key and store as conversations. Labels need two more scopes on the key: **`labels:read`** (lists,
  status, templates, check, preview, downloads, `label.*` events) and **`labels:write`** (generate). A Pricing admin
  issues a key with them; the owner pastes it in Settings → Pricing (Replace).
- **Test connection** also reads the templates list (`labels:read`) and adds one line: "Labels: allowed" or "Labels:
  this key can't read labels. Ask a Pricing admin to add labels:read and labels:write." Whether the key holds
  `labels:write` is known only at the first Generate (a 403 there shows §10's sentence).
- **No key, store unmatched, or no `labels:read`:** Session Manager makes no label calls. The Labels box is still
  there; its body says why in 09B's words plus the scope sentence above, with **Open Settings → Pricing**. Rows in
  other boxes show no marker, and ⋯ shows **Generate label** disabled with the same reason as its hint.

## 5. The Labels box (contract MC-L1)

- **Place:** a fourth box after Open / Published / All. Its count is the provider's `counts.missingPriceTag` (search
  applied), from the last list answer; before any answer, empty.
- **Inside it** the toolbar changes:
  - **Filters:** Missing price tag (default) · Not printed · Printed · All, each with its count from `counts`
    (`missingPriceTag`, `notPrinted`, `printed`, `total`).
  - **Search:** the same search box, now searching through the provider (SKU, title, product type: `q`), debounced
    like today. Changing the search or the filter goes to the first page.
  - The time filter, "To review" and the Unassigned card are hidden (the provider's list has no dates to filter on).
  - A small line under the toolbar: "Products created since <workFrom as 19 Nov 2026>" when the answer carries
    `workFrom`.
- **Order:** the provider's (newest created first). No time groups.
- **Pages:** 25 per page, through the provider's cursor. The pager shows **Previous** / **Next** and "Page 2 of 5"
  (pages from the filter's count). Going back reuses the cursors already seen. No jumping to a page number.
- **Rows** look like session rows (picture, name, SKU, pills, ⋯), with these columns: Item · Price · Label · Created.
  - **Matched to a session on this Mac** when a local item is linked to the same shop product (Pricing's numeric
    `productId` = the numeric tail of the item's Shopify link), else when exactly one local item has the same SKU.
    Then the row shows the session's name, its row picture (10H §4) and its usual pills, and opens that session.
  - **No session on this Mac:** the row shows Pricing's title and its `thumbUrl` picture (through 10H's search
    picture cache, 160 px). Opening it adopts the shop product (the existing adopt: one Shopify read, no write) and
    opens it as a shop-only item in the normal workspace (10H §6). After that it is matched like any other.
  - **Price:** `priceMinor` in öre as Swedish kronor, "8 900 kr" (thin space thousands); none: "No price".
  - **Label:** the marker (§7), from the row's own fields (no second call for these rows).
  - **Created:** the provider's `createdAt`, in the list's usual relative words.
- **Selection** (MC-L1 with §6):
  - Every row with a SKU can be ticked; the header box ticks the page.
  - When the whole page is ticked and the filter holds more, the bar offers **Select all N** (N = the filter's count).
    Pressing it pages the provider's list to its end for the filter and search shown (limit 500 per page) and selects
    every SKU it returns. The bar then reads "All N selected". The selection survives page changes in this box.
  - The batch bar in this box reads "N selected · **Generate labels for N**" (no Review & publish).
  - Leaving the box, or changing its filter or search, clears the selection.
- **Refresh:** on entering the box, after a run, and on a label event (§8), the page shown and its counts are read
  again (debounced to at most once per 2 s). Never a tight loop.

## 6. Generating labels (contract MC-L2)

Started from ⋯ on one row (any box, or the session page) or from the batch bar (Labels box).

1. **Check.** The app sends the products to the provider's check (`POST …/labels/plan`, by `productId` when known,
   else SKU; at most 300 per call, a bigger selection checked in consecutive calls and shown as one). The dialog
   opens at once with a busy state and shows:
   - one group per template: "<name> · <product types> · <W × H mm>", its products listed with SKU, name, price and
     what will happen;
   - products that can't be generated marked **Can't generate** with their `issues[].message`;
   - SKUs the provider didn't find (`unresolved`) as "Not found in Pricing", left out;
   - the footer and notes in Pricing's wording (LABELS §3.3): "{markPrinted} products will be marked Printed",
     "{failing} can't be generated with the current template. They're marked below.", "{reprint} already printed.
     They're reprinted and stay Printed."; for one product "The product will be marked Printed." / "Label status
     stays Printed." / "Label status stays Not printed.", "This product has no selling price. You can generate its
     label, but it stays marked Not printed.", "Printed {4 Nov}. Reprinting keeps it marked Printed.";
   - a **preview** of the selected product's label (`POST …/labels/preview`), shown as an image of the PDF's page,
     with ‹ › to step through a batch; while it loads, a placeholder of the label's size.
   - the button **Generate PDF**, or **Reprint PDF** for one already printed product; disabled when every product
     fails. **Cancel** changes nothing.
2. **Generate.** One press makes one run key per chunk of up to 300 ready products (outcome not `fail`), and calls
   `POST …/labels/runs` for each chunk in turn with its key. A network error or 5xx retries that chunk with the
   **same** key (twice, then the dialog offers Try again, which still reuses it). The dialog stays busy and can't be
   closed while a run is in flight. A double press starts nothing new.
3. **Done.** The title: batch "{rendered} of {items} labels generated" when anything failed, else "{items} labels
   generated"; one product "Label generated" / "Label not generated". Result lines: ✓ "{markedPrinted} marked
   Printed", ↻ "{reprinted} reprinted · stay Printed", ○ "{noPrice} stay Not printed · No selling price", ✕
   "{failed} failed" with each failure's SKU · name and message. Never "all done" when anything failed.
4. **Download.** One file in total: a single **Download PDF** that saves it, opens it in Preview and closes the
   dialog. Several files (several sizes or chunks): one row per file (filename, label count, W × H mm) each with
   **Download PDF**, plus **Done**. No file: **Close**. Download = the artifact's `downloadUrl` with the key, written
   to the Labels folder (`.part` then renamed; a taken name gets " (2)", " (3)" …) and opened in Preview. The dialog
   adds: "Print at 100 % (not Fit to page)."
5. **Afterwards:** failed products stay selected in the Labels box ("Failed products stay selected so you can fix and
   retry."); printed ones leave Missing price tag at the refresh that follows.

**⋯ menu entry:** "Generate label", or "Reprint label" when the row's state is Printed. Disabled with a hint when the
session's item has no shop product yet ("Publish first so Pricing can see it") or when labels are off (§4).

## 7. The label marker (contract MC-L3)

- **Words:** "Label printed · 4 Nov" (Printed, `labelPrintedAt`), "Missing price tag" (has a price, Not printed),
  "Price changed: new label needed" (Not printed with a `resetReason`; "SKU changed" / "Barcode changed" for those
  reasons), "No price yet" (no price). No marker when the state isn't known.
- **Where:** a pill in the row's pill line, in Open, Published and Labels; and beside the title on the session's page.
- **Source:** in Open and Published, one status call per page shown (`GET …/labels/status?productIds=…`, the page's
  rows that have a shop product, up to 25, so far within the limit of 200), after the page arrives and never
  delaying it. The session page asks for its one product when opened. Labels rows use their list fields.
- **Display copy:** the answers are kept in memory per product as the display copy, replaced by every newer answer and
  by §8's events. Nothing about label state is written to the database.

## 8. Staying in step (contract MC-L4)

- The 09B pull (every 10 s) already reads Pricing's event log; with `labels:read` it also carries `label.printed`
  `{productId, sku, runId, priceMinor}` and `label.reset` `{productId, sku, reason}`. Today they fall under "not for
  us: advance the cursor". From now on, each such event (deduplicated by id, the cursor advancing as today) updates
  the display copy for that product (Printed with the event's time, or Not printed with the reason) and tells the
  screen "labels changed" for that product.
- The screen then updates any marker of that product and refreshes the Labels box's page and counts (§5 Refresh).
- **No notification** of any kind for label events (owner L7). A reprint sends no event, which is expected.
- During the first connection (09B §5 step 2) label events are only paged past, as today.

## 9. Invariants

- **M-L1:** Session Manager never stores label state as its own truth: a marker or count on screen comes from the
  provider's latest answer or event; nothing about label state is written to the database.
- **M-L2:** only a run changes anything at the provider, and a run starts only from **Generate PDF** / **Reprint PDF**
  pressed in the check dialog, for products that check just showed. List, status, templates, check, preview and
  download only read.
- **M-L3:** one press makes one key per chunk; every retry of that chunk reuses it; a double press makes no second
  run.
- **M-L4:** a run with any failure is never reported as fully generated; every failed product is listed with its
  message.
- **M-L5:** labels never write to Shopify (opening an unmatched row only reads, through the existing adopt); the key
  stays in the Keychain and is never logged; delete session still never touches Shopify or Pricing.
- **M-L6:** without a key, a matched store and `labels:read`, no label call is made.

## 10. Errors (words shown)

| When | Words |
|---|---|
| 401 / 403 `insufficient_scope` | "Session Manager isn't allowed to print labels. Ask a Pricing admin." |
| 404 on the store | 09B's "This key is no longer tied to <shop>" |
| check / run 422 `no_products` | "None of these products were found in Pricing." |
| check item `issues[]` | the `message` as given |
| preview 404 / 409 / 422 | the `message` as given, in place of the preview |
| preview / run 503 `renderer_unavailable` | "Labels can't be generated right now. Try again in a moment." (nothing was marked) |
| download 410 `artifact_expired` | "This file has expired. Generate the labels again." |
| list 503 `product_list_unavailable` | "The product list isn't available right now." with Try again |
| network / 5xx | "Pricing can't be reached right now." with Try again; the last list stays on screen |
| saving the PDF fails | "Couldn't save the PDF: <reason>." with Try again (the file stays downloadable for 14 days) |

## 11. Out of scope

- Templates, template assignments, the default template, the Work-from date, prices: all in Pricing.
- Choosing another template from Session Manager (owner L8).
- Printing directly to a printer or a printer setup: the output is a PDF printed from Preview.
- Batch labels from Open or Published (owner L6).
- Webhooks: a desktop app has no public address; the event pull is used (09B §5).
- A history of label runs in Session Manager.

## 12. Acceptance scenarios

1. **To-do list.** Pricing has 12 priced, unprinted products. The Labels box shows 12, opens on Missing price tag
   with 12 rows newest first, each with picture, SKU, price and "Missing price tag".
2. **Search and filters.** Typing a SKU narrows the list through Pricing; Printed shows printed products with
   "Label printed · <date>"; the counts follow the search.
3. **One label from a published session.** ⋯ → Generate label → the check shows template and size, "The product will
   be marked Printed." and the label preview → Generate PDF → "Label generated" → Download PDF saves it in the Labels
   folder, opens it in Preview and closes. The row now shows "Label printed · today", and the Labels count drops by
   one.
4. **Batch with problems.** Three products: one priced, one with no price, one with no barcode on a barcode
   template. The check shows the third as Can't generate with its message and "1 products will be marked Printed";
   after Generate: "2 of 3 labels generated", the failure listed, the failed one still selected.
5. **Mixed sizes.** A batch over two templates of different sizes ends with two Download PDF rows and Done.
6. **Select all.** 47 missing; Select all 47 → Generate labels for 47 → one check, one run.
7. **Retry safety.** The network drops during Generate; Try again returns the same run; nothing is marked twice.
8. **Printed elsewhere.** A pricer prints a label in Pricing; within about 10 s the row's marker in Session Manager
   turns to "Label printed" and the count drops, with no sound.
9. **Price change.** A printed product's price changes in Shopify; within about 10 s its marker reads "Price changed:
   new label needed" and it reappears under Missing price tag.
10. **Product from the other Mac.** A Labels row with no session here opens as a shop-only item in the normal
    workspace; Generate label works from its ⋯.
11. **No permission.** A key without `labels:read`: the Labels box says so with Open Settings → Pricing; no label call
    is made.
12. **Unpublished session.** Its ⋯ shows Generate label disabled with "Publish first so Pricing can see it".

## 13. Milestones (for the planner)

- **L1 Labels provider and the marker:** the labels port and its Pricing adapter (list, status, templates, check,
  preview, run, download), the label events in the existing pull, the in-memory display copy and its "labels changed"
  event, Settings → Pricing's labels line, the marker on Open / Published rows and on the session page, the ⋯ entry
  (disabled states only until L3).
- **L2 The Labels box:** the fourth box and its count, the provider-backed list (filters, counts, search,
  Previous / Next pages), rows matched to sessions or shown as shop products (opening adopts), selection with Select
  all N, refresh on events.
- **L3 Generate and download:** the check dialog (groups, outcomes, issues, the preview image), runs with one key per
  chunk and safe retries, the done dialog, saving to the Labels folder and opening in Preview, the ⋯ entry and the
  batch bar wired, the errors of §10.

The first real Generate (it marks products Printed in the live Pricing) is the owner's, after L3.
