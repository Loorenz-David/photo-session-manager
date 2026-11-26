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
  - **Card 1 answered (owner 2026-11-26):** "the article number is the product barcode in shopify" → branch (a):
    `article_number` = the item's barcode (already read from the Shopify variant barcode); LV2·C9(b), LV3·C24, master
    plan PH6/§9 amended.
  - Plan relayed to the owner. **Owner go (2026-11-26):** "we can go! … I expect you complete the three steps
    autonomously"; resolve blocked sessions from the intention and the plan; if Codex runs out entirely, continue on
    Opus (runner change, flow log v1.7).

  - **LV1 run 1** (10:15–11:08): implemented (Codex Sol high, 3 unit checkpoints), review r1 CHANGES_REQUESTED (3 findings:
    frame listener across helper restarts, frame-channel failure stopping focus, a test expecting a derived value),
    fix r2 done and green but BLOCKED on its commit (Codex sandbox could not write the worktree's git lock under the main
    repo's `.git`). Orchestrator committed the fix (`f7d03aa1`), set the handoff DONE, added the git common dir to the
    runner's Codex sandbox, resumed for the re-review.
  - **LV1 re-review r2:** CHANGES_REQUESTED, one MUST (R2-B1: helper restart while live view is off leaves a dead frame
    connection; next camera session gets no frames; reproduced 3/3). FIX2_AUTHORISED in the plan's Review log; resumed.
  - **LV1 fix 2** (Codex Luna): R2-B1 fixed (`7bba1e49`); BLOCKED only on a P50 test failing inside the Codex sandbox
    (re-signing the bundled helper); passes outside it (orchestrator run). Handoff set DONE; runner gained a resume guard
    for fix r3; resumed for re-review r3.
  - **LV1 APPROVED** (`0f56b860`, re-review r3, APPROVED_WITH_NOTES; narrative relayed). Notes in
    `backlog_live/LV1_camera_live_view.md`. Branch pushed. **LV2 started.**
  - **LV2 APPROVED** (`APPROVED_WITH_NOTES`, review r1, no fix; narrative relayed). Codex hit its usage limit at
    11:58 (resets 15:15); the runner fell back to Claude Opus by itself (implement 11:58–13:05). Reviewer notes routed to
    LV3 folded into the LV3 plan as Unit 0 (phone Delete events, session-delete announce, rebuild observe, generic error
    bodies; rows C26–C29); master plan §8 gate runs `npm run build` first. Pushed. **LV3 started** (Fable).
  - **LV3 APPROVED** (review r1, APPROVED_WITH_NOTES, no fix; Fable implement 13:14–13:56, 5 units incl. U0; narrative
    relayed). Six notes in `backlog_live/LV3_phone_app.md` (capture toast matching and the look comparison with the
    prototype go to LV4). Plan lesson folded (jsdom has no PointerEvent, master plan §3). **All three build milestones
    approved**; pushed; pull request to `main` opened; waiting for the owner.

## Next
The owner reviews the pull request and the three narratives. Then LV4, the owner-attended hardware check (never
dispatched by the runner): packaged build with the SDK sidecar, both phones, the a7R V; the plan is
`plans/LV4_hardware_check.md`. Open notes for LV4: capture toast on real captures, look vs. the prototype, revoke while
connected, LV1 carry-overs.

## Run in flight
None.

## Needs the owner
- Review / merge the pull request to `main`; schedule the hardware check (LV4).
- Possible gap to raise: the phone does not draw the camera's own focus frame (10J LV-D14 "the live view shows where
  the camera is focusing"); only the tap marker and "AF LOCKED".

