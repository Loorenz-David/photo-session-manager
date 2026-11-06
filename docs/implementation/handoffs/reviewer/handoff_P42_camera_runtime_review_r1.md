---
plan: P42_camera_runtime
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-06
actor: opus
---
# Review P42_camera_runtime, round 1: APPROVED_WITH_NOTES

The implementation meets all 26 criterion rows at their boundaries, and every row's test fails on a wrong outcome. C3(b) is judged at
the orchestrator-amended 8 s. Suite, clippy, fmt, typecheck and UI are green, H1–H8 hold, and no assumption contradicts the book. Six
non-blocking notes went to the backlog. The most useful one for P46: a sidecar that cannot be spawned is never retried and never raises
`camera_sidecar_unavailable`.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Gate and evidence
- **Gate, disclosed deviation.** The tracker row read `NOT_STARTED`, not `IMPLEMENTED`. The implementer left it there on purpose
  (handoff §Deviations: C3(b)'s "16 s" contradicted MC-31). The orchestrator's ruling (`0bfed86`, plan Review log) amended C3(b) to 8 s,
  set the handoff to `DONE` and directed "resume at review". That removed the row's only cause. Checkpoint
  `cd76ef3a8fc22b2cfd86b4fbc8b2226d079e4814` is in `git log`. I treated the gate as satisfied in substance and record the stale row here.
  On approval I moved the row straight to `APPROVED`.
- **Perimeter.** `git show --stat cd76ef3` lists 47 paths, byte-for-byte equal to the handoff's "Files changed" (sorted diff empty). That
  is the plan's derived 45, plus `Cargo.lock` (declared dependencies; accepted by the orchestrator) and the plan file (the implementer's
  Review log entry). No extra or missing paths.
- **Commands (one run each, master plan §Environment).** Tree: HEAD `8f1facb`, `git status` clean; `git diff cd76ef3 HEAD -- crates
  apps tools` is empty, so the code equals the checkpoint.
  - `cargo test --workspace`: 606 passed, 0 failed. The P42 tests: 12 in `p42_camera_runtime` (24.9 s), 12 `camera_status::tests::mc31_*`,
    and `p42_tests::c8a_camera_commands_emit_retry_through_hub`.
  - `cargo clippy --workspace --all-targets -- -D warnings`: rc 0.
  - `cargo fmt --all -- --check`: rc 0.
  - `npm run typecheck`: rc 0.
  - `npm test`: 50 files, 369/369.
  - No stray `camera-sidecar` processes after the run.
  - No B4.

## Verified correct
- **One runtime per process (R-I18, M39).**
  - `CameraRuntime::new` is called only in `App::open_with_adapter`, and production constructs the App once (desktop `lib.rs:471`).
  - `build` takes the runtime as a parameter. `prepare` and `reopen` pass `self.camera_runtime` (the same `Arc`).
  - `App::shutdown` does not touch the runtime. `Drop for App` joins the background thread, which holds a clone, before the fields
    drop. So in the test `kill_and_restart`, the old runtime's last `Arc` drops and its `Drop` sends `quit` (C1(c) observes this).
- **`bind` last (L3).** `camera_runtime.bind(..)` is the statement immediately before `Ok(Self{..})`. It comes after
  `set_dispatcher`/`NudgeServer::start` and after all three thread spawns. A failed build therefore never re-targets or spawns.
- **Health copy (L4).** `bind` copies every `camera`/`camera_*` item (ok and not-ok) from `previous_health`. C7(a) discriminates: the
  TestApp runs with `run_startup = false`, so nothing else writes `camera_disconnected` into the new `HealthService`.
- **Spawn guard.** A later bind returns early while `supervisor.is_some()`. After `shutdown`, `bind` and `tick` return early on `stopped`.
- **Supervisor.**
  - Drained lines are handled before the exit inside one `tick`, under the supervisor `Mutex`.
  - Backoff is `RESTART_BACKOFF_S[attempts-1]` for attempts 1–5, then `RESTART_STEADY_S`. The counter resets when the exiting spawn
    lived ≥ 60 s on the injected clock.
  - `camera_sidecar_unavailable` is set at `consecutive_exits() >= 3` and cleared by any stdout line; the line does not reset the
    counter.
  - `quit_and_wait` writes `quit`, polls `try_wait` for `QUIT_GRACE_MS`, then `kill()` + `wait()`, so the pid is reaped, never left a
    zombie.
  - The supervisor kills only its own `Child`.
- **Secrets (R-I21, M39).**
  - Credentials are read once and cached under the `credentials` mutex.
  - Only the two sent values go to `add_secret`, before the spawn.
  - Percent-encoding keeps only the RFC 3986 unreserved set (bytewise, so UTF-8 is safe).
  - The credentials go only in the first stdin line. argv is `auto --save-dir <data>/camera/staging/sdk-<run_id>`; the env adds only
    `CRLAB_FINGERPRINT_DIR`.
  - stderr goes through `redact_registered` into a 50-entry ring and `tracing::warn!`. The first line is never logged.
  - `KeychainSecrets` maps `-25300` to `None`/`Ok` per the grounded `security-framework` 3 API. No test calls it.
- **Status (R-I37).** `derive_status` picks NeedsAction (minimum rank) › Busy › Connected › Connecting (only with `helper_running` and a
  reported link) › Disconnected. `health_items` lists the reasons by rank plus exactly one base item. The C5 inputs put the lower rank
  first, so a first-element or max-rank pick fails.
- **R-I33.** `CameraService::status`/`capture` apply both Capture One gates only when `depends_on_editing_provider` is true.
  `CameraViaCaptureOne` declares true; `FakeDirectProvider` declares false. C6(a) sets `capture_folder_not_set` and gets `Connected` and a
  `CaptureId`. The p04 rows still pass (C6(b)).
- **Commands and events (§6.7).** `CameraStatusResponse` is tagged `kind`, with `link`/`reason` snake_case and `message` from
  `status_message`. `camera_retry` emits through `app.events` (the hub). `TauriCameraEvents` is deleted. `TauriEvents::emit` gained the
  `camera_status_changed` arm and lost its `_ => {}`, so the match is now exhaustive.
- **Exit hook (L6; no row, read as directed).** `run()` is now `.build(..).expect(..).run(|handle, event| if RunEvent::Exit && let Ok(h) =
  state.handles() { h.camera_runtime.shutdown() })`. This matches the plan's contract; its effect is nil until a sidecar is installed (P50).
- **Rows → tests.** Each row's test observes the row's outcome at the row's boundary and would fail on a wrong outcome:
  - C1(a): a second spawn or a `quit` fails it.
  - C1(c): a missing `quit`/exit-0, or no new spawn, fails it.
  - C2(a): restarting after `shutdown`, or slow quitting, fails it.
  - C2(b): exits stays empty, and the elapsed time is in [3, 4) s.
  - C3(a): the pid count is asserted at offset − 100 ms and reached at each offset. Reaping before advancing pins the exit to the
    current manual time, so a wrong delay fails it on one side or the other.
  - C3(b): a counter reset on the line would spawn at 14.9 s, not 15 s; a missing clear fails the settle.
  - C4(a): exact first line; argv/env free of all four secret spellings; the fingerprint dir; staging contains `run_id`.
  - C4(b): exact ring contents.
  - C5: exact enum equality.
  - C6(a): the gates would turn the result into Disconnected/Err.
  - C7(a): a missing copy loses the item; C7(b): a new runtime changes `run_id` and spawns a second pid.
  - C8(a): the exact DTOs, no event from `camera_status`, exactly one payload from `camera_retry` (the missing-arm half bites; see note 4
    on the bypass half).
- **Assumptions (flow §6).** Lexical newest support folder (L21), `camera/staging/sdk-<run_id>` (L16), the 50-line ring (Delegated
  choices), the generic holder text (L15), and the fake-binary lookup (test infrastructure). None contradicts the book. The narrative
  states them for the owner, together with the planner readings (cable link for the Capture One tether, three crashes before the
  warning, 60 s counter reset).
- **H1–H8.**
  - H7: core holds only neutral camera vocabulary; no Sony, sidecar-protocol or env name is in `crates/core` (the Keychain service
    string is the app's own).
  - H8: the runtime lives in integrations, and the commands are thin mappings.
  - H2: the supervisor creates a staging dir and deletes nothing.
  - H5: no UI automation.
  - Secrets and no-live-services: only fake secrets in the testkit, and no test reaches the Keychain, a camera or Capture One.
  - Camera hard rules: no SDK file, credential or fingerprint is committed, and no foreign process is killed.

## Findings
None blocking (B1–B4). No finding carries a route.

## Notes written to backlog
1. **P42 → P46, P50.** A failed spawn leaves the supervisor stuck. A failed first spawn in `bind` stores `child: None, next_spawn_at:
   None`. A failed spawn in `tick` is not counted as an exit. So `camera_sidecar_unavailable` (§6.6, "could not start") is never set. A
   first-line write error also drops an unreaped `Child`.
2. **P42 → P46.** The `camera_sidecar_unavailable` detail reads `Camera helper is unavailable`; §6.6 registers `The camera helper could
   not start`.
3. **P42 → P50.** A failed Keychain read is cached as empty credentials for the whole run.
4. **P42 (test).** C8(a) cannot tell a direct `AppHandle::emit` from hub routing. Production was verified by reading.
5. **P42 (test).** C3(b) does not assert that the item is absent after the 1st and 2nd exits. The code (`>= 3`) was verified by reading.
6. **P42 → P51.** The new `CameraDot` kinds have no CSS colour.

## Lessons for the plans
- The gate check "tracker shows `IMPLEMENTED`" does not cover a BLOCKED → orchestrator-ruled → resumed round. When the orchestrator
  turns a BLOCKED handoff into `DONE`, it should also move the tracker row to `IMPLEMENTED`, or the review prompt should name the ruling
  as the gate evidence.
- An "after the Nth event" health row should state whether "not before" is part of the outcome (C3(b)'s planner reading 2 was in Notes,
  not in the row).

## Write perimeter
- `docs/implementation/handoffs/reviewer/handoff_P42_camera_runtime_review_r1.md` (this file, new)
- `docs/implementation/narratives/P42_camera_runtime.md` (new)
- `docs/implementation/master_plan.md` (P42 tracker row only)
- `docs/implementation/plans/P42_camera_runtime.md` (Review log entry appended)
- `docs/implementation/backlog.md` (six P42 rows appended to the table)
- No production or test code touched, and no probes (flow §4). Build artefacts went under `target/`; logs went to `/tmp/p42rev/` (outside
  the repo). No commits.
