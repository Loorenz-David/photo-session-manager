---
phase: P65_capture_sync_waiting_jobs_and_inventory
title: Photos reach Capture One in seconds (the app asks it to sync the folder), publishes and exports wait for Capture One instead of failing, and a new Shopify product gets inventory tracking with a chosen location and quantity that the app remembers
difficulty: high
projection: no
depends_on: [P63_final_check_fixes]
---
# P65 — Capture One sync on arrival, waiting jobs, inventory on create (owner 2026-11-11, changelog 45)

Written by the orchestrator (Opus) on 2026-11-11 from the owner's first publish at the work site with the P53–P64 SDK build.

## Evidence (log `psm.2026-11-11.log`, dev Mac)
- DSC05161 filed 07:10:19; first render try 07:10:36 `variant_not_found`; preview ~07:10:52 (≈ 33 s). DSC05162–64 filed 07:10:48–52; `variant_not_found`
  until ≈ 07:12:03; previews 07:12:09 (≈ 75 s). Once Capture One lists the variant, style + preview take < 0.5 s. 2026-11-10: DSC05103 > 60 s.
- The app never asks Capture One to synchronise (`grep synchroniz crates/integrations/src/editing/capture_one` is empty). Capture One's sdef: a collection's
  `synchronizing` "can be set to true for a session folder collection in order to initiate background synchronization".
- Publish pressed 07:11:26 (35 s after the last arrival): product created, then `export failed run=180 reason="provider"` → `Publish failed · variant_not_found`.
  `export.rs` ~727: a `render_output` job that cannot render fails the run, no retry. Retry at 07:12:17 succeeded in 13 s (P64's grouped calls all ok).
- Products are created with inventory tracking off and no quantity (`PRODUCT_SET`, `adapter.rs`); no inventory or location code exists.

## Goal
1. **Sync on arrival (10A).** Right after a photo is filed into an item folder (FTP, cable/SDK download, import), the app sets `synchronizing = true` on that
   folder's collection in the configured document (new `sync_folder.js` via the gate; refused like any script during a take-back — then done right after).
   Coalesce: at most one sync per folder per second; arrivals during a sync trigger one more after it. The variant follower keeps its current polling; the
   style and preview follow as soon as the variant appears. Log `capture_one.sync folder=… ms=…` and, when the variant first appears,
   `capture_one.variant_seen raw=… after_ms=…` (arrival → seen), so the owner's next session measures it.
2. **Jobs wait instead of failing (07/07A).** In export and publish, a render that meets `VariantNotFound`, `EditorLaunchDeferred`, Capture One busy/closed,
   or a timeout is **not a failure**: the run goes to a waiting state shown as `Waiting for Capture One…` (popover step, session row) and retries with
   back-off (1 s, 2 s, 4 s … capped at 15 s). Only after **10 minutes** of waiting does it fail, with `Capture One has not loaded DSC05162.ARW after 10 minutes`
   (or the matching reason). Other errors fail as today. A Publish pressed while previews are still pending is accepted and queues (the row shows
   `Waiting for Capture One…`), instead of being refused or failing. Cancel (P64) works while waiting. Preview renders keep their own retry.
3. **Inventory on create (07A/09).**
   - The item form (Shopify step, new items only) shows **Location** (drop-down of the shop's locations, read once per app run, `locations(first: 50)`
     active only) and **Quantity** (integer ≥ 0, default 1).
   - Their initial values are the remembered default (settings `commerce.inventory_default { location_id, location_name, quantity }`); after a successful
     create, the values used become the new default. Settings → Shopify shows and edits the default (location drop-down + quantity).
   - When the app **creates** a product: the variant's inventory item has `tracked: true` and `inventoryQuantities: [{ locationId, name: "available",
     quantity }]` in the same `productSet` (or the documented equivalent). Read back and verify tracked + quantity at that location.
   - Products **found** in the shop, and every later publish/republish of a created product, never change tracking, locations or quantities.
   - No default and no location chosen → the product is created as today (untracked) and the form says `Choose a location to track stock`. Shop without
     the needed access (`read_locations` / `write_inventory`) → the location list says `Your Shopify app needs the read_locations and write_inventory
     permissions` and creation proceeds untracked; never a failed publish because of inventory.

## Read first
Changelog 45, 44; book 07A (publish contracts), 09A, 10A (MC-18 arrival/variant, follower). P63 narrative (the script gate), P64 narrative (publish
progress/cancel). Code: `crates/integrations/src/editing/capture_one/{gate.rs, follow.rs, jxa.rs, jxa/*.js, adapter.rs}`, `crates/core/src/services/{export.rs,
publish*.rs, previews.rs, arrival.rs, camera_ledger.rs, settings.rs}`, `crates/integrations/src/commerce/shopify/{adapter.rs, graphql.rs, types.rs}`,
`apps/desktop/src/features/{form, publish, settings}`. Tests: p63, p64, p62, p44 testkit files; publish UI tests; Shopify fixtures (`fixtures/sv7_*`).

## Files expected to change (write perimeter)
The files above as needed; `crates/integrations/src/editing/capture_one/jxa/sync_folder.js` (new); `crates/testkit/tests/p65_capture_sync_waiting_jobs_and_inventory.rs`
(new); fake Capture One (`synchronizing` + a configurable delay before a filed photo becomes a variant) and fake Shopify (locations, inventory in productSet);
form/settings UI + tests; bindings. ≈ 25 paths, ≈ 800 lines.

## Criteria (7 criteria, 9 rows)
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | sync on arrival | fake Capture One lists a filed photo only after `synchronizing = true` on its folder (else after 60 s) | `sync_folder` called once for the folder; style applied and preview rendered within the fake's sync delay, not 60 s; log `capture_one.variant_seen … after_ms` | `[rule]` changelog 45 (1) |
| C1(b) | coalesced, gated | 3 arrivals in one folder within 200 ms; one during a take-back | ≤ 2 syncs for the folder; none while the take-back holds the gate, one right after it | `[rule]` changelog 45 (1); MC-34 |
| C2(a) | publish waits | publish pressed while 2 photos are not yet variants; they appear after 20 s (fake clock) | run not failed; step `Waiting for Capture One…`; publish completes after they appear, product has all photos once | `[rule]` changelog 45 (2) |
| C2(b) | waits are bounded | a photo never becomes a variant | after 10 min (fake clock) the run fails with `Capture One has not loaded <name> after 10 minutes`; a non-Capture-One error still fails at once | `[rule]` changelog 45 (2) |
| C2(c) | cancel while waiting | cancel during the wait | `Publish cancelled` (P64 semantics) | `[rule]` changelog 44 (2) |
| C3(a) | inventory on create | new item, location L1, quantity 1 | productSet carries tracked true + available 1 at L1; read-back verified; default saved = L1/1 | `[rule]` changelog 45 (3) |
| C3(b) | never on found/republish | found product; republish of a created product | no inventory/tracking fields in any request | `[rule]` changelog 45 (3) |
| C3(c) | form + settings + no access | form shows Location/Quantity prefilled from the default; Settings edits it; shop without location access | sentences as above; create proceeds untracked; publish not failed | `[rule]` changelog 45 (3) |
| C4(a) | existing suites | everything else | `cargo test --workspace`, `npm test` (known intermittent `p33_split_grid` aside), clippy, fmt, typecheck, sidecar stub tests green | `[rule]` master plan §7.2 |

## Delegated choices
Exact place of the waiting state in the job model (new run state vs. a flag), the back-off steps within the stated bounds, where Location/Quantity sit in
the Shopify step, the read-back shape. Record each in the handoff.

## Notes
- **Hard rules:** the suite never needs Capture One, the camera or a real shop; no real Shopify writes in tests; never kill/quit any process except Capture
  One through the take-back; never delete photos. Default implementer (Codex; Claude when Codex is exhausted).
- The Shopify GraphQL shapes (`productSet` inventory fields, `locations`) must follow the 2025-xx Admin API version the adapter already uses; check with the
  docs, record the exact fields in the handoff. The owner's real store is the first real test.

## Review mode (owner 2026-11-10: fast; kept for this phase)
- **Implementer:** high effort; run the full closing gate once (`cargo test --workspace`, clippy, fmt, typecheck, `npm test`) and put the stamp in the handoff.
- **Reviewer (time-boxed, still correct):** do not re-run `cargo test --workspace`. Run this phase's new test file(s), the test files of the crates and features the diff touches, clippy, fmt, typecheck and the touched UI test files; trust the implementer's full-suite stamp unless it is missing or contradicts what you see. Read every changed production path against the rows; only B1–B4 findings block. Aim to finish in about 15 minutes.

## Review log

### 2026-11-11 — implement r1 (actor `codex` slot; run by Claude Opus, pipeline-implementer)
**Built.**
- *Sync on arrival (10A).* New `CaptureOneScripting::sync_folder` (`jxa/sync_folder.js`: `collection.synchronizing = true` on the folder's
  collection), `EditingProvider::sync_folder` (default no-op), gated as a launch-capable script (`GatedScripting::launching`: refused during a
  take-back). `CaptureOneFollow::photo_arrived` marks the folder pending, switches the collection, then `run_syncs`; `tick` runs pending syncs.
  At most one sync per folder per `SYNC_MIN_INTERVAL_MS` (1 s); arrivals meanwhile → one more sync after it; a refused (`EditorLaunchDeferred`)
  sync stays pending and goes at the first tick after the take-back. Logs `capture_one.sync folder=… ms=…` and
  `capture_one.variant_seen raw=… after_ms=…` (arrival → first poll that finds the variant). The variant follower is unchanged otherwise.
- *Jobs wait (07/07A).* `ExportService` keeps `waits` (RAW → wait) for master renders refused or timed out because Capture One is not
  ready (`mechanisms::backoff::editor_wait`: `variant_not_found`, `editor_launch_deferred`, `document_not_open`, "not running", -600 →
  unavailable; `timeout`/"timed out"/-1712/"busy" → busy). Retried by `ExportService::tick` (called from `PreviewService::tick` through
  `ExportStaging::tick`) after 1, 2, 4, 8, then every 15 s; after `EDITOR_WAIT_MAX_MS` (10 min, measured from the first wait of that RAW and
  kept across retries until its master arrives) the runs fail with `editor_wait: <sentence>`; publish turns that into
  `Error::EditorWaitExpired(sentence)` (`Capture One has not loaded DSC05164.ARW after 10 minutes`; busy/closed have their own sentence). Other
  errors fail as before. Waiting is a **flag** on the run (delegated choice): `ExportState::Waiting` event, the run row's `jobs.error =
  'waiting_for_editor'` while running (`Store::set_export_waiting`), `SessionListRecord.waiting_for_editor` → `RowLabel::WaitingForEditor`
  while publishing, and `PublishActivity.text = "Waiting for Capture One…"` in the prepare step. The publish job's retry back-off is capped
  at 15 s (`JobHandler::max_backoff_ms`). Cancel in prepare ends the export run (`ExportService::cancel_run`) and finishes `Publish cancelled`.
- *Inventory on create (07A/09).* `CommerceAdapter::stock_locations` and `item_create_stocked` (defaults keep other adapters working);
  `StockDefaults` port, implemented by `SettingsStockDefaults` over settings.json `commerce.inventory_default {location_id, location_name,
  quantity}`. Migration `011_item_stock.sql`: `item_stock` (the form's per-item choice; an empty location = "No location" chosen) and
  `publish_item_stock` (frozen at confirm for items the publish will create). Prepare creates with stock when frozen; a `Tracked` create
  remembers the values as the default. Shopify: `productSet` variant gets `inventoryItem: {tracked: true}` and `inventoryQuantities:
  [{locationId, name: "available", quantity}]`; read-back `InventoryReadback` (`inventoryItem { tracked inventoryLevel(locationId:) {
  quantities(names: ["available"]) } }`); a GraphQL error / userErrors with no product → sent again without stock fields (untracked, never a
  failed publish); `locations(first: 50) { nodes { id name isActive } }`, ACCESS_DENIED → `missing_access = "Your Shopify app needs the
  read_locations and write_inventory permissions"`. Desktop commands `stock_locations`, `stock_default_get/set`, `item_stock_get/set`; UI:
  `StockFields` on the form's Product step (new items only), `StockSection` in Settings (shown with a saved integration), waiting pill.
**Judgment calls.** Waiting = flag, not a new run state; back-off 1/2/4/8/15 s; Location/Quantity at the end of the Product step; read-back =
separate query; locations cached in `PublishService` (once per app run = per built App); default remembered only after a tracked create;
stock frozen at confirm (H1). User-facing "Capture One" sentences in core follow the precedent in previews/editing_startup.
**Deviations.** New files beyond the plan's list: `crates/infra/src/sqlite/migrations/011_item_stock.sql`, `apps/desktop/src/services/stock.ts`,
`apps/desktop/src/features/form/StockFields.tsx`, `apps/desktop/src/features/settings/StockSection.tsx`, the UI test
`apps/desktop/src/features/form/__tests__/p65_stock_and_waiting.test.tsx`. Inherited tests updated to changelog 45: p09 `c6b` (timeout now waits,
fails after 10 min), p29 `failed_run` helper (same), p01 schema list (+2 tables). Shopify docs tool was not permitted in this session; the
Admin API fields above are from the API as known (2025-07+) and are unverified against a live shop — the owner's store is the first real test.
**Observation.** Most of this phase's code was committed by another session's commits `b468cad` ("backlog: …") and `8b26018` ("STATUS: …")
while this session was working; `b468cad` references the then-uncommitted `sync_folder.js` and `011_item_stock.sql` (HEAD did not build until
this checkpoint). No live Capture One probe: Capture One was not running.

### 2026-11-11 — review r1 (actor `opus`, pipeline-reviewer) — APPROVED_WITH_NOTES
**Gate.** Tracker `IMPLEMENTED`; checkpoint `b0edd78` in `git log`. Tree reviewed: `f46f1ba` (clean; only runner commits after the checkpoint).
**Perimeter.** Union of `b468cad` (34 paths), `8b26018` (5) and `b0edd78` (20) = the handoff's declared files; the extra paths in the two sweep commits (`backlog.md`, `STATUS.md`) are the orchestrator's own content. No undeclared path.
**Evidence (reviewer run, time-boxed per the plan's Review mode).** `cargo fmt --all -- --check` ok; `cargo clippy --workspace --all-targets -- -D warnings` exit 0; `npm run typecheck` ok; `vitest run src/features/{form,settings,sessions,publish}` 19 files / 125 passed; `cargo test -p psm-testkit --test p65… --test p01… --test p09… --test p29… --test p44… --test p62… --test p63… --test p64…` 8/20/8/26/25/12/4/8 passed; `cargo test -p psm-core -p psm-infra -p psm-integrations --lib` 67/10/36 passed. Full workspace not re-run (plan Review mode); the implementer's stamp is consumed: its 3 reds were the p29 helper (fixed, 26/26 here) and a load-timing p43 case (26/26 on the final tree in the handoff).
**Verified correct.**
- C1(a): `CaptureOneFollow::photo_arrived` → `request_sync` → `run_syncs` → gated `sync_folder.js` (`collectionForFolder` from `lib.js`, `synchronizing = true`, refuses when Capture One is not running, so it never launches it); `capture_one.sync` and `capture_one.variant_seen after_ms` logged. The test asserts exactly one sync op, preview issued 3–4 s after filing against a fake that lists only 3 s after a sync (else 60 s), and both log lines: fails if no sync is sent.
- C1(b): ≤ 1 sync per folder per 1 s, pending re-sync after; `EditorLaunchDeferred` keeps it pending. Test observes 1–2 syncs for 3 arrivals, 0 while held, exactly 1 after release.
- C2(a)–(c): `editor_wait` on `VariantNotFound`/`EditorLaunchDeferred`/`DocumentNotOpen` Display texts (checked in `error.rs`) and busy/timeout; `ExportService::tick` retries 1/2/4/8/15 s from `PreviewService::tick`; 10-minute deadline kept in `waited_since` across retries; `EDITOR_WAIT_EXPIRED` prefix → `Error::EditorWaitExpired(sentence)`; non-editor errors fail at once; `PublishJob` has no `max_attempts`, so a 10-minute wait cannot run out of attempts (backoff capped at 15 s); cancel in Prepare ends the export run. Tests assert exact step/error/label/gallery outcomes.
- C3(a)–(c): `productSet` variant carries `inventoryItem{tracked:true}` + `inventoryQuantities[{locationId,name:"available",quantity}]` only on `item_create_stocked`; stock frozen at confirm in `publish_item_stock` (H1), used only when the search finds no product (found/republish never send stock); default remembered only after `Tracked`; ACCESS_DENIED on locations → the sentence, on the create → re-sent untracked. Adapter and service tests assert the exact request variables, read-back count, outcome and settings.json; UI tests assert the prefilled Location/Quantity, the two sentences and the Settings save.
- Assumptions 1–11 consistent with changelog 45 / 07A / 10A; none contradicts the book.
- H1–H8: H1 stock frozen with the confirmation; H2 no file touched; H3 none; H4 retries idempotent (the create is not retried on transport errors, MC-11); H5 only the sdef `synchronizing` property; H6 stock only under a commerce integration; H7 location ids in an integration-keyed table / frozen publish table like existing link tables, "Capture One" sentences in core follow the editing_startup/previews precedent (see note 1); H8 commands thin over `PublishService`.
**Findings.** None blocking.
**Notes (backlog).** (1) core error-text classifier matches AppleScript numbers; (2) any top-level GraphQL error on a stocked create → silent untracked create; (3) stock GraphQL shapes unverified against Shopify's schema (docs tool unavailable to both sessions); the custom app needs `read_locations`/`write_inventory`; (4) the form's Stock section vanishes when locations cannot be read while the default still applies; (5) sweep commits `b468cad`/`8b26018` do not build.
