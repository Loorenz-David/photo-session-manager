# STATUS, research track — read first (updated by the research orchestrator at every stop)

Branch `feature/agentic`, worktree `~/Desktop/Developer/Photo-Session-Manager-agentic`. Never work in or run git
against the sibling checkout `~/Desktop/Developer/Photo-Session-Manager` (`main`, the owner's other work).
`docs/implementation/STATUS.md` belongs to `main`'s orchestrator; do not edit it from this track.

**2026-11-12: planned, nothing implemented.** Intention 08A RATIFIED and contracts 08B RATIFIED (owner, 2026-11-12).
Ten milestones R1–R10 in `docs/implementation/phases_research.txt`; rules in `docs/process/flow.md` §10; skeleton,
tracks and driver loop in `docs/implementation/master_plan_research.md`. `main` (with P70, the fast suite) was merged
into this branch at `2026-11-12`; the branch is local, not pushed.

- **Run in flight:** `tools/pipeline/run_phase.sh phase R2_research_foundation`, started 2026-11-12 14:34 (track A, this worktree).
- **Next after R2:**  Then Track A here: R3 → R4 → R7 → R9 → R10.
- **Ready, needs a second worktree (Track B, branch `research/track-b`):** `R1_research_benchmark`. The keys file
  `~/.psm_research.env` exists with the three names set (checked for presence only; the keys have not been used).
  Then R5 → R6 → R8.
- **Blocked on the owner:** R8 (search route and page-reading route, picked from R1's report); R10 (models per stage,
  picked from R9's report). Picks arrive as decision cards in the R1 and R9 narratives.
- **Unverified, watch for it:** this worktree has never been built (first cargo run is a full compile); the three
  template edits (`tools/pipeline/templates/{implement,review,fix}.md`) have never run; the research crate using the
  testkit as a dev-dependency while the testkit depends on it (first exercised in R3 unit 5); the planner assembled
  the 29 units from the earlier A-plans by script and spot-checked only some, so a unit's "What changed" list wins
  over older task wording inside it.
- **Owner preferences for this track:** implementation on Opus (Codex first only if the owner says so); speed over
  ceremony; app tests only where app files changed (flow §10); plain-language narratives per milestone.
