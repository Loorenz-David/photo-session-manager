---
phase: P49_capture_one_alongside
title: Capture One alongside — the current-collection rule (switch on every arrival while Capture One shows another collection, and on every activation; R46), waiting for the variant before render and style, the style applied once per RAW by script, and the Import feature fixed (the file is copied into the session's capture folder; Capture One's import is retired)
difficulty: high
projection: no
depends_on: [P48_camera_ownership]
---
# P49 — Capture One alongside the direct adapter (10E §Capture One alongside, 10A MC-18 step 6, R-I35, R46)

## Goal
With the direct adapter Capture One holds no tether, so it sees a new file only when its **current collection** is the folder the file landed in
(V1): the app sets the current collection to the active session's folder on every activation (MC-18 step 6) and **on every arrival while Capture One
shows another collection, whether or not Capture One is the frontmost app** (owner card CI1 answered "always jump", R46 at `e0ce354`). Renders and the
style **wait for the variant** (R-I35: `find_image` every 1 s for 15 s, then the normal render backoff, for as long as the photo exists and is stale;
`variant_not_found` never ends the retries and never marks a generation failed; a collection switch triggers an immediate retry). Next Capture
Adjustments do not apply to such files (V2), so the configured style is applied by script to each new variant, **once per RAW**, never after a restart or
on a retouched variant. The Import feature, broken in Capture One 16.8.6 (`import.js` -1700; and `import` copies files into the session root), is fixed
the way the collection rule allows: the app copies the imported file into the active session's capture folder and lets arrival and the collection rule
do the rest; Capture One's `import` is retired.
**Not in this phase:** the UI (P51), the Settings style field (exists), hardware (P52 row "New photo while Capture One shows another collection").

## Read first
Book: `10E_direct_camera_adapter.md` §Capture One alongside (whole incl. the CI1 answer and **R-I35**), §Shape (ArrivalService → Capture One scan + style)
· `10A_capture_one_contracts.md` MC-18 step 6 (R46), MC-19 (resolving a RAW: "`find_image` searches only the current collection"), MC-16, MC-17 ·
`10C_capture_one_setup_by_the_app.md` MC-26 step 5 (next-capture style; "with the direct camera adapter the app applies the style to each arriving variant
by script") · `06_photo_review_and_ordering.md` §Import · `05A_mechanism_contracts.md` MC-2 (import path; row creation) · `15_decisions.md` R46 ·
`16_changelog.md` entry 33 · `14_measurement_ledger.md` M24, M9 · evidence `sony_camera_lab.md` **§V1–V5** and the side finding (`import.js`) ·
`docs/implementation/backlog.md:257` (the import note) · master plan §6.4 (scripting ops gained/lost), §6.5 (migration 007), §6.6 (`editor_not_following`),
**§6.11** (Constants `VARIANT_WAIT_*`; the follow service), §7.5 (fake ops).
Code at `41bef14` (+ P42–P48 names):
- `crates/core/src/services/previews.rs:91–127` (`request`: a provider error → `retry_after` with `render_backoff(attempts)` — **V4 said renders stopped
  retrying: task 0 reproduces it**), `:224–246` (`tick`/`expire_renders` → `failed_generation` only on timeout), `:247–287` (`schedule_due`),
  `crates/core/src/mechanisms/backoff.rs:4–7` (`render_backoff`: 1, 2, 4, … 60 s), `crates/integrations/src/editing/capture_one/adapter.rs:64–75`
  (`variant_background`: `find_image` then `find_image_in_folder` → `VariantNotFound`), `:219–283` (`request_render`).
- `crates/core/src/services/sessions.rs:48–77` (`activate`, P48 version), `crates/core/src/services/editing_startup.rs`, `crates/core/src/services/photos.rs:
  21–26` (`import`: `arrival.import` then `editing.import` — the second call goes), `crates/core/src/services/arrival.rs:233–242` (`import`), `:243–284` (`accept`).
- `crates/integrations/src/editing/capture_one/{scripting.rs:68–87, jxa.rs, jxa/import.js, jxa/find_image.js, provision.rs (step 5 style)}`, `crates/core/src/
  ports/editing.rs:33–52`; `crates/infra/src/sqlite/migrations/002_domain.sql` (`raw_state`).
- `crates/testkit/src/fake_capture_one.rs:468–485` (`find_image`, `images`), `:168–183` (`add_image`), `:553–561` (`import` — removed), `TestApp::complete_render`.
- Dictionary `docs/reference/capture_one_16.8.6.sdef:388` (document `current collection`, settable), `:2203` (`apply style` on a layer), `:365` ("The images
  in the current collection"), `:2164` (`synchronize`: refuses session favorites, V1 — not used).
- **Tests that change their expectation:** `crates/testkit/tests/p05_capture_arrival.rs` row `c8a_import_active_and_provider_called` (asserts `import:1` in the
  fake's ops — becomes: the copy exists in the session folder and no `import` op). No other test changes.

## Files expected to change
Integrations (new): `crates/integrations/src/editing/capture_one/follow.rs` (`CaptureOneFollow`), `crates/integrations/src/editing/capture_one/jxa/
{set_current_collection.js, apply_style.js}`. Integrations (removed): `crates/integrations/src/editing/capture_one/jxa/import.js`. Integrations (changed):
`crates/integrations/src/editing/capture_one/{scripting.rs, jxa.rs, adapter.rs, gate.rs, mod.rs}` (`SCRIPTS` 20 → 21), `crates/integrations/src/lib.rs`
(follow service wired; `EditingStartup`/take-back step 5 set the collection), `crates/integrations/src/camera/direct/take_back.rs`.
Core (new): `crates/infra/src/sqlite/migrations/007_style_applied.sql`. Core (changed): `crates/core/src/ports/{editing.rs, store.rs, camera.rs (ArrivalFollow)}`,
`crates/core/src/services/{previews.rs, sessions.rs, editing_startup.rs, photos.rs, arrival.rs}`, `crates/core/src/domain/editing.rs` (`StyleOutcome` reuse).
Infra (changed): `crates/infra/src/sqlite/store.rs` (`style_applied_at` read/mark).
Testkit (changed): `crates/testkit/src/fake_capture_one.rs` (`current_collection`, `set_current_collection`, `apply_style`, `fail_set_current_collection`;
`import` removed; `find_image` honours `current_collection`), `crates/testkit/src/test_app.rs` (`arrival_photo(s, name)` = a ledger delivery + watcher).
Tests (new): `crates/testkit/tests/p49_capture_one_alongside.rs`. Tests (changed): `crates/testkit/tests/p05_capture_arrival.rs` (one row).
Estimated: 4 new paths, 1 removed, 17 changed, 1 new test, 1 adjusted test — 24 paths.

## Core mechanisms (unit tests allowed)
None (every row is a behaviour row at a service, store or fake-adapter boundary).

## The contract
- **The current-collection rule (R46):** `CaptureOneFollow` switches Capture One's current collection to the arriving photo's session folder **on every
  arrival whose folder differs from the collection the app last set**, regardless of which app is frontmost; `SessionsService::activate`/`deactivate` and
  `EditingStartup::run` switch it on every activation (MC-18 step 6). No policy, no frontmost check, no deferral.
- **Scripting:** `set_current_collection(id, folder)` (sets the document's `current collection` to the folder collection whose `file` is `folder`
  (`collections.whose({file: Path(folder)})`, the P04 VERIFY-11 pattern), then reads `current collection.file` back and compares by `canon`; mismatch →
  `Err(Provider("current collection not set"))`), `apply_style(id, variant, name)` (`apply style` on the variant's first layer, read back the layer's
  `styles` must contain `name`). `GatedScripting` gates **neither** (they presuppose a running Capture One and are called only after a `running()` check
  inside the follow service; a script against a not-running app would launch it — the follow service therefore checks `editor_info().running` first and
  skips otherwise; **this is why P48's gate list need not grow**, and the reviewer checks it).
- **`EditingProvider`:** `set_current_collection(folder: &str) -> Result<()>`, `apply_style(raw, name) -> Result<StyleOutcome>` (`variant_background` →
  `apply_style`; `VariantNotFound` propagates), `find_variant(raw) -> Result<bool>`; `import` removed.
- **MC-18 step 6:** `SessionsService::activate`/`deactivate` and `EditingStartup::run` call `set_current_collection(<target folder>)` after a successful
  `set_capture_folder`; an error → `editor_not_following` (direct adapter: no error returned; Capture One camera adapter: the error is returned as the
  folder's would be). The take-back's step 5 inherits it through `EditingStartup::run`.
- **`CaptureOneFollow`** (integrations; implements the new core port `ArrivalFollow { fn photo_arrived(&self, session_id: i64, raw: &str, folder: &Path) }`,
  called by `ArrivalService::accept` after every **inserted** row whose source is `Capture` from a delivery (rule 0) or `Import`): state `{ current:
  Option<PathBuf> (the last folder the app set), waiting: HashMap<raw, WaitState { since, last_poll }> }`. On arrival: if `editor_info().running` and
  `folder != current` → `set_current_collection(folder)` now (`current = folder`; failure → `editor_not_following`), then `previews.retry_now(every waiting
  raw)`; `waiting.insert(raw)`. `tick(now)` (from the App's background loop; in tests `TestApp::tick`): for each waiting raw whose `last_poll` is ≥
  `VARIANT_WAIT_POLL_MS` ago and `since + VARIANT_WAIT_MAX_MS ≥ now`: `editing.find_variant(raw)` → found → `apply_style_once(raw)` then
  `previews.retry_now(raw)`, remove from `waiting`; not found → poll again; past 15 s → remove (the preview service's backoff takes over). `apply_style_once
  (raw)`: `style_name` empty → nothing; `store.style_applied_at(raw).is_some()` → nothing; else `editing.apply_style(raw, name)` → `Applied` →
  `store.mark_style_applied(raw, now)`; `NotListed` → health `style_not_found` (the P24 text); `Err` → logged, not marked (retried at the next found).
- **Previews (R-I35):** `PreviewService::request`'s error path: `VariantNotFound` → `retry_after = now + render_backoff(attempts)` as today **but** attempts
  for `VariantNotFound` start counting only after `VARIANT_WAIT_MAX_MS` from the raw's first request (before that the follow service's `retry_now` drives
  the retries at 1 s), `failed_generation` is **never** set for `VariantNotFound` (only `expire_renders` sets it, unchanged); `retry_now(raw)` removes the
  raw's `retry_after` entry and runs `schedule_due(false)`. **Task 0 (before any change):** write the C4(c) test against the code at `41bef14` with the fake
  (`find_image` none ×N, then `add_image`) and record in the Review log what the current code does (V4 saw no recovery: name the path — e.g. a
  `render_in_flight` left behind, or `retry_after` never cleared), then implement.
- **Import (planner reading, Notes (1)):** `PhotosService::import(paths)`: target = `<capture root>/<folder_key of the active session | _Unassigned>/`;
  for each path: `dest = target/<basename>`, `__dupN` on collision (never overwrite: copy to `dest.tmp` then `rename_exclusive` into place), the original
  untouched; then `arrival.import(dest)` (source `Import`, attribution as today); `editing.import` is gone. The copy reaches Capture One through the follow
  service like a delivery.

## Ordered tasks
0. **V4 reproduction** (above) — recorded in the Review log with the code path named. 1. Registry (§6.11, §6.4): scripting ops, provider methods,
`ArrivalFollow`, migration 007, store methods. 2. Scripts + `jxa.rs` + fake ops (incl. `find_image` honouring `current_collection`). 3. `follow.rs`.
4. `previews.rs` (R-I35), `sessions.rs`/`editing_startup.rs` (step 6), `photos.rs`/`arrival.rs` (import). 5. Wiring in `lib.rs` (the follow service is per
`App`, holds `editing`, `store`, `previews`, `health`, `style_name`); `ArrivalService::new` gains `follow: Option<Arc<dyn ArrivalFollow>>`. 6. Tests; the p05
row; suite, clippy, fmt, typecheck, `npm test`.

## Criteria (5 criteria, 19 rows; 0 `[rule]` rows)
Fixtures: `a = TestApp::with_camera_script(ready_wifi)`; `F` the fake Capture One (`current_collection` default = the capture root); `arrive(s, name)` =
`a.arrival_photo(s, name)` (a wireless delivery through the ledger + watcher → one photo row; returns `raw`); `ops()`; `req()` = `F.state.issued` (render
requests); `style` = `a.settings_set({"style_name": "Bright Contrast"})` with `F.styles = ["Bright Contrast"]`.

| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | activation sets the current collection | `activate(s)` | `ops()` ends `set_captures:<root>/<key(s)>`, `set_current_collection:<root>/<key(s)>`; `F.current_collection == that folder` | 10A MC-18 step 6; R46; V1 |
| C1(b) | failure is a health item, not a block (direct adapter) | `F.fail_set_current_collection = true`; `activate(s)` | `Ok`; `active_session_id == s`; `H.has("editor_not_following")`; next `a.app.startup()` (fault cleared) sets it and clears the item | R-I33; 10A MC-18 step 5/6 |
| C1(c) | startup and take-back step 5 set it | `a.app.startup()` with `s` active; P48's take-back fixture | both leave `set_current_collection:<key(s)>` in `ops()` | MC-18 step 6; R-I32 step 5 ("run 10C's ensure … then run MC-18") |
| C2(a) | arrival switches while Capture One shows another collection | `F.current_collection = <root>/Other`; `arrive(s, "DSC1.ARW")` | `ops()` has `set_current_collection:<root>/<key(s)>` within the same tick | R46 ("on every arrival while it shows another collection, even while the photographer works in Capture One"); V1 |
| C2(b) | arrival into the current collection | `F.current_collection = <root>/<key(s)>`; `arrive(s, ..)` | no `set_current_collection` op | MC-18 step 6 (only while Capture One shows another collection) |
| C2(c) | every arrival, not only the first | after C2(a): the test sets `F.current_collection = Other` again (the photographer browsed away); `arrive(s, "DSC1b.ARW")` | a second `set_current_collection:<key(s)>` op | R46 ("on every arrival") |
| C2(d) | Capture One not running: no script, no launch | `F.running = false`; `arrive(s, ..)` | no `set_current_collection`, `find_image`, `launch` or `apply_style` op; the photo row exists | M40 (nothing launches Capture One but the gate's own paths); MC-18 step 6 |
| C3(a) | variant polled every second, no render before it is found | `arrive(s, "DSC2.ARW")` with `F.images` lacking the raw; `a.tick(1_000)` ×3 | `ops()` has exactly 3 `find_image:<raw>` (plus the arrival's own); `req()` empty; `a.app.previews.failed_generation_for(raw) == None` | R-I35 ("`find_image` is polled every 1 s for up to 15 s"); V4 |
| C3(b) | found → style then render | after C3(a): `F.add_image(raw, ["v1"])`; `style`; `a.tick(1_000)` | `ops()` has `apply_style:v1:Bright Contrast` **before** the `process` op; `req()` has one request for `v1` | R-I35; V2 ("`apply style` by script on the variant's layer works") |
| C3(c) | after 15 s the normal backoff; `variant_not_found` never fails the generation | `arrive`; `a.tick(1_000)` ×16; then `a.tick(1_000)` ×30 with `F.images` still empty; then `F.add_image`; `a.tick(1_000)` ×2 | the follow service's polls stop after the 15th; the preview path's `find_image` ops then come 1, 2, 4, 8, 16 s apart (`render_backoff`); `failed_generation_for(raw) == None` throughout; after `add_image` a render is issued | R-I35 ("then the normal render backoff applies … `variant_not_found` never ends the retries and never marks a generation failed"); V4 |
| C3(d) | a collection switch retries at once | `arrive(s, "DSC3.ARW")`; `F.images` empty; `a.tick(20_000)` (so `retry_after` is far ahead); `F.add_image(raw)` under folder `<key(s)>`; `F.current_collection = Other`; `activate(s)` (switches the collection) | a render is requested in the same tick as the switch, without waiting for `retry_after` | R-I35 ("a current-collection switch triggers an immediate retry") |
| C4(a) | style once per RAW | `style`; `arrive(s, "DSC4.ARW")` with the image present; `a.tick(1_000)`; then `F.state.ops.clear()`; `a.app.previews.on_variant_adjusted("v", raw)`; `a.tick(16_000)` (a re-render) | `apply_style` appears exactly once overall; `store.style_applied_at(raw).is_some()`; the re-render has no `apply_style` | R-I35 ("The style is applied at most once per RAW … and never again … on a retouched variant") |
| C4(b) | not again after a restart | after C4(a): `a.kill_and_restart()`; `a.app.startup()`; `a.tick(2_000)` | no `apply_style` op | R-I35 ("recorded in SQLite … never again after a restart") |
| C4(c) | style not listed | `a.settings_set({"style_name": "Nope"})`; `F.styles = ["Bright Contrast"]`; `arrive(..)`; tick | no `apply_style`; `H.has("style_not_found")` with `Style 'Nope' not found in Capture One`; the render is still requested | 10C MC-26 step 5 (health text); R-I35 |
| C4(d) | empty style | `style_name = ""`; `arrive`; tick | no `apply_style`; no health item | 10C MC-26 ("Empty `style_name` → … left as the photographer set them") |
| C4(e) | the style goes to the right variant | two arrivals `DSC5.ARW` (`v5`) and `DSC6.ARW` (`v6`) | `apply_style:v5:…` and `apply_style:v6:…`, each once | R-I35 |
| C5(a) | import copies into the session folder | `activate(s)`; a file `<temp>/orig/IMG.ARW` (bytes `x`); `a.photos().import([path])` | `root/<key(s)>/IMG.ARW` exists with `x`; the original still exists with `x`; the photo row's `raw_path == canon(root/<key(s)>/IMG.ARW)`, `source == Import`, session `s`; `ops()` has no `import` | 06 §Import ("follow the same path as captures … so adjustment and export work identically"); V1 (`import` copies into the session root, unsuitable); H2 (the original is untouched) |
| C5(b) | collision on import | import the same name again with bytes `y` | `root/<key(s)>/IMG__dup1.ARW` with `y`; `IMG.ARW` still `x`; two photo rows | MC-2 (one row per file); the `__dupN` rule of R-I27 reused |
| C5(c) | no active session → `_Unassigned`; Capture One follows | `deactivate()`; `F.current_collection = Other`; import → `root/_Unassigned/IMG2.ARW`, row in session 1, `set_current_collection:<root>/_Unassigned` in `ops()` | as stated | 06 §Import; MC-2 (`_Unassigned`); MC-18 step 6; R46 |

## Delegated choices
The exact JXA for `current collection` assignment (`doc.currentCollection = col` vs `.set(col)` — the implementer verifies against the sdef and records
which); how `TestApp::arrival_photo` drives the delivery (recommended: `a.ledger().delivery(..)` + `watcher_event` + `tick`); the `.tmp` naming of the
import copy; whether `find_variant` reuses `variant_background` directly (recommended).

## Manual try-out
(Capture One needed, camera not.) 1. Open a session in the app and look at Capture One: its browser shows that session's folder. 2. Drag a RAW file from
Finder onto the workspace: it appears in the app **and** in Capture One's browser within a few seconds, the original file stays where it was, and the
adjusted preview arrives. 3. If a style is set in Settings → Capture One, the new variant carries it (Capture One's Styles panel). 4. Browse another folder
in Capture One and import another file: Capture One jumps back to the session folder at once, even while you are working in it (your choice, R46).

## Notes
- **Hard rules (owner):** never commit a Sony SDK file, the lab's `sdk_shim` headers, a `.env`, a credential or an SSH fingerprint; no Python; never change the
  camera's network/FTP settings, delete a camera file (import **copies**; the original is never moved or deleted) or kill any process; live view out of scope;
  the suite never needs the camera. No `implementer: fable`.
- **Planner readings (reversible; for the narrative):** (1) **Import now copies the file into the session's capture folder** (`Capture/<folder key>/`, a
  `__dupN` suffix on a name clash) and the app's photo is that copy; the original file is untouched. Reason: Capture One's own `import` command fails in
  16.8.6 and, when it works, copies the file into the session root at a path the app does not track (V1), so the previous design could never show the
  adjusted preview. To change: "keep importing by Capture One" is not possible; "reference the original in place" means Capture One never sees it. (2)
  `editor_not_following` covers the collection switch too. (3) The variant wait is owned by the follow service; the preview service only stops treating
  `variant_not_found` as terminal. (4) Owner card CI1 is closed (R46): there is no frontmost check anywhere.
- **Hazards:** a JXA call against a non-running Capture One **launches it** (osascript); the follow service must check `editor_info().running` before any
  of its scripts (C2(d)), or M40 breaks during a take-back. `find_image` searches only the current collection (MC-19) — the fake must mirror that:
  `find_image` returns a variant only if the image's folder equals `current_collection` (fake change; C3(d) relies on it). `on_variant_adjusted` clears
  `retry_after` (`previews.rs:197–201`); the new `retry_now` must also wake `schedule_due`. Migration 007 is additive; `raw_state` rows exist only after the
  first render job (`upsert_raw_state`): `mark_style_applied` must upsert.
- Registry additions made by the planner in master plan §6.4, §6.5, §6.11, §7.5. The intention book is not changed; reading (1) is reported to the owner
  through the narrative and the handoff.
- Estimated size: ~24 paths, ~1 400 lines (≈ 650 code + scripts, ≈ 750 tests).

## Review log

- **2026-11-07 implement r1 (codex).** Built current-collection switching on activation, startup, take-back step 5 and each direct-camera/import arrival; 1 s variant polling for 15 s followed by the 1/2/4/8/16/32 s preview backoff, immediate retry on a collection switch, style applied to a found variant before rendering and marked once per RAW in migration 007; Import now copies into the session capture folder with exclusive publication and leaves its source intact. The fake honors current-collection membership and script style read-back. 19/19 P49 behavior rows pass. Full Rust workspace suite, clippy, fmt and frontend typecheck pass; full frontend suite passes 369/369 with one worker. The default parallel frontend command twice hit the previously recorded P33 C4(d) timing flake (368/369), and that test passed alone.
- **Task 0 V4 reproduction before edits.** A fake RAW with no variant produced `VariantNotFound` at `CaptureOneAdapter::variant_background`; `PreviewService::request` stored `retry_after`, and after adding the variant the pre-change code did issue a render. It did so after the exponential delay (the probe reached it within 64 one-second ticks), unlike the live V4 observation that rendering stopped. No `render_in_flight` was left behind in the fake; the live-only V4 failure path remains unproven by that pre-change probe. The phase now holds the initial render until the variant appears and its C3(c) test pins the later backoff times.
- **Assumptions.** A collection script reads Capture One's live collection on each arrival so the photographer can browse away and the next arrival jumps back; it uses the existing `collectionForFolder` helper and `doc.currentCollection = collection`, read back via `file`. The temporary import copy is published with an exclusive same-directory hard link and the temporary name is removed; this has the same no-overwrite result as the planned exclusive rename. Tethered Capture One captures retain their prior render path; the wait applies to direct-camera deliveries and imports.
- **Deviations.** Added `apps/desktop/src-tauri/src/commands/sessions.rs` to connect collection-switch retries at the shipped Tauri command boundary. Updated P01's schema-column assertion and P27's import/migration fixtures because migration 007 and copy-on-import change their expected outcomes; updated P05's duplicate-import expectation as required by the new `__dupN` rule. The planned `SCRIPTS` count of 21 treats the old separately installed Import script as part of the total; the array itself has 22 entries after adding two scripts and removing that separate Import install. No intention contract was changed.
- **2026-11-07 review r1 (opus) — CHANGES_REQUESTED.** Gate: tracker `IMPLEMENTED`, checkpoint `df50d3d` in `git log`. Perimeter: the 30 committed paths equal the handoff's declared list (plan list + the 3 disclosed deviations `commands/sessions.rs`, `p01`, `p27`; planned `domain/editing.rs` unchanged, `StyleOutcome` reused). Reviewer run on clean `bfa3b61` (production = `df50d3d`): `cargo test --workspace` green (p49 19/19), clippy `-D warnings`, fmt `--check`, `npm run typecheck`, `npm test` 369/369 green.
  **Verified correct:** C1(a)–(c) activation/deactivate/`EditingStartup::run` call `set_current_collection` after a successful `set_capture_folder`; direct adapter → `editor_not_following`, Capture One camera adapter → `CaptureFolderNotSet` as the folder's. C2(a)–(d): switch on every arrival by a live compare inside `set_current_collection.js` (implementer assumption 1; matches R46 better than the plan's `folder != current` state, which C2(c) would have failed); follow checks `editor_info().running` before any script; both new scripts check `running()` and use the lane-only `call`, never `launching` (launch-gate list need not grow, M40). C3(a)–(d): `variant_waiting` holds renders while the follow service polls at 1 s for 15 s; then `variant_ready` → backoff 1/2/4/8/16 (C3(c) pins exact poll seconds); `VariantNotFound` never sets `failed_generation`; `collection_switched` clears `retry_after` and reschedules (C3(d) fails without it). C4(a)–(e): `mark_style_applied` upserts with `COALESCE`; the script short-circuits when the style is already listed (H4 crash safety). C5(a)–(c): copy via `create_new` temp + exclusive `hard_link`, `__dupN`, source untouched (H2), `_Unassigned` + follow switch. Sdef: `current collection` settable, `available styles`, variant `styles`, `apply style … named`, collection `file` (hidden, deprecated) exist. H1–H8 hold. Assumptions 1–3 do not contradict the book.
  **F1 (blocking, B1, route: production).** `CaptureOneFollow::prepare_render` (installed as `PreviewService::before_render`) applies the configured style before **every** render of **any** RAW whose `style_applied_at` is NULL, not only to arriving variants. Probe (temporary test, deleted): photo arrives with `style_name = ""`, renders; style set to "Bright Contrast"; `on_variant_adjusted` (retouch); tick → `apply_style:vold:Bright Contrast` then `process`. The same reaches every pre-P49 photo after upgrade and tethered captures. Authority: 10E §Capture One alongside ("applies it to each new variant"; R-I35 "never … on a retouched variant"); plan §The contract (`apply_style_once` on a found arriving variant). Correction: restrict the style step to RAWs that arrived through `ArrivalFollow::photo_arrived` and have not yet had a first render, held so it survives a settings rebuild (e.g. a pending marker written at arrival, or "no rendered generation yet"); keep C3(b)'s plan order (style set after arrival, before the variant is found) working, and add a test for this case. Files: `crates/integrations/src/editing/capture_one/follow.rs`, `crates/core/src/services/previews.rs`, `crates/core/src/ports/store.rs`, `crates/infra/src/sqlite/store.rs`, `crates/infra/src/sqlite/migrations/007_style_applied.sql`, `crates/testkit/tests/p49_capture_one_alongside.rs`.
  **Notes → backlog (8):** take-back step 5 passes `follow: None` (no immediate retry); a style-script error in the before-render step fails the render, so previews stop while it persists; `CaptureOneFollow.current` is written, never read; C3(b) test sets the style before arrival, not after as the row does; import of a file already in the target folder makes a `__dup1`, and a crash between copy and `arrival.import` leaves an unregistered copy; collection read-back uses the deprecated `file` while `collectionForFolder` uses `folder` (live-unverified → P52); task 0 did not reproduce V4 in the fake (live path → P52); plan's `SCRIPTS` 21 vs 22 (planner lesson).

- **2026-11-07 fix 1 (codex).** F1: `ArrivalFollow::photo_arrived` persists style eligibility using the existing `style_applied_at` field's reserved `i64::MIN` sentinel. The Store API masks that sentinel as “not applied”; eligibility remains true only while `rendered_gen == 0`, so a first completed preview closes the window across settings rebuilds. C3(b) now sets the style after arrival and before variant discovery; new F1 boundary case renders with no style configured, then changes settings, retouches, and asserts that no style is applied. Final stamp: workspace suite green (including P49 20/20), clippy `-D warnings`, fmt `--check`, frontend typecheck and UI suite 369/369 green. See fix r2 handoff for the checkpoint.


- **2026-11-07 re-review r2 (opus) — CHANGES_REQUESTED (1 MUST).** Gate: tracker `IMPLEMENTED` with fix note; checkpoint `084f1ee` in `git log`. Perimeter: `git show --stat 084f1ee` = `ports/store.rs`, `sqlite/store.rs`, `capture_one/follow.rs`, `p49_capture_one_alongside.rs` (all on F1's allowed list) + tracker row + Review log; nothing extra. Reviewer run (HEAD `013663d`, production = `084f1ee`): `cargo test --workspace` green (p49 20/20), clippy `-D warnings`, fmt `--check`, `npm run typecheck`, `npm test` 369/369.
  **F1 resolved.** `photo_arrived` writes a pending sentinel (`i64::MIN`) into `style_applied_at`; `prepare_render`/`apply_style_once` require `style_applied_at = sentinel AND rendered_gen = 0`. Late style + retouch → no `apply_style` (`f1_retouch_after_first_render_does_not_receive_late_style`; it fails without the `rendered_gen = 0` clause). C3(b) now sets the style after the arrival, so the row's order is tested, and the eligibility survives the settings rebuild. The sentinel divergence (no new column) is sound in outcome: `style_applied_at()` masks it, and no other reader of the column exists.
  **F2 (blocking, B1 → MUST, route: production; introduced by the fix).** `SqliteStore::style_eligible_before_first_render` selects `style_applied_at=?2 AND rendered_gen=0`. For a row with `style_applied_at IS NULL` and `rendered_gen = 0`, SQLite yields NULL (NULL AND TRUE), and `r.get::<bool>` returns `InvalidColumnType`, so `prepare_render` returns `Err`. `PreviewService::request` treats this as a provider failure and retries on the backoff forever: the preview is never issued. This affects every RAW that did not go through `photo_arrived` while a style is configured: **every tethered Capture One capture** (`follow_capture = false`), pre-P49 photos not yet rendered, and any arrival whose eligibility write failed. Probes (temporary, deleted): (1) `with_fake_document`, `style_name = "Bright Contrast"`, photo + `upsert_raw_state`, `request_initial` → `style_eligible_before_first_render = Err(InvalidData("Invalid column type Null …"))`, `ops = []`, nothing issued; the same without a style → `process:v1`. (2) End to end: a tethered file in the active session's capture folder → `watcher_event` → 20 × `tick(1_000)` → one photo row, **0 renders issued** with the style, and `process:vt:PSM Preview` without it. Authority: 10C MC-26 step 5 (tethered captures take the style from next-capture settings; a style problem never stops capture/preview; the app's by-script style is for the direct adapter only); 10E R-I35 (style scoped to arriving variants); the live-preview contract (stale photo → render). Correction: make the eligibility read NULL-safe, for example `COALESCE(style_applied_at=?2,0) AND rendered_gen=0`, `style_applied_at IS ?2 AND …`, or read `Option<bool>` → `unwrap_or(false)`. Add one boundary test: a tethered capture (`TestApp::with_fake_document`, watcher arrival) with a style configured gets its render issued and has no `apply_style` op. Files: `crates/infra/src/sqlite/store.rs`, `crates/testkit/tests/p49_capture_one_alongside.rs`.
  **Triage:** F2 MUST (a user-visible wrong outcome: adjusted previews never arrive for tethered captures whenever a style is set). Fix budget spent: the runner stops and the orchestrator decides (`FIX2_AUTHORISED` or a card).
  **Notes → backlog (2):** the sentinel overloads a timestamp column; a still-in-flight first render reopens the window on a settings change.
