---
plan: P63_final_check_fixes
role: implement
round: 1
state: DONE
date: 2026-11-10
actor: claude-opus (pipeline-implementer)
---
# P63 final-check fixes — implement r1

## What was built
- **Capture One stays closed during "Take the camera back" (#22).** Every script the app sends to Capture One now passes one gate. While the
  take-back keeps Capture One closed (from its first step to its end), and while a relaunch waits for the camera helper after the
  photographer quit Capture One with the cable waiting (P54), no script runs: the caller gets "not now" (`EditorLaunchDeferred`), which is
  neither a failure nor an attempt. The only thing that ever starts Capture One is the explicit launch step. "Is Capture One running?" is now
  read from the Mac's process list, never asked of Capture One. **Cause:** the variant follower asked `editor_info.js` on every 50 ms
  background tick, and the take-back's own quit/started polls did too. That script checks `running()`, then sends `captureDoneScript()` and
  `version()`. Sent just as Capture One finished quitting, those events relaunch it. That explains the relaunch 0.35 s after the quit and the
  false `stopped at step 3: Capture One did not quit`.
- **A missing Capture One session is reported, not recreated (#27).** When the session chosen in Settings is gone (its folder, or its
  `.cosessiondb`), the app creates no folders under its path. The camera helper is not started until the session is back, so no staging
  folder is created. The receiver answers uploads `451` (the camera keeps them queued). The ensure never runs `make new document` for it. The
  header (camera line) and Health show `The Capture One session in Settings is missing — choose one in Settings`.
- **The camera helper's staging follows the session (#28).** After the session changes in Settings, the helper is restarted with
  `--save-dir <new session>/Capture/.incoming/sdk-<run>`, keeping the camera's cached address. The restart waits while a take-back, an
  FTP upload or a Save-to-PC transfer is in progress.
- **Save settings cannot be missed (#4).** The Save button sits in a sticky footer, always in view. Leaving Settings with unsaved changes
  (Back button, Escape/⌘[, the Health button) asks `Save your changes?` with Save / Discard / Cancel.
- **Unplug → Wi-Fi in ~6 s (#5).** The helper now remembers the camera's address from its own Wi-Fi session. After the cable goes, it retries
  that address straight away for up to 5 s, then falls back to discovery at once. An empty discovery while recovering is retried at once,
  without the growing backoff. **Cause** (app log, 2026-11-10 13:40:12–24Z): session lost at +0. The first Wi-Fi discovery started at
  +1.06 s and came back empty after 3.68 s. Then a 1.0 s backoff. The second discovery started at +5.8 s and found the camera after 3.84 s.
  Connect took 2.2 s, for 11.84 s in total. No cached endpoint was used (none existed in that helper run), and the camera's Wi-Fi was
  already up: its FTP uploads arrived at +3.7 s. Run 2's 6.1 s was the same path with the first discovery hitting.
- **Style is a drop-down (#6).** Settings → Style lists `No style`, then the photographer's own styles (from Capture One's Styles folder),
  then the built-in ones, exactly as Capture One names them. The stored `style_name` is the chosen name. When Capture One is not running
  or the list can't be read, the field is the editable text with `Couldn't read Capture One's styles`. Asking never launches Capture One.
- **P48 tests are fast again (#7).** `p48_camera_ownership` runs in 10 s (32/32), down from ~12 min. **Cause:** not P60's waits. On this Mac,
  starting a macOS FSEvents folder watcher takes 11–22 s, and `App::shutdown` joined the spool thread that was still inside that call, so
  every camera test paid it at teardown (measured: open 0.25 s, test body 1 s, shutdown 31–57 s). Shutdown now waits at most 1 s for that
  thread. The thread checks `stop` first and leaves by itself. No production timing value changed.

## Tests → criterion rows
| Row | Test (boundary) | Result |
|---|---|---|
| C1(a) | `p63_final_check_fixes::c1a_no_script_reaches_capture_one_while_the_take_back_keeps_it_closed`: take-back past its quit step with the follower due and a preview requested, fake relaunching on any Apple event. Ops between `quit` and `launch` are exactly step 5's `editor_info`; one launch, after our USB session; report `{5, camera_held}`; the follower polls again afterwards | pass |
| C2(a) | `p63::c2a_a_missing_session_is_reported_never_recreated`: configured session deleted, app restarted. Health `document_missing` = the sentence; status `NeedsAction{DocumentMissing}` with that sentence; STOR → `451`; activation; the session path still does not exist; no `make_document` | pass |
| C3(a) | `p63::c3a_the_helpers_staging_follows_the_session`: session changed while an FTP upload is open. No restart during the upload (argv unchanged for 800 ms); after it ends, a second spawn with `--save-dir <new>/Capture/.incoming/sdk-<run>` | pass |
| C4(a) | `apps/desktop/src/features/settings/__tests__/p63_settings_save_and_style.test.tsx` (4 tests): Save inside a `position: sticky; bottom: 0` footer; Back with a change → dialog, Cancel stays (hash and value kept), Health → Discard leaves without saving; Escape → Save saves the change then leaves; no change → no dialog | pass |
| C5(a) | `native/camera-sidecar/tests/test_transport_manager.cpp` "P63 C5(a) …" ×3 (production config, the log's SDK costs: discovery 3.8 s, connect 2.2 s, remote back at +3.7 s): cached endpoint valid → `recovery_ms ≤ 6100`; stale → within window + discovery + connect; none → no backoff gap | pass (31/31) |
| C6(b) | UI: same test file, 2 tests (options `No style`, `test-style`, `Vivid` in that order; choosing and saving stores `style_name: "test-style"`; unreadable → text field + sentence, editable, saved). Backend: `p63::c6b_capture_ones_styles_for_settings` (user first; a closed Capture One is not asked: no op) | pass |
| C6(a) | `cargo test -p psm-testkit --test p48_camera_ownership`: 32 passed, `finished in 10.23s` (47 s wall incl. compile), assertions unchanged (see Deviations for `c3c`'s fixture) | pass |
| C7(a) | the closing gate (Stamp) | see Stamp |

## Assumptions
- The "missing" rule applies to a session **chosen in Settings**. The app's own session is still created by Capture One when absent (10C
  MC-26 step 2: "Missing or null → the app's own document … created").
- The header sentence appears on the direct camera's status line (the header's camera dot/card). With the Capture One camera adapter
  (no camera support installed), only Health shows it.
- "Protected operation" for the helper restart = a take-back running, an FTP upload open, or a Save-to-PC transfer expected.
- In the P54 relaunch wait, `editor_info` still runs: the ensure needs it to reach `launch`, whose start-order wait is the one explicit
  launch step. All other scripts are refused there while Capture One is not running.
- Helper recovery policy values: cached endpoint retried at once for 5 s after a loss (`cached_endpoint_window_ms`); empty discoveries
  retried at once for 30 s after a loss (`recovery_fast_window_ms`), then the old backoff. The endpoint comes from this helper's own Wi-Fi
  connect (or the app's `--ip/--mac`). How fast the real SDK fails a direct connect to a camera whose remote is not up yet is unmeasured.
  The policy is bounded either way by the SDK's connect timeout.
- Style list: "the photographer's own" = names with a `.costyle` file under `~/Library/Application Support/Capture One/Styles`
  (recursive). A stored name that Capture One does not list (e.g. the owner's `Test-style`) stays selectable as "Test-style (not in Capture
  One)", so opening Settings never changes the setting silently.
- Tracker actor: the prompt template says `codex`; this session was Claude (Opus), so the row says `claude-opus`.

## Deviations
- **Files outside the plan's named list** (all needed by the rows): `crates/core/src/{domain/camera.rs, domain/editing.rs,
  mechanisms/camera_status.rs, ports/editing.rs, services/editing_startup.rs}`, `crates/integrations/src/lib.rs`,
  `crates/integrations/src/editing/capture_one/{scripting.rs, jxa.rs, provision.rs, jxa/list_styles.js (new)}`,
  `crates/testkit/src/fake_capture_one.rs`, `apps/desktop/src-tauri/src/{lib.rs, commands/settings.rs, commands/camera.rs}`,
  `apps/desktop/src/{services/bindings.ts (regenerated), services/styles.ts (new), lib/routes.ts, lib/keyboard.ts,
  features/shell/PageHeader.tsx}`.
- **`p48_camera_ownership.rs::c3c_make_document_deferred`**: no timing edit was needed (#7 was shutdown, not the tests). This one row
  deleted the *configured* session and expected the recreate path deferred by the gate. Under #27 a configured session is never recreated,
  so the row now runs on the app's own, not yet created, session (`settings_set({"document_id": ""})`). Assertions unchanged.
- **Contract note for the book:** 10E R-I20 says staging is "computed at the first spawn and never moved for this process". #28 moves it on
  a session change (restart). Fold upstream.
- `apps/desktop/src/services/bindings.ts` gained `editorStyles` / `EditorStylesDto` (generated by `export_bindings`).

## Files changed
Production: `crates/core/src/domain/camera.rs`, `crates/core/src/domain/editing.rs`, `crates/core/src/mechanisms/camera_status.rs`,
`crates/core/src/ports/editing.rs`, `crates/core/src/services/editing_startup.rs`, `crates/integrations/src/lib.rs`,
`crates/integrations/src/camera/runtime.rs`, `crates/integrations/src/camera/direct/adapter.rs`,
`crates/integrations/src/camera/direct/take_back.rs`, `crates/integrations/src/editing/capture_one/{adapter.rs, follow.rs, gate.rs, jxa.rs,
provision.rs, scripting.rs}`, `crates/integrations/src/editing/capture_one/jxa/list_styles.js`,
`apps/desktop/src-tauri/src/{lib.rs, commands/settings.rs, commands/camera.rs}`, `apps/desktop/src/features/settings/SettingsPage.tsx`,
`apps/desktop/src/features/shell/PageHeader.tsx`, `apps/desktop/src/lib/{routes.ts, keyboard.ts}`,
`apps/desktop/src/services/{bindings.ts, styles.ts}`, `native/camera-sidecar/include/crlab/transport.hpp`,
`native/camera-sidecar/src/{transport_manager.cpp, sdk_adapter.cpp}`.
Tests and harness: `crates/testkit/tests/p63_final_check_fixes.rs` (new), `crates/testkit/tests/p48_camera_ownership.rs`,
`crates/testkit/src/fake_capture_one.rs`, `apps/desktop/src/features/settings/__tests__/p63_settings_save_and_style.test.tsx` (new),
`native/camera-sidecar/tests/test_transport_manager.cpp`.
Docs: `docs/implementation/plans/P63_final_check_fixes.md` (Review log), `docs/implementation/master_plan.md` (P63 tracker row only).
Probe files: none (no applied-and-reverted edits; a scratch timing test `crates/testkit/tests/zz_scratch.rs` was created and deleted, never
committed).

## Checkpoint SHA
`0247c78` — `CHECKPOINT (not approved): implement P63 final-check fixes` (35 paths; this handoff is left for the runner to commit).

## Stamp
Tree: `d61b56c` + this phase's working-tree changes (the checkpoint commit's tree).
| Command | Result |
|---|---|
| `cargo test --workspace` | 56 test binaries ran: **901 passed, 1 failed**, then cargo stopped at the failing binary (73 min wall; the machine was loaded). The failure: `p62_retention_after_publish::c1a_…` at line 365, a missing line in its thread-local log capture (`storage.cleared rule=retention …`). Run alone, the file is **12/12, three times in a row**. This phase changed no storage or retention code; I read it as a load flake like the p42/p53 ones in the backlog, but I have not explained it. |
| the targets after P62 (`cargo test --workspace --no-fail-fast --test p63_final_check_fixes --test p64_publish_speed_and_visibility`), `cargo test --workspace --doc` | P63 4/4 (4.1 s), P64 8/8, doc-tests 0 |
| `cargo test -p psm-testkit --test p48_camera_ownership` | 32/32, finished in 10.23 s |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean |
| `cargo fmt --all -- --check` | clean |
| `npm run typecheck` (apps/desktop, Node 22) | clean |
| `npm test` | 64 files, **442/442** (the known-intermittent `p33_split_grid` passed this time) |
| `tools/sidecar/build.sh --stub --test` | **31 tests, 0 failed** |
| `tools/sidecar/build.sh --sdk <lab SDK>` | compiles and links; the stub was rebuilt afterwards (binary is gitignored) |

## Manual try-out
1. `cd ~/Desktop/Developer/Photo-Session-Manager && cargo test -p psm-testkit --test p63_final_check_fixes`: `4 passed`.
2. `cargo test -p psm-testkit --test p48_camera_ownership`: `32 passed`, finished in about 10 s.
3. `tools/sidecar/build.sh --stub --test`: `31 tests, 0 failed`.
4. In the app: open Settings, change any field, press Escape or Back. `Save your changes?` appears. Try Cancel, Discard and Save.
   Scroll to the top: the Save button stays visible at the bottom of the window.
5. With Capture One open: Settings → Style is a drop-down (No style, your styles, built-in). Choose `test-style`, Save. With Capture One
   closed the field is text with "Couldn't read Capture One's styles".
6. With the camera: plug the cable in while Capture One runs. The take-back should run with no Capture One relaunch until the app has
   the cable (one relaunch afterwards), and no "stopped at step 3" in the log.
7. With the camera: unplug the cable. Wi-Fi should be ready in about 6 s (`recovery_ms` in the log's `connection.ready`). This needs a
   helper built with the SDK (`tools/sidecar/build.sh --sdk …`).
8. In Settings choose another session while the camera is connected. The helper restarts within a second or two, with its staging under
   the new session (`sdk.save_info save_dir=…` in the log).
9. Rename the chosen session's folder in Finder. Within a second the camera header and Health say the session is missing. Camera uploads
   wait on the camera. Rename it back and everything resumes.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
