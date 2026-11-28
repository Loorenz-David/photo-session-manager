You are the orchestrator of the **live preview** build of Session Manager (book chapter 10J, RATIFIED 2026-11-26),
on branch `feature/live-preview`, in this worktree (`~/Desktop/Developer/Photo-Session-Manager-live`). Never touch the
`main` checkout, `STATUS.md` or other tracks' files; keep your memory in `docs/implementation/STATUS_live.md` (create
it) and update it at every stop.

Read, in order: `docs/process/orchestrator.md`; `docs/process/flow.md` (§10 milestones win for this track);
`docs/intention/book/10J_live_preview.md` (all of it: decisions LV-D1…D18, contracts MC-LV1…3, ledger M-LV1…6,
answered cards); the parts it amends: `10E_direct_camera_adapter.md`, `10G_shooting_templates.md` §4 note,
`11_architecture_and_packaging.md` §Phone remote, `docs/guide/03_rules_and_principles.md` camera rules; the evidence
`docs/intention/evidence/sony_camera_lab.md` §Live view and §Focus, and `docs/intention/evidence/live_preview_lab.md`.
The owner's phone design is local and git-ignored: `claude_design/live_preview/design_handoff_shoot_companion/`
(README is the authority for look, gestures, thresholds, haptics; high fidelity). The hardware-proven reference code
is the owner's camera lab, read-only: `~/Desktop/Developer/Camera-Connection-Tests` (`sdk/src/camera_liveview.cpp`,
`sdk/src/main.cpp` `cmd_liveview` with `--commands`, `camlab/pwa/`, `experiments/07…11`).

**1. Plan.** Write a planning prompt to `docs/implementation/prompts/reviewer/live_plan_r1.md` and launch one planner
session in the background (as the writer and labels tracks were planned):
`claude -p --agent pipeline-planner --model opus "$(cat docs/implementation/prompts/reviewer/live_plan_r1.md)"`.
It writes `docs/implementation/master_plan_live.md`, `docs/implementation/phases_live.txt`, one plan per milestone in
`docs/implementation/plans/LV<n>_*.md`, and `docs/implementation/handoffs/reviewer/handoff_live_plan_r1.md`.
The owner's planning constraints, to put in that prompt:
- **Few, grouped milestones** (flow §10: a milestone = ordered units with CHECKPOINT commits, one review at the end),
  aiming at about three build milestones plus one owner-attended hardware check, not many small phases; checks happen
  at milestone ends so time and tokens are not spent per unit.
- **No separate mechanism-inventory session**: the master plan carries a contracts section (as `master_plan_templates`
  §5 and `master_plan_writer` WC1…) that makes contract-grade the silent-failure mechanisms: MC-LV1 frames and their
  sequence/dedupe by camera frame number, MC-LV2 who receives frames, **MC-LV3 the current shot** (every exposure bound
  to the shot current at `exposed_at`, late Wi-Fi files, the camera's own button, Retake = deny now), the sidecar
  frame channel and command/clean-stop protocol, focus on its own thread, phone pairing token.
- **Complex UI and interactions are built and fixed by Fable** (`implementer: fable` on the phone-UI milestones);
  backend and sidecar by the default implementer. Projection off unless the planner sees a reason.
- Scope: the phone app only (the Mac's live view screen waits, card LV10); plain HTTP, no secure-context API; the
  sidecar gains live view, focus point, capture-at-point and clean quit ported from the lab; a Rust live-view hub;
  desktop delivery (Tauri channel) is not built now. Tauri and the HTTP server stay thin (H8); nothing Sony-specific in
  core (H7); never write exposure, network or FTP settings; the automated suite never needs the camera or a phone
  (fake sidecar); the last milestone is the owner-attended hardware check of M-LV1…M-LV6 using the lab procedures.
- Guide updates in the same change (flow §7A).

When the planner returns, relay the milestone map in plain words (what each builds, what I can try after it) and any
owner cards verbatim; wait for my go.

**2. Build.** Then run each milestone with `tools/pipeline/run_phase.sh phase <id>` in the background, never polling,
in `phases_live.txt` order, continuing while the runner returns success. Relay each narrative verbatim. Push
`feature/live-preview` at the end of a run; when all milestones are approved, open a pull request to `main` and wait
for me.

Talk to me in plain narrative: what happened, what I can try, choices made for me, what needs me. Nothing technical,
no plan references.
