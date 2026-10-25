---
phase: P04_capture_one_adapter
title: Capture One adapter — scripting layer, capture-folder switch, hooks, recipes, tags, camera
difficulty: high
projection: done (r0 2026-10-25, amended)
depends_on: [P00_spike, P01_storage_identity]
---
# P04 — Capture One adapter: scripting layer, capture-folder switch, hooks, recipes, tags, camera

> **Amended by the orchestrator 2026-10-25** from the round-0 projection
> (`handoffs/reviewer/handoff_P04_capture_one_adapter_projection_r0.md`, ledger L1–L25, D1–D6). Every amendment is
> answered by the book, the evidence or the code; none changes product behaviour.

## Goal
Implement the editing provider (`EditingProvider`) and the camera provider (`CameraProvider`) for Capture One on
top of a typed scripting layer (`CaptureOneScripting`) with a JXA implementation and the testkit fake: document
inspection (MC-19), per-session capture-folder switch with read-back and revert (MC-18), hook installation and
read-back (MC-15 install half), recipe checks and fixes (MC-16 recipes), RAW → variant resolution (MC-19), the
coalesced colour-tag mirror with read-back (MC-19/M26), render issuing (MC-16 issuing half), camera status and
capture. Adds the durable job runner and a thin `CameraService`. **Not in this phase:** consuming hook payloads
(P05), render completion, the render job map, staleness and the in-flight rule (P06), import of files (P05 calls
`import`; the scripting op is written here).

## Read first
master plan §6.1 (integrations crate), §6.4, §6.6 (health codes, tag mapping), §7.5 (`FakeCaptureOneScripting`) ·
book `01`, `14`, `10` (Camera provider, Editing provider, Capture One adapter) · contracts `10A` MC-15 (installation
paragraph), MC-16 (Recipes, Issuing), MC-18, MC-19; `05A` MC-2 (capture root, `_Unassigned`), MC-8 (folder key);
`07A` MC-11 (startup rule: every `running` job returns to `queued`) · evidence `capture_one_16.8.6.md`:
VERIFY-2 (l.39–46: runtime set, `/private` read-back, `""` → null), VERIFY-3 (l.65–73: must create folder; failed
set reads back `<cwd>/Capture`; `capture` collection keeps the root), document identity (l.75–79: `id` = folder
path, variant ids small integers, `images whose path`), VERIFY-4 (l.97–103: integer tags, write fires adjusted hook;
**mapping confirmed by the owner l.105–106**), VERIFY-5 (l.113–124: `process` returns job id, `root folder type`
not settable, `existing files` must be `overwrite`), VERIFY-11 (l.183–187, unprobed), **VERIFY-12 (l.189–207:
a bare `capture` writes nothing with the owner's AF settings; the autofocus recipe)**, VERIFY-1 result (l.209–214:
capture-done argv; clearing a hook with `""`), Transport (l.233–237), VERIFY-7 (l.239–243: `documents.byId`),
**Read-only smoke for P04 (l.245 to the end: no-camera shape, hook read-back shapes, capture root, variant ids)**.
`docs/reference/capture_one_16.8.6.sdef` l.1864 (`process` returns a job id **or a text starting with `ERROR`**).

## Files expected to change
New:
`crates/integrations/src/editing/capture_one/{scripting.rs (trait, DTOs), jxa.rs (impl over OsaRunner), adapter.rs
(EditingProvider impl), mirror.rs (MirrorTag handler), jxa/{document_info,save,set_captures,get_captures,set_hook,
get_hook,recipe_exists,set_recipe_props,find_image,find_image_in_folder,get_color_tag,set_color_tag,process,capture,
import}.js}`, `crates/integrations/src/camera/{mod.rs,via_capture_one.rs}`, `crates/infra/src/jobs.rs` (durable
runner), `crates/core/src/ports/{editing.rs,camera.rs}` (traits), `crates/core/src/services/{camera.rs
(CameraService), editing_startup.rs}`, `crates/core/src/domain/editing.rs` (provider-neutral types only, see Task 1),
`apps/desktop/src-tauri/src/commands/camera.rs`, `crates/testkit/src/fake_capture_one.rs`,
`crates/testkit/tests/p04_capture_one_adapter.rs`.
Changed:
`crates/core/src/{error.rs (capture_folder_not_set, hooks_not_installed, recipe_missing{name}, tag_verification,
variant_not_found, document_not_open, document_not_session), events.rs (camera_status_changed), ports/mod.rs,
ports/store.rs (jobs enqueue/poll/update, raw_state.last_own_write_at upsert, settings.active_session_id),
services/mod.rs, services/sessions.rs (activate/deactivate), services/review.rs (enqueue MirrorTag),
services/health.rs (holds startup results), domain/mod.rs}`, `crates/infra/src/{lib.rs, sqlite/store.rs}`,
`crates/integrations/{Cargo.toml, src/lib.rs (App::open takes the scripting implementation and the clock),
src/editing/mod.rs, src/editing/capture_one/mod.rs}`, `apps/desktop/src-tauri/src/{lib.rs, state.rs,
commands/mod.rs, commands/health.rs, commands/sessions.rs}`, `apps/desktop/src/services/bindings.ts` (regenerated),
`crates/testkit/src/{lib.rs, test_app.rs (providers wired; `fake_c1()` accessor; capture root in the fake
document — owned here, P05's mention is a no-op)}`.
No migration: every column needed exists (`jobs`, `raw_state.last_own_write_at`, `settings`).

## Core mechanisms (unit tests allowed)
- `editing::capture_one::adapter::tag_for(classification) -> i32` (MC-19 mapping) — one table test.
- `psm_infra::jobs::backoff(n) -> Duration` = `min(2^(n−1), 60)` s for `n ≥ 1`, where `n` is `attempts` **after**
  the increment. Table test n = 1…7 → 1, 2, 4, 8, 16, 32, 60.
- `editing::capture_one::jxa::parse_process_result(&str) -> Result<JobId>`: a text starting with `ERROR` →
  `Err(Provider(text))`, otherwise the job id. Two-case table test.
These live beside their modules (inline `#[cfg(test)]`, P00 `hooks.rs` precedent), not under `core/src/mechanisms/`.

## Ordered tasks
1. **Scripting layer.** `CaptureOneScripting` trait (ops: `document_info(id)`, `save(id)`, `set_captures(id, path)`,
   `get_captures(id)`, `set_hook(name, path)`, `get_hook(name) -> Option<String>`, `recipe_exists(name)`,
   `set_recipe_props(name, existing_files, output_sub_folder)`, `find_image(id, raw)`, `find_image_in_folder(id,
   folder, raw)`, `get_color_tag(variant)`, `set_color_tag(variant, n)`, `process(variant, recipe) -> JobId`,
   `capture()`, `import(id, paths)`). `document_info` returns at least `kind`, `captures`, `capture_root` (the
   `file` of the collection whose `id` is `capture`), `camera: Option<name>` (`currentDocument().camera()` is
   `null` with no camera: smoke l.245ff). `JxaScripting` runs one `.js` per op through `OsaRunner` (30 s timeout,
   serial queue). **Every script first checks `Application("Capture One").running()` and returns a not-running
   error without launching Capture One** (JXA `Application(…)` would otherwise start it). Output: each script's
   `run(argv)` returns `JSON.stringify(result)`; a non-zero exit is `Error::Provider(stderr)`. Hook read-backs are
   `Path` objects: convert with `.toString()`. `process` output goes through `parse_process_result`. **`capture.js`
   implements the VERIFY-12 recipe:** `camera.autofocusing = true`, wait 2 s, require `focusIndicator() ==
   "focused"`, then `capture()`; not focused → `Err(Provider("camera did not focus"))`.
   **H7:** Capture One concepts (document kind `session`/`catalog`, variant ids, recipe `existing files`) stay in
   `integrations`. `crates/core/src/domain/editing.rs` holds provider-neutral types only (e.g.
   `DocumentState{open, usable, capture_root, capture_folder}`); names delegated.
2. **Fake.** `FakeCaptureOneScripting` in testkit: in-memory document, records every op. Native behaviour (no fault
   needed): `set_captures` to a folder that does not exist on disk fails with "Cannot set because there is no file
   at this location" and leaves `captures == <cwd>/Capture` (VERIFY-3), so the create-before-set order is enforced
   by the fake itself. `private_prefix: bool` ⇒ path read-backs return `fs::canonicalize(path)` (the real-path
   shape, `/tmp/x` → `/private/tmp/x`), never a blind string prefix. Faults: `fail_next_set_captures` (one-shot),
   `readback_garbage` (sticky until cleared), `timeout_next_set_tag` (one-shot; the write is applied, the call
   returns `Timeout`), `drop_tag_writes` (sticky), `hooks_readback_null` (sticky). Variants start at tag **3**
   (unmapped). An image may hold several variants; ids are integers not equal to positions. The rest of the fake's
   API is delegated.
3. **Adapter.** `CaptureOneAdapter: EditingProvider`: `inspect_document` (kind must be `session` → else health
   `document_not_session`; missing or Capture One not running → `document_not_open`), `install_hooks` (set three
   properties to the compiled `.scpt` paths, read back, compare by `canon`, else `hooks_not_installed`),
   `check_recipes` (**per recipe**: each existing recipe is set to `existing files = overwrite` and `output sub
   folder = _session_manager`; each missing one adds its own `recipe_missing` item with the name in `detail`),
   `set_capture_folder(target)` = MC-18 steps 1–5 with revert, `render(raw, purpose)` (resolve variant at position
   1, `process`, return the job id; `recipe_missing` for that purpose's recipe → `Err(recipe_missing{name})`; P06
   owns the job map), `mirror_tag`, `import(paths)`, `capabilities()`.
4. **Jobs.** `psm_infra::jobs::JobRunner`: polls `jobs` (`status = queued`, `next_attempt_at ≤ now`), marks
   `running`, calls the handler registered for the kind (handlers are trait objects registered by `App`, because
   `psm-infra` cannot depend on `psm-integrations`). Success → `done`. **Retryable failure → `status = queued`,
   `attempts += 1`, `error = <code>`, `next_attempt_at = now + backoff(attempts)`.** `failed` is terminal and no P04
   kind reaches it. At startup every `running` → `queued` (07A MC-11).
   `MirrorTag`: payload `{photo_id}`. Enqueued **in the same transaction** by `classify` (any target, including
   `unreviewed`), `delete` and `undo`; not by `reorder`, `move` or `copy` (classification unchanged). Coalesce per
   RAW **only into a `queued` row** for the same target: the enqueue overwrites its payload with the new
   `photo_id` and resets `next_attempt_at` to now. A classify while that RAW's job is `running` inserts a new
   `queued` row. The latest-enqueuing row therefore wins for copy rows sharing one variant (no per-row change time
   is needed). The handler writes the mapping of that row's **current** classification, reads back, and requires
   equality (`tag_verification` otherwise). `raw_state.last_own_write_at = now` is recorded **whenever a write is
   issued, whatever its outcome** (upsert the `raw_state` row if absent), so P06 never reads our own write's
   adjusted hook as a retouch. A timed-out write is followed by a read before any retry; mechanism delegated
   (remember `error = timeout` and read first, or always read first and skip the write when the tag matches).
5. **Sessions and startup.** `SessionsService::activate(id)` → `set_capture_folder(<capture_root>/<items.folder_key>)`;
   success → `settings.active_session_id = id`; **failure → `Err(capture_folder_not_set)` and `active_session_id`
   unchanged** (a frame shot meanwhile lands in the previous folder and is attributed to that session by MC-2).
   `deactivate()` → `set_capture_folder(<capture_root>/_Unassigned)` and `active_session_id = null`.
   `EditingStartup::run()`: inspect, install hooks, check recipes, then re-apply the active session's folder, or
   `_Unassigned` when no session is active (MC-2); results into `HealthService`, which now holds the startup state
   (interior mutability or App-owned: delegated). **Startup runs off the main/IPC thread**; commands that reach the
   provider are `async` or do not hold the `App` lock across an osascript call (backlog P00 → P04).
   `App::open` takes the scripting implementation and the clock (production `JxaScripting` + `SystemClock`; TestApp
   the fake + `ManualClock`) so P00–P03 tests never reach osascript. TestApp can configure the fake and
   `document_id` before startup runs.
6. **Camera.** `CameraViaCaptureOne: CameraProvider` (`status()` from `document_info.camera`, `capture()`).
   `CameraService{status, retry, capture}` in core over `CameraProvider` + health (H8: the rule lives here, not in a
   command). **While health has `capture_folder_not_set`, `status()` = `Disconnected`** (no separate item added);
   otherwise it comes from the document's camera. `capture()` → `Err(capture_folder_not_set)` while that item is
   present. `retry` re-reads status and emits `camera_status_changed`. Commands stay thin.
7. Commands, regenerated bindings, tests.
8. **Optional live smoke** (read-only plus one round trip) against the owner's open `session test 1`:
   `document_info`, `get_captures`, `get_hook`, `recipe_exists`, `find_image`, and one captures set → read-back →
   restore. Results into the Review log. If the Codex sandbox refuses Apple Events, record UNPROBED (P00
   precedent); the orchestrator has already run the read-only part (evidence, smoke section).

**Health "has X"** ⇔ the report contains an item with `code == X` and `ok == false`; **"has no X"** ⇔ no such
item. Whether positive items are listed is delegated (the Manual try-out shows them).

## Criteria (8 criteria, 21 rows; 1 `[rule]` row: C7(a)). Fixture cells cite the evidence that grounds each Capture One-owned shape.
| Row | Boundary | Input (fake document unless noted) | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | `SessionsService::activate(s)` | capture root `<tmp>/Capture`, item key `K1`, fake `private_prefix = true` (real-path read-back: VERIFY-2 l.41–42) | `Ok`; fake op sequence contains `save` before `set_captures`; folder `<tmp>/Capture/K1/` exists on disk; fake `captures == <tmp>/Capture/K1` (by `canon`); health has no `capture_folder_not_set`; `active_session_id = s` | M25 |
| C1(b) | `activate(s)` | previous folder `P`, active session `r`; fake `fail_next_set_captures` ("Cannot set because there is no file at this location": VERIFY-3 l.66) | `Err(capture_folder_not_set)`; health has `capture_folder_not_set`; fake `captures == P` (re-applied through save + set); `active_session_id == r`; `CameraService::status() == Disconnected` even with fake `camera = Some("Sony ZV-E10")`; `CameraService::capture()` → `Err(capture_folder_not_set)` and the fake received no `capture` | M25 |
| C1(c) | `activate(s)` | fake `readback_garbage` (read-back `<cwd>/Capture`: VERIFY-3 l.67–69; synthetic pairing with a successful set, isolates step 4) | same outcome as C1(b) | M25 |
| C1(d) | `deactivate()` | an active session | `Ok`; fake `captures == <root>/_Unassigned`; the folder exists; `active_session_id` is null | MC-2 (`_Unassigned`), M25 |
| C2(a) | `install_hooks` | fake read-backs through `fs::canonicalize` (VERIFY-2 l.41; smoke) | three hook properties equal the compiled `.scpt` paths by `canon`; health has no `hooks_not_installed` | MC-15 (installation) |
| C2(b) | `install_hooks` | fake `hooks_readback_null` (cleared property reads back `null`: VERIFY-2 l.42) | health has `hooks_not_installed` | MC-15 |
| C3(a) | `check_recipes` | recipes `PSM Preview`, `PSM Web` exist with `existing files = add suffix` | both now `overwrite` and `output sub folder = _session_manager` in the fake (VERIFY-5 l.118–123 for why); health has no `recipe_missing` | MC-16 (Recipes) |
| C3(b) | `check_recipes` then `render` | `PSM Preview` exists (`add suffix`), `PSM Web` missing | `PSM Preview` is now `overwrite` / `_session_manager`; exactly one `recipe_missing` item, `detail == "PSM Web"`; `render(raw, Export)` → `Err(recipe_missing{name: "PSM Web"})`; `render(raw, Preview)` returns the fake's job id | MC-16 |
| C4(a) | `render(raw, Preview)` | the image is **only** in the current collection (`images whose path`: l.78); it has two variants, position 1 has id `"7"`, position 2 id `"3"` | fake receives exactly `process("7", "PSM Preview")`; `render` returns the fake's job id (UUID text: VERIFY-5 l.115) | MC-19, MC-16 (Issuing) |
| C4(b) | `render(raw, Preview)` | image absent from the current collection, present in the folder collection whose `file` is the RAW's folder, two variants as in C4(a) | fake receives exactly `process(<position-1 variant of the folder-collection image>, "PSM Preview")`; `render` returns its job id | MC-19 (fallback; VERIFY-11 unprobed live) |
| C4(c) | `MirrorTag` job for a RAW no collection holds | clock at `now` | job `status = queued`, `attempts = 1`, `error = "variant_not_found"`, `next_attempt_at == now + 1000`; health has `variant_not_found` | MC-19 (`variant_not_found`, backoff) |
| C5(a) | `classify` ×3 (`condition`, `deleted`, `main`) then run jobs to idle | one RAW, one variant (starts at tag 3) | fake variant tag `== 4`; the fake received **exactly one** `set_color_tag` for that variant; `get_color_tag` was called after it (read-back) | M26 |
| C5(b) | mirror job | fake `drop_tag_writes` (write ignored, read-back returns the old value); clock at `now` | job `status = queued`, `attempts = 1`, `error = "tag_verification"`, `next_attempt_at == now + 1000` | M26 (read-back required) |
| C5(c) | mirror job success | no `raw_state` row before | `raw_state.last_own_write_at == clock.now_ms()` at write time | MC-19 (`last_own_write`) |
| C5(d) | two rows (copy) on one RAW: classify copy row B `condition` at t1, then original row A `main` at t2 > t1 | jobs run to idle | tag `== 4` (latest-enqueuing row wins, though B has the higher id) | MC-19 (copy rows share one variant) |
| C5(e) | mirror mapping | four RAWs (variants start at tag 3): rows classified `deleted`; `main` then `unreviewed`; `main`; `condition` | tags `1`, `0`, `4`, `5` (VERIFY-4 l.97; owner-confirmed l.105–106) | M2, MC-19 |
| C6(a) | mirror job | fake `timeout_next_set_tag` (the write times out but is applied) | after retry the tag is correct **and** the fake received exactly one `set_color_tag` (a read preceded the retry); `last_own_write_at` was recorded for the timed-out write | MC-19 (timed-out write → read first) |
| C7(a) | `CameraService::status()` | fake `camera = Some("Sony ZV-E10")` (the real name string observed 13:20, evidence VERIFY-12/smoke), healthy folder | `Connected("Sony ZV-E10")` | `[rule]` ch10 §Camera provider (A2) — recorded exception to master plan §3 |
| C7(b) | `CameraService::status()` | fake `camera = None` (`camera()` is `null` without a camera: smoke) | `Disconnected`; health has `camera_disconnected` | MC-18 step 5 (red state) |
| C7(c) | `CameraService::capture()` | healthy document, capture folder read back correctly | fake receives `capture`; `Ok(())` | M25 (capture allowed once the folder reads back; C1(b) is the blocked half) |
| C8(a) | `EditingStartup::run` | fake document kind `catalog`; `document_id` matches the fake's id | health has `document_not_session`; the fake received no `set_hook` | MC-19 (kind `session` required) |

## Delegated choices
JXA script layout (one file per op vs. a dispatcher; one file per op is the default); serial-queue implementation
(tokio mutex or a dedicated thread); how the job runner is ticked in tests (`app.tick`); `HealthReport` ordering and
whether positive items are listed; where health state is held; DTO and provider-neutral type names; the
`App::open` / TestApp provider-injection API shape; how startup is moved off the IPC thread; the timed-out-write
read-first mechanism (Task 4); whether health `variant_not_found` clears when the RAW is later found (smallest
option consistent with MC-19). Each choice is recorded under **Assumptions**.

## Manual try-out
1. Set `document_id` in `settings.json` to your Capture One document folder (e.g. `/Users/…/Pictures/session test 1`);
   recipes named `PSM Preview` and `PSM Web` must exist (S1 checklist), both disabled is fine.
2. Launch the app with Capture One open. Health shows: document open (session), hooks installed, recipes found,
   camera (green if the ZV-E10 is tethered, red "disconnected" if not). Quit Capture One and relaunch the app:
   Capture One is **not** started; Health shows the document as not open.
3. Check that the hooks are installed:
   `osascript -l JavaScript -e 'Application("Capture One").captureDoneScript().toString()'` prints a path inside
   `~/Library/Application Support/com.loorenz.sessionmanager/hooks/`.
4. Rename a recipe → relaunch → Health shows "recipe missing: …" in red.

## Notes
- **Grounding (rule 17).** Every Capture One-owned shape in a fixture cell is cited to the evidence line that
  observed it; the fake reproduces those shapes and nothing else. Unprobed: VERIFY-11 (folder-collection fallback
  while Capture One browses another folder), two open documents (VERIFY-7 PARTIAL: the adapter addresses by id, so a
  second document only risks `capture` targeting). Probed since the plan was written: capture-done argv (VERIFY-1
  result), scripted capture recipe (VERIFY-12), no-camera shape and hook read-back shapes (smoke).
- The adjusted hook fired by our own tag write (VERIFY-4) is handled in P06 through the own-write window; P04 only
  records `last_own_write_at`.
- Camera currently **away** (owner, 2026-10-25); live capture is not needed by any row.
- Estimated size: ~35 files (15 small JXA scripts), ~2 400 lines.

## Review log
- 2026-10-25 projection r0 (Opus): AMENDMENTS_REQUIRED, 25 ledger rows + 6 decidability amendments, 0 owner cards.
- 2026-10-25 orchestrator: all ledger rows applied as recommended (L1 camera red state = `Disconnected`; L2
  `CameraService`; L3 activate/deactivate outcomes; L4 health "has"; L5/L6 job states and backoff; L7 coalescing by
  latest enqueue, no migration; L8 enqueue on classify/delete/undo; L9 coalesce only into `queued`; L10 own-write
  recorded on issue; L11 VERIFY-12 capture recipe; L12 `ERROR` envelope; L13 no launch; L14 off-IPC startup; L15
  DocumentInfo fields + H7; L16 provider injection; L17 per-recipe; L18 `_Unassigned` at startup; L19 job map to
  P06; L20 fake fidelity; L21 optional smoke (read-only part already run by the orchestrator, evidence); L22
  perimeter; L23 delegations; L24 C7(c) retraced to M25, C7(a) kept `[rule]` as a recorded exception; L25 line
  numbers) and D1–D6 in the rows. Relaunched without a second projection.
- 2026-10-25 19:30 orchestrator: implement r1 interrupted by the Codex usage limit (resets 19:58), no handoff. Its
  partial work is left **uncommitted in the working tree** (≈38 paths: scripting layer, JXA scripts, fake, jobs,
  camera service, tests file started). The next implementer continues from it: review it against this plan, keep
  what is right, finish, and checkpoint as usual. Archived and relaunched at 20:00.
