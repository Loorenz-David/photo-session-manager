---
phase: P29_master_and_output_render
title: Master and output render — the 16-bit PSM Master rendered on demand and cached, the Rust `render_output` job (turn, flip, straighten, pixel rect, Lanczos3, sRGB, one JPEG encode), the output cache key, local export through the app's own render
difficulty: high
projection: no
depends_on: [P28_composition_foundation]
---
# P29 — Master and output render: the 16-bit PSM Master rendered on demand and cached, the Rust `render_output` job, the output cache key, local export through the app's own render

## Goal
Implement 10D MC-28 (invariant M34) and move the local export onto it. Capture One's job now ends at the **master**: a full-size,
uncropped 16-bit TIFF through `PSM Master` (render purpose `master`, replacing `export`), uncompressed (VERIFY-15: a 61 MP master in 2.1–2.4 s and 361 MB; ZIP took 8 s), requested only when an output is needed and the
RAW has no master for its current `adjusted_gen`, stored at `masters/<path_key>/master-<adjusted_gen>.tif` (temp + rename; the previous
one deleted on commit). Every exported file is then made by the app's **`render_output` job** (infrastructure, one at a time on its own
runner): decode the master, convert to linear light, apply quarter turns and flip, straighten about the centre with high-quality
interpolation, cut the pixel rect of 10D MC-27's pixel rule (a split cuts its two halves), resize with Lanczos3 when the rect is larger
than the output size — never upscale — convert back to sRGB and encode **once**: JPEG, long edge 2048, quality 90, 4:4:4, sRGB profile
embedded (setting `output_format`, replacing the `PSM Web` recipe, which the app no longer creates, repairs or uses). Metadata: capture
date, camera and copyright copied from the master; GPS and private tags dropped; orientation written upright. Outputs are cached under the
key `(master adjusted_gen, composition revision, output_format)`; a change to any of them makes the cached file stale and it is replaced,
never exported. Masters are a cache too: deleted when a local export completes and the session is not dirty (the publish half is P30),
rendered again if needed. Export refuses, before any render, while a main or condition photo is `needs_review`, naming the photos; a master
whose geometry, orientation or (against an earlier master) pixel size contradicts the fingerprint makes its photos `needs_review` and fails
the run the same way; a compatible master makes its size the fingerprint's frame. The export run's staging folder
gains a `manifest.json` with every file's cache key, so P30's publish can tell a stale staging from a fresh one. The settings screen trades
"Web recipe" for the output size and quality. **Not in this phase:** outputs per slot in publishing, upload names, `photo_links` slots,
the gallery and damage-report changes, masters deleted after publish (all P30); any UI beyond the two settings fields; performance
measurements (P34).

## Read first
Book: `10D_composition_and_framing.md` **§MC-28 in full** (Recipes, Master, Output render, Thumbnail, Cache key, M34), §MC-27 Pixel rule and
Revisions (the master is compared with the fingerprint too: geometry read with the master render, orientation, and the master size once one exists), §MC-29 §Outputs (slots `single`, `left`, `right`; outputs expand in photo
order — this phase implements the expansion generically and P30 proves the split cases), §Grounding items 3, 5, 7 · `05A` MC-8 (the export
folder and names: `NN` now numbers outputs), MC-3 (integrity before rendering) · `10A` MC-16 (render jobs: issuing, completion acceptance,
re-issue once then fail; master timeout = the export's 120 s), MC-17 (`adjusted_gen` is the master's generation) · `07` §Completion without an
integration · `01` (H2: the master and RAW are never written; H3 as narrowed: exactly these pixel operations and no other) · `14` (M34; M7,
M15, M23 keep holding) · `11` §Layers · `12` §Settings · evidence `capture_one_16.8.6.md` §"VERIFY-15 result" (master time and size: uncompressed 2.1–2.4 s / 361 MB, the chosen
`Uncompressed`; the TIFF is tagged `sRGB IEC61966-2.1`; what else the master's metadata carries is **not recorded** — see Notes on the
metadata allowlist) and VERIFY-5/VERIFY-10 (processing-done timing) · master plan §3, §6.1 (`imaging/render.rs`, `media_store.rs`), §6.2 (crates:
`image`, `fast_image_resize`, `jpeg-encoder`, `kamadak-exif`; delegated: `imageproc`, `little_exif`), §6.3 (`masters/`, `outputs/`), §6.4
(`Imaging::render_output`, `RenderPurpose::Master`, `Store` additions), §6.6 (`needs_review` error, job kinds `render_master`, `render_output`),
§6.7 (`settings_get` keys), §6.8 (`output_format`; `web_recipe` retired), §6.10, §7.2, §7.5 (`SyntheticMaster`, `complete_master_render`) ·
process `flow.md` §4, §6, §6A · plans `P28_composition_foundation.md` (geometry contract, `Imaging` port, the materialise/fingerprint rule
this phase repeats for masters), `P27_retire_shape_on_arrival.md` (the interim `PSM Web` rule this phase ends; C3(b)/C3(c) tests change) ·
code (line numbers at `d763492`; P27/P28 shift some): `crates/core/src/services/export.rs` (`ExportFiles :30–47`; `Target :60–66`; `Run
:68–79`; `run :125–149`; `run_staged :151–187`; `staging_folder :189`; `snapshot :201–236`; `start :237–296` — integrity `:244–250`, the
per-photo `issue` loop `:277–294`; `issue :297–329` requests `RenderPurpose::Export`; `finalize :366–382`; `write_and_swap :384–417`
renames by position; `ExportStaging::completed :420–461` copies the Capture One output into staging; `failed :463–477`), `crates/core/
src/services/previews.rs` (`on_processing_done :126–180`: the `Export` arm `:165–169` hands the file to `export_staging`; `tick :212–237`
expiry → `callback.failed`), `crates/core/src/services/publish.rs` (`prepare :400–522` reuses a staging when `item.json` and every
`export_file_name` exist `:440–455` — unchanged here, P30 adds the manifest check), `crates/core/src/domain/editing.rs` (`RenderPurpose
:45–49`), `crates/core/src/ports/editing.rs` (`RenderCompletion :16–23`; `request_render :28–36`), `crates/core/src/mechanisms/naming.rs`
(`export_file_name :41–44`, `item_json :48–83`), `crates/core/src/mechanisms/composition.rs` (P28: `pixel_rect`, `split_halves`,
`default_rect`, `fingerprint_compatible`), `crates/infra/src/export_fs.rs` (`copy_in :30–35`, `write_in`, `swap`), `crates/infra/src/
jobs.rs` (`JobRunner::{only_kind, excluding_kind} :21–28`; `tick :33–77`), `crates/infra/src/previews_store.rs`, `crates/infra/src/
imaging.rs` (P28: `ImageOpsImpl`), `crates/integrations/src/editing/capture_one/{adapter.rs (`render :202–214` picks the recipe by purpose;
`expire_renders :344–448` timeouts 60/120 s), recipes.rs (`RecipeSpec::WEB` — removed), provision.rs (the recipe loop — drop the web entry)}`,
`crates/integrations/src/lib.rs` (`App::build`: runners `:490–491`, `render_export` registration `:529`, `InterruptedExport :855–863`;
`ExportFs::new :454`; `previews.export_staging :473`), `crates/infra/src/settings_file.rs` (`web_recipe` — removed; `output_format` added),
`crates/core/src/services/settings.rs` (allowed keys; validation), `apps/desktop/src-tauri/src/commands/settings.rs` (DTOs), `apps/desktop/
src/features/settings/SettingsPage.tsx` (`text('Web recipe') :99`), the sdef `:842–846` (`export crop method`), `:874` (`bits`), `:878`
(`TIFF compression`) · fakes and helpers: `crates/testkit/src/fake_capture_one.rs` (`new :67–78` builds `PSM Preview` + `PSM Web` → becomes
`PSM Preview` + `PSM Master`; `output_bytes :167–172` the stub; `process :584–595`; `issued :57`), `crates/testkit/src/test_app.rs`
(`complete_render :203–229`, `complete_all_renders :230–252`, `tick :192–201`), `crates/testkit/src/fixtures/synthetic_raw.rs` (the
hand-written IFD builder to extend into `SyntheticMaster`) · tests to reuse or change (named again by the implementer): `p09_local_export.rs`
(`export :112–115` → add `a.tick(0)` after completing masters so `render_output` runs; `c1a :118`, `c2a :139`, `c4a :189`, `c5a :208`,
`c6b :269` (timeout re-issue, now for the master)), `p13_publish_runs.rs` (`drain :107–112`; `c3e :489` hashes the staged file — still
true), `p14_publish_update.rs` (`drain :91`; `c5a–c5d :645–790`), `p27_retire_shape_on_arrival.rs` (C3(b): remove `PSM Master` from the fake
before the ensure; C3(c): invert — no `ensure_recipe:PSM Web`), `p00_spike.rs:77` (`web_recipe` assertion → `master_recipe`/`output_format`),
the frontend settings fixtures (`web_recipe` → `output_format: { long_edge: 2048, quality: 90 }`).

## Files expected to change
Backend (new): `crates/infra/src/imaging/render.rs` (the renderer), `crates/infra/src/imaging/linear.rs` (sRGB ⇄ linear LUTs), `crates/infra/
src/imaging/metadata.rs` (EXIF read from the master, APP1 write), `crates/infra/src/imaging/sRGB.icc` (or the runtime profile source —
Delegated choices), `crates/infra/src/media_store.rs` (`MasterFiles`, `OutputFiles`), `crates/integrations/src/render_output.rs`
(`RenderOutputJob`), `crates/core/src/domain/output.rs` (`OutputFormat`).
Backend (changed): `crates/core/src/domain/editing.rs` (`RenderPurpose::Master`), `crates/core/src/ports/{images.rs, store.rs}`,
`crates/core/src/services/{export.rs, previews.rs, settings.rs}`, `crates/core/src/error.rs` (`NeedsReview`), `crates/core/src/mechanisms/
naming.rs` (output numbering helper), `crates/infra/Cargo.toml` (`jpeg-encoder`, `kamadak-exif`; delegated extras), `crates/infra/src/
{lib.rs, imaging.rs → imaging/mod.rs, jobs.rs (`excluding_kinds`), settings_file.rs, sqlite/store.rs}`, `crates/integrations/src/
editing/capture_one/{adapter.rs, recipes.rs, provision.rs}`, `crates/integrations/src/lib.rs`, `crates/testkit/Cargo.toml` (`kamadak-exif`
dev), `crates/testkit/src/{test_app.rs, fake_capture_one.rs, fixtures/mod.rs, fixtures/synthetic_master.rs (new)}`, `apps/desktop/src-tauri/
src/commands/settings.rs`, `apps/desktop/src/services/bindings.ts` (regenerated), `apps/desktop/src/features/settings/SettingsPage.tsx`, the
settings fixtures.
Tests: `crates/testkit/tests/p29_master_and_output_render.rs` (new: C1–C8), `apps/desktop/src/features/settings/__tests__/
p29_output_format.test.tsx` (new: C4(f)), the P09/P13/P14/P27/P00 amendments above.

## Core mechanisms (unit tests allowed)
None new. (`pixel_rect`/`split_halves` keep P28's case-table test; the renderer is proven at its boundary, the produced file.)

## The render contract (copied into `render.rs` doc comments)
`Imaging::render_output(master: &Path, values: &CompositionValues, slot: OutputSlot, format: &OutputFormat, dest: &Path) -> Result<OutputInfo
{ width, height, frame: Frame }>`:
1. Decode the master to 16-bit RGB (`image`, TIFF). `frame = (W0, H0)`. Anything but RGB 8/16-bit → `Err(InvalidData)`.
2. Convert to **linear light** with a 65 536-entry LUT of the sRGB transfer function (the master is sRGB by recipe).
3. Quarter turns (clockwise) and flip (horizontal, applied **after** the turns, in the turned frame): pure index remaps. A clockwise
   quarter turn maps source `(x, y)` to turned `(H0 − 1 − y, x)` in a frame of `H0 × W0`; the horizontal flip maps turned `x'` to
   `W_t − 1 − x'`.
4. The pixel rect of MC-27's pixel rule on the turned frame `W_t × H_t` (`pixel_rect`, then `split_halves` and the half named by `slot`).
5. Straighten: with `θ = values.straighten` (degrees, clockwise on screen), output pixel `p` (pixel centre, turned-frame coordinates) samples
   the turned, flipped source at `s = c + R(−θ)(p − c)` with `c = (W_t/2, H_t/2)` (the exact formula of P28's contract), **bicubic** (or at
   least bilinear — Delegated choices name the library) interpolation in linear light; `θ = 0` is a plain copy (no resampling).
6. Resize to the output size with **Lanczos3** when the rect's long edge is greater than `format.long_edge`; the short edge is
   `round(long · short_rect / long_rect)`; **never upscale**.
7. Back to sRGB (LUT or exact transfer), 8 bits per channel; encode **once** as JPEG: quality `format.quality`, chroma **4:4:4**, an sRGB ICC
   profile in APP2, the EXIF APP1 of step 8; written to a temp file then renamed to `dest`.
8. Metadata: from the master's EXIF (`kamadak-exif` reads TIFF containers) copy **only** `Make`, `Model`, `DateTimeOriginal` (fallback
   `DateTime`), `Copyright` when present; write `Orientation = 1`; the writer may add the structural tags `ExifVersion`, `ColorSpace = 1`,
   `PixelXDimension`, `PixelYDimension`, `ResolutionUnit`, `XResolution`, `YResolution`, `YCbCrPositioning`; **nothing else** — no GPS IFD, no
   MakerNote, no XMP packet, no vendor tags (an allowlist, not a denylist).
The master and the RAW are opened read-only; the output is the only file written.

## Ordered tasks
0. **Registry first (master plan §6.10).** `RenderPurpose::Master` replaces `Export` everywhere (`render`, `expire_renders`, the registry);
   job kinds: per-photo tracking rows become `render_master` (target RAW, payload `{run_id, photo_id, adjusted_gen}`), the run row stays
   `render_export` (target `session:<id>`); new kind `render_output` (payload `{run_id, photo_id, slot, adjusted_gen, revision, format_key}`).
   `OutputFormat { long_edge: u32, quality: u8 }` (`psm_core::domain::output`; `Default` = 2048/90; `format_key() = "jpeg-{long_edge}-q{quality}"`).
   Settings key `output_format` (object; missing → default), `web_recipe` removed (serde ignores a legacy key). `Error::NeedsReview { photos:
   Vec<String> }` (code `needs_review`, `data.photos` = labels `Main 02`, `Condition 01` in display order; message `Re-frame first: Main 02,
   Condition 01`). `Imaging` gains `render_output`. `Store` gains `adopt_frame(raw, observed: Fingerprint) -> FrameCommit { needs_review: Vec<(i64,i64)> }`
   (the materialise/fingerprint rule of P28's `complete_preview`, factored so masters use it too: with `source master` a compatible observation
   **replaces** the stored fingerprint, so the master size becomes the frame), `adjusted_gen(raw) -> i64`. `MASTER_TIFF_COMPRESSION` stays
   `"Uncompressed"` (P27).
2. **Crates.** `crates/infra/Cargo.toml`: `jpeg-encoder = "0.7"` (0.7.1 on crates.io 2026-11-04; features for ICC/APP segments as the crate
   names them), `kamadak-exif = "0.6"` (0.6.1); the interpolation and EXIF-writing choice is delegated (`imageproc = "0.27"` and/or `little_exif
   = "0.6"` are the candidates verified to exist; a hand-written bicubic sampler and APP1 builder are equally acceptable — both are small).
3. **Recipes and settings.** `RecipeSpec::WEB` removed; the ensure loop ensures `PSM Preview` and `PSM Master` only; `CaptureOneAdapter` loses
   `web_recipe`; `render(Master)` uses `master_recipe`; the fake's default recipes become `PSM Preview` + `PSM Master`. `Settings.output_format:
   OutputFormat`; `SettingsService::set` allows `output_format` (object with integer `long_edge` in `512..=8192` and integer `quality` in
   `60..=100`, else `Err(InvalidValue("output_format"))`) and drops `web_recipe` (a patch naming it → `InvalidValue`). `SettingsDto`/`SettingsPatchDto`:
   `output_format: OutputFormatDto { long_edge, quality }` replaces `web_recipe`. Bindings regenerated.
4. **Media stores (`media_store.rs`).** `MasterFiles(AppPaths)`: `path(raw, adjusted_gen) = masters/<pk[0..2]>/<pk>/master-<g>.tif`;
   `commit(raw, g, source) -> PathBuf` (copy to `.tmp`, rename, then delete every other `master-*.tif` in the folder); `remove_all(raw)`.
   `OutputFiles(AppPaths)`: `path(raw, uid12, slot, g, rev, format_key) = outputs/<pk[0..2]>/<pk>/<uid12>-<slot>-g<g>-r<rev>-<format_key>.jpg`;
   `commit(dest, temp)`; `prune(raw, uid12, keep: &Path)` deletes the other `<uid12>-*.jpg` files. Both are plain file helpers behind
   core-side traits (`MasterStore`, `OutputStore` in `psm_core::ports::images`, or one `MediaFiles` trait — Delegated).
5. **Export through outputs.** `ExportService` reworked (same public surface: `run`, `run_staged`, `staging_folder`, `publish_swap`,
   `discard_staging`, `folder`): a target is one **(photo, slot)**; `start`: snapshot → integrity → **needs_review gate** (any main or condition
   photo of the snapshot with `state needs_review` → `emit(Failed)` and `Err(NeedsReview { photos })`, nothing rendered, no run row) → for each
   photo `comp = store.composition(id)`, `g = store.adjusted_gen(raw)` → for each `slot` of `comp.kind`: `dest = outputs.path(raw, uid12, slot,
   g, comp.revision, format_key)`; exists → `copy_in(staging, name, dest)` (target `Copied`); else if `masters.path(raw, g)` exists → enqueue
   `render_output` (`Store::enqueue_render_output(payload, now)`); else request the master once per RAW (`request_render(raw, Master, g,
   tracking = new_master_render_job(raw, run_id, photo_id, g), …)`, timeouts as the old export). Names: `export_file_name(key, output_index,
   outputs_in_group, group)` where outputs count slots in photo order (`mechanisms::naming::output_names(group_photos: &[(photo_id,
   &[OutputSlot])]) -> Vec<(photo_id, slot, name)>` — one helper, used by `start`, `write_and_swap` and P30's publish). `ExportStaging` becomes
   `master_ready(render, output)`: `masters.commit` → `imaging.dimensions(master)` → `store.adopt_frame(raw, Fingerprint { frame, source:
   Master, geometry: render.geometry })` (the geometry read with the master's `process`; materialise, adopt the master size, or needs_review) → needs_review non-empty → every run holding one of those photos fails with `needs_review` (`emit(Failed)`; the master file
   stays) → else enqueue `render_output` for each pending target of that RAW. New `output_ready(run_id, photo_id, slot, path)` → `copy_in`;
   all copied → `finalize`: `manifest.json` (`{"format_key": …, "outputs": {"<name>": {"photo_id", "slot", "adjusted_gen", "composition_rev"}}}`
   written **before** `item.json`), `item.json`, swap (unless deferred), then **masters lifecycle**: when `!defer_swap` (a local export) and
   `store.session(id).dirty == false` → `masters.remove_all(raw)` for every RAW of the run. `output_failed(run_id, reason)` → `fail(run, reason)`.
   `write_and_swap` drops deleted/moved photos and renames by the surviving output order as today.
6. **`RenderOutputJob`** (`crates/integrations/src/render_output.rs`, `JobHandler`): reads the payload; `run_id` unknown to the export service
   → `Ok(())` (a run from a previous process; logged); `comp = store.composition(photo_id)`; `values = comp.values()` or the default for its
   frame (`default_rect` on `imaging.dimensions(master)` — the frame was adopted at `master_ready`, so `rect` is Some here); `imaging.render_output(
   master, &values, slot, &format, temp)` → `outputs.commit(dest, temp)`, `outputs.prune(raw, uid12, dest)`, `export.output_ready(…)`; a render
   error → `export.output_failed(run_id, "render")` and `Ok(())` (no retry: the failure is deterministic). Registered on its own runner
   `App.render_jobs = JobRunner::new(store).only_kind("render_output")` with its own thread (like `publish_jobs`); the general runner uses
   `excluding_kinds(&["publish_step", "render_output"])`; `InterruptedExport` also handles `render_master`; a `render_output` row re-queued at
   restart finds no run and closes. `TestApp::tick` ticks `render_jobs` too.
7. **Renderer (`imaging/render.rs`)** per the contract; `linear.rs` holds the two LUTs; `metadata.rs` reads the master's EXIF and builds the
   APP1 segment; the ICC bytes per Delegated choices.
8. **Settings screen.** Replace `text('Web recipe', 'web_recipe')` with two number inputs: `Output size (long edge, px)` (`settings-output_long_edge`,
   min 512, max 8192) and `Output JPEG quality` (`settings-output_quality`, min 60, max 100); the save patch carries `output_format: { long_edge,
   quality }`.
9. **Testkit.** `fixtures/synthetic_master.rs`: `SyntheticMaster { w, h, pattern: Pattern, meta: Meta }` writing a **16-bit RGB uncompressed
   TIFF by hand** (little-endian, one strip, `BitsPerSample 16,16,16`, `Compression 1`, `Photometric 2`, `SamplesPerPixel 3`, `PlanarConfig 1`,
   plus `Make`, `Model`, `Copyright`, `Orientation`, an Exif sub-IFD (`0x8769`) with `DateTimeOriginal` (`0x9003`) and a GPS sub-IFD (`0x8825`)
   with `GPSLatitude` (`0x0002`) when `meta` asks for them — the TIFF 6.0 / EXIF 2.3 layouts; `kamadak-exif` must read them back, which is
   the fixture's own check). Patterns: `Solid([u16;3])`, `Marks { cols: Vec<u32>, rows: Vec<u32>, ink: [u16;3], paper: [u16;3] }` (1-px lines at
   the listed indices), `ColumnMarker { mid }` (grey `0x8000`, column `mid − 1` black, column `mid` white), `Checker1px` (black/white).
   `TestApp::complete_master_render(job, &SyntheticMaster) -> PathBuf` (writes `Output/<stem>.tif`, mtime = now, delivers `processing_done`);
   `complete_all_renders` writes a default `SyntheticMaster { 600, 400, Solid([0xC000; 3]) }` for jobs whose recipe is the master recipe and the
   stub JPEG for previews (so P09/P13/P14 keep working). `TestApp::decode(path) -> image::RgbImage`, `TestApp::outputs_dir(raw)`,
   `TestApp::masters_dir(raw)`.
10. Tests, `cargo test --workspace`, clippy, fmt, bindings regeneration (diff: `SettingsDto`/`SettingsPatchDto` `output_format`, `web_recipe`
    gone), `npm run typecheck`, `npm test`.

## Criteria (8 criteria, 27 rows; 1 `[rule]` row)
Fixtures: `K` = p09's session shape (`session`, `photo(a, s, name, class)` from `p09_local_export.rs`, extended: `photo` also delivers one
preview `SyntheticJpeg::rgb(2560, 1707, …)` so each composition has a frame 2560×1707 — **the masters below are 600×400 (ratio 1.5, compatible
within 0.02 %) unless the row says otherwise**). `export(a, s, master_for)` = `a.app.export.run(s)` → for each issued master job
`complete_master_render(job, master_for(variant))` → `a.tick(0)` until no `render_output` job is `queued`/`running`. `output(a, "K/K_01_main.jpg")`
decodes the staged/exported file. `dark(px) = max channel < 100`, `light(px) = min channel > 160`. `pk(raw)` = `path_key`.

| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | `with_fake_document()`; `K` with main `[a, b]`; `export` with `Solid` masters; `sha256_file(raw)` before and after | the first export of two RAWs | `issued` holds exactly `process(va, "PSM Master")` and `process(vb, "PSM Master")` and **no** `"PSM Web"`; after completion `masters/<pk(a)[0..2]>/<pk(a)>/master-<g>.tif` existed at `master_ready` (assert through the run's log or by making the export deferred: use `run_staged`, see C7) — for a local export assert instead that `K/K_01_main.jpg` and `K_02_main.jpg` exist and each RAW's SHA-256 is unchanged | MC-28 §Master ("rendered by Capture One through MC-16 (purpose master)"); M34 ("the master and RAW are never modified") |
| C1(b) | after (a): `a.app.export.run(s)` again, then `tick(0)` | an export with every output cached | **no** new `process` call (`issued.len()` unchanged); the export completes (`export_progress … Done`) and `K_01_main.jpg` is byte-equal to the first export's | MC-28 §Cache key ("An output is valid for (master adjusted_gen, composition revision, output_format)") |
| C1(c) | `K` with main `[a, b]`; `run` → complete `a`'s master only; let `b`'s time out (`tick(121_000)` twice: re-issue once, then fail) → the run fails; then `deliver_hook(variant_adjusted a)` outside the own-write window (`adjusted_gen` of `a` → `g+1`), `tick(20_000)`, complete its preview (`2560×1707`), `run` again and complete both masters | a kept master, then a new generation | after the failed run `master-<g>.tif` of `a` exists (kept: the run was not completed); after the second run completes `master-<g+1>.tif` was written and `master-<g>.tif` is **gone** (previous deleted on commit; assert before `finalize` deletes the folder by running this as a deferred `run_staged`, or assert the `outputs/` file name carries `g<g+1>`); the output file for `a` is `<uid12>-single-g<g+1>-r0-jpeg-2048-q90.jpg` | MC-28 §Master ("the previous one is deleted when a new one is committed"; "only when … the RAW has no master for its current adjusted_gen"); MC-16 (re-issue once, then fail) |
| C2(a) | `K` with main `[a]`; `composition_set(a, {normal, 1:1, named, 0 turns, none, 0, rect (0.25, 0, 0.5, 0.75)})` (pixel `{150, 0, 300, 300}` on 600×400); master `Marks { cols [298, 299], rows [198, 199], ink black, paper 0xC000 }` | a plain crop | `K_01_main.jpg` is 300×300; columns 148 and 149 are `dark` on every sampled row (rows 10, 150, 290) and columns 147 and 150 are `light`; rows 198 and 199 are `dark` at columns 10, 150, 290 and rows 197 and 200 `light` (the pixel rule placed the cut at `left 150`, `top 0` exactly) | MC-27 §Pixel rule; MC-28 §Output render ("cut the pixel rect"); M34 |
| C2(b) | master `Marks { cols [100, 101], rows [50, 51] }` 600×400; `composition_set(a, {normal, original, named, turns 1, flip horizontal, 0, rect (0, 0, 1, 1)})` | a quarter turn and a flip | output is 400×600; rows 100 and 101 are `dark` (the source column became a row: `turned (H0 − 1 − y, x)`), rows 99 and 102 `light`; columns **50 and 51** are `dark` and 49/52 `light` (without the flip they would be 348–349: the flip is applied after the turn, in the turned frame) | MC-28 §Output render ("apply quarter turns and flip"); MC-27 §Values ("applied first, lossless") |
| C2(c) | master `Marks { cols [299, 300] }` 600×400; `composition_set(a, {normal, 1:1, named, 0, none, straighten 10, rect (0.333333333, 0.25, 0.333333333, 0.5)})` (pixel `{200, 100, 200, 200}`) | a 10° straighten | output 200×200; for each of rows 0, 100, 199 the darkest column (min luminance) lies in `[116, 119]`, `[98, 101]`, `[81, 84]` respectively (a vertical centre line rotated clockwise by 10°: `x = 300 + d·sin 10°` with `d` the height above the centre — top right, bottom left; the opposite sign would give `[81, 84]` at row 0); the top-left and bottom-right output pixels are `light`-ish (`paper`, no empty corner: every channel > 120) | MC-28 §Output render ("apply straighten (rotation about the centre, high-quality interpolation)"); P28 geometry contract (sign convention) |
| C2(d) | a photo whose composition has **no rect** (never previewed: skip the preview in `photo`), `default_shape 1:1`; master `Marks { cols [98, 99, 500, 501] }` 600×400 | the default materialised from the master | output 400×400 (pixel `{100, 0, 400, 400}` = the largest centred square); the composition row after the run has `frame 600×400`, `rect (0.166666667, 0, 0.666666667, 1)`, `state ok`, `revision 0`; output column 0 is `light` and column 400 does not exist (the marks at 98–99 and 500–501 lie outside the square: no `dark` column anywhere) | MC-27 §Default (materialised late — P28 Notes); MC-28 §Master |
| C2(e) | `composition_set(a, {normal, original, …, (0, 0, 1, 1)})`; master 600×400 `Solid` | the whole frame | output 600×400 | MC-27 §Values (`original` keeps the whole frame) |
| C3(a) | master 1001×601 `ColumnMarker { mid: 500 }`; `composition_set(a, {split, 1:1, named, 0, none, 0, rect (0, 0.084026622, 1, 0.831946755)})` (`w·W = 1001 → 1000` even, `h·H = 500.0`; pixel `{0, 50, 1001→1000, 500}`) | a split | two staged files `K_01_main.jpg` (left) and `K_02_main.jpg` (right), both 500×500; left column 499 is `dark` and left column 498 is grey (`100 ≤ every channel ≤ 160`); right column 0 is `light` and right column 1 grey — adjacent halves share no column and miss none; `item.json` lists `["K_01_main.jpg", "K_02_main.jpg"]` under `main` | MC-27 §Pixel rule (split halves `[left, left+width/2)` and `[left+width/2, left+width)`); M34 ("adjacent equal squares with no shared or missing column"); MC-29 §Export (numbering per output) |
| C4(a) | master 3000×2000 `Solid`; `composition_set(a, original …)` | a rect larger than the output size | output 2048×1365 (`round(2048·2000/3000) = 1365`) | MC-28 §Output render ("resize to the output size (Lanczos3) when larger … long edge 2048") |
| C4(b) | master 600×400 `Solid`, `original` | a rect smaller than the output size | output 600×400 (never upscaled) | MC-28 §Output render ("never upscaled") |
| C4(c) | any output of (b) read as bytes | — | an APP2 segment (`FF E2`) starting with `ICC_PROFILE\0` is present and the embedded profile's `desc` text contains `sRGB`; the SOF0 segment (`FF C0`) lists three components whose sampling factor bytes are all `0x11` (4:4:4); exactly one `FF D8`…`FF D9` (one encode) | MC-28 §Output render ("4:4:4 chroma, sRGB profile embedded"; "encode once") |
| C4(d) | master 1024×1024 `Checker1px`; `settings_set({"output_format":{"long_edge":512,"quality":90}})`; `original` | the linear-light test | output 512×512; at 16 spread sample points every channel is within `[178, 196]` (the sRGB value of linear 0.5 is 187.5; an average taken in gamma space would read ≈ 128 and fail) | MC-28 §Output render ("convert to linear light … convert back to sRGB"); M34 ("colour shift") |
| C4(e) | `TestApp::new()`: `settings_get()`; `settings_set({"output_format":{"long_edge":1024,"quality":90}})`; `settings_set({"output_format":{"long_edge":100,"quality":90}})`; `settings_set({"output_format":{"long_edge":1024,"quality":101}})`; `settings_set({"web_recipe":"x"})` | the setting | default `{"long_edge":2048,"quality":90}` and no `web_recipe` key; the valid patch stored (`settings_get` shows it, the file has it); `100` → `Err(InvalidValue("output_format"))`, `101` → the same, stored value stays `1024/90`; `web_recipe` → `Err(InvalidValue("web_recipe"))` | MC-28 §Output render ("Output setting `output_format` (replaces the PSM Web recipe values): default JPEG, long edge 2048, quality 90") |
| C4(f) | `<SettingsPage>` with a fixture carrying `output_format: { long_edge: 2048, quality: 90 }` and no `web_recipe` | — | spinbuttons labelled `Output size (long edge, px)` (value 2048) and `Output JPEG quality` (value 90); no textbox labelled `Web recipe`; set 1600 and 85, `Save settings` → the patch's `output_format` equals `{ long_edge: 1600, quality: 85 }` | `[rule]` ch12 §Settings; MC-28 §Output render |
| C5(a) | master 600×400 `Solid` with `Meta { make: "Sony", model: "ILCE-7RM4", date_time_original: "2026:11:04 10:11:12", copyright: "© Loorenz", orientation: 6, gps: true }` | metadata | the output's EXIF (`kamadak-exif`): `Make == "Sony"`, `Model == "ILCE-7RM4"`, `DateTimeOriginal == "2026:11:04 10:11:12"`, `Copyright == "© Loorenz"`, `Orientation == 1`; **no** field in the GPS IFD; every field present is in the allowlist of the render contract step 8; no APP1 segment whose payload starts with `http://ns.adobe.com/xap/1.0/` (no XMP) | MC-28 §Output render ("Metadata: capture date, camera, copyright copied; GPS and Capture One private tags dropped; orientation written as upright") |
| C5(b) | master with `Meta::default()` (no EXIF tags beyond the TIFF structure) | a master without metadata | the output decodes; its EXIF has `Orientation == 1` and no `Make`/`Model`/`DateTimeOriginal`/`Copyright` | MC-28 §Output render (copy only what exists) |
| C6(a) | after C1(a): `composition_set(a, <valid, different rect>)` (revision 1); `run` + complete | a composition change | `outputs/<pk(a)[0..2]>/<pk(a)>/` holds exactly one file for `a`: `<uid12>-single-g<g>-r1-jpeg-2048-q90.jpg`; the `-r0-` file is gone; `K/K_01_main.jpg` is byte-equal to the `-r1-` file | MC-28 §Cache key ("a change to any of them makes it stale. Stale outputs are never exported") |
| C6(b) | after (a): `settings_set({"output_format":{"long_edge":1024,"quality":90}})`; `run` + complete | an output-format change | the only file for `a` is `…-r1-jpeg-1024-q90.jpg`; the exported `K_01_main.jpg` is 1024 px on its long edge | MC-28 §Cache key (`output_format` is part of the key) |
| C6(c) | after (b): read `K/manifest.json` (the swapped folder keeps it) | the manifest | `format_key == "jpeg-1024-q90"`; `outputs["K_01_main.jpg"] == {photo_id: a, slot: "single", adjusted_gen: g, composition_rev: 1}`; `item.json` unchanged in shape (M15: `main` lists `K_01_main.jpg`) | MC-28 §Cache key (the key recorded beside the files for P30's reuse check); M15 |
| C7(a) | after C1(a) (a completed local export; the session was never published, so it is not dirty) | masters after export | `masters/<pk(a)[0..2]>/<pk(a)>/` and `…/<pk(b)>/` do **not** exist; `outputs/…` files for `a` and `b` exist | MC-28 §Master ("Masters are a cache: … deleted once … its export completes and the session is not dirty") |
| C7(b) | the failed run of C1(c) (second master timed out) | a failed export | `master-<g>.tif` of `a` still exists after the failure; no `K/` folder was swapped in | MC-28 §Master (deleted only when the export **completes**) |
| C7(c) | after C7(a): `run` again | outputs cached, masters gone | completes with **no** `process` call and no `render_output` job (`SELECT COUNT(*) FROM jobs WHERE kind='render_output'` unchanged); files byte-equal | MC-28 §Master ("rendered again if needed later") — not needed while the outputs are valid; §Cache key |
| C8(a) | `K` with main `[a, b]`, `b` made `needs_review` (P28 C5(a) shape: an incompatible second preview 2560×1700) | export while a main photo needs review | `a.app.export.run(s)` → `Err(NeedsReview { photos: ["Main 02"] })`; **no** `process` call; no `.K.partial-*` folder; `export_progress … Failed` emitted; the `CommandError` of `export_run` has `code == "needs_review"` and `data.photos == "Main 02"` (desktop `p29_tests`) | MC-27 §Revisions ("publish confirmation and export refuse while any of the session's main or condition photos needs it (naming them)"); M33 ("never rendered for publish or export meanwhile") |
| C8(b) | a condition photo `needs_review`; an **unreviewed** photo `needs_review` (second app) | the label; an unreviewed photo | `photos == ["Condition 01"]`; with only the unreviewed photo flagged the export **proceeds** (unreviewed photos are not exported) | MC-27 §Revisions ("main or condition photos") |
| C8(c) | `K` with main `[a]` whose fingerprint is `{2560×1707, preview, default}`; `fake.set_variant_geometry(va, rotation 3)` before `run`; complete the master (600×400) | a master whose geometry moved | the composition becomes `needs_review` with `review_reason geometry`, fingerprint `{600×400, master, rotation 3}`; the run fails (`export_progress … Failed`); `master-<g>.tif` is committed and kept; no `render_output` job was enqueued for it; a later `run` → `Err(NeedsReview { photos: ["Main 01"] })` | MC-27 §Revisions ("With every preview and master render the adapter also reads the variant's Capture One geometry"); M33 |
| C8(d) | `K` with main `[a]`; `run` + complete with a 600×400 master (default geometry) → done (fingerprint now `{600×400, master, default}`); adjusted event (`g+1`), new preview `2560×1707` (compatible: a preview after a master); `run` + complete the master with **640×400** | a master whose size changed against the stored master size | after the preview: `state ok`, fingerprint still `{600×400, master, default}`; after the second master: `needs_review`, `review_reason frame`, fingerprint `{640×400, master, default}`, the run fails | MC-27 §Revisions ("the master size, when both exist, is equal") |

## Delegated choices
Interpolation library for the straighten (`imageproc::geometric_transformations` with `Interpolation::Bicubic` over `Rgb<u16>` or
`Rgb<f32>`, or a hand-written bicubic sampler); the EXIF writer (`little_exif`, or a hand-built APP1: a TIFF header + IFD0 + Exif IFD with the
allowlisted tags); the ICC bytes (a redistributable sRGB profile file committed with its licence note, or the macOS system profile at
`/System/Library/ColorSync/Profiles/sRGB Profile.icc` read at runtime with the committed file as fallback — C4(c) checks only that an sRGB
profile is embedded); whether linear light is held as `u16` or `f32` between steps (memory: a 61 MP master is 366 MB as `u16` RGB); the
thread model of the `render_jobs` runner (one dedicated thread is the default); the exact master-timeout handling of several runs sharing a
RAW (one `process`, all runs wait); whether `MasterStore`/`OutputStore` are one trait or two; the `Pattern` enum's extra variants; the Settings
input components.

## Manual try-out
1. Open a session with a few photos (their previews were re-rendered in P28) and press Export (no integration configured): Capture One renders
   one 16-bit TIFF per photo (watch `Output/_session_manager/` fill, then empty), then the app writes `~/Pictures/Session Manager Export/<SKU>/`
   with square JPEGs (the default composition) at 2048 px, plus `item.json` and `manifest.json`. Open one in Preview: the Inspector shows an
   sRGB profile, 2048 px, your camera model and capture date, no GPS.
2. Export again at once: it completes in a moment and Capture One renders nothing (outputs cached).
3. Settings → Output size 1600 → Save → Export: the files are 1600 px now. Because the masters were deleted after step 1 and the new size
   needs a fresh render, Capture One renders the TIFFs once more; export again and it renders nothing.
4. Settings shows "Output size" and "Output JPEG quality" instead of "Web recipe"; in Capture One, `PSM Web` still exists (left alone) and is
   no longer repaired by the app.
5. Rotate a photo 90° in Capture One (its composition becomes "needs review" as in P28 step 3): Export now refuses with "Re-frame first: Main 0n"
   and renders nothing. (Undoing the rotation does not clear it; the editor that confirms framing arrives in P32.)

## Notes
- **Why outputs live in app storage and not only in the staging folder.** The cache key is per output; a staging folder is per run and is
  swapped or discarded. Keeping `outputs/<pk>/<uid12>-<slot>-g<g>-r<rev>-<fmt>.jpg` makes "stale is never exported" a file-name fact (the
  key is the name) and makes a repeated export free. Pruning other files of the same `uid12` bounds the cache to one file per output.
- **Masters after a local export.** With no integration a session is never `published`, so `dirty` is always false and masters are always
  deleted after a completed export — the owner's "kept while the session is in use, deleted when archived" reading of 10D (orchestrator). The
  cost of a later re-export is one master render per photo (VERIFY-15 item 3 measures it).
- **Why the renderer never retries.** A failing decode or encode is deterministic; `RenderOutputJob` reports the run failed and closes the job,
  so the queue never loops. Capture One's side (the master render) keeps MC-16's re-issue-once-then-fail.
- **Linear light row (C4(d)) is the discriminating test** for the "convert to linear light" clause: every other row would pass a gamma-space
  pipeline. The checkerboard downscale by exactly 2 gives a flat 0.5-linear field under Lanczos3 (symmetric weights), i.e. sRGB 187–188.
- **Metadata grounding (rule 17) — one owed observation.** VERIFY-15 recorded the master's colour tag (`sRGB IEC61966-2.1`) but not which
  EXIF tags Capture One writes into the TIFF. The allowlist (Make, Model, DateTimeOriginal, Copyright) is the standard EXIF carrier; if the
  orchestrator can read one master with `exiftool`/`sips -g all` before dispatch and finds the tags elsewhere (XMP only), the coordinator amends
  step 8 and C5(a) and the synthetic master then carries that shape. Without that reading the row stays as written: an absent tag is simply not
  copied (C5(b)), so a wrong guess costs metadata, never pixels.
- **Interim rule ends.** `PSM Web` leaves the ensure and the settings; `web_recipe` in an existing `settings.json` is ignored on read and dropped
  on the next save (`Settings::set` writes the struct).
- Registry additions made by the planner in master plan §6.1–§6.8, §6.10, §7.5. No intention-book change.
- Estimated size: ~26 files, ~2 100 lines (≈ 700 renderer + stores, ≈ 450 export rework, ≈ 150 settings, ≈ 800 tests and fixtures incl. the
  TIFF writer).

## Review log

### 2026-11-04 — implement r1 (claude-opus, pipeline-implementer; Claude fallback run)
**Built.** `RenderPurpose::Master` (recipe `master_recipe`, timeout 120 s) replaces `Export`; `PSM Web` left the recipe specs, the
ensure loop, the adapter and the settings. `psm_core::domain::output::{OutputFormat (2048/90, format_key), OutputInfo, slot_name}`;
`Error::NeedsReview{photos}` (code `needs_review`, `data.photos` = labels joined `", "`). `ports::images`: `Imaging::render_output`,
`MasterStore`, `OutputStore` (two traits), `uid12`. `Store`: `adopt_frame -> FrameCommit`, `adjusted_gen`, `enqueue_render_output(raw,
payload, now)`, `new_master_render_job` (replaces `new_export_render_job`; kind `render_master`), `job_due(.., excluded_kinds: &[&str])`.
`ExportService` reworked around targets `(photo, slot)`: snapshot → integrity → needs_review gate (no run row, `Failed` emitted) → per
target cached output copied / `render_output` queued (master on disk) / one master `process` per RAW; `master_ready` commits the master,
reads its size, `adopt_frame(Master, geometry)`; needs_review fails every run holding a flagged photo (master kept); else queues the RAW's
waiting outputs. `output_ready`/`output_failed`; `finalize` writes `manifest.json` then `item.json`, swaps, and on a local export of a
non-dirty session releases the run's masters. `mechanisms::naming::{output_names, review_label}`. Infra: `imaging/{mod.rs, render.rs
(the contract verbatim in the doc comment), linear.rs, metadata.rs}`, `media_store.rs` (`MasterFiles`, `OutputFiles`), `jobs.rs`
`excluding_kinds`, `settings_file.rs` `output_format` (legacy `web_recipe` ignored, dropped on save), SQLite store additions.
Integrations: `render_output.rs` (`RenderOutputJob`), `App.render_jobs` on its own thread, general runner excludes `publish_step` and
`render_output`, `InterruptedExport` also closes `render_master`. Desktop: `OutputFormatDto` in `SettingsDto`/`SettingsPatchDto`,
bindings regenerated; Settings screen: "Output size (long edge, px)" and "Output JPEG quality" replace "Web recipe". Testkit:
`SyntheticMaster` (hand-written 16-bit LE TIFF, EXIF/GPS sub-IFDs, four patterns), `TestApp::{complete_master_render, is_master_job,
decode, outputs_dir, masters_dir}`, `complete_all_renders` writes a 600×400 solid master for master jobs, `tick` ticks `render_jobs`.

**Tests.** `crates/testkit/tests/p29_master_and_output_render.rs` — 26 tests, one per Rust row (C1(a)–C8(d) minus C4(f)); C8(a)'s
command half is `apps/desktop/src-tauri/src/lib.rs` `p29_tests::c8a_export_run_rejects_with_needs_review`; C4(f) is
`apps/desktop/src/features/settings/__tests__/p29_output_format.test.tsx`. Stamp: `cargo test --workspace` 508 passed / 0 failed,
clippy `-D warnings` and `fmt --check` clean, `npm run typecheck` clean, `npm test` 132 passed.

**Judgment calls (Delegated choices and holes).**
1. Interpolation: hand-written Catmull-Rom bicubic over the turned frame (no `imageproc`); linear light held as `u16` between steps.
2. EXIF writer: `kamadak-exif`'s `experimental::Writer` (no `little_exif`); the metadata is read before the pixels are decoded so the
   EXIF buffer of a 361 MB master is freed first.
3. ICC: built in code (`metadata::srgb_profile`, ICC v4.3, D50 PCS, Bradford-adapted colorants, parametric sRGB curve, desc
   `sRGB IEC61966-2.1`) — no profile file shipped or read at runtime. macOS `sips` reads it as `sRGB IEC61966-2.1`.
4. `MasterStore`/`OutputStore` are two traits; the render runner is one dedicated thread.
5. Several runs sharing a RAW: one `process`; every waiting run proceeds on `master_ready` or fails on the final timeout.
6. **Prune keeps the other slot of the same key.** Task 4 says `prune` deletes "the other `<uid12>-*.jpg` files"; read literally a
   split's right half would delete its left half's cache. `prune` deletes the photo's files whose key (`-g…-r…-<fmt>`) differs from
   the kept file's.
7. **Capture One's master TIFF is removed from its output folder after it is copied into `masters/`** (manual try-out step 1:
   "`Output/_session_manager/` fill, then empty"; 361 MB per master). The RAW and the committed master are never written.
8. A master completion no run waits for is ignored (not committed).
9. Masters released at finalize skip a RAW another in-flight run still needs.
10. `RenderOutputJob` names the output by the composition revision it actually rendered (read in the same row as the values) and
    passes it to `output_ready(.., revision)` — a save between enqueue and render can never mislabel the cache key. `output_ready`
    therefore carries `revision`; `enqueue_render_output` carries the RAW as the job target; `output_names` takes `key` and `group`.
11. Publish's retouch generation: a cached output has no master tracking row, so the run row's payload records `retouch_gens`
    (per photo, at run start) and `export_retouch_gen` reads it (P13/P14 unchanged in behaviour).
12. `adopt_frame`: any compatible **master** observation replaces the fingerprint (task 0), not only a preview → master one.
13. Labels padded like export names (`max(2, digits(n))`): `Main 02`, `Condition 01`.
14. `TestApp::tick` order: general jobs → `render_jobs` → `publish_jobs`, so a publish step sees its export done in the same tick
    (keeps P13/P15 timings).
15. Fixture shapes: C3(a) previews at 1001×601 and C4(d) at 2048×2048 (the composition must validate on the stored frame and the
    master must be compatible — a square master against a landscape preview is a frame change); the C6 chain uses 3000×2000
    masters (a 600×400 master cannot yield a 1024 px output: never upscaled); C8(a)/(b) flag photos with P28 C5(a)'s geometry change
    (the row's "incompatible second preview 2560×1700" is compatible by P28 C5(b)); C5(a)'s allowlist admits `ExifIFDPointer`
    (the structural pointer to the Exif IFD holding `DateTimeOriginal`, `ExifVersion`, `ColorSpace`, `PixelX/YDimension`).

**Deviations.**
- C3(a)'s rect `h = 0.831946755` is refused by P28's split check (`|1001 − 2·h·601| = 1.0000005 > 1`); the test uses `h = 0.83194676`,
  which gives the same pixel rect `{0, 50, 1001 → 1000, 500}`, so the asserted outcome is the row's.
- Tests outside the plan's amendment list changed because `RecipeSpec::WEB`/`web_recipe`/`job_due` changed: `p04` c3a/c3b (master
  recipe instead of web), `p17` c1a and the `job_due` call, `p24` c1a/c1b/c4a (a local "owner JPEG" spec stands in for the web spec),
  `p25` c5c (`tick(0)` after the renders), `p13` `start_upload` (runs the render runner), desktop `p21_tests` (`master_recipe` instead
  of `web_recipe`), seven frontend settings fixtures (`output_format` instead of `web_recipe`). P14 needed no change.
- The plan lists `crates/infra/src/imaging/sRGB.icc`; no profile file exists (judgment call 3).

**Observations.** Live probe (scratch crate outside the repo, read-only on `~/Pictures/session test 1/Output/session test 11952 1.tif`,
a Capture One 16.8.6 TIFF of a ZV-E10 RAW): decoded, `Make SONY`, `Model ZV-E10`, `DateTimeOriginal 2026:10:25 13:34:20` read from the
TIFF and copied; 2048×2048 rendered with a 2° straighten in 0.38 s (release build); embedded profile read by `sips` as `sRGB
IEC61966-2.1`. That file is an 8-bit export, not a `PSM Master` output; no live master render was triggered (camera not needed; the
suite never depends on Capture One). The metadata grounding the Notes call owed is thereby partly answered: Capture One TIFFs carry
Make/Model/DateTimeOriginal as EXIF tags, as step 8 assumes.
