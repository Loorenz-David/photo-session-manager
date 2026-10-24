---
plan: {{PHASE}}
role: projection
round: 0
date: {{DATE}}
---
# Projection of phase {{PHASE}} (round 0)

Repository root: `{{ROOT}}`. Read `docs/process/flow.md` first (wins over doctrine), then only
what the implementer will read: `docs/implementation/master_plan.md`, the intention chapters the
plan names, and `{{PLAN_FILE}}`. Do the implementer's first hour on paper. Product: the decision
ledger (plan gap / intention gap / free choice) and the decidability of every criterion row (can
the test be written now, with one exact outcome, at the named boundary?). You edit nothing.

Handoff at `{{HANDOFF}}` (frontmatter `plan`, `role: projection`, `round: 0`,
`verdict: PROJECTED_CLEAN|AMENDMENTS_REQUIRED`, `date`, `actor`), opening with 3–5 plain
sentences, then `⚠ OWNER DECISIONS REQUIRED (n)`, then the ledger table. Commit nothing.

Final message, first line exactly:
`HANDOFF: {{HANDOFF}} | STATE: <DONE|OWNER_DECISIONS_PENDING|BLOCKED> | VERDICT: <verdict> | OWNER_CARDS: <n>`
