---
phase: P59_item_type_from_shopify_product_type
title: Item type comes from Shopify's product Type — found items take their product's Type, new items pick one from the shop's existing Types, publish writes it back, and Customize fields presets become per type
difficulty: normal
projection: no
depends_on: [P58_fast_compose_on_camera_preview]
---
# P59 — Item type from Shopify's product Type (temporary source; owner 2026-11-09)

Written by the orchestrator (Opus) on 2026-11-09. The owner: "before we move to the other Mac installation we need to fix this. For now, as a temporary
measure, the item type comes from the Shopify [product] Type; if the item is not found and will be created, fetch the type options there are for the
user to choose one item type from that list. This will change later to the metafield `beyo.item_type`, but for now we use the Shopify product Type field,
not the metafield." Book changelog entry 42.

## Goal
The intention keys **Customize fields** presets by item type (08 §Groups and presets; 05 `items.item_type`, owner U/I, "drives attribute presets"). Today
nothing sets it: local items are created with `item_type = ''` (`apps/desktop/src/services/items.ts:22`, `itemCreate(sku, title, '', null)`) and items found
in the shop also get `''` (`crates/core/src/services/items.rs:~90`). So all items share one preset. After this phase:
1. **Found items:** an item found on Shopify takes its product's **Type** (`productType`) as `item_type`, when found and whenever the app refreshes that
   item from Shopify (search/link, opening the item, publish read-back). Existing linked items with an empty type get it on their next refresh.
2. **New items:** the New Session dialog, when it creates a local item, shows an **Item type** picker listing the shop's existing product Types (fetched
   from Shopify, sorted, searchable; cached for the session of the dialog). Choosing one is required when the list is available. With no commerce
   integration configured (H6), or if the fetch fails, the field is free text (and a failed fetch says `Couldn't load the shop's types — type one`).
3. **Editable later:** the item's type can be changed from the form header (same picker). A change re-scopes the form's preset only for fields the
   photographer has not filled (existing values are never overwritten).
4. **Publish writes it back:** creating a product sets its `productType` to the item type; updating a product whose item type changed in the app updates
   `productType` (inside the existing confirmed publish, H1). An empty item type writes nothing.
5. **Presets per type:** Customize fields saves under the item's type (as the code already keys it). The preset saved so far under the empty type `''`
   becomes the **default**: a type with no preset of its own starts from it (prefill and Customize), and saving creates that type's own preset.

**Not in this phase:** the `beyo.item_type` metaobject source (later, owner), Shopify product category, vendor, collections, any other product field.

## Read first
Book `05_domain_model.md` (items), `08_attributes_and_ai.md` §Groups and presets, `09_integrations.md` (commerce adapter, H7: platform concepts stay in the
adapter), `07A_publish_contracts.md` (create/update steps, H1), `16_changelog.md` entry 42. Code: `crates/core/src/services/{items.rs, attributes.rs,
sessions.rs}`, `crates/core/src/ports/` (commerce port: search hits and the product write), `crates/integrations/src/commerce/shopify/{adapter.rs, graphql.rs,
types.rs}`, `apps/desktop/src-tauri/src/commands/` (item create, presets), `apps/desktop/src/services/{items.ts, attributes.ts, bindings.ts}`,
`apps/desktop/src/features/sessions/NewSessionDialog.tsx`, `apps/desktop/src/features/form/{FormDialog.tsx, CustomizeMode*}`. Tests: P09–P14 and P41
commerce/form tests, Shopify fixtures under `crates/integrations/src/commerce/shopify/fixtures/`.

## Files expected to change (write perimeter)
The files above as needed; `crates/testkit/tests/p59_item_type_from_shopify_product_type.rs` (new); a UI test file
`p59_item_type_picker.test.tsx` (new); new/updated Shopify fixtures (made-up data only); generated bindings. Existing assertions change only where an item
type now appears (list each in the handoff). ≈ 16 paths, ≈ 500 lines.

## Core mechanisms (unit tests allowed)
Preset resolution (own type's preset, else the `''` default).

## The contract
- **Port (generic, H7):** commerce search hits carry an optional `item_type` (adapter: Shopify `productType`); a new port call `item_types() -> Vec<String>`
  (adapter: the shop's distinct product types — use the Admin GraphQL field that lists them, verified against the API version the adapter pins, e.g.
  `productTypes(first: 250)`; page through if needed; empty strings dropped); the product create/update input carries `item_type` (adapter: `productType`).
- **Command:** `item_types_list` → `Vec<String>` (error mapped like other commerce reads); `item_create(sku, title, item_type, barcode)` unchanged signature,
  now given the chosen type; an `item_type_set(item_id, item_type)` command for the form header.
- **Preset resolution:** `preset(type, group)` = own preset if saved, else the `''` preset, else none. `prefill_defaults` uses it.

## Criteria (6 criteria, 8 rows)
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | found item takes the product Type | fake Shopify product SKU `LAMP-1`, `productType: "Lamps"`; search and link | the item's `item_type` is `Lamps` | `[rule]` changelog 42; 05 items |
| C1(b) | refresh fills an empty type | existing linked item with `item_type ''`; its product has Type `Chairs`; open/refresh | `item_type` becomes `Chairs` | `[rule]` changelog 42 |
| C2(a) | type list | fake shop types `Lamps`, `Chairs`, `""`, `Lamps` | `item_types_list` → `["Chairs","Lamps"]` | `[rule]` changelog 42 |
| C2(b) | New Session picker | integration configured; create a new item | the dialog lists the shop's types; Create is disabled until one is chosen; `itemCreate` receives the chosen type | `[rule]` changelog 42 |
| C2(c) | no integration / fetch fails | no commerce integration (or the fetch rejects) | free-text Item type field (with the failure sentence when the fetch failed); `itemCreate` receives the typed text | `[rule]` H6 |
| C3(a) | publish writes the type | local item type `Lamps`, confirmed publish (create) / type changed to `Chairs`, publish (update) | the fake Shopify create input has `productType: "Lamps"` / the update input has `productType: "Chairs"`; an empty type sends no `productType` | `[rule]` 07A; H1 |
| C4(a) | presets per type with the old preset as default | a preset saved under `''` (Details: fields A, B with default A=1); item of type `Lamps` with no own preset | new session of `Lamps` is pre-filled from the `''` preset; saving Customize for `Lamps` creates a `Lamps` preset; a `Chairs` item still gets the `''` preset | `[rule]` 08 §Groups and presets |
| C5(a) | existing suites | everything else | `cargo test --workspace`, `npm test` (known intermittent `p33_split_grid` aside), clippy, fmt, typecheck green | `[rule]` master plan §7.2 |

## Delegated choices
The picker component (searchable select as elsewhere in the form); when exactly a linked item is refreshed (reuse the existing read paths); the form
header's placement of the type control.

## Manual try-out
With the shop connected: New Session → type a SKU that is not in the shop → the Item type list shows your shop's Types (Lamps, …) → pick one → Create.
Open an item that exists in the shop (e.g. the floor lamp) → its type reads `Lamps`. Customize the Details fields for a Lamps item → a Chairs item still
shows the old setup. Publish a new item → in Shopify admin, Product organization → Type shows the chosen type.

## Notes
- **Hard rules:** H1 (writes to Shopify only inside a confirmed publish), H6 (works without Shopify), H7 (`productType` only in the Shopify adapter; core
  sees `item_type`). Tests use fixtures and fakes, never the live store. Default implementer (Codex; owner: no Fable).
- Temporary source: the owner will move the source to the `beyo.item_type` metaobject later; keep the Shopify mapping in one adapter function so the
  later change is local.

## Review log

- **2026-11-09, codex, implement r1.** Built Shopify `productType` reads and writes, paginated `productTypes`, generic commerce type operations, local type editing and refresh, a dialog-cached searchable picker with free-text fallback, and per-type preset fallback. The publish item stores the type frozen at confirmation; the item link stores the last shop type so refresh keeps an unsent local edit. Tests: 5 Rust boundary tests (four in P59 testkit and one Tauri command), two UI boundary tests; full Rust suite 863/863, UI 421/421, clippy, fmt and typecheck green. Assumptions: a shop with zero Types uses free text; a local type edit takes precedence over later shop refreshes until its confirmed publish; default-provenance values may be replaced when the type changes, human values remain. Deviations: a SQLite migration and generic item-link base field were necessary for frozen publish and refresh semantics; existing schema/migration assertions, commerce signature calls and UI service mocks were updated to match those additions. Verified the pinned 2026-10 `productTypes` connection and `ProductSetInput.productType` in Shopify's Admin GraphQL reference. No live Shopify call.

- **2026-11-09, opus, review r1 — CHANGES_REQUESTED.** Gate: tracker IMPLEMENTED, checkpoint `c69c482` present; its 41 committed paths equal the handoff's declared list exactly. Stamp on `c69c482` code tree (HEAD `c87c0df` adds only the review prompt; porcelain clean): `cargo test --workspace` 863 passed / 0 failed, clippy `-D warnings` clean, fmt clean, typecheck clean, `npm test` 421/421. **Verified correct:** C1(a) adapter maps `productType` on all three read paths (VARIANT_SEARCH via `ProductRef` camelCase, TITLE_SEARCH json, PRODUCT_READ `ProductNode` rename) and `search_shop`/`adopt` carry it; C1(b) refresh fills an empty linked type and keeps a local edit (base field); C3(a) create sends `productType` only when non-empty, update writes via `productUpdate` inside the confirmed run with read-back, frozen at confirmation (H1, H4); C4(a) `resolve_preset` own → `''` → none, used by prefill, `presets_get` (relabelled so saving creates the own preset) and form ordering; C2(c) free text + failure sentence, `no_integration` code suppresses the sentence; H7 `productType` only in the Shopify adapter; H8 commands thin. Reviewer probe (scratch test, removed): adapter `item_types` pages with `after`, drops blank, dedupes, sorts → correct. Assumptions: none contradicts the book.
  - **F1 (B1/B3 H6, route: production).** `NewSessionDialog` `openAction` now awaits `refreshItem` for every `origin === 'linked'` local item, and the failure aborts the open. With no commerce integration (`ItemsService::refresh` → `NoIntegration`) or the shop unreachable, an existing linked item can no longer start a session. Reproduced: scratch dialog test with `refreshItem` rejecting → `createSession` never called; control with `refreshItem` resolving → opens. Authority: 01 H6; plan Goal 1 (refresh reuses read paths, best effort). Correction: a failed refresh must not block opening — fall back to the local item (as `FormDialog` already does); add the case to `p59_item_type_picker.test.tsx`. Files: `apps/desktop/src/features/sessions/NewSessionDialog.tsx`, `apps/desktop/src/features/sessions/__tests__/p59_item_type_picker.test.tsx` (optionally `crates/core/src/services/items.rs` so `refresh` returns the local item when no integration is configured).
  - **F2 (B2, route: production).** C2(a)'s test `p59_tests::c2a_item_types_list_returns_sorted_distinct_shop_types` runs through `FakeCommerce::item_types`, which itself sorts, dedupes and drops `""`; the production code that must do this (`ShopifyAdapter::item_types`) is not exercised, so the row's outcome is asserted against the fixture. Production verified correct by reviewer probe. Correction: make the C2(a) test fail on a wrong production outcome — either test `ShopifyAdapter::item_types` through `FakeTransport` with `productTypes` nodes `Lamps, Chairs, "", Lamps` (as C1(a)/C3(a) already do), or move sort/dedupe/blank-drop into `ItemsService::item_types` and make the fake return the raw list. Files: `crates/testkit/tests/p59_item_type_from_shopify_product_type.rs` and/or `apps/desktop/src-tauri/src/lib.rs` (p59_tests), `crates/testkit/src/fake_commerce.rs`, optionally `crates/core/src/services/items.rs`.
  - Notes → backlog (P59 reviewer r1): C2(b) listing clause unasserted; NULL type base lets a first refresh overwrite a local edit; local-origin published items never refresh on open; header picker commits free text while the type list is loading; native `<datalist>` instead of the form's searchable select.

- **2026-11-09, codex, fix r2.** Resolved F1 by falling back to the stored linked item if `refreshItem` fails; the P59 dialog test asserts that session creation still uses the local item. Resolved F2 by testing `ShopifyAdapter::item_types` through `FakeTransport` with `Lamps, Chairs, "", Lamps`, asserting `Chairs, Lamps`; removed the fake-backed Tauri test that could only confirm the fake's filtering. No divergences. Stamp: Rust workspace 863 passed / 0 failed; `cargo clippy --workspace --all-targets -- -D warnings` clean; `cargo fmt --all -- --check` clean; `npm run typecheck` clean; `npm test` 59 files / 422 passed. First full UI run hit the known intermittent P33 split-grid C4(d) failure; that file passed alone (7/7), and the full UI rerun passed. No live Shopify call.

- **2026-11-09, opus, re-review r2 — APPROVED_WITH_NOTES.** Gate: tracker IMPLEMENTED with fix note; fix checkpoint `a5ce4ab` present; HEAD `4ba8bf5` = fix + re-review prompt, porcelain clean. Perimeter: `git show --stat a5ce4ab` = 7 paths = fix handoff's list, all within r1's allowed files (NewSessionDialog + its P59 UI test, P59 testkit file, lib.rs `mod p59_tests` removed, own tracker row/Review log/handoff). Evidence (one run): `cargo test --workspace` 863/0 (−1 removed fake-backed test, +1 adapter test), clippy `-D warnings` clean, fmt clean, typecheck clean, `npm test` 59 files / 422 passed. **F1 resolved:** linked-item refresh in `openAction` is best effort (try/catch → stored item), same pattern as `FormDialog`; adopt of shop-only hits still surfaces errors (correct); new dialog test (refresh rejects → `createSession(7)`, `onOpened(3)`) fails on the pre-fix code (r1's scratch reproduction). **F2 resolved:** C2(a) drives `ShopifyAdapter::item_types` through `FakeTransport` (`ProductTypes` nodes `Lamps, Chairs, "", Lamps` → `Chairs, Lamps`); sort/dedupe/blank-drop now asserted on production (`BTreeSet` + `trim().is_empty()` filter); no fake-backed C2(a) test remains. Changed seam read adversarially: nothing wrong. Triage: no blocking items, no MUST. r1's 5 notes remain in backlog; no new notes. Narrative written.
