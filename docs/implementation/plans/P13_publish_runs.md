---
phase: P13_publish_runs
title: Publish runs — confirmation freeze, prepare, upload, gallery mirror, resume and retry
difficulty: high
projection: no
depends_on: [P09_local_export, P12_shopify_adapter]
---
# P13 — Publish runs: confirmation freeze, prepare, upload, gallery mirror, resume and retry

> **Amended by the orchestrator 2026-10-26** from the P12 round-0 projection
> (`handoffs/reviewer/handoff_P12_shopify_adapter_projection_r0.md`, L8, L10, L15, L22, L23): task 0 now wires the
> commerce adapter into the app (adapter from settings, schema refresh persisted, search integration and the
> `sku_in_shop` checks of 05B MC-22); task 1 reuses P12's `ConfirmedRun::from_persisted`.

## Goal
Implement the publish service as the resumable step machine of MC-11 over `CommerceAdapter`: confirmation writes
the frozen run (MC-10, M17), `prepare` creates or adopts the product and exports (P09 path), `upload` drives each
link through its states with crash-safe adoption by filename stem (M18), `update` reconciles the gallery exactly
(MC-12, M19, M3) — attribute writes come in P14 — `done` records `published_at`/`published_seq` and a notification,
retry re-runs the failed step only, and startup resumes active items. Exposes `publish_preview`, `publish_confirm`,
`publish_retry`, `notifications_list` and the `publish_progress` event. **Not in this phase:** attribute writes,
damage-report files, retouch replacement (P14); any UI (P15) beyond what the New-session dialog needs to show shop
hits (task 0).

## Read first
master plan §6.5 (`publish_runs`, `publish_items`, `photo_links`, `notifications`), §6.6 (`PublishStep`, `UploadState`),
§6.7, §7.5 (`FakeCommerce`) · book `01` (H1, H4), `09A` (MC-23 settings, MC-24 condition key), `14`, `07` (Publish, Failure and retry), `04` S-G, S-I, S-J, S-K ·
contracts `07A` MC-10, MC-11 (all but the MC-21 pointer), MC-12, MC-13 (writability); `05B` MC-22 (`sku_in_shop`);
`04` S-B (pick a shop product); `05A` MC-6 (`published_seq`), MC-7 (status inputs),
MC-3 (checks a), MC-4 (`last_activity_at` on done/failed) · evidence `shopify_admin.md` SV-2 (uniquifier), SV-4,
SV-6 (search) · P12 plan §Types (`ConfirmedRun`, `RemoteItem`, `DefinitionLink`).

## Files expected to change
`crates/core/src/services/publish.rs` (new: `PublishService`, step handlers), `crates/core/src/mechanisms/reconcile.rs`
(new), `crates/core/src/domain/publish.rs` (`PublishPreview` DTO; `ConfirmedRun` is P12's, unchanged),
task 0 wiring: `crates/integrations/src/lib.rs` (`App.commerce`, `commerce_from_settings`, startup schema refresh),
`crates/integrations/src/commerce/shopify/{adapter.rs, graphql.rs}` (`item_find` by SKU or title),
`crates/core/src/ports/commerce.rs` (`item_find`), `crates/core/src/services/{items.rs (search + `sku_in_shop`),
attributes.rs (`refresh_schema`)}`, `crates/core/src/ports/store.rs` + `crates/infra/src/sqlite/store.rs`
(`replace_attribute_definition_links`, item adoption with `item_links`), `apps/desktop/src-tauri/src/commands/items.rs`,
`apps/desktop/src/services/{items.ts, bindings.ts}`, `apps/desktop/src/features/sessions/NewSessionDialog.tsx` (pick a
shop hit), `docs/intention/evidence/shopify_admin.md` (SV-6 addendum: the title query, read-only),
`crates/core/src/services/{export.rs (reuse inside prepare), sessions.rs (status inputs from items)}`, Store trait + impl
(runs, items, links, notifications, resume queries), `crates/infra/src/jobs.rs` (`PublishStep` handler),
`apps/desktop/src-tauri/src/commands/publish.rs` (new), bindings, `crates/testkit/src/fake_commerce.rs` (new),
`crates/testkit/src/test_app.rs` (`fake_shop()`, `kill_and_restart()` = drop and re-open the `App` on the same data dir),
`crates/testkit/tests/p13_publish_runs.rs` (new).

## Core mechanisms (unit tests allowed)
- `mechanisms::reconcile::plan(d: &[MediaId], current: &[MediaId], own: &HashSet<MediaId>, announced: &HashSet<MediaId>) -> Result<Plan{deletes, moves}, ReconcileError::Foreign(n)>` (MC-12 steps 2–4).
- `mechanisms::publish::progress(step, uploaded, total)` already exists (P03); `publish_summary(preview) -> String` (the exact confirmation text).

## Ordered tasks
0. **Wiring (owned here, master plan §6.1).** `commerce_from_settings(&Settings) -> Option<Arc<dyn CommerceAdapter>>`
   builds `ShopifyAdapter` over `ReqwestTransport` when `settings.integration.kind == "shopify"` with `client_id` and
   `client_secret` (09A MC-23), passing `ShopifyConfig{shop_domain, api_version, client_id, client_secret,
   condition_photos_key}` (absent key → `"custom.damage_report"`, 09A MC-24); none →
   `None` and the app stays in Export mode (P09). `App.commerce` holds it; `TestApp` injects `FakeCommerce` instead.
   At startup (background thread, never the IPC thread), when an adapter exists, `AttributesService::refresh_schema`
   calls `attributes_schema()` and, in one transaction, `import_definitions("shopify", defs)` plus
   `replace_attribute_definition_links("shopify", links)`; on error the stored definitions and links stay unchanged.
   `ItemsService::search(q)` = local matches, then the adapter's `item_find(q)` hits (exact SKU via `item_search`, and
   title per S-B; the Shopify title query is recorded read-only in a `shopify_admin.md` SV-6 addendum) not already
   linked locally, returned as `Item{origin: linked, id: 0, …}` carrying `external_id`. Adapter unreachable → local
   results only, no error. Picking a shop hit (`item_adopt(external_id)`) creates a local `linked` item with the shop's
   SKU, title, barcode and an `item_links` row, then the session opens as for a local item. `sku_in_shop` (05B MC-22):
   `item_create` and `update_identity` refuse a SKU that `item_search` finds on a product other than the item's own
   link (`Err(sku_in_shop{sku})`), only when reachable.
1. `publish_preview(session_ids) -> PublishPreview` (per session: main thumbnails in order, condition thumbnails after
   exclusions, details fill count, unreviewed count, `announced_removals` from `item_get` = current media not owned by
   this item, `create_item` flag) and `publish_summary` (text: "Publish N images …", the N2 sentence, the N1 count, the
   create note). `publish_confirm(session_ids, summary_text)`: one transaction → `publish_runs` + `publish_items` with
   the frozen columns of MC-10; returns run id; enqueues `PublishStep` jobs. Every adapter call takes the `ConfirmedRun` obtained from P12's
   public `ConfirmedRun::from_persisted(store, run_id)` (the single constructor; no new one).
2. Step handler: commit `step` before work; `prepare` (SKU search 0/1/>1 rule, `item_links` written before any other
   call, `IntegrityCheck`, export via `ExportService` into the run's staging; `export_sha256` per link); `upload`
   (frozen main then condition; skip `deleted`/moved-away; `ready` links skipped; otherwise list media, adopt by stem,
   else stage → upload → create → poll; the adoption matcher is `stem + (SV-2 uniquifier)? + ".jpg"`, never a plain
   prefix, because MC-21 revision names `-r<n>` share the stem; each transition committed after the remote effect; progress events); `update`
   = `reconcile::plan` executed (fail before any deletion on `Foreign`; delete; reorder; poll; read back equals `D`),
   P14 appends attribute writes after this; `describe` skipped; `done` → `published_at`, `published_seq =
   confirmed_change_seq`, notification "Published · <time>", `last_activity_at`.
3. Failure: `step = failed`, `failed_step`, `error`; `publish_retry(item)` → `step = failed_step` and re-enqueue.
   Startup: `running` jobs → `queued`; items in an active step re-queued.
4. `FakeCommerce` (in-memory shop; `fail_after_effect(op)` makes the op take effect remotely then return an error
   once, simulating a kill between remote effect and local record; records `(op, run_id, mono_time)`).
5. Commands, bindings, tests.

## Criteria (9 criteria, 35 rows; 0 `[rule]` rows)
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | `publish_confirm([S], text)` | main [a,b,c], condition [d,e] with exclusion e, unreviewed [u] | one run + one item in one transaction; `frozen_main = [a,b,c]`, `frozen_condition = [d]`; u nowhere | M17 |
| C1(b) | same | values on the item incl. one invalid | `frozen_attributes` holds the canonical valid values only | M17, MC-10 |
| C1(c) | same | `item_get` shows media `[m1 (own), m9 (not owned)]` | `announced_removals = [m9]`; preview's removal count 1; `confirmed_summary == text` byte for byte | M17 (N1 count), MC-10 |
| C1(d) | same | session `change_seq = 7` | `confirmed_change_seq = 7`; `confirmed_at = now_ms` | MC-10 |
| C2(a) | run a full publish | — | every `FakeCommerce` mutation record carries `run_id` of a run whose `confirmed_at ≤` the call's time; zero mutations without a run | M17, M5 |
| C3(a) | `prepare` | `create_pending`, `item_search` → 0 | `item_create` called; `item_links` row (`external_id`, `create_pending = 0`) exists **before** the next remote call (fake op order) | MC-11 (prepare) |
| C3(b) | `prepare` | search → 1 | adopted into `item_links`; no `item_create` | MC-11 |
| C3(c) | `prepare` | search → 2 | item `failed`, `failed_step = prepare`, error `several_products_carry_sku` | MC-11 |
| C3(d) | `prepare` | RAW of a frozen photo changed | `failed` at `prepare` with `raw_changed{photo}`; no upload | M10 |
| C3(e) | `prepare` success | — | export files exist in the run staging; each link has `export_sha256` = SHA-256 of its file; `upload_state = exported` | MC-11, M15 |
| C4(a) | `upload` with `fail_after_effect(staged_uploads_create)` then `kill_and_restart` | 2 main photos | resume completes; final shop has exactly one media per photo; links `ready` | M18, M4 |
| C4(b) | same with `fail_after_effect(product_create_media)` | — | on resume `list_media` shows the created media; adopted (`attached`) without a second create; one media per photo | M18 |
| C4(c) | same with `fail_after_effect(poll → READY recorded)` i.e. kill after `attached` before `ready` | — | resume polls to `ready`; no duplicate | M18 |
| C4(d) | `upload` | one link already `ready` | fake receives no staged upload for it | MC-11 (ready skipped) |
| C4(e) | `upload` | photo p moved from item A's session (link ready for A) to item B's session; publish B | p uploaded again for B (filename stem same uid); A's link untouched | MC-11 (R-I4, link scoped by item) |
| C5(a) | `reconcile::plan` | `current = [x, m1, m2]`, `D = [m1, m2]`, x ∉ own ∪ announced | `Err(Foreign(1))`; fake received no delete | M19 (R-I5) |
| C5(b) | `plan` | `current = [o1, m2, m1, a1]`, `D = [m1, m2]`, own ∋ o1, announced ∋ a1 | `deletes = {o1, a1}`; `moves` = `[(m1, 0), (m2, 1)]` | M19 |
| C5(c) | `update` executed | — | fake product gallery ids `== D` in order; `cover == main 01`; no condition media in the gallery | M3, M19 |
| C5(d) | `update` | fake read-back returns a different order after reorder | item `failed` at `update`, error `gallery_verification` | MC-12 step 5 |
| C5(e) | `update` | a `D` element not `READY` | `failed`, error `upload_incomplete`, no deletion | MC-12 step 1 |
| C5(f) | `update` | fake returns not-found on one delete | counted as deleted; step proceeds | MC-12 step 3 |
| C6(a) | `upload` | frozen photo b marked `deleted` after confirmation | b skipped (no upload); gallery `D` excludes b; one notification at done | M17 (R-I6), M3 (no deleted photo leaves the Mac) |
| C6(b) | `upload` | frozen photo moved to another session after confirmation | skipped likewise | M17 |
| C7(a) | `done` | — | `published_at = now_ms`, `published_seq = confirmed_change_seq`, `notifications` row `kind = published` text starting "Published · "; status `published` | MC-11 (done), M14 |
| C7(b) | a `classify` during the run, then `done` | — | `dirty = true` after done (`change_seq > published_seq`) | M13 (R-I13) |
| C7(c) | `failed` at `upload` | — | `last_activity_at` updated; status `failed` if never published else label `PublishFailed` | MC-4, M14 |
| C8(a) | `publish_retry(item)` after a failure at `upload` | — | re-runs from `upload` only: fake receives no `item_search`/`item_create`; completes | M18 (retry from failed step) |
| C8(b) | `kill_and_restart` with an item in `upload` and a job `running` | — | job `queued` again; item re-queued at `upload`; run completes with no duplicates | MC-11 (startup) |
| C9(a) | `commerce_from_settings` | integration `{"kind":"shopify","shop_domain":"x.myshopify.com","api_version":"2026-10","client_id":"c","client_secret":"s"}` (no `condition_photos_key`); and no integration block | `Some` (capabilities = the nine of ch09) whose config has `condition_photos_key = "custom.damage_report"`; `None` | H6, H7 |
| C9(b) | startup schema refresh | `FakeCommerce` schema of 3 `(Definition, DefinitionLink)`; then a refresh returning 2; then a refresh that errors | `definitions(Some("shopify"))` = built-ins then the 3; link rows equal the 3 links; after the second refresh exactly the 2 (definitions and links); after the error still the 2 | M20, MC-13 |
| C9(c) | `items_search(q)` | local item `L-1 "Oak table"`; fake shop products `K-1 "Oak chair"` and `L-1` (already linked to the local item) | `items_search("Oak")` = `[L-1 (local), K-1 (linked, id 0, external_id)]`; no second `L-1`; fake shop unreachable → `[L-1]` and `Ok` | MC-11, H6 (S-B) |
| C9(d) | `item_adopt(external_id of K-1)` then `session_create` | — | one `linked` item `K-1` with the shop's title and barcode; `item_links` row with that external id; the session opens with that item | M18 (link before any write; S-B) |
| C9(e) | `item_create` / `update_identity` | fake shop holds `K-7` on another product; and on the item's own linked product | `Err(sku_in_shop{sku: "K-7"})` for create and for an identity edit of a different item; the item's own product → `Ok`; shop unreachable → `Ok` | M29 (05B MC-22) |
| C9(f) | `NewSessionDialog` (vitest, services mocked) | `items_search` returns a `linked` hit with `id 0` | the row reads "in the shop"; picking it calls `item_adopt(external_id)` then opens the returned session | S-B |
| C9(g) | `items_next_sku(prefix)` with the commerce adapter wired | prefix `VF`; local max `VF-0420`; fake shop holds `VF-0421` / holds `VF-0421` and `VF-0422` / unreachable | `"VF-0422"` / `"VF-0423"` / `"VF-0421"` (plain local proposal, `Ok`) | M29 (05B MC-22 next free number) |

## Delegated choices
Job granularity (one `PublishStep` job per item stepping through, or one job per step), staging folder for the run's
exports (`.<key>.partial-<run>` as P09, then swapped to `<key>/` at `done` so the local export mirrors the publish),
notification text time format, how `publish_progress` events are throttled.

## Manual try-out
1. With Shopify configured and a session Ready, open the Publish popover (P15 brings it; until then call
   `publish_confirm` via a dev control the implementer may add) — after P15: Publish → the row shows "Publishing n%"
   → "Published"; the product in the shop shows the gallery in your Main order.
2. Reorder two photos and publish again: the gallery follows; nothing else on the product changed.
3. Kill the app mid-upload (Activity Monitor), relaunch: the row resumes and finishes; the shop has one media per photo.

## Notes
- Next free SKU (05B MC-22, kept as written by the owner 2026-10-26): when the shop holds the proposal, the
  proposal advances to the next number free both locally and in the shop (one `item_search` per candidate,
  at most 50 candidates, then the plain local proposal); a SKU the photographer typed is never changed, only
  flagged "<SKU> is already in the shop" with Create disabled. Row C9(g). This delivers what P07's narrative
  promised.
- `update` here ends after the gallery read-back; P14 appends the attribute, title, tags, description and
  damage-report writes to the same step (the step is committed once, before any of it).
- Estimated size: ~18 files, ~3 000 lines (FakeCommerce ≈ 500, wiring ≈ 500).

## Review log
- 2026-10-26 (orchestrator): owner answers (book 09A, R32): task 0 builds the adapter from `client_id`/`client_secret`
  and passes `condition_photos_key` (default `custom.damage_report`); C9(a) input/outcome reworded; the next-free-SKU
  suggestion of 05B MC-22 is kept (owner kept it as written) and gets row C9(g) (35 rows).
- 2026-10-26 (codex, implement r1): built settings-based Shopify wiring, atomic definition/link refresh, title and
  exact-SKU search with adoption and shop SKU checks, confirmation freeze, prepare/export, resumable upload, gallery
  reconciliation, retry/startup resume, commands, event and notifications. Thirty-four Rust boundary tests plus one
  New-session UI test cover the 35 rows. The read-only title and file-list probes are recorded in
  `docs/intention/evidence/shopify_admin.md`.
  Assumptions: one queued `publish_step` job advances one item step; export renders use the existing P09 staging
  folder keyed by its export job id and swap at publish completion; notification time is UTC milliseconds in text;
  progress emits at each state change and ready link without throttling; title search uses Shopify prefix terms and
  local substring filtering; file recovery pages unfiltered files because SV-3's indexed filename search lagged.
  Deviations: P01's exact schema check ruled out an added `publish_items` column, so the export job id is kept in
  the existing app-state `settings` table until completion. Registration of the publish job is in `App` using the
  existing `JobRunner`, rather than an edit to `infra/jobs.rs`. Supporting DTO, event, error, module registration,
  generated-binding and older test-fixture files changed to wire the declared boundaries; the UI boundary has a
  dedicated frontend test file. `sessions.rs` needed no edit because its existing status query reads publish items.
- 2026-10-26 (opus, review r1): **CHANGES_REQUESTED.** Gate passed (tracker IMPLEMENTED, checkpoint `3b034dd`).
  The perimeter equals the declared files (37 paths). Suite 343 Rust / 79 UI; clippy, fmt and typecheck are green.
  All 35 rows hold at their boundaries. Variations run by the reviewer (scratch test file, deleted afterwards):
  reorder + republish follows the new order (M3); a classify during a run reaches `done` with
  `published_seq = confirmed_change_seq` and stays dirty. Verified by reading: H1 (every mutating port method
  takes `ConfirmedRun` built only by `from_persisted`), H2, H3, H5, H6 (no adapter → `publish = None`), H8 (commands
  thin), no live service in the suite, no secret in the diff. **F1** (B1, route production): `upload` returns
  "not yet" whenever the product holds any media with no filename and a non-FAILED status. A READY video, 3D model or
  external video never has one (the adapter reads filenames from MediaImage URLs only), so the item retries forever
  at `upload`. It never fails and never finishes. Reviewer probe: a product with a ready unnamed media, announced for
  removal, was still at `upload` with no error after 40 simulated minutes. Authority: ch07 Failure and retry, N1,
  07A MC-12. Correction: wait only while such a media is `Processing`/`Uploaded`. Ready or failed unnamed media are
  not adoption candidates and go to MC-12 (deleted if announced, `foreign_media_added` if not). Files:
  `crates/core/src/services/publish.rs`, `crates/testkit/tests/p13_publish_runs.rs` (+ `fake_commerce.rs` if needed).
  **F2** (B1, route production): `publish_step` jobs run synchronously inside `JobRunner::tick`, on the one background
  loop that also drains capture hooks, handles watcher events and ticks previews. An `upload` step (per photo: list,
  stage, PUT, create, poll ≥ 1 s) holds that loop for tens of seconds to minutes, so captures and previews stop
  arriving during every publish. Authority: ch07 "Publishing never blocks the workspace". Correction: run
  `publish_step` jobs on their own worker thread (a runner that picks only that kind, and the shared runner skips it).
  One boundary test: a capture delivered while an upload is held gets its photo row before the upload is released.
  Files: `crates/integrations/src/lib.rs`, `crates/infra/src/jobs.rs`, `crates/core/src/ports/store.rs`,
  `crates/infra/src/sqlite/store.rs`, `crates/testkit/src/test_app.rs`, `crates/testkit/tests/p13_publish_runs.rs`.
  11 notes went to the backlog. The first is an incomplete `integration` block making the app panic at launch.
  Lesson for the plans: no platform literal in core-service tasks (task 0's `"shopify"`), and C5(f) belongs at the
  adapter boundary.
