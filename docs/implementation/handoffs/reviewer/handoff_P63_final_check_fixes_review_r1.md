---
plan: P63_final_check_fixes
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-10
actor: opus (pipeline-reviewer)
---
# P63 final-check fixes: review r1

**Verdict: APPROVED_WITH_NOTES.** No B1–B4 finding. Eight notes went to `docs/implementation/backlog.md`.

## Verified correct
- **Gate.** The tracker showed `IMPLEMENTED`. Checkpoint `0247c78` is in `git log`.
- **Perimeter.** `git show --stat 0247c78` lists 35 paths. That equals the handoff's declared list: 27 production files, 5 test and harness files, the tracker and the plan log. 23 of them are outside the plan's named files. All 23 are disclosed and needed by the rows (the port and domain types, the Tauri command, the generated bindings, the routes guard, the fake).
- **Evidence the reviewer ran** (tree `0247c78`, time-boxed per the plan's Review mode):

  | Check | Result |
  |---|---|
  | `p63_final_check_fixes` | 4/4 |
  | `p48_camera_ownership` | 32/32, finished in 10.3 s |
  | `p54` | 6/6 |
  | `p57` | 8/8 |
  | `p60` | 8/8 |
  | `p62_retention_after_publish` | 12/12 (the implementer's full-run failure does not reproduce) |
  | `cargo test -p psm-core -p psm-integrations -p psm-desktop` | green |
  | `cargo clippy --workspace --all-targets -- -D warnings` | clean |
  | `cargo fmt --all -- --check` | clean |
  | `npm run typecheck` | clean |
  | `npm test` | 64 files, 442/442 |
  | `tools/sidecar/build.sh --stub --test` | 31/0 |

  The full workspace run is trusted from the implementer's stamp. That stamp is partial; see the notes.
- **C1(a).** This row is about scripts reaching Capture One during a take-back.
  - The code: `GatedScripting::call` refuses every script that is not on the permitted thread, in two cases. One: the take-back holds the gate. Two: the P54 relaunch wait applies and Capture One is not running.
  - `closed_here` is checked before the lane is taken and again once it is.
  - `launch` keeps its start-order wait, so it stays the one explicit launch step.
  - `quit` and `editor_info` are refused while the take-back holds the gate.
  - The follower's `running()` and the take-back's `running_now()` now call `is_running`. That reads `ps -axo comm=` with a 200 ms cache, and matches the same `Capture One` process name that `ownership.rs` already relies on.
  - Render-path scripts moved from `rendering` to `launching`. `launching` adds the R-I32 hold refusal and then goes through `call`.
  - The test: the fake relaunches on any Apple event (`apple_events_launch`). The test asserts that the ops between `quit` and `launch` are exactly `["editor_info"]` (step 5's own check), that there is one launch, that nothing launches before the cable session, and the report `{5, true}`. A follower `find_image`, a render or an `editor_info` poll in that window would fail it.
- **C2(a).** This row is about a missing session chosen in Settings.
  - `document_present` (the folder plus a `.cosessiondb` in it) gates three things:
    - the receiver's capture root, so uploads get `451`;
    - the helper spawn, so no staging folder is created (`spawn_deferred`);
    - the ensure, which sets the `document_missing` health item and returns `DocumentNotOpen` before `make_document`.
  - The rule applies only when `document_id` is set (`with_configured_document`, `AppBinding::document_configured`).
  - `NeedsActionReason::DocumentMissing` puts the exact sentence in the header.
  - The test asserts the Health detail, the header reason and message, `451`, that the path is still absent after an activation and 5 ticks, and that there is no `make_document`.
- **C3(a).** This row is about the helper's staging following the session.
  - `check_document` compares the current staging with `staging_for_document()`. It restarts the helper only when no take-back, open upload or expected transfer is in progress.
  - The old staging folder is removed only if it is empty (`remove_dir`), so H2 holds.
  - The test keeps an upload open, sees `argv` unchanged for 800 ms, then sees the second spawn with `<new>/Capture/.incoming/sdk-<run>`.
- **C4(a).** This row is about Save in Settings.
  - The Save button sits in a sticky footer (`position: sticky; bottom: 0`).
  - The `goTo` leave guard is used by Back, Escape/⌘[ (`useBackKeys`) and the Health button.
  - The dialog `Save your changes?` offers Cancel (stay), Discard (reset, then leave) and Save (`save.run()` returns `true` on success, then leaves).
  - Four UI tests cover the outcomes, including Discard leaving without saving and no prompt when nothing changed.
- **C5(a).** This row is about Wi-Fi recovery after an unplug.
  - In `transport_manager.cpp`, the cached endpoint is retried at once inside `cached_endpoint_window_ms` and then forgotten. An empty discovery is retried at once inside `recovery_fast_window_ms`.
  - `sdk_adapter.cpp` caches the endpoint of its own Wi-Fi connect.
  - The valid-endpoint test asserts `recovery_ms ≤ 6100` at the log's SDK costs, with the camera reachable at +3.7 s.
  - The cause (discovery came back empty, then a backoff, then a second discovery) is written in the handoff with timings.
- **C6(b).** This row is about the style drop-down.
  - The UI lists `No style`, then the user styles, then the built-in ones, in that order. Choosing `test-style` stores that exact name. When the list cannot be read, the field is text with the sentence and stays editable.
  - On the backend, the test checks the user/built-in split and that a closed Capture One gets no op. `GatedScripting::list_styles` checks `is_running` first.
  - `availableStyles` exists in the sdef (line 297).
- **C6(a) and C7(a).** Both are green as listed above. The p48 speed-up has a credible cause: `App::shutdown` joined a spool thread stuck inside a slow FSEvents start. No production timing value changed.
- **Assumptions.** None contradicts the intention. Each one, checked against the book:
  - The missing-session rule covers only a session chosen in Settings. This matches 10C step 2.
  - The header sentence shows only with the direct camera.
  - The set of protected operations for the restart.
  - `editor_info` still runs in the P54 wait. Its `running()` check is safe on a Capture One that has quit, and the ensure needs it to reach `launch`.
  - The recovery windows.
  - User styles are recognised by their `.costyle` files.
  - A stored style name that Capture One does not list is kept as an option.

  All of them are reported to the owner in the narrative.
- **H1–H8.**
  - H1: no new external write.
  - H2: no RAW is touched, and only an empty staging folder is ever removed.
  - H4: the restart and the deferral are idempotent.
  - H5: AppleScript dictionary and process list only.
  - H6: commerce is untouched.
  - H8: the Tauri command `editor_styles` is thin.
  - H7: the `.cosessiondb` check lives in core. That is a placement slip with no core logic depending on it. It is recorded as a note, following the P53 precedent for device-specific text in core.
  - No live services in the suite.

## Findings
None blocking (B1–B4).

## Notes written to backlog
1. **H7 placement.** `document_present` (the `.cosessiondb` check) lives in `psm_core::services::editing_startup`, and only integrations call it. Move it into `editing/capture_one`.
2. **Only the valid cached endpoint meets ~6 s.** A stale endpoint takes about 11.5 s and no endpoint about 10.8 s with the fake's timings. How fast the real SDK fails a direct connect is unmeasured.
3. **`restart_for_document` appends `--ip/--mac` after the spawn.** The restarted helper itself starts without the cached endpoint; only later respawns get it.
4. **The leave guard covers Back, Escape/⌘[ and Health only.** Quitting and direct hash changes do not ask.
5. **The style list comes from `availableStyles()`, which also includes presets.** The built-in group may be long or noisy.
6. **Shutdown no longer waits for a spool thread stuck in its watcher start.** The thread can briefly outlive its App on a reopen.
7. **The closing stamp is partial.** The workspace run stopped at a p62 flake. The reviewer re-ran p62 and the touched camera suites green. One full run on a quiet Mac is still owed.
8. **Book and registry drift.** 10E R-I20 is superseded by #28, and the new names are not in the registry. The `p48 c3c` fixture changed beyond timing (disclosed, needed by #27).

## Lessons for the plans
- Row C5(a) says "(cached endpoint valid / invalid) … ≤ ~6 s". With the camera's remote only back at +3.7 s and a 3.8 s discovery, ≤ 6 s is physically reachable only with a valid cached endpoint. Rows that bundle cases should state one bound per case.
- The plan's p48 line said "timing only, same assertions", but #27 forced a fixture change in `c3c`. When a phase changes a behaviour that an older test relies on as setup, the plan should allow that test's fixture to change.
- Two criteria share the label "C6" (C6(b) for styles and C6(a) for p48). Labels should be unique.

## Probe declaration
The reviewer made no production edits and applied no probes. Files the reviewer wrote: this handoff, `docs/implementation/narratives/P63_final_check_fixes.md`, the P63 row in `docs/implementation/master_plan.md`, the Review log entry in `docs/implementation/plans/P63_final_check_fixes.md`, and eight appended lines in `docs/implementation/backlog.md`. Test logs went to `/tmp/p63rev/` (outside the repo). No state changed outside the repo apart from build artifacts in `target/`.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
