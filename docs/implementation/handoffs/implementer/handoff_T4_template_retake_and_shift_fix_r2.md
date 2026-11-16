---
plan: T4_template_retake_and_shift
role: fix
round: 2
state: DONE
date: 2026-11-16
actor: claude sonnet 5.5 (pipeline-implementer)
---
# T4 — fix 1 (round 2)

## Finding → what changed → files
**T4-R1-B1** (B1, production). Shift the rest back was offered, and "succeeded" with nothing moving, after a Condition photo was denied while a later Condition photo existed.
- `can_shift_back` now compares each non-Deleted, template-placed photo's expected place (classification, slot) with the session's shot indexes as they are against the same indexes with every Deleted photo's shot left out. True only when one differs. `shift_back` uses the same function, so otherwise it answers `nothing_to_shift` with no write and no undo step.
- Helper: `shot_indexes_without` (skips a set of photos); `shot_indexes` delegates to it. `can_shift_back` takes the template and `now` as extra arguments.
- New test `t4_c2e2_denied_condition_shot_with_later_condition_photo_has_nothing_to_shift`: S0, F4 denied → `can_shift_back` false, `shift_back` → `NothingToShift`, undo label unchanged.
- Files: `crates/core/src/services/template_placement.rs`, `crates/testkit/tests/suite/t4_template_retake_shift.rs`.

## Divergences
None.

## Files changed
- `crates/core/src/services/template_placement.rs`
- `crates/testkit/tests/suite/t4_template_retake_shift.rs`
- `docs/implementation/plans/T4_template_retake_and_shift.md` (Tracker line; Review log entry)
- `docs/implementation/handoffs/implementer/handoff_T4_template_retake_and_shift_fix_r2.md` (this file)

No mutation probes. Not touched: `docs/guide/02_architecture.md` line ~88 still words `can_shift_back` as "a later … photo"; slightly stale after this fix, outside the finding's files (note for the orchestrator).

## Checks (run one at a time)
`t4_template_retake_shift::` 12/12 · `t2_template_placement::` 21/21 · `t3_template_hand::` 19/19 · clippy (core, infra, desktop, testkit, all targets, `-D warnings`) ok · `cargo fmt --all -- --check` ok · `npm run typecheck` ok. Full suite not run: the change is local to one service function and its one caller (no store, migration or shared code changed).

## Checkpoint SHA
See the commit `CHECKPOINT (not approved): T4 fix 1` (the SHA is the HEAD of the branch after this session).

## Stamp
Tree clean after the checkpoint commit; files above only.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
