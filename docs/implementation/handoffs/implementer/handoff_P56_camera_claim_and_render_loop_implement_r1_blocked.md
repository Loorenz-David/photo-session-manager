---
plan: P56_camera_claim_and_render_loop
role: implement
round: 1
state: BLOCKED
date: 2026-11-08
actor: codex
---

# P56 implement r1 — blocked at the contract gate

No production code was changed. Two requirements in the phase contradict the ratified camera contract and each other at the connection boundary. The render-loop investigation below is complete enough to guide an amended plan, but no unverified partial fix was installed.

## ⚠ OWNER DECISIONS REQUIRED (2)

### Card 1 — Claiming USB while Wi-Fi is connected

**Question** — May Session Manager briefly release a working Wi-Fi camera session to attempt USB within 300 ms of plug-in?

**Story** — The camera is connected over Wi-Fi when the cable is plugged in. The Sony SDK allows only one session, so a USB connect attempt needs the Wi-Fi session closed first. The current plan demands that immediate attempt while also saying the Wi-Fi session must not be dropped before USB is ready.

**Branches** — Yes: permit a brief Wi-Fi interruption for the fast USB race, then reconnect Wi-Fi promptly if USB fails. No: keep Wi-Fi uninterrupted and change the 300 ms requirement to a discovery or availability check that does not connect.

**Recommendation** — Yes, because the owner's priority is claiming the cable before the system service; require immediate Wi-Fi recovery on a failed claim.

**On silence** — The gate holds; no fast-claim implementation is approved.

**Trace** — P56 C1(a), Notes “Hazard”; 10E MC-31 §Transport policy and R-I18; M39.

### Card 2 — Trying USB again after three failures

**Question** — Should the three-failure USB rule change to periodic, bounded USB retries while the cable remains plugged in?

**Story** — Another program holds the cable, so Session Manager falls back to Wi-Fi after three failed attempts. If that program later releases the cable without a re-plug, the ratified rule never tries USB again. P56 instead expects attempts throughout 30 minutes and an automatic switch when one succeeds.

**Branches** — Yes: amend the contract to specify a retry interval, what happens to a working Wi-Fi session during each probe, and how a holder release is detected. No: keep the current rule and change P56 C2(a) to require a re-plug or holder-gone signal before USB is retried.

**Recommendation** — Yes, because it fulfills the owner's request that a reachable camera is actively held or being recovered without a manual re-plug.

**On silence** — The gate holds; no long-run retry policy is approved.

**Trace** — P56 C2(a), C2(b); 10E MC-31 §Transport policy; M39.

## What was built

Nothing was installed. The phase stopped at the plan/intention contradiction gate. I traced the helper state machine and the render service, then inspected the existing app log and database read-only to narrow F12.

The render-loop evidence is concrete: one RAW has `adjusted_gen=173`, `rendered_gen=169`, 170 completed preview jobs and 201 `processing_done` events, but zero `variant_adjusted` hook events. The log shows repeated `recipe_exists → find_image → process` sequences. In code, `PreviewService::check_sidecars` can raise the generation from a `.cos` mtime on focus or startup, and it compares against the preview file's mtime and `last_own_write_at` only. Capture One's own processing appears to update the sidecar; a later focus can therefore classify the export as an adjustment and request another export. This is the leading cause, not a proved fix. The database also has 274 `render_preview` jobs left `running`, so completion and duplicate-request accounting need a boundary test before changing them. No owner files were written, moved or deleted.

## Tests → criterion rows

No new tests were written or run because the phase cannot be implemented under the present contract. C1(a), C1(b), C2(a), C2(b), C3(a), C3(b), C3(c), and C3(d) remain unproved. Existing C++ tests explicitly assert the three-failure fallback and no periodic retry (`native/camera-sidecar/tests/test_transport_manager.cpp`), confirming the C2 conflict.

## Assumptions

- The plan's “first USB connect attempt” means an actual SDK connect, not discovery. That is the boundary named in C1(a).
- A successful USB connection requires releasing the existing Wi-Fi SDK session, as the single-session transport implementation and MC-31 describe.
- The read-only app database is diagnostic evidence only; its current state is not a test fixture or a substitute for a reproducible boundary test.

## Deviations

- Stopped before production and test edits because P56 C1(a)/Notes and C2(a)/MC-31 cannot be implemented simultaneously as written.
- The requested closing suite, lint, typecheck and sidecar stamp were not run; there is no implementation tree to validate. P56 remains `NOT_STARTED`, not `IMPLEMENTED`.
- The checkpoint SHA cannot be embedded literally in a file contained in that same commit. The self-identifying command below resolves it after commit.

## Files changed

- `docs/implementation/plans/P56_camera_claim_and_render_loop.md` — Review log finding.
- `docs/implementation/handoffs/implementer/handoff_P56_camera_claim_and_render_loop_implement_r1.md` — this handoff.

## Checkpoint SHA

The commit containing this handoff is the checkpoint: `git log -1 --format=%H -- docs/implementation/handoffs/implementer/handoff_P56_camera_claim_and_render_loop_implement_r1.md`. Its full SHA is reported in the final session message.

## Stamp

Command: none. Result: blocked at the semantic contract gate before implementation; no suite, lint, typecheck or sidecar build was run. `git status --porcelain` was clean before these two documentation changes.

## Manual try-out

None yet. The present build retains the known camera claim and Capture One export-loop failures. Do not leave it running for an hour on a nearly full disk; the P52 run 2 try-out follows an amended and implemented P56.
