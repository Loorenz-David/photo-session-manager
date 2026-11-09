---
plan: P59_item_type_from_shopify_product_type
role: implement
round: 1
state: DONE
date: 2026-11-09
actor: codex
---
# P59 implementer handoff

## What was built

Item type now comes from Shopify's Product Type for found and refreshed items. New sessions can choose a shop type in a searchable picker; when the shop is disconnected, unavailable or has no types, they can type one. The form header can edit the type. Customize fields uses a preset for that type, falling back to the old empty-type preset. A confirmed publish creates or updates the Shopify Product Type; an empty type sends no field. The confirmed type is frozen in the publish item, and a shop-type base on the item link preserves unsent local edits during refresh.

## Tests → criterion rows

| Row | Boundary proof |
|---|---|
| C1(a) | `c1a_found_item_takes_product_type` — adapter search Type and linked item Type = Lamps |
| C1(b) | `c1b_refresh_fills_linked_empty_type` — empty linked type fills, later shop change refreshes, local edit persists |
| C2(a) | `p59_tests::c2a_item_types_list_returns_sorted_distinct_shop_types` — Tauri command returns Chairs, Lamps |
| C2(b) | UI `C2(b) shop types are searchable and a listed choice is required` — Create gated; chosen type sent |
| C2(c) | UI `C2(c) absent integration permits a typed type` — free text and failed-fetch sentence |
| C3(a) | `c3a_confirmed_publish_creates_and_updates_product_type` — create, update, empty omission and confirmation freeze |
| C4(a) | `c4a_empty_type_preset_is_default_and_own_save_is_independent` — fallback, per-type save and filled-value preservation |
| C5(a) | Workspace stamps below — all suites and static checks green |

## Assumptions

- A connected shop with zero existing Types uses free text, as there is no option to choose.
- The shop's changed Type replaces a linked item's type when the local value still equals the last shop Type. A locally edited type stays until the next confirmed publish.
- Changing an item type replaces default-provenance prefill values; human-filled values stay intact. The picker requires an exact shop option when options exist.
- The Form header commits a selected shop option immediately and free text on blur.

## Deviations

- Added SQLite migration 009 and a generic `item_type_base` link field outside the expected path list. They persist the type frozen at confirmation and distinguish a shop refresh from an unsent local edit across restarts; both are needed for H1/H4 and the phase goal.
- Updated earlier commerce call sites, schema/migration test expectations, and UI service mocks for the new port and command. No earlier test assertion was weakened.
- Shopify's 2026-10 Admin GraphQL reference confirms the paginated [`productTypes`](https://shopify.dev/docs/api/admin-graphql/2026-10/queries/productTypes) connection and [`ProductSetInput.productType`](https://shopify.dev/docs/api/admin-graphql/2026-10/input-objects/ProductSetInput). No live shop request was made.

## Files changed

- `apps/desktop/src-tauri/src/commands/items.rs`
- `apps/desktop/src-tauri/src/lib.rs`
- `apps/desktop/src/features/compose/__tests__/harness.tsx`
- `apps/desktop/src/features/form/FormDialog.tsx`
- `apps/desktop/src/features/form/ItemTypePicker.tsx`
- `apps/desktop/src/features/sessions/NewSessionDialog.tsx`
- `apps/desktop/src/features/sessions/__tests__/p07_shell_sessions_ui.test.tsx`
- `apps/desktop/src/features/sessions/__tests__/p13_publish_runs.test.tsx`
- `apps/desktop/src/features/sessions/__tests__/p18_item_search.test.tsx`
- `apps/desktop/src/features/sessions/__tests__/p59_item_type_picker.test.tsx`
- `apps/desktop/src/features/shell/__tests__/p19_navigation.test.tsx`
- `apps/desktop/src/features/workspace/__tests__/p08_workspace_ui.test.tsx`
- `apps/desktop/src/features/workspace/__tests__/p19_drop.test.tsx`
- `apps/desktop/src/features/workspace/__tests__/p20_drop_continuity.test.tsx`
- `apps/desktop/src/features/workspace/__tests__/p22_workspace_tidy.test.tsx`
- `apps/desktop/src/features/workspace/__tests__/p26_double_drag.test.tsx`
- `apps/desktop/src/services/bindings.ts`
- `apps/desktop/src/services/items.ts`
- `crates/core/src/domain/item.rs`
- `crates/core/src/domain/publish.rs`
- `crates/core/src/ports/commerce.rs`
- `crates/core/src/ports/store.rs`
- `crates/core/src/services/attributes.rs`
- `crates/core/src/services/items.rs`
- `crates/core/src/services/publish.rs`
- `crates/infra/src/sqlite/migrations/009_publish_item_type.sql`
- `crates/infra/src/sqlite/store.rs`
- `crates/integrations/examples/shopify_probe.rs`
- `crates/integrations/src/commerce/shopify/adapter.rs`
- `crates/integrations/src/commerce/shopify/graphql.rs`
- `crates/integrations/src/commerce/shopify/types.rs`
- `crates/testkit/src/fake_commerce.rs`
- `crates/testkit/tests/p01_storage_identity.rs`
- `crates/testkit/tests/p12_shopify_adapter.rs`
- `crates/testkit/tests/p16_operations.rs`
- `crates/testkit/tests/p18_item_search.rs`
- `crates/testkit/tests/p27_retire_shape_on_arrival.rs`
- `crates/testkit/tests/p59_item_type_from_shopify_product_type.rs`
- `docs/implementation/handoffs/implementer/handoff_P59_item_type_from_shopify_product_type_implement_r1.md`
- `docs/implementation/master_plan.md`
- `docs/implementation/plans/P59_item_type_from_shopify_product_type.md`

## Checkpoint SHA

This handoff is in the single checkpoint commit. Resolve its SHA with `git log -1 --format=%H -- docs/implementation/handoffs/implementer/handoff_P59_item_type_from_shopify_product_type_implement_r1.md` (a commit cannot contain its own SHA).

## Stamp

| Command | Result |
|---|---|
| `cargo test --workspace` | PASS, 863 tests, 0 failures |
| `cd apps/desktop && npm test` | PASS, 421 tests, 0 failures |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo fmt --all -- --check` | PASS |
| `cd apps/desktop && npm run typecheck` | PASS |

## Manual try-out

1. Connect the shop, open New Session, search for a SKU that is not in the shop, choose Create, pick a listed Item type, then create the session.
2. Open a product that exists in the shop; its Product Type should appear as Item type. Change it in the form header, close and reopen the form, and check that the edit remains.
3. Customize the Details fields for that type. Open an item of another type and check that its preset remains separate. An item with no own preset should start from the old default.
4. Confirm and publish the new item. In Shopify Admin, inspect Product organization → Type for the chosen value. This is the only step that writes to the shop.
5. Disconnect the shop and create another session. The Item type field should accept free text.

## ⚠ OWNER DECISIONS REQUIRED (0)

None.
