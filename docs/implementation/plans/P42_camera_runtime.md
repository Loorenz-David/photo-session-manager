---
phase: P42_camera_runtime
title: Camera runtime — one camera layer per app process (sidecar supervisor over a fake sidecar, secrets port, the new CameraStatus and its precedence), carried unchanged across settings rebuilds
difficulty: high
projection: yes
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
(P51). The camera provider in use stays `CameraViaCaptureOne`.

## Read first
Book: `10E_direct_camera_adapter.md` §Port changes (status values, R-I33, R-I37), MC-31 §Process and §Secrets + inventory additions **R-I18**,
**R-I21**, M39 · `11_architecture_and_packaging.md` (one camera runtime per process) · `12_operations.md` (Keychain) · `14_measurement_ledger.md`
M39 · `01_objective_and_constraints.md` H7, H8 · process `flow.md` §4, §6, §9 · master plan §3 (camera hard rules), §6.1, §6.2 (camera crates),
§6.3 (`camera/` under the data dir), §6.4, §6.6, §6.7 (`camera_status` DTO, `camera_status_changed`), **§6.11** (Constants, Domain, Ports,
Mechanisms `camera_status.rs`, Runtime, Sidecar `Supervisor`), §7.5 (`with_camera`).
Code at `41bef14` (line numbers as of that commit):
- `crates/integrations/src/lib.rs` — `App` `:73–106`, `AppHandles` `:108–128`, `SettingsSetContext` `:130–140` and `prepare` `:147–191` (**the new
  runtime is built at `:164–173` before the old is shut down**), `apply_settings` `:274–292`, `handles` `:293–314`, `open_with_adapter` `:330–349`,
  `build` `:351–709` (camera provider `:431–434`, `events` `:435`, startup thread `:565–582` incl. `CameraService::status` `:579–580`, background loop
  `:607–649`), `settings_set` `:740–747`, `shutdown` `:750–761`, `reopen` `:764–783`, `EventHub` `:809–848`, `Drop for App` `:931–935`.
- `crates/core/src/ports/camera.rs:1–11` (`CameraProvider`), `crates/core/src/services/camera.rs:1–62` (the Capture One gates `:28–30`, `:52–58`),
  `crates/core/src/domain/editing.rs:725–736` (`CameraStatus`, `CameraCapabilities` — **move** to `domain/camera.rs`), `crates/core/src/events.rs:787–789`.
- `crates/integrations/src/camera/via_capture_one.rs:1–35`; `apps/desktop/src-tauri/src/commands/camera.rs:1–90` (DTO `:11–23`, `TauriCameraEvents`
  `:24–33`); `apps/desktop/src-tauri/src/lib.rs` `TauriEvents::emit` (**its `_ => {}` arm swallows `CameraStatusChanged` today** — add the arm);
  `apps/desktop/src-tauri/src/state.rs:922–938` (`settings_set` holds `save`, prepares, mutates, shuts the old down).
- `crates/infra/src/logging.rs:380–391` (`add_secret`), `:394–401` (`redact`), `:405–407`; `crates/infra/src/paths.rs:527–535` (`create`).
- `crates/testkit/src/test_app.rs:117–175` (`create`), `:182–196` (`restart`: drops the old `App`), `:218–226` (`tick`); `crates/testkit/src/fake_capture_one.rs:114`
  (`camera: Some("Sony ZV-E10")`).
- Frontend: `apps/desktop/src/features/shell/CameraDot.tsx` (whole), `apps/desktop/src/services/camera.ts`, `apps/desktop/src/services/events.ts:12–14`.
- **Tests that change their expectation:** `crates/testkit/tests/p04_capture_one_adapter.rs:34` (`status` helper type), `:404`
  (`CameraStatus::Connected("Sony ZV-E10".into())` → `CameraStatus::Connected { name: "Sony ZV-E10".into(), link: Link::Cable }`),
  `crates/testkit/tests/p25_tidy_up_after_provisioning.rs:127` (same); frontend mocks returning `{ kind: 'connected', name }` gain `link: 'cable'` in
  `apps/desktop/src/features/shell/__tests__/p19_navigation.test.tsx`, `features/workspace/__tests__/{p19_drop, p08_workspace_ui, p22_workspace_tidy,
  p20_drop_continuity, p26_double_drag}.test.tsx`, `features/compose/__tests__/harness.tsx`, `features/sessions/__tests__/p07_shell_sessions_ui.test.tsx`
  (grep `kind: 'connected'`). No other test changes.
- Lab (read-only): `/Users/davidloorenz/Desktop/Developer/Camera-Connection-Tests/docs/session-manager-handoff.md` §4.1 (process contract: stdin
  commands, stdout events, `quit`), `sdk/src/main.cpp:140–160` (`StdinQueue`: EOF pushes `quit`).

## Files expected to change
Core (new): `crates/core/src/domain/camera.rs`, `crates/core/src/mechanisms/camera_status.rs`, `crates/core/src/ports/secrets.rs`. Core (changed):
`crates/core/src/domain/{mod.rs, editing.rs}`, `crates/core/src/mechanisms/mod.rs`, `crates/core/src/ports/{mod.rs, camera.rs}`,
`crates/core/src/services/camera.rs`, `crates/core/src/events.rs`.
Infra (new): `crates/infra/src/keychain.rs`. Infra (changed): `crates/infra/src/{lib.rs, logging.rs, paths.rs}`, `crates/infra/Cargo.toml`.
Integrations (new): `crates/integrations/src/camera/runtime.rs`, `crates/integrations/src/camera/sidecar/{mod.rs, supervisor.rs}`,
`crates/integrations/src/camera/support/mod.rs` (the `installed_sidecar(paths)` seed). Integrations (changed): `crates/integrations/src/camera/{mod.rs,
via_capture_one.rs}`, `crates/integrations/src/lib.rs`, `crates/integrations/Cargo.toml` (uuid).
Desktop (changed): `apps/desktop/src-tauri/src/commands/camera.rs`, `apps/desktop/src-tauri/src/lib.rs`, `apps/desktop/src/services/bindings.ts`
(regenerated), `apps/desktop/src/features/shell/CameraDot.tsx` (minimal: the five `kind`s render; texts final in P51), the 7 frontend test files above.
Testkit (new): `crates/testkit/src/bin/fake_sidecar.rs`, `crates/testkit/src/fake_secrets.rs`. Testkit (changed): `crates/testkit/src/{lib.rs,
test_app.rs}`, `crates/testkit/Cargo.toml` (`[[bin]] fake_sidecar`). Tests (new): `crates/testkit/tests/p42_camera_runtime.rs`. Tests (changed):
`crates/testkit/tests/{p04_capture_one_adapter.rs, p25_tidy_up_after_provisioning.rs}`.
Estimated: 8 new source paths, 19 changed, 1 new test, 2 adjusted Rust tests, 7 adjusted frontend tests — 37 paths.

## Core mechanisms (unit tests allowed)
`crates/core/src/mechanisms/camera_status.rs` — `derive_status`, `NeedsActionReason::rank`, `health_items`, `status_message`: inline `#[cfg(test)]`
tests named `mc31_<slug>` (rows C5(a)–(l)). Everything else is a behaviour row.

## The runtime contract (master plan §6.11 names)
- `CameraRuntime::new(secrets: Arc<dyn SecretStore>, clock: Arc<dyn Clock>, paths: AppPaths) -> Arc<Self>`; `run_id` = a UUID v4 string.
  `App::open_with_adapter` creates it; **`App::build` takes it as a parameter** and stores `camera_runtime: Arc<CameraRuntime>` on `App`,
  `AppHandles` and `SettingsSetContext`; `prepare` (`lib.rs:164`) and `reopen` (`:765`) pass the same `Arc`. `App::shutdown` leaves it alone;
  `impl Drop for CameraRuntime` is what quits the sidecar (so a dropped `App` that was the last holder — a test's `kill_and_restart`, the real
  process exit — stops the helper, and a settings rebuild, during which the old and the new `App` both hold the `Arc`, never does).
- `bind(&self, AppBinding { events: Arc<EventHub>, health: Arc<HealthService>, provider: Arc<dyn CameraProvider>, paths })` in every `App::build`
  after `events`/`health` exist: re-targets the runtime's event sink to the new hub, re-publishes the current `CameraStatus` into the new
  `HealthService` through `camera_status::health_items`, and **on the first bind that finds a sidecar installed** (`support::installed_sidecar(paths)`:
  the newest `<data dir>/camera/sony/<version>/camera-sidecar` that is an executable file) spawns the supervisor; a later bind never spawns again
  while a supervisor exists (M39: one sidecar process per app process, settings saves included).
- `tick(&self, now: Instant)`: drains supervisor output (lines are opaque in this phase; stderr lines go through `psm_infra::logging::redact_registered`
  into a 50-line ring buffer `helper_log()` and to `tracing::warn!`), detects exit (`Child::try_wait`), schedules the restart (`RESTART_BACKOFF_S` then
  `RESTART_STEADY_S`; the counter resets when a spawn has lived 60 s), sets health `camera_sidecar_unavailable` after 3 consecutive exits and clears
  it on the first stdout line of a later spawn. Called from `App::build`'s background loop (`run_startup` only) and by `TestApp::tick`.
- `Supervisor::spawn(spec)`: `Command::new(program).args(args).envs(env).stdin(piped).stdout(piped).stderr(piped)`; before spawning, every secret
  that will be sent is registered with `logging::add_secret`; the **first stdin line** is `credentials user=<pct> password=<pct>` (RFC 3986 unreserved
  characters kept, everything else `%XX`); no secret in `args` or `env`; `quit_and_wait()` writes `quit\n`, waits `QUIT_GRACE_MS` for exit, then
  `kill()`. Credentials are read from the `SecretStore` at each spawn (`ACCESS_AUTH_USER`, `ACCESS_AUTH_PASSWORD`; empty values → the line is
  `credentials user= password=`; the sidecar then runs USB-only, P45).
- `CameraService::status`/`capture` (core) apply the `capture_folder_not_set` / `document_not_front` gates **only when**
  `provider.capabilities().depends_on_editing_provider` is true (`CameraViaCaptureOne` → true; the direct adapter → false).
- `KeychainSecrets { service: secrets::SERVICE }` (infra, `security-framework` 3): `get` → `get_generic_password(service, account)` (`None` on
  `errSecItemNotFound`), `set` → `set_generic_password`, `delete` → `delete_generic_password`. **Not exercised by the suite** (it would touch the
  owner's login keychain); exercised by P47's spike. `FakeSecrets` (testkit) is an in-memory map.

## The fake sidecar (this phase's minimal version; P45 adds scripted replay)
`crates/testkit/src/bin/fake_sidecar.rs`, started as `fake_sidecar auto --save-dir <dir>` with env `FAKE_SIDECAR_DIR=<recording dir>` (not a secret):
on start it appends its pid to `<rec>/pids`, writes its argv as one line to `<rec>/argv`, its full environment as JSON to `<rec>/env.<pid>.json`,
prints `{"timestamp":"<iso>","source":"sdk","event":"fake.started","pid":<pid>}` to stdout, then reads stdin line by line appending `<pid> <line>`
to `<rec>/stdin.log`. Behaviour comes from `<rec>/behaviour.json` read at start: `{"exit_after_ms": null|n, "ignore_quit": bool, "stderr": null|"text"}`
— `stderr` is printed to stderr once at start; `exit_after_ms` exits with code 3 after the delay; `quit` (or stdin EOF) exits 0 unless `ignore_quit`.
`TestApp::with_camera()` copies `env!("CARGO_BIN_EXE_fake_sidecar")` to `<data dir>/camera/sony/test/camera-sidecar`, writes a default
`behaviour.json`, fills `FakeSecrets` (`u name` / `p/w` / `f`), and exposes `fake_sidecar_recording() -> Recording { pids: Vec<u32>, argv: Vec<String>,
env: Vec<Map>, stdin: Vec<(u32, String)> }` and `camera_runtime()`. `TestApp::tick(ms)` also calls `camera_runtime().tick(clock.mono())`.

## Ordered tasks
0. **Registry first** (master plan §6.6, §6.11): `domain/camera.rs` with the enums and `CameraCapabilities` (field `depends_on_editing_provider`),
   `ports/secrets.rs`, `ports/camera.rs` (`capture() -> Result<CaptureId>`; `CameraViaCaptureOne::capture` returns `CaptureId(uuid)` after the
   script — the Capture One tether has no id of its own), `domain/editing.rs` loses `CameraStatus`/`CameraCapabilities` (re-export nothing; fix imports).
1. `mechanisms/camera_status.rs` with the unit tests of C5.
2. `services/camera.rs`: the capability-gated checks; health `camera` ok detail `<name> · cable` / `… · wireless`.
3. Infra: `keychain.rs`, `logging::redact_registered(line) -> String`, `paths.create` adds `camera`, `camera/fingerprints`.
4. Integrations: `camera/sidecar/supervisor.rs`, `camera/runtime.rs`, `camera/support/mod.rs::installed_sidecar`; `lib.rs` wiring (task text in §The
   runtime contract); `open_with_adapter` gains `secrets: Arc<dyn SecretStore>` (production: `KeychainSecrets`; `open_with`/`open` pass it).
5. Desktop: `CameraStatusResponse` (§6.7 shape), `TauriEvents` arm for `camera_status_changed`, bindings regenerated; `CameraDot.tsx` renders every
   `kind` (text: connected → `<name>`, connecting → `Connecting…`, busy → `Busy`, needs_action → `message`, disconnected → `Camera disconnected`).
6. Testkit: `fake_sidecar.rs`, `fake_secrets.rs`, `TestApp::with_camera` + accessors; the p04/p25 and frontend mock edits.
7. Tests `p42_camera_runtime.rs` (C1–C4, C6–C8); `cargo test --workspace`, clippy, fmt, `npm run typecheck`, `npm test`.

## Criteria (8 criteria, 26 rows; 0 `[rule]` rows)
Fixture: `a = TestApp::with_camera()` unless stated; `rec()` = `a.fake_sidecar_recording()`; `alive(pid)` = `kill(pid, 0)` succeeds (`libc::kill`
through a testkit helper); `H` = `a.app.health`. Clock: `a.tick(ms)`.

| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | sidecar process count across a runtime settings rebuild | after `with_camera()`: `a.tick(0)`; `p = rec().pids`; `a.settings_set({"sku_prefix":"PX"})` (rebuilds `App`); `a.tick(0)` | `p.len() == 1` and `alive(p[0])`; after the save `rec().pids == p` (no second spawn) and `alive(p[0])`; `rec().stdin` holds no `quit` | R-I18; M39 ("one sidecar process per app process … settings saves included") |
| C1(b) | theme-only save | `a.settings_set({"theme":"dark"})`; `a.tick(0)` | `rec().pids` unchanged, no `quit` line | R-I18 |
| C1(c) | process exit of the holder | `p0 = rec().pids[0]`; `a.kill_and_restart()`; `a.tick(0)` | `rec().stdin` has `(p0, "quit")` as the last line of `p0`; `!alive(p0)` within 3 s; `rec().pids.len() == 2` and `alive(pids[1])` | MC-31 §Process ("Shutdown: `quit`"); M39 |
| C2(a) | quit protocol | `a.camera_runtime().shutdown()` (what `Drop` does) | the fake's last stdin line is `quit`; the process exits with code 0 within 3 s (observed through `rec().pids` + `!alive`) | MC-31 §Process |
| C2(b) | quit ignored → kill | `behaviour.json {"ignore_quit": true}` before `with_camera()`; `shutdown()` | the process is gone after `shutdown()` returns, and `shutdown()` took ≥ `QUIT_GRACE_MS` (3 s) and < 4 s | MC-31 §Process ("wait up to 3 s, then kill") |
| C3(a) | restart backoff | `behaviour.json {"exit_after_ms": 0}`; `a.tick(0)` repeatedly in 100 ms steps up to 70 s of manual time, sampling `rec().pids.len()` | spawns happen at offsets 0, 1, 3, 7, 15, 31, 61 s (= cumulative `[1,2,4,8,16]` then 30); the count after each boundary is 1, 2, 3, 4, 5, 6, 7 and never higher between boundaries | MC-31 §Process ("restart with bounded backoff (1, 2, 4, 8, 16 s, then every 30 s)") |
| C3(b) | helper health | as C3(a) | after the 3rd exit `H.has("camera_sidecar_unavailable")`; then `behaviour.json {"exit_after_ms": null}` written before the next spawn and `a.tick` past it → the item is cleared once the new process printed its first line | MC-31 §Process; master plan §6.6 (`camera_sidecar_unavailable`) |
| C4(a) | credentials hand-over | `rec()` after the first spawn | the first stdin line of pid 0 is exactly `credentials user=u%20name password=p%2Fw`; no element of `rec().argv[0]` contains `u name`, `p/w`, `u%20name` or `p%2Fw`; no value in `rec().env[0]` contains them; `rec().env[0]["FAKE_SIDECAR_DIR"]` exists (the recording dir is the only test-only variable) | R-I21 ("first stdin line … percent-encoded"; "no secret appears in argv"); M39 ("a secret never appears in … argv") |
| C4(b) | stderr redaction | `behaviour.json {"stderr": "ssh password p/w refused"}`; `a.tick(0)` until `helper_log()` is non-empty | `camera_runtime().helper_log()` contains `ssh password [REDACTED] refused` and no entry contains `p/w` | R-I21 ("the sidecar's stderr reaches the log only through that redactor"); M39 ("a secret never appears in a log") |
| C5(a)–(g) | `derive_status` precedence among `NeedsAction` reasons (unit) | seven inputs, each with exactly two adjacent reasons of the R-I37 order present: (Auth, FingerprintMismatch), (FingerprintMismatch, FingerprintUnconfirmed), (FingerprintUnconfirmed, PhoneAppConnected), (PhoneAppConnected, HeldByOtherProgram), (HeldByOtherProgram, StoreDestinationUnverified), (StoreDestinationUnverified, FilesNotReceived{2}), (FilesNotReceived{2}, SupportNotInstalled) | each → `NeedsAction { reason: <the first of the pair>, .. }` | R-I37 (the precedence order); 10E §Port changes |
| C5(h) | `NeedsAction` over `Busy` | `needs_action: [(HeldByOtherProgram, Some("ptpcamerad"))]`, `busy: Some(Exposure)`, `session: Some(("ILCE-7RM5", Wireless))` | `NeedsAction { reason: HeldByOtherProgram, holder: Some("ptpcamerad") }` ("it wins even while connected on the other link") | R-I37; MC-34 |
| C5(i) | `Busy` over `Connected` | `busy: Some(Transfer)`, `session: Some((name, Cable))` | `Busy { reason: Transfer }` | R-I37 |
| C5(j) | `Connected` over `Connecting` | `session: Some((name, Cable))`, `connecting: Some(Wireless)` | `Connected { name, link: Cable }` | R-I37 |
| C5(k) | `Connecting` over `Disconnected` | `connecting: Some(Wireless)`, `helper_running: true`, nothing else | `Connecting { link: Wireless }`; with `connecting: None, helper_running: true` → `Disconnected`; with `helper_running: false` → `Disconnected` | R-I37 ("Connecting (sidecar running, no session) › Disconnected") |
| C5(l) | `health_items` lists every reason (unit) | `needs_action: [(FilesNotReceived{3}, None), (FingerprintUnconfirmed, None)]`, `session: Some((name, Wireless))` | items contain `camera_fingerprint_unconfirmed` (`Confirm the camera's fingerprint`), `camera_files_not_received` (`3 photos taken but not received for 60 s`) and the ok item `camera` (`<name> · wireless`), in the precedence order; `derive_status` → `NeedsAction { FingerprintUnconfirmed }` | R-I37 ("the health screen lists every reason that holds"); master plan §6.6 |
| C6(a) | `CameraService` independent of Capture One's health | a `FakeDirectProvider` (testkit, `depends_on_editing_provider: false`, status `Connected { "X", Wireless }`); `H.set("capture_folder_not_set", …)`; `CameraService::new(&p, &H).status()` and `.capture()` | `Ok(Connected { name: "X", link: Wireless })`; `capture()` → `Ok(CaptureId(_))`; `H` gains the ok item `camera` with detail `X · wireless` | R-I33; 10A MC-18 step 5 (direct adapter clause) |
| C6(b) | the Capture One camera adapter keeps its gates | `TestApp::with_fake_document()`; `H.set("capture_folder_not_set", …)` | `status()` → `Disconnected`; `capture()` → `Err(CaptureFolderNotSet)` (p04's `c…` rows at `:89`, `:111` keep passing unchanged and are the tests of this row) | R-I33 ("those gates stay only for the Capture One camera adapter"); M25 |
| C7(a) | camera health survives a rebuild | `fake.state.camera = None`; `a.app.health_check()`; `a.settings_set({"sku_prefix":"PX"})`; read `a.app.health.report(…)` **without** another check | the new `HealthService` holds `camera_disconnected` (re-published by `bind`) | R-I18 ("hands the camera runtime to the new runtime unchanged"); R-I37 |
| C7(b) | runtime identity across rebuilds | `id0 = a.app.handles().camera_runtime.run_id()`; `settings_set` (rebuild); `App::reopen()` | both reads equal `id0`; `rec().pids.len() == 1` | R-I18 |
| C8(a) | `camera_status` command (mock webview, `lib.rs p39_tests` pattern) | `with_fake_document`; `camera_status`; then `fake.state.camera = None`; `camera_retry` | `{"kind":"connected","name":"Sony ZV-E10","link":"cable"}`; then `{"kind":"disconnected"}` and a `camera_status_changed` event with that payload received by `listen_any` (the `TauriEvents` arm) | master plan §6.7; 10E §Port changes |

## Delegated choices
Module layout of `runtime.rs` (one file or a folder); how `tick` is scheduled on the background loop (every iteration is fine); the ring-buffer type;
whether the supervisor's reader threads use `std::sync::mpsc` or a `Mutex<Vec>`; the exact `Display` of `camera_sidecar_unavailable`'s detail beyond
the §6.6 text; the backoff-reset rule (60 s alive, planner reading, Notes (3)); the fake sidecar's ISO timestamp precision; how `TestApp::with_camera`
makes the fake executable (`fs::copy` + mode 0o755 or a symlink).

## Manual try-out
(The direct adapter does not exist yet; this phase changes nothing the photographer can see except the camera dot's wording.)
1. Start the app with Capture One and the ZV-E10 tethered as before: the header dot is green and reads the camera name, as it did.
2. Unplug the camera, click the dot: it turns red and says "Camera disconnected" (the retry still works).
3. Change a setting (Settings → SKU prefix) and save: the app keeps running; the Health page still shows the camera row.

## Notes
- **Hard rules (owner):** never commit a Sony SDK file, the lab's `sdk_shim` headers, a `.env`, a credential or an SSH fingerprint (public repo); no
  Python; never change the camera's network/FTP settings, delete a camera file or kill `ptpcamerad` or any other process (the supervisor kills only
  **its own child** after the 3 s grace, MC-31); live view out of scope; the suite never needs the camera. No `implementer: fable`.
- **Planner readings (reversible; for the narrative):** (1) `CameraViaCaptureOne` reports `link: Cable` (the Capture One tether is USB). (2) The
  helper-health item appears after three consecutive crashes, not the first (a single crash is recovered in 1 s). (3) The backoff counter resets
  once a spawn has stayed alive 60 s. (4) `CaptureId` for the Capture One adapter is a fresh UUID (nothing else identifies that capture).
- **Hazards:** `App::build` is called **before** `old.shutdown()` in `prepare` (`lib.rs:164–190`), so the runtime must be an `Arc` shared between the
  two Apps, never rebuilt; `Drop for App` calls `shutdown` (`:931`), which must not touch the runtime. `TauriEvents::emit`'s `_ => {}` (desktop `lib.rs`)
  silently drops `CameraStatusChanged` today — C8(a) fails without the new arm. `kill -0` on a reaped pid: the supervisor must `wait()` the child
  after `kill()` so C2(b) sees the pid gone, not a zombie. `env!("CARGO_BIN_EXE_fake_sidecar")` exists only in psm-testkit's own integration tests —
  every test using the fake lives in `crates/testkit/tests/`. The fake must read `behaviour.json` **at start**, so C3(b) rewrites it between spawns.
  `security-framework` functions block on a Keychain prompt — `KeychainSecrets` is never constructed in tests (`TestApp` passes `FakeSecrets`).
- **Projection** is on: R-I18 is the mechanism the inventory ranked first for silent failure (two sidecars, a dead second listener); the projectionist
  checks every `App::build` call site passes the same `Arc`, that `prepare`'s failure path (`lib.rs:186–189`) leaves the runtime bound to the **old**
  App, and that the fake-sidecar recording files make every C1–C4 row decidable.
- Registry additions made by the planner in master plan §6.1, §6.2, §6.3, §6.4, §6.6, §6.7, §6.11, §7.5. No intention-book change.
- Estimated size: ~37 paths, ~1 500 lines (≈ 800 core/infra/integrations/desktop, ≈ 700 tests + fake).

## Review log

