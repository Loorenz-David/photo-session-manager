---
plan: P54_camera_cable_and_take_back
role: review
round: 2
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-08
actor: opus
---
# P54 re-review r2: APPROVED_WITH_NOTES

Both r1 blocking findings are resolved as their corrections said. When Capture One quits and is about to be relaunched while the camera is
on the cable and our helper is not on USB, the relaunch wait now sends the helper one `usb-retry`. That is the same fresh chance take-back
step 4 gives, so a helper that had fallen back to Wi-Fi tries the freed cable inside the 15 s wait (F1). The C2 tests now drive the cable
and ready state through fake-helper events and assert the retry on the helper's stdin before any `launch` op (F2). Fix perimeter clean;
suite, lint, format, typecheck and helper tests green. No blocking item remains, so there is nothing to triage. Two new notes go to the
backlog.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Review history
r1 (`e8a95cd`) settled C1(a), C1(b), C3(a), C4(a), C4(b), C5(a), the adapter → start-order wiring, the assumptions vs 10E and H1–H8. It
blocked on F1 (B1: the relaunch wait never sends `usb-retry`) and F2 (B2: C2 tests bypass the helper boundary). This round covers only the
fix delta (`7e31f09`) and its seam: `StartOrder::wait_for_relaunch`, the runtime's retry registration, the C2 tests.

## Perimeter check
- **Gate:** tracker row P54 `IMPLEMENTED` with a fix-1 note; fix checkpoint `7e31f09` exists. Reviewer tree: HEAD `01a0dba` (the runner's
  re-review prompt commit on top), `git status --porcelain` clean.
- **`git show --stat 7e31f09`:** 6 paths: `crates/integrations/src/camera/direct/start_order.rs`, `crates/integrations/src/camera/runtime.rs`,
  `crates/testkit/tests/p54_camera_cable_and_take_back.rs`, the plan file (Review log line only), `master_plan.md` (P54 row only), the fix
  handoff. All are inside the r1 findings' allowed files (F1: `start_order.rs`, `gate.rs`, `runtime.rs`, the P54 test, sidecar scripts,
  Review log, tracker; F2: the P54 test, sidecar scripts). Matches the fix handoff's *Files changed* exactly.
- **Foreign commits between the checkpoints** (`887761d`, `48a40b7`, `63a8487`, `fbe3a84`): docs, STATUS, P56 plan, flow v1.3 and a
  one-line `run_phase.sh` model change. No product code. Not part of this fix and not findings.

## Evidence (reviewer run, tree `01a0dba` = `7e31f09` + prompt commit, clean)
| Command | Result |
|---|---|
| `cargo test --workspace --no-fail-fast` | 834 passed / 0 failed, exit 0 (P54 file 6/6 incl. `c2a`, `c2b`; desktop `p54_tests` 1/1) |
| `cargo clippy --workspace --all-targets -- -D warnings` | green |
| `cargo fmt --all -- --check` | green |
| `npm run typecheck` | green |
| `npm test` | 54 files, 407/407 |
| `tools/sidecar/build.sh --stub --test` | 22 tests, 0 failed |

The suite was run once as the review-entry gate (the tree differs from the fix stamp by the prompt commit). The first `cargo test` run hit
the tool's 10-minute background limit with no result and was restarted. Load average was 11–15, and many desktop-crate tests ran for over
60 s. That is environmental: nothing failed. The helper build was run twice because the first output tail cut off the count. The helper
was not touched by the fix.

## Findings resolved
### F1 (B1): resolved as corrected
- `StartOrder::wait_for_relaunch` now calls the registered `relaunch_retry` once, before its polling loop, when `needs_relaunch_wait()`
  (`usb_present == Some(true) && !usb_ready`). Both later-launch paths go through it: `GatedScripting::launch` (non-permitted, start order
  satisfied) and `launching` (non-permitted, needs wait, editor not running).
- `CameraRuntime::with_start_order` registers a closure holding a `Weak<CameraRuntime>`, so there is no reference cycle. It sends
  `Command::UsbRetry` through `send_command`, and a send error (helper not running) is logged without breaking the bounded wait.
- **Helper side, by reading:** `usb-retry` is accepted in any state (`main.cpp:538`). `drain_inputs` resets `usb_failures_`,
  `usb_retry_at_ = 0`, `failures_` and `next_attempt_ms_`. While `CONNECTED_WIFI`, `desired_transport()` then returns USB and the existing
  switch path runs. It waits for `min_dwell_ms` (3 s) and `usb_settle_ms` (counted from the last USB presence change, so long past) and is
  deferred while a transfer/exposure is busy. Then it disconnects Wi-Fi intentionally and tries USB after 1 s. That fits inside the 15 s
  wait. If the cable is still held, three failures return the helper to Wi-Fi (C3), and the wait ends at 15 s as before. Nothing is quit
  or killed.
- **No double retry in take-back:** take-back step 5 relaunches under `gate.permit`, so `permitted_here()` skips the relaunch wait. Only
  step 4's own `usb-retry` is sent.
- **Locking:** the retry runs with the `Inner` lock released. `send_command` takes the supervisor lock briefly, and no supervisor holder
  (`ensure_started`, `restart_if_not_usb`, tick poll, shutdown) calls into Capture One launch. No deadlock path. The `relaunch_retry`
  mutex is held across the call and nothing re-enters it.
- **App-start order unchanged:** `launch` still uses `wait()` while `Waiting`. The retry only exists on the later-relaunch path.

### F2 (B2): resolved as corrected
- **C2(a):** `usb.initial {present:true}`, USB `session.closed`, `usb.claimed_by` are injected as helper events; launch runs on a thread;
  the test waits for `usb-retry` on the fake helper's stdin and asserts exactly one; asserts no `launch` op; injects
  `connection.ready transport=usb`; the launch completes and the `launch` op appears. Bound case: a second USB `session.closed` → second
  `usb-retry` → no launch → `clock.advance(15_000)` → launch.
- **C2(b):** for each link, `usb.initial {present: link == Cable}` is injected after the ready, launch is immediate, and no `usb-retry`
  is sent.
- **Can it fail on a wrong outcome?** Missing `usb.initial` → present wiring: the wait-start `settle` panics. Missing retry: the stdin
  `settle` panics. Missing `connection.ready transport=usb` → ready wiring: the `settle` after it panics. A launch that does not wait:
  no retry is sent, or the `launch` op is already present. C2(b)'s USB case now carries `present:true`, so a ready flag that is not wired
  would make the launch wait (fail by hang/timeout). Over-engineering test: assertions are on helper stdin and editor ops, the boundaries
  the rows name. See note N2 for the one remaining internal sync point.

## Triage
| Item | Class | Reason |
|---|---|---|
| — | — | No blocking item remains after fix 1. F1 and F2 are resolved; nothing new reaches B1–B4. |

## Remaining findings
None blocking. Notes (appended to `docs/implementation/backlog.md` under "P54 reviewer r2"):
- **N1, relaunch retry drops Wi-Fi when someone else holds the cable → P52 run 2:** the one `usb-retry` per later relaunch also fires
  when the cable is held by `ptpcamerad`/Photos rather than by the Capture One that just quit. The helper then leaves a working Wi-Fi
  session, fails USB three times and falls back again. That is a Wi-Fi gap per relaunch. Its length depends on how long a connect against
  a held interface takes (the r1 F6 lead: possibly up to the 20 s connect timeout per attempt). A busy transfer defers the switch, so no
  shot is cut mid-transfer, but a shot taken during the gap waits for Wi-Fi to come back.
- **N2, C2 sync point:** `c2a` synchronizes on `StartOrder::needs_relaunch_wait()` between injected events and the launch thread. The
  assertions are at the boundary, but renaming that method would break the test with no behaviour change. Waiting on `sidecar_events()`
  (as `c2b` does) would remove the coupling.

The seven r1 notes stand as written.

## Carry-forward dispositions
| Note | Destination |
|---|---|
| r1: F7 diagnosis / orphaned `pending` | preview phase (F11) / P52 run 2 |
| r1: F6 cause unproven | P52 run 2 |
| r1: other holders need a re-plug | later camera phase (P56 claim-at-plug-in is the natural home) |
| r1: Connecting while the camera is off | owner / book (10E status rule) |
| r1: Retry edge cases | later camera phase |
| r1: H8 drift in `camera_retry` | next phase touching camera commands |
| r1: extra `editor_info` call | preview speed (F11) |
| r2 N1: Wi-Fi gap per relaunch while held elsewhere | P52 run 2 (watch A6 + a Capture One quit) |
| r2 N2: C2 sync point | next phase touching the P54 test file |

## Lessons for the plans
- When a row depends on another component acting (here: the helper retrying USB once it has fallen back), name the trigger in the row.
  r1 already recorded this. The fix confirms it: the missing trigger was a one-line send.

## Probe declaration
No production file was edited. No mutation probes (flow §4). Files written by this review: this handoff, `docs/implementation/backlog.md`
(appended), the plan's Review log entry, the P54 tracker row in `docs/implementation/master_plan.md`, the narrative
`docs/implementation/narratives/P54_camera_cable_and_take_back.md`. The build outputs from `cargo test`/`clippy` and
`tools/sidecar/build.sh --stub --test` (`target/`, the stub helper binary) are untracked/ignored build artifacts. One scratch log was
written outside the repo: `/tmp/p54r2_cargo_test.log`. Nothing committed.
