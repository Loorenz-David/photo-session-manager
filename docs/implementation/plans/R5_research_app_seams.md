---
phase: R5_research_app_seams
title: Research in the form and in publishing — what you do with a suggestion is remembered and Undo never erases a newer suggestion; in a publish weak guesses are left out, and an item whose suggestion replaced a preset or Shopify value waits for your decision while the others go ahead
difficulty: high
projection: yes
depends_on: [R3_research_rules]
---
# R5 — App seams: the photographer's actions on suggestions, and publishing with unchecked suggestions

**Tracker:** NOT_STARTED · 2026-11-12 · planner · —

(This line is the milestone's tracker row. Sessions change only this line of this file: state, date, actor, one-line note. The gate for `depends_on` is the Tracker line of each plan named there. `master_plan.md` §4 is not used for research milestones.)

## Goal
The two places where existing app behaviour changes by contract: the attribute actions with their feedback rows and guarded undo (MC-A12), and the publish confirmation with withheld values, conflicts and the `held` step (MC-A10). Suggestions and conflicts in these tests are made with `psm_research::writes` (R3·U5). **Not here:** UI (R6), runs (R4), wiring (R7).

## How to work this plan (flow.md §10)
This milestone is one runner phase made of 2 units. Work the units in order. After each unit run its **Unit test commands**, then make a `CHECKPOINT (not approved):` commit of that unit's paths. One review, one fix budget and one narrative cover the whole milestone. Row ids keep the number of the superseded plan they were written in (`A07·C1(a)`); `docs/implementation/research_row_mapping.md` says where every row went. Name each test after its row (`a07_c1a_<slug>`). Reviewer notes go to `docs/implementation/backlog_research/R5_research_app_seams.md`.

| Unit | What it builds | Criteria | Rows | Changes app files |
|---|---|---|---|---|
| U1 | Feedback rows, conflict actions, guarded undo and redo | 5 | 16 | yes |
| U2 | Withheld values, conflicts and the held item in a publish | 7 | 18 | yes |
| **Total** | | **12** | **34** | |

## Read first (whole milestone)
`docs/process/flow.md` (**§10 is the authority for this track**) · `docs/implementation/master_plan.md` §3, §6, §7 · `docs/implementation/master_plan_research.md` (all: layout §0, standing rules §3, sequencing and tracks §5, registry §6, commands §7.2, the storage decision §6.4) · book `01`, `14` §Research agent, `08A`, `08B` · then each unit's own Read first. Name key and sameness are `psm_core::mechanisms::{name_key, sameness}` (R2·U1); every other research rule is `psm_research::rules`.

## Unit 1 — Feedback rows, conflict actions, guarded undo and redo

**Unit id:** `R5·U1` · **Replaces:** A08 (all rows) · **Changes app files:** yes

### Goal
Every photographer action on an `ai` row writes one `research_feedback` row in the action's own transaction; a
conflict gets its three actions (Keep previous, Use suggestion, Edit); "Looks right" becomes an accept (the row
stays `ai`, status `accepted`), as MC-A12 says; and an attribute undo or redo applies only when the field still
holds the row the step expects, deleting or re-inserting the feedback rows it owns. **Not in this phase:** the
publish side of a resolution (R5·U2 adds the hold and its release to the same transaction), the popover and form UI
for conflicts (R6·U3), the panel (R6·U1).

### What changed from the superseded plan (this wins over any older wording below)
- "Research writes" in the rows are calls to `psm_research::writes::{suggest, lower}`; the testkit gains a dependency on `psm-research` for that.
- The feedback table and its store methods exist since R2·U2; this unit uses them.
- When a resolved key is pending on a `held` publish item, U2 adds the release to the same transaction.

### Read first
`master_plan_research.md` §3, §6.3, §6.4, §6.6, §7.4 · book `14` (MA18), `08` (Values and provenance), `08A` §7,
`08B` **MC-A12** (all), MC-A1 (prior, conflict, cleared set), `05A` MC-30 and its research pointer (line 307) · code
`crates/core/src/services/attributes.rs` (`set_value`, `accept_all` line 511, `clear` line 546, `record`),
`crates/core/src/services/history.rs` (`FieldChange`, `FieldState`, the attribute arm at lines 715–758,
`Error::StepCannotApply`), `apps/desktop/src-tauri/src/commands/attributes.rs`,
`apps/desktop/src/features/form/{FormDialog.tsx (rawValue, "Looks right"), AiPopover.tsx}`,
`apps/desktop/src/features/form/__tests__/{p11_form_ui.test.tsx, p22_form_tidy.test.tsx}`.

### Files
Edited: `crates/core/src/services/{attributes.rs, history.rs}`, `apps/desktop/src-tauri/src/{commands/attributes.rs, lib.rs}` (commands `attribute_ai_accept`, `attribute_conflict_resolve`), `apps/desktop/src/services/{bindings.ts (regenerated), attributes.ts}`, `apps/desktop/src/features/form/FormDialog.tsx` (Looks right calls the accept command), `apps/desktop/src/features/form/__tests__/{p11_form_ui.test.tsx, p22_form_tidy.test.tsx}`, `crates/testkit/Cargo.toml`. New: `crates/testkit/tests/suite/research/r5_feedback_and_undo.rs` (+ its `mod` line).

### Core mechanisms (the only functions with unit tests)
None new (the comparison "current row equals the expected row on every column except `updated_at`" is a private
function of `history.rs`, proven through C3).

### Ordered tasks
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

### Criteria (5 criteria, 16 rows, 1 `[rule]`)
Boundary: the service methods on a `TestApp`; outcome = the stored row, the `research_feedback` rows, the history state.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| A08·C1(a) | `accept` | `ai` + `suggested` A, medium, prior D | row `ai` + `accepted` A, prior null; one feedback row `accepted`, suggested A, medium, final A, with the step's token | MA18 |
| A08·C1(b) | `accept_all` | three suggestions: medium, `low`, and a conflict at high | only the first is accepted; one feedback row | MA18 (R-IA1) |
| A08·C1(c) | `keep_previous` | `ai` + `suggested` A with prior `{D, valid, default}` | row `default` D, AI columns and prior null; feedback `kept_previous`, suggested A, final D; a later `writes::suggest` of A is refused as cleared | MA18 |
| A08·C1(d) | `clear` | `ai` + `suggested` A with a prior; `ai` + `accepted` B | both rows `cleared`, prior null; feedback `cleared`, final null, twice | MA18 |
| A08·C1(e) | `set_value` (human edit) | `ai` + `suggested` A with a prior; new value H | row `human` H, prior null; feedback `edited`, suggested A, final H | MA18 |
| A08·C1(f) | any of the actions | a `human` row edited; a `default` row cleared | no feedback row | MA18 |
| A08·C2(a) | `writes::suggest` | a research write | no feedback row; history state unchanged | MC-A12 |
| A08·C3(a) | undo | clear suggestion A (step); research then writes B; undo | nothing changes (row still `ai` + `suggested` B); the step is gone from the history; notice `that field was changed by research` | MA18 (R-IA21) |
| A08·C3(b) | redo | clear A; undo (row is `ai` + `suggested` A again); research replaces A with B; redo | nothing changes; the step is dropped; the same notice | MA18 |
| A08·C3(c) | undo, two fields | `fill_defaults` fills X and Y (one step, both rows `default`); research then writes a suggestion over Y; undo | neither X nor Y changes (X keeps its default, Y its suggestion) | MA18 |
| A08·C3(d) | undo, unchanged | clear A; undo | row is `ai` + `suggested` A again, equal to before on every column except `updated_at` | MC-A12 |
| A08·C4(a) | feedback and undo | clear A (feedback `cleared`); undo | the feedback row is gone; `writes::suggest` of A is written again | MA18 |
| A08·C4(b) | feedback and redo | C4(a), then redo | the feedback row exists again with the same token; A is refused as cleared | MA18 |
| A08·C4(c) | step limit | 26 accepts on 26 fields | 26 feedback rows remain although the first step fell off the history | MC-A12 |
| A08·C5(a) | commands | `attribute_ai_accept`, `attribute_conflict_resolve` with `keep_previous`, `use_suggestion`, `edit` + value | each produces exactly one undo step and the row of C1(a), C1(c), C1(a), C1(e) | MC-A12 |
| A08·C5(b) | form UI | Looks right on a suggestion (UI test) | the accept command is called with the item and key; `attribute_values_set` is not | `[rule]` 08B MC-A12 |

### Delegated choices
Method names beyond those listed · where the token is generated · the label text of the new undo steps (follow the
existing "Clear <label>" style).

### Notes
- **Behaviour change, by contract:** "Looks right" used to make the value human-owned
  (`master_plan_research.md` §7.4); it now accepts the suggestion. The two UI tests named above are updated to
  assert the new call.
- C3(c) uses `fill_defaults` (P67) because its after-rows are `default`, a state research may write over (MC-A1);
  an accepted or human row cannot be changed by research, so a step over such rows could never show the guard.

### Unit test commands (run after the unit, then make the `CHECKPOINT (not approved):` commit)
`cargo test -p psm-testkit --test suite research::r5_feedback_and_undo::` · `cargo test -p psm-testkit --test suite p39_undo_history_core::` · `cargo test -p psm-testkit --test suite p40_undo_history_values::` · `cargo test -p psm-testkit --test suite p10_attributes::` · `cd apps/desktop && npx vitest run src/features/form` · `npm run typecheck`

## Unit 2 — Withheld values, conflicts and the held item in a publish

**Unit id:** `R5·U2` · **Replaces:** A09, all rows except C5(a), C5(b) (those are in R2·U2) · **Changes app files:** yes

### Goal
At confirmation, split each item's values into frozen, withheld (`ai` + `suggested` + `low`, no prior) and pending
(conflicts); record the origin of each; refuse the confirmation when what the popover showed is no longer true;
start an item that has pending keys at the new step `held`, where it sends nothing; freeze each pending key at its
resolution and release the item at the last one; keep the other items of the confirmation running. Backend only.
**Not in this phase:** the popover and row-label UI (R6·U3); research runs (the conflicts in this phase's tests are
made with `writes::suggest`).

### What changed from the superseded plan (this wins over any older wording below)
- The four `publish_items` columns, `PublishStep::Held`, `RowLabel::WaitingForYou` and the status rule exist since R2·U2 (rows A09·C5(a), A09·C5(b) are there); tasks 1 and 2 of the older list are done. There is no numbered migration.
- `partition` and `is_conflict` are core functions here (`crates/core/src/mechanisms/publish_partition.rs`), built on `psm_core::mechanisms::sameness` (R2·U1): core's publish code cannot depend on the research crate. `psm_research::rules::write_table::is_conflict` (R3·U5) is the same rule; this unit makes it call the core function so there is one implementation.

### Read first
`master_plan.md` §3, §6.5–§6.7, §7.5 · `master_plan_research.md` §3, §6.3–§6.6, §7.4 · book `01` (H1, H4), `14` (M5,
M14, M17, MA8, MA17), `07` (publish steps), `07A` **MC-10** (frozen confirmation and its research pointer), MC-11
(resume, startup re-queue), MC-13 (never clears), `05A` MC-7 and its research pointer (line 205), `08A` §7,
`08B` **MC-A10** (all), MC-A12 (resolution in the action's transaction), MC-A1 (conflict) · code
`crates/core/src/services/publish.rs` (`frozen_attributes` 472, `preview` 518, `confirm` 592–720, `run_item` 758,
`cancel` 241, `retry`), `crates/core/src/domain/{publish.rs (PublishItem), session.rs (PublishStep, RowLabel)}`,
`crates/core/src/mechanisms/status.rs`, `crates/core/src/services/attributes.rs` (R5·U1's actions),
`apps/desktop/src-tauri/src/commands/publish.rs`, `crates/testkit/src/fake_commerce.rs`.

### Files
New: `crates/core/src/mechanisms/publish_partition.rs`, `crates/testkit/tests/suite/research/r5_publish_hold.rs` (+ `mod` line). Edited: `crates/core/src/mechanisms/mod.rs`, `crates/core/src/services/{publish.rs, attributes.rs, history.rs}`, `crates/core/src/ports/store.rs`, `crates/infra/src/sqlite/store.rs`, `apps/desktop/src-tauri/src/commands/publish.rs`, `apps/desktop/src/services/{bindings.ts, publish.ts}`, `apps/desktop/src/features/publish/PublishPopover.tsx` (pass what the preview returned; no new UI), `crates/research/src/rules/write_table.rs` (one line: delegate `is_conflict`).

### Core mechanisms (the only functions with unit tests)
`psm_core::mechanisms::publish_partition::{partition, is_conflict}`.

### Ordered tasks
1. (Done in R2·U2: the four columns.)
2. (Done in R2·U2: `PublishStep::Held`, `RowLabel::WaitingForYou(n)` and the status rule.) Here: `StatusInput`'s unresolved count is filled from `pending_conflicts` minus `resolved_attributes`.
3. `partition(values, definitions)`: frozen = live, valid rows, except withheld (`ai` + `suggested` + `low` with no
   prior) and pending (`is_conflict`); a row with a prior that is not a conflict is frozen at any confidence.
   `origin` holds provenance, status, confidence and prior for every frozen, withheld or pending key.
4. `preview` returns, per item, `n_withheld` and the conflicts (key, label, prior value and origin, suggestion,
   confidence). `confirm(session_ids, summary, shown)` recomputes both from the rows it reads **inside the
   confirming transaction**; a difference for any item refuses the whole confirmation with `AI suggestions changed
   while you were confirming` and writes nothing. Otherwise it writes `frozen_attributes`, `pending_conflicts`
   (also part of `confirmed_summary`) and `frozen_attribute_origin`; an item with pending keys starts at `Held` and
   **no job is queued for it**.
5. Resolution: R5·U1's actions (`accept`, `keep_previous`, `set_value`, `clear`), when the key is pending on a `held`
   item of that product, write the key's resolved canonical value, action and time into `resolved_attributes` in
   the same transaction (`clear` records "no value"). The transaction that resolves the last pending key sets the
   step to `Prepare`, records `released_at` and queues the job. Undoing a resolution while the item is still held
   removes the entry (the key is pending again).
6. `run_item` sends, for a released item, the frozen values plus each resolved key's value (a "no value" key is not
   sent). `cancel` on a held item sets `Cancelled` at once. The startup re-queue leaves held items held.

### Criteria (7 criteria, 18 rows)
Boundary: `PublishService::{preview, confirm, cancel}`, the attribute actions, `TestApp::tick`, and the mutations
`FakeCommerce` recorded. "Sends" means a recorded `attributes_write`.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| A09·C1(a) | `confirm` → `publish_items` | one item with rows: `human`, `default`, `integration`, `ai` + `accepted`, `ai` + `suggested` high, `ai` + `suggested` medium | all six in `frozen_attributes`; step `prepare` | MA17 |
| A09·C1(b) | same | plus `ai` + `suggested` `low` with no prior | absent from `frozen_attributes`; `n_withheld` = 1; origin recorded for it | MA8 |
| A09·C1(c) | same | plus a conflict at `low` and a conflict at high | both absent from `frozen_attributes`, both in `pending_conflicts` with prior value, origin, suggestion and confidence; neither counted as withheld | MA17 |
| A09·C1(d) | same | plus `ai` + `suggested` `low` whose prior is the same value under `S` | frozen | MC-A10 |
| A09·C1(e) | same | a cleared row and an invalid row | neither frozen (as before this phase) | M17 |
| A09·C2(a) | `confirm` re-check | preview shows `n_withheld` 1; a second low suggestion is written; confirm with the shown set | refused with `AI suggestions changed while you were confirming`; no `publish_runs` row | MA17 (R-IA22) |
| A09·C2(b) | same | preview shows conflict keys `{k1}`; `k1` is resolved before confirm | refused the same way; no run row | MA17 |
| A09·C3(a) | held item | two sessions confirmed together, the first with one conflict | first item step `held`, no job row for it, no mutation for it after `tick` until idle; the second item reaches `done` with its gallery and values written | MA17 |
| A09·C3(b) | H1/M5 | C3(a) | the run's `confirmed_at` precedes every recorded mutation | M17 |
| A09·C4(a) | resolution, two pending keys | resolve `k1` with Keep previous | `resolved_attributes` holds `k1` = the prior's value with action and time; step still `held`; no mutation | MA17 |
| A09·C4(b) | same | then research writes a new suggestion on `k1`; then resolve `k2` with Use suggestion | step `prepare`, `released_at` set, one job queued; after `tick` the shop holds `k1` = the prior's value and `k2` = the suggestion; the later suggestion on `k1` was not sent | MA17 |
| A09·C4(c) | same | a held item whose only pending key is resolved by Edit to H; by Clear | Edit: released, H sent; Clear: released, the key is not in any write | MA17, M20 |
| A09·C4(d) | undo | a held item with two pending keys: resolve `k1`, then undo | `k1` has no entry in `resolved_attributes`; the item is still `held` with two pending keys | MC-A10 |
| A09·C6(a) | `cancel` | a held item | step `cancelled` immediately; no mutation ever recorded; the conflicts are still conflicts in the form | MC-A10 |
| A09·C6(b) | restart | `kill_and_restart` with a held item | still `held`, no job row | MA17, H4 |
| A09·C6(c) | restart | resolve the last key, then `kill_and_restart` before any tick | step `prepare` with its job; the item then reaches `done` once (no duplicate product or media) | H4, M18 |
| A09·C7(a) | conflict after confirmation | an item confirmed with no conflict; research then creates a conflict on key k before the update step runs | the item is not held; k is sent with the value frozen at confirmation | MA17 |
| A09·C8(a) | MA8 evidence | C4(b)'s finished item | `frozen_attribute_origin` and `resolved_attributes` together account for every key sent; no key sent was `ai` + `suggested` + `low` without a prior at confirmation | MA8 |

### Delegated choices
JSON layouts of the three new columns (key → object) · how `shown` is passed (`[{session_id, n_withheld,
conflict_keys}]`) · the progress percentage of `held` (0) · where the release function lives (`PublishService`,
called from the attribute actions through a narrow trait so `attributes.rs` does not depend on the whole service).

### Notes
- Hazard: adding `Held` reaches every exhaustive `match` on `PublishStep` (`publish.rs` lines 114, 264–281,
  767–813; `status.rs`; the TypeScript unions). `Held` is never an "active" step for the startup re-queue.
- Hazard: `confirm` today builds the preview outside the transaction and re-checks photos inside it; the new
  re-check follows the same pattern and must read the attribute rows **inside** (`publish.rs` 640–648).
- The 07A MC-10 skip of photos deleted or moved since still applies when a released item runs (C4 keeps one such
  photo out if the implementer wants a combined row; not required).

### Unit test commands (run after the unit, then make the `CHECKPOINT (not approved):` commit)
`cargo test -p psm-testkit --test suite research::r5_publish_hold::` · `cargo test -p psm-testkit --test suite p13_publish_runs::` · `cargo test -p psm-testkit --test suite p14_publish_update::` · `cargo test -p psm-testkit --test suite p64_publish_speed_and_visibility::` · `cargo test -p psm-research --lib rules::write_table` · `cd apps/desktop && npx vitest run src/features/publish` · `npm run typecheck`

## Closing gate (once, after the last unit; replaces the template's closing gate)
This milestone changed app files, so the full gate runs once: `cargo test --workspace` · `cargo clippy --workspace --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cd apps/desktop && npm run typecheck` · `npm test`. One cargo command at a time.

## Review mode
The reviewer runs: `cargo test -p psm-testkit --test suite research::r5_feedback_and_undo::`, `research::r5_publish_hold::`, `p39_undo_history_core::`, `p40_undo_history_values::`, `p10_attributes::`, `p13_publish_runs::`, `p14_publish_update::`, `p64_publish_speed_and_visibility::`, `p03_session_state_list::`, `npx vitest run src/features/form src/features/publish`, clippy, fmt, typecheck; the implementer's full-suite stamp stands. Notes: `docs/implementation/backlog_research/R5_research_app_seams.md`.

## Manual try-out (the narrative is built from this)
On scratch data with the test shop connected: Undo and Redo of form edits behave as before; publishing an item without suggestions works exactly as before. The new rules are visible in `cargo test -p psm-testkit --test suite research::r5_publish_hold::` until the buttons arrive in R6.

## Review log
