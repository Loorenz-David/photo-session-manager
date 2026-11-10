---
plan: P62_retention_after_publish
role: fix
round: 3
state: DONE
date: 2026-11-10
actor: claude-sonnet (pipeline-implementer)
---
# Handoff: P62 retention after publish (fix 2, round 3)

Gate: the tracker showed P62 `CHANGES_REQUESTED`, and the plan's Review log carries the orchestrator's `FIX2_AUTHORISED`. The review's only blocking finding was F2, routed `production`. The authorisation chose both parts of the reviewer's correction (the `created_at <= cleared_at` filter and the Unassigned route for arrivals).

## Finding → what changed → files
**F2 (B3 / H2, B1 vs M41 and 10F §4): a RAW that arrives in a cleared item after clearing was deleted.**
- **(a) Retention only touches what the item had when it was cleared.** New `StorageService::cleared_item_photos(item_id, session_id)` returns the session's photos, limited to `created_at <= cleared_at` once the item is marked cleared. `item_files` uses it, and so do the leftover pass and the clearing itself. `allowed_item_file`, the per-file guard at delete time, uses it too. A later RAW is never queued and never allowed, and neither are its Capture One files, caches and hash-matched duplicates.
- **(b) A cleared item is never a capture or import target.** In `ArrivalService::accept`, a photo whose session is cleared goes to Unassigned (session 1), whichever way the session was picked (ledger delivery, import into the active session, or folder attribution). In `PhotosService::import`, with a cleared item active, the file is copied to `_Unassigned` and not to the item's folder.
- **Tests (P62 test file).**
  - `c1d_a_photo_that_joins_a_cleared_item_afterwards_is_never_deleted`: a RAW inserted into the cleared session after clearing, with Capture One files. After the leftover interval with the disk full, the RAW and its files are still on disk and not in the cleared list.
  - `c1e_what_arrives_while_a_cleared_item_is_active_goes_to_unassigned`: a capture through the ledger and an import while the archived item is active. The item gains no photo, and both files are in Unassigned.
  - Both tests were seen failing with their change reverted (c1d: "the new RAW was deleted"; c1e: "the cleared item gained nothing"), then the change was restored. This was done on my own tree, in the working copy, and restored, not as a mutation ledger.
- **Files.** `crates/core/src/services/storage.rs`, `crates/core/src/services/arrival.rs`, `crates/core/src/services/photos.rs`, `crates/testkit/tests/p62_retention_after_publish.rs`.

## Divergences
None. The orchestrator's authorisation widened the review's file list to `arrival.rs` and `photos.rs`, which I touched only for part (b). The reviewer's probe imported through `PhotosService::import`. Because part (b) now routes that import to Unassigned, `c1d` (storage boundary) inserts the photo through the store, and `c1e` covers the import and capture paths.

## Judgment calls (also in the plan's Review log)
1. The redirect sits in `accept`, after the session is chosen, so one rule covers captures and imports.
2. The boundary is `<=`: a photo created in the same millisecond as the clearing counts as part of the cleared set.
3. A photo moved by hand into a cleared session keeps its older `created_at` and is still treated as part of it. The app does not offer moving into a read-only item (10F Q3).

## Files changed
This session (cycle-scoped):
- `crates/core/src/services/storage.rs`
- `crates/core/src/services/arrival.rs`
- `crates/core/src/services/photos.rs`
- `crates/testkit/tests/p62_retention_after_publish.rs`
- `docs/implementation/master_plan.md` (P62 tracker row only → `IMPLEMENTED`)
- `docs/implementation/plans/P62_retention_after_publish.md` (Review log entry appended)
- `docs/implementation/handoffs/implementer/handoff_P62_retention_after_publish_fix_r3.md` (this file)

No migration, port or infra file changed. No mutation probe left a file behind; the temporary reverts were restored and `git status` shows only the paths above. No architecture-graph tool is in this project's flow, so there is no graph delta.

## Checkpoint SHA
`6f0a234964f67f3a0cebf5ade95011d5c1e3577e` (subject `CHECKPOINT (not approved): P62 fix 2 …`). The SHA was added to this file after the commit; the runner commits that edit.

## Stamp (code tree = this checkpoint)
| Command | Result |
|---|---|
| `cargo test --workspace --no-fail-fast` | The run reached the 30-minute background limit inside the last test file (as the reviewer's did). 55 binaries finished: 888 passed, 0 failed. The last file, P62, was rerun alone: 12 passed, 0 failed. Doc tests: 0 tests, clean. **Total 900 passed, 0 failed.** |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean |
| `cargo fmt --all -- --check` | clean |
| `npm run typecheck` (apps/desktop) | clean |
| `npx vitest run` (apps/desktop) | not rerun: no frontend file changed in this cycle (the reviewer's last run was 431/431) |

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
