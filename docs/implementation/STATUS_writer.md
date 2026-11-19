# STATUS, publication writer — read first (updated by the orchestrator at every stop)

Branch `feature/agentic`, worktree `~/Desktop/Developer/Photo-Session-Manager-agentic`. Never work in the sibling
checkout `~/Desktop/Developer/Photo-Session-Manager`; never edit `STATUS.md`.
Authority: book `08C_publication_writer.md` (RATIFIED 2026-11-19, changelog A22). Evidence:
`docs/intention/evidence/publication_writer_research.md`. Process: `docs/process/flow.md` §10 (milestones of units),
with the deltas of `master_plan_writer.md` once written. Implementer Claude Opus (`PSM_IMPLEMENTER=claude`), reviewer Opus.

- **2026-11-18:** the owner asked for a publication agent. The draft 08C was shaped with the owner.
  - Research subagent: `evidence/publication_writer_research.md`.
  - Owner answers on 2026-11-19 closed all §13 decisions:
    - "attributed to" is allowed
    - the era field is used as it stands
    - restoration becomes fixed choices
    - the shop name and "Ships from Sweden" go in when they fit
    - the research examples are the start
    - the same provider as research
    - the grade word comes from Detailed condition
    - at most one use sentence
- **2026-11-19:** RATIFIED (`84fae97`). The planner session started: master plan, `phases_writer.txt`, plans `W1…`,
  handoff `handoffs/reviewer/handoff_writer_plan_r1.md`.
- **Next:** read the planner handoff, relay any owner cards, then run the milestones in order with
  `PSM_IMPLEMENTER=claude tools/pipeline/run_phase.sh phase <id>` (in the background; no polling). Install after
  milestones the owner can try.
