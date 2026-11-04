---
plan: P29_master_and_output_render
role: implement
round: 1
state: DONE
date: 2026-11-04
actor: claude-opus (pipeline-implementer, Claude fallback; the prompt template names codex)
---
# P29 master and output render — implement r1

Every exported file is now the app's own render. Export asks Capture One for one full-size 16-bit master per photo, only when
it has no master for that photo's current edit. The app crops, turns, flips, straightens and resizes it (in linear light), then
writes one sRGB JPEG with camera, date and copyright and no GPS. Outputs are cached by edit generation, composition revision
and output setting, so a repeat export renders nothing. Export refuses up front, naming the photos, while a main or condition
photo needs re-framing. Settings show "Output size" and "Output JPEG quality" instead of "Web recipe". All 27 rows are tested.
Suite, lint and typecheck are green.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## What was built
- **Master on demand (MC-28 §Master).** `RenderPurpose::Master` (recipe `master_recipe`, 120 s timeout, re-issue once then fail)
  replaces `Export`. Export requests the master only when no output for the current key is cached and
  `masters/<pk[0..2]>/<pk>/master-<g>.tif` does not exist, once per RAW. The master is committed with temp + rename, the previous
  generation is deleted, and Capture One's copy in its output folder is removed after the copy. Masters are released when a
  local export completes and the session is not dirty, and kept when a run fails.
- **The `render_output` job (render contract, `crates/infra/src/imaging/render.rs`; the contract text is its doc comment).** The
  steps: decode 8- or 16-bit RGB; 65 536-entry sRGB→linear LUT; quarter turns and flip as an index remap; pixel rect plus split half;
  Catmull-Rom bicubic straighten about the centre (θ = 0 is a plain copy); Lanczos3 (`fast_image_resize`, U16x3) only when the rect
  is larger than the long edge; linear→sRGB 8-bit LUT; one JPEG encode (`jpeg-encoder`, F_1_1 = 4:4:4, quality from the setting,
  built-in sRGB ICC v4 profile in APP2, EXIF APP1 from the allowlist built with `kamadak-exif`'s writer). It runs on its own runner
  and thread (`App.render_jobs`). A render error fails the run with no retry. A row left from an earlier process finds no run and
  closes.
- **Output cache (MC-28 §Cache key).** `outputs/<pk[0..2]>/<pk>/<uid12>-<slot>-g<g>-r<rev>-<format_key>.jpg`. A cached file is
  copied, never re-rendered. Other keys of the same photo are pruned when a new output commits.
- **Export through outputs.** Targets are (photo, slot) in photo order (`mechanisms::naming::output_names`). The staging folder gains
  `manifest.json` (`format_key` plus a per-file `{photo_id, slot, adjusted_gen, composition_rev}`), written before `item.json`.
  Deleted or moved photos are dropped and the rest renumbered, as before.
- **Frame adoption with masters.** `Store::adopt_frame` (P28's rule, now also called for masters) materialises a frameless default
  from the master. A compatible master becomes the frame. An incompatible one (moved geometry, or a different size from the stored
  master) sets `needs_review`, and every run holding that photo fails with `needs_review`.
- **Refusal.** `Error::NeedsReview{photos}` (`Re-frame first: Main 02, Condition 01`; command error code `needs_review`,
  `data.photos`). It is raised before any render and before any run row.
- **Settings.** `output_format {long_edge 512–8192, quality 60–100}` (default 2048/90). `web_recipe` is ignored on read, dropped on
  save, and a patch naming it is refused. The Settings screen has two number fields in place of "Web recipe". `PSM Web` is no
  longer created, repaired or used.

## Tests → criterion rows
All in `crates/testkit/tests/p29_master_and_output_render.rs` unless noted. Each asserts the row's exact outcome at the row's
boundary.

| Row | Test | Shape vs row |
|---|---|---|
| C1(a) | `c1a_first_export_renders_one_master_per_raw_and_never_writes_the_raw` | issued since the run = exactly `(va, PSM Master), (vb, PSM Master)`, no `PSM Web` anywhere; both `master-<g>.tif` exist at master_ready (checked before the render tick); both files exported; RAW SHA-256 unchanged |
| C1(b) | `c1b_export_with_every_output_cached_renders_nothing` | `issued.len()` unchanged, last progress `Done`, `K_01_main.jpg` byte-equal |
| C1(c) | `c1c_new_generation_replaces_the_kept_master` | after the failed run `master-<g>` exists; after the second run's master `master-<g+1>` exists and `master-<g>` is gone (checked before finalize); the outputs folder holds exactly `<uid12>-single-g<g+1>-r0-jpeg-2048-q90.jpg` |
| C2(a) | `c2a_plain_crop_cuts_the_pixel_rect` | 300×300; columns 148/149 dark and 147/150 light at rows 10/150/290; rows 198/199 dark and 197/200 light at columns 10/150/290 |
| C2(b) | `c2b_quarter_turn_then_flip_in_the_turned_frame` | 400×600; rows 100/101 dark, 99/102 light; columns 50/51 dark, 49/52 light |
| C2(c) | `c2c_straighten_rotates_clockwise_about_the_centre` | 200×200; darkest column of rows 0/100/199 in [116,119]/[98,101]/[81,84]; the two corners have every channel > 120 |
| C2(d) | `c2d_default_materialised_from_the_master` | 400×400; composition frame 600×400, rect (1/6, 0, 2/3, 1) ±1e-6, `ok`, revision 0; column 0 light; no dark pixel in any column at rows 10/200/390 |
| C2(e) | `c2e_original_keeps_the_whole_frame` | 600×400 |
| C3(a) | `c3a_split_cuts_two_adjacent_halves` | `K_01`/`K_02` both 500×500; left column 499 dark, 498 grey; right column 0 light, 1 grey; `item.json` main = both names (rect h adjusted, see Deviations) |
| C4(a) | `c4a_larger_rect_is_resized_to_the_long_edge` | 2048×1365 |
| C4(b) | `c4b_smaller_rect_is_never_upscaled` | 600×400 |
| C4(c) | `c4c_one_encode_with_srgb_profile_and_full_chroma` | APP2 `ICC_PROFILE\0` present, profile `desc` (mluc) contains `sRGB`; SOF0 has 3 components, each `0x11`; exactly one `FF D8` and one `FF D9`, at the start and the end |
| C4(d) | `c4d_downscale_averages_in_linear_light` | 512×512; 16 grid points, every channel in [178,196] |
| C4(e) | `c4e_output_format_setting_is_validated_and_stored` | default `{2048,90}`, no `web_recipe`; 1024/90 shown and in the file; 100 and 101 → `InvalidValue("output_format")` with the value kept; `web_recipe` → `InvalidValue("web_recipe")` |
| C4(f) | `apps/desktop/src/features/settings/__tests__/p29_output_format.test.tsx` | spinbuttons with values 2048/90; no "Web recipe" textbox; after editing to 1600/85 the patch's `output_format` equals `{1600, 85}` |
| C5(a) | `c5a_metadata_allowlist_copied_gps_dropped_orientation_upright` | fixture read back first (Make, GPSLatitude, Orientation 6); output has Make/Model/DateTimeOriginal/Copyright equal and Orientation 1; no GPS-context field; every field is in the allowlist (+ `ExifIFDPointer`); no XMP APP1 |
| C5(b) | `c5b_master_without_metadata_copies_nothing` | decodes; Orientation 1; none of the four tags |
| C6(a) | `c6a_composition_change_replaces_the_cached_output` | outputs folder = exactly `…-g<g>-r1-jpeg-2048-q90.jpg`; `K_01` byte-equal to it |
| C6(b) | `c6b_output_format_change_replaces_the_cached_output` | outputs folder = exactly `…-r1-jpeg-1024-q90.jpg`; exported long edge 1024 |
| C6(c) | `c6c_manifest_records_every_files_cache_key` | `format_key "jpeg-1024-q90"`; `outputs["K_01_main.jpg"]` equals the exact object; `item.json` main lists both |
| C7(a) | `c7a_masters_released_after_a_completed_local_export` | both masters folders absent; one output each |
| C7(b) | `c7b_failed_export_keeps_its_master` | `master-<g>.tif` of `a` exists; no `K/` |
| C7(c) | `c7c_cached_outputs_need_no_master` | no `process` call, `render_output` row count unchanged, `Done`, files byte-equal |
| C8(a) | `c8a_export_refuses_while_a_main_photo_needs_review` + desktop `p29_tests::c8a_export_run_rejects_with_needs_review` | `Err(NeedsReview{["Main 02"]})`, no new `process`, no `.K.partial-*`, last progress `Failed`; command error `code needs_review`, `data.photos "Main 02"`, message `Re-frame first: Main 02` |
| C8(b) | `c8b_condition_label_and_unreviewed_photos_do_not_block` | `["Condition 01"]`; with only an unreviewed photo flagged the export reaches `Done` and writes `K_01_main.jpg` |
| C8(c) | `c8c_master_with_moved_geometry_flags_review_and_fails_the_run` | starting fingerprint `{2560×1707, preview}` asserted; after the master: `needs_review`/`geometry`, fingerprint `{600×400, master, rotation 3}`, `Failed`, master kept, 0 `render_output` rows; a later run → `["Main 01"]` |
| C8(d) | `c8d_master_size_change_against_the_stored_master_flags_review` | after the first export the fingerprint is `{600×400, master, default}`; after the new preview `ok`, fingerprint unchanged; after a 640×400 master `needs_review`/`frame`, fingerprint `{640×400, master, default}`, `Failed` |

No orphan tests: every test in the two new files maps to a row above. The amended earlier tests are listed under Deviations.
Core mechanisms: none new, so there are no new unit tests.

## Assumptions
1. Interpolation is a hand-written Catmull-Rom bicubic (no `imageproc`). Linear light is held as `u16` between steps.
2. The EXIF writer is `kamadak-exif`'s `experimental::Writer` (no `little_exif`). The master's metadata is read before the pixels are decoded.
3. The ICC profile is built in code (ICC v4.3, D50, parametric sRGB curve, desc `sRGB IEC61966-2.1`). No profile file is shipped or
   read at runtime. macOS `sips` reports it as `sRGB IEC61966-2.1`.
4. `MasterStore` and `OutputStore` are two traits. The render runner has one dedicated thread.
5. Runs that share a RAW share one master `process`. All of them proceed when it arrives, or fail on its final timeout.
6. **Prune keeps the other slot of the same key.** Read literally, "delete the other `<uid12>-*.jpg`" would make a split's right
   half delete its left half's cache entry.
7. **Capture One's master TIFF is removed from `Output/_session_manager/` after it is copied into `masters/`** (manual try-out
   step 1: "fill, then empty"; 361 MB per master). The RAW and the committed master are never written.
8. A master completion that no run waits for is ignored and not committed.
9. When an export finalizes, masters are not released for a RAW that another in-flight run still needs.
10. `RenderOutputJob` names the output by the composition revision it actually rendered, so `output_ready` carries `revision`.
    `enqueue_render_output` takes the RAW as the job target. `output_names` takes `key` and `group`.
11. A cached output has no master tracking row, so the run row records `retouch_gens` at run start and `export_retouch_gen`
    reads it from there. Publish behaviour is unchanged.
12. `adopt_frame`: any compatible master observation replaces the fingerprint (plan task 0).
13. `needs_review` labels are padded like export names. `data.photos` is joined with `", "`, as in the message.
14. `TestApp::tick` runs general jobs, then render jobs, then publish jobs.
15. Fixture shapes: C3(a) uses a 1001×601 preview and C4(d) a 2048×2048 preview, so the rect validates and the master is
    compatible. The C6 chain uses 3000×2000 masters, because outputs are never upscaled and a 600×400 master cannot give a 1024 px
    output. C8 flags photos with P28 C5(a)'s geometry change, because the row's 2560×1700 preview is compatible by P28 C5(b).
    C5(a) also allows `ExifIFDPointer`.

## Deviations
- **C3(a) input:** the plan's rect `h = 0.831946755` is refused by P28's split validation (`|1001 − 2·h·601| = 1.0000005 > 1`).
  The test uses `h = 0.83194676`, which gives the same pixel rect `{0, 50, 1000, 500}`. Assertions are unchanged.
- **Tests changed beyond the plan's list**, forced by removing `RecipeSpec::WEB` and `web_recipe` and by the new `job_due`
  signature: `p04` c3a/c3b, `p17` c1a plus the `job_due` call, `p24` c1a/c1b/c4a (a local "owner JPEG" spec stands in for the
  web spec), `p25` c5c (`tick(0)`), `p13` `start_upload` (render runner tick), desktop `p21_tests` (`master_recipe`), and seven
  frontend settings fixtures. P14 needed no change.
- `crates/infra/src/imaging/sRGB.icc` is not created (Assumption 3).
- Tracker actor is `claude-opus` rather than the template's `codex` (this run used the Claude fallback).

## Files changed
Checkpoint `1d43c22` (55 files):
- **Core:** `crates/core/src/{domain/editing.rs, domain/mod.rs, domain/output.rs (new), error.rs, mechanisms/naming.rs,
  ports/images.rs, ports/store.rs, services/export.rs, services/previews.rs, services/settings.rs}`.
- **Infra:** `crates/infra/{Cargo.toml, src/lib.rs, src/imaging.rs → src/imaging/mod.rs, src/imaging/{render.rs, linear.rs,
  metadata.rs} (new), src/media_store.rs (new), src/jobs.rs, src/settings_file.rs, src/sqlite/store.rs}`.
- **Integrations:** `crates/integrations/src/{lib.rs, render_output.rs (new), editing/capture_one/{adapter.rs, recipes.rs,
  provision.rs}}`.
- **Testkit:** `crates/testkit/{Cargo.toml, src/test_app.rs, src/fake_capture_one.rs, src/fixtures/mod.rs,
  src/fixtures/synthetic_master.rs (new)}`.
- **Tests:** `crates/testkit/tests/{p29_master_and_output_render.rs (new), p00_spike.rs, p04_capture_one_adapter.rs,
  p09_local_export.rs, p13_publish_runs.rs, p17_real_run_fixes.rs, p24_capture_one_provisioning.rs,
  p25_tidy_up_after_provisioning.rs, p27_retire_shape_on_arrival.rs}`.
- **Desktop:** `apps/desktop/src-tauri/src/{lib.rs, commands/items.rs, commands/settings.rs}`, `apps/desktop/src/services/bindings.ts`
  (regenerated), `apps/desktop/src/features/settings/SettingsPage.tsx`, `apps/desktop/src/features/settings/__tests__/
  p29_output_format.test.tsx` (new), and settings fixtures in `features/{pending/__tests__/p18_pending_state, shell/__tests__/p19_navigation,
  shell/__tests__/p22_shell_tidy, settings/__tests__/p16_settings, p24_capture_one_provisioning, p26_frontend_leftovers,
  p27_default_shape}.test.tsx`.
- **Other:** `Cargo.lock`, `docs/implementation/master_plan.md` (P29 tracker row only),
  `docs/implementation/plans/P29_master_and_output_render.md` (Review log).

Outside the repo: a scratch probe crate at `/tmp/p29probe` (read-only probe; its `rm -rf` cleanup was refused by the permission
rule and is not needed). There are no mutation-probe files. The architecture graph is not in use in this project.

## Checkpoint SHA
`1d43c22` — `CHECKPOINT (not approved): implement P29 master and output render` (tree clean after commit).

## Stamp
| Command | Result |
|---|---|
| `cargo test --workspace --no-fail-fast` | exit 0; 508 passed, 0 failed (481 before this phase + 26 P29 + 1 desktop P29) |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean |
| `cargo fmt --all -- --check` | clean |
| `cd apps/desktop && npm run typecheck` | clean |
| `cd apps/desktop && npm test` | 21 files, 132 passed |

No pre-existing failures. **Live probe** (outside the suite): the renderer ran on a real Capture One 16.8.6 TIFF (ZV-E10,
6000×4000, 8-bit). Make, Model and DateTimeOriginal were copied; the 2048×2048 output with a 2° straighten took 0.38 s in a
release build; the embedded profile is read by `sips` as `sRGB IEC61966-2.1`. No `PSM Master` render was triggered live.

## Manual try-out
1. Open a session with a few main photos whose previews are done, and press **Export** (no Shopify configured). Capture One
   renders one 16-bit TIFF per photo into `Output/_session_manager/`, and each one disappears once the app has taken it.
   `~/Pictures/Session Manager Export/<SKU>/` then holds square 2048 px JPEGs (the default composition), `item.json` and
   `manifest.json`. Open one in Preview → Inspector: sRGB IEC61966-2.1 profile, 2048 px, your camera model and capture date, no GPS.
2. Export again right away: it finishes almost at once and Capture One renders nothing (the outputs are cached).
3. Settings → **Output size (long edge, px)** 1600 → Save → Export. The files are now 1600 px, and Capture One renders the TIFFs
   once more because masters are released after a completed export. Export again and nothing is rendered.
4. Settings shows "Output size" and "Output JPEG quality" and no "Web recipe". In Capture One, `PSM Web` is still there; the app
   leaves it alone.
5. Rotate a photo 90° in Capture One and wait for its preview. Export now refuses with "Re-frame first: Main 0n" and nothing is
   rendered.
