---
plan: P09_local_export
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-10-25
actor: opus
---
# P09 local export: review r1

**Verdict: APPROVED_WITH_NOTES.** Production code satisfies all 10 criterion rows at their boundaries, and each row's
test can fail on a wrong outcome. Nothing blocks under flow §5 (B1–B4). 5 notes went to the backlog.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Verified correct
- **Gate.** Tracker row `IMPLEMENTED`. Checkpoint `4940766` is in `git log`. HEAD `d996d42` adds only the implement
  handoff and review prompt (no `crates/` or `apps/` delta from the checkpoint), and `git status --porcelain` is clean.
- **Perimeter.** `git show --stat 4940766` lists 25 paths. They equal the handoff's "Files changed" list exactly: 19
  production, 4 test/harness, 2 docs. No extra or missing paths. The paths outside the plan's expected-files list are
  declared deviations (the adapter dedupe, error codes and `CommandError` mapping, event) or module wiring. The adapter
  change is sound: it stops an export render from being silently dropped behind a preview (needed by C1/C6, M23), and
  renders still queue one per base name (MC-16).
- **Stamp (B4)** on the checkpoint tree with the master plan §8 commands, all exit 0: `cargo test --workspace` (P09
  8/8, all other targets green), `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`,
  `npm run typecheck`, `npm test` (3 files, 45/45).
- **C1(a)** `snapshot` keeps Main/Condition only, sorted by `(position, id)`. The test asserts the exact sorted listing of
  `K/`, that the fake received no `process` for e/f, and that the last event is `(4,4,done)`.
- **C2(a)** Each target's file comes from `outputs[0]` of its own tracking job. The test compares every
  `K_NN_main.jpg` to `output_bytes(job issued for that variant)`, so a file↔job swap fails it. **C2(b)**
  `export_file_name` width = `max(2, digits(n_g))`. The test checks the exact 100-name listing.
- **C3(a)** `item_json` uses a sorted map, omits absent or empty values, pretty-prints with 2 spaces and adds `\n`. The
  test compares against the row's bytes verbatim.
- **C4(a)** `ExportFs::swap` renames `<key>`→`.<key>.old-<run>`, then staging→`<key>`, and only then removes old (and
  restores old if the second rename fails). The test asserts the exact listing of `K/` and of the export root.
- **C5(a)** `IntegrityCheck` runs on every frozen photo before the run row, the staging folder or any render. The test
  asserts `Err(RawChanged{photo_id: b})`, an unchanged `process` count, a byte-identical previous `K/`, a root of
  `["K"]` and a last event of `failed`.
- **C6(a)** A completion must match the adapter's live job map and then an uncopied target's `tracking_job_id`. If a
  stray completion were accepted, the target would be marked copied with the stray bytes, and the test's byte compare
  would fail. **C6(b)** The adapter expires Export after 120 s with one re-issue, and on the second expiry
  `PreviewService::tick` routes `render_export` to `ExportStaging::failed`, which discards staging. The test checks 2
  jobs per RAW after each tick, not-failed then failed, and an empty root.
- **C7(a)/(b)** `Header` renders Export only when `integration === null` (Unassigned keeps Assign product). The test
  clicks it and asserts `runExport(5)`. On `done`, one toast keyed `export` reads "Exported 4 files" with an Open folder
  action, and the test clicks through to `openExportFolder(5)`.
- **Concurrency.** Lock order is `runs` → store on every path. A completion that arrives while `run` is still issuing
  waits on the mutex and then finds the inserted run, so no completion is lost. The cost is latency (see backlog).
- **Hard constraints.** H2 (only Capture One outputs are copied, RAWs are only read), H3, H5 (`/usr/bin/open` on a
  folder), H7 (no provider ids in core export) and H8 (`export_run`, `export_open_folder`, `integration_get` are thin)
  hold. No secrets or live services are involved. `with_fake_document` moves `export_folder` into the temp dir.
- **Tests.** Every test traces to a row or to a listed core mechanism (`export_file_name`, `item_json`). No orphans.
  The over-engineering test passes: assertions are on files, bytes, events and the calls the fake received.
- **Assumptions 1–11.** None contradicts the intention. Renumbering the survivors after a mid-run delete matches MC-8
  positions after a delete. Abandoning a run on restart leaves `<key>/` exact (M15). All are reported in plain words in
  the narrative.

## Findings
None blocking.

## Notes written to backlog
1. P05/P09 → P13: `IntegrityCheck` reports and persists `raw_changed` when `raw_hash` is still NULL (the hash job has not
   run), so an export seconds after a capture fails and wrongly marks the photo.
2. P09 → P13: a restart mid-export leaves `.<key>.partial-<run>` behind forever. A crash between the swap renames leaves
   the last export under `.<key>.old-<run>`. `prepare` should sweep or restore these.
3. P09: `run` holds the run mutex while issuing every render, so completion delivery (previews too) stalls for the
   whole issue loop.
4. P09: when a run ends, only deletion or a move is re-checked. A photo reclassified or reordered mid-run exports by the
   frozen lists.
5. P09 (test): the fixtures create photos in id order, which equals position order, so an id-ordered export would pass.
   Position order is verified by reading.

## Lessons for the plans
- An export-ordering row should reorder photos before exporting, so that position order and creation order differ.
- The P06 → P09 backlog note ("`RenderCompletion` carries no publish item") is resolved without changing
  `RenderCompletion`: each export render has its own tracking job row, which identifies the target.

## Write perimeter (this session)
- `docs/implementation/master_plan.md` (P09 tracker row only → `APPROVED`)
- `docs/implementation/plans/P09_local_export.md` (Review log entry appended)
- `docs/implementation/backlog.md` (5 rows appended)
- `docs/implementation/narratives/P09_local_export.md` (new)
- this handoff (new)
- No production or test file was touched. No probes (flow §4). No commits. Scratch log `/tmp/p09_review_stamp.log`
  (outside the repo).
