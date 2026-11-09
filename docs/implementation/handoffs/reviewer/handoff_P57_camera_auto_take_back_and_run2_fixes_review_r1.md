---
plan: P57_camera_auto_take_back_and_run2_fixes
role: review
round: 1
verdict: CHANGES_REQUESTED
state: DONE
date: 2026-11-09
actor: opus (pipeline-reviewer)
---

# P57 review r1: CHANGES_REQUESTED (2 findings, both test-only)

The production code reads correct for every row, and suite, lint, typecheck and the helper tests are green. Two criterion rows fail B2.
Their tests pass unchanged on the pre-P57 tree `edb6f83`, the code that showed F16 and F18 in run 2. So they cannot fail on the
outcomes those rows exist to guard. The fix touches test files only.

## ⚠ OWNER DECISIONS REQUIRED (0)

None.

## Verified correct

**Gate.** Tracker `IMPLEMENTED`; checkpoint `763b99771f4383f45a3ed57964ff472db65c9e32` exists.

**Perimeter.** `git show --stat 763b997` lists 34 paths, exactly the handoff's Files changed (8 desktop, 7 core/infra, 6 integrations,
4 testkit, 7 native, 2 process). No extra or missing path.

**Evidence (reviewer run, clean tree `e450d91`, code = `763b997`).** `cargo test --workspace` 849 passed / 0 failed (56 groups);
`npm test` 412/413, the single failure being the known `p33_split_grid` C4(d) intermittent that the plan excludes (re-run alone: 7/7);
`npm run typecheck`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt --all -- --check` exit 0;
`tools/sidecar/build.sh --stub --test` 28 tests / 0 failed. `git status --porcelain` clean after the run. During the review the orchestrator committed `d6c8b00` (P58 plan, STATUS, tracker, changelog 39).
It touched docs only, so the code tree, and with it this evidence, is unchanged.

**Variation (scratch worktree `/tmp/p57rev/wt` at `edb6f83`, removed after).** With the P57 testkit helper `with_camera_script_running`
back-ported and the C2(a)/C4(a) test bodies copied verbatim, both tests **pass** on pre-P57 code. Re-run with the fake Capture One's
`call_delay_ms` = 1500 (C4) and `call_delay_ms`/`delay_document_info_ms` = 3000 (C2): both still pass.

**Rows, by reading the production code and the test:**
- **C1(a)** `CameraRuntime::auto_take_back` triggers when: setting on, plugged (`usb.arrived` / `usb.initial present`), holder resolved to
  Capture One (`ptpcamerad` + Capture One running → `holder_is_capture_one`), and not yet taken this plug-in. `auto_taken_this_plug`
  is set before the run starts and reset only by a removal or a `TakeBackBlocked` race. A helper restart's `usb.initial present` does not
  reset it (`swap`). Test: one quit before one launch, cable connected, a second claim makes nothing. Fails if the trigger is missing or repeats.
- **C1(b)** Each 20 ms tick re-evaluates `blocked_reason()` and shows `Waiting to take the cable: <reason>` in place of the held
  sentence (also applied when a claim arrives while waiting). Test: running `render_master` job → exact sentence, no quit; job done → quit.
  UI test: header button carries the sentence.
- **C1(c)** Setting off → early return and no waiting text; `set_auto_take_back` applied at bind and on the no-rebuild save path. Test: no
  quit, manual `blocked_reason() == None`. UI test: checkbox default on, toggles `auto_take_back:false`.
- **C2(a)** Camera tick now runs on its own 20 ms thread (`camera_background`, joined at shutdown). In the old loop it ran after
  arrival/follow/jobs/previews ticks, which can block on Capture One. The fix matches the cause the handoff found. **Test: see F2.**
- **C3(a)** `exposure_observed` (now under the ledger worker lock) inserts, then `match_late_exposure` links the oldest done, non-duplicate,
  unassociated delivery completed in `[at−300 s, at]` (`association='fifo'`, exposure `received`, session from the delivery): MC-33
  rule 3 mirrored. Test: filed `DSC05001.ARW`, exposure 12 s later → linked; watchdog at +61 s not raised. It fails without the match.
- **C4(a)** `prepare` nulls `theme`, `output_format`, `camera.auto_take_back` before comparing, so an output save takes `Simple`. That path calls only
  `export.set_format`, `set_auto_take_back`, `health.clear`: no Capture One call, no render, no rebuild. Runs capture their format at start.
  **Test: see F1.** (Side effect in backlog: the output render worker still reads the current format.)
- **C5(a)** `GatedScripting::launching`/`launch` re-check `refuses_here()` inside the lane, so a launch-capable script queued before the
  gate was held is refused once it gets the lane. Non-launching JXA scripts guard on `running()`, so they cannot relaunch. Test: quit, no
  launch while step 4 waits, one launch after the USB session, gate released. (In-lane race not reproduced: backlog note, not B2,
  because reproducing it needs control over lane ordering between threads.)
- **C6(a)** Helper emits `camera.ftp_state` on link change and every 60 s. Runtime: status 2 clears; non-zero error sets the exact plan
  sentence with the default-route LAN address (UDP connect, no packet) and the receiver port; a wireless delivery newer than the error
  baseline clears. Test: exact sentence, clear on upload, re-raise, clear on status 2.
- **C7(a)** `camera.discovered` (Wi-Fi) stores `--ip <ip> --mac <camera_id>` in the supervisor spec (the Wi-Fi camera id is the MAC,
  `camera_discovery.cpp`). `SdkCameraAdapter::discover(Wifi)` returns `direct-ip:` without discovery when an IP is set. A failed direct
  connect forgets the cache and is non-fatal → discovery. C++ test: `cached:wifi`, `connect:wifi`, `forget:cached`, then `discover:wifi`,
  connected.
- **C8(a)** Green as above.

**Assumptions (handoff).** None contradicts the intention. The FIFO late match follows MC-33's fallback. The default-route LAN address and
status 2 = connected are delegated choices. A fatal cached-direct failure → one discovery is the plan's goal 7. A pre-start race → still
pending, other failure → consumed is the smallest reading of "once per plug-in". The automatic trigger replaces 10E's click-only line by
changelog 38 (book body not amended: backlog).

**Hard constraints.** H1 n/a. H2: no RAW touched. H3: no pixel work. H4: late match persisted, idempotent on replay (only unassociated rows).
H5: Capture One driven only through its dictionary (save/quit/launch). H6 n/a. H7: core gains `FtpUnreachable`, a generic
`auto_take_back` setting and a store method; FTP is the app's own receiver (existing precedent). H8: trigger in the integrations runtime,
commands thin. Owner rules: no kill, no camera setting change (FTP state read-only), no SDK file or address committed (tests use
192.0.2.x), Capture One quit only by the take-back.

## Findings

### F1 (B2), route: production. C4(a)'s test cannot fail on the F18 outcome
- **What is wrong.** The row's input is "render queue busy (fake Capture One blocks `process`)". The test only leaves one preview render
  in flight against a fake that answers at once, in a TestApp with startup work off. It passes unchanged on `edb6f83`, the code that rebuilt
  the app on every output save and hung/re-rendered in run 2 (also with every fake call delayed 1.5 s). So a save that still rebuilds or
  re-renders passes the row.
- **Authority.** Plan C4(a); flow §5 B2; evidence §Acceptance run 2 A16/F18.
- **Correction.** Give the test the row's input: Capture One busy in a way a rebuild-on-save would hit (e.g. startup work on and slow/held
  fake calls, or a fixture where the old path re-orders previews). Keep the ≤ 1 s and no-new-`process` assertions. Show it red on `edb6f83`
  and green on the fix; record both in the handoff.
- **Files the fix may touch.** `crates/testkit/tests/p57_camera_auto_take_back_and_run2_fixes.rs`; if a hook is needed,
  `crates/testkit/src/{test_app.rs, fake_capture_one.rs}`. No production file.

### F2 (B2), route: production. C2(a)'s test cannot fail on the F16 outcome
- **What is wrong.** The handoff names F16's cause: the camera tick shared a thread with preview/Capture One work that can block. The test injects one
  line into an idle app, so nothing occupies that loop. It passes unchanged on `edb6f83`, where the tick still shared the blocking loop (also
  with the fake's calls delayed 3 s: in this fixture the old loop makes no Capture One call). A regression that put the camera tick back on
  the shared loop would stay green.
- **Authority.** Plan C2(a) (outcome "within 200 ms, not when another line arrives", trace P52 F16) and goal 2; flow §5 B2.
- **Correction.** Reproduce the cause at the boundary: make the app's shared background loop busy in a slow Capture One call (startup work
  on, a document and a pending preview or follow call, the fake's call delay > 1 s) when the single helper line arrives. Assert the status
  within 200 ms. Show it red on `edb6f83` and green on the fix.
- **Files the fix may touch.** Same as F1.

## Notes written to backlog
1. Mid-export output-setting change: `RenderOutputJob` renders with the current format while the run keeps its start key. The rest of a running
   export comes out at the new format under a manifest naming the old one (publish re-exports; Shopify unaffected). The handoff's "each run
   keeps its format" is only partly true.
2. `match_late_exposure` runs for every exposure: an on-time exposure can claim an older orphan delivery in the 300 s window and hide a real
   "not received".
3. C5's in-lane gate re-check is not reproduced by the test (lane-ordering race; confirmed by reading; watch on hardware).
4. Book/registry drift: 10E MC-34 still says click-only; registry lacks `camera_ftp_unreachable`, `ftp_unreachable`, `camera.auto_take_back`.
5. The FTP-unreachable warning can show while on the cable.
6. Automatic take-back failure is only logged; preconditions re-read every 20 ms while held; manual click while waiting shows "already running".
7. `FtpUnreachable` ties `HeldByOtherProgram` at rank 4.

## Lessons for the plans
- When a row says "the implementer finds the cause", the row's input should be amended (or the implementer told) to **reproduce the
  found cause**. A latency row tested on an idle app proves nothing about a blocked one.
- Rows that fix a run-observed defect should state the acceptance standard "red on the pre-phase commit". Earlier reviews used it
  (P26) and it would have caught both findings at implementation time.

## Write perimeter (this session)
- `docs/implementation/master_plan.md`: P57 tracker row → `CHANGES_REQUESTED`.
- `docs/implementation/plans/P57_camera_auto_take_back_and_run2_fixes.md`: Review log entry.
- `docs/implementation/backlog.md`: 7 P57 notes appended.
- This handoff.
- Scratch only, outside the repo: `/tmp/p57rev/` (logs; worktree `/tmp/p57rev/wt` at `edb6f83` with a back-ported testkit helper and a
  scratch test `p57_variation.rs`, removed with `git worktree remove --force`). No production file edited, not even temporarily.
- Side effect of the mandated stub test run: `tools/sidecar/build.sh --stub --test` rewrote the ignored
  `apps/desktop/src-tauri/binaries/camera-sidecar-aarch64-apple-darwin` (stub, as the implementer left it). Tree status clean.
