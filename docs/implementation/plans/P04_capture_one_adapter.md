---
phase: P04_capture_one_adapter
title: Capture One adapter — scripting layer, capture-folder switch, hooks, recipes, tags, camera
difficulty: high
projection: yes
depends_on: [P00_spike, P01_storage_identity]
---
# P04 — Capture One adapter: scripting layer, capture-folder switch, hooks, recipes, tags, camera

## Goal
Implement the editing provider (`EditingProvider`) and the camera provider (`CameraProvider`) for Capture One on
top of a typed scripting layer (`CaptureOneScripting`) with a JXA implementation and the testkit fake: document
inspection (MC-19), per-session capture-folder switch with read-back and revert (MC-18), hook installation and
read-back (MC-15 install half), recipe checks and fixes (MC-16 recipes), RAW → variant resolution (MC-19), the
coalesced colour-tag mirror with read-back (MC-19/M26), render issuing (MC-16 issuing half), camera status and
capture. Adds the durable job runner. **Not in this phase:** consuming hook payloads (P05), render completion,
staleness and the in-flight rule (P06), import of files (P05 calls `import`; the scripting op is written here).

## Read first
master plan §6.1 (integrations crate), §6.4, §6.6 (health codes, tag mapping), §7.5 (`FakeCaptureOneScripting`) ·
book `01`, `14`, `10` (Camera provider, Editing provider, Capture One adapter) · contracts `10A` MC-15 (installation
paragraph), MC-16 (Recipes, Issuing), MC-18, MC-19; `05A` MC-2 (capture root, `_Unassigned`), MC-8 (folder key) ·
evidence `capture_one_16.8.6.md`: VERIFY-2 (lines 39–46: runtime set, `/private` read-back, `""` → null),
VERIFY-3 (65–73: must create folder; failed set reads back `<cwd>/Capture`; `capture` collection keeps the root),
document identity (75–79: `id` = folder path, variant ids small integers, `images whose path`), VERIFY-4 (97–103:
integer tags, write fires adjusted hook), VERIFY-5 (110–121: `process` returns job id, `root folder type` not
settable, `existing files` must be `overwrite`), VERIFY-7 (218–222: `documents.byId`), Transport (212–214).

## Files expected to change
`crates/integrations/src/editing/capture_one/{scripting.rs (trait, DTOs), jxa.rs (impl over OsaRunner), adapter.rs
(EditingProvider impl), mirror.rs (tag job), jxa/{document_info,save,set_captures,get_captures,set_hook,get_hook,
recipe_exists,set_recipe_props,find_image,find_image_in_folder,get_color_tag,set_color_tag,process,capture,import}.js}`
(new), `crates/integrations/src/camera/{mod.rs,via_capture_one.rs}` (new), `crates/infra/src/jobs.rs` (new: durable
runner), `crates/core/src/ports/{editing.rs,camera.rs}` (traits), `crates/core/src/services/{sessions.rs (activate /
deactivate), review.rs (enqueue MirrorTag), health.rs (new items), editing_startup.rs (new)}`,
`crates/core/src/domain/{health.rs,editing.rs}`, `apps/desktop/src-tauri/src/commands/{sessions.rs,camera.rs}`
(`session_activate`, `session_deactivate`, `camera_status`, `camera_retry`, `camera_capture`), bindings,
`crates/testkit/src/{fake_capture_one.rs (new), test_app.rs (providers wired; `fake_c1()` accessor)}`,
`crates/testkit/tests/p04_capture_one_adapter.rs` (new).

## Core mechanisms (unit tests allowed)
- `editing::capture_one::adapter::tag_for(classification) -> i32` (MC-19 mapping) — trivial; a single table test.
- `psm_infra::jobs::backoff(attempts) -> Duration` (1, 2, 4 … capped 60 s).

## Ordered tasks
1. `CaptureOneScripting` trait (ops: `document_info(id)`, `save(id)`, `set_captures(id, path)`, `get_captures(id)`,
   `set_hook(name, path)`, `get_hook(name) -> Option<String>`, `recipe_exists(name)`, `set_recipe_props(name,
   existing_files, output_sub_folder)`, `find_image(id, raw)`, `find_image_in_folder(id, folder, raw)`,
   `get_color_tag(variant)`, `set_color_tag(variant, n)`, `process(variant, recipe) -> JobId`, `capture()`,
   `import(id, paths)`); `JxaScripting` runs one `.js` per op through `OsaRunner` (30 s timeout, serial queue).
2. `FakeCaptureOneScripting` in testkit (in-memory document; faults `fail_next_set_captures`, `readback_garbage`,
   `timeout_next_set_tag`, `drop_tag_writes`, `hooks_readback_null`; `private_prefix: bool` makes read-backs come
   through `/private`; records every op).
3. `CaptureOneAdapter: EditingProvider`: `inspect_document` (kind must be `session` → else health
   `document_not_session`; missing → `document_not_open`), `install_hooks` (set three properties, read back,
   compare by `canon`, else `hooks_not_installed`), `check_recipes` (both names exist → set `overwrite` and
   `_session_manager`; else `recipe_missing{name}`), `set_capture_folder(target)` = MC-18 steps 1–5 with revert,
   `render(raw, purpose)` (resolve variant, `process`, return job id; job-map entry recorded for P06),
   `mirror_tag(raw, classification)` (write, read back, `tag_verification` on mismatch, `last_own_write_at = now`),
   `import(paths)`, `capabilities()`.
4. `psm_infra::jobs::JobRunner`: polls `jobs` (`queued`, `next_attempt_at ≤ now`), marks `running`, calls the
   handler by kind, `done`/`failed` with `attempts += 1` and backoff; at startup every `running` → `queued`
   (MC-11 startup rule). `MirrorTag` handler coalesces per RAW (one queued job per `raw_path`; writes the row's
   **current** classification; copy rows: most recently changed row wins).
5. `SessionsService::activate(id)` → `set_capture_folder(<capture_root>/<folder_key>)` then `settings.active_session_id`;
   `deactivate()` → `_Unassigned`. `EditingStartup::run()` at `App::open`: inspect, install hooks, check recipes,
   re-apply the active session's folder; results into `HealthService`.
6. `CameraViaCaptureOne: CameraProvider`: `status()` from `document_info.camera`; `capture()` refused while health
   has `capture_folder_not_set`; `camera_retry` re-reads status and emits `camera_status_changed`.
7. Commands, bindings, tests.

## Criteria (8 criteria, 21 rows; 2 `[rule]` rows: C7(a), C7(c)). Fixture cells cite the evidence that grounds each Capture One-owned shape.
| Row | Boundary | Input (fake document unless noted) | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | `SessionsService::activate(s)` | capture root `<tmp>/Capture`, item key `K1`, fake `private_prefix = true` (read-back as `/private<path>`: VERIFY-2 l.41–42) | fake op sequence contains `save` before `set_captures`; folder `<tmp>/Capture/K1/` exists on disk; fake `captures == <tmp>/Capture/K1`; health has no `capture_folder_not_set`; `active_session_id = s` | M25 |
| C1(b) | `activate(s)` | fake `fail_next_set_captures` (raises "Cannot set because there is no file at this location": VERIFY-3 l.66) with previous folder `P` | health `capture_folder_not_set`; fake `captures == P` (re-applied through save + set); `camera_status() == Disconnected`-equivalent red state; `camera_capture` → `Err(capture_folder_not_set)` | M25 |
| C1(c) | `activate(s)` | fake `readback_garbage` (read-back returns `<cwd>/Capture`: VERIFY-3 l.67–69) | same outcome as C1(b) | M25 |
| C1(d) | `deactivate()` | an active session | fake `captures == <root>/_Unassigned`; the folder exists | MC-2 (`_Unassigned`), M25 |
| C2(a) | `install_hooks` | fake returns set paths through `/private` (VERIFY-2 l.41) | three hook properties equal the compiled `.scpt` POSIX paths by `canon`; health has no `hooks_not_installed` | M22 (installation) |
| C2(b) | `install_hooks` | fake `hooks_readback_null` (cleared property reads back `null`: VERIFY-2 l.42) | health `hooks_not_installed` | MC-15 |
| C3(a) | `check_recipes` | recipes `PSM Preview`, `PSM Web` exist with `existing files = add suffix` | both now `overwrite` and `output sub folder = _session_manager` in the fake (VERIFY-5 l.116–120 for why) | MC-16 (Recipes) |
| C3(b) | `check_recipes` then `render` | `PSM Web` missing | health `recipe_missing{name: "PSM Web"}`; `render(raw, Export)` → `Err(recipe_missing)`; `render(raw, Preview)` still issues | MC-16 |
| C4(a) | `render(raw, Preview)` | image present in the current collection (`images whose path`: evidence l.78) | fake receives `process(variant at position 1 of that image, "PSM Preview")`; returns the fake's job id (UUID text: VERIFY-5 l.111) | MC-19, MC-16 (Issuing) |
| C4(b) | `render` | image absent from the current collection, present in the folder collection whose `file` is the RAW's folder | found; `process` issued | MC-19 (fallback) |
| C4(c) | `MirrorTag` job for a RAW no collection holds | — | job `failed`-then-retried: `attempts = 1`, `next_attempt_at > now`, status `queued`; health `variant_not_found` | MC-19 (`variant_not_found`, backoff) |
| C5(a) | `classify` ×3 (`main`, `condition`, `main`) then run jobs to idle | one RAW, one variant | fake variant tag `== 4`; `get_color_tag` was called after the last write (read-back) | M26 |
| C5(b) | mirror job | fake `drop_tag_writes` (write ignored, read-back returns old value) | job `failed` with `tag_verification`; retried with backoff | M26 (read-back required) |
| C5(c) | mirror job success | — | `raw_state.last_own_write_at == clock.now_ms()` at write time | MC-19 (`last_own_write`) |
| C5(d) | two rows (copy) on one RAW; row A `main` at t1, row B `condition` at t2 > t1 | jobs run to idle | tag `== 5` (most recently changed row) | MC-19 (copy rows share one variant) |
| C5(e) | mirror mapping | rows classified `deleted`, `unreviewed`, `main`, `condition` (one RAW each) | tags `1`, `0`, `4`, `5` (VERIFY-4 l.97: integer property) | M2, MC-19 |
| C6(a) | mirror job | fake `timeout_next_set_tag` (the write times out but is applied) | after retry the tag is correct **and** the fake received exactly one `set_color_tag` (a read preceded the retry) | MC-19 (timed-out write → read first) |
| C7(a) | `camera_status()` | fake `camera = Some("Sony ZV-E10")` | `Connected("Sony ZV-E10")` | `[rule]` ch10 §Camera provider (A2) |
| C7(b) | `camera_status()` | fake `camera = None` | `Disconnected`; health `camera_disconnected` | MC-18 step 5 (red state) |
| C7(c) | `camera_capture()` | healthy document | fake receives `capture`; `Ok(())` | ch10 `camera.capture` — `[rule]`-class, counted with C7(a) |
| C8(a) | `EditingStartup::run` | document kind `catalog` | health `document_not_session`; no hook install attempted (fake has no `set_hook` op) | MC-19 (kind `session` required) |

## Delegated choices
JXA script layout (one file per op vs. a dispatcher with an op argument — one file per op is the plan's default),
serial-queue implementation (tokio mutex or a dedicated thread), how the job runner is ticked in tests (`app.tick`),
`HealthReport` ordering, DTO names for `DocumentInfo`.

## Manual try-out
1. Set `document_id` in `settings.json` to your Capture One document folder (e.g. `/Users/…/Pictures/session test 1`);
   recipes named `PSM Preview` and `PSM Web` must exist (S1 checklist), both disabled is fine.
2. Launch the app with Capture One open. Health shows: document open (session), hooks installed, recipes found,
   camera (green if the ZV-E10 is tethered).
3. In Capture One, Window → Scripts (or the preferences) shows the three hook properties pointing into
   `~/Library/Application Support/com.loorenz.sessionmanager/hooks/`.
4. Rename a recipe → relaunch → Health shows "recipe missing: …" in red.

## Notes
- **Grounding (rule 17).** Every Capture One-owned shape in a fixture cell is cited to the evidence line that
  observed it; the fake reproduces those shapes and nothing else. Unprobed: `capture done` argv (P05), two open
  documents (VERIFY-7 PARTIAL — the adapter addresses by id, so a second document only risks `capture` targeting).
- The adjusted hook fired by our own tag write (VERIFY-4) is handled in P06 through the own-write window; P04 only
  records `last_own_write_at`.
- Estimated size: ~30 files (15 small JXA scripts), ~2 200 lines.

## Review log
(empty)
