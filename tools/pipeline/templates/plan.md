---
plan: master
role: plan
round: 1
date: {{DATE}}
---
# Plan the Session Manager MVP

Repository root: `{{ROOT}}`. Read `docs/process/flow.md` first: it is the project's process
authority and **wins over your doctrine where they differ** (no tester role, no named mutations,
behaviour tests at boundaries, one automatic fix then triage, narratives for the owner). Gate: the
intention (`docs/intention/intention.md`) reads RATIFIED and the inventory handoff in
`docs/implementation/handoffs/reviewer/` reports DONE. Read the whole book, both evidence docs,
`docs/intention/evidence/design_handoff.md`, and `claude_design/README.md`, `NOTES.md`,
`COMPONENTS.md` (the UI is built from this handoff).

**Write** `docs/implementation/master_plan.md` and one plan file per phase in
`docs/implementation/plans/<phase>.md`, plus `docs/implementation/phases.txt` (one phase id per
line, in execution order).

**Master plan (thin):** goal + pointer to the book · sources-of-truth table · **standing rules:
import flow.md by reference and state the project specifics** (no tester role; no named
mutations; review blocking rules B1–B4; fix budget 1 then triage; narratives) · naming registry
(crates, modules, tables, enums, Tauri command names, event names, settings keys, test file
layout) · **environment topology, verified by running the commands**: Rust toolchain and Tauri
CLI presence, Node, the exact `cargo test`, `cargo clippy`, `npm test`, `npm run typecheck`,
`cargo tauri dev` / build commands, the app data directory, how a test gets a temp data dir and a
fake Capture One / fake Shopify adapter (tests never touch live services) · tracker (one row per
phase) · sequencing.

**Phases:** ≤ 8 criteria each; the first phase is the packaging spike from book chapter 11; order
the rest so the **local workflow is usable first** (capture arrival, review, order, live previews,
export) and Shopify publishing follows; later phases may run in parallel when `depends_on` allows.
Each plan file: frontmatter `phase`, `title`, `difficulty: normal|high` (high = silent-failure
mechanism, cross-module wiring, or large), `projection: no|yes` (yes only when a row's fixture is
owned by an external dependency), `depends_on: [..]` · Goal (with what is NOT in the phase) ·
Read-first list (book chapters, evidence, contracts) · Files expected to change · **Core
mechanisms** (the only functions that may have unit tests) · Ordered tasks · **Criteria table**:
row id, boundary, input, exact outcome, trace (M-id or contract) · **Delegated choices** (what the
implementer may decide alone) · **Manual try-out** (steps the owner can follow on this Mac after
the phase) · Notes · empty Review log.

Every path or symbol you cite must exist or be marked new; every count is derived; a row traces
to a ledger entry that supports it. Do not restate product semantics; cite chapters. Decision
cards only for what the book truly leaves open.

Handoff at `{{HANDOFF}}` (frontmatter `plan: master`, `role: plan`, `round: 1`, `state`, `date`,
`actor`): files written, the phase map in five lines per phase (what it builds, what the owner can
try after it), estimated size per phase, `⚠ OWNER DECISIONS REQUIRED (n)`. Commit nothing.

Final message, first line exactly:
`HANDOFF: {{HANDOFF}} | STATE: <DONE|OWNER_DECISIONS_PENDING|BLOCKED> | OWNER_CARDS: <n>`
