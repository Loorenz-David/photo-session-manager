---
phase: A08_research_feedback_and_undo
title: What you do with a suggestion is remembered — accepting, clearing, editing, keeping the previous value or using the suggestion each leave a record, and Undo never wipes out a suggestion that research wrote after the step
difficulty: high
projection: yes
depends_on: [A07_research_storage_and_write_permission]
---
# A08 — Feedback rows, conflict actions, guarded undo and redo (08B MC-A12; MA18)

## Goal
Every photographer action on an `ai` row writes one `research_feedback` row in the action's own transaction; a
conflict gets its three actions (Keep previous, Use suggestion, Edit); "Looks right" becomes an accept (the row
stays `ai`, status `accepted`), as MC-A12 says; and an attribute undo or redo applies only when the field still
holds the row the step expects, deleting or re-inserting the feedback rows it owns. **Not in this phase:** the
publish side of a resolution (A09 adds the hold and its release to the same transaction), the popover and form UI
for conflicts (A20), the panel (A18).

## Read first
`master_plan_research.md` §3, §6.3, §6.4, §6.6, §7.4 · book `14` (MA18), `08` (Values and provenance), `08A` §7,
`08B` **MC-A12** (all), MC-A1 (prior, conflict, cleared set), `05A` MC-30 and its research pointer (line 307) · code
`crates/core/src/services/attributes.rs` (`set_value`, `accept_all` line 511, `clear` line 546, `record`),
`crates/core/src/services/history.rs` (`FieldChange`, `FieldState`, the attribute arm at lines 715–758,
`Error::StepCannotApply`), `apps/desktop/src-tauri/src/commands/attributes.rs`,
`apps/desktop/src/features/form/{FormDialog.tsx (rawValue, "Looks right"), AiPopover.tsx}`,
`apps/desktop/src/features/form/__tests__/{p11_form_ui.test.tsx, p22_form_tidy.test.tsx}`.

## Dependencies and gate
Tracker row and gate: `master_plan_research.md` §4. Notes: `backlog_research.md`.

## Files expected to change
`crates/core/src/services/{attributes.rs, history.rs}`, `crates/core/src/ports/store.rs`,
`crates/infra/src/sqlite/store.rs` (feedback delete and re-insert by `step_token`),
`apps/desktop/src-tauri/src/{commands/attributes.rs, lib.rs}` (commands `attribute_ai_accept`,
`attribute_conflict_resolve`), `apps/desktop/src/services/{bindings.ts (regenerated), attributes.ts}`,
`apps/desktop/src/features/form/FormDialog.tsx` (Looks right calls the accept command),
`apps/desktop/src/features/form/__tests__/{p11_form_ui.test.tsx, p22_form_tidy.test.tsx}` (the Looks right
assertions), new `crates/testkit/tests/suite/research/a08_research_feedback_and_undo.rs` (+ its `mod` line), own
tracker row.

## Core mechanisms
None new (the comparison "current row equals the expected row on every column except `updated_at`" is a private
function of `history.rs`, proven through C3).

## Ordered tasks
1. `AttributesService::accept(item_id, key)` (Looks right, and Use suggestion on a conflict): `ai` + `suggested` →
   `ai` + `accepted`, `ai_prior` dropped; one undo step.
2. `AttributesService::keep_previous(item_id, key)`: the row becomes the prior's value, validity and provenance, the
   AI columns and `ai_prior` null; one undo step.
3. `clear` and a human edit (`set_value`) drop `ai_prior`. `accept_all` skips `low` (as today) **and every conflict**.
4. Each of these, when the row before the action had provenance `ai`, inserts one feedback row in the same
   transaction: the action, the suggested value and confidence from before, the final value (null for a clear; the
   prior's value for keep previous; the new canonical value for an edit), the time, and `step_token` = the history
   step's id.
5. History: an attribute step applies only if, for **every** field of the step, the current row equals the expected
   row (the step's after-row for undo, its before-row for redo) on every column except `updated_at`. Otherwise the
   whole step is not applied, it is dropped, and the notice reads `that field was changed by research`. Undoing a
   step deletes the feedback rows with its token; redoing re-inserts them. Steps that fall off the 25-step limit or
   are cleared at quit leave their rows.
6. Commands `attribute_ai_accept(item_id, key)` and `attribute_conflict_resolve(item_id, key, action, value?)`
   (thin); the form's Looks right calls the first.

## Criteria (5 criteria, 16 rows, 1 `[rule]`)
Boundary: the service methods on a `TestApp`; outcome = the stored row, the `research_feedback` rows, the history state.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | `accept` | `ai` + `suggested` A, medium, prior D | row `ai` + `accepted` A, prior null; one feedback row `accepted`, suggested A, medium, final A, with the step's token | MA18 |
| C1(b) | `accept_all` | three suggestions: medium, `low`, and a conflict at high | only the first is accepted; one feedback row | MA18 (R-IA1) |
| C1(c) | `keep_previous` | `ai` + `suggested` A with prior `{D, valid, default}` | row `default` D, AI columns and prior null; feedback `kept_previous`, suggested A, final D; a later `ai_suggest` of A is refused as cleared | MA18 |
| C1(d) | `clear` | `ai` + `suggested` A with a prior; `ai` + `accepted` B | both rows `cleared`, prior null; feedback `cleared`, final null, twice | MA18 |
| C1(e) | `set_value` (human edit) | `ai` + `suggested` A with a prior; new value H | row `human` H, prior null; feedback `edited`, suggested A, final H | MA18 |
| C1(f) | any of the actions | a `human` row edited; a `default` row cleared | no feedback row | MA18 |
| C2(a) | `ai_suggest` | a research write | no feedback row; history state unchanged | MC-A12 |
| C3(a) | undo | clear suggestion A (step); research then writes B; undo | nothing changes (row still `ai` + `suggested` B); the step is gone from the history; notice `that field was changed by research` | MA18 (R-IA21) |
| C3(b) | redo | clear A; undo (row is `ai` + `suggested` A again); research replaces A with B; redo | nothing changes; the step is dropped; the same notice | MA18 |
| C3(c) | undo, two fields | `fill_defaults` fills X and Y (one step, both rows `default`); research then writes a suggestion over Y; undo | neither X nor Y changes (X keeps its default, Y its suggestion) | MA18 |
| C3(d) | undo, unchanged | clear A; undo | row is `ai` + `suggested` A again, equal to before on every column except `updated_at` | MC-A12 |
| C4(a) | feedback and undo | clear A (feedback `cleared`); undo | the feedback row is gone; `ai_suggest` of A is written again | MA18 |
| C4(b) | feedback and redo | C4(a), then redo | the feedback row exists again with the same token; A is refused as cleared | MA18 |
| C4(c) | step limit | 26 accepts on 26 fields | 26 feedback rows remain although the first step fell off the history | MC-A12 |
| C5(a) | commands | `attribute_ai_accept`, `attribute_conflict_resolve` with `keep_previous`, `use_suggestion`, `edit` + value | each produces exactly one undo step and the row of C1(a), C1(c), C1(a), C1(e) | MC-A12 |
| C5(b) | form UI | Looks right on a suggestion (UI test) | the accept command is called with the item and key; `attribute_values_set` is not | `[rule]` 08B MC-A12 |

## Delegated choices
Method names beyond those listed · where the token is generated · the label text of the new undo steps (follow the
existing "Clear <label>" style).

## Manual try-out
On scratch data, with a suggestion made by hand (none exists yet without research): nothing new to try in the app
beyond Undo and Redo of form edits behaving as before. The rules are visible in
`cargo test -p psm-testkit --test suite research::a08_research_feedback_and_undo::`.

## Notes
- **Behaviour change, by contract:** "Looks right" used to make the value human-owned
  (`master_plan_research.md` §7.4); it now accepts the suggestion. The two UI tests named above are updated to
  assert the new call.
- C3(c) uses `fill_defaults` (P67) because its after-rows are `default`, a state research may write over (MC-A1);
  an accepted or human row cannot be changed by research, so a step over such rows could never show the guard.
- Review mode: this phase's module, `p39_undo_history_core::`, `p40_undo_history_values::`, `p10_attributes::`, the
  two form UI test files, clippy, fmt, typecheck.

## Review log
