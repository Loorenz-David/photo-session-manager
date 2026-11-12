---
plan: research
role: plan
round: 1
state: DONE
date: 2026-11-12
actor: Claude Fable 5.2 (pipeline-planner, implementation-planner doctrine; orchestrated)
---
# Research agent — master-plan part and phase plans, round 1

**Summary.** Gate passed: `book/08A_research_agent.md` reads RATIFIED and `book/08B_research_contracts.md` reads
CONTRACTS RATIFIED (owner, 2026-11-12). I wrote a research part of the master plan as a companion file, a separate
phase list and 24 phase plans, `A01`–`A24`, each with at most 8 criteria (148 criteria and 393 rows in total,
counted from the files). The runner needs no change. Sixteen phases (A03–A14, A18–A21) can be built now against
our own ports and fakes; the two benchmark phases wait for the owner's keys; three phases wait for an owner pick
that the benchmark reports will ask for. No production code was written, nothing was run, no external service was
called, nothing was committed.

## ⚠ OWNER DECISIONS REQUIRED (0)
None. The owner owes **inputs**, not decisions, before A01 can run (below). The picks (visual-search route,
page-reading route, per-stage models) are asked for later, as decision cards with measured numbers, by the
narratives of A01, A02 and A23.

## Layout decision
- Phase ids `A01_…`–`A24_…`; plan files in `docs/implementation/plans/` beside the `P…` files. `run_phase.sh`
  resolves plan, state, prompts, handoffs and narrative from the id, so nothing collides with `main` and the runner
  is unchanged.
- `docs/implementation/phases_research.txt` is the research phase list; `phases.txt` is untouched (`main` stopped
  appending to it at P52 and starts phases one by one, as research phases are started). The driver loop is in the
  companion file §0.
- `docs/implementation/master_plan_research.md` holds the research tracker, sequencing, naming registry and
  environment additions. **No line of `master_plan.md` is edited.** Each `A…` plan states that its tracker row and
  gate are in the companion file (the plan file wins over the prompt template).
- Reviewer notes go to `docs/implementation/backlog_research.md`; behaviour tests to
  `crates/testkit/tests/suite/research/` behind one `mod research;` line; research migrations are `a01_…`–`a03_…`
  in one block kept last in the migration list.
- Consequence recorded as a standing rule: until this branch is merged into `main`, the app built from it runs
  only on a scratch data folder (`PSM_DATA_DIR`), because `main` will add migrations that must sort before the
  research ones.

## Phase map
| Phase | One line | depends_on | Difficulty | Projection | Criteria / rows |
|---|---|---|---|---|---|
| A01_research_probes_and_search_benchmark | probes RESEARCH-VERIFY-1…10 with real calls; both visual-search routes on the 42 pieces; page reading on the nine sites; the tool crate; fixtures | — | high | no | 8 / 13 |
| A02_research_stage_model_benchmark | three model candidates on observe, read, condition against the answer key | A01 | normal | no | 5 / 9 |
| A03_research_values_and_sameness | name key, model output → value, sameness (MC-A2, MC-A1 `S`) | — | high | yes | 7 / 30 |
| A04_research_sites_and_fingerprints | hosts, trusted entries, site and page identity, own listing, reading order (MC-A3); fingerprint (MC-A7) | A03 | high | yes | 7 / 28 |
| A05_research_confidence | confidence derivation (MC-A4) | A04 | high | yes | 6 / 32 |
| A06_research_cost_and_call_rules | money, call cost, stop rule, the day (MC-A8); error classes, attempts, redaction (MC-A9) | A05 | high | yes | 7 / 29 |
| A07_research_storage_and_write_permission | migration a01, `ai_prior`, the write table in `ai_suggest`, cleared set, lowering (MC-A1) | A03 | high | yes | 7 / 26 |
| A08_research_feedback_and_undo | feedback rows, conflict actions, guarded undo/redo (MC-A12) | A07 | high | yes | 5 / 16 |
| A09_research_publish_hold | withheld and pending values, step `held`, resolution, release (MC-A10); migration a02 | A08 | high | yes | 8 / 20 |
| A10_research_ports_images_and_fakes | provider ports, `ResearchImage` and renderer, file store, fakes (MC-A5) | A06 | high | yes | 5 / 13 |
| A11_research_call_gateway | migration a03; record-then-send, outbound rows, attempts, cost (MC-A5, MC-A9, MC-A8) | A09, A10 | high | yes | 8 / 19 |
| A12_research_stages_and_decide | stages 1–7, unit commit, steps, Decide and writes, cap and failure endings | A11 | high | yes | 8 / 24 |
| A13_research_run_lifecycle | run rows, prerequisites, triggers, quiet period, readiness, start order, day limit, offline, the job; settings (MC-A6) | A12 | high | yes | 8 / 24 |
| A14_research_resume_reuse_follow_ups | kill-and-resume, reuse, follow-up trigger set (MC-A7) | A13 | high | yes | 6 / 17 |
| A15_research_model_adapter | OpenAI adapter from A01's captures | A11, A01 | high | yes | 6 / 14 |
| A16_research_visual_search_adapter | the visual-search adapter the owner picks | A11, A01 | normal | yes | 6 / 7 |
| A17_research_page_reader_and_shop_hosts | page reader by the picked route; shop hosts, own listing, unit names | A12, A01 | high | yes | 6 / 8 |
| A18_research_panel_ui | commands, event, live panel, "n to review", Research / Run again | A14 | normal | no | 6 / 12 |
| A19_research_settings_ui | Settings → Research | A13 | normal | no | 5 / 11 |
| A20_research_publish_ui | withheld line, conflicts warning and actions, "Waiting for you" | A09, A18 | normal | no | 5 / 9 |
| A21_research_retention_and_export | retention of call records, Export research (MC-A11) | A14 | high | yes | 7 / 13 |
| A22_research_report_and_replay_tool | `psm-research report` and `replay` | A21, A01 | normal | yes | 5 / 9 |
| A23_research_acceptance_benchmark | the 08A §11 measurement on the real engine; second model comparison (owner-attended) | A15–A22 | high | no | 5 / 6 |
| A24_research_defaults | shipped defaults and the reachability check | A23 | normal | no | 2 / 4 |

**Start now:** A03 → A14 in order, then A18, A19, A20, A21. **Blocked on the owner's keys:** A01, A02 (and A23).
**Blocked on A01's recorded evidence:** A15, and A22 for the tool crate only. **Blocked on the owner's pick from
A01's report:** A16 (visual-search route), A17 (page-reading route). **Blocked on the owner's model pick:** A24.
The blocked phases carry a `blocked_on` frontmatter line (the runner ignores it) and a task 0 that reports
`BLOCKED` when the pick is not recorded in the evidence doc under "Owner picks".

## What the owner must provide before A01 can run
One file outside the repo, `~/.psm_research.env`, mode 0600, with three lines:
- `OPENAI_API_KEY` — exists already; best a key of its own project with a USD 50 monthly budget limit set in the
  OpenAI dashboard.
- `SERPAPI_API_KEY` — new: an account at serpapi.com on the free plan (250 searches a month; the benchmark is
  capped at 130), key from the dashboard.
- `GOOGLE_VISION_API_KEY` — new: Google Cloud console → new project → enable "Cloud Vision API" → attach a billing
  account (the first 1,000 units a month are free; the benchmark is capped at 130) → Credentials → Create API key →
  restrict it to Cloud Vision API.
`~/.psm_shopify.env` (existing) is read for one read-only shop query. **Spending ceiling:** USD 50 for A01 and A02
together (15 + 35), enforced by the tool; USD 40 for A23.

## Benchmark design in short
- The tool is a Rust crate, `crates/research-tool` (bin `psm-research`). Its recordings use the export layout of
  MC-A11 and the call-record fields of MC-A9, so the same crate becomes the dev-Mac `report` and `replay` tool in
  A22 with one reader for both. What is not shared cheaply: A01 carries two small private helpers (host matching,
  micro-USD sum) because the core functions do not exist yet when it may run; A22 replaces them.
- "The right listing in the first 10" has no URL in the answer key. A01 scores a result as *naming* when its title
  or page text carries the key's designer or manufacturer, removes the piece's own listing, reports the 12
  unpublished pieces apart (the clean group: their photos are not on the web), and adds a 12-piece check sheet the
  owner marks by eye, since a naming result can be another piece by the same designer.
- A02 compares models only on the stages the answer key can score (observe, read, condition); compare, labels and
  decide are measured end to end in A23.

## External shapes (charter rule 17)
- Grounded by this session from the installed source: RESEARCH-VERIFY-11 (`reqwest 0.13.5` `error.rs`:
  `is_connect()` line 158 means no byte was sent; `is_timeout()` line 119 is also true for a connect timeout, so
  `is_connect()` is tested first; errors carry the URL, so the research HTTP seam strips it). Recorded in
  `master_plan_research.md` §7.4.
- Every row whose fixture a provider owns is in A15, A16 or A17, names a capture file that A01 or A02 writes, and
  cites its RESEARCH-VERIFY id. Those plans carry a "Fold before dispatch" paragraph for the coordinator: check
  the file names against the fixtures README and mark a row *structurally held* when A01 could only document a
  shape.
- Every other row is written against our own ports and their fakes.
- Not verified here because this session made no network call: the Unicode case-fold and general-category crates
  (A03 pins them), the HTML parser (A17). Each is an explicit task with the pin recorded.

## Points for the orchestrator (not cards)
1. **"Looks right" changes meaning.** MC-A12 makes it an accept (the value stays an AI value, status accepted);
   the form today turns it into a human edit. A08 implements the contract and updates two UI tests.
2. **Attribution keys are shop-specific.** A13 ships the two keys of 08A §6 as defaults so a designer field is
   never treated as a visible field. For a shop with other key names nothing stops a run; a refusal would need a
   ninth prerequisite in MC-A6.
3. **Address guard in the HTTP page reader** (A17) is ordered as a task and has no criterion, because no contract
   states it.
4. The intention map still says 08A/08B are "not planned yet".
5. `flow.md` log v1.3 names Opus for planning; this session ran on Fable at the owner's request of 2026-11-12. No
   plan sets `implementer: fable`.
6. A01 and A02 are implementer sessions that make real calls (P12 precedent). The implementer's sandbox must reach
   the network; otherwise launch with `PSM_IMPLEMENTER=claude`.

## Write perimeter (documents only; no code, no tool-recorded state, no commit)
- `docs/implementation/master_plan_research.md`
- `docs/implementation/phases_research.txt`
- `docs/implementation/plans/A01_research_probes_and_search_benchmark.md`
- `docs/implementation/plans/A02_research_stage_model_benchmark.md`
- `docs/implementation/plans/A03_research_values_and_sameness.md`
- `docs/implementation/plans/A04_research_sites_and_fingerprints.md`
- `docs/implementation/plans/A05_research_confidence.md`
- `docs/implementation/plans/A06_research_cost_and_call_rules.md`
- `docs/implementation/plans/A07_research_storage_and_write_permission.md`
- `docs/implementation/plans/A08_research_feedback_and_undo.md`
- `docs/implementation/plans/A09_research_publish_hold.md`
- `docs/implementation/plans/A10_research_ports_images_and_fakes.md`
- `docs/implementation/plans/A11_research_call_gateway.md`
- `docs/implementation/plans/A12_research_stages_and_decide.md`
- `docs/implementation/plans/A13_research_run_lifecycle.md`
- `docs/implementation/plans/A14_research_resume_reuse_follow_ups.md`
- `docs/implementation/plans/A15_research_model_adapter.md`
- `docs/implementation/plans/A16_research_visual_search_adapter.md`
- `docs/implementation/plans/A17_research_page_reader_and_shop_hosts.md`
- `docs/implementation/plans/A18_research_panel_ui.md`
- `docs/implementation/plans/A19_research_settings_ui.md`
- `docs/implementation/plans/A20_research_publish_ui.md`
- `docs/implementation/plans/A21_research_retention_and_export.md`
- `docs/implementation/plans/A22_research_report_and_replay_tool.md`
- `docs/implementation/plans/A23_research_acceptance_benchmark.md`
- `docs/implementation/plans/A24_research_defaults.md`
- `docs/implementation/handoffs/reviewer/handoff_research_plan_r1.md` (this file)

Not touched: `master_plan.md`, `phases.txt`, `backlog.md`, `STATUS.md`, the intention book, any code, the sibling
checkout.

## Checks run on the plan set
Every plan has the runner's frontmatter keys and the sections flow §9 names; every criterion row has five cells
and a trace cell; each "n criteria, n rows" heading equals the table's count; every phase id in the list file has
a plan file and a tracker row; every cited path either exists in the tree or is declared new in the registry.
Not checked by anything: whether a row's assertion is as strong as the row reads. Projection is on for the 17
phases that touch a silent-failure mechanism or a provider-owned fixture.
