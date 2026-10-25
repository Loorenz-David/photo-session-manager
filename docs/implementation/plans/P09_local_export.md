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

Export `key` = `folder_key(current SKU)` (05B MC-22), not the stored capture-folder key; an export under a
previous SKU is left untouched.

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
### 2026-10-25 — implement r1 (opus, `pipeline-implementer`)
**Built.** `ExportService` (core, `services/export.rs`): snapshot of main/condition in position order
(unreviewed, deleted excluded) → `IntegrityCheck` on each (first `raw_changed`/`raw_missing` fails the run before
any render) → run row (`jobs` kind `render_export`, payload = frozen lists) → staging `.<key>.partial-<run>/` →
one `request_render(raw, Export)` per photo with its own tracking row → completions (P06 hook point
`ExportStaging::completed`) copy `outputs[0]` to the position name → when all are in: drop photos now deleted or
moved out of the session (R-I6 analogue) and renumber the rest, write `item.json`, swap with renames, delete the
old folder → `export_progress{done}`. Any failure (final render timeout via the new `ExportStaging::failed`,
provider error at issue, file error) deletes staging and emits `failed`; `<key>/` is untouched. Mechanisms
`export_file_name`, `item_json` (+ `ExportGroup`) in `mechanisms/naming.rs`; `ExportFs` (infra) for staging/swap.
Commands `export_run` (async, off the main thread), `export_open_folder`, `integration_get`; event
`export_progress`; header **Export** (when `integration == null`) with one progress/result toast.

**Judgment calls.** (1) A photo deleted mid-run is skipped and the remaining files are renamed to contiguous
positions with `n_g` of the survivors, so M15 "named by the rule" still holds. (2) Run id = id of a `render_export`
job row (no new table; no migration). (3) `item.json` is written after all renders are copied, before the swap.
(4) Progress: one `running` event at start and after every copied file. (5) "Open folder" runs `/usr/bin/open` on
`<export folder>/<key>/` through a new thin command (no opener plugin dependency). (6) A second `export_run` while
the session's run is in flight returns that run's id. (7) A session with no main/condition photo exports just
`item.json` (button not disabled; the plan does not ask for a disabled state). (8) Restart mid-export: the
interrupted run is abandoned (a `render_export` job handler logs and completes the requeued row, so it cannot
wedge the job queue); its hidden `.<key>.partial-<run>` folder stays beside `<key>/` (never inside it) until
removed by hand. (9) Fake output bytes: a JPEG SOI/EOI frame around `image from <job id>`
(`FakeCaptureOneScripting::output_bytes`), and the fake records every `process` call (`issued`, `jobs_for`).

**Deviations (outside the listed perimeter).** `crates/integrations/src/editing/capture_one/adapter.rs`: the
in-flight dedupe of `request_render` now applies to previews only; before, an export render for a RAW with a
preview (or an earlier export) in flight was silently dropped and the run would never finish. Export renders still
queue behind the same base name (MC-16). `crates/core/src/error.rs` (+ `raw_changed{photo_id}`,
`raw_missing{photo_id}` from the §6.6 registry) and the forced `CommandError` mapping in
`apps/desktop/src-tauri/src/commands/items.rs`; `crates/core/src/events.rs` (`ExportProgress`, `ExportState`);
`crates/integrations/src/lib.rs` (wiring); `crates/infra/src/lib.rs` (module). Two commands beyond the registry's
`export_run`: `export_open_folder`, `integration_get` (no settings command exists before P16).

**Observations.** `IntegrityCheck` treats a photo whose hash job has not run yet (`raw_hash` NULL) as
`raw_changed`; an export started within seconds of a capture could fail on it. Not changed here (P05/P06 code);
noted for the backlog. No live Capture One export probe was run in this session (the suite is hardware-free).

### 2026-10-25 — review r1 (opus, `pipeline-reviewer`): **APPROVED_WITH_NOTES**
**Gate.** Tracker `IMPLEMENTED`; checkpoint `4940766` in `git log`. HEAD `d996d42` adds only the implement handoff and
review prompt; `git status` clean. **Perimeter:** `git show --stat 4940766` lists 25 paths, equal to the handoff's
declared list (19 production, 4 test/harness, 2 docs). The paths outside the plan's expected-files list (adapter,
`error.rs`, `events.rs`, `commands/items.rs`, `src-tauri/src/lib.rs`, `integrations/lib.rs`, `infra/lib.rs`,
`services/mod.rs`, `commands/mod.rs`) are declared deviations or module wiring; the adapter change (dedupe for previews only) is needed for
M23/C6 and keeps MC-16's one render per base name. **Stamp** on that tree, all exit 0: `cargo test --workspace`,
clippy `-D warnings`, `fmt --check`, `npm run typecheck`, `npm test` 45/45 (3 files).
**Verified correct (by reading production, then the test):**
- C1(a): `snapshot` keeps Main/Condition only, sorted by `(position, id)`; test asserts the exact listing and no
  `process` for e/f. C2(a): each file's bytes equal `output_bytes` of the job issued for that variant, so a swapped
  file↔job mapping fails. C2(b): `export_file_name` width = `max(2, digits(n_g))`; exact 100-name listing.
- C3(a): `item_json` builds a sorted map, omits empty/absent values, `to_string_pretty` (2 spaces) + `\n`; test compares
  the row's bytes verbatim. C4(a): `ExportFs::swap` renames `<key>`→`.old-<run>`, staging→`<key>`, then removes old;
  test asserts the exact listing of `K/` and of the export root.
- C5(a): `IntegrityCheck` runs on every frozen photo before the run row, staging or any render; test asserts
  `Err(RawChanged{b})`, unchanged `process` count, byte-identical previous `K/`, root = `["K"]`, last event `failed`.
- C6(a): completions are matched by the adapter's live job map, then by `tracking_job_id` of an uncopied target; the
  test would fail if a stale completion were accepted (the target would be marked copied with the stray bytes).
  C6(b): adapter expiry is 120 s for Export, one re-issue, then `ExportStaging::failed` discards staging; test checks
  2 jobs per RAW after each tick, not-failed then failed, empty root.
- C7(a)/(b): `Header` shows Export only for `integration === null` (Unassigned keeps Assign product); one toast keyed
  `export` ends in "Exported n files" with an Open folder action; tests click through to `runExport(5)` /
  `openExportFolder(5)`.
- Lock order is `runs` → store everywhere; a completion arriving while `run` issues waits on the mutex and then finds
  the inserted run, so none is lost. H2 (only Capture One outputs are copied; RAWs are only read), H3, H5
  (`/usr/bin/open` on a folder), H7 (no provider ids in core export), H8 (commands wrap the service) hold. Tests write
  only under the temp dir (`with_fake_document` moves `export_folder`). No orphan tests.
**Assumptions** 1–11: none contradicts the intention (renumbering survivors matches MC-8 positions after a delete;
restart abandonment leaves `<key>/` exact). Reported to the owner in the narrative.
**Findings:** none blocking. **Notes:** 5 rows appended to `backlog.md` (NULL-hash false `raw_changed`; leftover
hidden folders after a crash; run mutex held while issuing; mid-run reclassify/reorder uses the frozen lists; fixtures
do not separate id order from position order).
**Lesson for the plans:** an export ordering row should reorder photos before exporting, so position order and
creation order differ.
