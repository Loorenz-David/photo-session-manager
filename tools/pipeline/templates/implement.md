---
plan: {{PHASE}}
role: implement
round: {{ROUND}}
date: {{DATE}}
---
# Implement phase {{PHASE}}

Repository root: `{{ROOT}}` (your working directory). Read first, in this order:
1. `docs/process/flow.md` — the project's flow; it wins over your doctrine where they differ.
2. `docs/implementation/master_plan.md` — naming registry, environment topology (exact build and
   test commands), standing rules.
3. `docs/intention/intention.md`, then `docs/intention/book/01_objective_and_constraints.md` and
   `docs/intention/book/14_measurement_ledger.md`, then every chapter the plan file names.
4. `{{PLAN_FILE}}` — your task list and acceptance criteria. **The plan file wins over this prompt.**

**Gate.** The intention header reads RATIFIED; the master plan tracker shows every phase in
`depends_on` APPROVED and {{PHASE}} not yet APPROVED. Otherwise stop with `STATE: BLOCKED`.

**Build the phase.** Stay inside the plan's file perimeter; nothing from other phases. Use the
exact names in the registry. Mechanism contracts from the intention are implemented verbatim.

**Tests (flow.md §4).** One behavioural test per criterion row at the boundary the row names,
asserting the exact outcome. Unit tests only for the plan's **Core mechanisms** list. Nothing
that asserts internals.

**Test runs (flow.md §4A; the full Rust suite takes over an hour on this Mac).** While working, run
only this phase's test file (`cargo test -p psm-testkit --test <stem>`), the unit tests of the
module you changed, and the UI test files you touched (`npx vitest run <files>`); use
`cargo check -p <crate>` to see compile errors. Run the whole closing gate (suite, clippy, fmt,
typecheck, `npm test`) **exactly once**, at the very end, one command after the other. If it shows
a failure, fix it and re-run **only the failing test files**, not the whole suite. Never run two
cargo commands at the same time, never in the background, never piped into `head` (a cut pipe
can leave cargo holding the build lock).

**Hardware (flow.md §6A).** You may trigger real captures through Capture One's `capture`
command to check behaviour by hand; the test suite itself never requires the camera or Capture One.

**Judgment calls (flow.md §6).** Where the plan leaves a choice open, take the smallest option
consistent with the intention chapter the plan cites and record it under **Assumptions** in the
handoff. Never stop to ask. `BLOCKED` only for a plan/intention contradiction or an impossible task.

**Close.** Suite, lint and typecheck green (pre-existing failures named); tracker row for
{{PHASE}} → `IMPLEMENTED` (date, actor `codex`, one line); Review log entry in the plan file
(what you built, assumptions, deviations); the checkpoint commit (wrapper rules); the handoff at
`{{HANDOFF}}` with frontmatter `plan: {{PHASE}}`, `role: implement`, `round: {{ROUND}}`,
`state`, `date`, `actor` and these sections: **What was built** (product terms first) · **Tests →
criterion rows** · **Assumptions** · **Deviations** · **Files changed** · **Checkpoint SHA** ·
**Stamp** (command, result) · **Manual try-out** (how the owner can see it work, steps) ·
`⚠ OWNER DECISIONS REQUIRED (n)` (or one line: none).
