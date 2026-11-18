# 02 — Architecture: layers, crates, flows, persistence, events, frontend

Authorities: book `11_architecture_and_packaging.md` (layers), master plan §6 (names), the code.
Paths below are real at P70 (§Research: after R13); names in **bold** are the registry's.

## Layers and the dependency rule (book `11` §Layers; master plan §6.1 "Layer rule")
```
frontend (apps/desktop/src)        React/TS; calls services/*.ts → generated bindings; no business rules
Tauri edge (apps/desktop/src-tauri) commands: parse → service → serialize; events; menu; lifecycle
application services (crates/core/src/services)  orchestration over ports; callable from tests with fakes
core domain + mechanisms (crates/core/src/domain, mechanisms)  pure types and rules
ports (crates/core/src/ports)       traits: Store, Clock, EditingProvider, CameraProvider, CommerceAdapter, Imaging, RawPreviews,
                                    SecretStore, EventSink, CaptureSink, DeliveryFs, ProcessList, SleepInhibitor, StorageFs …
integrations (crates/integrations)  adapters implementing ports: Capture One, Sony sidecar, Shopify; plus the App wiring
infrastructure (crates/infra)       SQLite, files, osascript, FTP, Keychain, logging, jobs, imaging
```
Rules that follow: `psm-core` has no Tauri, SQL, HTTP or osascript types; nothing device-, app- or
platform-specific in core (H7; known debts are listed in `backlog.md` as "H7 family"); Tauri commands
are thin and run off the app lock (`spawn_blocking`); business logic never depends on Tauri (H8).

## Crates and their key modules
### `crates/core` (psm-core)
| Module | Holds |
|---|---|
| `domain/` | `item.rs`, `session.rs` (SessionStatus, PublishStep, RowLabel), `photo.rs` (Classification, Integrity), `attribute.rs`, `composition.rs` (10D values, Frame, Rect, EditorGeometry), `template.rs` (shooting templates, containers, presets), `output.rs` (OutputFormat), `publish.rs`, `camera.rs` (CameraStatus, Link, reasons, CameraSettings, Keychain account names, ledger constants), `editing.rs` (Job, ProvisionInputs, RecipeProps…), `storage.rs` (StorageSettings, ExportSettings) |
| `mechanisms/` (pure, unit-testable; the only place plans allow unit tests) | `arrival.rs` (candidate filter, path_key), `canon.rs` (NFC canonical path), `hash.rs` (SHA-256), `position.rs`, `status.rs` (status derivation, progress %), `time.rs`, `naming.rs` (folder key, export names), `template.rs` (item type key, `expected` place, `zoomed`, `preset_values`), `shot_order.rs` (P4 shot order: slots, counter runs, holds), `values.rs` (MC-9 canonical attribute values, MC-14 selection), `attribute_write.rs` (MC-13 diff), `reconcile.rs` (MC-12 gallery diff), `publish.rs` (frozen_main, upload waves), `upload_name.rs`, `staleness.rs`, `backoff.rs` (render backoff; `editor_wait` = "Waiting for Capture One…", 10 min max), `missing.rs` (files deleted outside the app), `composition.rs` + `composition_cases.json` (shared with the TS editor), `capture_ledger.rs` (MC-33 rules), `camera_status.rs` (precedence, sentences), `secrets.rs` (FTP password), `cleanup.rs` + `retention.rs` (10F decisions), `hook_payload.rs` |
| `ports/` | `store.rs` (Store/StoreTx; one impl: SQLite), `clock.rs`, `editing.rs`, `camera.rs` (CameraProvider, CaptureSink, ArrivalSource, ArrivalFollow, CompletedNames, DeliveryFs), `commerce.rs`, `images.rs`, `raw_previews.rs`, `secrets.rs`, `system.rs`, `storage.rs`, `events.rs` |
| `services/` | `arrival`, `review`, `photos`, `sessions`, `items`, `attributes`, `composition`, `templates` (local editor, each action saved immediately), `template_placement` (placement on arrival, the session's `TemplateView`, drop onto a container / retake, Back to template, Shift the rest back), `previews`, `export`, `prepare_masters`, `publish`, `history` (25 steps per session, in memory), `health`, `settings`, `integrity`, `editing_startup`, `editing_events` (hook dispatcher), `camera` (FTP address, "address changed"), `camera_ledger` (MC-33 worker), `storage` (10F clean-up/budget, denied files), `retention` (10F clearing, archived view) |
| `events.rs` | `AppEvent` (below); `error.rs`: `Error` with snake_case codes carried to the UI as `{code, message, data}` |

### `crates/infra` (psm-infra)
`paths.rs` (**AppPaths**, data dir), `settings_file.rs` (**Settings**, `settings.json`), `sqlite/store.rs`
(**SqliteStore**, migrations list) + `sqlite/migrations/NNN_*.sql`, `spool.rs` + `nudge.rs` (hook delivery),
`watcher.rs` (FSEvents/notify), `osascript.rs` (**OsaRunner**, 30 s timeout, cwd = data dir),
`jobs.rs` (**JobRunner**, `JobHandler`), `logging.rs` (daily file, secret redaction), `backup.rs`,
`rawpreview.rs` (embedded JPEG out of a RAW, MC-20), `imaging/` (the only pixel code: dimensions,
thumbnails, `render.rs` output renderer), `media_store.rs` (masters/outputs caches), `previews_store.rs`,
`export_fs.rs` (staging + swap), `storage_fs.rs`, `disk.rs`, `ftp/` (hand-written passive FTP receiver:
`server.rs`, `session.rs`, `peers.rs`, `names.rs`), `camera_files.rs` (`renamex_np(RENAME_EXCL)`, never
replaces), `keychain.rs`, `processes.rs` (`ps`, read-only), `sleep.rs` (IOKit idle-sleep assertion), `net.rs`.

### `crates/integrations` (psm-integrations)
| Module | Holds |
|---|---|
| `lib.rs` | **App** (every service, three job runners `jobs`/`publish_jobs`/`render_jobs`, background threads, `EventHub`, `Lanes`; plus the research runner `ResearchWiring`, §Research), **AppHandles** (clones for commands), `App::open` → `open_with_adapter_bind` → `build`; `apply_settings` (simple update vs full rebuild); `shutdown` |
| `editing/capture_one/` | `adapter.rs` (**CaptureOneAdapter** = EditingProvider), `scripting.rs` (**CaptureOneScripting** trait + DTOs), `jxa.rs` (**JxaScripting**: the `SCRIPTS` table, 24 entries, written to `<data>/hooks/jxa/*.js` at start, each prefixed with `jxa/lib.js`), `jxa/*.js`, `hooks.rs` + `hooks/*.applescript.tmpl` (compiled hook scripts), `gate.rs` (**GatedScripting**: user and background lanes, the one gate every script passes), `launch_gate.rs` (**LaunchGate**, held by the take-back), `follow.rs` (current collection + `sync_folder` on arrival), `mirror.rs` (colour tags), `renders.rs` (render bookkeeping), `provision.rs` (ensure sequence, error classification), `recipes.rs` (**RecipeSpec**: `PSM Preview` JPEG, `PSM Master` 8-bit TIFF uncompressed) |
| `camera/` | `runtime.rs` (**CameraRuntime**: one per process, owns supervisor, client, receiver, ledger, take-back, `tick`), `sidecar/{supervisor.rs, protocol.rs, client.rs, mod.rs (SidecarTimings, restart backoff)}`, `direct/{adapter.rs (DirectCameraAdapter), take_back.rs, ownership.rs, start_order.rs, filenames.rs}`, `support/{mod.rs, sony.rs, manifest.rs}` (support packages), `via_capture_one.rs` (fallback CameraProvider through Capture One) |
| `commerce/shopify/` | `adapter.rs` (**ShopifyAdapter** = CommerceAdapter; `ShopifyConfig`, `PollConfig`), `graphql.rs` (hand-written documents, one `pub const` per operation), `auth.rs` (client-credentials token, memory only), `transport.rs` (**HttpTransport**: reqwest / fake), `type_map.rs` (MC-13), `types.rs`, `errors.rs`, `fixtures/*.json` (sanitised captures replayed in tests) |
| `render_output.rs` | the `render_output` job handler (one output from a master) |

### `crates/testkit` (psm-testkit) — see `05_working.md` §Tests
`src/test_app.rs` (**TestApp**: temp data dir, real SQLite, `ManualClock`, `FakeCaptureOneScripting`
under the real adapter, `FakeCommerce`, `CollectingEventSink`, `FakeSecrets`, fake processes/sleep;
constructors `new`, `with_fake_document`, `with_fake_shop`, `with_camera*`), fakes, `fixtures/`,
`isolation.rs` (run one test in its own process), `logs.rs`, `bin/fake_sidecar.rs`.

### `apps/desktop/src-tauri` (psm-desktop)
`src/lib.rs`: `specta_builder()` (the command list), `TauriEvents` (AppEvent → Tauri event), `run()`
(opens the App, Edit menu with ⌘Z/⌘⇧Z, asset protocol scoped to `previews/` and `templates/`, window focus → preview
check, `camera_runtime.shutdown()` on exit), inline command tests, `export_bindings` test.
`src/state.rs`: **AppState** (`RwLock<App>`, `handles()`, `mutate()`, `settings_set`). `src/commands/*.rs`:
one file per area. `build.rs`: builds the stub sidecar if `binaries/camera-sidecar-aarch64-apple-darwin`
is missing. `tauri.conf.json`: `externalBin: binaries/camera-sidecar`, ad-hoc signing, min macOS 13.
`Info.plist`: `NSAppleEventsUsageDescription`, `NSLocalNetworkUsageDescription`.
Plugins: `tauri-plugin-dialog` (folder pickers) and `tauri-plugin-opener` (A6: web links open in the default
browser; `capabilities/default.json` allows `opener:allow-open-url` for `http://*` and `https://*` only). The
webview ignores `<a target="_blank">`: every link to a web page is `components/ExternalLink.tsx`
(`openExternal(url)`), never a bare `<a href>`.

## Runtime flows (step lists with the files involved)
### Shooting templates (book `10G`; T1–T4; contract `docs/implementation/master_plan_templates.md` §5 P1–P10)
- **Editor (T1).** Settings → Shooting templates (`features/settings/{SettingsNav.tsx, TemplatesSection.tsx,
  TemplateExample.tsx, PresetPreview.tsx}`) → `src/services/templates.ts` → eleven commands (incl.
  `template_container_label_set`: name + description, migration 016) → `TemplatesService`
  (`services/templates.rs`) over the `Store` port; every change is its own write (no Save). Example images
  are JPEG copies under `<data>/templates/<template id>/<uuid>.jpg` (`ImageOpsImpl::thumbnail`; the webview
  reads that folder through the asset protocol), shown black and white.
- **Placement on arrival (T2).** `ArrivalService::accept` → `template_placement::place_arrival` (one
  transaction; an error is logged and the photo stays where P66 put it). The session's shots (all photos,
  any classification, not `released`) and holds (reported shutters still expecting a file, ≤ 5 min) are
  ordered by `mechanisms::shot_order::order`; shot k goes to the container with `shoot_no` k
  (`mechanisms::template::expected`), beyond the last container → Condition. Only rows with
  `place_by_hand = 0` move; Main positions are renumbered dense by slot (`renumber_main`, P7). Presets
  become the photo's composition (P8); while the frame is unknown `pending_zoom` waits and
  `adopt_frame_tx` (`crates/infra/src/sqlite/store.rs`) applies it to the first rect. No undo step.
- **Hand changes (T3).** classify, reorder, Deny (`ReviewService`, `photo_place_mark`), a Compose edit
  (`frame_by_hand`) and a move/copy (row deleted) make a photo "by hand". `drop_on_container` fills an
  empty container or swaps two placed photos ("Move to container <n>"); `session_view` counts photos that
  differ; `back_to_template` re-places them (one `StepKind::Template` step "Back to template").
- **Shooting guidance (2026-11-16, after T4).** `session_view` also returns `next_shot`: the `shoot_no`
  the next shot would take: the first empty container left by a deleted shot (`gaps`) after the holds
  that will fill earlier gaps, else P5 applied to one past the shot order less the gaps; `None` past the
  last container. Display only; placement never reads it.
- **Retake and shift (T4).** A photo without a container dropped on a filled one: the old photo is denied
  as Deny does (red, undo window) and `released`; the dropped photo takes its container, its shot key
  (`key_at`, `key_name`) and the container's presets ("Retake container <n>"). Deny leaves the container
  empty (a denied photo still counts as a shot) until the next shot refills it: `place_arrival` calls
  `gap_for_arrival` (only when the arrival is the latest shot in the order) and `take_shot_key` (the
  denied photo is `released`, the arrival takes its `key_at`/`key_name`), so P5 puts the arrival in that
  container and nothing else moves; several gaps fill in shooting order (owner 2026-11-16 evening).
  `TemplateView.can_shift_back` says a later
  template-placed photo exists; `shift_back` releases the denied photos and re-places the template-placed
  ones ("Shift the rest back"). Every template step stores each photo's whole state (place, row,
  composition, denial) before and after; `HistoryService` restores it.
- **Commands** (`commands/templates.rs`): T1 `templates_list`, `template_get`, `template_create`,
  `template_delete`, `template_container_add`, `template_container_remove`,
  `template_container_example_set`, `template_containers_reorder`, `template_shooting_order_set`,
  `template_container_presets_set` · T2 `template_session_view` · T3 `template_drop`, `template_back` ·
  T4 `template_shift_back`. Error codes `template_exists`, `template_image_unreadable`, `no_template`,
  `nothing_to_shift`.
- **Frontend** (`features/workspace/`): `useTemplateView.ts` (re-reads `template_session_view` on
  `photo_added`, `photo_changed`, `session_changed`), `ContainerGrid.tsx` (faded numbered containers with
  their black-and-white example and "Shot n"; the container of `TemplateView.next_shot` is outlined "Next"),
  `TemplateInfoDialog.tsx` (the info button; Final order / Shooting order switch, next shot marked), `TemplateWarning.tsx` (the warning, Back to template,
  Shift the rest back), `templateDrop.ts` (`dropAction`: which drops are template drops), wired in
  `PhotoPanel.tsx`, `WorkspacePage.tsx`, `useWorkspaceState.ts`.

### Start-up (`psm_desktop_lib::run` → `open_logged` → `App::open` → `App::build`, `crates/integrations/src/lib.rs`)
1. `AppPaths::from_env_or_default` → `paths.create()` → `logging::init` (so the start is logged).
2. `JxaScripting::new` writes the 24 scripts to `<data>/hooks/jxa/`; one `LaunchGate`, one
   `StartOrder`, the two gated lanes and the one `CameraRuntime` are created for the process.
3. `Settings::open_lenient` (a bad file → defaults + health `settings_invalid`); `SqliteStore::open…`
   applies migrations; a backup is taken; `requeue_running_jobs` resets jobs left `running` by a crash.
4. Services and the three `JobRunner`s are built; `NudgeServer::start` on a loopback port; the hook
   scripts are compiled with that port (`hooks_compiled`); `spool.drain()` and `arrival.scan()`
   reconcile what arrived while the app was closed.
5. With `run_startup` (false in tests that do not want it): the `EditingStartup` thread (ensure the
   Capture One document, recipes, hooks; capture-folder switch), the commerce schema refresh thread,
   the camera thread (`camera_runtime.tick`), the general background loop (jobs, previews, follow,
   periodic scan/drain), the publish, render, prepare-masters and storage threads.
6. `camera_runtime.bind(AppBinding {…})` connects the runtime to this App; the Tauri `setup` installs
   `TauriEvents` as the event target and scopes the asset protocol to `previews/`.
7. A settings save that touches a runtime field builds a **new App** the same way (`App::prepare` →
   `PreparedSettings::Runtime`), transfers the event target, and shuts the old one down
   (`AppState::settings_set` in `src-tauri/src/state.rs`); the camera runtime, gate and lanes survive.

### Arrival (book `06` §Arrival, `10A` MC-15/MC-19/MC-20, `10E` MC-32/MC-33)
1. A RAW lands under the capture root `canon(<document folder>/Capture)`:
   - **Direct camera, Wi-Fi**: camera → `psm_infra::ftp` receiver (user `sessionmanager`, port 2121,
     passive 50000–50009, private peers) → staged as `.incoming/.<name>.part`, hashed → `LedgerService`
     (`services/camera_ledger.rs`, rules in `mechanisms/capture_ledger.rs`) decides session and final
     path (`<item folder>/<name>`; same bytes again → `.duplicates/`) → `camera_files::rename_exclusive`.
   - **Direct camera, cable**: the sidecar's Save-to-PC writes into `.incoming/sdk-<run id>/`; the
     `sdk.download_completed` event reaches the same ledger.
   - **Capture One tether**: Capture One writes the file; its `capture_done` hook (an AppleScript
     compiled from `hooks/capture_done.applescript.tmpl`) writes a JSON file into `<data>/spool/` and
     nudges the loopback listener (`infra/nudge.rs`); `Spool::drain` → `EditingEventsDispatcher`.
   - Fallbacks: `FsWatcher` on the capture root; startup scan (`ArrivalService::scan`).
2. `ArrivalService` (`services/arrival.rs`): `is_candidate` (RAW extension, no dot-prefixed folder),
   `path_key`, attribute to the active session else Unassigned; inserts the photo **approved** (P66),
   a `capture_events` row, queues `original_preview` + `hash` jobs; emits `photo_added`.
3. Original preview: `rawpreview::EmbeddedJpeg` copies the camera JPEG → `previews/<pk>/original.jpg`
   (P58: Compose works on it at once).
4. `CaptureOneFollow` (`follow.rs`): sets Capture One's current collection to the folder and calls
   `sync_folder` so the variant appears in seconds (P65); applies the Style once (`raw_state.style_applied_at`).
5. `PreviewService` (`services/previews.rs`): asks Capture One to `process` the variant with `PSM Preview`;
   the `processing_done` hook returns → `adjusted-<g>.jpg` + `thumb-<g>.jpg`; stale after an edit
   (`variant_adjusted` hook or settings-sidecar fingerprint), re-render after `preview_delay_s`
   (15 s, 10–30) or on window focus. `MirrorTag` writes the colour tag once the first preview exists.

### Review and history (book `06`, `05A` MC-30)
`commands/photos.rs` → `ReviewService`/`PhotosService` (`services/review.rs`, `photos.rs`) →
`mechanisms/position.rs` → `History` (`services/history.rs`, in memory, 25 steps, labels "Approve 02"…)
→ events `photo_changed`/`session_changed`/`history_changed` → UI `useWorkspaceState.ts`, `useHistory.ts`;
Edit-menu items via `edit_menu_set` and the `edit_menu` event. Deny (P66): `photo_denials` row; the
undo window closes at the next completed publish or after 24 h; `StorageService` then deletes the RAW,
previews, masters, outputs (rule `denied`) and `History::retire_photo` drops steps naming it.

### Compose and render (book `10D`)
Editor: `apps/desktop/src/features/compose/` (`geometry.ts` mirrors `mechanisms/composition.rs`, proven
on the same `composition_cases.json`; `machine.ts` gestures; `refit.ts`, `snapping.ts`) → commands
`composition_set/apply/reset` → `CompositionService` → `compositions` row (revision++, `origin=user`).
Render: `ExportService` asks Capture One for the **master** (`PSM Master`, once per RAW, cached at
`masters/<pk>/master-<adjusted_gen>.tif`; `PrepareMastersService` does this in the background once a
session is Ready and Capture One is idle, P69) → `render_output` job (`integrations/render_output.rs`
→ `infra/imaging/render.rs`: turn, flip, straighten, crop, resize, JPEG 4:4:4 sRGB with `output_format`
`{long_edge, quality}`) → `outputs/<pk>/<uid12>-<slot>-g<gen>-r<rev>-<format_key>.jpg`. Recipes are
checked and repaired before each render batch (10F §5).

### Export (book `07` §Completion, `05A` MC-8)
`export_run` → `ExportService::run` (`services/export.rs`): refuses with `needs_review{photos}` if a
composition needs re-framing; stages outputs (cached or freshly rendered) in a hidden folder beside
`<export folder>/<folder key>/`, writes `item.json` + `manifest.json`, swaps in (`infra/export_fs.rs`).
Progress event `export_progress{done,total,state}`; `state=waiting` while Capture One is not ready.

### Publish (book `07`, `07A`, `09`, `09A`; narratives P64, P65, P69)
1. `publish_preview` builds the confirmation (frozen main/condition lists, attribute diff, removals);
   `publish_confirm` writes `publish_runs` + `publish_items` (H1) and queues `publish_step` jobs on
   the **publish runner** (`App.publish_jobs`, own thread).
2. `PublishService` (`services/publish.rs`) runs the steps `prepare` (create the product for a local
   item, with stock if chosen; export = masters + outputs) → `upload` (`staged_uploads_create_many`,
   4 uploads at a time, `product_create_media_many`, `file_create_many` for condition files, poll) →
   `update` (gallery reconcile MC-12, metafields MC-13, damage report MC-14, title/type/tags/
   description/inventory read-back) → `done` (`published_at`, backup). Each step is idempotent and
   resumes after a crash (H4); `photo_links` rows track per-photo upload state and remote ids.
3. Waiting: a Capture One "not ready" error (`mechanisms/backoff.rs::editor_wait`) requeues with
   backoff 1/2/4/8 s then 15 s, label "Waiting for Capture One…", gives up after 10 min with a plain
   sentence. `publish_cancel` stops after the request in flight; `publish_activity` feeds the popover.
4. Shopify specifics: `ShopifyAdapter` over `graphql.rs` documents; token from `auth.rs`
   (client credentials, memory only); `api_version` from settings (default `2026-10`); files found by
   name (`FILES_BY_NAME`), inventory via `LOCATIONS`/`PRODUCT_SET`/`INVENTORY_READBACK`.
5. Log line at the end: `publish.timing item=<sku> masters_ms= outputs_ms= upload_ms= shopify_ms= total_ms= masters_reused=n/m`.

### Camera runtime, sidecar protocol, FTP receiver (book `10E` MC-31…MC-33; `native/camera-sidecar/README.md`)
- **One `CameraRuntime` per process** (`camera/runtime.rs`), created in `App::open_with_adapter_bind`
  and kept across settings rebuilds (R-I18); `bind(AppBinding)` connects it to the current App's store,
  health, events, capture root, editing provider, gate.
- Support installed? (`camera/support/mod.rs::installed`: `<data>/camera/sony/<ver>/` holds the seven
  Sony dylibs matching `native/camera-sidecar/manifest/sony-2.02.00.json` plus a copy of the bundled
  sidecar, re-signed ad hoc with `disable-library-validation`) → `Supervisor` spawns
  `camera-sidecar auto --save-dir <capture root>/.incoming/sdk-<run id> --usb-product-id …` (the helper
  is restarted when the document changes, P63); restart backoff 1/2/4/8/16 s then 30 s. Not
  installed → `CameraViaCaptureOne` is the provider.
- Protocol (`sidecar/protocol.rs`): stdout one JSON event per line (`camera.discovered`,
  `transport.state`, `connection.ready`, `session.lost`, `capture.*`, `usb.*`, `sdk.download_completed`,
  `ssh.fingerprint`, …), stdin one command per line (`credentials` first, then `capture`, `props`,
  `ftp-status`, `status`, `usb-retry`, `quit`); `client.rs` correlates command ids with outcomes.
  Fixture lines: `sidecar/fixtures/*.jsonl`; the test double is `testkit/src/bin/fake_sidecar.rs`
  replaying `testkit/src/fixtures/sidecar_scripts/*.txt`.
- `DirectCameraAdapter` (`direct/adapter.rs`) turns events into `CameraStatus` and the health codes it
  owns; `camera_files_not_received` belongs to the ledger watchdog. Fingerprints: the sidecar reports
  an unpinned Wi-Fi host key → `camera_fingerprint_pending` event → Settings/header card → confirm
  pins it under `<data>/camera/fingerprints/`.
- FTP receiver (`infra/ftp/`): started by the runtime with the Keychain password; a port change in
  settings reopens the listener in place. The "last upload address" (P68) drives the
  `camera_address_changed` health item (`services/camera.rs`).

### Take-back of the camera from Capture One (book `10E` MC-34; P48, P54, P57, P60, P63)
`camera_take_back` (or automatic on cable plug-in, setting `camera.auto_take_back`) → `TakeBackService`
(`direct/take_back.rs`): preconditions (`camera_take_back_availability`: no arriving file, no expected
photo, no running export/publish/preview render) → hold the `LaunchGate` → save the document → ask
Capture One to **quit** (`jxa/quit.js`; never kill) → `usb-retry` to the sidecar and wait for the USB
session → relaunch Capture One (`open -g -a`) → reopen the document → release the gate. While the gate
is held, every script in both lanes (`gate.rs`) is refused with `editor_launch_deferred`; `is_running`
is answered from the process list. At app start `StartOrder` (`direct/start_order.rs`) delays the first
Capture One launch until USB had its chance. `ownership.rs` names the holder (`ptpcamerad`, known programs).

### Storage and retention (book `10F`; P61, P62, P66)
`StorageService` (`services/storage.rs`, own thread): skips the tick while a capture, upload, render,
take-back, export or publish runs; deletes at most a batch per tick; rules in `mechanisms/cleanup.rs`
(duplicates ≥ 24 h, copied preview exports, unneeded masters, budget `storage.min_free_gb`) and
`mechanisms/retention.rs` (items published-and-clean for `retention_days_after_publish` days, after
`RetentionService` verified Shopify still holds every published image; `item_retention.keep_files` pin;
`archived_images` for the read-only archived view). Every deletion logs `storage.cleared rule=… path=… bytes=…`.

## Research (book `08A`, `08B` + changelog 12–18; `docs/implementation/master_plan_research.md` §5, §6)
Names below are real after R13. The engine is one isolated crate; app code changes only at the seams
listed in the research master plan §5 "Seams".

### `crates/research` (psm-research; depends on `psm-core` and `psm-infra`, never on integrations or the desktop crate)
| Module | Holds |
|---|---|
| `api.rs`, `api_impl.rs` | **ResearchApi** (the trait app code calls: `status`, `run`, `arranged`, `session_changed`, `settings_changed`, `settings_reset`, `validate_settings`, `secrets_set`, `secrets_state`, `site_add`, `site_remove`, `sites_reorder`, `export`, `purge_expired`) and its shapes (`ResearchStatus`, `RunView`, `StepView`, `Reason`, `SecretsInput`, `SecretsState`, `ExportSelection`); implemented for `ResearchService` |
| `domain.rs` | `RunState`, `RunTrigger` (`Arranged`, `Manual`; `Auto`/`FollowUp` are only read from old rows), `Stage` (observe, find, read, compare, labels, condition, decide), `StepKind`, `CallState`, `SiteClass`, `Verdict`, `ErrorClass`, `FieldClass`, `Prerequisite`, `CallFailure` |
| `config.rs` | **ResearchSettings** (the `research` slot of `settings.json`, read tolerantly by `from_slot`, checked by `validate`), **ResearchConfig** (the per-run snapshot, `from_settings` with the shop facts), the shipped defaults (stage models and efforts, prices, the detailed-condition table), `INSTRUCTIONS_VERSION`. With `adapters/`, the only place model and vendor names are literals (HA5) |
| `ports.rs` | the three provider ports **ModelProvider** (`call(&ModelRequest)`), **VisualSearchProvider** (`search(&ResearchImage)`), **PageReader** (`read(url)`), their replies (each carries an `Exchange {request, response}` for the call record) and `Providers` (an `Option` per port) |
| `adapters/` | `mod.rs::build` (adapters chosen by `research.providers.*`; default ids `openai`, `serpapi_lens`, `page_http`; keys read from the secrets port on every call), `http.rs` (the HTTP seam: `ReqwestResearchHttp` live, a fake in tests), `openai/` (Responses API, strict schemas), `visual_search/serpapi_lens.rs` (upload, then the Lens search), `page_reader/` (plain HTTP download, address guard, pacing of slow hosts, `extract.rs::product_extract`; A9: `Retry-After` waited only up to `SHORT_RETRY_AFTER_S` = 10 s, a 403/long 429 is `PageReply.blocked` and the run skips that host (`stages::read`); `shop.rs::ShopPageReader` wraps the reader in the app wiring and reads the connected shop's product pages through `CommerceAdapter::shop_product` (Shopify `productByIdentifier`), keeping the inner reader's id) |
| `rules/` (pure, inline unit tests) | `value_map` (MC-A2), `sites` (MC-A3: hosts, trusted sites, reading order, own listing), `fingerprint` (MC-A7, `RESEARCH_FINGERPRINT_VERSION`), `confidence` (MC-A4: `field_class`, `decide_*`), `cost` (MC-A8, integer micro-USD), `calls` (MC-A9: classify, retry action, redact; `RESEARCH_CALL_ATTEMPTS`), `write_table` (MC-A1: `never_written`, `decide_write`), `run_rules` (MC-A6: `first_failing`, `may_start`, `start_order`, `arranged_needed`), `derived_condition` (`derive`), `photo_plan` (`look_at`), `image_text` (MC-A11 export check: `scanned_parts`) |
| `runs.rs` | the life of a run around `execute`: when a run row is created (`arranged`, `run_again`), `tick` (start order), the start transaction, `status` for the panel |
| `stages.rs`, `decide.rs` | stages 1–6 as units (`execute` resumes at the first uncommitted unit) and the instructions each stage sends; stage 7 Decide (class, evidence, confidence, reasons, then the writes) |
| `gateway.rs` | **CallGateway::call**, the one door for every provider call: eligibility re-read, stop rule, call record and outbound rows are committed **before** the request leaves; reply, usage and cost are recorded after; retries, error classes, redaction |
| `writes.rs` | the one door for the form values research writes (MC-A1), inside the writing transaction; no undo step |
| `images.rs`, `files.rs` | **ResearchImage** (no public constructor: `ResearchImages::render` makes one only from an eligible photo's adjusted preview through `Imaging::render_research`, so a RAW or a master can never reach a port, MC-A5); the file store `<data>/research/images/<sha[0..2]>/<sha>.jpg` |
| `store.rs`, `schema.rs` | the engine's SQL and `schema::ensure` (§Storage below) |
| `service.rs`, `job.rs` | **ResearchService** (gateway, renderer, snapshots; the Keychain account names) and **ResearchJob**, the `jobs` handler of kind `research` (target = run id; an offline call holds the job, back-off at most 60 s) |
| `shop.rs`, `export.rs`, `retention.rs` | shop facts through `CommerceAdapter::public_listing`; the export of the research record (folder, then zip; scanned for secrets, fail closed; an image only where text can live, by its leading bytes: JPEG header, PNG text chunks, WebP EXIF/XMP, GIF extensions, AVIF/HEIF boxes but `mdat`, `rules::image_text::scanned_parts`, A7/A8; no matching run → `Error::NothingToExport`, nothing written); deletion of call records and their images after `record_days` |
| `fakes.rs`, `fixtures/` | scripted stand-in providers for tests; sanitised provider captures written by R1 (`fixtures/<provider>/`), never by hand |

### A run, step by step
1. **Trigger.** `session_arrange_next` (`commands/sessions.rs`) → `crates/integrations/src/arranged.rs`
   `arrange_and_next`: `SessionsService::arrange` sets `sessions.arranged_at` (first press only; no
   undo step, no `change_seq` bump), then `ResearchApi::arranged` queues a run (trigger `arranged`)
   when no blocking prerequisite fails, the session has a Main photo, no run is already queued, and
   the photo-derived input differs from the latest started run's snapshot
   (`rules::run_rules::arranged_needed`). `publish_preview` marks unmarked sessions the same way
   (`arranged::preview_for_publish`) and reports `researching` per item. `research_run` (the panel's
   Research / Run again) queues a `manual` run. A photo change creates no run.
2. **Prerequisites** (`rules::run_rules::first_failing`, first failing wins; the panel shows its
   sentence): no integration · no model key · no visual-search key · a price missing · nothing to
   research · no field chosen · day limit · offline; plus `WaitingForEditor` while a run waits for
   Capture One's adjusted previews (`source_wait_seconds`).
3. **Start.** The research runner (`ResearchWiring` in `crates/integrations/src/lib.rs`: its own
   thread waking every 250 ms, its own `JobRunner` limited to kind `research`, one turn at a time)
   calls `ResearchService::tick`: one run runs at a time; the start transaction sets `running`,
   freezes the settings, form and input snapshots on the run row and inserts the `jobs` row.
4. **Stages** (`stages::execute`): 1 **observe** (one call over at most `observe_max_photos` photos,
   Main first) → 2 **find** (visual search on the cover, Main position 1) → 3 **read** (pages in
   reading order: shop, trusted, untrusted; until `max_pages` are obtained) → 4 **compare** (the cover
   against each obtained page that has images and lists the same kind of piece) → 5 **labels** (one
   unit per photo stage 1 saw a marking on) → 6 **condition** (one unit over the Condition photos; no
   call when no condition field is chosen) → 7 **decide**. Each unit commits its output, its steps
   and the stage reached in one transaction, under a fingerprint; a committed unit never runs again
   (crash-safe, H4), and an `arranged` run reuses an earlier run's unit with the same fingerprint (a
   `manual` run reuses nothing). The cost stop rule (`max_cost_per_run`, `max_cost_per_day`) ends a
   run as `stopped_at_cap`.
5. **Decide and write** (`decide.rs`): per layout field its class, the evidence from the committed
   outputs, the confidence rules; one model call writes the reasons; one transaction applies the
   writes through `writes.rs` (`ai` + `suggested`; with an `ai_prior` when it replaced a default or
   shop value) together with the `concluded` / `not_determined` steps. `AppEvent::ResearchChanged` →
   Tauri event `research_changed {session_id}` → the form re-reads `research_status`.
   **Status across sessions (A6, 2026-11-17).** `research_status` also carries `queue` (`QueueView`:
   place in the MC-A6 start order, the running session's id and "name · SKU", `manual`) for a queued run.
   `research_overview(session_id?)` → `ResearchService::overview` (`runs.rs`): one pass over the latest
   run per unpublished session, the suggestion counts per item (the layout is read only for items holding
   a suggestion, through `derived_counts`) and the queued line; states `researching` / `waiting` (+
   position) / `to_review` / `done` / `failed`, plus `blocked` (+ the first failing creation
   prerequisite) for an arranged session with no run. The sessions list (`useResearchOverview(null)`,
   `features/sessions/ResearchPill.tsx`) and the session header (`useResearchOverview(id)`) re-read it on
   `onSessionsChanged` (which includes `research_changed`). Never N `research_status` calls for a list.
6. **Afterwards.** The person's Looks right / Clear / edit / Accept all / conflict resolution go
   through core (`services/attributes.rs`, `services/history.rs`; commands `attribute_ai_accept`,
   `attribute_ai_accept_all`, `attribute_conflict_resolve`), are undo steps and write
   `research_feedback` rows (MC-A12). Publish splits an item's values with
   `mechanisms/publish_partition.rs` (frozen / withheld / pending) and holds an item with pending
   conflicts at `PublishStep::Held` (MC-A10).

### Field classes (`rules::confidence::field_class`; first match wins)
never-written (MC-A1, `rules::write_table::never_written`: the Product step or group; media-list,
reference, rich-text, money and rating types; a non-writable field; the product title) · **not
chosen** (key not in `fill_keys`; treated exactly as never-written) · **derived**
(`detailed_condition.key`) · **condition** (the Condition group) · **attribution**
(`attribution_keys`; HA2: never above `low` on looks alone) · **measured** (dimension, weight) ·
**visible** (everything else).

### Storage
- Every research table is in the app database and written inside the app store's transactions
  (`SqliteStore::research_transaction` hands the closure the `StoreTx` and the raw connection).
  **Nothing is in the numbered migration list.** Two idempotent steps create the schema:
  `crates/infra/src/sqlite/ensure.rs` `ensure_research_columns` (run by the store at every open):
  `attribute_values.ai_prior`, `publish_items.{frozen_attribute_origin, pending_conflicts,
  resolved_attributes, released_at}`, `sessions.arranged_at`, table `research_feedback`; and
  `psm_research::schema::ensure` (called by `build_research` in integrations `lib.rs`): tables
  `research_runs`, `research_steps`, `research_calls`, `research_outbound`,
  `research_stage_outputs`. Columns: research master plan §6.4.
- Settings: the `research` key of `settings.json` is an opaque JSON slot in `psm_infra`
  (`Settings.research`); its shape is `ResearchSettings`. The engine reads it live, so a save is a
  simple update; only a change of `research.providers` rebuilds the App.
- Keys: Keychain only, under the app's one service, accounts `research-model-key`,
  `research-search-key`, `research-reader-key` (`crates/research/src/service.rs`).

### App-side touchpoints
Core: `domain/research.rs` (`Feedback`, `FeedbackAction`), `mechanisms/{name_key.rs, sameness.rs,
publish_partition.rs}`, `PublishStep::Held`, `RowLabel::WaitingForYou`, `AppEvent::ResearchChanged`,
`Imaging::render_research` (implemented in `crates/infra/src/imaging/research.rs`),
`CommerceAdapter::public_listing`. Commands: `apps/desktop/src-tauri/src/commands/research.rs`
(`research_status`, `research_run`, `research_settings_reset`, `research_secrets_set`,
`research_secrets_state`, `research_site_add`, `research_site_remove`, `research_sites_reorder`,
`research_export`; each answers `research_unavailable` when the engine could not be built, and the
rest of the app works: HA4), `session_arrange_next` in `commands/sessions.rs`, `ResearchSettingsDto`
in `commands/settings.rs`, `researching` / `n_withheld` / `conflicts` in `commands/publish.rs`.
Frontend: wrapper `apps/desktop/src/services/research.ts`. There is **no `features/research`
folder**: the panel and pills are in `features/form/` (`ResearchPanel.tsx`, `AiPopover.tsx`,
`AiPill.tsx`), the settings in `features/settings/` (`ResearchSection.tsx`, `ShopFieldPicker.tsx`),
conflicts in `features/publish/ConflictList.tsx`, the button in `features/shell/Header.tsx`, the
"Arranged" pill in `features/sessions/SessionRow.tsx`.

### The tool (`crates/research-tool`, package `psm-research-tool`, bin `psm-research`)
A dev-Mac CLI over the same crate (one-way: nothing in the app depends on it). `accept` opens the
real `App` headless on a scratch data folder with a stand-in read-only shop, runs the real engine
and adapters over the test pieces and exports each run; `score-accept` scores the exports against
the answer key; `report` summarises an export (or a benchmark recording); `replay` re-sends one
recorded stage of one run, optionally with another model, effort or instructions file; `probe`,
`bench-search`, `bench-models` are R1's benchmarks. Commands: `05_working.md` §Research tool.

## Persistence
| Where | What |
|---|---|
| `~/Library/Application Support/com.loorenz.sessionmanager/` (`PSM_DATA_DIR` overrides; `AppPaths`, master plan §6.3) | `session_manager.db` (SQLite, WAL) · `settings.json` (0600) · `previews/<pk[0..2]>/<pk>/{original.jpg, adjusted-<g>.jpg, thumb-<g>.jpg}` · `masters/…/master-<g>.tif` · `outputs/…` · `hooks/*.scpt` + `hooks/jxa/*.js` · `spool/`, `spool/bad/` · `logs/psm.<yyyy-mm-dd>.log` · `backup/session_manager-<stamp>.db` (last 30) · `camera/sony/<ver>/` (support package + sidecar copy) · `camera/fingerprints/` · `camera/staging/` · `research/images/<sha[0..2]>/<sha>.jpg` (research copies of photos, listing images) · `templates/<template id>/<uuid>.jpg` (shooting-template examples) |
| Under the capture root (`<document folder>/Capture`) | item folders by `folder_key`; `.incoming/` (FTP parts, `sdk-<run>/`); `.duplicates/`; Capture One's own `CaptureOne/` and `Output/_session_manager/` |
| SQLite schema | `crates/infra/src/sqlite/migrations/001_init.sql … 015_photo_places.sql`, applied by `rusqlite_migration` at `App::open` (`SqliteStore::migrations()`); numbering is sequential, files are append-only (a new change = `014_*.sql` + a new `M::up(include_str!)` line in `sqlite/store.rs`). Tables: master plan §6.5 (+ 008 `raw_state.settings_fingerprint`, 009 `publish_items.frozen_item_type`/`item_links.item_type_base`, 010 `item_retention`/`archived_images`, 011 `item_stock`/`publish_item_stock`, 012 `photo_denials`, 013 `attribute_saved_values`/`item_remote_titles`, 014 `shooting_templates`/`template_containers`, 015 `photo_places`; 014 and 015 use `CREATE TABLE IF NOT EXISTS`, names in `master_plan_templates.md` §4) |
| `settings.json` (`psm_infra::settings_file::Settings`; keys master plan §6.8) | `camera {receiver_port, passive_port_start, passive_port_end, auto_take_back}`, `document_id`, `preview_recipe`, `master_recipe`, `output_format {long_edge, quality}`, `style_name`, `default_shape`, `export_folder`, `preview_delay_s`, `theme`, `sku_prefix`, `integration {kind, shop_domain, api_version, client_id, client_secret, condition_photos_key}`, `storage {min_free_gb, retention_days_after_publish}`, `export {prepare_masters}`, `commerce {inventory_default}`, `research` (an opaque slot; shape in §Research). A file that does not parse never stops the app (health `settings_invalid`) |
| Keychain (`infra/keychain.rs`; names in `domain/camera.rs::secrets`) | service `com.loorenz.sessionmanager.camera`, accounts `access-auth-user`, `access-auth-password`, `ftp-password`, and the three `research-*-key` accounts (§Research). What research redacts and the export looks for (`integrations::research_secret_values`): the three `research-*-key` accounts plus the client secret, nothing of the camera's (A9: the short FTP code turned up by chance in records and image headers). Shopify client id/secret: `settings.json` only; access token: memory only |
| `settings` table | app state, not user config (`mechanisms_version`, `active_session_id`, spool dedupe keys) |

## Events: Rust → UI
`psm_core::events::AppEvent` → `EventHub` (integrations `lib.rs`) → `TauriEvents::emit` (desktop
`lib.rs`) → Tauri event names: `session_changed`, `photo_added`, `photo_changed`, `publish_progress`,
`export_progress`, `health_changed`, `camera_status_changed` (payload `CameraStatusResponse`),
`camera_fingerprint_pending`, `history_changed`, `research_changed`; plus `edit_menu` emitted by the menu handler. The UI
subscribes in `apps/desktop/src/services/events.ts` (and `history.ts`). Events are **not** in the
specta builder: adding one means a new `AppEvent` variant, a new match arm in `TauriEvents`, and a
listener in `events.ts`.

## Generated TypeScript bindings
`apps/desktop/src/services/bindings.ts` is generated by `tauri-specta` from `specta_builder()` (every
command in `collect_commands![…]`, `#[tauri::command] #[specta::specta]`). Regenerate with
`cargo test -p psm-desktop export_bindings`; the committed file must equal the regenerated one; never
hand-edit. Components never import `bindings.ts` directly: `services/*.ts` wrap `commands.*` and
`unwrap` the `Result` into `ServiceError {code, message, data}` (`services/sessions.ts`).

## Frontend structure (`apps/desktop/src`)
- `App.tsx`: hash routes `#/` (sessions), `#/session/:id` (workspace), `#/health`, `#/settings`;
  opening a session activates it (`session_activate` switches Capture One's capture folder); back on
  the list deactivates (captures → Unassigned). `lib/routes.ts`: `goTo` + leave guard (Settings
  unsaved-changes prompt). `lib/keyboard.ts`, `lib/pending.ts` (`useAction` lock), `lib/format.ts`.
- `features/`: `shell` (Header, CameraDot/CameraCard, NotificationsBell, ArrivalToasts, ThemeToggle),
  `sessions` (list, filters, batch bar, NewSessionDialog), `workspace` (Stage, PhotoGrid, PhotoPanel,
  SessionsPanel, review buttons/keys/swipe, ArchivedView, `useWorkspaceState.ts`, `order.ts`,
  `outputs.ts`; shooting templates: ContainerGrid, TemplateInfoDialog, TemplateWarning, `templateDrop.ts`,
  `useTemplateView.ts`), `compose` (editor; see above), `form` (FormDialog, StepTabs, MetafieldInput, kinds,
  ItemTypePicker, StockFields, AI pills, ResearchPanel), `publish` (PublishPopover, PublishProgress, BatchQueue,
  FailurePopover, NotificationsDrawer, `usePublish.ts`), `settings` (SettingsPage, CameraSection,
  FingerprintBox, StorageSection, StockSection, ResearchSection, ShopFieldPicker, `cameraSteps.ts`; a drawer of sections: SettingsNav, TemplatesSection, TemplateExample), `health`, `history`, `pending`.
- `components/ui/` shadcn primitives; `components/pending/` skeletons; `styles/tokens.css` from the
  design handoff. **Drop-downs** are `components/ui/select.tsx` (Radix Select; `ChoiceSelect` for plain
  value/label lists, "" = none), never a native `<select>` (owner 2026-11-16). A typed field with
  suggestions (the item type, `features/form/ItemTypePicker.tsx`) is a combobox: an input plus the app's
  own list in a Popover (chevron, arrows, Return, `hints` per option), never a `<datalist>`. Tests pick options with
  `src/test/select.ts` (`choose`, `chooseValue`, `optionsOf`); `src/test/setup.ts` adds the jsdom no-ops Radix needs.
- State: React hooks per feature; no global store. Preview files are loaded through the Tauri asset
  protocol (`convertFileSrc`), scoped to the `previews/` folder in `lib.rs`.
- Tests: vitest + jsdom + Testing Library, `features/<f>/__tests__/<phase>_<slug>.test.tsx`,
  services mocked with `vi.mock`; pure-TS mechanisms (`geometry.ts`, `machine.ts`, `refit.ts`,
  `snapping.ts`, `workspace/outputs.ts`) have unit tests.

## Logging
`psm_infra::logging::init` (called by `open_logged` before `App::open`): one file per day,
`<data>/logs/psm.<date>.log`, `tracing` text format, every registered secret replaced by `[REDACTED]`
(`add_secret`: the Shopify client secret, tokens, camera credentials). Provider calls log their script
name and `duration_ms` (`osascript.rs`); camera lines use target `psm::camera` (`ftp.upload …`,
`camera.event …`); storage lines `storage.cleared` / `storage.item_cleared` / `storage.not_cleared`;
publish `publish.timing`; Capture One sync `capture_one.sync` / `capture_one.variant_seen after_ms=`.
The camera helper's own stderr is kept in memory for the Health page (`camera_helper_log`).
