---
plan: P61_storage_cleanup_and_budget
role: implement
round: 1
state: DONE
date: 2026-11-09
actor: claude-opus (PSM_IMPLEMENTER=claude, run for codex)
---
# Handoff — P61 storage clean-up and budget (implement r1)

## What was built
- **Duplicates go after 24 hours (10F §1).** A copy in `<capture root>/.duplicates/` is deleted 24 h after it was diverted, but only if it is
  *verified*: a filed photo with the same SHA-256 is still on disk. The hash comes from the capture ledger when the ledger knows the file;
  a file the ledger does not know is hashed once in the background (at most 4 per pass, kept in memory) and qualifies by its file time. A
  same-name file with other bytes (a `__dup` photo) is never looked at (it is not in `.duplicates/`), and an unknown file whose hash matches
  no filed photo is never deleted, even under disk pressure.
- **Capture One's preview exports are not kept twice (10F §2).** When the app copies a preview export into its cache, the export (a JPEG in a
  `_session_manager` folder) is noted; the background job deletes it once its bytes still equal the cache copy. At start-up, exports already
  in `<document>/Output/_session_manager/` whose byte-identical copy is in the cache are found and cleared too. TIFFs, other folders and
  other recipes' exports are never touched.
- **Masters (10D MC-28).** Verified: already deleted after a session's publish (or export) completes and the session is not dirty, and the
  previous master of a photo is deleted on commit. No change needed except that the "still needed" check is now shared (`masters_in_use`).
- **Settings → Storage (10F §4).** A new section shows free space on the capture root's disk and the space used by RAW files, duplicates,
  Capture One preview exports, the preview cache and masters (computed in the background, refreshed at most every 5 min and after every
  clean-up), plus **Minimum free space (GB)** (default 20, setting `storage.min_free_gb`). Below the minimum the job clears, in order and only
  until it is above again: verified duplicates (even before 24 h), copied preview exports, masters no running export or publish needs. If
  that is not enough, the header and Health show `Disk almost full: N GB free; nothing more can be cleared safely`.
- **Never slows the app (10F §6).** The job runs on its own thread, every 250 ms, does nothing while a capture download is expected, an FTP
  upload runs, a render is in flight, the take-back holds Capture One, an export run is in flight, or a publish item is active; it deletes at
  most 20 files per batch with a 2 s pause between batches. Every deletion is permanent and logged at INFO:
  `storage.cleared rule=<duplicate_24h|preview_copied|budget_duplicate|budget_master> path=… bytes=…`.

## Tests → criterion rows
| Row | Test (boundary) |
|---|---|
| C1(a) | `p61_storage_cleanup_and_budget::c1a_a_verified_duplicate_goes_24_hours_after_it_was_diverted` — ledger duplicate at T; `storage.tick()` at T+23 h clears nothing, at T+25 h returns exactly `duplicate_24h` for that file; file gone, filed photo intact; the INFO log line asserted |
| C1(b) | `c1b_a_file_with_another_hash_is_never_touched` — `__dup` renamed photo + unknown-hash file in `.duplicates/`, 72 h later and 1 GB free: nothing cleared, both files intact |
| C2(a) | `c2a_a_preview_export_is_cleared_once_copied_into_the_cache` — real preview render completed into `Output/_session_manager/DSC1.jpg`; cleared list is exactly that export (`preview_copied`); cache copy byte-equal; `Output/DSC7.tif` and `Output/other/DSC1.jpg` intact |
| C3(a) | `c3a_masters_go_after_publish_while_a_publishing_session_keeps_its_own` — publish A (2 photos) to done while B stays in prepare (its second master held); A's master dirs gone, B's rendered master kept |
| C4(a) | `c4a_below_the_minimum_files_go_in_order_until_free_space_is_back` — fake disk 2 500 bytes short; cleared exactly `[budget_duplicate dup, preview_copied export, budget_master older master]`; newer master kept; no warning |
| C4(b) | `c4b_nothing_more_to_clear_warns_and_touches_no_raw_or_unpublished_file` — 1.234 GB free vs 20 GB; warning text exact in `storage.warning()` and the `disk_almost_full` health item; filed RAW (an unpublished photo), its cache and output intact. UI: `p61_storage_section.test.tsx` "C4(b)" shows the warning in the header element and the section |
| C5(a) | `c5a_clean_up_waits_for_a_running_render_then_clears_in_batches_of_20` — 25 copied exports, a preview render in flight: tick paused (`render`), nothing deleted; after completion 20, then 0 within the pause, then 5 |
| C6(a) | whole suite, clippy, fmt, typecheck, `npm test` (Stamp) |
| core mechanism | `mechanisms::cleanup::tests` (4): 24 h boundary, routine vs budget, order and stop, short flag |
| UI (C4) | `p61_storage_section.test.tsx`: per-kind lines and their formatted values, saving the minimum calls `storage_min_free_set(150)` |

## Assumptions
1. "Diverted at" for a duplicate the ledger knows is the ledger row's `completed_at` (app clock), not the file's time; the file time is used only for files the ledger does not know (the plan's "already on disk" case).
2. A duplicate is *verified* only if a filed copy (delivery with outcome photo/renamed, state done) with the same hash is still on disk; if the filed copy was deleted outside the app, the duplicate is kept (it may be the only copy).
3. "Copied into the cache" is checked by byte identity (size + SHA-256) of the export and its cache copy, right before deciding; an export Capture One overwrote since is left until its own copy notes it again.
4. The app's export folder is `<document>/Output/_session_manager/` (Capture One's default output location for the session); a noted export is accepted in any folder named `_session_manager`.
5. Budget level 3 skips masters of RAWs any export run still needs and of every photo of a session with an active publish item (prepare/upload/update/describe).
6. "A render is running" = Capture One renders issued and not finished; export runs (which include output renders) and active publish items pause the job too. "A capture" = the direct camera expecting a Save-to-PC download.
7. Pause between batches 2 s; refresh every 5 min; sizes are walked in full per refresh (no per-tick cap) — cheap metadata reads on the job's own thread. While paused nothing is computed, so Settings may show "Calculating…" during a long render backlog.
8. Minimum free space is whole GB (10⁹ bytes), 0–100 000; 0 turns the budget off. Saving it does not rebuild the runtime (like theme/output format) and is applied at the next tick.
9. "N GB free" in the warning is the free space after the planned deletions, one decimal (`1.2 GB`), like the existing disk-space message.
10. The header warning is a small text in the header (both pages), refreshed every 30 s; the Storage section refreshes every 5 s while open.
11. Storage has its own commands (`storage_get`, `storage_min_free_set`) instead of a new field in `SettingsDto`, so the settings DTO and its fixtures stay unchanged.

## Deviations
New or changed files outside the plan's named list (all within "the files above as needed"/bindings scope, declared here):
- `apps/desktop/src/features/shell/Header.tsx` + `apps/desktop/src/styles/globals.css` (`.disk-warning`) — the header warning C4(b) requires.
- `apps/desktop/src/services/storage.ts` — the frontend service the section uses.
- `crates/integrations/src/camera/runtime.rs` — `capture_in_progress()` for the pause check.
- `crates/core/src/ports/storage.rs`, `crates/core/src/domain/storage.rs`, `crates/core/src/mechanisms/cleanup.rs` — the port, settings/report types and the core mechanism of the new service.
- `crates/testkit/src/test_app.rs` — `complete_preview_export(job)` (export written where the real recipe writes it).
- `crates/core/src/services/export.rs` — `masters_in_use()`/`running()`; `release_masters` refactored onto the same check (behaviour unchanged).

## Files changed
apps/desktop/src-tauri/src/commands/settings.rs · apps/desktop/src-tauri/src/lib.rs · apps/desktop/src/features/settings/SettingsPage.tsx ·
apps/desktop/src/features/settings/StorageSection.tsx (new) · apps/desktop/src/features/settings/__tests__/p61_storage_section.test.tsx (new) ·
apps/desktop/src/features/shell/Header.tsx · apps/desktop/src/services/bindings.ts (generated) · apps/desktop/src/services/storage.ts (new) ·
apps/desktop/src/styles/globals.css · crates/core/src/domain/mod.rs · crates/core/src/domain/storage.rs (new) · crates/core/src/mechanisms/mod.rs ·
crates/core/src/mechanisms/cleanup.rs (new) · crates/core/src/ports/mod.rs · crates/core/src/ports/storage.rs (new) · crates/core/src/services/export.rs ·
crates/core/src/services/mod.rs · crates/core/src/services/previews.rs · crates/core/src/services/settings.rs · crates/core/src/services/storage.rs (new) ·
crates/infra/src/lib.rs · crates/infra/src/settings_file.rs · crates/infra/src/storage_fs.rs (new) · crates/integrations/src/camera/runtime.rs ·
crates/integrations/src/lib.rs · crates/testkit/src/test_app.rs · crates/testkit/tests/p61_storage_cleanup_and_budget.rs (new) ·
docs/implementation/master_plan.md (P61 tracker row) · docs/implementation/plans/P61_storage_cleanup_and_budget.md (Review log) · this handoff.
No migration, no archgraph (none in this repo).

## Checkpoint SHA
The `CHECKPOINT (not approved): P61 …` commit made at the end of this session (`git log --grep 'CHECKPOINT (not approved): P61'`); the SHA is in the final message.

## Stamp
| Command | Result |
|---|---|
| `cargo test -p psm-testkit --test p61_storage_cleanup_and_budget` | 7 passed, 0 failed |
| `cargo test -p psm-core cleanup::tests` | 4 passed |
| `cargo test --workspace --no-fail-fast` | exit 0 — 885 passed, 0 failed. (A first run, under load, failed `p42_camera_runtime::c2b_shutdown_kills_ignoring_child_after_grace` with "camera result did not settle" — its 3 s settle bound; it passes alone (12/12) and in the full rerun. Pre-existing timing, unrelated to storage.) |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean |
| `cargo fmt --all -- --check` | clean |
| `npm run typecheck` | clean |
| `npm test` | 61 files, 427/427 passed. (One earlier run timed out `p37_editor_chrome` "P38 C6(a) … 29 editor controls" at 5 s under full-suite load; passes alone 8/8 twice and in the rerun. Pre-existing intermittent, compose only.) |

## Manual try-out
1. Start the app (`cd apps/desktop && npm run tauri -- dev`), open Settings (gear in the header) and scroll to **Storage**: after a few seconds it
   shows the free space on your disk and the size of RAW files, duplicates, Capture One preview exports, previews and masters.
2. Shoot a few frames on the cable with Wi-Fi transfer on: the duplicates line grows. A day later (or at once, see 4) it shrinks; the app log
   (`~/Library/Application Support/…/logs`) shows `storage.cleared rule=duplicate_24h …` lines.
3. After previews have rendered, the "Capture One preview exports" line goes back to (almost) 0 MB: the exports in
   `<Capture One session>/Output/_session_manager/` are removed once copied.
4. Set **Minimum free space (GB)** above your current free space and press **Save minimum**: duplicates and preview exports are cleared at once
   and the numbers drop; if that is not enough, the header shows `Disk almost full: N GB free; nothing more can be cleared safely`. Set it back
   to 20 afterwards. RAW files are never removed.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
