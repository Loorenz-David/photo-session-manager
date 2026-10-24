# Implementation folder

Written and committed by the pipeline runner (`tools/pipeline/run_phase.sh`) and the role
sessions it launches. Process rules: `docs/process/flow.md`.

- `master_plan.md`, `plans/`, `phases.txt` — written by the planner.
- `prompts/<role>/` — prompts the runner rendered from `tools/pipeline/templates/`.
- `handoffs/<role>/` — session reports; the runner reads their frontmatter (`state`, `verdict`).
- `narratives/<phase>.md` — the owner's plain-language result per phase. Start here.
- `backlog.md` — reviewer notes that did not block.
- `state/<phase>.json` — runner state per phase.
