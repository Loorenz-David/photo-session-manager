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
| `ports/` | `store.rs` (Store/StoreTx; one impl: SQLite), `clock.rs`, `editing.rs`, `camera.rs` (CameraProvider, CaptureSink, ArrivalSource, ArrivalFollow, CompletedNames, DeliveryFs), `commerce.rs`, `images.rs`, `raw_previews.rs`, `secrets.rs`, `system.rs`, `storage.rs`, `trash.rs` (A17: the macOS Trash), `events.rs` |
| `services/` | `arrival`, `review`, `photos`, `sessions`, `items`, `attributes`, `composition`, `templates` (local editor, each action saved immediately), `template_placement` (placement on arrival, the session's `TemplateView`, drop onto a container / retake, Back to template, Shift the rest back), `previews`, `export`, `prepare_masters`, `publish`, `history` (25 steps per session, in memory), `health`, `settings`, `integrity`, `editing_startup`, `editing_events` (hook dispatcher), `camera` (FTP address, "address changed"), `camera_ledger` (MC-33 worker), `storage` (10F clean-up/budget, denied files), `retention` (10F clearing and its guards; the archived view is `RemoteItems::view`, H4) |
| `events.rs` | `AppEvent` (below); `error.rs`: `Error` with snake_case codes carried to the UI as `{code, message, data}` |

### `crates/infra` (psm-infra)
`paths.rs` (**AppPaths**, data dir), `settings_file.rs` (**Settings**, `settings.json`), `sqlite/store.rs`
(**SqliteStore**, migrations list) + `sqlite/migrations/NNN_*.sql`, `spool.rs` + `nudge.rs` (hook delivery),
`watcher.rs` (FSEvents/notify), `osascript.rs` (**OsaRunner**, 30 s timeout, cwd = data dir),
`jobs.rs` (**JobRunner**, `JobHandler`), `logging.rs` (daily file, secret redaction), `backup.rs`,
`rawpreview.rs` (embedded JPEG out of a RAW, MC-20; `JpegSource` for a photo whose file is a JPEG, MC-H4.6), `imaging/` (the only pixel code: dimensions,
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
**Search this photo window (A12).** Tauri's multiwebview (`tauri` feature `unstable`): `src/search_window.rs`
opens one window per session, `search-<session>`, holding two webviews: `search-ui-<session>` (the app's own
bundle on `#/search/<session>/<photo>`; `src/main.tsx` renders `features/search/SearchWindow.tsx` there instead of
`App`, so nothing of the main window runs in it; capability `capabilities/search.json`, local only) and
`search-web-<session>` (web pages; placed by the app's page over its browser area with `search_web_place`, hidden
while the window's own list shows). The page webview: `on_navigation` = `allowed_navigation` (every frame: public
http/https only, never the app's origins, localhost, the local network, `file:`/`asset:`/`ipc:`); new windows open
in place; downloads refused; `on_page_load` / title changes → events `search_web_page`, and the Google Lens route
(`SearchWindows`, managed state; `lens_on_page`: Google's start page → consent → post the photo with
`upload_script` (a same-site form post to `lens.google.com/v3/upload`, field `encoded_image`) → the results page
`www.google.com/search?…`) → event `search_lens`. The selected text is read by the app's page asking
(`search_web_selection`: `SELECTION_SCRIPT` evaluated in the page, the answer checked by
`parse_selection_report`). Commands `src/commands/search.rs` (`search_window_open`, `search_web_place`,
`search_web_go`, `search_web_step`, `search_lens_start`, `search_lens_give_up`, `search_lens_results`,
`search_web_selection`, `search_matches`, `search_visual`, `search_pages`, `search_page_save`,
`search_page_remove`, `search_put_text`, `search_ask`; A14 `search_put_back`, `search_open_review`); frontend
wrapper `services/search.ts`. Saved pages:
`saved_pages` (migration 018) through `crates/infra/src/sqlite/saved_pages.rs`.

**A13 corrections to that window (owner's first live test).**
- **Layout.** On macOS the window's content view runs under the title bar: probed, it is 900 pt tall with 32 pt
  of it under the title bar. A webview at y = 0 therefore hid its top behind the title bar, and the page
  webview covered the toolbar.
  - The app's page is not `auto_resize`d (Tauri scales that by the whole content size). `window::lay_out` reads
    the content view and `contentLayoutRect` natively (`objc2`, main thread) and places the page at
    `ui_bounds(width, height, title_inset)`. It runs on open and on every `Resized` / `ScaleFactorChanged`, and
    keeps the result in `SearchWindows.layout`.
  - `search_web_place` gets the browser area in the app page's CSS pixels (logical points, whole numbers via
    `browserBounds`) and places the page webview at `web_bounds(area, ui)`, offset and clamped inside the app's
    page.
  - The app's page watches every row of the browser column (`ResizeObserver`), so a bar appearing, growing or
    going above the area moves the webview. It sends a place only when it changed.
- **Browser identity.** The page webview sends `PAGE_USER_AGENT` (desktop Safari). With WebKit's default, which
  lacks `Version/… Safari/…`, Google serves basic pages, and the Lens post lands on an empty search page without
  `udm=26` (probed).
- **Lens route.** The photo stays in `LensState.photo` until results or give-up, and is posted again when the
  route, still waiting, lands on a Google page that isn't the results. That happens after the cookie page,
  which swallows the first post on a fresh profile (seen live), or after Google's home page. At most
  `MAX_POSTS` = 2 posts per search, then the window's own list. `nav_kind` names each loaded page by host and
  kind only.

**A14 corrections to that window.**
- **New windows open in place.** WebKit hands a page's new-window request to `on_new_window` with its address,
  but an empty one (`window.open('')`, then the address set on the returned window) arrives as `""`, which
  Tauri cannot parse and denies before the app sees it; `about:blank` would have loaded a blank page over the
  page (probed with wry 0.57.1 in a harness). Every frame of the page webview now runs
  `src-tauri/src/page_popups.js` at document start (`initialization_script_for_all_frames(popup_script())`):
  `window.open` with any target but `_self`/`_top`/`_parent` navigates the top page to the address (http/https
  only) and returns a stand-in window whose `location` (set, `href`, `assign`, `replace`) does the same; links,
  areas and forms aimed at a new window are re-aimed at `_top` in the capture phase. It reads and sends
  nothing; `allowed_navigation` still decides. What still reaches `on_new_window` goes through `new_window`
  (`OpenHere` for public web pages; `Ignore` for blank windows and anything else), logged by host.
  `refusal_reason` names why a navigation was refused (`empty`, `local_address`, `not_a_web_page`).
- **Selection bar** (`features/search/SelectionBar.tsx`). The page webview is a native view over the browser
  area, so nothing of the app's page can be drawn over it: the bar, its field list and the AI prompt stay in a
  fixed-height band under the area (no height change, so the page never jumps), tooltips are native `title`s
  and toasts sit bottom-left over the photo panel. "Put in" answers what the field held (`search_put_text` →
  `PutTextDto.previous`); the toast's Undo calls `search_put_back` (set it again, or clear). "Open AI review"
  calls `search_open_review`: the main window comes forward and gets `search_open_review`, which
  `features/search/useReviewFromSearch.ts` turns into the details on the AI review tab (opening that session
  first when another is shown).

**A18 corrections to that window.**
- **Search tabs.** Each search runs in its own page webview, `search-web-<session>-<tab>` (`web_label(session,
  tab)`; tab ids come from the app's page, at most `MAX_TABS` = 8, `window::ensure_tab` makes one on first use,
  `window::close_tab` / `search_tab_close` closes it). `SearchWindows.lens` is keyed by `(session, tab)`, so each
  tab has its own Lens route, posts and photo; `forget_tab` / `forget(session)`. Every command that touches a page
  takes `tab` (`search_web_place` also hides every other tab of the session, so exactly one shows), and the events
  `search_web_page` / `search_lens` carry it. The window opens with no page webview: its app page starts tab 1 on
  the photo it was opened on; a later "Search this photo" from the main window (`search_photo`) pins that photo and
  opens a new tab. Pinning another photo in the window only changes the reference picture; "Search this photo" on
  the pinned photo (hover) or "+" opens a new tab. Only the current tab's selection is read.
- **Zoom.** Page zoom per tab (`search_web_zoom(session, tab, current, action)` → `next_zoom` over `ZOOM_STEPS`
  50–300 %, applied with `Webview::set_zoom`; the app's page keeps each tab's level). The View menu has Zoom In
  (⌘+ as `CmdOrCtrl+NumpadAdd`, also ⌘=), Zoom Out (⌘−) and Actual Size (⌘0); `on_menu_event` sends `search_zoom`
  to the focused search window's app page only (`session_of_window`), which applies it to its current tab: menu key
  equivalents reach the app also while the page has the keyboard, and the page itself still talks to nothing. The
  pinned photo zooms in the app's page (`features/search/ZoomablePhoto.tsx`: wheel/pinch, click, drag, double-click).
- **Ask AI answers in words** (`AnswerCard.tsx`). `search_ask` takes `mode`: `suggest` (A12) or `answer`
  (`browse::AskMode::Answer`: instructions `ASK_ANSWER`, input `answer: true`, the Read schema then requires
  `answer` next to `keys`; nothing is written; `AskAnswer.answer` + `found` with each value's stored JSON). A found
  value is written only from a tap, through `search_put_value`, the same path as `search_put_text` (`put`, a
  person's edit with `previous` for Undo). The selection cap is 6,000 characters (`MAX_SELECTION_CHARS`; the page
  script cuts at 6,001 and the answer limit `MAX_REPORT_BYTES` is 64 KiB); `search_window.ask_ai_sent` logs mode,
  characters and the cap.

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

### Capture target switches (A10)
`session_activate` / `session_deactivate` take a ticket from `CAPTURE_TARGET_ORDER`
(`services/sessions.rs`, `SwitchOrder`) when called and run one at a time; a request a newer one
has overtaken is skipped, so overlapping switches (opening sessions one after another) cannot end on
a stale session. Each Capture One step logs `capture_one.switch_step`, each switch `capture_one.switch`.

**A16 (owner 2026-11-18):** with a camera that does not shoot through Capture One, the request first
**claims the filing target** (`SwitchOrder::claim` + `SessionsService::claim_target`, a short lock of
its own, never behind a running Capture One switch; `arrival.target` logs every change), then waits for
its turn for the Capture One steps (`with_target_claimed` keeps an older switch finishing late from
taking the target back). The switch's last step, `collection_switched`, only notes the switch: the
follow-up runs on the background loop (`CaptureOneFollow::tick` → `switch_follow_up` →
`PreviewService::session_opened`) and looks only at **the opened session's photos**
(`Store::preview_states_in_session`), logged as `capture_one.switch_follow_up`. Per-switch cost:
the Capture One calls plus one query and one preview pass over that session's photos, never the
library. A20: the Capture One calls do not grow with the number of sessions either:
`document_info.js` reads a fixed handful of properties (no collection list), `lib.js`
`collectionForFolder` reads every collection's folder in one Apple event (`doc.collections.folder()`),
and `ensure_provision` verifies the recipes and hooks once per Capture One process (pid from the
process list, `CaptureOneScripting::instance`), again after a restart, a failed ensure, a failed
render-time recipe repair, or the Health page's check (`EditingProvider::recheck_next`); the capture
style is still checked on every switch. `capture_one.switch_scripts` logs the scripts per switch. The recurring preview pass (`schedule_due`) and the focus sidecar check read
`preview_states_live` (no deleted photos, no published-and-unchanged or cleared sessions); the
library-wide missing-file check (`check_missing_files`, a file-exists check per RAW) runs only at
startup, window focus and Capture One reopening. Each preview-state read is one SQL query (it was one
query per RAW).

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
   - Fallbacks: `FsWatcher` on the capture root; startup scan (`ArrivalService::scan`); a scan when
     the watcher reports dropped events (`WatchEvent.rescan`, FSEvents "must scan subdirs").
   - **Thread (A10):** the capture watcher and `ArrivalService::tick` run on their own thread
     (`App.arrival_background`), never behind Capture One work; stability counts looks
     (`STABILITY_CHECKS` = 120), not wall time. With the background loop running, the arrival's
     Capture One follow-up goes through `DeferredFollow` (`follow.rs`): queued, then run by the
     loop (`flush` before `CaptureOneFollow::tick`); without it (tests) it stays inline. A16: the
     cheap part (`ArrivalFollow::arrival_noted`: style eligibility, wait for the variant) runs at once
     on the arrival thread, so the photo's `render_preview` job cannot render it before its Style.
2. `ArrivalService` (`services/arrival.rs`): `is_candidate` (RAW extension, no dot-prefixed folder),
   `path_key`, attribute to the active session else Unassigned; inserts the photo **approved** (P66),
   a `capture_events` row, queues `original_preview` + `hash` jobs; emits `photo_added`.
3. Original preview: `rawpreview::EmbeddedJpeg` copies the camera JPEG → `previews/<pk>/original.jpg`
   (P58: Compose works on it at once). **A photo's file may be a JPEG** (an imported JPEG, or a picture brought
   back from the shop, H3 / MC-H4.6): the `original_preview` job (`OriginalPreviewJob`, `integrations/src/lib.rs`)
   picks `rawpreview::JpegSource` for a `.jpg`/`.jpeg` target (any case): the JPEG itself when its long edge is at
   most 1600 px, else a copy scaled to 1600 px (never enlarged, pixels never rotated); `orientation` = its EXIF
   orientation; frame = its pixel size (width/height swapped for orientations 5–8) adopted as `FrameSource::Camera`,
   which Capture One's first preview replaces (P58 rule). Every other extension takes `EmbeddedJpeg`. The watcher's
   candidate rule is unchanged: **a `.jpg` written into a capture folder is never an arrival** (RAWs only).
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
   description/inventory read-back, then the search texts: `update_seo` in `services/publish.rs`,
   writer master plan WC8.4 — only when a frozen `product.seo_title` / `product.seo_description` is
   non-blank; reads `product_seo`, writes both fields in one `update_seo` only when one differs
   (trimmed), never clears a field without a frozen value, reads back and fails with
   `attribute_verification` naming the key) → `done` (`published_at`, backup). Each step is idempotent and
   resumes after a crash (H4); `photo_links` rows track per-photo upload state and remote ids.
2a. **Details-only publish (H4, MC-H3.6).** For a session whose item is `Remote(_)` (archived or shop-only;
   an `Unknown` adopted item is read first with `RemoteItems::ensure_record`, a press, wired by
   `PublishService::set_remote`) `preview` answers no photos, no thumbnails, no announced removals,
   `create_item` false and `details_only` true, with no `item_get`; `confirm` (the P62 refusal is gone)
   freezes empty photo lists, skips the in-transaction photo comparison and stores
   `publish_items.details_only = 1` (migration 021). The run reads the flag from the stored item on every
   resume: `prepare` and `upload` do nothing, `update` skips the gallery block (`update_gallery`) and leaves
   every media-list value (the condition photos) out of the writes (`update_details`); title, type, tags,
   description, metafields and search texts are written as for any publish. The item stays remote.
   **Rows (MC-H3.7):** with `with_remote`, a remote row counts its record (`RowPictures::remote_row`, from
   the one `remote_covers` read: gallery / condition counts), `unreviewed_count` 0, and its status input
   has `main_count` = gallery, `unreviewed` 0, missing previews 0 (`ready_session_ids` keeps the raw input);
   the label is the publish label while a publish is running, held, failed or cancelled, else `Archived`
   (archived rows via `cleared_sessions`, also without `with_remote`).
3. Waiting: a Capture One "not ready" error (`mechanisms/backoff.rs::editor_wait`) requeues with
   backoff 1/2/4/8 s then 15 s, label "Waiting for Capture One…", gives up after 10 min with a plain
   sentence. `publish_cancel` stops after the request in flight; `publish_activity` feeds the popover.
4. Shopify specifics: `ShopifyAdapter` over `graphql.rs` documents; token from `auth.rs`
   (client credentials, memory only); `api_version` from settings (default `2026-10`); files found by
   name (`FILES_BY_NAME`), inventory via `LOCATIONS`/`PRODUCT_SET`/`INVENTORY_READBACK`.
5. Log line at the end: `publish.timing item=<sku> masters_ms= outputs_ms= upload_ms= shopify_ms= total_ms= masters_reused=n/m`;
   the SEO step logs `publish.seo item=<publish item id> written=<bool>` (W5).
6. Search texts (W5): `product.seo_title` (Text) and `product.seo_description` (Textarea) are built-in
   Product fields, **always present** (`ALWAYS_PRESENT` + `append_always_present` in
   `services/attributes.rs`, applied by `layout_keys`, `form_get`, `presets_get`); read from the linked
   product as `integration` rows like the description (`RemoteValues.seo_*`, Shopify `seo` through
   `adapter::seo_of`); `CommerceAdapter::product_seo` / `update_seo` (`PRODUCT_SEO_READ`,
   `PRODUCT_UPDATE_SEO`; evidence `shopify_admin.md` §SEO search texts).
7. Restoration done (W6, writer master plan WC8-R): `local.restoration` (MultiChoice, 14 fixed choices
   `RESTORATION_CHOICES`) is a built-in **Condition** field, always present at the end of the Condition
   step (`ALWAYS_PRESENT`; with no stored preset, `active_definitions` lists non-Product built-ins after
   the shop's fields). App-local: never published (`write_set` only walks the shop's definitions), never
   researched (`rules::write_table::never_written`), in `item.json`. The writer sees it as a field and
   post-check rule 11 (`HitCode::Restoration`, phrases `writer::defaults::RESTORATION_PHRASES`) requires
   each chosen step in the condition report.

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
`archived_images`, the record a remote item is shown from, H4). Every deletion logs `storage.cleared rule=… path=… bytes=…`.
H1 (10H): retention never clears a shop-only item, nor an item brought back after its last completed publish
(`RetentionInput.shop_only`, `restored_at`); the verification records each picture's size (`archived_images.bytes`)
and is not started for further items of a scan while the call budget is low (`must_wait`, scan requested again).

### Remote items (book `10H`; master plan `master_plan_archive.md` §4, §5 MC-H3, MC-H5; H1)
An item is **remote** when its pictures are only in the shop: **archived** (`item_retention.cleared_at` set) or
**shop-only** (adopted from the search, linked, its session has no photo row, and its first read found pictures).
`mechanisms::remote::remote_state(RemoteInput)` is the one rule (`Store::remote_input(session_id)` reads its input in
one query); `Unknown` = adopted and not read yet. The **record** of remote pictures is `archived_images` for both
kinds (gallery then condition, each by position, with `bytes`); `item_retention.remote_checked_at` says it was read.
`RemoteItems` (`services/remote.rs`, `App.remote`, also in `AppHandles`; always present, `Local` without an
integration): `ensure_record` (the first-open read, one `media_readback` under one lock), `refresh` (Refresh from the
shop: remote items only, else `NotRemote`; record replaced with `cleared_at` kept, stale `remote/<item>/cover-*` and
`archived/<item>/*` cache files deleted, names by `retention::archived_file_name`), `queue_record` + `run_pending`
(the background step, thread `remote_background` started with the startup work like `storage_background`; queued
reads wait while `must_wait(budget())`, logged once as `remote.budget_wait available= maximum=`). **The call budget:**
the Shopify adapter keeps the newest `extensions.cost.throttleStatus` of every answer (also throttled ones) and
`CommerceAdapter::budget()` projects it (`projected`); background work checks `must_wait` (below 25 %), a press never
does. The adapter also has `media_urls` (one `nodes(ids:)` query per 25 ids) and `fetch_original` (full size);
downloads answer `Error::ImageGone` on 403/404 (`HttpTransport::download` returns the status).
**Row pictures (H2, MC-H1).** One rule for every list and search answer, `RowPictures::row_picture` (read once per
answer by `RemoteItems::row_pictures`, `Store::remote_covers` = one query for every session): main photo 1's
adjusted preview if its file exists, else its original preview if its file exists, else, for a remote item whose
cover picture (first gallery picture of the record) is not gone, `previews/remote/<item>/cover-<12 hex>.jpg` if it
exists (`cover_pending` when it does not), else none. **No path whose file is missing is ever handed to the screen**
(M-H4); without `with_remote` `SessionsService::list` applies the preview half only (`remote::local_picture`).
**The queue:** `request_covers(CoverSource {Page, Search}, sessions)` replaces that source's not-started cover
downloads with the pending ones and returns at once; `run_pending` takes them as one batch, at most
`PARALLEL_DOWNLOADS` (4) at once (`std::thread::scope`), each `fetch_image(url, COVER_EDGE 320)` → `<name>.part` →
rename → the item's other `cover-*.jpg` deleted → `AppEvent::PictureReady {Cover {session_id}, path}` (window event
`picture_ready`). **Stale rule:** a download answering `ImageGone` is stale; after the batch, when `must_wait` allows
(else kept for the next step: `Waiting` only when nothing else ran), the stale media ids not asked for in the last 10
minutes (`REFRESH_COOLDOWN_MS`, monotonic, in memory) go to `media_urls` in chunks of 25; a found address is stored
(`update_remote_urls`) and retried once, a missing one stores the picture as **gone** (empty URL: never downloaded
again until the record is replaced) and puts the item on Health `remote_picture_gone` (`1 picture is no longer in the
shop: <SKU>`); any other download error is logged only. The item leaves that Health item when its record is replaced
(first-open read, Refresh from the shop) or its session is deleted (`forget_item`, which also removes
`previews/remote/<item>/` and `previews/archived/<item>/`; no shop call).
**Pages and search pictures (H2, MC-H2).** `SessionsService::page(filter, query, created_from, only_ids, page)` (command
`sessions_page`, the Sessions page only) filters `list`'s rows (time filter, "To review" ids), cuts them to `PAGE_SIZE`
(25) by `mechanisms::remote::page_of` (clamped, at least one page; groups keep their `total`) and requests the page's
pending covers; `sessions_list` stays unpaged for the other callers. `items_search` answers each local result's row
picture and session (`RemoteItems::item_pictures`, pending covers requested with source `Search`); `items_search_shop`
keeps the shop's `featuredMedia` address (`ItemsService::search_shop_pictures`) and answers
`previews/remote/search/<search_name(url)>` when it exists, else queues it (`search_picture`: `fetch_image(url,
SEARCH_EDGE 160)`, `PictureReady {Search {external_id}}`; a gone picture is dropped, never refreshed). **The search
cache** is one flat folder capped at `SEARCH_CACHE_LIMIT_BYTES` (200 MB): after each write the file's modified time is
set to the app clock and `evict` deletes the oldest files (name ascending on a tie) until the folder fits.

**Bring back (H3, MC-H4; `services/bring_back.rs`, `impl RemoteItems`).** `RemoteItems::bring_back(session_id)`
(`RemoteItems::new` also takes `editing`, a `capture_root` closure and `history`; `App::build` passes a closure that calls
`editing.inspect_document()` and canonicalizes its capture root, **uncached**): (1) preconditions in order: an
integration (`NoIntegration`), state `Remote(_)` (an `Unknown` item is read first with `ensure_record`) else `NotRemote`,
no bring back of the session running (in-memory mark with its progress, `bring_back_progress`; cleared on every exit;
`BringBackRunning`), the latest publish not in held/prepare/upload/update/describe, the capture root known, no record
picture gone (each a `BringBackFailed {reason}`); (2) names `mechanisms::remote::bring_back_names` in
`<capture root>/<folder key>/` (`taken` = a photo row of any session has that canonical path; a store error fails before
any download); (3) every picture `fetch_original(url)` → `<name>.part` → rename, at most 4 at once, 1 + `DOWNLOAD_RETRIES`
(2) tries, a body not starting with `FF D8 FF` is a failed try; `ImageGone` → one `media_urls` for the stale ids (no budget
wait), the new address stored and tried with the full tries, none found → stored gone and `<n> photos are no longer in the
shop`; `AppEvent::BringBack {Running, done, total}` after each file (window event `bring_back_progress`, `onBringBack`);
any picture missing → **every file this attempt wrote is deleted**, nothing imported, `BringBack {Failed}`; (4)
`Store::bring_back(BringBackInput)` — **one transaction**, run inside `History::retire_session`'s `act` (the session's undo
and redo lists go): the session's earlier rows retired (deleted, no position, preview paths null, `raw_missing`, links for
the item deleted, `photo_places.released = 1`), one photo per file (canonical path, SHA-256, `source import`, composition
`original` with rect (0,0,1,1) revision 0, `raw_state` row, jobs `original_preview` + `render_preview`; in a templated
session main *k* placed by hand in container *k*), one `ready` link per photo (role `gallery` or
`attribute:<condition key>`, the stored `raw_state.retouch_gen`, revision 0), `item_retention` `cleared_at` null and
`restored_at` now; **`change_seq` untouched**; (5) `editing.sync_folder(folder)` (failure → `synced = false`),
`BringBack {Done}`, `SessionChanged`. No style, no colour tag, no shop write (M-H1), no editor delete (M-H2).

**The remote view and activation (H4, MC-H3.2–5, .8; replaces P62's archived view).** `RemoteItems::view(session_id)`
(command `session_remote_get`, `commands/remote.rs`) is a read plus a queue push, never a download and never a shop
call: `None` without an item; `Local` → no pictures; `Unknown` → no pictures and the first-open read queued; `Remote(_)`
→ the record's pictures, each with `previews/archived/<item>/<role>-<NN>-<12 hex>.jpg` when that file exists (`gone`
when its URL is empty), `bring_back_bytes = estimate_bytes(record)`, `check_failed` (the last first-open read failed;
in memory, cleared by a successful read) and the running bring back's progress. Missing files are queued as `View`
downloads (a new view of a session replaces only that session's not-started ones) and run in `run_pending`'s batch
like covers: `fetch_image(url, ARCHIVED_IMAGE_EDGE 1600)`, 4 at a time, the same stale rule, `PictureReady {Remote
{session_id, role, position}, path}`; a queued download whose file already exists is skipped. A failed queued
first-open read emits `SessionChanged` only when `check_failed` turns true (the screen reloads on it and its view
queues the read again). `session_remote_refresh` = `refresh` then `view`; `session_bring_back` = `bring_back`.
`item_keep_files_set` answers the remote view (`RemoteItems::set_keep_files`). **Activation:**
`SessionsService::activate` and `claim_target(Some(id))` read the state first (`with_remote`, else the store when the
service has a commerce integration) and, for `Remote(_)`, answer at once (`Ok` / `false`) with no editing call, no
camera target and no change of the active session; `session_activate` attaches `with_remote(app.remote)`. Every other
state activates as before. **Other readers (M-H4):** `SessionsService::view` answers no photos (nor compositions or
upload states) for a remote session; the conversations' attachable photos carry a preview path only when its file
exists (`remote::local_picture`); a media-list attribute write is refused with `FilesCleared` for a remote item
(`AttributesService` reads the state from the item's session when an integration is set).

**Delete session (A17, owner 2026-11-18; Session Manager only, never Shopify).** Commands
`session_delete_preview` / `session_delete` → `psm_integrations::session_delete` (when photos are filed
into the session: `CAPTURE_TARGET_ORDER` claim + `deactivate`, so Capture One leaves it; failure only logged)
→ `StorageService::delete_preview` / `delete_session` (`services/storage/session_delete.rs`, a child module
of the storage service: it holds the clean-up worker's lock and drops the item's queued files). Under
`History::retire_session` it checks the guards again, moves the filing target off the session, sends each
path to the `ports::trash::Trash` (infra `MacTrash`: `NSFileManager trashItemAtURL`; the suite's
`FakeTrash`, registered per app by `psm_integrations::with_test_trash`), then the item's export folder and
the capture folder when only Capture One's data is left, and only when every file moved calls
`Store::delete_session_rows` (`infra/sqlite/session_delete.rs`, one transaction, research tables by name
when they exist).

**Photo files follow their session (A21, owner 2026-11-18).** `psm_integrations::photo_files` wraps every
change of a photo's session: `photos_move` → `move_photos` (refused up front with `Error::FilesBusy` while
an export or publish step of a session involved runs), `photos_assign_unassigned` → `assign_unassigned`,
`history_undo`/`history_redo` → `after_history`; each then calls `StorageService::place_files`
(`services/storage/photo_files.rs`, a child module like `session_delete`: under the clean-up lock, per
misplaced photo it moves the RAW, `CaptureOne/Settings*/<name>.*` and `CaptureOne/Cache/**/<name>.*`
through `StorageFs::rename` (never over an existing file; on failure what moved goes back), the app's
path-keyed preview/master/output folders, then `Store::relocate_raw` (one transaction: photos with
their preview paths, `raw_state` with its key, change reasons, jobs); the editor's queued renders follow
(`EditingProvider::raw_moved`), a RAW it is rendering waits (`rendering`). Capture One is then asked to
re-read both folders (`sync_folder`; the capture root's collection when the target has none yet). A RAW
another session uses (a copy) or outside the capture root stays. Commands `session_files_misplaced` /
`session_files_fix` back the ⋯ menu's **Fix photo files location** (shown only when needed). The adapter
also asks Capture One to re-read a folder (once a minute at most) when a photo on disk is not in its list
of that folder (`variant_background` → `resync_folder`).

### Conversations with Pricing (book `09B`; contract MC-C1 in `docs/implementation/master_plan_conversations.md` §5; C1)
- **Port and vocabulary:** `crates/core/src/ports/conversations.rs` (`ConversationsProvider`: `stores`, `unresolved`,
  `conversation`, `events_after`, `post_message`, `mark_answered`, pure `describe`, pure `attribute_key` (C4: a quote's
  field name → the form's key; the mapping lives in `BeyoPricingAdapter::attribute_key`, H7); `ConversationStore`, one
  transaction per method, with C4's read `form_keys(session_id, integration)` = the form's field keys of the session's
  item, over core's `services::attributes::form_keys` (`layout_keys` restricted to active definitions, as `form_get`
  shows them); `ConversationSettingsSlot`), `domain/conversation.rs`, `mechanisms/conversation_rules.rs`
  (`check_backoff`, `unread`/`has_news`, `entering_read_mark`, `notice_lines`).
- **Quote chip jump (C4):** `ConversationsService::view` fills the view-only `ConversationView.jump_keys` (message id →
  key) when the conversation's session is on this Mac and `form_keys(session, integration)` has the key `attribute_key`
  gives (`integration` = `settings.integration.kind`, handed to the service at build); never stored. The command maps
  it into `quote.jump_key`; `QuoteBlock` makes the chip a button only when the conversation's session is on screen, and
  its click (`openField` in the conversations context) opens Details at that field (`FormDialog` `field`, a counter so
  every click jumps; a field the form has wins over the A19 AI-review landing).
- **Adapter:** `crates/integrations/src/conversations/beyo_pricing/` — `BeyoPricingAdapter` over `PricingHttp`
  (`transport.rs`; `ReqwestPricing`: own worker thread and runtime, connect 10 s / total 20 s, the key only in
  `Authorization: Bearer`), wire types `types.rs` (camelCase, unknown fields ignored), fixtures `fixtures/*.json` (the
  package's examples), `checked_server`, `DEFAULT_SERVER`, every sentence that names Pricing (`describe`).
  `conversations/mod.rs`: the slot over `settings.json` (`SettingsFileSlot`), the redaction-registering secret store
  (`RedactingSecrets`), the test seam `with_test_conversations(paths, transport)`, `build_service`.
- **Service:** `crates/core/src/services/conversations.rs` (`ConversationsService`, `App.conversations`, also in
  `AppHandles`; always present). State derived at read (`no_shop` / `no_key` / shop- or server-mismatch
  `store_unmatched` are memory-only; only connect and check outcomes are written to `conversation_sync`). The key is
  read from the Keychain once per App build (first `tick` or `connection()`, on a worker thread) and held in memory.
  `test_connection` / `key_set` call `stores()` only; `tick` checks iff the state is `ready` and the check is due:
  first connection (page the log from 0, list unresolved, read the named rest, save the cursor), then one events
  page per `tick`, each event's change and the cursor in one transaction. One process-wide **lane**
  (`ConversationsLane`, carried in `Lanes` through rebuilds) is held for one request and its apply; `connection()`
  never takes it.
- **Thread:** `conversations_background` in `App::build` (only when `run_startup`): `tick` every 250 ms, at once while
  `more`; joined in `shutdown` within `BACKGROUND_JOIN_BOUND` (a request in flight is not waited for).
  `TestApp` runs without it; tests call `tick` with `ManualClock` on `TestApp::with_pricing()` (`FakePricing`,
  `crates/testkit/src/fake_pricing.rs`).
- **Commands:** `apps/desktop/src-tauri/src/commands/conversations.rs` (`conversations_settings_get`,
  `conversations_settings_save`, `conversations_key_set`, `conversations_test`; `Unreachable`/`BadReply` →
  `CommandError` code `provider` in the adapter's words); UI `features/settings/PricingSection.tsx`, wrapper
  `services/conversations.ts`. C2 adds `conversations_list(scope, tab)`, `conversations_counts(session_id)`,
  `conversations_session_badges` over `ConversationsService::{list, counts, session_badges}`: read from the copy only
  (never the lane, never a request); a conversation's session = the session of the item whose SKU is exactly its SKU
  (`ConversationStore::item_sessions`), none = "Not on this Mac"; hidden in `no_key`, `no_shop`, `store_unmatched`.
- **Replies: the outbox and the lane (C3, MC-C2).** `ConversationsService::reply` checks the reply
  (`conversation_rules::checked_reply`) and writes one `conversation_outbox` row keyed by the frontend's
  `client_mutation_id` (`sending`, or `failed` with the state's sentence when refused / store gone); the same id again
  returns that row. `tick` **flushes** the `sending` rows first (after the `ready` test, before the due test), oldest
  first, each `post_message` and its outcome in one lane turn: sent → row `sent` + the message appended once + status
  `answered`; time-out / odd answer → `Couldn't send` for it and every later row of the flush without a request; 400 →
  the provider's words; 404 → "no longer exists"; 401 / store gone → MC-C1.8. A `failed` row leaves only by `retry`
  (same id) or `discard`. `view`, `reply`, `retry`, `discard` and `attachable` are local (never the lane, never a
  request); `mark_answered` takes the lane (`PATCH`; 409 re-reads the conversation). Every outbox change emits
  `ConversationsChanged` without notices. Attachable photos come from `ConversationStore::session_links` (the
  session's item's links with each photo's place and previews), filtered and ordered in the service. Commands
  `conversation_get`, `conversation_mark_read`, `conversation_reply`, `conversation_retry`, `conversation_discard`,
  `conversation_mark_answered`, `conversation_attachable`; `ConversationError` → codes `conversations_off`
  (`data.reason` = the state; every C3 call in `no_key` / `no_shop` / `store_unmatched`, and Mark answered when not
  `ready`), `conversation_resolved`, `provider`.
- **Panel and icon (C2):** `apps/desktop/src/features/conversations/` — `useConversations` (one instance in `App.tsx`,
  provided through `ConversationsContext` to every route: panel open, selected conversation, scope, header counts),
  `ConversationsButton` (header icon on Sessions, a session, Settings, Health), `ConversationsPanel` (Open / Resolved,
  mute button = the `sound` setting), `ConversationRow`, `SessionBadge` (+ `useSessionBadges`, Sessions list),
  `ConversationToasts` (sonner toasts with Open; 8 s, or until the next window focus + 10 s when in the background).
  Everything re-reads on `conversations_changed`.
- **The visit (C4, MC-C3):** `features/conversations/visit.ts` holds route and visit as **one** state
  (`{route, visit: {origin, capture, visited} | null}`), changed only by the pure `visitStep(state, action)` (actions
  `open-session`, `back`, `shoot-here`, `route-changed`). `App.tsx`'s `useNav` keeps it in one `useReducer`: a step
  computes the next state from a ref, writes the hash, then dispatches (so `PageHeader`, which reads the hash during
  render, already sees the new route); a `hashchange` that only confirms the route does nothing, any other is an
  ordinary navigation (`route-changed`, which ends a visit). `useCaptureTarget` receives `captureTarget(state)` (the
  visit's `capture` while visiting, else the route's session) and is the only caller of `activateSession` /
  `deactivateSession`. `openSession`, `visitBack`, `shootHere` and `visit` reach the panel through
  `ConversationsContext`; **Open session** runs through `lib/routes.ts` `leaveThen(target, proceed)`, so a dirty
  Settings page (whose guard receives the continuation) runs the jump itself from Discard / Save. `VisitBar.tsx` is the
  bar (capture SKU from `activeSession()` and the sessions list).
- **Chime and banner (C2, the alerts seam):** `apps/desktop/src-tauri/src/alerts.rs` (`trait Alerts {chime, banner}`,
  `SystemAlerts`, `banners(lines, more)`); `TauriEvents` holds an `Arc<dyn Alerts>` and a focus reader (an `AtomicBool`
  kept by the main window's `Focused` events). On `ConversationsChanged`: forward `conversations_changed`; `chime` →
  one `alerts.chime()`; `banner` and the window not focused → one banner per line (title "Session Manager").
  The chime is `psm_infra::chime::wav()` (generated, no sound file in the repo), written as `chime.wav` into the data
  folder when absent or different and played with `/usr/bin/afplay` on a thread of its own (never blocks the bridge).
  Banners go through `tauri-plugin-notification` (registered in `run()`, used from Rust only: no capability entry, no
  npm package); macOS asks once to allow them and never tells the app a refusal.

## Research (book `08A`, `08B` + changelog 12–18; `docs/implementation/master_plan_research.md` §5, §6)
Names below are real after R13. The engine is one isolated crate; app code changes only at the seams
listed in the research master plan §5 "Seams".

### `crates/research` (psm-research; depends on `psm-core` and `psm-infra`, never on integrations or the desktop crate)
| Module | Holds |
|---|---|
| `api.rs`, `api_impl.rs` | **ResearchApi** (the trait app code calls: `status`, `run`, `arranged`, `session_changed`, `settings_changed`, `settings_reset`, `validate_settings`, `secrets_set`, `secrets_state`, `site_add`, `site_remove`, `sites_reorder`, `export`, `purge_expired`, A12 `browse_matches`, `browse_search`, `browse_ask`) and its shapes (`ResearchStatus`, `RunView`, `StepView`, `Reason`, `SecretsInput`, `SecretsState`, `ExportSelection`); implemented for `ResearchService` |
| `domain.rs` | `RunState`, `RunTrigger` (`Arranged`, `Manual`; `Browse` (A12) holds one call of the search window and is never queued, started or listed by `store::runs`; `Auto`/`FollowUp` are only read from old rows; W1 `Look` (**Look again**: stages 1 and 6 only, no write; left out of `store::runs`, the overview, `status` and a research run's reuse; tier 0 of the start order beside `manual`)), `Stage` (observe, find, read, compare, labels, condition, decide), `StepKind`, `CallState`, `SiteClass`, `Verdict`, `ErrorClass`, `FieldClass`, `Prerequisite`, `CallFailure` |
| `config.rs` | **ResearchSettings** (the `research` slot of `settings.json`, read tolerantly by `from_slot`, checked by `validate`), **ResearchConfig** (the per-run snapshot, `from_settings` with the shop facts), the shipped defaults (stage models and efforts, prices, the detailed-condition table), `INSTRUCTIONS_VERSION`. With `adapters/`, the only place model and vendor names are literals (HA5) |
| `ports.rs` | the three provider ports **ModelProvider** (`call(&ModelRequest)`), **VisualSearchProvider** (`search(&ResearchImage)`), **PageReader** (`read(url)`), their replies (each carries an `Exchange {request, response}` for the call record) and `Providers` (an `Option` per port) |
| `adapters/` | `mod.rs::build` (adapters chosen by `research.providers.*`; default ids `openai`, `serpapi_lens`, `page_http`; keys read from the secrets port on every call), `http.rs` (the HTTP seam: `ReqwestResearchHttp` live, a fake in tests), `openai/` (Responses API, strict schemas), `visual_search/serpapi_lens.rs` (upload, then the Lens search), `page_reader/` (plain HTTP download, address guard, pacing of slow hosts, `extract.rs::product_extract`; A9: `Retry-After` waited only up to `SHORT_RETRY_AFTER_S` = 10 s, a 403/long 429 is `PageReply.blocked` and the run skips that host (`stages::read`); `shop.rs::ShopPageReader` wraps the reader in the app wiring and reads the connected shop's product pages through `CommerceAdapter::shop_product` (Shopify `productByIdentifier`), keeping the inner reader's id) |
| `rules/` (pure, inline unit tests) | `value_map` (MC-A2), `sites` (MC-A3: hosts, trusted sites, reading order, own listing), `fingerprint` (MC-A7, `RESEARCH_FINGERPRINT_VERSION`), `confidence` (MC-A4: `field_class`, `decide_*`), `cost` (MC-A8, integer micro-USD), `calls` (MC-A9: classify, retry action, redact; `RESEARCH_CALL_ATTEMPTS`), `write_table` (MC-A1: `never_written`, `decide_write`), `run_rules` (MC-A6: `first_failing`, `may_start`, `start_order`; A19 `arranged_queues`), `derived_condition` (`derive`), `photo_plan` (`look_at`), `image_text` (MC-A11 export check: `scanned_parts`) |
| `runs.rs` | the life of a run around `execute`: when a run row is created (`arranged`, `run_again`), `tick` (start order), the start transaction, `status` for the panel |
| `stages.rs`, `decide.rs` | stages 1–6 as units (`execute` resumes at the first uncommitted unit) and the instructions each stage sends; stage 7 Decide (class, evidence, confidence, reasons, then the writes) |
| `gateway.rs` | **CallGateway::call**, the one door for every provider call: eligibility re-read, stop rule, call record and outbound rows are committed **before** the request leaves; reply, usage and cost are recorded after; retries, error classes, redaction |
| `writes.rs` | the one door for the form values research writes (MC-A1), inside the writing transaction; no undo step |
| `browse.rs` (A12) | the search window's research parts: `lens_image` (the photo framed, reduced to 1600 px, no metadata; else the camera's original preview), `browse_matches` (the session's latest answered visual search, no call), `browse_search` (one visual search of the pinned photo), `browse_ask` (Ask AI: one `read`-stage model call on the selected text, unit `browse`, writes through `writes::suggest` at `medium` with the page as source). Each call is a finished `browse` run through the gateway (record, cost, day limit, export); `store::runs` leaves browse runs out, `store::all_runs` (the export) keeps them |
| `images.rs`, `files.rs` | **ResearchImage** (no public constructor: `ResearchImages::render` makes one only from an eligible photo's adjusted preview through `Imaging::render_research`, so a RAW or a master can never reach a port, MC-A5); the file store `<data>/research/images/<sha[0..2]>/<sha>.jpg` |
| `store.rs`, `schema.rs` | the engine's SQL and `schema::ensure` (§Storage below) |
| `service.rs`, `job.rs` | **ResearchService** (gateway, renderer, snapshots; the Keychain account names) and **ResearchJob**, the `jobs` handler of kind `research` (target = run id; an offline call holds the job, back-off at most 60 s) |
| `shop.rs`, `export.rs`, `retention.rs` | shop facts through `CommerceAdapter::public_listing`; the export of the research record (folder, then zip; scanned for secrets, fail closed; an image only where text can live, by its leading bytes: JPEG header, PNG text chunks, WebP EXIF/XMP, GIF extensions, AVIF/HEIF boxes but `mdat`, `rules::image_text::scanned_parts`, A7/A8; no matching run → `Error::NothingToExport`, nothing written); deletion of call records and their images after `record_days` |
| `writer/` (book 08C; W1) | `note.rs`: the product note research's stages 1 and 6 now store (`note` in their unit outputs, WC1), `decision` / `reviewed` (the owner's standing on a key, WC2), `assemble` (`NoteView`: observations with the owner's decisions replacing what research saw, `research_saw`, `stale_main` / `stale_condition` / `missing_main`, computed at read time), `look_again` (queues a `look` run); `api.rs`: **WriterApi** (`note`, `look_again`; W2 `request`, `answer`, `dismiss`), implemented for `ResearchService` in `api_impl.rs` and handed by the wiring as `AppHandles::writer`. W2 (engine only, behind the fakes; nothing in the app runs it before W3): `config.rs` (`WriterSettings`, the `writer` slot, `ResearchDeps.writer_settings`), `defaults.rs` (the shipped guide and examples), `input.rs` (what the writer sees), `postcheck.rs` (the deterministic check of every text), `run.rs` (a `write` run: request, start order, `Stage::Write` call, one rewrite, writes, findings), `writes.rs` (the writer's write path over MC-A1's table), `safeguard.rs` (the never-re-raise filter), `schema.rs`. W3 (in the app): `api.rs` adds `status` (`WriterStatus`: the latest `write` run, its texts with hits, attempts, rewrites, the item's open findings, steps, cost, inputs, staleness, why Write texts would refuse); `ResearchDeps.writer_settings` reads the live `writer` slot of `settings.json` (`build_research`) |
| `fakes.rs`, `fixtures/` | scripted stand-in providers for tests; sanitised provider captures written by R1 (`fixtures/<provider>/`), never by hand |

### A run, step by step
1. **Trigger.** `session_arrange_next` (`commands/sessions.rs`) → `crates/integrations/src/arranged.rs`
   `arrange_and_next`: `SessionsService::arrange` sets `sessions.arranged_at` (first press only; no
   undo step, no `change_seq` bump), then `ResearchApi::arranged` queues a run (trigger `arranged`)
   when no blocking prerequisite fails, the session has a Main photo, and none of its runs is queued,
   running, finished or stopped at the cost limit (A19 `rules::run_rules::arranged_queues`: research
   starts by itself at most once per session; a failed or cancelled run may be retried). The app
   then goes to the Sessions page with the New session dialog open (`Header.tsx`; the command's
   `next_session_id` is no longer followed). `publish_preview` marks unmarked sessions the same way
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
  `research_stage_outputs`, and (W2) `writer_findings` (the writer's findings and questions per
  item; `delete_session` removes a session's rows). Columns: research master plan §6.4, writer
  master plan §4.
- Settings: the `research` key of `settings.json` is an opaque JSON slot in `psm_infra`
  (`Settings.research`); its shape is `ResearchSettings`. The engine reads it live, so a save is a
  simple update; only a change of `research.providers` rebuilds the App.
- W3: the `writer` key of `settings.json` is the writer's opaque slot (`Settings.writer`, shape `WriterSettings`),
  validated by `WriterSettings::validate` in `SettingsSetContext::prepare`, merged field by field like `research`,
  read live by the engine; nulled in the rebuild comparison, so a save never rebuilds the App.
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
rest of the app works: HA4), `commands/writer.rs` (W1 `writer_note`, `writer_look_again`; W3 `writer_status`, `writer_request(session_id, origin)` with origins `next_to_review` / `all_checked` / `write_texts`, `writer_finding_answer`, `writer_finding_dismiss`; W4 `writer_settings_get`, `writer_settings_save(value)` (the whole slot; the versions rule `WriterSettings::saved_over` / `push_version`), `writer_instruction_reset(language)`, `writer_preview(language)` (built by `writer::run::request_of`, the function a run's call uses, over `writer::input::assemble`); same refusal), `session_arrange_next` in `commands/sessions.rs`, `ResearchSettingsDto`
in `commands/settings.rs`, `researching` / `n_withheld` / `conflicts` in `commands/publish.rs`.
Frontend: wrapper `apps/desktop/src/services/research.ts` (W1: `services/writer.ts`; the note block
`features/form/ProductNote.tsx` under the panel's steps, re-read with the panel's status; W3: `features/form/WriterBlock.tsx`
(`useWriterStatus`, re-read with the form's research status, and the block at the top of `AiReviewTab`),
`features/form/WriterPanel.tsx` (the panel's Writer block), the **Write texts** button in `FormDialog`'s Product step,
and `requestTexts` in `features/sessions/reviewQueue.ts`, which `AiReviewTab`'s "Next to review →", `Header`'s "Next to
review" and `FormDialog` (closing Details or unmounting while AI review shows "All checked ✓") call fire-and-forget). W4: `features/settings/WriterSection.tsx` (with `WriterInstruction.tsx`, `WriterExamples.tsx`; loaded when first opened, saved whole by **Save writer settings**) and the nav entry "Writer" after "Research".
**Title suggestion and publish (W3, writer master plan WC4.7–8).** The writer writes `product.title` as an
attribute row; `form_get` shows it in place of `items.title` while it is `ai` + `suggested` (or `accepted` and equal to
`items.title`). `accept` / `accept_all` also set `items.title` in the same undo step; `keep_previous` deletes the row;
`clear` on a shown suggestion clears the row only; a human title edit deletes the row (feedback `edited`). Undo and
redo restore the row and `items.title` together (`history.rs` checks the title beside its row). `partition` skips
every `product.title` row and `partition_in` passes the built-in definitions too, so a description with a prior is a
real conflict (held). A29: a title that is only the item's SKU is no stored state for the writer
(`writer/writes.rs::sku_placeholder`), and `ItemsService::adopt` records the shop title as the link's title base. On
publish, the Shopify adapter's `update_title` reads the product's handle (`ProductHandle`) and, while it is still the
SKU's (`adapter::sku_handle`), sends the title's handle (`adapter::handleize`) with `redirectNewHandle: true` in the
same `productUpdate`; once more with `-<sku>` when Shopify says the handle is taken. There is **no `features/research`
folder**: the panel and pills are in `features/form/` (`ResearchPanel.tsx`, `AiPopover.tsx`,
`AiPill.tsx`), the settings in `features/settings/` (`ResearchSection.tsx`, `ShopFieldPicker.tsx`),
conflicts in `features/publish/ConflictList.tsx`, the button in `features/shell/Header.tsx`, the
"Arranged" pill in `features/sessions/SessionRow.tsx`.
**Review queue (A19).** `features/sessions/reviewQueue.ts`: `useReviewQueue(current)` reads the sessions list's
order (`listSessions({filter: 'all'})`) and every session's research state (`useResearchOverview(null)`); a session
waits when its state is `to_review` (`waitingForReview`). `nextToReview(order, waiting, current)` is the first waiting
session after the current one, wrapping round. `App.tsx` holds the queue and `goNextToReview` (closes Details, opens
the next session's page, or the Sessions page with "All reviewed"), passed to `Header` (`reviewQueue`; the
button rule is `showsNextToReview`: arranged, research `to_review` or `done`, others waiting) and through
`WorkspacePage` (`reviewQueue`) to `FormDialog` (`reviewFirst`: opened without a position it lands on AI review once
when suggestions wait; `review` → `AiReviewTab.nextToReview`, shown as "All checked ✓"). `SessionsPage` filters its
rows with the "To review · n" toggle.

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
| `~/Library/Application Support/com.loorenz.sessionmanager/` (`PSM_DATA_DIR` overrides; `AppPaths`, master plan §6.3) | `session_manager.db` (SQLite, WAL) · `settings.json` (0600) · `previews/<pk[0..2]>/<pk>/{original.jpg, adjusted-<g>.jpg, thumb-<g>.jpg}` · `masters/…/master-<g>.tif` · `outputs/…` · `hooks/*.scpt` + `hooks/jxa/*.js` · `spool/`, `spool/bad/` · `logs/psm.<yyyy-mm-dd>.log` · `backup/session_manager-<stamp>.db` (last 30) · `camera/sony/<ver>/` (support package + sidecar copy) · `camera/fingerprints/` · `camera/staging/` · `research/images/<sha[0..2]>/<sha>.jpg` (research copies of photos, listing images) · `templates/<template id>/<uuid>.jpg` (shooting-template examples) · `previews/remote/<item id>/cover-<12 hex of sha256(media id)>.jpg` (H2: a remote item's row picture, 320 px, one per item) · `previews/remote/search/<16 hex of sha256(url without query)>.jpg` (H2: shop search pictures, 160 px, at most 200 MB, oldest first) · `previews/archived/<item id>/<role>-<NN>-<12 hex>.jpg` (P62: 1600 px remote pictures) |
| Under the capture root (`<document folder>/Capture`) | item folders by `folder_key`; `.incoming/` (FTP parts, `sdk-<run>/`); `.duplicates/`; Capture One's own `CaptureOne/` and `Output/_session_manager/` |
| SQLite schema | `crates/infra/src/sqlite/migrations/001_init.sql … 020_remote_items.sql`, applied by `rusqlite_migration` at `App::open` (`SqliteStore::migrations()`); numbering is sequential, files are append-only (a new change = `014_*.sql` + a new `M::up(include_str!)` line in `sqlite/store.rs`). Tables: master plan §6.5 (+ 008 `raw_state.settings_fingerprint`, 009 `publish_items.frozen_item_type`/`item_links.item_type_base`, 010 `item_retention`/`archived_images`, 011 `item_stock`/`publish_item_stock`, 012 `photo_denials`, 013 `attribute_saved_values`/`item_remote_titles`, 014 `shooting_templates`/`template_containers`, 015 `photo_places`, 016 container labels, 017 `session_changes` (A10: one row per counted change with its `kind` arrival/editor/app/auto and the `seq` it produced; read by `published_changes` for "Changed after publishing", pruned when a publish completes), 018 `saved_pages`, 019 `conversation_sync`/`conversations`/`conversation_messages`/`conversation_outbox` (C1; no session, item or photo column, so Delete session never touches them), 020 `item_retention.restored_at`/`remote_checked_at` and `archived_images.bytes` (H1: the record of remote pictures; `ALTER TABLE … ADD COLUMN`, so the `p27` rewind test drops them); 014, 015, 017 and 019 use `CREATE TABLE IF NOT EXISTS`, names in `master_plan_templates.md` §4) |
| `settings.json` (`psm_infra::settings_file::Settings`; keys master plan §6.8) | `camera {receiver_port, passive_port_start, passive_port_end, auto_take_back}`, `document_id`, `preview_recipe`, `master_recipe`, `output_format {long_edge, quality}`, `style_name`, `default_shape`, `export_folder`, `preview_delay_s`, `theme`, `sku_prefix`, `integration {kind, shop_domain, api_version, client_id, client_secret, condition_photos_key}`, `storage {min_free_gb, retention_days_after_publish}`, `export {prepare_masters}`, `commerce {inventory_default}`, `research` (an opaque slot; shape in §Research), `conversations {server, sound, banner}` (C1; read live through `ConversationSettingsSlot`, the default server stored as none, never rebuilds the App). A file that does not parse never stops the app (health `settings_invalid`) |
| Keychain (`infra/keychain.rs`; names in `domain/camera.rs::secrets`) | service `com.loorenz.sessionmanager.camera`, accounts `access-auth-user`, `access-auth-password`, `ftp-password`, the three `research-*-key` accounts (§Research), and `conversations-api-key` (C1: the Pricing key, registered for log redaction on `set` and on the first read). What research redacts and the export looks for (`integrations::research_secret_values`): the three `research-*-key` accounts plus the client secret, nothing of the camera's (A9: the short FTP code turned up by chance in records and image headers). Shopify client id/secret: `settings.json` only; access token: memory only |
| `settings` table | app state, not user config (`mechanisms_version`, `active_session_id`, spool dedupe keys) |

## Events: Rust → UI
`psm_core::events::AppEvent` → `EventHub` (integrations `lib.rs`) → `TauriEvents::emit` (desktop
`lib.rs`) → Tauri event names: `session_changed`, `photo_added`, `photo_changed`, `publish_progress`,
`export_progress`, `health_changed`, `camera_status_changed` (payload `CameraStatusResponse`),
`camera_fingerprint_pending`, `history_changed`, `research_changed`, `picture_ready {kind: 'cover' | 'search', session_id?, external_id?, path}` (H2; listener `onPictureReady`), `conversations_changed {toasts: [{conversation_id, text}], more}` (C1; built by `conversation_rules::notice_lines`, listener `onConversationsChanged`; C2: the same arm plays the chime and shows the Mac banners through `alerts.rs`); plus `edit_menu` emitted by the menu handler. The UI
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
  ItemTypePicker, StockFields, AI pills, AiReviewTab, ResearchPanel), `publish` (PublishPopover, PublishProgress, BatchQueue,
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
