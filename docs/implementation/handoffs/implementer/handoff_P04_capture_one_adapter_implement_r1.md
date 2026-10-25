---
plan: P04_capture_one_adapter
role: implement
round: 1
state: DONE
date: 2026-10-25
actor: codex
---
# P04 Capture One adapter — implement round 1

## What was built

Session Manager can now inspect the configured Capture One session, install its three hooks, correct the two render recipes, switch the capture folder to a session or `_Unassigned`, and block capture when the folder cannot be verified. Camera status, retry and capture are exposed through thin desktop commands. Classification changes enqueue durable, coalesced tag jobs that read the tag back and retry transient failures. The scripting layer uses one JXA script per operation; variant operations resolve from the RAW and its folder collection when another collection is selected.

## Tests → criterion rows

Each row below has one boundary behavior test in `crates/testkit/tests/p04_capture_one_adapter.rs`:

| Row | Test |
|---|---|
| C1(a) | `c1a_activate_saves_creates_verifies_and_persists` |
| C1(b) | `c1b_failed_set_restores_and_blocks_camera` |
| C1(c) | `c1c_readback_mismatch_restores_and_blocks_camera` |
| C1(d) | `c1d_deactivate_targets_unassigned` |
| C2(a) | `c2a_hooks_read_back_canonically` |
| C2(b) | `c2b_null_hook_readback_is_unhealthy` |
| C3(a) | `c3a_existing_recipes_are_corrected` |
| C3(b) | `c3b_missing_recipe_only_blocks_its_render` |
| C4(a) | `c4a_render_picks_position_one_in_current_collection` |
| C4(b) | `c4b_render_falls_back_to_folder_collection` |
| C4(c) | `c4c_missing_variant_retries_and_reports_health` |
| C5(a) | `c5a_coalesced_changes_write_latest_once` |
| C5(b) | `c5b_dropped_tag_write_retries` |
| C5(c) | `c5c_own_write_timestamp_is_upserted` |
| C5(d) | `c5d_last_enqueued_copy_row_wins` |
| C5(e) | `c5e_four_classifications_map_to_tags` |
| C6(a) | `c6a_timeout_read_first_on_retry` |
| C7(a) | `c7a_connected_camera_name` |
| C7(b) | `c7b_absent_camera_is_disconnected` |
| C7(c) | `c7c_healthy_capture_reaches_provider` |
| C8(a) | `c8a_catalog_stops_before_hook_install` |

The plan's three core mechanisms also have inline table tests for tag mapping, retry backoff and `process` result parsing. The inherited P04 work had a green 21/21 boundary baseline when this continuation began; no red baseline was recorded before its earlier edits.

## Assumptions

- One mutex serializes JXA calls. Startup runs on a worker thread; provider commands release the app lock and use blocking workers.
- Health stores startup and provider results in a mutex, including positive items. A successful mirror clears `variant_not_found`.
- A tag retry reads first, skipping a second write if the timed-out write already took effect. New classifications coalesce only into queued jobs.
- The default `TestApp::new()` preserves the P00 empty-document setting; P04 uses `with_fake_document()` to configure the fake before startup.
- The JXA variant ID maps to its RAW within the scripting instance. It is resolved again after restart before a job uses it.

## Deviations

- `apps/desktop/src-tauri/src/commands/items.rs` was changed outside the plan's expected changed paths to map the new provider errors into command error codes.
- `crates/core/src/ports/clock.rs` was changed outside that list to forward the clock trait through `Arc`; existing P00 tests pass `&Arc<ManualClock>` and the provider injection needs `Arc<dyn Clock>`.
- The optional live Capture One smoke was not run. VERIFY-11 remains live-unprobed; the fake tests cover the planned folder fallback. No camera or Capture One is required by the suite.

## Files changed

Full checkpoint write perimeter:

- `apps/desktop/src-tauri/src/commands/camera.rs`
- `apps/desktop/src-tauri/src/commands/health.rs`
- `apps/desktop/src-tauri/src/commands/items.rs`
- `apps/desktop/src-tauri/src/commands/mod.rs`
- `apps/desktop/src-tauri/src/commands/sessions.rs`
- `apps/desktop/src-tauri/src/lib.rs`
- `apps/desktop/src/services/bindings.ts`
- `crates/core/src/domain/editing.rs`
- `crates/core/src/domain/mod.rs`
- `crates/core/src/error.rs`
- `crates/core/src/events.rs`
- `crates/core/src/ports/camera.rs`
- `crates/core/src/ports/clock.rs`
- `crates/core/src/ports/editing.rs`
- `crates/core/src/ports/mod.rs`
- `crates/core/src/ports/store.rs`
- `crates/core/src/services/camera.rs`
- `crates/core/src/services/editing_startup.rs`
- `crates/core/src/services/health.rs`
- `crates/core/src/services/mod.rs`
- `crates/core/src/services/review.rs`
- `crates/core/src/services/sessions.rs`
- `crates/infra/src/jobs.rs`
- `crates/infra/src/lib.rs`
- `crates/infra/src/sqlite/store.rs`
- `crates/integrations/src/camera/mod.rs`
- `crates/integrations/src/camera/via_capture_one.rs`
- `crates/integrations/src/editing/capture_one/adapter.rs`
- `crates/integrations/src/editing/capture_one/jxa.rs`
- `crates/integrations/src/editing/capture_one/jxa/capture.js`
- `crates/integrations/src/editing/capture_one/jxa/document_info.js`
- `crates/integrations/src/editing/capture_one/jxa/find_image.js`
- `crates/integrations/src/editing/capture_one/jxa/find_image_in_folder.js`
- `crates/integrations/src/editing/capture_one/jxa/get_captures.js`
- `crates/integrations/src/editing/capture_one/jxa/get_color_tag.js`
- `crates/integrations/src/editing/capture_one/jxa/get_hook.js`
- `crates/integrations/src/editing/capture_one/jxa/import.js`
- `crates/integrations/src/editing/capture_one/jxa/process.js`
- `crates/integrations/src/editing/capture_one/jxa/recipe_exists.js`
- `crates/integrations/src/editing/capture_one/jxa/save.js`
- `crates/integrations/src/editing/capture_one/jxa/set_captures.js`
- `crates/integrations/src/editing/capture_one/jxa/set_color_tag.js`
- `crates/integrations/src/editing/capture_one/jxa/set_hook.js`
- `crates/integrations/src/editing/capture_one/jxa/set_recipe_props.js`
- `crates/integrations/src/editing/capture_one/mirror.rs`
- `crates/integrations/src/editing/capture_one/mod.rs`
- `crates/integrations/src/editing/capture_one/scripting.rs`
- `crates/integrations/src/lib.rs`
- `crates/testkit/src/fake_capture_one.rs`
- `crates/testkit/src/lib.rs`
- `crates/testkit/src/test_app.rs`
- `crates/testkit/tests/p04_capture_one_adapter.rs`
- `docs/implementation/handoffs/implementer/handoff_P04_capture_one_adapter_implement_r1.md`
- `docs/implementation/master_plan.md`
- `docs/implementation/plans/P04_capture_one_adapter.md`

## Checkpoint SHA

This handoff is included in the checkpoint commit. Resolve its SHA with `git log -1 --format=%H -- docs/implementation/handoffs/implementer/handoff_P04_capture_one_adapter_implement_r1.md` (the exact SHA cannot be embedded in a file that is part of its own commit).

## Stamp

| Command | Result |
|---|---|
| `cargo test --workspace` | PASS; 21/21 P04 behavior rows and all earlier suites |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo fmt --all -- --check` | PASS |
| `cd apps/desktop && npm run typecheck` | PASS |
| `cd apps/desktop && npm test` | PASS; no frontend test files yet |

An earlier workspace run failed because the partial P04 harness changed the P00 settings file mode and default `document_id`; the harness was corrected, P00 and P04 targeted tests passed, and the final workspace run passed.

## Manual try-out

1. In `~/Library/Application Support/com.loorenz.sessionmanager/settings.json`, set `document_id` to the Capture One session folder. Create recipes named `PSM Preview` and `PSM Web` in that session.
2. Open the session in Capture One, then from `apps/desktop` run `npm run tauri -- dev`. The Health page shows the session, hooks, recipes, and camera status. The camera can be disconnected for this check.
3. Run `osascript -l JavaScript -e 'Application("Capture One").captureDoneScript().toString()'` to see the installed hook path under the app data folder.
4. Rename one recipe, relaunch Session Manager, and refresh Health. It shows the missing recipe. Quit Capture One and relaunch: the app reports the document closed without starting Capture One.

## ⚠ OWNER DECISIONS REQUIRED (0)

None.
