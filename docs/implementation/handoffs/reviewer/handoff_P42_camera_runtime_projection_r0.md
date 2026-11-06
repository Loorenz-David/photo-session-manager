---
plan: P42_camera_runtime
role: projection
round: 0
verdict: AMENDMENTS_REQUIRED
date: 2026-11-06
actor: Claude Opus 5.5 (pipeline-projectionist, plan-projection doctrine under docs/process/flow.md)
---
# Projection of P42_camera_runtime (round 0)

The plan's design holds up: one camera layer per app process, shared across settings saves, is the right shape, and every place in the code
that rebuilds the app can be given that same shared layer. But as written, four of the 26 test rows cannot be set up at all, because the
fake camera helper starts before a test can tell it how to behave. Several more rows are racy, or would pass even when the code is wrong.
Two production gaps also need closing before an implementer starts. On a real app quit, the planned "stop the helper on drop" never runs.
And a failed settings save could leave camera messages pointing at a part of the app that no longer exists. None of this needs you: every
item is a paragraph-sized plan fix for the orchestrator, plus one wording fix in the book. After that, the implementer prompt can be compiled.

## ⚠ OWNER DECISIONS REQUIRED (0)
Nothing needs the owner.

## Gate
Intention `docs/intention/intention.md:3` **RATIFIED**; 10E header RATIFIED (changelog 31); inventory handoff
`handoff_direct_camera_inventory_r1.md` state DONE (card CI1 answered, R46); planning handoff `handoff_direct_camera_plan_r1.md` DONE, 0 cards.
Code at HEAD `2550e09` is identical to the plan's `41bef14` under `crates/`, `apps/`, `tools/` (`git diff --stat 41bef14 HEAD` empty).
Observation, not a gate failure: R-I18…R-I38 are recorded "pending owner ratification as a list" (changelog 32). P42 rests on R-I18, R-I21,
R-I33 and R-I37. The project proceeds on that basis, and the list is already in front of the owner in the inventory handoff.

## Decision ledger

| # | Decision point | Classification | Proposed routing |
|---|---|---|---|
| L1 | Rows C2(b), C3(a), C3(b) and C4(b) need `behaviour.json` in place **before the first spawn**. But the first spawn happens inside `with_camera()`: `bind` runs inside `App::build`, called from `open_with_adapter` at `test_app.rs:155`. And the recording dir only exists once `with_camera()` has created the temp dir. So "write behaviour.json before `with_camera()`" (C2(b)) is impossible, and in C3(a)/C4(b) the first process has already started with the default behaviour (no crash, no stderr). | plan gap (decidability: 4 rows) | Amend §The fake sidecar: add `TestApp::with_camera_behaviour(serde_json::Value)` (`with_camera()` = the defaults `{exit_after_ms:null, ignore_quit:false, stderr:null}`), which writes `behaviour.json` before `open_with_adapter`, and add `TestApp::set_fake_sidecar_behaviour(json)` for C3(b)'s mid-test rewrite. Reword C2(b), C3(a), C4(b) to use it. |
| L2 | How `FAKE_SIDECAR_DIR` reaches the child. `SidecarSpec.env` is "`CRLAB_FINGERPRINT_DIR` only" (master plan §6.11), so the only remaining route is the inherited process env. `std::env::set_var` is `unsafe` under edition 2024 (`Cargo.toml:6`), and it is racy across parallel tests that each have their own temp dir. | plan gap (fixture producibility) | Amend: the fake derives its recording dir from its own location (`std::env::current_exe()?.parent()/rec`, i.e. `<data dir>/camera/sony/test/rec/`). Pin `fs::copy` + mode 0o755 and remove the symlink option from Delegated choices. Replace C4(a)'s clause "`env[0]["FAKE_SIDECAR_DIR"]` exists" with "`env[0]["CRLAB_FINGERPRINT_DIR"] == <data dir>/camera/fingerprints`" (the spec's one variable, §6.11; `paths.create` adds the dir in task 3). |
| L3 | Where `bind` sits in `App::build`. The plan says "after `events`/`health` exist" (`:404`, `:435`). Two fallible steps come later: `spool.set_dispatcher(..)?` at `:493–496` and `NudgeServer::start(..)?` at `:497`. If either fails after `bind`, `prepare`'s error path (`:186–189`) returns with the runtime's sink and health pointing at the dropped new App's `EventHub`/`HealthService`. The old App keeps running while camera events and camera health go nowhere. Same for `reopen` (`:765–777`). This is the plan's own projection question, and the answer is **no**, not as written. | plan gap (silent failure) | Amend §The runtime contract: "`bind` is the last statement of `App::build`, after every `?` and the thread spawns, immediately before `Ok(Self { … })`; a failed build never re-targets the runtime and never spawns". Call-site check (also asked by the plan): `App::build` has exactly three callers, `open_with_adapter` `:339`, `prepare` `:164` and `reopen` `:765`. With the contract as written, the first creates the `Arc` and the other two pass the same one. `open`/`open_with` go through `open_with_adapter`. No other constructor exists. |
| L4 | C7(a) "bind re-publishes the current `CameraStatus` … through `camera_status::health_items`". In P42 the runtime has no current status: with `CameraViaCaptureOne` the status is computed by core `CameraService` straight into `HealthService` (`services/camera.rs:26–43`), and `health_items` takes `StatusInputs`, which the runtime does not have for this provider. Calling `provider.status()` inside `bind` would put an osascript call inside `App::build` on the settings-save path. | plan gap | Amend: in P42, `bind` copies every camera-family item (code `camera` or `camera_*`, `camera_sidecar_unavailable` included) from the previous binding's `HealthService` into the new one. Precedent: `prepare` copies `backup_failed` the same way (`lib.rs:175–183`). From P46 the runtime's own `StatusInputs` and `health_items` replace the copy. C7(a) then asserts exactly what that copy produces. |
| L5 | The C8(a) hazard says "`TauriEvents::emit`'s `_ => {}` swallows `CameraStatusChanged` — C8(a) fails without the new arm". That premise is false. `camera_retry` emits through its own `TauriCameraEvents(app_handle)` straight to the `AppHandle` (`commands/camera.rs:24–33`, `:68–72`), bypassing the `EventHub`. So C8(a) passes with or without the arm, and the row cannot catch a missing arm (charter rule 2 companion: the fixture does not depend on the predicate). Separately, §6.7 says `camera_status_changed` is "emitted by the runtime on every status change", but P42 has no runtime status source. | plan gap (decidability + scope) | Amend task 5: `camera_retry` emits through `handles.events` (the App's `EventHub`), and `TauriCameraEvents` is deleted. C8(a)'s fixture sets the hub target to `TauriEvents`, like `c7c_tauri_history_changed_event` (`lib.rs:126–129`). State the P42 scope: only `camera_retry` emits, and the runtime emitter arrives with P46. |
| L6 | App quit in production. Tauri 2.12.5 with tao 0.37.2 ends the process with `process::exit` (`tao-0.37.2/src/platform_impl/macos/event_loop.rs:197–202`), and the managed `AppState` is never dropped. So `impl Drop for CameraRuntime` never runs at a real quit, and the plan's "(…the real process exit — stops the helper)" is false. The only backstop is the kernel closing the child's stdin, which the lab sidecar turns into `quit` (`Camera-Connection-Tests/sdk/src/main.cpp:144–145`). Nothing on our side enforces "3 s, then kill" (MC-31 §Process). | plan gap (production; not visible in the suite) | Amend task 5 and Notes: desktop `run()` becomes `.build(generate_context!())?.run(\|h, e\| if let RunEvent::Exit = e { h.state::<AppState>().handles()…camera_runtime.shutdown() })`. `Drop` stays as the path for tests and `kill_and_restart` (C1(c)). No new row (the mock runtime has no exit loop); the reviewer checks by reading. Effect in P42 is nil, since no sidecar is installed in production before P50. |
| L7 | The child writes `pids`/`argv`/`env`/`stdin.log` **after** `Command::spawn` returns. C1(a), C1(c), C3, C4(a) and C7(b) read `rec()` immediately, so the positive reads are racy. The negative reads in C1(a)/(b) and C7(b) ("no second spawn", "no `quit`", `pids.len()==1`) can pass on a wrong implementation if they run before the child writes. C3(a) says "`a.tick(0)` repeatedly in 100 ms steps", but `tick`'s argument *is* the clock advance. And the exit of a real child is noticed by `try_wait` only on some later tick: under manual time it can land at any 100 ms offset, which shifts every following spawn. | plan gap (decidability) | Amend the Fixture paragraph: `settle(a, pred)` = loop { `a.tick(0)`; if `pred` → done; sleep 10 ms } bounded at 3 s real time. Positive reads go through `settle`. Negative reads come after a ≥ 500 ms real-time settle. C3(a) procedure: after each spawn, `settle` until the new pid is recorded **and** `!alive(pid)`. A reaped pid proves the supervisor saw the exit at the current manual time, because `kill(pid,0)` succeeds on a zombie. Only then advance with `a.tick(100)`. With that, offsets 0, 1, 3, 7, 15, 31, 61 s are exact. |
| L8 | C3(b) "after 3 consecutive exits … cleared on the first stdout line of a later spawn". Nothing says what resets "consecutive". The fake prints `fake.started` before honouring `exit_after_ms: 0`, so a counter reset by a stdout line never reaches 3. The order of line handling versus exit handling inside one `tick` is also open. | plan gap | Amend: consecutive exits = exits since the last spawn that lived 60 s (the backoff counter, Notes reading (3)). A stdout line clears the item but does not reset the counter. Inside one `tick`, drained lines are handled before the exit. The fake writes its records and its started line before arming the exit timer. |
| L9 | C2(a) "exits with code 0 … (observed through `rec().pids` + `!alive`)". An exit code cannot be observed through pids and liveness: only the supervisor's `wait` sees it. Also open: whether `shutdown()` is terminal, i.e. whether a later `tick` must not restart the helper. The test's `TestApp` drop runs `Drop` → `shutdown()` a second time. | plan gap (decidability) | Amend: the fake appends `<pid> exit <code>` to `<rec>/exits` before any voluntary exit (`Recording.exits: Vec<(u32, i32)>`), and C2(a) asserts `(p0, 0)`. Contract: `CameraRuntime::shutdown(&self)` is public, idempotent and terminal (no restart afterwards). |
| L10 | C4(b)'s exact text depends on an unrelated choice. The fixture FTP password is `f` (plan §The fake sidecar; master plan §7.5). If the implementer registers every camera secret with the redactor (R-I21: "every secret is registered … before the sidecar is spawned or the receiver starts" invites that), `refused` becomes `re[REDACTED]used` and the row fails. That gives the outcome a second cause (rule 2 companion). | plan gap (fixture) | Amend: give the FTP password fixture a distinctive value (e.g. `ftp-pw-unique-1`), and say P42 registers only the two values it sends (user, password). P43 registers the FTP password when the receiver starts. Coordinator fold: master plan §7.5 still lists `u` / `p` / `f`, which disagrees with the plan's `u name` / `p/w`. |
| L11 | C5(a)–(g) pair order. If each `needs_action` Vec lists the pair in precedence order, a `derive_status` that just takes element 0 passes all seven rows. "Auth" is shorthand for `AuthenticationRefused` (§6.6). | plan gap (decidability) | Amend: in every pair the lower-ranked reason comes first in the Vec, and the reason is named `AuthenticationRefused`. |
| L12 | C5(l) "items contain … in the precedence order": "contain" accepts extra items, and the order rule is implicit. | plan gap (exactness) | Amend: `health_items(..) == [(camera_fingerprint_unconfirmed, false, "Confirm the camera's fingerprint"), (camera_files_not_received, false, "3 photos taken but not received for 60 s"), (camera, true, "<name> · wireless")]`: needs-action items by rank, then exactly one base item. |
| L13 | MC-31 §Secrets body says credentials are "passed to the sidecar only through its environment at spawn". R-I21 (same chapter, inventory addition) says first stdin line, "not through its environment". The plan's Read-first sends the implementer to both, and flow §6 makes a plan/intention contradiction `BLOCKED`. | intention gap (wording) | Route upstream: the orchestrator rewrites the MC-31 §Secrets sentence to defer to R-I21 (a wording fold under R-I21, no semantic change). Plan Notes gets one line: "R-I21 supersedes MC-31 §Secrets' 'through its environment'". |
| L14 | R-I37 "`Connecting` (sidecar running, no session)" read literally makes "helper running, no session" equal `Connecting`. C5(k)'s second case says `Disconnected` when `connecting: None`. The plan's reading (those are necessary conditions, and the link comes from the sidecar's connecting report) is the sensible one: otherwise a camera that is off shows "Connecting…" for ever. | intention gap (wording; no owner card) | Orchestrator records the reading in R-I37's wording. Flag for P46's planner: P46 decides when `connecting` is `Some`, and that is where the owner-visible behaviour lands. |
| L15 | `status_message` / health detail for `HeldByOtherProgram` in P42. §6.6 gives `camera_held` the MC-34 sentence, which P48 builds. | free choice | Delegate explicitly: P42 placeholder `"<holder> is holding the camera"` (or `"Another program is holding the camera"` when `holder` is `None`); P48 replaces it. Not asserted in P42. |
| L16 | The `--save-dir` value in P42. §6.11 / R-I20 staging is `<capture root>/.incoming/sdk-<run id>/`, but the plan's `AppBinding` has no capture root. | free choice | Delegate explicitly: any directory under the data dir containing `run_id` (e.g. `<data dir>/camera/staging/sdk-<run_id>`), created before the spawn; the fake ignores it, and P46 switches to R-I20. |
| L17 | Credentials "read from the `SecretStore` at each spawn". The first spawn is inside `App::build` (at startup, before the window). Restarts are inside `tick` on the App's background loop, which also drains the spool, runs arrival and runs jobs. A Keychain prompt (spike S6: after every update) would stall capture arrival. | plan gap (low in P42: nothing installed in production) | Amend: read once at the first spawn and cache in the runtime; restarts reuse the cache, and P50's `camera_secrets_set` refreshes it. R-I18's "used at the sidecar's next spawn" still holds. |
| L18 | During a rebuild, the old and the new App's background loops both run (`run_startup` in production) and both call `tick` on the same runtime until `old.shutdown()` joins. | plan gap (hazard) | Add to Hazards: `tick` is serialised inside the runtime (the supervisor `Mutex` held across drain, exit detection and spawn), so two callers can never double-restart. |
| L19 | The `argv` recording shape: "argv as one line to `<rec>/argv`" + `Recording.argv: Vec<String>` + "no element of `rec().argv[0]`". It is not settled whether the file is per process or overwritten, or what an "element" of a String is. | plan gap (decidability) | Amend: `<rec>/argv.<pid>.json` (a JSON array, like `env.<pid>.json`); `Recording.argv: Vec<Vec<String>>` in spawn order. |
| L20 | `CameraCapabilities`: add all §6.6 fields now, or only `depends_on_editing_provider`? | free choice | Delegate: all §6.6 fields; `CameraViaCaptureOne` = `status, capture, transfer: true`, the rest `false`, `depends_on_editing_provider: true`. |
| L21 | What `installed_sidecar` means by "newest" version folder (string or semver order). | free choice | Delegate (tests have one folder; P50's `SupportPackages::installed` replaces it). |
| L22 | `CameraDot` titles and tooltips for the three new kinds. `p07_shell_sessions_ui.test.tsx:188–205` pins `title="Camera connected: <name>"`, `data-state` and the USB-mode help on `disconnected`, and the plan allows no edit there beyond `link: 'cable'`. | free choice | Delegate with that constraint: `data-state = kind`; connected and disconnected keep today's title and help. |
| L23 | `KeychainSecrets` "is never constructed in tests" vs "`open`/`open_with` pass it". `open_with` is called by `p00_spike.rs:114,128,344`, `p05_capture_arrival.rs:435` and desktop `lib.rs:560`. | plan gap (wording) | Amend the Hazard: constructing it is harmless. The rule is "no test calls `get`/`set`/`delete`", which holds because no sidecar is installed under those tests' data dirs. |

## Reality-check findings
- **Wrong line citations** (the code is unchanged since `41bef14`, so these were never right): `crates/infra/src/logging.rs:380–391/:394–401/:405–407`
  (file has 128 lines; `add_secret` `:12–23`, `redact` `:26–33`); `crates/infra/src/paths.rs:527–535` (53 lines; `create` `:31–39`);
  `crates/core/src/domain/editing.rs:725–736` (`CameraStatus` `:31`, `CameraCapabilities` `:37`); `crates/core/src/events.rs:787–789`
  (`CameraStatusChanged` `:23–24`); `apps/desktop/src-tauri/src/state.rs:922–938` (52 lines; `settings_set` `:22–38`);
  `crates/testkit/src/test_app.rs` `create :117–175` (→ `:114`), `restart :182–196` (→ `:185–197`), `tick :218–226` (→ `:213–222`);
  `fake_capture_one.rs:114` (→ `:115`). All `crates/integrations/src/lib.rs` citations, `commands/camera.rs:11–23/:24–33` and the p04/p25
  lines are correct.
- **Perimeter misses:** `crates/testkit/tests/p24_capture_one_provisioning.rs:3` imports `domain::editing::CameraStatus` and must change
  (editing.rs "re-exports nothing"). That contradicts "No other test changes", and the file is not in Files expected. C6(a)'s `FakeDirectProvider`
  ("testkit") has no file in the list. `alive(pid)` needs `libc` in `crates/testkit/Cargo.toml` (listed only for `[[bin]]`).
- **Counts (manifest rule 3):** the frontend list names 8 files (`harness.tsx` + 7 tests), not 7. The paths in Files expected sum to 10 new
  source, 22 changed, 1 new test, 2 adjusted Rust tests and 8 frontend files = 43, against the stated 8/19/1/2/7 = 37. With p24 and a `FakeDirectProvider`
  file the total is 45. "8 criteria, 26 rows" is correct.
- **Dependency grounding (rule 17):** `security-framework` 3.7.0 is already in `Cargo.lock` (via rustls). `get_generic_password`
  (`passwords.rs:40`), `set_generic_password` (`:26`) and `delete_generic_password` (`:63`) exist, and `errSecItemNotFound = -25300` is in
  `security-framework-sys-2.17.0/src/base.rs:61`. The contract matches the installed crate. `ManualClock::mono()` (`testkit/src/clock.rs`) is
  `start + ms`, so `tick(clock.mono())` under manual time is producible. Tauri exit behaviour: L6.
- **Prior-phase dependencies:** `HealthService::has` counts only `!ok` items (`health.rs:101–105`), so C6(a)'s ok item `camera` must be read
  through `report(..)`, not `has`. `"sku_prefix":"PX"` and `"theme":"dark"` are accepted patches (used in `p39_undo_history_core.rs:535,545`).
  `TestApp::restart` builds the new App **before** dropping the old one (`test_app.rs:185–197`). So during C1(c) two fake processes exist
  briefly, and the row's outcome still holds as written.

## Criteria decidability (per row: writable now? / fixture producible?)
| Row | Writable now | Producible | Note |
|---|---|---|---|
| C1(a) | no → yes after L7 | yes after L2 | negative reads need the 500 ms settle |
| C1(b) | after L7 | after L2 | holds today by construction (a theme save never builds); a regression guard |
| C1(c) | after L7 | after L2 | the Drop path only; the production quit path is L6 |
| C2(a) | no → after L9 | after L2 | exit code is unobservable as written |
| C2(b) | no → after L1 | after L1, L2 | ≥ 3 s / < 4 s on the real clock, decidable |
| C3(a) | no → after L1, L7 | after L1, L2 | needs the reap-then-advance procedure |
| C3(b) | no → after L1, L8 | after L1, L2 | counter semantics |
| C4(a) | after L19, L7 | after L2 | the env check reads the whole inherited environment; the secret sentinels are improbable there |
| C4(b) | no → after L1, L10 | after L1 | second cause via the FTP password `f` |
| C5(a)–(g) | after L11 | yes (pure) | |
| C5(h)–(k) | yes | yes | C5(k) wording: L14 |
| C5(l) | after L12 | yes | |
| C6(a) | yes, once the `FakeDirectProvider` file is named | yes | read the ok item via `report` |
| C6(b) | yes | yes | p04 rows set the gate through activation failure, not `H.set`; acceptable as stated |
| C7(a) | no → after L4 | yes | |
| C7(b) | after L7 | after L2 | |
| C8(a) | no → after L5 | yes | as written it passes without the arm |

## Trace verification
Every row has a trace cell. The claimed ledger entry M39 is served by C1(a), C1(c), C4(a) and C4(b). R-I18 is served by C1 and C7, R-I21 by C4,
R-I33 by C6 and R-I37 by C5 and C7(a). Weak: C3(b)'s outcome (health item after three exits) traces to planner reading (2), not to MC-31
§Process, which says nothing about helper health. That is acceptable because the reading is listed for the narrative. No untraced rows.

## Exit
AMENDMENTS_REQUIRED. L1–L12, L17–L19 and L23 are plan amendments for the orchestrator. L13 and L14 are wording folds in 10E. L15, L16
and L20–L22 need explicit delegation lines in §Delegated choices. The implementer prompt does not compile until every row is routed.
The review-log line is written by the coordinator.

*(No skeleton appendix: the paper skeleton was discarded per doctrine.)*
