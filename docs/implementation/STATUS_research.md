# STATUS, research track — read first (updated by the research orchestrator at every stop)

Branch `feature/agentic`, worktree `~/Desktop/Developer/Photo-Session-Manager-agentic`. Never work in or run git
against the sibling checkout `~/Desktop/Developer/Photo-Session-Manager` (`main`, the owner's other work).
`docs/implementation/STATUS.md` belongs to `main`'s orchestrator; do not edit it from this track.

**2026-11-12: R2–R7 APPROVED (with notes in `backlog_research/`); nothing running; R1 on track B waits on an owner card.** Intention 08A RATIFIED and contracts 08B RATIFIED (owner, 2026-11-12).
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
- **Done:** `R5_research_app_seams` APPROVED_WITH_NOTES 2026-11-12 18:44 (attempt 8; seven projection rounds before it,
  all archived; 08B changelogs 6, 8, 9, 10). Notes folded into R6·U3 (release events, undo after release, p15 fixture).
- **Done:** `R6_research_ui` APPROVED_WITH_NOTES 2026-11-12 19:12. Notes folded into R7·U1 (settings save, release signal
  incl. undo/redo, wait clock, `conflict` field + bindings allowance, Accept all count); the rest stays in
  `backlog_research/R6_research_ui.md` (popover Edit for non-text fields, shop public domain shown).
- **Done:** `R7_research_wiring_and_export` APPROVED_WITH_NOTES 2026-11-12 19:52. Notes folded into R8 (export secret scan,
  `wait_ends_at` row, long calls) and R9 (TZ pin); event→change rule stays in `backlog_research/`.
- **Owner, 2026-11-12 night:** attached billing (card A) and delegated: "take care of this and continue resolve
  questions given the intention" while asleep. The orchestrator verified the link in the console, ran the Vision
  benchmark (Lens 24/25 + 6/7 vs Vision 8/25 + 0/7), merged `research/track-b` into `feature/agentic` (`3297b17`,
  workspace check green), set the R1 implement handoff to DONE with a resolution note.
- **Done:** `R1_research_benchmark` APPROVED_WITH_NOTES 2026-11-13 01:12, 3 owner cards. Picks made by the orchestrator under
  the owner's delegation, all as recommended, recorded in the evidence doc "Owner picks": Lens via SerpApi (Vision
  the replacement, not built), plain HTTP with slow Shopify hosts, models observe Sol low / read Terra low /
  condition Terra high. R1 notes folded into R8·U3 and R9·U1/U2. Left for the owner: piece 30's corrected wood
  "Rosewood" is not a shop choice ("Santos Rosewood" is) in `docs/test-items/answer_key_corrections.json`;
  pamono country sites need their own trusted entries if wanted. Track B's build folder: deletion was denied by
  the permission prompt; the owner may delete `../Photo-Session-Manager-agentic-b/target`.
- **Done:** `R8_research_real_adapters` APPROVED_WITH_NOTES 2026-11-13 02:30 (attempt 3; two projection rounds archived;
  08B changelog 11; master plan v5–v6). Page-reader notes (charset, proxy, address ranges) folded into R9·U1 task 0;
  §3 Isolation lesson applied (v7). Not yet run live against the real services.
- **Run in flight:** `PSM_IMPLEMENTER=claude tools/pipeline/run_phase.sh phase R9_research_tool_and_acceptance`, started
  2026-11-13 02:31 (track A). It builds the tool and `accept`; the **measurement run (U2 tasks 3–6, real keys, ceiling
  USD 40) is owner-attended** and waits for the owner after the runner returns.
- **Next on track A:** the owner-attended measurement, then R10 (models from its result). Schedule changed
  (master plan v3): R5 and R6 moved to track A because track A was idle and B is busy with R1. Track B after R1:
  merge `feature/agentic` into `research/track-b` first (R4 and plan edits), then R8 once the owner's picks arrive.
- **Track B (owner said yes 2026-11-12):** worktree `~/Desktop/Developer/Photo-Session-Manager-agentic-b`, branch
  `research/track-b`, created at `2cb6454` (R3 approved; before any R4 commit), `npm ci` done.
  **R1 stopped there (exit 10, 18:02): implementer OWNER_DECISIONS_PENDING, 1 card (Google Cloud Vision billing).**
  Both units done and checkpointed (U1 `60f6d6c`, U2 `7b3a2f8`, closing `0446282`); USD 5.82 spent of 50. Not reviewed yet.
  Handoff `docs/implementation/handoffs/implementer/handoff_R1_research_benchmark_implement_r1.md` on track B. Was started
  2026-11-12 15:27. Real paid calls, ceiling USD 50 (plan). Keys `~/.psm_research.env` (0600) and `~/.psm_shopify.env`
  present (presence only). Then R8 on track B (R5, R6 moved to track A). Merge rule (master plan §5): after each approved B milestone
  merge `research/track-b` into `feature/agentic` here; after each approved A milestone merge `feature/agentic` into
  track B before B's next milestone. Track B's copy of this file is not maintained; this one is.
- **Owner answered 2026-11-12 (both "yes"):** a capped run lowers nothing (→ R7·U1 carried note); saving keys or
  Automatic on starts nothing (owner withdrew the "start the open session" pick: no extra work). 08B changelog 7.
- **Blocked on the owner:** R10 (models per stage,
  picked from R9's report). Picks arrive as decision cards in the R1 and R9 narratives.
- **Unverified, watch for it:** this worktree has never been built (first cargo run is a full compile); the three
  template edits (`tools/pipeline/templates/{implement,review,fix}.md`) have never run; the research crate using the
  testkit as a dev-dependency while the testkit depends on it (first exercised in R3 unit 5); the planner assembled
  the 29 units from the earlier A-plans by script and spot-checked only some, so a unit's "What changed" list wins
  over older task wording inside it.
- **Owner preferences for this track:** implementation on Opus (Codex first only if the owner says so); speed over
  ceremony; app tests only where app files changed (flow §10); plain-language narratives per milestone.
