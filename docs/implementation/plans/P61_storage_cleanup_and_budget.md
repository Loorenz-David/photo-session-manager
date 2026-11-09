---
phase: P61_storage_cleanup_and_budget
title: Storage, part 1 — verified duplicates go after 24 hours, Capture One's preview exports go once copied, masters go after publish, and Settings → Storage shows what uses the disk with a minimum-free-space limit that clears safe files in a fixed order
difficulty: normal
projection: no
depends_on: [P60_take_back_and_self_healing]
---
# P61 — Storage clean-up and budget (10F §1, §2, §4 levels 1–3, §6; masters per 10D MC-28)

Written by the orchestrator (Opus) on 2026-11-09. Chapter `docs/intention/book/10F_storage_and_retention.md` (RATIFIED, changelog 40–41). The owner moved
storage ahead of the work-Mac installation (2026-11-09 evening). Part 2 (retention after publish, Shopify as the archive) is P62.

## Goal
1. **Duplicates (10F §1).** A file in `<capture root>/.duplicates/` whose SHA-256 equals the filed photo's (the ledger already knows both hashes, MC-33)
   is deleted **24 h** after it was diverted. A same-name file with a different hash is never touched. Duplicates already on disk (from before this
   phase) qualify by their file time when the ledger has their hash; otherwise the app hashes them once in the background before deciding.
2. **Preview exports (10F §2).** Capture One's export in `Output/_session_manager/` is deleted as soon as the app has copied it into its preview cache
   (and on startup, any export there whose copy is current in the cache). Exports of other recipes and anything outside `_session_manager/` are never
   touched.
3. **Masters (10D MC-28).** Verify and, if missing, implement: a session's masters are deleted once its publish (or, with no integration, its export)
   completes and the session is not dirty; the previous master of a photo is deleted when a newer one is committed.
4. **Settings → Storage (10F §4).** Shows free space on the disk holding the capture root and the space used by RAWs, duplicates, preview exports, the
   preview cache and masters (computed in the background, refreshed at most every 5 min and after a clean-up). A **minimum free space** (default
   **20 GB**, editable). When free space falls below it, the app clears, in order and only until it is above the limit again: (1) verified duplicates
   (even before 24 h), (2) preview exports already copied, (3) masters not needed by a running publish/export. If that is not enough, the header and
   Health say `Disk almost full: N GB free; nothing more can be cleared safely` (levels 4–5, published items, come in P62).
5. **Never slows the app (10F §6).** All of this runs as a low-priority background job in small batches (≤ 20 files per batch, a pause between batches),
   paused while a capture, an upload, a render, a take-back or a publish is running. Every deletion is permanent and logged at INFO with the path, size
   and rule (`storage.cleared rule=duplicate_24h path=… bytes=…`).

**Not in this phase:** deleting RAWs or anything of published items (P62), the per-item "keep files" pin (P62), anything inside Capture One's own
folders (`CaptureOne/` settings and cache).

## Read first
Book `10F` (all), `10D` MC-28 (masters), `10E` MC-33 (duplicates, as amended), `01` H2 (as amended), changelog 40–43. Code: `crates/core/src/services/camera_ledger.rs`
(duplicate decisions and hashes), `crates/core/src/services/previews.rs` + `crates/integrations/src/editing/capture_one/{adapter.rs, renders.rs}` (where the
preview export is copied into the cache), masters/exports (`render_output`, publish completion in `crates/core/src/services/publish*.rs`), the job runner,
`crates/core/src/services/settings.rs` + `crates/infra/src/settings_file.rs`, Health, `apps/desktop/src/features/settings/`. Tests: p44, p53, p56, p58 testkit files.

## Files expected to change (write perimeter)
The files above as needed; a new core service `storage` (core: decisions; infra: disk free space and directory sizes); `crates/testkit/tests/p61_storage_cleanup_and_budget.rs`
(new); `apps/desktop/src/features/settings/StorageSection.tsx` (new) + its test; bindings. ≈ 18 paths, ≈ 700 lines.

## Core mechanisms (unit tests allowed)
The clean-up decision (which files qualify, in which order, until which limit).

## Criteria (6 criteria, 8 rows)
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | duplicates after 24 h | a verified duplicate diverted at T; clock T+23 h / T+25 h | not deleted / deleted, logged `rule=duplicate_24h` | `[rule]` 10F §1 |
| C1(b) | different hash never touched | same-name file with another hash (a `__dup` photo) and a file in `.duplicates` with unknown hash that hashes differently | neither deleted | `[rule]` 10F §1; MC-33 |
| C2(a) | preview export cleared once copied | export written to `Output/_session_manager/DSC1.jpg`, copied into the cache | the export file is gone; the cache copy is intact; a TIFF in `Output/` and a file in `Output/other/` untouched | `[rule]` 10F §2 |
| C3(a) | masters after publish | session published, not dirty, with 2 masters; another session publishing | the first session's masters deleted; the publishing session's kept | `[rule]` 10D MC-28 |
| C4(a) | budget order | free space below the limit; duplicates (fresh), preview exports, masters present | cleared in order duplicates → exports → masters, stopping as soon as free space ≥ limit (fake disk) | `[rule]` 10F §4 |
| C4(b) | nothing more to clear | still below the limit after levels 1–3 | header + Health `Disk almost full: N GB free; nothing more can be cleared safely`; no RAW or unpublished file touched | `[rule]` 10F §4; M41 |
| C5(a) | background and paused | a render is running when clean-up is due | no deletion until it ends; batches ≤ 20 files | `[rule]` 10F §6 |
| C6(a) | existing suites | everything else | `cargo test --workspace`, `npm test` (known intermittent `p33_split_grid` aside), clippy, fmt, typecheck green | `[rule]` master plan §7.2 |

## Delegated choices
How sizes are computed (walk with a cap per tick, cached); the Storage section's layout (one line per kind, the limit as a number of GB); the pause between
batches.

## Manual try-out
Settings → Storage shows your disk's free space and the size of RAWs, duplicates, previews and masters. Shoot on the cable: the duplicates folder grows,
then shrinks a day later. Set the minimum free space above your current free space: duplicates and preview copies are cleared at once and the numbers drop.

## Notes
- **Hard rules:** never delete a RAW, an unpublished item's file, or anything in Capture One's own folders in this phase; deletion is permanent but only
  for the three kinds above; tests use temp folders and a fake disk-free source. Default implementer (Codex).

## Review log

- **2026-11-09 implement r1 (claude-opus for codex, PSM_IMPLEMENTER=claude).** Built a core `StorageService` (`crates/core/src/services/storage.rs`)
  over a new `StorageFs` port (infra `storage_fs.rs`) and the clean-up decision `mechanisms::cleanup::plan_cleanup` (the plan's core mechanism, 4 unit
  tests). Its own background thread ticks every 250 ms; it pauses while a capture download is expected, an FTP upload runs, a render is in flight,
  the take-back holds Capture One, an export run is in flight or a publish item is active; batches of ≤ 20 files, 2 s pause. Duplicates: verified when a
  filed delivery with the same SHA-256 is on disk (ledger hash, else hashed once, ≤ 4 per scan); age from the ledger's `completed_at`, file time for
  files the ledger does not know. Preview exports: noted by the preview service after its copy (JPEG in a `_session_manager` folder only), deleted when
  byte-identical to the cache copy; at start-up existing exports are matched against the cache by size + hash. Masters (MC-28) were already released
  after publish/export (verified, C3 test); level 3 clears masters whose RAW no export run or active publish needs. Settings key `storage.min_free_gb`
  (default 20, 0–100000; saved without a runtime rebuild); commands `storage_get`, `storage_min_free_set`; `StorageSection` in Settings and a
  `DiskWarning` in the header; health code `disk_almost_full`. Deviations: `Header.tsx`, `globals.css`, `services/storage.ts`, `camera/runtime.rs`
  (`capture_in_progress`), testkit `complete_preview_export`. Handoff: `handoffs/implementer/handoff_P61_storage_cleanup_and_budget_implement_r1.md`.
