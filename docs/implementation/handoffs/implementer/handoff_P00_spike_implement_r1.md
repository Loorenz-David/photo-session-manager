---
plan: P00_spike
role: implement
round: 1
state: BLOCKED
date: 2026-10-25
actor: codex
---

## What was built

Session Manager now has a Health page, a local data directory with SQLite and settings, compiled Capture One hook scripts, a durable hook inbox, and a signed macOS app bundle. The Health page can ask Capture One for its name to exercise Automation permission. A compiled processing hook reached the running dev app and became a SQLite event. The bundle could not be opened in this session, so the packaging spike is incomplete.

## ⚠ OWNER DECISIONS REQUIRED (0)

None. The remaining issue is a launch failure for the orchestrator to investigate or retry in an interactive Mac session; it does not call for a product choice.

## Blocker

Command: `open -n --env PSM_DATA_DIR=/tmp/psm-spike-bundle -a '/Users/davidloorenz/Desktop/Developer/Photo-Session-Manager/apps/desktop/src-tauri/target/release/bundle/macos/Session Manager.app'`

Error: `The application /Users/davidloorenz/Desktop/Developer/Photo-Session-Manager/apps/desktop/src-tauri/target/release/bundle/macos/Session Manager.app cannot be opened for an unexpected reason, error=Error Domain=NSOSStatusErrorDomain Code=-10827 "kLSNoExecutableErr: The executable is missing" UserInfo={_LSLine=4277, _LSFunction=_LSOpenStuffCallLocal, _LSFile=LSOpenCore.mm, _LSErrorMessage=kLSNoExecutableErr}`.

The bundle contains the executable named by `CFBundleExecutable`, and `codesign --verify --deep --strict` succeeds. No alternate launch route was used after this refusal. Bundle-attributed Automation permission, real Capture One processing-hook delivery, FSEvents on the actual capture folder, and launch on the work Mac remain unverified. The master-plan tracker stays at `NOT_STARTED`; this blocked implementation is not claimed as `IMPLEMENTED`.

## Tests → criterion rows

`cargo test --workspace` passed every P00 row on the final tree. Criterion-to-test map:

| Row | Boundary test | Result |
|---|---|---|
| C1(a) | `c1a_first_open_creates_layout_and_defaults` | Pass: layout, mode 0600, defaults, mechanisms version |
| C1(b) | `c1b_second_open_preserves_settings_and_migration` | Pass: edited settings unchanged, migration version 1 |
| C2(a) | `c2a_osascript_uses_data_directory_as_cwd` | Pass: same physical directory |
| C2(b) | `c2b_osascript_times_out_and_reaps_child` | Pass: timeout within two seconds, no child remains |
| C3(a) | `c3a_compile_all_produces_three_scripts` | Pass: three nonempty `.scpt` files |
| C3(b) | `c3b_processing_hook_returns_and_spools_without_listener` | Pass: exit 0 under two seconds, exact JSON |
| C3(c) | `c3c_capture_hook_spools_argv` | Pass: exact capture argv JSON |
| C3(d) | `c3d_processing_hook_escapes_quote_and_backslash` | Pass: RAW path preserved |
| C4(a) | `c4a_valid_file_becomes_one_event_and_is_deleted` | Pass: one row, file deleted |
| C4(b) | `c4b_malformed_file_goes_to_bad` | Pass: file moved, zero rows |
| C4(c) | `c4c_drain_uses_filename_order` | Pass: row order matches names |
| C4(d) | `c4d_open_drains_file_left_while_closed` | Pass: startup consumes file |
| C5(a) | `c5a_nudge_consumes_spool_file` | Pass: 204, row within one second |
| C5(b) | `c5b_nudge_body_cannot_inject_event` | Pass: 204, zero rows, loopback address |
| C6(a) | `c6a_watcher_emits_absolute_path` | Pass: absolute path from temporary folder |
| C7(a) | `c7a_generated_bindings_expose_commands` | Pass: commands present; bindings export and npm typecheck passed |

The three permitted core-mechanism tests also pass: hook template rendering, AppleScript JSON escaping, and spool payload parsing. The real processing-hook invocation against the running dev app produced one `capture_events` row with the expected source, hook and payload.

## Assumptions

- The cwd criterion compares physical paths because macOS resolves temporary `/var` paths to `/private/var` in JXA.
- `FsWatcher` tries native FSEvents first and uses `notify` polling if stream startup fails. In this session native FSEvents returned `unable to start FSEvent stream`.
- The Health page probes Capture One on load and when “Test Capture One” is pressed. Its `OsaRunner` call uses the contracted 30-second timeout.
- The placeholder icon is acceptable for this spike. Node 22 / npm 10 are selected explicitly for frontend commands because `apps/desktop/` otherwise resolves Node 23.

## Deviations

- `Cargo.lock`, `apps/desktop/src/components/ui/button.tsx`, and `apps/desktop/src/lib/utils.ts` were added beyond the plan's explicit file list. They are the reproducible Cargo resolution and the minimal shadcn initialization output; Button is used on the Health page.
- The generated shadcn CSS was mapped back to `claude_design/tokens/tokens.css` and the COMPONENTS.md theme mapping, retaining Archivo and the project's colors.
- Bundle launch and the dependent live checklist items remain incomplete due to the command refusal above. No real Capture One hook property was installed or cleared, and no camera frame was triggered. No second Mac was available.
- The tracker was not advanced to `IMPLEMENTED` because the packaging goal requires a successful bundle launch and Automation check.

## Files changed

Checkpoint paths: `Cargo.lock`; `apps/desktop/components.json`, `package.json`, `package-lock.json`, `tsconfig.json`, `vite.config.ts`, `src-tauri/src/commands/health.rs`, `src/components/ui/button.tsx`, `src/features/health/HealthPage.tsx`, `src/lib/utils.ts`, `src/services/bindings.ts`, `src/styles/globals.css`; `crates/infra/Cargo.toml`, `src/logging.rs`, `src/spool.rs`, `src/watcher.rs`; `crates/integrations/src/lib.rs`; `crates/testkit/tests/p00_spike.rs`; `docs/implementation/master_plan.md`, `docs/implementation/plans/P00_spike.md`; `docs/intention/evidence/packaging_spike.md`.

This handoff is left for the runner to commit under flow §8.

## Checkpoint SHA

`5e650e7c6041afbf021426fbc687bd7c33a9a196` — `CHECKPOINT (not approved): P00 workspace and packaging spike`. The working tree was clean immediately after the checkpoint, before writing this handoff.

## Stamp

| Command | Result |
|---|---|
| `cargo --version` | 1.99.1 |
| `cargo test -p psm-testkit --test p00_spike` | 16/16 pass |
| `cargo test --workspace` | Pass: 16 criterion, 3 core-mechanism, 1 bindings-export tests |
| `cargo clippy --workspace --all-targets -- -D warnings` | Pass |
| `cargo fmt --all -- --check` | Pass |
| `cd apps/desktop && npm ci` with Node 22 | Pass |
| `cd apps/desktop && npm run typecheck` with Node 22 | Pass |
| `cd apps/desktop && npm test` with Node 22 | Pass; no frontend test files in P00 |
| `cd apps/desktop && npm run tauri -- build --bundles app` with Node 22 | Pass; ad-hoc-signed 14.45 MiB app |
| `codesign --verify --deep --strict 'apps/desktop/src-tauri/target/release/bundle/macos/Session Manager.app'` | Pass |
| Bundle `open` command above | Blocked: LaunchServices -10827 |

## Manual try-out

1. In `apps/desktop`, use Node 22, run `npm ci`, then `npm run tauri -- dev`. The Health page should show the local data directory and status items.
2. Open the Capture One session document. On the Health page, press “Test Capture One”; allow the macOS Automation prompt if shown. The Capture One row should show its application name. Quit and relaunch to check that permission is remembered.
3. Run `npm run tauri -- build --bundles app`. Try opening `src-tauri/target/release/bundle/macos/Session Manager.app` interactively (right-click → Open on first launch). This is the step that failed through LaunchServices in the automated session.
4. Inspect `~/Library/Application Support/com.loorenz.sessionmanager/` for `session_manager.db`, `settings.json`, `hooks/`, `spool/`, and `logs/`. With the app open, a processing hook should write to the spool and become a `capture_events` row.
