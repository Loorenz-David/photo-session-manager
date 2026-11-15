---
plan: research
role: plan
round: 4
state: DONE
date: 2026-11-15
actor: Claude Fable 5.2 (pipeline-planner, implementation-planner doctrine; orchestrated)
---
# Research agent — plan for R13 (research fills only the chosen fields; detailed condition from the filled levels)

**Summary.** One milestone plan written in the format of R11 and R12 and registered: `R13_research_chosen_fields`
(4 units, 15 criteria, 40 rows `A27·…`, 1 of them `[manual]`; changes app files; depends on R12; `projection: no`,
`difficulty: high`). Gate: intention map RATIFIED (2026-10-24), 08A RATIFIED and 08B CONTRACTS RATIFIED (2026-11-12);
08B changelog 17 is the owner decision implemented, its part (b) confirmed by the owner on 2026-11-15 (orchestrator's
message during this session). Documents only: no code, no cargo, no external call, no commit.

## ⚠ OWNER DECISIONS REQUIRED (0)
Nothing needs the owner. One note for the owner is at the end of this file ("Fields the owner may want to add").

## Write perimeter (complete)
| File | Change |
|---|---|
| `docs/implementation/plans/R13_research_chosen_fields.md` | new |
| `docs/implementation/master_plan_research.md` | §0 (ids, row ids), §3 (`[manual]`, real calls, ceiling USD 2, projection), §4 (one row, totals sentence), §5 (graph, schedule step 5, merge order, ledger coverage), new §6.9, §7.2 (one row), §9 (v12 line) |
| `docs/implementation/research_row_mapping.md` | appended: "Rows added by R13" (plan number A27; rows of A05, A12, A13, A23, A24, A25, A26 retired, amended or kept with an amended input; every new row) |
| `docs/implementation/phases_research.txt` | one id and one comment line appended |
| `docs/implementation/handoffs/reviewer/handoff_research_plan_r4.md` | this file |
No code, no test, no evidence doc, no book chapter, no `STATUS.md`, no tracker line of another plan, no archgraph state. Nothing read outside the worktree except the doctrine files. Scratch scripts only in the session scratchpad.

## Counts (computed from the plan file)
| Unit | Criteria | Rows | Changes app files |
|---|---|---|---|
| U1 the list of fields research fills (engine, settings DTO, bindings, the picker's binding, the tool's own list) | 7 | 20 | yes |
| U2 detailed condition from the levels that have a value | 2 | 10 | no |
| U3 Settings → Research: what the list means, and the warnings | 4 | 7 (all `[rule]`) | yes (TypeScript) |
| U4 the tool's owner list and a check on four pieces | 2 | 3 (1 `[manual]`) | no |
| **R13** | **15** | **40** | closing gate: full gate |

## Book text superseded (the plan carries the table; the orchestrator folds it into 08B)
08B MC-A1 "Fields never written" (plus: a key not in `fill_keys`) · MC-A4 "Field class" as R11 left it (the step "left alone" becomes "not chosen") · changelog 12 (a) and the R11 plan's U2 tasks 1–2, U6 task 3 (one picker, "Fields research fills") · MC-A6 prerequisites (a new one, "no field chosen", after "nothing to research") · MC-A7 Units row 6 (stage 6 only when a condition field is chosen) · the R11 plan's U3 rule, step (2) and the `excluded_keys` sentence (levels with no value are left out; restated in full in R13·U2).

## Decisions made by planning (no card; each is in the plan's "Points recorded by planning" and the v12 log line)
1. **Fresh-install default: empty; research then does nothing and says so.** HA5 forbids shop keys in the engine, and changelog 17 (a) says a new shop field is never filled until the person adds it. New prerequisite `NoFieldChosen`, sentence "Choose the fields research fills in Settings → Research" (the panel shows any reason sentence already; no panel change). Consequence stated in the plan: **R10's row A24·C1(a) is amended** — the shipped defaults alone no longer start a run; defaults plus one chosen field do (A27·C4(b), (c)). Not raised as a card because the owner's own sentence decides it; the default is one line if the owner wants otherwise.
2. **`fill_keys` is the only gate.** `attribution_keys` and `detailed_condition.key` say how a chosen field is decided; a designer, maker or detailed-condition field that is not in the list is not researched or derived. The screen warns (U3).
3. **No conversion of an old settings file.** A saved `excluded_keys` loads, is ignored, is not written back.
4. **Stage 6 is not called when no condition field is chosen** (new; see contradiction 3 below).
5. **The detailed condition's reason gains one sentence when levels are empty** ("<n> of <m> condition levels have no value."); grade and confidence are not lowered. The low-confidence rule is unchanged.
6. **Fingerprint version stays 2** (stage 7's identity changes content, so nothing old can be reused; R11's precedent).
7. **The tool without `--settings` chooses every field of its shop definitions** (the unfiltered baseline, as before); the owner's file narrows it to 22 keys.
8. **The check on a few pieces** is one `[manual]` row (A27·C15(a)): pieces 6, 12, 28, 33, ceiling USD 2, folder `~/psm-research-bench/a27`, run by the orchestrator.

## Where the code or the artifacts contradict the scope given
1. **The engine setting and the app's settings DTO cannot change in different units.** `to_dto` (`apps/desktop/src-tauri/src/commands/settings.rs`) deserializes the engine's settings into a DTO that requires `excluded_keys`; removing the field in the engine alone fails every `settings_get`. So U1 spans the engine, the DTO, the regenerated bindings and the picker's binding, and the milestone changes app files (full gate at the end).
2. **An empty default list breaks every existing test that expects a write** (`ResearchConfig::default()`, the engine harness's `test_settings()`, the testkit's `research_test_settings()`, the tool's `scratch_settings`). U1 task 9 makes the helpers choose every field their layouts use; the plan's hazard note and Review mode ask the reviewer to look for tests that went green because nothing was researched.
3. **Today a stage is called even when its field list is empty** (`stages.rs`, `condition`). With an allow-list a person can choose no condition field, and the condition photos would still go to the most expensive model. U1 task 4 and row A27·C2(c) stop that for stage 6 only; the same for Find/Read/Compare and Labels is named in U1's Notes as not built.
4. **The keys in `docs/test-items/beyo_test_samples_1.json` carry no `custom.` prefix** (`metafields` is a map of bare keys). All 22 owner keys exist in `crates/testkit/src/fixtures/definitions.rs` (the tool's shop definitions). Two have no value in the samples file and so cannot be scored: `custom.color_furniture`, `custom.condition_report`. None could not be found.
5. **`custom.condition_report` and `custom.condition_report_description` are in the Condition step** (the fixture groups every `custom.condition_*` key there); they are text, so stage 6 reads them and they are not condition levels.
6. **Changing the list does not start research.** "Arranged — next" creates a run only when photos changed (R11); after adding a field, an already researched item needs "Run research". Stated in the plan and its try-out; not changed.
7. **A run started by an earlier build has no list in its snapshot** and writes nothing more after the upgrade. Scratch installs only; stated, not handled.

## Reviewer notes of R11 and R12 (backlog_research)
None folded. Each touches a file R13 does not otherwise change or is a book matter: the fake model's `photo:<id>` fallback (`fakes.rs`, `r11_arranged.rs`), A25·C22(a)'s dead status clause (`r11_arranged.rs`), A25·C24(d) and C33(a) (UI tests of other rows), stage 7 past the cap for a derived-only decision (book), marks before a failing preview (book), the default detailed-condition table naming the owner's field (owner-directed). Two plan lessons were applied in writing: the pin table was built by grep for the setting, the function, the reason code and the retired texts, and A27·C5(a) states the whole loaded value (the lesson of the A25·C7(a) note).

## For the orchestrator
- Fold the plan's "What this plan supersedes in the book" table into 08B before or with R13's launch.
- `templates/implement.md` needs no change. R13 is launched like R12: the implementer stops after U4 task 2; task 3 is yours (four pieces, USD 2).
- U1 regenerates `bindings.ts` (`cargo test -p psm-desktop export_bindings`); `master_plan_research.md` §5's sentence "No other milestone regenerates `bindings.ts`" was left as it is and now has this third exception, recorded in §6.9.
- The scratch install's settings need the owner's list entered once in Settings → Research after R13 (the try-out lists what to tick).

## Fields the owner may want to add (note, not a card; nothing was added)
These fields describe the piece and are in the shop definitions, but are not in the owner's list of changelog 17: **Shape** (`custom.shape`), **Mark / Stamp** (`custom.mark_stamp`), **Contains Key** (`custom.contains_key`), **Extension Quantity** (`custom.extension_quantity`), **Extensions Quantity** (`custom.extensions_quantity`), **Extension Type** (`custom.extension_type`). In the samples file only `extension_quantity` (4 pieces) and `extension_type` (3 pieces) have values; the others have none, so a measurement could not score them. Mark / Stamp is what the label-reading step sees most directly. They can be ticked in Settings → Research at any time; adding them to the tool's file is one line each.
