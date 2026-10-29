---
plan: P19_navigation_and_drop
role: implement
round: 1
state: DONE
date: 2026-10-29
actor: codex
---
# P19 implementer handoff

## What was built
Every routed page uses one header. Sessions is the root; the workspace and Settings return to Sessions, and Health returns to Settings. Back is the first control, ⌘[ or Ctrl+[ follows the fixed parent, and Escape follows it on Settings and Health when no layer owns the key. Dropped photos appear in their destination slot before the store responds. A rejected reorder reloads the stored order and shows an error toast.

## Tests → criterion rows
| Row | Boundary test | Exact assertion |
|---|---|---|
| C1(a) | `p19_navigation.test.tsx` C1(a) | Health first Back button and header actions; hash becomes `#/settings` |
| C1(b) | `p19_navigation.test.tsx` C1(b) | Settings first Back button, one Sessions control and Health action; hash becomes `#/` |
| C1(c) | `p19_navigation.test.tsx` C1(c) | Item and Unassigned workspace Back returns to Sessions page |
| C1(d) | `p19_navigation.test.tsx` C1(d) | Root title first, no Back, New session present |
| C2(a) | `p19_navigation.test.tsx` C2(a) | ⌘[ and Escape on Settings/Health, typing exception |
| C2(b) | `p19_navigation.test.tsx` C2(b) | Workspace Escape preserves route, dialog blocks ⌘[, ⌘[ works after dismissal |
| C3(a) | `p19_drop.test.tsx` C3(a) | Main order `[2,5,1]` before command settlement and after reload |
| C3(b) | `p19_drop.test.tsx` C3(b) | Refusal restores `[1,2,5]` and toasts once |
| C3(c) | `p19_drop.test.tsx` C3(c) | Cross-group drop immediately shows Main `[1,2,4]`, empty Condition; classification only |

## Assumptions
- Health and Settings keep the 900 px content column below the full-width header.
- Ctrl+[ follows ⌘[. Escape from Settings or Health does not save unsaved fields.
- Open layers are detected by `dialog`, `alertdialog` and `menu` roles. Review-key classification remains non-optimistic.
- P08 drag fixture setup is copied into the new P19 test files. The default dnd-kit drop animation remains; no live visual try-out was available in this session.

## Deviations
- The root page puts its `h1` directly in the header so C1(d)'s first-element outcome holds. Other titles use `.header-identity`. No backend or binding changes.

## Files changed
- `apps/desktop/src/App.tsx`
- `apps/desktop/src/features/shell/PageHeader.tsx`
- `apps/desktop/src/features/shell/Header.tsx`
- `apps/desktop/src/features/health/HealthPage.tsx`
- `apps/desktop/src/features/settings/SettingsPage.tsx`
- `apps/desktop/src/lib/keyboard.ts`
- `apps/desktop/src/features/workspace/useWorkspaceState.ts`
- `apps/desktop/src/styles/globals.css`
- `apps/desktop/src/features/shell/__tests__/p19_navigation.test.tsx`
- `apps/desktop/src/features/workspace/__tests__/p19_drop.test.tsx`
- `docs/implementation/master_plan.md`
- `docs/implementation/plans/P19_navigation_and_drop.md`
- `docs/implementation/handoffs/implementer/handoff_P19_navigation_and_drop_implement_r1.md`

## Checkpoint SHA
The checkpoint is the commit containing this handoff; resolve it with `git log -1 --format=%H -- docs/implementation/handoffs/implementer/handoff_P19_navigation_and_drop_implement_r1.md` after commit. Base before this checkpoint: `731b21475cc656a813e285c4a85257c547a7f1d5`.

## Stamp
| Command | Result |
|---|---|
| `cargo test --workspace` | PASS, exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS, exit 0 |
| `cargo fmt --all -- --check` | PASS, exit 0 |
| `cd apps/desktop && npm run typecheck` | PASS, exit 0 |
| `cd apps/desktop && npm test` | PASS, 11 files / 111 tests |
| `git diff --check` | PASS, exit 0 |

## Manual try-out
1. Open Settings from the gear. Back to Sessions is top left and Health is on the right. Click Back; reopen Settings and try ⌘[ and Escape.
2. Open Health from Settings. Its Back leads to Settings, with Retry checks on the right.
3. Open a session with three Main photos. Drag the first to the third slot; the tile should settle there and renumber immediately. Drag a Main photo to Condition; it should appear at the end of Condition.
4. In a workspace, Escape should keep the route; ⌘[ should return to Sessions. The Sessions header has no Back.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
