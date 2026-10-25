---
phase: P12_shopify_adapter
title: Shopify adapter — grounding probes, GraphQL client, capability methods, type map
difficulty: high
projection: yes
depends_on: [P10_attributes]
---
# P12 — Shopify adapter: grounding probes, GraphQL client, capability methods, type map

## Goal
Implement `CommerceAdapter` for Shopify (Admin GraphQL, version pinned in settings, default `2026-10`) behind an
`HttpTransport` seam, after **grounding every Shopify-owned shape** the publish contracts depend on
(SHOPIFY-VERIFY-1…6) against the owner's dev store with the dev-app token and recording the answers in a new
evidence doc plus sanitized fixtures. Methods: `item_search` (exact SKU), `item_get`, `item_create` (DRAFT),
`attributes_schema` (type map → core definitions), `attributes_read`, `attributes_write` (batched), the media
primitives (`staged_uploads_create`, `upload`, `product_create_media`, `poll_media_ready`, `product_reorder_media`,
`poll_job`, `list_media`), `file_create`/`poll_file`, title/description/tags writes. Every mutating method takes
`&ConfirmedRun`. **Not in this phase:** the publish step machine, reconciliation or write-set logic (P13, P14) — the
adapter is primitives plus mapping; no UI.

## Precondition (orchestrator checks before launch)
`SHOPIFY_TOKEN` and `SHOPIFY_SHOP` (e.g. `68c5b4-6a.myshopify.com`) available to the session for task 0. Without
them the session reports `STATE: BLOCKED` at task 0 and does nothing else.

## Read first
master plan §3 (secrets), §6.1 (commerce module), §6.4 (`CommerceAdapter`, `HttpTransport`), §6.5 (`item_links`,
`photo_links`, `attribute_definition_links`), §7.5 (`FakeTransport`) · book `01` (H1, H4, H7), `14`, `09` (all), `08`
§Schema · contracts `07A` MC-10 (mutation token), MC-11 (prepare search rule, upload filename, adoption by stem),
MC-12 (reorder call shape), MC-13 (type map, write batching, description HTML, tags merge **payload** only), MC-14
(file upload role), **§Grounding owed (SHOPIFY-VERIFY-1…6)**; `05A` MC-9 · `claude_design/reference/
SHOPIFY_PRODUCT_METAFIELDS.md` (the 68 definitions the schema fixture must reproduce) · evidence
`docs/intention/evidence/shopify_admin.md` — **written by this phase's task 0**.

## Files expected to change
`docs/intention/evidence/shopify_admin.md` (new), `crates/integrations/src/commerce/{mod.rs, shopify/{mod.rs, adapter.rs,
graphql.rs (queries/mutations as const strings), transport.rs (trait + ReqwestTransport), types.rs (response structs),
type_map.rs, errors.rs, fixtures/*.json}}` (new), `crates/integrations/examples/shopify_probe.rs` (new: task 0 tool),
`crates/core/src/ports/commerce.rs` (trait; `ConfirmedRun` type with a crate-private constructor used by P13),
`crates/core/src/domain/publish.rs` (`ConfirmedRun`, `RemoteMedia`, `RemoteItem`), `crates/infra/src/settings_file.rs`
(integration block read), `crates/testkit/src/{fake_transport.rs (new), fixtures/shopify.rs (loader)}`,
`crates/testkit/tests/p12_shopify_adapter.rs` (new).

## Core mechanisms (unit tests allowed)
- `type_map::to_core(platform_type, validations) -> (CoreType, writable)` and `type_map::to_platform_string(core_type, canonical) -> String` (MC-13 table).
- `adapter::upload_filename(folder_key, uid, revision) -> String` (MC-11 / MC-21).
- `adapter::description_html(paragraphs) -> String` (MC-13 escaping).

## Ordered tasks
0. **Grounding probe** (`cargo run -p psm-integrations --example shopify_probe`, env `SHOPIFY_TOKEN`, `SHOPIFY_SHOP`,
   `SHOPIFY_API_VERSION=2026-10`): creates one DRAFT product `PSM-PROBE-<yyyymmdd>` (SKU on the default variant),
   sets one metafield of every type in MC-13's table that the shop defines (plus a scratch definition for types it
   lacks, deleted afterwards), uploads the same tiny JPEG twice under one filename (SV-2 uniquifier), creates a file,
   reorders media (SV-4), reads everything back, searches by SKU in both cases (SV-6), then deletes the probe product,
   files and scratch definitions. Writes `shopify_admin.md` with one section per SHOPIFY-VERIFY id (request, response,
   conclusion) and the sanitized request/response pairs to `fixtures/`. Shop domain is replaced by
   `example.myshopify.com`; the token never appears. The run is recorded in the Review log with the probe product id.
1. `HttpTransport` trait (`post_graphql(query, variables) -> Json`, `put_bytes(url, headers, bytes)` for staged uploads)
   + `ReqwestTransport` (timeouts 30 s; `X-Shopify-Access-Token` header from settings; never logged).
2. `ShopifyAdapter: CommerceAdapter` over the transport; `capabilities()` = the nine of ch09.
3. Type map (both directions) from SV-1; `attributes_schema` writes core definitions + `attribute_definition_links`
   (platform type, namespace); `group` = `condition` for `custom.condition_*`, `custom.damage_report`,
   `custom.condition_report`; `product` for title/description/tags; else `details`.
4. Media/file primitives with the request shapes of SV-2/4; `list_media(product)` returns `(media_id, filename,
   status, position)`; `metafields_set` batched at the SV-5 limit with the partial-failure mapping.
5. `FakeTransport` (fixture replay keyed by operation name + variables digest; unmatched request → test failure).
6. Tests (every row below against `FakeTransport`).

## Criteria (8 criteria, 53 rows; 0 `[rule]` rows). Fixture cells cite `shopify_admin.md` sections (SV-n) and fixture files.
| Row | Boundary | Input (fixture) | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | `item_search(sku "K-1")` | SV-6 fixture: 0 variants match | `[]` | MC-11 (prepare) |
| C1(b) | `item_search` | 1 match | one `RemoteItem{external_id, sku: "K-1", title, has_media}` | MC-11 |
| C1(c) | `item_search` | 2 products carry the SKU | two items (P13 decides the failure) | MC-11 |
| C1(d) | `item_search("k-1")` | fixture returns a product whose variant SKU is `K-1` (SV-6 case behaviour) | `[]` — comparison is byte-exact after NFC | MC-11 |
| C2(a) | `item_get(id)` | SV-2 fixture product with 3 media | `RemoteItem` with media `[(id, filename, READY, position)]` in gallery order | M19 (inputs `current`) |
| C3(a) | `item_create(run, sku, barcode, title)` | fixture `productCreate` | returns the new external id; request has `status: DRAFT`, variant `sku`, `barcode` | M18 |
| C4(a–t) | `attributes_schema` (20 rows) | the 68-definition fixture (`SHOPIFY_PRODUCT_METAFIELDS.md` shapes, SV-1): (a) `single_line_text_field` with choices → `choice` · (b) without → `text` · (c) `list.single_line_text_field` with choices → `multi_choice` · (d) without → `tags` · (e) `multi_line_text_field` → `textarea` · (f) `rich_text_field` → `rich_text` · (g) `number_integer` → `integer` · (h) `number_decimal` → `decimal` · (i) `dimension` → `dimension` · (j) `weight` → `weight` · (k) `volume` → `dimension` · (l) `boolean` · (m) `date` · (n) `date_time` → `reference` · (o) `color` · (p) `rating` (min/max in validations) · (q) `money` · (r) `url` · (s) `list.file_reference` images → `media_list` · (t) `file_reference`, `metaobject_reference`, `product_reference`, `list.metaobject_reference`, `list.product_reference` → `reference`, `writable = false` | the named core type and writability; `custom.condition_cracks` choices are exactly the severity five; `custom.damage_report` group `condition` | M20, MC-13 (type map) |
| C5(a–n) | `attributes_read` → `canonicalize` → `to_platform_string` (14 rows: choice, text, multi_choice, tags, textarea, rich_text, integer, decimal, dimension, boolean, date, color, rating, money/url) | SV-1 fixture value strings | the produced string equals the fixture's value string byte for byte (identity) | M16, M20 |
| C5(o) | `attributes_write(run, 30 values)` | SV-5 limit `L` | requests batched in ceil(30/L) `metafieldsSet` calls; the token is in the header, never in the body | MC-13 (batching) |
| C5(p) | `attributes_write` | SV-5 partial-failure fixture (`userErrors` naming two keys) | `Err(attribute_verification{keys: [the two]})`; the other keys reported written | MC-13 |
| C6(a) | `staged_uploads_create(run, …)` | — | request `filename == "<key>-<uid12>.jpg"`, `mimeType image/jpeg`, `resource IMAGE` | MC-11 (upload filename) |
| C6(b) | `upload(staged_target, bytes)` | SV-2 fixture target | `put_bytes` called with the fixture's URL and parameters in order | MC-11 |
| C6(c) | `product_create_media(run, product, resource_url, alt)` | — | returns media id; request carries `originalSource = resource_url` | M18 |
| C6(d) | `poll_media_ready(media)` | fixture sequence PROCESSING, PROCESSING, READY | returns after the third poll; `FAILED` → `Err` with the fixture's message | M18 |
| C6(e) | `product_reorder_media(run, product, D)` | D of 3 ids | one request with a move for **every** id, `newPosition` = index as SV-4 says (0-based) | M19 (MC-12 step 4) |
| C6(f) | `poll_job(job_id)` | fixture `done: false` then `true` | returns on the second poll | MC-12 |
| C6(g) | `list_media(product)` | SV-2 fixture with `k-ab12cd34ef56.jpg` and `k-ab12cd34ef56-1.jpg` (uniquified) | both returned with their exact filenames and positions | MC-11 (adoption by stem) |
| C7(a) | `file_create(run, resource_url, filename)` then `poll_file` | fixture PROCESSING → READY | returns the file id (`gid://shopify/MediaImage/…` or `GenericFile` as the fixture shows) | M21 |
| C7(b) | `attributes_write` for a `media_list` key | file ids `[f1, f2]` | `metafieldsSet` value is the JSON array `["f1","f2"]` as a string, type `list.file_reference` | M21 |
| C8(a) | `description_html` | `"a & b\n\nc < d \"q\""` | `<p>a &amp; b</p><p>c &lt; d &quot;q&quot;</p>` | MC-13 (description) |
| C8(b) | `update_tags(run, product, tags)` | `["a","b"]` | `productUpdate` request with `tags: ["a","b"]` (the merge itself is P14) | MC-13 (tags) |

## Delegated choices
GraphQL client style (hand-written queries; no codegen), how fixtures are keyed in `FakeTransport`, retry policy for
HTTP 429/5xx (bounded, with the `Retry-After` header; a timed-out mutation is **not** retried blindly — MC-11),
probe JPEG bytes, how the scratch definitions are named (prefix `psm_probe_`).

## Manual try-out
1. Put the token and shop domain in `settings.json` (`integration: {"kind":"shopify","shop_domain":"…","api_version":"2026-10","token":"…"}`).
2. Launch; Health shows "Shopify: token valid". In New session, type a SKU that exists in the shop → the product
   appears with its thumbnail. Open the form: Details and Condition now list the shop's fields.
3. Nothing is written to Shopify in this phase (the probe ran once during the session; its product was deleted).

## Notes
- **Rule 17**: every Shopify-owned shape in a fixture cell is grounded by task 0; a row whose SV answer turns out
  different from the contract's assumption (e.g. `productReorderMedia` 1-based) is resolved by amending `07A` through
  the orchestrator (fold-back), not by bending the test.
- The probe **writes to the owner's dev store once** (one draft product, two media, one file, scratch definitions —
  all deleted). The dev store is the owner's; the token is their authorization. Tests never call the network (rule 9).
- Estimated size: ~16 files + fixtures, ~2 600 lines.

## Review log
(empty)
