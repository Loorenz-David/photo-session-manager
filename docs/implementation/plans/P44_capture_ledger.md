---
phase: P44_capture_ledger
title: Capture ledger — the lab ledger ported as a core mechanism with the inventory's fixes (file key, association order with a 300 s FIFO window, global duplicates, no-replace rename, target folder at rename time, folder-key freeze, MC-2 rule 0 and captured_at), persisted in two tables with crash-ordered writes, the watchdog counts; the receiver wired to it
difficulty: high
projection: yes
depends_on: [P43_ftp_receiver]
---
# P44 — Capture ledger (MC-33), persistence, target folder, arrival seam

## Goal
10E MC-33 with the inventory additions R-I25…R-I29, the amended 05A MC-2 (rule 0) and MC-4 (`captured_at`), and 05B MC-22 (freeze at a committed
delivery); invariants M37 (first copy under a candidate name, identical copies in `.duplicates/`, nothing overwritten) and M38 (one `photos` row per
distinct camera file, in the session the association rule gives). The pure mechanism `capture_ledger.rs` (file key, association order, duplicate
decision, FIFO window and the candidate-set session rule, gaps, pending count) carries the lab's 15 ledger tests reinterpreted for the 10E model;
the `LedgerService` is the **one worker** that decides → writes the `pending` delivery row with its final path → renames with `renamex_np(RENAME_EXCL)`
(never `std::fs::rename`) → marks `done`, puts identical later copies in `.duplicates/`, renames same-name-different-bytes files `__dupN`, reads the
item's folder key **at the moment of the rename**, freezes it, and recovers `pending` rows at start. `ArrivalService` learns rule 0 (a file whose
canonical path is a delivery's `final_path` joins that delivery's session with `captured_at = exposed_at`, else `completed_at`). The receiver's
sink becomes the ledger (replacing P43's `NoopSink`), so an upload through the real receiver ends as exactly one photo.
**Not in this phase:** the sidecar and its events (P45/P46 — exposures are recorded here through `CaptureSink::exposure_observed` called by tests),
the adapter's Sony filename normalisation (`_N`, P46), the watchdog's `ftp-status` request and reconnect button (P46), the health UI (P51).

## Read first
Book: `10E_direct_camera_adapter.md` MC-33 (whole, incl. **R-I25** persistence, **R-I26** file key and association order, **R-I27** duplicates and the
worker, **R-I28** target folder and attribution, **R-I29** watchdog) and §Invariants M37, M38 · `05A_mechanism_contracts.md` MC-2 (capture root,
candidate filter, **rule 0**, rules 1–3, row creation) and MC-4 (`captured_at` exception) · `05B_sku_generation_and_change.md` (the stored folder key
follows the SKU until frozen; **freeze at a committed delivery**) · `14_measurement_ledger.md` M9, M29, M37, M38 · process `flow.md` §4 (core
mechanisms: "RAW hash, position math, gallery reconciliation diff" — the ledger joins that list), §6, §9 · master plan §3, §6.3 (`.incoming`,
`.duplicates`), §6.4, §6.5 (**migration 006**), §6.6 (ledger enums; `camera_photo_session_uncertain`, `camera_files_not_received`), **§6.11**
(Constants, Domain, Ports `CaptureSink`/`DeliveryFs`, Mechanisms `capture_ledger.rs`, Services `LedgerService`, Runtime `bind` (b)), §7.5.
Code at `41bef14`:
- `crates/core/src/services/arrival.rs:243–284` (`accept`: attribution `:249–258` and `captured_at` from mtime `:259–264` — **rule 0 goes before both**),
  `:57–68` (`capture_root`), `:91–153` (`scan`), `crates/core/src/mechanisms/arrival.rs:297–345` (`is_candidate`, `attribute`).
- `crates/core/src/services/items.rs:264–275` (`update_identity`: `item_has_photos` decides whether `folder_key` follows the SKU — **the freeze
  joins it**), `crates/infra/src/sqlite/store.rs:597–599` (`item_has_photos`), `:1375–1429` (`arrival_insert`), `:1430–1440` (`session_folders`),
  `:1159–1170` (`active_session_id`); `crates/core/src/ports/store.rs:33–262` (`StoreTx`, `Store`); `crates/infra/src/sqlite/migrations/002_domain.sql:1–30`.
- `crates/core/src/mechanisms/hash.rs:5` (`sha256_file`), `crates/core/src/mechanisms/canon.rs`, `crates/core/src/mechanisms/naming.rs`
  (`folder_key`), `crates/core/src/services/health.rs:625–654` (`set`/`set_ok`/`clear`/`has`).
- P42/P43 by their §6.11 names: `CameraRuntime::bind`, `CaptureSink`, `Delivery`, `FtpReceiver`, `CompletedNames`, `TestApp::{with_camera, ftp,
  deliveries}`; `crates/testkit/tests/p05_capture_arrival.rs:36–55` (`file`, `watch`, `hook` helpers — the real arrival path).
- Lab (read-only): `camlab/ledger.py` (`stem_of` `:52–58`, `seq_of` `:61–66`, `file_key` `:84–86`, `_file` association `:191–240`, `_refresh_duplicates`
  `:254–261`, `_update_status` `:270–282`, `finalize` `:285–293`, `sequence_gaps` `:295–307`), `tests/python/test_ledger.py` (the 15 tests; mapping below).
- `libc 0.2.191`: `renamex_np(from, to, flags)` and `RENAME_EXCL = 0x4` (`src/unix/bsd/apple/mod.rs:5010`, `:3557`): the rename fails with `EEXIST`
  instead of replacing.

## Files expected to change
Core (new): `crates/core/src/mechanisms/capture_ledger.rs`, `crates/core/src/services/camera_ledger.rs`. Core (changed): `crates/core/src/mechanisms/
mod.rs`, `crates/core/src/services/{mod.rs, arrival.rs, items.rs}`, `crates/core/src/ports/{camera.rs, store.rs}`, `crates/core/src/domain/camera.rs`
(ledger records: `Exposure`, `DeliveryRow`, `NewExposure`, `NewDelivery`).
Infra (new): `crates/infra/src/camera_files.rs`, `crates/infra/src/sqlite/migrations/006_camera_ledger.sql`. Infra (changed): `crates/infra/src/lib.rs`,
`crates/infra/src/sqlite/store.rs`, `crates/infra/Cargo.toml` (`libc`).
Integrations (changed): `crates/integrations/src/camera/{runtime.rs, mod.rs}` (`NoopSink` removed; the ledger bound per `App::build`), `crates/
integrations/src/lib.rs`.
Testkit (changed): `crates/testkit/src/test_app.rs` (`ledger()`, `exposure(session, at)`, `deliver(file)` helpers; `with_camera` no longer installs
`CollectingSink`), `crates/testkit/src/collecting_sink.rs` (kept for P43's standalone rows). Tests (new): `crates/testkit/tests/p44_capture_ledger.rs`.
Tests (changed): `crates/testkit/tests/p43_ftp_receiver.rs` (rows C1(a)–(c), C2(b), C2(d), C2(g) read the ledger's deliveries instead of the
`CollectingSink` for the `TestApp` fixture; the standalone rows are untouched), `crates/testkit/tests/p01_storage_identity.rs:91` region (the table
list gains the two tables).
Estimated: 4 new source paths, 12 changed, 1 new test, 2 adjusted tests — 19 paths.

## Core mechanisms (unit tests allowed)
`crates/core/src/mechanisms/capture_ledger.rs` — `file_key`, `stem_key`, `associate`, `duplicate_decision`, `dup_name`, `target_folder`,
`pending_count`, `sequence_gaps`: inline tests named `mc33_<slug>` (criteria C1 and C2 below, the ported lab tests). `psm_infra::camera_files::
rename_exclusive`: one inline test (`EEXIST` → `Exists`, otherwise `Moved`). Everything else is a behaviour row through `TestApp`.

## The ledger contract (master plan §6.11 Mechanisms + Services)
- **Exposure rows** are written by `CaptureSink::exposure_observed(e)`: `session_id = store.active_session_id()` read at that moment (`None` =
  Unassigned), `status = expecting`, `expected_files = None`, `initiated` from `e`. `session_uncertain()` sets every `expecting` row to `uncertain`.
- **Association order** for a delivery `(camera_filename, sha256, completed_at)`: `key = file_key(camera_filename)` (strips only `__dup<n>`, upper-cases);
  (1) a prior delivery with the same `key` → its exposure (`filename`); (2) a prior delivery with the same `stem_key` → its exposure (`stem`); (3) FIFO:
  candidates = `expecting` exposures with `exposed_at ≤ completed_at` and `completed_at − exposed_at ≤ FIFO_WINDOW_MS`, oldest first → the oldest
  (`fifo`, provisional); the delivery's **session** is the candidates' common `session_id` when they all agree, else the picked exposure's, and
  `ambiguous_with` lists the other sessions (→ health `camera_photo_session_uncertain`, detail `Photo <camera filename> may belong to <title>`);
  (4) none → `association: none`, session = `store.active_session_id()` at completion. `uncertain` and `dismissed` exposures are never candidates.
  A matched exposure becomes `received`.
- **Duplicate decision:** a prior delivery (any exposure, any age) with the same `key` **and** the same `sha256` → `Duplicate { of }` → final path
  `<capture root>/.duplicates/<name>`; same `key`, other hash → `RenamedPhoto { n }` with the lowest free `__dupN` under the target folder; else
  `Photo`. **One worker** (`Mutex` held from decision to `done`): decide → `insert_delivery(pending, final_path)` → `fs.create_dir_all(target)` →
  `fs.rename_exclusive(written_path, final_path)`; on `Exists`: SHA-256 of the existing file equal → the row becomes `duplicate` with final path under
  `.duplicates/` (renamed there, again exclusive; a collision there gets `__dupN` under `.duplicates/`), different → the next `__dupN`; then
  `set_delivery_done`. `std::fs::rename` is never called on a camera file.
- **Target folder** = `target_folder(capture_root, folder_key)` where `folder_key` = the stored `items.folder_key` of the session's item **read inside
  the rename step**, `_Unassigned` when the session is `None` or gone. A `done` delivery with a session **freezes** the item's folder key: `Store::
  item_folder_key_frozen(item_id)` = `item_has_photos || EXISTS(camera_deliveries done with session_id in the item's sessions)`, used by
  `ItemsService::update_identity` in place of `item_has_photos` (`items.rs:269`).
- **MC-2 rule 0** (`ArrivalService::accept`): `store.delivery_for_path(canonical)` → `Some((session_id, exposed_at_or_completed_at))` → that session
  (`None`/deleted → 1) and that `captured_at`; otherwise rules 1–3 and the mtime as today.
- **Recovery** (`recover_pending`, at every `bind`): a `pending` row whose `written_path` exists → run the rename step again; whose `final_path` exists →
  `done`; neither → left `pending` and logged.
- **Watchdog:** `pending_count(now)` = `expecting` rows with `WATCHDOG_MIN_AGE_MS < now − exposed_at < WATCHDOG_MAX_AGE_MS`; `watchdog(now, transfer_active:
  bool) -> WatchdogState { pending_count, gaps, raised, first_raise: bool }` sets `camera_files_not_received` iff `pending_count > 0 && !transfer_active`
  (detail `<n> photos taken but not received for 60 s` + ` · <g> gaps in the camera's numbering` when `g > 0`), clears it when the count is 0 or after
  `dismiss()` (those rows → `dismissed`); `first_raise` is true on the transition into raised (P46 sends `ftp-status` then). Gaps come from
  `sequence_gaps` over the stems of every delivery (`seq_of` rule: trailing `_<1–2 digits>` stripped **only** for gap reporting) and are logged
  `camera.sequence_gap`.
- `completed_names_since(ms)` → `(camera_filename, size, completed_at)` of `done` deliveries (the receiver's listings). `exposures(status)`,
  `deliveries()` for tests.

## Lab test mapping (15 → rows)
`stem_and_sequence_helpers` → C1(a),(b) · `single_capture_single_raw_is_provisional_not_complete` → C2(a) · `expected_count_allows_completion` →
C2(b) · `raw_plus_jpeg_grouped_by_stem` → C2(c) · `burst_with_lagging_uploads_assigned_in_order` → C2(d) · `bracketing_with_expected_files` → C2(e) ·
`file_before_any_capture_is_unassociated` → C2(f) · `failed_ack_never_receives_files` → not a ledger case any more (a refused capture creates no
exposure row; P46 C2) · `finalize_marks_missing_not_complete` → C6(c) · `uncertain_after_session_loss` → C7(a) · `sequence_gap_detection` → C6(d) ·
`sdk_filename_then_ftp_copy_matches_by_name` → C3(a) · `ledger_persists_json` → C5(a) (SQLite instead of JSON) · `same_file_by_sdk_and_ftp_is_a_duplicate`
→ C3(a) · `duplicates_do_not_complete_a_multi_file_capture_and_raw_jpeg_are_distinct` → C3(e).

## Ordered tasks
0. **Registry first** (§6.5, §6.11): migration `006_camera_ledger.sql`; `Store`/`StoreTx` methods `insert_exposure`, `set_exposure_status`,
   `set_exposures_status_where(from, to)`, `exposures(status: Option<ExposureStatus>)`, `insert_delivery`, `set_delivery_done`, `deliveries_by_key`,
   `deliveries_by_stem`, `pending_deliveries`, `delivery_for_path`, `completed_names_since`, `item_folder_key_frozen`; the domain records.
1. `mechanisms/capture_ledger.rs` + unit tests (C1, C2).
2. `camera_files.rs` (`rename_exclusive`, `DeliveryFs` impl `CameraFiles`).
3. `services/camera_ledger.rs` (`LedgerService: CaptureSink + CompletedNames`; the worker; recovery; watchdog).
4. `arrival.rs` rule 0 + `captured_at`; `items.rs` freeze.
5. Runtime: `bind` builds the `LedgerService` over the new store (same DB file) and installs it as the receiver's sink and names provider; `recover_pending`.
6. Testkit helpers; tests `p44_capture_ledger.rs`; the p43/p01 edits; suite, clippy, fmt, typecheck.

## Criteria (8 criteria, 33 rows; 0 `[rule]` rows)
Fixtures: unit rows use the mechanism functions directly (times in ms from `T0 = 0`). Behaviour rows: `a = TestApp::with_camera()`; `s`, `t` sessions
from `a.items().create_local(..)` (title = SKU = the name); `a.sessions().activate(s)`; `L = a.ledger()` (the `LedgerService`); `expose(at)` =
`L.exposure_observed(ExposureObserved { capture_id: Some(uuid), at, initiated: App, provisional: false })` after `a.clock` is set to `at`;
`deliver(name, bytes, via)` writes `bytes` to `root/.incoming/.<name>.part` (or `root/.incoming/sdk-x/<name>` for `Cable`) and calls
`L.delivery(Delivery { camera_filename: name, written_path, sha256: sha256(bytes), size, via, completed_at: a.clock.now_ms() })`; `row(name)` = the
`camera_deliveries` row; `photos(s)` = `store.photos_in_session(s)`; after a rename, `a.watcher_event(final_path)` + `a.tick(1_000)` runs MC-2 (the
real arrival path).

| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | `file_key` (unit) | `"DSC01234.ARW"`, `"dsc01234.arw"`, `"DSC01234__dup2.ARW"`, `"DSC01234_1.ARW"`, `"sub/DSC01234__dup1.ARW"` | `"DSC01234.ARW"`, `"DSC01234.ARW"`, `"DSC01234.ARW"`, **`"DSC01234_1.ARW"`** (core never strips `_N`), `"DSC01234.ARW"` | R-I26 ("Core strips only the app's own collision suffix `__dupN`"); H7 |
| C1(b) | `sequence_gaps` (unit) | stems `["DSC00001", "DSC00002", "DSC00004", "DSC00005_1", "IMG"]` | `[Gap { prefix: "DSC", after: 2, before: 4, missing: 1 }]` (`_1` stripped here only; `IMG` ignored) | MC-33 ("Gaps in camera file numbering are reported"); R-I29 |
| C2(a) | one exposure, one file (unit) | exposures `[e1 at 0 expecting]`; `associate("DSC00001.ARW", 6_000, prior [], candidates)` | `{ exposure_id: e1, association: Fifo, session_id: e1's, ambiguous_with: [] }`; the exposure's status after the service step is `received`, never a lab-style `complete` | MC-33 ("provisional"; "A shot is complete only when an expected count is known and met") |
| C2(b) | expected count (unit) | as C2(a) with `expected_files: Some(1)` | the service marks the exposure `received` and `complete_for(e1) == true`; with `expected_files: Some(2)` → `false` after one file | MC-33 ("elapsed time never completes a shot") |
| C2(c) | RAW + JPEG by stem (unit) | exposures e1 (0), e2 (2_000); files `DSC00001.JPG` @3 000, `DSC00001.ARW` @8 000, `DSC00002.JPG` @9 000, `DSC00002.ARW` @14 000 (each `associate` fed the prior decisions) | `[e1 Fifo, e1 Stem, e2 Fifo, e2 Stem]` | R-I26 rule (2); MC-33 ("RAW and JPEG with the same stem are one shot") |
| C2(d) | burst with lagging uploads (unit) | five exposures 1.5 s apart; files `DSC00010..14.ARW` at 8, 14, 20, 26, 32 s | file `i` → exposure `i` | R-I26 rule (3) FIFO; lab F-2 |
| C2(e) | bracketing with a known count (unit) | e1 (0, `expected_files: 3`), e2 (5_000); files `DSC00100..103.ARW` at 10–13 s | 100, 101, 102 → e1; 103 → e2 | MC-33 ("one exposure may produce several files") |
| C2(f) | file before any exposure (unit) | file at 1_000; exposure e1 at 5_000 | `{ exposure_id: None, association: None }`; e1 stays `expecting` | R-I26 rule (4); MC-33 |
| C2(g) | the 300 s window (unit) | e1 at 0 (never yields a file); e2 at 400_000; file at 405_000 | → e2 (`e1` is outside `completed_at − exposed_at ≤ 300 s`); a file at 200_000 with only e1 → e1 | R-I26 ("FIFO in a window … The window keeps one lost file … from capturing every later file forever") |
| C2(h) | candidate set decides the session (unit) | e1 (0, session s), e2 (10_000, session s); file at 20_000 | `session_id: s`, `ambiguous_with: []`; with e2 in session t → `exposure_id: e1`, `session_id: s`, `ambiguous_with: [t]` | R-I26 ("The whole FIFO candidate set … decides the session … If they differ, the pick decides, and the health screen lists the photo") |
| C3(a) | same file by cable then FTP is a duplicate | `expose(0)`; `deliver("DSC04622.ARW", data, Cable)` @2 000; `deliver("DSC04622.ARW", data, Wireless)` @4 000 | first → `outcome: photo`, `final_path = root/<key(s)>/DSC04622.ARW`, `association: fifo`; second → `outcome: duplicate`, `association: filename`, `final_path = root/.duplicates/DSC04622.ARW`, both files exist with `data`; `photos(s)` after the watcher has exactly one row | R-I27; M37, M38 ("whichever path delivered it first"; "An identical later delivery becomes none") |
| C3(b) | FTP first, cable second | as C3(a) with the `via`s swapped | the Wireless copy is the photo, the Cable copy the duplicate | R-I27; M37 ("by either path") |
| C3(c) | duplicate across exposures and time | `expose(0)`; `deliver("A.ARW", d1, Wireless)` @5 000; **no** exposure; `a.clock` + 2 h; `deliver("A.ARW", d1, Wireless)` | second → `duplicate` (`association: filename`); one photo | R-I27 ("whatever its exposure (or none) and however long ago") |
| C3(d) | same name, other bytes | `deliver("B.ARW", d1, Wireless)`; `deliver("B.ARW", d2, Wireless)` (d2 ≠ d1); `deliver("B.ARW", d3, Wireless)` | rows: `photo` → `B.ARW`; `renamed_photo` → `B__dup1.ARW`; `renamed_photo` → `B__dup2.ARW`; all three files exist with their bytes; three photos after the watcher | R-I27 ("Same key with a different hash → a new photo, named with the lowest free `__dupN`"); M38 ("a same-name file with other bytes becomes its own row") |
| C3(e) | duplicates do not complete a count; RAW and JPEG distinct | `expose(0)` with `expected_files: 2`; `deliver("DSC00010.ARW", d)`, `deliver("DSC00010.ARW", d)` (dup), `deliver("DSC00010.JPG", j)` | after the dup `complete_for(e) == false`; after the JPEG `true`; `.duplicates/` holds one file | MC-33 ("A shot is complete only when an expected count is known and met"); R-I27 |
| C3(f) | an existing file at the destination is never replaced | the test writes `root/<key(s)>/C.ARW` with bytes `x` by hand; `deliver("C.ARW", x, Wireless)`; then `deliver("C.ARW", y, Wireless)` | first → `duplicate` → `.duplicates/C.ARW`; second → `renamed_photo` → `C__dup1.ARW`; `root/<key>/C.ARW` still reads `x` | R-I27 ("The rename refuses to replace an existing file … On `EEXIST` the existing file's SHA-256 is compared"); M37 ("never overwritten") |
| C4(a) | target folder of the matched exposure's session | `activate(s)`; `expose(0)`; `activate(t)`; `deliver("D.ARW", d, Wireless)` @3 000 | `final_path = root/<key(s)>/D.ARW` (not `t`); after the watcher `photos(s)` has the row, `photos(t)` none | MC-33 ("of the session active at the matched exposure"); M38 |
| C4(b) | no exposure → session active at completion | `activate(t)`; `deliver("E.ARW", d, Wireless)` with no exposure | `final_path = root/<key(t)>/E.ARW`, `association: none`, row `session_id = t` | MC-33 ("or, if no exposure matches, of the session active when the file completed") |
| C4(c) | no session → `_Unassigned` | `a.sessions().deactivate()`; `deliver("F.ARW", …)` | `final_path = root/_Unassigned/F.ARW`; the photo lands in session 1 | R-I28 ("`_Unassigned` when the session is null or no longer exists") |
| C4(d) | folder key read at rename time; a SKU edit before the rename | `activate(s)` (item SKU `K1`, no photos); `expose(0)`; `a.items().update_identity(item, "K2", ..)` (key follows the SKU: folder `K2`); `deliver("G.ARW", d, Wireless)` | `final_path = root/K2/G.ARW`; the folder was created; after the watcher the photo is in `s` | R-I28 ("The folder key is read at the moment of the rename, and the folder is created if missing") |
| C4(e) | a committed delivery freezes the key | after C4(d): `a.items().update_identity(item, "K3", ..)` | `item.folder_key` stays `K2` (frozen like a first photo row); `store.item_folder_key_frozen(item) == true` | 05B MC-22 (amended); R-I28; M29 |
| C4(f) | MC-2 rule 0 is consulted before the folder rules | `deliver("H.ARW", d, Wireless)` for session `s` (folder `K2`); before the watcher runs the test deletes `s`'s row (`DELETE FROM sessions WHERE id = s`, the `mark_published` SQL pattern — "no longer exists") and `activate(t)`; then `a.watcher_event(final_path)` + `a.tick(1_000)` | the photo row is in session **1** (rule 0: the delivery's session is gone → Unassigned), not in `t` (what rule 3 would give once rule 2 finds no owner of `K2`) | 05A MC-2 rule 0 ("a file whose canonical path is a delivery's `final_path` is attributed to that delivery's session (null or deleted → Unassigned)"); R-I28 |
| C4(g) | `captured_at` from the exposure | `expose(1_000)`; `deliver(.., Wireless)` @5 000; watcher | `photo.captured_at == 1_000`; for a delivery with no exposure (`association: none`) completed at 7_000 → `captured_at == 7_000`; neither equals the file's mtime | 05A MC-4 (exception); R-I28; M11 |
| C5(a) | crash-ordered persistence | `deliver` with a `DeliveryFs` fake that fails `rename_exclusive` once (`a.ledger_fs_fail_next_rename()`) | the row exists with `state = pending` and its `final_path`; the `.part` still exists; `L.delivery` returned `Err`; after `a.kill_and_restart()` (bind → `recover_pending`) the row is `done` and the file is at `final_path` | R-I25 ("A delivery row is written `pending` … **before** the rename and set `done` after it. At startup, a `pending` row whose staging file still exists is renamed again") |
| C5(b) | pending with the final file present | a `pending` row whose `written_path` is gone and whose `final_path` exists (the test moves the file by hand to simulate a crash after the rename) ; restart | the row is `done`; nothing else changes | R-I25 ("one whose final file exists is set `done`") |
| C5(c) | a late file after a restart finds its exposure | `expose(0)`; `a.kill_and_restart()`; `deliver("I.ARW", d, Wireless)` @60 000 | `association: fifo` to that exposure; the exposure row is `received` | MC-33 ("The ledger persists across restarts so a late file still finds its exposure") |
| C6(a) | `pending_count` bounds (unit) | exposures `expecting` at ages 59 s, 60.001 s, 29 min, 30 min 1 s; one `received` at 5 min | `pending_count == 2` | R-I29 (`pending_count` = `expecting`, older than 60 s, newer than 30 min) |
| C6(b) | raise iff count > 0 and no transfer | `expose(0)`; `a.clock` → 61 s; `L.watchdog(now, transfer_active: true)` then `(now, false)` | first → `raised: false`, no health item; second → `raised: true`, `first_raise: true`, `H.has("camera_files_not_received")` with detail `1 photos taken but not received for 60 s`; a third call → `first_raise: false` | R-I29 ("shows iff `pending_count > 0` and no upload or download is in progress") |
| C6(c) | clear and dismiss [finalize_marks_missing_not_complete] | after C6(b): `deliver` the file → `watchdog` → cleared; or `L.dismiss()` → the exposure is `dismissed`, item cleared, `pending_count == 0` | as stated; a `dismissed` exposure is never a FIFO candidate for a later file (`association: none`) | R-I29 ("It clears when the count reaches 0 or the photographer dismisses it"); MC-33 (`missing_at_deadline` is an observation) |
| C6(d) | gaps on the health item | deliveries `DSC00001`, `DSC00002`, `DSC00004` with one `expecting` exposure older than 60 s | detail `1 photos taken but not received for 60 s · 1 gaps in the camera's numbering`; `WatchdogState.gaps == 1` | R-I29 ("Gaps … are logged and counted on the same health item; they never block anything") |
| C7(a) | uncertain after a session loss [uncertain_after_session_loss] | `expose(0)`; `L.session_uncertain()`; `deliver("J.ARW", d, Wireless)` @9 000 | the exposure is `uncertain`; the delivery has `association: none` (an uncertain exposure is not a candidate) and lands in the active session; the exposure stays `uncertain` | MC-33 ("`uncertain` after a session loss during a protected operation") |
| C8(a) | the receiver's sink is the ledger — one upload, one photo | `expose(0)`; `a.ftp().stor("DSC00001.ARW", data)` (the real receiver); `a.tick(0)` until the row is `done`; `a.watcher_event(final_path)`; `a.tick(1_000)` | `226`; `row("DSC00001.ARW") == { via: wireless, association: fifo, outcome: photo, state: done }`; `root/<key(s)>/DSC00001.ARW` reads `data`; `.incoming/` holds no `.part`; `photos(s).len() == 1` with `raw_path == final_path`; `a.ftp().size("DSC00001.ARW")` → `213 <len>` (the ledger is the receiver's `CompletedNames`) | M37, M38; MC-32 §2; R-I24 (listings from the ledger) |
| C8(b) | the candidate filter ignores the camera folders | `is_candidate(root/.incoming/.X.ARW.part)`, `is_candidate(root/.incoming/sdk-abc/X.ARW)`, `is_candidate(root/.duplicates/X.ARW)` (files created) | all `false`; `a.scan()` after C3(a) creates no row for the duplicate | 05A MC-2 (candidate filter: dot-prefixed directories); M9 |

## Delegated choices
Whether `associate` takes slices or iterators; the `DeliveryFs` fake's shape for C5(a) (recommended: a wrapper over `CameraFiles` with a
`fail_next_rename` flag, in testkit); the `.duplicates/` collision suffix for identical bytes (recommended `__dupN` there too); how `complete_for`
is exposed (a method on the service or a field of the row DTO); the exact `tracing` fields of `camera.sequence_gap`; index names in 006.

## Manual try-out
(Hardware-free.) 1. Nothing changes on screen yet. 2. Health has no new rows unless a photo is pending (impossible without the camera). 3. The database
gains two empty tables (`sqlite3 … ".tables"` shows `camera_exposures camera_deliveries`).

## Notes
- **Hard rules (owner):** never commit a Sony SDK file, the lab's `sdk_shim` headers, a `.env`, a credential or an SSH fingerprint; no Python
  (`ledger.py` is read, not run); never change the camera's network/FTP settings, delete a camera file (the ledger **moves** staging files it owns
  and never deletes; `.duplicates/` is never emptied) or kill any process; live view out of scope; the suite never needs the camera. No `implementer: fable`.
- **Planner readings (reversible):** (1) `complete_for` is informational: nothing in the product waits for "complete" (there is no `expected_files`
  source in this chapter). (2) A delivery whose rename fails leaves the row `pending`; recovery runs at every bind (settings save too), not only at
  start. (3) `camera_photo_session_uncertain` names the other session by its item title, else SKU. (4) The freeze is derived from the delivery rows,
  no new column on `items`.
- **Hazards:** `arrival_insert` canonicalises `raw_path` (`store.rs:1376`) — compare `final_path` **canonicalised** in `delivery_for_path`. The worker
  lock must cover the `pending` insert, or two copies finishing together could both decide `photo`. `renamex_np` works only on the same volume and
  APFS; `.incoming/` is under the capture root, so it is. `std::fs::rename` **replaces** on macOS — forbidden for camera files (R-I27). The watcher
  may fire for the `.part` rename target before `done` is written: rule 0 must read the `pending` row too (the `final_path` is written before the
  rename). `ItemsService::update_identity` is the only writer of `folder_key` (`items.rs:264–275`).
- **Projection** is on: the ledger is the inventory's #2–#5 silent-failure mechanisms; the projectionist checks that every unit row's expected
  outcome follows from the R-I26/R-I27 text (not from the lab's semantics where they differ: no window in the lab, per-capture duplicates in the lab)
  and that C4(f) is constructible as written.
- Registry additions made by the planner in master plan §6.4, §6.5, §6.6, §6.11. No intention-book change.
- Estimated size: ~19 paths, ~2 100 lines (≈ 1 000 mechanism + service + store, ≈ 1 100 tests).

## Review log


- **2026-11-06, carried from P42 projection r0** (planner): `bind` is the last statement of `App::build` (L3), so `recover_pending` and the ledger
  rebuild run only for a build that succeeded; timed reads in C8(a) ("`a.tick(0)` until the row is `done`") use P42's `settle(a, pred)` fixture (L7).
