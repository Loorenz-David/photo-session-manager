---
plan: P08_workspace_ui
role: fix
round: 2
state: DONE
date: 2026-10-25
actor: sonnet
---
# P08 workspace UI: fix 1 (round 2)

## Finding → what changed → files
**F1 (B1, production): a capture arriving in the open session did not become the stage photo.**
- Changed: in the `photo_added` branch of the `onWorkspaceEvents` handler, `activeRef.current = event.photo_id` became
  `setActiveId(event.photo_id)` (state and ref together). `setActiveId` joined the effect's dependency list.
  Result: the stage, A/C/⌫ and ←/→ all act on the newly arrived photo, including the first frame of an empty session.
- Test added (declared in the Review log as **candidate criterion C9**, traces to ch04 S-C): `C9 photo_added puts the new
  photo on the stage, and the next key acts on it`. Photo 1 (Main) is active; `photo_added{photo 2}` makes tile 2 the
  active one; `c` calls `classifyPhoto(2, 'condition')`. Observed red with the old line restored (1 failed / 23 passed),
  green with the fix.
- Files: `apps/desktop/src/features/workspace/useWorkspaceState.ts`,
  `apps/desktop/src/features/workspace/__tests__/p08_workspace_ui.test.tsx`.

## Divergences
None. The correction was implemented as written.

## Files changed (this cycle only)
- `apps/desktop/src/features/workspace/useWorkspaceState.ts`
- `apps/desktop/src/features/workspace/__tests__/p08_workspace_ui.test.tsx`
- `docs/implementation/master_plan.md` (P08 tracker row only → IMPLEMENTED)
- `docs/implementation/plans/P08_workspace_ui.md` (Review log entry appended)
- `docs/implementation/handoffs/implementer/handoff_P08_workspace_ui_fix_r2.md` (this file)
- No mutation probe files. (The check that the new test fails without the fix used `git stash` on the one production file and was restored byte-identical.)
- No architecture-graph delta (none present for this change; no symbol deleted or moved).

## Checkpoint SHA
The commit with subject `CHECKPOINT (not approved): P08 fix 1 …`, parent `2aa4ac2`. (A commit cannot contain its own hash; read it with `git log -1 --format=%h --grep='P08 fix 1'`.)

## Stamp
Tree: `2aa4ac2` + the four modified paths above (before the commit). All exit 0:
| Command | Result |
|---|---|
| `cargo test --workspace` | 183 passed, 0 failed |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean |
| `cargo fmt --all -- --check` | clean |
| `npm run typecheck` | clean |
| `npm test` | 2 files, 43 passed (was 42; +C9) |

(The project has no `npm run lint` script; Rust clippy/fmt and `tsc` are the lint gates, as in the review stamp.)

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
