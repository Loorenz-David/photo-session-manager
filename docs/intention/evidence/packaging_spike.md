# P00 packaging spike — 2026-10-25

The corrected SQLite pin resolves. The Rust workspace suite passes, the frontend builds, and Tauri produces an ad-hoc-signed macOS app. The dev process started and consumed a compiled processing hook through its live spool and nudge channel. LaunchServices refused to open the bundle in this sandboxed session, so bundle-attributed Automation permission and the remaining live Capture One probes are not verified.

| Checklist item | Result | Direct evidence |
|---|---|---|
| Development window and Health page | Process started; window visibility not verified | `npm run tauri -- dev` reached `Running target/debug/psm-desktop`; `/tmp/psm-spike-dev/` gained `session_manager.db`, mode-0600 `settings.json`, hooks, spool, logs and other directories. The webview could not be observed from this shell session. |
| Ad-hoc `.app` build | Pass | `npm run tauri -- build --bundles app` produced `Session Manager.app` (14.45 MiB on the final tree). `codesign --verify --deep --strict` exited 0. Its `Info.plist` contains `NSAppleEventsUsageDescription`; `CFBundleExecutable` is `psm-desktop` and that executable exists. |
| Bundle launch | Blocked | `open -n --env PSM_DATA_DIR=/tmp/psm-spike-bundle -a '/Users/davidloorenz/Desktop/Developer/Photo-Session-Manager/apps/desktop/src-tauri/target/release/bundle/macos/Session Manager.app'` returned `The application … cannot be opened for an unexpected reason, error=Error Domain=NSOSStatusErrorDomain Code=-10827 "kLSNoExecutableErr: The executable is missing"`. No alternate launch path was used after this refusal. |
| Bundle-attributed Capture One Automation prompt and remembered permission | Not verified | The bundle did not launch. The Health page now calls `Application("Capture One").name()` through `OsaRunner` on load or when “Test Capture One” is pressed, so the check is ready for an interactive run. |
| Compiled processing hook into running dev app | Pass at the hook → spool → SQLite seam | `osascript /tmp/psm-spike-dev/hooks/processing_done.scpt J1 /tmp/a.ARW /tmp/o.jpg` exited 0 in about 0.054 s. SQLite then contained one `capture_events` row with `source=hook`, `hook=processing_done`, and the exact expected JSON. This invocation did not originate from Capture One. |
| Capture One processing hook fired by the real application | Not verified | The bundled app could not launch, and no live Capture One property was installed. |
| FSEvents on `~/Pictures/session test 1/Capture` | Not verified | The `FsWatcher` boundary test passed on a temporary folder. Native FSEvents returned `unable to start FSEvent stream` in this session, so `FsWatcher` used its `notify` polling fallback. The real capture folder was not modified. |
| Launch on work Mac | Not done: no second Mac in session | The bundle is available locally for copying. |
| VERIFY-1 / VERIFY-11 capture probes | Not done: camera release not confirmed; bundle launch blocked | Prior evidence still marks capture-done argv and the variant lookup across collections as unprobed. No new claim is added to those VERIFY entries. |

## Build and test observation

`rusqlite_migration 2.6` and `rusqlite 0.40` resolve together. `cargo test --workspace` passed 16 P00 criterion tests, three core-mechanism tests, and the bindings export test. `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `npm run typecheck`, `npm test`, and the Tauri bundle build passed. The npm test command currently has no frontend test files; P00's frontend criterion is exercised by generated bindings plus TypeScript typecheck. Node 22 / npm 10 were selected explicitly in `apps/desktop/` because its default PATH chooses Node 23.
