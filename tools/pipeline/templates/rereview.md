---
plan: {{PHASE}}
role: review
round: {{ROUND}}
date: {{DATE}}
---
# Re-review phase {{PHASE}} after fix {{FIXN}} (round {{ROUND}})

Repository root: `{{ROOT}}`. Read first: `docs/process/flow.md` (wins over your doctrine),
`{{PLAN_FILE}}`, the previous review handoff `{{PREV_REVIEW}}`, the fix handoff `{{FIX_HANDOFF}}`,
and the intention chapters the findings cite.

**Gate.** Tracker shows {{PHASE}} `IMPLEMENTED` with a fix note; the fix checkpoint SHA exists.

**Delta-scoped.** 1. `git show --stat <fix checkpoint>`: only files the findings allowed changed;
anything else is a finding. 2. Run suite, lint, typecheck once. 3. For each previous blocking
finding: resolved as its correction said, or a declared divergence that is sound? 4. Read the
changed seam adversarially; settled areas are not re-verified, but anything seen wrong in passing
is reported. Same rules as the first review: no mutation probes, no production edits, B1–B4 only.

**Triage (flow.md §5).** If blocking items remain, classify each: **MUST** (user-visible wrong
outcome, hard constraint H1–H8, red suite) or **DROP** (style, more tests, refactor, speculative
robustness, anything the over-engineering test rejects). Write the triage table in the handoff.
No MUST → verdict `APPROVED_WITH_NOTES`, dropped items to `docs/implementation/backlog.md`.
Any MUST → verdict `CHANGES_REQUESTED` listing only the MUST items; the orchestrator decides.

On approval: tracker → `APPROVED`, Review log entry, narrative at `{{NARRATIVE}}` (shape in the
first-review prompt; mention in "Choices made for you" what the fix changed, in plain words).

**Handoff** at `{{HANDOFF}}` (frontmatter `plan`, `role: review`, `round: {{ROUND}}`, `verdict`,
`state`, `date`, `actor`): **Perimeter check** · **Findings resolved** · **Triage** · **Remaining
findings** · `⚠ OWNER DECISIONS REQUIRED (n)`. Commit nothing.

Final message, first line exactly:
`HANDOFF: {{HANDOFF}} | STATE: <DONE|OWNER_DECISIONS_PENDING|BLOCKED> | VERDICT: <verdict> | OWNER_CARDS: <n>`
