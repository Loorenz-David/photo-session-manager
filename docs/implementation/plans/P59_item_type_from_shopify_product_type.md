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
