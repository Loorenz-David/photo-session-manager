# Live preview — master plan (v1, 2026-11-26; v1.1 2026-11-26: fix milestones LV5, LV6 after the paused hardware check; v1.2 2026-11-27: LV8 sort and arrange, LV-D20)

**v1.2 amendments (2026-11-27, planner, from book 10J LV-D20, changelog D20; the owner's request R3 after the
hardware check):** §2 (LV7 recorded, LV8 added, totals), §3 (LV8 `implementer: fable`, its server unit), §4.2 and §4.3
(classify, insert moves, `set_after_take_out`), §4.5 (phone files), §4.6 (`by=classify`), §4.7 (test stems), §5
**PH4a** (new route and move targets) and **PH9** (the phone's sort and arrange rules), §6 (phone texts), §9. Nothing
earlier is superseded.

**v1.1 amendments (2026-11-26, planner, from the LV4 hardware check findings F1–F5, F7 and owner decision LV-D19;
evidence `STATUS_live.md` §"LV4 hardware check in progress" and the app log `psm.2026-11-26.log`):** §2 (LV5, LV6 and
the totals), §3 (LV6 `implementer: fable`), §4.3 (`PhoneTimings`), §4.6 (`reason=stalled`), §4.7 (new test stems),
§5 **CS4a** (F4), **FC4a** (F5), **HB5a** (F1), **PH5a** (F2), **PH6a** (F7), **PH8** (F3), §6 phone texts (LV-D19
(2)), §7, §9. Each amendment is marked in place; the text it replaces is kept and marked superseded.

Short on purpose. It adds to `docs/implementation/master_plan.md` (roles, standing rules §3, environment §7) and wins
over it on a name listed here. Process: `docs/process/flow.md` §10 (milestones of units) with the deltas of §3.
Planner: Claude Opus 5.5 (`pipeline-planner`), from book `docs/intention/book/10J_live_preview.md` (RATIFIED
2026-11-26, changelog D18). Branch `feature/live-preview`, worktree `~/Desktop/Developer/Photo-Session-Manager-live`.

## 1. Goal
Build book `10J` for the **phone**: the camera sidecar pulls live frames (paced, de-duplicated) and sends them over
its own binary channel; a Rust **live view hub** keeps the newest frame and gives it only to viewers that show Live;
focus from a tap (point, half-press, focused/not, focus frames, × recentres); a **current shot** per templated
session that binds every exposure (MC-LV3, replacing 10G's shot-index rule, refill and retake); the embedded HTTP +
WebSocket server with per-phone pairing tokens; the owner's **Shoot Companion** phone app; then an owner-attended
hardware check of M-LV1…M-LV6. Semantics live in 10J and the chapters it amends (10E, 10G, 11); §5 below is what 10J
leaves to planning: the exact rules of the mechanisms that fail silently. Where 10J and this file differ, 10J wins and
this file is amended. **v1.1:** the hardware check, paused part-way, found six defects (F1–F5, F7) and the owner
decided LV-D19; `LV5_live_fixes` and `LV6_phone_dock` fix them before `LV4` resumes (§2, §9).

**Not in this track:** the Mac's live view screen and the desktop Tauri-channel outlet (card LV10; the hub's outlet
interface stays open for it and for machine-guidance subscribers), HTTPS (card LV3), sleep (LV-D5 revised), session
completion or opening from the phone (LV-D18: the API leaves room, §5 P5), machine guidance, writing
`LiveView_Image_Quality` (LV-S1: no setting is required; LV-D2 lists it only "if the owner wants Low"), naming
"camera in use by another computer" (10J §Status says 10E should; the lab saw only a generic `0x8202`, evidence C12,
so a mapping needs its own evidence: a later 10E task).

## 2. Milestones (one runner phase each; order in `docs/implementation/phases_live.txt`)
Milestone ids `LV1`…`LV4` are this track's; they are **not** 10J's card ids LV3…LV10.

| Id | Builds | Units | Criteria | Rows | Implementer | Projection | depends_on | Useful on its own |
|---|---|---|---|---|---|---|---|---|
| `LV1_camera_live_view` | sidecar live view, frame channel, focus worker, clean quit (C++, ported from the lab); the live view hub; protocol, fake sidecar, adapter capability and focus | 3 | 17 | 61 | default | no | — | live frames and focus reach the app's hub; nothing visible yet |
| `LV2_current_shot_and_phone_server` | MC-LV3 in placement; phone operations; pairing and Settings → Phone; the HTTP server; the WebSocket | 5 | 29 | 101 | default | no | LV1 | the Mac shows "Next" as the current shot; a phone can be paired; the server answers (a placeholder page) |
| `LV3_phone_app` | the Shoot Companion app: shell and live, main screen gestures and capture, shot dock, preview, Move and Item drawers (+ U0: LV2's server carry-overs, added by the orchestrator 2026-11-26) | 5 | 29 | 85 | **fable** | no | LV2 | the photographer shoots from the phone |
| `LV5_live_fixes` (v1.1) | hardware-check fixes: shutters on the way until the photo exists (F4); Mac actions announced to phones (F3); preview version (F7 server); a Wi-Fi stall skips frames (F2); Settings → Phone list current; Focus Area not movable reported (F5); exposure readout (F1) | 4 | 12 | 49 | default | no | LV3 | the photographer can try the fixes on the hardware (dock unchanged) |
| `LV6_phone_dock` (v1.1) | LV-D19: dock without hold gestures (larger tiles, scroll, single tap); pictures never broken (F7 phone); removed text | 2 | 6 | 13 | **fable** | no | LV5 | the dock as the owner asked; tile pictures appear when ready |
| `LV4_hardware_check` | owner-attended: M-LV1…M-LV6 and the "at hardware acceptance" items on the packaged app (paused 2026-11-26 part-way; resumes after LV6) | 3 | 20 | 20 | — (owner + orchestrator) | no | LV3 (resumes after LV6, §9) | measured evidence; M-LV6 targets set by the owner |
| `LV7_tile_loading_cue` (v1.2, recorded; planned by the orchestrator 2026-11-27, owner request R2) | a quiet loading sweep on a waiting tile | 1 | 3 | 4 | **fable** | no | LV6 | the tile shows the photo is on its way |
| `LV8_phone_sort_and_arrange` (v1.2, LV-D20) | U1 the Mac's classify route and insert moves (PH4a); U2 the Preview's swipes, confirmation and strip; U3 press-hold-lift drag and drop in both strips; U4 cancels, the drop shown at once, rollback (PH9) | 4 | 15 | 70 | **fable** | no | LV7 | the photographer sorts and arranges a session's photos from the phone |

Totals (computed from the plan files by `grep`, v1.1): 6 milestones, 22 units, 113 criteria, 329 rows; every unit has
at most 8 criteria. (v1 read 4 milestones, 15 units, 91 criteria, 261 rows; LV3's U0 had not been counted.) **v1.2:**
rows counted as the lines matching `^| LV<n>·C<k>(<row>)` in each plan file (`grep -c`), criteria as the distinct
`LV<n>·C<k>` among them: LV1 17/61, LV2 29/101, LV3 29/83, LV4 20/20, LV5 12/49, LV6 6/14, LV7 3/4, LV8 15/70 →
8 milestones, **131 criteria, 402 rows** (the plan files win over this table: LV3's own table says 85 rows, LV6's 13;
grep finds 83 and 14). Each
milestone needs the one before approved (its `depends_on`; gate = the **Tracker** line of that plan
file). Start each with `tools/pipeline/run_phase.sh phase <id>` (the runner resolves any id as
`docs/implementation/plans/<id>.md`); `run_phase.sh all` reads `phases.txt`, not this list. `LV4` is never dispatched
by the runner (its task 0 answers `BLOCKED`, as P52 did).

## 3. Process deltas (flow §10 applies; these add to it)
- **No mechanism-inventory session** (owner, 2026-11-26). §5 stands in for it, as `master_plan_templates.md` §5 and
  `master_plan_writer.md` §5 did: one contract per silent-failure mechanism, each with its invariant. Trace cells cite
  10J's ledger `M-LV1`…`M-LV6`, its contracts `MC-LV1`…`MC-LV3`, the contracts of §5 (`FC`, `HB`, `CS`, `PH`), 10E's
  `MC-31`…`MC-33` and 10G's rules; `[rule]` only for UI rendering rows (design handoff).
- **Milestones of units** (flow §10): one implementer session works the units in order with a `CHECKPOINT (not
  approved):` commit after each, then **one** review, one fix budget, one narrative per milestone. Reason for more than
  8 criteria per milestone (charter: recorded reason): the owner asked for about three build milestones; a phase cycle
  per unit costs more than the work. Each unit stays ≤ 8 criteria.
- **Implementers (owner, 2026-11-26; supersedes flow log v1.3's "no Fable for implementation" for this track):**
  `LV3_phone_app` carries `implementer: fable` (complex UI and interactions are built and fixed by Fable). `LV1` and
  `LV2` (sidecar, hub, backend, server, the small Settings page) use the default implementer and **never** carry
  `implementer: fable` (guide: no Fable on camera phases). Reviewer Opus. **v1.1:** `LV6_phone_dock` (phone UI only)
  carries `implementer: fable`; `LV5_live_fixes` (sidecar C++, core, server, the small Settings list) uses the default
  implementer and never carries it. **v1.2:** `LV8_phone_sort_and_arrange` carries `implementer: fable`; its first unit
  is a small server change (one core function and two move variants in `template_placement`, one route; no camera
  code, no schema), kept in the milestone rather than a separate default-implementer milestone because a second review
  cycle would cost more than the change (planning prompt `prompts/reviewer/live_sort_plan_r1.md` allows it).
- **Projection off for LV5 and LV6 too (v1.1).** The two externally owned shapes in LV5 are grounded, not invented: the
  helper's `liveview.exposure` line is quoted from the hardware log, and the Focus Area values are the installed SDK's
  enum (`CrDeviceProperty.h:1456–1485` in the lab's SDK 2.02); the rest are this repo's shapes.
- **Projection off** for every milestone. Reason: the fixtures are this repo's own shapes (the sidecar protocol and
  frame record of §5 FC, the phone wire of §5 PH); the Sony SDK calls are ported from lab code proven on the a7R V
  (evidence `sony_camera_lab.md` §Live view, §Focus from the live picture) and are checked by `LV4`, not by any test;
  the only dependency-owned shapes at a test boundary (axum's WebSocket close frame as `tokio-tungstenite` receives
  it; browser Pointer/Fullscreen/Vibration APIs in jsdom) are standard and stated where used. (LV3 review lesson: this
  repo's jsdom has **no `PointerEvent`**; phone tests send mouse events of the pointer types with a `pointerId`, see
  `apps/desktop/src/phone/__tests__/helpers.ts`.)
- **Test runs** (flow §4, §4A, §10). While working: the unit's **Unit test commands** only. The full closing gate runs
  **once, at the end of each build milestone** (all three change app files). Several test stems go **after `--`**:
  `cargo test -p psm-testkit --test suite -- <stem>:: <stem>::` (one filter is allowed before `--`; two are refused,
  verified §8). C++: `tools/sidecar/build.sh --stub --test` after any sidecar edit.
- **Row ids** `LV<n>·C<k>(<row>)`, numbered once per milestone across its units; tests named after their row
  (`lv2_c4a_<slug>`). Reviewer notes: `docs/implementation/backlog_live/<milestone id>.md` (new folder).
- **Data.** No numbered migration on this branch (§6.4): the schema additions are idempotent `ensure` steps, so a
  build of this branch may open the owner's real data and `main`'s migration numbers can never collide at the join.
- **Guide** (flow §7A): every milestone updates `docs/guide/` in the same change; each plan names the files.
- **Join:** after `LV3` is approved the orchestrator pushes `feature/live-preview` and opens the PR into `main`;
  `LV4` runs on the packaged build of that branch (before or after the merge, the orchestrator's call).

## 4. Names (registry; exact)

### 4.1 Sidecar (C++, `native/camera-sidecar/`)
- New `include/crlab/liveview.hpp`: `LivePacer` (pacing and de-duplication, FC2), `FrameRecord` (encode, FC1),
  `LatestSlot<T>` (one-slot mailbox: `put` replaces, `take` empties; used by the sender thread), `FocusQueue`
  (newest-wins, FC4). Header-only or with `src/liveview.cpp` (new); pure, no SDK types, so the stub build tests them.
- New `src/live_view.cpp`: the SDK calls ported **verbatim in behaviour** from the lab's `sdk/src/camera_liveview.cpp`:
  `CameraSession::live_view_state`, `pull_live_view_frame`, `focus_state`, `set_focus_point`, `autofocus`,
  `focus_frames`. **Not ported:** `set_live_view_quality`, `restore_live_view_quality`, `set_focus_area` (the app never
  writes those, LV-D2/LV-D14) and `battery_percent`. Types `LiveViewFrame`, `LvPull`, `LiveViewState`, `FocusState`,
  `FocusFrame` added to `include/crlab/camera.hpp` from the lab's `sdk/include/crlab/camera.hpp`; the symbol probes the
  lab added (`CRLAB_HAS_GetLiveViewImage`, `…GetLiveViewImageInfo`, `…LvFrameNotUpdated`, `…LvFrameNo`,
  `…PROP_LiveViewStatus`, `…PROP_LiveView_Image_Quality`, `…PROP_LiveViewProtocol`, `…PROP_FocusMode`,
  `…PROP_FocusArea`, `…PROP_AF_Area_Position`, `…PROP_FocusIndication`, `…GetSelectLiveViewProperties`,
  `…LvPropAFAreaPosition`, `…FocusFrameInfoType`, `…PROP_S1`, `…LockIndicatorLocked`, `…UnlockIndicator`) copied
  into `src/sdk_symbols.def`; `src/sdk_stub.cpp` answers `LvPull::Unsupported` / empty.
- New `src/live_runner.cpp`: `LiveRunner` (per camera session: pull thread, sender thread, focus worker; FC2–FC5).
- `src/main.cpp` (`cmd_auto`): option `--frame-socket <path>`; command `focus`; the action lock around `capture`;
  `LiveRunner` start after `connection.ready`, stop **before** any disconnect; SIGINT/SIGTERM → the `quit` path.
- Test: new `tests/test_liveview.cpp`, a new `add_test(NAME liveview …)` in `CMakeLists.txt`.
- **Command** (stdin, one line): `focus x=<0..639> y=<0..479> af=<0|1> cmd_id=<id>`.
- **Events** (stdout JSON, the usual envelope): `liveview.started {epoch, transport, status, frame_channel}` ·
  `liveview.stopped {epoch, reason, frames, repeats, errors}` · `liveview.unavailable {reason}` ·
  `liveview.stats {epoch, frames, repeats, errors, fps}` (every 10 s while running) · `liveview.focus_frame {frames}`
  (`frames` = the lab's compact array `[[type,state,x_num,x_den,y_num,y_den,w,h],…]`) ·
  `liveview.exposure {shutter_speed, f_number, iso}` (the decoded strings `props` already produces; **superseded v1.1 by HB5a**: each value is the `props` object, the app reads its `display`) ·
  `focus.result {cmd_id, x, y, af, point_set, refused, point_ms, af_ms, total_ms}`. Values in §5 FC.

### 4.2 Core (`crates/core/`)
- New `src/domain/live.rs`: `CameraFrame {source: u64, frame_no: u32, width: u16, height: u16, jpeg: Vec<u8>, link:
  Link}` · `LiveFrame {seq: u64, start: u32, received: std::time::Instant, captured_at_ms: i64, width: u16, height:
  u16, jpeg: Arc<[u8]>, link: Link}` · `ViewerId(u64)` · `ViewerDemand {gen: u32, live: bool, visible: bool}` ·
  `FocusPoint {x: f64, y: f64}` (0..1 of the full live frame) · `FocusId(String)` · `FocusOutcome {Focused,
  NotFocused, NoReport, Superseded, Refused(FocusRefusal), Failed}` · `FocusRefusal {AreaNotMovable, NotConnected,
  Unsupported, BadPoint}` · `FocusFrame {cx, cy, w, h: f64, state: FocusFrameState {Focused, NotFocused, Unknown}}` ·
  `ExposureReadout {shutter: String, aperture: String, iso: String}` · `LiveState {Off, Unavailable{reason}, Live{link}}`
  · `LiveEvent {State(LiveState), Focus{focus_id, outcome}, FocusFrames(Vec<FocusFrame>), Exposure(ExposureReadout),
  Capture{capture_id, state: CaptureReportState {Acknowledged, Exposed, Failed}}}` · `PublishOutcome {Forwarded{seq},
  Repeat, Stale}` · `PhoneSettings {enabled: bool, port: u16}` (defaults `true`, `8686`).
- New `src/services/live_view.rs`: `LiveViewHub` (§5 HB), trait `LiveOutlet: Send + Sync {fn offer(&self, frame:
  Arc<LiveFrame>, gen: u32); fn withdraw(&self); fn event(&self, e: &LiveEvent);}` (all three never block),
  `LatestSlot<T>` (one-slot mailbox, core mechanism), `focus(provider, point) -> Result<FocusId>`,
  `focus_clear(provider) -> Result<FocusId>`.
- `src/ports/camera.rs`: `CameraProvider::focus(&self, point: FocusPoint, autofocus: bool) -> Result<FocusId>`
  (default `Err(Error::FocusUnsupported)`); `CameraCapabilities` + `live_view: bool`, `focus_point: bool`.
- New `src/services/current_shot.rs`: `CurrentShots` (process-lifetime state, §5 CS), `Current {Shot(i64 /*container
  id*/), Free}`, fns `current`, `set`, `bind_exposure`, `bind_arrival`, `advance`, `taken`, `default_shot`.
- New `src/mechanisms/current_shot.rs` (pure): `next_untaken(order: &[i64], taken: &HashSet<i64>, after: Option<i64>)
  -> Option<i64>` (CS4).
- `src/mechanisms/shot_order.rs`: `Shot` + `exposure: Option<i64>`; new `slot_owners(shots, holds) -> Vec<Slot
  {owner: Option<i64> /*exposure id*/, occupant: Occupant {Photo(i64), Hold(i64)}}>` (CS2).
- `src/mechanisms/template.rs`: new `shot_ratio(presets: &Presets, default_shape: Shape) -> Option<f64>` (PH6).
- `src/services/template_placement.rs`: placement by binding (CS3); new `retake(h, session_id, container_id)`,
  `move_to(h, photo_id, MoveTarget)` with `MoveTarget {Container(i64 /*final_no*/), MainPosition(i64), ExtraEnd,
  Condition(i64), ConditionEnd}`; `HandAction` + `shots: &CurrentShots`; `place_arrival` + `shots: &CurrentShots`;
  removed: `gaps`, `gap_for_arrival`, `take_shot_key`, `next_shot` (index-based), `expected_place(index, …)`.
  **v1.2 (LV8, PH4a):** `MoveTarget` + `ExtraInsert(i64)`, `ConditionInsert(i64)` (1-based place in the group after the
  move); new `classify_to(h, photo_id, ClassifyTo) -> Result<ReviewResult>` with `ClassifyTo {Main, Condition}`.
  `src/services/current_shot.rs`: `CurrentShots::set_after_take_out(session_id, container_id)` (as `set_after_retake`,
  log `by=classify`).
- New `src/services/phone_view.rs`: `phone_view(store, shots, default_shape, session_id) -> PhoneSessionView` (PH6).
- Domain: `ShotBinding {container_id: Option<i64>}` (`None` = bound to no container → Condition) in
  `src/domain/template.rs`; `NewExposure` and `Exposure` + `shot: Option<ShotBinding>`; `PlaceRow` + `shot:
  Option<ShotBinding>`.
- `src/events.rs`: `AppEvent::ActiveSessionChanged {session_id: Option<i64>}`, emitted by `SessionsService` when the
  active session changes (CS7). Desktop event name `active_session_changed`.
- `src/error.rs`: `FocusUnsupported` (`focus_unsupported`), `ShotFilled` (`shot_filled`), `SessionChanged`
  (`session_changed`), `NotInTemplate` (`not_in_template`), `PairingUnknown` (`pairing_unknown`), `PhonePortInvalid`
  (`phone_port_invalid`); their codes added where the desktop maps codes (`apps/desktop/src-tauri/src/commands/items.rs`).

### 4.3 Integrations (`crates/integrations/`)
- `src/camera/sidecar/protocol.rs`: `Command::Focus {x: u16, y: u16, af: bool, cmd_id: String}`; events
  `LiveviewStarted`, `LiveviewStopped`, `LiveviewUnavailable`, `LiveviewStats`, `LiveviewFocusFrame {frames:
  Vec<[i64; 8]>}`, `LiveviewExposure`, `FocusResult`.
- New `src/camera/sidecar/frames.rs`: `FrameListener` (Unix socket, FC1), `parse_record` (core mechanism), the reader
  thread (`camera-frames`) → `LiveViewHub::publish`.
- New `src/camera/direct/live.rs`: `to_grid(FocusPoint) -> (u16, u16)`, `frame_norm([i64; 8]) -> FocusFrame`,
  `exposure_readout(..) -> ExposureReadout`, sidecar event → `LiveEvent` (FC6).
- `src/camera/runtime.rs`: `--frame-socket` argument, listener lifecycle, `live_hub() -> Arc<LiveViewHub>`,
  `current_shots() -> Arc<CurrentShots>` (both created once per process with the runtime, carried through rebuilds,
  10E R-I18).
- New `src/phone/` (LV2): `mod.rs`, `pairing.rs` (`PhonePairings`), `host.rs` (Mac name, URLs, QR SVG), `server.rs`
  (`PhoneServer::start(config, handles: Arc<dyn Fn() -> Result<AppHandles> + Send + Sync>, hub)`, `rebind(port)`,
  `stop()`; `PhoneTimings {ping: 5 s, idle_close: 15 s, write_timeout: 2 s}` — **superseded v1.1 (F2, PH5a):**
  `PhoneTimings {ping: 5 s, idle_close: 15 s, stall_close: 15 s}`, `write_timeout` removed), `auth.rs`, `routes.rs`,
  `ws.rs`, `wire.rs` (DTOs, frame header, `error_code`), `web_root.rs`. **v1.2 (LV8):** `wire.rs` `ClassifyBody
  {session_id, to: "main" | "condition"}`, `MoveTargetDto` + `ExtraInsert {index}`, `ConditionInsert {index}`;
  `routes.rs` handler `classify_photo` for PH4a's route.
- Dependencies: `qrcode = { version = "0.15", default-features = false, features = ["svg"] }` (integrations);
  `axum` gains feature `"ws"` (infra's line `axum = "0.8"` → `axum = { version = "0.8", features = ["ws"] }`, and
  integrations depends on `axum` the same way); testkit dev-dependencies `tokio-tungstenite` (the version axum's `ws`
  resolves in `Cargo.lock`, so one copy is compiled) and `reqwest` (as integrations has it). The first build after these
  lines needs the network once (crates.io reachable on this Mac, §8).

### 4.4 Storage (`crates/infra/src/sqlite/ensure.rs`, new fn `ensure_live_columns`, called beside `ensure_research_columns`)
`camera_exposures` + `shot_bound INTEGER NOT NULL DEFAULT 0`, `shot_container_id INTEGER` · `photo_places` +
`shot_bound INTEGER NOT NULL DEFAULT 0`, `shot_container_id INTEGER` · `CREATE TABLE IF NOT EXISTS phone_pairings (id
INTEGER PRIMARY KEY, name TEXT NOT NULL, token_sha256 TEXT NOT NULL UNIQUE, created_at INTEGER NOT NULL, last_seen_at
INTEGER, revoked_at INTEGER)`. Store trait (`crates/core/src/ports/store.rs`) gains the reads and writes these need
(names delegated; one transaction per decision as today).

### 4.5 Settings, commands, frontend
- `psm_infra::settings_file::Settings.phone: PhoneSettings` (serde default); `phone` joins the patch allowlist of
  `crates/core/src/services/settings.rs`; nulled in the rebuild comparison of `crates/integrations/src/lib.rs` (beside
  `theme`): saving it never rebuilds the App; a port change calls `PhoneServer::rebind`.
- Tauri commands (new `apps/desktop/src-tauri/src/commands/phone.rs`, registered in `specta_builder()`; wrapper new
  `apps/desktop/src/services/phone.ts`): `phone_pairing_create(name?) -> PairingCreatedDto {id, name, url_name,
  url_address, qr_name_svg, qr_address_svg}` · `phone_pairings_list -> [PairingDto {id, name, created_at, last_seen_at,
  connected}]` · `phone_pairing_rename(id, name)` · `phone_pairing_revoke(id)` · `phone_server_status ->
  {enabled, running, port, error}`.
- Desktop UI: new `apps/desktop/src/features/settings/PhoneSection.tsx`, its entry in `SettingsNav.tsx`.
- Phone entry: new `apps/desktop/phone.html`; `vite.config.ts` builds two inputs (`index.html`, `phone.html`) into one
  `dist/`; phone code under new `apps/desktop/src/phone/` (`main.tsx`, `PhoneApp.tsx`, `api.ts`, `socket.ts`,
  `state.ts`, `geometry.ts`, `gestures.ts`, `components/…`, `phone.css`). Packaged: `tauri.conf.json` `bundle.resources`
  maps `../dist/` → `phone-web/`; the server's web root is `<Resources>/phone-web` in the packaged app, `PSM_PHONE_WEB`
  when set, else `apps/desktop/dist` in debug builds (`web_root.rs`).
- HTTP routes and WebSocket messages: §5 PH.
- **v1.2 (LV8):** new `apps/desktop/src/phone/arrange.ts` (`dropTarget`, `applyDrop`, `arrangementKey`, `nextPhoto`,
  `swipeAction`; PH9) and `components/ConfirmSheet.tsx`; `gestures.ts` gains `SWIPE {holdMs: 200, startPx: 14, commitPx:
  96, hintPx: 24}`, `DRAG {liftMs: 400, liftScale: 1.15, bandPx: 48, edgePx: 40, scrollStepPx: 8, scrollTickMs: 16,
  pendingMs: 8000}`, `HAPTIC.lift = 12`, `HAPTIC.dragTick = 6`; `api.ts` gains `classifyPhoto` and the two move
  kinds; `Dock.tsx` serves both strips (`testId` `dock` on the live screen, `preview-strip` in the Preview).

### 4.6 Log lines (never a token)
`live.source start= link= frames_10s= repeats_10s= errors_10s=` · `live.viewer viewer= pairing= live= visible=
sent_10s= skipped_10s=` · `live.stop reason=` · `focus.request focus= x= y= af=` · `focus.result focus= outcome= ms=`
· `current_shot.bound session= exposure= container=` (`container=none` for Condition) · `current_shot.set session=
container= by=phone|retake|advance|default` (v1.2: also `by=classify`, PH4a) · `phone.server listening port= addresses=` · `phone.server error=` ·
`phone.request route= status= pairing=` · `phone.viewer opened|closed viewer= pairing= reason=` (v1.1: close reasons
`open`, `closed`, `idle`, `write`, `revoked`, `overflow`, **`stalled`**; `write_timeout` no longer occurs, PH5a).

### 4.7 Tests
Rust stems (each with its `mod` line in `crates/testkit/tests/suite/main.rs`): `lv1_live_hub.rs`,
`lv1_frame_channel.rs`, `lv2_current_shot.rs`, `lv2_phone_ops.rs`, `lv2_phone_pairing.rs`, `lv2_phone_http.rs`,
`lv2_phone_ws.rs`. Testkit helpers (new, each with callers in those stems): `crates/testkit/src/live_outlet.rs`
(`RecordingOutlet`), `crates/testkit/src/phone.rs` (`TestPhone`: starts `PhoneServer` on `127.0.0.1:0` with the
TestApp's handles, a temp web root and short `PhoneTimings`). C++: `native/camera-sidecar/tests/test_liveview.cpp`.
UI: `apps/desktop/src/features/settings/__tests__/lv2_phone_settings.test.tsx`;
`apps/desktop/src/phone/__tests__/lv3_<slug>.test.tsx`. Existing tests that encode a rule 10J replaced (10G §4 rule 2,
§6 refill, `next_shot` by index) in `t2_template_placement.rs`, `t3_template_hand.rs`, `t4_template_retake_shift.rs`
are rewritten to MC-LV3 in LV2·U1, each listed in the handoff with the rule it encoded.
**v1.1:** Rust stems `lv5_current_shot.rs`, `lv5_mac_actions.rs`, `lv5_phone_socket.rs`, `lv5_camera_readouts.rs`;
UI `apps/desktop/src/features/settings/__tests__/lv5_phone_list.test.tsx`, `apps/desktop/src/phone/__tests__/
lv6_dock.test.tsx`, `lv6_images.test.tsx`; C++ rows added to `native/camera-sidecar/tests/test_liveview.cpp`. Tests
changed because the behaviour they encoded changed (each listed in its milestone's handoff):
`lv1_frame_channel.rs::lv1_c15b_exposure_mapping` (an invented exposure shape → the helper's real line, LV5·C12(b));
the `PhoneTimings` literals in `lv2_phone_ws.rs` and `crates/testkit/src/phone.rs`; six LV3 dock tests removed and
`lv3_c1c_…` re-texted (LV6, listed in `plans/LV6_phone_dock.md`).
**v1.2 (LV8):** Rust stem `lv8_phone_sort.rs`; UI `apps/desktop/src/phone/__tests__/{lv8_core.test.ts, lv8_swipe.test.tsx,
lv8_drag.test.tsx, lv8_drag_reliability.test.tsx}`. Tests removed or changed because LV-D20 changed their behaviour
(listed in `plans/LV8_phone_sort_and_arrange.md`): `lv3_c22c_…` removed (no Delete button), `lv6_c2c_…` removed (a long
press on a photo tile lifts), `lv3_c21a_…`/`lv3_c21b_…` (the hold starts after 200 ms), `lv6_c5a_…` (assertion scoped to
the viewport).

## 5. Contracts (stand in for the mechanism inventory; plans implement them verbatim)

### FC — The sidecar's frame channel, pacing, commands and clean stop (MC-LV1, 10E MC-31, LV-D14)
**FC1 Channel.** The app creates a Unix socket listener at `$TMPDIR/psm-lv-<sidecar run id>.sock` (mode 0600; a stale
file of that name is removed first) **before** spawning the sidecar and passes `--frame-socket <path>`. The sidecar
connects once at start; failure → `liveview.unavailable {reason: "frame_channel"}` and no live view for that process
(control line unaffected). The app accepts one connection per sidecar process; a new connection replaces the old (new
`source`). On sidecar exit or app shutdown the socket file is removed. **Record** (little-endian), 24-byte header then
the JPEG: `0..4` magic `b"PSLV"` · `4` version `1` · `5` kind `1` (frame) · `6..8` reserved `0` · `8..12` `epoch` u32 ·
`12..16` `frame_no` u32 (camera's `GetFrameNo`; 0 = not supplied) · `16..18` width u16 · `18..20` height u16 (from the
JPEG SOF; 0 if not parsed) · `20..24` `jpeg_len` u32, `1 ≤ jpeg_len ≤ 4 194 304`. A reader that sees a bad magic,
version, kind or length closes the connection (`live.stop reason=bad_record`) and live view is off until the next
sidecar process. Frames never travel on stdout.
**FC2 Pull loop** (`LiveRunner` pull thread, one per camera session; `LivePacer` decides). Started after
`connection.ready` when `live_view_state().status` is Enable (2); NotSupport (0) → `liveview.unavailable {reason:
"not_supported"}`; Disable (1) or not reported → `liveview.unavailable {reason: "disabled"}` (the app never enables
it). `epoch` = 1 + the number of earlier starts in this process. Per pull result:
| Result | Forward? | Next pull |
|---|---|---|
| `Ok`, `frame_no ≠ 0`, `frame_no ≠ last_frame_no` (or first of the epoch) | yes | `t_new + max(15, est − 10)` ms |
| `Ok`, `frame_no ≠ 0`, `frame_no == last_frame_no` (repeat) | no (`repeats += 1`) | +5 ms |
| `Ok`, `frame_no == 0`: (size, FNV-1a 64) ≠ previous frame's | yes | as a new frame |
| `Ok`, `frame_no == 0`: equal to previous | no (repeat) | +5 ms |
| `NotUpdated` | no | +5 ms |
| `Error`, `sdk_error == 0` (buffer size 0, first pull) | no, not an error | +20 ms |
| `Error`, other | no (`errors += 1`; `liveview.pull_error`-style log on the 1st and every 50th consecutive) | +20 ms |
| `Unsupported` | stop: `liveview.unavailable {reason: "sdk"}` | — |
`est` = median of the last 16 intervals between forwarded frames (initial 40 ms), clamped to 20…200 ms. The camera
never says "no new frame" (evidence: it repeats the last one), so the pacer is what keeps pulls near the frame rate.
**FC3 Sender.** The pull thread `put`s each forwarded record into a `LatestSlot` and returns at once; a sender thread
`take`s and writes it (blocking). A record not yet taken is replaced by the newer one. The pull thread never blocks on
the socket. `liveview.stats` every 10 s.
**FC4 Focus worker** (one per camera session; `FocusQueue`). `focus …` is parsed on the main loop:
`cmd.received`; invalid x/y (outside 0…639 / 0…479 or missing) → `cmd.rejected {reason: "bad_point"}`; not connected →
the existing `not connected (…)` rejection; else queued, then `cmd.done` (= accepted). Queue depth 1: a job waiting
when a newer one arrives is replaced and answered `focus.result {cmd_id: <old>, af: "superseded", point_set: false}`.
A job: (1) under the **action lock**; (2) `focus_state()`: `AF_Area_Position` not settable (**test superseded v1.1 by FC4a**: the Focus Area value decides) → `focus.result {refused:
"focus_area_not_movable", point_set: false, af: "skipped"}` and **nothing is written**; (3) `set_focus_point(x, y)`
(confirmation by the focus frame, ≤ 1 s, as the lab); (4) `af=1` → `autofocus(3000)` (S1 locked, wait, S1 unlocked),
`af=0` → `"skipped"`; (5) `focus.result {af: focused|not_focused|no_af_report|skipped, …, total_ms}`. While idle the
worker reads the focus frames every 250 ms and the exposure (`f_number`, `shutter_speed`, `iso`, decoded as `props`
does) every 1000 ms, emitting `liveview.focus_frame` / `liveview.exposure` **only when changed**. Neither ever runs on
the pull thread.
**FC4a Movable Focus Area (amended 2026-11-26, v1.1, finding F5; replaces FC4 step (2)'s test).** On the a7R V with
Focus Area **Wide**, `AF_Area_Position` stays settable, the point write is accepted, no focus frame confirms it
(`point_ms = -1`) and the half-press focuses where the camera chooses (log 14:13:50–14:14:09, `outcome=Focused`): step
(2)'s "not settable" test never fires. Step (2) is now: refuse (same `focus.result`, nothing written) unless
`focus_area_movable(focus_area, area_read, position_writable)` is true, a pure function: `area_read` (the camera has a
`FocusArea` property) → true iff `position_writable` and the value is one of 4, 5, 6, 7, 8 (Flexible Spot S/M/L, Expand
Flexible Spot, Flexible Spot), 0x14–0x18 (Tracking: Flexible Spot S/M/L, Expand, Flexible Spot), 0x19–0x1D (Flexible
Spot XS/XL/FreeSize1–3), 0x23–0x27 (Tracking: Flexible Spot XS/XL/FreeSize1–3); every other value (0 Unknown, 1 Wide,
2 Zone, 3 Center, 0x11–0x13 Tracking Wide/Zone/Center) → false. `!area_read` → `position_writable` (the old rule, for a
camera without the property). Values: the installed SDK 2.02 header `CrDeviceProperty.h:1456–1485` (`enum
CrFocusArea`); the lab saw Flexible Spot S = 4 movable and the a7R V offering 1–7, 26–29 (Experiment 10). A movable
area whose point is not confirmed by a focus frame (e.g. a tap near the edge, where the camera clamps the point) still
runs AF. The Focus Area is never written (LV-D14, FC6).
**FC5 Capture and the action lock.** `capture` keeps 10E's half-press-then-release (`af=1`) on the main loop, exactly as
today, but holds the action lock: it waits for a running focus job and no focus job starts during it. A capture after
a tap therefore refocuses at the tapped point ("capture-at-point"; evidence Experiment 11: photo sharp where tapped).
**FC6 Writes.** The only `SetDeviceProperty` uses in the sidecar are the store destination (10E), `AF_Area_Position`
(FC4), S1 and the shutter release (capture, autofocus). No live-view setting, Focus Area, exposure (mode or values),
network or FTP setting is ever written (LV-D2, M-LV5). Every write goes through one function taking an enum of these
four intents (review checks by reading; `LV4` checks on the camera).
**FC7 Clean stop.** Every stop path (stdin `quit`, stdin EOF, SIGINT, SIGTERM, a credentials or support restart that
sends `quit`) first stops the runner: stop flag; a focus job in its AF wait is cut (S1 unlocked at once); pull, sender
and focus threads joined (≤ 300 ms added to today's quit); then the existing disconnect (USB: *Memory Card* write ≤ 1 s).
A transport switch or session loss stops and joins the runner **before** the SDK disconnect/release. The pull thread
never calls the SDK on a closed session. The app's `quit` → 3 s grace → kill is unchanged (10E MC-31).
**Invariant FC.** No frame is forwarded twice in an epoch; a slow reader never blocks the pull thread; no camera
setting other than the store destination and the focus point is written; a sidecar stop always ends the session.

### HB — The live view hub (MC-LV1, MC-LV2; core service, no Tauri, no HTTP)
**HB1 Sources.** `source_started(link, source)` begins a start (`start += 1`), state `Live{link}`;
`source_stopped(reason)` → `Off` (or `Unavailable{reason}` from `liveview.unavailable`) and `newest = None`. The adapter
calls `source_started` on `liveview.started` with `status: "enabled"` and `frame_channel: "connected"`, and
`source_stopped` on `liveview.stopped`, session closed/lost, the frame connection ending, or sidecar exit. `source` =
`(frame connection number << 32) | epoch`.
**HB2 Publish** (called on the frame reader thread only). No start → `Stale` (dropped). Same `source` and `frame_no ≠
0` and `frame_no ≤ last_frame_no` → `Repeat` (dropped). Otherwise `seq = hub_seq + 1` (**one counter for the app
process, never reset**, so seq strictly increases within and across starts), `received = Instant::now()`,
`captured_at_ms = clock.now_ms()`; `newest = frame`; `Forwarded{seq}`. Then, for each viewer with `live && visible`,
`frame.seq > last_offered` and `frame.received > switched_at`: `outlet.offer(frame, viewer.gen)`, `last_offered =
seq`. The hub mutex is held only for this loop; it does no I/O; `offer` never blocks.
**HB3 Demand.** `subscribe(outlet) → ViewerId` with demand `{gen: 0, live: false, visible: false}`. `set_demand(id, d)`:
if `d.live && d.visible` and (the viewer was not wanting or `d.gen` differs) → `switched_at = Instant::now()` (the
stored `newest` is **not** offered: it is older than the switch); if not wanting → `outlet.withdraw()` (drops any frame
waiting in its slot). `unsubscribe(id)` removes it. A viewer receives frames only while it shows Live and is visible;
the first frame after a switch is newer than the switch and carries the switch's `gen`.
**HB4 Slow viewers.** Every outlet's frame buffer is a `LatestSlot` (depth 1, replace). A slow viewer skips frames; it
never queues and never delays `publish`, another viewer, the control line or capture.
**HB5 Events.** `report(e)` calls `event(&e)` on every subscribed outlet in subscription order, whatever its demand
(focus results, focus frames, exposure, capture reports, state). The adapter maps (FC6 names): `focus.result` →
`Focus{focus_id = cmd_id, outcome}` (`focused`→Focused, `not_focused`→NotFocused, `no_af_report`→NoReport,
`superseded`→Superseded, `refused: focus_area_not_movable`→Refused(AreaNotMovable), `skipped` with `af=0` →Focused is
**not** used: a recentre reports `Focused` only if the camera reported so, else `NoReport`), `liveview.focus_frame` →
`FocusFrames` (`cx = x_num/x_den`, `cy = y_num/y_den`, `w = width/640`, `h = height/480`, state 2→Focused,
1→NotFocused, else Unknown; a zero denominator drops that frame), `liveview.exposure` → `Exposure` ("f/8", "1/125",
"ISO 100": `f_number` without a trailing ".0", shutter as the camera's fraction or seconds with `"`, ISO prefixed; **superseded v1.1 by HB5a**),
`capture.acknowledged success=false` / `capture.exposure_missing` → `Capture{Failed}`, `capture.acknowledged
success=true` → `Acknowledged`, `capture.exposed` with an app capture id → `Exposed`.
**HB5a Exposure line and readout (amended 2026-11-26, v1.1, finding F1; replaces the `liveview.exposure` clause of HB5
and §4.1's "(the decoded strings `props` already produces)").** The real helper sends each of `shutter_speed`,
`f_number`, `iso` as the `props` object `{code, value, value_hex, display, writable, readable}`, or the string `""` when
it could not read the property (evidence: the line logged at 2026-11-26T14:06:58.216844Z as "unparsable", quoted in
`plans/LV5_live_fixes.md` U4). The protocol reads a field that is a string as its display, an object with a string
`display` as that string; anything else makes the line unparsable (as today). The readout maps the displays the
helper's `decode` produces (`native/camera-sidecar/src/camera_properties.cpp`): **aperture** `F<a>.<b>` → `f/<a>` when
`<b>` is `0`, else `f/<a>.<b>` ("F8.0" → "f/8", "F5.6" → "f/5.6", "F11.0" → "f/11"); **shutter** unchanged ("1/80",
`2.5"`, "BULB"); **ISO** `ISO <n>` unchanged, `AUTO` → "ISO AUTO"; a display that is empty or `special` → "–" (en dash)
for that part.
**HB6 Focus API.** `focus(provider, p)`: `!capabilities().focus_point` → `FocusUnsupported`; `p.x`/`p.y` not finite or
outside `[0, 1]` → `InvalidValue`; else `provider.focus(p, true)`. `focus_clear(provider)` = `provider.focus({0.5,
0.5}, false)` (× recentres, card LV9). Direct adapter: `to_grid` = `(round(x·639), round(y·479))` clamped (lab
`camlab/pwa/__main__.py:228`), sends `Command::Focus{x, y, af}` with a fresh `cmd_id`, returns it as the `FocusId`.
Nothing Sony-specific crosses into core.
**Invariant HB.** For every viewer, the `seq` of offered frames is strictly increasing; a viewer not wanting receives
no frame; the first frame after a switch was received after the switch; `publish` never waits on a viewer.

### CS — The current shot (MC-LV3; replaces 10G §4 rule 2, §6 refill and the retake aim)
Notation: `order(s)` = the session's template containers by `shoot_no` ascending; "photo in c" = a photo of the session,
not Deleted, whose place row has `slot = c.final_no` (Main).
**CS1 State.** `CurrentShots` holds `Option<{session_id, current: Current}>` in memory only (never stored; a relaunch
starts empty; it lives with the camera runtime, so a settings save keeps it). `current(s)`: untemplated (10G P1) →
none, no state touched. Held for another session or nothing held → hold `{s, default_shot(s)}`. Held for `s` with
`Shot(c)` and `c ∈ taken(s)` or `c` not in the template → `advance(s, c)` (CS4) first. Returns the held value.
**CS2 Slots and owners.** `slot_owners` takes the session's unreleased shots (each with its matched exposure: the
`camera_deliveries.exposure_id` of the delivery whose `final_path` is the photo's RAW, else `None`) and holds (the
`hold_times` exposures), sorts them by time exactly as `shot_order::order` does, records for each time-sorted position
its **owner** (a hold's exposure id; a shot's matched exposure id or `None`), then runs `order`'s counter-run
reassignment; each photo's **slot owner** is the owner of the position it lands in. (So a file the ledger's FIFO rule
matched to an earlier exposure takes the binding of the exposure its camera counter says it belongs to, as 10G §4 rule
1 requires.)
**CS3 Binding of a photo** (first that holds): (1) `photo_places.shot_bound = 1` → `photo_places.shot_container_id`;
(2) its slot owner is an exposure with `shot_bound = 1` → that exposure's `shot_container_id`; (3) otherwise
**unbound** (shot before this track): placement never moves it (it keeps its slot and classification; it still counts
in `taken`). A binding to a container no longer in the template counts as `None`. **Expected place** of a template-placed
photo (`place_by_hand = 0`, not Deleted, not released): binding `Some(c)` → Main, `slot = c.final_no`, presets of `c`
(10G §4 rule 3, applied when its place changes); binding `None` → Condition, appended in shot order (rule 4). If `c`
already holds a hand-placed photo or an earlier template-placed photo of the same binding, the later one goes to
Condition (hand wins, 10G §6 rule 1). Rules 5 and 6 of 10G §4 hold unchanged (a late earlier shot may move
template-placed photos; one transaction per arrival).
**CS4 Taken, default, advance.** `taken(s, now)` = {c : a photo is in c} ∪ {binding of the owner of every slot
occupied by a hold (CS2)} (reported shutters whose files are on the way, `FIFO_WINDOW_MS`). `default_shot(s)` =
`next_untaken(order, taken, None)`; `None` → `Free`. `advance(s, from)`: `next_untaken(order, taken, Some(from))`:
the first container after `from` in `order` not taken, else the first in `order` not taken, else `Free`. After a
`Free` binding the same rule runs with `after = None` (first not taken, else `Free`): 10J's rule "else the first empty
one" applies to a Condition shot too (the book wins over the prototype's "stay in free shot"; planning point P1 in LV2).
**CS4a On the way until the photo exists (amended 2026-11-26, v1.1, finding F4; makes CS4's "reported shutters whose
files are on the way" exact).** A hold ended when the exposure's file was **filed** (a `camera_deliveries` row with its
`exposure_id`), but the photo row appears only when the arrival scan takes the file in (≈ 1.1 s later on Wi-Fi), so in
that gap the container was neither held nor filled and the advance aimed at it (log: exposure 61 at 14:07:02.224 →
`set container=10 by=advance` while exposure 60's file, filed at 14:07:01.78, became a photo at 14:07:02.82). A
reported shutter is **on the way**, and holds its slot in CS2 exactly as a hold does, while: it is an exposure of the
session, `now − exposed_at ≤ FIFO_WINDOW_MS`, and **no photo of the session has `raw_path` equal to the `final_path`
of one of its non-duplicate deliveries** (whether its file is not delivered, pending, or filed and not taken in).
`taken`, `default_shot`, `advance`, `current` and `set` use this set; nothing else in CS1–CS8 changes. MC-LV3's "a
container with a bound, pending exposure is not empty" is the invariant; the arrival of the pending file never moves
the current shot (it fills a container that was already taken).
**CS5 Binding an exposure** (`LedgerService::exposure_observed`, inside the ledger worker lock, before the exposure row
is inserted): `s` = the active session (as today). Templated: `b = current(s)`; the row is inserted with `shot_bound =
1`, `shot_container_id = b` (Free → NULL); then the held current becomes `advance(s, b)` with the new exposure counted
as a hold; `current_shot.bound …`; `AppEvent::SessionChanged{s}`. Untemplated or no session: `shot_bound = 0`. If
`match_late_exposure` matches the new exposure to a delivery whose photo already has `photo_places.shot_bound = 1`
(the file came first, CS6), the exposure takes that binding and the current shot does **not** advance again.
**CS6 Binding at arrival.** In `place_arrival`, a templated session's arriving photo whose slot owner is `None` and
whose place row has `shot_bound = 0` (Capture One tether, a file no exposure matched) is bound to `current(s)` on its
place row (`shot_bound = 1`) and the current shot advances as in CS5. The binding is written once; later placement
runs never re-bind.
**CS7 Changes by the person.** `set(s, Shot(c) | Free)`: `s` ≠ the active session → `SessionChanged`; untemplated →
`NoTemplate`; `c` not in the template → `NotInTemplate`; `c ∈ taken(s)` → `ShotFilled`; else held = value,
`current_shot.set … by=phone`, `SessionChanged{s}`. `retake(h, s, c)`: `s` ≠ active → `SessionChanged`; no photo in `c`
→ `NotInTemplate`; else in one transaction the photo `P` in `c` is denied exactly as `ReviewService::delete` does
(Deleted, `photo_denied` started, mirror tag) **and released** (`photo_places.released = 1`), one undo step "Retake
shot <final_no>" restoring `P` to Main in `c` with its previous place row; then held = `Shot(c)`, `by=retake`,
`SessionChanged{s}`. The next exposure binds `c` (CS5) and fills it with `c`'s presets. A session switch
(`ActiveSessionChanged`) clears the held value; so does an `ActiveSessionChanged` back to the same session.
**CS8 Hand actions under binding** (10G §6 rules 1, 2, 3, 5 and Shift the rest back stay). A hand action sets
`place_by_hand = 1` and never changes a binding. **Drop onto a filled container** (Mac, rule 3): the dropped photo is
template-placed in `c` with `photo_places.shot_bound = 1, shot_container_id = c` and `c`'s presets; the photo that was
there is denied and released (one undo step "Retake container <n>", as today). **Back to template** sets
`place_by_hand = 0` on the differing photos; placement puts each at its binding. **Shift the rest back**: let `G` = the
containers left empty by Deleted, unreleased photos (bound there, no photo in it); the Deleted ones are released; every
template-placed photo whose binding is a container at rank `r` in `order` (Condition-bound ones counting as ranks
`N+1, N+2, …` in shot order) gets `photo_places` binding = the container at rank `r − |{g ∈ G : rank(g) < r}|` (Condition
if beyond `N`); one undo step, as today. `can_shift_back` = that would change at least one expected place. 10G §6 rule
4's **refill is gone** (CS7's Retake and the tap replace it). The **Mac's "Next" outline** (`TemplateView.next_shot`) =
the `shoot_no` of `current(s)` for the active session, of `default_shot(s)` for any other.
**Invariant CS.** Every exposure of a templated session is bound once, at the moment the camera reported it, to the
shot current then; a photo is placed by its binding whenever its file arrives; the current shot never points at a
taken container when read; untemplated sessions are exactly as today (P66).

### PH — Pairing, the server and the phone wire (11 §Phone remote, LV-D7, LV-D12, LV-D18, card LV3)
**PH1 Pairing.** `create(name?)`: 32 random bytes (`getrandom`) → token = unpadded base64url (43 chars); stored only as
lowercase hex SHA-256 in `phone_pairings.token_sha256`; name default "Phone <n>" (n = count of pairings ever + 1);
returns the token **once** (never readable again). `verify(token)` = the row with that hash and `revoked_at IS NULL`,
updating `last_seen_at`. `revoke(id)` sets `revoked_at`; every open socket of that pairing is closed with code 4401
within 1 s; later requests answer 401. Pairings are independent.
**PH2 Server.** One `PhoneServer` per app process, created in the desktop setup after the first App; it resolves the
current `AppHandles` per request (so App rebuilds are invisible) and holds the runtime's hub. Listens on `0.0.0.0:<port>`
(default 8686) when `phone.enabled`; a bind failure → `phone.server error=` and `phone_server_status.error` (the app
runs on). Peers outside 10E MC-32's private ranges (plus loopback) are refused 403 (reuse `psm_infra::ftp::peers`).
Handlers only parse, check auth and the session guard, call core services through `spawn_blocking`, and serialize (H8).
**PH3 Auth.** Every `/api/phone/v1/*` request and the manifest carry the token as `Authorization: Bearer <token>` or
query `pair=<token>` (images and the WebSocket cannot set headers). Missing/unknown/revoked → 401 `{error: {code:
"pairing_unknown", message}}`. Static files (`/phone/`, `/assets/*`, `/phone/icon.png`) need no token.
**PH4 Routes** (JSON; errors `{error: {code, message}}`; codes from `wire::error_code`, unknown → 500 `internal`):
`GET /phone/` → `phone.html` · `GET /assets/<file>` · `GET /phone/manifest.webmanifest?pair=` → `{name: "Shoot",
short_name: "Shoot", start_url: "/phone/?pair=<token>", scope: "/phone/", display: "standalone", background_color:
"#ffffff", theme_color: "#ffffff", icons: [{src: "/phone/icon.png", sizes: "512x512", type: "image/png"}]}` · `GET
/api/phone/v1/hello` → `{api: 1, pairing: {id, name}, mac_name, features: ["live", "capture", "focus", "current_shot",
"move"]}` · `GET /api/phone/v1/session` → `PhoneSessionView` (PH6) · `POST /api/phone/v1/capture {session_id}` →
`{capture_id}` · `POST /api/phone/v1/focus {session_id, x, y}` → `{focus_id}` · `POST /api/phone/v1/focus/clear
{session_id}` → `{focus_id}` · `POST /api/phone/v1/current {session_id, container_id: <id>|null}` · `POST
/api/phone/v1/retake {session_id, container_id}` · `POST /api/phone/v1/photos/<id>/move {session_id, target: {kind:
"container", container_id} | {kind: "main_position", position} | {kind: "extra_end"} | {kind: "condition", position} |
{kind: "condition_end"}}` · `POST /api/phone/v1/photos/<id>/delete {session_id}` · `GET
/api/phone/v1/photos/<id>/preview?pair=` → the photo's current preview file, as the Mac's session view shows it ·
`GET /api/phone/v1/containers/<id>/example?pair=` → the template example file as stored (black and white is the
phone's CSS filter, H3). **Session guard:** every POST's `session_id` must be the active session, else 409
`session_changed` and nothing happens (a tap racing a Mac switch never acts on the wrong session). Later (LV-D18) a
`POST /api/phone/v1/sessions/<id>/complete` or `/open` and a new `features` entry are added without changing these.
HTTPS later = the same paths; the phone derives `ws:`/`wss:` and all URLs from `location` (card LV3).
**PH4a Sort and arrange routes (added 2026-11-27, v1.2, LV-D20; LV8·U1).** `POST /api/phone/v1/photos/<id>/classify
{session_id, to: "main" | "condition"}` → `{ok: true}`, core `template_placement::classify_to` (same guard,
`session_photo` 404 and errors as the move route). **To Main:** a Condition photo → Main at the end (as `ExtraEnd`); a
photo in a template container → an extra at the end and that container becomes current; a Main photo outside the
template (or any Main photo of an untemplated session) → nothing changes. **To Condition:** a Main photo → Condition at
the end (as `ConditionEnd`); from a container, that container becomes current; a Condition photo → nothing changes.
Any other photo state → `invalid_value`. The current shot after a take-out = `CurrentShots::set_after_take_out(s, c)`
(as Retake's; CS1 moves it on if `c` is taken), set after the commit and **before** the announcement. "Nothing
changes" = 200, no undo step, no event. Move targets added to `POST /photos/<id>/move`: `{kind: "extra_insert",
index}` (the source is an extra, or a Main photo of an untemplated session) and `{kind: "condition_insert", index}`
(the source is a Condition photo): the photo is taken out of its group and put back so that it is at `index` (1-based,
`1..=group length`), the others keep their order; positions only (no place row changes, as the Condition swap); a
source outside that group or an index out of range → `invalid_value`; the same place → nothing changes. One undo step
"Move image" for every change; PH8 announcements through `HandAction::finish`. `ReviewService::reorder` is **not** used
(it puts a Main photo by hand into slot = position, the Mac's container semantics). Every other PH4 route unchanged.
**PH5 WebSocket** `GET /api/phone/v1/ws?pair=<token>` (auth before upgrade: 401). Server → phone **text**:
`{type: "hello", viewer_id}` · `{type: "session", session_id: <id>|null, rev}` (on `ActiveSessionChanged`,
`SessionChanged`, `PhotoAdded`, `PhotoChanged` of the active session; `rev` increases; the phone re-reads `GET
/session`) · `{type: "camera", status, live: "live"|"unavailable"|"off", reason: <string>|null, exposure: {shutter,
aperture, iso}|null}` (on change) · `{type: "focus", focus_id, outcome: "focused"|"not_focused"|"no_report"|
"superseded"|"refused"|"failed", reason: <string>|null}` · `{type: "focus_frames", frames: [{cx, cy, w, h, state}]}` ·
`{type: "capture", capture_id, state: "acknowledged"|"exposed"|"failed"}` · `{type: "demand_ack", gen}`. Phone → server
text: `{type: "demand", gen, live, visible}` (answered by `demand_ack` after `set_demand` returned). Server → phone
**binary** (one frame per message): 32-byte little-endian header `0..4` `b"PSMF"` · `4` version `1` · `5` flags `0` ·
`6..8` reserved · `8..16` `seq` u64 · `16..20` `gen` u32 · `20..22` width u16 · `22..24` height u16 · `24..32`
`captured_at_ms` i64; then the JPEG bytes exactly as received from the sidecar (H3). **Lifecycle:** a WS ping every
`ping` (5 s); no pong or message for `idle_close` (15 s) → close 4408 and unsubscribe; ~~a frame write not finished in
`write_timeout` (2 s) → close 4408~~ (superseded v1.1 by PH5a); a close → unsubscribe at once; each viewer's frame buffer is a `LatestSlot` and its
event queue holds at most 64 messages (overflow → close 4409). Several phones each get their own viewer (LV-D12).
**PH5a Stalls (amended 2026-11-26, v1.1, finding F2; replaces "a frame write not finished in `write_timeout` (2 s) →
close 4408").** Over Wi-Fi one frame write regularly took longer than 2 s and the whole socket was closed (Android 5×
in 2 min, `phone.viewer closed … reason=write_timeout` 13:51–13:53; iPhone 14:15:43). Now: **one write in flight at a
time, never cut short** (a WebSocket message dropped mid-write corrupts the stream); while it is in flight newer frames
replace the one waiting in the viewer's `LatestSlot` (the phone skips frames, MC-LV1/HB4); pings are sent between
writes; a write that has not finished after `stall_close` (default 15 s) → close 4408 `"stalled"` and unsubscribe; no
message heard for `idle_close` (15 s) → 4408 as before; revoke (4401) and a reader that ended are served while a write
is in flight (the write is dropped, the socket closed, the viewer gone within 1 s, PH1). Text writes follow the same
rule. `PhoneTimings {ping: 5 s, idle_close: 15 s, stall_close: 15 s}` (§4.3).
**PH6 PhoneSessionView** (`phone_view`): `{session: null | {id, item: {name (item title), sku, article_number: <string>
|null /*the item's barcode, owner card 1 answered 2026-11-26*/}, templated, current: {kind: "shot", container_id} | {kind: "free"} | null (untemplated),
shots: [{container_id, final_no, shoot_no, name (container name or "Container <final_no>"), ratio: <f64>|null, split,
zoom, example: bool, photo: Image|null}] in shooting order, extras: [Image] (Main photos without a container, in Main
order), condition: [Image] (Condition order), counts: {main, condition}}}`, `Image = {photo_id, revision, ratio:
<f64>|null}`. `ratio` = `shot_ratio(presets, settings.default_shape)`: split → 2.0; `1:1` 1.0, `4:5` 0.8, `3:2` 1.5,
`2:3` 0.6667, `16:9` 1.7778; `original`/`free` → null (the phone uses the live frame's aspect). Untemplated: `shots:
[]`, `extras` = the Main list.
**PH6a Preview version (amended 2026-11-26, v1.1, finding F7).** `Image = {photo_id, revision, ratio, preview:
<string>|null}`. `preview` = null while the photo has no preview file the preview route can serve (`preview_file`
gives none, or that file is missing on disk); otherwise a token that changes whenever the served file changes (its path
or its modification time); never a path or any part of one. The preview route ignores extra query parameters. The
phone requests a preview only when `preview` is a string, with `&v=<preview>` in the URL, and re-requests when it
changes (the `session` message after the preview step's `PhotoChanged` brings the new token). Evidence: 31 × `404` on
`/photos/{id}/preview` at the hardware check, requested before the previews existed (`arrival.preview_ready` ≈ 2.7 s
after the photo row) with a URL that never changed.
**PH7 Phone client rules** (LV3). Token: `?pair=` on `/phone/` → `localStorage["psm.phone.token"]`, removed from the URL
(`history.replaceState`), manifest link `href="/phone/manifest.webmanifest?pair=<token>"` set at start (so an installed
iPhone app starts with its token). Every request sends `Authorization: Bearer`; images and the socket use `?pair=`.
A 401 clears the token and shows the pairing screen. **Demand:** a `gen` counter in memory; every change of showing
Live (hold, lock, unlock, release) or of visibility sends `{type: "demand", gen: ++gen, live, visible}`. A binary frame
is drawn only if its `gen` equals the current `gen` and its `seq` exceeds the last drawn; after a switch to Live the
live layer stays hidden (template at full opacity) until the first frame of the new `gen` is drawn. **Reconnect:** a
close while visible → reconnect after 500, 1000, 2000, 4000 ms, then every 4000 ms; `visibilitychange` → visible →
reconnect at once if closed; on open, send the current demand with a new `gen`. No service worker, no wake lock, no
other secure-context API anywhere in the phone app (card LV3). **Full screen** button shown iff
`document.fullscreenEnabled && !document.fullscreenElement && !standalone` (Android over HTTP; iPhone installs full
screen).
**PH8 Change announcements (added 2026-11-26, v1.1, finding F3).** PH5's `session` message depends on the App's
`SessionChanged`/`PhotoAdded`/`PhotoChanged`, but the Mac's review actions emitted none (`photos_delete` →
`ReviewService::delete`: nothing; LV3·U0 announced only the phone's own delete) and an undo/redo announced only step
kinds that report changed photos (Composition, Template). Every operation of a core or application service that
commits a change to what `phone_view` returns for a session emits, after the commit, `PhotoChanged{photo, s}` for each
photo whose place, classification or image it changed (where it knows them) and `SessionChanged{s}` for each session
it changed. Emitters: `ReviewService` (`classify` incl. restore from Deleted, `reorder`, `delete`, `keep_published`,
`move_photos` (source and target), `copy_photos` (target)); `HistoryService` undo and redo of **every** step kind (each
session in the step's touched set); `HandAction::finish`, `CurrentShots`, the preview step and arrival as today. Tauri
commands and HTTP handlers never emit (H8); the phone's delete uses the same announcing path as the Mac's (no double
event).
**PH9 Sort and arrange on the phone (added 2026-11-27, v1.2, LV-D20; LV8·U2–U4).** The phone's client rules; the Mac's
are PH4a. Names in §4.5.
- **PH9.1 Swipe (the Preview's picture).** A touch is `pressing`; a move of more than `SWIPE.startPx` (14 px, in the
  picture's own coordinates as every Preview gesture, so a sideways-held long shot swipes along its picture) within
  `holdMs` (200 ms) → `swiping`, the axis decided once: `|dx| > |dy|` → horizontal (left/right by the sign of `dx` at
  release), `dy < 0` and `|dy| ≥ |dx|` → up, otherwise (down) → no action. A template shot's photo held within 14 px
  for 200 ms → `holding` (its template shows, LV3; sideways sets the overlay; no swipe until release). In `swiping` the
  picture follows the finger along the axis and a hint shows the outcome. Release with the distance along the
  direction ≥ `commitPx` (96) → commit; shorter, `pointercancel`, page hidden or `blur` → cancelled (springs back, no
  request). `swipeAction(subject, direction)`:

  | subject \ direction | left | right | up |
  |---|---|---|---|
  | a template shot's photo | `retake` (POST `/retake`) | `main_confirm` (the confirmation; Move → classify main) | `condition` (classify) |
  | an extra Main photo (untemplated: any Main photo) | `delete` (POST `/photos/<id>/delete`) | `none` ("ALREADY MAIN") | `condition` |
  | a Condition photo | `delete` | `main` (classify) | `none` ("ALREADY CONDITION") |

  A commit with an action sends its request and the Preview shows `nextPhoto(view, id)` **at once** (the photos in the
  strip's order — template shots with a photo in shooting order, extras, Condition — the one after it; none → the
  Preview closes), then the toast and the haptic (§6). A refusal goes through the phone's `refused` (toast by code,
  re-read); the Preview does not go back.
- **PH9.2 Lift and drag (both strips).** `idle → pressing → tap | scrolling | moved | lifted`; `lifted → dragging →
  dropping | cancelled`; `dropping → idle` at PH9.5's ends; `cancelled → idle`. `pressing`: a pointer down on a tile,
  captured. More than `DOCK.tapPx` (8) sideways before `liftMs` (400) → `scrolling` (LV6); more than 8 vertically →
  `moved`; release within 8 px before 400 ms → tap (LV6's rules). 400 ms within 8 px on a tile **with a photo** and no
  drop pending → `lifted`: homes measured, the `arrangementKey` stored, `vibrate(12)`. A tile without a photo never
  lifts (its release is a tap). Lifted or dragging: the tile follows the finger; the target (PH9.3) is recomputed on
  every move and every auto-scroll step; `vibrate(6)` when it changes to a different non-null target; within `edgePx`
  (40) of the strip's visible left or right edge `scrollLeft` moves 8 px toward that edge every 16 ms, clamped to
  `[0, scrollWidth − clientWidth]`. Release: a null target → cancelled; `main_confirm` → cancelled and the
  confirmation opens (Move → classify main, then PH9.5 as a drop); otherwise `dropping`: the request,
  `vibrate([30,40,60])`. No HTML5 drag and drop anywhere.
- **PH9.3 Targets.** Homes = each tile's rectangle in content coordinates (client left + `scrollLeft`), measured **once,
  at lift**, never during the drag (tiles sliding aside move on screen, not their homes). Finger x = `clientX` − strip
  left + `scrollLeft`. In band = finger y within [strip top − `bandPx` (48), strip bottom + 48]; outside → null. Over
  tile `t` = `|x − centre(t)| < 32`. `span(g)` = [centre of g's first tile − 32, centre of its last + 32) for g = the
  extras or the Condition photos (none when g is empty).

  | source | over a template shot `t` ≠ source | in span(extras) | in span(Condition) | anywhere else (own tile, Free shot, gaps) |
  |---|---|---|---|---|
  | a template shot's photo | swap `t` → move `{container, t}` | `main_confirm` | to Condition → classify | null |
  | an extra | swap `t` | insert `i` → move `{extra_insert, i}`; `i` = 1 + the other extras whose centre < x; `i` = own place → null | to Condition | null |
  | a Condition photo | swap `t` | to Main → classify | insert `i` → move `{condition_insert, i}` (same rule) | null |

  (Untemplated: no template shots; the extras are the Main list.) Swap is card LV8 (an empty shot is filled);
  across groups is book 06 ("goes to the end of the target").
- **PH9.4 Cancels.** In `pressing`, `lifted` or `dragging`, each of these ends the gesture with no request and the tile
  back home: `pointercancel`; a pointer down with another `pointerId`; `visibilitychange` to hidden; window `blur`;
  the strip unmounting; a session view whose `arrangementKey` differs from the stored one; release with a null target.
  `arrangementKey(view)` = the shots' `container_id:photo_id|-` in shooting order, then the extras' ids, then the
  Condition ids; preview tokens, revisions and the current shot are not in it (a picture arriving never cancels).
- **PH9.5 While the Mac answers a drop.** The strip shows `applyDrop(base, photo, drop)` (`base` = the view at the drop;
  swap exchanges, an empty target leaves the source's place empty and a non-template source takes the target's photo
  into its own place; insert reorders; to Main / to Condition appends to that group) until: (a) the request succeeded
  and the first session read **started after** its answer returns → that view; (b) the request failed → the last view
  from the Mac, `refused` (toast by code), a re-read; (c) `pendingMs` (8 s) without an answer → the last view from the
  Mac and a re-read, no toast. While a drop is pending nothing lifts (a long press is a tap). Swipes are not shown
  ahead in the strip.
- **Invariant PH9.** A drag sends at most one request, only on a release with a non-null target computed from homes
  measured at lift; the strip never shows a pending arrangement after one of PH9.5's ends; the phone never decides a
  placement: every outcome on screen after an end is the Mac's re-read.

**Invariant PH.** No request without a live pairing reaches a service; no POST acts on a session other than the
active one; the phone uses only relative URLs and no secure-context API.

## 6. Delegated choices decided once
- **Frame channel = Unix socket** (FC1), not an inherited fd: Rust's `std::process::Command` cannot pass an extra fd
  without `unsafe` `pre_exec`; a loopback TCP port would be reachable by any local process. `$TMPDIR` is per-user, mode
  0700, and short enough for the 104-byte socket path limit.
- **Hub in core** (`services/live_view.rs`, std primitives only; core has no tokio). Outlets adapt it: the WebSocket
  viewer in `phone/ws.rs`; later the Tauri channel and machine guidance implement `LiveOutlet`.
- **Hub and `CurrentShots` live with the camera runtime** (one per process, carried through rebuilds, 10E R-I18).
- **Server in `psm-integrations`** (`src/phone/`), on its own thread with its own tokio runtime as `NudgeServer` does
  (`crates/infra/src/nudge.rs`); started by `apps/desktop/src-tauri/src/lib.rs`.
- **Schema by `ensure`**, not a numbered migration (§3 Data): this branch runs beside `main`, whose next number is
  unknown at the join; `ensure_research_columns` set the precedent. Not an `ALTER … ADD COLUMN` migration, so the
  rewind test of `p27_retire_shape_on_arrival.rs` is not touched.
- **Port 8686** default (`phone.port`), refused if < 1024, equal to the FTP control port, or inside the FTP passive
  range (`phone_port_invalid`).
- **QR in Rust** (`qrcode` SVG string) returned by `phone_pairing_create`; no npm dependency. The QR encodes the
  `.local` URL (`scutil --get LocalHostName` + `.local`, evidence P2: use the name); the address URL (first private
  IPv4 of `psm_infra::net::local_addresses`) is shown as text with its own QR (the iPhone could not reach the Mac by
  name once, Experiment 11).
- **Phone texts** (exact): "Scan the QR code in Session Manager → Settings → Phone to connect this phone." (no token) ·
  ~~"This phone was removed in Session Manager. Scan a new QR code to connect it again."~~ (401; **superseded v1.1,
  LV-D19 (2)**: "This phone was removed in Session Manager. Delete this app from the home screen, then scan the new QR
  code in Session Manager → Settings → Phone and add it again.", for the 401 and the socket's 4401) · "Camera not connected" ·
  "Live unavailable on this camera" · "Camera busy" · "Could not focus there" (not focused) · "Set the camera's Focus
  Area to a movable spot (e.g. Flexible Spot) to focus from the phone" (refused) · "Not taken. Try again." (capture
  failed) · "Session changed on the Mac" (409). Toasts otherwise exactly as the design README §7 lists, plus "Condition
  image added" for a capture bound to no shot (the design's "Extra image NN added" does not occur: card LV7).
- **Phone fonts:** IBM Plex Mono bundled with the phone build (`@fontsource/ibm-plex-mono`, weights 400 and 500, OFL;
  a new npm dependency), never fetched from Google at runtime; UI text "Helvetica Neue", Helvetica, Arial.
- **Split halves** are labelled "Left · 1:1" and "Right · 1:1" (LV-D16; the design's A/B).
- **Phone Delete** (extras, condition) = Deny (`ReviewService::delete`); **Move** = swap (card LV8), a hand action.
  **v1.2 (LV-D20 (4)):** the Delete button is gone; the left swipe does the same request.
- **Phone texts v1.2 (LV8, exact):** the confirmation "Move this photo to Main, outside its template order?" with
  buttons "Move" and "Cancel" · swipe hints "DENY", "MAIN", "CONDITION", "ALREADY MAIN", "ALREADY CONDITION" · toasts
  "Image deleted" (left, extra or Condition), "Shot NN reopened" (left, template shot), "Moved to main images" (right),
  "Sent to condition images" (up) · a successful drop shows no toast; a refused one shows the `refused` text by code
  ("Session changed on the Mac", "That did not work. Try again."). Haptics (design README §Haptics): lift 12, target
  change 6, drop and Main/Condition swipes `[30,40,60]` (Move image), Deny `[30,40,30]` (Retake / delete).

## 7. Tracker
Each plan file's **Tracker** line is that milestone's row (sessions change only that line). Current: all four
`NOT_STARTED` (2026-11-26). (v1.1: LV1–LV3 are APPROVED per their plan files; LV4 is paused part-way, record in
`STATUS_live.md`; `LV5_live_fixes` and `LV6_phone_dock` written `NOT_STARTED` 2026-11-26.)

## 8. Environment (verified on this Mac, 2026-11-26, worktree `Photo-Session-Manager-live`)
| What | Command (repo root unless stated) | Measured |
|---|---|---|
| Sidecar stub + its tests | `tools/sidecar/build.sh --stub --test` | 2.9 s; ctest `100% tests passed out of 1` (the `transport_manager` program); writes `apps/desktop/src-tauri/binaries/camera-sidecar-aarch64-apple-darwin` (git-ignored). After any C++ edit re-run it (`build.rs` never rebuilds an existing binary). |
| Sidecar with the SDK (LV4 only) | `tools/sidecar/build.sh --sdk <lab>/vendor/sony-sdk` | not run by planning (SDK lives in the lab, never in the repo) |
| Behaviour tests, some stems | `cargo test -p psm-testkit --test suite -- t2_template_placement:: p45_sidecar_and_protocol::` | 1 min 0.6 s from a cold build of this worktree, 36 passed, 1428 filtered. `cargo test … --test suite a:: b::` (two filters before `--`) is **refused** by cargo. |
| Full Rust suite | `cargo test --workspace` | not re-measured; flow §4A: ~9 min warm |
| Frontend deps | `cd apps/desktop && npm ci` | **needed once in this worktree** (no `node_modules` here); 2.3 s |
| Typecheck | `cd apps/desktop && npm run typecheck` | 3.3 s, clean |
| UI tests, a folder | `cd apps/desktop && npx vitest run src/features/sessions` | 13 files, 67 tests, 5.8 s |
| Network for new crates | `curl -sI https://index.crates.io/config.json` | HTTP/2 200 (crates.io reachable; §4.3 dependencies fetch once) |
| Fake sidecar | `crates/testkit/src/bin/fake_sidecar.rs` (script directives: JSON line, `sleep`, `await`, `exit`, `stderr`; `rec/inject.txt`); started by `TestApp::with_camera_script…` | LV1 adds `--frame-socket` and the `frame`/`frames` directives (LV1·U3) |
| Phone entry in dev | `cd apps/desktop && npm run build` → `dist/phone.html`; a debug app serves `apps/desktop/dist` (or `PSM_PHONE_WEB=<dir>`) on port 8686 | built and checked in LV2·U4 |
| Phone entry packaged | `npm run tauri -- build --bundles app` → `Session Manager.app/Contents/Resources/phone-web/phone.html` | checked once at LV4 preparation (release build) |
Closing gate of every build milestone, once, one command after the other: `cd apps/desktop && npm run build` (first:
`psm-desktop` needs `apps/desktop/dist/`; LV2 review note) · `cargo test --workspace` · `cargo clippy
--workspace --all-targets -- -D warnings` · `cargo fmt --all -- --check` · `tools/sidecar/build.sh --stub --test` ·
`cd apps/desktop && npm run typecheck` · `npm test`. If reality disagrees with this table, the session updates it.

## 9. Sequencing
`LV1` → `LV2` → `LV3` → `LV4`, strictly (each needs the previous APPROVED). **v1.1 (2026-11-26, the hardware check
paused for fixes):** `LV1` → `LV2` → `LV3` → `LV5` → `LV6` → `LV4` (resumed). `LV5` needs `LV3`; `LV6` needs `LV5`
(the `preview` token of PH6a, served by LV5·U2); `LV4` resumes on a packaged build that includes `LV6`, owner-attended,
never run by the runner; its rows already passed are kept, the rows that found F1–F5, F7 (C18, the exposure readout,
the stall rows, the dock look) are re-run. (The LV4 plan file's `depends_on` still reads `LV3_phone_app`; the
orchestrator updates it when resuming, it is outside this planning session's write perimeter.) `LV2` needs `LV1`'s hub and focus API;
`LV3` needs `LV2`'s routes and wire (frozen in §5 PH, so `LV3`'s tests mock them); `LV4` needs the packaged build of
`LV3`. Owner card 1 answered 2026-11-26: "Article number" is the item's barcode (the Shopify product barcode Session Manager
already reads); LV2·C9(b) fills it, LV3·C24(a) hides the row only when an item has no barcode.
**v1.2 (2026-11-27):** `LV7` (approved) → `LV8_phone_sort_and_arrange` (`depends_on: [LV7_tile_loading_cue]`; it edits
the same phone files and keeps LV7's waiting tile). Runner: `tools/pipeline/run_phase.sh phase
LV8_phone_sort_and_arrange`. LV4 is done (STATUS_live); LV8's own try-out is on the hardware by the owner.
