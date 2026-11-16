---
plan: T3_template_hand_changes
role: implement
round: 1
state: DONE
date: 2026-11-16
actor: claude opus 5.5 (pipeline-implementer)
---
# T3 — Your changes win, and there is a way back · implement r1

## What was built
- **Photos that differ from the template are known.** In a templated session the session's template view now says, per photo, whether it differs from the template and how many do. A photo differs when the person placed it (or it came from before the template / from another session) and it is not where the template would put it now, or when the person re-framed it while it sits in a container that has framing presets. Deleted photos never differ.
- **A quiet warning with a way back.** Above the Main photos, one line: "2 photos differ from the template" with a **Back to template** button. Each differing photo's tile carries a small amber dot. The line disappears when nothing differs.
- **Back to template** puts every differing photo back where the template wants it now (container or Condition) and gives it that container's split / shape / zoom when the container has presets. It is one undo step ("Back to template"), with the usual undo toast; ⌘Z brings your hand placement back, ⌘⇧Z the template again. A denied photo is never brought back by it.
- **Drop onto a container.** Dragging a photo onto an empty faded container puts it there. Dragging a placed photo onto another placed photo (or its container) swaps the two; both are then marked as differing. Dropping a photo from Condition onto a filled container is refused for now with the toast "That container is filled" (T4 makes it the retake). One undo step, "Move to container <n>".
- **Undo keeps container order.** Undoing or redoing any older step in a templated session leaves the Main photos in container order (e.g. a photo you sent to Condition and then undo returns to its own container, between the photos that arrived meanwhile).
- **Commands:** `template_drop(photo_id, container_no)`, `template_back(session_id)`, both answering the touched session like the review commands; wrappers `templateDrop`, `templateBack`.
- Untemplated sessions: nothing changes (no row written, no step, no warning, drops behave as before).

## Tests → criterion rows
| Row | Test |
|---|---|
| T3·C1(a) | `crates/testkit/tests/suite/t3_template_hand.rs::t3_c1a_condition_photo_made_main_differs` |
| T3·C1(b) | `…::t3_c1b_photo_back_in_its_container_matches_again` |
| T3·C1(c) | `…::t3_c1c_reframed_photo_differs_only_with_presets` |
| T3·C1(d) | `…::t3_c1d_moved_photo_differs_in_its_new_session` |
| T3·C1(e) | `…::t3_c1e_copy_differs_original_stays` |
| T3·C1(f) | `…::t3_c1f_photo_row_with_place_row_can_be_deleted` |
| T3·C1(g) | `…::t3_c1g_undo_keeps_container_order` |
| T3·C2(a) | `…::t3_c2a_drop_fills_empty_container` |
| T3·C2(b) | `…::t3_c2b_drop_on_filled_container_swaps` |
| T3·C2(c) | `…::t3_c2c_drop_on_own_container_changes_nothing` |
| T3·C2(d) | `…::t3_c2d_condition_photo_onto_filled_container_refused` (asserts `Error::ContainerFilled`; its code string `container_filled` is the mapping in `commands/items.rs`) |
| T3·C2(e) | `…::t3_c2e_drop_refusals` |
| T3·C2(f) | `…::t3_c2f_drop_is_one_undo_step` |
| T3·C3(a) | `…::t3_c3a_back_to_template_undoes_the_swap` |
| T3·C3(b) | `…::t3_c3b_back_to_template_reapplies_presets` |
| T3·C3(c) | `…::t3_c3c_back_to_template_returns_extra_main_to_condition` |
| T3·C3(d) | `…::t3_c3d_back_to_template_is_one_undo_step` |
| T3·C3(e) | `…::t3_c3e_back_to_template_leaves_a_denial` |
| T3·C3(f) | `…::t3_c3f_photos_from_before_the_template_are_filed_by_back` |
| T3·C3(g) | `apps/desktop/src-tauri/src/lib.rs::t3_tests::t3_c3g_template_commands_answer_no_template` (wired app + `bindings.ts`) |
| T3·C4(a) | `apps/desktop/src/features/workspace/__tests__/t3_template_warning.test.tsx` "T3·C4(a) …" |
| T3·C4(b) | same file, "T3·C4(b) …" |
| T3·C4(c) | same file, "T3·C4(c) …" |
| T3·C4(d) | same file, "T3·C4(d) dropAction …" (the plan's one core mechanism, pure) |

## Assumptions
1. **One Main order rule.** P7's Main key is applied as (has no slot, slot, previous position). T2 fix 1's rule ("a photo placed by hand keeps its absolute position") is replaced, because it contradicts T3·C1(g) and T3·C2(b) (backlog note from the T2 re-review). The new rule keeps T2's dragged-order test green: a drag writes the dragged photo's position and slot first, and photos sharing a slot keep their order. The "placed by hand" sort key is dropped because, put before the position, it breaks a drag downwards (only the position orders photos sharing a slot correctly in both directions).
2. **A photo keeps its container number when it leaves Main.** Classify and Deny no longer clear the slot in the photo's row; only a Main photo gets a new slot from a reorder (a reorder inside Condition no longer writes its Condition position as a slot, closing the T2 r1 backlog note). This is what brings a photo classified Condition → Main (C1(b)) or undone back (C1(g)) into its own container. The view and "differs" read a slot only for Main photos.
3. **Undo/redo of a drag (reorder) in a templated Main** writes the slot of the position it restores, mirroring what the drag wrote.
4. **A swap exchanges positions too**, so ties in the order follow the swap.
5. **Back to template leaves the composition alone** (pending zoom included) for a photo going to a container without presets or to Condition (the plan's "unchanged"); it never restores the default composition there.
6. **The template view also re-reads on `history_changed`.** Review actions emit no photo events, so without this the warning would not update after a classify, a compose edit, a drop or an undo in the real app.
7. `session_view(store, id)` keeps its signature and reads holds at the system clock; `session_view_at(store, now, id)` takes the caller's clock (tests use it).
8. Back to template's toast reads "Back to template · Undo ⌘Z" (same id and lifetime as the Deny toast). The differs mark is an amber dot (`--warn`) at the tile's top right.

## Deviations
- Files outside the per-unit lists, each required: `apps/desktop/src-tauri/src/commands/items.rs` (exhaustive error-code match must name `ContainerFilled`), `crates/core/src/ports/store.rs` (`StoreTx::photo_denied_at`, for the step snapshot of a denial), `apps/desktop/src/features/workspace/PhotoTile.tsx` (`TemplateDiffersContext`, `data-differs`, the mark — so Condition tiles can be marked too), `apps/desktop/src/features/workspace/useTemplateView.ts` (assumption 6).
- `crates/core/src/domain/template.rs` unchanged: the step's per-photo state (`PhotoState`, `TemplateChange`) lives in `services/history.rs` next to the other step types.
- Known edge, not a row: a container photo dragged (plain reorder) behind the extra Main photos takes the slot number of its new position and so sorts before the extras, not after them. It is marked as differing, and Back to template restores it.
- No hardware probe was run (optional under flow §6A); the suite covers every row.
- The agent guide (`docs/guide/`) is not updated here; the templates master plan gives that to T4.

## Files changed
Rust: `crates/core/src/services/template_placement.rs`, `crates/core/src/services/history.rs`, `crates/core/src/error.rs`, `crates/core/src/ports/store.rs`, `crates/infra/src/sqlite/store.rs`, `apps/desktop/src-tauri/src/commands/templates.rs`, `apps/desktop/src-tauri/src/commands/items.rs`, `apps/desktop/src-tauri/src/lib.rs`, `crates/testkit/tests/suite/main.rs`, `crates/testkit/tests/suite/t3_template_hand.rs` (new).
TypeScript: `apps/desktop/src/services/bindings.ts` (generated), `apps/desktop/src/services/templates.ts`, `apps/desktop/src/features/workspace/{TemplateWarning.tsx (new), templateDrop.ts (new), PhotoPanel.tsx, ContainerGrid.tsx, PhotoTile.tsx, WorkspacePage.tsx, useWorkspaceState.ts, useTemplateView.ts, workspace.css}`, `apps/desktop/src/features/workspace/__tests__/t3_template_warning.test.tsx` (new).
Docs: `docs/implementation/plans/T3_template_hand_changes.md` (Tracker line, Review log), this handoff.
No schema change, no archgraph state, no settings key, no new event.

## Checkpoint SHA
- U1 `8057056` — differs in the view, order after undo, place row deleted with the photo row
- U2 `3d7ea16` — drop onto a container, `StepKind::Template`
- U3 `8db8fe4` — Back to template, commands, bindings
- U4 `f4e9432` — warning, marks, drop in the session view
- Closing: the `CHECKPOINT (not approved): T3 closing …` commit containing the clippy type alias in the T3 test file, the plan's Tracker/Review log and this handoff (its SHA is the commit that adds this file).

## Stamp
| Command | Result |
|---|---|
| `cargo test --workspace` | ok — 1540 tests, 0 failed (3 min 48 s wall) |
| `cargo clippy --workspace --all-targets -- -D warnings` | first run: 1 error (`type_complexity` in the T3 test helper); fixed with a type alias; re-run ok; `t3_template_hand::` re-run ok (19 passed) |
| `cargo fmt --all -- --check` | ok |
| `cd apps/desktop && npm run typecheck` | ok |
| `npm test` | ok — 85 files, 536 tests |
Unit commands, each run after its unit: `t3_template_hand::` (7 → 13 → 19 passed), `p39_undo_history_core::` (24 passed), `t2_template_placement::` (21 passed), `psm-core services::history` (3 passed), `cargo test -p psm-desktop` (29 passed), clippy core/infra/desktop ok, fmt ok, `npx vitest run src/features/workspace` (15 files, 83 tests), typecheck ok. Pre-existing failures: none seen.

## Manual try-out
On scratch data, with a "Dining chair" shooting template of three or more containers (Settings → Shooting templates) and a dining-chair session where the containers are filled and a few photos sit in Condition:
1. Move four Condition photos to Main (drag them into Main, or press A on each). A line above Main says "4 photos differ from the template", and each of those tiles has a small amber dot. Shoot another photo: nothing moves back.
2. Open another dining-chair session with containers filled. Drag the split photo in place 3 onto the photo in place 2: the two swap places, both get the dot, and the line says "2 photos differ from the template".
3. Press **Back to template**: both go back and the line disappears. Press ⌘Z: your swap is back. Press ⌘⇧Z: the template order again.
4. Re-frame the split photo in Compose: it gets the dot. Press Back to template: it gets the container's split and zoom again.
5. Send a Main photo to Condition so its container is empty, then drag a Condition photo onto that faded container: it is placed there. Dragging a Condition photo onto a filled container shows "That container is filled".

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
