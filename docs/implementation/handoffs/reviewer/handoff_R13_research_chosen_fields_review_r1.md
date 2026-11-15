---
plan: R13_research_chosen_fields
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-15
actor: opus
---

# Review R13_research_chosen_fields, round 1: APPROVED_WITH_NOTES

## Verified correct

**Gate.** The tracker read IMPLEMENTED. The checkpoints U1 `e80ce98`, U2 `f877640`, U3 `4d398b0`, U4 `f65c0bc` and closing `9330f3c` are all in `git log`.

**Perimeter.** `git show --stat` of each checkpoint, read in order, gives exactly the handoff's declared files. Nothing is extra and nothing is missing. `crates/integrations/src/lib.rs` is outside U1's named files. It is disclosed, and it is required by U1 task 1 ("not written back"). The desktop `settings_set` change is disclosed (backlog note 1).

**Checks** (once, on clean tree `5c33798`, the plan's Review mode, one cargo command at a time):

| Check | Result |
|---|---|
| `cargo test -p psm-research` | 163 lib + 233 engine + 1 doc passed |
| `cargo test -p psm-research-tool` | 30 passed |
| `cargo test -p psm-testkit --test suite research::` | 77 passed |
| `cargo test -p psm-desktop` | 26 passed |
| `npx vitest run src/features/settings src/features/form` | 127 passed |
| `npm run typecheck` | clean |
| `cargo clippy` (touched crates, `--all-targets -D warnings`) | clean |
| `cargo fmt --all -- --check` | clean |

The implementer's full workspace stamp stands.

**Production, per unit**
- **U1.**
  - `field_class`: MC-A1 never-written, then not in `fill_keys` (treated as never-written), then derived, condition, attribution, measured, visible. Every stage asks it through `class_of`/`fields_of`.
  - `chosen_field` is computed only over MC-A1-writable fields, so "nothing to research" keeps its meaning.
  - `NoFieldChosen` sits between `NothingToResearch` and `DayLimit`. It blocks creation and carries the plan's sentence.
  - Stage 6 is skipped when no field has class condition.
  - Stage 7's identity names `fill_keys`, and `RESEARCH_FINGERPRINT_VERSION` stays 2. `settings_reset` keeps `fill_keys`.
  - `from_slot` ignores `excluded_keys` (no `deny_unknown_fields`). The save path re-serializes the slot, which drops the key.
  - The DTO and the generated bindings carry only `fill_keys`.
  - The tool's `scratch_settings` chooses every `shop_definitions()` key before the `--settings` patch. The testkit's `research_test_settings` chooses every testkit shop key.
- **U2.** `derive` filters out empty levels before steps (3) to (6). It gives "no condition level has a value" when no level holds one. The count sentence " n of m condition levels have no value." appears only when a level is empty, and the low-confidence rule looks only at filled levels. `decide.rs` reads values from non-cleared rows of any provenance. A cleared level is excluded (`AttributeService::clear` marks it `Cleared`).
- **U3.** The picker's eligibility test (not product, writable, not media_list/reference/rich_text/money/rating) equals `write_table::never_written` for shop fields. A saved ineligible key is still listed with the plan's text. The helper texts and the three warning texts match the plan word for word. `RESET_KEEPS` names "Fields research fills".
- **U4.** The owner file holds the plan's 22 keys in order. A `jq -S` diff shows `extra_shop_hosts` and `detailed_condition` unchanged (only reformatted).

**Tests, per row.** All 39 automated rows have a test at the row's boundary that fails on a wrong outcome:
- A27·C1(a)–(c), C3(a)–(c) and C8(a)–(f) run on the named rule with exact expected values.
- C2(a): every field-carrying model request (observe, read, labels, condition, decide) puts its fields under `input.fields`, so the test's check covers every stage. Wood and designer are compared with an all-chosen run.
- C2(b): the row is unchanged, and the run's own "not researched" step for width proves that width was classed out.
- C2(c): `stage == "condition"` is the real stage name.
- C2(d): the live list is changed between start and execute. Designer is scripted, so reading the live list would write it.
- C3(d)/(e), C4(b)/(c), C5(a)/(b), C6(a), C7(a)/(b), C9(a)–(d), C10–C13 and C14(a)/(b): each observes the stated outcome.
- C14(a) also shows that a key outside the 22 reaches a request without the file, so the check is not vacuous.

**Hazard check (U1 Notes).** The engine harness lists `MAKER, WOOD, DESIGNER, WOODS, WIDTH, SCRATCHES, SUBTITLE`, which is every custom key the engine tests' layouts use. `derived.rs` adds the grade and veneer. The `test.*` MC-A1 fields are exercised only through the write layer (`writes.rs`) or as the sole layout field (`NothingToResearch`), and neither depends on `fill_keys`. The tool's `five_field_run` sets its own list. No older test turned green because its run researched nothing.

**Pin table.** Handled as the table says, with two exceptions that leave no row uncovered (backlog note 2). No deleted test took a still-valid row with it:
- `a25_c8a`/`c13b` → A27·C1
- `a25_c8b`/`c8c`/`c8d` → A27·C2(a)/C3(d)/C2(b)
- `a25_c12g` → A27·C8(a)
- `a26_c13b` → A27·C14(a)
- `a24_c1a` → A27·C4(b)/(c)
- `a25_c29a` → A27·C7(b)

**Assumptions.** None contradicts 08A/08B or changelog 17:
1. Stage 6 is omitted entirely. This is a delegated choice.
2. A saved unfillable key stays visible. This is U3 task 1.
3. A save normalizes the whole research slot. This follows from U1 task 1.
4. The harness chooses its fixture keys. This is a delegated choice.

**Hard constraints.**
- H1: no external write.
- H2/H3: no files or pixels touched.
- H4: stage 7's fingerprint changes with the list, so no old decision is reused under the new rule.
- H5: no UI automation of other apps.
- H6: nothing changes for the workflow without commerce.
- H7: no shop field names in core. The engine ships `fill_keys` empty, and the owner's keys live only in `docs/test-items`.
- H8: the commands stay thin.
- No secrets in the diff, and no live service in any test.

**Open.** A27·C15(a) `[manual]` stays open until the orchestrator runs U4 task 3 (USD 2 ceiling).

## Findings
None blocking (B1–B4).

## Notes written to backlog
In `docs/implementation/backlog_research/R13_research_chosen_fields.md` (the plan's named notes file):
1. `settings_set` now runs on the async runtime without `spawn_blocking`. It does not crash, but an app rebuild during a save holds an async worker.
2. Pin table: `a13_c1a`/`a13_c1b` (lifecycle) and `a25_c2b` (arranged) were not amended to include the new prerequisite. They are covered at the same boundaries by the `run_rules` tests and `a27_c3d`/`e`.
3. `a25_c32a` was renamed to `a27_c13a`, so the A25·C32(a) id is gone from test names, although its assertions remain.
4. A27·C2(a) does not script the plan's "a read page states a width". The test still discriminates through the requests and the "not researched" step.
5. A legacy `excluded_keys` stays in the file until the next research save. It is ignored on load.

Lessons for the plans: a pin table should name, per test, whether it is "amended" or "covered elsewhere". The two skipped amendments here were harmless only because the new rows restate the full case.

## Write perimeter of this session
- `docs/implementation/plans/R13_research_chosen_fields.md` (Tracker line and the Review log entry only)
- `docs/implementation/backlog_research/R13_research_chosen_fields.md` (new)
- `docs/implementation/narratives/R13_research_chosen_fields.md` (new)
- this handoff

No production or test file was edited. No probe touched any file. No database or state side effect. Nothing committed.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
