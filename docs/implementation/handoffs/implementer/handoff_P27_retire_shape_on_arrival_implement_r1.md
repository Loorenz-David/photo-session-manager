---
plan: P27_retire_shape_on_arrival
role: implement
round: 1
state: DONE
date: 2026-11-04
actor: codex
---
# P27 implement r1 handoff

## What was built

New captures and imports no longer ask Capture One to set a crop. Settings offers Default shape and Master recipe, reads old crop settings once through the fixed mapping, and removes the old key on save. Provisioning now repairs PSM Preview to full-frame 2560 px JPEG and creates an uncropped, uncompressed 16-bit PSM Master. PSM Web remains for export in this interim phase. Migration 003 removes queued and failed jobs of the retired crop kind; old health labels and the crop command are gone.

## ⚠ OWNER DECISIONS REQUIRED (0)

None.

## Tests → criterion rows

| Row | Boundary test | Exact outcome covered |
|---|---|---|
| C1(a) | `c1a_capture_arrives_without_crop_job_or_crop_write_and_renders_preview` | Three initial jobs, no crop op, PSM Preview issued |
| C1(b) | `c1b_import_arrives_without_crop_job_or_crop_write` | Three initial jobs and no crop op |
| C2(a) | `c2a_missing_new_keys_use_defaults_and_save_them` | Defaults returned and persisted |
| C2(b) | `c2b_legacy_shapes_map_and_disappear_after_save` | Six exact legacy mappings and old key removed |
| C2(c) | `c2c_settings_validate_shape_and_master_recipe` | Allowed values save; invalid values return the exact key error |
| C3(a) | `c3a_ensure_repairs_preview_to_2560_crop_ignored` | Preview recipe repaired to the specified fields |
| C3(b) | `c3b_ensure_creates_uncompressed_master` | Master created and read back with all specified fields |
| C3(c) | `c3c_web_remains_and_renamed_master_leaves_old_recipe` | Web remains ensured; renamed Master added without touching old name |
| C3(d) | `c3d_only_drifted_preview_property_is_written` | Only `exportCropMethod` repaired |
| C3(e) | `c3e_jxa_repairs_master_fields_and_reads_them_back` | JXA writes two drifted properties, leaves bits, reads all fields |
| C4(a) | `c4a_migration_removes_old_jobs_and_health_codes` | Old jobs deleted on restart, no retired health item or unknown retry |
| C4(b) | `C4(b) retired crop health code displays its raw name` | Health label is raw code |
| C5(a) | `C5(a) Settings shows the seven default shapes and saves the master recipe` | Select order/text, Master field and save patch |
| C6(a) | Optional live row, not run here | VERIFY-15 grounds property values; reviewer can run where Apple events are permitted |

Rust boundary tests are in `crates/testkit/tests/p27_retire_shape_on_arrival.rs`; UI tests are in `apps/desktop/src/features/settings/__tests__/p27_default_shape.test.tsx`. No core-mechanism units were added because the plan lists none.

## Assumptions

- The plan's legacy mapping is used literally: `off` → `original`, `4x5` → `4:5`, `2x3` → `2:3`, `16x9` → `16:9`, and all other legacy names → `1:1`.
- The new Master recipe is ensured before Web; Web still renders interim exports.
- The constructor keeps the plan's required argument order. A local `clippy::too_many_arguments` allowance avoids changing that interface.
- C6(a) was left for an Apple-events-capable run. The master plan records the Codex sandbox's Apple-events refusal in earlier phases; no live command was attempted here, and the suite has no Capture One dependency.

## Deviations

- The plan did not list `apps/desktop/src-tauri/src/commands/items.rs`; its match on the removed `CropRatioUnknown` variant had to be deleted to compile.
- The plan did not list `crates/testkit/tests/p04_capture_one_adapter.rs`; its pre-Web failure fixture had to include Master so it still tests the Web failure, and its successful ensure expectation now includes Master.
- Earlier test fixtures for the removed setting were updated or deleted as directed by the plan. No other phase behavior was changed.

## Files changed

Checkpoint perimeter (status prefixes are `M` modified, `A` added, `D` deleted):

- `M	apps/desktop/src-tauri/src/commands/items.rs`
- `M	apps/desktop/src-tauri/src/commands/settings.rs`
- `M	apps/desktop/src-tauri/src/lib.rs`
- `M	apps/desktop/src/features/health/HealthPage.tsx`
- `M	apps/desktop/src/features/pending/__tests__/p18_pending_state.test.tsx`
- `M	apps/desktop/src/features/settings/SettingsPage.tsx`
- `M	apps/desktop/src/features/settings/__tests__/p16_settings.test.tsx`
- `D	apps/desktop/src/features/settings/__tests__/p23_shape_on_arrival.test.tsx`
- `M	apps/desktop/src/features/settings/__tests__/p24_capture_one_provisioning.test.tsx`
- `M	apps/desktop/src/features/settings/__tests__/p26_frontend_leftovers.test.tsx`
- `A	apps/desktop/src/features/settings/__tests__/p27_default_shape.test.tsx`
- `M	apps/desktop/src/features/shell/__tests__/p19_navigation.test.tsx`
- `M	apps/desktop/src/features/shell/__tests__/p22_shell_tidy.test.tsx`
- `M	apps/desktop/src/services/bindings.ts`
- `M	apps/desktop/src/services/settings.ts`
- `M	crates/core/src/domain/editing.rs`
- `M	crates/core/src/error.rs`
- `M	crates/core/src/ports/editing.rs`
- `M	crates/core/src/ports/store.rs`
- `M	crates/core/src/services/arrival.rs`
- `M	crates/core/src/services/editing_startup.rs`
- `M	crates/core/src/services/settings.rs`
- `M	crates/infra/src/jobs.rs`
- `M	crates/infra/src/settings_file.rs`
- `A	crates/infra/src/sqlite/migrations/003_retire_shape_on_arrival.sql`
- `M	crates/infra/src/sqlite/store.rs`
- `M	crates/integrations/src/editing/capture_one/adapter.rs`
- `M	crates/integrations/src/editing/capture_one/gate.rs`
- `M	crates/integrations/src/editing/capture_one/jxa.rs`
- `D	crates/integrations/src/editing/capture_one/jxa/apply_crop.js`
- `D	crates/integrations/src/editing/capture_one/jxa/crop_aspect_ratios.js`
- `M	crates/integrations/src/editing/capture_one/jxa/ensure_recipe.js`
- `M	crates/integrations/src/editing/capture_one/mod.rs`
- `M	crates/integrations/src/editing/capture_one/provision.rs`
- `M	crates/integrations/src/editing/capture_one/recipes.rs`
- `M	crates/integrations/src/editing/capture_one/scripting.rs`
- `D	crates/integrations/src/editing/capture_one/shape.rs`
- `M	crates/integrations/src/lib.rs`
- `M	crates/testkit/src/fake_capture_one.rs`
- `M	crates/testkit/tests/p04_capture_one_adapter.rs`
- `M	crates/testkit/tests/p05_capture_arrival.rs`
- `M	crates/testkit/tests/p17_real_run_fixes.rs`
- `D	crates/testkit/tests/p23_shape_on_arrival.rs`
- `M	crates/testkit/tests/p24_capture_one_provisioning.rs`
- `M	crates/testkit/tests/p25_tidy_up_after_provisioning.rs`
- `A	crates/testkit/tests/p27_retire_shape_on_arrival.rs`
- `M	docs/implementation/master_plan.md`
- `M	docs/implementation/plans/P27_retire_shape_on_arrival.md`

The handoff itself is a new file for the runner's handoff commit, outside the checkpoint's 48-path perimeter. No mutation probes or external state writes were made.

## Checkpoint SHA

`463cb2e4c42d430ff92fed1836bcb89173fab46e` — `CHECKPOINT (not approved): retire shape on arrival`. `git status --porcelain` was empty immediately after the checkpoint, before this handoff was written.

## Stamp

- `cargo test --workspace` → exit 0, 456 Rust tests passed, no failures.
- `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
- `cargo fmt --all -- --check` → exit 0.
- `cd apps/desktop && npm run typecheck` → exit 0.
- `cd apps/desktop && npm test` → exit 0, 131 UI tests passed in 20 files.
- `cargo test -p psm-desktop export_bindings` → exit 0, bindings regenerated; `cropRatiosList` absent.
- `git diff --check` → exit 0 before checkpoint.

The final stamp ran on the content committed at the checkpoint; the handoff was written afterwards so it can name the actual SHA.

## Manual try-out

1. Launch with the existing settings file. In Settings, confirm Default shape is Square (1:1), Master recipe is PSM Master, and Shape on arrival is absent. Save; inspect `settings.json` for `default_shape` and the absence of `crop_on_arrival`.
2. In Capture One, inspect process recipes. PSM Preview is JPEG/sRGB, long edge 2560, quality 85, crop ignored. PSM Master is disabled 16-bit uncompressed TIFF/sRGB, full size, crop ignored. PSM Web remains available. Health says Recipes: Ready.
3. Shoot a frame. Its adjusted preview should show the full frame; Capture One's crop tool should be untouched. Set a crop by hand and verify the app's next preview still shows the full frame.
4. Export without a commerce integration. The interim export should still use PSM Web.
