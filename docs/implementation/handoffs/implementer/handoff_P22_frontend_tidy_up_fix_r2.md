---
plan: P22_frontend_tidy_up
role: fix
round: 2
state: DONE
date: 2026-10-29
actor: codex
---
# P22 fix 1 handoff

## Finding → what changed → files

- **F1 (B1):** A Main photo held over an unreviewed tile showed `00`, while `useDrop` assigned the end-of-Main slot. Added one `projectedSlot` helper and used it for both the overlay projection and `useDrop`, so a target absent from the ordered group maps to `group.length`. Extended C3(b) to hold over an unreviewed tile, assert the overlay's `03` destination, then assert the release calls `reorderPhoto(1, 3)`. Files: `apps/desktop/src/features/workspace/WorkspacePage.tsx`; `apps/desktop/src/features/workspace/__tests__/p22_workspace_tidy.test.tsx`.

## Divergences

None. Implemented the correction as written; no unrelated changes.

## Files changed

- `apps/desktop/src/features/workspace/WorkspacePage.tsx`
- `apps/desktop/src/features/workspace/__tests__/p22_workspace_tidy.test.tsx`
- `docs/implementation/master_plan.md` (P22 tracker row)
- `docs/implementation/plans/P22_frontend_tidy_up.md` (Review log)
- This handoff

## Checkpoint SHA

This handoff is part of the checkpoint commit; resolve its SHA with `git log -1 --format=%H -- docs/implementation/handoffs/implementer/handoff_P22_frontend_tidy_up_fix_r2.md`.

## Stamp

- Targeted C3(b): 2 tests passed.
- `npm test`: 15 files, 124 tests passed.
- `npm run typecheck`: exit 0.
- `cargo test --workspace`: exit 0; 421 tests passed, 0 failed.
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0.
- `cargo fmt --all -- --check`: exit 0.
- `git diff --check`: exit 0.
- No standalone frontend lint script is defined in `apps/desktop/package.json`.

## ⚠ OWNER DECISIONS REQUIRED (0)

none
