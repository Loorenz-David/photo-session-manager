---
phase: P17_real_run_fixes
title: Real-run fixes — settings rebuild, folder lookup, capture root, off-thread commands, render backoff
difficulty: high
projection: no
depends_on: [P16_operations]
---
# P17 — Real-run fixes: settings rebuild, folder lookup, capture root, off-thread commands, render backoff

## Goal
Fix the backend defects of the owner's first real run (`docs/implementation/real_run_2026-10-26.md`, R1, R2, R4,
R5 backend half, R6): a settings save rebuilds the editing and commerce adapters from the new settings; no JXA
script uses `collections.whose({file})` and every folder lookup goes through one shared collection scan; the
capture root is the session's `Capture` folder derived from the document id, never the favourite with id
`capture`; every Tauri command runs off the main thread and never holds the app lock across provider work;
deferred renders back off and never starve user-initiated Capture One calls. **Not in this phase:** the frontend
pending-state architecture and the local-first search (R3, R5 frontend half: P18), Keychain, the phone remote.

## Read first
`docs/implementation/real_run_2026-10-26.md` (all six findings; the causes are diagnosed) · master plan §6.1 (crates),
§6.4 (ports, `CaptureOneScripting`), §6.7 (commands), §6.8 (settings keys), §7.2 (commands), §7.5 (fakes) · book `01`,
`14`, `05A` MC-2 (capture root and attribution; **its first sentence "the `file` of the document collection whose
`id` is `capture`" is superseded by the evidence correction below — build on the evidence**, see Notes), `10A` MC-16
(recipes, issuing), MC-17 (render trigger), MC-18 (folder switch), MC-19 (document addressing, resolving a RAW,
"jobs stay queued and retry with backoff"), `11` §Layers (commands thin; business logic never depends on Tauri),
`12` §Settings, §Health · evidence `capture_one_16.8.6.md`: "Read-only smoke for P04" (document `id` = the document
folder path; hook read-back shapes), "VERIFY-11 result" (the working collection scan; `whose({file})` → -1725),
"**Correction (orchestrator, 2026-10-26, real run R4)**" (the favourite with id `capture` is the current capture
folder; the root is a separate favourite `Capture` with `file=…/Capture`) · dictionary `capture_one_16.8.6.sdef`
class `collection` (properties `name`, `id`, `kind`, `folder` (text/file), `file` (deprecated), elements `image`).

## Files expected to change
`crates/integrations/src/editing/capture_one/{scripting.rs (DocumentInfo gains `id` and `collections: Vec<CollectionInfo{id, name, folder}>`, loses `capture_root`; per-document ops take `id`), jxa.rs (no cached `document_id`, no own serial lock, prepends `jxa/lib.js` to every script it writes), gate.rs (new: `Lane`, `GatedScripting::pair`), adapter.rs (user and background lanes, root derivation, backoff-free), mod.rs}`,
`crates/integrations/src/editing/capture_one/jxa/{lib.js (new: `canonicalFolder`, `collectionForFolder`, `imageByPath`), document_info.js, process.js, get_color_tag.js, set_color_tag.js, find_image_in_folder.js}`,
`crates/integrations/src/camera/via_capture_one.rs` (user lane), `crates/integrations/src/lib.rs` (`App::handles() -> AppHandles`, `AppHandles::{health_check, settings_get, on_window_focus, …}`, gate wiring),
`crates/core/src/mechanisms/backoff.rs` (new: `render_backoff`), `crates/core/src/services/{previews.rs (per-RAW attempts + backoff; deferred render is `Ok` for the job), editing_startup.rs (`CaptureFolderNotSet` from inspect → health), sessions.rs (unchanged API; uses the derived root)}`,
`apps/desktop/src-tauri/{Cargo.toml (dev-deps: `tauri` feature `test`, `psm-testkit`, `tempfile`), src/state.rs (`AppState(RwLock<App>)`, `handles()`, `mutate()`), src/lib.rs (focus → `on_window_focus`; test `commands_are_async`, test `commands_do_not_wait_on_each_other`), src/commands/{health.rs, items.rs, sessions.rs, photos.rs, export.rs, attributes.rs, publish.rs, settings.rs, camera.rs} (all `pub async fn`, work in `tauri::async_runtime::spawn_blocking`)}`,
`crates/testkit/src/{fake_capture_one.rs (collections model: a `capture` favourite that follows `captures`; faults `delay_document_info_ms`, `fail_process`, `call_delay_ms`; per-document id checks on every per-document op), test_app.rs (capture root `<document>/Capture`; `settings_set` passthrough)}`,
`crates/testkit/tests/p05_capture_arrival.rs` (line 18: root helper follows the new location), `crates/testkit/tests/p17_real_run_fixes.rs` (new),
`apps/desktop/src/services/bindings.ts` (regenerated; command signatures unchanged).

## Core mechanisms (unit tests allowed)
- `psm_core::mechanisms::backoff::render_backoff(attempts: u32) -> Duration` = `min(2^(attempts−1), 60)` s for
  `attempts ≥ 1` (1, 2, 4, 8, 16, 32, 60, 60, …), the same curve as `psm_infra::jobs::backoff`.
- `editing::capture_one::adapter::capture_root(document_id: &str) -> PathBuf` = `canon(<document_id>/Capture)`.

## Ordered tasks
0. **Scripting seam (R1, R2).** `CaptureOneScripting`: `recipe_exists`, `set_recipe_props`, `get_color_tag`,
   `set_color_tag` and `process` take the document `id` as their first argument (the adapter passes its own
   `document_id`); `JxaScripting` keeps no document id and no serial mutex (task 5 serialises). `DocumentInfo`
   becomes `{kind, id, captures, collections: Vec<CollectionInfo{id, name, folder: Option<String>}>, camera}`
   (`document_info.js` returns `d.id()`, `d.captures().toString()`, and for every `c` of `d.collections()`
   `{id: c.id(), name: c.name(), folder: c.folder() === null ? null : c.folder().toString()}` — the dictionary's
   `folder` property; evidence VERIFY-11 result: 84 ms for 11 collections). The fake rejects any `id` other than its
   document's on every per-document op with `Error::DocumentNotOpen` (today only `document_info` checks it), so a
   stale id can never pass a row.
1. **One collection scan (R2).** New `jxa/lib.js` with `canonicalFolder(p)` (strip leading `/private`, trailing `/`),
   `collectionForFolder(doc, folder)` (iterate `doc.collections()`, first whose canonical `folder` equals the
   argument; never touches `currentCollection`) and `imageByPath(doc, raw, folder)` (`doc.images.whose({path})`,
   then the folder collection's `images.whose({path})`). `JxaScripting::new` writes every script as `lib.js` + the
   script source. `process.js`, `get_color_tag.js`, `set_color_tag.js`, `find_image_in_folder.js` call `imageByPath` /
   `collectionForFolder`; **no `whose({file` anywhere** (regex `whose\s*\(\s*\{\s*file`). `document_info.js` uses no
   lookup (it lists collections). No script scans more than once per call.
2. **Capture root (R4).** The adapter derives `DocumentState.capture_root = canon(<document id>/Capture)` and
   requires it to be an existing directory, else `Err(CaptureFolderNotSet)` (health `capture_folder_not_set`,
   detail "Capture root <path> not found"); `EditingStartup` maps that error to the health code and returns it.
   `DocumentState.capture_folder = captures`. The `collections` list is carried for health detail and the live row;
   **the favourite with id `capture` is never read for the root** (evidence Correction). `SessionsService::activate`
   / `deactivate` and `EditingStartup` keep targeting `<root>/<folder_key>` and `<root>/_Unassigned`, so a switch
   A → Unassigned → B or A → A never nests. `ArrivalService::capture_root` caches the derived root as today.
   Testkit: the fake's document folder is `<tmp>/Capture One Session`, the root `<tmp>/Capture One Session/Capture`
   (created by `TestApp`); the fake's `collections` hold `{id:"3", name:"Capture", folder:<root>}` and
   `{id:"capture", name:<basename of captures>, folder:<captures>}`, the second rewritten by every `set_captures`
   (what Capture One 16.8.6 does per the Correction). `p05_capture_arrival.rs:18` follows the new root.
3. **Settings rebuild (R1).** `App::settings_set` already reopens the runtime; with task 0 the rebuilt adapter's
   document id and recipes reach every script. `commerce_from_settings` runs at reopen as today (no override in
   production), so the integration block takes effect without restart. Keep `commerce_override` semantics for tests.
   Build the new runtime **before** stopping the old one's workers (backlog P16 note: a failed build must leave the
   old runtime running).
4. **Off the main thread (R5).** `AppState(pub RwLock<App>)` with two accessors in `state.rs`:
   `handles() -> AppHandles` (read lock held only to clone; `AppHandles` is a `Clone` struct of the `Arc`s plus
   `Settings` and `AppPaths`, built by `App::handles()`) and `mutate(|app| …)` (write lock; only `settings_set` uses
   it). Every command is `pub async fn`, takes `handles()` (or `mutate`) and does its work inside
   `tauri::async_runtime::spawn_blocking` (pattern: `commands/camera.rs`, `export_run`, `session_activate`). No
   command file touches the lock directly. `App::health_check` and `App::settings_get` move to `AppHandles`
   (`App` keeps thin forwarders). The window `Focused(true)` handler calls `handles.on_window_focus()`, which spawns
   a thread for `previews.on_focus()` and returns at once.
5. **Priority gate and backoff (R6).** `gate.rs`: `Lane::{User, Background}`; `GatedScripting::pair(inner) ->
   (user, background)`: one shared gate (a mutex plus a `user_waiting` counter and condvar); a background call
   takes the gate only when no user call is waiting, so a waiting user call is served before the next background
   call. `CaptureOneAdapter::new(user, background, …)`: `inspect_document`, `set_capture_folder`, `install_hooks`,
   `check_recipes`, `import` use the user lane; `render`/`request_render`/`finish_render`/`expire_renders`/
   `mirror_tag` the background lane; `CameraViaCaptureOne` the user lane. `App::build` and `TestApp` wrap the
   scripting exactly once. The adapter's render-registry mutex is not held across a scripting call.
   `PreviewService`: per-RAW `attempts` (starts 0); a deferred render (`request` → provider error) sets
   `attempts += 1` and `retry_after = now + render_backoff(attempts)`; a completed render or a `variant_adjusted`
   event for the RAW resets `attempts` and clears `retry_after`; `on_focus` does not bypass `retry_after`.
   `request_initial` returns `Ok(())` when the render is deferred (the service owns the retry; the `render_preview`
   job completes), so the job runner's own backoff no longer doubles the schedule. The 2 s constant disappears.
6. Regenerate bindings (`cargo test -p psm-desktop export_bindings`); tests; live rows C2(b) and C3(e) against
   the owner's running Capture One, results in the Review log.

## Criteria (5 criteria, 16 rows; 8 `[rule]` rows)
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | `app.settings_set({document_id: D, preview_recipe: "PSM Preview", web_recipe: "PSM Web"})` then `app.health_check()` | `TestApp::new()` (no document configured; fake knows only document D and answers `document_not_open` to any other id on every per-document op) | report: no item `document_not_open`, no `recipe_missing`; items `document`, `hooks`, `recipes` with `ok = true`; the fake's ops after the save contain `recipe_exists:PSM Preview` (the fake would have refused a stale id) | `[rule]` ch12 §Settings, §Health; MC-19 (every call addresses the configured document) |
| C1(b) | `settings_set({preview_recipe: "Nope"})` then `health_check`; then `settings_set({preview_recipe: "PSM Preview"})` then `health_check` | app of (a) | first report: exactly one `recipe_missing` item, detail `Nope`; second report: none | `[rule]` ch12 §Health; MC-16 (recipe names from settings) |
| C1(c) | `settings_set({integration: {kind: "shopify", shop_domain: "test.myshopify.com", api_version: "2026-10", client_id: "id", client_secret: "secret-1234", condition_photos_key: "custom.damage_report"}})` then `settings_set({integration: null})` | `TestApp::new()` (no commerce override, `run_startup = false`) | after the first save `app.commerce.is_some()` and `app.publish.is_some()`, `settings_get().integration.shop_domain == "test.myshopify.com"`; after the second both are `None`; no network call (adapter construction only) | `[rule]` ch12 §Settings; 09A MC-23 (store switch by settings only) |
| C2(a) | the files `JxaScripting::new(&paths)` writes under `<data>/hooks/jxa/` | temp data dir | no file matches `whose\s*\(\s*\{\s*file`; each of `process.js`, `get_color_tag.js`, `set_color_tag.js`, `find_image_in_folder.js` contains `imageByPath(` or `collectionForFolder(`; `function collectionForFolder` is defined exactly once in every written file (the prelude) and in exactly one repo source (`jxa/lib.js`) | `[rule]` 10A MC-19 (folder-collection fallback); evidence VERIFY-11 result |
| C2(b) | live: `osascript -l JavaScript <data>/hooks/jxa/process.js "/Users/davidloorenz/Pictures/session test 1" "/Users/davidloorenz/Pictures/session test 1/Capture/PSM_PROBE_A/session test 12199.ARW" "/Users/davidloorenz/Pictures/session test 1/Capture/PSM_PROBE_A" "<variant>" "PSM Preview"` while Capture One browses `Capture`; `<variant>` = what `find_image_in_folder.js` prints for that frame (`"130"` on 2026-10-26) | frames of 2026-10-26 | prints a quoted batch job id (no `ERROR`, no osascript error); `get_color_tag.js` with the same document, RAW, folder and variant prints `0`; Capture One still browses `Capture` afterwards; the output JPEG's location is noted in the Review log | `[rule]` 10A MC-16 (issuing), MC-19; live — the implementer runs it if its sandbox allows Apple events, else the reviewer |
| C3(a) | `app.editing.inspect_document()` | fake document `<tmp>/Capture One Session` with collections `{id "3", name "Capture", folder <doc>/Capture}` and `{id "capture", folder = current captures}`; `captures` set to `<doc>/Capture/PSM_PROBE_B` beforehand (the fake moves the `capture` favourite with it) | `capture_root == canon(<doc>/Capture)`, `capture_folder == <doc>/Capture/PSM_PROBE_B` | MC-2 (capture root; evidence Correction 2026-10-26), M25 |
| C3(b) | `sessions().activate(A)` twice, then `deactivate()`, then `activate(B)` | items with `folder_key` `A` and `B` (`<root>/A`, `<root>/B` absent beforehand) | the fake's `set_captures` ops are exactly `<root>/A`, `<root>/A`, `<root>/_Unassigned`, `<root>/B` in that order; `get_captures` reads back `<root>/B`; no op path contains `/_Unassigned/_Unassigned` or `/A/` | MC-18, M25 (captures never nest) |
| C3(c) | `deliver_hook(capture_done{argv: [<root>/B/x.ARW]})` and `deliver_hook(capture_done{argv: [<root>/_Unassigned/y.ARW]})` after (b), files written first | — | one `photos` row for `x.ARW` in session B; one for `y.ARW` in session 1; no other rows | MC-2 (attribution by folder), M9 |
| C3(d) | `inspect_document()` then `health_check()` | fake document whose `<doc>/Capture` directory is removed on disk (collections unchanged) | `Err(CaptureFolderNotSet)`; health item `capture_folder_not_set`, `ok = false`, detail contains `<doc>/Capture` | MC-18 step 5 (fail closed), M25 |
| C3(e) | live: `osascript -l JavaScript <data>/hooks/jxa/set_captures.js "/Users/davidloorenz/Pictures/session test 1" "/Users/davidloorenz/Pictures/session test 1/Capture/PSM_PROBE_B"`, then `document_info.js "/Users/davidloorenz/Pictures/session test 1"`, then `set_captures.js … "/Users/davidloorenz/Pictures/session test 1/Capture"` to restore | owner's running Capture One | the JSON has `id` = `/Users/davidloorenz/Pictures/session test 1`, `captures` ending `/Capture/PSM_PROBE_B`, one `collections` entry with `id "capture"` whose folder ends `/PSM_PROBE_B`, and one entry whose folder ends `/session test 1/Capture`; `currentCollection` unchanged | `[rule]` evidence Correction (2026-10-26); MC-2 |
| C4(a) | `apps/desktop/src-tauri/src/commands/*.rs` read by a `psm-desktop` test | — | count of `#[tauri::command]` == count of `pub async fn`; zero `pub fn` commands; no file under `commands/` contains `.0.lock(`, `.0.read(` or `.0.write(` (state access only through `AppState::handles()` / `mutate()` in `state.rs`) | `[rule]` ch11 §Layers (thin commands; no blocking work on the UI thread — real run R5) |
| C4(b) | `health_get` and `sessions_list` invoked through Tauri's mock runtime (`tauri::test::{mock_builder, mock_context, noop_assets, get_ipc_response}`, feature `test`; request shape = `tauri::webview::InvokeRequest{cmd, callback: CallbackFn(0), error: CallbackFn(1), url: "tauri://localhost", body: InvokeBody::Json(args), headers: default, invoke_key: tauri::test::INVOKE_KEY}` — `tauri-2.12.5/src/test/mod.rs` l.260–296) from two threads | `App::open_with_adapter` over a fake whose `document_info` sleeps 1 500 ms (`delay_document_info_ms`); `sessions_list{filter: "open", query: null}` sent 100 ms after `health_get` | `sessions_list` answers `ok` within 500 ms of being sent while `health_get` is still pending; `health_get` later answers `ok` | `[rule]` real run R5 (no command waits on another's provider call); ch11 §Layers |
| C4(c) | `app.handles().on_window_focus()` (what the `Focused(true)` window event calls) | fake `process` sleeping 500 ms (`call_delay_ms` on that op); one stale RAW with no render in flight | returns within 100 ms; the fake receives `process` for the RAW within 2 s | MC-17 (focus renders stale RAWs), real run R5 |
| C5(a) | `tick(1 000)` × 60 after a capture arrival at t = 0 whose render fails | fake fault `fail_process = true` (every `process` → `Err(Provider)`); RAW known, document configured | the fake receives `process` for the RAW at exactly t = 0, 1, 3, 7, 15, 31 s (6 calls in 60 s) and no other; `recipe_exists`/`find_image` ops have the same count (no lookup without an attempt); the `render_preview` job is `done`, not `failed` | MC-19 (retry with backoff), M24; core mechanism `render_backoff` |
| C5(b) | `fail_process = false` at t = 40 s, then `deliver_hook(variant_adjusted{raw})` at t = 40 s, then `tick(1 000)` up to t = 60 s | app of (a), preview delay 15 s | no `process` in (31 s, 55 s); exactly one `process` at the tick reaching 55 s, returning a job id now recorded in flight | M24, MC-17 (adjusted event resets the backoff; the delay rule applies) |
| C5(c) | `sessions().activate(B)` on the test thread while a second thread runs `app.jobs.tick(now)` over 10 queued `mirror_tag` jobs | fake `call_delay_ms = 20` on every op; activate issued 50 ms after the background thread starts | in the fake's op log, between any two consecutive ops of the activate call (`document_info`, `save`, `set_captures:…`, `get_captures`) at most one background op (`find_image:…`, `find_image_in_folder:…`, `get_color_tag:…`, `set_color_tag:…`) appears; activate returns `Ok` within 1 s; every mirror job still completes | MC-19 (one serial queue; user calls never starved — real run R6), M25 |

## Delegated choices
The gate's primitive (mutex + condvar vs. a two-lane channel), whether `AppHandles` is a struct of fields or wraps
`Arc<AppInner>`, the health detail wording for a missing capture root, the name of the thread `on_window_focus`
spawns, how the prelude is joined (`concat!` at compile time vs. at write time), whether `collections` entries
without a folder are kept or dropped, the mock-runtime test's placement (`lib.rs` tests module vs. `tests/`).

## Manual try-out
1. Launch the app with Capture One open on `session test 1`. Settings → enter the session folder and both recipe
   names → Save. Health shows document, hooks, recipes and capture folder green **without a relaunch**.
2. Capture One's capture folder now reads `…/session test 1/Capture/_Unassigned` (not nested). New session for an
   item → the folder becomes `…/Capture/<folder key>`; back to Sessions → `_Unassigned`; open the session again →
   `…/Capture/<folder key>` (never `…/_Unassigned/<folder key>`).
3. Shoot 3 frames in the open session: each appears in the workspace within ~2 s and its adjusted preview arrives a
   few seconds later while Capture One browses any folder.
4. While previews render, type in the sessions search and open a session: the window never freezes.
5. `logs/psm.<today>.log`: a failing render shows attempts 1, 2, 4, 8 … seconds apart, not every 2 s.

## Notes
- **Fold-back owed before launch (orchestrator):** 05A MC-2 "Capture root" sentence (the `capture` favourite) is
  contradicted by the evidence Correction of 2026-10-26; amend it to "`<document id>/Capture` (the session's Capture
  folder; the favourite with id `capture` is the current capture folder)". This plan builds on the evidence; the
  implementer treats the book sentence as superseded and does not block on it.
- The owner's session already holds nested folders (`Capture/_Unassigned/_Unassigned/…`) from the real run; the
  startup scan reads the root and one level only, so frames in deeper folders are not attributed. Leave them.
- `P16`'s backlog note on `health_get` under the lock and the P13 note on `items_search` under the lock are closed by
  task 4; the P16 note on `settings_set` stopping workers before building is closed by task 3.
- Estimated size: ~22 files, ~1 400 lines.

## Review log

- **2026-10-26 implement r1, codex.** Built explicit document-id scripting, one generated JXA folder-scan prelude, the session-root derivation from `<document id>/Capture`, settings rebuild with a failed-build rollback and a shared priority gate across rebuilds, asynchronous Tauri commands using cloned handles, nonblocking focus refresh, and per-RAW exponential render retries. `request_initial` completes its job after a deferred provider render. The render registry releases its mutex before scripting calls. All 16 criterion rows have automated boundary proxies (14 testkit, 2 desktop); core-mechanism tests cover backoff and capture-root derivation. The real Capture One rows C2(b) and C3(e) remain for reviewer live confirmation: `osascript -l JavaScript -e 'Application("Capture One").running()'` exited 1 with `Application can't be found. (-2700)` in this session, so no live script was issued and no capture folder was changed. The generated-script Node proxies for those rows pass. `cargo test --workspace --quiet` (405 Rust tests), clippy, fmt, npm typecheck, and 92 frontend tests pass.
  - **Assumptions:** Kept collection entries with a null `folder` in `DocumentInfo`; the shared scan skips them. The user and background lanes keep one gate across settings rebuilds. A focus event starts a standard Rust thread and returns immediately. The generated prelude is joined to each script at write time. A failed runtime build restores the previous settings file and leaves the old workers running.
  - **Deviations:** Updated `Cargo.lock` for desktop dev dependencies. Adjusted P04 test paths and the P16 folder-lookup test to use the generated script and its `folder` property; those earlier assertions encoded the superseded capture-root and standalone-script shapes. `bindings.ts` was regenerated but had no diff. The live rows were deferred under task 6's sandbox clause.
