---
phase: P08_workspace_ui
title: Workspace — stage, review, order, Original/Adjusted, Unassigned, import (frontend)
difficulty: high
projection: no
depends_on: [P06_live_previews, P07_shell_sessions_ui]
---
# P08 — Workspace: stage, review, order, Original/Adjusted, Unassigned, import (frontend)

## Goal
Build the session workspace from the design: the photo stage with the stacked original/adjusted images and the
700 ms crossfade, the Original/Adjusted pill (Space), the round Approve/Condition/Delete buttons, keyboard and
swipe actions, the "next unreviewed opens" rule, the undo toast and ⌘Z, the photo panel (Main grid, Condition grid,
unreviewed shown after Main) with dnd-kit reordering and cross-group drops, the folded strip, the ⋯/context menu
(Move/Copy to session…, Assign to product…, Delete), the Unassigned workspace with the sessions panel and drops,
file import, the item details card, and the toast "Open" for arrivals while elsewhere. Subscribes to `photo_added`,
`photo_changed`, `session_changed`. **Not in this phase:** Export/Publish header actions (P09, P15), the form (P11),
per-photo upload badges (P15). Large but purely frontend; **refine at prompt time** from the screens listed.

## Read first
master plan §6.7 (P02/P05 commands, events), §6.9 · book `01`, `14`, `06` (all), `04` S-C, S-E · contracts `05A`
MC-5 (display rule R-I2; drag between groups = classify into g), MC-2 (move/copy refusals, import); `10A` MC-20
(orientation applies to the original only) · evidence `design_handoff.md` §Review, §Original / Adjusted, §Keyboard ·
`claude_design/NOTES.md` §Screens 4–5, §Flows (Review, Reorder, Move or copy), §Undo timing, §Original / Adjusted
toggle; `COMPONENTS.md` §Workspace; screens `desktop/11–21`, `33`, `narrow/02`, `04`.

## Files expected to change
`apps/desktop/src/features/workspace/{WorkspacePage.tsx, Stage.tsx, StageImage.tsx, TogglePill.tsx, ReviewButtons.tsx,
SwipeLayer.tsx, PhotoPanel.tsx, PhotoGrid.tsx, PhotoTile.tsx, FoldedStrip.tsx, PhotoMenu.tsx, SessionsPanel.tsx,
ItemDetailsCard.tsx, ImportDropZone.tsx, useWorkspaceState.ts, useKeyboard.ts, order.ts (display order helper),
__tests__/p08_workspace_ui.test.tsx}`, `apps/desktop/src/services/{photos.ts, events.ts (subscriptions)}`,
`apps/desktop/src/features/shell/ArrivalToasts.tsx` (new), `apps/desktop/src/App.tsx` (route `/session/:id`),
`apps/desktop/src-tauri/capabilities/default.json` (drag-drop / dialog permissions if a plugin is used).

## Core mechanisms (unit tests allowed)
- `features/workspace/order.ts: displayOrder(photos) -> {main[], unreviewed[], condition[]}` (MC-5 display rule: main by position; unreviewed by `captured_at` then id; condition by position).

## Ordered tasks
1. Workspace state hook: load `session_get`, subscribe to the three events, keep the active photo id and the
   toggle mode (per session view).
2. Stage: two stacked `<img>` (original with CSS rotation from `orientation`, adjusted as delivered); crossfade
   700 ms; spinner on the pill while `adjusted_preview_path` is null; prev/next.
3. Review actions: buttons, keys A / C / ⌫, swipes via Motion (`drag`, thresholds 120 px horizontal, 110 px vertical);
   after an action the next unreviewed (display order) becomes active; the middle button reads "Move to Main" on a
   condition photo.
4. Undo: Sonner toast "Photo deleted · Undo ⌘Z" for 4.5 s with an Undo action; ⌘Z anywhere → `photos_undo`; the
   returned `active_photo_id` becomes active.
5. Panel: dnd-kit `SortableContext` per group, `DragOverlay`, drop in the other group → `photos_classify`; folded
   strip (`F`), 64 px; ⋯ menu + context menu; item card (`I`).
6. Unassigned workspace (`/session/1`): sessions panel open, droppable rows (published disabled), "Assign product"
   (reuses the New session dialog's search) → `photos_assign_unassigned`.
7. Import: Tauri drag-drop onto the workspace and a file picker → `photos_import(paths)`.
8. Arrival toast: on `photo_added{session_id: 1}` while not on `/session/1` → toast "Photos arrived in Unassigned"
   with "Open".
9. Tests.

## Criteria (8 criteria, 22 rows; 12 `[rule]` rows)
| Row | Boundary | Input (services mocked) | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | key `A` on active unreviewed photo p | — | `photos_classify(p, "main")` called once | `[rule]` ch06 §Classification |
| C1(b) | key `C` | — | `photos_classify(p, "condition")` | `[rule]` ch06 |
| C1(c) | key `⌫` | — | `photos_delete(p)` | `[rule]` ch06 |
| C1(d) | after any action | unreviewed u1 (`captured_at` 200, id 9), u2 (`captured_at` 100, id 12) remain | active becomes u2 (earliest `captured_at`), then u1 | M12 (display rule R-I2) |
| C2(a) | delete | fake timers | toast "Photo deleted · Undo ⌘Z" visible; gone at 4 500 ms | `[rule]` evidence §Review (undo timing) |
| C2(b) | `⌘Z` at 6 000 ms | `photos_undo` mock returns `{active_photo_id: p}` | `photos_undo(session)` called once; p is active and highlighted | `[rule]` ch06 (⌘Z until next delete) |
| C2(c) | toast Undo button click | — | `photos_undo` called once | `[rule]` ch06 |
| C3(a) | Stage | photo with `adjusted_preview_path = null` | original layer visible; pill shows a spinner | `[rule]` ch06 §Original / Adjusted |
| C3(b) | `photo_changed` event delivering an adjusted path | — | adjusted layer rendered with the crossfade class (700 ms transition) and `src` = `asset://` URL of the new path | M24 (new render shown) |
| C3(c) | `Space` then navigate to another photo | mode toggled to Original | the next photo also shows the original (mode is per session view) | `[rule]` ch06 (one toggle per session view) |
| C3(d) | Stage | original with `orientation = 6` | original `<img>` carries a 90° rotation; adjusted carries none | MC-20 (orientation applies to the original) |
| C4(a) | `displayOrder` + grid render | main [b(2), a(1)], unreviewed [u], condition [d(1)] | DOM order: a, b, u, then the condition section with d | M12 |
| C4(b) | dnd-kit drag end within Main | a from index 0 to 2 | `photos_reorder(a, 3)` | M12 |
| C4(c) | drag d from Condition onto Main at index 0 | — | `photos_classify(d, "main")` (not reorder) | M12 (drag between groups = classify) |
| C4(d) | folded strip render | same data | thumbnails in the same order with the "COND" divider before d | M12 |
| C5(a) | `/session/1` | — | sessions panel rendered open; header tag "No product"; primary action "Assign product" | `[rule]` ch06 §Unassigned |
| C5(b) | drop photos [p, q] on session row S | S open | `photos_move([p, q], S)` | `[rule]` ch06 §Unassigned (drag moves) |
| C5(c) | drop on a row whose status is `published` | — | row is not a drop target; no call | MC-2 (published sessions are not targets) |
| C5(d) | "Assign product" → pick item | — | `photos_assign_unassigned(item)` called | `[rule]` ch06 |
| C6(a) | file drop event with `[/x.ARW]` | — | `photos_import(["/x.ARW"])` | MC-2 (import path) |
| C7(a) | `photo_added{session_id: 1}` while on `/` | — | toast with "Open"; click navigates to `/session/1` | `[rule]` ch04 S-E |
| C8(a) | ⋯ menu "Copy to session…" → S | — | `photos_copy([p], S)` | MC-2 (copy) |

## Delegated choices
Asset protocol vs. `convertFileSrc` for preview URLs (must bust nothing: the path changes per render), swipe stamp
visuals, panel resize animation, context-menu library, how the picker is opened (Tauri dialog plugin acceptable),
whether `I` opens a popover or a sheet.

## Manual try-out
1. Open a session, shoot: the frame appears within ~1 s as the camera original with a spinner; the adjusted version
   fades in a few seconds later.
2. Press `A`, `C`, `⌫` on three frames; the next unreviewed opens each time. `⌘Z` brings the deleted one back where
   it was. Drag a Main photo to another slot; drag a Condition photo into Main.
3. Press `Space` to compare original/adjusted; edit the photo in Capture One and return to the app: the preview
   updates.
4. Close the session (Back), shoot one frame with no session open → toast "Photos arrived in Unassigned · Open";
   drag it onto a session in the panel.

## Notes
- Capture trigger from the UI (`S`) is only-if-cheap (ch13); `camera_capture` exists from P04 — adding the button is
  a delegated choice, not a criterion.
- Estimated size: ~22 files, ~2 600 lines.

## Review log
- 2026-10-25 implement r1 (actor `codex` per prompt; run on Claude Opus): **IMPLEMENTED.** Built the session workspace:
  `useWorkspaceState` (loads `session_get`, follows `photo_added`/`photo_changed`/`session_changed` for its session,
  keeps the active photo and the per-view Original/Adjusted mode), the stage (stacked original/adjusted `<img>`, original
  rotated per EXIF orientation 1–8, adjusted unrotated, 700 ms opacity crossfade, a new render laid over the previous one
  and faded in, prev/next, "Main n of m · k to review"), the pill (Space, spinner while `adjusted_preview_path` is null),
  round Delete / Condition-or-"Move to Main" / Approve buttons, keys A C ⌫ ←/→ ↑/↓ Space F I Esc ⌘Z, Motion swipes
  (120 px horizontal, 110 px up), next-unreviewed rule, undo toast (4.5 s, Undo action) and ⌘Z, the photo panel (Main
  then unreviewed after Main, Condition) with dnd-kit sortable groups, `DragOverlay`, reorder within a group,
  classify across groups, ⌘/⇧-click multi-selection, the folded 64 px strip with the COND divider, ⋯ and right-click
  menus (Move/Copy to session…, Assign to product…, Delete, Main/Condition), the item card (I), the Unassigned
  workspace (sessions panel open with droppable rows, published rows disabled, "No product" tag and "Assign product"
  in the header → item search → `photos_assign_unassigned` → opens the item's session), import by Finder drop (Tauri
  webview drag-drop) and by a file picker (Tauri dialog plugin), and the "Photos arrived in Unassigned · Open" toast.
  **Deviations (plan defect worked around, smallest option):** (1) `session_get` returned only counts, so the workspace
  had no photo list; it now returns `{session, item, photos}` (new `SessionsService::view` in core: item + photos
  except deleted; command DTOs `SessionViewResponse`, `PhotoResponse`; bindings regenerated). (2) Previews could not
  load: asset protocol enabled (`tauri` feature `protocol-asset`, `assetProtocol.enable`, scope set at startup to the
  data dir's `previews/` only). (3) `tauri-plugin-dialog` 2 / `@tauri-apps/plugin-dialog` 2.8.3 added for the picker
  (capability `dialog:allow-open`); `@dnd-kit/utilities` 3.2.2 added (CSS transform helper of dnd-kit). (4) Header.tsx
  (P07) gained the Unassigned tag and "Assign product"; P07's test mocks gained the new event/photo service exports.
  (5) New files not listed: `PickerDialogs.tsx`, `workspace.css`, `crates/testkit/tests/p08_workspace_ui.rs`.
  **Candidate criterion C0** (for the coordinator): `session_get` lists the item and every photo except deleted ones
  (ch06 "deleted hides the photo"; feeds M12's display) — test `c0_session_view_lists_item_and_visible_photos`.
  Judgment calls: `convertFileSrc` (asset protocol; each render has its own path, nothing to bust); a new capture in the
  open session becomes the active photo (S-C "appears on the stage"); A on a Main photo and C on a Condition photo
  change nothing and open the next unreviewed photo; a drag across groups does not advance the active photo; the
  menus act on the selection when the photo is selected; "Assign to product…" = `session_create(item)` then
  `photos_move`; Assign product lists existing items only (no create row); the item card is an in-stage card; the
  sessions panel can be reopened in any workspace; the import overlay shows while files hover; dropped paths are passed
  unfiltered (the backend's candidate rules decide); capture button `S` not added; unreviewed photos are numbered after
  Main in the grid (display only). No hardware probe run (no tethered camera in this session); the debug binary was
  launched against a scratch data dir and stayed up (no panic in the new setup code).
- 2026-10-25 review r1 (actor `opus`): **CHANGES_REQUESTED.** Tree `2b79552` (+ two docs-only runner commits, clean).
  Perimeter: the checkpoint's 41 paths equal the handoff's declared list. Suite `cargo test --workspace` 183/0, clippy
  and fmt clean, `npm run typecheck` clean, `npm test` 42/0. **Verified correct:** all 22 rows: production read and
  each test judged to observe the row's outcome at its boundary and to fail on a wrong one (C1(d) discriminates
  captured_at from id order; C2(a) at 4 499/4 500 ms; C2(b) after the toast is gone; C3(b) new asset URL + 700 ms
  opacity transition; C3(c) mode survives navigation; C3(d) rotation on the original only; C4(a)/(d) from an unsorted
  input; C4(b) position 3 and no classify; C4(c) classify and no reorder; C5(b) the whole selection; C5(c) a real drop
  on the disabled row makes no call; C5(d), C6(a), C7(a), C8(a) exact arguments). `displayOrder` matches MC-5 (R-I2).
  H1–H8: no violation (core `SessionsService::view` is Tauri-free; command thin; asset scope limited to `previews/`;
  no secrets; no live services in tests). Candidate C0 (`session_get` lists item + non-deleted photos) is sound.
  **F1 (B1, route production):** the `photo_added` handler in `useWorkspaceState.ts` sets only `activeRef`, never the
  rendered `activeId`, and `reload` keeps the stale state because the ref's photo is visible. Reviewer probe (a
  throwaway vitest file, deleted): opening an empty session and delivering `photo_added` leaves "Ready for the first
  photo" on the stage with no active photo (A/C/⌫ do nothing); with photo 1 active, a new capture 2 leaves the stage on
  photo 1, so C/⌫ act on photo 1 while ←/→ step from photo 2. Contradicts ch04 S-C / ch06 §Arrival ("appears on the
  stage") and the plan's Manual try-out step 1. Notes: 4 rows appended to `backlog.md`.
