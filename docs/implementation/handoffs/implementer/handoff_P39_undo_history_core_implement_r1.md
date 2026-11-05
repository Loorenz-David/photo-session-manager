---
plan: P39_undo_history_core
role: implement
round: 1
state: BLOCKED
date: 2026-11-05
actor: codex
---

# P39 implement r1 handoff

## What was built

Review actions now enter a 25-step history for each session. Undo and redo reverse classification, deletion, reorder, move and copy as whole database transactions. The history survives a settings rebuild and clears on quit. Undo of classification requeues the Capture One colour-tag mirror. A published or otherwise unusable step is dropped with a reason. The backend emits `history_changed`, exposes `history_get`, `history_undo` and `history_redo`, and keeps the existing `photos_undo` response shape for the current frontend. The old delete slot is no longer read or written; its database column remains.

The phase is blocked by a contradiction inside C4(h). The step's move undo is dropped when the source already holds the RAW, so the original photo remains in the target session. The preceding approval step then fails the plan's explicit classify precondition (`photo.session_id == session_id`). C4(h) requires that second undo to succeed. The test asserts the row's stated success and fails with `StepCannotApply { direction: "undo", label: "Approve 01", reason: "that photo already changed" }`. MC-30 also says an inapplicable step changes nothing and is dropped. The planner should amend C4(h)'s final assertion to expect that error and drop, or change the reversal contract and specify the resulting session/position/sequence semantics. The tracker remains `NOT_STARTED`; this checkpoint is not an implementation approval.

## Tests → criterion rows

| Row | Boundary test | Result |
|---|---|---|
| C1(a) | `c1a_limit_25` | pass |
| C1(b) | `c1b_new_step_clears_redo` | pass |
| C1(c) | `c1c_session_isolation` | pass |
| C1(d) | `c1d_state_labels` | pass |
| C2(a) | `c2a_classify_undo_mirror_and_sequence` | pass |
| C2(b) | `c2b_delete_undo_clamps_after_drop` | pass |
| C2(c) | `c2c_unreviewed_delete_undo` | pass |
| C2(d) | `c2d_classify_redo` | pass |
| C2(e) | `c2e_changed_photo_drops_step` | pass |
| C3(a) | `c3a_reorder_undo` | pass |
| C3(b) | `c3b_reorder_redo` | pass |
| C4(a) | `c4a_move_undo` | pass |
| C4(b) | `c4b_copy_undo_deletes_row_and_composition` | pass |
| C4(c) | `c4c_published_move_drop_then_noop_reorder` | pass |
| C4(d) | `c4d_published_copy_drop` | pass |
| C4(e) | `c4e_move_redo` | pass |
| C4(f) | `c4f_multi_move_order` | pass |
| C4(g) | `c4g_confirmed_running_publish_blocks_copy_undo` | pass |
| C4(h) | `c4h_raw_collision_blocks_move_undo` | fail at required second undo; contradiction above |
| C4(i) | `c4i_copy_undo_repoints_mirror` | pass |
| C5(a) | `c5a_arrival_create_settings_not_steps` | pass |
| C5(b) | `c5b_settings_rebuild_keeps_history` | pass |
| C5(c) | `c5c_quit_clears_history` | pass |
| C6(a) | `c6a_history_changed_events` | pass |
| C7(a) | `p39_tests::c7a_history_commands` | pass |
| C7(b) | `p39_tests::c7b_photos_undo_compatibility` | pass |
| C7(c) | `p39_tests::c7c_tauri_history_changed_event` | pass; mock app handle delivered the actual command event |

Core mechanism unit tests: `mc30_record_limit_and_state`, `mc30_record_clears_redo` pass. The prior P02 behaviour tests pass after the planned API and C3(d) expectation updates.

## Assumptions

- A transfer spanning source sessions records one step for each source, as the delegated choice recommends.
- The transfer label uses the target item's title, falling back to SKU when the title is empty; Unassigned uses its fixed name.
- The operation lock is held by each `History` instance. The history lists are shared across settings rebuilds, as specified.
- The app build call gained one argument, so its existing multi-argument signature received a local Clippy allowance.

## Deviations

- C4(h)'s final assertion conflicts with §Reversal rules and MC-30. I kept the strict precondition and left its boundary test red. No implementation was added to fake the required success.
- The tracker row was not marked `IMPLEMENTED` because the criterion and full suite cannot pass as written.
- No architecture graph artifact exists in this repository. No hardware or live Capture One probe was needed.

## Files changed

Checkpoint: `apps/desktop/src-tauri/src/commands/{history,items,mod,photos}.rs`, `apps/desktop/src-tauri/src/lib.rs`, `apps/desktop/src/services/bindings.ts`, `crates/core/src/domain/photo.rs`, `crates/core/src/{error,events}.rs`, `crates/core/src/ports/store.rs`, `crates/core/src/services/{history,mod,photos,review}.rs`, `crates/infra/src/sqlite/store.rs`, `crates/integrations/src/lib.rs`, `crates/testkit/src/test_app.rs`, `crates/testkit/tests/{p02_review_order,p05_capture_arrival,p27_retire_shape_on_arrival,p39_undo_history_core}.rs`, `docs/implementation/plans/P39_undo_history_core.md`. This handoff was written after the checkpoint for the runner to commit.

## Checkpoint SHA

`bbfede7` — `CHECKPOINT (not approved): P39 undo history core, blocked on C4h plan contradiction`.

## Stamp

| Command | Result |
|---|---|
| `cargo test --workspace` | **red**: only `p39_undo_history_core::c4h_raw_collision_blocks_move_undo`; 23/24 P39 service tests pass, all earlier test binaries and the 3 desktop command tests pass. The failing assertion is the required second undo; actual error is `StepCannotApply { direction: "undo", label: "Approve 01", reason: "that photo already changed" }`. |
| `cargo test -p psm-desktop p39_tests` | 3/3 pass after final DTO binding change. |
| `cargo test -p psm-desktop export_bindings` | pass; generated bindings updated. |
| `cargo clippy --workspace --all-targets -- -D warnings` | pass. |
| `cargo fmt --all -- --check` | pass. |
| `npm run typecheck` | pass. |
| `npm test` | 355/355 pass. |
| `git diff --check` | pass. |

## Manual try-out

1. Start the app, open a session, approve three photos with A, and press ⌘Z three times. Each press restores the newest approval to unreviewed; a fourth reports nothing to undo.
2. Delete a photo, reorder another, then press ⌘Z twice. The reorder reverses first, then the deleted photo returns to its old position. The queued colour tags follow in Capture One.
3. Move a photo to another session and press ⌘Z from its source session. The photo returns to its prior place.
4. Change the SKU prefix setting and return: history remains. Quit and relaunch: the history is empty while the photo data remains.

## ⚠ OWNER DECISIONS REQUIRED (0)

None. The planner/orchestrator can resolve the C4(h) contract contradiction from the existing intention.
