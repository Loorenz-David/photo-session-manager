---
plan: W4_writer_settings
role: implement
round: 1
state: DONE
date: 2026-11-19
actor: claude opus (pipeline-implementer)
---
# Handoff — W4_writer_settings, implement r1

Gate: intention RATIFIED; `W3_writer_review` Tracker APPROVED; W4 was NOT_STARTED. Passed.

## What was built
**Settings → Writer** is a new entry right after Settings → Research. It appears whenever Research does. In it the owner can:
- choose the writer's **model**: only models with a price in Settings → Research are offered. A saved model that has lost its price stays selected, marked "(no price)", so a save never swaps it silently.
- set the **effort**, the **language** (English or Swedish) and whether the writer **writes by itself on "Next to review"**.
- edit the **instruction** of the chosen language:
  - **Preview what is sent** shows the exact request (instructions, input, schema) for a built-in example item. The example uses example A's fields. Nothing is sent, and no run or call row is made.
  - **Earlier versions** lists the last 20 texts per language, with their dates. **Use this** puts one back in the editor; it is saved only by Save.
  - **Reset to default** returns to the shipped instruction and keeps the owner's text among the earlier versions.
- tick the **fields the writer may see**: Details and Condition fields. With no saved list the default set is ticked (no prices).
- edit the **style examples** for both languages (add, edit, remove).
- add **extra banned words** (one per line) and change the **target lengths**.
- set each **safeguard type** to off, note or ask. Each type shows "answered n · dismissed m".
- set the **questions per write** and the **shop name for search titles**.

**Save writer settings** always sends the whole `writer` slot. A refusal names the field and saves nothing. The writer reads the saved values at its next run.

Under the hood:
- **One request builder.** A run's call and the preview are built by one function, `writer::run::request_of`. It works over one input assembler, `writer::input::assemble`, and one snapshot resolver, `WriterSnapshot::resolve`.
- **Versions rule.** `WriterSettings::push_version` (unit-tested) applies it, inside `saved_over` and `reset_instruction`.
- **Shipped defaults stay live.** An instruction or example list equal to the shipped one is stored as the shipped default, so later builds' improvements still reach it.

## Tests → criterion rows
| Row | Test |
|---|---|
| W4·C1(a) | `apps/desktop/src-tauri/src/lib.rs` `w4_tests::w4_c1a_without_a_writer_slot_every_field_shows_its_default` |
| W4·C1(b) | `w4_tests::w4_c1b_the_counts_are_the_answered_and_dismissed_findings_per_type` |
| W4·C2(a) | `w4_tests::w4_c2a_a_changed_instruction_keeps_the_text_it_replaced_with_its_time` |
| W4·C2(b) | `crates/research/src/writer/config.rs` `w4_c2b_push_version_keeps_twenty_newest_first_dropping_the_oldest`, `w4_c2b_push_version_keeps_nothing_when_the_text_is_unchanged` (core mechanism) |
| W4·C2(c) | `w4_tests::w4_c2c_an_unchanged_text_keeps_no_version_and_an_invalid_value_saves_nothing` |
| W4·C3(a) | `w4_tests::w4_c3a_reset_returns_to_the_shipped_instruction_keeping_the_saved_text` |
| W4·C4(a) | `w4_tests::w4_c4a_the_preview_is_the_request_for_the_example_item_under_the_saved_settings` |
| W4·C4(b) | `w4_tests::w4_c4b_the_preview_makes_no_run_no_call_and_no_model_request` |
| W4·C5(a) | `w4_tests::w4_c5a_the_run_sees_only_the_saved_fields_and_asks_at_most_the_saved_number` |
| W4·C5(b) | `w4_tests::w4_c5b_the_run_calls_the_saved_model_and_effort` |
| W4·C6(a) | `apps/desktop/src/features/settings/__tests__/w4_writer_settings.test.tsx` `w4_c6a_writer_entry_and_the_defaults_with_the_priced_models` |
| W4·C6(b) | `w4_c6b_swedish_and_automatic_writing_off_save_the_whole_slot` |
| W4·C7(a) | `w4_c7a_an_edited_instruction_is_saved` |
| W4·C7(b) | `w4_c7b_preview_shows_the_instructions_input_and_schema` |
| W4·C7(c) | `w4_c7c_use_this_puts_an_earlier_version_in_the_editor_without_saving` |
| W4·C7(d) | `w4_c7d_reset_to_default_shows_the_returned_default_text` |
| W4·C8(a) | `w4_c8a_without_a_saved_list_the_default_fields_are_checked_and_prices_not` |
| W4·C8(b) | `w4_c8b_unchecking_a_field_saves_the_list_without_it` |
| W4·C9(a) | `w4_c9a_an_added_swedish_example_is_saved_after_the_existing_one` |
| W4·C10(a) | `w4_c10a_each_safeguard_type_shows_its_mode_and_counts` |
| W4·C10(b) | `w4_c10b_mode_off_questions_and_shop_name_are_saved` |

The U1 tests drive the real Tauri commands. They use a mock Tauri app over the wired `TestApp` with the research fakes, and check the answers, `settings.json`, the run and call rows, and the fake model's requests.

## Assumptions
Each is the smallest option consistent with 08C §8:
1. **The preview uses the saved settings** (plan task 4). An unsaved edit shows in the preview only after Save; the section says so under the buttons.
2. **The preview asks for all five texts.** There is no item, so there is no layout to narrow them.
3. **The example item's fields** are example A's fields line split into twelve labelled facts (`example.type` … `example.photo_note`), each marked as typed by the owner.
4. **Layout of the section.** Style examples show both languages at once; the instruction editor shows the chosen language.
5. **Effort is a free text field**, as in Settings → Research.
6. **Extra banned words** are entered one per line.
7. **Reset saves at once** and replaces only that language's instruction in the form. Other unsaved edits stay in the form.
8. **Shipped defaults are detected by text.** An instruction or example list saved equal to the shipped text is stored as "shipped default".
9. **The Writer entry is shown only when the Research entry is.** The writer needs the research engine and its prices.

## Deviations
- **Files outside U1's list,** each needed by the plan's own tasks:
  - `crates/research/src/writer/input.rs`: the shared `assemble`, so the preview and a run build the input through one function.
  - `crates/research/src/runs.rs`: one call to `WriterSnapshot::resolve`.
  - `crates/research/src/api_impl.rs`: the new trait methods.
- **`WriterApi::counts`** was added beside the registry's three W4 methods. The get reads the counts through the writer handle.
- **Where the U1 tests live.** They are desktop command tests in `apps/desktop/src-tauri/src/lib.rs` (module `w4_tests`), not a testkit stem. The row boundary is the command layer, which the testkit cannot reach.
- **Files outside U2's list:**
  - `__tests__/t1_settings_drawer.test.tsx`: its expected section list now includes "Writer". This is a behaviour change, not a test defect.
  - `docs/guide/01_product.md` and `docs/guide/02_architecture.md`: flow §7A, a new setting screen and new commands.

## Files changed
Declared write perimeter. All changes are in the two checkpoints except the closing files.
- U1 (`23a9668`):
  - `crates/research/src/writer/{api.rs, config.rs, input.rs, run.rs}`
  - `crates/research/src/{api_impl.rs, runs.rs}`
  - `apps/desktop/src-tauri/src/{commands/writer.rs, lib.rs}`
  - `apps/desktop/src/services/{bindings.ts (generated by export_bindings), writer.ts}`
- U2 (`82da541`):
  - `apps/desktop/src/features/settings/{WriterSection.tsx, WriterInstruction.tsx, WriterExamples.tsx, SettingsNav.tsx, SettingsPage.tsx}`
  - `apps/desktop/src/features/settings/__tests__/{w4_writer_settings.test.tsx, t1_settings_drawer.test.tsx}`
  - `docs/guide/{01_product.md, 02_architecture.md}`
- Closing:
  - `docs/implementation/plans/W4_writer_settings.md` (Tracker line, Review log)
  - this handoff

Not touched: no archgraph or other tool-recorded state; no mutation-probe files (the regime is off, flow §4).

## Checkpoint SHA
1. U1: `23a9668`
2. U2: `82da541`
3. Closing (tracker, review log, handoff): the `CHECKPOINT (not approved): W4_writer_settings closing` commit that follows these.

## Stamp
| Command | Result |
|---|---|
| `cargo test -p psm-research --lib writer::config` | 6 passed |
| `cargo test -p psm-testkit --test suite research::w3_writer_wiring::` | 8 passed |
| `cargo test -p psm-desktop` | 61 passed (bindings regenerated) |
| `cargo clippy -p psm-research -p psm-desktop --all-targets -- -D warnings` | clean |
| `npx vitest run src/features/settings` | 19 files, 96 passed |
| **Closing gate:** `cargo test --workspace` | 1829 passed, 0 failed |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean |
| `cargo fmt --all -- --check` | clean |
| `cd apps/desktop && npm run typecheck` | clean |
| `npm test` | 105 files, 659 passed |

There are no pre-existing failures.

## Manual try-out
Use a build of this branch, run with `PSM_DATA_DIR=$HOME/psm-research-dev`.
1. Open Settings → Writer (below Research). Change the instruction, for example add "Keep descriptions short." Press **Save writer settings**, then **Preview what is sent**: the dialog shows the request with your text first, example A's fields as the input, and the answer schema.
2. In a session press **Write texts**: the texts follow the new instruction. Back in Settings → Writer, **Earlier versions** lists the shipped text with today's date. **Reset to default** brings it back, and your text joins the list.
3. Set **Writing language** to Swedish, save, and write texts again: they arrive in Swedish.
4. Set **Field vs field** to off and untick a field under **Fields the writer may see**. Save and write again: that field is no longer in the writer's input (the research panel's Writer block counts the fields used).

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
