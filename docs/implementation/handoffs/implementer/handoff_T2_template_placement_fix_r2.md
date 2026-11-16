---
plan: T2_template_placement
role: fix
round: 2
state: DONE
date: 2026-11-16
actor: claude sonnet 5.5 (pipeline-implementer)
---
# T2 fix 1

## Finding → what changed → files
**B1-1 (reorder undone / off by one in a templated session).**
- New private `main_positions` in `template_placement.rs`, used by both `renumber_main` and the Main sort of `place_arrival`. A Main photo with a place row, `place_by_hand = 1` and a position keeps that position (shifted only to stay inside 1…m and clear of another such photo). All other Main photos fill the remaining positions by (no slot, slot, previous position, id). This is the reviewer's suggested P7 rule.
- `review.rs::reorder` and `store.rs::photo_place_mark` needed no change (reorder already writes positions before renumbering).
- One new behaviour test, `t2_b1_reorder_in_templated_session_keeps_the_dragged_order`: reviewer's probe scenario plus a later arrival.
- Files: `crates/core/src/services/template_placement.rs`, `crates/testkit/tests/suite/t2_template_placement.rs`.

## Divergences
- The review asked the orchestrator to settle P7's rule first; no settled text was in the artifacts, so the reviewer's suggestion was implemented as written. `docs/implementation/master_plan_templates.md` P7 still carries the old sentence and needs the new rule folded in by the orchestrator (T3 drop/fill/swap and T4 shift renumber through the same function). I did not edit it (outside the perimeter).
- `review.rs` and `store.rs` were allowed but not touched.

## Files changed
- `crates/core/src/services/template_placement.rs`
- `crates/testkit/tests/suite/t2_template_placement.rs`
- `docs/implementation/plans/T2_template_placement.md` (tracker row, Review log entry)
- this handoff
- Mutation probes: none.

Checks, one cargo command at a time: `t2_template_placement::` 21/21 (20 + the new one); `psm-core mechanisms::` 93/93; `p02_review_order::` 25, `p39_undo_history_core::` 24, `t1_templates::` 12, all green; clippy (core, infra, desktop, testkit, `-D warnings`), `cargo fmt --check`, `npm run typecheck`: green. Full suite not run: no shared store/migration code changed (the renumber is a no-op for untemplated sessions).

## Checkpoint SHA
`46d5ea8` (code and docs). This handoff line was added after the commit and is committed by the runner.

## Stamp
Code tree after the checkpoint commit; targeted runs above green, no failures.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
