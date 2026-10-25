---
plan: P09_local_export
role: implement
round: 1
state: DONE
date: 2026-10-25
actor: codex (run on Claude Opus 5.5, per prompt)
---
# P09 local export: implement r1 handoff

## What was built
A session can now be **finished without any shop connected** (H6, M7):
- **Export button.** In a session's header, when no integration is configured in settings, the primary action reads
  **Export** (Publish is shown instead only when `settings.integration` is set). Unassigned keeps "Assign product".
- **What Export does.** It takes the session's Main photos and Condition photos in their order (unreviewed and deleted
  photos are left out, N2), re-checks every RAW's fingerprint first (if one RAW changed or is missing, the export
  stops right away and names that photo; nothing is rendered), then asks Capture One to render each photo through the
  web recipe. Each render is copied into a hidden folder `.<SKU>.partial-<run>/` beside the export folder under its
  position name `<SKU>_01_main.jpg`, `<SKU>_01_condition.jpg`, … (two digits, three from 100 photos on). When all
  are in, `item.json` is written (sku, title, item type, barcode when set, empty attributes for now, the two file
  lists), and the hidden folder replaces `<export folder>/<SKU>/` by renames; the old folder is removed afterwards. So
  a re-export after deleting a photo leaves no stale file behind.
- **Failure is clean.** A render that never comes back is re-asked once after 120 s; a second 120 s without it fails
  the export. Any failure deletes the hidden folder and leaves the previous `<SKU>/` exactly as it was.
- **Feedback.** One toast follows the run: "Exporting n of N", then "Exported 4 files" with **Open folder** (opens
  the folder in Finder), or "Export failed".
- **Backend surface.** `ExportService` (core), `ExportFs` (infra), mechanisms `export_file_name` / `item_json`,
  commands `export_run`, `export_open_folder`, `integration_get`, event `export_progress{session_id, done, total, state}`.

## Tests → criterion rows
Rust file `crates/testkit/tests/p09_local_export.rs` (real SQLite, real adapter over `FakeCaptureOneScripting`,
real files in a temp dir; boundary = `ExportService::run`, the service `export_run` wraps one-to-one, plus
`complete_all_renders` / `tick` / `deliver_hook`). Frontend file
`apps/desktop/src/features/workspace/__tests__/p09_export_action.test.tsx` (real `Header` + `Toaster`, services mocked).

| Row | Test | Assertion vs row |
|---|---|---|
| C1(a) | `c1a_export_folder_holds_exactly_the_named_files` | sorted listing of `<export>/K/` equals exactly the 4 jpgs + `item.json`; `jobs_for(e)` and `jobs_for(f)` empty (no `process` for unreviewed/deleted); last event `(4,4,done)`. Exact. |
| C2(a) | `c2a_ten_mains_named_in_order_with_their_own_render` | for photo i: one job issued, `K_{i:02}_main.jpg` bytes == `output_bytes(job issued for that RAW's variant)`; listing exact. Exact (a file↔job swap fails it). |
| C2(b) | `c2b_hundred_mains_pad_to_three_digits` | listing equals `K_001_main.jpg … K_100_main.jpg` + `item.json`. Exact. |
| C3(a) | `c3a_item_json_bytes` | `item.json` string equals the row's bytes verbatim. Exact. |
| C4(a) | `c4a_second_export_leaves_no_stale_file` | after deleting b and re-exporting: `K/` lists exactly `K_01_main.jpg, K_02_main.jpg, item.json`; export root lists exactly `["K"]` (no `.K.partial-*`, `.K.old-*`). Exact. |
| C5(a) | `c5a_changed_raw_fails_before_any_render` | `run` returns `Err(RawChanged{photo_id: b})`; `process` count unchanged; every file name+bytes of the previous `K/` unchanged; root lists only `K`; last event `failed`. Exact. |
| C6(a) | `c6a_stray_completion_of_an_old_job_is_ignored` | stray `processing_done` with run 1's job id and different bytes delivered mid-run 2; final `K_01_main.jpg` == `output_bytes(run 2's job)`. Exact. |
| C6(b) | `c6b_export_render_timeout_reissues_once_then_fails` | staging dir exists after start; after `tick(120 s)` each RAW has exactly 2 jobs and the run is not failed; after the second `tick(120 s)` still 2 jobs each, last event `failed`, export root empty (staging removed). Exact. |
| C7(a) | `C7(a) with no integration configured the primary action reads "Export" …` | `integrationKind` → null: button named "Export" present, no Publish button; click → `runExport(5)`. Exact. |
| C7(b) | `C7(b) export_progress{state: done, total: 4} raises "Exported 4 files" with "Open folder"` | text "Exported 4 files" present; "Open folder" button present (click → `openExportFolder(5)`). Exact. |
| Core mechanism | `mechanisms::naming::tests::mc8_export_file_name_pads_to_group_width` | `export_file_name` widths for n_g 3/10/99/100/1000. |
| Core mechanism | `mechanisms::naming::tests::mc8_item_json_sorted_indented_absent_omitted` | nested attribute object sorted, UTF-8 unescaped, empty title omitted, barcode present, trailing newline. |

Every test in the two phase files maps to a row or to a listed core mechanism; no orphan tests.
**Red baseline: not captured.** The tests were written after the production code, so there is no recorded
failing-first run; I am stating that rather than reconstructing one.

## Assumptions
1. **Photo deleted during a run** (R-I6 analogue): skipped at the end; the remaining files are renamed to contiguous
   positions with the survivors' count as `n_g`, so names still follow MC-8 and `item.json` matches the files.
2. **Run id** = the id of a `jobs` row of kind `render_export` whose payload is the frozen lists; each photo's render
   has its own `render_export` tracking row. No new table, no migration.
3. **`item.json` timing**: written after every render is copied, just before the swap (delegated choice).
4. **Progress granularity**: one `running` event at start and one after each copied file; `done` carries the number
   of photo files written as both `done` and `total`.
5. **Open folder**: a thin command runs `/usr/bin/open <export folder>/<SKU>/` (no opener plugin added).
6. **Knowing "no integration"**: there is no settings command before P16, so a thin `integration_get` returns
   `settings.integration.kind` or null; the header shows Export on null (also if the call fails), Publish
   (disabled, later phase) when set, nothing while loading.
7. **Second click while running**: the button is disabled for that session until `done`/`failed`; a second
   `export_run` for the same session returns the running run's id.
8. **Empty session** (no main/condition photo): export writes just `item.json` (not blocked; the plan does not ask).
9. **Restart during an export**: the run is abandoned; a `render_export` job handler completes the requeued rows so
   the job queue is not blocked; the hidden `.<SKU>.partial-<run>` folder stays beside the export folder (never
   inside `<SKU>/`).
10. **Fake outputs**: `FakeCaptureOneScripting::output_bytes(job)` = JPEG SOI/APP0 … `image from <job>` … EOI, used for
    every render the test harness completes; the fake records each `process` call (`issued`, `jobs_for`).
11. **Export key** = `folder_key(current SKU)` (05B MC-22), the item's identity read at the start of the run.

## Deviations
- **Capture One adapter** (`crates/integrations/src/editing/capture_one/adapter.rs`, not in the perimeter):
  `request_render` used to silently ignore a request for a RAW that already had any render in flight. An export
  render for a RAW with a preview in flight (or a leftover render from an earlier failed export) was dropped, and the
  export would never finish. The dedupe now applies to preview-after-preview only; an export render still waits
  behind the same base name (MC-16 "at most one render in flight per output base name").
- **Error codes** `raw_changed{photo_id}`, `raw_missing{photo_id}` (registry §6.6) added to `psm_core::Error`,
  which forces the `CommandError` mapping in `apps/desktop/src-tauri/src/commands/items.rs`.
- **Events** (`crates/core/src/events.rs`): `AppEvent::ExportProgress` + `ExportState`; Tauri emission in `lib.rs`.
- **Wiring**: `crates/integrations/src/lib.rs` (App owns `export`, sets the previews export hook, registers the
  `render_export` handler), `crates/infra/src/lib.rs` (module).
- **Two commands beyond the registry's `export_run`**: `export_open_folder`, `integration_get` (see Assumptions 5, 6).
- **Test harness**: `TestApp::with_fake_document()` now points `export_folder` into the temp dir (so no test can write
  to `~/Pictures`); `TestApp::new()` keeps the defaults because P00's C1(a) asserts them.
- **Actor**: the prompt says `codex`; this session ran on Claude Opus 5.5 (tracker row says `opus`).

## Files changed
Production: `crates/core/src/services/export.rs` (new), `crates/core/src/mechanisms/naming.rs`,
`crates/core/src/services/previews.rs`, `crates/core/src/services/mod.rs`, `crates/core/src/ports/store.rs`,
`crates/core/src/error.rs`, `crates/core/src/events.rs`, `crates/infra/src/export_fs.rs` (new),
`crates/infra/src/lib.rs`, `crates/infra/src/sqlite/store.rs`, `crates/integrations/src/lib.rs`,
`crates/integrations/src/editing/capture_one/adapter.rs`, `apps/desktop/src-tauri/src/commands/export.rs` (new),
`apps/desktop/src-tauri/src/commands/mod.rs`, `apps/desktop/src-tauri/src/commands/items.rs`,
`apps/desktop/src-tauri/src/lib.rs`, `apps/desktop/src/services/bindings.ts` (regenerated),
`apps/desktop/src/services/export.ts` (new), `apps/desktop/src/features/shell/Header.tsx`.
Test harness and tests: `crates/testkit/src/fake_capture_one.rs`, `crates/testkit/src/test_app.rs`,
`crates/testkit/tests/p09_local_export.rs` (new),
`apps/desktop/src/features/workspace/__tests__/p09_export_action.test.tsx` (new).
Docs: `docs/implementation/master_plan.md` (P09 tracker row only), `docs/implementation/plans/P09_local_export.md`
(Review log). This handoff (committed by the runner).
Mutation-probe files: none (no mutation regime, flow §4).

## Checkpoint SHA
`4940766` — `CHECKPOINT (not approved): P09 local export — ordered web renders, MC-8 names, item.json, swap, Export action`
(tree clean after the commit).

## Stamp
Run at the checkpoint tree, 2026-10-25 (`PATH=$HOME/.cargo/bin:$PATH`; cargo is not on the runner shell's PATH):
| Command | Result |
|---|---|
| `cargo test --workspace` | exit 0 — all targets pass (P09: 8/8; P00–P08 unchanged and green; core unit tests incl. 2 new MC-8) |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo fmt --all -- --check` | exit 0 |
| `cargo test -p psm-desktop export_bindings` | regenerated `bindings.ts`; committed file equals the regenerated one |
| `cd apps/desktop && npm run typecheck` | exit 0 |
| `cd apps/desktop && npm test` | exit 0 — 3 files, 45 tests (2 new) |
Pre-existing failures: none. One inherited test (P00 C1(a), default settings) went red on my first harness change and
was satisfied by narrowing the change, not by editing the test.

## Manual try-out
Preconditions: Capture One open on the session document, recipe "PSM Web" present, no `integration` in `settings.json`.
1. Open a session and review every photo (approve, condition or delete). The header's right-hand action reads **Export**.
2. Click **Export**. A toast counts "Exporting n of N", then shows "Exported N files" with **Open folder**.
3. Click **Open folder**: Finder shows `~/Pictures/Session Manager Export/<SKU>/` with `<SKU>_01_main.jpg` … ,
   `<SKU>_01_condition.jpg` … and `item.json`.
4. Delete one main photo in the app and export again: the highest `_main.jpg` number is gone and nothing else is in
   the folder; no hidden `.<SKU>.partial-…` / `.<SKU>.old-…` folder stays in the export folder.
Not checked live in this session: no real Capture One export was run (the suite is hardware-free by rule).

## Notes for the reviewer / backlog
- `IntegrityCheck` (P05/P06 code) reports `raw_changed` for a photo whose hash job has not run yet (`raw_hash`
  NULL), so an export started seconds after a capture could fail on it. Left as is (outside this phase).

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
