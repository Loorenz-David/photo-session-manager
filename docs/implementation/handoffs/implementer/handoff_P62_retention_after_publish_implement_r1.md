---
plan: P62_retention_after_publish
role: implement
round: 1
state: DONE
date: 2026-11-10
actor: claude-opus (PSM_IMPLEMENTER fallback; the Codex run exited at launch)
---
# Handoff: P62 retention after publish (implement r1)

## What was built
- **Published items are cleared after N days (10F §3).** Settings → Storage has a new setting, **Keep files of published items (days)**: `storage.retention_days_after_publish`, default 4, range 0–3650.
  - An item qualifies when all of these hold: its latest publish run completed (`done`) and it has not changed since (not dirty); it is not pinned; it is not already cleared; and a Shopify integration is configured.
  - Once N days have passed since its publish, the background clean-up clears it. It deletes every RAW of the item's photos (main, condition, hidden/deleted, never reviewed; Q1) and the Capture One files named after those RAWs, which are `CaptureOne/Settings*/<RAW name>.cos` and `CaptureOne/Cache/**/<RAW name>.*`. It also deletes the RAWs' identical copies in `.duplicates/`, and their previews, masters and outputs.
  - Capture One's catalogue is never touched, so the images show as missing there (Q4).
  - Without an integration nothing is cleared (H6).
- **Shopify is checked first (10F Rules, M41).** Before anything of an item is deleted, the app reads the item back from Shopify: its gallery media and the files of its condition-photos field, with ids and URLs. It compares that with the item's published images, which are the output slots of the main and condition photos frozen in its last completed publish, each with a `ready` link.
  - If any image is missing, or the read fails, nothing of that item is deleted. Health shows `Not cleared: <SKU> — Shopify is missing N images` (or `… — Shopify could not be read`), and the item is checked again after 15 minutes.
  - Only after the check passes is the item marked cleared and its files queued. At deletion time a second guard requires that the item is marked cleared and that the file belongs to it.
- **After clearing, the photos come from Shopify and the item is read-only (Q3).** Opening a cleared item shows its Shopify images, main photos then condition photos. Each image is downloaded once at 1600 px and cached in `<data>/previews/archived/<item>/`.
  - The list labels the item **Archived in Shopify**.
  - These actions are refused with `This item's files were cleared after publishing`: classify, reorder, delete, move/copy in or out, compose (set, apply, reset), undo/redo, publish, local export, and the condition-photos field. Other attributes still save, and the details form stays available.
  - The preview service no longer reports a cleared item's photos as "deleted outside Session Manager".
- **Keep files on this Mac (Q2).** A checkbox in the item details card (key I, or the info button). A pinned item is never cleared, by retention or by the budget.
- **Budget levels 4–5 (10F §4).** When levels 1–3 from P61 still leave free space below the minimum, the clean-up continues in this order:
  - level 4: published items past retention (they are cleared anyway);
  - level 5: published items not yet past retention, oldest publish first, only while space is still short.

  Each item goes through the same Shopify check. Unpublished, dirty, publishing, failed and pinned items are never cleared. The "Disk almost full" warning now counts what levels 4–5 will free.
- **Background, logged (10F §6).** The clean-up runs on P61's job: it pauses during capture, upload, render, take-back, export or publish; it checks at most 5 items per scan and deletes in batches of 20 files. Each deletion is permanent and logged as `storage.cleared rule=retention|budget_published path=… bytes=…`. Each cleared item is also logged as `storage.item_cleared item=<SKU> …`.

## Tests → criterion rows
All Rust tests are in `crates/testkit/tests/p62_retention_after_publish.rs`. Every item in them is really published to the fake shop through the publish flow first.

| Row | Test (boundary) |
|---|---|
| C1(a) | `c1a_a_published_clean_item_is_cleared_four_days_after_publish_and_nothing_else`: the storage service, files on disk, the log. A published item with 2 main photos, 1 condition photo and 2 hidden photos, plus Capture One files and a `.duplicates/` copy. At T+3 d nothing is cleared. At T+4 d 1 min the `retention` deletions are exactly its files; every one is gone; the unpublished item's files are intact; one log line per deletion. |
| C1(b) | `c1b_dirty_unpublished_pinned_or_without_integration_nothing_is_deleted`: dirty, unpublished and pinned items at T+30 d with 0 bytes free, then an app without an integration with an item marked published and done. Nothing is deleted. |
| C2(a) | `c2a_shopify_missing_an_image_or_unreadable_clears_nothing_and_says_so_on_health`: one gallery media removed from the fake product gives no deletions and Health shows `Not cleared: SKU-7 — Shopify is missing 1 images`. An unreachable shop gives no deletions and `Not cleared: SKU-8 — Shopify could not be read`. |
| C3(a) | `c3a_a_cleared_item_shows_its_photos_from_shopify_cached_and_is_archived_in_the_list`: `ArchiveService::view` returns gallery 1–2 then condition 1 with the fake CDN URLs. The cached files equal the served bytes, the downloads asked for 1600 px, and a second view downloads nothing. The session list row label is `Archived`. UI: `p62_archived_ui.test.tsx` checks the pill text and that the workspace shows the cached images with no review controls. |
| C3(b) | `c3b_a_cleared_item_refuses_what_needs_images_and_saves_an_attribute_without_images`: classify, reorder, composition set and publish confirm each give `Error::FilesCleared` with the exact text. The photo is unchanged, and `product.description` saves. |
| C4(a) | `c4a_a_pinned_item_is_never_cleared_by_retention_or_by_the_budget`: at T+60 d with 0 bytes free, the unpinned item is cleared and the pinned one keeps every file. UI: the pin checkbox calls its handler with `true`. |
| C5(a) | `c5a_below_the_limit_the_oldest_publish_goes_first_then_the_next_only_if_still_below`: OLD (published 10 d ago), RECENT (2 d ago) and OPEN (unpublished). One byte short: only OLD (`retention`). Far short: OLD, then RECENT (`budget_published`), with all of OLD's deletions before RECENT's. OPEN is never cleared. |
| C6(a) | `c6a_only_files_named_after_the_cleared_raws_go_and_the_catalogue_is_not_modified`: another item's RAW and its Capture One files sit in the same folder and stay. The catalogue's bytes and modification time are unchanged. |
| C7(a) | `c7a_no_clearing_while_a_publish_runs_then_it_clears`: an upload step is active, so ticks return `paused = publish` and delete nothing. Once the publish ends, the item is cleared. |
| C8(a) | whole suite, clippy, fmt, typecheck, `npm test` (see Stamp) |
| core mechanism | `mechanisms::retention::tests` (3 tests): due exactly at N days (and at 0); never for dirty, unpublished, pinned, cleared, failed, still publishing, or without integration; budget order with its stop. |
| UI (settings) | `p62_archived_ui.test.tsx`: the retention days load (4) and saving 0 calls `setRetentionDays(0)`. |

## Assumptions
1. "Published" means the latest publish run is `done` and `published_at` is set, on top of 07's not-dirty rule. A clean item whose latest republish failed is not cleared (10F §4 excludes failed items). Age counts from `sessions.published_at`, which is the last completed publish.
2. "Every published image" means each output slot of the main and condition photos frozen in the last completed publish (photos since deleted excluded), each with a `ready` link. Gallery ids must be in the product's media and condition ids in the condition field's read-back. An image the shop returns without a URL counts as missing, because the item could not be shown from it.
3. A failed read uses its own health text, `Not cleared: <SKU> — Shopify could not be read`, rather than a made-up count. All not-cleared items share one health line (`retention_not_cleared`, items joined with "; "), labelled "Storage clean-up" on the Health page.
4. A RAW that another session still uses (via "Copy to session"), when that session's item is not cleared, is kept. So are its Capture One files, caches and duplicates (M41). It goes when the last session using it is cleared.
5. "Duplicates of the item" means the files in `.duplicates/` whose SHA-256 equals one of its cleared RAWs' hashes. The hash comes from the capture ledger when it knows the file; otherwise a file with the same name as one of the RAWs is hashed.
6. Capture One files named after a RAW are matched exactly: `Settings*/<RAW file name incl. extension>.cos` directly inside a `Settings*` folder, and `Cache/**/<RAW file name>.<anything>`.
7. The item is marked cleared, and becomes read-only, right after verification and before its files are deleted, so no edit can slip in between. If a file then fails to delete, it is logged and left.
8. The verification runs again for an item only every 15 minutes after a failure. At most 5 items are verified and cleared per scan; the rest follow at the next scans.
9. Archived images are stored as id + URL when the item is cleared (table `archived_images`). They are downloaded on first view at 1600 px (Shopify CDN `width=` parameter) into `<data>/previews/archived/<item id>/`. A failed download shows a placeholder and is retried on the next view.
10. Read-only covers everything that needs images: classify, reorder, delete, move/copy (from or into a cleared item), composition, undo/redo, publish, local export, and the media-list (condition photos) field. Other attributes save locally but can never be published, because republish is refused (Q3: a new photo session is needed).
11. The pin sits in the item details card. A cleared item shows no pin, since it no longer has files to keep. The archived label is the existing "Published" pill style with a dashed border.
12. A `storage` settings patch now merges into the stored values rather than replacing them, so saving the minimum keeps the retention days and the reverse. The retention days have their own commands (`storage_retention_get` / `storage_retention_set`), which leaves P61's `StorageDto` and its fixtures unchanged.
13. The new port method `CommerceAdapter::media_readback` uses one GraphQL query (`MEDIA_READBACK`: product media plus `metafield(...).references`). It is written against the documented Admin API shape and has not been run against a live store; the automated suite only uses the fake. `fetch_image` downloads https URLs only, at most 64 MB.

## Deviations
Files outside the plan's named perimeter; all are needed for the rows above:
- `crates/testkit/src/fake_commerce.rs`: the fake shop implements the read-back (with made-up `https://cdn.fake-shop.test/...` URLs) and the downloads (recorded in `image_fetches`).
- `crates/testkit/tests/p01_storage_identity.rs`: its exact schema list gains the two new tables. Migration 010 broke it otherwise, as earlier migrations did.
- `crates/integrations/src/commerce/shopify/transport.rs`: `HttpTransport::get_bytes` (a default refusal, so the test transport is unchanged) and the reqwest implementation.
- `crates/core/src/services/{review,composition,history,export,attributes,previews}.rs`: the read-only guards and the missing-file exemption.
- `apps/desktop/src/features/health/HealthPage.tsx`: the "Storage clean-up" label.
- New files: `crates/core/src/mechanisms/retention.rs`, `crates/core/src/services/retention.rs`, `apps/desktop/src/services/retention.ts`, `apps/desktop/src/features/workspace/ArchivedView.tsx`.
- Not done: the master plan's §6.5 table registry and §6.7 command list do not list migration 010 or the four new commands yet. That is the planner's or orchestrator's section; this session only edited its own tracker row.
- Found while running the suite: the first version of the condition-field guard read the store inside the attribute write's transaction and deadlocked `p10_attributes`. Fixed before the stamp: the guard now reads first.

## Files changed
Every path below is in checkpoint `5b82ffe` (46 paths). This handoff is committed by the runner.

- **Rust (core):** `crates/core/src/domain/{publish,session,storage}.rs`, `crates/core/src/error.rs`, `crates/core/src/mechanisms/{mod,retention}.rs`, `crates/core/src/ports/{commerce,store}.rs`, `crates/core/src/services/{attributes,composition,export,history,mod,previews,publish,retention,review,sessions,settings,storage}.rs`
- **Rust (infra, integrations):** `crates/infra/src/sqlite/store.rs`, `crates/infra/src/sqlite/migrations/010_retention.sql`, `crates/integrations/src/commerce/shopify/{adapter,graphql,transport}.rs`, `crates/integrations/src/lib.rs`
- **Rust (testkit):** `crates/testkit/src/fake_commerce.rs`, `crates/testkit/tests/p01_storage_identity.rs`, `crates/testkit/tests/p62_retention_after_publish.rs` (new)
- **Tauri:** `apps/desktop/src-tauri/src/commands/{items,sessions,settings}.rs`, `apps/desktop/src-tauri/src/lib.rs`
- **Frontend:** `apps/desktop/src/services/bindings.ts` (generated), `apps/desktop/src/services/retention.ts` (new), `apps/desktop/src/features/workspace/{ArchivedView.tsx (new), ItemDetailsCard.tsx, WorkspacePage.tsx, workspace.css}`, `apps/desktop/src/features/workspace/__tests__/p62_archived_ui.test.tsx` (new), `apps/desktop/src/features/sessions/StatusPill.tsx`, `apps/desktop/src/features/settings/StorageSection.tsx`, `apps/desktop/src/features/health/HealthPage.tsx`, `apps/desktop/src/styles/globals.css`
- **Docs:** `docs/implementation/master_plan.md` (P62 tracker row only), `docs/implementation/plans/P62_retention_after_publish.md` (Review log)
- No archgraph in this repo. No mutation-probe files (none under flow.md).

## Checkpoint SHA
`5b82ffe` (`CHECKPOINT (not approved): P62 retention after publish — …`)

## Stamp
| Command | Result |
|---|---|
| `cargo test -p psm-testkit --test p62_retention_after_publish` | 9 passed, 0 failed (rerun on the final tree) |
| `cargo test -p psm-core retention` | 3 passed (mechanism unit tests) |
| `cargo test --workspace --no-fail-fast` | 896 passed, 1 failed. The failure is `p42_camera_runtime::c3a_restarts_at_bounded_offsets` ("camera result did not settle", its 3 s settle bound under full-suite load). It is the pre-existing timing flake P61 recorded in the same file. `cargo test -p psm-testkit --test p42_camera_runtime` alone: 12 passed, 0 failed. |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean |
| `cargo fmt --all -- --check` | clean |
| `npm run typecheck` | clean |
| `npm test` | 430/431 with the default workers. The failure is `p37_editor_chrome` "P38 C6(a) … 29 editor controls", which timed out at 5 s under load: a pre-existing intermittent in untouched compose tests (backlog from P61). `npx vitest run --maxWorkers=4`: 62 files, 431/431. p37 alone: 8/8. |

## Manual try-out
1. Start the app (`cd apps/desktop && npm run tauri -- dev`) with the test store configured. In **Settings → Storage**, set **Keep files of published items (days)** to `0` and press **Save days**.
2. Publish a test item to the test store. Within a few seconds of the publish finishing (the clean-up waits while anything is publishing or rendering), the item's RAWs disappear from its folder in the Capture One session's `Capture/` folder. Their `.cos` and cache files in `CaptureOne/` go too. The app log shows `storage.item_cleared item=<SKU>` and `storage.cleared rule=retention …` lines.
3. The session list now shows the item as **Archived in Shopify**. Open it: its photos are shown from Shopify, main then condition. Dragging or reviewing is not possible, and an undo or publish attempt says "This item's files were cleared after publishing". **Edit details →** still opens the form, and the description still saves.
4. Open another published item, press **I** (item details) and tick **Keep files on this Mac**: it keeps its files, however old.
5. To see the safety check: publish an item, delete one of its images in the Shopify admin, then wait for the clean-up. Nothing is deleted and Health shows `Not cleared: <SKU> — Shopify is missing 1 images`.
6. Set the days back to `4` afterwards.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
