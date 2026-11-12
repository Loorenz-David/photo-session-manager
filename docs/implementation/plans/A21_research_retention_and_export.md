---
phase: A21_research_retention_and_export
title: The research record is kept for a set number of days and can be exported — one zipped folder per export with every step, every call, the images that were sent and what you did with each suggestion, and never a key, a RAW or a deleted photo
difficulty: high
projection: yes
depends_on: [A14_research_resume_reuse_follow_ups]
---
# A21 — Retention of call records and Export research (08A §10A; 08B MC-A11 Retention, Export selection, Exclusions; MA9, MA16)

## Goal
Remove call records and the image files only they reference once they are older than `record_days`; keep runs,
steps, feedback, outbound rows and stage outputs. Settings → Research → **Export research**, for a date range or
chosen items, writes the MC-A11 folder and zips it, leaving out the images of photos that are deleted at export
time and aborting if any file contains a secret. **Not in this phase:** the report and replay tool (A22); the
only-if-cheap automatic export folder (not built).

## Read first
`master_plan.md` §6.3 (data dir) · `master_plan_research.md` §3 (keys), §6.4, §6.6 · book `01` (H2), `14` (MA9,
MA16), `08A` §10A, `08B` **MC-A11** (all but Replay identity), MC-A9 (record fields), MC-A5 (stored copies), MC-A7
(reuse needs the files), `10F` (the retention mechanism's elapsed-time rule) · code
`crates/core/src/mechanisms/retention.rs`, `crates/core/src/services/{retention.rs, storage.rs}` (launch and hourly
checks), `crates/infra/src/{research_files.rs, export_fs.rs}`, `crates/integrations/src/lib.rs` (background
threads), `crates/testkit/Cargo.toml` (`zip`), `services/research/gateway.rs` (`secret_values`),
`apps/desktop/src/features/settings/ResearchSection.tsx` (A19).

## Dependencies and gate
Tracker row and gate: `master_plan_research.md` §4. Notes: `backlog_research.md`.

## Files expected to change
New: `crates/core/src/services/research/{retention.rs, export.rs}`,
`crates/testkit/tests/suite/research/a21_research_retention_and_export.rs` (+ `mod` line).
Edited: `crates/core/src/services/research/mod.rs`, `crates/core/src/ports/store.rs`,
`crates/infra/src/{sqlite/store.rs, research_files.rs}`, `crates/integrations/src/lib.rs` (the hourly check),
`apps/desktop/src-tauri/src/{commands/research.rs, lib.rs}` (`research_export`),
`apps/desktop/src/services/{research.ts, bindings.ts}`, `apps/desktop/src/features/settings/ResearchSection.tsx`
(the Export button, range and item choice), own tracker row.

## Core mechanisms
None new (the expiry test is `now ≥ started_at + record_days × 86 400 000`, proven at the service in C1).

## Ordered tasks
1. `ResearchService::purge_expired(now_ms)`: delete each `research_calls` row whose `started_at + record_days ×
   86 400 000 ≤ now`, and every image file no remaining call references. Run at launch and hourly.
2. `ResearchService::export(selection, dest) -> ExportResult`: selection = runs whose `started_at` local date is in
   `[from, to]` inclusive, or all runs of chosen items. Write `research-export-<local YYYYMMDD-HHMMSS>/<item id>-
   <folder key>/run-<id>/{run.json, steps.json, writes.json, feedback.json, calls/<call id>.json, images/<sha256>.jpg}`
   (`run.json`: the run row with its settings and form snapshots; `writes.json`: the stage 7 output; `feedback.json`:
   the item's feedback rows at or after the run's start). A run whose calls expired gets `"call record expired"` in
   `run.json`.
3. Exclusions at export time: the image of a photo that is now `deleted`, or whose row is gone, is not written, and
   every request that referenced it carries `"withheld": "photo deleted"` in place of the image reference.
4. Secret check, fail closed: before zipping, scan every written file for every value of `secret_values()`, plain
   and URL-encoded. A hit deletes the partial folder, writes no zip, and returns the file's name (never the value).
5. Zip the folder; only `.json` and `.jpg` files exist in it.
6. Command `research_export({from, to} | {item_ids}, dest_folder) -> path`; the Settings button with a date range
   or an item choice and a folder picker.

## Criteria (7 criteria, 13 rows, 1 `[rule]`)
Boundary: `ResearchService::{purge_expired, export}` on `TestApp::with_research()` after scripted runs; outcome =
table rows, files on disk, the zip's entries.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | retention | `record_days` 30; a call started exactly 30 × 86 400 000 ms ago; another 1 ms younger | the first is deleted, the second kept | MA16 |
| C1(b) | retention, files | two expired calls share one image with a call that has not expired; one image is referenced only by expired calls | the shared file stays; the other is deleted | MC-A11 |
| C1(c) | retention, kept | after the purge | the runs, steps, feedback rows, outbound rows and stage outputs are all still there | MA16 (R-IA26) |
| C1(d) | schedule | launch with an expired call; then the manual clock advances one hour past another call's expiry | each is purged without an explicit call | MC-A11 |
| C2(a) | selection by date | runs started at local 2026-11-10 23:59 and 2026-11-11 00:01; range 2026-11-11 to 2026-11-11 | only the second run is exported | MA9 |
| C2(b) | selection by items | two items with runs; one chosen | every run of that item, none of the other | MA9 |
| C3(a) | layout | one finished run with 6 calls and 3 sent images | exactly: `run.json`, `steps.json`, `writes.json`, `feedback.json`, six files in `calls/`, three in `images/` named by SHA-256, under `<item id>-<folder key>/run-<id>/`; the zip holds the same entries | MA9 |
| C3(b) | content | the same export, after the photographer accepted one suggestion and edited another | `steps.json` equals the run's steps; each call file holds the recorded request and response byte for byte; `feedback.json` holds the two actions with suggested and final values | MA9 |
| C3(c) | expired | a run whose calls were purged | `run.json` says `call record expired`; no `calls/` entries; steps and writes present | MA9 |
| C4(a) | deleted photo | a photo sent in two calls is classified `deleted` before the export | its image file is absent from the export; both requests carry `"withheld": "photo deleted"`; other images present | MA16 |
| C5(a) | secret check | a call record that contains the model key value (written straight through the store, bypassing the gateway) | the export returns the offending file's name; no folder and no zip remain at the destination; the returned text does not contain the value | MA16 |
| C6(a) | file types | any export | every entry ends in `.json` or `.jpg`; no file is a RAW or a master, and no entry's bytes equal a RAW or master of the session | MA16 (H2) |
| C7(a) | command and button (UI test) | choose a range and a folder, click Export research | `research_export` is called with them; the returned path is shown | `[rule]` 08A §10A |

## Delegated choices
Zip crate use (`zip`, already a dependency of integrations) · where the partial folder is built (a temp folder
beside the destination, renamed at the end) · JSON field names inside `run.json` (they are read by A22; fix them
in `master_plan_research.md` §6 when written). Not delegated: fail closed on a secret.

## Manual try-out
1. After a few research runs on scratch data: Settings → Research → Export research, choose today, pick the Desktop.
2. Unzip the file: one folder per item, one per run; open `steps.json` and a file in `calls/` in a text editor; open
   an image in `images/` — it is the reduced copy that was sent.
3. Delete a photo that research used, export again: its image is not in the new export.

## Notes
- Hazard: purge must not delete an image a newer run reuses (C1(b)); reuse (A14) already refuses a unit whose file
  is gone.
- Review mode: this phase's module, `research::a14…::`, `p61_storage_cleanup_and_budget::`, the settings UI test,
  clippy, fmt, typecheck.

## Review log
