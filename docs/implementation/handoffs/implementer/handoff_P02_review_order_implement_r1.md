---
plan: P02_review_order
role: implement
round: 1
state: DONE
date: 2026-10-25
actor: codex
---

# P02 review and order — implement handoff

## What was built

The photographer's review actions now classify, reorder, hide, restore, move and copy photos. Main and Condition positions stay contiguous; a delete keeps the RAW and saves one undo slot. Each effective action updates the affected sessions' change sequence and activity time in one SQLite transaction. Session responses expose the derived dirty flag. Six desktop commands and generated TypeScript bindings expose the actions for the future workspace UI.

## Tests → criterion rows

`crates/testkit/tests/p02_review_order.rs` has one boundary test for each of the 25 rows:

| Rows | Test IDs |
|---|---|
| C1(a–e) | `c1a_classify_appends_main`, `c1b_classify_first_condition`, `c1c_main_to_condition_closes_gap`, `c1d_condition_to_main_closes_gap`, `c1e_same_classification_is_noop` |
| C2(a–d) | `c2a_reorder_forward`, `c2b_reorder_backward`, `c2c_reorder_clamps_high`, `c2d_reorder_clamps_low` |
| C3(a–f) | `c3a_delete_hides_and_closes_gap`, `c3b_undo_restores_position_and_active`, `c3c_only_last_delete_undoes`, `c3d_undo_clamps_prior_position`, `c3e_empty_undo_fails_without_change`, `c3f_unreviewed_undo_stays_unpositioned` |
| C4(a–f) | `c4a_move_keeps_source_order`, `c4b_copy_preserves_raw_and_hash`, `c4c_move_unreviewed_keeps_null_position`, `c4d_existing_raw_refuses_atomically` (move and copy), `c4e_published_target_refuses_atomically`, `c4f_move_out_of_published_makes_dirty` |
| C5(a–c) | `c5a_each_operation_bumps_touched_sessions_once` (all seven sub-checks), `c5b_photo_change_after_publish_is_dirty`, `c5c_dirty_requires_published_and_newer_change` |
| C6(a) | `c6a_clock_stamps_all_touched_sessions` |

The allowed position mechanism has one unit test: `mechanisms::position::tests::mc5_position_operations`.

## Assumptions

- The existing JSON `undo_slot` column stores `{photo_id, prior_classification, prior_position}`.
- Repeating a classification or reorder that leaves the position unchanged is a no-op, including no sequence bump.
- A published target is refused when it is not dirty, as specified by ordered task 2 and C4(e). A dirty published target remains eligible.
- A move/copy batch can contain photos from several sources; each touched source receives one sequence bump, and the target receives one.
- The move/copy result returns affected sessions in ascending session ID order. Every result includes each session's `change_seq` and `dirty`.

## Deviations

- Command registration, error-code mapping, and the existing session response required `apps/desktop/src-tauri/src/lib.rs`, `commands/mod.rs`, `commands/items.rs`, `commands/sessions.rs`, and `crates/core/src/domain/session.rs` beyond the plan's enumerated files. These are necessary wiring for the listed commands and dirty DTO; no other phase behavior was added.
- The `mark_published` fixture updates only the disposable test database. No migration changed.

## Files changed

Checkpoint: `apps/desktop/src-tauri/src/commands/{items,mod,photos,sessions}.rs`, `apps/desktop/src-tauri/src/lib.rs`, `apps/desktop/src/services/bindings.ts`, `crates/core/src/domain/{photo,session}.rs`, `crates/core/src/error.rs`, `crates/core/src/mechanisms/{mod,position}.rs`, `crates/core/src/ports/store.rs`, `crates/core/src/services/{mod,review}.rs`, `crates/infra/src/sqlite/store.rs`, `crates/testkit/src/test_app.rs`, `crates/testkit/tests/p02_review_order.rs`, `docs/implementation/{master_plan.md,plans/P02_review_order.md}`. This handoff is left for the runner to commit per flow §8.

## Checkpoint SHA

`b167b76` — `CHECKPOINT (not approved): implement P02 review and order`.

## Stamp

Checkpoint tree `b167b76`; all commands exited 0 after the final test edit. `cargo test --workspace`: 25 P02 boundary tests plus the existing Rust suite and the position unit test passed. `cargo clippy --workspace --all-targets -- -D warnings`: passed. `cargo fmt --all -- --check`: passed. `PATH=/Users/davidloorenz/.nvm/versions/node/v22.22.3/bin:$PATH npm run typecheck`: passed. The same PATH with `npm test`: passed, with no frontend test files present. `git diff --check`: passed. Baseline failing IDs: none; final failing IDs: none.

## Manual try-out

There is no review UI until P08. From the repository root, run `cargo test -p psm-testkit --test p02_review_order` to see the 25 review scenarios pass against a disposable SQLite database. In the dev app, the Health page is unchanged. After P08, the workspace will expose these actions for direct use.

## ⚠ OWNER DECISIONS REQUIRED (0)

None.
