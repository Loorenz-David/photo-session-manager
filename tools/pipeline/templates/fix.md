---
plan: {{PHASE}}
role: fix
round: {{ROUND}}
date: {{DATE}}
---
# Fix cycle {{FIXN}} for phase {{PHASE}}

Repository root: `{{ROOT}}`. Read first: `docs/process/flow.md`, `docs/implementation/master_plan.md`,
`{{PLAN_FILE}}`, the review handoff `{{REVIEW_HANDOFF}}` (your task list is its blocking findings)
and the implementer handoff `{{IMPL_HANDOFF}}` (what exists and why).

**Gate.** Tracker shows {{PHASE}} `CHANGES_REQUESTED`. Otherwise `STATE: BLOCKED`.

**Do exactly this.** Resolve each blocking finding routed `production`, applying its correction
clause as written. Add nothing beyond the findings. Touch only the files each finding allows, plus
the test file a finding names. Notes in the backlog are not your task. If a correction cannot be
implemented as written, implement the sound alternative and explain which finding and why under
**Divergences**. Keep flow.md §4 for any test you add or change.

**Close.** Suite, lint and typecheck green; tracker row → `IMPLEMENTED` (note: fix {{FIXN}});
Review log entry; checkpoint commit of your perimeter only; handoff at `{{HANDOFF}}` with
frontmatter `plan`, `role: fix`, `round: {{ROUND}}`, `state`, `date`, `actor` and sections:
**Finding → what changed → files** · **Divergences** · **Files changed** · **Checkpoint SHA** ·
**Stamp** · `⚠ OWNER DECISIONS REQUIRED (n)` (or none).
