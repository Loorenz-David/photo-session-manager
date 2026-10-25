---
plan: P00_spike
role: implement
round: 1
state: BLOCKED
date: 2026-10-25
actor: codex
---

# P00 implementation handoff

## What was built

A partial Session Manager foundation: local app data paths and settings, the first SQLite migration, durable hook spool, loopback nudge listener, file watcher, timed `osascript` runner, Capture One hook templates, a Tauri shell, and a Health screen. The code has not compiled or run. There is no `.app` and no hardware-spike result.

## Tests → criterion rows

No criterion row was verified. The behaviour test file `crates/testkit/tests/p00_spike.rs` has not been created. C1(a,b), C2(a,b), C3(a–d), C4(a–d), C5(a,b), C6(a), and C7(a) all remain open. The whole suite, clippy, format check, npm test, and typecheck were not run. No real Capture One or camera operation was attempted.

## Assumptions

- Preserve the partial scaffold in a checkpoint so the next implementer can resume from it.
- The Health page follows the existing design tokens. The placeholder icon is temporary within the plan's delegated icon choice.

## Deviations

- `cargo check -p psm-integrations` encountered a sandbox refusal writing Cargo's cache. The session wrapper requires `STATE: BLOCKED` and forbids a workaround. The phase's instruction to switch to a local `CARGO_HOME` therefore was not followed; the wrapper has higher authority.
- Cargo also reported no matching `tauri-specta = "=2.0.0-rc.27"` in the index; rc.26 was the latest candidate shown. This pin comes from master plan §6.2 and must be resolved upstream before a build can proceed.
- `npm install` failed before producing `package-lock.json`: `Cannot read properties of null (reading 'edgesOut')`. This may be a separate npm environment issue; no workaround was attempted after the sandbox refusal.
- `iconutil --convert icns apps/desktop/src-tauri/icons/SessionManager.iconset --output apps/desktop/src-tauri/icons/icon.icns` returned `Invalid Iconset.` The iconset was checkpointed, but `icon.icns` is absent.
- A cleanup command, `rm -rf apps/desktop/src-tauri/icons/SessionManager.iconset`, was rejected by command review: `rm -f style commands are not permitted. Use a safer approach`. It was not retried or rerouted.
- `shadcn init`, bindings generation, app build, hardware spike, and packaging evidence are incomplete. The committed `bindings.ts` is explicitly provisional and must be replaced by the generator.
- The progress tracker stays `NOT_STARTED` because this phase has not reached `IMPLEMENTED`.

## Files changed

Checkpoint `24b27a7` adds the partial `Cargo.toml` workspace, `.cargo/config.toml`, four crate scaffolds, Tauri and frontend scaffolds, icons, and hook templates; it updates `.gitignore`, master plan §7, and the P00 Review log. This handoff is left for the runner to commit under flow §8. No files outside the plan perimeter were changed.

## Checkpoint SHA

`24b27a759f8a306b9d78ded4fdbbae451bf86306` — `CHECKPOINT (not approved): P00 partial scaffold blocked by Cargo sandbox`.

## Stamp

| Command | Result |
|---|---|
| `cargo --version` | Exit 0: `cargo 1.99.1 (5f94df478 2026-08-27)` |
| `cargo check -p psm-integrations` | Exit 101. Exact sandbox line: `warning: failed to write cache, path: /Users/davidloorenz/.cargo/registry/index/index.crates.io-1949cf8c6b5b557f/.cache/ch/ro/chrono, error: Operation not permitted (os error 1)`. Cargo also failed version selection for `tauri-specta = "=2.0.0-rc.27"`; it listed rc.26, rc.25 and rc.24 as nonmatching candidates. |
| `npm install` (from `apps/desktop`) | Exit 1: `npm error Cannot read properties of null (reading 'edgesOut')`. |
| `iconutil --convert icns apps/desktop/src-tauri/icons/SessionManager.iconset --output apps/desktop/src-tauri/icons/icon.icns` | Exit 1: `Invalid Iconset.` |
| `rm -rf apps/desktop/src-tauri/icons/SessionManager.iconset` | Command review rejected it with `Script error: exec_command failed: CreateProcess { message: "Rejected(\"\`/bin/zsh -lc 'rm -rf apps/desktop/src-tauri/icons/SessionManager.iconset'\` rejected: rm -f style commands are not permitted. Use a safer approach\")" }`. |

## Manual try-out

There is no working application to try yet. After the environment and dependency pin are resolved, complete the criteria and run `cd apps/desktop && npm ci && npm run tauri -- dev`; the intended result is a Session Manager window with Health status and the data directory. Then build and open the `.app` and run the Capture One spike checklist in the plan.

## ⚠ OWNER DECISIONS REQUIRED (0)

none
