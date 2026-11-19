---
plan: W6_restoration_choices
role: implement
round: 1
state: DONE
date: 2026-11-19
actor: claude (opus, pipeline-implementer)
---
# Handoff — W6_restoration_choices, implement r1

Gate: intention RATIFIED; `W5_seo_publish` Tracker APPROVED (2026-11-19); W6 Tracker was NOT_STARTED; owner card 1
answered A (Review log). The partial edits from the earlier run that stopped were already gone (the working tree was
clean at start), so this session started from scratch.

## What was built
- **"Restoration done" field.** Every item type's **Condition** step now ends with "Restoration done", a multi-choice
  field with the 14 fixed choices in WC8-R order: Cleaned, Sanded, Oiled, Waxed, Lacquered, Joints re-glued, Veneer
  repaired, New foam, New webbing, New springs, New papercord, New fabric, New leather, Rewired. It is added even when
  an item type's saved Condition layout predates it. Code: `local.restoration` in `builtin_definitions()`, plus an
  entry in `ALWAYS_PRESENT` (`crates/core/src/services/attributes.rs`).
- **Kept in Session Manager only.** The field is saved with the item and written to the export's `item.json`. A
  publish never sends it to Shopify, because `write_set` only walks the shop's definitions.
- **Research never fills it.** `rules::write_table::never_written` names it. A research run lists it as
  "Restoration done: not researched".
- **The writer uses it, and the post-check enforces it.** The writer receives the field like any other Condition
  field: a typed value is a fact. Post-check rule 11 (`HitCode::Restoration`) requires every chosen step to appear in
  the **condition report** through one of its WC8-R phrases, English or Swedish
  (`writer::defaults::RESTORATION_PHRASES`). Mentioning a step only in the description does not count (owner card 1).
  `CheckContext.restoration` is filled in `writer::run::check_context`, only when the field is visible and its value is
  a fact.

## Tests → criterion rows
| Row | Test |
|---|---|
| W6·C1(a) | `research::w6_restoration::w6_c1a_a_stored_condition_preset_ends_with_restoration_done` |
| W6·C1(b) | `research::w6_restoration::w6_c1b_two_choices_are_stored_as_a_set_by_a_human` |
| W6·C2(a) | `research::w6_restoration::w6_c2a_a_publish_never_sends_restoration_done` (the shop has one linked text field, so the publish does write attributes; the writes are exactly `[custom.material]`) |
| W6·C2(b) | `research::w6_restoration::w6_c2b_the_local_export_holds_restoration_done` |
| W6·C3(a) | `psm-research` lib `rules::write_table::tests::w6_c3a_restoration_done_is_never_written` |
| W6·C3(b) | `research::w6_restoration::w6_c3b_a_research_run_never_writes_restoration_done` (checked once by hand: with the `never_written` line disabled the test fails; the line was then restored) |
| W6·C4(a) | `research::w6_restoration::w6_c4a_the_writer_sees_restoration_done_as_a_typed_fact` (the wire code for the decision is `typed`) |
| W6·C5(a) | `writer::postcheck::tests::w6_c5a_every_chosen_step_in_the_report_is_no_hit` |
| W6·C5(b) | `writer::postcheck::tests::w6_c5b_a_step_only_the_description_states_is_one_hit_naming_it` |
| W6·C5(c) | `writer::postcheck::tests::w6_c5c_a_swedish_phrase_states_the_step` |

## Assumptions
1. **With no saved Condition layout, the field still comes last.** `active_definitions` now lists the built-in
   Product fields, then the shop's fields, then the other built-ins. WC8.1 puts always-present keys at the end of
   their step. The Product step's order does not change, because no shop field belongs to the Product group.
2. **A choice with no phrase row would be matched by its own name.** This cannot happen with the fixed list. It only
   protects against a future choice added without phrases.
3. **Delegated choice (how the choices are drawn):** the existing multi-choice control. With 14 options it renders as
   a multiselect, so no UI code changed. The research field pickers already hide built-in fields, so research
   settings cannot offer it.
4. **The photo note's `note_fields` and the writer's visible fields now include the field.** WC1.1 and WC2 define
   both as every Details and Condition layout key. Excluding it would mean changing those contracts.

## Deviations
- **One source file outside the plan's list:** `crates/research/src/writer/run.rs`. The post-check context is built
  there, and rule 11 needs the chosen steps.
- **Tests that list fields or built-ins had their expected lists updated; no assertion was weakened:**
  - `crates/research/tests/engine/w1_note.rs`: the stage 1 and stage 6 `note_fields` lists.
  - `crates/research/tests/engine/w2_writer_run.rs`: the writer `fields` list.
  - `crates/testkit/tests/suite/p13_publish_runs.rs`: the built-in count, 5 → 6.
- **Guide:** `docs/guide/02_architecture.md` gains item 7 (flow §7A).
- **The plan's unit test commands are not valid cargo syntax.** They pass two filters to one `cargo test`
  (`writer::postcheck rules::write_table`, and the three suite filters). Each filter was run separately.

## Files changed
The declared write perimeter, all in checkpoint `f988e5b`:
- `crates/core/src/services/attributes.rs`
- `crates/research/src/rules/write_table.rs`
- `crates/research/src/writer/defaults.rs`
- `crates/research/src/writer/postcheck.rs`
- `crates/research/src/writer/run.rs`
- `crates/research/tests/engine/w1_note.rs`
- `crates/research/tests/engine/w2_writer_run.rs`
- `crates/testkit/tests/suite/p13_publish_runs.rs`
- `crates/testkit/tests/suite/research/mod.rs`
- `crates/testkit/tests/suite/research/w6_restoration.rs` (new)
- `docs/guide/02_architecture.md`
- `docs/implementation/plans/W6_restoration_choices.md` (Tracker line and Review log only)

Plus this handoff, which the runner commits.

**Files touched by the temporary hand check:** `crates/research/src/rules/write_table.rs`. It was disabled for one
run of the C3(b) test, then restored byte for byte from a copy. Logs are in `.pipeline/w6/` (not committed).

## Checkpoint SHA
- U1: `f988e5b`

## Stamp
The closing gate was run once, one command at a time. The first workspace run stopped at the 3 engine expected-list
failures listed under Deviations. Those were fixed, the two failing files were re-run and passed (31/0 and 19/0), and
the workspace was then run with `--no-fail-fast` so that every test program ran.

| Command | Result |
|---|---|
| `cargo test --workspace --no-fail-fast` | green: suite 1039/0, research engine 328/0, research lib 259/0, core 102/0, other programs green |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean |
| `cargo fmt --all -- --check` | clean |
| `cd apps/desktop && npm run typecheck` | clean |
| `npm test` | 106 files, 661/0 |

Pre-existing failures: none. `bindings.ts` did not change.

## Manual try-out
1. Open a session and go to the Condition step. "Restoration done" is the last field. Pick Sanded and Oiled.
2. Press **Write texts**. The condition report should say the frame was sanded and oiled. If the writer leaves a step
   out of the condition report, the text is flagged with a check like `the restoration step "Oiled" is not in the
   condition report`.
3. Publish a linked item. The Shopify product gets no new field. Export the session: `item.json` holds
   `"local.restoration": ["Sanded", "Oiled"]`.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
