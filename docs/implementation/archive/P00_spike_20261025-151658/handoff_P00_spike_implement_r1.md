---
plan: P00_spike
role: implement
round: 1
state: BLOCKED
date: 2026-10-25
actor: codex
---

# P00 spike handoff

## What was built

The partial Session Manager scaffold now has a Health page frontend, an npm lockfile, a macOS icon, storage and spool handling, a loopback nudge listener, a file watcher, an osascript runner, and three compilable Capture One hook templates. Hook files are written before the app is notified. Processed spool filenames are recorded in the existing `settings` table in the same transaction as the event, preventing a second event if the app crashes after commit but before file deletion. The hook does not dispatch to photo services; that belongs to P05.

The pinned Cargo dependencies contradict each other. `rusqlite_migration 2.6.0` requires `rusqlite ^0.40.0`, while the master plan pins `rusqlite 0.41`. Their `libsqlite3-sys` versions both link `sqlite3`, so Cargo cannot resolve the workspace. This stops all Rust tests, generated bindings, the Tauri app, and the live packaging checklist. The phase is BLOCKED and its tracker row was not advanced to IMPLEMENTED.

## Tests → criterion rows

All 16 boundary tests were written in `crates/testkit/tests/p00_spike.rs`. None could execute because Cargo fails during dependency resolution. Each assertion targets the row's exact outcome; C7 also needs the blocked bindings export command. Core-mechanism unit tests were added only for the two mechanisms listed in the plan.

| Row | Test | Status |
|---|---|---|
| C1(a) | `c1a_first_open_creates_layout_and_defaults` | written, not run |
| C1(b) | `c1b_second_open_preserves_settings_and_migration` | written, not run |
| C2(a) | `c2a_osascript_uses_data_directory_as_cwd` | written, not run |
| C2(b) | `c2b_osascript_times_out_and_reaps_child` | written, not run |
| C3(a) | `c3a_compile_all_produces_three_scripts` | written, not run; all templates compiled by a separate direct probe |
| C3(b) | `c3b_processing_hook_returns_and_spools_without_listener` | written, not run; direct probe returned in 0.182 s and wrote one valid spool file |
| C3(c) | `c3c_capture_hook_spools_argv` | written, not run |
| C3(d) | `c3d_processing_hook_escapes_quote_and_backslash` | written, not run; direct probe preserved both characters |
| C4(a) | `c4a_valid_file_becomes_one_event_and_is_deleted` | written, not run |
| C4(b) | `c4b_malformed_file_goes_to_bad` | written, not run |
| C4(c) | `c4c_drain_uses_filename_order` | written, not run |
| C4(d) | `c4d_open_drains_file_left_while_closed` | written, not run |
| C5(a) | `c5a_nudge_consumes_spool_file` | written, not run |
| C5(b) | `c5b_nudge_body_cannot_inject_event` | written, not run |
| C6(a) | `c6a_watcher_emits_absolute_path` | written, not run |
| C7(a) | `c7a_generated_bindings_expose_commands` | written, not run; `npm run typecheck` passes against the provisional bindings, but regeneration is blocked |

## Assumptions

- In `apps/desktop/`, the default PATH selects Node 23/npm 11 and `npm install` fails with npm Arborist `edgesOut`. Prefixing PATH with the master plan's Node 22/npm 10 installation made `npm install` and `npm ci` pass. The master plan environment table records this.
- A processed spool filename marker in the existing SQLite `settings` table is the smallest schema-compatible way to make the commit/delete gap idempotent without adding a P01 table or changing `001_init.sql` after its prior checkpoint.
- The placeholder icon is sufficient for this spike; the Tauri icon CLI generated `icon.icns` from it.

## Deviations

- The master plan's `rusqlite 0.41` and `rusqlite_migration 2.6` pair cannot resolve. I did not change either pin. The upstream registry needs a compatible pair before implementation can finish.
- `@tailwindcss/vite` was added to the frontend dependency set because the existing Tailwind 4 CSS produced unhandled at-rule warnings; the next build completed without them.
- `shadcn init` was not run. The partial scaffold already has `components.json`; the UI is not runnable as a Tauri app until Cargo resolves.
- `apps/desktop/src/services/bindings.ts` is the provisional file from the earlier checkpoint. It was not hand-edited here; `cargo test -p psm-desktop export_bindings` failed before running.
- The live bundle, Automation permission, Capture One hook-to-app, FSEvents capture-folder, work-Mac, and capture probes are unverified. Details are in `docs/intention/evidence/packaging_spike.md`.

## Files changed

The checkpoint's full write perimeter (45 paths) is:

```text
apps/desktop/package-lock.json
apps/desktop/package.json
apps/desktop/src-tauri/Cargo.toml
apps/desktop/src-tauri/build.rs
apps/desktop/src-tauri/icons/icon.icns
apps/desktop/src-tauri/icons/icon.png
apps/desktop/src-tauri/src/commands/health.rs
apps/desktop/src-tauri/src/commands/mod.rs
apps/desktop/src-tauri/src/lib.rs
apps/desktop/src-tauri/src/main.rs
apps/desktop/src-tauri/src/state.rs
apps/desktop/vite.config.ts
crates/core/src/domain/mod.rs
crates/core/src/error.rs
crates/core/src/events.rs
crates/core/src/lib.rs
crates/core/src/ports/clock.rs
crates/core/src/ports/events.rs
crates/core/src/ports/mod.rs
crates/core/src/ports/store.rs
crates/core/src/services/health.rs
crates/core/src/services/mod.rs
crates/infra/src/lib.rs
crates/infra/src/logging.rs
crates/infra/src/nudge.rs
crates/infra/src/osascript.rs
crates/infra/src/paths.rs
crates/infra/src/settings_file.rs
crates/infra/src/spool.rs
crates/infra/src/sqlite/mod.rs
crates/infra/src/sqlite/store.rs
crates/infra/src/watcher.rs
crates/integrations/src/editing/capture_one/hooks.rs
crates/integrations/src/editing/capture_one/hooks/capture_done.applescript.tmpl
crates/integrations/src/editing/capture_one/hooks/variant_adjusted.applescript.tmpl
crates/integrations/src/editing/capture_one/mod.rs
crates/integrations/src/editing/mod.rs
crates/integrations/src/lib.rs
crates/testkit/src/clock.rs
crates/testkit/src/lib.rs
crates/testkit/src/test_app.rs
crates/testkit/tests/p00_spike.rs
docs/implementation/master_plan.md
docs/implementation/plans/P00_spike.md
docs/intention/evidence/packaging_spike.md
```

This handoff file was written after the checkpoint so it could name its SHA; it is the only uncommitted file for the runner to collect. No architecture graph exists in this repo.

## Checkpoint SHA

`406614d99cc3044f687db96fa6c8426fcbba0a39` — `CHECKPOINT (not approved): P00 partial spike blocked on SQLite pins`.

## Stamp

| Command | Result |
|---|---|
| `cargo --version` | exit 0: cargo 1.99.1 |
| `cargo check -p psm-integrations` | exit 101: conflicting `libsqlite3-sys` links via pinned `rusqlite` pair |
| `cargo test --workspace` | exit 101: same dependency resolution conflict; no tests executed |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 101: same dependency resolution conflict |
| `cargo fmt --all -- --check` | exit 0 |
| `cargo test -p psm-desktop export_bindings` | exit 101: same dependency resolution conflict |
| `PATH=/Users/davidloorenz/.nvm/versions/node/v22.22.3/bin:$PATH npm ci` in `apps/desktop/` | exit 0 |
| `PATH=/Users/davidloorenz/.nvm/versions/node/v22.22.3/bin:$PATH npm run typecheck` | exit 0 |
| `PATH=/Users/davidloorenz/.nvm/versions/node/v22.22.3/bin:$PATH npm test` | exit 0; no frontend test files (the 16 criterion tests are Rust) |
| `PATH=/Users/davidloorenz/.nvm/versions/node/v22.22.3/bin:$PATH npm run build` | exit 0; Vite bundle built without Tailwind warnings |
| `PATH=/Users/davidloorenz/.nvm/versions/node/v22.22.3/bin:$PATH npm run tauri -- build --bundles app` | exit 1: frontend built, then same Cargo dependency conflict; no `.app` |
| Direct `osacompile` of all three rendered templates, then `osascript` of `processing_done.scpt` | all compile exit 0; hook exit 0 in 0.182 s; one valid JSON file with escaped RAW path |

## Manual try-out

1. In `apps/desktop/`, use the planned Node 22 toolchain: `PATH=/Users/davidloorenz/.nvm/versions/node/v22.22.3/bin:$PATH npm ci` and `PATH=/Users/davidloorenz/.nvm/versions/node/v22.22.3/bin:$PATH npm run build`. The frontend build should pass.
2. After the SQLite dependency pins are corrected upstream, run `cargo test --workspace`, clippy, bindings export, and the Tauri build. Then follow the plan's Manual try-out steps to inspect the Health page, Automation prompt, data directory, and bundle.

## ⚠ OWNER DECISIONS REQUIRED (0)

None. The blocker is a technical correction to the master plan's pinned SQLite dependency pair for the orchestrator/planner.
