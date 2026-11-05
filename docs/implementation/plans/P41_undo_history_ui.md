---
phase: P41_undo_history_ui
title: Undo and redo in the app — ⌘Z and ⌘⇧Z everywhere (workspace, editor, dialogs; never inside a text field), the notice naming the step, the header's Undo and Redo icons with hover cards, the delete toast's Undo on the history, the editor following an undone framing, a grid Split on a selection as one step, and the Edit menu's "Undo <step>" / "Redo <step>"
difficulty: high
projection: no
depends_on: [P40_undo_history_compositions_and_attributes, P38_editor_polish]
---
# P41 — Undo and redo in the app: keys, notices, header icons, the editor, the Edit menu

## Goal
The frontend of R41 (05A MC-30 §Rules; 10D "Undo and redo work in the editor as everywhere in the app"). ⌘Z undoes and ⌘⇧Z redoes the open
session's newest step **wherever the photographer is** — the workspace, the editor, with the details form or a picker open — except while a
text field has the keyboard (its own undo). Every undo and redo shows a short notice naming the step ("Undone: Approve 03 · Redo ⌘⇧Z"); a
step that could not apply shows the backend's reason ("Can't undo Move to Lamp 2: that session is published"). The header gains an **Undo** and
a **Redo** icon whose hover cards name the step they would act on, disabled when there is none; the delete toast's Undo button calls the
history. While composing, an undone or redone framing of the stage photo snaps the editor to the stored composition. A grid Split/Normal on a
selection goes through `composition_apply` so it is one step. The app's **Edit menu** carries "Undo <step>" and "Redo <step>" with ⌘Z / ⌘⇧Z,
updated from the history, and its entries act through the same path as the keys. `photos_undo` and `undoDelete` are retired.
**Not in this phase:** anything the backend records or reverses (P39, P40); the editor's own `Revert` (Esc) — it stays a composition save and
therefore a step, nothing new here; colour and light tools.

## Read first
Book: `05A_mechanism_contracts.md` MC-30 §Rules (keys, the 25 limit as a given, the notice text, the header icons "disabled when there is none",
the Edit menu, "The delete toast … its Undo is the history's ⌘Z") and §Not steps ("Typing inside a text field is the field's own undo while the
field has the keyboard; ⌘Z there never reaches the history") · `10D_composition_and_framing.md` bullet "Undo and redo (⌘Z, ⌘⇧Z) work in the editor
as everywhere in the app" · `04_workflow_and_scenarios.md` S-C · `06_photo_review_and_ordering.md` §Classification · plans `P39_undo_history_core
.md` (§The step model: `HistoryState`, `HistoryOutcome`, the labels; the commands and the `history_changed` event), `P40_…` (Notes (6)), `P08_
workspace_ui.md` (C2 undo rows), `P35_composed_display_and_chrome.md` (`Hint`, `aria-disabled`), `P31_compose_mode.md` (machine rule 16 `Saved`;
the frame-box arithmetic its tests use), `P37_editor_chrome.md` (`HintCard`) · master plan §6.7, §6.10 (P39–P41 block), §7.2.
Code at `3e4b760` (+ P39/P40):
- `apps/desktop/src/features/workspace/useWorkspaceState.ts:11, :22–24` (`undoDelete`, `UNDO_TOAST_MS`, `UNDO_TOAST_ID`), `:174–183` (`undo`),
  `:185–199` (`remove`: the toast with `action: { label: 'Undo', onClick: () => void undo() }`), `:271` (`undo` returned).
- `features/workspace/useKeyboard.ts:8, :27–31` (the ⌘Z branch and the `undo` key — **removed**), `features/workspace/WorkspacePage.tsx:316`
  (`undo: () => void state.undo()`), `:227–248` (`setKind` — one `setComposition` per photo), `:302–327` (the key hooks and their `enabled` flags:
  `!dialogOpen && !composing`).
- `features/compose/useComposeKeys.ts:41` (`defaultPrevented || LAYER_OPEN || isTyping` guard), `:48–66` (the ⌘ branch returns without swallowing
  for ⌘Z — so a window-level ⌘Z handler runs in the editor), `:67` (`z` alone = loupe).
- `features/compose/ComposeMode.tsx:105–111` (`useCompositionSaves` with `onStored`/`onSettled` → `dispatch({ type: 'Saved' })`), `machine.ts:50–51,
  :378–383` (rule 16: `Saved` of this photo while idle with different values → `values = live = stored`, `View`), `useCompositionSaves.ts:21–30`
  (`usePendingCompositions`: the photos with a save in flight).
- `features/shell/Header.tsx:140–188` (the session header: `CameraDot`, divider, bell, Settings, Details, Export/Publish), `components/ui/
  tooltip.tsx` (`Hint`, `HintCard`), `lib/keyboard.ts:4–7` (`isTyping`).
- `services/photos.ts:36–39` (`undoDelete` — **removed**), `services/events.ts` (the `listen` wrappers), `services/bindings.ts` (regenerated in P39:
  `historyGet`, `historyUndo`, `historyRedo`).
- `apps/desktop/src-tauri/src/lib.rs:172–205` (`run()`: `tauri::Builder::default()…` — no `.menu`, so Tauri's default menu with a predefined
  Edit ▸ Undo/Redo is shown today), `:35–106` (`TauriEvents`), `:208–232` (`commands_are_async`); `commands/history.rs` (P39).
- Installed Tauri `2.12.5` menu API (`~/.cargo/registry/src/*/tauri-2.12.5/src/menu/`): `Menu::default(&handle)` `menu.rs:142` (the default
  layout: app submenu, File, Edit, View, Window, Help), `MenuItem::with_id(manager, id, text, enabled, accelerator: Option<&str>)` `normal.rs:48`,
  `MenuItem::set_text` `:94`, `set_enabled` `:105`, `Submenu::with_id_and_items`, `PredefinedMenuItem::{cut, copy, paste, select_all, separator}`
  `predefined.rs:15–73`, `Builder::menu(|handle| …)` `app.rs:2089`, `Builder::on_menu_event(|app, event| …)` `app.rs:2112`.
- Tests that **change their expectation**: `features/workspace/__tests__/p08_workspace_ui.test.tsx` C2(b) `:335–359` and C2(c) `:361–368` (mock
  `historyUndo` from `@/services/history` instead of `undoDelete`; the resolved value becomes `{ label: 'Delete 01', direction: 'undone',
  active_photo_id: 1, sessions: [] }`; the assertions on the active tile and the one call stay); the `vi.mock('@/services/photos')` factories that
  list `undoDelete: vi.fn()` (`p08 :20`, `p19_drop :21`, `p20_drop_continuity :34`, `p22_workspace_tidy :21`, `p26_double_drag :21`, `compose/
  __tests__/harness.tsx :26`) drop that line (harmless if kept; drop it for tidiness). Every other test keeps passing unchanged — see Notes
  (service module placement).

## Files expected to change
Frontend (new): `apps/desktop/src/services/history.ts`, `apps/desktop/src/features/history/{useHistory.ts, HistoryButtons.tsx, useHistoryKeys.ts}`.
Frontend (changed): `apps/desktop/src/{App.tsx}`, `features/shell/Header.tsx`, `features/workspace/{useWorkspaceState.ts, useKeyboard.ts,
WorkspacePage.tsx}`, `features/compose/ComposeMode.tsx`, `services/photos.ts`, `services/bindings.ts` (regenerated), `styles` or
`features/workspace/workspace.css` (the two header icons).
Desktop Rust (new): none. Desktop Rust (changed): `apps/desktop/src-tauri/src/lib.rs` (the menu, `on_menu_event`, `edit_menu` event),
`apps/desktop/src-tauri/src/commands/history.rs` (`edit_menu_set`), `apps/desktop/src-tauri/src/commands/photos.rs` (remove `photos_undo`),
`apps/desktop/src-tauri/src/state.rs` (the `EditMenu` handles).
Tests (new): `apps/desktop/src/features/history/__tests__/p41_undo_ui.test.tsx`, `apps/desktop/src/features/compose/__tests__/p41_editor_sync.test.tsx`.
Tests (changed): `p08_workspace_ui.test.tsx` and the six mock factories listed above.
Estimated: 4 new paths, 13 changed, 2 new tests, 7 adjusted tests.

## Core mechanisms (unit tests allowed)
None. Everything is asserted at the service mocks (`@/services/*`) and in the DOM.

## Ordered tasks
0. **Registry first (master plan §6.10).** `services/history.ts`: `historyGet(sessionId) -> Promise<HistoryState>` (`{ undo: string | null; redo:
   string | null }`), `historyUndo(sessionId) -> Promise<HistoryOutcome>` (`{ label, direction: 'undone' | 'redone', sessions, active_photo_id }`),
   `historyRedo(sessionId)`, `editMenuSet(undo: string | null, redo: string | null) -> Promise<void>`, `onHistoryChanged(handler)` (Tauri event
   `history_changed { session_id, undo, redo }`), `onEditMenu(handler)` (Tauri event `edit_menu { action: 'undo' | 'redo' }`) — **all history
   listeners live in this module, not in `events.ts`** (Notes). Texts: `Undone: <label> · Redo ⌘⇧Z`, `Redone: <label> · Undo ⌘Z`, `Nothing to undo.`,
   `Nothing to redo.`; a `step_cannot_apply` error shows its `message` verbatim. Header buttons: accessible names `Undo`, `Redo`, class
   `history-button` (icon-only, `Undo2` / `Redo2`), `aria-disabled="true"` when the label is null (never `disabled`: P35 rule, the card must still
   show); cards `Undo · ⌘Z — <label>` / `Undo · ⌘Z — Nothing to undo`, `Redo · ⌘⇧Z — <label>` / `Redo · ⌘⇧Z — Nothing to redo`. Toast ids: the
   delete toast keeps `photo-deleted`; the undo/redo notice uses id `history`. Tauri: command `edit_menu_set(undo: Option<String>, redo:
   Option<String>)`; menu item ids `edit.undo`, `edit.redo`; texts `Undo <label>` / `Undo` (disabled), `Redo <label>` / `Redo` (disabled);
   accelerators `CmdOrCtrl+Z`, `CmdOrCtrl+Shift+Z`; event `edit_menu`. Retired: `photos_undo`, `undoDelete`, `WorkspaceKeys.undo`.
1. **`useHistory`** (`features/history/useHistory.ts`): one module-level store (the `usePendingCompositions` pattern) keyed by the open session:
   `{ sessionId, undo, redo }` filled by `historyGet` when the session opens and replaced by every `history_changed` whose `session_id` matches
   (others ignored); `useHistoryState()` for readers; `undo(sessionId)` / `redo(sessionId)`: call the command; on success `toast(text, { id:
   'history' })`, `toast.dismiss('photo-deleted')`, and when `active_photo_id !== null` call the registered `onActivePhoto(id)` (set by the
   workspace); on a `ServiceError` show its `message` for `step_cannot_apply`, `Nothing to undo.` / `Nothing to redo.` for the two codes, else the
   message. `historyGet` failures are swallowed (an unmocked test environment has no Tauri). Whenever `undo`/`redo` labels change, call
   `editMenuSet(undo, redo)` (errors swallowed). `onEditMenu` → `undo(sessionId)` / `redo(sessionId)`.
2. **Keys** (`features/history/useHistoryKeys.ts`, mounted in `App` while a session is open): window `keydown`; `key.toLowerCase() === 'z'` with
   `metaKey || ctrlKey`, not `altKey`: `shiftKey` → redo, else undo; `preventDefault`; **returns without acting when `isTyping(event.target)`** (the
   field keeps its own ⌘Z) or when `event.defaultPrevented`. No `enabled` flag for dialogs or the editor: it runs in the workspace, the editor,
   over the details form, the pickers and the item card. `useWorkspaceKeyboard` loses its ⌘Z branch and the `undo` key; `useComposeKeys` is
   untouched (its ⌘ branch already lets ⌘Z through).
3. **Header icons** (`features/history/HistoryButtons.tsx`, `Header.tsx`): two `Hint`-wrapped icon buttons after the divider (before the bell) in
   the session header only (not on the Sessions page): `Undo` then `Redo`; click → `undo(sessionId)` / `redo(sessionId)`; `aria-disabled` and
   the card text per task 0.
4. **Workspace** (`useWorkspaceState.ts`, `WorkspacePage.tsx`): `undo` becomes `() => history.undo(sessionId)`; the delete toast's action keeps
   its `Undo` label and calls it; `onActivePhoto` registers `setActiveId` (and un-registers on unmount); after a `history_changed` for this
   session the view reloads (the backend emitted `PhotoChanged`/`SessionChanged` too; one reload). `setKind` (grid Split/Normal): when more than
   one photo would change, build `items = [{ photo_id, values: refit(...) }]` and call `applyComposition(items)` once, patching each applied photo
   as the ⋯ Apply branch does (`revision + 1`, `origin 'user'`, `state 'ok'`); one photo keeps `setComposition` (P33 rows).
5. **Editor follows the history** (`ComposeMode.tsx`): an effect on `active.composition` — when `active.id` is the machine's photo, the photo has
   no pending save (`usePendingCompositions()` does not hold it) and the stored `revision` differs from the last one the editor saw for this photo,
   `dispatch({ type: 'Saved', photoId: active.id, composition: active.composition })` (machine rule 16 snaps the view when the values differ).
6. **Edit menu** (`lib.rs`, `commands/history.rs`, `state.rs`): `Builder::menu(|handle| …)` builds `Menu::default(handle)`'s items with the Edit
   submenu replaced by `Submenu::with_id_and_items(handle, "edit", "Edit", true, &[&undo, &redo, &separator, &cut, &copy, &paste, &select_all])`
   where `undo = MenuItem::with_id(handle, "edit.undo", "Undo", false, Some("CmdOrCtrl+Z"))`, `redo = MenuItem::with_id(handle, "edit.redo",
   "Redo", false, Some("CmdOrCtrl+Shift+Z"))`; the two `MenuItem` handles are kept in managed state (`EditMenu { undo, redo }`); `on_menu_event`
   emits `edit_menu { action: "undo" | "redo" }` for those ids; `edit_menu_set` sets the texts (`Undo <label>` / `Undo`) and `set_enabled(label
   .is_some())`. Remove `photos_undo` from `collect_commands!` and `commands/photos.rs`; regenerate `bindings.ts`.
7. **Services** (`services/history.ts`, `services/photos.ts`): task 0; `undoDelete` and its comment go; `deletePhoto`'s comment reads "the delete
   is one undo step (MC-30)".
8. Tests: `p41_undo_ui.test.tsx` (C1–C4, C6, C7), `p41_editor_sync.test.tsx` (C5), the P08 edits, `npm run typecheck`, `npm test`; Rust:
   `cargo test --workspace` (bindings test, `commands_are_async`), clippy, fmt.

## Criteria (7 criteria, 16 rows; 12 `[rule]` rows — UI wiring rows whose contract is MC-30 §Rules; the other four trace MC-5, MC-30 and M36 directly)
Fixture: P08's harness (`openWorkspace([view…])`, `press`, `activeTile`, the `@/services/*` mocks) plus `vi.mock('@/services/history')` with
`historyGet`, `historyUndo`, `historyRedo`, `editMenuSet`, `onHistoryChanged` (captures the handler into `delivered.history`), `onEditMenu`
(`delivered.editMenu`). `SESSION` = 7 as in P08. Hover cards are read as in P37. "Icon-only" as in P37.

| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | keys in the workspace | `historyGet → { undo: 'Approve 03', redo: null }`; `historyUndo → { label: 'Approve 03', direction: 'undone', active_photo_id: null, sessions: [] }`; `press('z', { metaKey: true })`; then `press('z', { metaKey: true, shiftKey: true })` | `historyUndo` called once with `SESSION`; `historyRedo` called once with `SESSION`; `undoDelete` does not exist on `@/services/photos` (module export absent) | `[rule]` MC-30 ("⌘Z undoes the open session's newest step; ⌘⇧Z redoes the newest undone step") |
| C1(b) | keys in a text field | open the details form (press `d`), focus the `Title` input (`screen.getByLabelText`), `fireEvent.keyDown(input, { key: 'z', metaKey: true })`; and with `shiftKey` | neither `historyUndo` nor `historyRedo` is called; the form stays open | `[rule]` MC-30 §Not steps ("⌘Z there never reaches the history") |
| C1(c) | keys while composing | enter the editor (`r`), `press('z', { metaKey: true })`; then `press('z')` alone | `historyUndo` called once with `SESSION`; the plain `z` does not call it (the loupe's key; P34 C1 keeps passing) | `[rule]` 10D ("Undo and redo (⌘Z, ⌘⇧Z) work in the editor as everywhere in the app") |
| C2(a) | notices | `historyUndo → { label: 'Approve 03', direction: 'undone', active_photo_id: null, sessions: [] }`; ⌘Z; then `historyRedo → { label: 'Approve 03', direction: 'redone', … }`; ⌘⇧Z | a toast with the text exactly `Undone: Approve 03 · Redo ⌘⇧Z`; then one reading `Redone: Approve 03 · Undo ⌘Z` | `[rule]` MC-30 ("Every undo and redo shows a short notice naming the step ("Undone: Approve 03 · Redo ⌘⇧Z")") |
| C2(b) | a step that could not apply | `historyUndo` rejects with `new ServiceError('step_cannot_apply', "Can't undo Move to Lamp 2: that session is published", { label: 'Move to Lamp 2', reason: 'that session is published' })`; ⌘Z | a toast whose text is exactly `Can't undo Move to Lamp 2: that session is published`; nothing else changes (`activeTile()` unchanged) | `[rule]` MC-30 ("the app says why ("Can't undo Move to Lamp 2: that session is published")") |
| C2(c) | nothing to undo / redo | `historyUndo` rejects `ServiceError('nothing_to_undo', …)`; ⌘Z; `historyRedo` rejects `ServiceError('nothing_to_redo', …)`; ⌘⇧Z | toasts `Nothing to undo.` then `Nothing to redo.` | `[rule]` MC-30 (the icons "are disabled when there is none"); P08 C2 wording |
| C3(a) | header icons | `historyGet → { undo: 'Approve 03', redo: null }`; open the workspace; read the header | two icon-only buttons in the header, `Undo` before `Redo`, before the bell; `Undo` has no `aria-disabled`, its card reads `Undo · ⌘Z — Approve 03`; `Redo` has `aria-disabled="true"`, its card reads `Redo · ⌘⇧Z — Nothing to redo`; on the Sessions page (`openSessions`) no button named `Undo` or `Redo` exists | `[rule]` MC-30 ("The header has an Undo and a Redo icon whose hover card names the step they would act on; they are disabled when there is none") |
| C3(b) | the icons follow the event | from C3(a); deliver `history_changed { session_id: SESSION, undo: null, redo: 'Approve 03' }`; then deliver `{ session_id: 99, undo: 'Other', redo: null }` | after the first: `Undo` is `aria-disabled="true"` with card `Undo · ⌘Z — Nothing to undo`, `Redo` enabled with `Redo · ⌘⇧Z — Approve 03`; the second (another session) changes nothing | `[rule]` MC-30 (as C3(a)); master plan §6.7 (`history_changed`) |
| C3(c) | clicking the icons | from C3(a); click `Undo`; click `Redo` (disabled) | `historyUndo` called once with `SESSION`; `historyRedo` **not** called | `[rule]` MC-30; P35 (`aria-disabled` controls never act) |
| C4(a) | active photo after an undo | P08 C2(b) re-pointed: `historyUndo → { label: 'Delete 01', direction: 'undone', active_photo_id: 1, sessions: [] }`; delete tile 1 with Backspace, wait 6 000 ms, ⌘Z | `historyUndo` called once with `SESSION`; `activeTile() === 1`; the delete toast is gone | MC-5 ("photo becomes active"); MC-30 ("The delete toast … stays; its Undo is the history's ⌘Z") |
| C4(b) | the delete toast's Undo | P08 C2(c) re-pointed: Backspace; click the toast's `Undo` button | `historyUndo` called once with `SESSION`; `undoDelete` is not a function of the photos service | `[rule]` MC-30 ("The delete toast … its Undo is the history's ⌘Z") |
| C5(a) | the editor follows an undone framing | compose harness: enter the editor on photo 1 (default square `rect (0.166601562, 0, 0.666796875, 1)`, the entry values); deliver `photo_changed { photo_id: 1, session_id }` with `getSessionView` now answering photo 1's composition as `rect {0.25, (1 − 1280/1707)/2, 0.5, 1280/1707}`, `revision 2` (what an undo or redo leaves behind); wait for the reload; then press `Escape` | during the sync **no** `setComposition` call is made; the Esc (Revert to the entry values) makes **exactly one** `setComposition(1, v)` with the entry rect `(0.166601562, 0, 0.666796875, 1) ± 1e-6` — possible only because the machine's values became the stored `V1` (without the sync, Esc would save nothing: live = entry) | MC-30 (10D: "Undo and redo work in the editor"); P31 machine rule 16; M36 |
| C6(a) | grid Split on a selection is one step | workspace, select tiles 2 and 3 (both rendered, `kind normal`), ⋯ ▸ Split on tile 2 | exactly one `applyComposition` call with two items `{ photo_id: 2 | 3, values }` whose `values.kind === 'split'` and `rect` is the P33 re-fit of each photo's rect; no `setComposition` call; both tiles patched to `kind split` (P33 C4 assertions on `.composed-split`) | MC-30 ("Apply composition to selected … as one step"; P40 Notes (6)); M36 |
| C6(b) | a single grid Split keeps its path | one photo, ⋯ ▸ Split | one `setComposition` call, no `applyComposition` (P33 C4(a) keeps passing) | MC-30 (one save, one step) |
| C7(a) | the Edit menu follows the labels | `historyGet → { undo: 'Approve 03', redo: null }`; open the workspace; deliver `history_changed { SESSION, undo: null, redo: 'Approve 03' }` | `editMenuSet` called with `('Approve 03', null)` after the load, then with `(null, 'Approve 03')` after the event; no call for the other session's event | `[rule]` MC-30 ("The app's Edit menu carries "Undo <step>" and "Redo <step>"") |
| C7(b) | the Edit menu acts | deliver `edit_menu { action: 'undo' }`; then `{ action: 'redo' }` | `historyUndo` then `historyRedo`, each once with `SESSION` | `[rule]` MC-30 ("with the same keys") |

## Delegated choices
The module store's shape (`useSyncExternalStore` vs a tiny context); whether `useHistoryKeys` mounts in `App` or `WorkspacePage` (it must run
while composing and while dialogs are open, so `App` is recommended); the icons (`Undo2`, `Redo2`); where the two buttons sit in the header
(after the divider, before the bell, is the contract of C3(a)); whether the editor-sync effect keys on `revision` or on `updated_at`
(`revision`); the menu's remaining submenus (keep Tauri's default items; only Edit is rebuilt).

## Manual try-out
1. Open a session. Approve three photos (A, A, A). Press ⌘Z three times: each time the newest approval comes back as unreviewed and a small
   notice says "Undone: Approve 0n · Redo ⌘⇧Z". Press ⌘⇧Z: "Redone: Approve 01 · Undo ⌘Z". The two arrow icons at the top (left of the bell)
   follow: hover them to read "Undo · ⌘Z — Approve 02" and "Redo · ⌘⇧Z — …"; when there is nothing, the icon is greyed and the card says so.
2. Open the Edit menu in the menu bar: it reads "Undo Approve 02 ⌘Z" and "Redo … ⇧⌘Z"; pick "Undo …": the same as ⌘Z. **Press ⌘Z once in the
   workspace and check that exactly one step is undone** (not two): if two are undone, say so — the fix is to drop the menu accelerators (Notes).
3. Press R, drag the photo, Return; ⌘Z: the framing is back and the tile redraws. Press R again, press ⌘Z while in the editor: the framing on
   the stage jumps back to the previous one without leaving the editor; ⌘⇧Z brings it forward.
4. Press D, type in Description, press ⌘Z while the cursor is in the field: the typing is undone by the field itself, nothing else changes. Click
   outside the field (still in the form) and press ⌘Z: the last saved field edit is undone with a notice "Undone: Edit Description · Redo ⌘⇧Z".
5. Delete a photo: the toast "Photo deleted · Undo ⌘Z" appears; click its Undo: the photo is back and active. Delete another, wait for the toast
   to go, press ⌘Z: the same.
6. Select two photos, ⋯ ▸ Split: both split; ⌘Z once: both are Normal again.
7. Move a photo to a published session, publish… (or ask: move a photo out of a session that is published and clean back into it): ⌘Z says
   "Can't undo Move to …: that session is published" and the next ⌘Z undoes the step before.

## Notes
- **Planner readings (reversible; for the narrative's "Choices made for you")**:
  (1) ⌘Z and ⌘⇧Z work with any dialog open (details form, pickers) as long as the keyboard is not in a text field; inside a field they are the
  field's own undo.
  (2) The notice is a toast at the bottom right (like the delete toast), one at a time, replaced by the next.
  (3) The Undo/Redo icons sit in the session header beside the camera dot; they are not shown on the Sessions page (no open session).
  (4) The Edit menu entries carry the keys ⌘Z / ⇧⌘Z. In WKWebView the page's own key handler runs first and claims the key, so the menu entry
  fires only when picked with the mouse (observed: P08's ⌘Z handler has worked alongside Tauri's default Edit ▸ Undo ⌘Z since 2026-10-25). If
  the try-out ever shows a double undo, the fallback is menu items **without** accelerators (then the menu shows no key); nothing else changes.
  (5) After an undo of a framing while composing that photo, the editor jumps to the restored framing; a photo with a save still in flight is
  left alone until its save settles.
  (6) A grid Split or Normal on two or more selected photos is one step; on one photo it is one step too (its own save).
- **Service module placement (hazard for the suite)**: eleven existing tests mock `@/services/events` with factories that enumerate the exports
  (`p07`, `p08`, `p09`, `p15`, `p18`, `p19` ×2, `p20`, `p22`, `p26`, the compose harness). A new export in `events.ts` would make every one of them
  throw "No … export is defined on the mock" when `App` mounts. Hence `onHistoryChanged`/`onEditMenu` live in `services/history.ts`, which those
  tests leave unmocked: `historyGet` then rejects (no Tauri in jsdom) and `listen` rejects likewise — both swallowed by `useHistory` — so the old
  tests pass without edits. P41's own tests mock `@/services/history`.
- **Hazards**: Radix closes an open tooltip on Esc before the editor sees it (P36 Notes (9)) — no new Esc handling here. `useComposeKeys`
  must keep letting ⌘Z through (`:48–66`): do not add a `swallow()` to its ⌘ fall-through. `isTyping` treats `SELECT` as typing — ⌘Z with the
  Shape menu focused does nothing; acceptable (the menus blur on change since P36). `history_changed` arrives **before** or after
  `photo_changed` — the header reads the event's labels, the workspace reloads on either; never two reloads per step (`commandsInFlight` guard).
  The menu is built on the main thread at startup; `edit_menu_set` is a `pub async fn` command (the `commands_are_async` test) that touches the
  stored `MenuItem` handles (`Send + Sync` in Tauri 2). Removing `photos_undo` changes `bindings.ts`: the regenerated file must be committed.
- **C5(a)'s assertion**: the default square and the new `V1` rect are both squares, so the frame box is the same `{174, 24, 652, 652}` and the
  DOM cannot tell them apart; the row proves the sync through the machine's Esc rule (P31: Esc reverts to the entry values and saves when they
  differ) — one save with the entry rect after the sync, none without it.
- **Tests that change their expectation**: P08 C2(b), C2(c) (mock and resolved shape); the six `undoDelete: vi.fn()` lines. No other test may
  change; a test that goes red for any other reason is a defect of this phase.
- Registry additions made by the planner in master plan §6.7, §6.10. No intention-book change.
- Estimated size: ~26 paths (4 new, 13 changed, 2 new tests, 7 adjusted), ~1 000 lines (≈ 450 features/services/Rust, ≈ 550 tests).

## Review log

- **2026-11-05, planner note after P40's projection r0.** Checked for carry-over: none needed. The step labels P41 shows (`Edit Description`,
  `Clear Title`, `Unreview 03`, `Compose photo`) and the refusal texts (`that photo is gone`, `that session already has this photo`) come from the
  backend; P41's rows use literal label strings on mocked services, and `history_changed` is emitted by the backend for every step kind (P39
  C7(c)). The Edit menu labels follow the same strings through `edit_menu_set`.
