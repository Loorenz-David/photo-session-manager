---
plan: P26_frontend_leftovers
role: implement
round: 1
state: DONE
date: 2026-11-04
actor: codex
---
# P26 implementer handoff

## What was built
Settings shows the full folder path of the app's own Capture One session immediately after the photographer selects it, even before saving. In the workspace, two quick photo drags keep their combined on-screen order while either review command is pending; one refresh runs after the last command settles.

## Tests → criterion rows
| Row | Boundary test | Exact outcome |
|---|---|---|
| C1(a) | `p26_frontend_leftovers.test.tsx`: `C1(a) shows the full app session path before save and the newly picked folder` | Full backend default path after choosing the app session, no `~/`, then the picked folder path, with no save. |
| C2(a) | `p26_double_drag.test.tsx`: `C2(a) two quick drags keep the latest optimistic order until the last command reloads` | Calls `(1, 3)` then `(2, 2)`; `[5, 2, 1]` remains after the first resolve without a reload; exactly one reload after the second resolve keeps `[5, 2, 1]`. |

Both new tests failed before the production edits (C1(a): displayed `~/Pictures/Session Manager`; C2(a): order returned to `[1, 2, 5]` when the first command settled). The targeted suite including the inherited P22 and P24 cases passed after the edits (6/6).

## Assumptions
The last-settling review command always performs one reload and clears `reloadWanted`, including when no event was deferred. This uses the plan's delegated choice and keeps the existing single-command refresh.

## Deviations
None. No hardware or Capture One call was needed for this frontend phase.

## Files changed
- `apps/desktop/src/features/settings/SettingsPage.tsx`
- `apps/desktop/src/features/workspace/useWorkspaceState.ts`
- `apps/desktop/src/features/settings/__tests__/p26_frontend_leftovers.test.tsx`
- `apps/desktop/src/features/workspace/__tests__/p26_double_drag.test.tsx`
- `docs/implementation/master_plan.md` (P26 tracker row only)
- `docs/implementation/plans/P26_frontend_leftovers.md` (Review log only)
- `docs/implementation/handoffs/implementer/handoff_P26_frontend_leftovers_implement_r1.md`

## Checkpoint SHA
The checkpoint is the commit containing this handoff; resolve its SHA with `git log -1 --format=%H -- docs/implementation/handoffs/implementer/handoff_P26_frontend_leftovers_implement_r1.md`. The file cannot embed its own commit SHA in a single commit.

## Stamp
| Command | Result |
|---|---|
| `cargo test --workspace` | Passed, zero failures. |
| `cargo clippy --workspace --all-targets -- -D warnings` | Passed. |
| `cargo fmt --all -- --check` | Passed. |
| `cd apps/desktop && PATH=/Users/davidloorenz/.nvm/versions/node/v22.22.3/bin:$PATH npm run typecheck` | Passed. |
| `cd apps/desktop && PATH=/Users/davidloorenz/.nvm/versions/node/v22.22.3/bin:$PATH npm test` | Passed, 20 files / 131 tests. |

## Manual try-out
1. In Settings, choose another Capture One session, then click **Use the app's own session** without saving. The Effective path line should show `/Users/<you>/Pictures/Session Manager`, not a `~/` path. Choose another session again; it should immediately show that session's folder.
2. Open a session with at least four Main photos. Drag photo 01 to the last slot and immediately drag the new first photo to another slot. Both should stay where dropped without jumping back as the first drag finishes.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
