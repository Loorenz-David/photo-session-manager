---
phase: P48_camera_ownership
title: Camera ownership guard — naming the program that holds the camera, the start order (USB session before Capture One is launched), the one editor-launch gate, "Take the camera back" on the photographer's click (save, quit, usb-retry, relaunch, ensure), session activation independent of Capture One, the idle-sleep assertion while on the cable
difficulty: high
projection: done
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
take-back (a later owner decision), any hardware run (P52 row "Take the camera back"), R-I30's best-effort ImageCaptureCore scan (deferred until spike
S5 answers; see Notes).

## Read first
Book: `10E_direct_camera_adapter.md` MC-34 (whole incl. **R-I30, R-I31, R-I32, R-I33**), MC-31 **R-I23** (sleep assertion; `usb-retry`), §Invariants
M39, M40 · `10C_capture_one_setup_by_the_app.md` MC-26 step 1 (the gate; start order) · `10A_capture_one_contracts.md` MC-18 step 5 (direct adapter
clause) · `14_measurement_ledger.md` M39, M40 · evidence `sony_camera_lab.md` C1, §V5 (USB contenders), **§V6** (SDK first keeps the camera; release
loses it within 1 s; re-plug lost within 28 ms) · process `flow.md` §4, §6, §9 · master plan §3, §6.4 (`CaptureOneScripting::quit`, ports `ProcessList`,
`SleepInhibitor`), §6.6 (`camera_held`, `editor_not_following`, `editor_launch_deferred`, `take_back_blocked`, `take_back_failed`; `CameraStatus::NeedsAction.message`), §6.7
(`camera_take_back`, `camera_test_connection`), **§6.11** (Constants `START_ORDER_WAIT_MS`, `TAKE_BACK_*`; Mechanisms `StatusInputs`; Direct adapter `HolderReport`, `name_programs`,
`TakeBackService`, `StartOrder`, `LaunchGate`), §7.5 (`FakeProcessList`, `FakeSleepInhibitor`, fake `quit`, `restart_after_quit`, `fail_save`).
Projection: `handoffs/reviewer/handoff_P48_camera_ownership_projection_r0.md` (its launch-path enumeration at `33b9eca` is the authority for "every
launch path"; the ledger L1–L25 is folded into this plan, 2026-11-07).
Code at `5eba743` (+ P42–P46 names; content identical to the projection's `33b9eca`):
- `crates/integrations/src/editing/capture_one/gate.rs:1–140` (`GatedScripting::pair` `:25`, `::call` `:42–66` — the gate check goes here; every op is a
  method `:71–139`), `crates/integrations/src/editing/capture_one/provision.rs:71–130` (`ensure_provision`: `editor_info` `:76`, `launch` when not running
  `:83`, the launch wait loop `:95–100` on a real `Instant` with `EDITOR_WAIT = 45 s`, `lib.rs:74`), the per-step error handlers that write health
  (`:83–94` launch, `:143–152` make, `:188–201` open, `:243–253` recipe, `:285–291` style), `:305–322` (`front_provision_document`; `.ok()` swallows the
  error `:309`), `crates/integrations/src/editing/capture_one/adapter.rs:28–39` (`CaptureOneAdapter` fields; `user` is the gated user lane `:29`, `:52`;
  the `renders` registry → `renders_in_flight()`), `:122–127` (`ensure`, `bring_document_to_front`), `:155–162` (`inspect_document` maps a scripting error
  to `DocumentNotOpen`), `:177–191` (`set_capture_folder` sets and clears `capture_folder_not_set` itself).
- `crates/core/src/services/sessions.rs:10–17` (fields: no camera), `:51–68` (`activate`: ensure → front → `set_capture_folder` → **only then**
  `set_active_session_id` — reversed for the direct adapter), `:70–82` (`deactivate`), `crates/core/src/services/editing_startup.rs:15–65` (`run`:
  ensure `:16`, inspect `:17–40`, front `:43`, folder `:56–63` — step 5 of the take-back reuses it), `crates/core/src/services/camera.rs` (P42/P46
  version), `crates/core/src/services/health.rs`, `crates/core/src/domain/editing.rs:18–22` (`ProvisionReport`), `crates/core/src/domain/camera.rs:100`
  (`CameraStatus::NeedsAction`), `crates/core/src/mechanisms/camera_status.rs:6–12` (`StatusInputs`), `:40–50` (`status_message`: today
  `"<holder> is holding the camera"`), `:53–63` (`health_items` → the `camera_held` detail).
- `crates/integrations/src/lib.rs` (`App` fields `scripting`/`user_scripting`/`background_scripting` `:101–103`; `AppHandles::health_check` `:203–222`
  calls `EditingStartup::run` `:205` then `CameraService::status` — a launch path; the startup thread `:632–648`; the lanes are created once in
  `open_with_adapter_bind` `:377–378` and carried through `SettingsSetContext.lanes` `:141` and `reopen` `:273–276`; `App::startup` `:796–804`),
  `crates/integrations/src/camera/runtime.rs:53–62` (`AppBinding` fields: no editing provider, scripting, provision inputs, document id or gate),
  `crates/integrations/src/camera/direct/adapter.rs:158–167` (`AdapterState::add/remove`), `:281–287` (`HelperEvent::Exited` resets the session),
  `:440–443` (`UsbClaimedBy` → `holder_from_owners`).
- `apps/desktop/src-tauri/src/lib.rs` (`health_get` → `health_check`; `TauriEvents::emit` `:311–315` maps `CameraStatusChanged` through
  `CameraStatusResponse::from`), `apps/desktop/src-tauri/src/commands/camera.rs:24–48` (`CameraStatusResponse::NeedsAction { reason, holder, message }`;
  `message` from `status_message` `:46`), `apps/desktop/src-tauri/src/commands/sessions.rs:412–413`, `:434–435` (`SessionsService` built with
  `with_editing` only), `apps/desktop/src-tauri/src/commands/items.rs:28–32` (`CommandError { code, message, data: BTreeMap<String, String> }` — every
  data value is a string).
- `crates/testkit/src/test_app.rs:31–37` (fields `app`, `clock`, `fake`), `:56–61` (`sessions()`), `:112` (`with_fake_document`), `:131`
  (`with_camera_script`); `crates/testkit/src/fake_capture_one.rs:26–31` (`running`, `running_after_polls`: default `None` `:90`, counts down only after
  `launch` `:212–220`), `:203–235` (`editor_info`, `launch`), `:360–369` (`document_info` ignores `running`), `:393–397` (`save`), `:398–407`
  (`set_captures`; `fail_next_set_captures` is one-shot), `:468–485` (`find_image`); `crates/testkit/tests/p46_direct_camera_adapter.rs:84–89` (`needs()`
  builds a `NeedsAction`), `:805–810` (`c7a_holder_reported` compares the held status by equality); `crates/testkit/tests/p24_capture_one_provisioning.rs`
  and `p25_tidy_up_after_provisioning.rs` (activation rows that must keep passing for the Capture One adapter).
- Lab (read-only): `sdk/src/diagnostics.cpp:11–23` (`competing_processes`: `pgrep` of fixed names — ours reads one `ps` listing), `docs/session-manager-handoff.md` §8 row 4;
  `native/camera-sidecar/src/usb_monitor_macos.cpp:203–212` (`usb.claimed_by` is emitted only with non-empty owners, on a connect attempt — there is no
  "cleared" event), `transport_manager.cpp:163–172` (`usb-retry` resets the failure counts and attempts now).
- Dictionary: `docs/reference/capture_one_16.8.6.sdef:18` (`quit` command).

## Files expected to change
Integrations (new): `crates/integrations/src/editing/capture_one/launch_gate.rs`, `crates/integrations/src/editing/capture_one/jxa/quit.js`,
`crates/integrations/src/camera/direct/{ownership.rs, take_back.rs, start_order.rs}`. Integrations (changed): `crates/integrations/src/editing/capture_one/
{gate.rs, scripting.rs, jxa.rs, adapter.rs, provision.rs, mod.rs}` (`quit`, `SCRIPTS` 19 → 20, `renders_in_flight`, the deferral returns), `crates/integrations/src/camera/
{runtime.rs, direct/adapter.rs, direct/mod.rs}` (`AppBinding` additions, `set_process_list`/`set_sleep_inhibitor`, holder report, sleep hold/release),
`crates/integrations/src/lib.rs` (gate + start order created in `open_with_adapter_bind`, carried through `SettingsSetContext`/`reopen`; `EditingStartup`
sites `:205`, `:639`, `:797` gain the camera; `App::user_lane()`).
Core (new): `crates/core/src/ports/system.rs` (`ProcessList`, `SleepInhibitor`). Core (changed): `crates/core/src/ports/{editing.rs (quit, renders_in_flight), mod.rs}`,
`crates/core/src/domain/{editing.rs (ProvisionReport.deferred, ::deferred()), camera.rs (NeedsAction.message)}`, `crates/core/src/mechanisms/camera_status.rs`
(`StatusInputs.needs_action` carries the sentence; `derive_status`/`health_items` pass it on), `crates/core/src/services/{sessions.rs (with_camera), editing_startup.rs (camera, deferral,
editor_not_following), camera.rs}`, `crates/core/src/error.rs`.
Infra (new): `crates/infra/src/{processes.rs, sleep.rs}` (`#[link(name = "IOKit", kind = "framework")]` lives in `sleep.rs`). Infra (changed): `crates/infra/src/lib.rs`,
`crates/infra/Cargo.toml` (only if the `core-foundation` dependency option is taken, see Delegated choices).
Desktop (changed): `apps/desktop/src-tauri/src/commands/camera.rs` (`camera_take_back`, `camera_test_connection`; the status `message` from the status),
`apps/desktop/src-tauri/src/commands/sessions.rs` (`session_activate`/`session_deactivate` pass `app.camera`), `apps/desktop/src-tauri/src/commands/items.rs` (codes),
`apps/desktop/src-tauri/src/lib.rs`, `apps/desktop/src/services/bindings.ts` (regenerated).
Testkit (new): `crates/testkit/src/{fake_processes.rs, fake_sleep.rs}`. Testkit (changed): `crates/testkit/src/{lib.rs, test_app.rs (fake_processes, fake_sleep, take_back(),
launch_gate(), user_lane()), fake_capture_one.rs (quit, quit_after_polls, restart_after_quit, fail_save)}`, `crates/testkit/src/fixtures/sidecar_scripts/{take_back.txt, claimed_wifi.txt}`.
Tests (changed): `crates/testkit/tests/p46_direct_camera_adapter.rs` (two sites only: `needs()` adds `message: None`; `c7a`'s expected held status carries
the R-I30 system-service sentence — the fake process list is empty there). Tests (new): `crates/testkit/tests/p48_camera_ownership.rs`.
Derived from the lists above: 10 new source paths, 30 changed (10 integrations, 9 core, 2 infra, 5 desktop, 3 testkit, 1 earlier test), 1 new test, 2 scripts — **43 paths**.

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
- **Where the sentence lives (projection L9).** The direct adapter keeps the last `HolderReport`. `StatusInputs.needs_action` entries become
  `(NeedsActionReason, Option<String> /* holder */, Option<String> /* message */)`; the adapter writes `(HeldByOtherProgram, report.holder,
  report.sentence)` on every `usb.claimed_by`. `CameraStatus::NeedsAction` gains `message: Option<String>` (serialized snake_case; `derive_status` copies it;
  every other reason carries `None`). `health_items` uses `message` when `Some`, else `status_message(reason, holder)`; the desktop's
  `CameraStatusResponse` (`commands/camera.rs:44–48`, used by `camera_status`/`camera_retry` and by `TauriEvents::emit`) does the same, so the health
  detail and the command's `message` are one sentence. `status_message`'s `HeldByOtherProgram` arm stays as the fallback for a holder without a report.
- **Start order (R-I31, M40).** `StartOrder` is **process-wide**: created in `open_with_adapter_bind` beside the lanes, passed to `GatedScripting::pair`,
  reachable from the direct adapter and the take-back service, never rebuilt by `App::build` (it rides `SettingsSetContext`/`reopen` like the lanes).
  States `Waiting | Satisfied(reason)`; the direct adapter sets `Satisfied` on the first of `ConnectionReady { usb }`, `UsbInitial { present: false }`,
  `UsbFallbackToWifi`, `SdkUnavailable`, supervisor exit; **the first `ensure_provision` whose `editor_info` reports `running` sets
  `Satisfied(AlreadyRunning)`** (L4b: Capture One already open at app start satisfies the rule for the rest of the process). `StartOrder::wait(clock)` polls
  every 50 ms and returns `Satisfied(Timeout)` when `START_ORDER_WAIT_MS` have passed **since the first `wait()` call** (L4c). It is `Satisfied(NotNeeded)`
  from the start when no support package is installed. Once satisfied it never waits again in the process.
  **Where the wait sits (L5):** `GatedScripting::launch` = gate check (`EditorLaunchDeferred` if held and this thread is not permitted) → if **not**
  permitted, `start_order.wait()` **outside the lane mutex** (so no script on either lane is stalled by the wait, and `a.tick` on the test thread cannot
  deadlock) → gate check again → lane → `inner.launch()`. A launch on a **permitted** thread (take-back step 5) never waits for `StartOrder` (L4a:
  R-I32 step 5 "whatever step 4 gave, launch Capture One"). There is no `launch_after_start_order()`: `ensure_provision` holds `Arc<dyn
  CaptureOneScripting>` and only calls `launch` after `editor_info` said not running (`provision.rs:82–83`), so "only if not running" already holds.
- **Launch gate (R-I32, M40).** `LaunchGate { held: AtomicBool, permit: thread-local }` is **process-wide** (same lifetime rule as `StartOrder`; a per-build
  gate would hand a mid-take-back settings save an ungated lane, breaking M40): `hold() -> GateGuard` (released on drop); `GateGuard::permit<T>(f)` runs
  `f` with the current thread permitted; `is_held()`, `permitted_here()`. `GatedScripting::call` for `launch`, `front_document`, `make_document`,
  `open_document`, `ensure_recipe`, `ensure_capture_style` (the six scripts that do not check `running()` first; every other script, `quit` included,
  guards itself and is **not** gated) returns `Err(Error::EditorLaunchDeferred)` **without** running the script when `gate.is_held() &&
  !permitted_here()`. The check applies to **both** lanes.
  **Deferral inside the ensure (L2):** on `Error::EditorLaunchDeferred` from a gated call, `ensure_provision` returns `Ok(ProvisionReport::deferred())` (a
  new `deferred: bool` field; `ProvisionReport::deferred()` = `deferred: true`, everything else default) **before that step's error handler runs**, so the
  deferring step writes no health item (`editor_not_running`/`editor_not_installed`, `document_not_created`, `document_not_open`, `recipe_missing`,
  `style_not_found` are never set by a deferred step). Items written by steps that finished before the deferral stand (step 1's `editor_*` clears, step
  3's `document`). `front_provision_document` returns `Err(EditorLaunchDeferred)` before its `.ok()` and does not touch `document_not_front`.
  **Callers after a deferred ensure (L1):** `EditingStartup::run` returns `Ok(())` right after an `ensure` that reports `deferred`, with no inspect, front,
  folder or health write (`editor_not_following` untouched) — otherwise `inspect_document` → `document_info` throws "Capture One is not running" during
  take-back steps 3–4 and `run` would write a false `document_not_open`. `SessionsService::activate` with the direct adapter: a deferred ensure sets
  `editor_not_following` and returns `Ok(())` (the session is already stored); with the Capture One camera adapter it returns `Err(EditorLaunchDeferred)`
  without storing. Every launch path goes through this: `EditingStartup::run` (startup thread, `health_check`, the settings rebuild's startup thread,
  `App::startup`), `SessionsService::activate`, and the six scripts themselves (projection §Launch-path enumeration).
- **`quit.js` (L13).** `if (!a.running()) return JSON.stringify(null); a.quit(); …` — an unguarded `Application("Capture One").quit()` would launch
  Capture One if it exited between the click and step 3 (a launch outside step 5, M40). The fake records op `quit` either way.
- **"Take the camera back" (R-I32).** `TakeBackService::run(&self) -> Result<TakeBackReport { completed_steps: u8, camera_held: bool }>`:
  preconditions (`TakeBackInputs { uploads_in_progress, expected_transfer, youngest_expecting_exposure_age_ms, jobs_running: {render_master, render_output,
  publish_step}, renders_in_flight, holder_is_capture_one }`) → `TakeBackBlocked { reason }` with the §6.6 reasons, checked in that order; `jobs_running`
  counts rows in status **`running` only** — a `queued` export or publish row holds no Capture One render and does not block (L25); then: **1** `gate.hold()`;
  **2** `scripting.save(doc)` (the ungated `App.scripting`) — `Err` → `TakeBackFailed { step: 2, cause }`, gate released, Capture One untouched; **3**
  `scripting.quit()` then poll `editor_info().running` every 500 ms up to `TAKE_BACK_QUIT_WAIT_MS` — still running → `TakeBackFailed { step: 3, cause:
  "Capture One did not quit (a dialog may be open)" }`, **never killed**; **4** check `editor_info().running` at the step's start **and on every poll of
  its waits** (L7; R-I32 "during steps 3–4") — running (the photographer started it) → `TakeBackFailed { step: 4, cause: "Capture One was started
  meanwhile" }`; else `client.send(UsbRetry)` and wait up to `TAKE_BACK_READY_WAIT_MS` for `ConnectionReady { usb }`; a `UsbClaimedBy` within the first
  `TAKE_BACK_FREE_WAIT_MS` → resend `UsbRetry` after 1 s (the interface frees "immediately" after Capture One quits, V6); timeout → `camera_held = false`,
  continue; **5** `gate.permit(|| EditingStartup::run(..))` (launch — on the permitted thread, so no start-order wait — ensure, front, folder; P49 adds
  the current collection) under the still-held gate; its `Err` → `TakeBackFailed { step: 5 }`; **6** release (the guard drops). The service runs on the
  caller's `spawn_blocking` thread; a second `run` while one is in progress → `TakeBackBlocked { reason: "already running" }`. Over the wire (L18):
  `take_back_failed` → `CommandError { code: "take_back_failed", data: { "step": "<n>", "cause": "<cause>" } }` — `data` is a string map, so `step` is
  `"2"`…`"5"`, never a number.
  **Ownership of the take-back's inputs (L14):** `AppBinding` gains what the take-back needs (the editing provider, the ungated scripting, the provision
  inputs, the document id, the gate; names the implementer's). The runtime gets the test seams `CameraRuntime::set_process_list` / `set_sleep_inhibitor`,
  set before the first tick (like `set_delivery_fs`).
- **Activation independent of Capture One (R-I33).** `SessionsService::with_camera(&dyn CameraProvider)` (no camera = today's behaviour; `test_app.rs`
  `sessions()` and `commands/sessions.rs` `session_activate`/`session_deactivate` pass `app.camera`); `EditingStartup` likewise gains `camera:
  Option<&dyn CameraProvider>` (the three `lib.rs` sites pass `camera`). When `camera.capabilities().depends_on_editing_provider` is false: `activate`
  stores `active_session_id` **first**, then ensure → front → `set_capture_folder`; any error there (a deferral included) sets `editor_not_following`
  (`Capture One will not show this session's new photos until it is reachable`) and returns `Ok(())`; `deactivate` likewise. **One item, not two (L12):**
  with the direct adapter `activate`/`deactivate` and `EditingStartup::run` clear `capture_folder_not_set` (the Capture One adapter's own set at
  `adapter.rs:183–188` is undone) and set only `editor_not_following`. `EditingStartup::run` clears `editor_not_following` after a successful folder step.
  With the Capture One camera adapter both behave exactly as today (p24/p25 rows unchanged).
- **Sleep (R-I23).** The adapter calls `sleep.hold("Session Manager: camera connected by cable")` on `ConnectionReady { usb }` and `release()` on
  `SessionClosed | SessionLost` from USB, on **`HelperEvent::Exited`** (a crashed sidecar must not leave the Mac unable to sleep, L15) and on shutdown.
  `hold` is a no-op while held (a repeated USB `connection.ready` never creates a second IOPM assertion); `release` is a no-op when not held.
  `IokitSleepInhibitor`: `IOPMAssertionCreateWithName(<CFString "PreventUserIdleSystemSleep">, kIOPMAssertionLevelOn, <CFString name>, &id)` /
  `IOPMAssertionRelease(id)` — `kIOPMAssertionTypePreventUserIdleSystemSleep` is a `CFSTR` macro, not a linkable symbol, so the CFStrings are built
  through CoreFoundation (L16).
- **`camera_test_connection`** → `TakeBackService::holder_report()`: `programs` = the known programs running (a fresh `process_list.running()`, in
  `KNOWN_CAMERA_PROGRAMS` order) **in every case**; `holder` and `sentence` come from `name_programs(<current holder process>, running)` only when the
  adapter's status is `HeldByOtherProgram`, else `null` (L19) → `{ holder, programs, sentence }`.

## Ordered tasks
0. **Registry first** (§6.11): ports `ProcessList`, `SleepInhibitor`; `CaptureOneScripting::quit`; errors; constants; `ProvisionReport.deferred` +
   `::deferred()`; `CameraStatus::NeedsAction.message`; `StatusInputs.needs_action` with the message.
1. `ownership.rs` + unit tests; `processes.rs`, `sleep.rs`, fakes; the adapter's holder report into `StatusInputs`; `CameraStatusResponse` reads the message
   (and the two p46 test sites).
2. `launch_gate.rs`, `gate.rs` changes (the check on both lanes; `launch`'s wait outside the lane mutex), `start_order.rs` (process-wide, `AlreadyRunning`,
   deadline from the first wait), `provision.rs` (the deferral returns before each step's error handler; `front_provision_document`), `quit.js` (guarded) + `jxa.rs`.
3. `sessions.rs`/`editing_startup.rs` for the direct adapter (R-I33, L1, L12); `editor_not_following`; `with_camera` at the three service construction sites.
4. `take_back.rs` (preconditions over `running` jobs; step 4's check on every poll); the adapter's sleep hold/release (+ `Exited`); runtime wiring (the gate
   and the start order created in `open_with_adapter_bind` and carried like the lanes; `AppBinding` additions; `set_process_list`/`set_sleep_inhibitor`).
5. Commands `camera_take_back`, `camera_test_connection`; bindings.
6. Testkit fakes (`quit_after_polls`, `restart_after_quit`, `fail_save`, `FakeProcessList`, `FakeSleepInhibitor`), `TestApp::{take_back, launch_gate,
   user_lane, fake_processes, fake_sleep}`, scripts; tests `p48_camera_ownership.rs`; suite, clippy, fmt, typecheck.

## Criteria (8 criteria, 38 rows; 0 `[rule]` rows)
Fixtures: `a = TestApp::with_camera_script(<script>)` with `a.fake_processes.set(vec![…])` (default empty); `F = a.fake` (the fake Capture One: `running`
true, document open); `ops()` = `F.state.ops`; `st()` the camera status; `H` health; `TB = a.take_back()` (`TakeBackService`); `rec()` the sidecar
recording. Scripts: `claimed.txt` (P46), `claimed_wifi.txt` (`ready_wifi` lines then `usb.claimed_by {owners:"pid 512, ptpcamerad"}` + `connect.failed usb
0x8208`), `take_back.txt` (`claimed_wifi.txt` then `await usb-retry`, `connection.ready {transport:"usb", model:"ILCE-7RM5", store_destination:"verified"}`).
Waits in the services poll `a.clock`; tests advance it from the test thread while the service runs on another.
**Fixture rules (projection L6, L8, L17):** (i) every `F.running = false` and every fake `quit` is paired with `F.running_after_polls = Some(1)`, or the
ensure's launch wait burns 45 s of real time and ends in `DocumentNotOpen`; C2(g) resets **both** (`running = false`, `running_after_polls = Some(1)` —
the counter stays at 0 after C2(a)). (ii) **C3** uses `a = TestApp::with_camera_script("ready_usb")` ticked until `st() == Connected` (start order
satisfied; the direct adapter's status adds no Capture One op — `with_fake_document` would add a `document_info` op on every health check); `ops()` is
cleared right after `hold()`. (iii) **C5(b)–(f) inherit C5(a)'s fixture** (`claimed_wifi.txt`/`take_back.txt`, `fake_processes = ["Capture One"]`, ticks
until `H.has("camera_held")`, `activate(s)` before) with `ops()` cleared right before `TB.run()`; "`ops()` in order" means **subsequence** (the ensure adds
`document_info`/`ensure_recipe:*`/hook ops, `apply_folder` adds `save`/`get_captures`). (iv) C5(g) runs on the mock webview: before the call a ticker thread
holding the `CameraRuntime` and `ManualClock` Arcs advances the clock 500 ms and calls `camera_runtime.tick(clock.mono())` every 10 ms until a stop flag
is set after the call returns (the `TestApp` is destructured into `AppState`, so `a.tick` is not available while `get_ipc_response` blocks).

| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | `name_programs` (unit) | `("ptpcamerad", ["Finder", "Imaging Edge Desktop", "Capture One"])` | `programs == ["Capture One", "Imaging Edge Desktop"]`, `holder == Some("Capture One")`, `sentence == "Capture One is holding the camera's USB connection — Take the camera back"` | MC-34 §Take it back; R-I30 ("Capture One first") |
| C1(b) | (unit) | `("ptpcamerad", ["Photos"])`; `("ptpcamerad", ["Imaging Edge Desktop", "Adobe Lightroom"])` | `programs that can take the camera: Photos — set "Connecting this camera opens" to "No application"`; `programs that can take the camera: Imaging Edge Desktop, Adobe Lightroom — quit it and stop it opening at login` | MC-34 §Guide; R-I30 ("programs that can take the camera: …") |
| C1(c) | (unit) | `("ptpcamerad", [])` | `programs == []`, `holder == Some("ptpcamerad")`, the system-service sentence | MC-34 (R-I30) |
| C1(d) | (unit) | `("SomeApp", ["Capture One"])` | `programs == []`, `holder == Some("SomeApp")`, `SomeApp is using the camera: close it or stop it from using cameras` | MC-34 §Guide ("unknown programs") |
| C1(e) | status and health with a holder, Wi-Fi continues | `claimed_wifi.txt`; `fake_processes = ["Capture One"]`; ticks; then `a.capture()` | `st() == NeedsAction { HeldByOtherProgram, holder: Some("Capture One"), message: Some(C1(a)'s sentence) }`; `H["camera_held"].detail == C1(a)'s sentence`; `capture()` → `Ok` and the `capture` line is sent (shooting over Wi-Fi continues) | MC-34 §Detect, §Guide ("Wi-Fi continues"); R-I37 (`NeedsAction` wins while connected on the other link) |
| C1(f) | cleared when we hold the interface | after C1(e): inject `connection.ready {transport:"usb"}` | `st() == Connected { link: Cable }`; `camera_held` gone | MC-34 ("The health screen keeps the item until the holder is gone") |
| C2(a) | start order: launch waits for `connection.ready usb` | script `sleep 1500` then `ready_usb`; `F.running = false`, `F.running_after_polls = Some(1)`; `a.app.startup()` on a thread; the test ticks the runtime every 50 ms | while the ready line has not been processed, `ops()` has no `launch`; after it, `startup()` returns and `ops()` has `launch` **after** `editor_info` | M40 ("Capture One is launched only after one of MC-34's start-order conditions holds"); R-I31 |
| C2(b) | `usb.initial { present: false }` | script `usb.initial {present:false}` first; `F.running = false` (+ rule i); `startup()` | returns with `launch` in `ops()` before any clock advance beyond 100 ms | MC-34 (R-I31) |
| C2(c) | fallback to Wi-Fi | script `usb.fallback_to_wifi {usb_failures:3, usb_retry:"on next USB re-plug"}`; `F.running = false` (+ rule i) | launch proceeds at once | MC-34 (R-I31) |
| C2(d) | sidecar unavailable | script `exit 4` (after `sdk.unavailable`); `F.running = false` (+ rule i) | launch proceeds at once | MC-34 (R-I31 "the sidecar being unavailable or crashing") |
| C2(e) | the 15 s bound | script with no events (`sleep 100000`); `F.running = false` (+ rule i); the test advances `a.clock` by 15 s after `startup()` has called `wait()` | `startup()` returns after the advance with `launch` in `ops()`; before the advance it has not returned | MC-34 (R-I31 "15 s"; the deadline counts from the first `wait()`) |
| C2(f) | Capture One already running | `F.running = true`; script `sleep 100000` | `startup()` returns without waiting (no clock advance); `ops()` has no `launch`; the start order is `Satisfied(AlreadyRunning)` afterwards (a later `F.running = false` + rule i + `health_check` launches without a clock advance) | MC-34 (R-I31 "If Capture One is already running at app start, nothing is waited for") |
| C2(g) | satisfied once | after C2(a): `F.running = false` and `F.running_after_polls = Some(1)` again (the test resets both); `a.app.health_check()` | returns at once with a new `launch` (no second wait) | MC-34 (R-I31: start order is an app-start rule) |
| C3(a) | gate defers `ensure` at launch | `g = a.launch_gate().hold()`; `ops()` cleared; `F.running = false` (+ rule i); `a.app.editing.ensure(&provision)` | `Ok(ProvisionReport { deferred: true, .. })`; `ops() == ["editor_info"]`; `editor_not_running`/`editor_not_installed` not set | R-I32 ("While Take the camera back holds the gate, these return deferred without running a script"); M40 |
| C3(b) | `front_document` deferred | gate held; `ops()` cleared; `a.app.editing.bring_document_to_front()` | `Err(EditorLaunchDeferred)`; `ops()` empty; `document_not_front` unchanged | MC-34 (R-I32) |
| C3(c) | `make_document` deferred | gate held; `F.open_documents.clear()` and the document folder removed so ensure would create it; `ensure` | `deferred: true`; no `make_document` op; `document_not_created` not set | MC-34 (R-I32) |
| C3(d) | `open_document` deferred | gate held; `F.open_documents.clear()` (folder exists); `ensure` | `deferred: true`; no `open_document` op; `document_not_open` not set by this ensure | MC-34 (R-I32) |
| C3(e) | `ensure_recipe` deferred | gate held; document open; `ensure` (step 4 calls `ensure_recipe` unconditionally) | `deferred: true`; no `ensure_recipe:*` op; `recipe_missing` not set; the `document` ok item from step 3 stands | MC-34 (R-I32) |
| C3(f) | `ensure_capture_style` deferred (user-lane boundary; the ensure never reaches the style step while held, so the row targets the lane) | gate held; `ops()` cleared; `a.user_lane().ensure_capture_style(<document id>, "PSM House")` | `Err(EditorLaunchDeferred)`; `ops()` empty | MC-34 (R-I32) |
| C3(g) | a health check while held never launches | gate held; `ops()` cleared; `F.running = false` (+ rule i); `a.app.health_check()` | `Ok`; `ops() == ["editor_info"]` (no `launch`, no `document_info`: the startup stops after the deferred ensure); the report has no new `editor_not_running` or `document_not_open` item | M40 ("nothing but its own step 5 launches Capture One") |
| C3(h) | released | `drop(g)`; `F.running = false` (+ rule i); `ensure` | `deferred: false`; `launch` in `ops()` | R-I32 step 6; M40 |
| C4(a) | blocked: upload in progress (unit + boundary) | `TakeBackInputs { uploads_in_progress: 1, .. }`; boundary: `a.ftp()` holds an open `STOR`; `TB.run()` | `Err(TakeBackBlocked { reason: "a file is still arriving" })`; `ops()` has no `save` | R-I32 ("enabled only when no upload or download is in progress") |
| C4(b) | blocked: a photo expected | exposure 5 s old `expecting` → `"a photo is still expected"`; exposure 11 s old → not this reason | as stated | MC-34 (R-I32 "no exposure younger than 10 s is still expecting a file") |
| C4(c) | blocked: export running | a `render_master` job `running` (inserted by the test) → `"an export is running"`; a `queued` `render_master` row alone → not this reason | as stated | MC-34 (R-I32 "no export or publish run is active"; `running` only) |
| C4(d) | blocked: publish running | a `publish_step` job `running` → `"a publish is running"` | as stated | MC-34 (R-I32) |
| C4(e) | blocked: a preview rendering | `renders_in_flight == 1` (a render issued to the fake, not completed) → `"a preview is rendering"` | as stated | MC-34 (R-I32 "no render is in flight") |
| C4(f) | blocked: Capture One is not the holder | `ready_usb` (we hold the camera) or `claimed` with `fake_processes = ["Photos"]` | `"Capture One is not holding the camera"` | MC-34 ("If the USB interface is held by the system camera service on Capture One's behalf") |
| C5(a) | the sequence succeeds | `take_back.txt`; `fake_processes = ["Capture One"]`; `activate(s)` before; `F.quit_after_polls = Some(1)`, `F.running_after_polls = Some(1)`; `ops()` cleared; `r = TB.run()` (test thread ticks the runtime and advances the clock in 500 ms steps) | `r == Ok(TakeBackReport { completed_steps: 5, camera_held: true })`; `ops()` contains, as a subsequence in this order: `save`, `quit`, `editor_info`…, `launch`, `editor_info`…, `set_captures:<root>/<key(s)>`; `rec().stdin` contains one `usb-retry`; `st() == Connected { link: Cable }`; after `r` the gate is released (`ensure` → `deferred: false`) | R-I32 steps 1–6; MC-34 §Take it back ("saves the Capture One document, quits Capture One, opens the USB session, relaunches Capture One and reopens the document"); M39, M40 |
| C5(b) | save fails → nothing else happens | C5(a)'s fixture; `F.fail_save = true`; `TB.run()` | `Err(TakeBackFailed { step: 2, .. })`; `ops() == ["save"]`; `F.running` still true; gate released | R-I32 ("`save` the document. On failure, stop with Capture One untouched") |
| C5(c) | quit does not finish → never killed | C5(a)'s fixture; `F.quit_after_polls = None` (never stops); clock advanced 21 s in steps | `Err(TakeBackFailed { step: 3, .. })`; `F.running == true`; no `launch`; no `usb-retry` sent; `a.fake_processes` untouched (no kill API exists) | R-I32 step 3 ("If it is still running (a dialog is open), stop; it is never killed"); M39 ("No other process is ever killed") |
| C5(d) | step 4 times out, step 5 still runs | C5(a)'s fixture with script `claimed_wifi.txt` + `await usb-retry` + `sleep 100000` (never ready); `F.quit_after_polls = Some(1)`, `F.running_after_polls = Some(1)`; clock + 16 s | `Ok(TakeBackReport { completed_steps: 5, camera_held: false })`; `launch` in `ops()` (step 5 launched on the permitted thread without a start-order wait); `st()` still `NeedsAction { HeldByOtherProgram, .. }` | R-I32 step 5 ("Whatever step 4 gave, launch Capture One") |
| C5(e) | the photographer started Capture One meanwhile | C5(a)'s fixture; `F.quit_after_polls = Some(1)`, `F.restart_after_quit = true` (the `editor_info` after the first one that reports not-running reports `running = true`, with no `launch` op) | `Err(TakeBackFailed { step: 4, cause: "Capture One was started meanwhile" })`; no `launch` by us; no `usb-retry` | R-I32 ("If Capture One starts during steps 3–4 and the app did not start it … stop and report") |
| C5(f) | quit only on click | `claimed_wifi.txt`; 30 s of ticks with the holder present and `H` showing `camera_held` | `ops()` has no `quit` | M39 ("Capture One is quit only on the photographer's click"); MC-34 ("It runs only on the photographer's click") |
| C5(g) | `camera_take_back` command (mock webview; fixture rule iv) | C5(a)'s fixture → `{"completed_steps":5,"camera_held":true}`; C4(a)'s fixture → error `{"code":"take_back_blocked","data":{"reason":"a file is still arriving"}}` | as stated | MC-34 (R-I32); master plan §6.7 |
| C6(a) | activation stores first, Capture One follows or not | direct adapter active (`ready_usb`); `F.fail_next_set_captures = true`; `a.sessions().activate(s)` | `Ok(())`; `store.active_session_id() == Some(s)`; `H.has("editor_not_following")` with the §6.6 detail; `!H.has("capture_folder_not_set")`; `st()` unchanged (`Connected`); `a.capture()` → `Ok` | R-I33; 10A MC-18 step 5 (direct adapter: "the camera status and capture are not touched") |
| C6(b) | retried at the next ensure | after C6(a): `a.app.startup()` (folder step succeeds now) | `editor_not_following` cleared; `set_captures:<key(s)>` in `ops()`; with the Capture One camera adapter (`with_fake_document`) `activate` with `fail_next_set_captures` still returns `Err(CaptureFolderNotSet)` and does **not** store the session (p24/p25 rows unchanged) | R-I33 ("retried at the next ensure"); M25 (Capture One adapter only) |
| C7(a) | sleep assertion follows the USB session | `ready_usb` → `a.fake_sleep.held() == Some("Session Manager: camera connected by cable")`; inject `connection.ready {transport:"usb"}` again → still `Some(..)` and `a.fake_sleep.holds() == 1`; inject `session.lost usb` → `None`; `ready_wifi` → `None`; a second app with script `ready_usb` then `sleep 200` then `exit 1` → `None` once the exit is processed | as stated | MC-31 (R-I23 "While a USB session is open, the app holds a macOS idle-sleep assertion") |
| C8(a) | `camera_test_connection` (mock webview) | C1(e)'s fixture → `{"holder":"Capture One","programs":["Capture One"],"sentence":"Capture One is holding the camera's USB connection — Take the camera back"}`; `ready_usb` fixture with `fake_processes = ["Photos"]` → `{"holder":null,"programs":["Photos"],"sentence":null}` (lists who **could** take it) | as stated | MC-34 §Setup check ("lists every program that could take the camera"); master plan §6.7 |

## Delegated choices
The thread-local permit's implementation (`thread_local!` `Cell<bool>` or a token in the guard); `PsProcessList`'s dedupe and ordering; how the take-back
thread advances in tests (recommended: every wait polls `clock.mono()` and sleeps 10 ms real time); `TakeBackReport`'s extra fields; `renders_in_flight` as a
port method or an adapter downcast (recommended: `EditingProvider::renders_in_flight() -> usize`); the names of the `AppBinding` additions and of the
process-wide gate/start-order handles, under the L14 constraint stated in the contract (created in `open_with_adapter_bind`, passed to
`GatedScripting::pair`, reachable by the take-back service, never rebuilt by `App::build`); the CoreFoundation binding for `sleep.rs` (a direct
`core-foundation` dependency — `core-foundation` reaches psm-infra only transitively via `security-framework` today — or an `extern` block linking
CoreFoundation and IOKit; `#[link(name = "IOKit", kind = "framework")]` in `sleep.rs`, no row: the suite uses `FakeSleepInhibitor`).

## Manual try-out
(Hardware-free; the real take-back is P52's row.) 1. Start the app with Capture One already open: no change. 2. Health → the camera row behaves as before;
"Retry checks" still works. 3. Open a session, switch sessions: Capture One follows as before. 4. Nothing else visible; the "Take the camera back" button
arrives with P51.

## Notes
- **Hard rules (owner):** never commit a Sony SDK file, the lab's `sdk_shim` headers, a `.env`, a credential or an SSH fingerprint; no Python; never change the
  camera's network/FTP settings, delete a camera file, or **kill `ptpcamerad` or any process** (Capture One is asked to `quit` by AppleScript, never
  killed; C5(c)); the only process we kill is our own sidecar child after `quit` (P42); live view out of scope; the suite never needs the camera. No `implementer: fable`.
- **Planner readings (reversible):** (1) step 4's "interface free" is inferred from `usb-retry` succeeding; a `usb.claimed_by` during the 5 s free-wait
  triggers one more `usb-retry` per second (V6: the interface frees immediately when Capture One quits; the sidecar emits `usb.claimed_by` only with
  non-empty owners on a connect attempt, so there is no "cleared" event to wait for — confirmed at projection L24). (2) *struck at projection L1*: callers
  do stop after a deferred ensure (see the contract). (3) `editor_not_following` is one item for both the folder and (P49) the collection.
  (4) The holder for `NeedsAction.holder` is the first named program, else the process. (5) "An export or publish run is active" = a `render_master`,
  `render_output` or `publish_step` job in status `running`; queued rows hold no Capture One render (L25). (6) `camera_test_connection.programs` lists the
  known programs running in every case, holder or not (L19).
- **Deferred (projection L20):** R-I30's best-effort scan for "any other user process that has ImageCaptureCore loaded" is **not** in P48 — it waits for
  spike S5's answer; P48 names the fixed list only. Process names are owned by macOS (rule 17): a read-only `ps -axo comm=` on the dev Mac (2026-11-07)
  confirmed `Capture One`; Imaging Edge Desktop's executable name is unverified and the "Photos widget" is `PhotosReliveWidget` (does not match
  `Photos`). No P48 row depends on either (C1 is a pure function over a fake list); the check is a backlog/P52 note.
- **Hazards:** `GatedScripting` is **also** the background lane — the gate check applies to both lanes. `App.scripting` (ungated) is what the take-back
  uses for `save`/`quit`; the user lane for everything else under `permit`. `ensure_provision` holds `provision_lock` — the take-back's step 5 runs it on
  the take-back thread while a health check may wait on the lock; fine, but never call `ensure` from inside the lock, and never wait on `StartOrder`
  inside the lane mutex (L5). The `quit` script addresses the application, not the document (`Application("Capture One").quit()`) and is guarded by
  `running()`; the fake's `quit` must flip `running` only after `quit_after_polls` `editor_info` calls so C5(c)/(e) are constructible, and
  `restart_after_quit` flips it back on the next `editor_info` with no `launch` op (C5(e)). The fake's `document_info`/`set_captures` ignore `running`
  (`fake_capture_one.rs:360–369, 398–401`), which is why C3(g) asserts `ops()` exactly. `sessions.rs` must branch on the **provider's** capability, not
  on a settings flag (R-I33 is about the adapter). The `frontmost` seed of an earlier draft is dropped (no caller in P48; P49 adds it if needed, L21).
- **Projection** r0 (2026-11-07, `handoffs/reviewer/handoff_P48_camera_ownership_projection_r0.md`): AMENDMENTS_REQUIRED, L1–L25, 0 owner cards; every
  launch path at `33b9eca` goes through the six gated methods once L1, L13 and L14 hold. All 25 rows are folded into this version (Review log).
- Registry additions made by the planner in master plan §6.4, §6.6, §6.7, §6.11, §7.5; amended after projection (§6.6 `take_back_failed{step, cause}`,
  the `already running` reason, the `camera_held` examples, `NeedsAction.message`; §6.7 `camera_take_back` shape; §6.11 `name_programs(owner_process, …)`,
  the six-script gate list without `quit`, process-wide gate/start order, `StatusInputs` message; §7.5 `restart_after_quit`, `fail_save`). No intention-book change.
- Estimated size: ~43 paths, ~2 500 lines (≈ 1 100 code, ≈ 1 400 tests + fakes).

## Review log
- 2026-11-07, orchestrator amendments after projection r0 (L1–L25 folded; 0 owner cards). Planner (Fable) under orchestrator rulings: L1 callers stop
  after a deferred ensure (planner reading 2 struck; C3(g) asserts `ops() == ["editor_info"]`); L2 deferral returns before each step's health handler;
  L3 C3(f) re-targeted to `a.user_lane()`; L4 a permitted launch never waits, first `running` editor_info → `Satisfied(AlreadyRunning)`, deadline from
  the first `wait()`; L5 `launch_after_start_order()` dropped, wait outside the lane mutex; L6 `running_after_polls = Some(1)` pairing (fixture rule i);
  L7 `restart_after_quit` + step 4 checks on every poll; L8 C5(g) ticker thread; L9 the sentence rides `StatusInputs` and `CameraStatus::NeedsAction.message`
  (two p46 test sites adjusted, declared in Files); L10 `SessionsService::with_camera`, `commands/sessions.rs` in the perimeter; L11 `domain/editing.rs`;
  L12 one item (`editor_not_following`) with the direct adapter, C6(a) asserts `!capture_folder_not_set`; L13 `quit.js` guarded, §6.11 gate list = six;
  L14 process-wide gate/start order, `AppBinding` additions, `set_process_list`/`set_sleep_inhibitor`; L15 release on `Exited`, idempotent hold/release,
  C7(a) extended; L16 CoreFoundation binding delegated; L17 fixture rules ii–iii and C8(a)'s full sentence; L18 `take_back_failed` data as strings,
  §6.6/§6.7/§6.11 fixed, P51 C2(c) mock corrected; L19 `programs` = known running in every case; L20 Notes + backlog note; L21 `frontmost` seed dropped;
  L22 trace prefixes; L23 anchors at `5eba743`; L24 reading 1 stands; L25 `running` only (C4(c) adds the `queued` case). Rows 38 → 38; paths 33 (typed) →
  43 (derived). Nothing declined.
- 2026-11-07, implementer r1 (Claude, `PSM_IMPLEMENTER=claude`). **Built:** `ownership.rs` (`KNOWN_CAMERA_PROGRAMS`, `name_programs`,
  `known_running`), the adapter's holder report on every `usb.claimed_by` (one `process_list.running()` per claim) into `StatusInputs` as
  `(HeldByOtherProgram, holder, sentence)`; `CameraStatus::NeedsAction.message`; `health_items`/`CameraStatusResponse` use the message;
  `StartOrder` (process-wide, `AlreadyRunning` from any gated `editor_info` that reports running, deadline from the first `wait()`) and
  `LaunchGate` (thread-local permit keyed by gate) created in `open_with_adapter_bind` and carried in a new `Lanes` struct through
  `SettingsSetContext`/`reopen`; `GatedScripting` refuses the six scripts on both lanes, `launch` waits outside the lane mutex off the permitted
  thread; `ensure_provision` returns `ProvisionReport::deferred()` before each step's health handler, `front_provision_document` returns the
  deferral before its `.ok()`; `quit.js` (guarded, `SCRIPTS` 19 → 20); `EditingStartup { camera }` stops after a deferred ensure and, with the
  direct adapter, turns a failed folder step into `editor_not_following`; `SessionsService::with_camera` (R-I33: store first, `editor_not_following`,
  never `capture_folder_not_set`); `TakeBackService` (preconditions, steps 1–6, step 4 re-checks `editor_info` every 500 ms of clock time and
  re-sends `usb-retry` 1 s after a claim inside the free wait, `holder_report`); idle-sleep hold/release via `StepOutcome.sleep` (released on USB
  `session.closed|lost`, on a Wi-Fi `connection.ready`, on `Exited` and at shutdown); `PsProcessList`, `IokitSleepInhibitor` (extern block over
  CoreFoundation + IOKit, no new dependency); commands `camera_take_back`, `camera_test_connection`; error codes; bindings regenerated.
  **Tests:** `p48_camera_ownership.rs` 32 rows (C1(e)–C7(a)), desktop `p48_tests` (C5(g), C8(a)), unit tests for `name_programs` (C1(a)–(d)) and
  `preconditions` (C4 halves + order). **Assumptions** and **deviations** are listed in the handoff (`handoffs/implementer/
  handoff_P48_camera_ownership_implement_r1.md`): no `EditingProvider::quit` (the take-back calls `CaptureOneScripting::quit` on the ungated
  scripting), `AlreadyRunning` set in `GatedScripting::editor_info`, test fixture rule (i) also resets the fake's `launched` flag (C2(g)),
  `services/camera.rs` and `infra/Cargo.toml` unchanged.
