# STATUS — live preview (book 10J)

Running log of the live preview track on branch `feature/live-preview`, worktree
`~/Desktop/Developer/Photo-Session-Manager-live`. The main tracker `STATUS.md` and other tracks' files are not edited
by this track. Wake prompt: `docs/process/prompts/wake_orchestrator_live.md`.

## Where things stand
- **2026-11-26:** 10J RATIFIED (D18). Orchestrator started on Opus 5.5.
  - Planning prompt written: `docs/implementation/prompts/reviewer/live_plan_r1.md` (owner constraints: ~3 build
    milestones + 1 owner-attended hardware check, units with CHECKPOINT commits, one review per milestone; no
    inventory session, contracts section in the master plan; `implementer: fable` on phone-UI milestones only;
    projection off; phone app only, plain HTTP; sidecar live view/focus/capture-at-point/clean quit ported from the
    lab; Rust hub; no desktop delivery now; fake sidecar; guide updates in the same change).
  - **Run in flight:** planner (`claude -p --agent pipeline-planner --model opus`), started 2026-11-26, log
    `.pipeline/runs/live_plan_r1.log`.

## Next
When the planner returns: relay the milestone map and any owner cards verbatim; **wait for the owner's go** (this
track's wake prompt overrides orchestrator.md's "start immediately"). Then `tools/pipeline/run_phase.sh phase <id>`
per line of `phases_live.txt`, in the background, never polled; push `feature/live-preview` at the end of a run; PR to
`main` when all milestones are approved.

## Needs the owner
Nothing yet.
