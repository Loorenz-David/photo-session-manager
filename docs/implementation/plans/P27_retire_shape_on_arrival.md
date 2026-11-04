---
phase: P27_retire_shape_on_arrival
title: Retire shape on arrival — no more Capture One crop writes, `default_shape` replaces `crop_on_arrival`, PSM Preview at 2560 px crop-ignored, PSM Master provisioned
difficulty: high
projection: no
depends_on: [P26_frontend_leftovers]
---
# P27 — Retire shape on arrival: no more Capture One crop writes, `default_shape` replaces `crop_on_arrival`, PSM Preview at 2560 px crop-ignored, PSM Master provisioned

## Goal
First phase of book chapter `10D_composition_and_framing.md` (owner decision 2026-11-04, R37, changelog 27; 10B retired, M31 retired).
The app stops writing Capture One's crop: the `crop_on_arrival` job (10B MC-25, P23) is removed with its scripts, adapter methods,
health items, command and Settings control; a new photo gets **no** Capture One call beyond the renders it already had. The setting
`default_shape` (10D MC-27 §Default: `original | 1:1 | 4:5 | 3:2 | 2:3 | 16:9 | free`, missing → `1:1`) replaces `crop_on_arrival` in
`settings.json` and on the Settings screen ("Default shape"); a file that still carries `crop_on_arrival` is read once through a fixed
mapping and rewritten on the next save. The recipes change to what 10D MC-28 and 10C step 4 now say: `PSM Preview` becomes JPEG, sRGB,
long edge **2560**, quality **85**, `export crop method = ignore`; a new `PSM Master` (16-bit TIFF, sRGB, full size, lossless, crop
ignored, disabled) is created and repaired by the same ensure (setting `master_recipe`, default `PSM Master`). **Interim rule of this
phase:** `PSM Web` is still ensured and still renders exports, so the local workflow keeps working until P29 moves export onto the
master render; P29 removes it. **VERIFY-15 was run live by the orchestrator on 2026-11-04 and is recorded** (evidence "VERIFY-15 result",
commit `069eca4`): crop-ignored export works, every property reads back, the master is **uncompressed** 16-bit TIFF; the implementer starts
from the recorded results and never re-runs the probe. **Not in this
phase:** the `compositions` table, the composition commands, thumbnails and the fingerprint (P28); the master render, `render_output`,
`output_format` and the removal of `PSM Web` (P29); outputs in export and publishing (P30); the editor (P31–P34); any change to
`photo_links`.

## Read first
Book: `10D_composition_and_framing.md` §What the photographer sees (first bullet), §MC-27 Default and Validation (the shape names),
§MC-28 Recipes, §Grounding (VERIFY-15) · `10B_shape_on_arrival.md` header (retired; what is being removed) · `10C_capture_one_setup_by_the_app.md`
step 4 (recipes: missing → created, drifted → set back, other names never touched, `PSM Web` left alone) · `10A` MC-16 §Recipes (two
names: preview, master; `existing files = overwrite`, `output sub folder = _session_manager`) · `01` (H2, H3 as narrowed, H5, H7) ·
`14` (M32; M23, M24 for the rows this phase must not break; M31 struck) · `12` §Settings · `15` R37 · `16` entry 27 · evidence
`capture_one_16.8.6.md` §"Recipe property read-back (P24 task 0)" (a fresh recipe reads TIFF, Adobe RGB, `Fixed`, `percent`, 100,
q100, disabled; accepted spellings `outputFormat 'JPEG'`, `colorProfile 'sRGB Color Space Profile'`, `scalingMethod 'Long_Edge'`,
`scalingUnit 'pixels'`; `make` on an existing name throws and leaves `Untitled Recipe`) and **§"VERIFY-15 result" (2026-11-04: the spellings set and read back on
a fresh recipe — `outputFormat 'TIFF'`, `bits 16`, `tiffCompression 'Uncompressed'`/`'ZIP'`, `colorProfile 'sRGB Color Space Profile'`,
`exportCropMethod 'ignore'`; JPEG `scalingMethod 'Long_Edge'`, `scalingUnit 'pixels'`, `primaryScalingValue 2560`, `jpegQuality 85`; a fresh
recipe's scaling `Fixed` 100 `percent` = full size; the 61 MP master: uncompressed 2.1–2.4 s / 361 MB, ZIP 8.0–8.5 s)** · dictionary `docs/reference/capture_one_16.8.6.sdef` `:798–810` `recipeFileFormat`
(`JPEG`, `TIFF`), `:812–816` `tiffCompress` (`Uncompressed`, `LZW`, `ZIP`), `:818–826` `scalingType` (`Fixed`, `Long_Edge`), `:828–834`
`measurementUnit` (`pixels`, `percent`), `:842–846` `export crop method` (`respect`, `ignore`, `crop to path`), recipe class `:867–945`
(`bits` integer rw `:874`, `TIFF compression` `:878`, `export crop method` `:934`, `ignore crop` hidden `:933`) · master plan §3, §5
(preconditions: VERIFY-15), §6.4 (`CaptureOneScripting`, `RecipeProps`, `RecipeSpec`), §6.6 (codes being removed), §6.7, §6.8
(`default_shape`, `master_recipe`), §6.9, §6.10 (the composition skeleton this phase starts), §7.2, §7.5 · process `flow.md` §4, §6,
§6A · plans `P23_shape_on_arrival.md` (what this phase removes: tasks 1–9 list every file it touched), `P24_capture_one_provisioning.md`
task 4 and C4 rows (the recipe ensure this phase extends; `JXA_HARNESS :450–483` in `p24_capture_one_provisioning.rs` is the Node
harness shape for C3(e)) · code (line numbers at `d763492`, 2026-11-04): `crates/core/src/ports/editing.rs` (`EditingProvider`:
`crop_aspect_ratios :45`, `shape_on_arrival :46` — removed), `crates/core/src/domain/editing.rs` (`ShapeOutcome :51–56` — removed;
`RenderPurpose :45–49` unchanged here), `crates/core/src/error.rs` (`CropRatioUnknown :47` — removed), `crates/core/src/services/
editing_startup.rs` (`crop_on_arrival :12`, the ratio check `:43–56` — removed), `crates/core/src/services/arrival.rs`
(`shape_on_arrival :28`, `with_shape_on_arrival :51–54`, `arrival_insert(..., self.shape_on_arrival) :270–276`), `crates/core/src/
ports/store.rs` (`arrival_insert :111–116` third argument; `job_fail`, `count_jobs`, `requeue_failed_jobs :124–131` stay),
`crates/core/src/services/settings.rs` (allowed keys `:38–49`; the `crop_on_arrival` rule `:75–77`), `crates/infra/src/settings_file.rs`
(`Settings :13–25`: `web_recipe :16`, `crop_on_arrival :18`; `Default :42–58`; `open_lenient :72–90`), `crates/infra/src/jobs.rs`
(`:55–75`: the `CropRatioUnknown` arm `:59` and the terminal branch `:62–66` — keep `max_attempts`, drop the crop arm), `crates/infra/src/
sqlite/store.rs` (`arrival_insert :1070–1150`: `raw_seen :1096`, `shape_job_exists :1103`, the job loop `:1125–1136`;
`SqliteStore::open :27–38` migrations list), `crates/integrations/src/editing/capture_one/{scripting.rs (`RecipeProps :29–41`,
`crop_aspect_ratios`/`apply_crop_if_untouched :75–77`), gate.rs (`:127–137`), jxa.rs (`SCRIPTS :16` — 21 entries; `:43–46` the two crop
scripts; `:202–220` the two methods), jxa/crop_aspect_ratios.js, jxa/apply_crop.js, jxa/ensure_recipe.js (`keys :18–19`), recipes.rs
(`RecipeSpec::PREVIEW 1600/80 :11–14`, `WEB 2048/90 :15–18`, `props :19–32`), adapter.rs (`web_recipe :36`, `crop_aspect_ratios :474`,
`shape_on_arrival :477–487`), provision.rs (`:222–256` the recipe loop over `(&self.preview_recipe, PREVIEW), (&self.web_recipe, WEB)`),
shape.rs (whole file — removed), mod.rs (`pub mod shape`)}`, `crates/integrations/src/lib.rs` (`refresh_shape_health :62–76`;
`health_check :192, :212–220`; `build :413–416`, `:447` `with_shape_on_arrival`, `:500–509` the `crop_on_arrival` registration,
`:548–558` `startup_crop`; `startup :684–685`), `apps/desktop/src-tauri/src/commands/settings.rs` (`SettingsDto :21–33`,
`SettingsPatchDto :39–50`, `crop_ratios_list :79`), `apps/desktop/src-tauri/src/lib.rs` (`collect_commands! :111–155`, last entry
`crop_ratios_list`), `apps/desktop/src/services/settings.ts` (`listCropRatios :26–28`), `apps/desktop/src/features/settings/
SettingsPage.tsx` (`TextKey :14`; the save patch `:44–54`; `text('Web recipe') :99`; the Shape on arrival select `:101–108`),
`apps/desktop/src/features/health/HealthPage.tsx` (`names :9–17`) · fakes: `crates/testkit/src/fake_capture_one.rs` (`FakeVariant
crop fields :10–16`; `recipes` built from `RecipeSpec :68–78`; `crop_ratios/fail_apply_crop/timeout_next_apply_crop :51–53`;
`set_variant_crop :202–218`; `ensure_recipe :305–357` with the `repair!` macro; `crop_aspect_ratios :531`, `apply_crop_if_untouched
:537–583`) · tests to change or delete (named again by the implementer): `crates/testkit/tests/p23_shape_on_arrival.rs` (**delete**),
`apps/desktop/src/features/settings/__tests__/p23_shape_on_arrival.test.tsx` (**delete**), `crates/testkit/tests/p05_capture_arrival.rs`
(`c5c_four_followup_jobs :346–369` — the expected list loses `crop_on_arrival`: three jobs), `p17_real_run_fixes.rs` (`failed_render_fixture
:280–283` — drop the `crop_on_arrival: off` save), `p25_tidy_up_after_provisioning.rs` (`c7a_unknown_shape_does_not_requeue_until_listed
:364` — **delete**), `p00_spike.rs:77` (`web_recipe` assertion stays this phase), the six frontend settings fixtures that carry
`crop_on_arrival: 'Square'` (`p16_settings.test.tsx:16`, `p24_capture_one_provisioning.test.tsx:16`, `p26_frontend_leftovers.test.tsx:14`,
`shell/__tests__/p22_shell_tidy.test.tsx:17`, `shell/__tests__/p19_navigation.test.tsx:155`, `pending/__tests__/p18_pending_state.test.tsx:128`)
and the `@/services/settings` mock factories that list `listCropRatios` (`p16_settings.test.tsx:9`, `p19_navigation.test.tsx:16`,
`p18_pending_state.test.tsx:22`, `p22_shell_tidy.test.tsx`, `p24_capture_one_provisioning.test.tsx`, `p26_frontend_leftovers.test.tsx`).

## Files expected to change
Backend: `crates/core/src/ports/editing.rs`, `crates/core/src/domain/editing.rs`, `crates/core/src/error.rs`, `crates/core/src/services/
editing_startup.rs`, `crates/core/src/services/arrival.rs`, `crates/core/src/ports/store.rs`, `crates/core/src/services/settings.rs`,
`crates/infra/src/settings_file.rs`, `crates/infra/src/jobs.rs`, `crates/infra/src/sqlite/store.rs`, `crates/infra/src/sqlite/migrations/
003_retire_shape_on_arrival.sql` (new), `crates/integrations/src/editing/capture_one/{scripting.rs, gate.rs, jxa.rs, jxa/ensure_recipe.js,
recipes.rs, adapter.rs, provision.rs, mod.rs}`, deleted: `crates/integrations/src/editing/capture_one/{shape.rs, jxa/crop_aspect_ratios.js,
jxa/apply_crop.js}`, `crates/integrations/src/lib.rs`, `crates/testkit/src/fake_capture_one.rs`, `apps/desktop/src-tauri/src/commands/
settings.rs`, `apps/desktop/src-tauri/src/lib.rs`, `apps/desktop/src/services/bindings.ts` (regenerated).
Frontend: `apps/desktop/src/services/settings.ts`, `apps/desktop/src/features/settings/SettingsPage.tsx`, `apps/desktop/src/features/health/
HealthPage.tsx`.
Tests: `crates/testkit/tests/p27_retire_shape_on_arrival.rs` (new: C1–C4(a)), `apps/desktop/src/features/settings/__tests__/
p27_default_shape.test.tsx` (new: C4(b), C5), the deletions and fixture changes listed under Read first.

## Core mechanisms (unit tests allowed)
None.

## VERIFY-15 already run by the orchestrator (2026-11-04) — read before task 1
Results are in `docs/intention/evidence/capture_one_16.8.6.md` §"VERIFY-15 result" (commit `069eca4`). **Do not re-run it.** Binding
consequences for this phase: (1) `export crop method = ignore` exports the full developed frame for 16-bit TIFF and JPEG — the base of 10D
holds; (2) every property the app sets reads back with the spellings quoted under Read first — `RecipeProps`, `RecipeSpec` and the fake copy them
verbatim; (3) the master is **uncompressed** (`MASTER_TIFF_COMPRESSION = "Uncompressed"`, a named constant in `recipes.rs`, from 10D MC-28); (4)
"full size" is `scalingMethod 'Fixed'`, `scalingUnit 'percent'`, `primaryScalingValue 100`; (5) the output lands in
`<session>/Output/<outputSubFolder>/<image name>.<ext>` (MC-16 unchanged). What VERIFY-15 found about Capture One's own straighten and
keystone (they keep 3:2 and shrink the export) matters to P28's fingerprint, not to this phase.

## Ordered tasks
0. **Gate.** The evidence section "VERIFY-15 result" exists and states that the crop-ignored export produced the full frame (it does);
   otherwise stop `BLOCKED`. Nothing live is run.
1. **Registry first (master plan §6.4, §6.6, §6.7, §6.8, §6.10).** Remove `ShapeOutcome`, `Error::CropRatioUnknown`, `EditingProvider::
   {crop_aspect_ratios, shape_on_arrival}`, `CaptureOneScripting::{crop_aspect_ratios, apply_crop_if_untouched}`, job kind `crop_on_arrival`,
   health codes `crop_not_applied`/`crop_ratio_unknown`, command `crop_ratios_list`, setting `crop_on_arrival`, constant
   `CROP_GIVE_UP_ATTEMPTS`, module `shape.rs`, scripts `crop_aspect_ratios.js`/`apply_crop.js` (`SCRIPTS` 21 → 19). Add settings keys
   `default_shape: String` (default `"1:1"`) and `master_recipe: String` (default `"PSM Master"`); `RecipeProps` gains
   `export_crop_method: String`, `bits: i64`, `tiff_compression: String` (JXA keys `exportCropMethod`, `bits`, `tiffCompression`);
   `RecipeSpec::PREVIEW` = JPEG, bits 8, `Long_Edge`/`pixels`/2560, q85, `tiff_compression "Uncompressed"`, `export_crop_method "ignore"`;
   `RecipeSpec::MASTER` = TIFF, bits 16, `Fixed`/`percent`/100 (VERIFY-15: a fresh recipe's `Fixed` 100 `percent` is full size), q100 (unused by
   TIFF; set anyway so the read-back compares), `tiff_compression MASTER_TIFF_COMPRESSION` (`"Uncompressed"`), `export_crop_method "ignore"`;
   `RecipeSpec::WEB` unchanged this phase (JPEG 2048 q90, bits 8, `Uncompressed`, `export_crop_method "respect"` — **interim**, removed in
   P29). All three keep `enabled false`, `SRGB_PROFILE`, `existing_files "overwrite"`, `output_sub_folder "_session_manager"`.
2. **Migration `003_retire_shape_on_arrival.sql`:** `DELETE FROM jobs WHERE kind='crop_on_arrival';` (a leftover queued row would otherwise
   loop through P21's `unknown_job_kind` retry forever). Register it in `SqliteStore::open`.
3. **Arrival without the crop job.** `Store::arrival_insert(photo, path_key)` (the bool removed): the job loop inserts `original_preview`,
   `hash` (same `shared_hash`/`hash_job_exists` rule), `render_preview`; `raw_seen` and `shape_job_exists` go. `ArrivalService` loses
   `shape_on_arrival`/`with_shape_on_arrival`. `App::build` loses the registration, `refresh_shape_health`, both `requeue_failed_jobs`
   calls for the kind, `startup_crop`; `AppHandles::health_check` likewise; `EditingStartup` loses `crop_on_arrival` and the ratio check.
   `JobRunner::tick` keeps `max_attempts` (generic) and loses the `CropRatioUnknown` arm.
4. **Settings.** `Settings { default_shape, master_recipe }` replace `crop_on_arrival`; `web_recipe` stays (interim). **Legacy read** in
   `Settings::open_lenient` and `Settings::open`: when the parsed JSON has no `default_shape` key and has `crop_on_arrival`, `default_shape`
   = `Square → "1:1"`, `off → "original"`, `4x5 → "4:5"`, `2x3 → "2:3"`, `16x9 → "16:9"`, `3x2`? (not a Capture One name) → `"1:1"`,
   anything else (`Unconstrained`, `Original`, `Output`, `1x2`, `1x3`, `3x4`, `5x7`, unknown) → `"1:1"`; the mapping lives in one function
   `legacy_default_shape(&str) -> &'static str` in `settings_file.rs`. Serde ignores the unknown `crop_on_arrival` key on read; the next
   `Settings::set` writes the struct, so the key disappears. `SettingsService::set`: allowed keys drop `crop_on_arrival`, add `default_shape`
   (value must be one of the seven names, else `Err(InvalidValue("default_shape"))`) and `master_recipe` (non-empty string). `SettingsDto`/
   `SettingsPatchDto`: `default_shape`, `master_recipe` replace `crop_on_arrival` (`web_recipe` stays). Bindings regenerated (`cargo test -p
   psm-desktop export_bindings`).
5. **Recipes.** `ensure_recipe.js` `keys` gains `'exportCropMethod'`, `'bits'`, `'tiffCompression'` (set when different, read back into
   `props`). `provision.rs` recipe loop: `(&self.preview_recipe, PREVIEW)`, `(&self.master_recipe, MASTER)`, `(&self.web_recipe, WEB)` —
   in that order; `CaptureOneAdapter` gains `master_recipe: String` (constructor argument after `web_recipe`); `render(Master)` is **not**
   added here (P29); `RenderPurpose::Export` keeps `web_recipe`. The fake's `ensure_recipe` repair macro covers the three new fields; its
   default recipes are built from the new `RecipeSpec`s (`PSM Preview`, `PSM Web`; `PSM Master` absent until ensured, so C3(b) sees a
   `make_recipe:PSM Master` op). The fake's fresh-recipe defaults (`:316–326`) gain `export_crop_method "respect"`, `bits 16`,
   `tiff_compression "Uncompressed"` (evidence: a fresh recipe is TIFF, 16 bit, `Fixed` 100 `percent`).
6. **Settings screen.** Replace the "Shape on arrival" select with `<select id="settings-default_shape">` labelled `Default shape`, options
   in this order with these visible texts: `original` → `Original`, `1:1` → `Square (1:1)`, `4:5` → `4:5`, `3:2` → `3:2`, `2:3` → `2:3`,
   `16:9` → `16:9`, `free` → `Free`; after "Preview recipe" add `text('Master recipe', 'master_recipe')`; keep "Web recipe" (interim). The
   save patch carries `default_shape` and `master_recipe`; `listCropRatios` and the `cropRatios` state go. `HealthPage.names` drops the two
   crop entries.
7. **Fake clean-up.** `FakeVariant` loses `crop_ratio`/`crop`/`max_crop`; `FakeState` loses `crop_ratios`, `fail_apply_crop`,
   `timeout_next_apply_crop`; `set_variant_crop` goes.
8. Tests (§6.9: `crates/testkit/tests/p27_retire_shape_on_arrival.rs`, `apps/desktop/src/features/settings/__tests__/p27_default_shape.test.tsx`),
   the deletions and fixture updates, `cargo test --workspace`, clippy, fmt, bindings regeneration (diff expected: `SettingsDto`,
   `SettingsPatchDto`, `cropRatiosList` gone), `npm run typecheck`, `npm test`. The live row C6(a) runs only where Apple events are allowed
   (P16/P17/P23 precedent: else the reviewer), results in the Review log.

## Criteria (6 criteria, 14 rows; 3 `[rule]` rows)
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | `TestApp::with_fake_document()`; p05 `session(&a,"K1")` activated; `fake.add_image(<root>/K1/a.ARW, &["7"], false)`; p05 `file` + `hook`; `a.tick(0)`; `a.tick(20_000)` | a capture arrival, default settings | `SELECT kind FROM jobs WHERE target=<raw> ORDER BY kind` = `[hash, original_preview, render_preview]` (no `crop_on_arrival`); the fake's `ops` contain no entry starting with `apply_crop:` or `crop_aspect_ratios`; variant `7` has no crop write; the preview render was issued (`issued` holds one `process("7","PSM Preview")`) | 10D MC-27 §Default ("No Capture One call is made"); H5; M24 unchanged |
| C1(b) | same app; p05 `external("x.ARW")`; `service(&a).import(&[x])`; `tick(0)` | an import | same job list for `x.ARW`; no crop op | 10D MC-27 §Default |
| C2(a) | `TestApp::new()`: `settings_get()` | a settings file written before this phase with neither key (`TestApp::create` writes the P26-shape file: delete `default_shape` from it before the first open, keep no `crop_on_arrival`) | `default_shape == "1:1"`, `master_recipe == "PSM Master"`, no `crop_on_arrival` key in the returned object; stored `settings.json` after one `settings_set({"theme":"dark"})` contains `"default_shape": "1:1"` and `"master_recipe": "PSM Master"` | 10D MC-27 §Default (missing → `1:1`); 10A MC-16 §Recipes |
| C2(b) | six fresh `TestApp::new()`s whose `settings.json` is rewritten **before** `App` opens (write the file, then `TestApp::create`-equivalent open: use `App::open_with` over the written dir as p00 `c1b` does) with `"crop_on_arrival"` = `"Square"`, `"off"`, `"4x5"`, `"2x3"`, `"16x9"`, `"Unconstrained"` and **no** `default_shape` | legacy files | `settings_get().default_shape` = `"1:1"`, `"original"`, `"4:5"`, `"2:3"`, `"16:9"`, `"1:1"` respectively; after `settings_set({"theme":"dark"})` the stored file has `default_shape` and no `crop_on_arrival` | 10D MC-27 §Default (`crop_on_arrival` → `default_shape`; `original` keeps the whole frame); planner mapping (Notes) |
| C2(c) | `TestApp::new()`: `settings_set({"default_shape":"4:5"})`; `settings_set({"default_shape":"Square"})`; `settings_set({"default_shape":""})`; `settings_set({"crop_on_arrival":"off"})`; `settings_set({"master_recipe":"My Master"})`; `settings_set({"master_recipe":""})` | valid and invalid patches | first → stored `4:5`; `Square`, `""` → `Err(InvalidValue("default_shape"))`, stored stays `4:5`; `crop_on_arrival` → `Err(InvalidValue("crop_on_arrival"))` (not an allowed key); `My Master` stored and `settings_get().master_recipe == "My Master"`; `""` → `Err(InvalidValue("master_recipe"))` | 10D MC-27 §Values (the seven shape names); [rule] ch12 §Settings |
| C3(a) | `with_fake_document()`; `a.app.startup()` (the ensure) | the fake's recipes as the P26 fake creates them (`PSM Preview` 1600/80, `PSM Web`, no `PSM Master`) | `ops` contain `ensure_recipe:PSM Preview` and `set_recipe_props:PSM Preview:primaryScalingValue,jpegQuality,exportCropMethod,bits,…` (the drifted keys, order as the fake's macro emits them; `existingFiles` and `outputSubFolder` are also repaired from the fake's `add suffix`/`""` as before); after the ensure the fake's `recipes["PSM Preview"]` equals `RecipeSpec::PREVIEW.props()` field by field: `output_format "JPEG"`, `bits 8`, `scaling_method "Long_Edge"`, `scaling_unit "pixels"`, `primary_scaling_value 2560`, `jpeg_quality 85`, `export_crop_method "ignore"`, `color_profile SRGB_PROFILE`, `enabled false` | 10D MC-28 §Recipes; 10C step 4; M32 |
| C3(b) | same run | — | `ops` contain `ensure_recipe:PSM Master` and `make_recipe:PSM Master`; `recipes["PSM Master"]` equals `RecipeSpec::MASTER.props()`: `output_format "TIFF"`, `bits 16`, `tiff_compression == MASTER_TIFF_COMPRESSION` (asserted by the constant's name; its value is 10D MC-28's `Uncompressed`), `scaling_method "Fixed"`, `scaling_unit "percent"`, `primary_scaling_value 100`, `export_crop_method "ignore"`, `color_profile SRGB_PROFILE`, `enabled false`, `existing_files "overwrite"`, `output_sub_folder "_session_manager"` | 10D MC-28 §Recipes (16-bit TIFF, sRGB, full size, lossless, crop ignored, disabled); M32 |
| C3(c) | same run; then `settings_set({"master_recipe":"PSM Master 2"})` and `startup()` again | the interim rule; a renamed master recipe | `ops` of the first run contain `ensure_recipe:PSM Web` (export keeps its recipe this phase); the second run contains `ensure_recipe:PSM Master 2` and `make_recipe:PSM Master 2`, and `recipes` still holds `PSM Master` untouched (other names never touched) | 10C step 4 ("recipes with other names are never touched"); this plan's interim rule (Notes) |
| C3(d) | `with_fake_document()`; before `startup()` set `fake.state.recipes["PSM Preview"].export_crop_method = "respect"` and `.primary_scaling_value = 2560`, `.jpeg_quality = 85`, `.bits = 8` (everything else already right) | one drifted dependent property | `ops` contain exactly `set_recipe_props:PSM Preview:exportCropMethod` for that recipe (only the drifted key is written); read-back `"ignore"` | 10C step 4 ("a dependent property drifted → set back; other properties are left alone"); M32 |
| C3(e) | `node -e <JXA_HARNESS> <jxa>/ensure_recipe.js` (P24 harness: fake `app.make`, `doc.recipes.byName(n)` with `exists()`, recipe properties as getter/setter pairs recording `writes`) | argv `[doc, "PSM Master", JSON(RecipeSpec::MASTER.props() as the Rust side serialises it)]` over a recipe whose `exportCropMethod()` returns `'respect'`, `bits()` `16`, `tiffCompression()` `'LZW'` | `writes` contain `exportCropMethod='ignore'` and `tiffCompression=<MASTER_TIFF_COMPRESSION>` and **not** `bits` (already 16); the returned `props` carry `exportCropMethod`, `bits`, `tiffCompression`, `rootFolderType`; `changed` lists exactly the written keys | 10C step 4; VERIFY-15 (the spellings the script sends are the ones Capture One read back) |
| C4(a) | `with_fake_document()`; insert by SQL a `jobs` row `('crop_on_arrival', <raw>, '{}', 'queued', 0, …)` and a `failed` one with `error='provider'`; `a.kill_and_restart()`; `a.tick(0)`; `a.app.health_check()` | leftover rows of the retired kind in an existing database | after the restart `SELECT COUNT(*) FROM jobs WHERE kind='crop_on_arrival'` is `0`; the health report has no item with code `crop_not_applied` or `crop_ratio_unknown`; `tick` issues no `unknown_job_kind` retry (no `jobs` row has `error='unknown_job_kind'`) | 10B retired (changelog 27); P21 `unknown_job_kind` rule (master plan §6.6) |
| C4(b) | `<HealthPage>` (p16 C3(b) render) | `getHealth` → `[{code:'crop_not_applied', ok:false, detail:'x'}]` | the row's `.health-label` text is the raw code `crop_not_applied` (no friendly name any more: the code is not in `names`) | `[rule]` ch12 §Health screen; 10B retired |
| C5(a) | `<SettingsPage>` (p16 render shape) with `getSettings` → the p16 `shown` fixture rewritten for this phase (`default_shape: '1:1'`, `master_recipe: 'PSM Master'`, `web_recipe: 'PSM Web'`, no `crop_on_arrival`) | — | a `combobox` labelled `Default shape` whose options' values are, in order, `original, 1:1, 4:5, 3:2, 2:3, 16:9, free` with texts `Original, Square (1:1), 4:5, 3:2, 2:3, 16:9, Free`, value `1:1`; no element labelled `Shape on arrival`; a textbox labelled `Master recipe` with value `PSM Master`; choose `4:5`, type `My Master`, click `Save settings` → the patch has `default_shape === '4:5'` and `master_recipe === 'My Master'` and no `crop_on_arrival` key | `[rule]` ch12 §Settings + 10D MC-27 §Default |
| C6(a) | **live (optional; where Apple events are allowed; the app's own recipes, nothing to restore):** the real ensure against Capture One 16.8.6 through `JxaScripting` over a scratch data dir with `document_id` = the app's own document or `session test 1` | Capture One running | after the ensure, JXA reads back `PSM Preview`: `exportCropMethod() == 'ignore'`, `primaryScalingValue() == 2560`, `jpegQuality() == 85`, `bits() == 8`; `PSM Master`: `outputFormat() == 'TIFF'`, `bits() == 16`, `tiffCompression() == <MASTER_TIFF_COMPRESSION>`, `exportCropMethod() == 'ignore'`, `enabled() == false`; a second ensure writes nothing (`changed == []` for both) | `[rule]` evidence VERIFY-15 + 10C step 4 (live; the suite never needs Capture One, flow §6A) |

## Delegated choices
Whether `RecipeSpec` becomes a struct of all dependent values or three `const` `RecipeProps`; the order of keys in `ensure_recipe.js`
(the C3 ops assert membership, not order, except C3(d) where exactly one key is written); whether `legacy_default_shape` is a `match` or a
table; the Settings select component (native, as P23); whether the legacy mapping also logs at `info`; the test module split (one Rust
file is the default); the fake's fresh-recipe `bits` default (VERIFY-15 recorded a fresh recipe as 16-bit TIFF).

## Manual try-out
1. Launch the app with your existing settings (which say `"crop_on_arrival": "Square"`): Settings shows "Default shape: Square (1:1)" and a
   "Master recipe" field reading `PSM Master`; no "Shape on arrival". Save once: `settings.json` now has `default_shape` and no
   `crop_on_arrival`.
2. In Capture One, open the process recipes: `PSM Preview` reads 2560 px, quality 85, crop ignored; a new `PSM Master` (TIFF 16-bit, sRGB,
   full size, disabled) exists; `PSM Web` is unchanged. Health shows "Recipes: Ready".
3. Shoot one frame: it arrives, its preview appears after the usual delay at the **full frame** (no square crop is set in Capture One any
   more; the square framing returns through the app's own composition in the next phases). In Capture One the variant's crop tool is
   untouched.
4. Set a crop by hand in Capture One on that photo: the app's preview ignores it (the preview recipe ignores crops) and shows the full frame.
5. Export (no integration): the files are still rendered through `PSM Web` exactly as before this phase.

## Notes
- **Why the legacy mapping is fixed here and not an owner card.** The book says `default_shape` replaces `crop_on_arrival` and "missing →
  `1:1`"; a file with `crop_on_arrival` set is the owner's own dev Mac (`Square`), which maps to `1:1` under any reading. `off` → `original`
  is the only non-trivial choice ("leave as shot" = the whole frame); it is recorded here and in the narrative's "Choices made for you".
- **Why `PSM Web` survives this phase.** Export still renders through Capture One (`RenderPurpose::Export` → `web_recipe`) until P29 moves it
  onto the master render. Dropping the recipe now would make a fresh Mac's export fail with `recipe_missing`. P29 removes `web_recipe`,
  `RecipeSpec::WEB` and the ensure entry, and from then on `PSM Web` is "a recipe with another name" that the app never touches.
- **Fingerprint note for P28.** From this phase on, every new adjusted preview is crop-ignored and 2560 px. Previews rendered before this
  phase (1600 px, Capture One's crop respected) stay on disk until their RAW is next adjusted; P28 marks them stale once so they are
  re-rendered, and records the frame only from previews rendered after this phase (there is no way to tell an old preview's framing).
- **Projection stays off**: every externally owned shape here (recipe property spellings) is grounded by the recorded VERIFY-15;
  the suite runs on the fake.
- Registry additions and removals made by the planner in master plan §6.4, §6.6, §6.7, §6.8, §6.10, §7.5. No intention-book change.
- Estimated size: ~30 files (12 of them deletions or fixture edits), ~700 lines net negative on the Rust side, ~250 lines of new tests.

## Review log
