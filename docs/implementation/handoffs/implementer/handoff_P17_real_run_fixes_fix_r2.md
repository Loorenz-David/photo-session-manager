---
plan: P17_real_run_fixes
role: fix
round: 2
state: DONE
date: 2026-10-26
actor: codex
---

# P17 fix 1 handoff

## Finding → what changed → files

**F1 (production, H4): settings rebuild recovery requeued work still owned by live workers.** `App::build` now performs `requeue_running_jobs` and active-publish re-enqueue only when `start_backup` is true. That flag marks a real start; settings rebuilds pass false, while real starts retain crash recovery. Added one stored-state behavior test: it claims the active publish item's `publish_step`, performs a non-theme settings save, then checks that the job remains `running` and exactly one `publish_step` job exists for that item.

Files: `crates/integrations/src/lib.rs`; `crates/testkit/tests/p17_real_run_fixes.rs`.

## Divergences

None. Applied the correction and stored-state test as written.

## Files changed

- `crates/integrations/src/lib.rs`
- `crates/testkit/tests/p17_real_run_fixes.rs`
- `docs/implementation/plans/P17_real_run_fixes.md` (Review log fix 1 entry)
- `docs/implementation/master_plan.md` (P17 tracker → `IMPLEMENTED`, fix 1 note)
- `docs/implementation/handoffs/implementer/handoff_P17_real_run_fixes_fix_r2.md` (this handoff)

The checkpoint commit contains only the two production/test files allowed by F1. The plan, tracker, and handoff are runner-owned coordination artifacts.

## Checkpoint SHA

`43adbee106319609b5ca480b0b2cbedece2cffe9` — `CHECKPOINT (not approved): fix P17 settings recovery`.

## Stamp

| Command | Result |
|---|---|
| `cargo test --workspace` | Green; 406 Rust tests, 0 failed. |
| `cargo clippy --workspace --all-targets -- -D warnings` | Green. |
| `cargo fmt --all -- --check` | Green. |
| `cd apps/desktop && PATH=/Users/davidloorenz/.nvm/versions/node/v22.22.3/bin:$PATH npm run typecheck` | Green. |
| `cd apps/desktop && PATH=/Users/davidloorenz/.nvm/versions/node/v22.22.3/bin:$PATH npm test` | Green; 92 tests in 7 files. |
| `git diff --check` | Green. |

## ⚠ OWNER DECISIONS REQUIRED (0)

None.
