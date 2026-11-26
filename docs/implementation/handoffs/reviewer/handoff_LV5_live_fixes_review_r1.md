---
plan: LV5_live_fixes
role: review
round: 1
verdict: CHANGES_REQUESTED
state: DONE
date: 2026-11-26
actor: opus
---
# Handoff: LV5 live fixes, review r1

**Verdict: CHANGES_REQUESTED.** One blocking finding, B2, `route: production`, and it touches one test file only. The
two fast-Wi-Fi sequence tests (C3(a), C3(b)) assert bindings and placements that are the same with and without the F4
fix. The one clause of those rows that tells the fixed code from the broken code is "the advance never names a
container whose file is on the way". Neither test asserts it. The handoff explains the omission by saying the test kit
cannot capture log lines, but it can (`psm_testkit::logs`, the `logged` helper in `lv2_current_shot.rs`). The
production code of all four units is correct, and every check is green.

## ⚠ OWNER DECISIONS REQUIRED (0)
None. The finding is a test correction inside the plan's own row.

## Gate
- The tracker read `IMPLEMENTED` (2026-11-26, codex).
- Unit checkpoints `b487c8de`, `f82f0886`, `78018009`, `49ee1c8e` and the closing checkpoints `8f12723c`, `989ec516`
  are all in `git log`. The orchestrator's plan amendment is `5fcbc90e`.
- Tree at review: `af96b16d`, clean before and after the review.

## Verified correct
- **Perimeter.** The union of `git show --stat` over the six checkpoints equals the handoff's "Files changed" list, and
  every path is in the plan's unit file lists or the orchestrator's perimeter amendment (`p40_undo_history_values.rs`).
  Two slips in the bookkeeping, with no effect on the code: the handoff lists `lv2_phone_ws.rs` under U4, but it was
  committed in U3 as the plan says, and `main.rs` is not listed under U3. No Tauri command file and no phone route
  file changed.
- **Checks (Review mode, run once on `af96b16d`).**
  - `cargo test -p psm-testkit --test suite -- lv5_current_shot:: lv5_mac_actions:: lv5_phone_socket:: lv5_camera_readouts:: lv2_current_shot:: lv2_phone_ws:: lv1_frame_channel::`: 101 passed.
  - `tools/sidecar/build.sh --stub --test`: 2 CTest targets passed (27 live-view cases, including the 10 C11 cases).
  - `npx vitest run src/features/settings`: 131 passed.
  - `npm run typecheck`: clean.
  - `cargo clippy -p psm-core -p psm-infra -p psm-integrations -p psm-testkit --all-targets -- -D warnings`: clean.
  - `cargo fmt --all -- --check`: clean.
  - The implementer's full-suite stamp stands.
- **U1 (CS4a).** `hold_times` now matches CS4a word for word. An exposure of the session inside `FIFO_WINDOW_MS` holds
  its slot until a photo of the same session has `raw_path` = the `final_path` of one of its non-duplicate deliveries.
  The `status='expecting'` filter is gone, so a filed (`received`) exposure keeps holding. Duplicate deliveries never
  end a hold (MC-33). The read stays inside the binding transaction (CS5). `next_untaken`/`slot_owners` are unchanged.
  - C1(a), C1(b), C4(a), C4(b) fail on the old query: `current` would be `Shot(A)`/`Shot(C)`, and `set(A)` would
    succeed.
  - C1(c) fails if the hold ignored the window.
  - C2(a/b) assert placement and `current = Free`. With every container taken, "no `current_shot.set` line" has no
    distinct observable, so the state proof is equivalent.
- **U2 (PH8, PH6a).**
  - `History::announce` emits after commit: `PhotoChanged` per changed photo, then `SessionChanged` per deduplicated
    session.
  - `ReviewService` `classify`/`reorder`/`delete`/`keep_published`/`move_photos`/`copy_photos` announce. `touched`
    holds the source and the target (`review.rs:401,413`).
  - `HistoryService::reverse` announces every touched session for every step kind.
  - `delete_announced` is a one-line alias, so the phone delete raises no double event. No `emit` was added in
    `src-tauri/src/commands/` or `phone/routes.rs` (H8).
  - The preview token is a hash of path and mtime. It is `None` when `preview_file` gives none or the file's metadata
    is missing, and it never carries a path.
  - C5(a–g) and C6(a–e) fail on the old code: no `session` message within 1 s, which panics.
  - C8(a/b) observe null → token → a different token.
  - The amended P40 `c7a_attribute_undo_announces_session` matches the orchestrator's ruling.
- **U3 (PH5a).**
  - One write is in flight at a time, pinned in `write_wait`. Revoke, reader-closed, idle and stall are served
    while it is in flight.
  - The in-flight write is dropped only on a close reason, and the loop breaks at once.
  - A frame is taken from `LatestSlot` only between writes. Pings are sent between writes. `revoked` and
    `reader_done` use `notify_one`, which stores a permit, so a wake is not lost between polls.
  - The close frame is sent best effort within 200 ms, and then the viewer unsubscribes.
  - C9(a) asserts at most 2 frames with the last = seq 40, then the next frame, with no close.
  - In C9(b), timing tells stall from idle: gone within 5 s while idle is 10 s.
  - C9(c) passes only if revoke is served during the write: gone within 1 s while stall is 3 s.
  - C10(a/b) cover the 5 s re-read and clearing it on unmount.
- **U4 (FC4a, HB5a).**
  - `focus_area_movable` matches FC4a's set: 4–8, 0x14–0x1D, 0x23–0x27. `area_read` = `focus_area >= 0`, and
    `FocusState` defaults to −1.
  - The refusal branch existed before and only its condition changed. It writes nothing, and no new
    `SetDeviceProperty` intent exists (FC6, M-LV5).
  - `req_display` accepts a string or an object with a string `display`. Anything else is unparsable (C12(e)).
  - `exposure_readout` maps all four C12(c) tuples and "" or "special" to "–".
  - `lv1_c15b` now uses the real object shape with literal expectations.
  - C12(b)/(f) observe the hub event and the phone socket message.
- **Assumptions.**
  - None contradicts the intention.
  - The sixth one is factually wrong: "the existing test kit cannot capture log lines". It is the root of R1-B1 below.
- **H1–H8.** H1–H8 hold. Nothing external is written. No RAW is touched. No pixel work. No Sony values in core: the
  Focus Area numbers stay in the sidecar and the display mapping in integrations. Services emit and handlers don't
  (H8). No live service and no secret is involved.
- **Guide (§7A).** 02 is updated (current shot, phone server: announcements, preview token, stall rule). 03 is updated
  (on-the-way rule, Focus Area). 04 is updated (`PhoneImage.preview` coupling and exposure displays). No mention of
  `write_timeout` is left anywhere.

## Findings

### R1-B1 (blocking, B2, route: production): C3(a) and C3(b) cannot fail on the defect they reproduce
- **What is wrong.** In both planned sequences every shutter is bound by 3.6 s, before any file is filed. s5 at
  4.8 s binds to no container with or without the fix, because the held value is already `Free`. The only difference
  is the advance after s5:
  - Old query: s1's hold ended when its file was filed (C3(a)), or s1's and s2's holds ended (C3(b)). The advance
    sets `Shot(W)` (`current_shot.set … container=W by=advance`), aiming the Mac's Next and the phone at a shot whose
    photo is still arriving. This is the F4 symptom.
  - Fixed query: `Free`.
  - Placements are identical either way (each photo follows its exposure's binding). So both tests, as written, pass
    on the unfixed code.
  - The row's discriminating clause ("every `current_shot.set … by=advance` line names a container no earlier
    exposure is bound to, or `none`") is asserted neither as a log line nor as state.
- **Reviewer probe.** I added `assert_eq!(current(&a, s), Current::Free)` after `e5` in both tests (a test file, not
  production code), ran `lv5_current_shot::lv5_c3`, and both passed. The file was restored with `git checkout` and its
  checksum is identical (`be2458e6…`). The fixed production code is right; only the tests are blind.
- **Authority.** Rows LV5·C3(a), LV5·C3(b) (outcome and trace CS4a, M-LV4). flow.md §5 B2. The orchestrator's
  instruction in the plan's Review log, to close the trace gaps if the harness can capture logs, otherwise to record
  the state proof: the harness can (`crates/testkit/src/logs.rs`, `lv2_current_shot.rs::logged`), and no state proof
  was added for C3.
- **Correction.** In both tests, assert the current shot after each injected shutter. Each value must be a container no
  earlier exposure is bound to, or `Free`:
  - C3(a): X, Y, Z, Free, then **Free after s5**.
  - C3(b): X after s1; after the tap, Z; then X, Y, Free, **Free after s5**.
  Capturing the `current_shot.set` lines with the existing `logged` helper is an acceptable alternative if those lines
  are written on the test thread. The file and intake times may also be aligned with the row (s3 at 5.6 s, s4 at 7.6 s,
  each taken in 1.0 s after filing).
- **Files the fix may touch.** `crates/testkit/tests/suite/lv5_current_shot.rs` only.

## Notes written to backlog
In `docs/implementation/backlog_live/LV5_live_fixes.md`, as the plan's Review mode names it:
1. C3 clock offsets differ from the row.
2. The C5/C6/C7 tests assert `rev > 0`, not "above every earlier rev".
3. The Settings → Phone 5 s re-read shows an error toast on every failed read.
4. The handoff's file list puts `lv2_phone_ws.rs` under the wrong unit.
5. The guide 04 coupling for U2 is prose above the table, not a table row.

## Lessons for the plans
- A row whose only discriminating clause is a log line should name a state outcome beside it (here: `current` after
  s5). In the C3 sequences, bindings and placements cannot tell the fixed code from the broken code. Planners should
  check that each scenario row has at least one asserted outcome that differs before and after the fix.
- C9(a)'s parenthetical ("the old 500 ms `write_timeout` would have closed it") holds only for `TestPhone`'s default
  timings. With the row's own `stall_close: 3 s`, the old code at `write_timeout: 3 s` would also survive a 1.5 s stall.
  The row still discriminates frame skipping and the close, so it is fine. Just don't reuse the reasoning.
- Plan prompts should point to `psm_testkit::logs` and the `logged` pattern wherever a row names a log line, so an
  implementer doesn't conclude that logs can't be captured.

## Probe declaration
- Touched and restored: `crates/testkit/tests/suite/lv5_current_shot.rs`. Two temporary assertions were added, then
  `git checkout --`. The SHA-1 is identical before and after (`be2458e63d0e121a18c189ccf693240f8b98d9ef`).
- `tools/sidecar/build.sh --stub --test` rebuilt the ignored stub binary
  `apps/desktop/src-tauri/binaries/camera-sidecar-aarch64-apple-darwin`, as every run does. `git status` stayed clean.
- No production file was edited. No database or app state was touched outside the test temp dirs.

## Files written by this review
This handoff · the tracker line and Review log entry in `docs/implementation/plans/LV5_live_fixes.md` ·
`docs/implementation/backlog_live/LV5_live_fixes.md` (new).
