---
plan: P05_capture_arrival
role: implement
round: 1
state: DONE
date: 2026-10-25
actor: codex
---
# P05 capture arrival — implementation handoff

## What was built

A capture now appears once in the session indicated by its capture folder, or in Unassigned. Hidden Capture One files and non-Sony RAWs are ignored. Hook deliveries go directly to arrival; watcher and startup-scan deliveries wait for two matching file stats at least one second apart. A timed-out delivery is marked on its original capture-event row. New photos start unreviewed, remember file mtime, emit `photo_added`, and queue an original preview, SHA-256 hash, and adjusted-preview render. The preview job byte-copies the largest embedded JPEG and stores its orientation. Import and whole-Unassigned assignment are callable through the two new Tauri commands and generated bindings.

The existing Sony RAW `session test 11957.ARW` was read without writing to it. The production TIFF walker found the 1088×1080 JPEG at offset 114850 (221976 bytes) and orientation 1, matching the Capture One evidence.

## Tests → criterion rows

Every row has one boundary test in `crates/testkit/tests/p05_capture_arrival.rs` (33/33):

| Criterion row | Test |
|---|---|
| C1(a) | `c1a_hidden_file_ignored` |
| C1(b) | `c1b_capture_one_folder_ignored` |
| C1(c) | `c1c_dot_capture_folder_ignored` |
| C1(d) | `c1d_jpeg_ignored` |
| C1(e) | `c1e_two_levels_ignored` |
| C1(f) | `c1f_lowercase_extension_accepted` |
| C1(g) | `c1g_directory_ignored` |
| C2(a) | `c2a_growth_requires_next_stable_stat` |
| C2(b) | `c2b_growth_times_out` |
| C3(a) | `c3a_unassigned_folder_wins` |
| C3(b) | `c3b_published_folder_becomes_dirty` |
| C3(c) | `c3c_folder_wins_over_active` |
| C3(d) | `c3d_unknown_folder_uses_active` |
| C3(e) | `c3e_unknown_folder_uses_unassigned` |
| C3(f) | `c3f_root_uses_active` |
| C4(a) | `c4a_hook_watcher_scan_one_row` |
| C4(b) | `c4b_watcher_then_hook_one_row` |
| C4(c) | `c4c_concurrent_delivery_one_row` |
| C4(d) | `c4d_every_delivery_recorded` |
| C5(a) | `c5a_row_defaults_uid_and_raw_state` |
| C5(b) | `c5b_file_mtime_not_exif` |
| C5(c) | `c5c_three_followup_jobs` |
| C6(a) | `c6a_largest_ifd_stream_copied` |
| C6(b) | `c6b_invalid_largest_uses_smaller` |
| C6(c) | `c6c_orientation_stored` |
| C6(d) | `c6d_raw_unchanged` |
| C6(e) | `c6e_no_jpeg_finishes_without_preview` |
| C7(a) | `c7a_startup_scans_root_and_children` |
| C7(b) | `c7b_startup_ignores_cache_and_known_raw` |
| C8(a) | `c8a_import_active_and_provider_called` |
| C8(b) | `c8b_import_without_active_unassigned` |
| C8(c) | `c8c_import_twice_one_row` |
| C8(d) | `c8d_assign_whole_unassigned_set` |

Core-mechanism unit tests additionally cover hook payload shapes, attribution precedence, and a SubIFD JPEG plus IFD0 orientation. The P05 tests use fake Capture One and disposable SQLite only.

## Assumptions

- Imports accept any regular file; the Sony camera adapter declares `arw` for capture watcher/scan filtering. This keeps the import picker open to the file types Capture One itself accepts.
- When Capture One is unavailable at app start, spool delivery and capture scanning are deferred and retried when the configured capture root can be inspected. The spool file remains for replay.
- `processing_done` and `variant_adjusted` dispatch to logging handlers until P06 supplies their behavior; P05 issues the initial adjusted-preview render job.
- A failed stability check updates the event row for that delivery instead of appending a second event. Each watcher/scan delivery therefore remains one recorded event.

## Deviations

- The literal expected-file list did not name all module exports and composition wiring required for the new services. The additional core/infra/testkit `mod.rs` or `lib.rs` exports, camera extension port and Sony implementation, `AppEvent`, integration app composition, and Tauri event bridge are included in the checkpoint. These connect P05 behavior only.
- No live shutter was fired in this session. The automated suite remains hardware-free; the existing Sony RAW was used for a read-only production-parser probe.

## Files changed

Checkpoint commit (29 paths):

- `apps/desktop/src-tauri/src/commands/photos.rs`
- `apps/desktop/src-tauri/src/lib.rs`
- `apps/desktop/src/services/bindings.ts`
- `crates/core/src/events.rs`
- `crates/core/src/mechanisms/arrival.rs`
- `crates/core/src/mechanisms/hook_payload.rs`
- `crates/core/src/mechanisms/mod.rs`
- `crates/core/src/ports/camera.rs`
- `crates/core/src/ports/mod.rs`
- `crates/core/src/ports/raw_previews.rs`
- `crates/core/src/ports/store.rs`
- `crates/core/src/services/arrival.rs`
- `crates/core/src/services/editing_events.rs`
- `crates/core/src/services/mod.rs`
- `crates/core/src/services/photos.rs`
- `crates/infra/src/lib.rs`
- `crates/infra/src/rawpreview.rs`
- `crates/infra/src/spool.rs`
- `crates/infra/src/sqlite/store.rs`
- `crates/integrations/src/camera/via_capture_one.rs`
- `crates/integrations/src/lib.rs`
- `crates/testkit/src/clock.rs`
- `crates/testkit/src/fixtures/mod.rs`
- `crates/testkit/src/fixtures/synthetic_raw.rs`
- `crates/testkit/src/lib.rs`
- `crates/testkit/src/test_app.rs`
- `crates/testkit/tests/p05_capture_arrival.rs`
- `docs/implementation/master_plan.md`
- `docs/implementation/plans/P05_capture_arrival.md`

Report path: `docs/implementation/handoffs/implementer/handoff_P05_capture_arrival_implement_r1.md`. It is written after the checkpoint so it can name the SHA; the runner commits handoffs under flow §8.

## Checkpoint SHA

`8a873695a088d719b71e50c399a657305620cb62` — `CHECKPOINT (not approved): implement P05 capture arrival`. `git status --porcelain` was empty immediately after the commit, before this runner-owned handoff was written.

## Stamp

| Command | Result |
|---|---|
| `cargo test -p psm-testkit --test p05_capture_arrival` | 33/33 passed |
| `cargo test --workspace` | Green; 33 P05 boundary tests, earlier phase tests and unit tests all passed |
| `cargo clippy --workspace --all-targets -- -D warnings` | Green |
| `cargo fmt --all -- --check` | Green |
| `cd apps/desktop && npm run typecheck` | Green |
| `cd apps/desktop && npm test` | Green (no frontend test files in this phase) |
| `git diff --check` | Green |
| Read-only real Sony RAW probe | `candidate=Some((114850, 221976, 1088, 1080))`, `orientation=Some(1)` |

The first clippy pass found three collapsible `if` forms; they were corrected before the final green stamp. No pre-existing failures remain.

## Manual try-out

1. Open the configured Capture One session document and connect the Sony camera. Start Session Manager from `apps/desktop` with `npm run tauri -- dev`; the Health page should show the document and hooks ready.
2. Press the shutter in Capture One. Leave the app running for at least a few seconds so the hook, watcher, and jobs can finish. A capture in `<Capture>/<item folder key>/` joins that item's session; a capture in `_Unassigned` joins session 1.
3. Inspect the database with `sqlite3 "$HOME/Library/Application Support/com.loorenz.sessionmanager/session_manager.db" 'select id,session_id,raw_path,classification,captured_at,original_preview_path from photos order by id desc limit 5;'`. It should show one row per RAW. Run `sqlite3 "$HOME/Library/Application Support/com.loorenz.sessionmanager/session_manager.db" 'select source,hook,error from capture_events order by id desc limit 10;'` to see hook and watcher deliveries without a duplicate photo.
4. Open the `original_preview_path` reported by SQLite. It is the camera's embedded JPEG. The adjusted-preview completion and on-screen workspace arrive in P06 and P08.

## ⚠ OWNER DECISIONS REQUIRED (0)

none
