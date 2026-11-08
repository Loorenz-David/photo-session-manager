---
plan: P56_camera_claim_and_render_loop
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-08
actor: opus (pipeline-reviewer, Opus 5.5)
---

# P56 review r1: APPROVED_WITH_NOTES

Gate: the tracker showed `IMPLEMENTED`, and checkpoint `ab6ee3d9fe7a14102286fb044718586cb7d28bfd` exists (it is the commit holding the implementer handoff). All 8 criterion rows are true at their boundary, and their tests fail on a wrong outcome. The suite, lint, typecheck and helper tests are green. H1–H8 hold. No assumption contradicts the intention or orchestrator rulings 1–2. 7 notes went to the backlog. No blocking findings.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Verified correct

**Perimeter.** `git show --stat ab6ee3d` lists 24 paths, the same as the handoff's Files changed: 15 production, 6 test/harness, 3 docs. The paths outside the plan's estimate are all declared in the handoff's Deviations, and each serves a row:
- `crates/infra/src/disk.rs` and the `lib.rs` module line;
- migration 008, plus `infra/sqlite/store.rs` and `core/ports/store.rs`;
- `core/ports/editing.rs`;
- `integrations/src/lib.rs`;
- `testkit/src/test_app.rs`;
- `HealthPage.tsx`.

The changed old assertions are the five C++ cases the plan's Notes allow, plus two schema-only test edits (`p01`, `p27`).

**Gate (B4).** Run on the clean tree `319afef`, whose code is identical to `ab6ee3d` (only a prompt file was added):

| Command | Result |
|---|---|
| `cargo test --workspace` | exit 0, 841 passed / 0 failed |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo fmt --all -- --check` | exit 0 |
| `npm run typecheck` | exit 0 |
| `npm test` | 411/411 |
| `tools/sidecar/build.sh --stub --test` | exit 0 |

This is the reviewer's one stamp. The tree differed from the implementer's stamp by the committed prompt only.

**Rows**

C1(a): connect at plug-in.
- Production: `drain_inputs` UsbPresence(true) calls `arm_claim`. In `CONNECTED_WIFI` with no busy reason, the next `step` sets `claim_until_ = now + usb_settle_ms`, calls `intentional_disconnect`, then `try_connect(Usb)` in the same step.
- `schedule_retry` with `claim_until_ >= 0` increments only `claim_attempts_`. It retries at `usb_claim_retry_ms` (200 ms). A held refusal, or the window's end (also checked between attempts in `step`), calls `park_usb`. That sets `usb_retry_at_ = INT64_MAX` and `next_attempt_ms_ = now`, so `desired_transport` returns Wi-Fi.
- The old `now - usb_changed_at_ < usb_settle_ms` gate is removed.
- Test (production timings, timed call log):
  - `disconnect` comes before `discover:usb`, and USB starts ≤ 300 ms after the event;
  - USB attempts start ≤ 500 ms apart, none after t0+2.5 s, and there are ≥ 5 of them (more than 3), yet only one fallback, with `usb_failures` = the window's count;
  - Wi-Fi reconnect starts ≤ t0+2.8 s, and there is no USB probe for the following 30 s;
  - held case: one USB connect, then Wi-Fi ≤ 300 ms after the refusal;
  - success case: USB connects on the 3rd attempt and Wi-Fi is not reconnected.
- Each assertion would fail under the old 2.5 s wait or under counted failures.

C1(b): busy still defers.
- Production: the claim branch returns with `switch.deferred` while `busy_ != None`.
- Test: 6 s pass with no USB call and no `session.closed`. Then busy clears and the test sees `disconnect` → `discover:usb` ≤ 300 ms, and a USB session.

C2(a): never idle, USB retried on release.
- Production: parked USB is never desired while Wi-Fi is connected, because `last_failed_` is reset to None on connect. `UsbHolder(false)` after a seen hold emits `usb.released`, resets the counters and re-arms the claim.
- `main.cpp` polls `usb_exclusive_owners` every 500 ms while the cable is plugged in, the monitor works, USB is enabled and the transport is not USB. `usb_held_by_other` ignores the helper's own pid.
- Test: 30 simulated minutes with zero USB calls and no change in closes, losses or readies. On release, USB starts ≤ 1 s later, with Wi-Fi closed first. A release that is refused again goes back to Wi-Fi ≤ 300 ms after the refusal.
- The poller itself is wiring around the tested manager input, which is the row's named boundary.

C2(b): never idle when Wi-Fi drops.
- Production: while USB is parked and the last failure was Wi-Fi, `desired_transport` returns USB; otherwise Wi-Fi. So the two links alternate on the backoff.
- Test: ≥ 10 attempts of each link in the last 10 of 30 minutes, never ERROR, no `transport.idle`. Wi-Fi comes back with one session.

C3(a): render completion.
- Production: `resolve_output` takes the reported path if it is fresh (written after the request, with 2 s slack). Otherwise it takes the newest fresh `<stem> <n>.<ext>` with the same extension and a numeric counter.
- `check_sidecars` adopts a fingerprint when none is recorded. It raises a generation only when the `<DL>` fingerprint differs **and** the sidecar is newer than the preview and outside the own-write window. Completion records the fingerprint.
- Test: the fake uses add-suffix naming and rewrites the sidecar after processing. The output is recorded once (generation 1/1, the preview bytes are those of `DSC04767 1.jpg`, one row done, none running). An hour of focus every minute plus saves every 10 minutes gives exactly one `process`. A control edit gives exactly one more, which proves the check is live.
- The handoff states that disabling either fix reddens the test.

C3(b): render that cannot complete.
- N = 2: MC-16 issues once and re-issues once. Then `failed_generation` is set and `report_failures` sets the Health detail.
- Test: the Health detail is asserted exactly, still 2 `process` an hour later, no row left `running`.

C3(c): no re-export of current previews.
- Test: restart (`restart_with_startup`) with a leftover `running` row (`new_render_job` inserts `running`) and sidecars rewritten while the app was closed. Result: no `process`, no `running` row, generations stay 1/1.

C3(d): master renders do not pile up.
- Production: the adapter calls `check_master_space` before `recipe_exists`/`process` for masters. Below 5 000 000 000 bytes it sets `disk_space_low` and returns an error. Nothing is deleted.
- `document_id` is a filesystem path (`effective_document_id`), so `statfs` of its nearest existing ancestor works in production. A test default of 1 PiB keeps the suite independent of the host disk.
- Test: one `PSM Master` across a second export. At 3 GB free: the export errors, zero `PSM Master` calls for the new photo, the Health text is exact, and existing files are kept.

**Variation (read-only).** I checked one real owner sidecar (`session test 1/…/DSC04958.ARW.cos`):
- all global adjustment keys, including `Crop`, sit inside `<DL>`;
- `Color_Tag_Index` sits in `<AL>`;
- the processing history sits in `<PHS>`;
- the bookkeeping (`LowResProcessCounter`, `LastMetadataModificationDate`) sits outside `<DL>`.

This is consistent with the fingerprint's premise. No file was modified.

**Assumptions** (handoff 1–9). None contradicts the intention:
- Assumption 2 (the window's end parks USB even without a holder) follows ruling 2: "no periodic USB attempt while Wi-Fi is connected". Ruling 2's "USB attempts continue on the existing backoff while Wi-Fi is not connected" is kept by the alternation (Assumption 4).
- Assumption 7 (RAWs without a recorded fingerprint adopt the current one) trades a missed pre-update offline edit for no re-render wave.

All of them are explained to the owner in the narrative.

**Hard constraints.**
- H1: no external write.
- H2/H4: nothing deleted; migration 008 is additive and the fingerprint upsert is idempotent.
- H3: no pixel work.
- H5: AppleScript only; the IOKit read is a property read.
- H6: untouched.
- H7: the core port (`SettingsSidecar`, `resolve_render_output`) and the `raw_state.settings_fingerprint` column are editor-neutral. The core Health string names Capture One, which has precedent in `editing_startup.rs` and `history.rs` (recorded as a note).
- H8: no Tauri in the services.
- No secrets and no Sony files committed. The suite needs neither the camera nor Capture One.

## Findings
None blocking (B1–B4). Nothing reaches should-fix under flow §5.

## Notes written to backlog
1. The `session test 1NNNN` outputs in the evidence are not explained. Such names would end in a 2-try failure. Recipe properties are checked only at provisioning, so mid-session drift persists until a restart.
2. The edit fingerprint covers `<DL>` only. Local adjustment layers were not observed and may sit outside it.
3. `failed_generation` lives in memory, so a failed photo is re-ordered twice at every start.
4. The `disk_space_low` warning clears only on the next master request.
5. Holder flapping: each release closes Wi-Fi for a claim, with no debounce. Goes to P52 run 2.
6. Assumption 2 consequence: an interface that is slow (over 2.5 s) and unheld stays on Wi-Fi until re-plug or Retry. Goes to P52 run 2.
7. Bookkeeping:
   - the §6.6 health registry needs `preview_not_rendered` and `disk_space_low`;
   - the IMPLEMENTED tracker actor read `codex`;
   - the core Health string names Capture One.

**Lessons for the plans.** Row C3(a) asked the fake to "model the observed `session test 1NNNN` / variant naming". The evidence held two different naming patterns, and only one was the measured cause. A row should name the one it binds, or require both.

**Carry-forward.** Notes 1, 2, 5 and 6 → P52 run 2 (live check). Notes 3, 4 → the next preview/export phase. Note 7 → orchestrator.

## Write perimeter (this session)
- `docs/implementation/backlog.md`: 7 lines appended.
- `docs/implementation/master_plan.md`: P56 tracker row only.
- `docs/implementation/plans/P56_camera_claim_and_render_loop.md`: Review log entry appended.
- `docs/implementation/narratives/P56_camera_claim_and_render_loop.md`: new.
- This handoff: new.

No production or test file was edited. No mutation probes were run (flow §4). The gate run wrote only to `target/` build output and `/tmp/p56rev/` logs. The owner's sidecar was read only. No commits.
