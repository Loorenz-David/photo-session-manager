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

## Live checks run by the orchestrator outside the Codex sandbox (2026-10-25, 15:29–15:50, with the owner)

The built bundle was launched from the orchestrator's shell (not sandboxed):
`open -n --env PSM_DATA_DIR=<scratch> "apps/desktop/src-tauri/target/release/bundle/macos/Session Manager.app"`.
The `-10827` refusal above is a property of the Codex sandbox, not of the bundle.

| Checklist item | Result | Evidence |
|---|---|---|
| (b) Launch the `.app` | **PASS** | Process started, data dir laid out; the owner saw the window with the Health page (screenshot 15:49: Data directory, Capture One hooks "Compiled", Event listener "Listening", Capture One "Capture One", all green). |
| (c) Automation prompt attributed to Session Manager | **NOT PROVEN; probe defect** | No prompt appeared, on load or on "Test Capture One". The probe script is `Application("Capture One").name()`, which JXA answers from the bundle without sending an Apple event, so it cannot trigger or test the permission. `probe_capture_one` also rewrites the script on every call. The button gives no visible feedback (same text before and after). Plan amended. |
| (d) Real Capture One processing hook → app | **PASS** | `processingDoneScript` set to `<data>/hooks/processing_done.scpt`; `process("…/session test 11952.ARW", {recipe: "JPEG - QuickProof (preview size)"})` returned job `5F4E3B10…` in 32 ms; Capture One answered a query 8 ms later (not blocked); one `capture_events` row `hook=processing_done` with `job`, `raw`, `outputs: ["…/Output/session test 11952.jpg"]`. |
| (g) VERIFY-1 capture-done hook with a real capture | **PASS** | `captureDoneScript` set to `<data>/hooks/capture_done.scpt`; autofocus (focused), `capture()`; `capture_events` row `hook=capture_done`, `argv: ["/Users/davidloorenz/Pictures/session test 1/Capture/session test 11957.ARW"]` (the full RAW path as the only argument), received 1.74 s after the capture call. |
| Clearing a hook property | Finding | Setting the property to `null` (JXA) or `missing value` (AppleScript) raises -1700. Setting it to `""` clears it; read-back is `null` / `missing value`. Both hooks were cleared this way after the checks. |
| (e) FSEvents on the real capture folder | Not done | The P00 app does not watch the capture folder yet; covered when capture arrival is built. |
| (f) Work Mac | Not done: no second Mac in session. | |

## P00 implement continuation (2026-10-25)

The Health probe now writes its JXA script once during `App::open` and calls
`Application("Capture One").currentDocument().name()`. This query requires a real Apple event;
the former application-name query did not. The Health button reads “Testing…” while the query is
running, then the Capture One row shows the document name or error and a local “checked hh:mm:ss”
time. The probe is invoked at startup and by the button.

`cargo test -p psm-testkit --test p00_spike` passed all 16 boundary rows. The updated frontend
typechecked, and `npm run tauri -- build --bundles app` produced an ad-hoc-signed bundle;
`codesign --verify --deep --strict` passed. The earlier live checks above establish bundle
launch and hook delivery outside this sandbox. The updated real-event probe's Automation prompt,
permission memory, and document-name display await an interactive bundle run by the owner or
orchestrator. The work Mac remains unavailable. The real capture-folder watcher probe was not
rerun in this continuation; its temporary-folder boundary test passed.

## (c) Automation prompt — PASS (orchestrator with the owner, 2026-10-25 19:25)
The approved P00 bundle (`Session Manager.app`, built 15:52, probe `Application("Capture One").currentDocument().name()`)
was opened with a scratch data dir while Capture One had `session test 1` open. macOS showed the Automation prompt
attributed to **Session Manager**; the owner allowed it; the first probe took 2 848 ms (waiting on the prompt) and
the Health page showed `session test 1`. After quitting and reopening, the probe took 153 ms, **no prompt appeared**
(owner confirmed) and the document name showed again. The permission is attributed to the bundle and remembered.
