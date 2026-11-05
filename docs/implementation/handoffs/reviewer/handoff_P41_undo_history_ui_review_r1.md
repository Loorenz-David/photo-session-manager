---
plan: P41_undo_history_ui
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-05
actor: opus
---
# Review P41_undo_history_ui, round 1: APPROVED_WITH_NOTES

The implementation meets all 16 criterion rows at their boundaries. Every row's test fails on a wrong outcome. Suite, clippy, fmt and
typecheck are green, H1–H8 hold, and no assumption contradicts the book. There are four non-blocking notes. One of them, ⌘Z inside a text
field with the new custom Edit-menu items, cannot be observed in jsdom, so it goes to the owner's try-out (narrative step 5).

## ⚠ OWNER DECISIONS REQUIRED (0)
None. The P40 carry-over (undoing a re-frame of a `needs_review` photo restores `ok`) was already told to the owner in P40's narrative,
with the words to change it. P41's narrative "Not yet" repeats it. No card.

## Gate and evidence
- Tracker row read `IMPLEMENTED`; checkpoint `84fb44302d61a714a1dae41838ac6715376c5c17` is present in `git log`.
- Perimeter: `git show --stat 84fb443` lists 28 paths, identical to the handoff's "Files changed". The plan's estimate missed
  `features/shell/__tests__/p19_navigation.test.tsx`, which drops one `undoDelete: vi.fn()` mock line. The handoff disclosed it and the
  change is harmless, so it is not a finding.
- Commands. Tree: clean at `7e72698`; the code is identical to `84fb443`, and later commits are runner docs only.
  - `cargo test --workspace`: all `test result: ok`.
  - `cargo clippy --workspace --all-targets -- -D warnings`: green.
  - `cargo fmt --all -- --check`: rc 0.
  - `npx tsc --noEmit -p tsconfig.json`: rc 0.
  - `vitest run`: 369/369 in 9 of 10 complete runs. The 10th failed the known `p33_split_grid.test.tsx` C4(d) flake (the 1 s `waitFor`
    timed out at 1055 ms; tracked in the backlog since P35).
  - An 11th run was discarded: every test in it took about 897 s, so the machine was suspended.
  - No B4.
- Over-evidence disclosure: the reviewer started a duplicate `cargo test --workspace` on the same tree because a count regex was wrong.
  It was stopped before it finished. Several UI suite repeats were deliberate, to capture the intermittent failure's ID (flaky-capture rule).

## Verified correct
- **C1(a)–(c) keys.**
  - `useHistoryKeys` is mounted in `App` while a session is open: a window `keydown` hook guarded only by `defaultPrevented`, `isTyping`
    and `altKey`, requiring meta/ctrl+`z`; shift means redo.
  - `useWorkspaceKeyboard` lost its ⌘Z branch and `WorkspaceKeys.undo`. `useComposeKeys` is untouched.
  - C1(c) bites: a plain `z` that reached the history would make the call count 2.
  - C1(b) discriminates. Changed-input probe: a scratch test, `zz_reviewer_probe.test.tsx`, deleted afterwards, sent ⌘Z from the Product
    tab and ⌘⇧Z from the label inside the open details dialog. Both reached `historyUndo`/`historyRedo`. So nothing else (Radix, the
    dialog) swallows the key, and only the field guard keeps the row green.
- **C2(a)–(c) notices.**
  - `act()` toasts `Undone: <label> · Redo ⌘⇧Z` / `Redone: <label> · Undo ⌘Z` under id `history`.
  - `step_cannot_apply` shows its message verbatim; `nothing_to_undo` / `nothing_to_redo` show the fixed texts.
  - C2(a) also asserts the first notice is replaced.
- **C3(a)–(c) header.**
  - `HistoryButtons` renders `Undo2`/`Redo2` icon-only buttons with class `history-button`, placed after the divider and before the bell,
    in the session header only.
  - They use `aria-disabled` (never `disabled`), and the click is a no-op when the label is null.
  - Card texts match task 0.
  - The store ignores `history_changed` for another `session_id`, and stale `historyGet` answers after an event (the `changed` flag).
- **C4(a)/(b) delete toast.**
  - The toast's `Undo` action and the late ⌘Z both go through `history.undo(sessionId)`.
  - `toast.dismiss('photo-deleted')` runs on success, and `registerActivePhoto` selects `active_photo_id` (test: `activeTile() === 1`).
  - `undoDelete`, `photos_undo`, its `collect_commands!` entry, its binding and the P39 compatibility test are all gone. C1(a) checks the
    real module's exports.
- **C5(a) editor sync.**
  - The effect on `active.composition` dispatches `Saved` when the stored `revision` differs from the last one seen for that photo and no
    save is pending (`usePendingCompositions`).
  - The editor's own saves produce a same-values `Saved`, which rule 16 makes a no-op.
  - The test bites: without the sync, Esc finds live = entry and saves nothing, so `toHaveBeenCalledTimes(1)` fails.
- **C6(a)/(b) grid Split.**
  - Two or more changed photos go through one `applyComposition(items)` with `refit` values, and the patch sets `revision + 1` and origin
    `user`.
  - A single photo keeps `setComposition`.
  - The test asserts the re-fit rect to 6 decimals, the absence of any set call, and `.composed-split` on both tiles.
- **C7(a)/(b) Edit menu.**
  - `publish()` calls `editMenuSet(undo, redo)` on every real label change.
  - `onEditMenu` routes to the same `undo`/`redo` path.
  - Rust: `Menu::default` with its Edit submenu replaced by `edit.undo` / `edit.redo` (disabled by default, accelerators
    `CmdOrCtrl+Z` / `CmdOrCtrl+Shift+Z`), then separator, cut, copy, paste, select all.
  - The handles are kept in managed `EditMenu<Wry>`. `on_menu_event` emits `edit_menu {action}`. `edit_menu_set` is a `pub async fn`
    that sets each item's text and enabled state.
  - `history_changed` listeners live in `services/history.ts`, as the plan's Notes require, so older test mocks are untouched.
- **H1–H8.** No change under `crates/`. The one new command, `edit_menu_set`, only touches menu handles and is thin shell (H8). Nothing
  writes externally (H1) and no RAW is touched (H2). No secrets, and no live services in the suite.
- **Assumptions.** The handoff lists `useSyncExternalStore`, compose-harness session id 5, active-photo selection plus a queued reload,
  `revision` as the sync key, and no live Capture One probe. All are within MC-30 and the plan's Delegated choices, so none is B1. They
  are written into the narrative's "Choices made for you" where the owner can see them.

## Findings
None blocking (B1–B4).

## Notes written to backlog
1. **P41 (by reading; not observed live).** The custom `edit.undo` / `edit.redo` items with ⌘Z accelerators replace the predefined
   `undo:` / `redo:` selectors.
   - If WebKit does not consume ⌘Z in a focused field, the accelerator fires `edit_menu`. The history then undoes a step while the user
     is typing, and the field's own undo is lost. That would contradict MC-30 §Not steps.
   - The owner's try-out (narrative step 5) decides.
   - Suggested fix: in `onEditMenu`, when `isTyping(document.activeElement)`, run `document.execCommand(action)` instead of the history.
2. **P41 (by reading).** A multi-photo grid Split/Normal ignores `result.refused`, so a refused photo silently stays as it was. Before
   P41, each failed save showed "Not saved".
3. **P41 (by reading).** One undo can trigger up to three workspace reloads: the active-photo callback, `photo_changed` and
   `history_changed`. They arrive as separate tasks, and the microtask coalescing only merges calls within one task. Harmless; the plan
   says "never two reloads".
4. **P41 → P33.** The C4(d) suite flake recurred: 1 of 10 complete runs.

## Lessons for the plans
- A plan that replaces a platform-predefined menu item, or any native behaviour the webview relies on, should name the native behaviour
  it removes, and give a try-out step that checks it. Here that is field undo via the `undo:` selector. P41's Notes covered the
  double-undo risk but not the field case.
- Count mock-factory edits from a grep, not by hand. The plan listed six `undoDelete` mock lines; there were seven.

## Write perimeter (this session)
- `docs/implementation/handoffs/reviewer/handoff_P41_undo_history_ui_review_r1.md` (new).
- `docs/implementation/narratives/P41_undo_history_ui.md` (new).
- `docs/implementation/master_plan.md`: the P41 tracker row only, `IMPLEMENTED` → `APPROVED`.
- `docs/implementation/plans/P41_undo_history_ui.md`: Review log entry appended.
- `docs/implementation/backlog.md`: 4 rows appended.
- Probe: `apps/desktop/src/features/history/__tests__/zz_reviewer_probe.test.tsx` was created, run and deleted. `git status --porcelain`
  was empty afterwards.
- No production file was edited, no database or app state was touched, and nothing was committed.
