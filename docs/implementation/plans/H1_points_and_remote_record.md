---
phase: H1_points_and_remote_record
title: Shopify points and the record of remote pictures — the app reads Shopify's remaining points after every answer and its background work waits when they are low; it can ask for many picture addresses in one request, download a picture at full size, and tell "this picture is gone" from "the download failed"; it records which pictures Shopify holds for an archived product and for a shop product that was never photographed on this Mac
difficulty: high
projection: yes
depends_on: []
blocked_on: none
---
# H1 — Shopify points and the record of remote pictures

**Tracker:** NOT_STARTED

(This line is the milestone's tracker row. Sessions change only this line of this file. The gate for `depends_on` is the Tracker line of the plan named there.)

## Goal
Book `10H` §3 (vocabulary), §6 (the shop-only first-open read, Refresh from Shopify) and §8 (Shopify points) by contracts **MC-H5**, **MC-H3.1–3** and the adapter half of **MC-H1.4** (`docs/implementation/master_plan_archive.md` §5). (U1) The Shopify adapter: the call budget, the batched URL read, picture sizes in the readback, full-size and typed downloads. (U2) The record of remote pictures: migration 020, the remote state rule, the first-open read, Refresh from Shopify, the background step, retention's two new exclusions. **Not here:** any picture on screen, the list, the search (H2); bring back (H3); the view, activation and commands (H4). No frontend file changes.

## How to work this plan (flow.md §10, master_plan_archive.md §3)
One runner phase of 2 units, in order; after each, its **Unit test commands**, then a `CHECKPOINT (not approved):` commit of that unit's paths. Row ids `H1·C<n>(<row>)`; tests named after their row (`h1_c1a_<slug>`). Open details: the smallest option consistent with 10H and §5, recorded under **Assumptions**. Reviewer notes: `docs/implementation/backlog_archive/H1_points_and_remote_record.md`. **Never call Shopify or its CDN, never launch the built app** (master plan §3).

| Unit | What it builds | Criteria | Rows | Changes app files |
|---|---|---|---|---|
| U1 | The adapter: budget, URL read, sizes, downloads | 6 | 28 | **yes** |
| U2 | The record, the state rule, first-open read, refresh, retention exclusions | 6 | 27 | **yes** |
| **Total** | | **12** | **55** | |

## Read first (whole milestone)
`docs/process/flow.md` (§4, §4A, §10) · `docs/implementation/master_plan_archive.md` §3 (**Rule 9**, **Rule 17**), §4, **§5 MC-H5, MC-H1.4, MC-H3.1–3, MC-H3.8, MC-H4.8** · book `10H` §3, §4 (stale URLs), §6 (first open, refresh), §8, §9 · book `10F` (whole; clearing and M41) · `docs/guide/03_rules_and_principles.md` (H1, H6, H7; Persistence rules; Tests) · `docs/guide/04_change_impact.md` rows "The schema", "`AppPaths` layout".

## Points recorded by planning (each is one sentence in the narrative's "Choices made for you")
- **Nothing to see yet:** this milestone gives the app the knowledge the next ones show.
- **Background work waits for Shopify's points; your own presses never do.**
- **A shop product gets its picture list the first time it is needed;** a product whose pictures Shopify is still processing is asked again later.
- **The clean-up after publishing now also remembers each picture's size,** so the app can later say how much a bring back downloads.

## Unit 1 — The adapter: budget, URL read, sizes, downloads

**Unit id:** `H1·U1` · **Rows:** H1·C1–C6 · **Changes app files:** yes

### Goal
`CommerceAdapter::{budget, media_urls, fetch_original}`, `MediaImage.bytes`, `Error::ImageGone`, `HttpTransport::download`, `mechanisms::remote::must_wait`.

### Read first
Master plan §3 **Rule 17** (which shapes are captured, which schema-grounded) · `crates/integrations/src/commerce/shopify/adapter.rs` (`call` :432, `call_inner` :449-492, `media_readback` :1681-1694, `fetch_image` :1696-1713, `media_readback_of` :1970-2010, `PollConfig` :158) · `graphql.rs` (`MEDIA_READBACK` :32, `MEDIA_POLL` :34) · `transport.rs` (`HttpTransport` :37-57, `MAX_DOWNLOAD_BYTES` :60, `ReqwestTransport::download` :193-218, `get_bytes` :226) · `errors.rs` (`is_throttled`, `graphql_errors`) · `crates/core/src/ports/commerce.rs` (`media_readback` :213, `fetch_image` :216) · `crates/core/src/domain/publish.rs` (`MediaImage` :61, `MediaReadback`) · `crates/core/src/error.rs` · `crates/testkit/src/fake_transport.rs` (rules, `replay`, `requests`) and `crates/testkit/tests/suite/p12_shopify_adapter.rs` (how an adapter is built over `FakeTransport` and a fixture is loaded, :60-75) · fixture `crates/integrations/src/commerce/shopify/fixtures/sv4_job_poll_done.json` (the captured `extensions.cost` shape).

### Files
New: `crates/core/src/domain/remote.rs`, `crates/core/src/mechanisms/remote.rs`, `crates/integrations/src/commerce/shopify/fixtures/{h1_media_urls.json, h1_media_readback_sizes.json}`, `crates/testkit/tests/suite/h1_shopify_reads.rs`. Edited: `crates/core/src/{domain/mod.rs, mechanisms/mod.rs, domain/publish.rs, ports/commerce.rs, error.rs}`, `crates/integrations/src/commerce/shopify/{adapter.rs, graphql.rs, transport.rs}`, `crates/testkit/src/{fake_transport.rs, fake_commerce.rs}`, `crates/testkit/tests/suite/main.rs`, `apps/desktop/src-tauri/src/commands/items.rs` (the `CommandError` code of `ImageGone`, beside `files_cleared` :151).

### Core mechanisms (the only functions with unit tests)
`mechanisms::remote::must_wait` (inline tests in `mechanisms/remote.rs`) · `projected` in `commerce/shopify/adapter.rs` (inline test).

### Ordered tasks
1. `domain/remote.rs`: `CallBudget {available: f64, maximum: f64}` (only this type in U1). `mechanisms/remote.rs`: `BUDGET_FLOOR`, `must_wait` (MC-H5.3).
2. Port (`ports/commerce.rs`): `budget`, `media_urls`, `fetch_original` with the defaults of master plan §4 Ports. `MediaImage` gains `bytes: Option<i64>`; every constructor is updated (`media_readback_of`, `FakeCommerce::media_readback`, `retention.rs` reads only `id` and `url`). `Error::ImageGone` (`#[error("image_gone")]`) and its command code.
3. Transport: `HttpTransport::get_bytes` becomes `download(&self, url: &str) -> Result<Download>`, `Download {status: u16, bytes: Vec<u8>}` (default: `Err(Provider("download not supported"))`). `ReqwestTransport::download` returns the status and the body for any HTTP answer (the size limit stays: a body over `MAX_DOWNLOAD_BYTES` is `Err(Provider("download: too large"))`); transport errors stay `map_error`'s. `FakeTransport` gets `on_download(url, Vec<Download>)` (a queue per exact URL, the last one repeating) and records `Recorded::Download {url}`; an unmatched download panics like an unmatched request.
4. Adapter downloads: a private `get(url)` maps `Download`: `200` → bytes; `403` or `404` → `Error::ImageGone`; any other status → `Error::Provider("shopify download: HTTP <n>")`. `fetch_image` keeps its URL rule (`?width=` or `&width=`); `fetch_original` requests the URL unchanged. Both refuse a non-`https://` URL before any request and log the existing `provider call` line (op `fetch_image` / `fetch_original`).
5. Budget (MC-H5.1–2): in `call_inner`, after each reply is received (also a throttled or failed one), read `body.extensions.cost.throttleStatus`; when `maximumAvailable`, `currentlyAvailable` and `restoreRate` are all numbers (`as_f64`), store them with `Instant::now()` in a `Mutex<Option<…>>` of the adapter. `projected` is the pure rule; `budget()` applies it with the elapsed seconds. Log `shopify.budget` is the caller's (U2), not the adapter's.
6. `MEDIA_URLS` in `graphql.rs`: `query MediaUrls($ids: [ID!]!) { nodes(ids: $ids) { id ... on MediaImage { image { url } originalSource { fileSize } } ... on GenericFile { url originalFileSize } } }`. `media_urls(ids)`: empty → `Ok(vec![])` without a request; otherwise one `call` per chunk of `URL_REFRESH_CHUNK` (25) ids, in order; each node maps to `None` (JSON `null`) or `Some(MediaImage {id, url: image.url or url, bytes: originalSource.fileSize or originalFileSize})`; the answers are joined in the order of `ids`. A reply whose `nodes` length differs from the chunk's is `Err(Provider("shopify media urls: <got> of <asked>"))`.
7. `MEDIA_READBACK` gains the sizes: gallery `... on MediaImage { image { url } originalSource { url fileSize } }`; condition references `... on MediaImage { id image { url } originalSource { fileSize } } ... on GenericFile { id url originalFileSize }`. `media_readback_of` fills `bytes`; the URL choice is unchanged (`image.url`, else `originalSource.url`).
8. Fixtures (schema-grounded, `source` says so; values are placeholders, master plan §3 Secrets): `h1_media_urls.json` = request `MediaUrls` with three ids; response `nodes: [ {id: "gid://shopify/MediaImage/1001", image: {url: "https://cdn.shopify.com/s/files/1/0000/0000/0000/files/a.jpg"}, originalSource: {fileSize: 1843211}}, null, {id: "gid://shopify/GenericFile/1003", url: "https://cdn.shopify.com/s/files/1/0000/0000/0000/files/c.jpg", originalFileSize: 904112} ]` with an `extensions.cost` block copied from `sv4_job_poll_done.json`. `h1_media_readback_sizes.json` = request `MediaReadback`; response a product with two gallery `MediaImage` nodes (the second with `originalSource.fileSize: null`) and a `metafield` whose `value` is the JSON list of three ids and whose `references.nodes` hold a `MediaImage` (with `originalSource.fileSize`) and a `GenericFile` (with `originalFileSize`) — the third id has no node.
9. `FakeCommerce` (for U2 and later milestones; each addition has a caller in U2's tests): per-media `bytes` (default `None`; `set_media_bytes(id, n)`), `budget: Option<CallBudget>` (`set_budget`), `media_urls` (answers from the shop's media and files: `None` for an id it does not hold; records the ids of each call in `url_reads`), `fetch_original` (records the URL in `original_fetches`; serves `set_image(media_id, bytes)` when registered, else `fake_image_bytes`), `image_faults` (`fail_image(url, Vec<ImageFault {Gone, Fail}>)`: consumed one per `fetch_image`/`fetch_original` of that URL), `set_fetch_delay(ms)` and `max_fetches_in_flight()` (as `max_uploads_in_flight`, :112).
10. Tests for the rows.

### Criteria (6 criteria, 28 rows)
Boundary: the real `ShopifyAdapter` over `FakeTransport` (outcome = the value returned and the requests the transport received), except H1·C2–C3 (pure).
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| H1·C1(a) | `ShopifyAdapter::budget()` | a new adapter, no call | `None` | MC-H5.2 |
| H1·C1(b) | same | one query answered `200` with the body of `sv4_job_poll_done.json` (captured) with `restoreRate` set to `0.0` | `Some {available: 1999.0, maximum: 2000.0}` | MC-H5.1; 10H §8 |
| H1·C1(c) | same | after (b), a call answered with the body of `sv0_graphql_401.json` (no `extensions`; the call fails) | unchanged: `Some {1999.0, 2000.0}` | MC-H5.1 |
| H1·C1(d) | same | one query answered with `throttleStatus {maximumAvailable: 1000, currentlyAvailable: 954, restoreRate: 0}` (all integers, the shopify.dev rate-limits example with the rate zeroed) | `Some {954.0, 1000.0}` | MC-H5.1 (numbers read as `f64`) |
| H1·C1(e) | same | (b), then a query answered with `currentlyAvailable: 500`, `maximumAvailable: 2000.0`, `restoreRate: 0.0` | `Some {500.0, 2000.0}` | MC-H5.1 (the newest reading wins) |
| H1·C2(a) | `projected` | `(100, 1000, 50, 4.0)` | `300.0` | MC-H5.2 |
| H1·C2(b) | same | `(100, 1000, 50, 30.0)` | `1000.0` | MC-H5.2 (never above the maximum) |
| H1·C2(c) | same | `(954, 1000, 50, 0.0)` | `954.0` | MC-H5.2 |
| H1·C2(d) | same | `(954, 1000, 0, 60.0)` | `954.0` | MC-H5.2 |
| H1·C3(a) | `must_wait` | `Some {249.0, 1000.0}` | `true` | MC-H5.3; 10H §8 (below 25 %) |
| H1·C3(b) | same | `Some {250.0, 1000.0}` | `false` | MC-H5.3 |
| H1·C3(c) | same | `None` | `false` | MC-H5.3 |
| H1·C3(d) | same | `Some {0.0, 0.0}` | `false` | MC-H5.3 |
| H1·C4(a) | `media_urls` | the three ids of `h1_media_urls.json` (schema-grounded), the transport replaying it | `[Some {id …/1001, url …/a.jpg, bytes 1843211}, None, Some {id …/1003, url …/c.jpg, bytes 904112}]`; the transport received one `MediaUrls` request whose `ids` are the three, in order | MC-H1.4; 10H §4 |
| H1·C4(b) | same | 26 ids; the transport answers a 25-node and a 1-node reply | 26 entries in the order asked; the transport received exactly two `MediaUrls` requests, with 25 and 1 ids | MC-H5.5; 10H §4 (at most 25 per query) |
| H1·C4(c) | same | no ids | `[]`; the transport received no request | MC-H5.5 |
| H1·C4(d) | same | one id; the node is a `MediaImage` with `image: null` (still processing) | `[Some {id, url: None, bytes: None}]` | MC-H1.4 (a missing URL) |
| H1·C4(e) | same | two ids; the reply's `nodes` has one entry | `Err(Provider)` naming `1 of 2` | MC-H1.4 (never a wrong pairing) |
| H1·C5(a) | `media_readback` | the product of `h1_media_readback_sizes.json` | gallery `[{id, url = image.url, bytes = fileSize}, {id, url, bytes: None}]` in the reply's order | MC-H3.2; MC-H4.1 (sizes) |
| H1·C5(b) | same | same | condition, in the metafield value's order: the `MediaImage` with `bytes = originalSource.fileSize`, the `GenericFile` with `url` and `bytes = originalFileSize`, and the third id with `url: None`, `bytes: None` | MC-H3.2 |
| H1·C5(c) | same | a reply whose gallery node has `image: null` and `originalSource {url, fileSize}` | that picture's `url` = the `originalSource.url`, `bytes` = `fileSize` (the existing fallback) | 10F Rules (M41 readback unchanged) |
| H1·C5(d) | same | `product: null` | `Err(NotFound)` | M41 (unchanged) |
| H1·C6(a) | `fetch_image(url, 320)` | `https://cdn.shopify.com/…/a.jpg`; the transport answers `Download {200, B}` | `B`; the transport received a download of `…/a.jpg?width=320` | MC-H1.2 |
| H1·C6(b) | `fetch_original(url)` | `https://cdn.shopify.com/…/a.jpg?v=17`; `Download {200, B}` | `B`; the transport received a download of exactly that URL | MC-H4.3; 10H §7.1 (no size parameter) |
| H1·C6(c) | `fetch_image` | the transport answers `Download {404, []}` | `Err(ImageGone)` | MC-H1.4; 10H §4 |
| H1·C6(d) | `fetch_original` | the transport answers `Download {403, []}` | `Err(ImageGone)` | MC-H1.4 |
| H1·C6(e) | `fetch_image` | the transport answers `Download {500, []}` | `Err(Provider)` (not `ImageGone`) whose text holds no URL | MC-H1.4 (any other error is not "gone") |
| H1·C6(f) | `fetch_original` | `http://example.test/a.jpg` | `Err(InvalidData)`; the transport received nothing | H1 (read-only, https only; existing rule) |

Fixture grounding (rule 17): `extensions.cost` = captured (`sv4_job_poll_done.json`, and the shopify.dev rate-limits page for the all-integer form); `nodes(ids:)`, `originalSource.fileSize`, `originalFileSize` = schema-grounded (shopify.dev Admin GraphQL 2026-10 `queries/nodes`, `objects/MediaImageOriginalSource`, `objects/GenericFile`); `Download {status, bytes}` = the value `ReqwestTransport::download` builds from `response.status()` and `response.bytes()` after task 3.

### Delegated choices
Whether the stored reading lives in the adapter struct or a small `Throttle` type · the exact `Provider` texts beyond the parts the rows name.

### Notes
- Hazard: `originalSource.url` is "valid only for a short period" (shopify.dev); never store it when `image.url` exists (the existing order in `media_readback_of` is right; do not flip it).
- Hazard: `call_inner` returns early on errors; read the budget before those returns, or a throttled answer (the one that matters most) is never read.
- Hazard: the budget is read under a mutex that must not be held across the HTTP request (uploads run in parallel, P64).
- Hazard: `nodes` answers `null` for an id it does not know; `parse` into a typed `Vec<Node>` would fail on it. Map `Json::Null` first.
- `fetch_image`'s only production caller today is `ArchiveService::download` (`retention.rs:220`); it keeps working unchanged in this milestone.

### Unit test commands
`cargo test -p psm-core --lib mechanisms::remote` · `cargo test -p psm-integrations --lib commerce::` · `cargo test -p psm-testkit --test suite h1_shopify_reads::` · `cargo test -p psm-testkit --test suite p12_shopify_adapter::` · `cargo test -p psm-testkit --test suite p62_retention_after_publish::` · `cargo clippy -p psm-core -p psm-integrations -p psm-testkit --all-targets -- -D warnings`

## Unit 2 — The record of remote pictures

**Unit id:** `H1·U2` · **Rows:** H1·C7–C12 · **Changes app files:** yes

### Goal
Migration 020, the store methods, `remote_state`, `RemoteItems` with `state`, `ensure_record`, `refresh`, `run_pending` (queued first-open reads only, in this milestone), its wiring into `App`, and retention's exclusions.

### Read first
Master plan §4 (Vocabulary, Ports, Storage, Service), §5 **MC-H3.1–3**, **MC-H3.8**, **MC-H4.8**, **MC-H5.3–4** · `crates/core/src/services/retention.rs` (whole: `verify` :100-139, `ArchiveService` :160-248) · `crates/core/src/mechanisms/retention.rs` (`RetentionInput` :12, `eligibility` :40) · `crates/core/src/services/storage.rs` (`retention_scan` :702-825, `clear_item` :827-898) · `crates/core/src/domain/storage.rs` (:45-62) · `crates/core/src/domain/item.rs` (`ItemOrigin`) · `crates/core/src/services/items.rs` (`adopt` :170-202) · `crates/infra/src/sqlite/store.rs` (`migrations` :67-90, `item_retention` :3210, `mark_item_cleared` :3236, `archived_images` :3255, `cleared_sessions` :3272) · `crates/infra/src/sqlite/migrations/010_retention.sql` · `crates/infra/src/sqlite/session_delete.rs` (:250-260) · `crates/integrations/src/lib.rs` (`storage_background` :1163-1190, shutdown :1345-1392, how `App` and `AppHandles` carry a service) · `crates/testkit/src/test_app.rs` (`with_fake_shop` :384, `commerce` :40) · `crates/testkit/tests/suite/p62_retention_after_publish.rs` (helpers `shop_app`, `item`, `published`, `clear_all`, `product_of` :50-300) and `p59_item_type_from_shopify_product_type.rs` (how a test adopts a shop product with `ItemsService::adopt`) · `crates/testkit/tests/suite/p27_retire_shape_on_arrival.rs` :346-372 (the rewind test).

### Files
New: `crates/infra/src/sqlite/migrations/020_remote_items.sql`, `crates/core/src/services/remote.rs`, `crates/testkit/tests/suite/h1_remote_record.rs`. Edited: `crates/core/src/{domain/remote.rs, domain/storage.rs, mechanisms/remote.rs, mechanisms/retention.rs, ports/store.rs, services/mod.rs, services/retention.rs, services/storage.rs}`, `crates/infra/src/sqlite/store.rs`, `crates/integrations/src/lib.rs`, `crates/testkit/src/{test_app.rs, fake_commerce.rs}`, `crates/testkit/tests/suite/{main.rs, p27_retire_shape_on_arrival.rs}`, `docs/guide/{02_architecture.md, 04_change_impact.md}`.

### Core mechanisms (the only functions with unit tests)
`mechanisms::remote::remote_state` · `mechanisms::retention::eligibility` (its existing inline tests gain the rows of H1·C11).

### Ordered tasks
1. Migration `020_remote_items.sql` (master plan §4 Storage) with its `M::up` line; **add the three `DROP COLUMN` lines to the rewind test** (`p27_…:354-368`). `ItemRetention` gains `restored_at`, `remote_checked_at`; `ArchivedImage` gains `bytes`; `item_retention`, `mark_item_cleared`, `archived_images` read and write them.
2. Store methods (each one transaction): `set_remote_record(item_id, images, checked_at)` (upserts the `item_retention` row keeping `keep_files`, `cleared_at`, `restored_at`; sets `remote_checked_at`; deletes and re-inserts the item's `archived_images`), `update_remote_urls(item_id, updates)`, `remote_input(session_id) -> RemoteInput` (`cleared`, `linked` for integration `shopify`, `adopted` = the item's origin is `linked`, `photo_rows` = `COUNT(*)` of the session's photos of any classification, `checked`, `pictures` = the record's row count; a session without an item answers all-false).
3. `domain/remote.rs`: `RemoteKind`, `RemoteState`. `mechanisms/remote.rs`: `RemoteInput`, `remote_state` (MC-H3.1).
4. `RemoteItems` (`services/remote.rs`; constructor of master plan §4 — the arguments later milestones use are taken now so the wiring is written once): `state(session_id)` (`Local` when no integration); `ensure_record(session_id)` (MC-H3.2: lock, re-read the input, read only when still `Unknown`); `queue_record(session_id)` and `run_pending()` (takes the queued sessions; `Waiting` without a read when `must_wait(budget())`, logging `shopify.budget`; else `ensure_record` each, errors logged; `Idle` when the queue is empty); `refresh(session_id)` (MC-H3.3; in this milestone the cache folders it prunes are still empty, the pruning code is written and covered by H1·C10(d) with files the test places).
5. Wiring: `App.remote`, `AppHandles.remote`, the `remote_background` thread (master plan §4 Service) and its bounded join; `TestApp` exposes `remote()` and never starts the thread.
6. Retention: `RetentionInput` gains `shop_only: bool` and `restored_at: Option<i64>`; `eligibility` answers `Never` for `shop_only`, and for `restored_at = Some(r)` with `published_at ≤ r` (MC-H3.8, MC-H4.8). `retention_scan` and `clear_item` fill them from `RemoteItems::state` / `item_retention`. `retention::verify` copies `bytes` into the `ArchivedImage`s it returns. `retention_scan`: before each `clear_item`, when `must_wait(config.commerce.budget())` → stop taking further items in this scan and set `work.scan_requested = true` (MC-H5.3).
7. Guide: `02_architecture.md` (§Persistence: the two columns and `bytes`; a short §Remote items: state rule, record, background step), `04_change_impact.md` (a row "a new `ALTER TABLE … ADD COLUMN` migration → add its `DROP COLUMN` to the `p27` rewind test"; a row "a new background GraphQL call → check `must_wait` first, MC-H5").
8. Tests for the rows.

### Criteria (6 criteria, 27 rows)
Boundary: `remote_state` and `eligibility` (pure); `RemoteItems` and `StorageService` on `TestApp::with_fake_shop()` (outcome = stored rows, the state, the reads and mutations `FakeCommerce` recorded, emitted events). "Archived item" = an item published and then cleared through the real clearing (`p62`'s `published` + `clear_all`); "adopted item" = `ItemsService::adopt` of a `FakeCommerce` product followed by `SessionsService::create`.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| H1·C7(a) | `remote_state` | `cleared` true, everything else false/0 | `Remote(Archived)` | MC-H3.1; 10H §3 |
| H1·C7(b) | same | `cleared` true, `adopted` and `linked` true, `photo_rows` 4, `checked` true, `pictures` 0 | `Remote(Archived)` | MC-H3.1 (cleared wins) |
| H1·C7(c) | same | not cleared; `adopted`, `linked`; `photo_rows` 0; not checked | `Unknown` | MC-H3.1 |
| H1·C7(d) | same | as (c), `checked`, `pictures` 3 | `Remote(ShopOnly)` | MC-H3.1; 10H §3 |
| H1·C7(e) | same | as (c), `checked`, `pictures` 0 | `Local` | MC-H3.1 |
| H1·C7(f) | same | as (d) with `photo_rows` 1 | `Local` | MC-H3.1 ("never photographed on this Mac") |
| H1·C7(g) | same | as (d) with `adopted` false; as (d) with `linked` false | `Local`; `Local` | MC-H3.1 |
| H1·C8(a) | `SqliteStore` on `TestApp` | `set_remote_record(item, [gallery 1 {m1, u1, bytes 1000}, condition 1 {c1, u2, bytes None}], 77)` | `archived_images(item)` returns the two pictures with those bytes, gallery first; `item_retention(item).remote_checked_at = Some(77)`, `cleared_at` and `restored_at` `None` | MC-H3.2 (round trip) |
| H1·C8(b) | the app's start on an older database | a data folder migrated only to 019 holding one `item_retention` row (`keep_files` 1, `cleared_at` 5) and two `archived_images` rows (written by SQL), then opened by the app | the item's retention reads `keep_files` true, `cleared_at` 5, both new times `None`; its two pictures read back with `bytes: None` | charter rule 7 (follow-up migration; existing data kept) |
| H1·C9(a) | `RemoteItems::ensure_record` | an adopted item whose product has gallery media `[m1, m2, m3]` (sizes set) and condition files `[c1, c2]` | the record = gallery 1–3 then condition 1–2 with those ids, the fake's URLs and sizes; `remote_checked_at` = the clock's now; `state` = `Remote(ShopOnly)`; `FakeCommerce` recorded exactly one `media_readback` read; one `SessionChanged` for the session | MC-H3.2; 10H §6 |
| H1·C9(b) | same | an adopted item whose product has no media and no condition file | no record row; `remote_checked_at` set; `state` = `Local`; one read | MC-H3.1, .2 |
| H1·C9(c) | same | after (a), called again | no further read (the read count stays 1) | MC-H5.5; 10H §6 ("later opens use the record") |
| H1·C9(d) | same | an adopted item; two threads call at the same time | exactly one `media_readback` read; the record as in (a) | MC-H3.2 (one lock) |
| H1·C9(e) | same | the shop is unreachable (`unreachable` set) | `Err`; no record row, `remote_checked_at` `None`, `state` = `Unknown` | MC-H3.2 |
| H1·C9(f) | same | the product's second gallery media has no URL (still processing) | `Err`; nothing written; `state` = `Unknown` | MC-H3.2 |
| H1·C9(g) | `queue_record` + `run_pending` | an adopted item queued; `FakeCommerce` budget `{100, 1000}`; then budget `{900, 1000}` | first `run_pending` = `Waiting` and no read was recorded; second = `Worked`, one read, `state` = `Remote(ShopOnly)` | MC-H5.3; 10H §8 |
| H1·C10(a) | `RemoteItems::refresh` | an archived item (3 gallery, 2 condition); the fake shop's gallery is then reordered to `[m2, m1, m3]` and a condition file `c3` added | the record = gallery `m2, m1, m3` at positions 1–3, condition `c1, c2, c3`; `cleared_at` unchanged; exactly one `media_readback` read by the refresh | MC-H3.3; 10H §6 |
| H1·C10(b) | same | as (a) with budget `{100, 1000}` | the same outcome at once (the read is made) | MC-H5.4 ("a press is never delayed") |
| H1·C10(c) | same | a session whose item is `Local` (local photos) | `Err(NotRemote)`; no read | MC-H3.3 |
| H1·C10(d) | same | an archived item; the test places `remote/<item>/cover-000000000000.jpg` and `archived/<item>/gallery-01-000000000000.jpg` (names no current picture has) and `archived/<item>/<the P62 name of gallery picture 1>` | after the refresh the two stale files are gone and the third file remains | MC-H3.3 |
| H1·C10(e) | same | (a), (b) and H1·C9(a) | `FakeCommerce::mutation_ops()` is empty | M-H1 |
| H1·C11(a) | `eligibility` | published, clean, latest step `Done`, integration, not pinned, not cleared, past the retention days, `shop_only` true | `Never` | MC-H3.8 |
| H1·C11(b) | same | as (a) with `shop_only` false, `restored_at` = `published_at` | `Never` | MC-H4.8; 10H §7.7 |
| H1·C11(c) | same | as (b) with `restored_at` = `published_at − 1` | `Due` | MC-H4.8 |
| H1·C12(a) | `StorageService` scan (10F clearing) | a published item whose fake media have sizes, past retention | cleared; its record's pictures carry those sizes | MC-H4.1 (sizes known for the prompt); 10F §3 |
| H1·C12(b) | same | two published items past retention; `FakeCommerce` budget `{100, 1000}` | after one scan neither is cleared, no `media_readback` read was made and Health has no `retention_not_cleared` | MC-H5.3; 10H §8 (10F's verification waits) |
| H1·C12(c) | same | after (b), budget `{900, 1000}`, the next scan | both are cleared | MC-H5.3 |

### Delegated choices
How the queue and the lock are held inside `RemoteItems` · whether `refresh` and `ensure_record` share one private function.

### Notes
- Hazard (the rewind test): see task 1; forgetting it turns `p27_retire_shape_on_arrival::c4a_…` red with "duplicate column name".
- Hazard: `set_remote_record` must not touch `cleared_at` or `keep_files` (a refresh of an archived item would otherwise un-archive it or drop its pin).
- Hazard (MC-H3.8): a shop-only item that later completes a details-only publish (H4) would be "published and clean"; without the `shop_only` exclusion the clearing would verify zero images, call `mark_item_cleared` with an empty list and wipe its record. The pure rows are here; H4·C8 holds the service row.
- Hazard (H7): `RemoteInput.linked` reads the integration's link through the existing `"shopify"` literal the code already uses (`retention::INTEGRATION`, `retention.rs:22`); do not add a second literal.
- `mark_item_cleared` leaves `restored_at` as it is (MC-H4.8 compares it with `published_at`).

### Unit test commands
`cargo test -p psm-core --lib mechanisms::remote` · `cargo test -p psm-core --lib mechanisms::retention` · `cargo test -p psm-testkit --test suite h1_remote_record::` · `cargo test -p psm-testkit --test suite p62_retention_after_publish::` · `cargo test -p psm-testkit --test suite p27_retire_shape_on_arrival::` · `cargo test -p psm-testkit --test suite a17_delete_session::` · `cargo clippy -p psm-core -p psm-infra -p psm-integrations -p psm-testkit --all-targets -- -D warnings`

## Closing gate (once, after the last unit; replaces the template's closing gate)
`cargo test --workspace` · `cargo clippy --workspace --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cd apps/desktop && npm run typecheck` · `npm test`. One cargo command at a time.

## Review mode
The reviewer reads the units in order and runs, once: `cargo test -p psm-core --lib mechanisms::remote` · `cargo test -p psm-testkit --test suite h1_shopify_reads::` · `cargo test -p psm-testkit --test suite h1_remote_record::` · `cargo test -p psm-testkit --test suite p62_retention_after_publish::` · clippy on the touched crates, fmt. The implementer's full-suite stamp stands. Only B1–B4 block. Also check, by reading: the three new port methods take no `ConfirmedRun` and the adapter issues only queries and downloads for them; the budget is read before `call_inner`'s error returns; the new fixtures' `source` says "schema-grounded" and carry no real shop value; `set_remote_record` leaves `cleared_at` and `keep_files`; no Shopify or Capture One name was added to `crates/core`. Notes go to `docs/implementation/backlog_archive/H1_points_and_remote_record.md`.

## Manual try-out (the narrative is built from this; done by the owner on the installed app, never by a session)
Nothing to try on screen. After the next milestone the sessions list shows what this one prepared. (For the orchestrator's live check after H4: the log line `shopify.budget` appears only when a background step waited.)

## Review log
