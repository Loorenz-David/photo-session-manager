---
phase: P48_camera_ownership
title: Camera ownership guard — naming the program that holds the camera, the start order (USB session before Capture One is launched), the one editor-launch gate, "Take the camera back" on the photographer's click (save, quit, usb-retry, relaunch, ensure), session activation independent of Capture One, the idle-sleep assertion while on the cable
difficulty: high
projection: yes
depends_on: [P46_direct_camera_adapter]
---
# P48 — Camera ownership guard (MC-34; M39, M40)

## Goal
Session Manager keeps the camera and, when it cannot, names who has it and how to fix it (10E MC-34 with inventory additions **R-I30** holder naming,
**R-I31** start order, **R-I32** the editor-launch gate and the take-back sequence, **R-I33** activation independent of Capture One; invariants M39
"Capture One is quit only on the photographer's click", "no other process is ever killed", M40 "while Take the camera back runs, nothing but its own
step 5 launches Capture One; at app start … Capture One is launched only after one of the start-order conditions holds"). Also R-I23's idle-sleep
assertion while a USB session is open, and the backend of "Test camera connection".
**Not in this phase:** the buttons and sentences on screen (P51; the backend returns them), the current-collection switch and style (P49), the automatic
take-back (a later owner decision), any hardware run (P52 row "Take the camera back").

## Read first
Book: `10E_direct_camera_adapter.md` MC-34 (whole incl. **R-I30, R-I31, R-I32, R-I33**), MC-31 **R-I23** (sleep assertion; `usb-retry`), §Invariants
M39, M40 · `10C_capture_one_setup_by_the_app.md` MC-26 step 1 (the gate; start order) · `10A_capture_one_contracts.md` MC-18 step 5 (direct adapter
clause) · `14_measurement_ledger.md` M39, M40 · evidence `sony_camera_lab.md` C1, §V5 (USB contenders), **§V6** (SDK first keeps the camera; release
loses it within 1 s; re-plug lost within 28 ms) · process `flow.md` §4, §6, §9 · master plan §3, §6.4 (`CaptureOneScripting::quit`, ports `ProcessList`,
`SleepInhibitor`), §6.6 (`camera_held`, `editor_not_following`, `editor_launch_deferred`, `take_back_blocked`, `take_back_failed`), §6.7
(`camera_take_back`, `camera_test_connection`), **§6.11** (Constants `START_ORDER_WAIT_MS`, `TAKE_BACK_*`; Direct adapter `HolderReport`, `name_programs`,
`TakeBackService`, `StartOrder`, `LaunchGate`), §7.5 (`FakeProcessList`, `FakeSleepInhibitor`, fake `quit`).
Code at `41bef14` (+ P42–P46 names):
- `crates/integrations/src/editing/capture_one/gate.rs:1–140` (`GatedScripting::call` `:40–66` — the gate check goes here; every op is a method `:68–139`),
  `crates/integrations/src/editing/capture_one/provision.rs:66–130` (`ensure_provision`: `editor_info` then `launch` when not running `:75–100`; the
  wait loop), `:323–339` (`front_provision_document`), `crates/integrations/src/editing/capture_one/adapter.rs:28–39` (`CaptureOneAdapter` fields; the
  `renders` registry → `renders_in_flight()`), `:122–127` (`ensure`, `bring_document_to_front`).
- `crates/core/src/services/sessions.rs:48–77` (`activate`: ensure → front → `set_capture_folder` → **only then** `set_active_session_id` — reversed for
  the direct adapter), `:79–92` (`deactivate`), `crates/core/src/services/editing_startup.rs:14–65` (`run`: ensure, inspect, front, folder — step 5 of
  the take-back reuses it), `crates/core/src/services/camera.rs` (P42/P46 version), `crates/core/src/services/health.rs`.
- `crates/integrations/src/lib.rs` (`AppHandles::health_check` `:194–223` calls `EditingStartup::run` — a launch path; the startup thread `:565–582`;
  `scripting` vs the gated lanes `:97–99`, `:337–338`).
- `apps/desktop/src-tauri/src/lib.rs` (`health_get` → `health_check`), `apps/desktop/src-tauri/src/commands/{camera.rs, health.rs}`.
- `crates/testkit/src/fake_capture_one.rs:203–235` (`editor_info`, `launch`; `running_after_polls`), `:393–397` (`save`), `:468–485` (`find_image`);
  `crates/testkit/tests/p24_capture_one_provisioning.rs` and `p25_tidy_up_after_provisioning.rs` (activation rows that must keep passing for the
  Capture One adapter).
- Lab (read-only): `sdk/src/diagnostics.cpp:11–23` (`competing_processes`: `pgrep` of fixed names — ours reads one `ps` listing), `docs/session-manager-handoff.md` §8 row 4.
- Dictionary: `docs/reference/capture_one_16.8.6.sdef:18` (`quit` command), `:70` (`frontmost`).

## Files expected to change
Integrations (new): `crates/integrations/src/editing/capture_one/launch_gate.rs`, `crates/integrations/src/editing/capture_one/jxa/quit.js`,
`crates/integrations/src/camera/direct/{ownership.rs, take_back.rs, start_order.rs}`. Integrations (changed): `crates/integrations/src/editing/capture_one/
{gate.rs, scripting.rs, jxa.rs, adapter.rs, provision.rs, mod.rs}` (`quit`, `SCRIPTS` 19 → 20, `renders_in_flight`), `crates/integrations/src/camera/
{runtime.rs, direct/adapter.rs, direct/mod.rs}`, `crates/integrations/src/lib.rs`.
Core (changed): `crates/core/src/ports/{editing.rs (quit, renders_in_flight), system.rs (new: ProcessList, SleepInhibitor), mod.rs}`,
`crates/core/src/services/{sessions.rs, editing_startup.rs, camera.rs}`, `crates/core/src/error.rs`.
Infra (new): `crates/infra/src/{processes.rs, sleep.rs}`. Infra (changed): `crates/infra/src/lib.rs`, `crates/infra/Cargo.toml` (IOKit link via `#[link(name
= "IOKit", kind = "framework")]`).
Desktop (changed): `apps/desktop/src-tauri/src/commands/camera.rs` (`camera_take_back`, `camera_test_connection`), `apps/desktop/src-tauri/src/lib.rs`,
`apps/desktop/src-tauri/src/commands/items.rs` (codes), `apps/desktop/src/services/bindings.ts` (regenerated).
Testkit (new): `crates/testkit/src/{fake_processes.rs, fake_sleep.rs}`. Testkit (changed): `crates/testkit/src/{lib.rs, test_app.rs, fake_capture_one.rs}`
(`quit`, `quit_after_polls`, `fail_save`, `frontmost` seed), `crates/testkit/src/fixtures/sidecar_scripts/{take_back.txt, claimed_wifi.txt}`. Tests (new):
`crates/testkit/tests/p48_camera_ownership.rs`.
Estimated: 9 new source paths, 21 changed, 1 new test, 2 scripts — 33 paths.

## Core mechanisms (unit tests allowed)
`direct/ownership.rs::name_programs` (rows C1(a)–(d)); `direct/take_back.rs::preconditions(&TakeBackInputs) -> Result<(), TakeBackBlocked>` (C4). Everything
else is a behaviour row.

## The ownership contract
- **Holder naming (R-I30).** `KNOWN_CAMERA_PROGRAMS: [&str; 6] = ["Capture One", "Imaging Edge Desktop", "Image Capture", "Photos", "Adobe Lightroom
  Classic", "Adobe Lightroom"]`. `name_programs(owner_process: &str, running: &[String]) -> HolderReport`: `owner == "ptpcamerad"` → `programs` = the known
  names that are running, in the list's order (Capture One first); sentence: Capture One present → `Capture One is holding the camera's USB connection —
  Take the camera back`; otherwise non-empty → `programs that can take the camera: <names joined ", "> — <guide of the first>` with guides `Imaging Edge
  Desktop` → `quit it and stop it opening at login`, `Image Capture` | `Photos` → `set "Connecting this camera opens" to "No application"`, Lightroom →
  `quit it`; empty → `The system camera service is holding the camera: close apps that use cameras`. Any other owner → `programs: []`, sentence `<owner>
  is using the camera: close it or stop it from using cameras`. `holder` (for `NeedsAction.holder` and the dot) = `programs[0]` else the owner process
  name. The adapter calls `process_list.running()` once per `usb.claimed_by` event and on `camera_test_connection`. `PsProcessList` runs `/bin/ps -axo
  comm=` and maps each line to the last path component (so `/Applications/Capture One.app/Contents/MacOS/Capture One` → `Capture One`).
- **Start order (R-I31, M40).** `StartOrder` on the runtime: states `Waiting | Satisfied(reason)`; the direct adapter sets `Satisfied` on the first of
  `ConnectionReady { usb }`, `UsbInitial { present: false }`, `UsbFallbackToWifi`, `SdkUnavailable`, supervisor exit; `StartOrder::wait(clock, deadline =
  START_ORDER_WAIT_MS)` polls every 50 ms and returns `Satisfied(Timeout)` at the deadline. It is `Satisfied(NotNeeded)` from the start when no support
  package is installed. `GatedScripting::launch` (user lane) calls `start_order.wait()` **before** `inner.launch()` **only if** Capture One is not
  running (the caller, `ensure_provision`, already knows; pass through a `launch_after_start_order()` method on the gate) — once satisfied it never waits
  again in the process.
- **Launch gate (R-I32, M40).** `LaunchGate { held: AtomicBool, permit: thread-local }`: `hold() -> GateGuard` (released on drop); `GateGuard::permit<T>(f)`
  runs `f` with the current thread permitted. `GatedScripting::call` for `launch`, `front_document`, `make_document`, `open_document`, `ensure_recipe`,
  `ensure_capture_style` (every script that does not check `running()` first) returns `Err(Error::EditorLaunchDeferred)` **without** running the script
  when `gate.is_held() && !permitted_here()`. `ensure_provision` maps that error to `Ok(ProvisionReport::deferred())` (a new `deferred: bool` field; no
  health item is set or cleared for a deferred ensure). Every launch path goes through this: `EditingStartup::run` (startup thread, `health_check`, the
  settings rebuild's startup thread), `SessionsService::activate`, and the scripts themselves.
- **"Take the camera back" (R-I32).** `TakeBackService::run(&self) -> Result<TakeBackReport { completed_steps: u8, camera_held: bool }>`:
  preconditions (`TakeBackInputs { uploads_in_progress, expected_transfer, youngest_expecting_exposure_age_ms, jobs_running: {render_master, render_output,
  publish_step}, renders_in_flight, holder_is_capture_one }`) → `TakeBackBlocked { reason }` with the §6.6 reasons, checked in that order; then: **1** `gate.hold()`;
  **2** `scripting.save(doc)` (the ungated `App.scripting`) — `Err` → `TakeBackFailed { step: 2, cause }`, gate released, Capture One untouched; **3**
  `scripting.quit()` then poll `editor_info().running` every 500 ms up to `TAKE_BACK_QUIT_WAIT_MS` — still running → `TakeBackFailed { step: 3, cause:
  "Capture One did not quit (a dialog may be open)" }`, **never killed**; **4** if `editor_info().running` now (the photographer started it) → `TakeBackFailed
  { step: 4, cause: "Capture One was started meanwhile" }`; else `client.send(UsbRetry)` and wait up to `TAKE_BACK_READY_WAIT_MS` for `ConnectionReady { usb }`;
  a `UsbClaimedBy` within the first `TAKE_BACK_FREE_WAIT_MS` → resend `UsbRetry` after 1 s (the interface frees "immediately" after Capture One quits,
  V6); timeout → `camera_held = false`, continue; **5** `gate.permit(|| EditingStartup::run(..))` (launch, ensure, front, folder — P49 adds the current
  collection) under the still-held gate; its `Err` → `TakeBackFailed { step: 5 }`; **6** release (the guard drops). The service runs on the caller's
  `spawn_blocking` thread; a second `run` while one is in progress → `TakeBackBlocked { reason: "already running" }`.
- **Activation independent of Capture One (R-I33).** When `camera.capabilities().depends_on_editing_provider` is false: `activate` stores
  `active_session_id` **first**, then ensure → front → `set_capture_folder`; any error there sets `editor_not_following` (`Capture One will not show this
  session's new photos until it is reachable`) and returns `Ok(())`; `deactivate` likewise. `EditingStartup::run` clears `editor_not_following` after a
  successful folder step. With the Capture One camera adapter both behave exactly as today (p24/p25 rows unchanged).
- **Sleep (R-I23).** The adapter calls `sleep.hold("Session Manager: camera connected by cable")` on `ConnectionReady { usb }` and `release()` on
  `SessionClosed | SessionLost` from USB, and on shutdown. `IokitSleepInhibitor`: `IOPMAssertionCreateWithName(kIOPMAssertionTypePreventUserIdleSystemSleep,
  kIOPMAssertionLevelOn, name, &id)` / `IOPMAssertionRelease(id)`.
- **`camera_test_connection`** → `TakeBackService::holder_report()`: the adapter's current `HeldByOtherProgram` holder (or none) + `name_programs` over a
  fresh `process_list.running()` → `{ holder, programs, sentence }`.

## Ordered tasks
0. **Registry first** (§6.11): ports `ProcessList`, `SleepInhibitor`; `CaptureOneScripting::quit`; errors; constants; `ProvisionReport.deferred`.
1. `ownership.rs` + unit tests; `processes.rs`, `sleep.rs`, fakes.
2. `launch_gate.rs`, `gate.rs` changes, `start_order.rs`, `provision.rs` (deferred, start-order wait at the launch step), `quit.js` + `jxa.rs`.
3. `sessions.rs`/`editing_startup.rs` for the direct adapter (R-I33); `editor_not_following`.
4. `take_back.rs`; the adapter's sleep hold/release; runtime wiring (the take-back service needs: the gate, the ungated scripting, the editing provider,
   the store, the receiver handle, the ledger, the client, the process list — all reachable from the runtime binding).
5. Commands `camera_take_back`, `camera_test_connection`; bindings.
6. Testkit fakes and scripts; tests `p48_camera_ownership.rs`; suite, clippy, fmt, typecheck.

## Criteria (8 criteria, 38 rows; 0 `[rule]` rows)
Fixtures: `a = TestApp::with_camera_script(<script>)` with `a.fake_processes.set(vec![…])`; `F` = the fake Capture One (`running` true, document open);
`ops()` = `F.state.ops` (cleared where stated); `st()` the camera status; `H` health; `TB = a.take_back()` (`TakeBackService`); `rec()` the sidecar
recording. Scripts: `claimed.txt` (P46), `claimed_wifi.txt` (`ready_wifi` lines then `usb.claimed_by {owners:"pid 512, ptpcamerad"}` + `connect.failed usb
0x8208`), `take_back.txt` (`claimed_wifi.txt` then `await usb-retry`, `connection.ready {transport:"usb", model:"ILCE-7RM5", store_destination:"verified"}`).
Waits in the services poll `a.clock`; tests advance it from the test thread while the service runs on another.

| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | `name_programs` (unit) | `("ptpcamerad", ["Finder", "Imaging Edge Desktop", "Capture One"])` | `programs == ["Capture One", "Imaging Edge Desktop"]`, `holder == Some("Capture One")`, `sentence == "Capture One is holding the camera's USB connection — Take the camera back"` | R-I30 ("Capture One first"); MC-34 §Take it back |
| C1(b) | (unit) | `("ptpcamerad", ["Photos"])`; `("ptpcamerad", ["Imaging Edge Desktop", "Adobe Lightroom"])` | `programs that can take the camera: Photos — set "Connecting this camera opens" to "No application"`; `programs that can take the camera: Imaging Edge Desktop, Adobe Lightroom — quit it and stop it opening at login` | MC-34 §Guide; R-I30 ("programs that can take the camera: …") |
| C1(c) | (unit) | `("ptpcamerad", [])` | `programs == []`, `holder == Some("ptpcamerad")`, the system-service sentence | R-I30 |
| C1(d) | (unit) | `("SomeApp", ["Capture One"])` | `programs == []`, `holder == Some("SomeApp")`, `SomeApp is using the camera: close it or stop it from using cameras` | MC-34 §Guide ("unknown programs") |
| C1(e) | status and health with a holder, Wi-Fi continues | `claimed_wifi.txt`; `fake_processes = ["Capture One"]`; ticks; then `a.capture()` | `st() == NeedsAction { HeldByOtherProgram, holder: Some("Capture One") }`; `H["camera_held"].detail == C1(a)'s sentence`; `capture()` → `Ok` and the `capture` line is sent (shooting over Wi-Fi continues) | MC-34 §Detect, §Guide ("Wi-Fi continues"); R-I37 (`NeedsAction` wins while connected on the other link) |
| C1(f) | cleared when we hold the interface | after C1(e): inject `connection.ready {transport:"usb"}` | `st() == Connected { link: Cable }`; `camera_held` gone | MC-34 ("The health screen keeps the item until the holder is gone") |
| C2(a) | start order: launch waits for `connection.ready usb` | script `sleep 1500` then `ready_usb`; `F.running = false`; `a.app.startup()` on a thread; the test ticks the runtime every 50 ms | while the ready line has not been processed, `ops()` has no `launch`; after it, `startup()` returns and `ops()` has `launch` **after** `editor_info` | R-I31; M40 ("Capture One is launched only after one of MC-34's start-order conditions holds") |
| C2(b) | `usb.initial { present: false }` | script `usb.initial {present:false}` first; `F.running = false`; `startup()` | returns with `launch` in `ops()` before any clock advance beyond 100 ms | R-I31 |
| C2(c) | fallback to Wi-Fi | script `usb.fallback_to_wifi {usb_failures:3, usb_retry:"on next USB re-plug"}` | launch proceeds at once | R-I31 |
| C2(d) | sidecar unavailable | script `exit 4` (after `sdk.unavailable`) | launch proceeds at once | R-I31 ("the sidecar being unavailable or crashing") |
| C2(e) | the 15 s bound | script with no events (`sleep 100000`); the test advances `a.clock` by 15 s | `startup()` returns after the advance with `launch` in `ops()`; before the advance it has not returned | R-I31 ("15 s") |
| C2(f) | Capture One already running | `F.running = true`; script `sleep 100000` | `startup()` returns without waiting (no clock advance); `ops()` has no `launch` | R-I31 ("If Capture One is already running at app start, nothing is waited for") |
| C2(g) | satisfied once | after C2(a): `F.running = false` again (the test resets the fake); `a.app.health_check()` | returns at once with a new `launch` (no second wait) | R-I31 (start order is an app-start rule) |
| C3(a) | gate defers `ensure` at launch | `g = a.launch_gate().hold()`; `F.running = false`; `a.app.editing.ensure(&provision)` | `Ok(ProvisionReport { deferred: true, .. })`; `ops() == ["editor_info"]`; no health item changed | R-I32 ("While Take the camera back holds the gate, these return deferred without running a script"); M40 |
| C3(b) | `front_document` deferred | gate held; `a.app.editing.bring_document_to_front()` | `Err(EditorLaunchDeferred)`; `ops()` empty | R-I32 |
| C3(c) | `make_document` deferred | gate held; `F.open_documents.clear()` and the document folder removed so ensure would create it; `ensure` | `deferred: true`; no `make_document` op | R-I32 |
| C3(d) | `open_document` deferred | gate held; `F.open_documents.clear()` (folder exists); `ensure` | `deferred: true`; no `open_document` op | R-I32 |
| C3(e) | `ensure_recipe` deferred | gate held; `F.recipes.clear()`; `ensure` | `deferred: true`; no `ensure_recipe` op | R-I32 |
| C3(f) | `ensure_capture_style` deferred | gate held; `style_name` set; `ensure` | `deferred: true`; no `ensure_capture_style` op | R-I32 |
| C3(g) | a health check while held never launches | gate held; `F.running = false`; `a.app.health_check()` | `Ok`; `ops()` has no `launch`; the report has no new `editor_not_running` item | M40 ("nothing but its own step 5 launches Capture One") |
| C3(h) | released | `drop(g)`; `ensure` | `deferred: false`; `launch` in `ops()` | R-I32 step 6 |
| C4(a) | blocked: upload in progress (unit + boundary) | `TakeBackInputs { uploads_in_progress: 1, .. }`; boundary: `a.ftp()` holds an open `STOR`; `TB.run()` | `Err(TakeBackBlocked { reason: "a file is still arriving" })`; `ops()` has no `save` | R-I32 ("enabled only when no upload or download is in progress") |
| C4(b) | blocked: a photo expected | exposure 5 s old `expecting` → `"a photo is still expected"`; exposure 11 s old → not this reason | as stated | R-I32 ("no exposure younger than 10 s is still expecting a file") |
| C4(c) | blocked: export running | a `render_master` job `running` (inserted by the test) → `"an export is running"` | as stated | R-I32 ("no export or publish run is active") |
| C4(d) | blocked: publish running | a `publish_step` job `running` → `"a publish is running"` | as stated | R-I32 |
| C4(e) | blocked: a preview rendering | `renders_in_flight == 1` (a render issued to the fake, not completed) → `"a preview is rendering"` | as stated | R-I32 ("no render is in flight") |
| C4(f) | blocked: Capture One is not the holder | `ready_usb` (we hold the camera) or `claimed` with `fake_processes = ["Photos"]` | `"Capture One is not holding the camera"` | MC-34 ("If the USB interface is held by the system camera service on Capture One's behalf") |
| C5(a) | the sequence succeeds | `take_back.txt`; `fake_processes = ["Capture One"]`; `activate(s)` before; `ops()` cleared; `r = TB.run()` (test thread ticks the runtime and advances the clock in 500 ms steps) | `r == Ok(TakeBackReport { completed_steps: 5, camera_held: true })`; `ops()` in order: `save`, `quit`, (`editor_info`…), `launch`, `editor_info`…, `set_captures:<root>/<key(s)>`; `rec().stdin` contains one `usb-retry`; `st() == Connected { link: Cable }`; after `r` the gate is released (`ensure` → `deferred: false`) | R-I32 steps 1–6; MC-34 §Take it back ("saves the Capture One document, quits Capture One, opens the USB session, relaunches Capture One and reopens the document"); M39, M40 |
| C5(b) | save fails → nothing else happens | `F.fail_save = true`; `TB.run()` | `Err(TakeBackFailed { step: 2, .. })`; `ops() == ["save"]`; `F.running` still true; gate released | R-I32 ("`save` the document. On failure, stop with Capture One untouched") |
| C5(c) | quit does not finish → never killed | `F.quit_after_polls = None` (never stops); clock advanced 21 s in steps | `Err(TakeBackFailed { step: 3, .. })`; `F.running == true`; no `launch`; no `usb-retry` sent; `a.fake_processes` untouched (no kill API exists) | R-I32 step 3 ("If it is still running (a dialog is open), stop; it is never killed"); M39 ("No other process is ever killed") |
| C5(d) | step 4 times out, step 5 still runs | script `claimed_wifi.txt` + `await usb-retry` + `sleep 100000` (never ready); clock + 16 s | `Ok(TakeBackReport { completed_steps: 5, camera_held: false })`; `launch` in `ops()`; `st()` still `NeedsAction { HeldByOtherProgram }` | R-I32 step 5 ("Whatever step 4 gave, launch Capture One") |
| C5(e) | the photographer started Capture One meanwhile | `F.quit_after_polls = Some(1)` then the test sets `F.running = true` again right after the `quit` op (before the service's step-4 check) | `Err(TakeBackFailed { step: 4, cause: "Capture One was started meanwhile" })`; no `launch` by us; no `usb-retry` | R-I32 ("If Capture One starts during steps 3–4 and the app did not start it … stop and report") |
| C5(f) | quit only on click | `claimed_wifi.txt`; 30 s of ticks with the holder present and `H` showing `camera_held` | `ops()` has no `quit` | M39 ("Capture One is quit only on the photographer's click"); MC-34 ("It runs only on the photographer's click") |
| C5(g) | `camera_take_back` command (mock webview) | C5(a)'s fixture → `{"completed_steps":5,"camera_held":true}`; C4(a)'s fixture → error `{"code":"take_back_blocked","data":{"reason":"a file is still arriving"}}` | as stated | master plan §6.7 |
| C6(a) | activation stores first, Capture One follows or not | direct adapter active (`ready_usb`); `F.fail_next_set_captures = true`; `a.sessions().activate(s)` | `Ok(())`; `store.active_session_id() == Some(s)`; `H.has("editor_not_following")` with the §6.6 detail; `st()` unchanged (`Connected`); `a.capture()` → `Ok` | R-I33; 10A MC-18 step 5 (direct adapter: "the camera status and capture are not touched") |
| C6(b) | retried at the next ensure | after C6(a): `a.app.startup()` (folder step succeeds now) | `editor_not_following` cleared; `set_captures:<key(s)>` in `ops()`; with the Capture One camera adapter (`with_fake_document`) `activate` with `fail_next_set_captures` still returns `Err(CaptureFolderNotSet)` and does **not** store the session (p24/p25 rows unchanged) | R-I33 ("retried at the next ensure"); M25 (Capture One adapter only) |
| C7(a) | sleep assertion follows the USB session | `ready_usb` → `a.fake_sleep.held() == Some("Session Manager: camera connected by cable")`; inject `session.lost usb` → `None`; `ready_wifi` → `None` | as stated | R-I23 ("While a USB session is open, the app holds a macOS idle-sleep assertion") |
| C8(a) | `camera_test_connection` (mock webview) | C1(e)'s fixture → `{"holder":"Capture One","programs":["Capture One"],"sentence":"Capture One is holding …"}`; `ready_usb` fixture with `fake_processes = ["Photos"]` → `{"holder":null,"programs":["Photos"],"sentence":null}` (lists who **could** take it) | as stated | MC-34 §Setup check ("lists every program that could take the camera"); master plan §6.7 |

## Delegated choices
The thread-local permit's implementation (`thread_local!` `Cell<bool>` or a token in the guard); whether `StartOrder` lives on the runtime or the adapter;
`PsProcessList`'s dedupe and ordering; how the take-back thread advances in tests (recommended: every wait polls `clock.mono()` and sleeps 10 ms real time);
`TakeBackReport`'s extra fields; `renders_in_flight` as a port method or an adapter downcast (recommended: `EditingProvider::renders_in_flight() -> usize`).

## Manual try-out
(Hardware-free; the real take-back is P52's row.) 1. Start the app with Capture One already open: no change. 2. Health → the camera row behaves as before;
"Retry checks" still works. 3. Open a session, switch sessions: Capture One follows as before. 4. Nothing else visible; the "Take the camera back" button
arrives with P51.

## Notes
- **Hard rules (owner):** never commit a Sony SDK file, the lab's `sdk_shim` headers, a `.env`, a credential or an SSH fingerprint; no Python; never change the
  camera's network/FTP settings, delete a camera file, or **kill `ptpcamerad` or any process** (Capture One is asked to `quit` by AppleScript, never
  killed; C5(c)); the only process we kill is our own sidecar child after `quit` (P42); live view out of scope; the suite never needs the camera. No `implementer: fable`.
- **Planner readings (reversible):** (1) step 4's "interface free" is inferred from `usb-retry` succeeding; a `usb.claimed_by` during the 5 s free-wait
  triggers one more `usb-retry` per second (V6: the interface frees immediately when Capture One quits). (2) A deferred `ensure` is `Ok` with `deferred:
  true` so callers (startup thread, health check) need no change. (3) `editor_not_following` is one item for both the folder and (P49) the collection.
  (4) The holder for `NeedsAction.holder` is the first named program, else the process.
- **Hazards:** `GatedScripting` is **also** the background lane — the gate check applies to both lanes. `App.scripting` (ungated) is what the take-back
  uses for `save`/`quit`; the user lane for everything else under `permit`. `ensure_provision` holds `provision_lock` — the take-back's step 5 runs it on
  the take-back thread while a health check may wait on the lock; fine, but never call `ensure` from inside the lock. The `quit` script addresses the
  application, not the document (`Application("Capture One").quit()`); the fake's `quit` must flip `running` only after `quit_after_polls` `editor_info`
  calls so C5(c)/(e) are constructible. `sessions.rs` must branch on the **provider's** capability, not on a settings flag (R-I33 is about the adapter).
- **Projection** is on: the inventory's #7 and #8 mechanisms (launch paths, activation coupling); the projectionist enumerates every call site that can
  launch Capture One at `41bef14` and checks each goes through the gate, and that C2/C5's threaded fixtures are constructible with the manual clock.
- Registry additions made by the planner in master plan §6.4, §6.6, §6.7, §6.11, §7.5. No intention-book change.
- Estimated size: ~33 paths, ~2 300 lines (≈ 1 000 code, ≈ 1 300 tests + fakes).

## Review log

