---
phase: P44_capture_ledger
title: Capture ledger — the lab ledger ported as a core mechanism with the inventory's fixes (file key, association order with a 300 s FIFO window, global duplicates, no-replace rename, target folder at rename time, folder-key freeze, MC-2 rule 0 and captured_at), persisted in two tables with crash-ordered writes, the watchdog counts; the receiver wired to it
difficulty: high
projection: done
depends_on: [P43_ftp_receiver]
---
# P44 — Capture ledger (MC-33), persistence, target folder, arrival seam

## Goal
10E MC-33 with the inventory additions R-I25…R-I29 (R-I26 as clarified by changelog 36), the amended 05A MC-2 (rule 0) and MC-4 (`captured_at`),
and 05B MC-22 (freeze at a committed delivery); invariants M37 (first copy under a candidate name, identical copies in `.duplicates/`, nothing
overwritten) and M38 (one `photos` row per distinct camera file, in the session the association rule gives). The pure mechanism
`capture_ledger.rs` (file key, association order, duplicate decision, FIFO window and the candidate-set session rule, completion, gaps, pending
count) carries the lab's 15 ledger tests reinterpreted for the 10E model; the `LedgerService` is the **one worker** (one process-wide lock, owned by
the `CameraRuntime`) that decides → writes the `pending` delivery row with its final path → renames with `renamex_np(RENAME_EXCL)` (never
`std::fs::rename`) → marks `done`, puts identical later copies in `.duplicates/`, renames same-name-different-bytes files `__dupN`, reads the item's
folder key **at the moment of the rename**, freezes it, and recovers `pending` rows at every bind. `ArrivalService` learns rule 0 (a file whose
canonical path is a delivery's `final_path` joins that delivery's session with `captured_at = exposed_at`, else `completed_at`). The receiver's sink
becomes the ledger (replacing P43's `NoopSink`), so an upload through the real receiver ends as exactly one photo.
**Not in this phase:** the sidecar and its events (P45/P46 — exposures are recorded here through `CaptureSink::exposure_observed` called by tests),
the adapter's Sony filename normalisation (`_N`, P46), the watchdog's `ftp-status` request and reconnect button (P46), the health UI and the
dismissal of `camera_photo_session_uncertain` (P51), and **R-I28's last sentence** — `ArrivalService` taking the capture root from the settings
formula while the direct runtime is bound — which is **P46 Task 2 / row C8(c)** (projection r0 L18). In this phase `ArrivalService::capture_root`
still asks Capture One (`services/arrival.rs:57–68`); in `TestApp` both formulas resolve to the same path, so no P44 row can see the difference.

## Read first
Book: `10E_direct_camera_adapter.md` MC-33 (whole, incl. **R-I25** persistence, **R-I26** file key and association order **with the orchestrator's
clarification paragraph** (rule (1) needs the same SHA-256, rule (2) is bounded by 300 s; changelog entry 36), **R-I27** duplicates and the worker,
**R-I28** target folder and attribution, **R-I29** watchdog) and §Invariants M37, M38 · `05A_mechanism_contracts.md` MC-2 (capture root, candidate
filter, **rule 0**, rules 1–3, row creation) and MC-4 (`captured_at` exception) · `05B_sku_generation_and_change.md` (the stored folder key follows
the SKU until frozen; **freeze at a committed delivery**) · `14_measurement_ledger.md` M9, M29, M37, M38 · `16_changelog.md` entry 36 · process
`flow.md` §4 (core mechanisms: "RAW hash, position math, gallery reconciliation diff" — the ledger joins that list), §6, §9 · master plan §3, §6.3
(`.incoming`, `.duplicates`), §6.4, §6.5 (**migration 006**), §6.6 (ledger enums; `camera_photo_session_uncertain`, `camera_files_not_received`),
**§6.11** (Constants, Domain, Ports `CaptureSink`/`DeliveryFs`/`CompletedNames`, Mechanisms `capture_ledger.rs`, Services `LedgerService`, Runtime
`bind` (b)), §7.5.
Code at HEAD (`2e78bab`, 2026-11-06; line numbers checked there):
- `crates/core/src/services/arrival.rs:243–284` (`accept`: attribution `:249–258` and `captured_at` from mtime `:259–264` — **rule 0 goes before
  both**), `:57–68` (`capture_root`, asks Capture One and caches), `:91–153` (`scan`); `crates/core/src/mechanisms/arrival.rs:12–36` (`is_candidate`;
  the dot-directory rule `:23–28`; depth ≤ 2 `:20–22`), `:38–60` (`attribute`).
- `crates/core/src/services/items.rs:254–275` (`update_identity`: `tx.item_has_photos` at `:269` decides whether `folder_key` follows the SKU — **the
  freeze joins it**, inside `store.transaction`, so the predicate is a `StoreTx` method), `crates/infra/src/sqlite/store.rs:597–599` (`item_has_photos`,
  `StoreTx` impl), `:1375–1429` (`arrival_insert`; canonicalises `raw_path` at `:1376`), `:1430–1440` (`session_folders`), `:1159–1170`
  (`active_session_id`); `crates/core/src/ports/store.rs:33–262` (`StoreTx`, `Store`); `crates/infra/src/sqlite/migrations/002_domain.sql:1–30`.
- `crates/core/src/mechanisms/hash.rs:5` (`sha256_file`), `crates/core/src/mechanisms/canon.rs`, `crates/core/src/mechanisms/naming.rs`
  (`folder_key`), `crates/core/src/services/health.rs:76–105` (`set` `:76–84` **keeps one item per code** — a second `set` of the same code replaces
  the detail; `set_ok`, `clear`, `has`).
- `crates/core/src/domain/camera.rs:4–8` (the constants), `:118–131` (`ExposureObserved` — **no `expected_files` field**; `Delivery`).
- P42/P43 by their §6.11 names: `CameraRuntime::{bind, set_capture_sink}` and `RoutingSink`/`RoutingNames` (`crates/integrations/src/camera/
  runtime.rs:32–66`; `AppBinding` `:32–40` has **no `store`** — Task 5 adds it), `NoopSink` (`crates/integrations/src/camera/mod.rs:11–24`),
  `CompletedNames` (`crates/infra/src/ftp/mod.rs:17–19` — **moves to core**, Task 0), the receiver's sink call (`crates/infra/src/ftp/session.rs:
  140–146`: a sink `Err` becomes an io error → `426 Transfer aborted` `:342`, so the camera re-sends into the **same** `.part`), `App::open_with_adapter`
  (`crates/integrations/src/lib.rs:348`) and `App::build` (`:400`).
- Testkit: `TestApp::with_camera` installs `CollectingSink` (`crates/testkit/src/test_app.rs:213–215`), `kill_and_restart` (`:233`, builds a **new**
  `CameraRuntime` through `App::open_with_adapter`), `deliveries()` (`:297–299`, reads the `CollectingSink` — re-pointed by Task 6), `mark_published`
  (`:91–100`, the second-connection SQL pattern used for fixture writes), `capture_root()` (`:300`); `ManualClock` has only `advance(ms)`
  (`crates/testkit/src/clock.rs:18`) — "set the clock to `at`" means `advance(at − now)`; `FtpClient::cmd` (`crates/testkit/src/ftp_client.rs:27`) —
  there is **no `size()`**, C8(a) sends `cmd("SIZE DSC00001.ARW")`; `settle(a, pred)` is a **local fn** in `crates/testkit/tests/p42_camera_runtime.rs:16`
  (copy it into `p44`); `crates/testkit/tests/p05_capture_arrival.rs:31–55` (`db`, `count`, `session`, `watch`, `hook` helpers — there is **no `file`
  helper**; tests write files with `std::fs`); `crates/testkit/tests/p43_ftp_receiver.rs` (`part(root, name)` `:35`, the standalone `receiver(..)`
  helper `:47–66` and its `CollectingSink` rows `:358–426`; C1(a) `:100–122`, C2(g) `:261–274`, C4(a)–(c) `:314–356` — the rows Task 6 edits).
- Lab (read-only): `camlab/ledger.py` (`stem_of` `:52–58`, `seq_of` `:61–66`, `file_key` `:84–86`, `_file` association `:191–240`, `_refresh_duplicates`
  `:254–261`, `_update_status` `:270–282`, `finalize` `:285–293`, `sequence_gaps` `:295–307`, eligibility `unique_files < expected_files` `:116–119`),
  `tests/python/test_ledger.py` (the 15 tests; mapping below).
- `libc 0.2.191` (`Cargo.lock:2344–2345`): `renamex_np(from, to, flags)` and `RENAME_EXCL = 0x4` (`src/unix/bsd/apple/mod.rs:5010`, `:3557`): −1 with
  `errno EEXIST` instead of replacing. The tempdir is on APFS, so `Exists` is producible in the inline test.

## Files expected to change
Core (new): `crates/core/src/mechanisms/capture_ledger.rs`, `crates/core/src/services/camera_ledger.rs`. Core (changed): `crates/core/src/mechanisms/
mod.rs`, `crates/core/src/services/{mod.rs, arrival.rs, items.rs}`, `crates/core/src/ports/{camera.rs (`CompletedNames` lands here), store.rs}`,
`crates/core/src/domain/camera.rs` (ledger records: `Exposure`, `DeliveryRow`, `NewExposure`, `NewDelivery`).
Infra (new): `crates/infra/src/camera_files.rs`, `crates/infra/src/sqlite/migrations/006_camera_ledger.sql`. Infra (changed): `crates/infra/src/lib.rs`,
`crates/infra/src/sqlite/store.rs`, `crates/infra/Cargo.toml` (`libc`), `crates/infra/src/ftp/mod.rs` (`CompletedNames` becomes `pub use
psm_core::ports::camera::CompletedNames`). Workspace (changed): `Cargo.lock` (`libc` added to `psm-infra`'s dependencies).
Integrations (changed): `crates/integrations/src/camera/{runtime.rs, mod.rs}` (`NoopSink` removed; the ledger bound per `App::build` over the
runtime's one `LedgerShared`; the `delivery_fs` slot), `crates/integrations/src/lib.rs`.
Testkit (changed): `crates/testkit/src/test_app.rs` (`ledger()`, `deliveries()` re-pointed to the ledger's rows, `ledger_fs_fail_next_rename()`,
`set_expected_files(exposure_id, n)`; `with_camera` no longer installs `CollectingSink`), `crates/testkit/src/collecting_sink.rs` (kept for P43's
standalone rows; its `CompletedNames` import follows the move). Tests (new): `crates/testkit/tests/p44_capture_ledger.rs`. Tests (changed):
`crates/testkit/tests/p43_ftp_receiver.rs` (Task 6: the `TestApp`-fixture rows that read the `.part` or set names — see Task 6; the standalone rows are
untouched), `crates/testkit/tests/p01_storage_identity.rs` (the table list at `:82–255` gains `camera_exposures` and `camera_deliveries` with their
columns).
Derived from the lists above: 4 new source paths, 17 changed source paths (core 7, infra 4, workspace 1, integrations 3, testkit src 2), 1 new test,
2 changed tests — **24 paths**.

## Core mechanisms (unit tests allowed)
`crates/core/src/mechanisms/capture_ledger.rs` — `file_key`, `stem_key`, `associate`, `is_complete`, `duplicate_decision`, `dup_name`, `target_folder`,
`pending_count`, `sequence_gaps`: inline tests named `mc33_<slug>` (criteria C1 and C2 below, the ported lab tests; **pure rows assert only the
function's return value**, never service state). `psm_infra::camera_files::rename_exclusive`: one inline test (`EEXIST` → `Exists`, otherwise `Moved`).
Everything else is a behaviour row through `TestApp`.

## The ledger contract (master plan §6.11 Mechanisms + Services)
- **Types** (`capture_ledger.rs`): `FifoCandidate { exposure_id: i64, exposed_at: i64, session_id: Option<i64>, expected_files: Option<u32> }`;
  `PriorDelivery { id: i64, file_key: String, stem_key: String, sha256: String, exposure_id: Option<i64>, session_id: Option<i64>, outcome:
  DeliveryOutcome, completed_at: i64, final_name: String }`; `AssociationDecision { exposure_id: Option<i64>, association: Association, session_id:
  Option<i64>, ambiguous_with: Vec<Option<i64>> /* other sessions in the candidate set; `None` = Unassigned */ }`.
- **Exposure rows** are written by `CaptureSink::exposure_observed(e)`: `session_id = store.active_session_id()` read at that moment (`None` =
  Unassigned), `status = expecting`, `expected_files = None` (no input path sets it in this chapter; tests set it by SQL, see C3(e)), `initiated` from
  `e`. Camera-initiated exposures are candidates like any other. `session_uncertain()` sets every `expecting` row to `uncertain`.
- **Association order** for a delivery `(camera_filename, sha256, completed_at)`: `key = file_key(camera_filename)` (strips only `__dup<n>`, upper-cases);
  (1) a prior **`done`** delivery with the same `key` **and the same `sha256`** (an identical copy) → its exposure (`filename`); (2) a prior `done`
  delivery with the same `stem_key` and `|completed_at − prior.completed_at| ≤ FIFO_WINDOW_MS` → its exposure (`stem`); (3) FIFO: candidates =
  `expecting` exposures with `exposed_at ≤ completed_at` and `completed_at − exposed_at ≤ FIFO_WINDOW_MS` (both inclusive) that are **still expecting
  a file** — the count of non-duplicate priors with that `exposure_id` is `< expected_files.unwrap_or(1)` — oldest first → the oldest (`fifo`,
  provisional); the delivery's **session** is the candidates' common `session_id` when they all agree, else the picked exposure's, and
  `ambiguous_with` lists the candidates' sessions that differ from the picked exposure's (distinct, in candidate order) → health `camera_photo_session_uncertain`, detail `Photo
  <camera filename> may belong to <title>` where `<title>` is the other session's item title (SKU when the title is empty, `Unassigned` for `None`);
  **latest wins** (`HealthService::set` keeps one item per code) and in P44 the item is cleared only by a restart (dismissal is P51); (4) none →
  `association: none`, session = `store.active_session_id()` at completion. `uncertain` and `dismissed` exposures are never candidates. **Split of
  work:** `associate(file_key, stem_key, sha256, completed_at, prior: &[PriorDelivery], candidates: &[FifoCandidate])` applies every filter above itself
  (the sha match, the stem window, `exposed_at ≤ completed_at`, the window, "still expecting"); the service passes **every** exposure with status
  `expecting` and **every** `done` prior with the same `key` or `stem_key` — it never pre-filters by time or count, so the unit rows feed all exposures
  plus the growing `prior` list and cannot assert their own fixture. Rules (1)/(2) return the prior's `exposure_id` **and** `session_id` even when the
  prior had no exposure (RAW + JPEG of an exposure-less shot stay together).
- **Completion:** `is_complete(expected_files: Option<u32>, non_duplicate_count: u32) -> bool` = `expected_files.is_some_and(|n| non_duplicate_count
  >= n)`; elapsed time never completes a shot. **The matched exposure becomes `received`** when `expected_files` is `None` or its non-duplicate
  deliveries reach `expected_files`; until then it stays `expecting` (a FIFO candidate, counted by the watchdog). The status is written **in the same
  transaction as the `pending` insert**. `LedgerService::complete_for(exposure_id) -> bool` wraps `is_complete` over the stored rows (informational;
  nothing in the product waits for it).
- **Duplicate decision:** a prior `done` delivery (any exposure, any age) with the same `key` **and** the same `sha256` → `Duplicate { of }` → final path
  `<capture root>/.duplicates/<name>`; same `key`, other hash → `RenamedPhoto { n }` with the lowest free `__dupN` (computed from the priors' final
  names, then bumped on `Exists`); else `Photo`. **Decisions are made against `done` rows only.** A delivery whose `written_path` equals a `pending` row's
  `written_path` **takes over that row** (the camera re-sent the same staging file after a `426`): the row's `sha256`/`size`/`completed_at` are updated,
  the decision is re-made, no second row is inserted. **One worker** (the runtime's process-wide `LedgerShared.worker: Mutex<()>`, held from decision to
  `done`): decide → `insert_delivery(pending, written_path, final_path)` + the exposure status → `fs.create_dir_all(target)` → `fs.rename_exclusive
  (written_path, final_path)`; on `Exists`: SHA-256 of the existing file equal → the row becomes `duplicate` with final path under `.duplicates/`
  (renamed there, again exclusive; a collision there gets `__dupN` under `.duplicates/`), different → the next `__dupN`; then `set_delivery_done`.
  **Any rename error other than `EEXIST`** (`ENOTSUP` on a non-APFS volume, `EXDEV`, `ENOENT`) is returned as `Err` — never a fallback to
  `std::fs::rename`, `link` or a copy; the row stays `pending` and the take-over rule above keeps the photo when the camera re-sends. `std::fs::rename`
  is never called on a camera file.
- **Target folder** = `target_folder(capture_root, folder_key)` where `folder_key` = the stored `items.folder_key` of the session's item **read inside
  the rename step**, `_Unassigned` when the session is `None` or gone. A delivery with a session **freezes** the item's folder key: `StoreTx::
  item_folder_key_frozen(item_id) -> bool` = `item_has_photos(item_id) OR EXISTS (camera_deliveries WHERE outcome IN ('photo','renamed_photo') AND state
  IN ('pending','done') AND session_id IN (sessions of the item))` (counting `pending` closes the read→`done` window; `duplicate` rows commit nothing to a
  session folder), used by `ItemsService::update_identity` in place of `item_has_photos` (`items.rs:269`); a `Store` variant with the same body for
  C4(e). No new column on `items`; no write.
- **MC-2 rule 0** (`ArrivalService::accept`): `store.delivery_for_path(canonical)` → `Some((session_id, exposed_at_or_completed_at))` → that session
  (`None`/deleted → 1) and that `captured_at`; otherwise rules 1–3 and the mtime as today. Reads `pending` rows too (the watcher may fire before `done`).
- **Recovery** (`recover_pending`, at every `bind`, **under the same worker lock**): a `pending` row whose `written_path` exists → rename again to the
  row's **stored** `final_path` (on `Exists`: the same hash → `.duplicates/` / different → `__dupN` logic, updating the row; the folder key is **not**
  re-read); whose `final_path` exists → `done`; neither → left `pending` and logged.
- **Watchdog:** `pending_count(now)` = `expecting` rows with `WATCHDOG_MIN_AGE_MS < now − exposed_at < WATCHDOG_MAX_AGE_MS` (both exclusive);
  `watchdog(now, transfer_active: bool) -> WatchdogState { pending_count, gaps, raised, first_raise: bool }` sets `camera_files_not_received` iff
  `pending_count > 0 && !transfer_active` (detail `<n> photos taken but not received for 60 s` + ` · <g> gaps in the camera's numbering` when `g > 0`),
  clears it when the count is 0 or after `dismiss()` (those rows → `dismissed`); `first_raise` is true on the transition into raised (P46 sends
  `ftp-status` then); the raised flag lives in `LedgerShared` so it survives a settings save. **Gaps** come from `sequence_gaps` over the stems of
  deliveries with `completed_at > now − WATCHDOG_MAX_AGE_MS` (`seq_of` rule: trailing `_<1–2 digits>` stripped **only** for gap reporting); `g` =
  the number of `Gap` entries; logged `camera.sequence_gap`.
- **Decided by the orchestrator after projection r0 (not open):** `CaptureSink::transfer_in_progress` is a **no-op** in P44 (P46 computes
  `transfer_active` itself) · `missing_at_deadline` is **reserved**: it stays in the enum and the 006 CHECK list and **nothing in P44 writes it** (no
  deadline sweep) · `completed_names_since(ms)` → `(camera_filename, size, completed_at)` of `done` deliveries, **deduplicated by name keeping the
  latest `completed_at`** (the receiver's listings) · `exposures(status)`, `deliveries()` for tests.
- **Runtime wiring** (§6.11 Runtime (b)): the `CameraRuntime` owns one `ledger_shared: Arc<LedgerShared { worker: Mutex<()>, raised: Mutex<bool> }>`
  created with the runtime (one per app process, R-I18); every `bind` builds a `LedgerService` over the new store **sharing it** and runs
  `recover_pending` under `worker`. The runtime also holds `delivery_fs: RwLock<Arc<dyn DeliveryFs>>` (default `CameraFiles`), read at each `bind`;
  `set_delivery_fs(fs)` is the test seam (C5(a), C5(d)). `kill_and_restart` builds a fresh runtime, so recovery runs over the real `CameraFiles`.

## Lab test mapping (15 → rows)
`stem_and_sequence_helpers` → C1(a),(b) · `single_capture_single_raw_is_provisional_not_complete` → C2(a) · `expected_count_allows_completion` →
C2(b) · `raw_plus_jpeg_grouped_by_stem` → C2(c) · `burst_with_lagging_uploads_assigned_in_order` → C2(d) · `bracketing_with_expected_files` → C2(e) ·
`file_before_any_capture_is_unassociated` → C2(f) · `failed_ack_never_receives_files` → not a ledger case any more (a refused capture creates no
exposure row; P46 C2) · `finalize_marks_missing_not_complete` → C6(c) · `uncertain_after_session_loss` → C7(a) · `sequence_gap_detection` → C6(d) ·
`sdk_filename_then_ftp_copy_matches_by_name` → C3(a) · `ledger_persists_json` → C5(a) (SQLite instead of JSON) · `same_file_by_sdk_and_ftp_is_a_duplicate`
→ C3(a) · `duplicates_do_not_complete_a_multi_file_capture_and_raw_jpeg_are_distinct` → C3(e).

## Ordered tasks
0. **Registry first** (§6.4, §6.5, §6.11): migration `006_camera_ledger.sql` (**with `written_path TEXT NOT NULL`**); move `CompletedNames` to
   `psm_core::ports::camera` and re-export it from `psm_infra::ftp` (P43's `use psm_infra::ftp::CompletedNames` keeps working); `Store`/`StoreTx`
   methods `insert_exposure`, `set_exposure_status`, `set_exposures_status_where(from, to)`, `exposures(status: Option<ExposureStatus>)`,
   `insert_delivery`, `update_pending_delivery(id, sha256, size, completed_at, final_path, outcome, association, exposure_id, session_id)` (the
   take-over), `set_delivery_done`, `deliveries_by_key`, `deliveries_by_stem`, `pending_deliveries`, `pending_delivery_for_written_path`,
   `delivery_for_path`, `completed_names_since`, `item_folder_key_frozen` (both `Store` and `StoreTx`); the domain records.
1. `mechanisms/capture_ledger.rs` + unit tests (C1, C2).
2. `camera_files.rs` (`rename_exclusive`, `DeliveryFs` impl `CameraFiles`).
3. `services/camera_ledger.rs` (`LedgerService: CaptureSink + CompletedNames`, built over a `LedgerShared`; the worker with the take-over rule;
   recovery; `complete_for`; watchdog).
4. `arrival.rs` rule 0 + `captured_at`; `items.rs` freeze (`tx.item_folder_key_frozen`).
5. Runtime: `AppBinding` gains `store`; `CameraRuntime` gains `ledger_shared` (created in `App::open_with_adapter` with the runtime) and
   `delivery_fs`/`set_delivery_fs`; `bind` builds the `LedgerService` over the new store sharing `ledger_shared`, installs it as the receiver's sink
   and names provider (`set_capture_sink`), and runs `recover_pending` under the worker lock; `NoopSink` removed.
6. Testkit helpers (`ledger()`, `deliveries()` → the ledger's `done`+`pending` rows mapped to `Delivery` with `written_path`,
   `ledger_fs_fail_next_rename()` → `set_delivery_fs(FailNextRename(CameraFiles))`, `set_expected_files(exposure_id, n)` → `UPDATE camera_exposures SET
   expected_files=?2 WHERE id=?1` through a second connection (the `mark_published` pattern)); tests `p44_capture_ledger.rs`; **the p43 edits**:
   C1(a) (`:100–122`) and C2(g) (`:261–274`) read `root/_Unassigned/<name>` (no active session) instead of the `.part` and drop the "capture root has
   one entry" count; C4(a)–(c) (`:314–356`) move to a standalone receiver with `CollectingSink` (the `receiver(..)` helper `:47–66`) because listings
   now come from the ledger; the `a.deliveries().is_empty()` / `.len()` rows (C2(a), C2(c), C2(e), C2(f), C3(c), C8(a)) stay unedited and meaningful
   through the re-pointed helper; the p01 table list; suite, clippy, fmt, typecheck.

## Criteria (8 criteria, 36 rows; 0 `[rule]` rows)
Fixtures: unit rows call the mechanism functions directly (times in ms from `T0 = 0`); `associate` is fed **every** exposure of the fixture as a
`FifoCandidate` plus the `prior` list grown with each previous decision (`outcome: photo` unless stated); `initiated: App` unless stated. Behaviour
rows: `a = TestApp::with_camera()`; `s`, `t` sessions from `a.items().create_local(..)` (title = SKU = the name); `a.sessions().activate(s)`;
`L = a.ledger()` (the `LedgerService`; **re-fetched after a restart**); `expose(at)` = `L.exposure_observed(ExposureObserved { capture_id: Some(uuid),
at, initiated: App, provisional: false })` after `a.clock.advance(at − now)`; `deliver(name, bytes, via)` writes `bytes` to `root/.incoming/.<name>.part`
(or `root/.incoming/sdk-x/<name>` for `Cable`) and calls `L.delivery(Delivery { camera_filename: name, written_path, sha256: sha256(bytes), size, via,
completed_at: a.clock.now_ms() })`; `row(name)` = the `camera_deliveries` row; `photos(s)` = `store.photos_in_session(s)`; `H = a.app.health`; after a
rename, `a.watcher_event(final_path)` + `a.tick(1_000)` runs MC-2 (the real arrival path); "until" = the copied `settle`.

| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | `file_key` (unit) | `"DSC01234.ARW"`, `"dsc01234.arw"`, `"DSC01234__dup2.ARW"`, `"DSC01234_1.ARW"`, `"sub/DSC01234__dup1.ARW"` | `"DSC01234.ARW"`, `"DSC01234.ARW"`, `"DSC01234.ARW"`, **`"DSC01234_1.ARW"`** (core never strips `_N`), `"DSC01234.ARW"` | R-I26 ("Core strips only the app's own collision suffix `__dupN`"); H7 |
| C1(b) | `sequence_gaps` (unit) | stems `["DSC00001", "DSC00002", "DSC00004", "DSC00005_1", "IMG"]` | `[Gap { prefix: "DSC", after: 2, before: 4, missing: 1 }]` (`_1` stripped here only; `IMG` ignored) | MC-33 ("Gaps in camera file numbering are reported"); R-I29 |
| C2(a) | one exposure, one file (unit) | exposures `[e1 at 0, initiated: Camera]`; `associate("DSC00001.ARW", 6_000, prior [], candidates)`; then `is_complete(None, 1)` | `{ exposure_id: Some(e1), association: Fifo, session_id: e1's, ambiguous_with: [] }`; `is_complete(None, 1) == false` (provisional, never a lab-style `complete`) | MC-33 ("provisional"; "A shot is complete only when an expected count is known and met"; "Camera-initiated exposures are shots, not orphans") |
| C2(b) | expected count (unit) | `is_complete(Some(1), 1)`, `is_complete(Some(2), 1)`, `is_complete(Some(2), 2)`, `is_complete(None, 5)` | `true`, `false`, `true`, `false` | MC-33 ("A shot is complete only when an expected count is known and met") |
| C2(c) | RAW + JPEG by stem (unit) | exposures e1 (0), e2 (2_000); files `DSC00001.JPG` @3 000, `DSC00001.ARW` @8 000, `DSC00002.JPG` @9 000, `DSC00002.ARW` @14 000 (distinct hashes; every call fed both exposures and the prior decisions) | `[e1 Fifo, e1 Stem, e2 Fifo, e2 Stem]` (the stem matches are 5 s apart, inside R-I26 rule (2)'s 300 s) | R-I26 rule (2) as clarified (changelog 36); MC-33 ("RAW and JPEG with the same stem are one shot") |
| C2(d) | burst with lagging uploads (unit) | five exposures 1.5 s apart (0 … 6_000); files `DSC00010..14.ARW` at 8, 14, 20, 26, 32 s; every call fed all five exposures plus the priors | file `i` → exposure `i` (each exposure leaves the candidate set once it has one non-duplicate delivery) | R-I26 rule (3) FIFO; lab F-2 |
| C2(e) | bracketing with a known count (unit) | e1 (0, `expected_files: Some(3)`), e2 (5_000, `None`); files `DSC00100..103.ARW` at 10–13 s; every call fed both exposures plus the priors | 100, 101, 102 → e1 (`Fifo`; e1 stays a candidate while its non-duplicate priors `< 3`); 103 → e2 | MC-33 ("one exposure may produce several files"); R-I26 rule (3) |
| C2(f) | file before any exposure (unit) | exposures `[e1 at 5_000]`; file at 1_000 | `{ exposure_id: None, association: None, .. }` (`exposed_at ≤ completed_at` fails) | R-I26 rule (4); MC-33 |
| C2(g) | the 300 s window, bounds pinned (unit) | e1 at 0 (never yields a file), e2 at 400_000; file at 405_000 → ; with only e1: file at 200_000 → ; file at **300_000** → ; file at **300_001** → ; file at **0** → | → e2 (e1 is outside); → e1; → e1 (`completed_at − exposed_at ≤ 300 s` is inclusive); → `None`; → e1 (`exposed_at ≤ completed_at` is inclusive) | R-I26 ("FIFO in a window … The window keeps one lost file … from capturing every later file forever") |
| C2(h) | candidate set decides the session (unit) | e1 (0, session s), e2 (10_000, session s); file at 20_000 → ; with e2 in session t → ; with e2 in Unassigned (`session_id: None`) → | → `{ exposure_id: Some(e1), session_id: Some(s), ambiguous_with: [] }`; → `{ exposure_id: Some(e1), session_id: Some(s), ambiguous_with: [Some(t)] }`; → `ambiguous_with: [None]` | R-I26 ("The whole FIFO candidate set … decides the session … If they differ, the pick decides, and the health screen lists the photo") |
| C3(a) | same file by cable then FTP is a duplicate | `expose(0)`; `deliver("DSC04622.ARW", data, Cable)` @2 000; `deliver("DSC04622.ARW", data, Wireless)` @4 000 | first → `outcome: photo`, `final_path = root/<key(s)>/DSC04622.ARW`, `association: fifo`; second → `outcome: duplicate`, `association: filename`, `final_path = root/.duplicates/DSC04622.ARW`, both files exist with `data`; `photos(s)` after the watcher has exactly one row | R-I27; M37, M38 ("whichever path delivered it first"; "An identical later delivery becomes none") |
| C3(b) | FTP first, cable second | as C3(a) with the `via`s swapped | the Wireless copy is the photo, the Cable copy the duplicate | R-I27; M37 ("by either path") |
| C3(c) | duplicate across exposures and time | `expose(0)`; `deliver("A.ARW", d1, Wireless)` @5 000; **no** exposure; `a.clock` + 2 h; `deliver("A.ARW", d1, Wireless)` | second → `duplicate` (`association: filename`); one photo | R-I27 ("whatever its exposure (or none) and however long ago"); R-I26 rule (1) as clarified (identical bytes still give `filename`) |
| C3(d) | same name, other bytes | `deliver("B.ARW", d1, Wireless)`; `deliver("B.ARW", d2, Wireless)` (d2 ≠ d1); `deliver("B.ARW", d3, Wireless)` | rows: `photo` → `B.ARW`; `renamed_photo` → `B__dup1.ARW`; `renamed_photo` → `B__dup2.ARW`; all three files exist with their bytes; three photos after the watcher | R-I27 ("Same key with a different hash → a new photo, named with the lowest free `__dupN`"); M38 ("a same-name file with other bytes becomes its own row") |
| C3(e) | duplicates do not complete a count; RAW and JPEG distinct | `expose(0)` → `e`; `a.set_expected_files(e, 2)`; `deliver("DSC00010.ARW", d)` @1 000; `deliver("DSC00010.ARW", d)` @2 000 (dup); `deliver("DSC00010.JPG", j)` @3 000 | after the ARW: `e` is `expecting`, `L.complete_for(e) == false`; after the dup: still `expecting`, `false`; after the JPEG (`association: stem`): `e` is `received`, `complete_for(e) == true`; `.duplicates/` holds exactly one file | MC-33 ("A shot is complete only when an expected count is known and met"); R-I27 |
| C3(f) | an existing file at the destination is never replaced | the test writes `root/<key(s)>/C.ARW` with bytes `x` by hand; `deliver("C.ARW", x, Wireless)`; then `deliver("C.ARW", y, Wireless)` | first → `duplicate` → `.duplicates/C.ARW`; second → `renamed_photo` → `C__dup1.ARW`; `root/<key>/C.ARW` still reads `x` | R-I27 ("The rename refuses to replace an existing file … On `EEXIST` the existing file's SHA-256 is compared"); M37 ("never overwritten") |
| C3(g) | a reused camera name after the counter wraps | `activate(s)`; `expose(0)`; `deliver("DSC01234.ARW", d1, Wireless)` @5 000; `a.clock` + 2 h; `activate(t)`; `expose(now)` → e2; `deliver("DSC01234.ARW", d2, Wireless)` @+3 000 (d2 ≠ d1) | second → `association: fifo`, `exposure_id: e2`, `session_id: t`, `outcome: renamed_photo`, `final_path = root/<key(t)>/DSC01234__dup1.ARW`; e2 is `received`; after the watcher `photos(t).len() == 1` and `photos(s).len() == 1` (not two in `s`) | R-I26 clarification (changelog 36: "a name alone never ties a new file to an old exposure; a reused name with other bytes goes through rule (3)"); R-I27; M38; 10E §What the photographer gets ("the session that was open when the shutter fired") |
| C4(a) | target folder of the matched exposure's session | `activate(s)`; `expose(0)`; `activate(t)`; `deliver("D.ARW", d, Wireless)` @3 000 | `final_path = root/<key(s)>/D.ARW` (not `t`); after the watcher `photos(s)` has the row, `photos(t)` none | MC-33 ("of the session active at the matched exposure"); M38 |
| C4(b) | no exposure → session active at completion | `activate(t)`; `deliver("E.ARW", d, Wireless)` with no exposure | `final_path = root/<key(t)>/E.ARW`, `association: none`, row `session_id = t` | MC-33 ("or, if no exposure matches, of the session active when the file completed") |
| C4(c) | no session → `_Unassigned` | `a.sessions().deactivate()`; `deliver("F.ARW", …)` | `final_path = root/_Unassigned/F.ARW`; the photo lands in session 1 | R-I28 ("`_Unassigned` when the session is null or no longer exists") |
| C4(d) | folder key read at rename time; a SKU edit before the rename | `activate(s)` (item SKU `K1`, no photos); `expose(0)`; `a.items().update_identity(item, "K2", ..)` (key follows the SKU: folder `K2`); `deliver("G.ARW", d, Wireless)` | `final_path = root/K2/G.ARW`; the folder was created; after the watcher the photo is in `s` | R-I28 ("The folder key is read at the moment of the rename, and the folder is created if missing") |
| C4(e) | a committed delivery freezes the key (no photo row) | `activate(s)` (SKU `K1`); `a.items().update_identity(item, "K2", ..)`; `deliver("G2.ARW", d, Wireless)` → `done`; **no watcher event**; assert `photos(s)` is empty; `a.items().update_identity(item, "K3", ..)` | `item.folder_key` stays `K2` (frozen by the delivery alone); `store.item_folder_key_frozen(item) == true`; with no delivery and no photo the same edit moves the key to `K3` | 05B MC-22 (amended); R-I28 ("A committed delivery freezes it like a first photo row"); M29 |
| C4(f) | MC-2 rule 0 is consulted before the folder rules | `deliver("H.ARW", d, Wireless)` for session `s` (folder `K2`); before the watcher runs the test deletes `s`'s row (`DELETE FROM sessions WHERE id = s`, the `mark_published` SQL pattern — "no longer exists"; foreign keys are off, so it succeeds) and `activate(t)`; then `a.watcher_event(final_path)` + `a.tick(1_000)` | the photo row is in session **1** (rule 0: the delivery's session is gone → Unassigned), not in `t` (what rule 3 would give once rule 2 finds no owner of `K2`) | 05A MC-2 rule 0 ("a file whose canonical path is a delivery's `final_path` is attributed to that delivery's session (null or deleted → Unassigned)"); R-I28 |
| C4(g) | `captured_at` from the exposure | `expose(1_000)`; `deliver(.., Wireless)` @5 000; watcher | `photo.captured_at == 1_000`; for a delivery with no exposure (`association: none`) completed at 7_000 → `captured_at == 7_000`; neither equals the file's mtime | 05A MC-4 (exception); R-I28; M11 |
| C5(a) | crash-ordered persistence | `a.ledger_fs_fail_next_rename()` (the runtime's `delivery_fs` slot); `expose(0)`; `deliver("I0.ARW", d, Wireless)` | `L.delivery` returned `Err`; the row exists with `state = pending`, its `final_path` and `written_path`; the `.part` still exists; the exposure is already `received`; after `a.kill_and_restart()` (a fresh runtime over the real `CameraFiles`; bind → `recover_pending`) the row is `done` and the file is at the stored `final_path` | R-I25 ("A delivery row is written `pending` … **before** the rename and set `done` after it. At startup, a `pending` row whose staging file still exists is renamed again") |
| C5(b) | pending with the final file present | a `pending` row whose `written_path` is gone and whose `final_path` exists (the test moves the file by hand to simulate a crash after the rename); restart | the row is `done`; the final file's bytes are unchanged; the `camera_deliveries` row count is unchanged; `.duplicates/` is empty; no `__dup` file exists under the capture root | R-I25 ("one whose final file exists is set `done`") |
| C5(c) | a late file after a restart finds its exposure | `expose(0)`; `a.kill_and_restart()`; `L = a.ledger()`; `deliver("I.ARW", d, Wireless)` @60 000 | `association: fifo` to that exposure; the exposure row is `received` | MC-33 ("The ledger persists across restarts so a late file still finds its exposure") |
| C5(d) | a re-sent staging file takes over its `pending` row | `a.ledger_fs_fail_next_rename()`; `expose(0)`; `deliver("K.ARW", d, Wireless)` @2 000 → `Err`; then `deliver("K.ARW", d, Wireless)` @4 000 with the **same** `written_path` (the camera's re-send after `426` rewrites the same `.part`) | second → `Ok`, `outcome: photo`; exactly **one** `camera_deliveries` row for `K.ARW`, `state: done`, `completed_at: 4_000`; the file is at `root/<key(s)>/K.ARW`; `.duplicates/` is empty; after the watcher `photos(s).len() == 1` | R-I25 (pending before the rename); R-I27 ("One ledger worker makes every decision"); M38 (one photo, never none) |
| C6(a) | `pending_count` bounds pinned (unit) | exposures `expecting` at ages 59 s, **60.000 s**, 60.001 s, 29 min, **30 min 0 s**, 30 min 1 s; one `received` at 5 min | `pending_count == 2` (60.001 s and 29 min; both bounds exclusive) | R-I29 (`pending_count` = `expecting`, older than 60 s, newer than 30 min) |
| C6(b) | raise iff count > 0 and no transfer | `expose(0)`; `a.clock` → 61 s; `L.watchdog(now, transfer_active: true)` then `(now, false)` | first → `raised: false`, no health item; second → `raised: true`, `first_raise: true`, `H.has("camera_files_not_received")` with detail `1 photos taken but not received for 60 s`; a third call → `first_raise: false` | R-I29 ("shows iff `pending_count > 0` and no upload or download is in progress") |
| C6(c) | clear and dismiss [finalize_marks_missing_not_complete] | after C6(b): `deliver` the file → `watchdog` → cleared; or `L.dismiss()` → the exposure is `dismissed`, item cleared, `pending_count == 0` | as stated; a `dismissed` exposure is never a FIFO candidate for a later file (`association: none`) | R-I29 ("It clears when the count reaches 0 or the photographer dismisses it"); MC-33 (`missing_at_deadline` is an observation; nothing writes it in P44) |
| C6(d) | gaps on the health item | `deliver("DSC00001.ARW", d1, Wireless)` @0 and `deliver("DSC00004.ARW", d2, Wireless)` @1 000 (no exposures: both `association: none`); `expose(2_000)`; `a.clock` → 63 000; `L.watchdog(now, false)` | `pending_count == 1`, `gaps == 1` (one `Gap { prefix: "DSC", after: 1, before: 4, missing: 2 }` entry), detail `1 photos taken but not received for 60 s · 1 gaps in the camera's numbering` | R-I29 ("Gaps … are logged and counted on the same health item; they never block anything") |
| C7(a) | uncertain after a session loss [uncertain_after_session_loss] | `expose(0)`; `L.session_uncertain()`; `deliver("J.ARW", d, Wireless)` @9 000 | the exposure is `uncertain`; the delivery has `association: none` (an uncertain exposure is not a candidate) and lands in the active session; the exposure stays `uncertain` | MC-33 ("`uncertain` after a session loss during a protected operation") |
| C7(b) | a split candidate set lists the photo on the health screen | `activate(s)`; `expose(0)` (e1); `activate(t)`; `expose(1_000)` (e2); `activate(s)`; `expose(2_000)` (e3); `deliver("M.ARW", d, Wireless)` @5 000; then `deliver("N.ARW", d2, Wireless)` @6 000 | M → `exposure_id: e1`, `association: fifo`, `session_id: s`, `ambiguous_with: [Some(t)]`, `final_path = root/<key(s)>/M.ARW`; `H.has("camera_photo_session_uncertain")` with detail `Photo M.ARW may belong to t`; N → `exposure_id: e2`, `session_id: t`, `ambiguous_with: [Some(s)]`; the detail now reads `Photo N.ARW may belong to s` (latest wins, one item per code) | R-I26 ("If they differ, the pick decides, and the health screen lists the photo ("may belong to <other session>")"); master plan §6.6 (`camera_photo_session_uncertain`) |
| C8(a) | the receiver's sink is the ledger — one upload, one photo | `expose(0)`; `a.ftp().stor("DSC00001.ARW", data)` (the real receiver); until `row("DSC00001.ARW").state == done`; `a.watcher_event(final_path)`; `a.tick(1_000)` | `226`; `row("DSC00001.ARW") == { via: wireless, association: fifo, outcome: photo, state: done }`; `root/<key(s)>/DSC00001.ARW` reads `data`; `.incoming/` holds no `.part`; `photos(s).len() == 1` with `raw_path == final_path`; `c.cmd("SIZE DSC00001.ARW")` → `213 <len>` (the ledger is the receiver's `CompletedNames`) | M37, M38; MC-32 §2; R-I24 (listings from the ledger) |
| C8(b) | the arrival path ignores the camera folders | after C3(a) (`root/.duplicates/DSC04622.ARW` exists): the test writes `root/.incoming/X.ARW` with RAW bytes; `a.watcher_event(root/.duplicates/DSC04622.ARW)` + `a.tick(1_000)`; `a.watcher_event(root/.incoming/X.ARW)` + `a.tick(1_000)`; `a.scan()` | `photos(s).len()` stays 1 and `COUNT(*) FROM photos` stays 1 (neither the duplicate nor the staged file becomes a photo) | 05A MC-2 (candidate filter: dot-prefixed directories `.incoming`, `.duplicates`); M9 |

## Delegated choices
Whether `associate` takes slices or iterators; the exact `tracing` fields of `camera.sequence_gap`; index names in 006; the names of the `Store`
row DTOs' fields beyond those the contract fixes; whether `LedgerShared.raised` is a `Mutex<bool>` or an `AtomicBool`; the `FailNextRename`
wrapper's module in testkit. (Settled by the orchestrator after projection r0, see the contract: `transfer_in_progress` no-op, `missing_at_deadline`
reserved, status written with the `pending` insert, recovery to the stored `final_path`, non-`EEXIST` errors → `Err`, names deduplicated, the
`.duplicates/` collision suffix `__dupN`, `complete_for` a service method over `is_complete`.)

## Manual try-out
(Hardware-free.) 1. Nothing changes on screen yet. 2. Health has no new rows unless a photo is pending (impossible without the camera). 3. The database
gains two empty tables (`sqlite3 … ".tables"` shows `camera_exposures camera_deliveries`).

## Notes
- **Hard rules (owner):** never commit a Sony SDK file, the lab's `sdk_shim` headers, a `.env`, a credential or an SSH fingerprint; no Python
  (`ledger.py` is read, not run); never change the camera's network/FTP settings, delete a camera file (the ledger **moves** staging files it owns
  and never deletes; `.duplicates/` is never emptied) or kill any process; live view out of scope; the suite never needs the camera or Capture One.
  Implemented by Claude agents (owner 2026-11-06, `PSM_IMPLEMENTER=claude`: Opus implements, Sonnet fixes).
- **Planner readings (reversible):** (1) `complete_for` is informational: nothing in the product waits for "complete" (there is no `expected_files`
  source in this chapter). (2) A delivery whose rename fails leaves the row `pending` and returns `Err` (the receiver answers `426`, the camera
  re-sends, the re-send takes the row over); recovery runs at every bind (settings save too), under the process-wide worker lock. (3)
  `camera_photo_session_uncertain` names the other session by its item title, else SKU, `Unassigned` for `None`. (4) The freeze is derived from the
  delivery rows, no new column on `items`. (5) Rules (1)/(2) carry the prior's `exposure_id` and `session_id` even when the prior had none, so the two
  halves of an exposure-less RAW + JPEG shot stay together. (6) `AssociationDecision.session_id` is meaningful only when `exposure_id` is `Some`; for
  `association: none` the service substitutes `active_session_id()`.
- **Hazards:** `arrival_insert` canonicalises `raw_path` (`store.rs:1376`) — compare `final_path` **canonicalised** in `delivery_for_path`. The worker
  lock is **one per process** (`LedgerShared`, owned by the runtime): a lock per `LedgerService` would let an in-flight delivery under the old service
  and a new one under the rebuilt service both decide `photo` during a settings save, and recovery on the new service could rename a `.part` the old
  worker has just inserted as `pending`. `renamex_np` works only on the same volume and APFS; `.incoming/` is under the capture root, so it is; any
  other error is `Err`, never a fallback. `std::fs::rename` **replaces** on macOS — forbidden for camera files (R-I27). The watcher may fire for the
  `.part` rename target before `done` is written: rule 0 must read the `pending` row too (the `final_path` is written before the rename).
  `ItemsService::update_identity` is the only writer of `folder_key` (`items.rs:254–275`). `is_candidate` rejects `.incoming/.X.ARW.part` by
  extension and `.incoming/sdk-abc/X.ARW` by depth as well as by the dot rule, which is why C8(b) goes through the watcher with a RAW name at depth 2.
- **Projection** was on and ran (r0, 2026-11-06): the ledger is the inventory's #2–#5 silent-failure mechanisms. Its 24 ledger rows are folded into
  this revision (Review log); the projectionist verified C4(f) constructible as written and both external shapes (`renamex_np`/`RENAME_EXCL`, SQLite
  foreign keys off) producible.
- Registry additions made by the planner in master plan §6.4, §6.5, §6.6, §6.11 (aligned to this plan's names after projection r0). Intention-book
  change: none by the planner; the orchestrator's R-I26 clarification is changelog entry 36.
- Estimated size: ~24 paths, ~2 300 lines (≈ 1 050 mechanism + service + store + runtime, ≈ 1 250 tests).

## Review log


- **2026-11-06, carried from P42 projection r0** (planner): `bind` is the last statement of `App::build` (L3), so `recover_pending` and the ledger
  rebuild run only for a build that succeeded; timed reads in C8(a) ("until the row is `done`") use P42's `settle(a, pred)` fixture (L7).
- **2026-11-06, orchestrator amendments after projection r0 (L1–L24 folded; card 1 answered from the book as the R-I26 clarification, changelog 36)**
  (planner): L1 rules (1)/(2) bounded by SHA-256 and the 300 s window, new row C3(g), C3(c) unchanged, card closed · L2 the `pending`-row take-over
  and decisions against `done` rows only, new row C5(d) · L3 one process-wide worker lock and raised flag (`LedgerShared`) shared by every
  `LedgerService` the runtime builds, recovery under it, §6.11 (b) and Task 5 amended · L4 the `received` transition stated · L5 `FifoCandidate`/
  `PriorDelivery` fields and the filter split (`associate` filters, the service passes everything) · L6 `is_complete` pure, `set_expected_files`
  fixture write for C3(e), C2(b)/C2(e) rewritten · L7 `written_path TEXT NOT NULL` in 006 · L8 `CompletedNames` moved to `psm_core::ports::camera`,
  infra re-exports · L9 `StoreTx::item_folder_key_frozen` predicate incl. `pending`, `freeze_folder_key` struck from §6.4 · L10 C4(e) fixture without a
  photo row · L11 the `delivery_fs` seam on the runtime · L12 p43 widening and `deliveries()` re-pointed · L13 C8(b) as a behaviour row · L14 new row
  C7(b), `ambiguous_with: Vec<Option<i64>>`, latest wins · L15 gaps over the 30 min window, `g` = `Gap` entries, C6(d) fixture reordered · L16 bounds
  pinned in C2(g)/C6(a), a `Camera`-initiated fixture in C2(a) · L17 service clauses dropped from pure rows · L18 to P46 Task 2 / C8(c), listed under
  "Not in this phase" · L19–L22 decided in writing (contract) · L23 §6.11/§6.4 aligned to the plan's names · L24 C5(b) exact. Reality: anchors at HEAD
  `2e78bab`, line citations corrected, fixture-helper notes added, 24 paths derived, C2(b)'s trace re-pointed. Rows 33 → 36 (C3 +1, C5 +1, C7 +1).
- **2026-11-06, implement r1** (opus, `PSM_IMPLEMENTER=claude`): built the 24 declared paths. `capture_ledger.rs` (file key, stem key,
  `associate` with every filter inside, `is_complete`, `duplicate_decision`, `dup_name`, `target_folder`, `pending_count`, `sequence_gaps` — 11
  `mc33_*` unit tests for C1, C2, C6(a)); `psm_infra::camera_files` (`renamex_np(RENAME_EXCL)`, `CameraFiles`, one inline test); `LedgerService`
  (one worker under the runtime's `LedgerShared`; decide → pending row + exposure status in one transaction that also reads the folder key →
  `create_dir_all` → exclusive rename with the `EEXIST` rules → `done`; take-over of a `pending` row by `written_path`; `recover_pending` at
  every bind; watchdog, `dismiss`, `complete_for`, `completed_names_since`); migration 006; MC-2 rule 0 + `captured_at` in `ArrivalService::accept`;
  `item_folder_key_frozen` in `update_identity`; runtime `ledger_shared`/`delivery_fs`/`set_delivery_fs`/`ledger()`, `NoopSink` removed;
  `p44_capture_ledger.rs` (25 behaviour rows C3–C8). Suite 671/0, clippy, fmt, typecheck, UI 369/369.
  **Judgment calls:** (1) `update_pending_delivery(id, &PendingUpdate)` — the registry's nine fields grouped in one struct (clippy's argument
  limit). (2) One `Store` method beyond the registry list: `deliveries_for_exposure(exposure_id)` — the `received` transition and `complete_for`
  count an exposure's non-duplicate deliveries across all stems (a bracket of `DSC00100..102` shares no key or stem). (3) The matched exposure's
  status = `received` iff its non-duplicate `pending`/`done` deliveries reach `expected_files.unwrap_or(1)`, else `expecting`; only
  `expecting`/`received` exposures are moved (an `uncertain` or `dismissed` one is never touched). When a rename's `EEXIST` turns a delivery into a
  duplicate, or a take-over moves a row's FIFO pick, the old exposure is re-evaluated the same way. (4) The take-over restores the exposure its own
  FIFO pick had made `received` as a candidate for the re-made decision. (5) The folder key is read inside the transaction that inserts the
  `pending` row (the "moment of the rename" is the commit of the row whose final path the rename uses; the freeze then holds). (6) Rule 0 applies to
  the capture path only (hook/watcher/scan), not to imports. (7) `WatchdogState.gaps` is the `Vec<Gap>` (`g` = its length); while a transfer is
  active the item is cleared and the raised flag reset, so `first_raise` fires again when the notice reappears. (8) `dismiss()` dismisses only the
  exposures the count covers (60 s < age < 30 min), not a shot taken seconds ago. (9) `session_uncertain` and `dismiss` take the worker lock. (10)
  Health detail names the first `ambiguous_with` session. (11) Before the first bind the runtime's sink refuses deliveries (`Err` → the receiver
  answers 426) instead of P43's accept-and-drop `NoopSink`. (12) Final paths are stored NFC-normalised from the canonical capture root, so rule 0
  compares like with like.
  **Deviations:** (a) p43: besides C1(a), C2(g) and C4(a)–(c), five more `TestApp` rows read the `.part` after a successful upload — C1(b), C2(b),
  C2(d), C3(a) and C6(b) — and now read `root/_Unassigned/<name>` (same category the Files section names; assertions otherwise unchanged); C4(a)–(c)
  use a local `listing_receiver` (a standalone receiver with its own `ManualClock`, because C4(b)/(c) move the clock). (b) Migration 006 uses
  `IF NOT EXISTS`: `p27 c4a_migration_removes_old_jobs_and_health_codes` rewinds `user_version` to 2 and re-runs every later migration.
  (c) No red baseline was captured before the first production edit (flow §4 has no baseline step; stated, not reconstructed).
  **Observations:** no hardware probe — in P44 only the FTP receiver feeds the ledger, and it starts only with an installed support package; the
  suite covers the path through the real receiver (C8(a)).
- **2026-11-06, review r1 — APPROVED_WITH_NOTES** (opus, pipeline-reviewer). Gate: tracker `IMPLEMENTED`, checkpoint `046d3c5` in `git log`.
  Perimeter: `git show --stat 046d3c5` = the 26 declared paths (plan's 24 + tracker row + this log); the p43 edits beyond Task 6's named rows
  (C1(b), C2(b), C2(d), C3(a), C6(b)) are the same `.part` → `_Unassigned/<name>` category with assertions otherwise unchanged; C4(a)–(c) on a
  standalone `listing_receiver`. Gates (reviewer run, code tree `046d3c5`): `cargo test --workspace` 671/0 (p44 25/25), clippy `-D warnings`,
  `fmt --check`, `npm run typecheck`, `npm test` 369/369 — all exit 0. **Verified correct by reading:** `associate` applies every filter itself
  (rule (1) same key + SHA-256 → `filename`; rule (2) same stem within ±300 s → `stem`; FIFO with both bounds inclusive and "still expecting"
  counted from non-duplicate priors; session = pick's, `ambiguous_with` distinct in candidate order; rules (1)/(2) carry the prior's exposure
  and session); the service passes every `expecting` exposure and every `done` prior of the key or stem; duplicate decision against `done` rows
  only, lowest free `__dupN` from the priors' final names; one process-wide `LedgerShared.worker` held from decision to `done` (also by
  recovery, `session_uncertain`, `dismiss`); `pending` row + exposure status in one transaction that also reads the folder key, before the
  rename; `renamex_np(RENAME_EXCL)`, `EEXIST` → `Exists`, every other errno → `Err` with no fallback (no `std::fs::rename` on camera files);
  `EEXIST` loop rewrites the row's final path before each retry (same hash → `.duplicates/`, else next `__dupN`, `.duplicates/` likewise);
  take-over by `written_path` updates the row in place; recovery to the stored `final_path`, final-present → `done`, neither → logged;
  `item_folder_key_frozen` = photo row OR `photo`/`renamed_photo` delivery in `pending`/`done`, used in `update_identity`'s transaction; rule 0
  reads pending rows, deleted session → Unassigned, `captured_at = COALESCE(exposed_at, completed_at)`; `completed_names_since` deduplicated by
  name; watchdog bounds exclusive, raise iff count > 0 and no transfer, detail strings exact; migration 006 matches R-I25 plus `written_path`,
  `created_at`. **Tests:** every row's test observes the row's outcome at its boundary and fails on a wrong one (C4(f) discriminates rule 0
  from rule 3; C4(e) has no photo row; C5(d) pins one row; C2 rows feed all exposures plus the growing priors). C7(b) observes `ambiguous_with`
  through the health detail only (not stored); C2(h) pins the struct. **Assumptions 1–12:** none contradicts 10E MC-33 (4: reading the key in
  the pending-row transaction is equivalent to "at the rename" because the freeze holds from that commit). H1–H8 hold (H2 as amended by 10E:
  the ledger moves only staging files it owns; H7: no `_N` stripping in core, `renamex_np` in infra). **Notes → backlog (5):** recovery's
  `EEXIST`→duplicate does not re-evaluate the exposure (→ P46); `first_raise` re-fires after each transfer (→ P46); `Unbound` also refuses
  exposures before the first bind (→ P46); §6.11 registry drift (`deliveries_for_exposure`, `PendingUpdate`) for the coordinator; C7(b)
  `ambiguous_with` observed via health only. **Lessons for the plans:** Task 6's p43 list named 2 of the 7 `.part`-reading rows: enumerate by
  grep, not by memory; a status-recomputing path (recovery) that shares a helper with the worker should state which side effects it repeats.
  Probe declaration: no production file touched; no probe files; no state side effects.
