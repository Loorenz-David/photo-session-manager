---
phase: R5_research_app_seams
title: Research in the form and in publishing — what you do with a suggestion is remembered and Undo never erases a newer suggestion; in a publish weak guesses are left out, and an item whose suggestion replaced a preset or Shopify value waits for your decision while the others go ahead
difficulty: high
projection: no
depends_on: [R3_research_rules]
---
# R5 — App seams: the photographer's actions on suggestions, and publishing with unchecked suggestions

**Tracker:** APPROVED · 2026-11-12 · opus · review r1 APPROVED_WITH_NOTES: 42/42 rows hold at their boundaries, no B1–B4; r5 suite 40, neighbours 167, desktop `r5_` 2, vitest 268, clippy, fmt, typecheck green; 3 notes in `backlog_research/R5_research_app_seams.md`.

(This line is the milestone's tracker row. Sessions change only this line of this file: state, date, actor, one-line note. The gate for `depends_on` is the Tracker line of each plan named there. `master_plan.md` §4 is not used for research milestones.)

## Goal
The two places where existing app behaviour changes by contract: the attribute actions with their feedback rows and guarded undo (MC-A12), and the publish confirmation with withheld values, conflicts and the `held` step (MC-A10). Suggestions and conflicts in these tests are made with `psm_research::writes` (R3·U5). **Not here:** UI (R6), runs (R4), wiring (R7).

## How to work this plan (flow.md §10)
This milestone is one runner phase made of 2 units. Work the units in order. After each unit run its **Unit test commands**, then make a `CHECKPOINT (not approved):` commit of that unit's paths. One review, one fix budget and one narrative cover the whole milestone. Row ids keep the number of the superseded plan they were written in (`A07·C1(a)`); `docs/implementation/research_row_mapping.md` says where every row went. Name each test after its row (`a07_c1a_<slug>`). Reviewer notes go to `docs/implementation/backlog_research/R5_research_app_seams.md`.

| Unit | What it builds | Criteria | Rows | Changes app files |
|---|---|---|---|---|
| U1 | Feedback rows, conflict actions, guarded undo and redo | 5 | 16 | yes |
| U2 | Withheld values, conflicts and the held item in a publish | 7 | 26 | yes |
| **Total** | | **12** | **42** | |

## Read first (whole milestone)
`docs/process/flow.md` (**§10 is the authority for this track**) · `docs/implementation/master_plan.md` §3, §6, §7 · `docs/implementation/master_plan_research.md` (all: layout §0, standing rules §3, sequencing and tracks §5, registry §6, commands §7.2, the storage decision §6.4) · book `01`, `14` §Research agent, `08A`, `08B` · then each unit's own Read first. Name key and sameness are `psm_core::mechanisms::{name_key, sameness}` (R2·U1); every other research rule is `psm_research::rules`.

## Carried from earlier reviews (orchestrator, 2026-11-12; wins over older wording below)
- **U2, one conflict count (R2 review).** The session list (`store.rs`, pending keys without an entry in `resolved_attributes`) and the publish activity text (`services/publish.rs`, `PublishStep::Held`: `pending_conflicts.len() - resolved_attributes.len()`) derive the held count differently. Use one helper, MC-A10's "n = pending keys without a resolution", in both places (`store.rs` session list ~1704, `publish.rs::activity` ~276). No separate row: a resolved key that is not pending cannot be produced by any action (projection L9).

## Amendments after projection r0 (orchestrator, 2026-11-12; ledger L1–L15 of `handoff_R5_research_app_seams_projection_r0.md`)
The rows and tasks below are already amended in place; this list says where. Line numbers in the Read first lists
drifted by about 5 lines (projection, reality checks); find the functions by name.
- L1 → A08·C4(a) outcome. L2 → U2 task 5 and Delegated choices. L3, L4 → U2 task 5, rows A09·C4(e), A09·C4(f); book
  08B MC-A10/MC-A12 (changelog 6). L5 → U1 Files, A08·C5(a), Unit test commands, Review mode. L6 → U1 Files and task 0.
  L7 → U2 Files and task 4. L8 → A09·C1(d). L9 → the carried note above. L10 → A09·C3(a). L11 → A09·C3(b).
  L12 → A09·C7(a). L13, L14, L15 → U1 Delegated choices (decided, not free).
- Projection attempt 2 (N1–N9, same handoff path, archived attempt 1 under `docs/implementation/archive/`): N1, N3, N4,
  N7 → U2 task 5 and rows A09·C4(g)–(i). N2 → U2 task 6 and A09·C4(b). N5 → A09·C3(a). N6 → U1 Files (helper note).
  N8, N9 → U2 Decided by the orchestrator. N1 and N8 are also folded into R6·U3.
- Projection attempt 3 (P1–P6): P1 → U1 Files, task 6, A08·C5(b), R6·U3 note. P2 → U2 task 6, A09·C6(a). P3 → U2 task 4,
  A09·C4(b). P4 → U2 Files. P5 → U2 task 4, A09·C2(a), R6·U3 note. P6 → U1 Decided by the orchestrator.
- Projection attempt 4 (Q1–Q4): Q1 → U2 Files, A09·C2(a). Q2 → U1 Decided by the orchestrator. Q3, Q4 → U2 task 3,
  row A09·C1(f); book 08B MC-A10 (changelog 8).
- Projection attempt 5 (S1–S4): S1 → U2 task 3, row A09·C1(g). S2 → U2 task 4, A09·C2(a). S3 → U1 task 4, A08·C1(f);
  08B MC-A12 (changelog 9). S4 → U2 task 3; 08B MC-A10 (changelog 9); R6·U3 note.
- Projection attempt 7 (T1–T5, F1): T1 → U1 Q2 bullet. T2 → U1 P6 bullet, A08·C4(b). T3 → U2 task 5, row A09·C4(j);
  08B changelog 10. T4 → U1 Files. T5 → U2 Files. F1 → U2 Delegated choices.
- **Projection is off from attempt 8** (orchestrator, 2026-11-12): seven rounds, 0 owner cards, each round smaller;
  the review is the gate from here (frontmatter `projection: no`; master plan amendment v4).

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
Edited: `crates/core/src/services/{attributes.rs, history.rs}`, `apps/desktop/src-tauri/src/{commands/attributes.rs, lib.rs}` (commands `attribute_ai_accept`, `attribute_conflict_resolve`), `apps/desktop/src/services/{bindings.ts (regenerated), attributes.ts}`, `apps/desktop/src/features/form/FormDialog.tsx` (Looks right calls the accept command), `apps/desktop/src/features/form/AiPopover.tsx` (Looks right only on `ai` + `suggested`; P1), `apps/desktop/src/features/form/__tests__/{p11_form_ui.test.tsx, p22_form_tidy.test.tsx}`, `crates/testkit/Cargo.toml`, `crates/testkit/src/test_app.rs` (a helper that makes a conflict through `psm_research::writes::suggest`, used by U1, U2 and the desktop command test; the helper imports the definition, and the C5(a) test configures settings so `integration.kind` is that integration before `AppState::new`, else the command path sees only the built-in Product fields and answers "unknown attribute" (projection N6)), `crates/core/src/mechanisms/mod.rs`, `crates/core/src/ports/store.rs` and `crates/infra/src/sqlite/store.rs` (feedback rows by step token; update `final_value`; T4). New: `crates/core/src/mechanisms/publish_partition.rs` with **`is_conflict` only** (U2 adds `partition`; projection L6), `crates/testkit/tests/suite/research/r5_feedback_and_undo.rs` (+ its `mod` line). A08·C5(a)'s test is a `#[cfg(test)]` module in `apps/desktop/src-tauri/src/lib.rs` through the Tauri mock runtime (precedent: the R2 research command tests in that file); the testkit cannot call desktop commands (projection L5).

### Core mechanisms (the only functions with unit tests)
None new (the comparison "current row equals the expected row on every column except `updated_at`" is a private
function of `history.rs`, proven through C3).

### Ordered tasks
0. `psm_core::mechanisms::publish_partition::is_conflict` (built on `sameness`, R2·U1) and its `mod.rs` line; tasks 2–3
   and `accept_all` use it. U2 adds `partition` next to it.
1. `AttributesService::accept(item_id, key)` (Looks right, and Use suggestion on a conflict): `ai` + `suggested` →
   `ai` + `accepted`, `ai_prior` dropped; one undo step.
2. `AttributesService::keep_previous(item_id, key)`: the row becomes the prior's value, validity and provenance, the
   AI columns and `ai_prior` null; one undo step.
3. `clear` and a human edit (`set_value`) drop `ai_prior`. `accept_all` skips `low` (as today) **and every conflict**.
4. Each of these, when the row before the action had provenance `ai` **and status `suggested` or `accepted`** (not
   `cleared`: its verdict was recorded by the clear; S3, 08B changelog 9), inserts one feedback row in the same
   transaction: the action, the suggested value and confidence from before, the final value (null for a clear; the
   prior's value for keep previous; the new canonical value for an edit), the time, and `step_token` = the history
   step's id.
5. History: an attribute step applies only if, for **every** field of the step, the current row equals the expected
   row (the step's after-row for undo, its before-row for redo) on every column except `updated_at`. Otherwise the
   whole step is not applied, it is dropped, and the notice reads `that field was changed by research`. Undoing a
   step deletes the feedback rows with its token; redoing re-inserts them. Steps that fall off the 25-step limit or
   are cleared at quit leave their rows.
6. Commands `attribute_ai_accept(item_id, key)` and `attribute_conflict_resolve(item_id, key, action, value?)`
   (thin); the form's Looks right calls the first. The popover offers **Looks right** only when
   `ai_status === 'suggested'`; on an `ai` + `accepted` row it offers Clear only, so an accepted suggestion never
   answers "Could not accept: invalid_value" (P1; MC-A12 lists Looks right only from `ai` + `suggested`).

### Criteria (5 criteria, 16 rows, 1 `[rule]`)
Boundary: the service methods on a `TestApp`; outcome = the stored row, the `research_feedback` rows, the history state.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| A08·C1(a) | `accept` | `ai` + `suggested` A, medium, prior D | row `ai` + `accepted` A, prior null; one feedback row `accepted`, suggested A, medium, final A, with the step's token | MA18 |
| A08·C1(b) | `accept_all` | three suggestions: medium, `low`, and a conflict at high | only the first is accepted; one feedback row | MA18 (R-IA1) |
| A08·C1(c) | `keep_previous` | `ai` + `suggested` A with prior `{D, valid, default}` | row `default` D, AI columns and prior null; feedback `kept_previous`, suggested A, final D; a later `writes::suggest` of A is refused as cleared | MA18 |
| A08·C1(d) | `clear` | `ai` + `suggested` A with a prior; `ai` + `accepted` B | both rows `cleared`, prior null; feedback `cleared`, final null, twice | MA18 |
| A08·C1(e) | `set_value` (human edit) | `ai` + `suggested` A with a prior; new value H | row `human` H, prior null; feedback `edited`, suggested A, final H | MA18 |
| A08·C1(f) | any of the actions | a `human` row edited; a `default` row cleared; an `ai` + `cleared` row edited (human H) | no feedback row | MA18 (changelog 9) |
| A08·C2(a) | `writes::suggest` | a research write | no feedback row; history state unchanged | MC-A12 |
| A08·C3(a) | undo | clear suggestion A (step); research then writes B; undo | nothing changes (row still `ai` + `suggested` B); the step is gone from the history; notice `that field was changed by research` | MA18 (R-IA21) |
| A08·C3(b) | redo | clear A; undo (row is `ai` + `suggested` A again); research replaces A with B; redo | nothing changes; the step is dropped; the same notice | MA18 |
| A08·C3(c) | undo, two fields | `fill_defaults` fills X and Y (one step, both rows `default`); research then writes a suggestion over Y; undo | neither X nor Y changes (X keeps its default, Y its suggestion) | MA18 |
| A08·C3(d) | undo, unchanged | clear A; undo | row is `ai` + `suggested` A again, equal to before on every column except `updated_at` | MC-A12 |
| A08·C4(a) | feedback and undo | clear A (feedback `cleared`); undo | the feedback row is gone; `writes::suggest` of A with the original confidence, reason and sources returns `Unchanged` (not `MatchesCleared`), and the row is untouched | MA18 |
| A08·C4(b) | feedback and redo | C4(a), then redo | the feedback row exists again with the same token (this clause carries the row; the stored row is itself `ai` + `cleared` A again); the same holds when, between the undo and the redo, a clear in another session wrote a feedback row | MA18 |
| A08·C4(c) | step limit | 26 accepts on 26 fields | 26 feedback rows remain although the first step fell off the history | MC-A12 |
| A08·C5(a) | commands (`#[cfg(test)]` module in `apps/desktop/src-tauri/src/lib.rs`; conflict fixture from the testkit helper) | `attribute_ai_accept`, `attribute_conflict_resolve` with `keep_previous`, `use_suggestion`, `edit` + value | each produces exactly one undo step and the row of C1(a), C1(c), C1(a), C1(e) | MC-A12 |
| A08·C5(b) | form UI | Looks right on a suggestion (UI test) | the accept command is called with the item and key; `attribute_values_set` is not; on an `ai` + `accepted` row the popover offers no Looks right, and Clear stays | `[rule]` 08B MC-A12 |

### Delegated choices
Method names beyond those listed · where the token is generated · the label text of the new undo steps (follow the
existing "Clear <label>" style).

### Decided by the orchestrator (projection L13–L15)
- **Actions on a row in the wrong state** (`accept` on a row that is not `ai` + `suggested`; `keep_previous` with no
  `ai_prior` or not a conflict; `attribute_conflict_resolve(edit)` without a value, or a value with another action):
  refused with `Error::InvalidValue`; nothing written (no step, no feedback row). `value` is JSON text as in
  `attribute_values_set`; `action` is a snake_case enum `keep_previous | use_suggestion | edit`.
- **The guard and titles:** the guard compares `FieldState::Row` fields only; title fields restore as today
  (`p40` `c4d_title_undo_redo` unchanged).
- **Coalesced typing:** when an edit coalesces into a step whose token owns an `edited` feedback row, that row's
  `final_value` is updated to the new canonical value in the same transaction, so the feedback never holds a typing
  prefix. No row.
- **What redo re-inserts (P6):** undo reads the feedback rows carrying the step's token before deleting them and
  keeps them on the step; redo re-inserts exactly those (so a coalesced `final_value` is not lost), with
  `insert_feedback` (a fresh id; the old id is not kept: `research_feedback.id` has no AUTOINCREMENT, so an id can be
  reused by another session's row and a re-insert with the old id would fail every later redo; T2).
- **Where coalescing is decided (Q2):** once, inside the action's transaction: the service reads the session's last
  undo step through `History` (the predicate of `coalesce_attribute` plus U2 task 5's "never coalesce into a step
  that recorded a resolution"), updates the feedback row when it merges, and hands that same decision to
  `History::record`, which does not re-decide. **Lock order (T1):** the action takes `History.operation` **before** it
  opens its store transaction and holds it until the step is recorded, as `retire_photo` and `reverse` do (order:
  operation → store). `History` gains a record variant that runs under that held guard. Nothing acquires
  `operation` inside a store transaction (the store transaction holds the connection mutex; the reverse order
  deadlocks against a concurrent undo).

### Notes
- **Behaviour change, by contract:** "Looks right" used to make the value human-owned
  (`master_plan_research.md` §7.4); it now accepts the suggestion. The two UI tests named above are updated to
  assert the new call.
- C3(c) uses `fill_defaults` (P67) because its after-rows are `default`, a state research may write over (MC-A1);
  an accepted or human row cannot be changed by research, so a step over such rows could never show the guard.

### Unit test commands (run after the unit, then make the `CHECKPOINT (not approved):` commit)
`cargo test -p psm-testkit --test suite research::r5_feedback_and_undo::` · `cargo test -p psm-testkit --test suite p39_undo_history_core::` · `cargo test -p psm-testkit --test suite p40_undo_history_values::` · `cargo test -p psm-testkit --test suite p10_attributes::` · `cargo test -p psm-desktop r5_` (the C5(a) module; name it `r5_…`) · `cd apps/desktop && npx vitest run src/features/form` · `npm run typecheck`. Bindings regenerate with `cargo test -p psm-desktop export_bindings` (master_plan §7).

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
New: `crates/testkit/tests/suite/research/r5_publish_hold.rs` (+ `mod` line). Edited: `crates/core/src/mechanisms/publish_partition.rs` (created by U1; U2 adds `partition`), `apps/desktop/src-tauri/src/lib.rs` (the `r5_` module gains the A09·C2(a) code assertion), `crates/core/src/mechanisms/mod.rs`, `crates/core/src/services/{publish.rs, attributes.rs, history.rs}`, `crates/core/src/ports/store.rs`, `crates/core/src/domain/publish.rs` (`PublishPreviewItem` gains `n_withheld`, `conflicts`; `PublishItem` is the home of the one "pending keys without a resolution" helper; P4), `crates/core/src/error.rs` (P5), `apps/desktop/src-tauri/src/commands/items.rs` (one match arm: `Error::SuggestionsChanged` → code `suggestions_changed`; Q1), `crates/testkit/src/test_app.rs` (`confirm_as_shown`; P4), `crates/infra/src/sqlite/store.rs`, `apps/desktop/src-tauri/src/commands/publish.rs`, `apps/desktop/src/services/{bindings.ts, publish.ts}`, `apps/desktop/src/features/publish/PublishPopover.tsx` (pass what the preview returned; no new UI), `crates/research/src/rules/write_table.rs` (one line: delegate `is_conflict`). **Mechanical, pass the shown set** (projection L7): the 42 `confirm` call sites in the testkit files `p13`, `p14`, `p15`, `p16`, `p17`, `p21`, `p25`, `p30`, `p59`, `p61`, `p62`, `p64`, `p65`, `p66`, `p69` (`crates/testkit/tests/suite/`), through one helper `TestApp::confirm_as_shown(ids, summary)` that takes the shown set from a fresh `preview`; `apps/desktop/src/features/publish/__tests__/p15_publish_ui.test.tsx` (the `publishConfirm` call assertion gains the shown set; its preview literal gains `n_withheld`, `conflicts`) and `apps/desktop/src/features/compose/__tests__/p32_reframe.test.tsx` (preview literal).

### Core mechanisms (the only functions with unit tests)
`psm_core::mechanisms::publish_partition::{partition, is_conflict}`.

### Ordered tasks
1. (Done in R2·U2: the four columns.)
2. (Done in R2·U2: `PublishStep::Held`, `RowLabel::WaitingForYou(n)` and the status rule.) Here: `StatusInput`'s unresolved count is filled from `pending_conflicts` minus `resolved_attributes`.
3. `partition(values, definitions)`: frozen = live, valid rows, except withheld (`ai` + `suggested` + `low` with no
   prior) and pending (`is_conflict`); a row with a prior that is not a conflict is frozen at any confidence.
   `origin` holds provenance, status, confidence and prior for every frozen, withheld or pending key.
   `partition` also takes the item's current layout. An `ai` + `suggested` row with an `ai_prior` whose key has **no
   active definition** is left out of the run like an out-of-layout conflict (not frozen, not pending, not
   withheld; no definition means no type for `S`, so every prior-carrying suggestion is left out, fail closed); every
   other row with no definition is frozen or withheld as today, and `write_set` never sends it (Q3, S1). The
   definitions are `attribute_definitions("shopify")` as `preview` and `update` use them today (research never
   writes a built-in Product field). A conflict whose key
   is **not in the item's current layout** is not pending either: it is left out of the run (not frozen, not held,
   not counted as withheld); the shop keeps its value (07A MC-13), a new product gets none for that key (Q4; 08B
   MC-A10, changelog 8). Otherwise an item could wait on a field the form cannot show.
   **Pending is decided at confirmation.** A later layout or definition change (Customize fields, an item-type
   change, a schema refresh) neither releases the item nor drops a pending key; Cancel and publish again is the way
   out. Add no re-evaluation hooks to `presets_set`, `set_type` or `refresh_schema` (S4; 08B changelog 9).
4. `preview` returns, per item, `n_withheld` and the conflicts (key, label, prior value and origin, suggestion,
   confidence). `confirm(session_ids, summary, shown)` recomputes both from the rows it reads **inside the
   confirming transaction**; `shown` is required, never an `Option` that skips the re-check (fail-open on H1, R-IA22); a session in
   `session_ids` with no entry in `shown` is compared as `{n_withheld: 0, conflict_keys: ∅}`, never skipped (S2); a difference for any item refuses the whole confirmation with `AI suggestions changed
   while you were confirming` and writes nothing. Otherwise it writes `frozen_attributes`, `pending_conflicts`
   (also part of `confirmed_summary`) and `frozen_attribute_origin`; an item with pending keys starts at `Held` and
   **no job is queued for it**. Only the enqueue is skipped: its stock choice is frozen at confirmation like any
   other item's (`set_publish_item_stock` in the same loop; 07A MC-10, changelog 45), so the released new product is
   created with the stock the photographer chose (P3). The refusal is a dedicated variant
   `Error::SuggestionsChanged` (`#[error("AI suggestions changed while you were confirming")]`, command code
   `suggestions_changed`), so R6·U3 can rebuild the preview on that code alone (P5).
5. Resolution: R5·U1's actions (`accept`, `keep_previous`, `set_value`, `clear`), when the key is pending on a `held`
   item of that product, write the key's resolved canonical value, action and time into `resolved_attributes` in
   the same transaction (`clear` records "no value"). The transaction that resolves the last pending key sets the
   step to `Prepare`, records `released_at` and queues the job. **The hold step is a core function over the
   transaction** (e.g. `services::publish::resolve_pending_in(tx, item_id, key, resolution, step_token, now)`), called
   **unconditionally** by `accept`, `keep_previous`, `set_value` (human) and `clear` inside their own transaction; no
   injected collaborator (projection L2: an optional hook could be left unwired on the command path and a held item
   would never release). Queueing inside the transaction needs a `StoreTx` enqueue method (today
   `enqueue_publish_item` is only on `Store`, `ports/store.rs` ~294).
   A resolution entry stores the resolving step's token. An action on a key that already has an entry changes nothing
   in `resolved_attributes` (MC-A10: later writes are not in this run). Undo removes only the entries carrying the
   undone step's token, while the item is still held (projection L3). Redoing a resolution while the item is still
   `held` records it again with the step's token, and releases the item when it was the last pending key (08B
   MC-A10/MC-A12, changelog 6; projection L4). The attribute step keeps what it needs to re-record its resolutions
   (the entries it wrote: value and action), as it keeps its feedback rows for re-insert (N7).
   **A redo that changes a key pending without an entry on a `held` item is that key's resolution** (T3; 08B
   changelog 10), recorded with the redo's step token as the action it replays (Clear → no value; Edit → the value,
   no value when it does not fit; Keep previous → the prior's value; Use suggestion / Accept all → the suggestion),
   through the same `resolve_pending_in` call; it releases the item when it was the last pending key. The step's
   kept feedback row gives the action and final value.
   **Callers include `accept_all`** for each key it accepts, in its transaction: a pending key that research turned
   into a non-conflict is accepted by Accept all and must resolve (N3).
   **A step that recorded a resolution is never coalesced into** (`history.rs` `coalesce_attribute`): a later edit of
   the key is a new step and changes nothing in the entry (MC-A10). This keeps the history, the feedback row and the
   entry in agreement; the L15 coalescing rule applies only to steps that resolved nothing (N1). The form side
   (Edit on a conflict saves once, on explicit commit, never on the 300 ms debounce) is R6·U3's.
   **A resolution whose row after the action is not live and valid records "no value"** (never sent, like Clear):
   Keep previous of an invalid `integration` prior, or an Edit that does not fit its definition (N4; the frozen rule
   of MC-A10 and 07A MC-13).
6. The values a released item uses are `frozen_attributes` plus each resolved key's value, "no value" keys left out,
   in **every** run-path read of the frozen list: `ensure_export`'s `run_staged` (the staged `item.json` that becomes
   the local export at `publish_swap`) and `update`'s `write_set` (N2). Merging at release or at read time is the
   implementer's choice. `cancel` on a held item sets `Cancelled` at once, and `activity` of a `held` item reports `can_cancel: true` (no
   retry), so the progress view offers Cancel for it (P2: today `active()` excludes `Held`, so `can_cancel` is false). The startup re-queue leaves held items held.

### Criteria (7 criteria, 26 rows)
Boundary: `PublishService::{preview, confirm, cancel}`, the attribute actions, `TestApp::tick`, and the mutations
`FakeCommerce` recorded. "Sends" means a recorded `attributes_write`.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| A09·C1(a) | `confirm` → `publish_items` | one item with rows: `human`, `default`, `integration`, `ai` + `accepted`, `ai` + `suggested` high, `ai` + `suggested` medium | all six in `frozen_attributes`; step `prepare` | MA17 |
| A09·C1(b) | same | plus `ai` + `suggested` `low` with no prior | absent from `frozen_attributes`; `n_withheld` = 1; origin recorded for it | MA8 |
| A09·C1(c) | same | plus a conflict at `low` and a conflict at high | both absent from `frozen_attributes`, both in `pending_conflicts` with prior value, origin, suggestion and confidence; neither counted as withheld | MA17 |
| A09·C1(d) | same | plus `ai` + `suggested` `low` whose prior is the same value under `S`, made by research replacing its own suggestion: `writes::suggest` B over default D (prior D), then `writes::suggest` d (S-equal to D) at `low` (the prior is kept; not a conflict). A direct write S-equal to a live default returns `AlreadyHolds` and cannot make it | frozen | MC-A10 |
| A09·C1(e) | same | a cleared row and an invalid row | neither frozen (as before this phase) | M17 |
| A09·C1(f) | same | a conflict on a key that research wrote while it was in the layout, then the layout changed (Customize fields) so the key is no longer in it | absent from `frozen_attributes` and `pending_conflicts`, not counted in `n_withheld`; step `prepare` | MC-A10 (changelog 8) |
| A09·C1(g) | same | a conflict on `k`, then `refresh_schema` with `FakeShop.definitions` minus `k`; confirm | `k` absent from `frozen_attributes` and `pending_conflicts`, not in the staged `item.json`; step `prepare` | MC-A10 (changelog 8) |
| A09·C2(a) | `confirm` re-check | preview shows `n_withheld` 1; a second low suggestion is written; confirm with the shown set; and separately, confirm with an empty `shown` for an item with `n_withheld` 1 | refused with `Error::SuggestionsChanged` (`AI suggestions changed while you were confirming`); no `publish_runs` row; and, in the desktop `r5_` test module, `CommandError::from(Error::SuggestionsChanged).code == "suggestions_changed"` (the string R6·U3 keys on) | MA17 (R-IA22) |
| A09·C2(b) | same | preview shows conflict keys `{k1}`; `k1` is resolved before confirm | refused the same way; no run row | MA17 |
| A09·C3(a) | held item | two sessions confirmed together, the first with one conflict | first item step `held`, no job row for it; after `tick` until idle `FakeShop.products` holds exactly one product, with the second item's SKU, and `FakeShop.values` has entries only under that product's id; the second item reaches `done` with its gallery and values written | MA17 |
| A09·C3(b) | H1/M5 | C3(a) | the run's `confirmed_at` ≤ every recorded mutation's `at` (the `p13` form) | M17 |
| A09·C4(a) | resolution, two pending keys | resolve `k1` with Keep previous | `resolved_attributes` holds `k1` = the prior's value with action and time; step still `held`; no mutation | MA17 |
| A09·C4(b) | same | then research writes a new suggestion on `k1`; then resolve `k2` with Use suggestion | step `prepare`, `released_at` set, one job queued; after `tick` the shop holds `k1` = the prior's value and `k2` = the suggestion; the later suggestion on `k1` was not sent; the item's local export `item.json` holds `k1` = the prior's value and `k2` = the suggestion; the item is a new product whose form chose a stock, and the released product is created with that location and quantity (`FakeShop.stock`, `stock_requests`) | MA17 |
| A09·C4(c) | same | a held item whose only pending key is resolved by Edit to H; by Clear | Edit: released, H sent; Clear: released, the key is not in any write | MA17, M20 |
| A09·C4(d) | undo | a held item with two pending keys: resolve `k1`, then undo | `k1` has no entry in `resolved_attributes`; the item is still `held` with two pending keys | MC-A10 |
| A09·C4(e) | later action, undo | a held item with two pending keys: resolve `k1` with Keep previous; edit `k1` again (human); undo once | `k1`'s entry is unchanged by the edit and by the undo (same value, action and token); the item still waits for `k2` only | MC-A10 |
| A09·C4(f) | redo | a held item with two pending keys: resolve `k1`, undo, redo; then resolve `k2` | after the redo `k1`'s entry is present with the step's token and the item is still `held`; after resolving `k2` the step is `prepare` with one job | MC-A10 (changelog 6) |
| A09·C4(g) | typing on a held item | a held item with one pending key `k`; `set_value` "Wa", then `set_value` "Walnut" (as the form's debounce would) | two undo steps; the entry for `k` is "Wa" with the first step's token; the item was released by the first call (pins N1; R6·U3's commit rule keeps a prefix from being the real value) | MC-A10 |
| A09·C4(j) | redo becomes a resolution | conflict on `k`; Clear; undo; confirm (held, one pending key); redo | `k` has an entry "no value" with the redo step's token; step `prepare` with one job; after `tick` `k` is in no write | MC-A10 (changelog 10) |
| A09·C4(h) | Accept all | a held item with two pending keys; research turns `k1` into a non-conflict at medium (the C1(d) path); `accept_all` | `k1` has an entry with action `accepted`; the item still waits for `k2` only | MC-A10, MC-A12 |
| A09·C4(i) | invalid prior | a linked item whose shop value for `k` does not fit its definition (`read_integration_values`), then `writes::suggest` over it; confirm (held); Keep previous | entry "no value"; the item is released; `k` is in no write | MC-A10, 07A MC-13 |
| A09·C6(a) | `cancel` | a held item | before the cancel, `activity` reports step `held` and `can_cancel` true; then step `cancelled` immediately; no mutation ever recorded; the conflicts are still conflicts in the form | MC-A10 |
| A09·C6(b) | restart | `kill_and_restart` with a held item | still `held`, no job row | MA17, H4 |
| A09·C6(c) | restart | resolve the last key, then `kill_and_restart` before any tick | step `prepare` with its job; the item then reaches `done` once (no duplicate product or media) | H4, M18 |
| A09·C7(a) | conflict after confirmation | an item that is a new product (the shop does not hold D for k) confirmed with no conflict; research then creates a conflict on key k before the update step runs | the item is not held; k is sent with the value frozen at confirmation | MA17 |
| A09·C8(a) | MA8 evidence | C4(b)'s finished item | `frozen_attribute_origin` and `resolved_attributes` together account for every key sent; no key sent was `ai` + `suggested` + `low` without a prior at confirmation | MA8 |

### Delegated choices
JSON layouts of the three new columns (key → object) · how `shown` is passed (`[{session_id, n_withheld,
conflict_keys}]`) · the progress percentage of `held` (0) · the name of the core hold function (it is a function over the transaction, called unconditionally; task 5) ·
which held item a resolution acts on when a session has more than one: the session's latest publish item, as the
row label reads it (F1).

### Decided by the orchestrator (projection N8, N9)
- In R5 the `pending_conflicts` column is the record of what waited; `publish_summary` (the confirmed text) is
  unchanged. R6·U3 adds the withheld and waiting lines to the text when the popover shows them.
- `preview`'s `details_filled` is unchanged in R5 (withheld and pending rows still count as filled).

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
The reviewer runs: `cargo test -p psm-testkit --test suite research::r5_feedback_and_undo::`, `research::r5_publish_hold::`, `p39_undo_history_core::`, `p40_undo_history_values::`, `p10_attributes::`, `p13_publish_runs::`, `p14_publish_update::`, `p64_publish_speed_and_visibility::`, `p03_session_state_list::`, `cargo test -p psm-desktop r5_`, `npx vitest run src/features/form src/features/publish src/features/compose`, clippy, fmt, typecheck; the implementer's full-suite stamp stands. Notes: `docs/implementation/backlog_research/R5_research_app_seams.md`.

## Manual try-out (the narrative is built from this)
On scratch data with the test shop connected: Undo and Redo of form edits behave as before; publishing an item without suggestions works exactly as before. The new rules are visible in `cargo test -p psm-testkit --test suite research::r5_publish_hold::` until the buttons arrive in R6.

## Review log

### 2026-11-12 · implement r1 · opus
**Built.** U1 (`c8b22ae`): `AttributesService::{accept, keep_previous}`; every photographer action runs through one `act` helper that takes `History`'s operation lock before its store transaction and records its step under that held guard (`History::{lock, last_step, record_held}`; operation → store); each action on an `ai` + `suggested`/`accepted` row writes one `research_feedback` row with the step's token (a UUID v4 made per action); `clear` and a human edit drop `ai_prior`; `accept_all` skips `low` and every conflict (`mechanisms::publish_partition::is_conflict`); attribute undo/redo apply only when every row field still equals the expected row (except `updated_at`), else `StepCannotApply` "that field was changed by research" and the step is dropped; undo reads and deletes its feedback rows and keeps them on the step, redo re-inserts them with fresh ids; a coalesced edit updates its step's `edited` row's `final_value`; store `feedback_by_step`, `update_feedback_final_value`; commands `attribute_ai_accept`, `attribute_conflict_resolve` (`keep_previous | use_suggestion | edit`); Looks right calls the accept command and shows only on `suggested`. U2 (`1e56542`): `publish_partition::partition` (frozen / withheld / pending / left out; origin per key) and `services::attributes::layout_keys`; `preview` returns `n_withheld` and `conflicts`; `confirm(ids, summary, shown)` recomputes inside its transaction and refuses with `Error::SuggestionsChanged` (code `suggestions_changed`); pending items start `held`, stock frozen, no job; `services::publish::{resolve_pending_in, unresolve_in}` called by every action (via `note_action`) and by undo/redo (redo of a key pending without an entry is its resolution); release in the same transaction (`StoreTx::release_publish_item` queues the job); cancel of a held item is immediate; `activity` offers Cancel on `held`; the run reads `PublishItem::run_attributes()` (frozen + resolved, "no value" left out) in `ensure_export` and `update`; one count helper `PublishItem::unresolved_conflicts` for the row label and the progress text; research's `is_conflict` delegates to core; the 42 testkit `confirm` calls go through `ConfirmAsShown::confirm_as_shown`.

**Judgment calls** (full list with reasons in `handoffs/implementer/handoff_R5_research_app_seams_implement_r1.md` §Assumptions). First to look at:
- JSON layouts: `pending_conflicts[key]` = the conflict exactly as the preview shows it (`key, label, prior_value, prior_valid, prior_provenance, suggestion, confidence`); `frozen_attribute_origin[key]` = `{class: frozen|withheld|pending, provenance, status, confidence, prior}`; `resolved_attributes[key]` = `{value (null = no value), action, at, step_token}`.
- Redo resolution takes the value from the entry the step recorded, else from the step's after-row (live and valid → its value, else no value), and the action from that entry, else the step's kept feedback row.
- An edit that writes a feedback row, or records a resolution, never merges into the previous step.
- `accept_all` treats a suggestion with a prior whose key has no definition as a conflict (skipped, fail closed).

**Deviations.** `ConfirmAsShown` is a testkit trait on `PublishService` (plus `TestApp::confirm_as_shown`), so each call site keeps its own service expression (several hold the service or a second app); one rename and one `use` line per file. Files beyond the lists: `Cargo.lock` and `crates/testkit/src/lib.rs` (re-exports). The closing gate's `cargo test --workspace` ran as the same tests in sequence: `--workspace --exclude psm-testkit`, testkit `--lib --bins`, and the suite in two filters (455 + 407 = 862 of 862) to stay inside the session's 10-minute command limit. Tracker actor is `opus` (this session ran on Claude Opus), not `codex`.

**Observations.** The release writes no event; a held session's label changes on the next list read (R6·U3 may want an event). `npm test`: `p33_split_grid.test.tsx` C4(d) failed once under full load and passed alone (backlog rows P35/P37/P41 → P33). No hardware or live service used.


### 2026-11-12 · review r1 · opus · APPROVED_WITH_NOTES
**Gate.** Tracker IMPLEMENTED; checkpoints U1 `c8b22ae`, U2 `1e56542`, closing `6d0a520` in `git log`. **Perimeter.** `git show --stat` of each checkpoint = the handoff's declared lists exactly (U1 20 paths incl. `Cargo.lock`, `crates/testkit/src/lib.rs`; U2 36 paths incl. the 15 testkit `confirm` call-site files); closing commit = this file's Tracker line and Review log plus the implementer handoff. **Checks** (Review mode, once, clean tree `82f5613` = `6d0a520` + review prompt): `--test suite research::r5_` 40 passed; `p39 p40 p10 p13 p14 p64 p03` 167 passed; `cargo test -p psm-desktop r5_` 2 passed; clippy `-D warnings` clean; `fmt --check` clean; `npm run typecheck` clean; `vitest src/features/{form,publish,compose}` 268 passed. Implementer's full-suite stamp (862/862, split in parts; one known `p33` UI flake, file untouched) stands.

**Verified correct (code read, then its test, per row).** U1: every photographer action runs through `AttributesService::act` (operation lock → store transaction → `record_held`, T1/Q2); `note_action` writes one feedback row only when the row before was `ai` + `suggested`/`accepted` (S3), with the step's UUID token, final value null for Clear, prior's for Keep previous, new canonical for Edit; `accept`/`keep_previous` refuse wrong-state rows with `InvalidValue` before any write; `clear` and human `set_value` drop `ai_prior`; `accept_all` skips `low` and every `is_conflict` (no definition + prior = skipped, fail closed); research writes (`writes::suggest`) go through no `act` (no step, no feedback). History: the guard compares every `FieldState::Row` field (after-row for undo, before-row for redo) with `same_row` (all columns but `updated_at`) before applying anything, else `StepCannotApply` "that field was changed by research" and the step is dropped; titles untouched; undo reads + deletes the step's feedback rows and keeps them on the step, redo re-inserts with fresh ids (T2); a coalesced Edit updates its step's `edited` row's `final_value`; never merges when it wrote feedback or a resolution (N1). Commands are thin (action/value shape check, then the service). Looks right calls `attribute_ai_accept` and shows only on `suggested`. U2: `partition` = frozen / withheld / pending / left out exactly per MC-A10 (out-of-layout and no-definition conflicts left out, a prior that is not a conflict frozen at any confidence); `confirm` recomputes inside its transaction from `partition_in` and refuses on any difference (missing `shown` entry = 0/∅, S2); held items get no enqueue but their stock is frozen; `resolve_pending_in` is called unconditionally from every action and from redo (T3), entry `{value, action, at, step_token}`, "no value" when the row after is not live+valid (N4), release + job in the same transaction; `unresolve_in` removes only the undone token's entries while held; `run_attributes()` feeds both `run_staged` and `update` (`write_set`, condition, description, tags); cancel of a held item is one conditional UPDATE; `activity` offers Cancel on `held`; `active()` still excludes `Held`, so the startup re-queue leaves it; one count helper `unresolved_conflicts` used by the session list and the activity text (carried R2 note); research's `is_conflict` delegates to core. Tests: each of the 42 rows' tests observes the row's outcome at its named boundary (stored row, feedback rows, history state, `publish_items` columns, job rows, `FakeShop` writes/products/stock, staged `item.json`) and would fail on a wrong one. H1–H8: no mutation before a recorded confirmation or a recorded resolution; release atomic with its job (H4); no Tauri dependency in core; the `"shopify"` definition read is the existing, plan-prescribed pattern.

**Assumptions** 1–13 read against 08B MC-A10/MC-A12: none contradicts the book (narrative "Choices made for you").

**Notes** (3, none blocks) in `docs/implementation/backlog_research/R5_research_app_seams.md`: release emits no event (→ R6·U3); undoing a resolution after release brings the conflict back in the form while the run sent the resolved value (→ R6·U3 / 08B); `p15` C5(a) asserts a shown set for a session not in the confirmation (fixture reuse).
