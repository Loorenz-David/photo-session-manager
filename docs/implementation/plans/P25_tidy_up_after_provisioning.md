---
phase: P25_tidy_up_after_provisioning
title: Tidy-up after provisioning — no stray throwaway document, honest front/health state, empty default style, no false "RAW changed", logged startup, calmer shape retries, media found by filename
difficulty: high
projection: no
depends_on: [P24_capture_one_provisioning]
---
# P25 — Tidy-up after provisioning: no stray throwaway document, honest front/health state, empty default style, no false "RAW changed", logged startup, calmer shape retries, media found by filename

## Goal
Close the backend leftovers the owner said "go" to on 2026-11-04 (backlog rows 173, 168, 106 (+15, 69), 133 (+11), 164, 107,
167; the P24 narrative's "Not yet"; the owner's style decision, book 10C step 5 / changelog 26). When Capture One quietly refuses
to open the app's session, the `PSM Open Window` throwaway is closed again and Health says "could not open" — nothing else of the
photographer's is ever closed. The red "Capture One document" item and the camera's `Disconnected` no longer outlive the situation
they describe: they clear on the next check while no session is active and on deactivation. `settings_get` tells the UI the app's
own document path (`default_document_id`) so P26 can show the full path. The default `style_name` is empty, so a fresh setup
starts with a green Style row; an explicit name in an existing file is kept. A photo whose hash job has not run yet is hashed by
the integrity check instead of being marked `raw_changed`, so an export or publish seconds after the last shot goes through.
Logging starts before the app opens, so the startup provider calls reach `logs/`. A Health check re-queues `crop_ratio_unknown`
shape failures only when the name is listed again. The Shopify adapter identifies the media it just created by the upload
filename, never by gallery position. The P24/P17 test rows get the assertions the reviewer listed. **Not in this phase:** the two
frontend halves (P26: the full path in Settings, deferring a command's own reload while another drag is in flight), P09's stale
`.partial-*`/`.old-*` staging sweep (backlog 107's second clause: not trivial — `prepare` only discards its own previous staging;
left in the backlog), any change to which Health actions re-queue (`health_get` keeps re-queuing like "Retry checks"), gallery
`alt` text, anything post-MVP.

## Read first
`docs/implementation/backlog.md` rows cited per task (line numbers as of 2026-11-04: 11, 15, 69, 106, 107, 133, 164, 167, 168,
173) · `docs/implementation/narratives/P24_capture_one_provisioning.md` §Not yet · master plan §3 (standing rules), §6.4 (ports:
`Store::requeue_failed_jobs`, `CommerceAdapter`, `CaptureOneScripting`), §6.6 (health codes `document_not_front`,
`crop_ratio_unknown`, `crop_not_applied`, `style_not_found`; `Integrity`), §6.7 (`settings_get` keys), §6.8 (`style_name`, `document_id`),
§6.9 (test layout), §7.2 (commands), §7.5 (`TestApp`, `FakeCaptureOneScripting`, `FakeCommerce`, `FakeTransport`) · process `flow.md` §4
(one behaviour test per row), §6 (no questions), §6A (**the suite never reaches Capture One** — P24 F1) · book `01` (H2: no write
handle to a RAW; H6; H7), `14` (M10, M18, M32; M22/M25/M31 must not break), `10C` MC-26 step 3 ("Other open documents are never
closed, saved, switched or modified"), step 5 (**Default empty**, owner 2026-11-04, changelog 26), §Front document ("frontmost when a
session is active"), M32 · `05A` MC-3 (hash computed after stability, once per canonical path; **a mismatch** marks `raw_changed`;
M10) · `07A` MC-11 (upload filename `<folder_key>-<12 hex>.jpg`; "Position-based names are never used to identify remote media";
adoption by filename, uniquifier `_<uuid v4>`; "More than one match: adopt the first in gallery order") · `10B` MC-25 §Failures ("until
a later attempt succeeds"; the unknown-name item) · `12` §Logs ("every provider call with duration"), §Health, §Settings · evidence
`capture_one_16.8.6.md` "VERIFY-14 result" `:313–323` (`open` replaces the front window; the throwaway form; ~5 s per open) and the
P24 plan Review log r1/r2 (`currentDocument()` may read `null` right after `open`; **`open` of a missing file does not throw — the
throwaway stays open and in front**) · evidence `shopify_admin.md` SV-2 `:175–206` (`productUpdate` with `media` returns the new media
in `newest` = `media(first: 1, reverse: true)`, status `PROCESSING`, **no filename until `READY`**: `image.url` and `originalSource.url`
are `null` while processing; the filename is the last path segment of `image.url` before `?`; uniquifier `_<uuid>`; `newest` equalled
the last gallery node on all three probe creates) · code (line numbers at `c90b95d`, 2026-11-04):
`crates/integrations/src/editing/capture_one/jxa/open_document.js` (`:3–6` `make` then `open`; the `catch :7–11` closes the throwaway only
when the script **threw**), `jxa.rs` (`SCRIPTS :16` — 21 entries; `open_document :135–147` creates and removes the scratch parent),
`provision.rs` (`clear_after :57`; step 3 `:187–204`: an `open_error` skips the wait and yields `document_not_open` with `: {error}`;
`front_provision_document :305–321` — `document_not_front` is cleared **only** at `:319`), `crates/core/src/services/editing_startup.rs`
(`:58–61` front only when a session is active; nothing clears the item otherwise), `sessions.rs` (`activate :51–68`, `deactivate :70–81`),
`camera.rs` (`status :27` and `capture :54` gated on `document_not_front`), `health.rs` (`set :76`, `clear :96`, `has :101`),
`crates/infra/src/settings_file.rs` (`Default :41–58`, `style_name: "PSM House" :47`; `effective_document_id :62–68`),
`crates/integrations/src/lib.rs` (`EDITOR_WAIT :60`; `refresh_shape_health :62–74`; `AppHandles::health_check :191–215` — re-queue `:193–195`
runs **before** the `EditingStartup` block; `settings_get :227–241` adds `effective_document_id`; `App::open :309–313`, `open_with :315`,
`build :344` with `start_backup`, the real-start re-queue `:401–403`, `document_id`/`provision` `:404–412`; the startup thread warns only
`:470–476`; the hash job `:804–806` calls `set_raw_hash`), `crates/infra/src/sqlite/store.rs` (`count_jobs :937–940` with `exclude_error`,
`requeue_failed_jobs :941–944`, `set_raw_hash :1209–1216` — `WHERE raw_hash IS NULL`, so a later hash job is a no-op; shared hash on
insert `:1052–1054`), `crates/core/src/ports/store.rs` (`count_jobs :125`, `requeue_failed_jobs :126`, `set_raw_hash :153`,
`set_photo_integrity :161`), `crates/infra/src/jobs.rs` (`:55–75`: `CropRatioUnknown` → `job_fail` at once, no back-off `:60–66`),
`crates/integrations/src/editing/capture_one/shape.rs` (`run :23–48`), `crates/core/src/services/integrity.rs` (`verify :12–24`: a `None`
hash compares unequal to any file → `RawChanged` persisted), `export.rs` (`:244–250`), `publish.rs` (`:424–430`; `name_matches :534–545`;
the stem `:593–597`; the upload loop `:602–660`), `crates/core/src/domain/photo.rs` (`Integrity :23–36`), `crates/core/src/mechanisms/hash.rs`
(`sha256_file :5`), `apps/desktop/src-tauri/src/lib.rs` (`run :158–161`: `App::open` **then** `logging::init`; `p17_tests :210`,
`p21_tests :305` — the desktop test modules over `psm_testkit`), `crates/infra/src/logging.rs` (`init :99–113`: the
`set_global_default` result is ignored — **one subscriber per process**), `crates/infra/src/paths.rs` (`create :31–39` makes `logs/`),
`crates/integrations/src/commerce/shopify/adapter.rs` (`upload_filename :95`; `remote_media :160–171`; `mutate :288`; `product :299–305`
over `graphql::PRODUCT_READ`; `poll_until :379–394`; `product_create_media :813–838` — takes `newest.nodes[0].id` unchecked `:834`;
`poll_media_ready :840–856`; `list_media :892–894`), `graphql.rs` (`PRODUCT_READ :9`, `PRODUCT_CREATE_MEDIA :21`, `MEDIA_POLL :23`),
`apps/desktop/src-tauri/src/commands/settings.rs` (`SettingsDto :20–33`), `apps/desktop/src/services/bindings.ts` (`SettingsDto_*
:525–553`) · fakes: `crates/testkit/src/fake_capture_one.rs` (`camera :26`, default `Some("Sony ZV-E10") :114`; `crop_ratios :51`;
`styles :94` default `["PSM House"]` — **stays**; `open_document :274`, `front_document :294`, `document_info :372`,
`crop_aspect_ratios :531`, `shape_on_arrival :553`, `capture :596`), `crates/testkit/src/fake_commerce.rs` (`product_create_media
:344–373` records the staged filename — unchanged), `crates/testkit/src/fake_transport.rs` (`on_graphql :76`, `replay :98`,
`replay_any :106`, `graphql(op) :147`), `crates/testkit/src/fixtures/shopify.rs` (`fixture(name)`, `.reply()`, `.data()`),
`crates/testkit/src/test_app.rs` (`add_photo :56–67` stores the **fake** hash `hash-<raw>`; `create :95–150`, `run_startup = false
:138`; `kill_and_restart :158`), `crates/testkit/src/clock.rs` (`ManualClock`) · tests to reuse or change: `crates/testkit/tests/
p24_capture_one_provisioning.rs` (`item :11`, `inputs :19`, `ops :25`, `detail :28`; `c1a :38–103`, `c1b :105–129`, `c1e :222–275`,
`c1f :277–313` (relies on the default style), `c2b :353–383`, `c2c :385–414`, `c3a :416–448`, `JXA_HARNESS :450–483`, `script_result :485`,
`c4b :544`), `p17_real_run_fixes.rs` (`c1b :75–85`), `p00_spike.rs` (`:78` asserts `"PSM House"`; `c1b :94–130`: the fake-app
construction `App::open_with(paths, FakeCaptureOneScripting, ManualClock, false)` with `paths.home` = temp dir), `p01_storage_identity.rs`
(`photo :17–33` takes `Option<hash>`; `item_session :35`; `c4b :373–395`), `p09_local_export.rs` (`app :14`, `c1_session :99`, `export :112`,
`names :64`), `p13_publish_runs.rs` (`app :17`, `session :22`, `photo :30–55` stores the real `sha256_file`; `c3d :463`),
`p23_shape_on_arrival.rs` (`app :57`, `file :63`, `hook :69`, `jobs :76`, `variant :86`, `ops :97`, `health :116` — runs `health_check()`;
`drive_to_failure :371`; `c3c :378–415`, `c3d :417–448` — must keep passing), `p12_shopify_adapter.rs` (`shop :87`, `run :93`,
`staged_target :738`, `c6c :787–800`, `c6d :803`; fixtures `sv2_media_create`, `sv2_media_poll_ready`, `sv2_media_poll_processing`,
`sv2_item_get` — the real three-media gallery incl. the uniquified `psm-probe-ab12cd34ef56_4fffd4cd-….jpg`), frontend `Settings`-typed
fixtures (`p16_settings.test.tsx:16`, `p24_capture_one_provisioning.test.tsx:16`, `p23_shape_on_arrival.test.tsx:14`,
`p19_navigation.test.tsx:155`, `p22_shell_tidy.test.tsx:17`, `p18_pending_state.test.tsx:128`).

## Files expected to change
`crates/integrations/src/editing/capture_one/jxa/open_document.js` (post-`open` front check), `crates/core/src/services/{editing_startup.rs
(clear `document_not_front` when no session is active), sessions.rs (`deactivate` clears it), integrity.rs (NULL hash → hash now)}`,
`crates/infra/src/settings_file.rs` (`style_name` default `""`; `default_document_id`), `crates/integrations/src/lib.rs` (`settings_get`
`default_document_id`; `health_check` order + exclusion; the `app open` log line in `build`), `crates/core/src/ports/store.rs` +
`crates/infra/src/sqlite/store.rs` (`requeue_failed_jobs` gains `exclude_error`), `crates/core/src/services/publish.rs` (`name_matches`
moves to a shared home, see task 8), `crates/core/src/mechanisms/upload_name.rs` (new, or `pub` on the service — Delegated choices),
`crates/integrations/src/commerce/shopify/adapter.rs` (`product_create_media` resolves the id by filename), `apps/desktop/src-tauri/src/
lib.rs` (`open_logged`; `run()` uses it; new test module `p25_tests`), `apps/desktop/src-tauri/src/commands/settings.rs`
(`SettingsDto.default_document_id`), `apps/desktop/src/services/bindings.ts` (regenerated: one new field),
`crates/testkit/tests/p25_tidy_up_after_provisioning.rs` (new: C1–C5, C7), `crates/testkit/tests/p12_shopify_adapter.rs` (C8 rows beside
`c6c`; `c6c` gains the `MediaPoll` reply), the backfilled assertions (task 9): `crates/testkit/tests/p24_capture_one_provisioning.rs`
(`c1a`, `c1b`, `c1e`, `c3a`; `c1f` sets the style explicitly), `p17_real_run_fixes.rs` (`c1b`), `p00_spike.rs` (`:78` → `""`), the six
frontend `Settings` fixtures above (`default_document_id` added; no behaviour change, typecheck only).

## Core mechanisms (unit tests allowed)
None new. (`classify_editor_error` keeps its P24 unit test; `name_matches` keeps being proven through the publish rows.)

## Ordered tasks
1. **Close the throwaway when `open` fails silently (backlog 173; 10C step 3).** `open_document.js`: after `a.open(Path(argv[0]))`,
   read `const front = a.currentDocument();`. If `front` is non-null and `String(front.name()).startsWith('PSM Open Window')`, Capture One
   did not replace the throwaway's window: `a.close(front)` and `throw new Error('Capture One did not open ' + argv[0])`. A `null`
   front (observed live after a successful open) or any other document in front is success (`return JSON.stringify(null)`). The
   `catch` keeps its behaviour. Nothing else is ever closed: the only `close` the script can issue names a document whose name
   starts with `PSM Open Window`. Rust side unchanged: the thrown error reaches `provision.rs:188` as `open_error`, the wait is skipped
   and Health reads `Capture One could not open <file>: <error>` (C1(e)(2) already proves that half over the fake).
2. **`document_not_front` describes only an active session (backlog 168; 10C §Front document, M32 "when a session is active").**
   `EditingStartup::run`: when `active.is_none()`, `health.clear("document_not_front")` (the branch beside the existing `bring_document_to_front()?`).
   `SessionsService::deactivate`: after `set_active_session_id(None)`, `health.clear("document_not_front")`. `activate` is unchanged:
   a refused activation still sets the item and blocks capture (P24 C2(b) stands); the item now lapses at the next check or
   deactivation instead of waiting for a successful activation.
3. **`default_document_id` (P24 narrative "Not yet"; backend half of P26 C1).** `Settings::default_document_id(&self, paths) -> PathBuf`
   = `paths.home/Pictures/Session Manager` (no settings involved; `effective_document_id` uses it). `AppHandles::settings_get` adds
   `"default_document_id": <string>` beside `effective_document_id`; `SettingsDto` gains `default_document_id: String`; bindings regenerated.
   The six frontend `Settings` fixtures gain the key so `npm run typecheck` stays green (P26 renders it).
4. **Empty default style (10C step 5, changelog 26).** `Settings::default().style_name = String::new()`. `p00_spike.rs:78` asserts `""`.
   `p24 c1f` no longer relies on the default: it saves `{"style_name":"PSM House"}` first and keeps the rest of its assertions (the
   fake's `styles` default `["PSM House"]` stays, master plan §7.5). An existing `settings.json` that explicitly holds a name is
   untouched (serde reads it; nothing rewrites the file).
5. **A missing hash is hashed, never a mismatch (backlog 106/15/69; 05A MC-3).** `IntegrityCheck::verify`: when `photo.raw_hash` is
   `None`, compute `sha256_file(raw_path)`; on success `store.set_raw_hash(&photo.raw_path, &actual)` (the same write the hash job
   makes; it fills every NULL row of that path, so the later job is a no-op), persist `Integrity::Ok` and return it; `NotFound` →
   `RawMissing` as today; other errors propagate. A photo **with** a stored hash behaves exactly as today (p01 c4b/c4c/c4d, p09 c5a,
   p13 c3d stay green). No caller changes: export and publish prepare already use `verify`.
6. **Logging before the app opens (backlog 133/11; ch12 §Logs).** `apps/desktop/src-tauri/src/lib.rs`: `pub fn open_logged(paths: AppPaths,
   open: impl FnOnce(AppPaths) -> psm_core::Result<App>) -> psm_core::Result<(Option<tracing_appender::non_blocking::WorkerGuard>, App)>`
   = `paths.create()?` → `let guard = psm_infra::logging::init(&paths)` → `let app = open(paths)?` → `Ok((guard, app))`. `run()` becomes
   `let (_logging, app) = open_logged(paths, psm_integrations::App::open).expect(...)` (the guard lives as long as `run`, as today).
   `App::build` emits, as its first statement after `paths.create()`, `tracing::info!(data_dir = %paths.root.display(), start = start_backup,
   "app open")` — the marker a test can look for without Capture One (the fake issues no `osascript`, so no "provider call" line exists
   in the suite; in production the `editor_info` call of the startup thread now follows it in the file).
7. **Re-queue `crop_ratio_unknown` only when the name is listed again (backlog 164; 10B MC-25 §Failures).** `Store::requeue_failed_jobs(kind,
   exclude_error: Option<&str>, now_ms)` (SQLite: `AND (?3 IS NULL OR error IS NULL OR error<>?3)`, the `count_jobs` form). `AppHandles::
   health_check`: keep `refresh_shape_health` first (counted first — P23 C3(c)), run the `EditingStartup` block (it sets or clears
   `crop_ratio_unknown` from the live ratio list), **then** `requeue_failed_jobs("crop_on_arrival", health.has("crop_ratio_unknown")
   .then_some("crop_ratio_unknown"), now)` when the setting is not `off`. The real-start re-queue in `App::build` passes `None` (unchanged:
   one attempt per launch is the owner's chance to see a corrected name applied). `provider`/`variant_not_found` failures keep re-queuing
   on every check (P23 C3(c)).
8. **The created media is found by filename (backlog 107 first clause; 07A MC-11).** `ShopifyAdapter::product_create_media`: `stem` = the
   last path segment of `resource_url`, without `.jpg` (the staged filename MC-11 sent). After the mutation: `candidate` = `newest.nodes[0].id`
   (absent → the existing error). Poll `MEDIA_POLL` for `candidate` with `poll_until` until `status == "READY"` (`FAILED` → `Err(Provider
   (<mediaErrors[0].message>))` exactly as `poll_media_ready`; timeout → `Error::Timeout`), read its filename (last path segment of
   `image.url` before `?`, else of `originalSource.url`); if it matches the stem → return `candidate`. Else `list_media(external_id)` and
   return the id of the **first** `RemoteMedia` in gallery order whose `filename` matches (MC-11 "adopt the first in gallery order"); none →
   `Err(errors::provider("product_create_media", "created media <stem>.jpg not found in the gallery"))`. "Matches" is the one MC-11 rule
   (`<stem>.jpg` or `<stem>_<uuid v4>.jpg`), shared with `PublishService::name_matches` — move it to `psm_core::mechanisms::upload_name::
   matches(stem, name)` (or make the service function `pub`; Delegated choices) and call it from both; the adapter never re-implements it.
   `FakeCommerce` is unchanged. `poll_media_ready` stays on the port (publish's second poll returns at once on `READY`).
9. **Backfill the P24/P17 assertions (backlog 167).** `p24 c1a`: after the first check, `ops` hold no `open_document:`; the report has no
   item whose code starts with `document_not_`, `editor_` or equals `recipe_missing`. `p24 c1b`: the cleared `ops` hold no `make_document:`,
   `open_document:` or `make_recipe:`. `p24 c1e` third app: `detail("hooks") == Some((true, "Installed"))`. `p24 c3a`: after the explicit
   save `settings_get()["document_id"] == "/Users/me/Pictures/Other"`; the pre-existing file's bytes are equal before and after
   `kill_and_restart()` + `settings_get()`. `p17 c1b`: `fake.recipes["PSM Preview"]` equals its value before the `Nope` save and no
   `set_recipe_props:PSM Preview:` op was issued.
10. Tests (§6.9: `crates/testkit/tests/p25_tidy_up_after_provisioning.rs`; the C8 rows in `p12_shopify_adapter.rs`; the C6 row in the desktop
    crate's `p25_tests`; the task-9 edits), `cargo test --workspace`, clippy, fmt, bindings regeneration (diff expected: `SettingsDto.
    default_document_id`), `npm run typecheck`, `npm test` (fixture-only TS changes). **No test may construct `App::open(..)` or a real
    `JxaScripting`-backed `App`** (P24 F1); the live Shopify store is never contacted (fixtures only).

## Criteria (8 criteria, 12 rows; 4 `[rule]` rows)
`fake` = `a.fake.state.lock().unwrap()`; `ops`, `detail`, `item` as in p24 `:11–36`; `home` = `a.dir.path()`; `DEFAULT` = `<home>/Pictures/Session
Manager`; `DOC` = the fake's configured document `<home>/Capture One Session`; `root` = `<DOC>/Capture`. Harness for C1 = p24
`JXA_HARNESS :450–483` extended: `currentDocument()` returns an object with `id()` **and** `name()` (or `null`); `make` of a `document`
records the call and makes `{id: argv[1] + '/PSM Open Window', name: 'PSM Open Window'}` current; `open` records the call and then, by mode,
makes the target `{id: '/tmp/x/Session Manager', name: 'Session Manager.cosessiondb'}` current (`replaced`), makes `null` current (`null`;
the P24 r1 live observation), or leaves the throwaway current (`refused`; the P24 r2 live observation); `close(d)` records `{op: 'close',
value: d.id()}`. The harness prints `{result, calls}` or `{error, calls}`.

| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | `node -e <harness> <jxa>/open_document.js` with argv `['/tmp/x/Session Manager/Session Manager.cosessiondb', '/tmp/x/scratch']`, three runs | modes `replaced`, `null`, `refused` | `replaced` and `null`: `result == null`; `calls` are exactly `[make{new:'document', withProperties.name:'PSM Open Window', path:'/tmp/x/scratch'}, open('/tmp/x/Session Manager/Session Manager.cosessiondb')]` — no `close`; `refused`: `error` contains `/tmp/x/Session Manager/Session Manager.cosessiondb`; `calls` are exactly `[make…, open…, close('/tmp/x/scratch/PSM Open Window')]` — the one `close` names the throwaway's id and no other document | M32 / MC-26 step 3 (other documents never closed; the throwaway never left behind); evidence VERIFY-14 + P24 Review log r2 (silent `open` failure) |
| C2(a) | `with_fake_document()`; `s = item(&a, "K1")`; `fake.fail_front = true`, `front_document = Some("/elsewhere/Instagram")`; `a.sessions().activate(s)`; then `fake.fail_front = false`; ops cleared; `a.app.health_check()`; `CameraService::new(a.app.camera.as_ref(), a.app.health.as_ref())` → `status()`, `capture()` | a refused activation, then a check while no session is active | `activate` → `Err(Error::DocumentNotFront)` and `detail("document_not_front")` is `Some((false, _))` (P24 C2(b)); after the check: `detail("document_not_front") == None`; the cleared `ops` hold no `front:` and hold `set_captures:<root>/_Unassigned`; `active_session_id() == None`; `status() == Ok(CameraStatus::Connected("Sony ZV-E10"))`; `capture() == Ok(())` and `ops` now contain `capture` | 10C §Front document / M32 ("frontmost **when a session is active**"); ch12 §Health (an item describes a current fact); backlog 168 |
| C2(b) | `with_fake_document()`; `activate(s)` → `Ok`; `fake.fail_front = true`, `front_document = Some("/elsewhere/Instagram")`; `health_check()`; then `a.sessions().deactivate()`; then `fake.front_document = Some("/elsewhere/Instagram")` again, ops cleared, `health_check()` | the front lost while active, then deactivation | after the first check: `detail("document_not_front")` is `Some((false, "Capture One is showing another session; captures would land there"))` (re-asserted while active, P24 C2(c)); `deactivate()` → `Ok`; right after it: `detail("document_not_front") == None`, `active_session_id() == None`, the last `set_captures:` op is `set_captures:<root>/_Unassigned`; after the second check: still `None`, the cleared `ops` hold no `front:` | M32 / 10C §Front document; MC-18 (deactivation routes captures to `_Unassigned`) |
| C3(a) | `TestApp::new()`: `settings_get()`; `settings_set({"document_id":"/Users/me/Pictures/Other"})`; `settings_set({"document_id":""})` | nothing configured, an explicit document, cleared | first: `["default_document_id"] == DEFAULT` and `["effective_document_id"] == DEFAULT`; after the explicit save (both the returned value and `settings_get()`): `default_document_id == DEFAULT` unchanged, `effective_document_id == "/Users/me/Pictures/Other"`, `document_id` the same; after `""`: `effective_document_id == DEFAULT`, `default_document_id == DEFAULT` | `[rule]` 10C MC-26 §Setting (the app's own document is a fixed place the UI can name); ch12 §Settings; P24 narrative "Not yet" (short form shown until save) |
| C4(a) | `TestApp::new()` (its `settings.json` written from the defaults); `settings_get()`; `fs::read(settings.json)`; `health_check()` | a fresh setup | `["style_name"] == ""`; the file parses with `"style_name": ""`; the report has no `style_not_found` item and has `hooks` ok; `ops` hold no `ensure_capture_style:`; `fake.next_capture_styles == []` (left as the photographer set them) | `[rule]` 10C step 5 "Default empty" (changelog 26); M32 (the sequence still completes) |
| C4(b) | `with_fake_document()`; `settings.json` rewritten with `"style_name":"PSM House"` (the owner's existing file), `kill_and_restart()`, `settings_get()`, `health_check()`; separately `TestApp::new()`, `settings.json` rewritten **without** the `style_name` key, `kill_and_restart()`, `settings_get()` | an explicit name; a file from before the key existed | explicit: `["style_name"] == "PSM House"`; after the check `fake.next_capture_styles == ["PSM House"]`, exactly one `set_next_capture_styles:PSM House` op, no `style_not_found`; the file's bytes are unchanged by the restart and the check; absent key: `["style_name"] == ""` | `[rule]` 10C step 5 (a named style is applied); ch12 §Settings (an explicit value is honoured, the file is not rewritten) |
| C5(a) | `TestApp::new()`; `fresh.ARW` written; p01 `photo(&a, &path, item_session(&a,"A"), None)`; `IntegrityCheck::new(a.store()).verify(id)`; then the file appended with `b"changed"`, `verify(id)` again; separately a second photo with `None` whose file is deleted before `verify` | a photo whose hash job has not run; the same photo changed later; a missing unhashed file | first `verify` → `Ok(Integrity::Ok)`; `store.photo(id).raw_hash == Some(sha256 of the original bytes)` and `.integrity == Ok`; second → `Ok(RawChanged)`, `raw_hash` unchanged (the first value), `integrity == RawChanged`; the deleted one → `Ok(RawMissing)`, `raw_hash == None`, `integrity == RawMissing` | 05A MC-3 / M10 (hash first; a mismatch only against a stored hash; `raw_hash` never recomputed in place); H2 (read-only hashing) |
| C5(b) | p13 shape: `app()`, `s = session(&a, "VF-NULL")`, one Main photo inserted like p13 `photo` but with `raw_hash: None` and **no** hash job; `publish.confirm(&[s], "yes")`; `a.tick(1100)` | publishing seconds after the last shot | the publish item's `failed_step == None` and `error == None` (prepare passed; no `raw_changed`); `store.photo(p).raw_hash == Some(sha256_file(raw))`, `integrity == Ok`; the fake shop received at least one `staged_uploads_create` (the run went on past prepare) | M18 / 07A MC-11 prepare ("re-verify RAW hashes (05A MC-3)") — a photo is never refused for a hash nobody computed yet; backlog 106 |
| C5(c) | p09 shape: `app()`, `s = session(&a, "K")`, one Main photo inserted with `raw_hash: None` (p09 `photo` variant), `a.app.export.run(s)`, `complete_all_renders()` | exporting seconds after the last shot | `run` → `Ok`; `<export root>/K/K_01_main.jpg` exists; `store.photo(p).raw_hash == Some(sha256_file(raw))`, `integrity == Ok` | M10 / 05A MC-3 ("blocks that photo's export" only on a real mismatch); backlog 69 |
| C6(a) | desktop crate `p25_tests`: `paths = AppPaths::for_dir(tmp)` with `paths.home = tmp`; `open_logged(paths, \|p\| App::open_with(p, Arc::new(FakeCaptureOneScripting::new(DOC, root)), Arc::new(ManualClock::new(0)), false))` (p00 `c1b` construction); drop the returned guard and the app; read `<tmp>/logs/psm.*.log` | the app opened through the production entry's seam | `Ok`; exactly one `psm.*.log` file exists under `<tmp>/logs/`; its text contains `app open` and the data dir path `<tmp>` (the line `build` emitted **before** `init` would have run in the old order); this is the only test in the desktop test binary that calls `open_logged` (one global subscriber per process — Notes) | `[rule]` ch12 §Logs (every provider call with duration — the startup ones were lost before `init`); backlog 133 |
| C7(a) | `with_fake_document()`; `settings_set({"crop_on_arrival":"Squircle"})`; `hook` for `K1/g.ARW` (image `"15"`, p23 `file`/`hook`); `tick(0)`; ops cleared; `health_check()` twice; then `fake.crop_ratios.push("Squircle")` (Capture One now lists it); `health_check()`; `tick(0)` | an unknown name failed once; two Health visits; the name listed again | after the arrival tick: the `crop_on_arrival` row is `failed`, `error == "crop_ratio_unknown"`, `attempts == 1`; after the two checks: the row is **still** `failed` with `attempts == 1` and `error == "crop_ratio_unknown"`, the cleared `ops` hold no `set_crop:`, each report has `crop_ratio_unknown` (`ok == false`) and no `crop_not_applied`; after the third check: the row is `queued` with `attempts == 0` and the report has no `crop_ratio_unknown`; after `tick(0)`: `done`, variant `15` reads `Squircle` | 10B MC-25 §Failures ("until a later attempt succeeds"; the unknown-name item stays distinct); M31; backlog 164 — P23 C3(c) (`provider` failures re-queued on every check) and C3(d) (corrected setting) must still pass unchanged |
| C8(a) | p12 shape (`shop()`, `run()`, `staged_target()`, resource filename `psm-probe-ab12cd34ef56.jpg`), four runs with scripted replies: (1) `ProductCreateMedia` → `sv2_media_create`; `MediaPoll` → `sv2_media_poll_ready`; (2) `sv2_media_create` with `newest.nodes[0].id` rewritten to `gid://shopify/MediaImage/46644767686809`; `MediaPoll` → `sv2_media_poll_ready` with `id` and both URLs rewritten to that node's values from `sv2_item_get` (`psm-probe-0123456789ab.jpg`); `ProductRead` → `sv2_item_get`; (3) `newest` rewritten to `gid://shopify/MediaImage/46644767457433`; `MediaPoll` → that node's values (`psm-probe-ab12cd34ef56_4fffd4cd-17a1-449a-92f7-2e862798500b.jpg`); (4) as (2) but `ProductRead` → `sv2_item_get` with the two `ab12cd34ef56` nodes removed | newest is ours; newest is another media; newest is ours under a uniquifier; ours nowhere | (1) returns `gid://shopify/MediaImage/46644767260825`; `graphql("MediaPoll").len() == 1`, `graphql("ProductRead")` empty; (2) returns `gid://shopify/MediaImage/46644767260825` (first match in gallery order, not the uniquified `…457433`); `graphql("ProductRead").len() == 1`; (3) returns `gid://shopify/MediaImage/46644767457433`, no `ProductRead`; (4) `Err(Error::Provider(m))` with `m` containing `psm-probe-ab12cd34ef56`; in every run exactly one `ProductCreateMedia` was sent with `media[0].originalSource == resource` | M18 / 07A MC-11 ("Position-based names are never used to identify remote media"; "adopt the first in gallery order"); SV-2 (filename only once `READY`; the uniquifier form) — fixtures are the captured SV-2 replies with ids/URLs taken from the captured gallery, never invented |

## Delegated choices
Whether `default_document_id` is a `Settings` method or a free function in `lib.rs`; the home of the shared filename matcher
(`psm_core::mechanisms::upload_name::matches` or `pub fn name_matches` on the publish service — one function, two callers); whether the
adapter's stem is derived in `product_create_media` or passed from a small helper; the exact wording of the two new `Provider` messages
beyond the required substrings; whether the `MediaPoll` inside `product_create_media` reuses `poll_media_ready`'s closure or a sibling
that also returns the filename; the name and placement of `open_logged` (a `fn` in `lib.rs` is the plan's default); the exact text of the
`app open` log line beyond the two required fields; whether `requeue_failed_jobs` keeps its name with a third argument or becomes
`requeue_failed_jobs_except`; the harness modes' names; whether C5(b) drives one `tick(1100)` or p13's full series (the row's assertions
hold either way); how the p24/p17 backfills are expressed (assertion lines in the existing tests — never new tests).

## Manual try-out
1. Quit the app. Move `~/Library/Application Support/com.loorenz.sessionmanager/settings.json` aside (keep it) and launch: Settings shows
   an empty Style field and Health has no "Style" row. Put the file back and relaunch: your saved settings return; if your file still
   says `PSM House` and you have no such style, the red "Style" row is back — clear the field and Save, as the P24 narrative said.
2. Make a folder `~/Pictures/Scratch` with an empty text file `Scratch.cosessiondb` inside. Settings → "Use another session…" → pick it →
   Save. Capture One cannot open that file: Health shows "Capture One document — Capture One could not open …/Scratch.cosessiondb…", and
   Capture One's Window menu lists **no** "PSM Open Window" document; your other documents are untouched. "Use the app's own session" →
   Save; delete `~/Pictures/Scratch`.
3. Open a session in the app, then click your Instagram session in Capture One and press Health → Retry checks: the app's document comes
   back to the front. Now close the session in the app (deactivate), click Instagram in Capture One again, Retry checks: nothing switches,
   there is no red "Capture One document" row, and the camera shows connected (not "disconnected").
4. Tether the camera, shoot one frame and press Export within a second or two: it exports; no "RAW changed" anywhere. Same with Publish
   on a session whose last shot was seconds ago.
5. Open `~/Library/Application Support/com.loorenz.sessionmanager/logs/psm.<today>.log` right after a launch: the first lines are
   `app open` followed by the startup `provider call` lines (`op=editor_info`, `document_info`, …) with their durations.
6. Quit the app, set `"crop_on_arrival": "Squircle"` in `settings.json`, launch, shoot two frames: Health shows "Shape on arrival —
   'Squircle' is not a Capture One crop ratio". Open Health three or four times: the log shows no new `apply_crop` provider calls. Pick a
   real shape in Settings, Save, Retry checks: the two frames are shaped.
7. Publish a session with new photos: it works as before; in the log every `ProductCreateMedia` is followed by a `MediaPoll` before
   the upload is marked attached.

## Notes
- **Verified against the code before planning (2026-11-04, `c90b95d`); nothing in the owner's list is already fixed.** Items 1–9 are
  all present at the cited lines; item 10's five assertion gaps are all still open (p24 `c1a :85–96` loops over four prefixes but not
  `open_document:` after the *first* check and never checks the absent codes; `c1b` asserts the `set_recipe_props` op only; `c1e`'s third
  app asserts `document` and `recipes` but not `hooks`; `c3a` asserts `effective_document_id` after the explicit save but not the stored
  `document_id`, and never compares the file's bytes; p17 `c1b :75–85` never looks at `PSM Preview`).
- **Why the script, not Rust, detects the silent `open` (task 1).** Only the script sees Capture One's front document right after `open`;
  Rust would need a second `osascript` round trip and could not tell "still loading" from "refused". Live evidence (P24 r2): the script's
  `open` returned after ~3.9 s with the target open, and returned success at once for a missing file with the throwaway in front —
  so when `open` returns, the front is either the target, `null`, or the untouched throwaway. The check is by **name prefix** of the
  throwaway the app itself made; no other document can match it.
- **Why `document_not_front` clears when no session is active (task 2).** M32 binds the front "when a session is active"; with none
  active, captures route to `_Unassigned` in whatever document is in front, which is the pre-P24 behaviour and not a fault. A refused
  activation keeps the item until the next check or deactivation (P24 C2(b) unchanged), so the photographer still learns why the
  activation was refused.
- **`default_document_id` vs `effective_document_id` (task 3).** The frontend cannot know `$HOME`; while `document_id` is set, `effective` equals
  it, so the UI has no way to show the app's own path before saving — hence a third, constant key.
- **The owner's own `settings.json` already holds `"style_name": "PSM House"`** (written from the old default on first run). Task 4 changes
  the default only; the file keeps its explicit value until the owner clears the field (try-out 1; the P24 narrative already told them).
- **Hash-first, not "pending" (task 5).** MC-3 computes the hash "after stability (MC-2) and only once per canonical path"; a photo that is
  in the database arrived after stability, so hashing it at verify time is the same computation the job would make, through the same
  `set_raw_hash` (which fills only NULL rows). A "pending" outcome would need a new `Integrity` variant and a wait in two services for no
  gain. `TestApp::add_photo` stores a **fake** hash (`hash-<raw>`), so rows C5(a)–(c) insert their photos with `NewPhoto { raw_hash: None }`
  directly (p01/p09/p13 helper shapes), never through `add_photo`.
- **One global `tracing` subscriber per process (task 6, C6(a)).** `logging::init` ignores the `set_global_default` error, so a second call in
  the same test binary installs nothing and the file stays empty. C6(a) is the only test in `apps/desktop/src-tauri` that initialises
  logging; the Review log must record that constraint for later desktop tests. The guard is dropped before the file is read (the
  non-blocking appender flushes on drop). `App::open_with(.., false)` keeps the suite away from Capture One and the spool watcher; the
  `app open` line is emitted by `build` regardless of `run_startup`.
- **What a Health visit still does (task 7).** `health_get` runs `health_check` (P23 chose "Retry checks" as the trigger; the page's load
  uses the same command). This phase removes only the pointless burst — one Capture One call per waiting photo while the name is still
  unknown; it does not change which actions check. A real start still re-queues everything once (the P23 try-out "start Capture One,
  relaunch" path).
- **Why `product_create_media` polls (task 8).** SV-2: the create reply carries no filename while the media is `PROCESSING`, and `alt` is
  the item title for gallery media (a product-visible field; changing it to the filename would be an owner choice, not made here). So
  the adapter waits for `READY` (1–2 s live, as publish already does right after) and compares the CDN filename; on a mismatch it reads
  the gallery once. A timeout inside create leaves the link `uploaded` with no `external_id`; the next attempt's existing adoption step
  waits for processing media and adopts by filename (MC-11's crash-recovery path), so the change is safe under M18. Fixtures: the real
  SV-2 captures; where a row needs "another media is newest", the node is one of the three real media of the captured gallery
  (`sv2_item_get`), so every id and URL shape is one Shopify produced.
- **Backlog 107's second clause (P09 staging sweep) is left out**, as the owner allowed: `prepare` discards only its own previous staging;
  a sweep of stale `.<key>.partial-*`/`.old-*` needs its own rule for what is "stale" and is not trivial. The row stays open in the backlog
  with this phase's mark on the first clause only.
- **Tests that change their expectation** (to be named again in the handoff): `p00 :78` (`""`), `p24 c1f` (explicit style save first), `p12 c6c`
  (one `MediaPoll` reply added), the six frontend `Settings` fixtures (`default_document_id`). Nothing else changes meaning; the task-9
  backfills only add assertions.
- Registry additions made by the planner (master plan §6.4 `requeue_failed_jobs` signature, §6.6 the `document_not_front` clearing rule and
  the `Integrity` NULL-hash rule, §6.7 `default_document_id`, §6.8 `style_name` default `""`). No intention-book change: 10C step 5 and
  changelog 26 were written by the orchestrator on 2026-11-04 (uncommitted in the tree at planning time; the plan relies on them as the
  owner's decision). The P24 plan's C1(f) wording ("the default style listed") is history and is not edited.
- Estimated size: ~22 files, ~700 lines (half tests). One session; difficulty high (JXA, core, jobs, Shopify adapter, desktop entry).

## Review log
- **2026-11-04 implement r1 (codex).** Closed the throwaway after a silent Capture One open refusal; cleared stale front-document health when no session is active; exposed the fixed default document path; made the fresh style default empty; filled a missing RAW hash at integrity verification; initialized logging before app open; withheld unknown-ratio shape retries until the ratio is listed; and resolved newly created Shopify media by its ready filename. Backfilled the P24/P17 assertions and updated the old default-style expectation. All 12 P25 rows have boundary tests (10 in `p25_tidy_up_after_provisioning.rs`, one in desktop `p25_tests`, one in `p12_shopify_adapter.rs`). Full workspace suite, clippy, fmt, UI typecheck and UI suite passed. Assumptions: used `Settings::default_document_id` and a shared core `upload_name::matches`; retained the existing provider error wording except for the required filename and made the matcher require a v4 UUID as MC-11 says. The desktop logging test is the only desktop test that initializes the process-global tracing subscriber. Deviation: `apps/desktop/src-tauri/Cargo.toml` and `Cargo.lock` were needed to name the existing `tracing-appender` guard in `open_logged`'s return type. The P24 script harness was given a document `name()` because the script now reads it. No live Capture One or Shopify call was made.
- **2026-11-04 review r1 (opus): APPROVED_WITH_NOTES.** Gate: tracker `IMPLEMENTED`, checkpoint `b2f2ce8` in `git log`. Perimeter: `git show --stat b2f2ce8` = the handoff's 30 declared paths exactly (incl. the declared deviation `apps/desktop/src-tauri/Cargo.toml` + `Cargo.lock` for the `tracing-appender` guard type; master plan diff = own tracker row only). Suite on clean `5c1f62e` (code identical to `b2f2ce8`): `cargo test --workspace` 457 passed / 0 failed, clippy `-D warnings` and `fmt --check` clean, `npm run typecheck` clean, `npm test` 18 files / 129 tests. Verified correct, row by row (production read, then test): **C1(a)** `open_document.js` reads `currentDocument()` after `open`, closes only a front document whose name starts with `PSM Open Window`, then throws with the path; the catch cannot double-close because the front is gone; the harness asserts the exact ordered calls in all three modes, so an always-close, never-close or wrong-document close fails. **C2(a)** `EditingStartup::run` clears `document_not_front` in the no-active-session branch; the test sets the item by a refused activation and fails without the clear; camera status/capture recover. **C2(b)** `deactivate` clears it after `set_active_session_id(None)`; asserted immediately after deactivate and after a later check with no `front:` op. **C3(a)** `Settings::default_document_id` = `home/Pictures/Session Manager`, `effective_document_id` delegates to it; `settings_get` adds the key; DTO + bindings carry it. **C4(a)/(b)** default `style_name` `""`; explicit file value survives restart byte-identical and is applied once; absent key → `""`. **C5(a)–(c)** `verify` hashes a NULL-hash photo through `set_raw_hash` (fills NULL rows only) and persists `Ok`; a stored hash still yields `RawChanged` without overwrite; missing file → `RawMissing`, hash stays NULL; publish and export of an unhashed photo pass, with the hash written. Without the change all three fail (first verify would return `RawChanged`). **C6(a)** `open_logged` = create → `logging::init` → open; `build` logs `app open` with `data_dir` first; the test fails if init followed open. **C7(a)** `health_check` re-queues after `EditingStartup` and excludes `crop_ratio_unknown` while the item is set (SQL mirrors `count_jobs`); the real-start re-queue passes `None`; P23 C3(c)/(d) green. **C8(a)** `product_create_media` polls the `newest` candidate to `READY`, compares its CDN filename with the staged stem via the shared `upload_name::matches`, otherwise adopts the first gallery match, otherwise errors with the stem; a position-based implementation fails case 2. Timeout leaves the link `uploaded` with no id, and the next attempt waits for processing media and adopts by filename (M18 holds). Backfills (task 9) present as planned. Assumptions: `Settings::default_document_id` and `psm_core::mechanisms::upload_name::matches` (delegated choices); the matcher now requires a v4 UUID, which is consistent with `shopify_admin.md` SV-2 ("Uniquifier = `_` + a UUID v4"), and publish adoption narrows from any UUID to v4; real-start re-queue unchanged (plan task 7). None contradicts the intention. H1–H8: no new external write path (H1), hashing read-only via `sha256_file` (H2), no pixel work (H3), create-media resolution idempotent under crash (H4), Capture One only through its dictionary (H5), no commerce dependency added to the local path (H6), core names no Shopify/Capture One concept in the new code (`upload_name` is generic) (H7), Tauri side thin (`open_logged` is wiring) (H8). The suite reached neither Capture One nor Shopify (§6A). **Standing constraint for later desktop tests:** `p25_tests::c6a` is the only test in the `apps/desktop/src-tauri` test binary that may install the global tracing subscriber. Notes → backlog (4): silent-`open` detection not verified live; create-media fallback fails one attempt when our media is still processing; the one-subscriber constraint; `document_not_front` lingers when the editing check stops before the session step. No blocking findings.
