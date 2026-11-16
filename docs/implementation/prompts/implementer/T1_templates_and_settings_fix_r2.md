---
plan: T1_templates_and_settings
role: fix
round: 2
date: 2026-11-16
---
# Fix cycle 1 for phase T1_templates_and_settings

Repository root: `/Users/davidloorenz/Desktop/Developer/Photo-Session-Manager-agentic`. Read first: `docs/process/flow.md`, `docs/implementation/master_plan.md`,
`/Users/davidloorenz/Desktop/Developer/Photo-Session-Manager-agentic/docs/implementation/plans/T1_templates_and_settings.md`, the review handoff `/Users/davidloorenz/Desktop/Developer/Photo-Session-Manager-agentic/docs/implementation/handoffs/reviewer/handoff_T1_templates_and_settings_review_r1.md` (your task list is its blocking findings)
and the implementer handoff `/Users/davidloorenz/Desktop/Developer/Photo-Session-Manager-agentic/docs/implementation/handoffs/implementer/handoff_T1_templates_and_settings_implement_r1.md` (what exists and why).

**Gate.** Tracker shows T1_templates_and_settings `CHANGES_REQUESTED`. Otherwise `STATE: BLOCKED`.

**Do exactly this.** Resolve each blocking finding routed `production`, applying its correction
clause as written. Add nothing beyond the findings. Touch only the files each finding allows, plus
the test file a finding names. Notes in the backlog are not your task. If a correction cannot be
implemented as written, implement the sound alternative and explain which finding and why under
**Divergences**. Keep flow.md §4 for any test you add or change.

**Test runs (flow.md §4A).** While fixing, run only the test files the findings name (a phase's behaviour tests:
`cargo test -p psm-testkit --test suite <stem>::`) and the tests of the modules you changed. At the end run the test files of the crates and UI features
your fix touched, clippy, fmt and typecheck once, one command after the other; the full suite
only if the fix changed shared code (store, migrations, services used by every phase); a plan with
a **Closing gate** section names the commands for its units instead (flow.md §10). Never two
cargo commands at the same time, never in the background, never piped into `head`.

**Close.** Suite, lint and typecheck green; tracker row → `IMPLEMENTED` (note: fix 1);
Review log entry; checkpoint commit of your perimeter only; handoff at `/Users/davidloorenz/Desktop/Developer/Photo-Session-Manager-agentic/docs/implementation/handoffs/implementer/handoff_T1_templates_and_settings_fix_r2.md` with
frontmatter `plan`, `role: fix`, `round: 2`, `state`, `date`, `actor` and sections:
**Finding → what changed → files** · **Divergences** · **Files changed** · **Checkpoint SHA** ·
**Stamp** · `⚠ OWNER DECISIONS REQUIRED (n)` (or none).
