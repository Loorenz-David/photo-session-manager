---
plan: P28_composition_foundation
role: implement
round: 1
state: DONE
date: 2026-11-04
actor: codex
---
# P28 implement r1 handoff

## What was built

Each photo now has a saved composition. New captures and imports begin with the configured default shape; the first adjusted preview supplies the frame needed to centre its rect. Existing photos receive default rows and a one-time preview refresh without becoming dirty. The backend validates saved framing, tracks revisions and published-session changes, flags Capture One geometry or orientation changes for review, and makes a 640 px tile thumbnail. `composition_set`, `composition_reset`, and `session_get` expose this data through typed commands. The editor and final-output renderer remain in later phases.

## Tests → criterion rows

| Row | Boundary test | Outcome |
|---|---|---|
| C1(a) | core `c1a_default_rect` | 11 centred default rectangles across shapes, turns, and split |
| C1(b) | core `c1b_pixel_rect_and_halves` | Half-away pixel rounding and adjacent equal split squares |
| C1(c) | core `c1c_validate` | 13 validation inputs produce the first specified result |
| C1(d) | core `c1d_fingerprint_and_inside` | 12 fingerprint results and two straighten-corner results |
| C2(a) | testkit `c2a_capture_arrival_has_frameless_default` | Capture owns a frameless default row without crop write |
| C2(b) | testkit `c2b_first_preview_materialises_default_and_thumbnail` | First frame sets square rect and fingerprint; exact 640 × 427 thumbnail path |
| C2(c) | testkit `c2c_original_default_uses_whole_frame` | Original shape uses the full frame |
| C2(d) | testkit `c2d_copy_preserves_default_and_user_composition` | Copies preserve default and user rows except photo ID |
| C2(e) | testkit `c2e_undecodable_preview_keeps_composition_frameless` | Undecodable preview is recorded without thumbnail or fingerprint |
| C3(a) | testkit `c3a_valid_save_survives_restart` | Valid user save and revision read back identically after restart |
| C3(b) | testkit `c3b_invalid_saves_leave_row_unchanged` | Four invalid saves return exact reasons without writes or events |
| C3(c) | testkit `c3c_identical_save_is_noop` | Identical save leaves revision, timestamp, sequence, and events unchanged |
| C3(d) | testkit `c3d_published_photo_composition_change_dirties_session` | Visible published photo change increments sequence and dirties |
| C3(e) | testkit `c3e_deleted_photo_change_does_not_dirty` | Deleted photo save does not increment sequence or dirty |
| C3(f) | testkit `c3f_frameless_save_is_refused` | Unknown frame refuses save and retains frameless row |
| C4(a) | testkit `c4a_reset_uses_current_default_and_is_idempotent` | Reset uses current 4:5 default, emits once, second reset is no-op |
| C5(a) | testkit `c5a_geometry_change_flags_review_and_replaces_fingerprint` | 3° source rotation flags geometry review and keeps rect |
| C5(b) | testkit `c5b_compatible_preview_sizes_keep_composition` | Compatible preview sizes keep framing and ok state |
| C5(c) | testkit `c5c_save_confirms_needs_review` | Save clears review and retains confirmed values |
| C5(d) | testkit `c5d_shared_raw_marks_both_rows_for_review` | Both composition rows on a RAW become needs_review |
| C5(e) | testkit `c5e_orientation_change_flags_frame_review` | Portrait source flags frame review and keeps rect |
| C6(a) | testkit `c6a_restart_backfills_missing_composition_rows_without_dirty` | Backfill restores three defaults without sequence change |
| C6(b) | testkit `c6b_real_start_rerenders_old_previews_once_without_retouch` | One-time stale mark changes generation only and reissues preview |
| C7(a) | testkit `c7a_session_view_aligns_compositions_and_thumbnails` | Session view aligns each photo, composition, and thumbnail |
| C7(b) | desktop `c7b_commands_validate_and_serialize_compositions` | Set/reset command DTO, exact errors, and session_get fields |

## Assumptions

- Thumbnail JPEG quality is 80; `image::imageops` Lanczos3 resizes it. `image::image_dimensions` reads the source dimensions without pixel decoding.
- A compatible preview retains its stored preview fingerprint. A later master may replace that fingerprint; preview pixel sizes are not compared.
- The testkit's `restart_with_startup` represents a real start. The plan's Read first reference called `kill_and_restart` a real start, but its implementation passes `false` for startup.

## Deviations

- `Cargo.lock` changed for the new image crates. The P01 schema assertion, P17 JXA script fixture, and P27 migration fixture were updated because they encoded earlier table or script shapes. Shell and pending photo fixtures also needed the new required composition field; the plan's fixture list omitted them. These are the only paths beyond the enumerated perimeter.
- No red baseline was captured before the first production edit. All 25 criterion-row tests and the final full suite pass.
- No live Capture One geometry probe was run; the adapter's script harness and fake render path were tested.

## Files changed

The checkpoint contains these 56 paths:

```text
Cargo.lock
apps/desktop/src-tauri/src/commands/compositions.rs
apps/desktop/src-tauri/src/commands/items.rs
apps/desktop/src-tauri/src/commands/mod.rs
apps/desktop/src-tauri/src/commands/sessions.rs
apps/desktop/src-tauri/src/lib.rs
apps/desktop/src/features/form/__tests__/p11_form_ui.test.tsx
apps/desktop/src/features/pending/__tests__/p18_pending_state.test.tsx
apps/desktop/src/features/publish/__tests__/p15_publish_ui.test.tsx
apps/desktop/src/features/shell/__tests__/p19_navigation.test.tsx
apps/desktop/src/features/workspace/__tests__/p08_workspace_ui.test.tsx
apps/desktop/src/features/workspace/__tests__/p19_drop.test.tsx
apps/desktop/src/features/workspace/__tests__/p20_drop_continuity.test.tsx
apps/desktop/src/features/workspace/__tests__/p22_workspace_tidy.test.tsx
apps/desktop/src/features/workspace/__tests__/p26_double_drag.test.tsx
apps/desktop/src/services/bindings.ts
crates/core/src/domain/composition.rs
crates/core/src/domain/mod.rs
crates/core/src/domain/photo.rs
crates/core/src/domain/session.rs
crates/core/src/error.rs
crates/core/src/mechanisms/composition.rs
crates/core/src/mechanisms/composition_cases.json
crates/core/src/mechanisms/mod.rs
crates/core/src/ports/editing.rs
crates/core/src/ports/images.rs
crates/core/src/ports/mod.rs
crates/core/src/ports/store.rs
crates/core/src/services/composition.rs
crates/core/src/services/mod.rs
crates/core/src/services/previews.rs
crates/core/src/services/sessions.rs
crates/core/src/services/settings.rs
crates/infra/Cargo.toml
crates/infra/src/imaging.rs
crates/infra/src/lib.rs
crates/infra/src/previews_store.rs
crates/infra/src/sqlite/migrations/004_compositions.sql
crates/infra/src/sqlite/store.rs
crates/integrations/src/editing/capture_one/adapter.rs
crates/integrations/src/editing/capture_one/gate.rs
crates/integrations/src/editing/capture_one/jxa.rs
crates/integrations/src/editing/capture_one/jxa/process.js
crates/integrations/src/editing/capture_one/scripting.rs
crates/integrations/src/lib.rs
crates/testkit/Cargo.toml
crates/testkit/src/fake_capture_one.rs
crates/testkit/src/fixtures/mod.rs
crates/testkit/src/fixtures/synthetic_jpeg.rs
crates/testkit/src/test_app.rs
crates/testkit/tests/p01_storage_identity.rs
crates/testkit/tests/p17_real_run_fixes.rs
crates/testkit/tests/p27_retire_shape_on_arrival.rs
crates/testkit/tests/p28_composition_foundation.rs
docs/implementation/master_plan.md
docs/implementation/plans/P28_composition_foundation.md
```

This handoff is the remaining changed path for the runner to commit.

## Checkpoint SHA

`9b24cbec2ab3a36a7cda2a896917b671bd7b88fc` — `CHECKPOINT (not approved): implement P28 composition foundation`.

## Stamp

| Command | Result |
|---|---|
| `cargo test --workspace` | PASS, 481 Rust tests; 0 failures |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo fmt --all -- --check` | PASS |
| `cd apps/desktop && npm run typecheck` | PASS |
| `cd apps/desktop && npm test` | PASS, 131 UI tests in 20 files |
| `git diff --cached --check` before checkpoint | PASS |

Earlier runs exposed stale P01, P17, and P27 test fixtures; all were updated and passed in the final run. The final test tree is the checkpoint above, with this handoff added afterward.

## Manual try-out

1. Launch the app and open an existing session. Older adjusted previews should refresh once, without making the session need publishing. Tiles still show whole frames until the editor phase.
2. Shoot or import a frame. After its adjusted preview appears, inspect `~/Library/Application Support/com.loorenz.sessionmanager/previews/`: the photo's folder should contain `adjusted-<n>.jpg` and a `thumb-<n>.jpg` whose long edge is 640 px.
3. Straighten that photo by about 3° in Capture One and wait for preview refresh. In the app database, run `sqlite3 session_manager.db "select photo_id,state,review_reason from compositions"` to see `needs_review` and `geometry` for the photo. A 90° orientation change produces `frame` instead. Confirmation UI arrives in a later phase.
4. Set Default shape to `4:5`, save, then shoot a frame. Its new composition row should have `shape = 4:5`.

## ⚠ OWNER DECISIONS REQUIRED (0)

None.
