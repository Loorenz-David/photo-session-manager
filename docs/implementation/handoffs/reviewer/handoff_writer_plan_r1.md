---
plan: writer
role: plan
round: 1
state: OWNER_DECISIONS_PENDING
date: 2026-11-19
actor: Claude Opus 5.5 (pipeline-planner, implementation-planner doctrine; orchestrated)
---
# Publication writer — plans for W1–W6 (book 08C)

**Summary.** A short master plan (`master_plan_writer.md`) with the contracts that stand in for the mechanism
inventory (WC1–WC8, §5), the shipped writing guide (§9: English and Swedish instructions, examples A–D with recorded
adaptations), six milestone plans in the units format of flow §10, and the phase list. Gate: the intention map lists
08C as ratified; 08C's status header reads **RATIFIED** (owner, 2026-11-19, changelog A22). The mechanism-inventory
gate is replaced by §5 at the orchestrator's instruction (recorded as a process delta, as for templates). Every plan
was grounded in the code at `0139f57` (paths verified to exist) and Shopify's `seo` types were read from the Admin
schema. One owner card; it gates W6 only.

Counts (computed from the plan files): 6 milestones, 17 units, 73 criteria, 231 rows; every unit ≤ 8 criteria.
| Milestone | Units | Criteria | Rows | App files | Projection |
|---|---|---|---|---|---|
| W1_product_note | 3 | 14 | 54 | yes | no |
| W2_writer_engine | 5 | 25 | 96 | three (one wiring line, session delete, schema tripwire list) | no |
| W3_writer_review | 3 | 10 | 30 | yes | no |
| W4_writer_settings | 2 | 10 | 21 | yes | no |
| W5_seo_publish | 3 | 9 | 20 | yes | **yes** |
| W6_restoration_choices | 1 | 5 | 10 | yes | no (gated on card 1) |

## ⚠ OWNER DECISIONS REQUIRED (1)

**Card 1 — where the Restoration choices live**

**Question** — Should "Restoration done" (sanded, oiled, new foam, papercord…) be kept only in Session Manager (A), or be a new field in your Shopify store that you create once in the admin (B)?

**Story** — You tick Sanded and Oiled on a teak chair, and the writer says "We sanded and oiled the frame" in the description and the condition report. With A the ticks stay in Session Manager and its export; Shopify gets the texts. With B you first add a list field with these choices in Shopify's admin; the ticks then also go to Shopify, and every new choice must be added there too.

**Branches** — **A**: nothing to do in Shopify; the choices are fixed in the app. **B**: one admin task now and one per new choice; the choices travel to Shopify.

**Recommendation** — A, because the texts already tell the buyer and it needs no Shopify work.

**On silence** — W1–W5 go ahead; W6 waits.

**Trace** — 08C §13; WC8-R; W6.

## Decisions taken in planning (technical; the narratives carry them in plain words)
- **The title has one home** (`items.title`, always `human` or shop-read), so the writer's title is an `ai` row of key
  `product.title`; it becomes the item's title only on Accept, and the publish partition skips it (WC4.3, WC4.7–8).
  Consequence: an unchecked title is never published, unlike an unchecked `medium` description (MC-A10 unchanged).
- **`partition_in` gains the built-in definitions** so a writer description or search text with a prior is a real
  conflict (held) instead of being left out silently, which is what the code does today for a key with no definition.
- **One queue for three run kinds.** Writer (`write`) and Look again (`look`) runs are `research_runs` rows, so the
  per-session queued/running indexes, the one-running-app-wide rule, the call record, the day cap and the export are
  shared. WC3 orders the request rules so a `write` run never meets a research or look run in the queued slot; Run
  again cancels a queued `write`/`look` run. Research's views exclude both triggers (WC1.9 lists every call site).
- **Separate `writer` settings slot** (not inside `research`): the research DTO round-trip would drop it; the slot is
  nulled in the rebuild comparison so a save never rebuilds the App.
- **The writer is a module of the research crate**, sharing the gateway, the run store and the fakes.
- **`INSTRUCTIONS_VERSION` 2 → 3**: every stage of the next research of each session is called again once.
- Further choices: master plan §6.

## Risks
1. **No measurement ledger for 08C.** Trace cells cite HW1–HW7, 08C sections and WC1–WC8. The orchestrator may
   backfill a lettered section in book `14` from WC1–WC8; no row id changes.
2. **Shopify `seo` facts are not yet captured.** What an unset field reads as (null or "") and what a 75-character
   title does are captured in W5·U2 task 1 with P12's probe credentials; W5·C4(a), C5(b) and C8(a) depend on them.
   Projection is on for W5.
3. **Post-check heuristics.** The names rule uses the choices of `research.attribution_keys` fields; with no
   attribution keys chosen it checks only years and model numbers. Use-sentence markers and the Swedish damage words
   are fixed lists and can miss or over-flag; a false hit only flags a text (low confidence), it never blocks.
4. **Core change to the title** (W3·U1): undo must restore `items.title` and the row together; W3·C2(a) pins it.
5. **"Leaving a session with All checked"** is detected in the frontend (route change or Details closing); W3·C7(b)
   pins one way; another way of leaving may not trigger. The automatic trigger is once per session anyway, and
   "Write texts" always works.
6. **The Swedish instruction is the planner's text** (§9.2). Evidence question 11 (who proofreads Swedish) is still
   open; the owner tunes it in Settings → Writer (W4).
7. **Default model** `gpt-6.1-sol` at `medium` is not measured for writing quality; 08C §13 says models are tested
   later.
8. **W2 changes three app files**, so its closing gate is the full suite although the writer is not reachable in
   the app until W3.

## Write perimeter (complete)
| File | Change |
|---|---|
| `docs/implementation/master_plan_writer.md` | new |
| `docs/implementation/phases_writer.txt` | new |
| `docs/implementation/plans/W1_product_note.md` | new |
| `docs/implementation/plans/W2_writer_engine.md` | new |
| `docs/implementation/plans/W3_writer_review.md` | new |
| `docs/implementation/plans/W4_writer_settings.md` | new |
| `docs/implementation/plans/W5_seo_publish.md` | new |
| `docs/implementation/plans/W6_restoration_choices.md` | new |
| `docs/implementation/handoffs/reviewer/handoff_writer_plan_r1.md` | this file |
One commit on `feature/agentic` holding exactly these nine files. No code, no test run, no book chapter, no
`STATUS.md`, no `STATUS_writer.md`, no `master_plan.md` or `master_plan_research.md`, no tracker line of another plan,
no archgraph state, no push. Outside the worktree: read the doctrine files under `~/agent-skills/`; one read-only
Shopify Admin schema lookup (types `SEO`, `SEOInput`, `ProductUpdateInput.seo`, `Product.seo`; no store data); a
measuring script in the session scratchpad.
