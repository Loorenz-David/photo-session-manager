---
plan: P65_capture_sync_waiting_jobs_and_inventory
role: implement
round: 1
state: DONE
date: 2026-11-11
actor: codex
---
# P65 implement r1 — handoff

Run by Claude Opus (pipeline-implementer agent) in the `codex` actor slot. All 9 rows have a passing test at their boundary;
suite, clippy, fmt, typecheck and UI suite green (stamp below). **Read "Deviations" first: another session committed most of
this phase's code while it was being written.**

⚠ OWNER DECISIONS REQUIRED (0) — none.

## What was built
- **Photos reach Capture One in seconds.** When the app files a photo into an item's folder (FTP, cable download, import), it asks
  Capture One to synchronise that folder (`synchronizing = true` on the folder's collection, new `jxa/sync_folder.js`). At most one
  sync per folder per second; photos arriving meanwhile get one more sync after it. While "Take the camera back" holds Capture One
  the sync is refused like every script and is sent at the first tick after. Logs: `capture_one.sync folder=… ms=…` and
  `capture_one.variant_seen raw=… after_ms=…` (from filing to Capture One listing the photo).
- **Publishes and exports wait for Capture One.** A full-size render Capture One cannot take yet (photo not loaded, busy, closed, or
  timed out) no longer fails the run: the run waits (`Waiting for Capture One…` in the publish popover and as the session row's
  label), retries after 1, 2, 4, 8, then every 15 s, and fails only after 10 minutes with `Capture One has not loaded
  DSC05162.ARW after 10 minutes` (busy: `Capture One did not render … within 10 minutes`; closed: `Capture One was not available to
  render … for 10 minutes`). Any other error fails at once as before. Publish pressed before previews exist is accepted and waits.
  Cancel while waiting ends with `Publish cancelled` and stops the export.
- **Stock on create.** The item form's Product step (new items only) shows Location (the shop's active locations, read once per app
  run) and Quantity (default 1), prefilled from the remembered default; "No location" → `Choose a location to track stock` and the
  product is created untracked. The choice is frozen at confirmation; the create sends `tracked: true` and the quantity available at
  the location in the same `productSet`, reads it back, and the values become the new default (`commerce.inventory_default` in
  settings.json), editable in Settings → "Stock for new products". Found products and later publishes never send stock fields. A
  shop app without `read_locations`/`write_inventory` shows `Your Shopify app needs the read_locations and write_inventory
  permissions`; the create is retried without stock fields and the publish does not fail.

## Tests → criterion rows
| Row | Test (boundary) | Assertion shape |
|---|---|---|
| C1(a) | `p65…::c1a_filed_photo_is_synced_and_styled_within_the_sync_delay` (FTP arrival → fake Capture One ops, store, log) | exactly 1 `sync_folder:<item folder>`; `apply_style` before `process`; preview asked 3–4 s after filing (fake lists 3 s after sync, else 60 s); preview path stored; `capture_one.variant_seen raw=<raw> after_ms=3000\|4000` and `capture_one.sync folder=<folder> ms=` logged — exact |
| C1(b) | `p65…::c1b_syncs_are_coalesced_per_folder_and_held_back_by_the_take_back` (fake ops) | 3 arrivals in 120 ms → 1–2 syncs; during a held gate 0 syncs over 3 s, exactly 1 after release — as the row states (≤ 2) |
| C2(a) | `p65…::c2a_publish_waits_for_capture_one_then_completes` (publish service, session list, fake shop) | step stays Prepare for 20 s; activity text `Waiting for Capture One…`, no error, cancellable; row label `WaitingForEditor`; after the variants appear: Done, gallery = both photos once in order, row Published — exact. UI: `p65_stock_and_waiting.test.tsx` "C2(a)…" (pill + popover text) |
| C2(b) | `p65…::c2b_the_wait_ends_after_ten_minutes_and_other_errors_fail_at_once` | still Prepare at 9 min 45 s; then `(Failed, "Capture One has not loaded DSC05164.ARW after 10 minutes")`, row PublishFailed; a `process failed` refusal → `(Failed, "provider: process failed")` within 2 s — exact |
| C2(c) | `p65…::c2c_cancel_while_waiting_for_capture_one` | `(Cancelled, None)`, text `Publish cancelled`, row PublishCancelled, no export run in flight — exact |
| C3(a) | `p65…::c3a_created_product_is_tracked_with_the_quantity_at_the_location_and_remembered` (ShopifyAdapter over FakeTransport; publish over FakeCommerce; settings.json) | ProductSet variant carries `inventoryItem {tracked: true}` and `inventoryQuantities [{locationId: L1, name: available, quantity: 1}]`; one `InventoryReadback`; outcome `Tracked{verified: true}`; publish sends exactly one stocked create with L1/1; default and settings.json = L1/1 — exact |
| C3(b) | `p65…::c3b_found_products_and_republishes_never_touch_stock` | found product: no stock request; created product republished: still exactly 1 stock request, 1 create; adapter plain create sends no `tracked`/`inventoryQuantities` — exact |
| C3(c) | `p65…::c3c_form_default_settings_and_a_shop_without_location_access` + UI `p65_stock_and_waiting.test.tsx` (4 "C3(c)" cases) | locations read once; item prefilled from Settings default; "No location" → untracked create; negative quantity refused; no access → the sentence, publish Done untracked; adapter: Locations ACCESS_DENIED → sentence, ProductSet ACCESS_DENIED → resent without stock fields, `Untracked{reason: sentence}`. UI: Location/Quantity prefilled, change saved for the item, `Choose a location to track stock`, sentence in form and Settings, Settings save — exact |
| C4(a) | full stamp below | — |

No test outside these rows was added. Inherited tests changed (rule changed by changelog 45, not weakened): p09 `c6b` (a timed-out
master now waits and the export fails 10–15 min later, asked again meanwhile), p29 `failed_run` helper (reaches the failure the same
way), p01 schema list (+`item_stock`, `publish_item_stock`).

## Assumptions
1. Waiting is a **flag** on the export run (not a new run state): `ExportState::Waiting` event, the run row's `jobs.error =
   'waiting_for_editor'` while running, `RowLabel::WaitingForEditor` while publishing, popover text in the prepare step.
2. Back-off 1, 2, 4, 8, then 15 s. The 10 minutes count from the photo's first wait and survive retries until its master arrives (a
   render that keeps timing out still ends; such a run fails 10–14 min in, at its next timeout).
3. "Capture One not ready" = `variant_not_found`, `editor_launch_deferred`, `document_not_open`, "not running", -600 (unavailable);
   `timeout`, "timed out", -1712, "busy" (busy). Only master renders of exports/publishes wait; preview renders keep their own retry;
   the app's own output renders (`render_output`) still fail at once (not Capture One).
4. The publish job's retry back-off is capped at 15 s (so a publish notices its export within 15 s, and a cancel lands within 15 s).
5. Location/Quantity sit at the end of the Product step; shown for items not linked to the shop (an item that will be *found* by SKU
   also shows them — its stock is simply never sent).
6. The stock is frozen at confirmation (H1) in `publish_item_stock`; the per-item form choice lives in `item_stock` (empty location =
   "No location" chosen, which overrides the default).
7. The default is remembered only after a create that was tracked (not after an untracked fallback).
8. Read-back = a separate `InventoryReadback` query; a mismatch is logged, never a failure.
9. Locations are read once per built app (a settings save that rebuilds the app reads them again); a failed read is retried next time.
10. On a refused stocked `productSet` (GraphQL error or userErrors with no product) the product is created again without stock fields;
    an HTTP/transport error is not retried (outcome unknown, MC-11).
11. User-facing sentences naming Capture One are built in core, following the existing precedent (previews, editing startup).

## Deviations
- **Commits by another session.** While this session worked, commits `b468cad` ("backlog: show the Mac's address in the FTP steps;
  easier FTP password", 08:50) and `8b26018` ("STATUS: work Mac installed and in use…", 09:08), authored by the orchestrator side,
  swept most of this phase's uncommitted production code into them (34 and 3 of this phase's paths). `b468cad` references
  `jxa/sync_folder.js` and `011_item_stock.sql`, which it did not contain, so **`b468cad` and `8b26018` do not build**; `b0edd78` makes
  the tree whole. No history was rewritten. The phase's perimeter is therefore the union below across the three commits.
- New files beyond the plan's list: `crates/infra/src/sqlite/migrations/011_item_stock.sql`, `apps/desktop/src/services/stock.ts`,
  `apps/desktop/src/features/form/StockFields.tsx`, `apps/desktop/src/features/settings/StockSection.tsx`,
  `apps/desktop/src/features/form/__tests__/p65_stock_and_waiting.test.tsx`.
- Synthetic Shopify replies (locations, stocked create, read-back, ACCESS_DENIED) are inline in the test file, not fixture files.
- The Shopify docs search tool was refused in this session (not permitted); the fields used are `ProductSetInput.variants[].inventoryItem.tracked`,
  `.inventoryQuantities[{locationId, name: "available", quantity}]`, `locations(first: 50) { nodes { id name isActive } }`,
  `inventoryItem { tracked inventoryLevel(locationId:) { quantities(names: ["available"]) { name quantity } } }` on API 2026-10, from the
  Admin API as known — unverified against a shop; the owner's store is the first real test.
- New registry names (for the master plan §6.7/§6.8, not edited here): commands `stock_locations`, `stock_default_get`, `stock_default_set`,
  `item_stock_get` (→ `{new_item, choice}`), `item_stock_set`; settings key `commerce.inventory_default`; error code `editor_wait_expired`;
  row label `waiting_for_editor`; export state `waiting`; tables `item_stock`, `publish_item_stock`.
- No live probe: Capture One was not running on this Mac, and the session did not launch it.

## Files changed
Production (in `b468cad`/`8b26018` and/or `b0edd78`): `crates/core/src/{error.rs, events.rs, domain/publish.rs, domain/session.rs,
mechanisms/backoff.rs, ports/commerce.rs, ports/editing.rs, ports/store.rs, services/export.rs, services/previews.rs, services/publish.rs,
services/sessions.rs}`, `crates/infra/src/{jobs.rs, settings_file.rs, sqlite/store.rs, sqlite/migrations/011_item_stock.sql}`,
`crates/integrations/src/{lib.rs, commerce/shopify/adapter.rs, commerce/shopify/graphql.rs, editing/capture_one/{adapter.rs, follow.rs,
gate.rs, jxa.rs, scripting.rs, jxa/sync_folder.js}}`, `apps/desktop/src-tauri/src/{lib.rs, commands/items.rs, commands/publish.rs,
commands/sessions.rs}`, `apps/desktop/src/{services/bindings.ts, services/stock.ts, features/form/FormDialog.tsx, features/form/StockFields.tsx,
features/settings/SettingsPage.tsx, features/settings/StockSection.tsx, features/sessions/StatusPill.tsx}`.
Test support: `crates/testkit/src/{fake_capture_one.rs, fake_commerce.rs, test_app.rs}`.
Tests: `crates/testkit/tests/p65_capture_sync_waiting_jobs_and_inventory.rs` (new), `apps/desktop/src/features/form/__tests__/p65_stock_and_waiting.test.tsx`
(new), `crates/testkit/tests/{p01_storage_identity.rs, p09_local_export.rs, p29_master_and_output_render.rs}`.
Docs: `docs/implementation/master_plan.md` (P65 tracker row only), `docs/implementation/plans/P65_capture_sync_waiting_jobs_and_inventory.md`
(Review log). No mutation probes (regime off).

## Checkpoint SHA
`b0edd78` (clean tree after it).

## Stamp
| Command | Result |
|---|---|
| `cargo test --workspace --no-fail-fast` (tree = `b0edd78` minus the p29 helper edit) | 919 passed, 3 failed: p29 `c1c`/`c7b` (old timeout rule in a helper; fixed after that run compiled) and p43 `c7b_port_reopens_in_place` ("did not settle within 15 s" in a 287 s, heavily loaded run) |
| `cargo test -p psm-testkit --test p29_master_and_output_render --test p43_ftp_receiver` (final tree) | 26/26 and 26/26 (p43 in 4.4 s) |
| `cargo test -p psm-testkit --test p65_capture_sync_waiting_jobs_and_inventory` / `--test p09_local_export` | 8/8, 8/8 |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo fmt --all -- --check` | ok |
| `npm run typecheck` | ok |
| `npm test` | 65 files, 447/447 |
| `tools/sidecar/build.sh --stub --test` | exit 0, ctest 100 % |
| `cargo test -p psm-desktop export_bindings` | bindings.ts regenerated and committed |

## Manual try-out
1. Open the app with Capture One running on the session, activate an item, take a photo with the camera over Wi-Fi (FTP). In
   `~/Library/Logs/…/psm.<date>.log` look for `capture_one.sync folder=…` and then `capture_one.variant_seen … after_ms=…`
   (expected a few seconds instead of 17–80 s); the style and preview follow right after.
2. Take two more photos and press Publish at once: the session row reads "Waiting for Capture One…"; hovering it shows the same in
   the popover; when the previews arrive the publish continues and completes. Cancel publish while it waits → "Publish cancelled".
3. Open the form of a new item → Product step: Location (your shop's locations) and Quantity (1). Pick a location, publish; in Shopify
   the product's inventory is tracked with that quantity at that location. Settings → "Stock for new products" now shows that choice.
4. If the shop app lacks the permissions, the form and Settings say "Your Shopify app needs the read_locations and write_inventory
   permissions" and the product is still created (untracked).
