---
phase: P42_camera_runtime
title: Camera runtime — one camera layer per app process (sidecar supervisor over a fake sidecar, secrets port, the new CameraStatus and its precedence), carried unchanged across settings rebuilds
difficulty: high
projection: done
depends_on: [P41_undo_history_ui]
---
# P42 — Camera runtime: the process-wide camera layer, the sidecar supervisor, the status model

## Goal
The foundation every later camera phase builds on (10E MC-31, inventory R-I18, R-I21, R-I33, R-I37; M39): a `CameraRuntime` created **once per
app process**, outside the runtime that a settings save rebuilds (`App::build`), holding the sidecar **supervisor** (spawn, first-line credentials,
restart with bounded backoff, `quit` → 3 s → kill; never two processes) and the slots the receiver (P43), the ledger (P44) and the direct adapter
(P46) fill later; the `SecretStore` port with its Keychain implementation and its fake; the neutral camera domain (`CameraStatus` with five
variants, `Link`, `BusyReason`, `NeedsActionReason`) and the **status precedence** mechanism; `CameraService` no longer gated by Capture One's
health for a provider that does not depend on the editing provider; the `camera_status` command's new DTO.
**Not in this phase:** the FTP receiver (P43), the ledger and its tables (P44), the real sidecar and the protocol parser (P45; here the sidecar's
lines are opaque and the test double is a minimal fake binary), the direct adapter (P46), camera settings keys (P43/P50), the Settings and Health UI
(P51). The camera provider in use stays `CameraViaCaptureOne`; **no runtime status source exists yet**, so in P42 only `camera_retry` emits
`camera_status_changed` (through the App's `EventHub`) and the runtime's own emitter arrives with P46.

## Read first
Book: `10E_direct_camera_adapter.md` §Port changes (status values, R-I33, R-I37 as reworded at `789f035`: `Connecting` needs a reported connecting
attempt), MC-31 §Process and §Secrets (deferring to R-I21 since `789f035`) + inventory additions **R-I18**, **R-I21**, M39 · `11_architecture_and_
packaging.md` (one camera runtime per process) · `12_operations.md` (Keychain) · `14_measurement_ledger.md` M39 · `01_objective_and_constraints.md` H7,
H8 · process `flow.md` §4, §6, §9 · master plan §3 (camera hard rules), §6.1, §6.2 (camera crates), §6.3 (`camera/` under the data dir), §6.4, §6.6,
§6.7 (`camera_status` DTO, `camera_status_changed`), **§6.11** (Constants, Domain, Ports, Mechanisms `camera_status.rs`, Runtime, Sidecar
`Supervisor`), §7.5 (`with_camera`, `with_camera_behaviour`).
Code at `41bef14` (= HEAD `2550e09` under `crates/`, `apps/`, `tools/`; line numbers verified by the projection):
- `crates/integrations/src/lib.rs` — `App` `:73–106`, `AppHandles` `:108–128`, `SettingsSetContext` `:130–140` and `prepare` `:147–191` (**the new
  runtime is built at `:164–173` before the old is shut down; the failure path `:186–189` keeps the old App**), `apply_settings` `:274–292`, `handles`
  `:293–314`, `open_with_adapter` `:330–349`, `build` `:351–709` (camera provider `:431–434`, `events` `:435`, `set_dispatcher`/`NudgeServer::start` — the
  last fallible steps — `:493–497`, startup thread `:565–582` incl. `CameraService::status` `:579–580`, background loop `:607–649`, `Ok(Self {..})` `:676`),
  `settings_set` `:740–747`, `shutdown` `:750–761`, `reopen` `:764–783`, `EventHub` `:809–848`, `Drop for App` `:931–935`. `App::build` has exactly three
  callers: `open_with_adapter :339`, `prepare :164`, `reopen :765`; `open`/`open_with` go through `open_with_adapter`.
- `crates/core/src/ports/camera.rs:1–11` (`CameraProvider`), `crates/core/src/services/camera.rs:1–62` (the Capture One gates `:28–30`, `:52–58`),
  `crates/core/src/domain/editing.rs:31` (`CameraStatus`), `:37` (`CameraCapabilities`) — **move** both to `domain/camera.rs`; `crates/core/src/events.rs:
  23–24` (`CameraStatusChanged`); `crates/core/src/services/health.rs:101–105` (`has` counts only `!ok` items — read ok items through `report`).
- `crates/integrations/src/camera/via_capture_one.rs:1–35`; `apps/desktop/src-tauri/src/commands/camera.rs:1–90` (DTO `:11–23`; `TauriCameraEvents`
  `:24–33` and its use in `camera_retry` `:68–72` — **deleted**: it bypasses the `EventHub`); `apps/desktop/src-tauri/src/lib.rs` (`TauriEvents::emit` arms
  and its `_ => {}`; `run()`; `p39_tests::c7c_tauri_history_changed_event` `:126–129` — the hub-target pattern C8(a) uses); `apps/desktop/src-tauri/src/
  state.rs:22–38` (`settings_set`).
- `crates/infra/src/logging.rs:12–23` (`add_secret`), `:26–33` (`redact`), `:37` (`redacting`); `crates/infra/src/paths.rs:31–39` (`create`).
- `crates/testkit/src/test_app.rs:114–175` (`create`; `open_with_adapter` at `:155` — **the first spawn happens here**), `:185–197` (`restart`: builds the
  new App **before** dropping the old one), `:213–222` (`tick`); `crates/testkit/src/clock.rs` (`ManualClock::mono()` = `start + ms`); `crates/testkit/src/
  fake_capture_one.rs:115` (`camera: Some("Sony ZV-E10")`).
- Frontend: `apps/desktop/src/features/shell/CameraDot.tsx` (whole), `apps/desktop/src/services/camera.ts`, `apps/desktop/src/services/events.ts:12–14`;
  `apps/desktop/src/features/sessions/__tests__/p07_shell_sessions_ui.test.tsx:188–205` pins `title="Camera connected: <name>"`, `data-state` and the
  USB-mode help text on `disconnected` — kept.
- **Tests that change their expectation:** `crates/testkit/tests/p04_capture_one_adapter.rs:34` (`status` helper type), `:404`
  (`CameraStatus::Connected("Sony ZV-E10".into())` → `CameraStatus::Connected { name: "Sony ZV-E10".into(), link: Link::Cable }`),
  `crates/testkit/tests/p25_tidy_up_after_provisioning.rs:127` (same), `crates/testkit/tests/p24_capture_one_provisioning.rs:3` (the import
  `domain::editing::CameraStatus` → `domain::camera::CameraStatus`; `:407` unchanged); eight frontend files whose mocks return `{ kind: 'connected',
  name }` gain `link: 'cable'`: `apps/desktop/src/features/compose/__tests__/harness.tsx`, `features/shell/__tests__/p19_navigation.test.tsx`,
  `features/workspace/__tests__/{p19_drop, p08_workspace_ui, p22_workspace_tidy, p20_drop_continuity, p26_double_drag}.test.tsx`, `features/sessions/
  __tests__/p07_shell_sessions_ui.test.tsx` (grep `kind: 'connected'`). No other test changes.
- Lab (read-only): `/Users/davidloorenz/Desktop/Developer/Camera-Connection-Tests/docs/session-manager-handoff.md` §4.1 (process contract: stdin
  commands, stdout events, `quit`), `sdk/src/main.cpp:140–160` (`StdinQueue`: EOF pushes `quit` — the kernel backstop at a real quit, L6).
- Dependency grounding (rule 17, verified by the projection): `security-framework 3.7.0` is already in `Cargo.lock`; `passwords::get_generic_password`
  (`passwords.rs:40`), `set_generic_password` (`:26`), `delete_generic_password` (`:63`); `errSecItemNotFound = -25300` (`security-framework-sys-2.17.0/
  src/base.rs:61`). Tauri 2.12.5 / tao 0.37.2 end the process with `process::exit` (`tao-0.37.2/src/platform_impl/macos/event_loop.rs:197–202`): the
  managed `AppState` is **never dropped** at a real quit.

## Files expected to change
Core (new): `crates/core/src/domain/camera.rs`, `crates/core/src/mechanisms/camera_status.rs`, `crates/core/src/ports/secrets.rs`. Core (changed):
`crates/core/src/domain/{mod.rs, editing.rs}`, `crates/core/src/mechanisms/mod.rs`, `crates/core/src/ports/{mod.rs, camera.rs}`,
`crates/core/src/services/camera.rs`, `crates/core/src/events.rs`.
Infra (new): `crates/infra/src/keychain.rs`. Infra (changed): `crates/infra/src/{lib.rs, logging.rs, paths.rs}`, `crates/infra/Cargo.toml`.
Integrations (new): `crates/integrations/src/camera/runtime.rs`, `crates/integrations/src/camera/sidecar/{mod.rs, supervisor.rs}`,
`crates/integrations/src/camera/support/mod.rs` (the `installed_sidecar(paths)` seed). Integrations (changed): `crates/integrations/src/camera/{mod.rs,
via_capture_one.rs}`, `crates/integrations/src/lib.rs`, `crates/integrations/Cargo.toml` (uuid).
Desktop (changed): `apps/desktop/src-tauri/src/commands/camera.rs`, `apps/desktop/src-tauri/src/lib.rs` (`TauriEvents` arm; `run()` with the `RunEvent::Exit`
hook), `apps/desktop/src/services/bindings.ts` (regenerated), `apps/desktop/src/features/shell/CameraDot.tsx` (minimal: the five `kind`s render; texts final
in P51), the eight frontend files above.
Testkit (new): `crates/testkit/src/bin/fake_sidecar.rs`, `crates/testkit/src/fake_secrets.rs`, `crates/testkit/src/fake_direct_camera.rs`
(`FakeDirectProvider`). Testkit (changed): `crates/testkit/src/{lib.rs, test_app.rs}`, `crates/testkit/Cargo.toml` (`[[bin]] fake_sidecar`; `libc` for
`alive(pid)` and `SO_LINGER` later). Tests (new): `crates/testkit/tests/p42_camera_runtime.rs`. Tests (changed): `crates/testkit/tests/
{p04_capture_one_adapter.rs, p25_tidy_up_after_provisioning.rs, p24_capture_one_provisioning.rs}`.
Derived count: 11 new source paths, 22 changed, 1 new test, 3 adjusted Rust tests, 8 adjusted frontend files — 45 paths.

## Core mechanisms (unit tests allowed)
`crates/core/src/mechanisms/camera_status.rs` — `derive_status`, `NeedsActionReason::rank`, `health_items`, `status_message`: inline `#[cfg(test)]`
tests named `mc31_<slug>` (rows C5(a)–(l)). Everything else is a behaviour row.

## The runtime contract (master plan §6.11 names)
- `CameraRuntime::new(secrets: Arc<dyn SecretStore>, clock: Arc<dyn Clock>, paths: AppPaths) -> Arc<Self>`; `run_id` = a UUID v4 string.
  `App::open_with_adapter` creates it; **`App::build` takes it as a parameter** and stores `camera_runtime: Arc<CameraRuntime>` on `App`,
  `AppHandles` and `SettingsSetContext`; `prepare` (`lib.rs:164`) and `reopen` (`:765`) pass the same `Arc` (the three callers above are the whole set).
  `App::shutdown` leaves it alone.
- **`bind` is the last statement of `App::build`**, after every `?` (`set_dispatcher`, `NudgeServer::start`) and every thread spawn, immediately before
  `Ok(Self { … })`: a failed build never re-targets the runtime and never spawns, so `prepare`'s and `reopen`'s error paths leave the runtime bound to the
  **old** App (L3). `bind(&self, AppBinding { events: Arc<EventHub>, health: Arc<HealthService>, previous_health: Option<Arc<HealthService>>, provider:
  Arc<dyn CameraProvider>, paths })`: (a) re-targets the runtime's event sink to the new hub; (b) **copies every camera-family health item** (code `camera`
  or `camera_*`, `camera_sidecar_unavailable` included) from `previous_health` into the new `HealthService` — the `backup_failed` precedent at
  `lib.rs:175–183`; from P46 the runtime's own `StatusInputs` + `health_items` replace this copy (L4); (c) on the first bind that finds a sidecar installed
  (`support::installed_sidecar(paths)`: the newest `<data dir>/camera/sony/<version>/camera-sidecar` that is an executable file) spawns the supervisor; a
  later bind never spawns again while a supervisor exists (M39: one sidecar process per app process, settings saves included).
- **Shutdown.** `CameraRuntime::shutdown(&self)` is public, **idempotent and terminal**: after it, no `tick` restarts the helper (L9). Two paths call it:
  `impl Drop for CameraRuntime` (tests, `kill_and_restart`: the old App's `Arc` is the last holder) and, in production, desktop `run()`'s exit hook —
  `.build(generate_context!())?.run(|handle, event| if let tauri::RunEvent::Exit = event { if let Ok(h) = handle.state::<AppState>().handles() {
  h.camera_runtime.shutdown() } })` — because tao ends the process with `process::exit` and the managed state is never dropped (L6). The kernel closing the
  child's stdin (which the sidecar turns into `quit`) is only the backstop. No row covers the exit hook (the mock runtime has no exit loop); the reviewer
  reads it. In P42 production has no sidecar installed, so the hook's effect is nil until P50.
- **`tick(&self, now: Instant)`** is serialised inside the runtime: the supervisor `Mutex` is held across draining output, exit detection and a spawn, so
  the old and the new App's background loops (both run until `old.shutdown()` joins) can never double-restart (L18). Inside one tick, drained **lines are
  handled before the exit** (L8). Lines are opaque in this phase; stderr lines go through `psm_infra::logging::redact_registered` into a 50-line ring buffer
  `helper_log()` and to `tracing::warn!`. Exit → schedule the restart at `now + RESTART_BACKOFF_S[attempts]` (then `RESTART_STEADY_S`); **consecutive
  exits** = exits since the last spawn that lived 60 s (the counter resets then, Notes (3)); health `camera_sidecar_unavailable` is set when the
  consecutive count reaches 3 and **cleared by the first stdout line of a later spawn (the counter is not reset by a line)**. Called from `App::build`'s
  background loop (`run_startup` only) and by `TestApp::tick`.
- **`Supervisor::spawn(spec)`**: `Command::new(program).args(["auto", "--save-dir", <staging>]).envs([("CRLAB_FINGERPRINT_DIR", <data dir>/camera/
  fingerprints)]).stdin(piped).stdout(piped).stderr(piped)`; `<staging>` in P42 is any directory under the data dir containing `run_id` (e.g. `<data
  dir>/camera/staging/sdk-<run_id>`), created before the spawn; P46 switches it to R-I20's `<capture root>/.incoming/sdk-<run id>/` (L16). **Credentials
  are read from the `SecretStore` once, at the first spawn, and cached in the runtime**; restarts reuse the cache (a Keychain prompt must never stall the
  background loop); P50's `camera_secrets_set` refreshes the cache (R-I18 "used at the sidecar's next spawn" holds) (L17). Before the first spawn the two
  values that will be sent (user, password) — **and only those** — are registered with `logging::add_secret`; the FTP password is registered by P43 when
  the receiver starts (L10). The **first stdin line** is `credentials user=<pct> password=<pct>` (RFC 3986 unreserved kept, everything else `%XX`); no
  secret in `args` or `env`; empty values → `credentials user= password=` (the sidecar then runs USB-only, P45). `quit_and_wait()` writes `quit\n`, waits
  `QUIT_GRACE_MS` for exit, then `kill()` + `wait()` (never a zombie). The child's exit status is observed by the supervisor's `wait`/`try_wait` only.
- `CameraService::status`/`capture` (core) apply the `capture_folder_not_set` / `document_not_front` gates **only when**
  `provider.capabilities().depends_on_editing_provider` is true (`CameraViaCaptureOne` → true; the direct adapter → false). The ok item `camera`
  (detail `<name> · cable` / `… · wireless`) is read through `health.report(..)`, since `has` sees only `!ok` items.
- `camera_retry` emits `CameraStatusChanged` through `handles.events` (the App's `EventHub`); `TauriCameraEvents` is deleted and `TauriEvents::emit` gains
  the `camera_status_changed` arm (payload = `CameraStatusResponse`) (L5). `camera_status` does not emit.
- `KeychainSecrets { service: secrets::SERVICE }` (infra, `security-framework` 3): `get` → `get_generic_password(service, account)` (`None` on
  `errSecItemNotFound`), `set` → `set_generic_password`, `delete` → `delete_generic_password`. `open`/`open_with` construct it (harmless: `p00_spike.rs:
  114,128,344`, `p05_capture_arrival.rs:435`, desktop `lib.rs:560`); **no test ever calls `get`/`set`/`delete`** because no sidecar is installed under
  those tests' data dirs; `TestApp` passes `FakeSecrets` (an in-memory map) (L23).

## The fake sidecar (this phase's minimal version; P45 adds scripted replay)
`crates/testkit/src/bin/fake_sidecar.rs`, started as `fake_sidecar auto --save-dir <dir>`. **Its recording dir is derived from its own location**:
`rec = current_exe().parent()/rec` = `<data dir>/camera/sony/test/rec/` — no test-only environment variable (`std::env::set_var` is `unsafe` under
edition 2024 and racy across parallel tests) (L2). At start it reads `<rec>/behaviour.json` (`{"exit_after_ms": null|n, "ignore_quit": bool, "stderr":
null|"text"}`; missing → defaults), then, **before** printing anything or arming the exit timer: appends its pid to `<rec>/pids`, writes its argv as a JSON
array to `<rec>/argv.<pid>.json`, its full environment as a JSON object to `<rec>/env.<pid>.json` (L19); then prints `stderr` once if set, prints
`{"timestamp":"<iso>","source":"sdk","event":"fake.started","pid":<pid>}` to stdout, and reads stdin line by line appending `<pid> <line>` to
`<rec>/stdin.log`. `exit_after_ms` exits with code 3 after the delay; `quit` (or stdin EOF) exits 0 unless `ignore_quit`. **Before any voluntary exit** it
appends `<pid> <code>` to `<rec>/exits` (L9).
`TestApp::with_camera()` = `with_camera_behaviour(json!({}))`; **`TestApp::with_camera_behaviour(behaviour: serde_json::Value)`** creates the temp data dir,
copies `env!("CARGO_BIN_EXE_fake_sidecar")` to `<data dir>/camera/sony/test/camera-sidecar` (`fs::copy` + mode 0o755 — no symlink), creates `rec/` and
writes `behaviour.json` **before** `open_with_adapter` (the first spawn, `test_app.rs:155`) (L1); `TestApp::set_fake_sidecar_behaviour(json)` rewrites
`behaviour.json` mid-test (read by the **next** spawn). It fills `FakeSecrets` with `access-auth-user = "u name"`, `access-auth-password = "p/w"`,
`ftp-password = "ftp-pw-unique-1"` (a value that appears in no row's expected text, L10). Accessors: `fake_sidecar_recording() -> Recording { pids:
Vec<u32>, argv: Vec<Vec<String>> /* spawn order */, env: Vec<Map<String, String>>, stdin: Vec<(u32, String)>, exits: Vec<(u32, i32)> }`, `camera_runtime()`.
`TestApp::tick(ms)` also calls `camera_runtime().tick(clock.mono())`.

## Ordered tasks
0. **Registry first** (master plan §6.6, §6.11): `domain/camera.rs` with the enums and `CameraCapabilities` (**all** §6.6 fields; `CameraViaCaptureOne`
   = `status, capture, transfer: true`, the rest `false`, `depends_on_editing_provider: true`, L20), `ports/secrets.rs`, `ports/camera.rs` (`capture() ->
   Result<CaptureId>`; `CameraViaCaptureOne::capture` returns `CaptureId(uuid)` after the script — the Capture One tether has no id of its own),
   `domain/editing.rs` loses `CameraStatus`/`CameraCapabilities` (re-export nothing; fix imports incl. p24 `:3`).
1. `mechanisms/camera_status.rs` with the unit tests of C5.
2. `services/camera.rs`: the capability-gated checks; health `camera` ok detail `<name> · cable` / `… · wireless`.
3. Infra: `keychain.rs`, `logging::redact_registered(line) -> String`, `paths.create` adds `camera`, `camera/fingerprints`, `camera/staging`.
4. Integrations: `camera/sidecar/supervisor.rs`, `camera/runtime.rs` (bind last, the health copy, serialised tick, terminal shutdown, the credentials
   cache), `camera/support/mod.rs::installed_sidecar`; `lib.rs` wiring (§The runtime contract); `open_with_adapter` gains `secrets: Arc<dyn SecretStore>`
   (production: `KeychainSecrets`; `open_with`/`open` pass it).
5. Desktop: `CameraStatusResponse` (§6.7 shape), the `TauriEvents` arm, `camera_retry` through `handles.events` (delete `TauriCameraEvents`), the
   `RunEvent::Exit` hook in `run()`, bindings regenerated; `CameraDot.tsx` renders every `kind` (`data-state = kind`; `connected` and `disconnected` keep
   today's title and help text; new kinds: connecting → `Connecting…`, busy → `Busy`, needs_action → `message`) (L22).
6. Testkit: `fake_sidecar.rs`, `fake_secrets.rs`, `fake_direct_camera.rs`, `TestApp::with_camera_behaviour` + accessors; the p04/p25/p24 and frontend
   mock edits.
7. Tests `p42_camera_runtime.rs` (C1–C4, C6–C8); `cargo test --workspace`, clippy, fmt, `npm run typecheck`, `npm test`.

## Criteria (8 criteria, 26 rows; 0 `[rule]` rows)
Fixture: `a = TestApp::with_camera()` unless stated; `rec()` = `a.fake_sidecar_recording()`; `alive(pid)` = `libc::kill(pid, 0) == 0` (true on a zombie
too, so a reaped pid proves the supervisor's `wait` ran); `H` = `a.app.health`. **Settling (L7):** `settle(a, pred)` = loop { `a.tick(0)`; if `pred()` →
done; sleep 10 ms } bounded at 3 s real time (panics at the bound); every **positive** read of `rec()` goes through `settle`; every **negative** read
("no second spawn", "no `quit`", "unchanged") is taken after `settle` on a 500 ms real-time timer. Clock advances are `a.tick(ms)`; a bare `a.tick(0)`
only runs the runtime at the current manual time.

| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | sidecar process count across a runtime settings rebuild | `settle` until `rec().pids.len() == 1`; `p = rec().pids`; `a.settings_set({"sku_prefix":"PX"})` (rebuilds `App`); 500 ms settle | `alive(p[0])`; after the save `rec().pids == p` (no second spawn) and `alive(p[0])`; `rec().stdin` holds no `quit` | R-I18; M39 ("one sidecar process per app process … settings saves included") |
| C1(b) | theme-only save | `a.settings_set({"theme":"dark"})`; 500 ms settle | `rec().pids` unchanged, no `quit` line | R-I18 |
| C1(c) | process exit of the holder (the `Drop` path) | `p0 = rec().pids[0]`; `a.kill_and_restart()` (builds the new App, then drops the old: two fakes exist briefly); `settle` until `rec().pids.len() == 2` and `!alive(p0)` | `rec().stdin`'s last line for `p0` is `(p0, "quit")`; `rec().exits` contains `(p0, 0)`; `alive(pids[1])` | MC-31 §Process ("Shutdown: `quit`"); M39 |
| C2(a) | quit protocol | `settle` until the pid is recorded; `a.camera_runtime().shutdown()`; `settle` until `!alive(p0)` | the fake's last stdin line is `quit`; `rec().exits == [(p0, 0)]`; the wait took < 3 s; a later `a.tick(60_000)` + 500 ms settle spawns nothing (`pids.len() == 1`; shutdown is terminal); a second `shutdown()` returns at once | MC-31 §Process |
| C2(b) | quit ignored → kill | `a = TestApp::with_camera_behaviour(json!({"ignore_quit": true}))`; `settle` until the pid is recorded; `t = Instant::now()`; `shutdown()` | `!alive(p0)` when `shutdown()` returns; `rec().exits` is empty (killed, no voluntary exit); `t.elapsed()` ≥ `QUIT_GRACE_MS` (3 s) and < 4 s | MC-31 §Process ("wait up to 3 s, then kill") |
| C3(a) | restart backoff | `a = with_camera_behaviour(json!({"exit_after_ms": 0}))`; procedure per spawn: `settle` until `rec().pids.len() == n` **and** `!alive(pids[n-1])` (the exit has been seen at the current manual time), then advance in `a.tick(100)` steps, sampling `rec().pids.len()` after a 500 ms settle at each boundary | spawns happen at manual offsets 0, 1, 3, 7, 15, 31, 61 s (cumulative `[1,2,4,8,16]` then 30); `pids.len()` after each boundary is 1, 2, 3, 4, 5, 6, 7 and is never higher between boundaries (checked at offset − 100 ms) | MC-31 §Process ("restart with bounded backoff (1, 2, 4, 8, 16 s, then every 30 s)") |
| C3(b) | helper health | as C3(a) | after the 3rd observed exit `H.has("camera_sidecar_unavailable")`; `a.set_fake_sidecar_behaviour(json!({}))` before the 4th spawn and `a.tick` past it → the item is cleared once the new process's `fake.started` line is drained (`settle` until `!H.has(..)`); the restart counter is not reset by that line: the 4th process is killed by the test (`libc::kill(pid, SIGKILL)` — our own child) and the next spawn comes 16 s later, not 1 s | MC-31 §Process; master plan §6.6 (`camera_sidecar_unavailable`); Notes (2), (3) |
| C4(a) | credentials hand-over | `settle` until `rec().stdin` has one line for pid 0 | that first line is exactly `credentials user=u%20name password=p%2Fw`; no element of `rec().argv[0]` contains `u name`, `p/w`, `u%20name` or `p%2Fw`; no value in `rec().env[0]` contains them; `rec().env[0]["CRLAB_FINGERPRINT_DIR"] == <data dir>/camera/fingerprints` (the spec's one variable); `rec().argv[0][1..] == ["auto", "--save-dir", <a path under the data dir containing run_id>]` | R-I21 ("first stdin line … percent-encoded"; "no secret appears in argv"); M39 ("a secret never appears in … argv") |
| C4(b) | stderr redaction | `a = with_camera_behaviour(json!({"stderr": "ssh password p/w refused"}))`; `settle` until `helper_log()` is non-empty | `camera_runtime().helper_log()` contains `ssh password [REDACTED] refused` exactly, and no entry contains `p/w` | R-I21 ("the sidecar's stderr reaches the log only through that redactor"); M39 ("a secret never appears in a log") |
| C5(a)–(g) | `derive_status` precedence among `NeedsAction` reasons (unit) | seven inputs, each with exactly two adjacent reasons of the R-I37 order, **the lower-ranked one first in the Vec**: `[FingerprintMismatch, AuthenticationRefused]`, `[FingerprintUnconfirmed, FingerprintMismatch]`, `[PhoneAppConnected, FingerprintUnconfirmed]`, `[HeldByOtherProgram, PhoneAppConnected]`, `[StoreDestinationUnverified, HeldByOtherProgram]`, `[FilesNotReceived{2}, StoreDestinationUnverified]`, `[SupportNotInstalled, FilesNotReceived{2}]` | each → `NeedsAction { reason: <the higher-ranked = second element>, .. }` | R-I37 (the precedence order); 10E §Port changes |
| C5(h) | `NeedsAction` over `Busy` | `needs_action: [(HeldByOtherProgram, Some("ptpcamerad"))]`, `busy: Some(Exposure)`, `session: Some(("ILCE-7RM5", Wireless))` | `NeedsAction { reason: HeldByOtherProgram, holder: Some("ptpcamerad") }` ("it wins even while connected on the other link") | R-I37; MC-34 |
| C5(i) | `Busy` over `Connected` | `busy: Some(Transfer)`, `session: Some((name, Cable))` | `Busy { reason: Transfer }` | R-I37 |
| C5(j) | `Connected` over `Connecting` | `session: Some((name, Cable))`, `connecting: Some(Wireless)` | `Connected { name, link: Cable }` | R-I37 |
| C5(k) | `Connecting` over `Disconnected` | `connecting: Some(Wireless)`, `helper_running: true`, nothing else | `Connecting { link: Wireless }`; with `connecting: None, helper_running: true` → `Disconnected` (R-I37 as reworded at `789f035`: a running sidecar reporting no connecting attempt is `Disconnected`); with `helper_running: false` → `Disconnected` | R-I37 ("`Connecting` (sidecar running, no session, and the sidecar reports it is connecting on a link) › `Disconnected`") |
| C5(l) | `health_items` is exactly the ranked list plus one base item (unit) | `needs_action: [(FilesNotReceived{3}, None), (FingerprintUnconfirmed, None)]`, `session: Some((name, Wireless))` | `health_items(..)` **equals** `[("camera_fingerprint_unconfirmed", false, "Confirm the camera's fingerprint"), ("camera_files_not_received", false, "3 photos taken but not received for 60 s"), ("camera", true, "<name> · wireless")]` (needs-action items by rank, then exactly one base item); `derive_status` → `NeedsAction { FingerprintUnconfirmed }` | R-I37 ("the health screen lists every reason that holds"); master plan §6.6 |
| C6(a) | `CameraService` independent of Capture One's health | `FakeDirectProvider` (`crates/testkit/src/fake_direct_camera.rs`: `depends_on_editing_provider: false`, status `Connected { "X", Wireless }`); `H.set("capture_folder_not_set", …)`; `CameraService::new(&p, &H).status()` and `.capture()` | `Ok(Connected { name: "X", link: Wireless })`; `capture()` → `Ok(CaptureId(_))`; `H.report("", false, false).items` contains `{ code: "camera", ok: true, detail: "X · wireless" }` | R-I33; 10A MC-18 step 5 (direct adapter clause) |
| C6(b) | the Capture One camera adapter keeps its gates | `TestApp::with_fake_document()`; the gate set through an activation failure as p04 does | `status()` → `Disconnected`; `capture()` → `Err(CaptureFolderNotSet)` (p04's rows at `:89`, `:111` keep passing unchanged and are the tests of this row) | R-I33 ("those gates stay only for the Capture One camera adapter"); M25 |
| C7(a) | camera health survives a rebuild (the copy) | `fake.state.camera = None`; `a.app.health_check()` (sets `camera_disconnected`); `a.settings_set({"sku_prefix":"PX"})`; read `a.app.health.report(..)` **without** another check | the new `HealthService` holds `camera_disconnected` with the same detail (copied by `bind` from the previous health); with `fake.state.camera = Some("Sony ZV-E10")` and a check before the save, the new health holds the ok item `camera` instead | R-I18 ("hands the camera runtime to the new runtime unchanged"); R-I37 |
| C7(b) | runtime identity across rebuilds | `id0 = a.app.handles().camera_runtime.run_id()`; `settings_set` (rebuild); `a.app.reopen()`; 500 ms settle | both reads equal `id0`; `rec().pids.len() == 1` | R-I18 |
| C8(a) | `camera_status` and `camera_retry` commands (mock webview) | `with_fake_document`; `handles.events.set_target(Arc::new(TauriEvents(handle)))` and `listen_any("camera_status_changed", ..)` (the `c7c` pattern); `camera_status`; then `fake.state.camera = None`; `camera_retry` | `camera_status` → `{"kind":"connected","name":"Sony ZV-E10","link":"cable"}` and **no** event; `camera_retry` → `{"kind":"disconnected"}` and exactly one `camera_status_changed` payload `{"kind":"disconnected"}` received through the hub (fails if `camera_retry` bypasses the hub or the `TauriEvents` arm is missing) | master plan §6.7; 10E §Port changes |

## Delegated choices
Module layout of `runtime.rs` (one file or a folder); how `tick` is scheduled on the background loop (every iteration is fine); the ring-buffer type;
whether the supervisor's reader threads use `std::sync::mpsc` or a `Mutex<Vec>`; the exact `Display` of `camera_sidecar_unavailable`'s detail beyond
the §6.6 text; the fake sidecar's ISO timestamp precision. **Explicit delegations from the projection:** (L15) `status_message` for `HeldByOtherProgram`
in P42 is the placeholder `<holder> is holding the camera` (`Another program is holding the camera` when `holder` is `None`); P48 replaces it; not
asserted here. (L16) the `--save-dir` value: any directory under the data dir containing `run_id`, created before the spawn (P46 switches to R-I20).
(L20) `CameraCapabilities` carries all §6.6 fields now (values in task 0). (L21) `installed_sidecar`'s "newest" is lexical order of the version folder
names (tests have one folder; P50 replaces it). (L22) `CameraDot`: `data-state = kind`; `connected` and `disconnected` keep today's title and help text
(`p07_shell_sessions_ui.test.tsx:188–205`); the new kinds' texts as task 5.

## Manual try-out
(The direct adapter does not exist yet; this phase changes nothing the photographer can see except the camera dot's wording.)
1. Start the app with Capture One and the ZV-E10 tethered as before: the header dot is green and reads the camera name, as it did.
2. Unplug the camera, click the dot: it turns red and says "Camera disconnected" (the retry still works).
3. Change a setting (Settings → SKU prefix) and save: the app keeps running; the Health page still shows the camera row.

## Notes
- **Hard rules (owner):** never commit a Sony SDK file, the lab's `sdk_shim` headers, a `.env`, a credential or an SSH fingerprint (public repo); no
  Python; never change the camera's network/FTP settings, delete a camera file or kill `ptpcamerad` or any other process (the supervisor kills only
  **its own child** after the 3 s grace, MC-31; C3(b)'s test kills its own fake); live view out of scope; the suite never needs the camera. No `implementer: fable`.
- **Planner readings (reversible; for the narrative):** (1) `CameraViaCaptureOne` reports `link: Cable` (the Capture One tether is USB). (2) The
  helper-health item appears after three consecutive crashes, not the first (a single crash is recovered in 1 s). (3) The backoff counter resets
  once a spawn has stayed alive 60 s. (4) `CaptureId` for the Capture One adapter is a fresh UUID (nothing else identifies that capture). (5) R-I21
  supersedes MC-31 §Secrets' former "through its environment" (folded at `789f035`).
- **Hazards:** `App::build` is called **before** `old.shutdown()` in `prepare` (`lib.rs:164–190`), so the runtime must be an `Arc` shared between the
  two Apps, never rebuilt; `Drop for App` calls `shutdown` (`:931`), which must not touch the runtime. `bind` **last** in `build` (L3), or a failed
  `set_dispatcher`/`NudgeServer::start` leaves camera events pointing at a dropped App. `tick` is serialised (L18): two background loops run during a
  rebuild. `kill -0` on a reaped pid: the supervisor must `wait()` the child after `kill()` so C2(b) sees the pid gone, not a zombie. `env!("CARGO_BIN_EXE_
  fake_sidecar")` exists only in psm-testkit's own integration tests — every test using the fake lives in `crates/testkit/tests/`. The fake reads
  `behaviour.json` **at start**, so C3(b) rewrites it between spawns with `set_fake_sidecar_behaviour`. `security-framework` functions block on a Keychain
  prompt — constructing `KeychainSecrets` is harmless (L23), calling it from a test is not: no test's data dir holds a sidecar, so none spawns. The real
  app quit never drops `AppState` (tao `process::exit`): the `RunEvent::Exit` hook is the production shutdown path (L6). `camera_retry` must emit through
  the App's `EventHub`, or C8(a) fails. Only the two sent credentials are registered with the redactor in P42 (L10), or C4(b)'s text changes.
- **Projection** done (round 0, `handoff_P42_camera_runtime_projection_r0.md`; amendments folded below).
- Registry additions made by the planner in master plan §6.1, §6.2, §6.3, §6.4, §6.6, §6.7, §6.11, §7.5. Intention wording folds L13/L14 by the
  orchestrator at `789f035`.
- Estimated size: ~45 paths, ~1 600 lines (≈ 850 core/infra/integrations/desktop, ≈ 750 tests + fake).

## Review log

- **2026-11-06, orchestrator amendments after projection r0** (`handoffs/reviewer/handoff_P42_camera_runtime_projection_r0.md`, folded by the planner;
  `projection: yes` → `done`). L1 → `TestApp::with_camera_behaviour(json)` writes `behaviour.json` before the first spawn, `set_fake_sidecar_behaviour(json)`
  for mid-test rewrites; C2(b), C3(a), C3(b), C4(b) reworded. L2 → the fake derives its recording dir from `current_exe().parent()/rec`; `fs::copy` + 0o755
  pinned, symlink option removed; C4(a) checks `CRLAB_FINGERPRINT_DIR` instead of a test variable. L3 → `bind` is the last statement of `App::build`; the
  three callers named. L4 → `bind` copies the camera-family health items from the previous `HealthService` (the `backup_failed` precedent); C7(a) asserts
  the copy; P46 replaces it with the runtime's own `health_items`. L5 → `camera_retry` emits through `handles.events`, `TauriCameraEvents` deleted; C8(a)
  sets the hub target and asserts exactly one event; the P42 scope (only `camera_retry` emits) stated in Goal. L6 → the `RunEvent::Exit` hook route (as
  directed by the orchestrator): desktop `run()` calls `camera_runtime.shutdown()` on exit; `Drop` stays for tests; no new row, reviewer reads it. L7 →
  `settle(a, pred)` fixture; positive reads settle, negative reads after a 500 ms settle; C3(a)'s reap-then-advance procedure. L8 → consecutive exits =
  since the last spawn that lived 60 s; a stdout line clears the item but does not reset the counter (C3(b) asserts the next spawn is 16 s later); lines
  before exit inside a tick; the fake records before arming the exit. L9 → `<rec>/exits` with `(pid, code)`; `shutdown` public, idempotent, terminal
  (C2(a) asserts no restart after it). L10 → FTP password fixture `ftp-pw-unique-1`; P42 registers only the two sent values; master plan §7.5 aligned
  (`u name` / `p/w` / `ftp-pw-unique-1`). L11 → C5(a)–(g) list the lower-ranked reason first; `AuthenticationRefused` named. L12 → C5(l) asserts equality
  with the ranked list plus one base item. L13, L14 → folded upstream by the orchestrator (`789f035`); Notes (5) and C5(k) cite them. L15, L16, L20,
  L21, L22 → explicit lines in §Delegated choices. L17 → credentials read once at the first spawn and cached; P50 refreshes. L18 → `tick` serialised
  under the supervisor `Mutex` (contract + Hazards). L19 → `argv.<pid>.json`, `Recording.argv: Vec<Vec<String>>`. L23 → the Hazard reworded ("no test
  calls get/set/delete"). Reality checks → every wrong citation corrected (logging, paths, editing, events, state, test_app, fake); p24 `:3` added to the
  adjusted tests; `crates/testkit/src/fake_direct_camera.rs` added; `libc` on testkit; counts re-derived (11/22/1/3/8 = 45); `has` vs `report` in C6(a).
  Rows: 26 → 26 (none added or removed; 14 reworded). Nothing declined. Carried to P43–P46 and P50 (their Review logs): the settle fixture (L7), the
  distinctive secret fixtures (L10), the credential cache (L17), tick serialisation (L18), the fake's recording dir and argv shape (L2, L19), and
  `bind`-last (L3).
