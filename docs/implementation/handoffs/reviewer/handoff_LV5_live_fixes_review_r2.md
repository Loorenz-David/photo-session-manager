---
plan: LV5_live_fixes
role: review
round: 2
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-26
actor: opus
---
# Handoff: LV5 live fixes, re-review r2

**Verdict: APPROVED_WITH_NOTES.**
- The one blocking finding from round 1, R1-B1, is resolved exactly as its correction asked. The two fast-Wi-Fi
  sequence tests now assert the current shot after every shutter and after the phone tap. `Free` after s5 tells the
  fixed hold query from the old one.
- The fix touched the one allowed file and no production code.
- Every check is green. No blocking item remains, so there is nothing to triage as MUST.
- The notes stay in `docs/implementation/backlog_live/LV5_live_fixes.md`: round 1's five, the orchestrator's two, and
  one addendum from this round.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Review history
- **Round 1 settled** U1–U4 production code, the perimeter, H1–H8, the guide, and C1, C2, C4–C12. Its one blocking
  item was R1-B1: C3(a)/(b) could not fail on the F4 defect.
- **This round** checks the delta and the C3 seam only.

## Gate
- The tracker read `IMPLEMENTED (2026-11-26, codex) — fix 1: R1-B1 resolved …`.
- The fix checkpoint is `fd620c4a`. The orchestrator made the commit with the fix session's own command, because the
  Codex fix sandbox refused the worktree's `index.lock` (handoff orchestrator note; backlog entry).
- Tree at review: `f4468621`. `git status --porcelain` was empty before and after.

## Perimeter check
- `git show --stat fd620c4a`: `crates/testkit/tests/suite/lv5_current_shot.rs` | 11 +, and nothing else. That is the
  only file R1-B1 allowed.
- `4019ee71` (runner): the fix handoff and `plans/LV5_live_fixes.md`, the tracker line and one Review log line. That
  is the fix role's own bookkeeping.
- `e25b5f4e` (orchestrator): its note on the fix handoff, and two backlog lines.
- `f4468621` (runner): this re-review prompt.
- No production file changed, and no file outside the perimeter changed.

## Findings resolved
**R1-B1 (B2) is resolved as corrected.** The diff adds `assert_eq!(current(&a, s), …)` after each injected shutter.
- C3(a) asserts X, Y, Z, Free, Free after s1–s5.
- C3(b) asserts X, then `Shot(Z)` after the tap, then X, Y, Free, Free.

These are exactly the sequences the correction named.

Why the assertions can fail on the defect (checked by reading; flow.md §4 forbids production probes):
- **Old query.** `hold_times`'s old `NOT EXISTS(delivery)` ended a hold when the exposure's file was filed. In C3(a),
  s1's file is filed at 4.0 s, before s5 at 4.8 s, so W stops being taken. In C3(b), s2's file is filed at 4.0 s and
  s1's at 4.4 s, so Z and W stop being taken. Either way, `advance_after_binding` after s5's Free binding holds
  `Shot(W)`, and `resolve` keeps W because it is not taken. `current == Free` fails.
- **Current query** (`store.rs:916`). The hold ends only when a non-duplicate delivery's `final_path` is a photo of the
  session. Neither file has been taken in at 4.8 s, so the result is `Free`.
- **No race.** `*held = Some((s, next))` is written inside the exposure-insert transaction
  (`current_shot.rs` around l.375). `inject_exposure` returns only after the committed row is visible, so every
  `current` read sees the settled advance.
- **Proof of the row.** The log clause of C3(a)/(b) ("every `current_shot.set … by=advance` names an unbound container
  or `none`") is now proven by state at every advance, which round 1's correction accepted as the row's proof.

Checks, each once on `f4468621`, one cargo command at a time:
- `cargo test -p psm-testkit --test suite -- lv5_current_shot:: lv2_current_shot::`: 40 passed, 0 failed (5.2 s).
  This covers the named file and the LV2 current-shot stem, which shares its fixture helpers.
- `cargo clippy -p psm-testkit --all-targets -- -D warnings`: clean.
- `cargo fmt --all -- --check`: clean.
- `cd apps/desktop && npm run typecheck`: clean.
- The fix's full-suite stamp stands. The fix changed one test module, so no other test can change.

**Variation, not a re-run** (reason recorded before running): the fix stamp saw
`lv2_phone_ws::lv2_c28b_a_socket_that_stops_answering_is_closed_4408` fail once under full-suite load, then pass
alone. U3 rewrote that socket's writer. I ran `lv2_phone_ws:: lv5_phone_socket::` once at module scope: 14 passed.
- Idle and stall both close with 4408 (`ws.rs:504–505`). The close frame is sent best effort under a 200 ms timeout
  (`ws.rs:510`), so under heavy load the client can see the socket drop without a code.
- This is not a user-visible wrong outcome, because the phone reconnects either way (plan P2). It is not B4 either:
  the stamp's failing case is green on rerun, and the orchestrator already logged it.
- I added a reviewer addendum to the existing backlog entry.

## Triage (flow.md §5)
| Item | Class | Why |
|---|---|---|
| (none left blocking) | — | R1-B1 resolved; nothing new reaches B1–B4 |
| `lv2_c28b` one-off miss in the fix's full gate (orchestrator note, reviewer addendum) | DROP → backlog (already there) | timing flake at 500/600 ms test timings under load; outcome a user sees is unchanged (the phone reconnects); green at module and targeted scope |
| `p33_split_grid` C4(d) one-off miss (orchestrator note) | DROP → backlog (already there) | UI timing in a file LV5 did not touch; green on file rerun |

No MUST, so the verdict is `APPROVED_WITH_NOTES`.

## Remaining findings
None blocking. The notes, all in `docs/implementation/backlog_live/LV5_live_fixes.md`, each with its destination:
- **Round 1's five:**
  1. C3 clock offsets differ from the row.
  2. `rev > 0`, not "above every earlier rev".
  3. A Settings error toast on every failed 5 s re-read.
  4. The handoff's unit attribution for `lv2_phone_ws.rs`.
  5. The guide 04 coupling as prose, not a table row (→ LV6).
- **The orchestrator's two:** the two flaky tests, and the Codex fix sandbox's git lock.
- **This round's addendum:** the `lv2_c28b` cause analysis and a suggested test change at its next touch.

## Lessons for the plans
- A fix prompt that names one file should also say who commits when the fix sandbox cannot. This round's checkpoint
  had to be made by the orchestrator, and the handoff recorded "No checkpoint SHA". The runner already flags this. The
  backlog entry on the Luna sandbox is the fix.
- Socket tests with sub-second close timings (`lv2_c28b`: 500/600 ms) are load-sensitive once the close frame is best
  effort. Rows that assert a close code should allow for "the viewer is gone and the socket dropped" or use timings
  well above the 200 ms close budget.

## Probe declaration
No probes: no production or test file was edited, even temporarily. The cargo and npm runs wrote only build output
under `target/` and the test temp dirs. `git status` was clean before the review started.

## Files written by this review
- This handoff.
- `docs/implementation/plans/LV5_live_fixes.md`: the tracker line (→ APPROVED) and one Review log entry.
- `docs/implementation/backlog_live/LV5_live_fixes.md`: one addendum bullet.
- `docs/implementation/narratives/LV5_live_fixes.md` (new).
- No commits. The runner commits these files.
