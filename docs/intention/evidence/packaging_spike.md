# P00 packaging spike — partial record (2026-10-25)

The Cargo workspace and frontend scaffold exist. A dependency conflict in the pinned Rust versions stopped the app build before a bundle could be launched. This record distinguishes direct probes from work that still needs the built app.

| Checklist item | Result | Evidence |
|---|---|---|
| Development window and Health page | Not run | `cargo check -p psm-integrations` cannot resolve the pinned SQLite dependencies. |
| Ad-hoc `.app` build and launch | Not run | The same Cargo dependency conflict prevents the Tauri build. The Tauri icon CLI did generate `icon.icns`. |
| Bundle-attributed Automation prompt and remembered permission | Not run | There is no launchable bundle. |
| Real Capture One processing hook into the app | Not run | The app cannot start. The `processing_done` AppleScript template compiled and, when invoked directly with `J1`, a RAW path containing `"` and `\`, and an output path, returned in 0.182 s, exit 0, writing one valid JSON spool file. This probe had no nudge listener. |
| FSEvents watch on the Capture folder | Not run | The watcher cannot be exercised through the app until Cargo resolves. |
| Launch on work Mac | Not done: no second Mac in session and no `.app` bundle. | — |
| VERIFY-1 / VERIFY-11 capture probes | Not run | These need the app hook installation and bundle, which are blocked by Cargo resolution. |

## Build and dependency observation

`cargo check -p psm-integrations` failed before compilation: `rusqlite_migration v2.6.0` depends on `rusqlite ^0.40.0` and `libsqlite3-sys ^0.38.0`, while the master plan pins `rusqlite 0.41` and `libsqlite3-sys ^0.39.0`. Cargo rejects both native libraries with `links = "sqlite3"`. This is a conflict in the plan's pinned pair, not a source-code failure. The pin needs amendment upstream before the spike can finish.

The frontend installed under the planned Node 22 / npm 10 toolchain. `npm run typecheck`, `npm test`, and `npm run build` succeeded. The initial build emitted Tailwind at-rule warnings; adding `@tailwindcss/vite` removed them on the next build.
