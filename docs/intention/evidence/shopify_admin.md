# Shopify Admin API — grounding evidence (SHOPIFY-VERIFY-0…7)

Written by P12 task 0, 2026-10-26 (implementer, Claude Opus 5.5). Tool:
`cargo run -p psm-integrations --example shopify_probe -- <step>` (env `SHOPIFY_SHOP`, `SHOPIFY_CLIENT_ID`,
`SHOPIFY_CLIENT_SECRET` from `~/.psm_shopify.env`, `SHOPIFY_API_VERSION=2026-10`).

- **Store:** the owner's **test store** (owner 2026-10-26). In this document and in every fixture its domain
  is written `example.myshopify.com`. Shop currency USD.
- **API version:** `2026-10`, at `https://<shop>/admin/api/2026-10/graphql.json`. The schema was read by
  introspection over the same token. Facts that differ from older versions: `ProductVariant` has no
  `barcode` field (it has `barcodes { nodes { value type } }`, and input `barcodes: [{value}]`), and
  `productCreateMedia` and `productDeleteMedia` do not exist.
- **Fixtures:** `crates/integrations/src/commerce/shopify/fixtures/*.json`, 70 files. Each is
  `{"source", "request", "response": {"status", "body"}}`. `source` says either "captured: P12 probe capture <name>"
  or "synthetic: <the declared edit>". The GraphQL documents in `graphql.rs` are the exact strings the probe sent.
  The adapter therefore sends the documents whose replies the fixtures hold.
- **Redaction (L20).** The probe's `Redactor` runs before anything is printed or written. It writes the shop
  domain as `example.myshopify.com` and the shop handle as `example`. The client ID becomes `<CLIENT_ID>`, the client
  secret `<CLIENT_SECRET>` and every access token `<ACCESS_TOKEN>`. Every signed-URL query string becomes
  `?<SIGNED_QUERY>`, and every staged-upload `parameters[].value` except `content_type` becomes `<PARAM_VALUE:name>`.
  CDN paths `/s/files/1/<a>/<b>/<c>/` become `0000/0000/0000`. Curating the fixtures from `target/shopify_probe/captures`
  (not committed) also replaced the shop's numeric id in storage paths with zeros and applied the parameter
  placeholder to the PUT capture headers. A grep of the fixtures for the shop domain, the shop handle, the client ID,
  the secret, `shpat_` and the numeric shop id finds nothing.
- **Access scopes granted** (token reply `scope`): `write_products,write_metaobject_definitions,write_metaobjects,write_files`.
  Write implies read, so this covers L20's product, file and metafield-definition needs.
- **Web docs:** `WebFetch` and `WebSearch` were refused by the session's permissions. Shopify's own descriptions came from
  GraphQL introspection instead (`__type { enumValues { description } }`). The refused-grant codes `shop_not_permitted`
  and `app_not_installed` are the ones 09A MC-23 names.

---

## SV-0 — client credentials token (09A MC-23)

**Request.** `POST https://example.myshopify.com/admin/oauth/access_token` with
`Content-Type: application/x-www-form-urlencoded` and the fields `grant_type=client_credentials`, `client_id`,
`client_secret`. **Response 200:**
```json
{"access_token": "<ACCESS_TOKEN>", "expires_in": 86399, "scope": "write_products,write_metaobject_definitions,write_metaobjects,write_files"}
```
The token lives in field `access_token`, the lifetime in **`expires_in` = 86399 s** (24 h − 1 s), and the granted
scopes in `scope`. A JSON-shaped POST to the same endpoint was answered 400, so the form body is the encoding to use.

**Wrong secret.** Status **400**, body `{"error": "invalid_request", "error_description": "Missing or invalid client secret"}`.
Fixture `sv0_token_wrong_secret` (captured).

**GraphQL with a wrong token.** Status **401**, body
`{"errors": "[API] Invalid API key or access token (unrecognized login or wrong password)"}`. Note that `errors` is a
**string** here, not a list. Fixture `sv0_graphql_401` (captured).

**Not reproducible on the owner's setup:** `shop_not_permitted` and `app_not_installed`. Their fixtures
`sv0_token_shop_not_permitted` and `sv0_token_app_not_installed` are **declared synthetic**: the captured 400
`{error, error_description}` shape with `error` set to the 09A MC-23 code.

**Conclusion.** The token fetch, the 24 h reuse with a refresh 60 s before `expires_in`, the one retry after a 401,
and the error mapping of 09A MC-23 all hold as written.
`ClientCredentialsAuth` maps `error = shop_not_permitted` / `app_not_installed` to those errors, and any other
refusal to `credentials_rejected`.

## SV-1 — definitions, types, value strings, access, foreign namespaces

**Definitions.** `query Definitions($after)` reads `metafieldDefinitions(ownerType: PRODUCT, first: 50, after:)` with
`id namespace key name description type { name } validations { name value } access { admin }`, paginated by
`pageInfo { hasNextPage endCursor }`. The test store had **66** product definitions (it is not the real store, whose
68 definitions are listed in `claude_design/reference/SHOPIFY_PRODUCT_METAFIELDS.md`). Namespaces: `app`, `beyo`,
`beyo_derived`, `custom`, `test_data`. Every one has `access.admin = PUBLIC_READ_WRITE`.
Captured before the scratch definitions: `sv1_definitions_before_p1/p2`. Captured after them (this is the C4 input):
`sv1_definitions_p1/p2`, **72** definitions.

**Scratch definitions** (namespace `custom`, display name `psm_probe_*`; every one deleted afterwards):
`psm_probe_rich_text_field` (`rich_text_field`), `psm_probe_file_reference` (`file_reference`, Image+Video),
`psm_probe_product_reference`, `psm_probe_list_product_reference`, `psm_probe_list_file_reference`
(`list.file_reference`, Image; this is the second file list for C4(v)), and `condition_psm_probe_cracks`
(`single_line_text_field` with the severity choices `N/A, None, Minor, Moderate, Major`. The test store has no
`custom.condition_*` definition, so this one stands in for `custom.condition_cracks`). The store already had
every other MC-13 type: `number_decimal` (`beyo.fx_decimal`), `weight`, `volume`, `boolean`, `date`, `date_time`,
`color`, `money`, `rating` (`scale_min` 1.0, `scale_max` 5.0) and `url`. It also has `json` and `list.dimension`; both map to `reference`.

**`access.admin` values** (Shopify's own descriptions, from introspection):
`PRIVATE` "The merchant and other apps have no access"; `PUBLIC_READ` "…read-only access";
`PUBLIC_READ_WRITE` "The merchant and other apps have read and write access"; `MERCHANT_READ` "The merchant has
read-only access. No other apps have access"; `MERCHANT_READ_WRITE` "The merchant has read and write access. No other
apps have access". **Conclusion:** this app may write a definition only when `access.admin = PUBLIC_READ_WRITE`
(`type_map::access_permits_write`). Access is visible on every definition, so the MC-13 writability rule needs no
mechanism change.

**Value strings** (one value of every type, set with `metafieldsSet` and then read back with `MetafieldsRead`;
fixtures `sv1_values_set` and `sv1_values_readback`):

| Type (key) | Sent | Stored and read back |
|---|---|---|
| `single_line_text_field` with choices (`custom.extention_type`) | `Two Outside` | `Two Outside` |
| `single_line_text_field` (`beyo.fx_text`) | `Hej världen ✓` | same (UTF-8 kept) |
| `list.single_line_text_field` with choices (`custom.testing_listing`) | `["Completely restored","Light wear consistent with age and use"]` | same |
| `list.single_line_text_field` (`beyo.fx_list_text`) | `["teak","oak"]` | same |
| `multi_line_text_field` (`beyo.fx_text_long`) | `line one\nline two\n\nsecond paragraph` | same |
| `rich_text_field` | `{"type":"root","children":[{"type":"paragraph","children":[{"type":"text","value":…}]},…]}` | same tree, keys reordered (`children` before `type`) |
| `number_integer` | `42` | `42` |
| `number_decimal` | `12.50` | **`12.5`** (trailing zero dropped) |
| `dimension` | `{"value":45.5,"unit":"cm"}` | **`{"value":45.5,"unit":"CENTIMETERS"}`** |
| `weight` | `{"value":2.5,"unit":"kg"}` | `{"value":2.5,"unit":"KILOGRAMS"}` |
| `volume` | `{"value":1.5,"unit":"l"}` | `{"value":1.5,"unit":"LITERS"}` |
| `boolean` | `true` | `true` |
| `date` | `2026-10-26` | `2026-10-26` |
| `date_time` | `2026-10-26T10:00:00Z` | same (`reference` in the MVP) |
| `color` | `#A1B2C3`, later `#a1b2c3` | case **kept** as sent (`sv1_values_readback_uppercase_color`, `sv1_values_readback`) |
| `rating` | `{"value":"4.5","scale_min":"1.0","scale_max":"5.0"}` | `{"scale_min":"1.0","scale_max":"5.0","value":"4.5"}` (strings) |
| `money` | `{"amount":"12.50","currency_code":"USD"}` | same (`12.50` kept) |
| `url` | `https://example.com/item?x=1` | same |
| `list.file_reference` (`custom.damage_report`) | `["gid://shopify/MediaImage/…","gid://shopify/MediaImage/…"]` | same (SV-3, `sv3_damage_report_set`) |

**Unit codes.** One write per candidate unit; every candidate was accepted and stored as the upper-case name.
- dimension: `mm cm m in ft yd` → `MILLIMETERS CENTIMETERS METERS INCHES FEET YARDS`.
- weight: `oz lb g kg` → `OUNCES POUNDS GRAMS KILOGRAMS`.
- volume: `ml cl l m3 us_fl_oz us_pt us_qt us_gal imp_fl_oz imp_pt imp_qt imp_gal` → `MILLILITERS CENTILITERS LITERS CUBIC_METERS FLUID_OUNCES PINTS QUARTS GALLONS IMPERIAL_FLUID_OUNCES IMPERIAL_PINTS IMPERIAL_QUARTS IMPERIAL_GALLONS`.

The upper-case names are accepted on write as well. Definitions carry no `units` validation, so the core
`validations.units` is these stored codes (`type_map::{DIMENSION,WEIGHT,VOLUME}_UNITS`).

**Foreign namespaces (L7).** One write per namespace on the probe product. All four were **accepted**:
`reviews.rating`, `shopify--discovery--product_recommendation.related_products_display`,
`globo--filter--product_recommendation.psm_probe_text` and `app.item_location` (fixtures `sv1_foreign_*`).
On the test store none of these is an app-owned definition. Writing to `reviews.rating` and to the
`shopify--discovery--…` key made Shopify **auto-create its standard definitions** ("Product rating" and "Related products
settings", both `PUBLIC_READ_WRITE`). These writes are not representative of the real store's app-owned definitions. There the
`access` rule above decides. No stop case is involved: access is visible on every definition.

**Conclusion.** The MC-13 table holds. Literals absorbed: the unit codes; decimal normalisation (`12.50` → `12.5`;
canonical forms compare equal); colour case kept as sent; and the rating and money value shapes.

## SV-7 — create and identity (L1)

**Create.** `mutation ProductSet($input) { productSet(synchronous: true, input: $input) {…} }` with
`{title, status: DRAFT, productOptions: [{name: "Title", values: [{name: "Default Title"}]}],
variants: [{optionValues: [{optionName: "Title", name: "Default Title"}], sku, barcodes: [{value}]}]}`.
**One call** returns the product with `status DRAFT`, and its default variant carries `sku` (also on `inventoryItem.sku`) and
`barcodes.nodes[0].value` (fixtures `sv7_create` and `sv7_create_readback`). **The create is atomic.**

**Identity write.** `mutation ItemSetIdentity($productId, $variants) { productVariantsBulkUpdate(…) {
productVariants { id sku barcodes(first: 5) { nodes { value type } } inventoryItem { sku } } userErrors {…} } }`.
- SKU only, `{id, inventoryItem: {sku}}`: the SKU changed and the **barcode was kept** (`sv7_identity_sku_only` and its readback).
- Barcode only, `{id, barcodes: [{value}]}`: the barcode was replaced and the SKU kept (`sv7_identity_barcode_only` and its readback).
- The mutation payload's `productVariants[0]` is the post-write state. A separate `ProductRead` agrees with it.

## SV-6 — search by exact SKU (L1)

`query VariantSearch($query) { productVariants(first: 10, query: $query) { nodes { id sku barcodes product { id title
status featuredMedia { preview { image { url } } } } } } }` with `query = sku:"<SKU>"`.

| Search | Time after create | Found |
|---|---|---|
| `sku:"PSM-PROBE-SKU-1"` | 1.0 s | 1 (`sv6_search_immediate`) |
| same | 5.7 s | 1 (`sv6_search_5s`) |
| same | 30.6 s | 1 (`sv6_search_30s`) |
| `sku:"psm-probe-sku-1"` (other case) | — | **1**: search is **case-insensitive** (`sv6_search_other_case`) |
| `sku:PSM-PROBE-SKU-1` (unquoted) | — | 1 |
| `sku:"PSM-PROBE-SKU"` (prefix) | — | 0 (`sv6_search_prefix`) |
| `sku:"PSM-PROBE-NO-SUCH-SKU"` | — | 0 (`sv6_search_none`) |

**Conclusion.** No lag was observed (the product was found 1 s after create), so no stop case applies. Search is
case-insensitive, so the adapter filters the returned variants byte for byte after NFC (MC-11).

### SV-6 addendum — read-only title search (P13, 2026-10-26)

The test store accepted `products(first: 5, query: "Oak*")` with fields `id`, `title`,
`featuredMedia.preview.image.url`, and `variants(first: 1).nodes { sku, barcodes, inventoryItem.sku }`.
The read-only reply contained five nodes, all with a default variant SKU; three titles contained
the case-insensitive query text. No GraphQL errors were returned. The adapter asks for up to 20
nodes, filters the returned titles locally for the typed text, and merges them with exact-SKU
hits. The probe used `shopify_probe gql` with only the query above; it made no shop mutation.
Shopify's [products query](https://shopify.dev/docs/api/admin-graphql/2026-10/queries/products)
and [search syntax](https://shopify.dev/docs/api/usage/search-syntax) document the query and
prefix wildcard.

## SV-2 — staged upload, media create, filenames, uniquifier (L15–L18)

**Staged target.** `stagedUploadsCreate(input: [{resource: IMAGE, filename, mimeType: "image/jpeg", httpMethod: PUT,
fileSize}])` returns `stagedTargets[0] = {url, resourceUrl, parameters}`. The `url` is a signed Google Cloud Storage URL
(`https://shopify-staged-uploads.storage.googleapis.com/tmp/<shop>/files/<uuid>/<filename>?<SIGNED_QUERY>`).
`parameters` = `[{name: content_type, value: image/jpeg}, {name: acl, value: <PARAM_VALUE:acl>}]`. `resourceUrl` is
the same URL without its query.

**Upload.** `PUT <url>` with the parameters sent **as request headers** (`content_type`, `acl`) and the JPEG as the
body was answered **200** with an empty body (`sv2_put`).

**Media create.** In 2026-10 media are created with
`productUpdate(product: {id}, media: [{originalSource: <resourceUrl>, alt, mediaContentType: IMAGE}])`.
The new media id comes back in the payload's `product.media(first: 1, reverse: true)`, which is the last media in
gallery order (alias `newest`; it equalled the last node of the full list on all three creates). At creation the
status is `PROCESSING` (`sv2_media_create`). A first selection using `media(last: 1)` was refused with
"using last without before is not supported". The media it had created was put on the ledger and deleted.

**Polling.** `query MediaPoll($id) { node(id: $id) { … on MediaImage { status fileStatus image { url } originalSource { url }
mediaErrors {…} } } }`: `PROCESSING` (`image: null`, `originalSource.url: null`) then `READY`, within 1–2 s for a
64×64 JPEG.

**Filenames (MC-11 adoption).** There is no `filename` field on `MediaImage` or `File`. The filename is readable as
the last path segment of `image.url` (CDN URL `…/files/<filename>?v=…`) and of `originalSource.url`. It is readable
**once the media is processed**; while it is `PROCESSING` both URLs are null (`alt` is readable immediately). Three uploads:
`psm-probe-ab12cd34ef56.jpg`, then the same name again, then `psm-probe-0123456789ab.jpg`. The gallery
(`sv2_item_get`) shows:
`psm-probe-ab12cd34ef56.jpg`, **`psm-probe-ab12cd34ef56_4fffd4cd-17a1-449a-92f7-2e862798500b.jpg`**,
`psm-probe-0123456789ab.jpg`. **Uniquifier = `_` + a UUID v4 before the extension.** `files(query: "filename:<stem>")`
finds both the original and the uniquified media (`sv2_files_by_name`).

**Conclusion.** The filename is readable, so no stop case applies. Note for P13: a media still `PROCESSING` exposes no
filename yet, so the adoption matcher must wait for `READY` (`poll_media_ready`) before deciding there is no match.
The matcher form is `stem + ("_" + uuid)? + ".jpg"` (L15). It is never a plain prefix, because MC-21 revision names (`-r<n>`) share the stem.

## SV-3 — files (MC-14) and the content-hash question

`fileCreate(files: [{originalSource, filename, contentType: IMAGE, alt, duplicateResolutionMode: APPEND_UUID}])`
returns `files[0] = {id: "gid://shopify/MediaImage/…", fileStatus: UPLOADED}`. **The id type is `MediaImage`**.
`query FilePoll($id)` goes `PROCESSING` → `READY` after 2 polls. A second `fileCreate` with the same filename
gets `psm-probe-cafe00000001_b4c2fd63-9256-4524-a086-8c299e4f20c8.jpg`, the same uniquifier as SV-2.
`files(query: "filename:psm-probe-cafe00000001")` returned **no** nodes right after both files were READY
(`sv3_files_by_name`). The files search index lags, so it is not a reliable recovery lookup for files. Writing the
damage report `metafieldsSet(type: list.file_reference, value: "[\"gid…\",\"gid…\"]")` was accepted
(`sv3_damage_report_set`).
P13 recovery therefore pages through the unfiltered `files` connection and compares the returned
CDN filename. New uploads set `alt` to the requested filename so a still-processing file can be
recognized and waited on before the CDN filename appears. The pagination and `alt` fields are
documented by Shopify's [files query](https://shopify.dev/docs/api/admin-graphql/2026-10/queries/files)
and [FileCreateInput](https://shopify.dev/docs/api/admin-graphql/2026-10/input-objects/FileCreateInput).
P13 read-only test-store probe of `files(first: 5, after: null)` returned five nodes, `pageInfo`,
`fileStatus` and `alt` without GraphQL errors. No file was created for this probe.
**SHOPIFY-VERIFY-3 (content hash):** `MediaImage`, `File` and `GenericFile` expose no checksum or hash field
(`originalSource.fileSize` only). MC-11 (R-I10) by-filename adoption stands.

## SV-4 — reorder, job, delete (L9)

**Reorder.** `productReorderMedia(id, moves: [{id, newPosition: "<n>"}])` (`newPosition` is an `UnsignedInt64`,
sent as a string) returns `{job: {id, done: false}}`. `query JobPoll($id) { job(id:) { id done } }` gave `done: true` on
the first poll. Moving the three media to positions `0,1,2` in reverse order produced exactly the reversed gallery
(`sv4_reorder_readback`). **The position base is 0.** The 1-based variant (positions 1, 2, 3) did not produce the
order its indexes describe (`sv4_reorder_one_based*`).

**Delete.** In 2026-10, removing media from a product is
`fileUpdate(files: [{id: <media id>, referencesToRemove: [<product id>]}])`. It succeeds, and the media leaves
`product.media` (`sv4_delete_media`, `sv4_delete_readback`). Repeating it is also a success with no error
(`sv4_delete_media_again`). Once the file itself is deleted (`fileDelete(fileIds:)`, `sv4_file_delete`), both calls
reply with `userErrors: [{code: "FILE_DOES_NOT_EXIST", …}]`: `fileUpdate` in `sv4_delete_media_not_found` and a
second `fileDelete` in `sv4_file_delete_again`. **Conclusion:** `product_delete_media` uses `fileUpdate … referencesToRemove`
(removal from this product's gallery only), and treats `FILE_DOES_NOT_EXIST` as removed (MC-12 step 3).

## SV-5 — `metafieldsSet` limit and partial failure (L4)

- 26 entries: rejected as a whole, `userErrors: [{code: LESS_THAN_OR_EQUAL_TO, field: ["metafields"], message: "Exceeded the
  maximum metafields input limit of 25."}]` (`sv5_limit_26`). 25 entries: accepted (`sv5_limit_25`). **L = 25.**
- A 3-entry call with one invalid value (entry 1, `number_integer` = `not-a-number`) gave
  `userErrors: [{code: INVALID_VALUE, elementIndex: null, field: ["metafields","1","value"], message: "Value must be an integer."}]`
  and `metafields: []`. The read-back shows that **neither valid entry was written** (`sv5_partial_one_invalid_readback`).
  **The call is atomic.**
- A 3-entry call with entries 0 and 2 invalid gave two user errors, `field: ["metafields","0","value"]` and
  `["metafields","2","value"]` (`sv5_partial_two_invalid`). **Entries are identified by `field[1]` (the index as a
  string); `elementIndex` is null.**

**Conclusion.** Failure is atomic per call, which is one of the two allowed semantics, so no stop case applies. The
adapter maps `field[1]` (or `elementIndex` when present) to the batch's key and returns `attribute_verification{keys}`.
It stops at the first failing batch, so only earlier batches were written.

## Product field writes (no SV id; for P14)

`productUpdate(product: {id, tags: ["a","b"]})` set the tags (`tags_update`).
`productUpdate(product: {id, descriptionHtml: "<p>a &amp; b</p><p>c &lt; d &quot;q&quot;</p>"})` was accepted, but
Shopify **reads it back as `<p>a &amp; b</p><p>c &lt; d "q"</p>`**: it normalises `&quot;` to `"`
(`description_update`). Any P14 read-back comparison of the description must allow for that normalisation.

## Adapter live smoke (the adapter's own documents, after the probe)

`shopify_probe adapter_smoke` ran the real `ShopifyAdapter` over `ReqwestTransport`, with a `ConfirmedRun` from a
scratch database. In order:
- `item_search` returned 0 before the create and 1 after it (SKU and barcode equal to the inputs).
- `item_set_identity(SKU only)` returned the new SKU and kept the barcode. A no-change call sent no mutation.
- Two uploads (staged IMAGE target, PUT, `product_create_media`, `poll_media_ready`) became READY.
- `list_media` returned the filenames `psm-probe-smoke-5e5e5e5e0001.jpg` and `…0002.jpg`.
- After a reverse reorder and its job poll, the read-back gallery equalled D.
- `file_create` from an **IMAGE** staged target reached READY. This confirms the adapter's single staged resource
  type also works for files.
- `attributes_schema` returned 67 definitions, with `media_list` = `["custom.damage_report"]`.
- `attributes_write` of damage report, dimension, rating (scale sent as `"1"`/`"5"`, stored `"1.0"`/`"5.0"`), money
  (`"12.5"`, stored `"12.50"`) and colour returned `Ok`. `attributes_read` gave back the canonical-equal values.
- The title, description and tags writes returned `Ok`.
- `product_delete_media` returned `Ok` twice, leaving one media in the gallery.

## Cleanup ledger

Every id was recorded in `target/shopify_probe/ledger.json` before the next step. All of these were created and deleted:
- product `gid://shopify/Product/15444448280729`;
- media `…/MediaImage/46644766245017` (the media of the refused `last:` selection), `…46644767260825`, `…46644767457433`,
  `…46644767686809`;
- files `…/MediaImage/46644770275481` and `…46644770472089`;
- scratch definitions `…/MetafieldDefinition/479996936345`, `479996969113`, `479997001881`, `479997034649`, `479997067417` and `479997100185`;
- the auto-created standard definition `reviews.rating` (`…/MetafieldDefinition/479999590553`).

The adapter smoke created and deleted product `…/Product/15444456145049`, media `…46644866875545` and `…46644867661977`,
and file `…46644868546713`.

Afterwards: no variant matches `sku:PSM-PROBE*`, and no definition key contains `psm_probe`. The files search
returned 0, but SV-3 shows that this search lags, so the ledger's confirmed `fileDelete` replies are what count.

**Leftover (not deletable by this app).** The standard definition
`shopify--discovery--product_recommendation.related_products_display` ("Related products settings", choices
`ahead` / `only manual`) was auto-created by the SV-1 foreign-namespace write. `metafieldDefinitionDelete` answers
`ACCESS_DENIED` (it belongs to Shopify's Search & Discovery app). It has no values, and the owner may delete it in
Settings → Custom data → Products or leave it. The other auto-created standard definition, `reviews.rating`, was
deleted. After cleanup the store has 67 product definitions: the original 66 plus this one.
