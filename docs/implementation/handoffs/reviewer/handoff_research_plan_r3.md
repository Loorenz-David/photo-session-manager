---
plan: research
role: plan
round: 3
state: OWNER_DECISIONS_PENDING
date: 2026-11-15
actor: Claude Fable 5.2 (pipeline-planner, implementation-planner doctrine; orchestrated)
---
# Research agent — plans for R11 (research starts when the session is arranged) and R12 (research costs less)

**Summary.** Two milestone plans written in the format of R7 and R10 and registered: `R11_research_on_arranged`
(6 units, 33 criteria, 99 rows `A25·…`, changes app files) and `R12_research_cost` (4 units, 16 criteria, 37 rows
`A26·…`, 3 of them `[manual]`, research and tool crates only). Gate: intention map RATIFIED (2026-10-24), 08A RATIFIED
and 08B CONTRACTS RATIFIED (2026-11-12); 08B changelog 12 and 13 are the owner decisions implemented. Documents only: no
code, no cargo, no external call, no commit. One owner card, which blocks nothing (the table it asks for is a setting).

## ⚠ OWNER DECISIONS REQUIRED (1)

**Card 1 — the detailed-condition table**

**Question** — Which grade should research write in "Detailed condition" for each worst condition level: None → Very Good, Minor → Very Good, Moderate → Good, Major → Worn, and never Excellent?

**Story** — A teak sideboard has Scratch: Minor, Veneer damage: Moderate, the rest None. Today research guesses the grade from the photos and was wrong on 19 of 42 pieces. Your own listings do not settle a rule: with Minor as the worst level you wrote Very Good 17 times and Good 5 times; with Moderate, each twice.

**Branches** — **A**, the table above: agrees with 21 of your 28 graded listings. **B**, stricter (Minor → Good, Moderate → Worn): agrees with 7 of 28. **C**, your own table, entered in Settings → Research.

**Recommendation** — A, because it agrees with most of what you already published.

**On silence** — nothing is derived: the table ships empty and Detailed condition is filled as it is today until you enter one.

**Trace** — R11·U3 rows A25·C12–C14, U6 row A25·C31; 08B changelog 12 (b).

## Write perimeter (complete)
| File | Change |
|---|---|
| `docs/implementation/plans/R11_research_on_arranged.md` | new |
| `docs/implementation/plans/R12_research_cost.md` | new |
| `docs/implementation/master_plan_research.md` | §0 (ids), §3 (`[manual]`, real calls, ceiling, projection), §4 (two rows, totals sentence), §5 (graph, seams, schedule, merge order, bindings sentence, ledger coverage), §6.3, §6.5, §6.6 (pointer), new §6.8, §7.2 (two rows), new §7.4A, §8 (two points), §9 (v10 line) |
| `docs/implementation/research_row_mapping.md` | appended: "Rows added by R11 and R12" (plan numbers A25, A26; rows of earlier plans retired or amended; every new row) |
| `docs/implementation/phases_research.txt` | two ids and one comment line appended |
| `docs/implementation/handoffs/reviewer/handoff_research_plan_r3.md` | this file |
No code, no test, no evidence doc, no book chapter, no `STATUS.md`, no tracker line of another plan, no archgraph state. Read-only outside the worktree: `~/psm-research-bench/a01/pages/calls` (R1's recorded listing pages), to ground R12·U2 under charter rule 17; the figures are in `master_plan_research.md` §7.4A. Scratch scripts only in the session scratchpad.

## Counts (computed from the plan files)
| Milestone | Unit | Criteria | Rows | Changes app files |
|---|---|---|---|---|
| R11 | U1 the arranged trigger in the engine | 7 | 22 | tests only (three testkit files) |
| R11 | U2 fields left alone, extra shop addresses, reset | 4 | 12 | no |
| R11 | U3 detailed condition derived; no attribution from another shop product | 5 | 26 | no |
| R11 | U4 session mark, "Arranged — next", publishing marks first | 7 | 17 | yes (full app suite at the closing gate) |
| R11 | U5 button, list tag, publish line, panel hint | 4 | 9 | yes (TypeScript) |
| R11 | U6 Settings pickers, detailed-condition table, reset | 6 | 13 | yes (TypeScript) |
| **R11** | | **33** | **99** | closing gate: full gate + `cargo test -p psm-research` |
| R12 | U1 photo look in one call | 7 | 17 | no |
| R12 | U2 product part of a listing page | 2 | 9 | no |
| R12 | U3 same kind or no comparison | 3 | 6 | no |
| R12 | U4 second acceptance measurement | 4 | 5 (3 `[manual]`) | no |
| **R12** | | **16** | **37** | closing gate: research and tool crates + the `research::` app stems |

## Book text superseded (each plan carries the table; the orchestrator folds it into 08A and 08B)
**R11.** 08A §5 "Start" and "Quiet period"; 08B MC-A6 event-table rows for a first Main photo and for a photo-derived change, the "Quiet period" paragraph, the `auto`-off row (R-IA25), the eighth prerequisite, and MA13's quiet-period clause · 08A §5 "New photos after a run"; 08B MC-A7 "Follow-up trigger set" as a trigger (it now decides an "Arranged" press) · 08B MC-A7 "Reuse" (also in an `arranged` run) · 08B changelog 12 (c) (covered by changelog 13, no separate deferral of stage 6) · 08A §6 `high` row, second clause, and 08B MC-A4 `high(v)`, `medium(v)`, "Support" and enumerated rows 2 and 3, **for attribution fields only** (a page of the shop counts at no level) · 08B MC-A4 "Field class" (two new steps: left alone, derived) · 08A §6 visible-field rule for the detailed-condition field once the setting names it · 08B MC-A3 "Shop hosts" (plus `extra_shop_hosts`).
**R12.** 08A §5 stage table row 1 and 08B MC-A7 Units row 1 (one stage 1 unit over the chosen photos; owner card 2 = B is superseded for stage 1 only, on the owner's cost idea of 2026-11-13; Compare still uses the cover) · 08A §5 "Photos used" (at most `observe_max_photos`) · 08B MC-A7 follow-up trigger set wording for stage 1 · MC-A7 Units rows 3 and 4 (Read identity gains the item type; no Compare unit for a page that lists another kind) · `RESEARCH_FINGERPRINT_VERSION` 1 → 2, instructions version 2 · 08A §4 page reader text (the product extract).

## Where the code or the artifacts contradict the scope given
1. **"The existing condition-photos field picker" does not exist.** It is a text box ("Condition photos key", `SettingsPage.tsx`); the attribution keys are text boxes too. R11·U6 builds one picker component and uses it three times; the condition-photos box is left as it is.
2. **No shoot template exists** in code or book (08A §3 "Not existing", §12 deferred). R12·U1 uses position only (first Main photo wide, the rest closer) and states what a template would need: per item type a name per Main position and the fields each serves.
3. **"Own-shop listing counts only when it is this item's own listing" meets R-IA9**, which excludes that page from reading and evidence (`rules::sites::reading_order`). Resolved as: for designer and maker no shop page is evidence; the own listing stays unread. No observable difference (it can only state what the form already holds).
4. **"Skip the model when structured data states the field outright" has no production cause.** On R1's 375 recorded pages no trusted site states designer or maker in a structured field; `brand` names the seller (the shop's own pages say "Beyo Vintage"); one site gives a `creator` on 3 of 35 pages. Not built; the structured block goes to the model as text.
5. **The parser saves little money after R10.** Read moved to a model at about a tenth of the price; the extract mainly removes "more from this dealer" lists (pamono pages are about 20,000 characters, the product part about 10,000). Observe is now about half of a run, so R12·U1 is the lever.
6. **App tests depend on the research crate's behaviour** (`research::r7_wiring`, `r7_export_schedule` start runs with `auto` on). So R11·U1, a research-crate unit, edits three testkit files, and R12's closing gate runs the `research::` stems although it changes no app file. Recorded in `master_plan_research.md` §7.2, §8.
7. **Opening a session routes captures to it** (`App.tsx`). "Arranged — next" therefore also moves where the camera's files go, like opening any session.
8. `master_plan_research.md` §4 lists R8 as 18 / 29 while its v6 log line says 20 / 35; not touched (another phase's row).

## Decisions made by planning (no card; each is in the R11 or R12 plan and in the v10 log line)
The mark is `sessions.arranged_at`, added by the ensure step, never a numbered migration · `auto` and `quiet_seconds` are removed, not kept inert; old slots load · "next" is the nearest unmarked session created after the open one, else the oldest unmarked (published, archived, empty and Unassigned skipped) · publishing marks at the popover's preview; Confirm is never disabled · reset keeps trusted sites, designer and maker fields, fields left alone, extra addresses, the detailed-condition table and the language · row ids: one plan number per plan (A25, A26), so the `a2x_cNy_<slug>` convention holds without running past A29 · R12·U4's spending ceiling is USD 20.

## For the orchestrator
- Fold the two "What this plan supersedes in the book" tables into 08A and 08B as changelog entries before or with R11's launch (only the orchestrator and the owner edit the book).
- `templates/implement.md` needs no change. R12 is launched like R9: the implementer stops after U4 task 2.
- R12·U4 task 2 uses the card's answer; unanswered, the settings file carries no `detailed_condition`.
- No plan sets `implementer:`; both have `projection: no`. R11·U4 touches publish preview but not the hold (MC-A10); if a projection is wanted for it, that is one frontmatter word.
