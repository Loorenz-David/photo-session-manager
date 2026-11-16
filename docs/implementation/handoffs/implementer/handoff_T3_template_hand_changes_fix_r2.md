---
plan: T3_template_hand_changes
role: fix
round: 2
state: DONE
date: 2026-11-16
actor: claude sonnet 5.5 (pipeline-implementer)
---
# T3 fix 1 — undo of a reorder among extra Main photos

## Finding → what changed → files
**T3-R1-B1 (production).** Undo of a reorder in a templated session wrote `slot = <restored position>`, so an extra Main photo that had no slot kept sorting ahead and `renumber_main` put the drag's order back.
- `StepKind::Reorder` gained `place_before` and `place_after` (slot, placed by hand). `ReviewService::reorder` reads them from the photo's place row around the drag's `photo_place_mark`.
- `history.rs` `apply` writes `place_before` on undo and `place_after` on redo (`photo_place_row` + `photo_place_put`), replacing the `photo_place_mark(…, Some(target))` call. A row is created only when the value to write is not the default and none exists, so untemplated sessions write nothing.
- Test `t3_c1g_undo_of_a_reorder_among_extra_main_photos_restores_the_order`: F4, F5 → Main; `reorder(F5, 4)`; undo → F4 4, F5 5; redo → F5 4, F4 5.
- Files: `crates/core/src/services/history.rs`, `crates/core/src/services/review.rs`, `crates/testkit/tests/suite/t3_template_hand.rs`.

## Divergences
- None. No store port change was needed (existing `photo_place_put`/`photo_place_row` suffice). Undo also restores `place_by_hand` as it was before the drag, which the correction clause ("leave the place row as it was") implies.

## Files changed
- `crates/core/src/services/history.rs`
- `crates/core/src/services/review.rs`
- `crates/testkit/tests/suite/t3_template_hand.rs`
- `docs/implementation/plans/T3_template_hand_changes.md` (Tracker line, Review log)
- `docs/implementation/handoffs/implementer/handoff_T3_template_hand_changes_fix_r2.md`

No mutation probes; no production file was edited and reverted.

## Checkpoint SHA
`4e1ba0c` (`CHECKPOINT (not approved): T3 fix 1 …`). This line was added after the commit; the runner commits the handoff.

## Stamp
Run one after the other on the fixed tree: `t3_template_hand::` 20/20 · `p39_undo_history_core::` 24 · `t2_template_placement::` 21 · `p02_review_order::` 25 · `p62_retention` 12 · `t1_templates::` 12 · `cargo test -p psm-core` 96 · clippy (core, infra, desktop, testkit, `-D warnings`) ok · `cargo fmt --check` ok · `npm run typecheck` ok. Full suite not run (change local to the Reorder step; no store, migration or binding change).

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
