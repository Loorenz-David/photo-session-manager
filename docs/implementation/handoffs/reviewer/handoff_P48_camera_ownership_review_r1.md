---
plan: P48_camera_ownership
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-07
actor: opus (pipeline-reviewer)
---
# Review: P48 camera ownership, round 1 (relaunched after the gate block)

**Verdict: APPROVED_WITH_NOTES.** There is no B1–B4 finding. The production code satisfies all 38 rows, and each row's test observes its
outcome at the row's boundary and would fail on a wrong outcome. The suite, lint and typecheck are green on the reviewed tree. No
assumption contradicts 10E. 6 notes went to the backlog. This replaces the earlier r1 handoff (BLOCKED at the gate: no checkpoint then).

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Gate
- Tracker row was `IMPLEMENTED`. Checkpoint `4da7fe9` (`CHECKPOINT (not approved): P48_camera_ownership implement r1`, committed by the
  orchestrator after the implementer lost its network) is in `git log`.
- Reviewed tree: `577265d`. It is `4da7fe9` plus the orchestrator's handoff SHA line and STATUS only, so there is no code drift. The tree
  was clean before my edits.

## Verified correct
- **Perimeter.** `git show --stat 4da7fe9` lists 41 paths. That equals the handoff's Files list, which is the plan's derived 43 minus
  `crates/core/src/services/camera.rs` and `crates/infra/Cargo.toml` (both disclosed as unchanged: the gating already branches on the
  capability, and the extern-block option was taken). By group: integrations 5 new + 10 changed, core 1 + 8, infra 2 + 1, desktop 5,
  testkit 4 + 3, tests 2 (the p46 change is the two declared sites). The tracker and the plan log were committed by the runner in
  `fb036a6`. No path is missing and none is undeclared.
- **Suite (one reviewer run, code tree `4da7fe9`).** Every command exited 0:
  - `cargo test --workspace`: 769 passed, 0 failed, over 48 binaries. That includes `p48_camera_ownership` 32/32, desktop `p48_tests`
    2/2 (C5(g), C8(a)), unit `ownership::tests` C1(a)–(d) and `take_back::tests` (the C4 halves plus the check order).
  - `cargo clippy --workspace --all-targets -- -D warnings`.
  - `cargo fmt --all -- --check`.
  - `npm run typecheck`.
  - `npm test`: 50 files, 369/369.

  `bindings.ts` was unchanged after the run (`export_bindings` matches the committed file).
- **Holder naming (R-I30), C1.** `name_programs` returns:
  - for `ptpcamerad`: the known programs in `KNOWN_CAMERA_PROGRAMS` order (Capture One first) and the exact sentences, with guides
    Imaging Edge → "quit it and stop it opening at login", Photos/Image Capture → the "Connecting this camera opens" fix, Lightroom →
    "quit it", and the system-service sentence when the list is empty;
  - for any other owner: `programs: []` and the "<owner> is using the camera" sentence.

  `holder` = `programs[0]`, else the owner. The adapter reads the process list once per `usb.claimed_by`, outside its state lock, and
  writes `(HeldByOtherProgram, holder, sentence)`. `derive_status` copies `message`. `health_items` and `CameraStatusResponse` use it, so
  the health detail and the command message are one sentence. `PsProcessList` is read-only (`/bin/ps -axo comm=`, last path component).
  C1(e)/(f) assert the full held status, the health detail, the `capture` line over Wi-Fi, and the clear on a USB `connection.ready`.
- **Start order (R-I31, M40), C2.**
  - `StartOrder` is created once in `open_with_adapter_bind`, `Waiting` iff a support package is installed (the same test that selects
    the direct adapter), and shared by the lanes and the runtime. It rides `Lanes` through `SettingsSetContext`/`reopen` and is never
    rebuilt by `build`.
  - It is satisfied by USB `connection.ready`, `usb.initial{present:false}`, `usb.fallback_to_wifi`, `sdk.unavailable`, a failed spawn,
    `HelperEvent::Exited`, or a gated `editor_info` that reports running. Otherwise it times out 15 s of clock time after the first
    `wait()`. The first reason wins.
  - `GatedScripting::launch` checks the gate. Off the permitted thread it waits outside the lane mutex, then re-checks the gate before
    the lane. The permitted step 5 never waits.
  - The C2(a) test asserts no `launch` while the USB ready line is unprocessed (real 1.5 s delay in the script). C2(b)–(d) return only
    if the event satisfies the order (zero clock advance). C2(e) is still waiting after 300 ms real time and returns after +15 s. C2(f)
    and (g) launch again with no clock advance.
- **Editor-launch gate (R-I32, M40), C3.**
  - `LaunchGate` is process-wide, with a thread-local permit keyed by the gate's address and released on guard drop.
  - Exactly the six scripts are refused on both lanes when the gate is held and the thread is not permitted. `quit` (the script guards
    `running()`) and `editor_info` are not gated.
  - `ensure_provision` returns `ProvisionReport::deferred()` at launch, make, open, recipe and style before each step's health handler.
  - `front_provision_document` returns the deferral before its `.ok()`.
  - `EditingStartup::run` returns `Ok` right after a deferred ensure, so a health check while held makes exactly one `editor_info` call
    and writes no false `editor_not_running`/`document_not_open` item (C3(g) asserts `ops() == ["editor_info"]`).
  - C3(a)–(h) each assert the missing op and the health item that was not written.
- **"Take the camera back" (R-I32, M39), C4/C5.**
  - Preconditions are checked in §6.6 order: uploads or an expected download; an `expecting` exposure younger than 10 s; `running`
    (not `queued`) render/publish jobs; renders in flight; Capture One as holder.
  - A second concurrent run is refused with "already running".
  - The steps:
    1. The gate is held by a guard that drops on every path.
    2. `save` goes through the ungated scripting; an error is step 2 and nothing else runs.
    3. Guarded `quit`, then a poll every 500 ms of clock time for ≤ 20 s; still running is step 3, and there is no kill API anywhere.
    4. `editor_info` is checked at the start and every 500 ms ("started meanwhile" is step 4). Then `usb-retry`, re-sent 1 s after a
       claim seen inside the first 5 s, and a wait ≤ 15 s for our USB session; a timeout gives `camera_held: false`.
    5. `EditingStartup` runs under `permit` (launch with no start-order wait, ensure, front, folder for the active session).
  - The C5 tests pin each branch: the op subsequence, one `usb-retry`, Cable afterwards, the gate released, step 2/3/4 failures with
    Capture One left running and no `usb-retry`, step 5 after a step 4 timeout, and no `quit` over 30 s of ticks.
  - Over the wire, `take_back_failed` carries `step` as a string and `take_back_blocked` carries `reason` (C5(g) asserts the exact
    error JSON).
- **Activation independent of Capture One (R-I33), C6.** `SessionsService::with_camera` is wired at `test_app.sessions()` and both
  desktop commands, and `EditingStartup.camera` at all lib sites. With the direct adapter, `activate` and `deactivate` store first. Any
  failure afterwards (including a deferral) sets `editor_not_following` (the §6.6 text), clears `capture_folder_not_set`, and returns
  `Ok`. `EditingStartup` clears it after a successful folder step. With the Capture One camera adapter the behaviour is byte-for-byte
  as before (C6(b)'s second half; p24/p25 green).
- **Sleep (R-I23), C7.** The adapter emits `sleep` only on a change: hold on USB `connection.ready`; release on USB `session.closed`/
  `session.lost`, Wi-Fi `connection.ready`, and `Exited`. The runtime also releases at shutdown. `IokitSleepInhibitor` is idempotent and
  builds both CFStrings, then releases them. C7(a) covers the repeat (`holds() == 1`), the loss, Wi-Fi, and a crashed helper.
- **Test camera connection, C8.** `holder_report` lists the known programs from a fresh process listing in every case. It names the
  holder and the sentence only while the derived status is `HeldByOtherProgram` (L19).
- **Assumptions 1–11.** Each one is inside the plan's Delegated choices or is the smallest reading of the contract. None contradicts
  10E MC-34/MC-31: no `EditingProvider::quit`; `AlreadyRunning` set in the gated `editor_info` (only `ensure_provision` calls it there);
  failed spawn as "unavailable"; the `Lanes` struct; sleep release on Wi-Fi ready; unknown `editor_info` keeps polling; `holder_is_capture_one`
  over the entry list; `completed_steps` 5; clear on follow; `ps` dedupe; the fixture resetting `launched`. They are told to the owner in
  the narrative.
- **Hard constraints.**
  - H5: Capture One is driven only by its dictionary's `quit`.
  - H7: core gains neutral ports only (`ProcessList`, `SleepInhibitor`, `renders_in_flight`, `ProvisionReport.deferred`); see the note
    on the `editor_not_following` text.
  - H8: the commands are `spawn_blocking` → service.
  - H2 and H4: untouched.
  - Secrets: none. No live service: the suite uses `FakeProcessList`/`FakeSleepInhibitor` on every `TestApp`, rebuilds included.
  - Camera hard rules: no process killed, no camera setting changed.

## Findings
None blocking (B1–B4). Notes only (below).

## Notes written to backlog
1. **Gate checked before the lane mutex (→ P51/P52).** `launching()` checks `refuses_here()` and then waits for the lane. A script that
   waited on a busy lane can run after the take-back holds the gate, which can launch Capture One mid-sequence. Step 4 then reports a
   false "started meanwhile". Fix: re-check inside `call` (M40).
2. **Step 5 folder failure (→ P51).** With the direct adapter, `EditingStartup::run` sets `editor_not_following` but still returns `Err`.
   The take-back then answers `take_back_failed{5}` and drops `camera_held`.
3. **Start-order wait holds `provision_lock` (→ P51).** At app start a Health or Settings load can wait up to 15 s.
4. **H7 spirit.** The `editor_not_following` sentence naming Capture One lives in core (plan-specified; precedent in `history.rs`).
5. **Open P46 note.** There is still no "holder gone" clear trigger for `HeldByOtherProgram` (MC-34 "until the holder is gone").
6. **Test hygiene.** C2(e) would hang rather than fail on a wrong bound (unbounded `join`).

Carry-forward: notes 1–3 and 5 → P51 (UI and wording) / P52 (hardware rows "Take the camera back", "USB resumes once it is released").
Note 4 waits for a second editing provider. Note 6 can go to any later test touch.

## Lessons for the plans
- A gate whose check must be atomic with the guarded action should say "checked under the lane lock" in the contract. "Returns deferred
  without running a script" was implemented as check-then-acquire.
- Take-back step 5 reuses `EditingStartup::run`, whose error semantics (folder failure = `Err`) predate R-I33. The plan could have said
  whether an R-I33 health item counts as step 5 success.

## Write perimeter (this session)
`docs/implementation/master_plan.md` (P48 tracker row only), `docs/implementation/plans/P48_camera_ownership.md` (Review log entry),
`docs/implementation/backlog.md` (6 notes appended), `docs/implementation/narratives/P48_camera_ownership.md` (new), this handoff
(rewritten). No production or test code was touched, and nothing was edited even temporarily. No probes, no archgraph or other
tool-recorded state, no commits.
