---
plan: LV1_camera_live_view
role: review
round: 3
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-26
actor: opus
---
# Handoff: LV1 camera live view, re-review r3

**Verdict: APPROVED_WITH_NOTES.** Fix 2 (`7bba1e49`) resolves R2-B1 as corrected. While the frame reader waits for live
view to start, a newer helper connection now replaces the parked one. A restart with live view off and a restart with
live view on both deliver the new helper's frames. No blocking item remains. Three notes go to the live backlog. Tracker
→ `APPROVED`; the narrative is written.

## Review history
- **r1** settled everything except R1-B1, R1-B2 and R1-B3: the sidecar, the hub, the protocol, FC4–FC7, H1–H8 and the
  guide.
- **r2** settled R1-B2 and R1-B3, and the "listener outlives helper processes" half of R1-B1. It left R2-B1: a dead
  parked connection survives a restart that happens while live view is off.
- **This round** covers only fix 2's delta: `FrameListener::read_stream`'s wait loop and the rewritten C13(c).

## Gate
- The tracker read `IMPLEMENTED (2026-11-26, codex fix 2; orchestrator)` with the fix note.
- Checkpoint `7bba1e49` ("CHECKPOINT (not approved): LV1 camera live view fix 2") is in `git log`.
- `FIX2_AUTHORISED` is present in the Review log.
- The tree at review was `c4a6d798`, clean.
- The fix's red workspace gate (P50 `c8a`) was re-run by the orchestrator outside the Codex sandbox, and it passed (code
  signing is refused inside the sandbox). That is a sandbox artefact, not a fix defect.

## Perimeter check
- `git show --stat 7bba1e49` lists:
  - `crates/integrations/src/camera/sidecar/frames.rs` (+24): in the FIX2_AUTHORISED list.
  - `crates/testkit/tests/suite/lv1_frame_channel.rs` (+3/−6): in the list.
  - `docs/implementation/plans/LV1_camera_live_view.md`: the tracker line, a Review log entry, and **the C13(c)
    criterion row rewritten**.
  - `docs/implementation/handoffs/implementer/handoff_LV1_camera_live_view_fix_r3.md`: the fix's own handoff.
- `runtime.rs` was allowed and is untouched.
- Later commits (`943c707b`, `c4a6d798`) are orchestrator and runner artifacts: STATUS, state, the prompt, the handoff
  note, and a `run_phase.sh` resume guard.
- **Result: code in perimeter.** The C13(c) row rewrite is covered by the authorisation's "extends C13(c)". But it
  *replaced* the row's frames-flowing case rather than adding to it, so it is recorded as note R3-N1 and process lesson
  R3-N3. It is not blocking, because the behaviour it no longer tests was verified correct this round (see Variation).

## Checks
Each was run once, one cargo command at a time, on clean `c4a6d798`:

| Command | Result |
|---|---|
| `cargo test -p psm-testkit --test suite lv1_frame_channel::` | 24 passed |
| `cargo test -p psm-integrations camera::` | 23 passed |
| `cargo clippy -p psm-integrations -p psm-testkit --all-targets -- -D warnings` | clean |
| `cargo fmt --all -- --check` | clean |
| `npm run typecheck` (apps/desktop) | clean |

- I did not run the full suite or the sidecar build. The fix changed one Rust file and one test, and touched no sidecar,
  shared store or service.
- The fix's stamps (stub sidecar, UI tests, workspace clippy) stand.

## Findings resolved
- **R2-B1: resolved as corrected (the correction's first option).**
  - In `read_stream`'s activation-wait loop, each iteration calls `listener.accept()` (non-blocking) before it checks
    `active`.
  - A pending connection replaces `*stream`. The old stream is dropped, `connection` is incremented, `bad` is cleared and
    `started` is reset. If a source is already active, `hub.source_started` is re-issued with the new source id.
  - **Ordering argument.** The real helper connects its frame socket in `cmd_auto` before it can emit
    `liveview.started`. So by the time `start_source` sets `active`, the new connection is already in the backlog.
  - Accept runs before the `active` check, so the replacement always wins over activation of the dead stream. If
    activation lands inside the 5 ms sleep, the next iteration still accepts first, and the hub gets a second
    `source_started` with the new id. That is harmless: HB1, a new source.
  - The loop is also passed once per published frame. So while frames are flowing, a newer connection (necessarily a
    new process, per FC1) also replaces the current stream. This matches FC1's "a new connection replaces the old". The
    cost is one syscall per frame.
  - **Test.** The new C13(c) is r2's probe (a), which failed 3/3 on the r2 code. It now passes, so it discriminates.
- **Variation (new evidence this round).** I appended two scratch tests to `lv1_frame_channel.rs`, ran them 3×, and
  then reverted the file (`git checkout --`; `shasum -c` OK; `git status` clean). No production file was edited.
  - (1) The pre-fix-2 C13(c) verbatim: a restart while frames are flowing; afterwards 3 frames, seq strictly increasing.
    **3/3 pass.**
  - (2) Two crash restarts with live view off, then live view and frames. Then a crash restart with live view on, then
    live view and frames again: frames from each helper are offered, with seq increasing across sources. **3/3 pass.**
  - A first version of (2) timed out because it did not advance the mock clock past the supervisor's second-crash
    backoff of 1 s. That was a probe artefact; the probe passed once it ticked the clock.

## Triage (flow §5)
No blocking (B1–B4) item remains. Items seen this round:

| Item | Classification | Reason |
|---|---|---|
| R3-N1 C13(c) replaced, not extended: the frames-flowing restart (old R1-B1 case, HB1 seq continuity) lost its row and test | DROP → backlog | More tests. The behaviour was verified correct by variation (1). No user-visible wrong outcome |
| R3-N2 a non-`WouldBlock` accept error in the wait loop returns, dropping a parked live stream without `source_stopped` | DROP → backlog | Speculative robustness. No observed trigger. Joins r2's accept-error note |
| R3-N3 the fix session rewrote a plan criterion row | DROP → backlog (process lesson) | Process, not behaviour |

## Remaining findings
None blocking. Notes:
- **R3-N1 (note, route: plan/verification).** Restore the frames-flowing restart as a sibling row, C13(e): "after (a):
  the fake exits 1; the restarted fake sends `liveview.started` epoch 1 and `frames 1 1 2 20` → both offered, seq above
  every earlier seq". Do it in the next phase that touches `frames.rs` or the fake (LV2). Variation (1) above is that
  test, ready to paste.
- **R3-N2 (note, route: production).** Treat `ConnectionAborted`/`Interrupted` from the in-loop `accept` like
  `WouldBlock`. This goes to the next `frames.rs` touch or the LV4 hardware check.
- **R3-N3 (note, process).** Fix prompts should say "add row X" when a new case is wanted, and fix sessions should add a
  sibling row rather than edit an existing one.
- r2's note on the dead `previous_connections`/`connection_count()` is still open. It was not in fix 2's findings.

## Lessons for the plans
- Correction text that says "extend row X with case Y" is read as "replace". Write "add row X(e) for case Y; keep X".
- When a fix touches a file that has open backlog notes, the fix prompt could list those notes as optional, in-perimeter
  cleanups. Fix 2 touched `frames.rs` but left the dead rebind argument.

## Carry-forward
| Note | Destination |
|---|---|
| R3-N1 restore the frames-flowing restart row | LV2 (orchestrator adds C13(e) or an LV2 row) |
| R3-N2 accept errors in the wait loop | next `frames.rs` touch / LV4 |
| R3-N3 fix sessions add rows, do not rewrite them | orchestrator (fix prompt template) |
| r1 and r2 notes | unchanged destinations, in `backlog_live/LV1_camera_live_view.md` |

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Write perimeter (this session)
- `docs/implementation/plans/LV1_camera_live_view.md`: the Tracker line (→ APPROVED) and one Review log entry.
- `docs/implementation/handoffs/reviewer/handoff_LV1_camera_live_view_review_r3.md` (this file).
- `docs/implementation/narratives/LV1_camera_live_view.md` (new).
- `docs/implementation/backlog_live/LV1_camera_live_view.md`: three appended notes.
- `docs/implementation/backlog.md`: one appended pointer line.
- **Probe declaration.**
  - I temporarily appended two test functions to `crates/testkit/tests/suite/lv1_frame_channel.rs`, edited one of them
    once (clock ticks), and ran them 7 times in total.
  - I restored the file with `git checkout --`; `shasum -c /tmp/lv1r3_probe.sha` reports OK.
  - No production file was edited.
- **Side effects.**
  - Test runs may leave `psm-lv-*.sock` files in `$TMPDIR` (r1 note).
  - Build outputs were refreshed in `target/`.
- No commits; the runner makes the approval commit.
