---
phase: A07_research_storage_and_write_permission
title: What research may write — a suggestion never replaces a value you typed or accepted, never brings back a value you cleared, remembers the preset or Shopify value it replaced, and is lowered instead of deleted when its support is gone
difficulty: high
projection: yes
depends_on: [A03_research_values_and_sameness]
---
# A07 — Storage and the write table (08B MC-A1; HA1; MA1, MA10)

## Goal
Make `AttributesService::ai_suggest` the one door research writes through, and make it apply MC-A1 inside its
transaction: the fields never written, the write table per stored state, the prior value (`ai_prior`) and the
conflict it defines, the cleared set, lowering, and the unchanged idempotency check. Add the first research
migration. **Not in this phase:** the photographer's actions and their feedback rows (A08; this phase only reads
`research_feedback`), publishing (A09), anything that calls `ai_suggest` in production (A12).

## Read first
`master_plan.md` §3, §6.5, §6.9, §7.3, §7.5 · `master_plan_research.md` §0 (migrations, test folder), §3, §6.3–§6.5,
§7.4 · book `01`, `14` (MA1, MA10), `08` (Values and provenance, Groups and presets), `08A` §2 (HA1), §5 (What a run
may write), `08B` **MC-A1** (all), MC-A2 (the mapped value arrives already valid) · code
`crates/core/src/services/attributes.rs` (`ai_suggest` line 476, `write`, `same_content` line 81, `clear`,
`active_definitions`, presets), `crates/core/src/domain/attribute.rs` (`Value`), `crates/core/src/ports/store.rs`
(`StoreTx::{attribute_values, put_attribute_value, attribute_preset}`), `crates/infra/src/sqlite/store.rs`
(`migrations()`), `mechanisms/research/sameness.rs` (A03).

## Dependencies and gate
Tracker row and gate: `master_plan_research.md` §4. Notes: `backlog_research.md`.

## Files expected to change
New: `crates/infra/src/sqlite/migrations/a01_research_values.sql`; `crates/core/src/mechanisms/research/write_table.rs`;
`crates/testkit/tests/suite/research/{mod.rs, a07_research_storage_and_write_permission.rs}` (create `mod.rs` and the
`mod research;` line in `tests/suite/main.rs` if absent).
Edited: `crates/core/src/domain/attribute.rs` (`Value.ai_prior: Option<AiPrior>`), `crates/core/src/domain/research.rs`
(`AiPrior`, `Feedback`, `FeedbackAction`), `crates/core/src/services/attributes.rs`, `crates/core/src/ports/store.rs`
and `crates/infra/src/sqlite/store.rs` (the column; `research_feedback` insert and read; the migration appended in
the research block), every place that builds a `Value` literal or its DTO (`apps/desktop/src-tauri/src/commands/
attributes.rs`, `apps/desktop/src/services/bindings.ts` regenerated, test fixtures that list the fields), own tracker row.

## Core mechanisms (unit tests allowed)
`write_table::{never_written, decide_write, is_conflict}` (inline tests may cover the table; the criteria below are
proven at the service boundary).

## Ordered tasks
1. Migration `a01_research_values.sql` (`master_plan_research.md` §6.4), appended after the last `0NN` entry under
   the comment `// research (feature/agentic): keep this block last`.
2. `Value.ai_prior` end to end (store, DTO, bindings). No behaviour change for existing writers: they write null.
3. `write_table`: `never_written` (key not in the item's current layout; a `product`-step field; core types
   `media_list`, `reference`, `rich_text`, `money`, `rating`; `writable = false`; `product.title`), `decide_write`
   (the table, fail closed), `is_conflict` (`ai` + `suggested`, has a prior, not `S(value, prior.value)`).
4. `ai_suggest` reads the stored row, the definition, the layout (the presets of the item's type, all three steps)
   and the cleared set **inside its transaction** and applies `decide_write`. It returns an outcome the caller can
   put in the trace: written, unchanged, refused (with the table's reason), `matches a value you cleared`, `already
   holds this value`, `not researched`.
5. Cleared set `C(item, key)`: the stored row's value when it is cleared, plus the `suggested_value` of every
   `research_feedback` row of the field with action `cleared` or `kept_previous`; membership by `S`.
6. `ai_lower(item_id, key, reason)`: on an `ai` + `suggested` row, the same value with confidence `low`, the sources
   kept, the reason `No longer supported: <reason>`; on any other row, nothing.
7. Keep `same_content`: an equal write is not written. Research writes record no undo step (they pass no action,
   as today).

## Criteria (7 criteria, 26 rows)
Boundary of every row: `AttributesService::ai_suggest` / `ai_lower` on a `TestApp` with a real SQLite store; the
outcome is the stored row read back (`attribute_values`) and the returned outcome.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | write table | no stored row; suggest A, medium | row `ai` + `suggested`, A, medium, `ai_prior` null | MA10 |
| C1(b) | same | stored `default`, live, D; suggest A | row `ai` + `suggested` A; `ai_prior` = `{D, valid, default}` | MA10 (R-IA1) |
| C1(c) | same | stored `integration`, live, valid I; suggest A | row A; `ai_prior` = `{I, true, integration}` | MA10 |
| C1(d) | same | stored `integration`, live, **invalid** raw value; suggest A | row A; `ai_prior` = `{raw, false, integration}` | MA10 |
| C1(e) | same | stored `ai` + `suggested` A with prior D; suggest B | row B; `ai_prior` still D | MA10 |
| C1(f) | same | stored `ai` + `accepted` A; suggest B | refused; row byte-identical to before | MA1 |
| C1(g) | same | stored `human`, live; suggest B | refused; row unchanged | MA1 |
| C1(h) | same | stored `human` + `cleared`; suggest B | refused; row unchanged | MA10 (R-IA8) |
| C1(i) | same | stored `ai` + `cleared` X; suggest X; suggest Y | X: `matches a value you cleared`, row unchanged; Y: row `ai` + `suggested` Y | MA10 |
| C1(j) | same | stored `default` + `cleared` X; stored `integration` + `cleared` X; each: suggest X, then Y | X refused as cleared; Y written | MA10 (R-IA8) |
| C1(k) | same | stored `ai` with no status (written straight through the store); suggest B | refused; row unchanged | MA10 |
| C2(a) | prior and conflict | C1(b)'s row | `is_conflict` true | MC-A1 |
| C2(b) | same | stored `default` `Teak` (text); suggest `teak ` | not written, outcome `already holds this value`; row still `default` | MC-A1 |
| C2(c) | same | stored `ai` + `suggested` A with prior D; suggest a value the same as D under `S` | row holds it with the prior kept; `is_conflict` false | MC-A1 |
| C2(d) | same | a suggestion with no prior | `is_conflict` false | MC-A1 |
| C3(a) | cleared set | feedback row `cleared` with suggested X; current row `ai` + `suggested` Y, then cleared; suggest X; suggest Y; suggest Z | X refused, Y refused, Z written | MA10 (R-IA8) |
| C3(b) | same | feedback row `kept_previous` with suggested X; suggest X | refused as cleared | MA10 |
| C3(c) | same | text field, feedback `cleared` with `Hans J. Wegner`; suggest `hans j wegner` | refused as cleared (membership by `S`) | MA10 |
| C3(d) | same | feedback row `accepted` with suggested X on a now-empty field; suggest X | written | MC-A1 |
| C4(a) | `ai_lower` | stored `ai` + `suggested` A, high, sources `[u1]`; lower with reason `the two listings now disagree` | value A, confidence `low`, sources `[u1]`, reason `No longer supported: the two listings now disagree`, status `suggested` | MA2 |
| C4(b) | same | stored `ai` + `accepted`; stored `human`; no row | nothing changes, three times | MA1 |
| C5(a) | never written | a key not in the item's layout; suggest A | outcome `not researched`; no row created | MA10 |
| C5(b) | same | a `product`-step field in the layout; a `media_list`, a `reference`, a `rich_text`, a `money`, a `rating` field; a definition with `writable = false`; `product.title` | `not researched`, eight times; no row written, the item's title unchanged | MA10 (R-IA6, R-IA7) |
| C6(a) | idempotency | the same suggestion (value, confidence, reason, sources) twice | the second returns unchanged; `updated_at` unchanged; the session's activity time unchanged | MA14 |
| C6(b) | undo | a research write, then `History` state for the session | no undo step exists for it | MC-A12 |
| C7(a) | migration | a database stopped one migration before `a01` holding one `attribute_values` row; migrate to latest | the row is intact with `ai_prior` null; `research_feedback` exists and is empty | MC-A1 |

## Delegated choices
The outcome type's name and variants beyond the reasons listed · whether `decide_write` returns the new row or an
instruction · the feedback store methods' names. Not delegated: the reasons' texts quoted above, and that every
read used by the decision happens inside the writing transaction.

## Manual try-out
Nothing visible yet: no research runs. The app behaves as before; open a form on scratch data
(`PSM_DATA_DIR=$HOME/psm-research-dev`) and check that existing values, defaults and "from Shopify" values still show.

## Notes
- Hazard (earned in P67): a new field on `Value` changes many struct literals and one generated DTO. List the
  touched test files under **Deviations**; no test's expectation may change except to name the new field.
- `ai_suggest`'s new refusals are outcomes, not errors, except where today's callers rely on the `human` error; keep
  that error text for the existing P10 test or update that one assertion and say so.
- The cleared set reads `research_feedback`, which only A08 writes in production; this phase's tests insert rows
  through the store method they add (it has a production caller in A08).
- Review mode: this phase's module (`cargo test -p psm-testkit --test suite research::a07_research_storage_and_write_permission::`),
  `p10_attributes::`, `p67_form_values_defaults_and_batch_step::`, `p01_storage_identity::`, `cargo test -p psm-core
  mechanisms::research`, clippy, fmt, typecheck. No full suite unless the implementer's stamp is missing.

## Review log
