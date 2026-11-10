---
plan: P61_storage_cleanup_and_budget
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-10
actor: opus
---
# Review: P61 storage clean-up and budget (round 1)

**Verdict: APPROVED_WITH_NOTES.** No B1–B4 finding. Eight notes went to the backlog.

## Gate and stamp
- Tracker row `IMPLEMENTED`. Checkpoint `214d730` is in `git log`. HEAD `3382aa1` adds only the review prompt.
- `git show --stat 214d730`: 30 paths, equal to the handoff's **Files changed** list. Paths outside the plan's named list are all in **Deviations**: `Header.tsx`, `globals.css`, `services/storage.ts`, `camera/runtime.rs`, the port/domain/mechanism files, `test_app.rs`, `export.rs`.
- **`cargo test --workspace --no-fail-fast`: 885 passed / 0 failed.** The single full run reached the tool's 1-hour limit inside `p50`, after 805 passes and no failures. I re-ran `p50`–`p61` as named `--test` targets (80 passed) plus `cargo test --workspace --doc` (exit 0). The machine was heavily loaded: the installed Session Manager.app and Capture One were running, load 5–24.
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0. `cargo fmt --all -- --check`: exit 0. `npm run typecheck`: exit 0.
- **`npm test`:** three runs each had exactly one 5 s vitest timeout, in compose tests this phase does not touch and that import none of its code:
  - `p37_editor_chrome` "P38 C6(a) … 29 editor controls" twice, at 5068 and 5168 ms. It takes 634 ms alone.
  - `p32_controls` "C3(c) Rotate…" once. It passes alone.

  The same tree is **427/427 with `npx vitest run --maxWorkers=4`**. The implementer's stamp records the same p37 timeout. This is not B4: the tree is green and the reds are load timeouts in unrelated files. It is logged in the backlog.

## Verified correct
- **C1(a)** `StorageService::duplicate_candidates` takes a ledger-known duplicate's age from `completed_at` (app clock). `plan_cleanup` deletes at `now ≥ since + 24 h` with rule `duplicate_24h`, and `delete()` logs `storage.cleared rule=… path=… bytes=…` at INFO. The test asserts nothing at T+23 h, then exactly one `duplicate_24h` deletion at T+25 h, the file gone, the filed copy byte-intact and the log line. It would fail on a wrong threshold, and on a file-time age: the file time is wall clock while the test's clock is advanced.
- **C1(b)** A candidate needs a filed (`Photo`/`RenamedPhoto`, `Done`) row with the same SHA-256 whose file is still on disk. An unknown file is hashed (≤ 4 per scan) and never qualifies on a mismatch. `__dup` photos live outside `.duplicates/` and are never listed. `allowed()` also rejects any duplicate path whose parent is not `<capture root>/.duplicates`. The test runs 72 h later at 1 GB free, under budget pressure, and nothing is cleared.
- **C2(a)** `PreviewService` notes `(export, cache copy)` right after `copy_adjusted`, only for a JPEG directly in a `_session_manager` folder. Deletion needs size + SHA-256 equality with the cache copy at scan time, and unchanged size/mtime at delete time. The test drives a real preview render into `Output/_session_manager/DSC1.jpg`: exactly that export is cleared, the cache copy is byte-equal, and `Output/DSC7.tif` and `Output/other/DSC1.jpg` are intact.
- **C3(a)** MC-28 release was already in place. The refactor folds the old `state.masters.contains_key` skip into `in_use()`, with the same behaviour. The test publishes A to `Done` (not dirty), so both master dirs are gone, while B stays in `prepare` with its rendered master kept.
- **C4(a)** The routine rules run first (24 h duplicates, copied exports), then levels 1 → 3, oldest first within a level, stopping once `free ≥ min`. On a fake disk 2 500 bytes short, the test asserts the exact ordered list `[budget_duplicate, preview_copied, budget_master(older)]` and that the newer master is kept. It would fail on a wrong order or a missing stop.
- **C4(b)** `plan.short` sets the exact text in `storage.warning()` and in Health's `disk_almost_full` item (`ok=false`). `DiskWarning` is rendered in both `Header` branches (read), and the UI test shows the text in the warning element and in the section. The RAW of an unpublished photo, its preview cache and its output are byte-intact.
- **C5(a)** `tick` checks the pause first and returns `paused="render"` before any scan or delete. The pause covers capture download, FTP upload, renders in flight, take-back hold, export run and active publish item. A batch is at most 20 files, with a 2 s `next_batch_at`. Asserted: paused with no deletion, then 20, then 0 inside the pause, then 5.
- **Core mechanism:** the 4 `cleanup::tests` are on the one function the plan lists, input → output.
- **Settings:** the `storage` patch is validated (`min_free_gb` 0–100 000). A storage-only change is "Simple" (no runtime rebuild) and pushes the new minimum into the live service. A rebuild constructs the service with the saved minimum.
- **Assumptions 1–11:** none contradicts 10F, MC-28 or MC-33. They are put to the owner in plain words in the narrative. Assumption 2 is conservative and creates a P62 obligation (backlog).
- **H1–H8:**
  - H2: what can be deleted is limited, per kind, to `.duplicates/` copies (10F §1 amends MC-33), `_session_manager` JPEGs and `master-*.tif` under `<data>/masters`. No RAW, no output, no Capture One folder.
  - H4: deletes re-check identity and the masters still needed.
  - H7: core names only the existing `CaptureOne` skip (precedent: `arrival.rs`) and the app's own export sub-folder.
  - H8: the commands are thin.
  - H1, H3, H5, H6: untouched.
  - Tests use temp folders and a stand-in disk. No live service, no secret in the diff.

## Findings
None blocking (B1–B4).

## Notes written to backlog
1. Any active publish item pauses the whole storage job, including levels 1–2 and the warning. A publish stuck in `prepare` holds clean-up indefinitely.
2. The disk-full warning is only recomputed by unpaused scans, so it goes stale during long render backlogs.
3. Pre-phase duplicates are hashed back-to-back (4 × ~47 MB per 2 s). A first start with many of them keeps the disk busy for minutes.
4. A noted export whose `remove` fails stays noted and triggers a full scan every 2 s, forever.
5. The start-up match and the size line assume `<document>/Output/_session_manager`. A custom Capture One output location is missed.
6. → P62: duplicates of an item whose RAWs P62 clears no longer verify and would be kept forever. P62 must delete them explicitly (10F §3).
7. Registry and book drift: the new service, port, settings key, commands, Health code and testkit helper are not in the registry, and 10F still reads "not planned".
8. Compose UI tests time out at vitest's 5 s under full-suite load (p37, p32). They pass alone, and the suite is 427/427 at 4 workers.

## Carry-forward
| Note | Destination |
|---|---|
| 6 (duplicates of cleared items) | P62 plan |
| 7 (registry/book drift) | orchestrator, before P62 planning |
| 1, 2 (publish pause, stale warning) | owner try-out / P62 if the owner wants clearing during publish |
| 3, 4, 5, 8 | backlog (work-Mac timing, tidy-up phase) |

## Lessons for the plans
- C5(a)'s "paused while a publish is running" together with C4's "masters not needed by a running publish" left the implementer free to pause everything during any publish. A future plan should say whether levels 1–2 may run during a publish.
- The plan's C6(a) names `p33_split_grid` as the known intermittent. The intermittents seen now are `p37_editor_chrome` and `p32_controls` (load timeouts). The master plan's caveat list should be updated, or the timeouts fixed.

## Write perimeter (this session)
`docs/implementation/backlog.md` (8 lines appended) · `docs/implementation/master_plan.md` (P61 tracker row only) ·
`docs/implementation/plans/P61_storage_cleanup_and_budget.md` (Review log entry) ·
`docs/implementation/narratives/P61_storage_cleanup_and_budget.md` (new) · this handoff.
No production or test code was edited, and no probes were applied. Scratch logs are in `/tmp/p61rev/` (outside the repo). Nothing was committed.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
