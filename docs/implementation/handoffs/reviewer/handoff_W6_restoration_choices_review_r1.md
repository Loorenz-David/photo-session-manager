---
plan: W6_restoration_choices
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-19
actor: opus (pipeline-reviewer)
---
# Handoff — W6_restoration_choices, review r1

**Gate:** passed. Tracker IMPLEMENTED; checkpoint `f988e5b` is in `git log`; intention RATIFIED; owner card 1
answered A (plan Review log).

## Verified correct
- **Perimeter.** `git show --stat f988e5b` lists exactly the handoff's 12 declared paths. The one file outside the
  plan's Files list (`crates/research/src/writer/run.rs`) is disclosed and necessary, because the check context is
  built there. The updated older tests (`engine/w1_note.rs`, `engine/w2_writer_run.rs`, `suite/p13_publish_runs.rs`)
  only add the new key to expected lists or move a count from 5 to 6. No assertion was weakened.
- **Checks (run once, one cargo command at a time, Review mode):**
  | Command | Result |
  |---|---|
  | `cargo test -p psm-research --lib -- writer::postcheck rules::write_table` | 46 passed / 0 failed |
  | `cargo test -p psm-testkit --test suite -- research::w6_restoration:: p09_local_export:: p10_attributes:: p13_publish_runs::` | 81 passed / 0 failed (all six W6 tests listed) |
  | `cargo clippy -p psm-core -p psm-research -p psm-testkit --all-targets -- -D warnings` | clean |
  | `cargo fmt --all -- --check` | clean |
  The implementer's full-suite stamp stands: suite 1039/0, engine 328/0, lib 259/0, workspace clippy, typecheck, npm 661/0.
- **Rows (production read, then test):**
  - C1(a): `local.restoration` is in `builtin_definitions()` (MultiChoice, "Restoration done", Condition, the 14 WC8-R choices in order) and in `ALWAYS_PRESENT`. The test stores a two-key Condition preset and asserts the exact key list, label, type and choices.
  - C1(b): stored by the existing MultiChoice rule (`values.rs`). The test asserts the exact stored JSON, `valid` and provenance human.
  - C2(a): `write_set` walks only shop definitions. The test links one shop text field so the publish really writes attributes, then asserts the writes are exactly `[custom.material]` and the step is `Done` (H1).
  - C2(b): asserts `item.json` `attributes["local.restoration"] == ["Sanded","Oiled"]`.
  - C3(a): `never_written` returns true for the key and false for the same definition under another key, so the test checks the key and not the type or group.
  - C3(b): the fakes propose Oiled. The test asserts the run finishes with no stored row, and that the last decide step lists the key with "Restoration done: not researched" (MA10).
  - C4(a): reads the recorded writer request and asserts `decision: "typed"`, `fact: true`, `value: "New foam"` (WC2).
  - C5(a)–(c): `RESTORATION_PHRASES` is the WC8-R table verbatim in both languages. Rule 11 runs only on `condition_report`, as an NFC-lowercased substring match (WC5's phrase rule), with one hit per missing choice naming it (owner card 1). The tests assert no hit, exactly one hit on `condition_report` naming "Oiled" while the description says "sanded and oiled", and a Swedish no-hit that would fail on the English phrases.
  - `check_context` fills the choices only when the field is in the writer input as a fact.
  Every row's test observes the outcome at the row's boundary and would fail on a wrong one.
- **Assumptions 1–4:** none contradicts the intention.
  - 1: always-present built-ins come at the end of their step when there is no preset (WC8.1). The Product order is unchanged.
  - 2: the own-name fallback is unreachable with the fixed list.
  - 3: matches the Delegated choice. The research pickers offer only shop fields (`shopFieldsOf`).
  - 4: follows WC1.1 / WC2 ("every Details and Condition layout key").
- **H1–H8:**
  - H1: nothing is published.
  - H2–H5: not touched.
  - H6: the field works with no shop.
  - H7: no device, app or platform concept in core. The choices are domain vocabulary (see note 3).
  - H8: services only, no Tauri.
  No live service is called and no secrets are added.
- **Guide:** `docs/guide/02_architecture.md` item 7 is accurate.

## Findings
None blocking.

## Notes written to backlog
Written to `docs/implementation/backlog_writer/W6_restoration_choices.md`, the file the plan's Review mode names for
the writer track (W5 precedent), not `docs/implementation/backlog.md`:
1. "soiled" satisfies the Oiled phrase (substring match per WC5). Fix in the phrase table or with a word-boundary match.
2. The default instruction asks for "the restoration in one sentence" in the condition report, not each step, while rule 11 checks the report. This invites a paid retry. Needs a wording change in the instruction.
3. The 14 choices are furniture-workshop constants in a core module. Not H7, but tied to one trade.
4. The plan's unit test commands need `--` before multiple filters (planner template).
5. The tracker line named `codex` as actor while the implementer was Claude Opus (record only).

## Lessons for the plans
- Writer-track plan templates: write multi-filter test commands as `cargo test … -- <f1> <f2>`.
- Prompt vs plan: the review prompt names `docs/implementation/backlog.md`, while writer-track plans name `backlog_writer/<phase>.md`. Align the prompt template with the track.

## Write perimeter
- `docs/implementation/plans/W6_restoration_choices.md`: Tracker line and an appended Review log entry only
- `docs/implementation/backlog_writer/W6_restoration_choices.md` (new)
- `docs/implementation/narratives/W6_restoration_choices.md` (new)
- this handoff

No production or test file was edited, not even temporarily. No probes were run and no state was changed. Nothing
was committed.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
