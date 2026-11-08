---
plan: P54_camera_cable_and_take_back
role: review
round: 1
verdict: CHANGES_REQUESTED
state: DONE
date: 2026-11-08
actor: opus
---
# P54 review r1: CHANGES_REQUESTED (2 blocking: 1 × B1, 1 × B2)

The take-back guard, the read-only availability query, the Connecting fixes and the helper's bounded USB retry are correct and
tested at their boundaries. What blocks is the relaunch wait. It never asks the helper to try USB, and after the helper has fallen back
to Wi-Fi (the state P54 itself makes reliable) it does not try. So the wait only delays Capture One by 15 s, and then Capture One
takes the cable again (F1). Also, the C2 tests set the start-order state directly instead of going through the helper's events (F2).

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Verified correct
- **Gate:** tracker `IMPLEMENTED`; checkpoint `e8a95cd` (resolved as the handoff says) exists. Reviewer tree: HEAD `72b460e` (prompt
  commit only on top of `e8a95cd`), `git status --porcelain` clean before the review's own writes.
- **Perimeter:** `git show --stat e8a95cd` = 24 paths, exactly the handoff's *Files changed* (the brace groups expand to the same
  set). The Deviations disclose the HealthPage/P51 test edits and the P45 `0 failed` change, which the plan allows.
- **Stamp (reviewer run, code tree `e8a95cd`):** `cargo test --workspace` 834 passed / 0 failed (P54 6/6, desktop `p54_tests` 1/1);
  `cargo clippy --workspace --all-targets -- -D warnings` green; `cargo fmt --all -- --check` green; `npm run typecheck` green;
  `npm test` 407/407 (54 files); `tools/sidecar/build.sh --stub --test` 22 tests, 0 failed. No B4. (One run each: the reviewer's tree
  differs from the implementer's stamp by the runner's prompt commit only. The run is the review-entry gate, not a repeat.)
- **C1(b):** `renders_in_flight` = `max(jobs, busy)`. `busy` holds a base name from just before `render_started` until the job ends,
  so a render Capture One is processing, or is being started, still blocks. `c1b` issues a real render through the adapter and asserts
  the exact reason `a preview is rendering`. The pre-existing P48 `c4e` still asserts count 1.
- **C1(a):** the excluded `pending` entries always sit behind a `busy` mark for the same key, except orphaned ones (backlog). So
  excluding them never lets take-back quit Capture One under an issued render, which the plan's hazard requires. `c1a` runs the full
  five-step take-back with a deferred preview and asserts that the preview is in flight once the variant appears. Caveat (backlog,
  not blocking): a `variant_not_found` deferral never enters the adapter registry, so `c1a` also passes on the pre-P54 count, and
  the real F7 trigger is unproven.
- **C3(a):** after `usb_failures_before_wifi_fallback` failures (connect failures or empty enumeration), `usb_retry_at_ = INT64_MAX`
  and Wi-Fi is attempted at once. The per-attempt USB delay is capped at 2 s, so even a 10 s/30 s backoff reaches `CONNECTED_WIFI`
  within the plan's 10 s. The C++ case asserts exactly three USB connects, one `usb.fallback_to_wifi` and `ConnectedWifi` for all
  three rigs, and a busy-deferred then safe switch back after a re-plug. It is a Core mechanism, so the unit test is allowed.
- **C4(a):** `DISCONNECTED`/`RECOVERING` no longer clear `connecting`, and `ERROR` still does. `c4a` injects the cold-start sequence
  with the real helper's reasons, ending in a retryable `DISCONNECTED`, and waits for `Connecting{Wireless}`, then `Connected`. It
  fails on the old mapping, where the batch ends `Disconnected`. The UI test asserts the exact label `Connecting · Wi-Fi`.
- **C4(b):** `camera_retry` sends `usb-retry` (resets the helper's failures and `next_attempt_ms_ = now`, so it tries again at
  once) and returns `connecting` with the link (cable if USB is present, else Wi-Fi). The desktop IPC test asserts the exact JSON
  `{"kind":"connecting","link":"wireless"}` and finds `usb-retry` in the fake helper's stdin. The UI test asserts that the dot
  changes on Retry.
- **C5(a):** `blocked_reason` = `ALREADY_RUNNING` or the first failing precondition, read from store counts, runtime counters and
  adapter state, with no scripting call (`c5a` asserts the fake's op list is unchanged). The UI polls every 1 s while the action is
  shown, starts disabled (`Checking…`), and shows the reason. `p54_camera_cable` asserts disabled + `an export is running`, that a
  click does nothing, then enabled.
- **Adapter → start-order wiring (by reading):** `usb.initial`/`usb.arrived`/`usb.removed` → `usb_present`;
  `connection.ready transport=usb` → ready; `usb.claimed_by`, USB `session.closed`/`session.lost`, helper exit → not ready. The real
  helper emits `usb.initial` from IOKit at start (`main.cpp:514`). The one-shot app-start order is unchanged (`launch` uses `wait`
  while `Waiting`).
- **Assumptions vs book:** none contradicts 10E. (1) A retryable `DISCONNECTED` keeps Connecting: a judgment call on 10E's status
  rule, reported to the owner and in the backlog. (2) Empty enumeration counts toward fallback: consistent with MC-31 §Transport
  policy. (3) The 15 s wait applies to each later launch: within the plan's Delegated choices. (4) 1 s query refresh: fine.
  (5) Deferred previews belong to the preview service's retry state: true by reading (see the C1(a) caveat).
- **H1–H8:** H1–H7 hold (core changes only a doc comment; no RAW, pixel, UI-automation or platform-concept change; Capture One
  only through its dictionary). H8: `camera_retry` now dispatches by capability and emits the event in the command. That is drift,
  noted in the backlog, not a violation of H8's "thin interface" severe enough to block. Owner camera rules hold: nothing is killed or
  quit automatically, and no SDK, credential or fingerprint is committed.

## Findings

### F1: blocking (B1), route: production. The relaunch wait never gives the helper a chance at the freed cable
- **What is wrong.** `GatedScripting::launch`/`launching` → `StartOrder::wait_for_relaunch` waits up to 15 s for
  `connection.ready transport=usb`, but nothing tells the helper to try USB. In exactly the case the wait targets (cable present, our
  helper not on USB because Capture One/`ptpcamerad` held the interface), the helper has already had its three failed USB attempts
  and set `usb_retry_at_ = INT64_MAX` ("USB gets a fresh chance on the next physical re-plug", `transport_manager.cpp:186–194`).
  Its only resets are a re-plug or a `usb-retry` command (`drain_inputs`), and no Rust path sends `usb-retry` except take-back step 4
  and Retry. So after the photographer quits Capture One, the helper stays on Wi-Fi, the wait runs out at 15 s, and Capture One is
  relaunched and takes the cable again, as in P52's F10. P54's working fallback (C3) makes this the normal state, where in P52 run 1
  the helper was still retrying USB.
- **Authority.** Plan Goal 2 (F10: "the app waits for our USB session … before relaunching Capture One"), Goal 3 ("a re-plug or the
  holder quitting switches back to USB as today"), plan *Manual try-out* ("quitting Capture One on the cable lets our helper take it
  first"); 10E MC-34 "Get there first and never let go".
- **Correction.** When the relaunch wait starts (cable present, our helper not on USB, Capture One not running), send the helper one
  `usb-retry`, the same fresh chance take-back step 4 gives, so it attempts the now-free cable within the wait. This quits or kills
  nothing. Prove it in the C2(a) test (see F2) by asserting that the fake helper received `usb-retry` before any launch.
- **Files the fix may touch.** `crates/integrations/src/camera/direct/start_order.rs`, `crates/integrations/src/editing/capture_one/gate.rs`,
  `crates/integrations/src/camera/runtime.rs` (if the command is sent from there), `crates/testkit/tests/p54_camera_cable_and_take_back.rs`,
  `crates/testkit/src/fixtures/sidecar_scripts/` (a new script if needed), the plan's Review log, tracker row.

### F2: blocking (B2), route: production. The C2 tests skip the helper boundary the rows name
- **What is wrong.** `c2a` drives the wait with `a.app.start_order().camera_state(Some(true), Some(false))` and
  `camera_state(None, Some(true))`, not with helper events. If the adapter → start-order wiring were missing (no `usb_present` from
  `usb.initial`/`usb.arrived`, no ready from `connection.ready transport=usb`), the product would relaunch at once and both C2 tests
  would stay green. `c2b`'s "helper already on USB" case uses `ready_usb.txt`, which has no `usb.initial`, so `usb_present` is `None`
  and the case passes whatever the ready flag says. By the over-engineering test (flow §4), renaming `StartOrder::camera_state`
  would break `c2a` with no behaviour change.
- **Authority.** Plan C2(a) ("camera on the cable, helper not on USB … until our helper reports `connection.ready transport=usb`"), C2(b)
  ("or helper already on USB"); flow §4 rule 1, §5 B2.
- **Correction.** Drive C2 through the fake helper. C2(a): `usb.initial {present:true}` plus a claim or failed USB connect, Capture
  One stopped; assert no `launch` op and that the helper got `usb-retry` (F1); inject `connection.ready transport=usb` and assert the
  launch; keep the 15 s bound case. C2(b): run the USB case with `usb.initial {present:true}` before the ready, and the no-cable case
  with `present:false`.
- **Files the fix may touch.** `crates/testkit/tests/p54_camera_cable_and_take_back.rs`, `crates/testkit/src/fixtures/sidecar_scripts/`.

## Notes written to backlog
Seven lines under "P54 reviewer r1" in `docs/implementation/backlog.md`: F7 diagnosis vs code (likely orphaned `pending` entry,
which also leaves that photo's preview stuck forever); F6 cause unproven (the 2 s cap only matters above default backoff); holders
other than Capture One going away still need a re-plug; Connecting shown indefinitely while the camera is off; Retry edge cases
(helper not running → error; terminal `ERROR`); H8 drift in `camera_retry`; extra `editor_info` call per launching script while the
cable is held elsewhere.

## Lessons for the plans
- P54's F7 diagnosis ("`pending` keeps renders deferred on `variant_not_found`") does not match the sequential code path. A hardware
  finding whose cause is inferred, not logged, should carry "cause unconfirmed", and its row should name the observable symptom
  (take-back refused while no render is issued), not the presumed internal state.
- C2 and C3 interact: the plan made Wi-Fi fallback reliable (C3) and a relaunch wait for USB (C2) in the same phase, without saying
  who makes the helper retry USB once it has fallen back. Rows that depend on another component acting should name the trigger.

## Probe declaration
No production file was edited. No mutation probes (flow §4). Files written by this review: this handoff,
`docs/implementation/backlog.md` (appended), the plan's Review log entry, the tracker row in `docs/implementation/master_plan.md`.
Build outputs from `cargo test`/`tools/sidecar/build.sh --stub --test` are untracked build artifacts (as in every review).
