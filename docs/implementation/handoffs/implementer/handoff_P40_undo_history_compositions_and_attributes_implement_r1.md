---
plan: P40_undo_history_compositions_and_attributes
role: implement
round: 1
state: DONE
date: 2026-11-05
actor: codex
---

# P40 implement r1

## What was built

Composition saves, resets, and Apply to selected now enter the session's undo history. An Apply is one step covering the photos it changed. Undo and redo save fresh composition revisions in one transaction, refuse a photo needing re-frame, and notify photo and session views after commit. Field edits, title edits, Clear, and Accept all now enter the same history. Reversal restores complete attribute rows, including provenance and timestamps, or removes rows that previously did not exist. Consecutive saves to one field coalesce. Redoing a copy whose source row is gone now drops that step with the specified message.

## ⚠ OWNER DECISIONS REQUIRED (0)

None.

## Tests → criterion rows

| Row | Boundary test |
|---|---|
| C1(a) | `c1a_composition_undo_saves_new_revision` |
| C1(b) | `c1b_composition_redo_saves_new_revision` |
| C1(c) | `c1c_reset_is_step` |
| C1(d) | `c1d_same_composition_is_one_step` |
| C2(a) | `c2a_apply_one_step_with_partial_refusal` |
| C2(b) | `c2b_apply_redo` |
| C2(c) | `p40_tests::c2c_composition_apply_command_is_one_step` |
| C3(a) | `c3a_needs_review_drops_composition_step` |
| C4(a) | `c4a_ai_row_restored_byte_for_byte_without_dirty` |
| C4(b) | `c4b_first_value_undo_removes_row` |
| C4(c) | `c4c_redo_reinstates_human_row` |
| C4(d) | `c4d_title_undo_redo` |
| C4(e) | `c4e_typing_coalesces` |
| C4(f) | `c4f_other_field_breaks_coalescing` |
| C5(a) | `c5a_accept_all_is_one_step` |
| C5(b) | `c5b_clear_is_separate_step` |
| C6(a) | `c6a_one_history_across_kinds` |
| C7(a) | `c7a_attribute_events_only_history` |
| C7(b) | `p40_tests::c7b_attribute_command_undo_restores_absence` |
| C8(a) | `c8a_copy_redo_missing_source_drops_step` |

The two command tests are in `apps/desktop/src-tauri/src/lib.rs`; the other 18 are in `crates/testkit/tests/p40_undo_history_values.rs`. The listed core mechanism has `mc30_attribute_steps_coalesce` in `history.rs`, covering a merge and Clear, Accept all, and other-field non-merges.

## Assumptions

- The nested `composition_apply` item uses `photo_id`, as the existing DTO and generated TypeScript binding do. C2(c)'s `photoId` spelling does not match that established nested wire shape.
- Apply retains partial success: refused or unchanged photos are excluded from its history step. A cross-session selection records under the first changed photo's session, per the plan.
- Schema refresh does not attach history; only the photographer's attribute command paths do. AI suggestion writes do not record a step.

## Deviations

- The C2(c) request fixture uses `photo_id` in its nested item. No DTO or frontend contract was changed.
- No file-perimeter deviation. No live Capture One probe was needed or run.

## Files changed

Checkpoint: `crates/core/src/services/{history.rs,composition.rs,attributes.rs}`, `crates/core/src/ports/store.rs`, `crates/infra/src/sqlite/store.rs`, `crates/integrations/src/lib.rs`, `apps/desktop/src-tauri/src/commands/{attributes.rs,compositions.rs}`, `apps/desktop/src-tauri/src/lib.rs`, `crates/testkit/src/test_app.rs`, `crates/testkit/tests/p40_undo_history_values.rs`, `docs/implementation/master_plan.md`, and `docs/implementation/plans/P40_undo_history_compositions_and_attributes.md`. This handoff is the only post-checkpoint path, for the runner to commit.

## Checkpoint SHA

`69d384274590b9e0b807e2ba0c78853d30943bc9` — `CHECKPOINT (not approved): implement P40 undo history for compositions and attributes`.

## Stamp

| Command | Result |
|---|---|
| `cargo test --workspace --quiet` | pass, all workspace tests (including 18 P40 testkit tests and two P40 command tests) |
| `cargo clippy --workspace --all-targets -- -D warnings` | pass |
| `cargo fmt --all -- --check` | pass |
| `cd apps/desktop && npm run typecheck` | pass |
| `cd apps/desktop && npm test` | pass, 48 files / 355 tests |

The first clippy run reported `type_complexity` on the new composition result. A named result type resolved it; the full Rust suite was rerun after that change. No pre-existing failures were observed. The stamped code is the checkpoint commit above.

## Manual try-out

1. In the running app, open a reviewed photo's editor (R), change its frame, and save with Return. Use the existing Undo shortcut (⌘Z): the prior frame returns and the photo redraws. A second undo reaches the previous photo action.
2. Copy one frame, select several photos, and choose Apply composition to selected. One Undo restores every changed photo's earlier frame.
3. Open Details (D), type a color and close the form. One Undo clears the entire newly typed value. Opening Details again shows the restored value.
4. Change a framed photo's geometry in Capture One and wait for Re-frame. Undo refuses the stale framing with a notice; the next Undo reaches the preceding step. Full Redo keys, history icons and Edit menu arrive in P41.
