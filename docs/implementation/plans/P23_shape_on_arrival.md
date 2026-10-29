---
phase: P23_shape_on_arrival
title: Shape on arrival — the app sets Capture One's crop shape (Square by default) once on every new photo; tiles show true proportions
difficulty: high
projection: no
depends_on: [P22_frontend_tidy_up]
---
# P23 — Shape on arrival: the app sets Capture One's crop shape (Square by default) once on every new photo; tiles show true proportions

## Goal
Implement book chapter `10B_shape_on_arrival.md` (owner decision 2026-10-29, R35, invariant M31). Every RAW that becomes a
photo for the first time (capture arrival by hook, watcher or startup scan; import) gets one `crop_on_arrival` job on the
serialised Capture One lane. The job asks the editing adapter to set the configured crop aspect ratio **only if the
variant's crop is still the untouched full frame**, in one JXA call (read state and write in the same script, so no
photographer edit can slip between them). A crop the photographer set is never touched (skipped). The attempt happens
once per RAW and survives a relaunch; copies, moves and later arrivals of the same RAW never re-apply. Failures retry
with the MC-19 back-off and give up after a fixed number of attempts into a health item ("Shape not applied to n
photos"); a configured name Capture One does not list gives the health item "Shape on arrival: '<name>' is not a
Capture One crop ratio" and no write. The setting `crop_on_arrival` (default `Square`, `off` = leave as shot) is in
`settings.json` and in the Settings screen as "Shape on arrival", a select fed by the adapter's list plus "Leave as
shot". Workspace tiles show the whole adjusted (or original) preview fitted inside the tile instead of trimmed to fill
it; the stage already fits. The app's own crop write never counts as a retouch and never dirties a session (own-write
window, 07B MC-21). **Not in this phase:** any other crop operation (rotation, straightening, moving a crop), applying
the shape to photos that arrived before this phase or while the setting was `off`, read-back of crops set by hand,
per-photo shape choices in the UI, changes to the sessions-list thumbnails, Capture One's Next Capture Adjustments.

## Read first
Book: `10B_shape_on_arrival.md` (whole chapter: MC-25, M31) · `01` (H2, H3: Capture One does the cropping, H5, H7, H8) ·
`14` (M31; M24, M26, M28 for the rows this phase must not break) · `10` §Editing provider, §Capture One adapter (Basic
layer sentence points here) · `10A` MC-17 (adjusted events, own-write window, sidecar fallback), MC-19 (variant resolution,
the serialised background lane, back-off, 30 s timeout, `mark_own_write`), MC-20 (the original preview is unchanged) ·
`05A` MC-2 (arrival, row creation, "the transaction is the only dedupe"; move and copy) · `07B` MC-21 (retouch unless inside
the own-write window; the app's own writes never dirty) · `12` §Settings, §Health screen · `15` R35 · evidence
`capture_one_16.8.6.md` §"Crop by script (orchestrator probe, 2026-10-29)" (the only grounded crop facts: untouched variant
reads `crop aspect ratio` `""` and `crop width/height` 5981×3987; `available crop aspect ratios` = `Unconstrained, Original,
Output, Square, 1x2, 1x3, 2x3, 3x4, 4x5, 5x7, 16x9`; `variant.cropAspectRatio = "Square"` → 3987×3987 centred; the adjusted
hook fired and `adjusted-13.jpg` came out 1600×1600; `app.maximumCrop(variant)` restores the full frame exactly) ·
dictionary `docs/reference/capture_one_16.8.6.sdef` variant properties `:1193–1215` (`crop aspect ratio`, `available crop
aspect ratios`, `crop width`, `crop height`, `crop size`) and the `maximum crop` command `:2036–2042` (direct parameter
variant; optional `apply` boolean "Directly apply the calculated crop. Defaults to true"; result `rectangle`) ·
master plan §3 (standing rules), §6.4 (ports: `EditingProvider`, `CaptureOneScripting`), §6.5 (`jobs`, `raw_state`), §6.6
(job kinds, error and health codes, `CROP_GIVE_UP_ATTEMPTS`), §6.7 (commands; `crop_ratios_list`), §6.8 (settings keys;
`crop_on_arrival`), §6.9 (test layout), §7.2 (commands), §7.5 (`TestApp`, `FakeCaptureOneScripting`) · process `flow.md` §4
(one behaviour test per row), §6 (no questions), §6A (the suite never needs Capture One) · plans `P21_backend_robustness.md`
(running before this phase: `Settings::open_lenient`, health codes, `JobRunner::tick` unknown kinds, `AppState` save lock —
anything in `settings_file.rs`, `jobs.rs`, `health.rs`, `lib.rs`, `state.rs` may have moved a few lines) and
`P22_frontend_tidy_up.md` (running before this phase: `SettingsPage.tsx` secret field focus/blur and the `back` prop removal;
new test files `features/shell/__tests__/p22_shell_tidy.test.tsx` and `features/workspace/__tests__/p22_workspace_tidy.test.tsx`;
`useWorkspaceState.ts` deferred reloads) · code (line numbers at `6c5e3c4`, 2026-10-29): `crates/core/src/ports/editing.rs`
(`EditingProvider`: `mirror_tag`, `import`, `capabilities`), `crates/core/src/domain/editing.rs` (`EditingCapabilities`,
`Job`), `crates/core/src/domain/mod.rs:12` (`HealthItem`), `crates/core/src/error.rs` (`Error`; `VariantNotFound`,
`Timeout`, `Provider`, `InvalidValue`), `crates/core/src/services/arrival.rs` (`ArrivalService::new`, `accept` → `store
.arrival_insert(NewPhoto, &key)`, `import`), `crates/core/src/services/photos.rs` (`PhotosService::import`),
`crates/core/src/services/editing_startup.rs` (`EditingStartup::run`; `check_recipes` branch `:45`),
`crates/core/src/services/health.rs` (`HealthService::{set, set_ok, clear, has, report}`), `crates/core/src/services/
settings.rs` (allowed keys `:36–48`; `get` adds `effective_preview_delay_s`), `crates/core/src/services/previews.rs`
(`on_variant_adjusted` `:181`: retouch ⇔ outside `last_own_write_at + 30 s`; `tick`, `request_initial`),
`crates/core/src/services/review.rs` (`move_photos` `:149`, `copy_photos` `:153`), `crates/core/src/ports/store.rs` (`Store`:
`arrival_insert`, `job_due`, `job_done`, `job_retry`, `requeue_running_jobs`, `mark_own_write`, `preview_state`,
`adjusted_event`), `crates/infra/src/sqlite/store.rs` (`arrival_insert` `:1052`: `existing` lookup, the `hash_job_exists`
once-per-RAW pattern, the job loop `:1091`; `adjusted_event` `:704`; `job_done` `:916`, `job_retry` `:927`,
`requeue_running_jobs` `:932`), `crates/infra/src/jobs.rs` (`JobHandler`, `JobRunner::tick` `:33`, the error→code match
`:53`, `backoff`), `crates/infra/src/settings_file.rs` (`Settings` struct, `Default`, `open_lenient`, `set`),
`crates/integrations/src/editing/capture_one/scripting.rs` (`CaptureOneScripting` trait), `jxa.rs` (`SCRIPTS` `:13` — 14
entries, `IMPORT` `:32`, `call`, `variant_location`; every script is written as `lib.js` + `\n` + script into
`<data>/hooks/jxa/`), `jxa/lib.js` (`imageByPath(doc, raw, folder)`), `jxa/set_color_tag.js` and `jxa/get_color_tag.js` (the
argv shape `[doc, raw, folder, variant, …]` this phase's script follows), `adapter.rs` (`variant_background` `:59`,
`mirror_tag` `:464`: `mark_own_write` **before** the write, timeout handling, read-back), `gate.rs` (`GatedScripting`: every
trait method is forwarded through the gate), `mirror.rs` (`MirrorTag` job handler: the shape this phase's handler copies),
`crates/integrations/src/lib.rs` (`App::build`: `open_lenient` `:321`, `requeue_running_jobs` `:349` inside the real-start
branch, `ArrivalService::new` `:385`, `runner.register("mirror_tag", …)` `:437`, the startup `EditingStartup` thread `:484`;
`AppHandles::health_check` `:168`), `apps/desktop/src-tauri/src/commands/settings.rs` (`SettingsDto`, `SettingsPatchDto`),
`apps/desktop/src-tauri/src/commands/health.rs` (the command shape to copy for `crop_ratios_list`), `apps/desktop/src-tauri/
src/lib.rs` (`specta_builder` `collect_commands!`), `apps/desktop/src/services/settings.ts`, `apps/desktop/src/features/
settings/SettingsPage.tsx` (`text(...)`/`shop(...)` field builders `:64`, the Capture One section `:75`), `apps/desktop/src/
features/health/HealthPage.tsx` (`names` `:9`), `apps/desktop/src/features/workspace/{PhotoTile.tsx (`Thumb`,
`PhotoTileFace`), workspace.css:64–71 (`.photo-tile`, `.tile-grid { aspect-ratio: 4 / 3 }`, `.thumb-image { object-fit:
cover }`), StageImage.tsx + workspace.css:28 (`.stage-layer { object-fit: contain }` — already fits)}` · tests to reuse:
`crates/testkit/tests/p05_capture_arrival.rs` (`app`, `root`, `file`, `external`, `db`, `session`, `watch`, `hook`, `service`
`:75` builds a `PhotosService`), `p04_capture_one_adapter.rs` (`last_job` `:43`), `p06_live_previews.rs` (`state` `:62`,
`change_seq`/`retouch_gen` assertions `:378–407`), `p17_real_run_fixes.rs` (`c2b` `:437–485`, `c3e` `:488–530`: the **Node
harness** — `node -e <harness> <script paths>` with a fake `Application` object, `eval` of the generated script, `run(argv)`),
`p16_operations.rs` (`stored`, `settings_get` assertions `:45–60`), `crates/testkit/src/fake_capture_one.rs`
(`FakeVariant`, `FakeState`, `ops`, `add_image`), `crates/testkit/src/test_app.rs` (`deliver_hook`, `tick`, `kill_and_restart`
= a real start (`start_backup` true, `run_startup` false), `complete_all_renders`, `mark_published`, `add_photo`),
`apps/desktop/src/features/settings/__tests__/p16_settings.test.tsx` (`shown` fixture; the `@/services/settings` mock
factory lists its exports explicitly), `features/shell/__tests__/p19_navigation.test.tsx:16` and `features/pending/__tests__/
p18_pending_state.test.tsx:22` (same mock factory shape), `features/workspace/__tests__/p08_workspace_ui.test.tsx` (render
helpers).

## Files expected to change
Backend: `crates/core/src/domain/editing.rs` (`ShapeOutcome`), `crates/core/src/error.rs` (`CropRatioUnknown { name }`),
`crates/core/src/ports/editing.rs` (`crop_aspect_ratios`, `shape_on_arrival`), `crates/core/src/ports/store.rs`
(`arrival_insert` third argument; `count_jobs`, `requeue_failed_jobs`, `job_fail`), `crates/core/src/services/arrival.rs`
(carries `shape_on_arrival: bool`), `crates/core/src/services/editing_startup.rs` (ratio check), `crates/core/src/services/
settings.rs` (allowed key + validation), `crates/infra/src/sqlite/store.rs` (job enqueue in `arrival_insert`; the three
store methods), `crates/infra/src/jobs.rs` (`JobHandler::max_attempts`, terminal failures → `job_fail`),
`crates/infra/src/settings_file.rs` (`crop_on_arrival`), `crates/integrations/src/editing/capture_one/{scripting.rs (two
trait methods), gate.rs (forwarding), jxa.rs (`SCRIPTS` 14 → 16; two call sites), jxa/crop_aspect_ratios.js (new),
jxa/apply_crop.js (new), adapter.rs (two `EditingProvider` methods), shape.rs (new: `ShapeOnArrival` job handler)}`,
`crates/integrations/src/editing/capture_one/mod.rs` (`pub mod shape`), `crates/integrations/src/lib.rs` (`ArrivalService`
flag, handler registration, `EditingStartup` ratio argument at both call sites, failed-job re-queue at real start and in
`health_check`, `crop_not_applied` derivation), `crates/testkit/src/fake_capture_one.rs` (crop state per variant, list,
faults, ops), `apps/desktop/src-tauri/src/commands/settings.rs` (`crop_on_arrival` in both DTOs; `crop_ratios_list`
command), `apps/desktop/src-tauri/src/lib.rs` (`collect_commands!`), `apps/desktop/src/services/bindings.ts` (regenerated).
Frontend: `apps/desktop/src/services/settings.ts` (`listCropRatios`), `apps/desktop/src/features/settings/SettingsPage.tsx`
("Shape on arrival" select), `apps/desktop/src/features/health/HealthPage.tsx` (`names` for the two codes),
`apps/desktop/src/features/workspace/workspace.css` (`.thumb-image` fits).
Tests: `crates/testkit/tests/p23_shape_on_arrival.rs` (new: C1–C4(a)), `apps/desktop/src/features/settings/__tests__/
p23_shape_on_arrival.test.tsx` (new: C4(b), C5(b)), `apps/desktop/src/features/workspace/__tests__/p23_tile_fit.test.tsx`
(new: C5(a)); mock factories of `@/services/settings` gain `listCropRatios: vi.fn(async () => [])` where `<SettingsPage>` is
rendered: `features/settings/__tests__/p16_settings.test.tsx:9`, `features/shell/__tests__/p19_navigation.test.tsx:16`,
`features/pending/__tests__/p18_pending_state.test.tsx:22`, and P22's `features/shell/__tests__/p22_shell_tidy.test.tsx` (name
per the P22 plan; verify at prompt time); the p16 `shown` fixture gains `crop_on_arrival: 'Square'` (typecheck).

## Core mechanisms (unit tests allowed)
None. (The untouched-frame rule lives in `apply_crop.js` and is tested through the Node harness, C1.)

## Ordered tasks
0. **Registry first (master plan §6.4, §6.6, §6.7, §6.8).** `ShapeOutcome { Applied, Skipped, UnknownRatio }` in
   `psm_core::domain::editing`; `Error::CropRatioUnknown { name }` (`crop_ratio_unknown`); job kind `crop_on_arrival`
   (`JobKind::CropOnArrival`); health codes `crop_not_applied`, `crop_ratio_unknown`; settings key `crop_on_arrival`;
   command `crop_ratios_list`; constant `CROP_GIVE_UP_ATTEMPTS` (`crates/integrations/src/editing/capture_one/shape.rs`,
   value per master plan §6.6 — see Notes, owner card 1).
1. **Scripting: two operations, one race-free call.** `CaptureOneScripting` gains `fn crop_aspect_ratios(&self, id: &str)
   -> Result<Vec<String>>` and `fn apply_crop_if_untouched(&self, id: &str, variant: &str, ratio: &str) ->
   Result<ShapeOutcome>`; `GatedScripting` forwards both; `JxaScripting` adds `crop_aspect_ratios.js` (argv `[doc]`) and
   `apply_crop.js` (argv `[doc, raw, folder, variant, ratio]`, located through `variant_location` like `set_color_tag`) to
   `SCRIPTS` (count 16). `crop_aspect_ratios.js`: the first variant it can find (the document's own `images`, then each
   collection's `images`) → `JSON.stringify(variant.availableCropAspectRatios())`; no variant anywhere → `[]`.
   `apply_crop.js`, in this order and in **one** osascript run: (1) `imageByPath` → `variant = image.variants.byId(id)` (none →
   throw `variant not found`); (2) `available = variant.availableCropAspectRatios()`; `ratio` not in it → return
   `{"outcome":"unknown_ratio","ratio":<current>,"width":w,"height":h,"available":[…]}` — no `maximumCrop`, no write;
   (3) `current = variant.cropAspectRatio()`; if `current` is neither `""` nor `"Unconstrained"` → `skipped` — no
   `maximumCrop`, no write; (4) `max = a.maximumCrop(variant, {apply: false})`; read the rectangle's width and height
   (accept the array form `[x, y, w, h]` and the object form `{width, height}` — the live row records which one Capture One
   returns); a missing or non-numeric size **throws** (an unknown comparator is an error, never a skip); (5)
   `cropWidth()/cropHeight()` ≠ max → `skipped`, no write; (6) otherwise `variant.cropAspectRatio = ratio`, read back,
   return `{"outcome":"applied","ratio":<read back>,"width":…,"height":…}`. Rust parses the JSON; `applied` with a read-back
   ratio ≠ requested → `Err(Error::Provider("crop read-back mismatch"))` (retried like a tag that did not read back).
2. **Adapter.** `EditingProvider` gains `fn crop_aspect_ratios(&self) -> Result<Vec<String>>` (background lane) and
   `fn shape_on_arrival(&self, raw: &str, ratio: &str, now_ms: i64) -> Result<ShapeOutcome>`: resolve the variant with
   `variant_background` (MC-19 fallback included), `store.mark_own_write(raw, now_ms)` **before** the call (the hook echo
   of a write must fall inside the own-write window; a skipped attempt marking own-write costs nothing on a brand-new RAW —
   Notes), then `background.apply_crop_if_untouched(...)`; a `Timeout` is returned as `Timeout` (the next attempt's single
   call re-reads the state first, so a write that landed is seen as a non-empty ratio → `Skipped`, which is the right end
   state). `UnknownRatio` → `Err(Error::CropRatioUnknown { name })`. `capabilities()` unchanged (one adapter; no new flag).
3. **Job kind and handler.** `psm_integrations::editing::capture_one::shape::ShapeOnArrival { editing, store, health,
   previews: Arc<PreviewService>, ratio: String }` (`ratio` = the setting, `"off"` included) implements `JobHandler`:
   `ratio == "off"` → `Ok(())` (the job closes as done; nothing is asked of Capture One); else
   `editing.shape_on_arrival(&job.target, &ratio, now_ms)`: `Applied` → `previews.on_variant_adjusted("", &job.target)` (the
   hook's own MC-17 entry point: the own-write window just opened makes it a non-retouch adjusted event, the RAW is stale
   and re-renders after the delay or on focus even when Capture One's hook does not fire — Notes), `health.clear(
   "variant_not_found")`, `Ok(())`; `Skipped`
   → `Ok(())`; `Err(VariantNotFound)` → `health.set("variant_not_found", raw)` and the error (as `MirrorTag` does); other
   errors propagate. `JobHandler` gains `fn max_attempts(&self) -> Option<i64> { None }`; `ShapeOnArrival` returns
   `Some(CROP_GIVE_UP_ATTEMPTS)`. `JobRunner::tick`, on `Err(error)`: `code` as today plus `Error::CropRatioUnknown{..} =>
   "crop_ratio_unknown"`; **terminal** ⇔ the error is `CropRatioUnknown` **or** `handler.max_attempts()` is `Some(m)` and
   `job.attempts + 1 >= m`; terminal → `store.job_fail(job.id, code, now_ms)` (`status='failed'`, `attempts+1`, `error=code`);
   otherwise `job_retry` with `backoff(attempts)` exactly as today (1, 2, 4, 8, 16, 32, 60, 60 … s). `Store` gains
   `job_fail`, `count_jobs(kind, status, exclude_error: Option<&str>) -> i64` and `requeue_failed_jobs(kind, now_ms) -> usize`
   (`failed` → `queued`, `attempts = 0`, `error = NULL`, `next_attempt_at = now`). Registration in `App::build` beside
   `mirror_tag`, **always** (so a queued `crop_on_arrival` row never hits P21's `unknown_job_kind` path when the setting is
   `off`). The settings rebuild (`settings_set`) rebuilds the runtime, so the handler always carries the current name.
4. **Enqueue on first sight, once per RAW, in the arrival transaction.** `Store::arrival_insert(new, path_key,
   shape_on_arrival: bool)`: inside the existing transaction, when a photo row is inserted **and** no `photos` row existed
   for `new.raw_path` in any session before this insert (an extra `SELECT EXISTS … WHERE raw_path=?1` taken before the
   insert; `existing` is per-session for imports) **and** no `jobs` row of kind `crop_on_arrival` exists for the target
   (the `hash_job_exists` pattern) **and** `shape_on_arrival` is true, insert `('crop_on_arrival', raw, '{}', 'queued', 0,
   created_at, …)` **before** `render_preview` in the job order (`original_preview`, `crop_on_arrival`, `hash`,
   `render_preview`), so the first preview render usually already carries the shape. `ArrivalService` carries
   `shape_on_arrival: bool` (`settings.crop_on_arrival != "off"`, set in `App::build`; constructor argument or a `with_…`
   setter — Delegated choices) and passes it from `accept` (hook, watcher, scan and import all go through `accept`).
   `photo_insert_or_get`, `copy_photo` and `move_photos` enqueue nothing (MC-25: never for a copy or move).
5. **Setting.** `Settings.crop_on_arrival: String`, default `"Square"` (struct-level `#[serde(default)]` already fills a
   missing key). `SettingsService::set` allows the key; a value that is not a non-empty string → `Err(InvalidValue(
   "crop_on_arrival"))`; any non-empty string is stored as typed (core passes the name opaquely, H7). `SettingsDto` and
   `SettingsPatchDto` gain `crop_on_arrival` (`String` / `Option<String>`). New command `crop_ratios_list(state) ->
   Result<Vec<String>, CommandError>` = `app.editing.crop_aspect_ratios()` (thin; `spawn_blocking` like `health_get`); on
   `Err` it rejects with the provider error (the screen then offers only "Leave as shot" and the current value). Bindings
   regenerated (`cargo test -p psm-desktop export_bindings`).
6. **Health.** (a) `EditingStartup` gains `crop_on_arrival: Option<&str>` (None when `off`); after the recipes step:
   `Some(name)` → `provider.crop_aspect_ratios()`; `Ok(list)` with `!list.is_empty()`: `list.contains(name)` →
   `health.clear("crop_ratio_unknown")`, else `health.set("crop_ratio_unknown", format!("Shape on arrival: '{name}' is not a
   Capture One crop ratio"))`; `Ok([])` or `Err` → no change (no verdict without a list); `None` →
   `clear("crop_ratio_unknown")`. Both call sites (`App::build` startup thread, `AppHandles::health_check`) pass the setting.
   The job sets the same item (same text) on `crop_ratio_unknown` and clears it on `Applied`. (b) `crop_not_applied` is
   **derived**: `n = store.count_jobs("crop_on_arrival", "failed", Some("crop_ratio_unknown"))`; `n > 0` → `health.set(
   "crop_not_applied", "Shape not applied to {n} photo{s}")` (`photo` for 1, `photos` otherwise); `n == 0` → `clear`.
   Derived in `App::build` (real start) and at the start of `AppHandles::health_check`, in both places **counted first,
   then re-queued** (c), so a report names what had failed when the check began and goes green only on a check after a
   successful attempt (10B: "until a later attempt succeeds"). (c) **Later attempts.** `store.requeue_failed_jobs(
   "crop_on_arrival", now)` runs at a real start (`App::build` with `start_backup`, beside `requeue_running_jobs`; a settings
   rebuild does not re-queue — P17 F1) and in `AppHandles::health_check` ("Retry checks"), in both cases only when the
   setting is not `off`.
7. **Settings screen.** `services/settings.ts`: `listCropRatios(): Promise<string[]>` over `commands.cropRatiosList()`.
   `SettingsPage`, Capture One section, after Style: label `Shape on arrival` for a native `<select id="settings-crop_on_arrival">`
   whose options are `Leave as shot` (value `off`) first, then one option per name from `listCropRatios()` (visible text =
   the name), plus the stored value as an extra option when it is neither `off` nor in the list (so an unknown stored name
   stays visible and selectable); the list is fetched once on mount, a failed fetch is an empty list (no error shown). The
   save patch carries `crop_on_arrival: settings.crop_on_arrival`. `HealthPage.names` gains `crop_not_applied: 'Shape on
   arrival'` and `crop_ratio_unknown: 'Shape on arrival'`.
8. **True proportions.** `workspace.css:70` `.thumb-image` → `object-fit: contain` (tile keeps `aspect-ratio: 4 / 3` and
   its `--stage` background, so a square photo shows letterboxed in a 4:3 tile — "fitted inside the tile, not trimmed").
   `.stage-layer` already uses `contain` (unchanged). The drag overlay and the folded strip render `PhotoTileFace`/`Thumb`,
   so they follow. `.row-thumb img` (sessions list, 36×28 px) is out of scope (Goal).
9. **Fake (`FakeCaptureOneScripting`).** `FakeVariant` gains `crop_ratio: String` (default `""`), `crop: (i64, i64)` (default
   `(5981, 3987)`), `max_crop: (i64, i64)` (default `(5981, 3987)`); `FakeState` gains `crop_ratios: Vec<String>` (default =
   the evidence list of 11 names), `fail_apply_crop: Option<String>` (every `apply_crop_if_untouched` → `Err(Provider(msg))`
   while set), `timeout_next_apply_crop: bool`. `crop_aspect_ratios(id)` → `Ok(crop_ratios.clone())` (op `crop_aspect_ratios`).
   `apply_crop_if_untouched(id, variant, ratio)`: op `apply_crop:<variant>:<ratio>`; faults first; variant unknown →
   `Err(VariantNotFound)`; `ratio` not in `crop_ratios` → `Ok(UnknownRatio)`; `crop_ratio ∈ {"", "Unconstrained"}` and
   `crop == max_crop` → set `crop_ratio = ratio`, for `"Square"` `crop = (min, min)` of `max_crop` (other names leave `crop`
   as it is), push op `set_crop:<variant>:<ratio>`, `Ok(Applied)`; else `Ok(Skipped)`. A helper `set_variant_crop(variant,
   ratio, (w, h))` lets a test pre-crop a variant.
10. Tests (§6.9: `crates/testkit/tests/p23_shape_on_arrival.rs`; the two frontend files), `cargo test --workspace`, clippy,
    fmt, bindings regeneration (diff expected: `SettingsDto`, `SettingsPatchDto`, `cropRatiosList`), `npm run typecheck`,
    `npm test`. The live row C6(a) is run by the implementer if its sandbox allows Apple events, else by the reviewer
    (P16/P17 precedent); its outcome and the observed `maximumCrop` return shape go in the Review log.

## Criteria (6 criteria, 15 rows; 4 `[rule]` rows)
Harness shape for C1 (P17 `c2b`/`c3e` pattern; the generated scripts are written by `JxaScripting::new(&AppPaths::for_dir
(tempdir))` into `<dir>/hooks/jxa/`): `global.Application = () => app` with `app = { running: () => true, documents: { byId:
id => doc }, maximumCrop: (v, opts) => { calls.push(opts); return max; } }`; `doc = { images: { whose: ({path}) => () =>
path === raw ? [image] : [] }, collections: () => [] }`; `image = { variants: { byId: id => variant } }`; `variant` exposes
`cropWidth()`, `cropHeight()`, `availableCropAspectRatios()` returning the evidence list, and `cropAspectRatio` modelled as
P17 modelled `captures` (`Object.defineProperty`: get → `() => ratio`, set → records `writes.push(v)` and sets `ratio = v`).
`max` = `{x: 0, y: 0, width: 5981, height: 3987}` (object form; the script also accepts `[x, y, w, h]`). The harness prints
`JSON.stringify({result: JSON.parse(run(argv)), calls, writes})` (or `{error: String(e)}` when `run` throws).

| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | `node -e <harness> <jxa>/apply_crop.js` | variant `ratio: ""`, `cropWidth 5981`, `cropHeight 3987`; argv `[doc, raw, folder, "7", "Square"]` | `result.outcome == "applied"`, `result.ratio == "Square"`; `calls == [{apply: false}]` (exactly one `maximumCrop`, before the write); `writes == ["Square"]` | MC-25 (untouched full frame → the configured ratio is set, in one call) |
| C1(b) | same harness, two runs | (1) variant `ratio: "4x5"`, crop `4784×3987`; (2) variant `ratio: "Unconstrained"`, crop `3000×2000` (max `5981×3987`) | (1) `outcome == "skipped"`, `calls == []`, `writes == []`; (2) `outcome == "skipped"`, `calls == [{apply: false}]`, `writes == []` | MC-25 (a photographer-set crop is left exactly as it is; both halves of the untouched conjunction) |
| C1(c) | same harness, three runs; plus `node -e <harness> <jxa>/crop_aspect_ratios.js` | (1) argv ratio `"Squircle"` on an untouched variant; (2) `"Square"` on an untouched variant with `app.maximumCrop` returning `null`; (3) `crop_aspect_ratios.js` with argv `[doc]` and `doc.images()` → `[image]` | (1) `outcome == "unknown_ratio"`, `result.available` equals the evidence list, `calls == []`, `writes == []`; (2) `run` throws (the harness prints `error`), `writes == []`; (3) prints the evidence list of 11 names, `Square` among them | MC-25 (unknown name → no crop attempted; an unreadable maximum is never read as "untouched"); 10B §Setting (the adapter lists the names) |
| C2(a) | `TestApp::with_fake_document()`; p05 `session(&a, "K1")` activated; `fake.add_image(<root>/K1/a.ARW, &["7"], false)`; p05 `file` + `hook` for `K1/a.ARW`; `a.tick(0)`; then `deliver_hook(variant_adjusted{variant_id:"7", raw})` at `+1 000 ms` | default settings (`crop_on_arrival` absent from the file) | after `tick`: one `jobs` row `kind='crop_on_arrival'`, `target == canon(a.ARW)`, `status='done'`; fake variant `7` has `crop_ratio == "Square"`, `crop == (3987, 3987)`; `preview_state(raw).last_own_write_at == Some(<clock at the tick>)` and `retouch_gen == 0`; the row's `photos.session_id` is K1's session; after the echo hook: `retouch_gen` still `0`, `adjusted_gen >= 1`, the session's `change_seq` unchanged by the hook | M31 / MC-25; M28 (the app's own write never counts as a retouch) |
| C2(b) | `with_fake_document()`; two external RAWs `x.ARW`, `y.ARW` (p05 `external`), fake images `"11"` (untouched) and `"12"` pre-cropped `set_variant_crop("12", "4x5", (4784, 3987))`; `service(&a).import(&[x, y])`; `a.tick(0)`; then `a.kill_and_restart()`; `a.scan()`; `service(&a).import(&[x, y])` again; `a.tick(0)` | import of one untouched and one hand-cropped RAW; a relaunch; the same import again | after the first tick: two `crop_on_arrival` rows, both `done`; variant `11` reads `Square`/`(3987, 3987)`, variant `12` still `4x5`/`(4784, 3987)`; ops contain exactly one `set_crop:11:Square` and no `set_crop:12`; after the restart, scan and second import: still exactly two `crop_on_arrival` rows (`SELECT COUNT(*) … WHERE kind='crop_on_arrival'`), ops still hold exactly one `set_crop:` | M31 / MC-25 (import counts; once per RAW; a relaunch does not re-apply; skipped counts as attempted) |
| C2(c) | `with_fake_document()`: (1) `settings_set({"crop_on_arrival":"off"})` then p05 `hook` for `K1/b.ARW` (image `"8"`), `tick(0)`; (2) fresh app, default setting: `hook` + `watch` (p05 helpers) for `K1/c.ARW` (image `"9"`) before any tick, then `tick(0)`; (3) fresh app: `add_photo(s1, "d.ARW")` (image `"10"`), `mark_published(s1)`, then `review().copy_photos(&[id], s2)` and `review().move_photos(&[id], s3)`, `tick(0)` | `off`; a double delivery; copy and move of an existing (published) photo | (1) a `photos` row exists for `b.ARW`, **no** `crop_on_arrival` row, no `apply_crop:` op; (2) exactly one `crop_on_arrival` row for `c.ARW`, one `set_crop:9:Square` op; (3) no `crop_on_arrival` row at all, no `apply_crop:` op (three rows on `d.ARW`, zero jobs of the kind) | M31 (no published RAW written; never for a copy or move); MC-25 §When it applies |
| C3(a) | `with_fake_document()`; p05 `hook` for `K1/e.ARW` **without** `add_image`; `tick(0)`; then `fake.add_image(e.ARW, &["13"], true)` (folder collection only); `tick(1_000)` | the variant is not in Capture One yet, then appears in the folder collection | after the first tick: the `crop_on_arrival` row has `status='queued'`, `attempts == 1`, `error == "variant_not_found"`, `next_attempt_at == <t0> + 1_000`; health has `variant_not_found` (`ok == false`); after the second tick: `status='done'`, variant `13` reads `Square`, `(3987, 3987)`; ops contain `find_image_in_folder:<e.ARW>`; health has no `variant_not_found` and no `crop_not_applied` | MC-25 §Failures (variant not found yet → MC-19 back-off, then applied); MC-19 (folder-collection fallback) |
| C3(b) | `with_fake_document()`; `fake.add_image(<root>/K1/h.ARW, &["16"], false)`; `fake.state…fail_apply_crop = Some("busy")`; `hook` for `K1/h.ARW`; `tick(0)`; `complete_all_renders()`; `fail_apply_crop = None`; `tick(1_000)`; `tick(15_000)` | Capture One refuses once; the first preview render was already issued with the uncropped frame | after the first tick: the row is `queued`, `attempts == 1`, `error == "provider"`, `next_attempt_at == <t0> + 1_000`; the fake's `issued` holds one `process("16", "PSM Preview")`; after `tick(1_000)`: the row is `done`, variant `16` reads `Square`, `preview_state(raw).adjusted_gen > rendered_gen` (stale) and `retouch_gen == 0`; after `tick(15_000)`: `issued` holds **two** `process("16", "PSM Preview")` entries (the applied shape re-rendered the preview without any hook) | M24 / MC-17 (the preview shows the new shape); MC-25 §Consequences; M28 (not a retouch) |
| C3(c) | `with_fake_document()`; `fake.state…fail_apply_crop = Some("busy")`; `hook` for `K1/f.ARW` (image `"14"`); drive `tick` through the back-off until the row is no longer `queued`; `app.health_check()` (#1); drive `tick` through the series again until the row is no longer `queued`; `fail_apply_crop = None`; `app.health_check()` (#2); `tick(0)`; `app.health_check()` (#3); separately, a second app brought to the same `failed` state → `kill_and_restart()` | a Capture One that keeps refusing, then recovers | at give-up: `status='failed'`, `error == "provider"`, `attempts == CROP_GIVE_UP_ATTEMPTS`, variant `14` still `""`; report #1 has item `crop_not_applied`, `ok == false`, detail exactly `Shape not applied to 1 photo`, and right after #1 the row is `queued` with `attempts == 0` (the check counted first, then re-queued); the second series ends `failed` again with `attempts == CROP_GIVE_UP_ATTEMPTS`; report #2 has the item again (detail unchanged) and the row is `queued`, `attempts == 0`; after `tick(0)`: `done`, variant `14` reads `Square`; report #3 has no `crop_not_applied`; the restarted app: right after `kill_and_restart()` the row is `queued` with `attempts == 0` | MC-25 §Failures (give up after the limit; health until a later attempt succeeds); master plan §6.6 `CROP_GIVE_UP_ATTEMPTS` (asserted by name, not by literal — charter rule 13) |
| C3(d) | `with_fake_document()`; `settings_set({"crop_on_arrival":"Squircle"})`; `app.health_check()`; `hook` for `K1/g.ARW` (image `"15"`); `tick(0)`; `app.health_check()`; then `settings_set({"crop_on_arrival":"Square"})`; `app.health_check()`; `tick(0)`; `app.health_check()` | a name Capture One does not list, then corrected | first report: item `crop_ratio_unknown`, `ok == false`, detail exactly `Shape on arrival: 'Squircle' is not a Capture One crop ratio`; after the arrival tick: the `crop_on_arrival` row is `failed` with `error == "crop_ratio_unknown"`, no `set_crop:` op, variant `15` reads `""`; the second report keeps `crop_ratio_unknown` and has **no** `crop_not_applied`; after the correction and its `health_check()`: no `crop_ratio_unknown`, the row is `queued`; after `tick(0)`: `done`, variant `15` reads `Square`; the last report has neither item | MC-25 §Failures (unknown name → no crop attempted, the exact health text); H7 (core never knows the names) |
| C4(a) | `TestApp::new()`: `settings_get()`; `settings_set({"crop_on_arrival":"off"})`; `settings_set({"crop_on_arrival":""})`; `settings_set({"crop_on_arrival": 5})`; `app.handles().editing.crop_aspect_ratios()` | a settings file written before this phase (no key) | `settings_get()["crop_on_arrival"] == "Square"`; after the `off` save: stored `settings.json` has `"crop_on_arrival": "off"` and `settings_get()` shows `off`; the empty string and the number are both `Err(Error::InvalidValue(_))` and the stored value stays `off`; `crop_aspect_ratios()` returns the fake's 11 names with `Square` among them | MC-25 §Setting (missing → Square; `off`; the adapter's list) |
| C4(b) | `<SettingsPage>` (p16 render shape) with `getSettings` → p16 `shown` + `crop_on_arrival: 'Square'`, `listCropRatios` → `['Unconstrained','Original','Output','Square','4x5']`; second render with `crop_on_arrival: 'Squircle'` and `listCropRatios` rejecting | — | first render: a `combobox`/`select` labelled `Shape on arrival` whose options' texts are, in order, `Leave as shot, Unconstrained, Original, Output, Square, 4x5`, value `Square`; choose `Leave as shot`, click `Save settings` → the patch has `crop_on_arrival === 'off'`; choose `4x5`, save → `'4x5'`; second render: options `Leave as shot, Squircle` (the stored name stays selectable), value `Squircle`, no `role="alert"` | `[rule]` ch12 §Settings + 10B §Setting ("Shape on arrival" offers the adapter's list plus "Leave as shot") |
| C5(a) | `import css from '../workspace.css?raw'` (vitest) | the stylesheet the webview loads | the declaration block of the `.thumb-image` rule contains `object-fit: contain` and not `object-fit: cover`; the `.stage-layer` block contains `object-fit: contain`; `.tile-grid` still declares `aspect-ratio: 4 / 3` | `[rule]` 10B §What the photographer sees (whole preview fitted inside the tile, not trimmed) |
| C5(b) | `<HealthPage>` (p16 C3(b) render) | `getHealth` → `[{code:'crop_not_applied', ok:false, detail:'Shape not applied to 2 photos'}, {code:'crop_ratio_unknown', ok:false, detail:"Shape on arrival: 'Squircle' is not a Capture One crop ratio"}]` | both details are shown, each inside a `.health-row[data-ok="false"]` whose `.health-label` text is `Shape on arrival` (no raw code shown) | `[rule]` ch12 §Health screen; 10B §Failures |
| C6(a) | **live (optional, run where Apple events are allowed; must restore):** scripts as `JxaScripting::new` writes them, into a temp dir outside the repo; `D="/Users/davidloorenz/Pictures/session test 1"`, `F="$D/Capture/PSM_PROBE_A"`, `R="$F/session test 12199.ARW"`, `V` = what `find_image_in_folder.js "$D" "$F" "$R"` prints (`"130"` on 2026-10-26) | Capture One 16.8.6 running with `session test 1` open; the camera need not be tethered | (1) `crop_aspect_ratios.js "$D"` prints a list containing `Square`; (2) `apply_crop.js "$D" "$R" "$F" "$V" Square` prints `outcome "applied"`, `ratio "Square"`, `width == height` (if it prints `skipped` because the probe frame is already cropped, first restore with the one-liner in (4) and repeat); (3) the same call again prints `skipped` with `ratio "Square"` and the same width/height (nothing re-written); (4) restore: `osascript -l JavaScript -e 'const a=Application("Capture One");const d=a.documents.byId("'"$D"'");const v=d.images.whose({path:"'"$R"'"})()[0].variants.byId("'"$V"'");a.maximumCrop(v);JSON.stringify([v.cropAspectRatio(),v.cropWidth(),v.cropHeight()])'` prints `["",5981,3987]` (the untouched reading of the evidence); (5) the Review log records the JS shape `maximumCrop(v, {apply:false})` returned (array or object) and that step (3) did not change the crop | `[rule]` evidence "Crop by script" (2026-10-29) + 10A MC-19 (live; the suite never needs Capture One, flow §6A) |

## Delegated choices
How `ArrivalService` receives the flag (constructor argument or `with_shape_on_arrival(bool)`); whether `ShapeOutcome` is
`Copy`; the exact `Provider` message texts; whether `apply_crop.js` reads `cropSize()` or `cropWidth()/cropHeight()`; the
rectangle-parsing helper's shape (array and object forms both accepted, else throw); whether `count_jobs` takes the
excluded error as a parameter or a second method; where the health count is computed (`App::build` end and `health_check`
start are required, a refresh after each job run is optional); the native `<select>` versus a shadcn component (none is
installed; native is the plan's default); whether the Settings page shows "Not available" under the select when the list
could not be read (no `role="alert"`); the option text for `off` is fixed (`Leave as shot`); the fake's crop for non-Square
names (unchanged size is enough); whether `set_variant_crop` lives on the fake or in the test file; test module split
(one Rust file is the plan's default).

## Manual try-out
1. Set the camera's aspect to 1:1 (or leave it at 3:2 — it makes no difference). Open a session in the app, shoot one
   frame. Within a few seconds the new tile shows the whole photo (no trimmed edges); after the preview delay the tile
   and the stage show a **square** photo, and in Capture One the same variant has the crop tool set to Square.
2. In Capture One, change that photo's crop by hand (any ratio, any size). Wait, press the app's window, relaunch the
   app: the crop stays exactly as you set it; the app never changes it back.
3. Import a photo from the Finder by dropping it into the session: it becomes square too. Drop the same file again: nothing
   changes in Capture One.
4. Settings → "Shape on arrival": the list shows Capture One's names (Square, 4x5, 16x9, …) and "Leave as shot". Choose
   "Leave as shot", Save, shoot a frame: it stays as shot. Choose "Square" again.
5. Quit Capture One, shoot nothing, but import a file into the session; wait about three minutes: Settings → Health shows a
   red "Shape on arrival — Shape not applied to 1 photo". Start Capture One, press "Retry checks": the row disappears and
   the photo becomes square.
6. In `~/Library/Application Support/com.loorenz.sessionmanager/settings.json` set `"crop_on_arrival": "Squircle"` and
   relaunch: Health shows "Shape on arrival: 'Squircle' is not a Capture One crop ratio"; new frames are left as shot.
   Fix the name in Settings and press Retry checks: the waiting frames become square.
7. Publish a session with square photos: the images in the shop are square (the web recipe exports the crop).

## Notes
- **Why one JXA call (race).** The photographer may crop a frame in Capture One within the second after it arrives. A
  read in one osascript run and a write in another could overwrite that crop. `apply_crop.js` reads and writes in the same
  run; between the two there is nothing but the comparison. The 30 s timeout and the lane gate are unchanged.
- **Safety order inside the script.** The ratio check runs before `maximumCrop`; only an empty or `Unconstrained` ratio
  reaches the maximum-crop call, which is made with `{apply: false}`. The evidence verified `maximumCrop(variant)` (apply
  by default) and the setter; `apply: false` and the rectangle's JS shape are **not yet observed** — the live row C6(a)
  observes them. If `apply: false` turned out to apply, the exposure is a freehand (`Unconstrained`) crop smaller than
  the frame being expanded; the implementer then stops and reports (flow §6: an impossible task), with
  `image.dimensions` (sdef `:1637`) as the candidate comparator for the orchestrator to decide.
- **Why `mark_own_write` before the call, even for a skip.** MC-21 counts an adjusted event as a retouch unless it falls
  inside `last_own_write_at + 30 s`. The write's hook echo must be inside the window, and the single call cannot mark
  "only if written" before it writes. A skipped attempt also marks own-write; on a brand-new RAW (no earlier row, hence
  no publish link) a suppressed retouch within those 30 s changes nothing observable (retouch bumps `change_seq` only on
  sessions holding a `main`/`condition` row of the RAW, which a fresh row is not). C2(a) asserts `retouch_gen == 0`.
- **Why the job marks the RAW stale itself (task 3).** The book expects Capture One's adjusted hook to re-render the
  preview (observed live for a tethered frame, which is the primary variant). MC-17's hook fires only for the primary
  variant, and the sidecar fallback is suppressed inside the own-write window, so an import of several files would leave
  all but one preview rectangular while the export is square. One non-retouch `adjusted_event` after `Applied` closes this
  at the cost of at most one extra render, coalesced by MC-17's delay with the echo when both arrive. The job uses the
  hook's entry point (`PreviewService::on_variant_adjusted`) rather than `Store::adjusted_event`, so `last_adjusted` is set
  and the render follows the delay/focus rule instead of firing at once. C3(b) observes the second `process`.
- **Give-up limit — book hole, owner card 1.** 10B says "give up after the same limit" with MC-19 as the reference, but
  MC-19 names no limit (the colour-tag mirror retries at the 60 s cap forever; `jobs.rs` never fails a job). The plan
  carries `CROP_GIVE_UP_ATTEMPTS` as a named constant; the recommended value **8** (back-off 1+2+4+8+16+32+60+60 s ≈ 3 min,
  the order of MC-16's "re-issue once, then fail") is **pending the owner's answer**; every row asserts the constant by
  name. The coordinator fills the value into master plan §6.6 before the implementer prompt is compiled.
- **"Until a later attempt succeeds".** The book names no trigger for the later attempt. Chosen: a real start and the Health
  page's "Retry checks" re-queue `failed` shape jobs with a fresh back-off series; a settings rebuild does not (P17 F1: a
  rebuild must not touch jobs while the old workers live; `failed` rows are owned by nobody, but the rule is kept simple).
  `crop_ratio_unknown` failures are excluded from the "not applied" count so the two book messages stay distinct; a
  corrected name is applied after the next Retry checks or start (try-out step 6).
- **Health text.** The book writes "Shape not applied to n photos"; for n = 1 the plan pins `Shape not applied to 1 photo`
  (C3(c)), plural otherwise (C5(b) shows `2 photos`).
- **Projection stays off** (P17 precedent: JXA scripts and live rows ran without it). The externally owned shapes this phase
  relies on are grounded in the 2026-10-29 evidence except `maximumCrop(…, {apply: false})` and its return shape, which the
  live row observes before anything depends on them; the script fails loud (throws) on an unparseable rectangle.
- **P21/P22 are running while this is planned.** Expected drift: `settings_file.rs` (lenient open already landed),
  `jobs.rs` (unknown-kind retry already landed), `health.rs`, `lib.rs`, `state.rs`, `SettingsPage.tsx` (secret field,
  `back` prop), and new P22 test files that mock `@/services/settings`. Line numbers in Read-first are from `6c5e3c4`; names
  are the contract.
- **Why `.thumb-image` and not the tile's aspect.** The grid, dnd-kit layout and the P19/P20 drop behaviour measure 4:3
  tiles; changing the fit, not the tile, shows the true shape without touching the drag geometry. The sessions-list
  thumbnails (`.row-thumb img`, 36×28) keep `cover` (not tiles, Goal).
- Registry additions made by the planner (master plan §6.4, §6.6, §6.7, §6.8, §7.5). No intention-book change is needed
  beyond the hole carded above; `docs/intention` is untouched by this plan.
- Estimated size: ~24 files, ~950 lines (half tests).

## Review log
- 2026-10-29 (orchestrator): card 1 answered from 10B's intent (option A): `CROP_GIVE_UP_ATTEMPTS` = 8; 10B §Failures and master plan §6.6 amended (changelog 23). Gate open.

- 2026-10-29 (codex, implement r1): Implemented first-sight crop jobs, single-call Capture One crop guard, retry/give-up and health, setting/command/UI, true-proportion tiles and fake adapter. Assumptions: C4(a) uses a configured fake document for the provider-list assertion because `TestApp::new()` has no document; `crop_aspect_ratios.js` searches all images for the first variant; native select and `cropWidth`/`cropHeight` are used. Deviations: updated `commands/items.rs` for the new exhaustive error code, P05 C5(c) to expect the fourth arrival job, and P17 C5(a) render-only fixture to set shape `off`; C5(a) stylesheet test reads the CSS file because Vitest returns an empty string for `?raw` in this config. Optional live C6(a) not run, so `maximumCrop(..., {apply:false})` return shape and side-effect remain for live verification. 14/14 automated rows pass; full Rust suite, frontend suite, clippy, fmt and typecheck green.

- 2026-10-29 (opus, review r1): **CHANGES_REQUESTED.** Gate: tracker `IMPLEMENTED`, checkpoint `0476e66` present, and its 39 paths equal the handoff's declared list. Reviewer re-ran the stamp: 432 Rust and 127 UI tests pass, and clippy, fmt and typecheck are green. Rows C1–C5 hold at their boundaries in the suite. Live row C6(a) was run by the reviewer (Capture One 16.8.6, `session test 1`, `PSM_PROBE_A/session test 12199.ARW`, variant `130`) and **fails**: (1) `crop_aspect_ratios.js` printed the 11 evidence names ✓; (2) `apply_crop.js … 130 Square` printed `{"outcome":"skipped","ratio":null,"width":5981,"height":3987}` on the untouched frame. **F1** (B1, route production): an untouched variant reads `cropAspectRatio()` as `null` in JXA, not `""`, so step (3) skips every untouched frame. **F2** (B1, route production): `maximumCrop(v, {apply:false})` returns the JS **object** `{x:3000, y:1993, width:2981, height:1994}` for a 5981×3987 frame (Capture One's rectangle is `{centerX, centerY, width, height}`; JXA's QuickDraw decoding yields `width − centerX` and `height − centerY`, so the true size is `x + width` × `y + height`), and step (5) would skip even after F1 is fixed. Observed live: `apply:false` does **not** apply (a Square crop stayed 3987×3987 through the call); the read-back after setting `Square` returns `"Square"`; `maximumCrop(v)` restored `[null,5981,3987]`, the exact pre-probe reading. Fix scope: `jxa/apply_crop.js` and the C1 harness in `crates/testkit/tests/p23_shape_on_arrival.rs`; the re-review re-runs C6(a) live. 6 notes in backlog (H7 text in core, the evidence's `""`, the Health page re-queue, and three test notes).

- 2026-10-29 (codex, fix r2): Resolved both production findings within the permitted script and harness files. The untouched guard now accepts `null` and `undefined` alongside `""` and `Unconstrained`; maximum crop size is decoded from the observed object as `x + width` and `y + height`, with the existing numeric/finite validation still throwing for malformed sizes. The C1 Node fixture now uses the live `null` ratio and decoded rectangle; it also retains explicit `""` and `Unconstrained` coverage. Targeted C1(a–c) tests and the closing suite/lint/typecheck stamp are green. No divergence or additional judgment call.
