# Phase L1_labels_provider_and_marker: every session shows whether its price label is printed

## What you can do now
Session Manager now asks Pricing about price labels and shows the answer as a small marker. On the Sessions page (Open, Published, All), each published session's row says **Label printed · 4 Nov**, **Missing price tag**, **No price yet**, or **Price changed: new label needed** (likewise when the SKU or barcode changed), exactly as Pricing says. The list shows up first and the markers appear a moment later. The same marker stands beside the session's title when you open it. When someone prints a label in Pricing, or a price change makes a label stale, the marker follows within the regular 10-second check, with no sound. Settings → Pricing gains a **Labels** line that says whether your key may read labels. A key without that permission keeps conversations working and simply shows no markers. The ⋯ menu of a session has a **Generate label** entry. In this phase it is always greyed out, with a hint saying why. The only thing about labels saved on this Mac is whether your key may read them.

## Try it yourself
You need a Pricing key that may read labels (Settings → Pricing → Replace, then paste it).
1. Start the app:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager-agentic/apps/desktop
   npm run tauri -- dev
   ```
2. Settings → Pricing → **Test connection**. Under the connection line it should say "Labels: allowed". With your old key (no label permission) it should say "Labels: this key can't read labels…", and conversations keep working.
3. Open **Sessions**. Rows of published products show a marker such as "Label printed · <date>", "Missing price tag" or "No price yet".
4. In Pricing, print a label for a product marked "Missing price tag". Within about ten seconds its marker should read "Label printed · <today>".
5. In Shopify, change the price of a product whose label is printed. Within about ten seconds its marker should read "Price changed: new label needed".
6. Open a published session. The same marker stands beside its title. Go back to the list, change something in Pricing, and open the same session again. The marker is asked for again and shows the new state.
7. Open the ⋯ menu of a session you never published. "Generate label" is greyed out, with "Publish first so Pricing can see it".
8. To run the automated checks for this phase:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager-agentic
   cargo test -p psm-testkit --test suite l1_labels_access::
   cargo test -p psm-testkit --test suite l1_labels_markers::
   cd apps/desktop && npx vitest run src/features/labels
   ```

## Choices made for you
- Each time you open a session, its marker is asked for again from Pricing, including when you reopen the session you just left. The first build missed this case: it reused the stored answer when you came back to the same session, so a change Pricing never announced would not show. The review caught this and it is fixed.
- When Pricing answers with an error that has no explanation attached, the app treats it as "Pricing answered in a way this version does not understand". Say "show Pricing's raw error" if you want more detail.
- Label file downloads refuse a few more disguised addresses than the plan required, for safety. Nothing you would use is refused.
- When a label run reports a result word the app does not know, that label counts as not made. It shows as failed, never as done.
- If you press Test connection while markers are loading, the late answer from before the test is dropped, so markers never mix two stores.
- Removing or replacing the key does not clear the markers already on screen. They update the next time the page loads. Say "clear the markers at once when the key goes" if you want that sooner (it is noted for the next labels phase).
- The ⋯ hint is a second line under "Generate label". The "printed" marker is neutral grey and the others are amber. On the session page, the marker sits with the title's other pills.
- Dates in the marker use English month names (4 Nov), on your Mac's calendar day.

## Not yet
- The Labels box with its list of products to label (next labels phase).
- Generating, previewing or printing labels from Session Manager. The ⋯ entry stays greyed out until then.
- No live Pricing or Shopify call was made in this phase's checks. Steps 2 to 6 above are your first real test.

## Needs you
Nothing.
