---
phase: R13_research_chosen_fields
title: Research fills only the fields you chose — Settings → Research gets one list, "Fields research fills", in place of "Fields research leaves alone"; a field that is not on it is never asked for, suggested or paid for, and a new shop field stays empty until you add it; the detailed condition is worked out from the condition levels that have a value, so an empty level no longer holds the grade back
difficulty: high
projection: no
depends_on: [R12_research_cost]
blocked_on: none (U4's check on a few pieces, task 3, is the orchestrator's, with a spending ceiling of USD 2; U1–U3 and U4 tasks 1–2 are not blocked)
---
# R13 — Research fills only the fields you chose

**Tracker:** NOT_STARTED · 2026-11-15 · planner · plan written (4 units, 15 criteria, 40 rows `A27·…`, 1 of them `[manual]`); needs R12 APPROVED

(This line is the milestone's tracker row. Sessions change only this line of this file: state, date, actor, one-line note. The gate for `depends_on` is the Tracker line of each plan named there. `master_plan.md` §4 is not used for research milestones.)

## Goal
Implement the owner's decision of 08B changelog 17. (U1) The setting `fill_keys` ("Fields research fills") replaces `excluded_keys` everywhere it is read: the engine, the settings shape the app sends to the screen, the picker, the tool's own settings. A field that is not in the list is treated exactly as a never-written field of MC-A1. (U2) The detailed condition is derived from the condition levels that have a value; a level with no value no longer makes it undetermined. (U3) Settings → Research says what the list means and warns when a designer, maker or detailed-condition field is not on it. (U4) The measurement tool's owner settings carry the owner's list, and the orchestrator checks it on a few pieces. **Not here:** a new 42-piece measurement; any change to which values may be written to a chosen field or at what confidence; the publish hold (MC-A10); skipping the search, read and compare stages when no chosen field needs them (see Notes of U1).

## How to work this plan (flow.md §10)
This milestone is one runner phase made of 4 units. Work the units in order. After each unit run its **Unit test commands**, then make a `CHECKPOINT (not approved):` commit of that unit's paths. One review, one fix budget and one narrative cover the whole milestone. Row ids of this plan are `A27·C<n>(<row>)` (the next free plan number in `docs/implementation/research_row_mapping.md`); criteria are numbered once across the units. Name each test after its row (`a27_c1a_<slug>`). Reviewer notes go to `docs/implementation/backlog_research/R13_research_chosen_fields.md`. **If launched by `run_phase.sh`, the implementer builds U1–U3 and tasks 1–2 of U4 with their automated rows, then reports `STATE: DONE`; the check on a few pieces (U4 task 3) is the orchestrator's.**

| Unit | What it builds | Criteria | Rows | Changes app files |
|---|---|---|---|---|
| U1 | The list of fields research fills, in the engine and at the app's settings seam | 7 | 20 | **yes** (settings DTO, generated bindings, the picker's binding, test helpers) |
| U2 | Detailed condition from the levels that have a value | 2 | 10 | no |
| U3 | Settings → Research: what the list means, and the warnings | 4 | 7 | **yes** (TypeScript only) |
| U4 | The tool: the owner's list, and a check on a few pieces | 2 | 3 | no |
| **Total** | | **15** | **40** | |

## Read first (whole milestone)
`docs/process/flow.md` (**§10 is the authority for this track**) · `docs/implementation/master_plan.md` §3, §7 · `docs/implementation/master_plan_research.md` (all; §6.9 holds the names this plan adds and retires) · book `01`, `14` §Research agent, `08A`, `08B` (**changelog 17 is the owner's decision this plan implements; it wins over older text in 08A, 08B and the R11 plan**) · evidence `docs/intention/evidence/research_providers.md` ("Acceptance measurement 2": the detailed-condition result and the finding at its end) · the R11 plan, Units 2, 3 and 6 (what `excluded_keys`, the derived field and the pickers are today) · then each unit's own Read first.

## What this plan supersedes in the book (the orchestrator folds these into 08A and 08B; until then this section is the rule)
| Book text | Replaced by |
|---|---|
| 08B MC-A1 "Fields never written by research": the list ends at SKU and barcode | U1: plus **a key that is not in `research.fill_keys`**. It is checked after the others, and the trace's last step names the field "not researched" like them. Authority: changelog 17 (a). |
| 08B MC-A4 "Field class" as R11 left it (seven steps: never-written · left alone (`excluded_keys`) · derived · condition · attribution · measured · visible) | U1: seven steps, first match wins: never-written (MC-A1) · **not chosen** (key not in `fill_keys`), treated exactly as never-written · derived (`detailed_condition.key`) · condition · attribution · measured · visible. The step "left alone" and the setting `excluded_keys` are gone. |
| 08B changelog 12 (a) "The list is a setting (Settings → Research, 'fields research leaves alone' …)" and the R11 plan, Unit 2 tasks 1–2 and Unit 6 task 3 | U1, U3: one picker, "Fields research fills". The physical fields are simply not chosen. |
| 08B MC-A6 "Prerequisites": seven, the fifth "nothing to research" (as R11 left them) | U1: eight. A new sixth, **no field chosen** (the layout has fields research may write, and none of them is in `fill_keys`), between "nothing to research" and "day limit". It stops a run row from being created, like the first five. |
| 08B MC-A7 Units row 6: "one per run when there is at least one Condition photo" (stage 6) | U1: and at least one condition field is chosen. With none, stage 6 makes no call. |
| The R11 plan, Unit 3 "The rule", step (2): "a level field has no value → not determined, 'condition levels are incomplete: <labels>'" | U2: levels with no value are left out; the worst **filled** level wins. Only when no level field has a value is nothing derived ("no condition level has a value"). The whole rule is restated in U2. Authority: changelog 17 (b), confirmed by the owner 2026-11-15. |
| The R11 plan, Unit 3 "The rule": "A level field in `excluded_keys` still counts here" | U2: a level field that is not in `fill_keys` still counts when it holds a value (the person filled it). |
| 08A §6 / 08B MC-A4, for a key in `attribution_keys` or named by `detailed_condition.key` | U1: those two settings say **how** a chosen field is decided. They never make research fill a field that is not in `fill_keys`. |

## Points recorded by planning (decided here; each is one sentence in the narrative's "Choices made for you")
- **The list is empty on a fresh install, and research then does nothing.** The engine may not name a shop's fields (HA5), and changelog 17 (a) says a new shop field is never filled until the person adds it; on a fresh install every field is new. The form's research panel and "Run research" say "Choose the fields research fills in Settings → Research". This amends R10's row A24·C1(a) ("the shipped defaults start a requested run"): the shipped defaults plus one chosen field start a run (A27·C4(c)).
- **One gate, two modifiers.** `fill_keys` decides whether a field is researched. `attribution_keys` (designer and maker) and `detailed_condition.key` only decide how. A designer field that is not on the list is not researched; a detailed-condition field that is not on the list is not derived. The screen warns in both cases (U3).
- **An old settings file is not converted.** A saved `excluded_keys` loads without error, is ignored and is not written back (as R11 did for `auto` and `quiet_seconds`). Nothing is filled until the person chooses fields. Only scratch installs ever ran this branch.
- **A run already started keeps its own list.** A run executes from the settings snapshot taken when it started. A run that an earlier build left unfinished has no list in its snapshot and therefore writes nothing more; the next run uses the saved list.
- **Changing the list starts nothing.** "Arranged — next" creates a run only when the photos changed (R11). After adding a field to the list, an item already researched gets it through **Run research**.
- **No money for condition fields nobody chose.** When no condition field is on the list, the condition photos are not sent to the model at all (stage 6 is the most expensive single call of a run).
- **The reason says how much the grade rests on.** When some levels have no value, the detailed condition's reason ends with "<n> of <m> condition levels have no value." The grade and its confidence are not lowered for that (the owner's decision).
- **The fingerprint version stays 2.** Stage 7's identity names `fill_keys` where it named `excluded_keys`, so every stage 7 fingerprint changes and no decision made under the old rule can be reused; R11 changed the same identity without a bump. `a04_c7a` and A26·C6(b) stand.
- **The tool without a settings file measures every field,** as before this plan: its own scratch settings choose every field of its shop definitions. The owner's file narrows it.

## Tests that pin what this plan retires or changes (found by grep for `excluded_keys`, `field_class(`, `NothingToResearch`, "leaves alone", "incomplete", `ResearchSettings::default()`; each is handled in the unit named)
| Test | File | Pins | What happens, where |
|---|---|---|---|
| `a05_c5a_field_class_first_match`, `a25_c13a_detail_grade_is_derived`, `a25_c13c_empty_grade_key_keeps_visible` and every other inline call of `field_class` (10 in the file) | `crates/research/src/rules/confidence.rs` | the fourth parameter is the left-alone list, passed empty | U1: kept with their outcomes; each passes the field's key in the chosen list (rows A05·C5(a), A25·C13(a), (c) keep their ids) |
| `a25_c8a_excluded_fields_are_never_written`, `a25_c13b_excluded_precedes_derived` | same | the left-alone step | U1: deleted; A27·C1(a), (b) replace them |
| `a25_c8b_excluded_fields_leave_all_requests_and_writes`, `a25_c8c_all_fields_left_alone_means_nothing_to_research`, `a25_c8d_existing_suggestion_in_left_alone_field_is_untouched` | `crates/research/tests/engine/settings_r11.rs` | `excluded_keys` in a run | U1: deleted; A27·C2(a), A27·C3(d), A27·C2(b) replace them |
| `a25_c10a_reset_keeps_personal_lists_and_restores_defaults` | same | the reset keeps `excluded_keys` | U1: amended, it keeps `fill_keys` and the value has no `excluded_keys` (A27·C5(b) states it; A25·C10(a) keeps its id for the rest) |
| `a25_c10b_saved_reset_allows_arranged_run` | same | a reset value starts a run | U1: kept; its live slot holds a list with a field of the layout (the reset keeps it) |
| `a25_c11a_new_settings_default_empty` | same | `excluded_keys` empty by default | U1: amended, the clause becomes A27·C4(a); the other two defaults stand |
| the stage 7 fingerprint literal (`"excluded_keys": []`) in `a12_c1c_stage_outputs_one_per_unit_with_fingerprints` | `crates/research/tests/engine/stages.rs` | stage 7's identity | U1: the literal names `fill_keys` with the run's list |
| `a13_c1a_each_prerequisite_alone`, `a13_c1b_the_first_failing_is_named`; the inline test of `first_failing` and its helper `passing()` | `crates/research/tests/engine/lifecycle.rs`, `crates/research/src/rules/run_rules.rs` | seven prerequisites | U1: amended to eight (rows A13·C1(a), (b) keep their ids); A27·C3(a)–(c) are the new adjacent pairs |
| `a25_c2b_creation_prerequisites_name_the_reason` | `crates/research/tests/engine/arranged.rs` | the creation-blocking reasons | U1: gains the new one (the row keeps its id; A27·C3(d) is the full statement) |
| `a24_c1a_shipped_defaults_start_a_requested_run` | `crates/research/tests/engine/defaults.rs` | a fresh app with no settings slot starts a run | U1: amended into A27·C4(b) and (c) |
| `test_settings()`, `Harness::config()`, `Harness::stage_config()` and every engine test that starts a run through them | `crates/research/tests/engine/harness.rs` and the files using it | every field of the test's layout is researched | U1: the helpers choose every field the tests' layouts use, so each test keeps its outcome; a test about the list sets it itself |
| `a25_c14b_missing_levels_do_not_write_grade`; inline `a25_c12g_missing_value_names_label` | `crates/research/tests/engine/derived.rs`, `crates/research/src/rules/derived_condition.rs` | an empty level makes the grade undetermined | U2: `a25_c12g` deleted (A27·C8(a) replaces it); `a25_c14b` amended to the new reason (A27·C9(b)) |
| `research_test_settings()` | `crates/testkit/src/test_app.rs` | how app tests get a run that writes | U1: gains `fill_keys` with every key of the testkit's shop definitions |
| `a25_c23a_new_commands_reach_wired_app_and_bindings` (the list of names the bindings must contain) | `apps/desktop/src-tauri/src/lib.rs` | `excluded_keys` in the bindings | U1: the name becomes `fill_keys`, and the bindings no longer contain `excluded_keys` (A27·C7(a)) |
| the fixtures (`excluded_keys: []`) | `apps/desktop/src/features/settings/__tests__/{r6_research_settings.test.tsx, r7_research_export.test.tsx, r11_research_pickers.test.tsx}` | the settings shape | U1: `fill_keys: []` |
| `a25_c29a_fields_left_alone_start_unchecked_and_save_the_two_checked` | `…/__tests__/r11_research_pickers.test.tsx` | the left-alone picker | U1: deleted; A27·C7(b) replaces it |
| `a25_c32a_reset_names_what_it_keeps_calls_the_command_once_and_shows_the_returned_values` | same | the six kept things by name | U3: amended, "Fields research fills" in place of "Fields research leaves alone" (A27·C13(a) states it) |
| `a26_c13b_the_owner_settings_leave_six_fields_alone`, the constant `LEFT_ALONE` | `crates/research-tool/tests/tool/u4_accept.rs` | the six left-alone keys of the owner's file | U1: deleted (the file's `excluded_keys` is no longer read); A27·C14(a) replaces it in U4 |
| `a23_c2a_accept_runs_two_pieces_in_the_scratch_folder`, `a26_c13a_accept_measures_the_shipped_models` | same | `accept` without a settings file runs | U1: kept green by the tool's own list (A27·C6(a)) |
One shipped default is new (`fill_keys`, empty) and one is removed (`excluded_keys`). No other default changes; A24·C2(a), A24·C2(b) and `a26_owner_detailed_condition_default` stand.

## Unit 1 — The list of fields research fills

**Unit id:** `R13·U1` · **Rows:** A27·C1–C7 · **Changes app files:** yes (`apps/desktop/src-tauri/src/{commands/settings.rs, lib.rs}`, the generated `bindings.ts`, `ResearchSection.tsx`, three UI test files, `crates/testkit/src/test_app.rs`)

### Goal
`fill_keys` replaces `excluded_keys` in one step, everywhere the setting is read, so every checkpoint is green. A field not in the list is in no request of any stage, gets no value, and is named "not researched". With no chosen field in the item's form, no run is created and the panel says what to do. **Not in this unit:** the detailed-condition rule (U2); the screen's helper texts and warnings (U3); the owner's settings file (U4).

### Read first
`master_plan_research.md` §6.3, §6.6, §6.8, §6.9 · book `08B` MC-A1 (fields never written; lowering), MC-A4 (field class), MC-A6 (prerequisites), MC-A7 (units; settings snapshot), changelog 17 (a) · the R11 plan, Unit 2 · code `crates/research/src/{config.rs (ResearchSettings, ResearchConfig, from_slot, from_settings), rules/confidence.rs (field_class), rules/run_rules.rs (PrerequisiteInput, first_failing), domain.rs (Prerequisite), stages.rs (class_of, fields_of, condition), decide.rs (the stage 7 identity), runs.rs (prerequisite_input), api_impl.rs (settings_reset)}`, `crates/research-tool/src/accept.rs (scratch_settings, shop_definitions)`, `apps/desktop/src-tauri/src/commands/settings.rs (ResearchSettingsDto, to_dto)`, `apps/desktop/src/features/settings/{ResearchSection.tsx, ShopFieldPicker.tsx}`.

### Files
Edited: `crates/research/src/{config.rs, rules/confidence.rs, rules/run_rules.rs, domain.rs, stages.rs, decide.rs, runs.rs, api_impl.rs}`; `crates/research/tests/engine/{harness.rs, settings_r11.rs, stages.rs, lifecycle.rs, arranged.rs, defaults.rs, main.rs}`; `crates/research-tool/src/accept.rs`; `crates/research-tool/tests/tool/u4_accept.rs` (and `u3_report_and_replay.rs` if its run helper needs the list); `crates/testkit/src/test_app.rs`; `apps/desktop/src-tauri/src/{commands/settings.rs, lib.rs}`; `apps/desktop/src/services/bindings.ts` (generated by `cargo test -p psm-desktop export_bindings`, never by hand); `apps/desktop/src/features/settings/ResearchSection.tsx`; `apps/desktop/src/features/settings/__tests__/{r6_research_settings.test.tsx, r7_research_export.test.tsx, r11_research_pickers.test.tsx}`; `master_plan_research.md` §6.3 (the `Prerequisite` list). New: `crates/research/tests/engine/chosen_fields.rs`, `apps/desktop/src/features/settings/__tests__/r13_research_fills.test.tsx`.

### Core mechanisms (the only functions with unit tests)
`rules::confidence::field_class` (the not-chosen step) · `rules::run_rules::first_failing` (the new prerequisite and its two neighbours).

### Ordered tasks
1. `ResearchSettings` and the run's settings snapshot (`ResearchConfig`) gain `fill_keys` (a list of field keys, default empty) and lose `excluded_keys`. A saved slot that still carries `excluded_keys` loads; the key is ignored and not written back.
2. `rules::confidence::field_class` takes the chosen list in place of the left-alone list. Order, first match wins: never-written (MC-A1) · key not in the chosen list → never-written · derived · condition · attribution · measured · visible. Everything that asked the left-alone list asks this (the requests of every stage, the writes, the final step's "not researched", no lowering of an earlier suggestion).
3. Prerequisite `NoFieldChosen`, sentence "Choose the fields research fills in Settings → Research". It fails when the item's layout has a field research may write (MC-A1) and none of those is in `fill_keys`. Order: … price missing · nothing to research · **no field chosen** · day limit · offline. It blocks the creation of a run row, like the five before it. "Nothing to research" keeps its meaning (MC-A1 alone) and its sentence.
4. Stage 6 makes no call when the layout has no condition field of class condition (none chosen): no Condition request, no stage 6 `looked` step, nothing spent. How the stage is left out is delegated.
5. Stage 7's identity carries `fill_keys` in place of `excluded_keys`. `RESEARCH_FINGERPRINT_VERSION` stays 2.
6. `ResearchApi::settings_reset()` keeps `fill_keys` from the live value (with the trusted sites, designer and maker fields, extra shop addresses, the detailed-condition table and the language) and returns no `excluded_keys`.
7. The tool keeps running without a settings file: `scratch_settings` sets `fill_keys` to the key of every definition of `accept::shop_definitions()`, among its own entries (before the `--settings` patch). Delete `a26_c13b_…` and `LEFT_ALONE` (U4 writes the row that replaces it).
8. The settings DTO: `fill_keys` in place of `excluded_keys`; regenerate `bindings.ts`. In `ResearchSection.tsx` the picker group "Fields research leaves alone" becomes **"Fields research fills"**, bound to `fill_keys`, nothing preselected; `RESET_KEEPS` names it. The UI fixtures follow.
9. Test helpers: the engine harness (`test_settings()`, `Harness::config()`) and the testkit's `research_test_settings()` choose every field the tests' layouts use, so existing tests keep their outcomes. A test about the list sets `fill_keys` itself and never relies on a helper's list. Then the tests of the table above and this unit's rows.

### Criteria (7 criteria, 20 rows)
Boundary: the named rule; `ResearchService` runs on the research harness with scripted fakes (outcome = the requests the fakes received, steps, written rows, the run's settings snapshot, `status`); `accept` over the HTTP seam stub; the wired desktop command test; `ResearchSection` rendered in a UI test.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| A27·C1(a) | `rules::confidence::field_class` | five fields, each with its key in the chosen list: a Details text field, a dimension field, a Condition choice field, a text field that is also an attribution key, a choice field named by the detailed-condition key | visible, measured, condition, attribution, derived | MC-A4 (changelog 17 a) |
| A27·C1(b) | same | the same five with an empty chosen list | all five never-written | MC-A4 (changelog 17 a) |
| A27·C1(c) | same | a field MC-A1 never writes (the caller passes never-written true) whose key is in the chosen list | never-written | MC-A1 |
| A27·C2(a) | a full run | layout: wood (visible), colour (visible), width (dimension), designer (attribution, set by the test), scratch (condition); `fill_keys` = wood and designer; a read page states a width; a Condition photo exists | no model request of any stage lists colour, width or scratch among its fields; no value is written for the three; the final step lists the three as "not researched"; wood and designer are written with the same value and confidence as in the same run with all five chosen | MA10, MA6 |
| A27·C2(b) | a full run | width holds an `ai` + `suggested` value at `medium` from an earlier run; the list then no longer holds width; a new run finds nothing for it | the row is unchanged (value, confidence, reason, sources) | MA10 (MC-A1 lowering) |
| A27·C2(c) | a full run | `fill_keys` = wood only; the layout has scratch (condition); two Condition photos exist | the model receives no Condition request; no step of stage 6; the run ends `finished` and wood is decided | MC-A7 (stage 6 unit), MA5 |
| A27·C2(d) | a run held at its first model call | started with `fill_keys` = wood; while it is held, the saved list becomes wood and colour; released | the run's settings snapshot lists wood only; colour gets no value and is named "not researched" | MC-A7 (settings snapshot) |
| A27·C3(a) | `rules::run_rules::first_failing` | everything passes except: no chosen field | `NoFieldChosen` | MC-A6 |
| A27·C3(b) | same | no writable field and no chosen field | `NothingToResearch` | MC-A6 (order) |
| A27·C3(c) | same | no chosen field and the day limit reached | `NoFieldChosen` | MC-A6 (order) |
| A27·C3(d) | `arranged`, `run`, `status` | keys set, a Main photo with a source, a layout with fields research may write, `fill_keys` empty | `arranged` answers no; `run` is refused and its text contains "Choose the fields research fills in Settings → Research"; no run row; `status` names code `NoFieldChosen` with that sentence and says a run cannot be started | MA7 |
| A27·C3(e) | `arranged`, `status` | the same, `fill_keys` holding only a key that is not in the item's layout | `arranged` answers no; no run row; `status` names `NoFieldChosen` | MA7 |
| A27·C4(a) | `ResearchSettings::default()` | the default value and its JSON form | `fill_keys` is empty; the JSON has no `excluded_keys` | MC-A4 (changelog 17 a) |
| A27·C4(b) | a fresh app (no `research` slot, both keys set, the fakes under the default provider ids), `run`, `status` | a session with a Main photo with a source | `run` is refused; no run row; `status` names `NoFieldChosen` | MA7 |
| A27·C4(c) | the same fresh app (rule 10) | the saved slot holds only `fill_keys` with one field of the layout; `run`, then one tick | one run, trigger `manual`, state `running`; its settings snapshot has the stages and provider ids of `ResearchSettings::default()`; `status` names no reason | MA7, MA5 (rule 10) |
| A27·C5(a) | `ResearchSettings::from_slot`; then `status` | a saved slot `{"excluded_keys": [<a layout field>], "max_pages": 7}` | the loaded value equals `ResearchSettings { max_pages: 7, ..default }`; its JSON has no `excluded_keys`; with that slot `status` names `NoFieldChosen` | MC-A4 (changelog 17 a) |
| A27·C5(b) | `ResearchApi::settings_reset` | a live slot with no prices, empty stage models and `fill_keys` of two keys | the returned value has the stages and prices of `ResearchSettings::default()`, `fill_keys` with those two keys in order, and no `excluded_keys` | MC-A6 (prerequisites 4 and 6) |
| A27·C6(a) | `accept` (HTTP seam stub replaying captures, as A23·C2(a)) | two pieces, no `--settings` | each run's settings snapshot has `fill_keys` equal to the keys of `accept::shop_definitions()` in their order; both runs were started | MA7 (rule 10) |
| A27·C7(a) | the wired app, commands `settings_get` and `settings_set`; the generated bindings | a settings file whose `research` slot has `excluded_keys` with one key and no `fill_keys`; then a set with `fill_keys` of one key | the get answers `fill_keys` empty and no `excluded_keys`; after the set the saved slot holds that one key; `bindings.ts` contains `fill_keys` and does not contain `excluded_keys` | MA7 |
| A27·C7(b) | `ResearchSection` (UI test; `listDefinitions` and the save mocked) | a shop with five fields, `fill_keys` empty; check two; Save | a group "Fields research fills" lists the five, none checked at first; the save receives `fill_keys` with exactly the two keys; there is no group "Fields research leaves alone" | `[rule]` 08B changelog 17 (a) |

### Delegated choices
How stage 6 is left out (no unit, or a unit committed without a call) · how the harness helpers build their list (a fixed list of the test keys, or the definitions the test's store holds) · the name of the parameter in `field_class`.

### Notes
- Hazard: `ResearchConfig::default()` now has an empty list, so a test that builds a snapshot by hand researches nothing unless its helper fills the list. A test that goes green because nothing was written is the failure to look for; rows that assert "not written" (A27·C2(a), (b), (d)) also assert a written field in the same run or a named step.
- Hazard: `to_dto` (`commands/settings.rs`) turns the engine's settings into the DTO; a DTO that still requires `excluded_keys` fails every `settings_get`. The engine and the DTO change in this unit together for that reason.
- Not built, on purpose: skipping Find, Read and Compare when no chosen field can use a listing, and skipping Labels when no chosen field is visible or attribution. The owner's list has fields of every class, so those calls are always needed there.
- Carried from the R10 review: a test about attribution sets `attribution_keys` itself.

### Unit test commands (run after the unit, then make the `CHECKPOINT (not approved):` commit)
`cargo test -p psm-research --lib rules::confidence` · `cargo test -p psm-research --lib rules::run_rules` · `cargo test -p psm-research --test engine chosen_fields::` · `cargo test -p psm-research` · `cargo test -p psm-research-tool` · `cargo test -p psm-testkit --test suite research::` · `cargo test -p psm-desktop` · `cd apps/desktop && npx vitest run src/features/settings` · `npm run typecheck` · `cargo clippy -p psm-research --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cargo check --workspace`

## Unit 2 — Detailed condition from the levels that have a value

**Unit id:** `R13·U2` · **Rows:** A27·C8–C9 · **Changes app files:** no

### Goal
A piece where research saw only minor scratches and nothing else gets the grade for Minor. Function, Odor and Stability can never be answered from photos; they no longer hold the grade back. **Not in this unit:** the table (it stays the setting R11 built and R12 filled); the low-confidence rule (unchanged).

### Read first
`master_plan_research.md` §6.5, §6.9 · book `08B` MC-A1 (write table), MC-A2 (choice mapping), MC-A4, changelog 17 (b) · evidence "Acceptance measurement 2" (detailed condition 0 right, 0 wrong, 40 open) · the R11 plan, Unit 3 "The rule" · code `crates/research/src/{rules/derived_condition.rs, decide.rs (where `derive` is called)}`, `crates/research/tests/engine/derived.rs`.

### The rule (this replaces "The rule" of the R11 plan, Unit 3, in full; charter rule 5)
- **Setting.** `detailed_condition` = a field key and an ordered list of levels, best first. Each level names a condition value (the text of a choice of the condition level fields, such as "Minor") and either the choice of the detailed-condition field to write, or "counts for nothing" (for a value such as "N/A").
- **Condition level fields** of an item: the fields of its layout whose definition group is `condition` and whose core type is `choice`, in layout order. Whether a level field is in `fill_keys` makes no difference here.
- **The value of a level field** is the live value its row holds in stage 7's writing transaction, after this run's condition writes: a row that is not cleared, of any provenance. A field with no such row has no value.
- **Derivation,** first match wins: (1) the layout has no condition level field → not determined, "no condition levels in this form". (2) no level field has a value → not determined, "no condition level has a value". (3) a value held by a level field is not in the setting's list → not determined, "no grade is set for <value>". (4) every held value counts for nothing → not determined, "no condition level applies". (5) otherwise the worst held value is the one latest in the list; its choice is mapped by MC-A2's choice rule against the field's current choices; no such choice → not determined, "the setting names a choice the field does not have". (6) the mapped choice is written at `low` when any held value is an `ai` value at `low`, else at `medium`. It is never `high`. **A level field with no value takes no part in steps (3) to (6).**
- **Reason** (rule-written, not sent to the model): "From the condition levels: the worst is <value> (<labels of the fields holding it>)." When at least one level field has no value, one sentence follows: " <n> of <m> condition levels have no value." (n = level fields with no value, m = all level fields). **Sources:** none.
- **The write** goes through MC-A1 like every other. **The field itself** is derived only when its key is in `fill_keys` (U1); with an empty `detailed_condition.key` it has no special class.

### Files
Edited: `crates/research/src/{rules/derived_condition.rs, decide.rs}`, `crates/research/tests/engine/derived.rs`.

### Core mechanisms (the only functions with unit tests)
`rules::derived_condition::derive`.

### Ordered tasks
1. `derive`: steps (2) to (6) and the reason as above. The inline tests `a25_c12a`–`f`, `h`–`k` stay as they are (every level filled); `a25_c12g` is deleted.
2. `decide.rs`: nothing but what the rule needs at the call site (the values are already read there, with a field that has no row as no value).
3. The tests: this unit's rows; `a25_c14b` takes the new reason.

### Criteria (2 criteria, 10 rows)
Boundary: the named rule; full runs on the research harness with scripted fakes. The setting, as in A25·C12: None → A, Minor → B, Moderate → C, Major → D, N/A → counts for nothing; the detailed-condition field's choices are A, B, C, D.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| A27·C8(a) | `derive` | two level fields: "Field 0" holds Minor, "Veneer" has no value | B, `medium`, reason "From the condition levels: the worst is Minor (Field 0). 1 of 2 condition levels have no value." | MC-A4 (changelog 17 b) |
| A27·C8(b) | `derive` | two level fields, neither has a value | not determined, "no condition level has a value" | MC-A4 (changelog 17 b) |
| A27·C8(c) | `derive` | three level fields: None, Moderate, and one with no value | C, `medium` | MC-A4 (changelog 17 b) |
| A27·C8(d) | `derive` | one level field with no value, and Minor held as an `ai` value at `low` | B, `low` | MC-A4 |
| A27·C8(e) | `derive` | N/A, and one level field with no value | not determined, "no condition level applies" | MC-A4 |
| A27·C8(f) | `derive` | the setting's list without Major; one field holds Major, one has no value | not determined, "no grade is set for Major" | MC-A4 |
| A27·C9(a) | a full run | the grade field and the two level fields scratch and veneer are in `fill_keys`; two Condition photos; stage 6 reads scratch Minor (sure) and gives no read for veneer | the grade row is B, provenance `ai`, status `suggested`, confidence `medium`, reason "From the condition levels: the worst is Minor (<scratch's label>). 1 of 2 condition levels have no value.", no sources; one `concluded` step names the key | MA6, MA11 |
| A27·C9(b) | a full run | the same list; no Condition photo and no stored level | no row is written for the grade; its `not_determined` step says "no condition level has a value" | MA6 |
| A27·C9(c) | a full run | `fill_keys` holds the grade field and wood, not the level fields; scratch holds a `human` value Minor, veneer has no value; two Condition photos | the grade row is B at `medium`; the model receives no Condition request; scratch and veneer are unchanged | MA10, MA11 |
| A27·C9(d) | a full run | `fill_keys` holds scratch, veneer and wood, not the grade field; stage 6 reads scratch Minor and veneer Moderate, both sure | scratch and veneer are written; no row is written for the grade; the final step names it "not researched" | MA10 (changelog 17 a) |

### Delegated choices
How the count sentence is built · whether `derive` filters the empty levels itself or its caller does.

### Notes
- A25·C14(a) (every level read) keeps its exact reason: with no empty level there is no count sentence.
- A25·C14(c) (every level a human value, no Condition photo) stands.
- The owner's shop: Function, Odor and Stability were answered on 0 of 10 pieces in both measurements, and Missing parts, Veneer and Upholstery often not. After this unit the grade follows whatever was answered.

### Unit test commands (run after the unit, then make the `CHECKPOINT (not approved):` commit)
`cargo test -p psm-research --lib rules::derived_condition` · `cargo test -p psm-research --test engine derived::` · `cargo test -p psm-research` · `cargo clippy -p psm-research --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cargo check --workspace`

## Unit 3 — Settings → Research: what the list means, and the warnings

**Unit id:** `R13·U3` · **Rows:** A27·C10–C13 · **Changes app files:** yes (TypeScript only)

### Goal
The person can see at a glance why research fills nothing, cannot tick a kind of field research never fills, and is told when the designer, maker or detailed-condition field is not on the list. **Not in this unit:** the picker's binding and saving (U1, A27·C7(b)); the Designer and maker suggestion of R11 (unchanged).

### Read first
`docs/intention/evidence/design_handoff.md` · `master_plan_research.md` §6.9 · book `08B` MC-A1 (the kinds of field research never writes), changelog 17 · the R11 plan, Unit 6 · code `apps/desktop/src/features/settings/{ResearchSection.tsx, ShopFieldPicker.tsx}`, `apps/desktop/src/features/settings/__tests__/{r11_research_pickers.test.tsx, r13_research_fills.test.tsx}`.

### Files
Edited: `apps/desktop/src/features/settings/{ResearchSection.tsx, ShopFieldPicker.tsx}`, `apps/desktop/src/features/settings/__tests__/{r11_research_pickers.test.tsx, r13_research_fills.test.tsx}`.

### Core mechanisms (the only functions with unit tests)
None.

### Ordered tasks
1. **Fields research fills** lists only the shop's fields research can fill: a field whose core type is `media_list`, `reference`, `rich_text`, `money` or `rating`, or that is not writable, is not offered (MC-A1). A saved key of such a field is still listed, marked "Research never fills this kind of field", so it can be unchecked. The Designer and maker picker is unchanged.
2. Helper line under the heading: "Research suggests values only for the fields ticked here. A new field in your shop stays empty until you tick it." With nothing ticked: "No field is chosen: research will not fill anything."
3. Under **Designer and maker**: when a chosen designer or maker field is not ticked in "Fields research fills", the line "Not among the fields research fills, so research will not suggest it: <labels>".
4. Under **Detailed condition**: when a field is chosen there and it is not ticked in "Fields research fills", the line "Not among the fields research fills, so the detailed condition will not be worked out." And a helper line under the table: "Levels with no value do not hold the grade back."
5. The reset confirmation lists "Fields research fills" among what is kept (the constant follows from U1; the test is amended here).

### Criteria (4 criteria, 7 rows, all `[rule]`)
Boundary: `ResearchSection` rendered in a UI test with `listDefinitions`, the settings save and the reset command mocked; outcome = what is shown and what the save received.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| A27·C10(a) | Fields research fills | a shop with a text field, a choice field, and one field each of core type `media_list`, `reference`, `rich_text`, `money`, `rating`, and a text field that is not writable; `fill_keys` empty | exactly the text field and the choice field are listed; the Designer and maker picker lists all eight | `[rule]` 08B MC-A1 |
| A27·C10(b) | Fields research fills | the same shop; `fill_keys` holds the `rating` field's key | that field is listed, checked, with "Research never fills this kind of field"; unchecking it and saving sends `fill_keys` empty | `[rule]` 08B MC-A1 |
| A27·C11(a) | Fields research fills | `fill_keys` empty; then one field is checked | first "No field is chosen: research will not fill anything." is shown; after the check it is not | `[rule]` 08B changelog 17 (a) |
| A27·C12(a) | Designer and maker | `attribution_keys` holds the field labelled "Designer"; `fill_keys` holds another field | "Not among the fields research fills, so research will not suggest it: Designer" is shown in the Designer and maker group | `[rule]` 08B changelog 17 (a) |
| A27·C12(b) | Designer and maker | the same; then "Designer" is checked in "Fields research fills" | the line is gone | `[rule]` 08B changelog 17 (a) |
| A27·C12(c) | Detailed condition | a detailed-condition field is chosen; `fill_keys` does not hold it; then it is checked in "Fields research fills" | first "Not among the fields research fills, so the detailed condition will not be worked out." is shown in the Detailed condition group; after the check it is not | `[rule]` 08B changelog 17 (b) |
| A27·C13(a) | Reset | click the reset button | the confirmation lists six kept things, "Fields research fills" among them, and none is "Fields research leaves alone" | `[rule]` R10 review (carried) |

### Delegated choices
How the picker is told which fields to leave out (a property of `ShopFieldPicker`) · the layout of the lines.

### Notes
- The owner's shop has 68 product fields; about a dozen are of kinds research never fills (the `shopify.*` reference fields, the review fields, the files field).
- For the owner's shop the narrative lists what to tick (the list of U4 task 1).

### Unit test commands (run after the unit, then make the `CHECKPOINT (not approved):` commit)
`cd apps/desktop && npx vitest run src/features/settings` · `npm run typecheck`

## Unit 4 — The tool: the owner's list, and a check on a few pieces

**Unit id:** `R13·U4` · **Rows:** A27·C14–C15 · **Changes app files:** no

### Goal
The measurement tool's owner settings carry the owner's list in place of the six left-alone fields, and a short run on a few pieces shows that research now writes only chosen fields and that the detailed condition is derived. **Not in this unit:** a third 42-piece measurement; any change to the engine (a defect found is a finding for the orchestrator).

### How this unit runs
The implementer builds tasks 1–2 and their two automated rows. **Task 3 is the orchestrator's.** Spending ceiling **USD 2** (`--ceiling-usd 2`; four pieces at about USD 0.11 each, evidence "Acceptance measurement 2"). Keys come from `~/.psm_research.env` (`master_plan_research.md` §7.6), read by the tool, never by a session. Out folder `~/psm-research-bench/a27`, outside the repo.

### Read first
`master_plan_research.md` §3 (Rule 9, Keys, Spending ceilings), §7.6 · the R12 plan, Unit 4 (how the second measurement was run) · evidence "Acceptance measurement 2" · `docs/test-items/accept_settings_owner.json` · code `crates/research-tool/src/accept.rs`, `crates/research-tool/tests/tool/u4_accept.rs`, `crates/testkit/src/fixtures/definitions.rs`.

### Files
Edited: `docs/test-items/accept_settings_owner.json`, `crates/research-tool/tests/tool/u4_accept.rs`; by the orchestrator's run: `docs/intention/evidence/research_providers.md` (a short section "Chosen fields check (R13)") and this plan's Review log. Outside the repo: `~/psm-research-bench/a27/`.

### Core mechanisms (the only functions with unit tests)
None.

### Ordered tasks
1. `docs/test-items/accept_settings_owner.json`: remove `excluded_keys`; add `fill_keys` with these 22 keys in this order (each verified by the planner against `crates/testkit/src/fixtures/definitions.rs`, the tool's shop definitions): `custom.designer`, `custom.manufacturer`, `custom.wood_type`, `custom.country`, `custom.years`, `custom.color_furniture`, `custom.quantity`, `custom.condition_cracks`, `custom.condition_dents_chips`, `custom.condition_function`, `custom.condition_missing_parts`, `custom.condition_mold`, `custom.condition_odor`, `custom.condition_scratch`, `custom.condition_stability`, `custom.condition_stains_discoloration`, `custom.condition_upholstery`, `custom.condition_veneer_damage`, `custom.condition_water_damage`, `custom.condition_report`, `custom.condition_report_description`, `custom.detailed_condition`. `extra_shop_hosts` and `detailed_condition` stay as they are.
2. The two automated rows.
3. `[manual]` (orchestrator) `accept --pieces docs/test-items/beyo_test_samples_1.json --data ~/psm-research-bench/a27 --only 6,12,28,33 --settings docs/test-items/accept_settings_owner.json --ceiling-usd 2`. Over the four runs' `writes.json` and steps, record in the evidence doc section and in this plan's Review log: (1) how many values were written for a key outside the 22 (expected 0), naming any; (2) per piece, the condition levels written, the detailed condition written and whether it is the table's grade for the worst written level; (3) what was spent. Anything that contradicts a contract is a finding with the run id, not fixed here.

### Criteria (2 criteria, 3 rows, 1 `[manual]`)
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| A27·C14(a) | `accept` (HTTP seam stub replaying captures, as A23·C2(a)) | two pieces with `--settings docs/test-items/accept_settings_owner.json`; and the same two without it | with the file: each run's settings snapshot has `fill_keys` equal to the file's 22 keys in order and the three extra shop addresses; every field key of every model request the stub received is one of the 22, and none is `custom.detailed_condition`. Without the file: at least one field key outside the 22 reaches a model request | MA10 |
| A27·C14(b) | the file `docs/test-items/accept_settings_owner.json` against `accept::shop_definitions()` | the file as committed | it has no `excluded_keys`; `fill_keys` has 22 entries without repeats; each is the key of a definition of the tool's shop | MA6 |
| A27·C15(a) | evidence doc and Review log `[manual]` | the four runs of task 3 | the three answers of task 3, each with its count | MA10, MA11, HA6 |

### Delegated choices
None.

### Notes
- Two of the 22 keys (`custom.color_furniture`, `custom.condition_report`) have no value in `docs/test-items/beyo_test_samples_1.json`, so a measurement cannot score them; they are in the shop definitions and research may fill them.
- `custom.condition_report` and `custom.condition_report_description` are text fields in the Condition step: they are read by stage 6 and are not condition level fields (a level field is a choice).
- The pieces of task 3: 6 and 33 are the look-alike pieces of the first measurement, 12 has a name-form difference, 28 has no designer in the shop. Any four would do; these have the most to show.
- The test stub answers every model call with a capture the engine refuses (as A26·C13 did), so A27·C14(a) proves the requests and the snapshot, not written values; written values are task 3's.

### Unit test commands (run after the unit, then make the `CHECKPOINT (not approved):` commit)
`cargo test -p psm-research-tool u4_accept::` · `cargo clippy -p psm-research-tool --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cargo check --workspace`

## Closing gate (once, after the last unit; replaces the template's closing gate)
This milestone changed app files, so the full gate runs once: `cargo test --workspace` (it includes the research and tool crates) · `cargo clippy --workspace --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `cd apps/desktop && npm run typecheck` · `npm test`. One cargo command at a time.

## Review mode
The reviewer reads the units in order and runs, once: `cargo test -p psm-research` · `cargo test -p psm-research-tool` · `cargo test -p psm-testkit --test suite research::` · `cargo test -p psm-desktop` · `cd apps/desktop && npx vitest run src/features/settings src/features/form` · clippy, fmt, typecheck. The implementer's full-suite stamp stands. Only B1–B4 block. The reviewer also checks that every test of "Tests that pin what this plan retires or changes" was handled as the table says, that no deleted test took a still-valid row with it, and that no test turned green only because its run researched nothing (U1 Notes, first hazard). The `[manual]` row A27·C15(a) is judged from the evidence doc section and the Review log entry when the orchestrator has run task 3; before that it is open. Notes go to `docs/implementation/backlog_research/R13_research_chosen_fields.md`.

## Manual try-out (the narrative is built from this)
On scratch data (`PSM_DATA_DIR=$HOME/psm-research-dev npm run tauri -- dev`), shop connected, keys entered:
1. Open an item's form and its research panel before touching the settings: it says "Choose the fields research fills in Settings → Research", and "Arranged — next" starts no research.
2. Settings → Research → **Fields research fills**. The dimensions, weight and damage details are simply left unticked; the review, price and location fields can be left unticked too, and files and reference fields are not offered at all. Tick: Designer, Manufacturer, Wood type, Country, Years, Color, Quantity, the twelve condition levels, Condition report, Condition Report Description, Detailed condition. Save.
3. Untick Designer: a line under **Designer and maker** says research will not suggest it. Tick it again.
4. Shoot a piece with a few main photos and two condition photos, sort them, press **Arranged — next**. In the form: only ticked fields have suggestions; price, link, location and the Etsy description are untouched.
5. Detailed condition has a grade although Function, Odor and Stability are empty; its reason names the worst level and says how many levels have no value.
6. Add a field to the list afterwards: nothing starts by itself; **Run research** on the item fills it.

## Review log
