---
phase: P57_camera_auto_take_back_and_run2_fixes
title: After the second camera test — the cable is taken back automatically when you plug in while Capture One holds it, helper messages reach the app at once, no false "not received" warnings, saving settings no longer hangs or re-exports everything, a camera that cannot reach the Mac is named, and a restarted helper reconnects without a full search
difficulty: high
projection: no
depends_on: [P56_camera_claim_and_render_loop]
---
# P57 — Automatic take-back + P52 run 2 fixes (F15–F21)

Written by the orchestrator (Opus) on 2026-11-09 from P52 run 2 (evidence `docs/intention/evidence/sony_camera_lab.md` §Acceptance run 2). The owner:
"we can make the take camera back to be automatic as you proposed earlier … and perform the fixes so that we can move to installing the app on the work pc".

## Goal
1. **Automatic take-back (owner decision 2026-11-09).** With Capture One running, macOS's camera service (`ptpcamerad`, started with Capture One) claims
   the cable at enumeration, before the SDK can even search (run 2: claim in < 0.2 s; our first connect ~4.4 s). Today the photographer must click
   "Take the camera back" (~4 s). After this phase, when the camera is plugged in and the helper reports the cable held (`usb.claimed_by`) while
   Capture One is running, the app runs the existing take-back by itself, **once per plug-in**: same preconditions (P54); if one blocks, the header says
   `Waiting to take the cable: <reason>` and the take-back starts when it clears (still the same plug-in). A setting
   `camera.auto_take_back` (default **on**; Settings → Camera: "Take the cable back from Capture One automatically when the camera is plugged in")
   turns it off, restoring the click-only behaviour. This replaces the owner rule "Capture One is quit only on the photographer's click" for this case.
2. **F16/F19 — helper messages arrive late.** On USB, a photo's helper events reached the app 12 s late, together with the next shot's, and the photo
   appeared only then. The helper flushes stdout per line (`native/camera-sidecar/src/json_log.cpp:200`). After this phase every helper line is
   handled by the app within 200 ms of being written, independently of later lines. The implementer finds where the delay comes from (the Rust reader in
   `crates/integrations/src/camera/sidecar/{client.rs, supervisor.rs}`, a buffered reader waiting for more data, a lock held by the event consumer, or
   the helper's SDK-callback-to-stdout path in `main.cpp`) and writes the cause in the handoff.
3. **F20 — false "not received".** A late exposure event must match a file that was filed before it (same camera file name / stem within the ledger's
   association window), so no `FilesNotReceived` is raised when every photo arrived.
4. **F18 — settings save.** Saving Settings returns within 1 s whatever the render queue is doing, and the UI leaves "Saving…". Settings that do not
   concern the camera, the receiver or the document do not re-open the runtime. Changing an **output** setting (output size, output JPEG quality) does
   not re-render existing previews (previews use the preview recipe; output settings apply to future exports and masters) — unless the book says
   previews follow them, in which case keep the re-render but run it in the background and keep the save fast (implementer: cite the chapter).
5. **F17 — take-back relaunches Capture One too early.** While a take-back runs (manual or automatic), no other path (the P49/P54 relaunch-after-quit,
   the editor gate, Health) relaunches Capture One before take-back step 4 (our USB session) has finished or failed; take-back reports success when the
   outcome is a USB session with Capture One back (no "stopped after step 4" on a good outcome).
6. **F15 — camera cannot reach the Mac.** The helper reads the camera's FTP state (`ftp_connection_status`, `ftp_connection_error`, as in the run 2 log)
   at connect and every 60 s during a session. A connection error raises a header state:
   `The camera can't reach this Mac for uploads. On the camera, set the FTP server's host to <this Mac's current LAN address> and port <receiver port>.`
   It clears when the camera's FTP connects (status reads connected, or an upload arrives).
7. **F21 — reconnect without a full search.** After a helper restart or a lost Wi-Fi session, the helper first tries to connect to the camera it was last
   connected to (cached endpoint from the last `camera.discovered`), skipping the ~3.8 s SDK discovery; on failure it falls back to discovery. If the
   SDK offers no way to build a camera object without discovery, record that in the handoff and leave discovery as is (C7 then asserts only the fallback).

**Not in this phase:** grabbing the cable directly before `ptpcamerad` or an own PTP connection (owner: later; backlog), storage policy, faster previews
(F11), live view.

## Read first
Evidence §Acceptance run 2 (and §Acceptance P52 run 1, §V6). Book `10E_direct_camera_adapter.md` MC-31, MC-32, MC-33, MC-34 (§Take it back, R-I32),
`16_changelog.md` entries 37–38. P54, P55, P56 narratives and Review logs. Code: `crates/integrations/src/camera/direct/{take_back.rs, start_order.rs,
ownership.rs, adapter.rs}`, `crates/integrations/src/camera/{runtime.rs, sidecar/client.rs, sidecar/supervisor.rs, sidecar/protocol.rs}`,
`crates/integrations/src/editing/capture_one/{gate.rs, launch_gate.rs, adapter.rs}`, `crates/core/src/services/{camera_ledger.rs, settings.rs, previews.rs}`,
`crates/core/src/mechanisms/capture_ledger.rs`, `crates/core/src/domain/camera.rs`, `crates/core/src/mechanisms/camera_status.rs`, `crates/infra/src/settings_file.rs`,
`native/camera-sidecar/src/{main.cpp, json_log.cpp, transport_manager.cpp, camera_discovery.cpp, camera_connection.cpp, camera_properties.cpp}`,
`apps/desktop/src/features/settings/{SettingsPage.tsx, CameraSection.tsx}`, the header camera popover. Tests: p48, p53, p54, p55, p56 testkit files, C++ tests.

## Files expected to change (write perimeter)
The files above as needed, `crates/testkit/tests/p57_camera_auto_take_back_and_run2_fixes.rs` (new), new C++ cases, UI tests for the setting and the
waiting sentence, generated bindings. Existing assertions change only where this contract changes them (list each in the handoff). ≈ 25 paths, ≈ 800 lines.

## Core mechanisms (unit tests allowed)
Take-back trigger decision (plug-in + held + Capture One running + setting on + not already done for this plug-in); ledger late-exposure association.

## Criteria (8 criteria, 10 rows)
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | automatic take-back | setting on; fake Capture One running; fake helper: `usb.arrived` then `usb.claimed_by` | the take-back runs once without a command (fake Capture One receives one quit, then one launch after our USB session); a second `usb.claimed_by` in the same plug-in does not start another | `[rule]` owner decision 2026-11-09; MC-34 §Take it back |
| C1(b) | automatic take-back waits | as C1(a) with an export running | header sentence `Waiting to take the cable: an export is running`; no quit; when the export ends, the take-back runs | `[rule]` owner decision; R-I32 |
| C1(c) | setting off | `camera.auto_take_back=false`; as C1(a) | no quit; the manual "Take the camera back" state as today | `[rule]` owner decision |
| C2(a) | helper line latency | fake helper writes one event line, then stays silent for 5 s | the app's camera status / ledger reflects that event within 200 ms (not when another line arrives) | `[rule]` MC-31; P52 F16 |
| C3(a) | late exposure | FTP delivery of `DSC05001.ARW` filed, then 12 s later `capture.exposed` (camera-initiated) | no `camera_files_not_received` after +61 s | `[rule]` MC-33; P52 F20 |
| C4(a) | settings save | render queue busy (fake Capture One blocks `process`); save a changed `output_quality` | `settings_save` returns ≤ 1 s; no new preview render issued for existing photos (or, if the book requires it, issued in the background after the save returned) | `[rule]` 12 §Settings; P52 F18 |
| C5(a) | no early relaunch during take-back | take-back in progress (step 1 quit done), Capture One gone | no launch of Capture One by any path before step 4 ends; final state success with no failure notice | `[rule]` MC-34; P52 F17 |
| C6(a) | camera can't reach the Mac | fake helper `camera.ftp_state` with `ftp_connection_error` non-zero / status not connected | header needs-action with the sentence naming the Mac's LAN address and receiver port; cleared on a connected status or an upload | `[rule]` MC-32; P52 F15 |
| C7(a) | reconnect without search | C++ transport test: endpoint cached from a previous connection; helper restarted (or Wi-Fi lost) | the first action is a direct connect to the cached endpoint (no discovery); on its failure, discovery | `[rule]` MC-31 §Process; P52 F21 |
| C8(a) | existing suites | everything else | `cargo test --workspace`, `npm test` (known intermittent `p33_split_grid` aside), clippy, fmt, typecheck, `tools/sidecar/build.sh --stub --test` green | `[rule]` master plan §7.2 |

## Delegated choices
Where the automatic trigger lives (adapter vs runtime); the waiting sentence's position (header, like other camera states); how the Mac's LAN address is
found (the interface with the default route); the FTP-state poll mechanism; how the endpoint is cached.

## Manual try-out
On the work Mac / next hands-on: plug in with Capture One open → Capture One closes and reopens by itself, dot `Cable` within ~5 s, no click; take
two photos on the cable → each appears in Session Manager within ~2 s; change the JPEG quality and save → instant; point the camera's FTP host at a wrong
address → header names the right one.

## Notes
- **Hard rules (owner):** never commit a Sony SDK file, a `.env`, a credential, an SSH fingerprint, the camera's ID or the Mac's address; no Python; never
  change the camera's network/FTP settings, delete camera files or exported files, or kill any process; Capture One is quit only by the take-back (now
  also automatic, per the owner's decision, when the setting is on); live view out of scope; the suite never needs the camera or Capture One. Lab repo
  read-only. Default implementer (Codex). If the SDK is present locally, prove the SDK build compiles, then restore the stub; never commit build output.
- Book changelog entry 38 records the owner's decision.

## Review log

- 2026-11-09 implement r1 (codex): Added default-on automatic take-back once per plug-in, waiting header and off switch; moved camera polling out of the shared render loop; associated late exposure with an already-filed delivery; made output format saves fast without preview work; rechecked the editor launch gate after lane waits; read FTP state at connect/60 s and surfaced the Mac address/receiver port; cached the discovered Wi-Fi endpoint for direct connection before search. F16 cause: the app's camera tick shared a thread with blocking preview and Capture One work; the helper already flushed each stdout line. Assumptions: the SDK exposure callback has no camera filename, so late delivery association uses the oldest unassociated delivery in the ledger's five-minute window; a direct cached-IP failure, even if marked fatal, gets one discovery fallback; the Mac's LAN address comes from the default-route interface; FTP status value 2 is connected, per the lab fixture. Deviations: C++ test target links the stub adapter to exercise cached endpoint creation; existing P43 camera-settings assertion and P54 manual take-back fixture changed for the new default-on setting. Rust, UI, lint, typecheck and sidecar stamps are in the implementer handoff.
- 2026-11-09 review r1 (opus, pipeline-reviewer): **CHANGES_REQUESTED**. Perimeter: `763b997` commits exactly the 34 declared paths. Evidence (reviewer run, clean tree `e450d91`, code = `763b997`): `cargo test --workspace` 849/0 (56 groups), `npm test` 412/413 with only the known `p33_split_grid` C4(d) intermittent (7/7 alone), `npm run typecheck`, clippy `-D warnings`, `cargo fmt --check`, `tools/sidecar/build.sh --stub --test` 28/0, all green. Variation: C2(a) and C4(a) run unchanged on the pre-P57 tree `edb6f83` in a scratch worktree (testkit `with_camera_script_running` back-ported) → both **pass**; re-run with the fake's `call_delay_ms`/`delay_document_info_ms` at 1.5–3 s → both still pass. Verified correct by reading: auto take-back trigger (setting on, plug-in seen by `usb.arrived`/`usb.initial`, holder = Capture One via `ptpcamerad`, once per plug-in, retry only after a `TakeBackBlocked` race, reset on removal, helper restarts do not reset); waiting sentence `Waiting to take the cable: <reason>` re-evaluated every 20 ms tick and cleared on start/off/unplug; setting off → no trigger, manual availability unchanged; camera tick moved to its own 20 ms thread (old loop ran it after follow/jobs/previews ticks that can block on Capture One); late exposure linked to the oldest done, non-duplicate, unassociated delivery within 300 s (`fifo`, consistent with MC-33 rule 3 mirrored); output-format and auto-take-back saves take the no-rebuild path (no Capture One or render call possible), runs capture their format at start for cache lookup, payload and manifest; in-lane launch-gate re-check closes the queued-launch race behind F17 (non-launching JXA scripts guard on `running()`); FTP state read at link change and every 60 s, error → exact plan sentence with default-route LAN address and receiver port, cleared by status 2 or a newer wireless delivery; cached Wi-Fi endpoint passed as `--ip/--mac` (MAC = discovered camera id), direct connect first, failure (even fatal) forgets it and falls back to discovery. H1–H8 hold; no assumption contradicts the book (changelog 38 supersedes 10E's click-only line). Findings: **F1 (B2)** C4(a) test lacks the row's input (Capture One busy) and passes on the pre-P57 code that hung and re-rendered in run 2; **F2 (B2)** C2(a) test never occupies the shared background loop, the cause the handoff found, and passes on the pre-P57 code. Both route production, test files only. 7 backlog notes (mid-export output change renders the rest of a running export at the new format; late-exposure match can claim an orphan delivery and hide a real "not received"; C5's in-lane race not reproduced; book/registry drift; FTP warning on the cable; silent automatic take-back failure + 20 ms precondition polling; FtpUnreachable ties HeldByOtherProgram in rank).
- 2026-11-09 fix 1 r2 (codex): Resolved F1 by enabling startup and a delayed fake Capture One document call while a preview is queued; settings save stays under one second and exactly one preview process runs when the queue resumes. Resolved F2 by waiting until startup's fake document call is active before injecting the one helper line; camera status reaches Wireless within 200 ms while the shared loop remains blocked. Evidence on pre-P57 `edb6f83` in a scratch worktree with the startup/delay fixture back-ported and isolated C2(a)/C4(a) cases: C2(a) fails at the 200 ms status assertion (`line took 200.369917ms`); C4(a) fails at the ≤1 s settings-save assertion (observed elapsed >1 s). Both cases pass on the P57 fix. No production files changed. Full Rust workspace 849/0, UI 413/413, sidecar stub 28/0, clippy, fmt, and typecheck green.
