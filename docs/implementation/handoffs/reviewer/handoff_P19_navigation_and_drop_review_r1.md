---
plan: P19_navigation_and_drop
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-10-29
actor: opus
---
# P19 review r1: APPROVED_WITH_NOTES

All 9 criterion rows hold at their boundaries. The perimeter equals the declared files, and the suite, lint and typecheck are green. There are no blocking findings. 4 notes went to the backlog. Tracker → APPROVED. The owner narrative is written.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Verified correct
- **Gate:** tracker `IMPLEMENTED`. Checkpoint `3058be84c41f8fe9e9ce63b739988402e708cfb8` is in `git log` and is the commit containing the implementer handoff.
- **Perimeter:** `git show --stat 3058be8` lists 13 paths, exactly the handoff's "Files changed". Nothing is extra or missing. `WorkspacePage.tsx` was not needed (the plan's "only if"). After `3058be8` only the runner's review prompt was added.
- **Suite (re-run on HEAD `156faf6`, master-plan commands):** `cargo test --workspace` 407 passed / 0 failed. `cargo clippy --workspace --all-targets -- -D warnings` exit 0. `cargo fmt --all -- --check` exit 0. `npm run typecheck` exit 0. `npm test` 11 files / 111 tests passed.
- **Rows that can fail:** both P19 test files were copied into a throwaway `git worktree` at the base `731b214` (pre-phase production code, untouched) and run there. Result: 8 of 9 red (C1(a), C1(b), C1(c), C2(a), C2(b), C3(a), C3(b), C3(c)). C1(d) is green on the base by construction: the root already had the `h1` first and no Back. It is a regression guard that fails if the root gains a Back control or loses its leading title. The worktree was removed. No file in the real tree was edited.
- **C1(a):** `HealthPage` → `PageHeader back={Settings, #/settings}`. The test asserts the first banner button is named `Back to Settings`, that `Retry checks` is present, that the exact names `Sessions`/`Settings` are absent (the aria-label overrides the visible "Settings" text, so the Back button does not collide), and that the click leads to `#/settings`.
- **C1(b):** the filter counts a button if its aria-label or its text mentions "Sessions", so a leftover "Sessions" button would make the count 2. The `Health` action is present, and a click on Back leads to `#/`.
- **C1(c):** the item workspace (5) and Unassigned (1) are rendered in turn. Back is the first button, and Unassigned shows `Unassigned photos`. A click on Back leads to `#/` and renders the Sessions page (`New session` present, no `.photo-panel`), through App's existing hashchange routing. The test's fixture uses an empty `listSessions`, not the "one row" the plan names. The outcome is header-only, so the row is unaffected.
- **C1(d):** `header.firstElementChild` is an `H1` reading `Session Manager`. No button name starts with `Back`.
- **C2(a):** ⌘[ and Escape on `window` move Settings to `#/` and Health to `#/settings`. Escape dispatched on the focused `Client secret` input keeps `#/settings` (`isTyping`). On the base tree the keys do nothing, so the test is red there.
- **C2(b):** in the workspace, Escape keeps `#/session/1`, because the workspace PageHeader passes `escapeGoesBack` false and `useWorkspaceKeyboard` keeps Escape. ⌘[ with the picker dialog open keeps the route (layer check). After Escape closes the dialog, ⌘[ leads to `#/`.
- **Layer detection (read):** `useBackKeys` returns when the document contains `[role=dialog|alertdialog|menu]`. That covers Radix Popover content (`role=dialog`: AiPopover, IdentityEditor, SelectInput, MultiSelect), the FormDialog, the PhotoMenu DropdownMenu and ContextMenu (`role=menu`), the NotificationsDrawer (`aside role=dialog`) and the PublishPopover review (`section role=dialog`).
- **C3(a)/(b)/(c), production (read):** `run` applies `optimistic` through `setView` before its first `await`, so the new order commits in the same React batch as dnd-kit's `onDragEnd` / `active` clear. The `reorder` transform takes the group's `displayOrder`, splices the photo to `clamp(position − 1)` and renumbers only that group 1..n. Unreviewed and deleted photos are left alone. The drag `classify` (`advance = false`) sets the target classification with `position = target length + 1` and renumbers the source group (ch06 §Order and 05A: "Drag between groups uses classify into g (end of target), whatever the drop index"). On success and on failure, `reload()` replaces the optimistic list. A failure toasts `refusalText` once.
- **C3 tests:** the release is followed synchronously by `panelOrder` before the command promise settles. C3(a) also checks `reorderPhoto(1, 3)` exactly once and the order after resolve + reload. The base tree shows `[1,2,5]` at the first assertion, which proves the second fixture view was not consumed before the drag. C3(b) restores `[1,2,5]` after the rejection, with one `toast.error`. C3(c) gives Main `[1,2,4]`, an empty Condition, `classifyPhoto(4,'main')` and no `reorderPhoto`.
- **Unchanged paths:** review-key classify (`advance = true`), delete, undo, move and copy pass no transform. The photo menu's "Move to Main/Condition" (`toggle`) shares the `advance = false` path and is now immediate as well, which agrees with ch06 (end of target). The folded strip shares the `DndContext` and `useDrop`.
- **Hard constraints H1–H8:** the phase is frontend-only, with no Rust, binding, command or external-write change. Components still call `services/` (H8). No secret appears in the diff, and no test needs a live service.
- **Assumptions (all within the plan's Delegated choices; none contradicts the intention):** the 900 px column stays under a full-width header. Ctrl+[ joins ⌘[. Escape and ⌘[ leave Settings without saving. Layers are detected by the dialog, alertdialog and menu roles. Review-key classify is not optimistic. The P08 fixtures were copied, not extracted. The default dnd-kit drop animation was kept. All are reported to the owner in the narrative. **Deviation** (the root `h1` is the direct first child of the header, not wrapped in `.header-identity`) is what C1(d) requires. Accepted.
- **Over-engineering test:** the tests query roles, accessible names, hashes and DOM order. There are no call counts beyond the row-mandated "called once with" and no private helpers.

## Findings
None blocking (B1–B4: none).

## Notes written to backlog
1. `parentRoute` is exported but unused, and pages hard-code `back.to`. `Header`'s `goHome` prop and the `.health-subtitle` rule are also dead. This leaves two sources of truth for a page's parent.
2. A workspace event reload that arrives during an in-flight drag command can briefly restore the stored (old) order, a rare echo of R8.
3. ⌘[ ignores typing focus. On Settings it can drop unsaved edits without a prompt.
4. R8 has not been verified live: jsdom proves the DOM order at release, not the animation. The owner's try-out step 5 is the proof, and plan task 4's `dropAnimation={null}` fallback stays available.

**Carry-forward:** all 4 notes → `docs/implementation/backlog.md` (P19 rows). Note 4 also → the owner's narrative try-out step and the orchestrator's next real run.

## Lessons for the plans
- Task 1 asked for `parentRoute` but task 2 hands each page a literal `back.to`, and no criterion row observes `parentRoute`. So the function shipped unused. When a plan names a single-source helper, a row (or the task wording) should route its callers through it.
- For visual defects such as R8, a jsdom proxy row is the right automated boundary, but the plan should name who performs the live check when the implementer cannot run the GUI.

## Probe declaration
No production file was edited. A temporary `git worktree` at `/tmp/p19_base` (base `731b214`, with the two P19 test files copied in and `node_modules` symlinked) was created, run and removed with `git worktree remove --force`. `git worktree list` shows only the main tree, and the main tree had no unexpected changes. No database or app state was touched.

## Write perimeter
- `docs/implementation/handoffs/reviewer/handoff_P19_navigation_and_drop_review_r1.md` (this file)
- `docs/implementation/narratives/P19_navigation_and_drop.md`
- `docs/implementation/plans/P19_navigation_and_drop.md` (Review log entry appended)
- `docs/implementation/master_plan.md` (P19 tracker row only → APPROVED)
- `docs/implementation/backlog.md` (4 P19 rows appended)
