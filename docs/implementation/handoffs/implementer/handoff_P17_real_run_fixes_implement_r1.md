---
plan: P17_real_run_fixes
role: implement
round: 1
state: DONE
date: 2026-10-26
actor: codex
---
# P17 implementer handoff, round 1

## What was built
Saving settings now rebuilds the Capture One and commerce adapters without a relaunch. Capture One scripting addresses the configured document on every per-document call; the generated scripts share one folder-collection scan and never use `collections.whose({file})`. The capture root comes from `<document id>/Capture`, so switching sessions does not nest capture folders. Missing roots fail closed with a health item.

All Tauri commands are async and run blocking work on worker threads through cloned app handles. A settings save prepares its new runtime outside the app lock, swaps it under the write lock, and stops the old workers afterward. Window focus schedules stale-preview work on a thread. Deferred preview renders use per-RAW exponential backoff and complete their queue job; a user-priority scripting gate serves waiting user calls ahead of background calls. The gate is shared across settings rebuilds.

## Tests → criterion rows

| Row | Boundary test and exact assertion |
|---|---|
| C1(a) | `c1a_settings_rebuild_addresses_new_document`: health document/hooks/recipes green after save, no document/recipe error, configured recipe reached fake. |
| C1(b) | `c1b_recipe_names_take_effect_without_relaunch`: one missing recipe named `Nope`, then none after correction. |
| C1(c) | `c1c_integration_rebuilds_without_network`: commerce/publish appear with Shopify settings and disappear when cleared; configured domain reads back. |
| C2(a) | `c2a_written_jxa_scripts_share_one_folder_scan`: generated scripts contain one prelude definition each and no `whose({file})`; all four lookup scripts use it. |
| C2(b) | `c2b_process_finds_image_while_browsing_other_collection`: generated process/tag scripts return a job id and tag 0 without changing the selected collection, using a Node-hosted JXA seam. Live Capture One confirmation remains for review. |
| C3(a) | `c3a_capture_root_comes_from_document_id`: root stays `<document>/Capture` while `captures` and the `capture` favourite move to a child. |
| C3(b) | `c3b_switches_never_nest`: exact A, A, `_Unassigned`, B capture-folder sequence. |
| C3(c) | `c3c_arrivals_follow_derived_root`: B RAW goes to B; `_Unassigned` RAW goes to session 1; exactly two rows. |
| C3(d) | `c3d_missing_capture_root_is_unhealthy`: inspect returns `CaptureFolderNotSet`, health names the missing path. |
| C3(e) | `c3e_document_info_reports_moving_capture_favourite_and_fixed_root`: generated JXA reports document id, moving `capture` favourite, fixed Capture collection and unchanged selection in a Node-hosted seam. Live Capture One confirmation remains for review. |
| C4(a) | `commands_are_async`: every command declaration is async and no command file reads or writes the app lock directly. |
| C4(b) | `commands_do_not_wait_on_each_other`: Tauri mock IPC `sessions_list` returns within 500 ms while `health_get` waits for a 1.5 s provider call. |
| C4(c) | `c4c_focus_returns_before_provider_render`: focus returns within 100 ms; a delayed provider receives the render within 2 s. |
| C5(a) | `c5a_failed_renders_back_off_and_finish_job`: attempts at exactly 0, 1, 3, 7, 15, 31 s, six lookups, queue job `done`. |
| C5(b) | `c5b_adjustment_resets_backoff_then_waits_delay`: no render before 55 s after a 40 s adjustment; one render at 55 s is in flight. |
| C5(c) | `c5c_user_folder_switch_preempts_background_jobs`: activation finishes within 1 s with at most one background scripting op between consecutive activation ops; ten mirror jobs finish. |

Core mechanism tests: `mc19_render_backoff` asserts 1, 2, 4, 8, 16, 32, 60, 60, 60 s; `mc2_capture_root_is_document_capture_directory` asserts the canonical document Capture directory and missing-root failure. Existing P04 and P16 tests were updated for the corrected root and generated JXA prelude.

## Assumptions

- Collection entries with no folder remain in `DocumentInfo`; folder lookup skips them.
- The user-priority gate is created once per app and reused across settings rebuilds.
- Focus starts a standard Rust thread; the Tauri event handler returns immediately.
- `JxaScripting::new` joins `lib.js` with every script at write time.
- A failed runtime build restores the prior settings file and keeps the old runtime active.

## Deviations

- `Cargo.lock` changed for the desktop test dependencies.
- P04 and P16 behavior tests were updated outside the listed perimeter: P04 asserted the superseded top-level Capture path, and P16 evaluated the source script without the newly required generated prelude and used the deprecated collection `file` property.
- `bindings.ts` was regenerated and matched the committed file, so it has no diff.
- Live C2(b) and C3(e) probes were deferred as task 6 permits when Apple events are unavailable. The prerequisite `osascript -l JavaScript -e 'Application("Capture One").running()'` exited 1 with `execution error: Error: Error: Application can't be found. (-2700)`. No real capture or capture-folder change was made in this session.

## Files changed

- `Cargo.lock`
- `apps/desktop/src-tauri/Cargo.toml`
- `apps/desktop/src-tauri/src/commands/attributes.rs`
- `apps/desktop/src-tauri/src/commands/camera.rs`
- `apps/desktop/src-tauri/src/commands/export.rs`
- `apps/desktop/src-tauri/src/commands/health.rs`
- `apps/desktop/src-tauri/src/commands/items.rs`
- `apps/desktop/src-tauri/src/commands/photos.rs`
- `apps/desktop/src-tauri/src/commands/publish.rs`
- `apps/desktop/src-tauri/src/commands/sessions.rs`
- `apps/desktop/src-tauri/src/commands/settings.rs`
- `apps/desktop/src-tauri/src/lib.rs`
- `apps/desktop/src-tauri/src/state.rs`
- `crates/core/src/mechanisms/backoff.rs`
- `crates/core/src/mechanisms/mod.rs`
- `crates/core/src/services/editing_startup.rs`
- `crates/core/src/services/previews.rs`
- `crates/integrations/src/editing/capture_one/adapter.rs`
- `crates/integrations/src/editing/capture_one/gate.rs`
- `crates/integrations/src/editing/capture_one/jxa.rs`
- `crates/integrations/src/editing/capture_one/jxa/document_info.js`
- `crates/integrations/src/editing/capture_one/jxa/find_image_in_folder.js`
- `crates/integrations/src/editing/capture_one/jxa/get_color_tag.js`
- `crates/integrations/src/editing/capture_one/jxa/lib.js`
- `crates/integrations/src/editing/capture_one/jxa/process.js`
- `crates/integrations/src/editing/capture_one/jxa/set_color_tag.js`
- `crates/integrations/src/editing/capture_one/mod.rs`
- `crates/integrations/src/editing/capture_one/scripting.rs`
- `crates/integrations/src/lib.rs`
- `crates/testkit/src/fake_capture_one.rs`
- `crates/testkit/src/test_app.rs`
- `crates/testkit/tests/p04_capture_one_adapter.rs`
- `crates/testkit/tests/p05_capture_arrival.rs`
- `crates/testkit/tests/p16_operations.rs`
- `crates/testkit/tests/p17_real_run_fixes.rs`
- `docs/implementation/master_plan.md`
- `docs/implementation/plans/P17_real_run_fixes.md`
- `docs/implementation/handoffs/implementer/handoff_P17_real_run_fixes_implement_r1.md` (this handoff; runner commits it)

## Checkpoint SHA

`7dd73eb8e52bbead147f5733eb1b4eb6c5bac184` — `CHECKPOINT (not approved): implement P17 real-run backend fixes`. The runner owns this handoff's later commit under flow §3.

## Stamp

| Command | Result |
|---|---|
| `cargo test --workspace --quiet` | Green, 405 Rust tests. |
| `cargo clippy --workspace --all-targets -- -D warnings` | Green. |
| `cargo fmt --all -- --check` | Green. |
| `cd apps/desktop && PATH=/Users/davidloorenz/.nvm/versions/node/v22.22.3/bin:$PATH npm run typecheck` | Green. |
| `cd apps/desktop && PATH=/Users/davidloorenz/.nvm/versions/node/v22.22.3/bin:$PATH npm test` | Green, 92 tests in 7 files. |
| `cargo test -p psm-desktop export_bindings` | Green; generated bindings unchanged. |
| `osascript -l JavaScript -e 'Application("Capture One").running()'` | Exit 1, `Application can't be found. (-2700)`; live probes await reviewer. |

## Manual try-out

1. Open the `session test 1` document in Capture One. From `apps/desktop`, run `npm run tauri -- dev` using the Node 22 path in the stamp if needed.
2. In Settings, save the document folder and both recipe names. Health should show document, hooks, recipes and capture folder green without relaunching.
3. In Capture One, verify that the capture folder is `<document>/Capture/_Unassigned`. Open an item session, return to Sessions, and reopen it; the folder should alternate between `_Unassigned` and `<folder key>` directly under `Capture`.
4. Shoot three frames and watch them appear with adjusted previews while Capture One browses another collection. During rendering, type in session search and open a session; the window should remain responsive.
5. If rendering fails, inspect the day's app log: retries should spread by 1, 2, 4, 8 seconds and longer, rather than every 2 seconds.

## ⚠ OWNER DECISIONS REQUIRED (0)

None.
