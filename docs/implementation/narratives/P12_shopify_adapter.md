# Phase P12_shopify_adapter: the connection to your Shopify store, built and tested

## What you can do now
The app now has its own Shopify connection. It logs in with your app's client ID and secret, keeps the access key in memory only, and renews it before it runs out. It can find a product by its exact SKU, create a draft product with SKU and barcode, change the SKU or barcode later, read the shop's fields and translate them into the app's field types, read and write field values, upload photos and put them in gallery order, remove photos from a product, and fill the damage-report field with condition photos. Before any of this was written, every Shopify behaviour it relies on was tried once on your test store, written down, and then cleaned up. The app does not use this connection yet: nothing on screen changes until the next phase connects it.

## Try it yourself
1. Run this phase's checks. They use the recorded Shopify replies, so no network or store is needed:
   ```
   export PATH="$HOME/.cargo/bin:$PATH"
   cd ~/Desktop/Developer/Photo-Session-Manager
   cargo test -p psm-testkit --test p12_shopify_adapter
   ```
   The last line should read `test result: ok. 68 passed`.
2. Read what Shopify actually did, one section per question asked, ending with the list of everything that was created and deleted:
   ```
   open docs/intention/evidence/shopify_admin.md
   ```
3. In your test store's Shopify admin, search products for `PSM-PROBE`: nothing is found. Under Settings → Custom data → Products there are no `psm_probe_…` fields. One field the test caused is still there, "Related products settings". It belongs to Shopify's Search & Discovery app, so this app may not delete it. It holds no values, and you can delete it yourself if you like.
4. Optional, and this writes to the test store: run the connection once for real. It creates one draft product, two photos and one file, then deletes them again:
   ```
   set -a; . ~/.psm_shopify.env; set +a
   cargo run -p psm-integrations --example shopify_probe -- adapter_smoke
   ```

## Choices made for you
- The test store has 66 product fields, not the 68 of your real store, so the checks use the test store's fields plus six test fields it created and later deleted. Your real store's fields are read when the app connects to it. Nothing to change.
- SKU search on Shopify ignores upper and lower case, so the app compares the SKU letter for letter itself: "k-1" never matches a product whose SKU is "K-1". To change this, say "SKU search may ignore case".
- When a photo leaves a product's gallery, it is taken off that product only. The image stays in your shop's Files, so nothing else that uses it breaks. To change this, say "delete removed photos from Files too".
- A shop field counts as writable only when Shopify says every app may write it. Fields that another app keeps for itself appear read-only. To change this, say "try writing other apps' fields".
- Shopify's own limits on a field, such as a maximum length or a number range, are not checked in the form. A value that breaks one is refused when you publish, and the message names the field. To change this, say "check the shop's field limits in the form".
- When some field values in a batch are refused, the app stops there and names exactly the refused fields. Shopify writes a batch of up to 25 values all or nothing, so earlier batches are saved and the failing batch is not. Nothing to change.
- A busy or overloaded shop is asked again up to three times. A change whose result is unknown, for example after a timeout, is never sent again blindly: the next attempt first checks what happened. Nothing to change.
- Waiting for a photo to finish processing checks once a second for up to two minutes. Nothing to change.
- Besides the one test pass the plan asked for, the session ran the finished connection once more against the test store, and it cleaned that up too. To forbid extra test writes in future phases, say "only the planned test writes on my store".

## Not yet
- The app does not use the shop yet. The health screen, finding an existing product when you start a session, and the shop's fields in the form come in the next phase.
- Publishing (sending photos, order and fields to a product) is built on top of this in the next two phases.
- Searching by title or barcode is not there; search is by exact SKU only.

## Needs you
Nothing.
