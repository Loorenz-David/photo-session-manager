---
plan: P18_pending_state_ui
role: fix
round: 2
state: DONE
date: 2026-10-26
actor: codex
---

# P18 fix 1 handoff

## Finding → what changed → files

**F1 (B1, production):** Shop hits from the previous query reappeared as completed results while a new shop query was pending.

Both item-search dialogs now record which query produced the shop results and render those results only when that key matches the current title. Each new shop lookup returns the heading to its searching state. Added behavior cases for both dialogs that first resolve `a`, then start `ab` with its response held, and assert that the `a` hit disappears until the `ab` result arrives.

Files: `apps/desktop/src/features/sessions/NewSessionDialog.tsx`, `apps/desktop/src/features/workspace/PickerDialogs.tsx`, `apps/desktop/src/features/sessions/__tests__/p18_item_search.test.tsx`.

## Divergences

None. No judgment call was needed.

## Files changed

Checkpoint perimeter:

- `apps/desktop/src/features/sessions/NewSessionDialog.tsx`
- `apps/desktop/src/features/workspace/PickerDialogs.tsx`
- `apps/desktop/src/features/sessions/__tests__/p18_item_search.test.tsx`

Closeout artifacts updated:

- `docs/implementation/master_plan.md` — P18 tracker row only (`IMPLEMENTED`, fix 1).
- `docs/implementation/plans/P18_pending_state_ui.md` — fix 1 Review log entry.
- `docs/implementation/handoffs/implementer/handoff_P18_pending_state_ui_fix_r2.md` — this handoff.

## Checkpoint SHA

`bcd4f51e60bd8f9b37b3968d55749c23afbdbc41` — `CHECKPOINT (not approved): fix P18 stale shop hits`.

## Stamp

Ran on checkpoint `bcd4f51e60bd8f9b37b3968d55749c23afbdbc41`:

| Command | Result |
|---|---|
| `cargo test --workspace` | PASS, 407 tests |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo fmt --all -- --check` | PASS |
| `cd apps/desktop && npm test` | PASS, 9 files / 102 tests |
| `cd apps/desktop && npm run typecheck` | PASS |
| `git diff --check` | PASS |

Focused behavior file: `p18_item_search.test.tsx`, 4 tests passed. No live service calls. Checkpoint commit contains only the three perimeter files above.

## ⚠ OWNER DECISIONS REQUIRED (0)

none
