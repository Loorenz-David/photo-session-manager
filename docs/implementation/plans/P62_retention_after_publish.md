---
phase: P62_retention_after_publish
title: Storage, part 2 — N days after an item is published and clean, its local files (RAWs included) are cleared after Shopify is verified to hold every published image; the item then shows its photos from Shopify, read-only; a "keep files" pin exempts an item; the storage budget can clear published items oldest first
difficulty: high
projection: no
depends_on: [P61_storage_cleanup_and_budget]
---
# P62 — Retention after publish; Shopify as the archive (10F §3, §4 levels 4–5, Rules, Q1–Q5, M41)

Written by the orchestrator (Opus) on 2026-11-09. Chapter `docs/intention/book/10F_storage_and_retention.md` (RATIFIED, changelog 40–41; amends H2).

## Goal
1. **Retention (10F §3).** Setting `storage.retention_days_after_publish` (default **4**, Settings → Storage). An item that is **published and not dirty**
   (07 status `published`), with no "keep files" pin, and whose publish completed ≥ N days ago is **cleared**: its RAWs (all its photos — main,
   condition, hidden/deleted, never reviewed, Q1), their Capture One sidecars and cache entries for those files (`CaptureOne/Settings*/<name>.cos`,
   `CaptureOne/Cache/**/<name>.*` in the same folder — only files named after a cleared RAW), its duplicates, masters, outputs and previews. The
   Capture One catalogue is not edited; the images show as missing there (Q4). Without a commerce integration (H6) nothing is cleared.
2. **Verify first (10F Rules; M41).** Before clearing an item, read back from Shopify the item's product media (ids and URLs) and check that every
   published image of the item (its `item_links`/publish records) is present. If any is missing or the read fails, clear nothing for that item, log it,
   and show it on Health (`Not cleared: <SKU> — Shopify is missing N images`).
3. **After clearing: Shopify images, read-only (Q3).** The item's photos are shown from Shopify (downloaded on demand at preview size, cached small in the
   preview cache); the item is read-only for anything that needs images (classify, reorder, compose, retouch, republish); attributes that need no images
   stay editable. The item list marks it `Archived in Shopify`.
4. **Keep files (Q2).** A per-item toggle "Keep files on this Mac" (item menu / form header) exempts the item from §1 and from the budget.
5. **Budget levels 4–5 (10F §4).** When P61's levels 1–3 do not bring free space above the limit: (4) clear published items past retention not yet
   cleared, then (5) published items not yet past retention, **oldest publish first**, each with the §2 verification; never an item that is not
   published-and-clean or is pinned.
6. **Background, logged (10F §6, Rules).** As P61: low priority, small batches, paused during capture/upload/render/take-back/publish; every deletion
   permanent and logged with path, size and rule (`rule=retention|budget_published`).

## Read first
Book `10F` (all, incl. M41), `01` H2 (amended), `07` (status, dirty), `07A` (publish records, media read-back), `07B` (retouch after publish — disabled
after clearing), `09A`, changelog 40–43. P61 handoff/narrative. Code: P61's `storage` service, publish services, the Shopify adapter (media read-back),
`crates/core/src/services/{sessions.rs, items.rs}`, the item list and form UI. Tests: P13–P18 publish tests, P61 tests; Shopify fixtures (made-up).

## Files expected to change (write perimeter)
P61's storage service and Settings section, publish/item services, the Shopify adapter (a media read-back call if not present), a migration (pin,
`cleared_at`), UI for the pin and the archived state, `crates/testkit/tests/p62_retention_after_publish.rs` (new), UI tests. ≈ 22 paths, ≈ 900 lines.

## Core mechanisms (unit tests allowed)
Retention eligibility (published, clean, not pinned, age, integration present); clearing order for the budget.

## Criteria (8 criteria, 10 rows)
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | retention clears after N days | published clean item, published at T; N = 4; clock T+3 d / T+4 d+1 min; fake Shopify holds all images | not cleared / cleared: its RAWs, `.cos`/cache files named after them, duplicates, masters, outputs, previews deleted; other items' files untouched; each deletion logged | `[rule]` 10F §3; H2 as amended |
| C1(b) | not eligible | dirty item / unpublished item / pinned item / no integration | nothing deleted | `[rule]` 10F §3; Q2; H6; M41 |
| C2(a) | verify first | fake Shopify missing one published image (or the read fails) | nothing cleared for that item; Health `Not cleared: <SKU> — Shopify is missing 1 images` | `[rule]` 10F Rules; M41 |
| C3(a) | Shopify images after clearing | cleared item opened | photos shown from the fake Shopify URLs (cached); list label `Archived in Shopify` | `[rule]` 10F §3 |
| C3(b) | read-only | cleared item: classify / reorder / compose / publish attempted | refused with `This item's files were cleared after publishing` ; an attribute without images still saves | `[rule]` 10F Q3 |
| C4(a) | keep files | pin set | never cleared by retention or budget | `[rule]` 10F Q2 |
| C5(a) | budget levels 4–5 | below the limit after P61 levels; items A (published 10 d ago), B (2 d ago), C (unpublished) | A cleared first, then B only if still below; C never | `[rule]` 10F §4 |
| C6(a) | Capture One untouched beyond named files | cleared item's folder also contains another item's RAW and Capture One files | only files named after the cleared RAWs are removed; the catalogue file is not modified | `[rule]` 10F Q4 |
| C7(a) | background | publish running | no clearing until it ends | `[rule]` 10F §6 |
| C8(a) | existing suites | everything else | `cargo test --workspace`, `npm test` (known intermittent `p33_split_grid` aside), clippy, fmt, typecheck green | `[rule]` master plan §7.2 |

## Delegated choices
Where Shopify images are cached and at which size (preview size, ≤ 2560 px); the archived label's style; the pin's placement.

## Manual try-out
In Settings → Storage set retention to 0 days. Publish a test item to your test store, wait for the next clean-up: its RAWs disappear from the capture
folder, the item shows "Archived in Shopify" and its photos come from Shopify; reordering is refused. Pin another published item: it keeps its files.

## Notes
- **Hard rules:** this phase is the only place RAWs may be deleted (owner 2026-11-09, 10F), and only for published, clean, unpinned items after Shopify
  verification; never for unpublished items; tests use temp folders and a fake Shopify, never the live store. Default implementer (Codex).
- **Hazard:** a bug here destroys photos. Every deletion path must go through the eligibility + verification gate; the reviewer reads every call site.

## Review log

- **2026-11-10 implement r1 (claude-opus for codex; Codex exited at launch).** Retention lives in P61's `StorageService` (`services/storage.rs`):
  each scan lists items through `session_list_records`, decides with the core mechanism `mechanisms::retention::{eligibility, plan_retention}`
  (published = latest run `done` and `published_at` set, not dirty, not pinned, not cleared, integration configured; due at `published_at + N days`;
  budget: due items oldest publish first, then not-yet-due ones oldest first while free space is short; 3 unit tests), and for each chosen item
  (≤ 5 per scan) calls `services::retention::verify`: the last done publish's frozen main/condition photos × output slots → their `ready` links →
  must all be in Shopify's read-back (new port method `CommerceAdapter::media_readback`: gallery media + condition-field files, ids and URLs, one
  GraphQL query `MEDIA_READBACK`). Verified → eligibility re-checked → `mark_item_cleared` (stores the images to show) → the item's files queued
  (RAWs of every photo unless another not-cleared session uses the RAW; `CaptureOne/Settings*/<RAW name>.cos` and `CaptureOne/Cache/**/<RAW name>.*`;
  previews/masters/outputs folders of each RAW; `.duplicates/` files with a RAW's hash) and deleted in batches of 20 behind a second guard
  (`allowed_item_file`: item cleared and path belongs to it). Not verified → health `retention_not_cleared` (`Not cleared: <SKU> — Shopify is
  missing N images` / `… — Shopify could not be read`), retried after 15 min. Logged `storage.cleared rule=retention|budget_published path=… bytes=…`.
  Read-only guard `Error::FilesCleared` ("This item's files were cleared after publishing") in classify/reorder/delete/move/copy, composition
  set/apply/reset, undo/redo, publish confirm, export run, and media-list attribute writes; other attributes save. The preview service no longer
  marks a cleared item's photos missing. Archived view: `ArchiveService::view` downloads each image once (`fetch_image`, 1600 px) into
  `<data>/previews/archived/<item>/`; list label `RowLabel::Archived`. Pin: `ArchiveService::set_keep_files`. Settings `storage.retention_days_after_publish`
  (0–3650, default 4; storage patches now merge). Migration `010_retention.sql` (tables `item_retention`, `archived_images`). Commands
  `item_archive_get`, `item_keep_files_set`, `storage_retention_get`, `storage_retention_set`. UI: `ArchivedView` replaces stage and panel for a
  cleared item (details form still available), `Archived in Shopify` pill, pin checkbox in the item card, retention days in Settings → Storage.
  Deviations: testkit `fake_commerce.rs` (read-back, fake CDN URLs, downloads), `p01_storage_identity.rs` (two new tables in its schema list),
  `shopify/transport.rs` (`get_bytes`), `HealthPage.tsx` (label). Handoff: `handoffs/implementer/handoff_P62_retention_after_publish_implement_r1.md`.


- **2026-11-10 review r1 (opus, pipeline-reviewer): CHANGES_REQUESTED.** Perimeter = the 46 declared paths (`5b82ffe`). Reviewer stamp on code tree
  `5b82ffe`: Rust 896 passed / 1 failed (`p42_camera_runtime::c1c_restart_reaps_old_child`, the known 3 s settle flake under load; that file alone 12/12),
  clippy, fmt, typecheck clean, `npm test` 431/431. Verified correct: the deletion gate on every call site (eligibility → `verify` Holds → fresh
  re-check + pause → `mark_item_cleared` → `item_files` → per-file `allowed_item_file` + P61's size/mtime check; no adapter → no retention);
  C1(a)–C7(a) each have a boundary test that fails on a wrong outcome (C1(a) set equality both ways; C1(b) each exclusion its own cause; C2(a)
  exact Health text; C3(a) URLs, cache bytes, 1600 px, no second fetch, `Archived`; C3(b) exact error text; C5(a) level-5 stop and order;
  C6(a) neighbour + catalogue untouched; C7(a) paused then cleared); 3 mechanism unit tests; assumptions 1–13 consistent with 10F.
  **F1 (blocking, B3: H4; B1 vs 10F §3/§4; route production):** clearing is not resumable. `clear_item` sets `cleared_at`, then queues the
  item's files in the in-memory `Work.queue` (20 files / 2 s); a cleared item is `Eligibility::Never`, and nothing re-lists its files. A quit,
  crash or `App` rebuild mid-queue, or a file whose size/mtime changed before its turn, leaves those files (RAWs included) on disk permanently
  while the item reads "Archived in Shopify". Reviewer probe (test file created, run, deleted; tree clean): 47-file item, first batch of 20
  deleted, `kill_and_restart`, 40 ticks → 27 files left (e.g. `HALF_C0.ARW`, `HALF_D.ARW`, `.cos`). Correction: every scan also queues the
  remaining on-disk files of items already marked cleared (same `item_files` + `allowed_item_file`; no scan-request loop on an undeletable
  file); one behaviour test: interrupted by `kill_and_restart` after the first batch → after restart every file of the item gone, other items
  untouched. Fix may touch `crates/core/src/services/storage.rs`, `crates/testkit/tests/p62_retention_after_publish.rs` (and, only if the rule must persist,
  a new migration 011 + `store.rs` + `ports/store.rs`; 010 is checkpointed). 7 notes → backlog. Handoff:
  `handoffs/reviewer/handoff_P62_retention_after_publish_review_r1.md`.

- **2026-11-10 fix 1 (claude sonnet, pipeline-implementer, fix r2).** F1 resolved in `crates/core/src/services/storage.rs`: `scan` now calls the new
  `queue_cleared_leftovers` before `retention_scan`. It lists the files still on disk (`item_files`, same shared-RAW rule) of every session whose item is
  already marked cleared and queues them as `QueuedKind::Item`, so the existing per-file `allowed_item_file` guard and P61's size/mtime check apply
  unchanged; rule logged `retention`. Judgment calls: (1) the rule is not persisted (the reviewer allowed `retention`), so no migration 011; (2) the
  pass runs at the first scan after start and then at most every `REFRESH_EVERY` (5 min), tracked by `Work.leftovers_at` — a file that cannot be
  deleted is retried every 5 min, never in a scan-request loop; (3) it does not depend on a commerce adapter being configured (the item was already
  verified and cleared); (4) its bytes are added to the free space expected for the "Disk almost full" warning. Test added:
  `c1c_clearing_interrupted_by_a_restart_finishes_after_it_and_touches_nothing_else` (first batch deleted, `kill_and_restart`, ticks → every file of the
  item gone, the other item's files intact). Perimeter: storage.rs, the P62 test file, this log, the tracker row. Handoff:
  `handoffs/implementer/handoff_P62_retention_after_publish_fix_r2.md`.

- **2026-11-10 re-review r2 (opus, pipeline-reviewer): CHANGES_REQUESTED (one MUST).** Perimeter: `2aab879` = storage.rs, the P62 test file, the
  tracker row, this log (all allowed; no migration 011). Reviewer stamp on code tree `2aab879`: Rust 898 passed / 0 failed (workspace run cut at 30 min inside p50 after 808/0; p50 + remaining files rerun: 90/0); clippy, fmt, typecheck clean; UI 431/431.
  **F1 resolved:** `scan` → `queue_cleared_leftovers` (first scan after start, then ≤ every 5 min via `Work.leftovers_at`) re-queues `item_files` of
  cleared sessions as `QueuedKind::Item` (same `allowed_item_file` + size/mtime guard; queue replaced, never doubled; pause in `tick` precedes it;
  no scan-request loop); `c1c` bites on the r1 scenario. **F2 (blocking, B3: H2; B1 vs M41 / 10F §4; route production; triage MUST):** the pass
  and `allowed_item_file` use `photos_in_session` as it is *now*, so a RAW that arrives in a cleared item's session after clearing (UI activates
  an archived item on open; arrival/import do not check `cleared_at`; `arrival_insert` makes the item dirty) is deleted within 5 min, never
  published or verified. Reviewer probe (test file created, run, deleted; tree clean): cleared PROBE, import `outside_new.ARW` into it →
  dirty = true → +6 min → deleted by rule `retention`. Correction: leftovers and `allowed_item_file` consider only photos with
  `created_at <= cleared_at` (or: refuse/route arrivals into a cleared session to Unassigned); one test: a RAW imported after clearing survives the
  leftover interval. Fix may touch `crates/core/src/services/storage.rs`, `crates/testkit/tests/p62_retention_after_publish.rs` (alternative:
  `arrival.rs` / `photos.rs`). N1 (leftover bytes counted as freed each pass) → backlog. Handoff:
  `handoffs/reviewer/handoff_P62_retention_after_publish_review_r2.md`.
- 2026-11-10 orchestrator: FIX2_AUTHORISED: re-review r2 F2 is a MUST data-loss defect (a RAW captured or imported into a cleared item after clearing is deleted by the leftover pass; H2, M41, 10F §4). Fix 2 does both parts of the reviewer's correction: (a) in `storage.rs`, the leftover pass and `allowed_item_file` for `QueuedKind::Item` consider only photos with `created_at <= cleared_at` (and their Capture One files, caches and hash-matched duplicates); (b) a cleared item is never a capture/import target: arrivals and imports while a cleared session is active go to Unassigned (10F Q3, read-only), touching `crates/core/src/services/arrival.rs` and/or `photos.rs` only as needed. Tests: the reviewer's probe scenario at the storage boundary (the new RAW survives the leftover interval; the item's original files are gone) and one arrival test (a capture while the cleared item is active lands in Unassigned). Nothing else changes.
