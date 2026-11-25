# Phase L4_owner_corrections: your label corrections: easy-to-read cards, straight to Preview, and products from the other Mac under Published

## What you can do now
The label check now has one card per product. Each card shows the picture on the left with the SKU under it, the title on one line, then the price and what will happen. Small pills appear under a card only when something needs saying: "No price", "Can't generate: …", "Not published yet" and a **QR link** button.

The dialog keeps its size. Only the list scrolls, so **Cancel** and **Generate PDF** are always in view.

Pressing **Generate PDF** now goes straight to Preview: every file is saved in Documents › Session Manager Labels and opened, ready for ⌘P. When everything worked, the dialog closes with a short note. When something went wrong, it stays open and shows the problems as cards.

Products photographed on the other Mac now sit under **Published** in the Sessions list, and the Open number no longer counts them.

## Try it yourself
You need the Pricing key with `labels:read` and `labels:write`. **Generating really marks products Printed in Pricing**, so pick products you mean to label.
1. Start the app:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager-agentic/apps/desktop
   npm run tauri -- dev
   ```
2. In the Sessions list, open once a product that was photographed on the other Mac. It should now sit under **Published** with "Archived in Shopify", and the Open number should not count it.
3. Open **Labels**, tick three or more products and press **Generate labels for N**.
   - The dialog should keep its size.
   - The list of products should scroll, while the summary and **Cancel / Generate PDF** stay at the bottom.
4. Look at the cards.
   - Each shows a picture (or a grey tile when none is on this Mac yet), the SKU under it, a one-line title (hover to see all of it), then the price and what will happen.
   - Pills appear only when needed.
5. On a product that is not on the web shop yet, press **QR link**.
   - The address the QR code opens should appear under the pills.
   - Press **QR link** again to hide it. Clicking the address opens your browser.
6. Click another card. It gets a frame, and its label preview shows on the right.
7. Press **Generate PDF**.
   - The PDF should open in Preview by itself, the dialog closes, and a note says "N labels generated · opened in Preview".
   - Print with ⌘P at 100 %.
8. Try a batch with a product that can't be generated (for example one without a barcode on a barcode template).
   - The files that were made still open in Preview.
   - The dialog stays with "2 of 3 labels generated" and shows the problem product as a card with its reason.
   - After **Close**, that product is still ticked in the Labels box.
9. From a session's ⋯ menu choose **Generate label**, then **Generate PDF**. Preview should open, and the note says "Label generated · opened in Preview".
10. To run the automated checks for this phase:
    ```
    cd ~/Desktop/Developer/Photo-Session-Manager-agentic
    cargo test -p psm-testkit --test suite h4_details_only_publish::
    cargo test -p psm-testkit --test suite l4_labels_check_cards::
    cargo test -p psm-integrations --lib conversations::beyo_pricing::labels
    cargo test -p psm-desktop
    cd apps/desktop && npx vitest run src/features/labels
    ```

## Choices made for you
Say the phrase in quotes to change a choice.

**Products from the other Mac**
- Such a product counts as Published from its first open, once the app has read its Shopify pictures. In the moment before that read, or when its shop record has no picture at all, it stays under Open. Say "count it as Published right away" to change that.
- It moves back to Open only while a publish of it is running, waiting for you, failed or cancelled.
- After its photos are brought back to this Mac it sits under Open, as before. This is the open question below.

**The check dialog**
- A card's picture is the one already on this Mac: the session's row picture, else the picture the Labels list already downloaded. A picture that is still downloading shows the grey tile, and the card does not update while the dialog is open. Say "update card pictures live" to change that.
- Pictures are found for products you have seen in the Labels list or picked with Select all since the last Test connection. For a product opened only from a session, the picture comes from that session.
- "QR link" appears only when Pricing's note names an address. The app reads the address out of Pricing's sentence, so if Pricing changes its wording you see "Not published yet" without the button.
- The dialog is at most 720 points tall (less on a small screen).
- Warning colours: amber for "No price" and "Not published yet", red for "Can't generate", grey for any other note. The selected card gets a dark frame.
- Cards are selected with a click, or with Enter or Space after you tab to one. They are not buttons, so they don't count among the dialog's buttons.

**Generate and Preview**
- A product Pricing did not find keeps the dialog open after Generate and counts in the title ("2 of 3 labels generated"), even though the check already showed it as "Not found in Pricing".
- The note says "Label generated · opened in Preview" whenever exactly one label was made, also from a batch.
- A file that was saved but that Preview could not open is not saved again. The dialog shows where it is. Say "offer to open it again" to add a button for that.
- The dialog cannot be closed while the files are being saved and opened (a few seconds after Generate). During that time the check stays on screen with "Opening in Preview…" and the title still reads "Generate labels for N".
- The result still lists the numbers (marked Printed, reprinted, stay Not printed, failed) above the problem cards. Products Pricing did not find are listed there too.
- A problem card shows the product's picture, SKU, title and price from the check, with one red "Can't generate: …" pill. When the product was not in the check, it uses the name the failure gives, and the price reads "No price".
- While **Try again** saves a file again, **Close** stays usable.
- Select all now also remembers each product's SKU. So after Select all, opening a product finds its session by SKU, as it already did after browsing the list.

## Not yet
- A product from the other Mac whose photos were brought back stays under Open until you publish it from this Mac (see the question below).
- A card's picture does not appear while the dialog is open, even if it finishes downloading.
- Two small polish items are noted for later. A product without a title in Pricing shows an empty title line. The problem cards in the result view look clickable, but clicking does nothing.

## Needs you
**Card 2: a product from the other Mac after you bring its photos back.** **Open.**
- **Question:** After you bring back the photos of a product from the other Mac, should it stay under Published
  until a photo really changes (yes), or sit under Open from then on, as today (no)?
- **Story:** You open a sofa that was photographed on the other Mac and start to rotate a photo; the app brings its 8
  photos back to this Mac. Today the sofa then sits under Open, even if you cancel the rotation and change nothing,
  and stays there until you publish it from this Mac.
- **Branches:** **Yes**: it stays under Published until a photo changes; this needs extra work after L4 (the app
  must remember "published, as it came back"). **No**: nothing more is built; a brought-back product is under Open
  until you publish it here.
- **Recommendation:** No: photos only come back when you start a photo edit, so the product is about to change anyway.
- **On silence:** L4 is built (it does not depend on this); a brought-back product stays under Open as today, and
  nothing more is planned.
- **Trace:** 09C §14.3 second bullet against book 07 "Dirty" and 10H §7; MC-H3.7A; L4·U1 Notes (no row asserts it).
