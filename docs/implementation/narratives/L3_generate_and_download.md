# Phase L3_generate_and_download: generate, download and print price labels

## What you can do now
You can now make price labels from Session Manager.

**Where to start.** Every published session's ⋯ menu has **Generate label** (or **Reprint label** when its label is already printed). So does the session page, and so does every row of the Labels box. In the Labels box, **Generate labels for N** is now on: it works for the products you ticked, in the order you ticked them.

**The check.** A dialog first shows what will happen, and nothing changes yet:
- the products grouped by label template, each with its price;
- what will happen to each product: "Will be marked Printed", "Reprint · stays Printed", "No price · stays Not printed" or "Can't generate", with Pricing's reason;
- a picture of the real label.

**Generate PDF.** Pricing makes the labels and marks the products Printed. The result tells you exactly what was marked, what stayed Not printed and what failed. If the connection drops, the app tries again with the same request, so nothing is ever marked twice.

**Download PDF.** The file is saved in Documents › Session Manager Labels and opened in Preview, ready to print at 100 %.

## Try it yourself
You need a Pricing key that may read and write labels (`labels:read` and `labels:write`). **Generating really marks products Printed in Pricing**, so the first time pick a product you mean to label.
1. Start the app:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager-agentic/apps/desktop
   npm run tauri -- dev
   ```
2. Go to **Sessions → Published** and open the ⋯ of a product that has a price. Choose **Generate label**.
   - The dialog should first say "Checking…".
   - Then it shows the template with its size, for example "Label 70×100 · 70 × 100 mm".
   - It shows "The product will be marked Printed." and, on the right, a picture of the label on white.
   - Press **Cancel**. Nothing should change in Pricing.
3. Open it again and press **Generate PDF**. The dialog stays busy and cannot be closed until Pricing answers, then shows "Label generated". Press **Download PDF**.
   - The file should appear in Documents › Session Manager Labels and open in Preview. macOS asks once to allow access to the Documents folder.
   - The dialog closes. Print at 100 % (not Fit to page).
4. Back in the list, the row should read "Label printed · today". Its ⋯ should now say **Reprint label**. The Labels box's number drops by one at its next refresh.
5. Open **Labels** and tick three products, one of them without a price. Press **Generate labels for 3**.
   - The check should list them by template and say how many will be marked Printed.
   - Press **Generate PDF**. The result should say what was marked Printed and what stays Not printed.
6. Try a product Pricing cannot make a label for (for example no barcode on a barcode template). It shows "Can't generate" with the reason. After Generate, the title reads "N of M labels generated" and that product is still ticked in the Labels box.
7. Try a batch that uses two label sizes. It should end with one row per file, each with its own **Download PDF**, plus **Done**.
8. Turn Wi-Fi off and press **Generate PDF**. Wait for "Pricing can't be reached right now.", turn Wi-Fi back on and press **Try again**. The labels should be generated once, not twice.
9. Open the ⋯ of a session that was never published. **Generate label** should be greyed out with "Publish first so Pricing can see it".
10. To run the automated checks for this phase:
    ```
    cd ~/Desktop/Developer/Photo-Session-Manager-agentic
    cargo test -p psm-testkit --test suite l3_labels_check::
    cargo test -p psm-testkit --test suite l3_labels_generate::
    cargo test -p psm-testkit --test suite l3_labels_download::
    cargo test -p psm-desktop
    cd apps/desktop && npx vitest run src/features/labels
    ```

## Choices made for you
Say the phrase in quotes to change a choice.

**The check**
- The done dialog counts products the check refused as failed ("2 of 3 labels generated"), although only the products that passed are sent to Pricing.
- Products are always sent to Pricing by their product id, and Pricing's own choice of template is never overridden.
- The outcome words on each line are "Will be marked Printed", "Reprint · stays Printed", "No price · stays Not printed" and "Can't generate".
- For a batch, the footer always shows "{n} products will be marked Printed", even when n is 0. This is Pricing's own wording, plural included. Say "hide the zero line" to drop it at 0.
- A single product that can't be generated gets no footer sentence, because its line already says why.
- A reprint without a known print date reads "Label status stays Printed. Reprinting keeps it marked Printed." with no date.
- Pricing's extra notes on a product are shown in small text under its line.
- Until the dialog shows the result, its title reads "Generate label" for one product and "Generate labels for N" for a batch.
- A dialog opened with one product uses the one-product wording. A batch keeps the batch wording even if Pricing doesn't know one of its products.

**The preview picture**
- The preview is a picture of the PDF's page, made on this Mac and shown on white. If it can't be made, the dialog says so and Generate still works.
- While the picture loads, an empty tile of the label's shape shows. A product with no template gets a square one.
- ‹ › step through the products in the order they are listed on screen.
- A product's picture is asked for once per dialog.

**Generating**
- A dropped connection, or Pricing being too busy to make labels, is retried twice at once with the same request. After that the dialog offers **Try again**, which continues where it stopped. Labels already made are never made again.
- **Try again** after a failed Generate appears only for "can't be reached" and "renderer busy". Any other refusal (for example a key that isn't allowed) offers only **Close**, which shows what was generated so far. Say "always offer Try again" to change it.
- Leaving a batch after a failed part shows what was generated so far. The rest is listed as not generated and stays selected.
- A new check can't start while labels are being generated.
- Run requests are not kept across a quit. If the app quits during Generate, generating again makes a new run, and products already marked come back as reprints.
- Pressing Generate again on a finished dialog shows the same result, even if Pricing is now unreachable.
- A reprinted product keeps the print date the check showed.

**Saving and downloading**
- PDFs always land in Documents › Session Manager Labels. A name already taken gets " (2)", " (3)" …. A file name from Pricing is cleaned so it can't land anywhere else.
- If Preview can't be opened, the PDF is still saved. The dialog says "Saved, but Preview couldn't open it:" with the file's path, and stays open until you close it.
- macOS asks once to let Session Manager use the Documents folder, the first time a PDF is saved.
- A download that fails while saving ("Couldn't save the PDF: …") or because Pricing can't be reached offers **Try again**. An expired file only says to generate again.
- After a Cancel that followed a failed batch, the files of the parts that finished can still be downloaded.

**After a batch**
- After a batch, the Labels box keeps ticked only the products that failed or were not generated, and asks Pricing for the page again.
- Closing a dialog opened from a single Labels row's ⋯ also refreshes the box after a run.
- While a label dialog is open, Escape doesn't clear the Labels box selection.

## Not yet
- Choosing a label template from Session Manager. Pricing picks it.
- Printing straight to a printer. You print from Preview.
- Making labels in bulk from Open or Published. Batches come from the Labels box.
- A history of past label runs.
- If you remove or replace your Pricing key while the Labels box is open, its rows stay until you leave and re-enter the box. This was noted earlier and is still open.
- No real Pricing or Shopify call was made by any check in this phase, and no real label was generated. Your first **Generate PDF** (steps 2 to 8) is the first real run.

## Needs you
Nothing.
