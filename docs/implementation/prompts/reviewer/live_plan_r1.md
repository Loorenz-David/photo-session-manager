---
plan: live
role: plan
round: 1
date: 2026-11-26
---
# Plan the live preview track (book 10J) — phone app, sidecar live view, hub, current shot

Repository root: `/Users/davidloorenz/Desktop/Developer/Photo-Session-Manager-live` (git worktree, branch
`feature/live-preview`; never touch the `main` checkout, `docs/implementation/STATUS.md` or other tracks' files).

## Gate and read order
Gate: `docs/intention/book/10J_live_preview.md` reads **RATIFIED** (owner 2026-11-26, changelog D18). There is **no
separate mechanism-inventory session** for this track (owner): your master plan's contracts section stands in for it,
as `docs/implementation/master_plan_templates.md` §5 and `docs/implementation/master_plan_writer.md` §5 (WC1…) did.
Record that as a process delta.

Read, in order:
1. `docs/process/flow.md` — the process authority; it wins over your doctrine where they differ. **§10 (milestones of
   units) applies to this track**, adapted below. Also `docs/guide/README.md`, `03_rules_and_principles.md`,
   `04_change_impact.md`.
2. `docs/intention/book/10J_live_preview.md`, all of it: decisions LV-D1…LV-D18, the working loop, the phone design
   section, the architecture, contracts MC-LV1…MC-LV3, ledger M-LV1…M-LV6, answered cards LV3…LV10.
3. What 10J amends or builds on: `10E_direct_camera_adapter.md` (MC-31 sidecar protocol, MC-32 FTP, MC-33 capture
   ledger and `exposed_at`, MC-34 ownership, the "own frame channel" it reserved), `10G_shooting_templates.md` (§3–§6,
   with the §4 note that MC-LV3 replaces rule 2 and §6's refill/retake), `11_architecture_and_packaging.md` §Phone
   remote, `docs/guide/03_rules_and_principles.md` §Camera hard rules.
4. Evidence: `docs/intention/evidence/sony_camera_lab.md` §Live view — Experiment 07 and §Focus from the live picture —
   Experiment 10; `docs/intention/evidence/live_preview_lab.md` (all).
5. The owner's phone design, local and git-ignored, **high fidelity**:
   `claude_design/live_preview/design_handoff_shoot_companion/README.md` and its `prototype/`. The README is the
   authority for look, gestures, thresholds, timings and haptics; the book (LV-D15…LV-D18, cards LV5…LV10) wins where
   the two differ (e.g. after the last template shot captures are Condition, the phone's Move swaps, × on AF lock
   recentres the camera's focus point, halves are Left/Right). Never copy design files into the repo (design folders
   are not committed); plans cite them by path.
6. The hardware-proven reference code, **read-only**: the owner's camera lab `~/Desktop/Developer/Camera-Connection-Tests`
   — `sdk/src/camera_liveview.cpp`, `sdk/src/main.cpp` (`cmd_liveview` with `--commands`: focus point, half-press
   focus, capture-at-point, clean `quit`), `camlab/pwa/` (the phone page and WebSocket server used in Experiments 08 and
   11), `experiments/07…11-*.md` (the procedures the hardware check reuses). Compare with this repo's
   `native/camera-sidecar/` (which has no live view yet), the fake sidecar `crates/testkit/src/bin/fake_sidecar.rs`,
   the direct camera adapter and its protocol, and the existing frontend and `crates/infra` server dependencies.

## Write
- `docs/implementation/master_plan_live.md`
- `docs/implementation/phases_live.txt` (one milestone id per line, in execution order)
- one plan per milestone: `docs/implementation/plans/LV<n>_<slug>.md` (ids LV1, LV2, …; note in the master plan that
  these milestone ids are distinct from 10J's card ids LV3…LV10)
- the handoff `docs/implementation/handoffs/reviewer/handoff_live_plan_r1.md`

Commit nothing. Change no code. Do not edit the book; if the book has a hole, write a decision card.

## The owner's planning constraints
1. **Few, grouped milestones** (flow §10: a milestone = one plan of ordered units, a `CHECKPOINT (not approved):`
   commit after each unit, **one review at the end**, one fix budget, one narrative). Aim at **about three build
   milestones plus one owner-attended hardware check**, not many small phases. Checks (full suites) happen at
   milestone ends, not per unit, so time and tokens are not spent per unit. Units ≤ 8 criteria each, as §10 says.
   Each plan must still work with `tools/pipeline/run_phase.sh phase <id>` (frontmatter `phase`, `title`,
   `difficulty`, `projection`, `depends_on`, optional `implementer`; read the runner and `tools/pipeline/templates/`
   to be sure the plan shape fits them, as the labels and writer milestone plans do).
2. **Contracts section in the master plan** (contract-grade, code-level, plans implement them verbatim), covering the
   silent-failure mechanisms:
   - **MC-LV1 frames**: `seq` per live-view start, mapping from the camera frame number, dedupe of repeats (the camera
     never says "no new frame"; it returns the last one again), pacing pulls to the frame interval, the zero-size
     first pull, newest-only in the hub, a viewer never receiving a lower `seq` than one it has, slow viewers skip and
     never queue or slow another viewer or the capture path.
   - **MC-LV2 who receives frames**: only viewers showing Live and visible; demand changes; a disappearing viewer
     (phone locked, Wi-Fi drop, socket closed); a switch to Live shows a frame newer than the switch.
   - **MC-LV3 the current shot**: one per templated session; default = first empty container in shooting order after
     reported-but-not-arrived exposures; tap to change; **every exposure bound to the shot current at the camera's
     `exposed_at`** (phone, Mac, or the camera's own button), so a late Wi-Fi file still lands there; advance rule
     (next empty after it in shooting order, else first empty, else none → Condition); **Retake = Deny the photo now**
     (undoable) and make that shot current; presets applied on fill; "placed by template"; never survives a session
     switch or relaunch; untemplated sessions have none. Show exactly how it replaces 10G's shot-index rule, refill
     and retake in the existing placement code, and how it interacts with the capture ledger's FIFO matching.
   - **The sidecar frame channel** (binary, separate from the JSON control line) and the **command / clean-stop
     protocol** (focus point, half-press focus, capture-at-point, focus-frame reports, clean `quit`/disconnect on
     every stop path; never an abrupt cut-off).
   - **Focus on its own thread** (never on the pull loop; capture likewise), the point mapping (0..1 → the camera's
     grid in the adapter, nothing Sony-specific in core), the "focus area not movable → report, never change" rule,
     and × recentring.
   - **Phone pairing token** (per phone, revocable, QR from the Mac, how the server checks it on HTTP and WebSocket),
     and the phone API shaped so session completion / opening a session from the phone can be added later (LV-D18)
     and HTTPS later without changing the phone app (card LV3).
3. **Implementers.** Complex UI and interactions are built and fixed by Fable: put **`implementer: fable` on the
   phone-UI milestone(s)**. Backend, sidecar and hub milestones use the default implementer and **never** carry
   `implementer: fable` (guide: no Fable on camera phases). This owner instruction supersedes flow log v1.3's "no
   Fable for implementation" for this track; record it as a process delta. **Projection off** unless you see a
   concrete reason (say it in the handoff).
4. **Scope.**
   - **The phone app only.** The Mac's live view screen waits (card LV10); the desktop Tauri-channel delivery is not
     built now (keep the hub's outlet interface open for it and for machine-guidance subscribers).
   - **Plain HTTP**, no secure-context API anywhere in the phone app; Android Full screen button; iPhone installable
     full screen (manifest), no service worker or wake lock dependence.
   - **The sidecar gains** live view (pull, pace, dedupe, frame channel), focus point, half-press focus with
     focused/not-focused report and the focus frame, capture-at-point, and clean quit — **ported from the lab code**.
   - **A Rust live-view hub** (service, not in Tauri or the server), live view as an adapter-declared capability
     (`live_view`), core vocabulary only (live frame, viewer, demand).
   - The embedded HTTP + WebSocket server (book 11) serving the phone entry of the same React/TypeScript frontend,
     following the Mac's active session (LV-D18).
5. **Hard rules.** Tauri and the HTTP server stay thin (H8); nothing Sony-specific in core (H7); frames passed through
   untouched, overlays and black-and-white are UI drawing only (H3); **never write exposure (mode or values), network
   or FTP settings** — the only writes are the focus point (and none other is needed on the a7R V; LV-D2/M-LV5); a
   capture keeps 10E's half-press-then-release. **The automated suite never needs the camera, Capture One or a
   phone**: extend the fake sidecar with a frame channel and the new commands; phone behaviour is tested at the
   server/WebSocket boundary and in the frontend's own tests.
6. **The last milestone is the owner-attended hardware check** of M-LV1…M-LV6 using the lab procedures (Experiments
   07, 08, 10, 11 adapted to the packaged app): numbered steps the owner and a session can run, what is recorded
   where (the evidence docs and the Review log), the targets for M-LV6 to be set by the owner at that check. Include
   the items 10J and the lab list as "at hardware acceptance" (Wi-Fi focus, photo sharp where tapped, several phones,
   reconnect after sleep, iPhone reachability).
7. **Guide updates in the same change** (flow §7A): each milestone that adds a service, flow, command, event, rule or
   coupling updates the matching `docs/guide/` file; list which in each plan.

## Plan content (as for the labels and writer tracks)
**Master plan (thin):** goal + pointer to 10J · sources-of-truth table · process deltas (flow §10 as adapted here; no
inventory; Fable on the phone-UI milestone(s); test-run rules per milestone) · naming registry (crates/modules, sidecar
commands and frame-channel format, events, Tauri commands if any, HTTP routes and WebSocket messages, settings keys,
test files under `crates/testkit/tests/suite/` and the frontend test layout) · **contracts section** (above) ·
delegated choices decided once · environment pointers, **verified by running the commands** (how the sidecar is built
and its tests run, the fake sidecar, `cargo test -p psm-testkit --test suite <stem>::`, frontend test and typecheck
commands, how the phone entry is built and served in dev and in the packaged app) · tracker · sequencing.

**Each milestone plan:** frontmatter as above · goal (and what is NOT in it) · read-first list · units in order, each
with files expected to change, **core mechanisms** (the only functions that may have unit tests), ordered tasks, a
**criteria table** (row id, boundary, input, exact outcome, trace to M-LV1…6 / MC-LV1…3 / a master-plan contract) ·
delegated choices · **manual try-out** (steps on this Mac the owner can follow; the narrative is built from it) · guide
files to update · notes · empty Review log.

Every path or symbol you cite must exist or be marked new; every count is derived by script; every row traces to a
ledger entry or contract that supports it; no product semantics restated, cite chapters. Decision cards only for what
the book truly leaves open; write each card in the owner's language (question, story, branches, recommendation, on
silence, trace) and mark the milestone it blocks with `blocked_on:`.

## Handoff
At `docs/implementation/handoffs/reviewer/handoff_live_plan_r1.md` (frontmatter `plan: live`, `role: plan`,
`round: 1`, `state`, `date`, `actor`): files written; counts table (milestones, units, criteria, rows, app files,
projection, implementer); ledger coverage (every M-LV and MC-LV served); the milestone map in plain words, about five
lines per milestone (what it builds, **what the owner can try after it**); estimated size per milestone; anything
probed on this Mac; `⚠ OWNER DECISIONS REQUIRED (n)` with cards verbatim.

Final message, first line exactly:
`HANDOFF: docs/implementation/handoffs/reviewer/handoff_live_plan_r1.md | STATE: <DONE|OWNER_DECISIONS_PENDING|BLOCKED> | OWNER_CARDS: <n>`
