---
plan: P41_undo_history_ui
role: implement
round: 1
state: DONE
date: 2026-11-05
actor: codex
---

# What was built

Undo and redo now work for the open session from ⌘Z / ⌘⇧Z across the workspace, editor and open dialogs, while a focused text field keeps its own keys. The header shows Undo and Redo icons with step names in hover cards; the Edit menu tracks those same names and sends its selections through the same history path. Each result shows one named notice, and the delete toast's Undo uses history. Restored deleted photos become active. The editor follows a changed stored framing. Grid Split/Normal on multiple changed photos uses one Apply step. The old delete-only command and service export were retired.

# Tests → criterion rows

| Row | Behaviour test and boundary |
|---|---|
| C1(a) | `p41_undo_ui.test.tsx` C1(a): workspace keys call history; actual photos service has no `undoDelete` export |
| C1(b) | `p41_undo_ui.test.tsx` C1(b): focused Title input in the details dialog keeps both keys |
| C1(c) | `p41_undo_ui.test.tsx` C1(c): editor ⌘Z calls history; plain Z does not |
| C2(a) | `p41_undo_ui.test.tsx` C2(a): exact undo and redo notices, replaced under one toast id |
| C2(b) | `p41_undo_ui.test.tsx` C2(b): exact refusal reason, active tile unchanged |
| C2(c) | `p41_undo_ui.test.tsx` C2(c): exact empty-history notices |
| C3(a) | `p41_undo_ui.test.tsx` C3(a): ordered icon-only header controls, exact cards, absent on Sessions page |
| C3(b) | `p41_undo_ui.test.tsx` C3(b): same-session event updates cards and enabled state; foreign event ignored |
| C3(c) | `p41_undo_ui.test.tsx` C3(c): disabled Redo click does nothing |
| C4(a) | Updated `p08_workspace_ui.test.tsx` C2(b): late ⌘Z restores and selects deleted photo, dismisses toast |
| C4(b) | Updated `p08_workspace_ui.test.tsx` C2(c): delete toast's Undo calls history |
| C5(a) | `p41_editor_sync.test.tsx` C5(a): history revision sync makes Esc save the precise entry rect once, without a save during sync |
| C6(a) | `p41_undo_ui.test.tsx` C6(a): selection Split sends two exact re-fitted values in one Apply, patches both tiles |
| C6(b) | `p41_undo_ui.test.tsx` C6(b): one Split uses one set, no Apply |
| C7(a) | `p41_undo_ui.test.tsx` C7(a): menu setter follows loaded and event labels for the current session only |
| C7(b) | `p41_undo_ui.test.tsx` C7(b): menu Undo then Redo call their respective history commands in order |

All 16 rows assert the specified public or service boundary outcome. P08 and P33 targeted regressions passed. No core-mechanism unit test was added.

# Assumptions

- `useSyncExternalStore` backs the one open-session label store. The existing compose harness uses session id 5, so P41 cases use that id rather than the plan's example id 7; session isolation is asserted with a different id.
- An active-photo history result selects that photo and queues a workspace reload. A `history_changed` event also queues reload for attribute-only reversals; nearby workspace events are coalesced in one microtask.
- The editor uses composition `revision` to recognize an external stored change and waits until a local save is no longer pending.
- This phase's automated validation uses the fake services and does not need a live Capture One capture.

# Deviations

No product-behaviour or file-perimeter deviation. The P39 `photos_undo` compatibility test was retired with its command. The prior P40 review note about undoing a framing on a `needs_review` photo restoring `ok` belongs to backend semantics and remains recorded there; P41 did not change that behavior.

# Files changed

Checkpoint: `apps/desktop/src-tauri/src/{lib.rs,state.rs,commands/history.rs,commands/photos.rs}`; `apps/desktop/src/{App.tsx,services/history.ts,services/photos.ts,services/bindings.ts}`; `apps/desktop/src/features/history/{useHistory.ts,useHistoryKeys.ts,HistoryButtons.tsx,__tests__/p41_undo_ui.test.tsx}`; `features/compose/{ComposeMode.tsx,__tests__/harness.tsx,__tests__/p41_editor_sync.test.tsx}`; `features/shell/{Header.tsx,__tests__/p19_navigation.test.tsx}`; `features/workspace/{WorkspacePage.tsx,useKeyboard.ts,useWorkspaceState.ts,workspace.css,__tests__/p08_workspace_ui.test.tsx,__tests__/p19_drop.test.tsx,__tests__/p20_drop_continuity.test.tsx,__tests__/p22_workspace_tidy.test.tsx,__tests__/p26_double_drag.test.tsx}`; `docs/implementation/{master_plan.md,plans/P41_undo_history_ui.md}`.

This handoff is written after the checkpoint so it can contain the actual SHA; per flow §8, the runner commits handoffs. No other file was changed.

# Checkpoint SHA

`84fb44302d61a714a1dae41838ac6715376c5c17` (`CHECKPOINT (not approved): implement P41 undo history UI`). The tracked tree was clean immediately after this commit, before this runner-owned handoff was written.

# Stamp

| Command | Result |
|---|---|
| `cargo test -p psm-desktop export_bindings` | passed; regenerated `bindings.ts` |
| `cargo test --workspace` | passed, all Rust unit, integration and doc tests |
| `cd apps/desktop && npm test` | passed, 50 files / 369 tests |
| `cargo clippy --workspace --all-targets -- -D warnings` | passed |
| `cargo fmt --all -- --check` | passed |
| `cd apps/desktop && npm run typecheck` | passed |
| `git diff --cached --check` | passed before checkpoint |

Stamp tree: checkpoint `84fb44302d61a714a1dae41838ac6715376c5c17`; the handoff is the only subsequent worktree addition. No pre-existing failure. No live service was called by the suite.

# Manual try-out

1. Open a session and approve three photos. Press ⌘Z repeatedly, then ⌘⇧Z. The classifications and one-at-a-time named notices should follow the steps.
2. Hover the Undo and Redo icons left of the bell; use each enabled icon. Open Edit in the menu bar and choose a named Undo or Redo. Press ⌘Z once in the workspace and verify exactly one step reverses.
3. Enter Compose with R, save a framing change, then use ⌘Z / ⌘⇧Z while still in Compose. The framing should jump to the stored version.
4. Open Details with D. Type in a field and press ⌘Z while its cursor is inside: only that field's text should undo. Click outside the field and press ⌘Z: the session's latest saved step should undo.
5. Delete a photo and click the toast's Undo: the photo should return active. Repeat after allowing the toast to expire, using ⌘Z.
6. Select two rendered photos and choose Split from a tile menu. One ⌘Z should make both Normal again.

# ⚠ OWNER DECISIONS REQUIRED (0)

None.
