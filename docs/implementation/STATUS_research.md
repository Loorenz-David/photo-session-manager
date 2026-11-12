# STATUS, research track — read first (updated by the research orchestrator at every stop)

Branch `feature/agentic`, worktree `~/Desktop/Developer/Photo-Session-Manager-agentic`. Never work in or run git
against the sibling checkout `~/Desktop/Developer/Photo-Session-Manager` (`main`, the owner's other work).
`docs/implementation/STATUS.md` belongs to `main`'s orchestrator; do not edit it from this track.

**2026-11-12: R2, R3, R4 APPROVED (with notes in `backlog_research/`); R5 running here, R1 on track B.** Intention 08A RATIFIED and contracts 08B RATIFIED (owner, 2026-11-12).
Ten milestones R1–R10 in `docs/implementation/phases_research.txt`; rules in `docs/process/flow.md` §10; skeleton,
tracks and driver loop in `docs/implementation/master_plan_research.md`. `main` (with P70, the fast suite) was merged
into this branch at `2026-11-12`; the branch is local, not pushed.

- **Done:** `R2_research_foundation` APPROVED_WITH_NOTES 2026-11-12 (implemented by Codex, the runner's default, before the
  Opus preference below was applied; reviewed by Opus). Narrative relayed to the owner.
- **Done:** `R3_research_rules` APPROVED_WITH_NOTES 2026-11-12 (Opus implementer). Notes folded: PriceMissing scope → R4
  plan task 2; MC-A4 invariant scoped → 08B changelog 4. Other notes stay in `backlog_research/R3_research_rules.md`
  (→ R6/R4·U3 Details+Product field; → R8 quota-vs-key order). Narrative relayed.
- **Done:** `R4_research_engine` APPROVED_WITH_NOTES 2026-11-12. All routed notes of R2–R4 folded into R5–R10 plan files
  ("Carried from earlier reviews" sections; implementers never read `backlog_research/`), master plan v3, 08B
  changelog 5 (MA12 sent requests). Two notes are the owner's (below, "Needs the owner").
- **Run in flight:** `PSM_IMPLEMENTER=claude tools/pipeline/run_phase.sh phase R5_research_app_seams`, attempt 5, started 2026-11-12 17:07 (track A). Attempt 4 stopped at projection (Q1–Q4, small; amended, 08B changelog 8, archived). Attempt 3 stopped at projection (P1–P6, small; amended, archived). Attempt 2 stopped at projection again (N1–N9, publish-side gaps, 0 owner cards; amended, archived). Attempt 1 stopped at projection r0 (AMENDMENTS_REQUIRED, 0 owner cards); plan amended L1–L15, 08B changelog 6, attempt archived.
  Implementer on Opus via `PSM_IMPLEMENTER=claude` (the runner defaults to Codex; set the variable for every research run).
- **Next on track A:** R6 → R7 (needs R4, R6) → R9 (needs R1, R8 merged from B) → R10. Schedule changed
  (master plan v3): R5 and R6 moved to track A because track A was idle and B is busy with R1. Track B after R1:
  merge `feature/agentic` into `research/track-b` first (R4 and plan edits), then R8 once the owner's picks arrive.
- **Track B (owner said yes 2026-11-12):** worktree `~/Desktop/Developer/Photo-Session-Manager-agentic-b`, branch
  `research/track-b`, created at `2cb6454` (R3 approved; before any R4 commit), `npm ci` done.
  **Run in flight there:** `PSM_IMPLEMENTER=claude tools/pipeline/run_phase.sh phase R1_research_benchmark`, started
  2026-11-12 15:27. Real paid calls, ceiling USD 50 (plan). Keys `~/.psm_research.env` (0600) and `~/.psm_shopify.env`
  present (presence only). Then R8 on track B (R5, R6 moved to track A). Merge rule (master plan §5): after each approved B milestone
  merge `research/track-b` into `feature/agentic` here; after each approved A milestone merge `feature/agentic` into
  track B before B's next milestone. Track B's copy of this file is not maintained; this one is.
- **Owner answered 2026-11-12 (both "yes"):** a capped run lowers nothing (→ R7·U1 carried note); saving keys or
  Automatic on starts nothing (owner withdrew the "start the open session" pick: no extra work). 08B changelog 7.
- **Blocked on the owner:** R8 (search route and page-reading route, picked from R1's report); R10 (models per stage,
  picked from R9's report). Picks arrive as decision cards in the R1 and R9 narratives.
- **Unverified, watch for it:** this worktree has never been built (first cargo run is a full compile); the three
  template edits (`tools/pipeline/templates/{implement,review,fix}.md`) have never run; the research crate using the
  testkit as a dev-dependency while the testkit depends on it (first exercised in R3 unit 5); the planner assembled
  the 29 units from the earlier A-plans by script and spot-checked only some, so a unit's "What changed" list wins
  over older task wording inside it.
- **Owner preferences for this track:** implementation on Opus (Codex first only if the owner says so); speed over
  ceremony; app tests only where app files changed (flow §10); plain-language narratives per milestone.
