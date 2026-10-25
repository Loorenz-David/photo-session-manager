---
phase: P09_local_export
title: Local export — ordered web renders, naming, item.json, the Export action
difficulty: high
projection: no
depends_on: [P06_live_previews, P02_review_order]
---
# P09 — Local export: ordered web renders, naming, item.json, the Export action

## Goal
Complete a session without any integration (H6, M7): re-verify RAW hashes, render every main and condition photo
through the web recipe into a staging folder under the position-based names of MC-8, write `item.json`, and swap
the folder atomically so no stale file survives (M15). Expose `export_run`, the `export_progress` event and the
header **Export** action shown when no integration is configured. This export path is also step one of every
publish (P13 reuses it). **Not in this phase:** attribute values in `item.json` (P10 adds that row; here the object
is empty), uploads (P13).

## Read first
master plan §6.3 (export folder default), §6.6 (`RenderPurpose`, `JobKind::RenderExport`), §6.7 · book `01`, `14`,
`07` §Completion without an integration, `04` S-L · contracts `05A` MC-8 (Export layout, item.json), MC-3 (checks a),
MC-2/N2 (unreviewed never export: `15` R19); `10A` MC-16 (export purpose, 120 s timeout) · `07A` MC-10 (R-I6: a
frozen photo deleted meanwhile is skipped — the same rule applies to an export's photo list).

## Files expected to change
`crates/core/src/services/export.rs` (new: `ExportService`), `crates/core/src/mechanisms/naming.rs` (`export_file_name`,
`item_json`), `crates/core/src/services/previews.rs` (export completion hook point filled), `crates/infra/src/export_fs.rs`
(new: staging dir, swap with renames, cleanup), Store trait + impl (export run id, photo list snapshot),
`apps/desktop/src-tauri/src/commands/export.rs` (`export_run`), bindings, `apps/desktop/src/features/shell/Header.tsx`
(Export action when `integration == null`), `apps/desktop/src/services/export.ts`, `apps/desktop/src/features/workspace/__tests__/p09_export_action.test.tsx`,
`crates/testkit/src/{fake_capture_one.rs (export render writes a JPEG whose bytes encode the job id), test_app.rs (`complete_all_renders()`)}`,
`crates/testkit/tests/p09_local_export.rs` (new).

## Core mechanisms (unit tests allowed)
- `mechanisms::naming::export_file_name(key, position, n_g, group) -> String` (`<key>_<NN>_<main|condition>.jpg`, `NN` zero-padded to `max(2, digits(n_g))`).
- `mechanisms::naming::item_json(item, attributes, main_names, condition_names) -> String` (UTF-8, sorted keys, two-space indent, trailing newline, absent values omitted).

## Ordered tasks
1. `ExportService::run(session_id) -> ExportRunId`: snapshot the photo lists (main, condition in position order;
   unreviewed and deleted excluded), run `IntegrityCheck` on each (any `raw_changed`/`raw_missing` → fail before any
   render, naming the photo), create `<export folder>/.<key>.partial-<run id>/`, issue `render(raw, Export)` per photo
   with its target name; completions (P06 hook point) copy `outputs[0]` to the staging name; progress events
   `{done, total, state: running}`; a photo deleted during the run is skipped (R-I6 analogue); when all are in:
   write `item.json`, rename the old `<key>/` to `.<key>.old-<run id>`, rename staging to `<key>/`, delete the old
   folder; emit `export_progress{state: done}`. Any failure: delete staging, leave `<key>/` as it was, emit `failed`.
2. Render timeout for Export (120 s, one re-issue) comes from P06; a final timeout fails the run.
3. Command, bindings; header action + toast ("Exported n files · Open folder" via the opener plugin or `open`).
4. Tests, with `FakeCaptureOneScripting` writing distinguishable bytes per job id so the file ↔ job mapping is observable.

## Criteria (7 criteria, 10 rows; 1 `[rule]` row)
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | `export_run(S)` + `complete_all_renders` | key `K`; main [a,b,c], condition [d], unreviewed [e], deleted [f] | `<export>/K/` contains exactly `K_01_main.jpg`, `K_02_main.jpg`, `K_03_main.jpg`, `K_01_condition.jpg`, `item.json` and nothing else; no render was requested for e or f | M7, M15 (N2, H2) |
| C2(a) | export | 10 main photos | names `K_01_main.jpg` … `K_10_main.jpg`; each file's bytes are the fake output of the job issued for **that** RAW | MC-8, M23 |
| C2(b) | export | 100 main photos | `K_001_main.jpg` … `K_100_main.jpg` | MC-8 (`max(2, digits(n_g))`) |
| C3(a) | `item.json` | item `sku K`, no barcode, title "Oak", type "Sideboard"; lists as in C1 | file bytes equal exactly `{\n  "attributes": {},\n  "condition": [\n    "K_01_condition.jpg"\n  ],\n  "item_type": "Sideboard",\n  "main": [\n    "K_01_main.jpg",\n    "K_02_main.jpg",\n    "K_03_main.jpg"\n  ],\n  "schema": 1,\n  "sku": "K",\n  "title": "Oak"\n}\n` (barcode omitted) | M7, MC-8 |
| C4(a) | second export after removing one main photo | first export had 3 mains, now 2 | `K_03_main.jpg` absent; `.K.partial-*` and `.K.old-*` absent | M15 (no stale file) |
| C5(a) | export | photo b's RAW appended to after hashing | run `failed` with `raw_changed{photo_id: b}` before any render (fake received no `process`); previous `<export>/K/` untouched; no partial folder | M10 |
| C6(a) | export with a stray completion | `processing_done` for an old job id of the same RAW delivered mid-run | ignored; the final file bytes come from this run's job | M23 |
| C6(b) | export, no completion | `tick(120 s)` then `tick(120 s)` | one re-issue per RAW, then run `failed`; staging removed | MC-16 (export timeout) |
| C7(a) | `Header` render (frontend) | `settings.integration == null` | primary action reads "Export"; click → `export_run(session)` | H6 (core workflow without integration) |
| C7(b) | `export_progress{state: done, total: 4}` event (frontend) | — | toast "Exported 4 files" with "Open folder" | `[rule]` ch07 §Completion without an integration |

## Delegated choices
Progress granularity, how the test makes the fake's export output distinguishable per job, the opener used for
"Open folder", whether `item.json` is written before or after the renders are copied (before the swap in any case).

## Manual try-out
1. Review a session (every photo approved, conditioned or deleted) → header shows "Export".
2. Click Export; the toast reports the count; `~/Pictures/Session Manager Export/<SKU>/` holds `<SKU>_01_main.jpg`…
   and `item.json`.
3. Delete one main photo in the app, export again → the last `_main.jpg` number is gone; no extra files remain.

## Notes
- The web recipe's root is "Output location" (not settable by script, MC-16); the app never computes output paths —
  it copies `outputs[0]` from the completion payload.
- Estimated size: ~10 files, ~1 300 lines.

## Review log
(empty)
