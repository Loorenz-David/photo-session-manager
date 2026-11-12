---
phase: A18_research_panel_ui
title: The research panel comes alive — you watch the steps appear while research runs, see why it is not running when it is not, start it yourself with Research / Run again, and the form header tells you how many suggestions are waiting for you
difficulty: normal
projection: no
depends_on: [A14_research_resume_reuse_follow_ups]
---
# A18 — Commands, events, the live research panel, "n to review" (08A §1, §5, §7, §8, §12 must-ship; 08B MC-A6 panel states, MC-A10 "n to review"; MA7)

## Goal
Thin commands over `ResearchService`, the `research_changed` event, and the research panel rebuilt from stored
steps: live while a run is `running`, with a Finished state ("n fields filled · n to review"), the prerequisite
sentence when research cannot run, and **Research** / **Run again**. The form header shows "n to review" and opens
the panel. **Not in this phase:** settings (A19), conflict buttons and publish texts (A20), the only-if-cheap
items of 08A §12 (cancel a run, research one field again, show cost) — they are not built.

## Read first
`master_plan.md` §3, §6.7, §6.9 (frontend tests) · `master_plan_research.md` §6.6 (commands, event, sentences) ·
book `08A` §1, §5 (Not determined), §7, §8 (`research_steps`; derived values), `08B` MC-A6 (prerequisites, panel
sentences), MC-A10 ("n to review") · evidence `design_handoff.md` (AI pill, popover, panel rules), `claude_design/
NOTES.md` §Screens 7 and §AI agent rules · code `apps/desktop/src/features/form/{ResearchPanel.tsx, FormDialog.tsx
(lines 34, 323), AiPopover.tsx, AiPill.tsx}`, `apps/desktop/src/services/{attributes.ts, events.ts, bindings.ts}`,
`apps/desktop/src-tauri/src/{commands/attributes.rs, commands/mod.rs, lib.rs (collect_commands), state.rs}`,
`crates/core/src/services/research/{mod.rs, runs.rs}` (`status`, `run_again`).

## Dependencies and gate
Tracker row and gate: `master_plan_research.md` §4. Notes: `backlog_research.md`.

## Files expected to change
New: `apps/desktop/src-tauri/src/commands/research.rs`, `apps/desktop/src/services/research.ts`,
`apps/desktop/src/features/form/__tests__/a18_research_panel.test.tsx`,
`crates/testkit/tests/suite/research/a18_research_panel_ui.rs` (+ `mod` line).
Edited: `apps/desktop/src-tauri/src/{commands/mod.rs, lib.rs, state.rs}` (event forwarding),
`apps/desktop/src/services/{bindings.ts (regenerated), events.ts}`, `apps/desktop/src/features/form/{ResearchPanel.tsx,
FormDialog.tsx}`, `crates/core/src/services/research/mod.rs` (the status DTO's derived counts), own tracker row.

## Core mechanisms
None.

## Ordered tasks
1. `research_status(session_id) -> ResearchStatus {reason: Option<code + sentence>, run: Option<{id, state, trigger,
   stage_reached, error}>, steps: [{seq, stage, kind, summary, detail, source_url, source_title, photo_ids,
   field_keys}], fields_filled, to_review, can_run}` and `research_run(session_id)`. Derived, never stored:
   `fields_filled` = keys the latest run wrote; `to_review` = live `ai` + `suggested` rows whose key is in the
   current layout, at every confidence.
2. Forward `AppEvent::ResearchChanged` as the Tauri event `research_changed {session_id}`.
3. Panel: the steps of the latest run in order, each with its kind's verb (Looked, Searched, Read with the source
   link, Compared, Concluded, Not determined) and detail; while `running`, a live indicator and new steps appended
   on `research_changed`; when `finished` or `stopped_at_cap`: `Finished · n fields filled · n to review` (for a
   capped run the line also says the run stopped at its cost limit); when `failed`: the stage and error and **Run
   again**; when there is no run and research can run: **Research**; when it cannot: the sentence. Focusing a field
   from its popover dims the steps whose field keys do not include it (existing behaviour, now from `field_keys`).
   Accept all stays, unchanged.
4. Form header: `n to review` (hidden at 0), opening the panel.

## Criteria (6 criteria, 12 rows, 7 `[rule]`)
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | `ResearchService::status` (suite) | a finished run that wrote three fields, one accepted since, one of the remaining two `low` | `fields_filled` 3; `to_review` 2 | MC-A10 |
| C1(b) | same | a field with a suggestion is removed from the layout | `to_review` drops by one | MC-A10 |
| C1(c) | same | no model key | reason `NoModelKey` with its sentence; `can_run` false | MA7 |
| C2(a) | panel (UI test) | status with five steps of kinds looked, searched, read, compared, concluded | five items in `seq` order with their verbs; the read step links its source URL | `[rule]` 08A §8 |
| C2(b) | panel | run `running`; a `research_changed` event arrives and the status now has one more step | the new step is shown without reopening; the live indicator is present | `[rule]` 08A §1 |
| C2(c) | panel | run `finished`, 3 filled, 2 to review | `Finished · 3 fields filled · 2 to review` | `[rule]` 08A §3 (design mock-up) |
| C2(d) | panel | run `failed` with stage `read` and an error | both shown; a **Run again** button | `[rule]` 08A §10 |
| C3(a) | panel | no run, `can_run` true; click **Research** | `research_run` called with the session id | `[rule]` 08A §5 |
| C3(b) | panel | reason `DayLimit`; reason `WaitingForEditor` | `Day limit reached`; `Waiting for Capture One`; no Research button for the first | MA7 |
| C4(a) | panel | focus on field `designer` from its popover | steps whose field keys lack `designer` are dimmed; clearing the focus undims | `[rule]` design_handoff §AI |
| C5(a) | form header | `to_review` 2; then 0 | `2 to review`, opening the panel on click; then no label | `[rule]` 08A §7 |
| C6(a) | commands (suite) | `research_run` on a session with a finished run | one `queued` run with trigger `manual` (the command adds nothing to `run_again`) | MC-A6 |

## Delegated choices
Visual details within the design handoff's tokens and components · how reused units of a follow-up are shown (one
line "n earlier steps reused") · polling fallback if an event is missed (re-read on panel open). Refine at prompt
time against `claude_design/`.

## Manual try-out
(Real research needs A15–A17 and A19; with them in place:)
1. `cd apps/desktop && PSM_DATA_DIR=$HOME/psm-research-dev npm run tauri -- dev`, create a session, shoot or import a photo.
2. Open the form. The header shows the research state; open the panel and watch steps appear.
3. When it says Finished, check the suggested fields carry the AI pill, and that "n to review" matches.
4. Press **Run again** and see a new run start.
Without a key the panel says "Add the model key in Settings → Research" and everything else works as before.

## Notes
- No `implementer: fable` (flow log v1.3).
- Review mode: this phase's suite module, `npx vitest run src/features/form`, typecheck, clippy, fmt.

## Review log
