---
plan: P08_workspace_ui
role: implement
round: 1
state: DONE
date: 2026-10-25
actor: codex (run on Claude Opus 5.5, per prompt)
---
# P08 workspace UI: implement r1 handoff

## What was built
Opening a session now shows the **workspace** instead of the placeholder:
- **Photo stage.** Shows the active photo as two stacked images: the camera original, rotated by the RAW's orientation, and Capture One's adjusted version, which is never rotated. Switching between them crossfades over 700 ms. While the adjusted render is still on its way, the original shows and the pill spins. When a new render arrives (photo_changed), it fades in over the old one. The stage also has prev/next buttons and a "Main 5 of 7 · 3 to review" line.
- **Review.** The round buttons Delete, Move to Condition (reads "Move to Main" on a condition photo) and Approve. Keys: A, C, ⌫/Delete, ←/→ (↑/↓), Space, F, I, Esc, ⌘Z. Swipes on the stage: right approves, up sends to Condition, left deletes (120 px sideways / 110 px up). After an action the next unreviewed photo opens, in display order. Delete shows "Photo deleted · Undo ⌘Z" for 4.5 s with an Undo button. ⌘Z keeps working after the toast is gone, and the restored photo becomes active.
- **Photo panel.** Main images come first, then the unreviewed photos after Main (with a dot), then Condition. Number and Cover badges, a "Processing" tag, the active photo highlighted. Dragging inside a group reorders it. Dragging into the other group classifies the photo there. ⌘/⇧-click selects several photos. F folds the panel to a 64 px strip with a "COND" divider. The ⋯ button and right-click open a menu: Move to Main/Condition, Assign to product…, Move to session…, Copy to session…, Delete.
- **Item card (I).** Shows title, SKU, barcode, type and photo counts.
- **Unassigned workspace.** The header shows a "No product" tag, and "Assign product" replaces Publish. Assign product searches items and then moves the whole Unassigned set to that item's session, then opens it. The sessions panel opens on the right; dropping photos on a row moves them there. Published rows are dimmed and are not drop targets.
- **Import.** Drop files from Finder onto the window, or use the import button in the panel header (macOS file picker). Both go to `photos_import`.
- **Arrival toast.** A photo that lands in Unassigned while you are anywhere else raises "Photos arrived in Unassigned" with an Open button.
- **Backend (deviation, see below).** `session_get` now returns the session, its item and its visible photos. Preview files load through the asset protocol. The dialog plugin is installed.

## Tests → criterion rows
Frontend file: `apps/desktop/src/features/workspace/__tests__/p08_workspace_ui.test.tsx`. All services are mocked; tests go through the real `<App/>` at `#/session/:id`.

| Row | Test | Assertion vs row |
|---|---|---|
| C1(a) | `C1(a) A on the active unreviewed photo classifies it into Main` | `classifyPhoto` called once with `(1,'main')`: exact |
| C1(b) | `C1(b) C classifies it into Condition` | `(1,'condition')`, once: exact |
| C1(c) | `C1(c) ⌫ deletes it` | `deletePhoto(1)`, once: exact |
| C1(d) | `C1(d) after each action the earliest-captured remaining unreviewed photo opens` | u1(captured 200, id 9), u2(100, id 12): active becomes 12, the second A classifies 12, then active becomes 9. Exact (catches ordering by id) |
| C2(a) | `C2(a) delete shows … closes at 4 500 ms` | fake timers: text present and `data-removed=false` at 4 499 ms, `true` at 4 500 ms, out of the DOM after Sonner's 200 ms exit |
| C2(b) | `C2(b) ⌘Z at 6 000 ms …` | toast gone at 6 000 ms, active = q; ⌘Z → `undoDelete(5)` once; active (highlighted tile) = p: exact |
| C2(c) | `C2(c) the toast Undo button undoes` | `undoDelete(5)` once: exact |
| C3(a) | `C3(a) with no adjusted render …` | original `data-visible=true`, no adjusted layer, pill has `role=status` "Processing" |
| C3(b) | `C3(b) photo_changed delivering an adjusted path …` | after the event: adjusted `src = asset:///previews/1/adjusted-2.jpg`, class `crossfade`, inline `transition: opacity 700ms` |
| C3(c) | `C3(c) Space switches to Original for the whole session view …` | after Space and →: photo 2's original visible, its (loaded) adjusted hidden, pill `aria-pressed=true`. Fails if the mode resets per photo |
| C3(d) | `C3(d) orientation 6 rotates the original …` | original `transform: rotate(90deg)`, adjusted no transform |
| C4(a) | `C4(a) the grid shows Main by position, …` | DOM order [a,b,u,d]; d alone in the Condition region |
| C4(b) | `C4(b) dragging a from index 0 to index 2 …` | real dnd-kit pointer drag (rects laid out in the test): `reorderPhoto(1,3)`, no classify |
| C4(c) | `C4(c) dragging d from Condition onto Main index 0 …` | `classifyPhoto(4,'main')`, no reorder |
| C4(d) | `C4(d) the folded strip …` | strip sequence `[1,2,3,'COND',4]` |
| C5(a) | `C5(a) the sessions panel is open, …` | Sessions panel present, "No product", "Assign product" button, no Publish |
| C5(b) | `C5(b) dropping the selected photos p and q on session S …` | ⌘-click p and q, drag p onto row 7 → `movePhotos([1,2],7)` |
| C5(c) | `C5(c) a published session row is not a drop target` | row `data-droppable=false`; the drop makes no `movePhotos` call |
| C5(d) | `C5(d) "Assign product" then picking an item …` | `assignUnassigned(50)` |
| C6(a) | `C6(a) a file drop of /x.ARW imports that path` | `importPhotos(['/x.ARW'])` |
| C7(a) | `C7(a) a photo arriving in Unassigned while on the Sessions page …` | toast text, then Open → hash `#/session/1` |
| C8(a) | `C8(a) ⋯ "Copy to session…" then S …` | `copyPhotos([1],7)` |
| Core mechanism `displayOrder` | `orders Main and Condition by position and unreviewed by captured_at, then id` | unit test, input → output (MC-5 display rule, id tie-break) |
| **Candidate C0** (not a plan row) | `crates/testkit/tests/p08_workspace_ui.rs::c0_session_view_lists_item_and_visible_photos` | `SessionsService::view` returns the item and every photo except the deleted one; Unassigned has no item. Declared in the Review log for the coordinator |

Counts: 22/22 rows covered. Vitest: 23 P08 tests (22 rows plus 1 core mechanism). Rust: 1 candidate test. Red baseline: **not captured.** The tests were written after the production code, so there are no failing ids to report.

## Assumptions
- **Preview URLs:** `convertFileSrc` through the asset protocol. Every render has its own path (`adjusted-<g>.jpg`), so there is no cache to bust.
- **New captures:** a capture arriving in the open session becomes the active photo (ch04 S-C, "appears on the stage").
- **A and C on photos already in that group:** A on a Main photo and C on a Condition photo change nothing and just open the next unreviewed photo.
- **Drags:** dragging a photo across groups classifies it but does not move the active photo.
- **Multi-selection:** ⌘/⇧-click selects several photos. Dragging, the ⋯ menu and right-click act on the whole selection when the photo is part of it.
- **"Assign to product…" (single photo or selection):** runs `session_create(item)` (which returns an existing session) and then `photos_move`.
- **"Assign product" (whole Unassigned set):** searches existing items only (no "Create" row). Afterwards the app opens the item's session.
- **Item card:** opens inside the stage, top right. The P07 header ⓘ icon stays decorative.
- **Sessions panel:** opens by default in Unassigned. Any workspace can open it with a "Sessions" button on the stage.
- **Dropped and picked files:** passed to `photos_import` unfiltered. The backend's candidate rules decide what counts as a photo; the picker has no filter.
- **Grid numbering:** unreviewed photos are numbered after Main in the grid (display only, as in screen 11).
- **Not added:** the capture button `S` (only-if-cheap) and a "Create" button in the sessions panel.

## Deviations
1. **Plan defect, worked around with the smallest option.** P01's `session_get` returned counts only, so the frontend had no way to get the photo list. Task 1 ("load `session_get`") assumes it does.
   - `session_get` now returns `SessionViewResponse {session, item, photos}`. It is backed by the new `SessionsService::view` in `crates/core/src/services/sessions.rs` and the new `SessionView` in `crates/core/src/domain/session.rs`, with DTOs in `apps/desktop/src-tauri/src/commands/sessions.rs`.
   - `PhotoClassification` now also derives `Serialize` (`commands/photos.rs`).
   - Bindings regenerated.
   - The coordinator should fold this into the master plan §6.7 (`session_get` payload) and decide whether to adopt candidate criterion C0.
2. **Asset protocol enabled** so previews can load at all:
   - `tauri` feature `protocol-asset`;
   - `app.security.assetProtocol.enable` in `tauri.conf.json`, with an empty static scope;
   - at startup, `lib.rs` allows only `<data dir>/previews/`.
   - P07's handoff noted that P08 owns this choice.
3. **New dependencies** beyond the master plan §6.2 pins:
   - `tauri-plugin-dialog = "2"` (2.8.3) and `@tauri-apps/plugin-dialog@2.8.3`, for the picker (capability `dialog:allow-open`);
   - `@dnd-kit/utilities@3.2.2`, dnd-kit's own CSS transform helper.
   - `@dnd-kit/core@6.3.1`, `@dnd-kit/sortable@10.0.0` and `motion@14.6.0` were installed as pinned.
4. **P07 files touched:**
   - `features/shell/Header.tsx`: Unassigned "No product" tag and "Assign product" button. Row C5(a) needs both in the header.
   - `features/sessions/__tests__/p07_shell_sessions_ui.test.tsx`: its `@/services/events` mock gained the three new exports, and `@/services/photos` is mocked with an empty view. No assertion changed.
5. **New files not in the plan list:** `features/workspace/PickerDialogs.tsx` (session and item pickers), `features/workspace/workspace.css` (styles, imported by `WorkspacePage`), `crates/testkit/tests/p08_workspace_ui.rs` (candidate C0).

## Files changed
Mutation-probe files: none (the mutation regime is off).
- `Cargo.lock`
- `apps/desktop/package.json`, `apps/desktop/package-lock.json`
- `apps/desktop/src-tauri/Cargo.toml`, `capabilities/default.json`, `tauri.conf.json`, `src/lib.rs`, `src/commands/sessions.rs`, `src/commands/photos.rs`
- `apps/desktop/src/services/bindings.ts` (regenerated), `services/photos.ts` (new), `services/events.ts`
- `apps/desktop/src/App.tsx`
- `apps/desktop/src/features/shell/Header.tsx`, `features/shell/ArrivalToasts.tsx` (new)
- `apps/desktop/src/features/workspace/` (all new): `WorkspacePage.tsx`, `Stage.tsx`, `StageImage.tsx`, `TogglePill.tsx`, `ReviewButtons.tsx`, `SwipeLayer.tsx`, `PhotoPanel.tsx`, `PhotoGrid.tsx`, `PhotoTile.tsx`, `FoldedStrip.tsx`, `PhotoMenu.tsx`, `SessionsPanel.tsx`, `ItemDetailsCard.tsx`, `ImportDropZone.tsx`, `PickerDialogs.tsx`, `useWorkspaceState.ts`, `useKeyboard.ts`, `order.ts`, `workspace.css`, `__tests__/p08_workspace_ui.test.tsx`
- `apps/desktop/src/features/sessions/__tests__/p07_shell_sessions_ui.test.tsx`
- `crates/core/src/domain/session.rs`, `crates/core/src/services/sessions.rs`
- `crates/testkit/tests/p08_workspace_ui.rs` (new)
- `docs/implementation/master_plan.md` (P08 tracker row only), `docs/implementation/plans/P08_workspace_ui.md` (Review log)
- This handoff (not in the checkpoint; the runner commits handoffs)

## Checkpoint SHA
`2b79552` — "CHECKPOINT (not approved): P08 workspace — stage, review, order, Unassigned, import". The tree was clean after the commit.

## Stamp
Tree `2b79552`. There were no pre-existing failures, and every command below exited 0.

| Command | Result |
|---|---|
| `cargo test --workspace` | 183 passed, 0 failed (includes `export_bindings`; the regenerated `bindings.ts` equals the committed one) |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean |
| `cargo fmt --all -- --check` | clean |
| `npm run typecheck` (apps/desktop, Node 22) | clean |
| `npm test` (apps/desktop) | 2 files, 42 passed (P07 19, P08 23) |
| Extra: `npx vite build` | built; only Vite's chunk-size warning |
| Extra: launch smoke | `target/debug/psm-desktop` with `PSM_DATA_DIR=/tmp/psm-p08-smoke` stayed up with no panic; stopped after 5 s |

Not run: a hardware probe (no tethered camera in this session) and a look at the live UI.

## Manual try-out
Start the app with `cd apps/desktop && npm run tauri -- dev`.
1. Open a session and shoot a frame. Within about a second it appears on the stage as the camera original, with a spinning pill. A few seconds later the adjusted version fades in.
2. Press A, C and ⌫ on three frames; the next unreviewed photo opens each time. ⌘Z brings the deleted one back where it was, and it becomes active. Drag a Main photo to another slot. Drag a Condition photo onto Main and it joins the end of Main.
3. Press Space to compare original and adjusted; the choice stays as you move with ←/→. Edit the photo in Capture One and come back to the app: the preview fades to the new render.
4. Press F to fold the panel to the strip, and F again to unfold it. Press I for the item card. Right-click a photo, or use ⋯ above the stage, and choose "Copy to session…".
5. Go back to Sessions (Back) and shoot one frame. The toast "Photos arrived in Unassigned" appears; click Open. The sessions panel is open: drag the photo onto a session. Or click "Assign product", search, and pick an item.
6. Drag a RAW from Finder onto the workspace, or click the import icon in the panel header, to import it.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
