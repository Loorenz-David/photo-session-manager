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
- **2026-11-19: plans written** (`1d9b985`; handoff `handoffs/reviewer/handoff_writer_plan_r1.md`).
  - 6 milestones: W1 product note, W2 engine, W3 review, W4 settings, W5 SEO publish (projection), W6
    restoration choices. 17 units, 73 criteria, 231 rows. Contracts WC1–WC8 in `master_plan_writer.md` §5
    stand in for the inventory.
  - **Owner card 1** (Restoration choices only in the app (A, recommended) or as a new Shopify field (B))
    gates W6 only. Relayed.
  - W5 U2 task 1 writes a test SEO title on the probe product `PSM-PROBE`, using `~/.psm_shopify.env` (the owner's
    test store per the 2026-10-26 answer), then restores it. Confirm with the owner that this file still points to
    the test store before W5.
  - **Chain W1→W4 started** in the background: `scratchpad/writer_chain.sh`, log `scratchpad/writer_chain.log`.
- **2026-11-19: owner answers.**
  - Card 1 = **A**: restoration choices stay in Session Manager; the customer reads them in the condition report
    description. W6 unblocked; master plan rule 11 now checks the condition report only.
  - The store in `~/.psm_shopify.env` is now the **real shop**. W5 U2 task 1 amended: the probe creates its own DRAFT
    test product, probes it, deletes it, and touches no existing product.
  - Chain: W1 done, W2 under review (10:36).
- **2026-11-19 12:02: W1–W4 APPROVED** (all APPROVED_WITH_NOTES; narratives in `narratives/W1…W4`). Release build of
  `143369f` installed on the dev Mac (previous build kept as "2026-11-18 A21"); fresh zip on the Desktop.
  - W5 gains **U4, carried fixes** from the W1–W4 reviewer notes: "A 1 cm" read as a model number, findings about
    texts never written, a silent answer, "$0.00" costs, the stuck condition banner, the guide rows and the
    "shopify" constant. Master plan WC1.7, WC5.10(iii) and WC6 rule 2 amended.
  - Chain W5→W6 started: `scratchpad/writer_chain2.sh`, log `scratchpad/writer_chain2.log`.
