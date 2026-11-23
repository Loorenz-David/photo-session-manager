---
phase: L2_labels_box
title: The Labels box — a fourth box in the sessions list shows the products whose price label is still to print, as Pricing lists them: missing price tags first, with Not printed / Printed / All, counts, search and Previous / Next pages; a row whose product has a session on this Mac looks and opens like that session, any other row shows Pricing's title and picture and opens as a shop-only product; rows can be ticked, also all N at once, ready for label generation
difficulty: normal
projection: no
depends_on: [L1_labels_provider_and_marker]
---
# L2 — The Labels box

**Tracker:** APPROVED (2026-11-23, opus, review r1 APPROVED_WITH_NOTES: perimeter = the declared paths of e81b52d3 / 70f4d19b / 80a720ca / 3fe83622 (useSessionLabels.ts and the guide edits disclosed); review-mode checks green (core mechanisms 14, suite l2_labels_list 36, psm-desktop 103, vitest labels+sessions 117, clippy, fmt, typecheck); all 78 rows have a boundary test that fails on a wrong outcome; H1–H8 hold; no assumption contradicts 09C / MC-L1; 4 backlog notes)

(This line is the milestone's tracker row. Sessions change only this line of this file. The gate for `depends_on` is the Tracker line of the plan named there.)

## Goal
Book `09C` §5 by contract **MC-L1** (`docs/implementation/master_plan_labels.md` §5). (U1) The provider-backed list in the service: rows, matching to sessions, pictures, opening an unmatched row, the box's number, Select all. (U2) The commands, the pure page state (`pageStep`) and the box's words. (U3) The box itself: toolbar, rows, Previous / Next, selection, refresh. **Not here:** the check and done dialogs, runs, downloads, the enabled ⋯ entry and the working "Generate labels for N" button (L3: in L2 the button is on the bar and disabled); batch labels from Open or Published (09C §11).

**Owner card 1** (master plan §2A) decides criterion L2·C5 and rows L2·C10(a), L2·C14(c). This plan is written for the recommended answer **yes**; the changes for **no** are listed there. The milestone does not start before the card is answered.

## How to work this plan (flow.md §10, master_plan_labels.md §3)
One runner phase of 3 units, in order; after each, its **Unit test commands**, then a `CHECKPOINT (not approved):` commit of that unit's paths. Row ids `L2·C<n>(<row>)`; tests named after their row (`l2_c1a_<slug>`). Open details: the smallest option consistent with 09C §5 and MC-L1, recorded under **Assumptions**. Reviewer notes: `docs/implementation/backlog_labels/L2_labels_box.md`. **Never call Pricing, Shopify or a CDN, never launch the built app** (master plan §3).

| Unit | What it builds | Criteria | Rows | Changes app files |
|---|---|---|---|---|
| U1 | The list in the service: rows, matching, pictures, opening, the number, Select all | 6 | 36 | **yes** |
| U2 | Commands, the page state, the words | 3 | 19 | **yes** |
| U3 | The Labels box on screen | 5 | 23 | **yes** |
| **Total** | | **14** | **78** | |

## Read first (whole milestone)
`docs/process/flow.md` (§4, §4A, §10) · `docs/implementation/master_plan_labels.md` §2A, §3, §4 (Vocabulary, Constants, Pure rules, Ports, Service, Commands, Words, Tests), **§5 MC-L1 (every item), ACC.1, ACC.4, MC-L3.2**, §6 · book `09C` §5, §9 (M-L1, M-L2, M-L5, M-L6), §12 scenarios 1, 2, 10, 11 · book `10H` §3, §4, §6 (shop-only items, the search picture cache) · `docs/implementation/master_plan_archive.md` §5 MC-H1.1, MC-H2.5 · the L1 plan (what `LabelsService`, `LabelCopy`, `FakePricing` and the adapter already offer) · `docs/guide/04_change_impact.md` rows "A Tauri command signature", "Anything that hands the screen a preview path".

## Points recorded by planning (each is one sentence in the narrative's "Choices made for you")
- **The Labels box has Previous and Next only;** "Page 2 of 5" is computed from the filter's count.
- **A Labels row whose product has the same SKU as a session on this Mac opens that session,** even when that session is linked to another shop product.
- **Opening a Labels row with no session here creates the session at once** (one read of the shop product, nothing written to Shopify) and opens it as a shop-only product.
- **Select all N selects only products that have a SKU,** so the bar may read "All 58 selected" under "Select all 60".
- **The search box in the Labels box searches SKU, title and product type through Pricing;** the time filter and "To review" are hidden there.
- **The list refreshes at most once every 2 seconds** when labels change; your ticks and your page are kept.
- **"Generate labels for N" is on the bar but greyed out** until label generation lands (L3).

## Unit 1 — The list in the service

**Unit id:** `L2·U1` · **Rows:** L2·C1–C6 · **Changes app files:** yes

### Goal
`LabelsService::{list, summary, select_all, open_product}`, `match_product`, `LabelStore::linked_items`, and the fake Pricing's product list.

### Read first
Master plan §5 **MC-L1.1–.8** · `crates/core/src/services/labels.rs` (L1) · `crates/core/src/mechanisms/labels.rs` (L1: `LabelCopy`) · `crates/core/src/services/sessions.rs` (`page` :599, `create` :370) · `crates/core/src/domain/session.rs` (`SessionRow` :122, `SessionPage` :169, `ListFilter` :108) · `crates/core/src/services/items.rs` (`adopt` :191-223, `collision` :225) · `crates/core/src/services/remote.rs` (`search_picture` :442, `run_pending` :634, `row_pictures` :359) · `crates/core/src/mechanisms/remote.rs` (`search_name` :33, `SEARCH_EDGE`) · `crates/core/src/domain/remote.rs` (`PictureTarget` :18) · `crates/core/src/ports/commerce.rs` (`item_get` :71, `fetch_image` :219) · `crates/infra/src/sqlite/{labels.rs (L1), conversations.rs (`item_sessions` :725)}` · `crates/testkit/src/{fake_pricing.rs, fake_commerce.rs (`add_product` :281, `image_fetches` :228, `mutation_ops` :250, `set_hook` :232), test_app.rs}` · `crates/testkit/tests/suite/{h2_pages_and_search.rs (search-picture tests), h1_remote_record.rs (`adopted` :115)}` · the package's `API.md` `GET …/labels/products`.

### Files
New: `crates/testkit/tests/suite/l2_labels_list.rs`. Edited: `crates/core/src/{domain/label.rs, mechanisms/labels.rs, ports/labels.rs, services/labels.rs}`, `crates/infra/src/sqlite/labels.rs`, `crates/integrations/src/{labels.rs, lib.rs}` (the service now takes `store`, `remote`, `commerce`), `crates/testkit/src/{fake_pricing.rs, fake_commerce.rs}`, `crates/testkit/tests/suite/main.rs`, `docs/guide/{02_architecture.md, 04_change_impact.md}`.

### Core mechanisms (the only functions with unit tests)
`mechanisms::labels::match_product`.

### Ordered tasks
1. Vocabulary of L2 (master plan §4: `LinkedItem`, `LabelRow`, `LabelPage`, `Selected`) and the constants `LIST_PAGE`, `SELECT_ALL_PAGE`, `SELECT_ALL_MAX_PAGES`, `COUNT_FRESH_MS`, `QUERY_MAX`. `match_product` (MC-L1.3) with unit tests: link match; SKU match; link wins over SKU; two items linked to the same product → the lowest item id; no SKU and no link → none; SKU differing only in case → none.
2. `LabelStore::linked_items` for `SqliteStore` (one read: every item, its `LINK_INTEGRATION` link's external id, its latest session id).
3. `LabelsService::list` by MC-L1.1–.5 (one `SessionsService::new(store, clock).with_remote(remote).page(All, None, None, Some(&ids), 1)` for the matched sessions of the page; none when there is no match). `summary(refresh)` by MC-L1.6 **(card 1)**. `select_all` by MC-L1.7. `open_product` by MC-L1.8 (`ItemsService::new(store, clock).with_commerce(commerce).adopt`, `SessionsService::create`).
4. `FakePricing` (master plan §4 Tests, the **L2** list): `GET /labels/products` over its label products, newest created first; `filter` by the package's meaning (`missingPriceTag` = priced and not printed; `notPrinted` includes unpriced; `printed`; none = all); `q` matching SKU, title or product type, case-insensitive; `limit` 1–500; `cursor` an opaque text the fake made (e.g. the next offset, encoded), `nextCursor: null` on the last page; `counts` over the searched list before the filter; `workFrom` (default `"2026-11-19"`, `set_work_from`); `repeat_cursor_next()`; `add_label_products(n)` for many products at once.
5. Guide: `02_architecture.md` §Labels (the list, matching, the picture rule, opening), `04_change_impact.md` (a row "the matching rule → `match_product`, `linked_items`").
6. Tests for the rows.

### Criteria (6 criteria, 36 rows)
Boundary: `App.labels` on `TestApp::with_pricing()`, connected and `Allowed` as in the L1 plan unless the row says otherwise; outcome = the returned value, `FakePricing::request_lines()`, the fake shop's recorded calls, files on disk, collected events, the items and sessions in the store. "Pn" = a label product of the fake Pricing with product id `n` (digits); "a linked session for Pn" as in the L1 plan.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| L2·C1(a) | `list(MissingPriceTag, None, None)` | P1 (created first), P2, P3 (created last), all priced and not printed | exactly one label request, `GET /stores/<id>/labels/products?filter=missingPriceTag&limit=25`; rows P3, P2, P1 in that order, each with its `sku`, `title`, `price_minor`, `state {NotPrinted, None, Some(true), None}`, `created_at`; `counts.missing_price_tag = 3`; `next_cursor None` | MC-L1.1, .2; §12.1 |
| L2·C1(b) | `list(MissingPriceTag, None, None)` then `list(MissingPriceTag, None, <its next_cursor>)` | 30 such products | 25 rows and a cursor; then a request carrying that cursor (percent-encoded) and `limit=25`, 5 rows, `next_cursor None`; no product appears twice | MC-L1.1; 09C §5 Pages |
| L2·C1(c) | `list(Printed, Some("  teak  "), None)`; `list(All, Some("   "), None)`; `list(All, Some(<150 characters>), None)` | — | the requests carry `filter=printed&q=teak`; neither `filter` nor `q`; `q` = the first 100 characters | MC-L1.1 |
| L2·C1(d) | `list(All, None, None)` | the fake's `workFrom` is `2026-11-19` | `work_from Some("2026-11-19")` | 09C §5 (the line under the toolbar) |
| L2·C1(e) | after (a): `session_labels([s], false)` | `s` a linked session for P2 | P2's state as the list gave it; no status request | MC-L1.5; MC-L3.2; 09C §7 ("Labels rows use their list fields") |
| L2·C1(f) | `list(MissingPriceTag, None, None)` | no key saved; a key under `deny_labels()` (access already `Denied`) | `Err(Off(NoKey))`; `Err(Denied)`; no label request for either | ACC.1; M-L6; §12.11 |
| L2·C1(g) | `list(…)` | the fake answers 503 `product_list_unavailable`; 500; 403 `insufficient_scope` | `Err(Provider(ListUnavailable))`; `Err(Provider(Unreachable(_)))`; `Err(Provider(NotAllowed))` and `access()` = `Denied` | 09C §10; ACC.4 |
| L2·C2(a) | `list(All, None, None)` | a linked session `s1` for P1 | P1's row has `session Some(row)` with `row.id = s1` and the session's own SKU and title | MC-L1.3(i); 09C §5 Rows |
| L2·C2(b) | same | P2 has no linked item; a local item with SKU `AB-1` has session `s2`; Pricing's row for P2 has `sku: "AB-1"` | P2's row has `session.id = s2` | MC-L1.3(ii) |
| L2·C2(c) | same | item A linked to P1 has session `sA`; item B (not linked) has SKU `AB-1` and session `sB`; Pricing's row for P1 has `sku: "AB-1"` | P1's row has `session.id = sA` | MC-L1.3 (link first) |
| L2·C2(d) | same | P3 has no linked item and no item has its SKU | `session None`; `title` = Pricing's title; `commerce_id Some("gid://shopify/Product/3")` | MC-L1.2, .3 |
| L2·C2(e) | same | Pricing's row has `sku: null` and no linked item; Pricing's row has `sku: "ab-1"` while the local item's SKU is `AB-1` | `session None` for both | MC-L1.3 (exact SKU) |
| L2·C2(f) | same | an item linked to P4 that has no session | `session None` | MC-L1.3 |
| L2·C3(a) | `list(All, None, None)` | the matched session of L2·C2(a) has a main photo at position 1 whose preview file exists; then that file is deleted and the list asked again | `session.cover_preview_path` = that path; then none | MC-L1.4; M-H4 |
| L2·C3(b) | `list(All, None, None)`, `remote.run_pending()`, `list` again | P3 unmatched with `thumbUrl` = `https://cdn.shopify.com/s/files/1/0000/0000/0000/files/p3_160x.jpg` | first `picture_path None`; then the file `<previews>/remote/search/<search_name(url)>` exists, the fake shop recorded the fetch `(that url, 160)`, `PictureReady {Search {external_id: "gid://shopify/Product/3"}, path}` was emitted; the second list answers `picture_path Some(that file)` | MC-L1.4; 09C §5 Rows; MC-H2.5 |
| L2·C3(c) | `list`, `run_pending` | an unmatched product with `thumbUrl: null`; one with `thumbUrl` = `http://cdn.example/x.jpg` | `picture_path None`; the fake shop recorded no fetch for either | MC-L1.4 (`https://` only) |
| L2·C3(d) | after (a)–(c) | — | the fake shop's `mutation_ops()` is empty; no request line contains `/labels/runs` | M-L5; M-L2 |
| L2·C4(a) | `open_product("1")` | the linked session `s1` for P1 | `Ok(s1)`; the fake shop saw no call | MC-L1.8 |
| L2·C4(b) | `open_product("4")` | the fake shop holds product `gid://shopify/Product/4` (SKU `SH-4`, a title); no item here | a session id; its item has SKU `SH-4`, origin `linked` and the link `gid://shopify/Product/4`; the fake shop saw exactly one read of that product and its `mutation_ops()` is empty | MC-L1.8; M-L5; §12.10 |
| L2·C4(c) | after (b): `open_product("4")` again; then `list(All, None, None)` | — | the same session id with no shop call; P4's row now has that session | MC-L1.8 ("after that it is matched") |
| L2·C4(d) | `open_product("5")` | an item linked to P5 without a session | a new session of that item; no shop call | MC-L1.8 |
| L2·C4(e) | `open_product("6")` | the shop product `gid://shopify/Product/6` has no SKU | `Err(Storage(InvalidValue(_)))`; no item and no session were created | MC-L1.8 (errors pass through) |
| L2·C4(f) | `open_product("7")` | the shop product 7 has SKU `SH-7`; a local item has the SKU `sh-7` (another SKU, the same folder key without regard to case) | `Err(Storage(SkuCollision {..}))`; nothing created | MC-L1.8 |
| L2·C4(g) | `open_product("abc")` | — | `Err(Storage(InvalidValue(_)))`; no shop call | MC-L1.8 (`commerce_id_of` none) |
| L2·C5(a) | `summary(true)` on a freshly connected app | three priced, unprinted products | exactly one label request, `GET …/labels/products?filter=missingPriceTag&limit=1`; `Some(3)` | MC-L1.6 (card 1); §12.1 |
| L2·C5(b) | `summary(false)` before any answer; after (a) | — | `None` with no label request; `Some(3)` with no label request | MC-L1.6 |
| L2·C5(c) | after (a): `summary(true)` within 10 s of the manual clock; again after 11 s | — | no label request; one label request | MC-L1.6 (`COUNT_FRESH_MS`) |
| L2·C5(d) | after `list(Printed, Some("oak"), None)` whose counts say `missing_price_tag: 1`: `summary(false)`; then `summary(true)` | — | `Some(1)`; then one unsearched request and `Some(3)` | MC-L1.6 (the box's number follows the search inside the box) |
| L2·C5(e) | `summary(true)` | no key saved; access `Denied` | `None` and no label request for both | ACC.1; M-L6 |
| L2·C5(f) | after (a): `test_connection()`, then `summary(false)` | — | `None` | ACC.2 (counts forgotten) |
| L2·C6(a) | `select_all(MissingPriceTag, None)` | 620 priced, unprinted products with SKUs | two label requests: `…?filter=missingPriceTag&limit=500`, then the same with the first answer's cursor; 620 `Selected`, in the list's order, no repeat | MC-L1.7; 09C §5 Selection; §12.6 |
| L2·C6(b) | same | three products, one with `sku: null` | the two with a SKU | MC-L1.7 |
| L2·C6(c) | `select_all(Printed, Some("oak"))` | 600 printed products matching | every request carries `filter=printed` and `q=oak` | MC-L1.7 |
| L2·C6(d) | `select_all(…)` | `repeat_cursor_next()` (the second answer repeats the first cursor) | `Err(Provider(BadReply(_)))`; at most two label requests were made | MC-L1.7 (never an endless walk) |
| L2·C6(e) | `select_all(…)` | 620 products; the second page fails with 500 | `Err(Provider(Unreachable(_)))` (no partial selection) | MC-L1.7 |
| L2·C6(f) | `select_all(…)` | no key saved | `Err(Off(NoKey))`; no label request | ACC.1; M-L6 |

### Delegated choices
How `list` joins the matched sessions' rows to the label rows · whether `summary`'s small read also updates the copy for its one row (it may) · the fake's cursor text.

### Notes
- Hazard: `ItemsService::adopt` (`items.rs:191`) creates the item and its link but **no session** (the New session dialog calls `createSession` after it); `open_product` must create it. `SessionsService::create` answers the existing session when there is one (`sessions.rs:370`), so a second open is safe.
- Hazard: the fake shop's own product ids are `product-<n>` (`fake_commerce.rs:284`); a product a label row can be matched to or adopted from needs `add_product_with_id("gid://shopify/Product/<digits>", …)` (L1).
- Hazard: `RemoteItems::search_picture` answers none without a commerce integration (`remote.rs:443`) and never downloads; the picture arrives by `run_pending` (the background thread in the app, called by hand in tests).
- Hazard: `SessionsService::page` with an empty `only_ids` list answers no rows, with `None` it answers the first 25 sessions: pass `Some(&ids)` and skip the call when no row matched.
- Hazard: `thumbUrl` comes from Pricing and is fetched by the shop adapter's picture download. Only `https://` addresses are handed on (MC-L1.4).
- `open_product` needs no label access (it reads the shop, not Pricing); it is reachable only from a row the box showed.
- The fake shop has no product without a SKU today (`add_product` always sets one); L2·C4(e) needs one: give `FakeCommerce` the smallest way to make it (first caller that row).

### Unit test commands
`cargo test -p psm-core --lib mechanisms::labels` · `cargo test -p psm-testkit --test suite l2_labels_list::` · `cargo test -p psm-testkit --test suite l1_labels_markers::` · `cargo check --workspace` · `cargo clippy -p psm-core -p psm-infra -p psm-integrations -p psm-testkit --all-targets -- -D warnings`

## Unit 2 — Commands, the page state, the words

**Unit id:** `L2·U2` · **Rows:** L2·C7–C9 · **Changes app files:** yes

### Goal
`labels_list`, `labels_summary`, `labels_select_all`, `labels_open`; the pure `pageStep` / `askCursor` / `canNext` / `pageLabel` of `labelPages.ts`; `formatPrice` and `workFromLine` of `labelWords.ts`.

### Read first
Master plan §4 (Commands, Words, Frontend), **§5 MC-L1.9** · `apps/desktop/src-tauri/src/commands/{labels.rs (L1), sessions.rs (`SessionRowResponse` and its `From`, `session_page` :478)}` · `apps/desktop/src-tauri/src/lib.rs` (`l1_tests`) · `apps/desktop/src/services/labels.ts` (L1) · `apps/desktop/src/features/labels/labelWords.ts` (L1) · `apps/desktop/src/features/conversations/visit.ts` (a pure state function with its unit tests, as the model for `pageStep`).

### Files
New: `apps/desktop/src/features/labels/labelPages.ts`, `apps/desktop/src/features/labels/__tests__/{l2_label_pages.test.ts, l2_label_words.test.ts}`. Edited: `apps/desktop/src-tauri/src/{lib.rs, commands/labels.rs}`, `apps/desktop/src/services/{bindings.ts (generated), labels.ts}`, `apps/desktop/src/features/labels/labelWords.ts`.

### Core mechanisms (the only functions with unit tests)
`labelPages.ts::{pageStep, askCursor, canNext, pageLabel}` · `labelWords.ts::{formatPrice, workFromLine}`.

### Ordered tasks
1. The four commands (thin; one service call each on a blocking worker), their DTOs (`LabelRowDto` carries the matched session as the existing `SessionRowResponse`, `picture_path` as text, `marker` in L1's wire form), registered in `specta_builder()`; `bindings.ts` regenerated; wrappers `labelsList`, `labelsSummary`, `labelsSelectAll`, `labelsOpen`.
2. `labelPages.ts`: state `{cursors: (string | null)[], index: number}` (initial `{cursors: [null], index: 0}`); actions `answer {index, nextCursor, rows}`, `next`, `previous`, `reset`; `pageStep(state, action)` by MC-L1.9 (an `answer` for an index other than the current one is ignored; `rows = 0` on an index above 0 → the initial state); `askCursor(state)` = `cursors[index]`; `canNext(state)` ⇔ `cursors[index + 1]` is a text; `pageLabel(index, count)` = `Page <index + 1> of <max(index + 1, ceil(count / 25), 1)>`; `LABELS_REFRESH_MS`.
3. `labelWords.ts`: `formatPrice(minor)` — whole units = `minor / 100`; the integer part grouped in threes with U+2009 (thin space), then U+00A0 and `kr`; a remainder is shown as `,<two digits>`; `null` → `No price`. `workFromLine(text)` — `YYYY-MM-DD` → `Products created since <d MMM yyyy>`, built from the text's three parts (never through a `Date` made from the text, which shifts by the time zone); anything else → `null`.
4. Tests for the rows.

### Criteria (3 criteria, 19 rows)
Boundary for C7: the command bodies in the desktop crate's `l2_tests` on `TestApp::with_pricing()`. For C8, C9: the functions.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| L2·C7(a) | `labels_list("missing_price_tag", null, null)` | the app of L2·C2(a) plus the unmatched P3 | `{rows: [{product_id: "3", commerce_id: "gid://shopify/Product/3", sku, title, product_type, price_minor, marker: {status: "not_printed", printed_at: null, has_price: true, reset: null}, created_at, session: null, picture_path: null}, {product_id: "1", …, session: {id: s1, …}}], next_cursor: null, counts: {total, has_price, missing_price, missing_price_tag, printed, not_printed}, work_from: "2026-11-19"}` | MC-L1.2; Commands |
| L2·C7(b) | `labels_list(…)` | no key saved; access `Denied` | `CommandError {code: "labels_off", data: {reason: "no_key"}}`; `CommandError {code: "labels_denied", message: "Session Manager isn't allowed to print labels. Ask a Pricing admin."}` | ACC.1; Commands |
| L2·C7(c) | `labels_list(…)` | the fake answers 503 `product_list_unavailable` | `CommandError {code: "labels_list_unavailable", message: "The product list isn't available right now."}` | 09C §10 |
| L2·C7(d) | `labels_summary(true)` | three priced, unprinted products; then no key saved | `{missing_price_tag: 3}`; `{missing_price_tag: null}` | MC-L1.6 (card 1) |
| L2·C7(e) | `labels_select_all("missing_price_tag", null)` | three products, one without SKU | `[{product_id, sku}, {product_id, sku}]` | MC-L1.7 |
| L2·C7(f) | `labels_open("4")` | the shop product of L2·C4(b) | `{session_id: <the new session>}` | MC-L1.8 |
| L2·C8(a) | `pageStep` / `askCursor` / `canNext` | the initial state | index 0; `askCursor` = `null`; `canNext` false | MC-L1.9 |
| L2·C8(b) | same | `answer {0, "c1", 25}`; then `next` | `canNext` true; then index 1 and `askCursor` = `"c1"` | MC-L1.9 |
| L2·C8(c) | same | at index 1: `answer {1, "c2", 25}`, `previous`, `next` | after `previous`: index 0, `askCursor` = `null`; after `next`: index 1, `askCursor` = `"c1"` (the cursor already seen) | MC-L1.9; 09C §5 ("reuses the cursors already seen") |
| L2·C8(d) | same | at index 1: `answer {1, null, 5}` | `canNext` false | MC-L1.9 |
| L2·C8(e) | same | cursors `[null, "c1", "c2", "c3"]`, index 1: `answer {1, "c2b", 25}` | cursors `[null, "c1", "c2b"]`, index 1 | MC-L1.9 (later cursors dropped) |
| L2·C8(f) | same | at index 2: `answer {2, null, 0}` | the initial state | MC-L1.9 (an empty later page) |
| L2·C8(g) | same | at index 0: `answer {2, "x", 25}`; `previous`; `next` while `canNext` is false | the state is unchanged by each | MC-L1.9 |
| L2·C8(h) | same | any state: `reset` | the initial state | MC-L1.9; 09C §5 ("goes to the first page") |
| L2·C8(i) | `pageLabel` | `(1, 101)`; `(0, 0)`; `(5, 30)`; `(0, 25)` | `Page 2 of 5`; `Page 1 of 1`; `Page 6 of 6`; `Page 1 of 1` | MC-L1.9; 09C §5 |
| L2·C9(a) | `formatPrice` | `890000`; `99900`; `0` | `8<U+2009>900<U+00A0>kr`; `999<U+00A0>kr`; `0<U+00A0>kr` | 09C §5 Rows (Price) |
| L2·C9(b) | `formatPrice` | `1234567800`; `890050` | `12<U+2009>345<U+2009>678<U+00A0>kr`; `8<U+2009>900,50<U+00A0>kr` | 09C §5 Rows |
| L2·C9(c) | `formatPrice` | `null` | `No price` | 09C §5 Rows |
| L2·C9(d) | `workFromLine` | `"2026-11-19"`; `"2026-01-05"`; `null`; `"soon"` | `Products created since 19 Nov 2026`; `Products created since 5 Jan 2026`; `null`; `null` | 09C §5 |

### Delegated choices
Whether the cursor stack lives in `LabelsBox` state or a reducer hook · the DTO's field order.

### Notes
- Hazard: `picture_path` and the session's `cover_preview_path` reach the screen only through `previewUrl` (`services/sessions.ts:106`); both are already existing-file paths (M-H4).
- Hazard: a `workFrom` built with `new Date("2026-11-19")` is UTC midnight and reads "18 Nov" west of Greenwich; L2·C9(d) must pass in any time zone.
- The thin space and the no-break space are written as escapes in the code and in the tests, never as invisible literal characters.

### Unit test commands
`cargo test -p psm-desktop` · `cd apps/desktop && npx vitest run src/features/labels` · `npm run typecheck` · `cargo clippy -p psm-desktop --all-targets -- -D warnings`

## Unit 3 — The Labels box on screen

**Unit id:** `L2·U3` · **Rows:** L2·C10–C14 · **Changes app files:** yes

### Goal
The fourth box in `Filters`, `LabelsBox` (toolbar, sub-filters, rows, Previous / Next, the off and error bodies), `LabelRow`, `LabelsBatchBar`, the selection and the refresh.

### Read first
Master plan **§5 MC-L1.9**, §4 Words · book `09C` §5 · `apps/desktop/src/features/sessions/{SessionsPage.tsx (whole), Filters.tsx, SessionRow.tsx, BatchBar.tsx, Pagination.tsx, HoverPreview.tsx (`Thumb`), usePictures.ts, UnassignedCard.tsx, TimeFilter.tsx}` · `apps/desktop/src/features/labels/` (L1, U2) · `apps/desktop/src/features/settings/SettingsNav.tsx` (`openSettingsSection` :17) · `apps/desktop/src/features/conversations/connectionLine.ts` · `apps/desktop/src/lib/{format.ts (`activityLabel` :20), keyboard.ts (`useDebounced`, `useKeyboard`)}` · `apps/desktop/src/features/sessions/__tests__/h2_list_pages_and_pictures.test.tsx` (the mocks and fake timers of a Sessions-page test) · `docs/intention/evidence/design_handoff.md`.

### Files
New: `apps/desktop/src/features/labels/{LabelsBox.tsx, LabelRow.tsx, LabelsBatchBar.tsx}`, `apps/desktop/src/features/labels/__tests__/{l2_labels_box.test.tsx, l2_labels_rows.test.tsx, l2_labels_selection.test.tsx}`. Edited: `apps/desktop/src/features/sessions/{SessionsPage.tsx, Filters.tsx}`, `apps/desktop/src/styles/globals.css`, the existing UI tests that render `SessionsPage` (their `@/services/labels` mock gains `labelsSummary` answering `{missing_price_tag: null}`; **no existing assertion changes**).

### Core mechanisms (the only functions with unit tests)
None (U2's functions are used here).

### Ordered tasks
1. `Filters`: a fourth item `Labels` after `All` (value `labels`), its number given by the page: the last list answer's `counts.missing_price_tag` while in the box, else `labelsSummary`'s **(card 1: one `labelsSummary({refresh: true})` when the Sessions page mounts)**; empty when none.
2. `SessionsPage`: `filter` may be `labels`. In the box the page renders the same search field (placeholder `SKU, title or type`), `Filters`, and `LabelsBox` with the debounced query; it does not render the time filter, "To review", the Unassigned card, the session table, its page control or the publish batch bar. It keeps asking `sessionsPage` for the box it left (so the other boxes' numbers stay current) but renders none of its rows and asks no `labelsStatus` for them.
3. `LabelsBox`: sub-filters `Missing price tag` (default) · `Not printed` · `Printed` · `All` with their counts; the `workFromLine`; the table (labels `Item` · `Price` · `Label` · `Created`, no groups); `Previous` / `Next` and `pageLabel`; the calls, the cursor stack, the refresh throttle and the selection by MC-L1.9; the bodies for `labels_off` (09B's words through `conversationsSettingsGet` + `connectionLine`, `Connect Pricing in Settings → Pricing` when there is none), `labels_denied` (the scope sentence), each with the button `Open Settings → Pricing` (`openSettingsSection('pricing')`); `labels_list_unavailable`, `labels_unreachable` and any other error: the message with `Try again`, the last rows staying on screen.
4. `LabelRow`: with a session — the session's name (`item_type || title`), its SKU, its picture (`usePictures().cover(id) ?? cover_preview_path`), its usual pills (Arranged, research, writer, conversations: the same inputs `SessionRow` gets) and the label pill from the row's own marker, the session's ⋯ (`SessionActionsMenu` with the row's marker and the access), click → `navigate(session.id)`. Without one — Pricing's title, its SKU, the picture `usePictures().search(commerce_id) ?? picture_path` or the placeholder, the label pill, no ⋯ (L3 adds it), click → `labelsOpen(product_id)` then `navigate` (a failure → an error toast with its message, no navigation). Both: `formatPrice(price_minor)`, the marker text in the Label column, `activityLabel(created_at)` or nothing, a tick when the row has a SKU.
5. `LabelsBatchBar`: `<N> selected` (or `All <N> selected` after Select all), `Select all <count>` by the rule of MC-L1.9, `Generate labels for <N>` (**disabled in L2**), the clear button.
6. **(card 1)** Outside the box, `labels_changed` asks `labelsSummary({refresh: true})` (throttled like the box's refresh) and never `labelsList`.
7. Tests for the rows.

### Criteria (5 criteria, 23 rows)
Boundary: `SessionsPage` rendered in UI tests with `@/services/labels`, `@/services/sessions`, `@/services/events`, `@/services/conversations` and the other services mocked, fake timers for the debounce and the throttle (outcome = what is rendered and the exact lists of `labelsList`, `labelsSummary`, `labelsSelectAll`, `labelsOpen` calls). "The standard answer" = 25 rows, `next_cursor: "c1"`, counts `{total: 214, has_price: 180, missing_price: 34, missing_price_tag: 60, printed: 168, not_printed: 46}`, `work_from: "2026-11-19"`.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| L2·C10(a) | `SessionsPage` in Open | `labelsSummary` answers `{missing_price_tag: 12}`; then, on a fresh render, `{missing_price_tag: null}` | a box `Labels` after `All` showing `12`, after exactly one `labelsSummary({refresh: true})` and no `labelsList`; with `null` the box shows no number | MC-L1.6 (card 1); 09C §5 Place; §12.1 |
| L2·C10(b) | click `Labels` | `labelsList` gives the standard answer | exactly one `labelsList({filter: "missing_price_tag", query: null, cursor: null})`; sub-filters `Missing price tag 60` (pressed), `Not printed 46`, `Printed 168`, `All 214`; the labels `Item`, `Price`, `Label`, `Created`; no time filter, no "To review", no Unassigned card, no group header; the `Labels` box shows `60` | MC-L1.9; 09C §5 |
| L2·C10(c) | in the box: click `Printed`; then type `oak` and let the debounce pass | — | `labelsList({filter: "printed", query: null, cursor: null})`; then `labelsList({filter: "printed", query: "oak", cursor: null})` | MC-L1.9; §12.2 |
| L2·C10(d) | in the box | the answer has `work_from: "2026-11-19"`; an answer with `work_from: null` | the line `Products created since 19 Nov 2026`; no such line | 09C §5 |
| L2·C10(e) | click `Labels` | `labelsList` rejects with `labels_off` (`reason: "no_key"`); with `labels_denied` | `Connect Pricing in Settings → Pricing` and a button `Open Settings → Pricing` whose press calls `openSettingsSection('pricing')`; `This key can't read labels. Ask a Pricing admin to add labels:read and labels:write.` with the same button; no row in either | 09C §4; §12.11 |
| L2·C10(f) | in the box after the standard answer | a refresh's `labelsList` rejects with `labels_unreachable` (message `Pricing can't be reached right now.`); then `Try again` is pressed | the message and `Try again` are shown **and the 25 rows are still on screen**; the press makes one more `labelsList` with the same arguments | 09C §10 ("the last list stays on screen") |
| L2·C10(g) | click `Labels` | `labelsList` rejects with `labels_list_unavailable` (message `The product list isn't available right now.`) | that message with `Try again` | 09C §10 |
| L2·C11(a) | a row with a session | `session: {id: 7, item_type: "Chair", sku: "LOC-7", cover_preview_path: "/p/a.jpg", arranged: true}`, `price_minor: 890000`, a printed marker of 4 Nov | the name `Chair`, `LOC-7`, an `<img>` of `previewUrl("/p/a.jpg")`, the pills `Arranged` and `Label printed · 4 Nov`, the price `8 900 kr` (thin space), a Created text; a click calls `navigate(7)` and no `labelsOpen` | MC-L1.9; 09C §5 Rows |
| L2·C11(b) | a row without a session | `title: "Test table"`, `sku: "TEST-1802"`, `commerce_id: "gid://shopify/Product/3"`, `picture_path: null`, marker not printed with a price | `Test table`, `TEST-1802`, the placeholder (no `<img>`), the Label text `Missing price tag`; after `picture_ready {kind: "search", external_id: "gid://shopify/Product/3", path: "/p/s.jpg"}` an `<img>` of `previewUrl("/p/s.jpg")` and no new `labelsList` | MC-L1.4; 09C §5 Rows |
| L2·C11(c) | click the row of (b) | `labelsOpen` answers `{session_id: 31}`; on another render it rejects with message `M` | `labelsOpen({product_id: "3"})` then `navigate(31)`; an error toast `M` and no navigation | MC-L1.8; §12.10 |
| L2·C11(d) | rows | `price_minor: null`; `created_at: null`; `sku: null` | `No price`; an empty Created cell; no tick on that row | 09C §5 Rows, Selection |
| L2·C12(a) | in the box after the standard answer | — | `Page 1 of 3`; `Next` enabled; `Previous` disabled; no page-number button | MC-L1.9; 09C §5 Pages |
| L2·C12(b) | press `Next`; then `Previous` | the second answer has `next_cursor: "c2"` | `labelsList({…, cursor: "c1"})` and `Page 2 of 3`; then `labelsList({…, cursor: null})` and `Page 1 of 3` | MC-L1.9 |
| L2·C12(c) | on page 2: click `Not printed` | — | `labelsList({filter: "not_printed", query: null, cursor: null})` and `Page 1 of …` | MC-L1.9; 09C §5 |
| L2·C13(a) | tick two rows | — | the bar reads `2 selected`; it has `Generate labels for 2`, **disabled**; it has no `Review & publish` | MC-L1.9; 09C §5 Selection |
| L2·C13(b) | tick the header box (25 rows, one without SKU) | — | the 24 rows with a SKU are ticked; the bar reads `24 selected` and offers `Select all 60` | MC-L1.9 |
| L2·C13(c) | press `Select all 60` | `labelsSelectAll` answers 58 entries | exactly one `labelsSelectAll({filter: "missing_price_tag", query: null})`; the bar reads `All 58 selected` | MC-L1.9; §12.6 |
| L2·C13(d) | tick row A on page 1, press `Next`, press `Previous` | — | row A is still ticked and the bar still reads `1 selected` on page 2 and back on page 1 | MC-L1.9 ("survives page changes") |
| L2·C13(e) | with two rows ticked: click `Printed`; again with two ticked: type in the search; again: click `Open`; again: press Escape | — | the selection is empty after each | MC-L1.9 |
| L2·C14(a) | in the box on page 2 with one row ticked: `labels_changed {product_ids: ["1"], access: false}` | — | one `labelsList` with the same filter, query and `cursor: "c1"`; the tick and `Page 2 of 3` are kept | MC-L1.9 Refresh; MC-L4.5 |
| L2·C14(b) | in the box: three `labels_changed` events at 0 s, 0.5 s and 1 s | — | exactly two `labelsList` refreshes: one at 0 s, one at 2 s | MC-L1.9 ("at most once per 2 s") |
| L2·C14(c) | in Open: `labels_changed {product_ids: ["1"], access: false}` | `labelsSummary` then answers `{missing_price_tag: 11}` | one `labelsSummary({refresh: true})`, no `labelsList`; the `Labels` box shows `11` | MC-L1.6 (card 1); §12.8 |
| L2·C14(d) | leave the box (click `Open`) and enter it again | — | entering makes one `labelsList` for the first page (`cursor: null`) | MC-L1.9 Refresh ("on entering the box") |

### Delegated choices
The box's layout and the sub-filter control (design handoff tokens; the same toggle group as the boxes is the smallest) · where `Page k of n` sits between `Previous` and `Next` · the search placeholder's exact words · whether the box keeps its sub-filter when re-entered (it starts on Missing price tag).

### Notes
- Hazard: `SessionsPage`'s `selected` set, `selectable` effect and `BatchBar` belong to the session boxes (`SessionsPage.tsx:88-110`, `:215-221`); the Labels selection is a separate state inside `LabelsBox` keyed by product id. Do not mix them.
- Hazard: a refresh must ask the cursor of the page shown (`askCursor`), not the first page, and must not clear the selection (L2·C14(a)).
- Hazard: the throttle fires at once and then at most once more per window; a plain trailing debounce would delay the first refresh by 2 s and a plain leading one would lose the last event.
- Hazard: the existing Sessions-page tests assert the list of `sessionsPage` calls; entering and leaving the Labels box must not add calls in those tests (they never enter it).
- The Labels box's own number inside the box follows the search (the list answer's counts); outside it is the unsearched number (card 1).

### Unit test commands
`cd apps/desktop && npx vitest run src/features/labels src/features/sessions` · `npm run typecheck`

## Closing gate (once, after the last unit; replaces the template's closing gate)
`cargo test --workspace` · `cargo clippy --workspace --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cd apps/desktop && npm run typecheck` · `npm test`. One cargo command at a time.

## Review mode
The reviewer reads the units in order and runs, once: `cargo test -p psm-core --lib mechanisms::labels` · `cargo test -p psm-testkit --test suite l2_labels_list::` · `cargo test -p psm-desktop` · `cd apps/desktop && npx vitest run src/features/labels src/features/sessions` · clippy on the touched crates, fmt, typecheck. The implementer's full-suite stamp stands. Only B1–B4 block. Also check, by reading: `list`, `summary`, `select_all` and `open_product` never call `run`, `plan`, `preview` or `download`; `open_product` calls only `adopt` and `create`; the list never downloads a picture itself; only `https://` picture addresses are handed to the shop adapter; nothing about a label row is written to the database; core names no platform; the existing Sessions-page tests changed only their mocks. Notes go to `docs/implementation/backlog_labels/L2_labels_box.md`.

## Manual try-out (the narrative is built from this; done by the owner on the installed app, never by a session)
Needs a Pricing key with `labels:read`.
1. Open Sessions. Next to Open / Published / All there is a **Labels** box with the number of labels still to print.
2. Click it. The list shows products with a price and no printed label, newest first, each with picture, SKU, price and "Missing price tag". Under the toolbar: "Products created since <date>".
3. Click Printed: printed products with "Label printed · <date>". Type a SKU in the search box: the list and the numbers follow.
4. With more than 25 products, press Next and Previous; the line reads "Page 2 of N".
5. Click a product you photographed on this Mac: its session opens. Click one from the other Mac: it opens as a product whose photos are on Shopify.
6. Tick two rows: "2 selected". Tick the header box, then "Select all N": "All N selected". (Generate labels comes with the next step.)
7. Print a label in Pricing: within about ten seconds the product leaves Missing price tag and the number drops, without a sound.

## Review log

### 2026-11-23 — implement r1 (opus, implementer)
**Built.** U1: `LinkedItem`, `LabelRow`, `LabelPage`, `Selected`; the constants; `match_product` (6 unit tests); `LabelStore::linked_items` (one SQL read); `LabelsService::{list, summary, select_all, open_product}` by MC-L1.1–.8 (the service now takes `store`, `remote`, `commerce`); `connected()` / `key_changed()` also forget the counts; `FakePricing` `GET /labels/products` (`add_label_products`, `set_work_from`, `repeat_cursor_next`, and `fail_matching_after` for L2·C6(e)); `FakeCommerce::add_product_without_sku`; 36 suite tests `l2_labels_list::`. U2: the four commands and DTOs, bindings, wrappers; `labelPages.ts` (`pageStep`, `askCursor`, `canNext`, `pageLabel`, `LABELS_REFRESH_MS`); `formatPrice`, `workFromLine`; 6 desktop tests `l2_tests`, 13 unit tests. U3: `Filters`' fourth box, `SessionsPage` in the box, `LabelsBox`, `LabelRow`, `LabelsBatchBar`, CSS; 23 UI tests in three files.

**Assumptions (judgment calls).**
1. `open_product` matches by MC-L1.3 with the SKU **the last list answer showed** for that product (kept in memory, product id → SKU, forgotten with the counts); the command carries only the product id, and a row whose SKU matched a session opens that session directly on screen anyway.
2. `summary(refresh)` passes a provider error through (the screen keeps the number it had); `summary` applies its one row to the display copy (delegated: "it may").
3. A connection `Ready` without a store answers `Off(Ready)` (cannot happen in practice; ACC.1 names no other word).
4. The fake's cursor is `o:<offset>` (so a percent-encoded cursor is visible on the wire: `cursor=o%3A25`).
5. Inside the box the search field has its own text (separate from the sessions search), kept when leaving and re-entering; the sub-filter starts on Missing price tag at each entry (delegated).
6. **Try again** asks the first page (MC-L1.9); the selection is kept.
7. The throttle of `labels_changed` counts only events (entering the box does not open a window).
8. Leaving the box asks `labelsSummary({refresh: true})` once, so the number outside the box is the unsearched one again (card 1 note: "outside it is the unsearched number"); the service answers without a request when its count is fresh and unsearched.
9. The marker words appear both as the pill and in the Label column (task U3.4 names both).
10. A matched row's ⋯ gets the access `allowed` (a list answer implies it); the box's empty page reads "No products here".

**Deviations.** (a) `apps/desktop/src/features/labels/useSessionLabels.ts` (outside the U3 file list): with no ids on screen it now clears its remembered ids, so `labels_changed` asks no `labels_status` while the Labels box is shown (U3 task 2: "asks no `labelsStatus` for them"); behaviour with rows on screen is unchanged. (b) `docs/guide/{02_architecture.md, 04_change_impact.md}` also changed in U2 and U3 (commands, frontend), as flow §7A asks, though only U1 lists them. (c) The tracker actor reads `opus` (the session ran on Claude Opus), not `codex` as the prompt template says. (d) `FakePricing::fail_matching_after` is a small test-kit addition not named in the plan's L2 list (the only way to fail the second page of L2·C6(e) while the first passes).

**Observations.** The first run of every new test passed; nothing in the plan contradicted the code. `npm test` once failed `p33_split_grid` C4(d), a known intermittent (`04_change_impact.md` §Fragile spots); green when re-run alone.


### 2026-11-23 — review r1 (opus, pipeline-reviewer): APPROVED_WITH_NOTES
**Perimeter.** `git show --stat` of `e81b52d3` (U1, 13 paths), `70f4d19b` (U2, 10), `80a720ca` (U3, 12), `3fe83622` (close, plan only) = the handoff's Files changed; the paths beyond the unit lists are disclosed deviations (`useSessionLabels.ts`; guide edits in U2/U3; `l1_markers.test.tsx` mock line only). No existing Sessions-page test changed.
**Checks (review mode, once each, tree = `5f371d5f`, clean):** `cargo test -p psm-core --lib mechanisms::labels` 14 ok · `cargo test -p psm-testkit --test suite l2_labels_list::` 36 ok · `cargo test -p psm-desktop` 103 ok · `npx vitest run src/features/labels src/features/sessions` 117 ok · `cargo clippy -p psm-core -p psm-infra -p psm-integrations -p psm-testkit -p psm-desktop --all-targets -D warnings` ok · `cargo fmt --all --check` ok · `npm run typecheck` ok. Implementer's full-suite stamp at `80a720ca` stands (later commits touch docs only).
**Verified correct.** MC-L1.1 (gate → `Off`/`Denied` before any request; `search_text` trim / empty → none / 100 chars; cursor passed through); .2 (rows in provider order, state from the row, `commerce_id_of`); .3 (`match_product`: link first, lowest item id, exact SKU; `linked_items` one SELECT, latest session = `MAX(s.id)`; one `SessionsService::page(All, …, Some(&ids), 1)` only when ids non-empty); .4 (`https://` ∧ `commerce_id` ∧ commerce present → `search_picture`, which only enqueues; no download in `list`); .5 (`copy.start()` before the request, rows applied, counts remembered with `searched`); .6 (card 1: refresh only when nothing remembered ∨ searched ∨ ≥ `COUNT_FRESH_MS`; not allowed → none; ACC.2/.3 forget counts); .7 (500 per page, repeated cursor and 200 pages → `BadReply`, any error → that error, SKU-less rows dropped); .8 (match → latest session or `create`; else `commerce_id_of` → `adopt` → `create`; errors pass through before any write). Reading checks of Review mode: no `run`/`plan`/`preview`/`download` from the four calls; `open_product` calls only `adopt` and `create`; nothing about a label row is written to the DB; core names no platform (H7); commands are one service call on a blocking worker (H8). Screen: `pageStep` per MC-L1.9 (stale index ignored, empty later page → initial, later cursors dropped); `LabelsBox` asks the first page on enter / filter / search / Try again, refresh asks `askCursor` of the page shown and keeps rows, page and selection; throttle leading + one folded trailing call per 2 s; `labels_rejected` / empty later page go back to page 1 once; selection map with Select all replacing it, cleared on filter, search, leaving (unmount) and Escape; Generate disabled. Every C1–C14 row's test observes the row's outcome at its boundary and fails on a wrong one (exact request lines, exact row tuples, exact error variants, exact call lists, rendered text).
**Assumptions** 1–10 read against 09C §5 and MC-L1: none contradicts; all go to the narrative.
**Findings.** None blocking.
**Notes** (`docs/implementation/backlog_labels/L2_labels_box.md`): the box keeps rows after the key goes (open L1 ACC.3 note, → L3/orchestrator); invisible literal spaces in `l2_label_words.test.ts`; the "no To review" sub-check of C10(b) cannot fail in its fixture; `open_product`'s SKU match depends on the SKUs seen since the last connect. L1 backlog notes 3 and 5 (stale `useSessionLabels` ids) are resolved by this phase.
