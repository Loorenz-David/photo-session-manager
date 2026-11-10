---
plan: P62_retention_after_publish
role: fix
round: 2
state: DONE
date: 2026-11-10
actor: claude-sonnet (pipeline-implementer)
---
# Handoff: P62 retention after publish (fix 1, round 2)

Gate: tracker showed P62 `CHANGES_REQUESTED`. The review's only blocking finding was F1, routed `production`.

## Finding → what changed → files
**F1 (B3 / H4): clearing is not resumable.**
- **What changed.** Each storage scan now also looks for files still on disk that belong to items already marked cleared, and queues them for deletion. The new function is `StorageService::queue_cleared_leftovers`, called from `scan` before `retention_scan`.
  - For every session whose item is marked cleared, it takes `item_files`, which applies the same shared-RAW rule. It queues the result as `QueuedKind::Item`, so the existing per-file `allowed_item_file` guard and P61's size/mtime check apply unchanged.
  - The rule logged is `retention`. The reviewer's correction allowed this, so nothing new is persisted and there is no migration 011.
  - There is no scan-request loop. The pass runs at the first scan after start, then at most every `REFRESH_EVERY` (5 minutes), tracked by `Work.leftovers_at`. A file that cannot be deleted is retried every 5 minutes at most.
  - The pass does not depend on a commerce adapter. The item was already verified and cleared.
  - The leftovers' bytes count towards the free space expected by the "Disk almost full" warning.
- **Test added.** `c1c_clearing_interrupted_by_a_restart_finishes_after_it_and_touches_nothing_else`. A published item is cleared and only its first batch is deleted, so some files are left. Then `kill_and_restart`, then ticks. Every file of the item is gone and the other item's files are untouched. The pre-fix defect is the one the reviewer probed (27 of 47 files left); I did not re-run the old code, because flow.md has no mutation regime.
- **Files.** `crates/core/src/services/storage.rs`, `crates/testkit/tests/p62_retention_after_publish.rs`.

## Divergences
None. Both files the review allowed were enough, and the rule did not need to persist.

## Files changed
This session (cycle-scoped), all in checkpoint `2aab879`:
- `crates/core/src/services/storage.rs`
- `crates/testkit/tests/p62_retention_after_publish.rs`
- `docs/implementation/master_plan.md` (P62 tracker row only → `IMPLEMENTED`, note: fix 1)
- `docs/implementation/plans/P62_retention_after_publish.md` (Review log entry)

This handoff is committed by the runner. No mutation-probe files. No archgraph in this repo.

## Checkpoint SHA
`2aab879` (`CHECKPOINT (not approved): P62 fix 1 — clearing resumes after a restart (F1)`)

## Stamp (code tree `2aab879`)
| Command | Result |
|---|---|
| `cargo test -p psm-testkit --test p62_retention_after_publish` | 10 passed, 0 failed (9 existing + `c1c`) |
| `cargo test --workspace --no-fail-fast` | 898 passed, 0 failed (the review's baseline was 896 passed + 1 timing flake; `c1c` adds one) |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean |
| `cargo fmt --all -- --check` | clean |
| `npm run typecheck` | clean |
| `npx vitest run --maxWorkers=4` | 62 files, 431/431 |

The first full workspace run was stopped by the 10-minute command limit. I re-ran it detached and it finished green.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
