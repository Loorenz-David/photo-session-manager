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
  - First planner launch failed at once (prompt starting with `---` passed as an argument was read as a CLI option);
    relaunched with the prompt on stdin, as the runner does.
  - **Plan written** (planner handoff `handoffs/reviewer/handoff_live_plan_r1.md`, OWNER_DECISIONS_PENDING, 1 card):
    LV1 camera live view to the hub (default implementer), LV2 current shot and phone server (default; the riskiest:
    placement rewrite under MC-LV3), LV3 phone app (`implementer: fable`), LV4 owner-attended hardware check (never
    dispatched by the runner). 15 units, 91 criteria, 261 rows; projection off. Planner installed `node_modules` in
    this worktree (`npm ci`; git-ignored).
  - Planner finding for later: `cargo test ... --test suite a:: b::` with two filters before `--` is refused; flow §4A
    wording could be corrected (not done; main's flow file).
  - Plan relayed to the owner; **waiting for the owner's go** before LV1.

## Next
When the planner returns: relay the milestone map and any owner cards verbatim; **wait for the owner's go** (this
track's wake prompt overrides orchestrator.md's "start immediately"). Then `tools/pipeline/run_phase.sh phase <id>`
per line of `phases_live.txt`, in the background, never polled; push `feature/live-preview` at the end of a run; PR to
`main` when all milestones are approved.

## Needs the owner
- Go for the build (LV1 → LV2 → LV3, then LV4 together).
- Card 1 (verbatim in the planner handoff): where the phone's "Article number" comes from — (a) barcode, (b) a shop
  product field the owner names (recommended), (c) leave it out. On silence the phone hides that row; nothing waits.
