---
phase: P12_shopify_adapter
title: Shopify adapter — grounding probes, GraphQL client, capability methods, type map
difficulty: high
projection: done (r0 2026-10-26, amended)
depends_on: [P10_attributes]
---
# P12 — Shopify adapter: grounding probes, GraphQL client, capability methods, type map

> **Amended by the orchestrator 2026-10-26** from the round-0 projection
> (`handoffs/reviewer/handoff_P12_shopify_adapter_projection_r0.md`, ledger L1–L25). The recommended routing was
> taken for every entry. Shopify-behaviour claims the projection marked *believed* are now explicit probe questions
> in task 0. The adapter is wired into the app by P13, not here (L10).

## Goal
Implement `CommerceAdapter` for Shopify (Admin GraphQL, version pinned in settings, default `2026-10`) behind an
`HttpTransport` seam. Before any adapter code, **ground every Shopify-owned shape** the publish contracts depend on
(SHOPIFY-VERIFY-0…7) against the owner's test store (task 0), authenticated with the client credentials grant
(09A MC-23). Record the answers in a new evidence
doc plus sanitized fixtures. Methods: `item_search` (exact SKU), `item_get`, `item_create` (DRAFT + default-variant SKU
and barcode, the SV-7 shape), `item_set_identity` (default-variant SKU/barcode diff write + read-back, 05B MC-22),
`attributes_schema` (type map → core definitions + links), `attributes_read`, `attributes_write` (batched), the media
primitives (`staged_uploads_create`, `upload`, `product_create_media`, `poll_media_ready`, `product_reorder_media`,
`poll_job`, `list_media`, `product_delete_media`), `file_create`/`poll_file`, title/description/tags writes, and the access-token handling of 09A MC-23
(`ClientCredentialsAuth`). Every mutating method takes `&ConfirmedRun`. P12 owns the single `ConfirmedRun` constructor (L8). **Not in this phase:** the
publish step machine, reconciliation or write-set logic (P13, P14). Also not here: wiring the adapter into `App`,
persisting the schema, and search integration (P13 task 0, L10). The adapter is primitives plus mapping, with no UI.

## Precondition (orchestrator checks before launch)
`~/.psm_shopify.env` holds `SHOPIFY_SHOP`, `SHOPIFY_CLIENT_ID` and `SHOPIFY_CLIENT_SECRET` (a Dev Dashboard app installed
on the store, same organization; 09A MC-23), and the orchestrator has confirmed that a client credentials token
request succeeds, without printing the token. The session loads the file with
`set -a; . ~/.psm_shopify.env; set +a` before task 0 and never copies it or its values anywhere. The app needs these
access scopes (L20): `read_products`, `write_products`, `read_files`, `write_files`, and metafield-definition
read/write (covered by the product scopes for product-owner definitions; the probe records the `scope` the token
response grants). Without them the session reports `STATE: BLOCKED` at task 0 and does nothing else.
**Exception:** if `docs/intention/evidence/shopify_admin.md` already exists (an earlier attempt ran the probe), the
probe is **not** re-run. The session continues from the evidence and fixtures and needs no credentials (L14).

**Probe store: the owner's test store (`SHOPIFY_SHOP`), owner 2026-10-26; every probe step (reads and writes) runs
there.** The real store comes later by changing `shop_domain`, `client_id` and `client_secret` in settings only
(09A MC-23), so nothing store-specific may be hard-coded: keys, definitions and ids always come from the configured
store, and fixtures are the test store's captures, sanitized.

## Read first
master plan §3 (secrets), §6.1 (commerce module), §6.4 (`CommerceAdapter`, `HttpTransport`), §6.5 (`item_links`,
`photo_links`, `attribute_definition_links`), §7.5 (`FakeTransport`) · book `01` (H1, H4, H7), `14`, `09` (all), `08`
§Schema · contracts `07A` MC-10 (mutation token), MC-11 (prepare search rule, upload filename, adoption by stem),
MC-12 (reorder call shape, step 3 delete), MC-13 (type map, writability, write batching, description HTML, tags
merge **payload** only), MC-14 (file upload role), **§Grounding owed (SHOPIFY-VERIFY-1…7)**; `05A` MC-9; `05B`
MC-22 (identity write) · **`09A` MC-23 (credentials, access token, error messages), MC-24 (configured
condition-photos field)** · `claude_design/reference/SHOPIFY_PRODUCT_METAFIELDS.md` (the 68 definitions the schema
fixture must reproduce) · evidence `docs/intention/evidence/shopify_admin.md`, **written by this phase's task 0**.

## Files expected to change
New:
- `docs/intention/evidence/shopify_admin.md`
- `crates/integrations/src/commerce/{mod.rs, shopify/{mod.rs, adapter.rs, graphql.rs (queries/mutations as const
  strings), transport.rs (trait + ReqwestTransport), auth.rs (`ClientCredentialsAuth`), types.rs (response structs), type_map.rs, errors.rs, fixtures/*.json}}`
- `crates/integrations/examples/shopify_probe.rs` (the task 0 tool)
- `crates/core/src/ports/commerce.rs` (the `CommerceAdapter` trait, which uses `ConfirmedRun` from `domain/publish.rs`)
- `crates/core/src/domain/publish.rs` (`ConfirmedRun`, `RemoteItem`, `RemoteMedia`, `RemoteIdentity`, `DefinitionLink`,
  `WriteValue`)
- `crates/testkit/src/{fake_transport.rs, fixtures/shopify.rs (loader)}`
- `crates/testkit/tests/p12_shopify_adapter.rs`

Changed:
- `crates/core/src/{error.rs (`AttributeVerification{keys}`, `NotConfirmed`, `ShopNotPermitted`, `AppNotInstalled`,
  `CredentialsRejected`), ports/mod.rs, domain/mod.rs, ports/store.rs
  (`confirmed_run`)}`
- `crates/infra/src/sqlite/store.rs` (`confirmed_run` impl)
- `crates/infra/src/settings_file.rs`: a typed parse of the existing `integration: Option<Value>` key
  (`{"kind":"shopify","shop_domain","api_version","client_id","client_secret","condition_photos_key"}`; an absent
  `condition_photos_key` reads as `"custom.damage_report"`; 09A MC-23/MC-24)
- `crates/integrations/{Cargo.toml (reqwest), src/lib.rs (module export only; no `App` field, L10)}`
- `Cargo.lock`
- `crates/testkit/src/{lib.rs, fixtures/mod.rs, test_app.rs (`confirmed_run()` helper)}`

## Core mechanisms (unit tests allowed)
- `type_map::to_core(platform_type, validations, access) -> (CoreType, writable)`, `type_map::to_platform_string(core_type,
  canonical) -> String` and `type_map::from_platform_string(core_type, platform_type, s) -> Json` (MC-13 table; the
  platform→core direction maps `currency_code`→`currency`, `scale_min/scale_max`→`min/max`, the rich-text tree →
  paragraphs, and a list JSON string → array; L5).
- `adapter::upload_filename(folder_key, uid, revision) -> String` (MC-11 / MC-21).
- `adapter::description_html(paragraphs) -> String` (MC-13 escaping).

## Types (defined once in `crates/core/src/domain/publish.rs`; L8, L11, L3, L22)
- `ConfirmedRun`: private fields (`run_id`, `confirmed_at`). There is **exactly one** constructor, public:
  `ConfirmedRun::from_persisted(store: &dyn Store, run_id) -> Result<ConfirmedRun>`. It returns `Err(not_confirmed)`
  unless the `publish_runs` row exists and has `confirmed_at`. There is no other constructor, test-only or
  feature-gated. P13 reuses it.
- `Store::confirmed_run(run_id) -> Result<Option<i64>>`: the row's `confirmed_at` when the row exists and is
  confirmed; otherwise `None`.
- `RemoteItem{external_id, title, sku: Option<String>, barcode: Option<String>, thumbnail_url: Option<String>, media:
  Vec<RemoteMedia>}`. `item_search` may leave `media` empty; `item_get` fills it.
- `RemoteMedia{id, filename, status, position}`.
- `RemoteIdentity{sku, barcode: Option<String>}`.
- `DefinitionLink{platform_type, platform_meta: Json}`: opaque to core (namespace, access and other platform data
  live in `platform_meta`).
- `WriteValue{key, link: DefinitionLink, value: Json}` (canonical value).

## Ordered tasks
0. **Grounding probe**. Run `cargo run -p psm-integrations --example shopify_probe` with env `SHOPIFY_SHOP`,
   `SHOPIFY_CLIENT_ID`, `SHOPIFY_CLIENT_SECRET` (loaded from `~/.psm_shopify.env`) and `SHOPIFY_API_VERSION=2026-10`. **Skip this task
   entirely if `shopify_admin.md` exists.** The probe works against the probe store (see Precondition). It records
   every created id in a cleanup ledger *before* the next step. Each SHOPIFY-VERIFY id gets its own section in
   `shopify_admin.md` with request, response and conclusion. The questions it must answer:
   - **SV-0 (access token; 09A MC-23).** The client credentials request
     (`POST https://<shop>/admin/oauth/access_token`, form or JSON body, whichever Shopify accepts) and the exact
     response fields: token field name, lifetime field (`expires_in` or other) and its value, granted `scope`.
     Then the same request with a deliberately wrong secret: record status and body (the `credentials_rejected`
     fixture). Then one GraphQL call with a deliberately wrong token: record the status (expected 401) and body.
     The `shop_not_permitted` / `app_not_installed` replies are not reproducible on the owner's setup; their
     fixtures are declared synthetic in the SV-0 response shape, using the error strings of Shopify's
     client-credentials documentation.
   - **SV-1 (types, access).** Read all `metafieldDefinitions` (paginate by cursor), including each definition's
     `access`. Create one scratch definition `psm_probe_<type>` for each MC-13 type the shop lacks (expected:
     `number_decimal`, `weight`, `volume`, `boolean`, `date`, `date_time`, `color`, `money`, single
     `product_reference`). Set one value of every MC-13 type on the probe product and read each value string back.
     Record the unit codes Shopify uses in `validations` for dimension, weight and volume. Attempt one write per
     foreign namespace (`reviews.*`, `shopify--discovery--*`, `globo--filter--*`, `app.*`) on the probe product and
     record whether it is accepted (L7).
   - **SV-7 (create, identity; L1).** The create mutation that sets status DRAFT plus the default variant's SKU and
     barcode **in one call**, if one exists (expected: `productSet` with `synchronous: true`). Also the identity-write
     mutation and its read-back (expected: `productVariantsBulkUpdate`, SKU on `inventoryItem.sku`, barcode on the
     variant).
   - **SV-6 (search; L1).** Search by the probe SKU immediately after create, then at +5 s and +30 s, and record
     whether it is found each time. Also search in the other letter case.
   - **SV-2 (upload, media; L16, L17, L18, L15).** `stagedUploadsCreate` with `httpMethod: PUT` and `resource: IMAGE`:
     record the target and whether `parameters` are sent as headers. Upload **three** tiny JPEGs: two under one
     filename (to see the uniquifier with `duplicateResolutionMode: APPEND_UUID` stated explicitly) and one with a
     different filename. Record the 2026-10 media-create mutation, where the new media id is returned, and whether
     `filename` is readable on media and on files.
   - **SV-3 (files).** `fileCreate` then poll to READY; record the returned id type.
   - **SV-4 (reorder, delete; L9).** Reorder all three media and record the position base. Delete one media, then
     delete the same id again and record the not-found reply.
   - **SV-5 (metafieldsSet).** Record the per-call entry limit `L`. Send one deliberately invalid value in a 3-entry
     call, then read back the other two: is the call atomic? How do `userErrors` point at entries (`elementIndex` /
     `field`)? (L4)
   Afterwards, delete the probe product, files and scratch definitions. On any failure, print the leftover ledger;
   the session records it in the Review log and the handoff.
   **Fixtures** go to `fixtures/` as sanitized request/response pairs. **Redaction list (L20):** the shop domain
   becomes `example.myshopify.com`. The client ID, the client secret, every access token, every signed-URL query
   string and every staged-upload `parameters` value become placeholders, keeping their shape. Resource/CDN URLs keep
   their shape with the shop path replaced. The secret and the tokens never appear anywhere, including terminal
   output pasted into the handoff.
   **Stop or absorb (L14):** the implementer **absorbs** literal values of Shopify-owned shapes that rows cite by SV
   id (L, position base, uniquifier text, poll shape, value strings, unit codes) and records each one under
   Assumptions. The implementer **stops** (`STATE: BLOCKED`, after a `CHECKPOINT` commit of the evidence doc and
   fixtures only) when an answer breaks a contract mechanism. Stop cases:
   - media/file filename unreadable (MC-11 adoption);
   - no atomic create, or the SKU search misses the product at +5 s (MC-11 prepare, M18);
   - foreign-namespace writes are rejected *and* access is not visible in the definition (MC-13 writability);
   - partial-failure semantics other than "atomic per call" or "per-entry".
   The orchestrator then amends 07A and relaunches. On relaunch the probe is not re-run.
1. `HttpTransport` trait: `post_graphql(access_token, query, variables) -> Result<HttpReply{status, body: Json}>`,
   `post_form(url, fields) -> Result<HttpReply>` (the token request), `put_bytes(url, headers, bytes)` for staged
   uploads; plus `ReqwestTransport`: timeouts 30 s; the access token goes in the `X-Shopify-Access-Token` header and
   is never logged. Delegated runtime choice: see Delegated choices.
   `ClientCredentialsAuth` (09A MC-23) over the transport and an injectable `Clock`: fetches a token on first use
   with `grant_type=client_credentials`, `client_id`, `client_secret` to `https://<shop>/admin/oauth/access_token`;
   keeps it in memory; fetches again when `now ≥ obtained_at + lifetime − 60 s` (lifetime from the SV-0 field; if
   SV-0 shows none, the token is kept until a 401, recorded under Assumptions). Every GraphQL call goes through it:
   on a 401 it fetches a new token and retries that call **once**; a second 401 → `Err(credentials_rejected)`. Grant
   refusals map to `shop_not_permitted`, `app_not_installed`, otherwise `credentials_rejected`. Neither the secret
   nor a token appears in any error's `Display`/`Debug`.
2. `ShopifyAdapter: CommerceAdapter` over the transport, built from `ShopifyConfig{shop_domain, api_version,
   client_id, client_secret, condition_photos_key}` plus a clock and the polling config. `capabilities()` = the nine of
   ch09.
3. Type map, both directions, from SV-1. Signature:
   `attributes_schema() -> Result<Vec<(Definition, DefinitionLink)>>`. The adapter only returns the pairs; it holds
   no Store, and persistence belongs to P13 task 0 (L22). `writable` = type-writable ∧ the definition's `access`
   permits writes by this app (L7). `group` is `condition` for `custom.condition_*`, the configured `condition_photos_key` and
   `custom.condition_report`; every other definition is `details`. **Only** the definition whose `namespace.key`
   equals `condition_photos_key` and whose type is `list.file_reference` maps to `media_list` (09A MC-24); every
   other `file_reference`/`list.file_reference` definition, and the configured key if it has another type, maps to
   `reference` (read-only). A missing or mistyped key is not an error here: the return simply has no `media_list`
   definition (health derives `condition_field_missing` from the stored definitions in P16). The adapter emits **no** `product`-group
   definitions: title, description and tags stay the core built-ins (L21).
   `attributes_read(external_id) -> Result<Vec<(key, Json)>>` uses `from_platform_string`. A rich-text value that
   cannot be represented is returned verbatim and flagged, so the service stores it `valid = false` (MC-9).
   `attributes_write(run, external_id, &[WriteValue])` formats each value with `to_platform_string`. For a
   `media_list` key it sends the file ids, not the canonical `{"excluded":…}` (C7(b)).
4. Media and file primitives with the request shapes of SV-2/3/4. `list_media(product) -> Vec<RemoteMedia>`;
   `product_delete_media(run, product, media_ids)`, where not-found counts as deleted (MC-12 step 3, L9);
   `metafields_set` batched at the SV-5 limit with the partial-failure mapping of C5(r). Polling interval and
   attempt cap are injectable; tests use 0 ms. Exhaustion → `Error::Timeout` (L19).
   `item_set_identity(run, external_id, sku: &str, barcode: Option<&str>) -> Result<RemoteIdentity>`: reads the
   default variant, writes only the fields that differ, and never sends a `None` barcode. It returns the read-back
   (L3).
5. `FakeTransport`: fixture replay keyed by operation name + variables digest, with **per-key response sequences**
   (for polls). An unmatched request fails the test. It records every request (`query`, `variables` and the access-token argument of
   `post_graphql`; `post_form` URL and fields; `put_bytes` URL and headers).
6. `TestApp::confirmed_run()` inserts a confirmed `publish_runs` row (direct SQL on the test's data dir is fine)
   and returns `ConfirmedRun::from_persisted(...)`. Its callers are every mutating row.
7. Tests: every row below runs against `FakeTransport`.

## Criteria (9 criteria, 68 rows; 0 `[rule]` rows). Fixture cells cite `shopify_admin.md` sections (SV-n) and fixture files.
| Row | Boundary | Input (fixture) | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | `item_search(sku "K-1")` | SV-6 fixture: 0 variants match | `[]` | MC-11 (prepare) |
| C1(b) | `item_search` | 1 match (SV-6) | one `RemoteItem{external_id, title, sku: Some("K-1"), barcode, thumbnail_url, media: []}` with the fixture's values | MC-11 |
| C1(c) | `item_search` | 2 products carry the SKU (declared synthetic: the SV-6 response shape with a second node) | two items (P13 decides the failure) | MC-11 |
| C1(d) | `item_search("k-1")` | a product whose variant SKU is `K-1`: the SV-6 other-case capture if search is case-insensitive, otherwise a declared synthetic guard | `[]`; comparison is byte-exact after NFC | MC-11 |
| C2(a) | `item_get(id)` | SV-2 fixture product with 3 media | `RemoteItem` with media `[RemoteMedia{id, filename, READY, position}]` × 3 in gallery order | M19 (inputs `current`) |
| C3(x) | `item_set_identity(run, id, "K-2", None)` | SV-7 fixture: variant SKU `K-1`, barcode `123` | exactly one recorded mutation, whose variables carry `sku "K-2"` and no barcode; returns `RemoteIdentity{sku: "K-2", barcode: Some("123")}` | MC-22 (05B) |
| C3(a) | `item_create(run, sku, barcode, title)` | SV-7 create fixture | returns the new external id; the recorded request(s) are the SV-7 shape; the fixture read-back shows `status DRAFT` and default-variant `sku`/`barcode` equal to the inputs | M18 |
| C3(b) | `ConfirmedRun::from_persisted(store, id)` | `publish_runs` row with `confirmed_at` null; and an id with no row | `Err(not_confirmed)` both times; a confirmed row → `Ok` | M17 (MC-10) |
| C4(a–t) | `attributes_schema()` return value (20 rows) | the captured SV-1 `metafieldDefinitions` response (68) **plus** the captured read-back of the `psm_probe_*` scratch definitions (both captured, not hand-written). (a) `single_line_text_field` with choices → `choice` · (b) without → `text` · (c) `list.single_line_text_field` with choices → `multi_choice` · (d) without → `tags` · (e) `multi_line_text_field` → `textarea` · (f) `rich_text_field` → `rich_text` · (g) `number_integer` → `integer` · (h) `number_decimal` → `decimal` · (i) `dimension` → `dimension` · (j) `weight` → `weight` · (k) `volume` → `dimension` · (l) `boolean` · (m) `date` · (n) `date_time` → `reference` · (o) `color` · (p) `rating` · (q) `money` · (r) `url` · (s) `list.file_reference` images with key = configured `condition_photos_key` (`custom.damage_report`) → `media_list` · (t) `file_reference`, `metaobject_reference`, `product_reference`, `list.metaobject_reference`, `list.product_reference` → `reference` | the named core type. `writable = true` for (a)–(s) except (n), and `false` for (n) and (t). Dimension/weight/volume `validations.units` = the unit codes SV-1 records for that Shopify type. Rating `validations = {"min": "<canonical>", "max": "<canonical>"}` from `scale_min`/`scale_max`. `custom.condition_cracks` choices are exactly the severity five. `custom.damage_report` (the configured key) group `condition`. No returned definition has group `product` | M20, MC-13 (type map), MC-24 |
| C4(u) | `attributes_schema()` | a definition whose SV-1 `access` forbids writes by this app (captured; if the shop has none, a declared synthetic edit of a captured definition's `access`) | that definition `writable = false`, whatever its type | MC-13 (writability, L7) |
| C4(v) | `attributes_schema()` with `condition_photos_key = custom.damage_report` | a second `list.file_reference` images definition (captured if the shop has one; otherwise a declared synthetic copy of the `custom.damage_report` definition under another key) | that definition is `reference`, `writable = false`; exactly one returned definition is `media_list` | MC-24, M30 |
| C4(w) | `attributes_schema()` with `condition_photos_key = custom.nope` | the SV-1 capture | no returned definition is `media_list`; `custom.damage_report` is `reference`, `writable = false`; `Ok` | MC-24, M30 |
| C5(a–p) | `attributes_read` → `canonicalize` → `to_platform_string` (16 rows: choice, text, multi_choice, tags, textarea, rich_text, integer, decimal, dimension, weight, boolean, date, color, rating, money, url) | SV-1 fixture value string `s` for that type | `c = canonicalize(read(s))` equals the canonical JSON stated in the row, **and** `canonicalize(read(to_platform_string(c))) == c`. For text, textarea, choice, url, color, boolean, date, integer and the list types, also `to_platform_string(c) == s` byte for byte | M16, M20 |
| C5(q) | `attributes_write(run, id, 30 values)` | SV-5 limit `L` | requests batched in ⌈30/L⌉ `metafieldsSet` calls; no recorded `query` or `variables` contains the access token or the client secret | MC-13 (batching) |
| C5(r) | `attributes_write` | SV-5 partial-failure fixture (`userErrors` pointing at two entries by `elementIndex`/`field`) | `Err(attribute_verification{keys: [the two]})`, keys mapped from the entry index; only keys of earlier, successful batches are reported written; no claim about other keys in the failing batch | MC-13 |
| C6(a) | `staged_uploads_create(run, …)` | — | request `filename == "<key>-<uid12>.jpg"`, `mimeType image/jpeg`, `resource IMAGE`, `httpMethod PUT` | MC-11 (upload filename) |
| C6(b) | `upload(run, staged_target, bytes)` | SV-2 fixture target | `put_bytes` URL = the fixture URL; its headers equal the fixture `parameters` as a name→value map | MC-11 |
| C6(c) | `product_create_media(run, product, resource_url, alt)` | SV-2 media-create fixture | returns the media id from where SV-2 says it comes back; request carries `originalSource = resource_url` | M18 |
| C6(d) | `poll_media_ready(media)` | fixture sequence PROCESSING, PROCESSING, READY; and a sequence ending FAILED | returns after the third poll; `FAILED` → `Err(Provider(<fixture message>))`; never READY within the cap → `Err(Timeout)` | M18 |
| C6(e) | `product_reorder_media(run, product, D)` | D of 3 ids (SV-4) | one request with a move for **every** id; `newPosition` = index in the base SV-4 records | M19 (MC-12 step 4) |
| C6(f) | `poll_job(job_id)` | fixture `done: false` then `true` | returns on the second poll | MC-12 |
| C6(g) | `list_media(product)` | SV-2 fixture with the two filenames SV-2 records for the duplicate upload (original and uniquified) | both returned with their exact filenames and positions | MC-11 (adoption by stem) |
| C6(h) | `product_delete_media(run, product, [id])` | SV-4 not-found reply fixture | `Ok(())` | MC-12 step 3 |
| C7(a) | `file_create(run, resource_url, filename)` then `poll_file` | SV-3 fixture PROCESSING → READY | returns the file id of the type the fixture shows; the request carries `duplicateResolutionMode: APPEND_UUID` | M21 |
| C7(b) | `attributes_write` for a `media_list` key | file ids `[f1, f2]` | `metafieldsSet` value is the JSON array `["f1","f2"]` as a string, type `list.file_reference` | M21 |
| C8(a) | `description_html` | `"a & b\n\nc < d \"q\""` | `<p>a &amp; b</p><p>c &lt; d &quot;q&quot;</p>` | MC-13 (description) |
| C8(b) | `update_tags(run, product, tags)` | `["a","b"]` | `productUpdate` request with `tags: ["a","b"]` (the merge itself is P14) | MC-13 (tags) |
| C9(a) | two `item_search` calls | SV-0 token fixture (`T1`) | exactly one recorded `post_form` to `https://example.myshopify.com/admin/oauth/access_token` with `grant_type=client_credentials`, the configured `client_id` and `client_secret`; both GraphQL calls carry `T1` | MC-23 |
| C9(b) | `item_search`, clock advanced to `lifetime − 59 s` after the fetch, `item_search` | token fixtures `T1`, then `T2` | a second token request before the second call; the second call carries `T2` | MC-23 |
| C9(c) | `item_search` | GraphQL replies 401 (SV-0 capture) then 200; and 401, 401 | one new token request, the call retried once, `Ok`; for 401, 401: `Err(credentials_rejected)` after exactly 2 GraphQL attempts | MC-23 |
| C9(d) | first call | token reply `shop_not_permitted` (declared synthetic, SV-0 shape) | `Err(shop_not_permitted)`; no GraphQL request sent | MC-23 |
| C9(e) | first call | token reply `app_not_installed` (declared synthetic, SV-0 shape) | `Err(app_not_installed)`; no GraphQL request sent | MC-23 |
| C9(f) | first call | SV-0 wrong-secret reply (captured) | `Err(credentials_rejected)`; no GraphQL request sent | MC-23 |
| C9(g) | the sequences of C9(a)–(f) plus C6(a)–(b) | client secret `"SECRET-XYZ"`, tokens `T1`, `T2` | no recorded GraphQL `query`/`variables`, no `put_bytes` URL or header, and no returned error's `Display` or `Debug` contains `SECRET-XYZ`, `T1` or `T2` (the token appears only as the `post_graphql` access-token argument) | M30, B3 (rule 8) |

Count: C1 4, C2 1, C3 3, C4 23, C5 18, C6 8, C7 2, C8 2, C9 7 = 68.
MC-10/M17 ("every mutating method takes `&ConfirmedRun`") is enforced by the signatures plus C3(b). P13 C2(a)
serves it at runtime.

## Delegated choices
- GraphQL client style (hand-written queries; no codegen).
- How fixtures are keyed in `FakeTransport` (it must support per-key sequences).
- Retry policy for HTTP 429/5xx: bounded, with the `Retry-After` header. A timed-out mutation is **not** retried
  blindly (MC-11).
- Probe JPEG bytes. Scratch definition names (prefix `psm_probe_`).
- **Transport runtime (L13):** the ports are sync, and `reqwest` is pinned without `blocking`. Either enable
  `blocking` and use the client only off the async runtime, or give `ReqwestTransport` a private runtime. The
  constraint: no panic when the transport is created, used or dropped from a Tauri async command. Record the chosen
  reqwest features in Assumptions.
- **Token header placement (L12):** `ReqwestTransport` puts the access token in `X-Shopify-Access-Token`. A loopback
  server may check it (rule 9), with no extra row. Token request encoding (form or JSON) follows SV-0.
- **Polling (L19):** interval and attempt cap, injectable.
- **Search breadth (L23):** P12 searches by exact SKU only (MC-11). Title/barcode search goes with the wiring in P13
  task 0, or is named under "Not yet".
- **Minor (L24):** cursor pagination of `metafieldDefinitions`. A single newline inside a `description_html`
  paragraph is kept as is. `capabilities()` has no row. `upload` takes `&ConfirmedRun` (recommended, for
  uniformity with MC-10). Title and description write methods have no row.
- **Rich text (L5):** how a non-representable rich-text value is flagged (recommended: returned verbatim with a flag
  so the service stores it `valid = false`).

## Manual try-out
1. Read `docs/intention/evidence/shopify_admin.md`: one section per SV-1…SV-7 with request, response and
   conclusion, plus the cleanup ledger showing the probe product, media, file and scratch definitions created and
   deleted.
2. In the Shopify admin of the test store, search for `PSM-PROBE`: nothing is found. Settings → Custom data →
   Products has no `psm_probe_*` definitions.
3. Run `cargo test -p psm-testkit --test p12_shopify_adapter`: all rows pass, with no network.
4. The app does not use the shop yet. Connecting it (Health, New-session search, the form's shop fields) comes
   with the next phase.

## Notes
- **Rule 17**: task 0 grounds every Shopify-owned shape in a fixture cell. If an SV answer differs from the
  contract's assumption, the absorb/stop rule of task 0 applies: absorb literals, stop on mechanism breaks. The fix
  is an `07A` amendment through the orchestrator, never bending the test.
- The probe **writes to the owner's test store once** (owner 2026-10-26): one draft product, three media, one file
  and scratch definitions, all deleted. The owner's credentials file is the authorization. Tests never call the network (rule 9).
- P13 note (L15): the adoption matcher is `stem + (SV-2 uniquifier)? + ".jpg"`, never a plain prefix, because
  MC-21 revision names (`-r<n>`) share the stem.
- Estimated size: ~22 files + fixtures, ~2 900 lines.

## Review log
- 2026-10-26 (orchestrator): projection r0 returned AMENDMENTS_REQUIRED
  (`handoffs/reviewer/handoff_P12_shopify_adapter_projection_r0.md`). Applied L1–L25 with the recommended routing:
  - Task 0: SV-7 and the SV-6 lag check; three uploads; scratch definitions per missing type; access plus
    foreign-namespace writes; the delete-media shape; redaction list; cleanup ledger; token scopes; the absorb/stop
    rule; the probe is not re-run when evidence exists.
  - Types: `ConfirmedRun::from_persisted` as the single public constructor; `Store::confirmed_run`; the
    `TestApp::confirmed_run()` helper; `RemoteItem`/`RemoteMedia`/`RemoteIdentity` defined.
  - Methods: `attributes_schema`/`attributes_write` signatures; no `product.*` definitions; `product_delete_media`;
    injectable polling.
  - Rows: C3(a), C3(x), C4 inputs/outcomes, C5 canonical round trip (+weight, money/url split), C5(q)(r),
    C6(a)(b)(c)(d)(g) and C7(a) rewritten; C3(b), C4(u), C6(h) added. 54 → 59 rows.
  - Scope: perimeter completed; the Manual try-out limited to what P12 delivers; App wiring, schema persistence and
    search integration moved to P13 task 0 (master plan amended).
  - Book: fold-backs in place in 07A (SV-6 lag, SV-7, MC-11 grounding note, MC-13 writability) and 09, changelog
    entry 17. The probe store depends on owner card 1.
- 2026-10-26 (orchestrator): owner answers folded in (book 09A, R32, changelog 18). Probe store = the owner's test
  store (card 1 closed; the real store later is a settings change). Credentials are a Dev Dashboard app's client ID
  and secret: `ClientCredentialsAuth` (in-memory token, refresh, one 401 retry, error mapping), SV-0 token probe,
  env from `~/.psm_shopify.env`, redaction of secret and tokens, settings parse with `client_id`, `client_secret`,
  `condition_photos_key`. Only the configured condition-photos key maps to `media_list` (09A MC-24). Rows: C4(v),
  C4(w) and criterion C9 (a)–(g) added; C4(s), C5(q) reworded. 59 → 68 rows, 8 → 9 criteria.
- 2026-10-26 (implementer, Claude Opus 5.5, implement r1): **IMPLEMENTED.**
  - **Task 0 ran** against the test store (API 2026-10). Evidence is in `docs/intention/evidence/shopify_admin.md`
    (SV-0…SV-7 plus a live smoke of the adapter's own documents); 70 sanitized fixtures. No stop case was hit:
    `productSet(synchronous: true)` creates DRAFT + SKU + barcode atomically; search finds the product 1 s after
    create (case-insensitive); filenames are readable from `image.url` once media is READY (uniquifier `_<uuid>`);
    `access.admin` is visible on every definition; `metafieldsSet` is atomic per call (L = 25, entries named by
    `field[1]`).
  - **Absorbed literals:** `expires_in` 86399 s; L = 25; reorder base 0; job `{done}`; uniquifier `_<uuid v4>`;
    unit codes upper-case names (`CENTIMETERS`, `KILOGRAMS`, `LITERS`…); `number_decimal` `12.50` → `12.5`; colour
    case kept; `FILE_DOES_NOT_EXIST` as the not-found reply.
  - **2026-10 shape changes absorbed:** `productCreateMedia` / `productDeleteMedia` do not exist. Media are created
    with `productUpdate(media:)` (new id = `media(first: 1, reverse: true)`) and removed from the product with
    `fileUpdate(referencesToRemove:)`. Variants carry `barcodes` (list), not `barcode`.
  - **Built:** `CommerceAdapter` port; `ConfirmedRun` (single constructor) + `Store::confirmed_run`; the
    `RemoteItem`/`RemoteMedia`/`RemoteIdentity`/`DefinitionLink`/`WriteValue`/`StagedTarget` types; five error
    variants; `HttpTransport` + `ReqwestTransport` (private worker thread with its own current-thread runtime;
    reqwest features `json`, `rustls`, no defaults, no `blocking`); `ClientCredentialsAuth`; the type map;
    `ShopifyAdapter`; the typed `integration` settings parse; `FakeTransport`, the fixture loader and
    `TestApp::confirmed_run()`. Tests: 68 rows + 5 core-mechanism unit tests.
  - **Fixture deviation (absorbed, flagged):** the test store has 66 definitions, not the real store's 68. The C4
    input is the test store's captured list plus 6 scratch definitions (72). The `custom.condition_cracks` clause is
    served by the scratch `custom.condition_psm_probe_cracks` (severity five). The real store's `condition_*` fields
    were not captured; they are read from the configured store at schema refresh (09A MC-23).
  - **Deviation:** `apps/desktop/src-tauri/src/commands/items.rs` gained five match arms for the new error variants.
    The match is exhaustive, so the build would not compile without them.
  - **Leftover on the test store:** the standard definition
    `shopify--discovery--product_recommendation.related_products_display`, auto-created by the SV-1
    foreign-namespace write. This app gets `ACCESS_DENIED` when deleting it. It has no values.
  - **Notes for P13/P14:** media still PROCESSING expose no filename (adopt only after READY); the files search
    index lags (do not rely on `files(query: filename:)` for recovery); Shopify reads `&quot;` back as `"` in
    `descriptionHtml`; re-check definition `access` on the real store at the switch (the foreign-namespace
    writes here are not representative).
