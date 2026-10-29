---
plan: P23_shape_on_arrival
role: implement
round: 1
state: DONE
date: 2026-10-29
actor: codex
---

# What was built

New captures and imported RAWs now ask Capture One to apply the configured crop shape once, Square by default. A hand-made crop is left intact. The request reads and writes inside one JXA call on the serialized provider lane. Failed requests retry with exponential back-off, give up after eight attempts, and appear in Health; Retry checks and a real restart requeue them. An unknown crop name appears separately in Health and makes no crop write. Settings offers Capture One's ratio names and Leave as shot; tiles fit the whole preview inside their existing frames.

# Tests → criterion rows

| Row | Behaviour test | Boundary and exact outcome |
|---|---|---|
| C1(a) | `p23_shape_on_arrival::c1a_untouched_frame_is_shaped_in_one_call` | Node runs emitted JXA; applies Square with one non-applying maximum read and one write. |
| C1(b) | `c1b_hand_crops_are_skipped` | Node JXA skips fixed-ratio and resized Unconstrained crops without writing. |
| C1(c) | `c1c_unknown_ratio_and_unreadable_max_never_write` | Node JXA reports unknown names, throws on unreadable maximum, and lists eleven ratios. |
| C2(a) | `c2a_capture_shapes_without_retouch` | Capture hook creates one done crop job, Square variant, own-write state, no retouch or change-sequence bump on echo. |
| C2(b) | `c2b_import_attempts_each_raw_only_once` | Import shapes untouched RAW, skips hand crop, and does not reapply after relaunch and repeat import. |
| C2(c) | `c2c_off_duplicate_delivery_and_copy_move_do_not_shape_again` | Off enqueues nothing; hook and watcher produce one job; copy/move enqueue nothing. |
| C3(a) | `c3a_missing_variant_retries_then_folder_variant_applies` | Variant-not-found backs off one second, then folder fallback applies Square and clears health. |
| C3(b) | `c3b_retrying_shape_marks_preview_stale_and_rerenders` | Failed first attempt retries; later applied crop marks preview stale and issues another render without a hook. |
| C3(c) | `c3c_give_up_health_requeues_and_success_clears` | Eight failures terminate; Health counts and requeues; success clears on next check; real restart requeues. |
| C3(d) | `c3d_unknown_name_has_distinct_health_then_retries_after_correction` | Unknown name fails without crop write, stays separate from not-applied count, then applies after correction and Retry checks. |
| C4(a) | `c4a_setting_defaults_validates_and_lists_provider_names` | Missing setting defaults to Square; off persists; invalid values reject; configured fake provider lists eleven names. |
| C4(b) | `p23_shape_on_arrival.test.tsx` first test | Select lists provider names plus off, saves off and 4x5, and retains an unknown stored value without an alert. |
| C5(a) | `p23_tile_fit.test.tsx` | Loaded stylesheet has tile and stage `contain` and keeps the 4:3 tile. |
| C5(b) | `p23_shape_on_arrival.test.tsx` second test | Both crop health failures show the exact detail under the Shape on arrival label. |
| C6(a) | Optional live probe, not run | `maximumCrop(variant, {apply:false})` return shape and side effect await a live Capture One probe; no camera or Capture One is needed by the suite. |

# Assumptions

- `TestApp::new()` has no configured Capture One document, so the provider-list part of C4(a) uses a configured fake document. The default and validation parts still use `TestApp::new()`.
- The ratios script searches document images, then collection images, for the first variant it can find. The select is native. The crop script compares `cropWidth` and `cropHeight` with the maximum rectangle.
- The optional live probe was left for a session with Capture One ready; the fake and Node harness cover the automated rows.

# Deviations

- `apps/desktop/src-tauri/src/commands/items.rs` needed the new error code in its exhaustive `Error` match.
- P05 C5(c) now expects four arrival jobs. P17 C5(a)'s render-only fixture sets shape on arrival to off so that crop-triggered preview staleness does not change its render retry schedule. These two inherited tests are outside the plan's listed test files.
- C5(a) reads `workspace.css` with Node's filesystem API. Vitest returned an empty string for the planned `?raw` import in this project's test configuration.
- C6(a) was not run; it is optional in the phase plan. There is no live evidence yet for the maximum-crop return form or the `apply:false` side effect.

# Files changed

Checkpoint `0476e661d60a2e79e4507be143fcb85eb2c52d5f` contains exactly these 39 paths:

- `apps/desktop/src-tauri/src/commands/items.rs`
- `apps/desktop/src-tauri/src/commands/settings.rs`
- `apps/desktop/src-tauri/src/lib.rs`
- `apps/desktop/src/features/health/HealthPage.tsx`
- `apps/desktop/src/features/pending/__tests__/p18_pending_state.test.tsx`
- `apps/desktop/src/features/settings/SettingsPage.tsx`
- `apps/desktop/src/features/settings/__tests__/p16_settings.test.tsx`
- `apps/desktop/src/features/settings/__tests__/p23_shape_on_arrival.test.tsx`
- `apps/desktop/src/features/shell/__tests__/p19_navigation.test.tsx`
- `apps/desktop/src/features/shell/__tests__/p22_shell_tidy.test.tsx`
- `apps/desktop/src/features/workspace/__tests__/p23_tile_fit.test.tsx`
- `apps/desktop/src/features/workspace/workspace.css`
- `apps/desktop/src/services/bindings.ts`
- `apps/desktop/src/services/settings.ts`
- `crates/core/src/domain/editing.rs`
- `crates/core/src/error.rs`
- `crates/core/src/ports/editing.rs`
- `crates/core/src/ports/store.rs`
- `crates/core/src/services/arrival.rs`
- `crates/core/src/services/editing_startup.rs`
- `crates/core/src/services/settings.rs`
- `crates/infra/src/jobs.rs`
- `crates/infra/src/settings_file.rs`
- `crates/infra/src/sqlite/store.rs`
- `crates/integrations/src/editing/capture_one/adapter.rs`
- `crates/integrations/src/editing/capture_one/gate.rs`
- `crates/integrations/src/editing/capture_one/jxa.rs`
- `crates/integrations/src/editing/capture_one/jxa/apply_crop.js`
- `crates/integrations/src/editing/capture_one/jxa/crop_aspect_ratios.js`
- `crates/integrations/src/editing/capture_one/mod.rs`
- `crates/integrations/src/editing/capture_one/scripting.rs`
- `crates/integrations/src/editing/capture_one/shape.rs`
- `crates/integrations/src/lib.rs`
- `crates/testkit/src/fake_capture_one.rs`
- `crates/testkit/tests/p05_capture_arrival.rs`
- `crates/testkit/tests/p17_real_run_fixes.rs`
- `crates/testkit/tests/p23_shape_on_arrival.rs`
- `docs/implementation/master_plan.md`
- `docs/implementation/plans/P23_shape_on_arrival.md`

This handoff is left for the runner to commit as required by master plan §3.

# Checkpoint SHA

`0476e661d60a2e79e4507be143fcb85eb2c52d5f` — `CHECKPOINT (not approved): implement P23 shape on arrival`.

# Stamp

| Command | Result |
|---|---|
| `cargo test -p psm-testkit --test p23_shape_on_arrival` | 11 passed |
| `cargo test --workspace` | exit 0, all Rust tests passed |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo fmt --all -- --check` | exit 0 |
| `cd apps/desktop && npm run typecheck` | exit 0 |
| `cd apps/desktop && npm test` | 17 files, 127 passed |
| `git diff --check` | exit 0 |

The first full stamp exposed two inherited assertions affected by the new job and crop-triggered preview refresh. Both were adjusted as described above; the final stamp is green.

# Manual try-out

1. Open a Capture One session and the app, then shoot one frame. After the preview refresh, its tile and stage should show a whole square photo; the same Capture One variant should show Square crop.
2. Change that crop by hand in Capture One, then focus and relaunch the app. The hand crop should stay as you set it.
3. Drop an uncropped RAW from Finder into the session. It should become square. Drop it again; Capture One should not receive another shape write.
4. In Settings, choose Leave as shot and Save settings. The next frame should stay as shot. Choose Square again afterward.
5. With Capture One unavailable, import a RAW and wait for retries to finish. Health should say Shape not applied to 1 photo. Start Capture One and choose Retry checks; the crop should apply and the health item should clear on the following check.
6. For an unknown-name check, set `crop_on_arrival` to `Squircle` in the app's settings file and relaunch. Health should show the unknown ratio message. Correct the name in Settings, choose Retry checks, and confirm the waiting frame becomes square.
7. Optional live script probe C6(a) remains to be run with the exact instructions in the plan; restore its test frame with `maximumCrop` afterward.

# ⚠ OWNER DECISIONS REQUIRED (0)

None.
