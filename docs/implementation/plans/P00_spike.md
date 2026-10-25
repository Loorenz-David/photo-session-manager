---
phase: P00_spike
title: Packaging spike and workspace skeleton
difficulty: high
projection: no
depends_on: []
---
# P00 — Packaging spike and workspace skeleton

## Goal
Create the Cargo workspace, the Tauri app and the infrastructure the whole product stands on, and prove the
packaging risks of book chapter 11 on this Mac: the app runs from the bundle, talks to Capture One through
`osascript` (Automation permission attributed to the bundle), watches a folder, keeps SQLite and settings in the
app data directory, and receives Capture One hook events through the spool + nudge channel (MC-15) — including
events fired while the app was closed. Ends with an ad-hoc-signed `.app` and the spike record written to the
evidence. **Not in this phase:** domain tables beyond `settings`/`capture_events`, any product feature, any UI
beyond a Health page, hook *dispatch* (P05), render handling (P06).

## Read first
master plan §3, §6 (all), §7 · `docs/process/flow.md` · book `intention.md`, `01`, `14`, `11` (Processes, Frontend ↔
backend, Layers, Repository layout, Packaging spike), `12` (Data directory, Settings, Logs) · contracts `10A` MC-15
(hooks, payloads, consumption), MC-18 (osascript cwd rule, evidence paragraph), MC-19 (30 s timeout) · evidence
`capture_one_16.8.6.md` VERIFY-1, VERIFY-2, VERIFY-8 (delivery measurements), VERIFY-12 (how to re-run the capture
probe) · `claude_design/COMPONENTS.md` §Theme variables and `claude_design/tokens/tokens.css` (copied, not re-designed).

## Files expected to change (all new)
`Cargo.toml`, `rust-toolchain.toml`, `.cargo/config.toml`, `.gitignore` (add `.cargo-home/`, `.npm-cache/`,
`apps/desktop/src-tauri/target/`), `crates/core/{Cargo.toml,src/lib.rs,src/domain/mod.rs,src/ports/{mod,clock,store,events}.rs,src/events.rs,src/error.rs,src/services/{mod,health}.rs}`,
`crates/infra/{Cargo.toml,src/lib.rs,src/paths.rs,src/settings_file.rs,src/sqlite/{mod.rs,store.rs,migrations/001_init.sql},src/spool.rs,src/nudge.rs,src/watcher.rs,src/osascript.rs,src/logging.rs}`,
`crates/integrations/{Cargo.toml,src/lib.rs,src/editing/mod.rs,src/editing/capture_one/{mod.rs,hooks.rs,hooks/capture_done.applescript.tmpl,hooks/processing_done.applescript.tmpl,hooks/variant_adjusted.applescript.tmpl}}`,
`crates/testkit/{Cargo.toml,src/lib.rs,src/test_app.rs,src/clock.rs,tests/p00_spike.rs}`,
`apps/desktop/{package.json,package-lock.json,index.html,vite.config.ts,tsconfig.json,tailwind config per shadcn init,components.json,src/main.tsx,src/App.tsx,src/styles/tokens.css,src/styles/globals.css,src/services/bindings.ts,src/services/health.ts,src/features/health/HealthPage.tsx}`,
`apps/desktop/src-tauri/{Cargo.toml,build.rs,tauri.conf.json,Info.plist,capabilities/default.json,icons/*,src/main.rs,src/lib.rs,src/state.rs,src/commands/{mod.rs,health.rs}}`,
`docs/intention/evidence/packaging_spike.md` (new: the spike record), master plan §7 (amend with observed versions and sandbox outcome).

## Core mechanisms (unit tests allowed)
- `psm_integrations::editing::capture_one::hooks::render_template(tmpl, spool_dir, port) -> String` (MC-15 template fill, JSON escaping helper `applescript_json_escape`).
- `psm_infra::spool::parse_spool_file(bytes) -> Result<HookPayload>` (shape per MC-15 payload table).

## Ordered tasks
0. **Toolchain gate.** `cargo --version` must succeed (owner installs Rust: master plan §7.1). If `cargo fetch` or
   `npm ci` is refused by the sandbox when writing under `~/.cargo` or `~/.npm`, use `CARGO_HOME=$ROOT/.cargo-home`
   and `npm_config_cache=$ROOT/.npm-cache` and record the outcome in master plan §7.1 (Codex sandbox row).
1. Workspace: root `Cargo.toml` (members per master plan §6.1), `rust-toolchain.toml`, crate skeletons with the
   module files listed above; pinned versions from §6.2.
2. `psm_infra::paths::AppPaths` (`from_env_or_default()`: `PSM_DATA_DIR` else Tauri app-data; `for_dir(path)`),
   creating the layout of §6.3. `settings_file::Settings` with the §6.8 keys and defaults, written at first open
   with mode `0600`, re-read (not overwritten) at later opens.
3. SQLite `Store::open(paths)` with `001_init.sql` (`settings`, `capture_events`), WAL, `mechanisms_version = "1"`
   written once (05A conventions).
4. `psm_infra::osascript::OsaRunner::run(script_path, args, timeout) -> Result<String>`: spawns
   `osascript -l JavaScript <script> <args…>` with **cwd = data dir**, kills on timeout, logs duration at `info`.
5. Hook templates (AppleScript, MC-15 body: `try`, build JSON, write `<spool>/<epoch µs>-<pid>-<hook>.json.tmp`,
   rename to `.json`, `do shell script "curl -s -m 1 http://127.0.0.1:<port>/nudge >/dev/null 2>&1 &"`, no error
   escapes) rendered and compiled with `osacompile -o <data>/hooks/<name>.scpt` at startup
   (`hooks::compile_all(paths, port)`). The `capture_done` template writes `{"hook":"capture_done","argv":[…]}` and,
   when argv is empty, adds `"last_captured_file"` from `last captured file` of the current document (MC-15).
6. `spool::Spool::drain(store)`: files in name order; each → one `capture_events` row (`source = hook`) in a
   transaction, then delete; malformed → `spool/bad/`. `nudge::NudgeServer` (axum, bound to `127.0.0.1:0`, port
   recorded on `App`), `GET /nudge` → 204 and a drain. Drain also at startup, on FSEvents change of `spool/`, and
   every 5 s (tokio interval).
7. `watcher::FsWatcher::watch(dir) -> mpsc::Receiver<WatchEvent{path}>` over `notify` (FSEvents backend), absolute
   paths, recursive one level (MC-2 needs root + one level).
8. `psm_core::services::health::HealthService::report()` → `HealthReport{items: Vec<HealthItem>}` with the P00 items
   `data_dir`, `hooks_compiled`, `nudge_listening`; later phases add theirs. `health_get`, `app_paths_get` commands.
9. Tauri app (`psm-desktop`): `tauri.conf.json` (identifier, productName, `build.devUrl = http://localhost:1420`,
   `beforeDevCommand = npm run dev`, `frontendDist = ../dist`, `bundle.macOS.signingIdentity = "-"`,
   `minimumSystemVersion`), `Info.plist` with `NSAppleEventsUsageDescription`, state wiring (`App` in managed state),
   specta builder + `export_bindings` test. Frontend: Vite + React + TS, shadcn init with the theme-variable map of
   `COMPONENTS.md`, `tokens.css` copied, a Health page listing `health_get` items and the data dir.
10. `psm-testkit`: `TestApp::new()` (temp dir, `ManualClock`, real Store, real Spool/Nudge, no providers yet),
    `deliver_hook(json)`; behaviour tests `tests/p00_spike.rs` for every criterion row below.
11. **Spike checklist** (manual, rule 1 exemption; results into `docs/intention/evidence/packaging_spike.md` and the
    Review log): (a) `npm run tauri -- dev` opens the window and the Health page; (b) `npm run tauri -- build
    --bundles app`, launch the `.app`; (c) from the bundle, `OsaRunner` runs `Application("Capture One").name()` →
    the Automation prompt appears **once**, attributed to "Session Manager", and is remembered on relaunch;
    (d) set `processingDoneScript` on Capture One to `<data>/hooks/processing_done.scpt`, process one image →
    a `capture_events` row appears (VERIFY-8 with the real app), Capture One not blocked; clear the property after;
    (e) `FsWatcher` on `~/Pictures/session test 1/Capture` sees a copied file; (f) copy the `.app` to the work Mac if
    reachable, right-click → Open launches (else record "not done: no second Mac in session"); (g) if the owner has
    made the camera release (evidence VERIFY-12), re-run VERIFY-1 for `capture done script` and VERIFY-11, and
    append results under those VERIFY items in `capture_one_16.8.6.md`.

## Criteria (7 criteria, 16 rows; 3 `[rule]` rows: C1(a), C1(b), C7(a))
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | `App::open(AppPaths::for_dir(tmp))` | empty temp dir | `session_manager.db`, `settings.json` (mode `0600`, containing every §6.8 key with its default; `sku_prefix` absent; `integration: null`), `previews/`, `hooks/`, `spool/`, `spool/bad/`, `logs/`, `backup/` exist; `settings` table has `mechanisms_version = "1"` | 05A conventions; `[rule]` ch12 §Settings (0600) |
| C1(b) | second `App::open` on the same dir | `settings.json` edited to `preview_delay_s: 20` between opens | file content unchanged after the second open; `mechanisms_version` still `"1"`; no second migration | 05A conventions `[rule]` |
| C2(a) | `OsaRunner::run` | JXA script printing the process cwd (`$.NSFileManager.defaultManager.currentDirectoryPath.js`) | returned stdout equals the data dir path | MC-18 (osascript cwd rule) |
| C2(b) | `OsaRunner::run` with timeout 1 s | JXA script that `delay(3)` | `Err(Error::Timeout)` within 2 s; no `osascript` child remains | MC-19 (per-call timeout) |
| C3(a) | `hooks::compile_all` | data dir, port | `hooks/capture_done.scpt`, `hooks/processing_done.scpt`, `hooks/variant_adjusted.scpt` exist and are non-empty | M22 |
| C3(b) | `osascript <hooks>/processing_done.scpt J1 /tmp/a.ARW /tmp/o.jpg` (no listener on the port) | the compiled hook itself | returns within 2 s, exit 0; exactly one `spool/*-processing_done.json` whose content parses to `{"hook":"processing_done","job":"J1","raw":"/tmp/a.ARW","outputs":["/tmp/o.jpg"]}` | M22 (hooks never wait; closed port) |
| C3(c) | `osascript <hooks>/capture_done.scpt /tmp/a.ARW` | argv with one path | one `*-capture_done.json` parsing to `{"hook":"capture_done","argv":["/tmp/a.ARW"]}` | M22 |
| C3(d) | `osascript <hooks>/processing_done.scpt J1 '/tmp/we"ird\\x.ARW' /tmp/o.jpg` | path with `"` and `\` | the JSON parses and `raw` equals the input string byte for byte | MC-15 (escaping) |
| C4(a) | `Spool::drain` | one valid spool file | exactly one `capture_events` row (`source = hook`, `hook = processing_done`, `payload` = file content); file deleted | M22 |
| C4(b) | `Spool::drain` | file with invalid JSON | moved to `spool/bad/<same name>`; zero rows | MC-15 (malformed) |
| C4(c) | `Spool::drain` | two files named `…0001-…json`, `…0002-…json` written in reverse order | rows in name order (`0001` first) | MC-15 (name order) |
| C4(d) | `App::open` | spool file present before open | after open returns, one row exists and the file is gone | M22 (fired while closed) |
| C5(a) | HTTP `GET http://127.0.0.1:<app.nudge_port()>/nudge` | a spool file written just before the request | response 204; within 1 s the file is consumed and its row exists | M22 (nudge) |
| C5(b) | HTTP `POST /nudge` with a JSON body naming a RAW | request body | 204, zero `capture_events` rows (the listener carries no data); bound address is `127.0.0.1` | MC-15 (loopback, no injection) |
| C6(a) | `FsWatcher::watch(tmp)` | create `tmp/x.ARW` | one `WatchEvent{path}` with the absolute path within 2 s | MC-2 (watcher channel) |
| C7(a) | `cargo test -p psm-desktop export_bindings` then `npm run typecheck` | — | `bindings.ts` contains `health_get` and `app_paths_get`; typecheck exit 0 | `[rule]` ch11 §Frontend ↔ backend (R26) |

## Delegated choices
Icon set (placeholder icons acceptable), Vite port, exact `axum` route handler structure, log line format (must
include duration for provider calls), how `TestApp` finds the free nudge port, shadcn component subset for the
Health page, how `.gitignore` entries are grouped, eslint (optional; not part of "lint" for B4).

## Manual try-out (owner, after approval)
1. `cd apps/desktop && npm ci && npm run tauri -- dev` → a window titled "Session Manager" with a Health page.
2. Open Capture One with your session document; on the Health page press "Test Capture One" (or wait for the
   startup check): macOS asks once whether Session Manager may control Capture One → Allow. The page shows the
   document name.
3. Quit and relaunch: no second permission prompt.
4. `npm run tauri -- build --bundles app`; open `src-tauri/target/release/bundle/macos/Session Manager.app`
   (right-click → Open the first time).
5. Look in `~/Library/Application Support/com.loorenz.sessionmanager/`: `session_manager.db`, `settings.json`,
   `hooks/`, `spool/`, `logs/`.

## Notes
- Hook shapes: `processing_done` argv is grounded (evidence VERIFY-1: 13 observations); `variant_adjusted` argv is
  empty and the hook queries `primary variant` (VERIFY-9); `capture_done` argv is **unprobed** — the template
  writes every argument plus `last_captured_file` so P05 can accept either shape.
- The spike checklist is manual by the rule-1 exemption; C3–C5 are its automated proxies.
- Expect the Automation prompt only when running from a bundle or from `npm run tauri -- dev` (the dev binary is
  also a bundle under Tauri); a bare `cargo run` attributes the permission to the terminal.
- Estimated size: ~45 files, ~2 500 lines (half of it scaffolding).

## Review log
(empty)
