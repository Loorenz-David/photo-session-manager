---
plan: P08_workspace_ui
role: review
round: 1
verdict: CHANGES_REQUESTED
state: DONE
date: 2026-10-25
actor: opus
---
# P08 workspace UI: review r1

**Summary.** The phase is almost entirely right. All 22 criterion rows hold at their boundaries, and their tests
can fail. Suite, lint and typecheck are green, and no hard constraint is broken. There is **one blocking finding
(F1, B1)**: a capture that arrives in the open session never becomes the photo on the stage. The first frame of a
fresh session therefore leaves the stage empty. A later frame leaves the review keys acting on the previous photo.
The fix is small and local to `useWorkspaceState.ts`.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Gate
- Tracker showed `P08_workspace_ui | IMPLEMENTED`. The checkpoint `2b79552` exists. HEAD `2be7dd3` differs from it only by the
  two runner commits (implement handoff, review prompt). `git status --porcelain` was clean at entry and at exit.

## Verified correct
**Perimeter.** `git show --stat 2b79552` lists 41 paths, and they equal the handoff's "Files changed" list exactly
(Cargo.lock; package.json/lock; src-tauri Cargo.toml, capabilities, tauri.conf.json, lib.rs, commands/sessions.rs,
commands/photos.rs; bindings.ts, services/photos.ts, services/events.ts; App.tsx; Header.tsx, ArrivalToasts.tsx;
the 20 files of `features/workspace/`; the P07 test; core `domain/session.rs`, `services/sessions.rs`; testkit
`p08_workspace_ui.rs`; master plan, plan file). Unlisted files are declared under Deviations (5).

**Stamp (L4, tree `2b79552` + docs-only commits, clean).** All five commands exited 0:
| Command | Result |
|---|---|
| `cargo test --workspace` | 183 passed, 0 failed (includes `export_bindings`) |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo fmt --all -- --check` | exit 0 |
| `npm run typecheck` (Node 22) | exit 0 |
| `npm test` | 2 files, 42 passed |

**Rows: production read, test judged by reading.**
| Row | Production that makes it true | Test observes the outcome and can fail |
|---|---|---|
| C1(a–c) | `useKeyboard` a/c/backspace → `WorkspacePage.review` → `classify`/`remove` | exact args, called once |
| C1(d) | `nextAfterAction`: first unreviewed in `displayOrder` excluding the acted photo | u1 (200, id 9) and u2 (100, id 12): the active photo becomes 12, then 9. Ordering by id would give 9 first, so the test fails on that |
| C2(a) | `remove` → Sonner toast, `duration: 4500` | `data-removed` false at 4 499 ms, true at 4 500 ms |
| C2(b) | ⌘Z → `undo` → `undoDelete(session)`; `active_photo_id` set as state and ref | at 6 000 ms the toast is gone, then exact call; the highlighted tile is p |
| C2(c) | toast action `Undo` → `undo` | exact call |
| C3(a) | `StageImage` shows the original until an adjusted layer has loaded; `TogglePill` shows a spinner when the path is null | original `data-visible=true`, no adjusted layer, spinner `role=status` |
| C3(b) | event → `reload` → new `adjusted_preview_path` → new layer through `previewUrl` (`convertFileSrc`) with a 700 ms opacity transition | new `src`, `crossfade` class, 700 ms |
| C3(c) | `mode` lives in the hook (per session view); `StageImage` is keyed per photo | after Space and →, photo 2 shows its original. A per-photo mode would fail |
| C3(d) | `orientationTransform(6)` on the original only | `rotate(90deg)` vs `''` |
| C4(a)/(d) | `displayOrder` (MC-5, R-I2) → Main + unreviewed, then Condition; strip with a COND separator | the input is deliberately unsorted [b,u,d,a]; output [1,2,3,4] and [1,2,3,'COND',4] |
| C4(b) | `useDrop`: within a group, position = over-tile index + 1 | real pointer drag; `reorderPhoto(1,3)`, no classify |
| C4(c) | a different group → `classify(…, false)` (end of target, MC-5) | `classifyPhoto(4,'main')`, no reorder |
| C5(a) | `sessionsOpen` defaults to true in Unassigned; Header shows "No product" and "Assign product" | presence, and Publish absent |
| C5(b) | session drop → the selection in display order → `move` | ⌘-click p and q, drag p: `[1,2]` to 7 |
| C5(c) | `useDroppable({disabled: status==='published'})` | `data-droppable=false`, and a real drop makes no call. A droppable row would produce a call, so the test fails |
| C5(d) | `assignWhole` → `assignUnassigned(item.id)` | exact |
| C6(a) | `ImportDropZone` → `importFiles(paths)` | exact |
| C7(a) | `ArrivalToasts`: `session_id 1` while not on `/session/1` → toast with Open → navigate | text, then the hash |
| C8(a) | ⋯ → `copy` → `SessionPickerDialog` → `state.copy` | `copyPhotos([1],7)` |
| Core `displayOrder` | as above | unit test, input → output, ties broken by id |
| Candidate C0 | `SessionsService::view` (core, Tauri-free): item + photos except deleted | Rust behaviour test; declared in the Review log, not orphan |

Over-engineering test: no test reaches into internals. They observe DOM, attributes and service-mock arguments only.

**Hard constraints H1–H8 against the diff.**
- H1: no external writes.
- H2: Delete calls `photos_delete` (hide).
- H3: the CSS transform is display only.
- H5: no UI automation.
- H6: no commerce dependency.
- H7: `PhotoResponse` carries no provider ref or variant id.
- H8: `view` is in core and `session_get` stays thin.
- Asset protocol: the static scope is empty and only `<data dir>/previews/` is allowed at runtime.
- No secrets and no live services in tests.

**Assumptions** (all consistent with the book except where F1 shows the code does not do what the assumption says):
`convertFileSrc` · new capture becomes active (**intended but not implemented, F1**) · A on Main / C on Condition only
advance · drags do not advance · menus act on the selection · "Assign to product…" = `session_create` + `photos_move` ·
"Assign product" searches existing items only (no Create row) · item card in-stage · sessions panel can be opened anywhere ·
dropped paths unfiltered · unreviewed tiles numbered after Main · no `S` capture button.

## Findings

### F1: a capture arriving in the open session does not become the stage photo. Blocking B1, `route: production`
- **What is wrong.** The `onWorkspaceEvents` handler in `useWorkspaceState.ts` (≈ line 104) sets `activeRef.current =
  event.photo_id` and calls `reload()`. `reload` calls `setActiveId` only when the ref's photo is *not* visible. After the
  reload it is visible, so the rendered `activeId` state never changes. Reviewer probe (a throwaway vitest file built
  from the P08 test's helpers, run once and deleted; no production file touched):
  - An empty session, then `photo_added{photo 1}`: the panel shows the tile, the stage still reads "Ready for the first
    photo", there is no active tile, and no original image appears. A/C/⌫ do nothing because `active` is null.
  - Photo 1 (Main) is active, then `photo_added{photo 2}`: the stage stays on photo 1. C or ⌫ would now classify or
    delete **photo 1**, while ←/→ step from photo 2, because `step` reads the ref.
- **Authority.** ch04 S-C ("Within about a second the photo appears on the stage … the photographer presses A, C or
  Delete"); ch06 §Arrival; the plan's Manual try-out step 1. The implementer's own assumption ("a capture arriving in
  the open session becomes the active photo") states the intended behaviour.
- **Correction.** In the `photo_added` branch, set the active photo through `setActiveId(event.photo_id)` (state and ref
  together), so the stage, the review actions and navigation agree. Add one behaviour test in the P08 test file:
  `photo_added` for the open session puts that photo on the stage, and a following key acts on it. Declare that test in
  the Review log as a candidate criterion (it traces to ch04 S-C), so it is not an orphan.
- **Files the fix may touch.** `apps/desktop/src/features/workspace/useWorkspaceState.ts`,
  `apps/desktop/src/features/workspace/__tests__/p08_workspace_ui.test.tsx`, plus the plan Review log, the tracker row
  and the fix handoff.

## Notes written to backlog
1. P08: the import toast counts every handed-over path, not what `photos_import` accepted.
2. P08: dragging a Main photo onto an unreviewed tile reorders it to the end of Main.
3. P08: unreviewed tiles are numbered after Main and look like positions.
4. P07 → P11: the header still reads its identity from `sessions_list(all)` although `session_get` now returns the
   item (carry-forward of the P07 note).

## Lessons for the plans (coordinator)
- Master plan §6.7 should record the new `session_get` payload `{session, item, photos}` (deviation 1). The coordinator
  should decide whether candidate C0 becomes a row.
- The plan had no row for "a new capture in the open session appears on the stage", although it is the first line of
  S-C and of the Manual try-out. Live-event → stage rows belong in UI plans that subscribe to arrival events.
- The new dependencies (`tauri-plugin-dialog` 2.8.3, `@dnd-kit/utilities` 3.2.2) and the `protocol-asset` feature
  belong in master plan §6.2.

## Write perimeter (this session)
- `docs/implementation/handoffs/reviewer/handoff_P08_workspace_ui_review_r1.md` (this file, new)
- `docs/implementation/master_plan.md` (P08 tracker row only)
- `docs/implementation/plans/P08_workspace_ui.md` (Review log entry appended)
- `docs/implementation/backlog.md` (4 rows appended)
- Probe files: `apps/desktop/src/features/workspace/__tests__/zz_reviewer_probe.test.tsx` and `/tmp/p08_probe.txt`.
  Both were created, run and deleted. `git status` was clean afterwards, and no production file was touched.
  Logs: `/tmp/p08_rev_cargo.log`, `/tmp/p08_rev_clippy.log` (outside the repo).
- No commits (the runner commits).
