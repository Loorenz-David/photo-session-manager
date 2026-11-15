---
plan: research
role: plan
round: 2
state: DONE
date: 2026-11-12
actor: Claude Fable 5.2 (pipeline-planner, implementation-planner doctrine; orchestrated)
---
# Research agent — plan restructured into milestones on an isolated crate (round 2)

**Summary.** On the owner's decision of 2026-11-12 (`docs/process/flow.md` §10) the 24 phase plans `A01`–`A24`
(`d116654`) are regrouped into 10 milestone plans `R1`–`R10`, one runner phase each, made of 29 units. The engine
moves to its own crate `crates/research` with its tests inside it; app code changes only at named seams. All 393
criterion rows are kept with their outcomes, 3 freeze rows are new, and a mapping file says where each row went. The
storage question is decided by the contracts, with no owner trade-off left. Documents and three template files
only: no code, no cargo, no external call, no commit. Supersedes `handoff_research_plan_r1.md`.

## ⚠ OWNER DECISIONS REQUIRED (0)
None. Owner inputs are unchanged: the keys file for R1; the picks asked for by R1's and R9's narratives.

## Storage decision (no card)
All research tables stay in the **app database file**, written inside the app store's transactions; nothing is
added to the numbered migration list. A separate SQLite file for the engine's tables was evaluated and rejected
because it breaks contracts: MC-A7 requires stage 7's attribute writes and its steps to commit together, and the
app database runs in WAL mode, where a commit across two files is not atomic (a crash leaves values without steps,
or a committed stage without its values: MA4, MA6, MA14, H4); MC-A5 needs the eligibility read and the call and
outbound inserts in one transaction (MA12); MC-A6's start transaction and the `jobs` row are app data. The
migration-order problem with `main` is removed another way: the research schema is created by two idempotent schema
steps outside the numbered list (`psm_infra::sqlite::ensure` for the app-owned columns and the feedback table,
`psm_research::schema::ensure` for the engine's five tables), and the engine's SQL runs through one new store method,
`SqliteStore::research_transaction`. Full reasoning: `master_plan_research.md` §6.4.

## Milestones
| Milestone | Units (old plans) | Criteria / rows | App files | Projection | depends_on | Closing gate |
|---|---|---|---|---|---|---|
| R1_research_benchmark | U1 probes and search benchmark (A01) · U2 model comparison (A02) | 13 / 22 | no | no | — (keys) | `cargo test -p psm-research-tool` + tool lint + `cargo check --workspace` |
| R2_research_foundation | U1 name key, sameness (A03 C1, C7) · U2 storage seam, frozen shapes (A07 C7, A09 C5, new F·C1–C3) · U3 renderer (A10 C1–C2) | 9 / 23 | yes | no | — | full gate |
| R3_research_rules | U1 value mapping (A03 C2–C6) · U2 sites, fingerprints (A04) · U3 confidence (A05) · U4 cost, call rules (A06) · U5 write table (A07 C1–C6) | 31 / 132 | no | no | R2 | `cargo test -p psm-research` + research lint + `cargo check --workspace` |
| R4_research_engine | U1 ports, image guard, tables, fakes (A10 C3–C5, A11 C8) · U2 gateway (A11 C1–C7) · U3 stages (A12) · U4 lifecycle (A13 minus 4 rows) · U5 resume (A14) · U6 research API (A18 C1, C6; A19 C1–C4) | 39 / 99 | no | no | R3 | as R3 |
| R5_research_app_seams | U1 feedback and undo (A08) · U2 publish hold (A09 minus C5) | 12 / 34 | yes | yes | R3 | full gate |
| R6_research_ui | U1 panel (A18 C2–C5) · U2 settings (A19 C5) · U3 publish review (A20) | 10 / 21 | yes, TypeScript only | no | R5 | `npm run typecheck` · `npm test` |
| R7_research_wiring_and_export | U1 wiring (A13 C1(c), C2(a), C8(c), C8(d)) · U2 retention, export (A21) | 10 / 17 | yes | no | R4, R6 | full gate + `cargo test -p psm-research` |
| R8_research_real_adapters | U1 OpenAI (A15) · U2 visual search (A16) · U3 page reader, shop facts (A17) | 18 / 29 | yes (U3) | yes | R4, R1 (+ picks) | full gate + `cargo test -p psm-research` |
| R9_research_tool_and_acceptance | U1 report, replay (A22) · U2 acceptance (A23) | 10 / 15 | no | no | R7, R8, R1 | tool tests + tool lint |
| R10_research_defaults | U1 defaults (A24) | 2 / 4 | no | no | R9 (+ model pick) | `cargo test -p psm-research` + research lint |

Full gate = `cargo test --workspace` · `cargo clippy --workspace --all-targets -- -D warnings` · `cargo fmt --all --
--check` · `npm run typecheck` · `npm test`, once, after the last unit. Per-unit commands are written in each unit.
Ten milestones instead of the suggested eight: a foundation milestone had to come first to freeze the shared shapes
(without it no two tracks can run), and the defaults are their own one-unit milestone because they wait for an owner
pick that the acceptance milestone itself produces.

## Parallel schedule
R2 alone → track A: R3 → R4 → R7 → R9 → R10; track B (second worktree, branch `research/track-b`): R1 when the keys
exist → R5 → R6 → R8. Concurrent pairs: R3 ∥ R1 · R4 ∥ (R5, then R6) · R7 ∥ R8. Each milestone is merged when
approved; the other track merges `feature/agentic` before starting its next milestone. Frozen by R2 so the tracks
do not meet: name key and sameness, `Value.ai_prior`, feedback rows and their store methods, the publish columns,
`PublishStep::Held` and its label, the `research_changed` event, `CommerceAdapter::public_listing`, the `research`
settings slot, `SqliteStore::research_transaction`, `Imaging::render_research`, the `ResearchApi` trait and shapes,
the eight research commands and their generated bindings. Expected merge conflicts are one-line (`Cargo.toml`
members, `Cargo.lock`, two `mod` lists). Details: `master_plan_research.md` §5.

## Seams (app code that changes)
Values vocabulary (core) · storage (infra schema step, one store method, `ai_prior`, feedback) · the research-copy
renderer (infra) · shapes for publish, events, settings, shop (core, infra) · research commands · attribute actions
and undo · publish withheld/conflicts/held · UI · wiring in `crates/integrations/src/lib.rs` · Shopify shop facts.
Table with files and units: `master_plan_research.md` §5. App crates call the research crate only through
`psm_research::api::ResearchApi` and four constructors used in `crates/integrations/src/lib.rs`.

## Template changes (compatible with `main`'s P-phases: each applies only to a plan that has the named section)
- `tools/pipeline/templates/implement.md`: the gate may be a tracker the plan names; a paragraph for plans with
  `## Unit` sections (units in order, unit test commands, a checkpoint after each, the plan's Closing gate replaces
  the template's, the handoff lists every checkpoint).
- `tools/pipeline/templates/review.md`: several checkpoints → check the range; a plan's Closing gate names the commands.
- `tools/pipeline/templates/fix.md`: a plan's Closing gate names the commands.
`run_phase.sh`, `rereview.md`, `projection.md` and the wrappers are unchanged.

## Row mapping
`docs/implementation/research_row_mapping.md` (generated; one line per row). Of the 393 old rows: 79 keep their
boundary · 107 are pure rules with the same boundary, now in the research crate instead of core · 164 moved their
boundary or test home to the research crate (same service or port; the research harness instead of the testkit
suite; `psm_research::writes` instead of `AttributesService::ai_suggest`; the `ResearchApi` trait instead of a Tauri
command) · 26 moved their test home to the tool crate · 6 changed boundary inside the app (a schema step instead of
a numbered migration; the renderer called directly) · 11 are `[manual]` rows, unchanged. 8 rows had a boundary or
input cell reworded; no outcome was weakened. 3 rows are new (`F·C1`–`F·C3`, R2·U2). Row ids keep the old plan's
number (`A07·C1(a)`), so every row is traceable to `d116654`.

## Points for the orchestrator (not cards)
1. R6's closing gate has no `cargo test --workspace` (TypeScript only); a literal reading of flow §10 would add it.
2. A dev-dependency cycle (`psm-research` → `psm-testkit` → … → `psm-research`) is relied on; Cargo allows it, but no
   cargo command was run in this session. R3·U5 exercises it first.
3. The implementer wrapper says "one commit"; the plans and the implement template say one per unit.
4. Design changes made to fit the crate move, each stated in the unit that carries it: research writes go through
   `psm_research::writes` (core's unused `ai_suggest` stays); unit names come from the imported definitions instead
   of a new port method; `ResearchSettings` is owned by the research crate (the settings file has a JSON slot);
   adapters are chosen in `psm_research::adapters::build`.
5. Unchanged from round 1: "Looks right" changes meaning by contract (R5·U1); attribution keys are shop-specific; the
   HTTP page reader's address guard has no contract; the intention map says "not planned yet".
6. A second worktree needs `npm ci` and a second build folder (tens of gigabytes).

## Write perimeter (documents and templates only; nothing committed)
Written: `docs/implementation/master_plan_research.md` (v2) · `docs/implementation/phases_research.txt` ·
`docs/implementation/research_row_mapping.md` · `docs/implementation/plans/{R1_research_benchmark,
R2_research_foundation, R3_research_rules, R4_research_engine, R5_research_app_seams, R6_research_ui,
R7_research_wiring_and_export, R8_research_real_adapters, R9_research_tool_and_acceptance, R10_research_defaults}.md` ·
`tools/pipeline/templates/{implement.md, review.md, fix.md}` · this handoff.
Deleted (in history at `d116654`): the 24 files `docs/implementation/plans/A01_…md` … `A24_…md`.
Not touched: `master_plan.md`, `phases.txt`, `backlog.md`, `STATUS.md`, `flow.md`, the intention book, any code, the
sibling checkout, `handoff_research_plan_r1.md`.

## Checks run on the plan set
Every old row id is placed exactly once (393 of 393); every unit has at most 8 criteria; every row has five cells
and a trace; each plan has the runner's frontmatter keys, a Tracker line, a Closing gate and a Review mode; every
`depends_on` id and every id in the list file has a plan file. Not checked by anything: whether older task wording
inside a unit still reads perfectly after the move (each unit's "What changed" list wins where they differ), and
whether a row's assertion is as strong as it reads.
