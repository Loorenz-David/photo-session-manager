# STATUS, shooting templates — read first (updated by the orchestrator at every stop)

Branch `feature/agentic`, worktree `~/Desktop/Developer/Photo-Session-Manager-agentic` (in sync with `main` at
`23cd3c6`). Never work in the sibling checkout `~/Desktop/Developer/Photo-Session-Manager`; never edit `STATUS.md`.
Authority: book `10G_shooting_templates.md` (RATIFIED 2026-11-16 00:20). Plans: `phases_templates.txt`,
`master_plan_templates.md`, `plans/T1…T4`. Process deltas: 10G §10 (no inventory, no projection; Codex implements,
Opus reviews, one fix round). Deadline: on the work Mac the morning of 2026-11-16.

- **2026-11-16 ~01:00: plans written** (planner handoff `handoffs/reviewer/handoff_templates_plan_r1.md`: 4 phases, 13
  units, 22 criteria, 102 rows). **Owner card 1** (drag a placed photo onto another filled container: swap or delete):
  owner asleep, on-silence default **A, swap** is built (10G §10 delegation); tell the owner in the morning, B is one
  rule in one place. Phases chained overnight by `scratchpad/templates_chain.sh` (T1→T4, stops at the first non-zero
  exit). After T4: full checks, real-key scan, PR into `main`, merge (owner answer 4).
- **01:18 T1 fix BLOCKED only because the Codex fix sandbox refused `git add`**; orchestrator re-ran the settings
  tests (80/80), committed `a801f3a`, set the fix handoff DONE, resumed. **T1 APPROVED** (review r2).
- **01:35 T2: Codex hit its usage limit (until 03:43)** after Units 1–2 (`efcc86c`, `0bff768`); partial Unit 3
  committed as `fa8a484`, resume note in the T2 plan; chain restarted from T2 with `PSM_IMPLEMENTER=claude`
  (`scratchpad/templates_chain2.sh`, T2→T4).
- **02:03 T2 APPROVED_WITH_NOTES** (Claude implementer; one fix round: a dragged Main photo now keeps its place).
- **02:40 T3 APPROVED_WITH_NOTES** (one fix round: undo of a drag between extras restores both photos).
- **03:10 T4 APPROVED_WITH_NOTES** (one fix round: Shift the rest back shows only when something would move). Chain done.
- **Join checks on tip `929f2f1`:** cargo test, clippy `-D warnings`, fmt, vitest 538/538 (86 files), frontend build all
  green; real-key scan of tracked files and of every commit `origin/main..feature/agentic`: 0 hits. `origin/main` unmoved
  at `23cd3c6`, nothing to merge. PR into `main` and merge per owner answer 4.
- **Morning items for the owner:** card 1 built as swap (B = one rule); no hardware try-out yet (first run with the
  Sony is on the work Mac); small edge left: a placed photo dragged to behind the extras gets a dot without moving, ⌘Z
  does not clear it, Back to template does.
