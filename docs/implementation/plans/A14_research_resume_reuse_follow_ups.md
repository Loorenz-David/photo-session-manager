---
phase: A14_research_resume_reuse_follow_ups
title: Research survives a crash without doing or paying for anything twice, and when you add or change photos it redoes only the part that changed
difficulty: high
projection: yes
depends_on: [A13_research_run_lifecycle]
---
# A14 — Kill-and-resume, reuse, follow-up runs (08B MC-A7 Resume, Reuse, Follow-up trigger set; H4; MA4, MA14)

## Goal
Make a run resumable at any instant with the same end state as an uninterrupted run; copy unchanged units from the
session's latest earlier run in `auto` and `follow_up` runs; queue a follow-up when, and only when, the
photo-derived inputs changed. **Not in this phase:** anything in the UI; retention (A21; here "the file is
present" is the only completeness test).

## Read first
`master_plan_research.md` §6.4, §6.5, §7.5 · book `01` (H4), `14` (MA4, MA14), `08A` §5 (New photos after a run),
§10, `08B` **MC-A7** (all), MC-A6 (the event table's follow-up row; "a run is running" column), MC-A8 (an `unknown`
call is charged the fallback) · code `services/research/{stages.rs, decide.rs, runs.rs, gateway.rs}`,
`mechanisms/research/{fingerprint.rs, run_rules.rs}`, `crates/testkit/src/fake_research.rs` (`before_call` hook),
`TestApp::kill_and_restart`.

## Dependencies and gate
Tracker row and gate: `master_plan_research.md` §4. Notes: `backlog_research.md`.

## Files expected to change
`crates/core/src/services/research/{stages.rs, decide.rs, runs.rs, gateway.rs}`,
`crates/core/src/mechanisms/research/run_rules.rs` (`follow_up_needed`), `crates/core/src/domain/research.rs`
(`InputSnapshot`), store methods for the latest earlier run's outputs, new
`crates/testkit/tests/suite/research/a14_research_resume_reuse_follow_ups.rs` (+ `mod` line), own tracker row.

## Core mechanisms (unit tests allowed)
`run_rules::follow_up_needed`.

## Ordered tasks
1. **Resume.** `execute` continues at the first stage with an uncommitted unit; a committed unit never runs again;
   a call row still `sent` with no response is closed as `unknown` (charged the fallback) and re-issued as the next
   attempt; stage 7 re-applies its writes through `ai_suggest`, where the same-content check makes a repeat a no-op.
2. **Reuse.** In an `auto` or `follow_up` run, a unit whose fingerprint equals a committed unit's in the latest
   earlier run of the session is copied with `reused_from_run_id`, with no call and no step, provided every file
   its output references is still present. A `manual` run reuses nothing.
3. **Input snapshot and trigger set.** At start the run stores `input_snapshot`: the set of stage 1 unit photo
   identities (photo id, composition revision), the cover identity, the stage 6 Condition identity.
   `follow_up_needed(latest, now)` is true iff one of the three differs. `session_changed` (A13) queues a
   `follow_up` then, per MC-A6's table: none queued or running → create; one queued → keep it (its start moves with
   the quiet period); one running → create unless one is queued.

## Criteria (6 criteria, 17 rows)
Boundary: `TestApp::with_research()`; a kill is `kill_and_restart` from the fake's `before_call` hook or between
ticks; "equals the uninterrupted run" compares, against a reference run of the same script on a second `TestApp`:
the stage outputs (stage, unit key, fingerprint, output), the steps (seq, kind, summary, field keys) and the
`attribute_values` rows (all columns but `updated_at`).
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | kill before a unit's call | kill when the second observe call arrives (before its record is committed) | equals the uninterrupted run; the first observe unit is requested once in total | MA14 |
| C1(b) | kill with a call in flight | kill inside the hook of the first read call (record `sent`, no response) | that call row becomes `unknown`, charged the fallback; a second row with the next attempt number answers; outputs, steps and values equal the uninterrupted run (cost differs by exactly the fallback) | MA14, MA15 |
| C1(c) | kill between units | kill after the find unit committed, before any read | equals the uninterrupted run; the search fake has one request in total | MA4 |
| C1(d) | kill before Decide | kill after stage 6 committed | equals the uninterrupted run | MA4 |
| C1(e) | kill inside Decide | kill when the stage 7 call arrives | equals the uninterrupted run; each written key has exactly one `concluded` step | MA4 |
| C1(f) | kill after the writes | kill after the stage 7 commit, before the run row is set `finished` | run `finished`; no step and no attribute row changed by the resume | MA14 |
| C2(a) | repeat of stage 7 | `execute` called again on a finished run's stage 7 | no attribute row's `updated_at` changes; no step is added | MA14 |
| C3(a) | follow-up, new Main photo | a finished run; a third main photo is added (no marking) | the follow-up calls the model for one observe unit and for stage 7 only; every other unit row carries `reused_from_run_id`; no step repeats an earlier run's `looked` for the old photos | MA14 |
| C3(b) | follow-up, new Condition photo | a condition photo is added | calls: stage 6 and stage 7 only | MA14 |
| C3(c) | follow-up, new cover | main position 1 changes by reorder | calls: find, read, compare and stage 7; observe units are reused | MA14 (R-IA17) |
| C3(d) | follow-up, recompose | one main photo is re-framed | calls: that photo's observe unit and stage 7 (plus find, read and compare iff it is the cover) | MA14 |
| C3(e) | missing file | the stored copy a reusable unit references is deleted from disk | that unit is run again, not copied | MC-A7 |
| C4(a) | manual run | Run again on an unchanged session | every unit is called again; no row carries `reused_from_run_id` | MC-A7 (R-IA18) |
| C5(a) | triggers | after a finished run, each alone: a main photo added; a main photo classified `deleted`; a `deleted` photo restored to main; a photo re-framed; main → condition; condition → main; a new cover by reorder | one `follow_up` run queued, seven times | MA14 (R-IA17) |
| C5(b) | non-triggers | each alone: a Capture One retouch (new `adjusted_gen` only); a stage's model changed in settings; two non-cover main photos swapped | no run queued, three times | MC-A7 |
| C6(a) | changes during a run | three photo changes while a run is `running` | exactly one `follow_up` queued; it starts after the run ends and the quiet period has passed | MA13 |
| C6(b) | `follow_up_needed` | two snapshots differing only in the order the photo identities are listed | false | MC-A7 |

## Delegated choices
How the reference run is built in the tests (a helper running the same script on a fresh `TestApp`) · where
`input_snapshot` is computed · whether a reused unit's steps are shown from the earlier run in the panel (they are
not copied; A18 shows the latest run's steps and says "reused" for the rest — refine there).

## Manual try-out
Nothing visible yet. `cargo test -p psm-testkit --test suite research::a14_research_resume_reuse_follow_ups::`.

## Notes
- C1 enumerates the kill points the commit rule creates (charter rule 2): before a call record, with a call in
  flight, between units, between stages 6 and 7, inside stage 7, after its commit.
- C5(b)'s first case: a retouch changes the adjusted preview but not the photo identity of MC-A7's stage 1 unit
  (photo id, composition revision); the contract excludes it on purpose.
- Review mode: this phase's module, `research::a12…::`, `research::a13…::`, clippy, fmt.

## Review log
