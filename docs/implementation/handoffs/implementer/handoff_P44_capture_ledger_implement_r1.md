---
plan: P44_capture_ledger
role: implement
round: 1
state: DONE
date: 2026-11-06
actor: opus
---
<!-- actor: Claude Opus via PSM_IMPLEMENTER=claude; the prompt template says `codex`, but this session was not Codex. -->

# P44 — Capture ledger — implement r1

## What was built
- **Every camera file becomes exactly one photo, in the right session.** A file the camera delivers (today only over Wi-Fi, through the
  P43 FTP receiver) now goes through the capture ledger. The first copy of a camera file is moved from the staging folder into its session's
  folder (`<capture root>/<item folder key>/`, or `_Unassigned` with no session). A later identical copy, by cable or Wi-Fi, at any time, goes to
  `<capture root>/.duplicates/` and never becomes a photo. A file with a reused name but other bytes becomes its own photo, named `…__dup1`,
  `…__dup2`. Nothing is ever overwritten or deleted: the move refuses to replace an existing file (macOS `renamex_np(RENAME_EXCL)`).
- **Which session.** A file joins the session that was open when the shutter fired (the oldest exposure within 300 s still waiting for a file),
  else the session open when the file arrived. A RAW and its JPEG stay together. If the shots waiting in that window belong to different sessions,
  the photo still goes to the picked one and the Health page lists it (`Photo M.ARW may belong to t`).
- **The photo row follows the ledger's decision** (MC-2 rule 0): a file at a ledger final path joins that delivery's session (Unassigned if
  that session was deleted), and its capture time is the shutter time, not the file's arrival time.
- **The folder name freezes** once a delivery is committed to an item's session, so a SKU edit cannot move a folder that a file is about to land
  in.
- **Crash safety.** The ledger records where a file will go *before* moving it and marks it done after. After a crash, the next start (or settings
  save) finishes the move. If a move fails (e.g. a non-APFS volume), the receiver answers 426, the camera re-sends into the same staging file, and
  the re-send takes over the waiting record: still one photo.
- **Watchdog.** "N photos taken but not received for 60 s" (plus "· g gaps in the camera's numbering") is raised when shots older than 60 s
  (newer than 30 min) have no file and no transfer is active; it clears when the files arrive or when dismissed. (P46 wires the timer; P44
  provides `LedgerService::watchdog`.)
- Code: `crates/core/src/mechanisms/capture_ledger.rs` (pure rules), `crates/core/src/services/camera_ledger.rs` (`LedgerService`,
  `LedgerShared`, `WatchdogState`), `crates/infra/src/camera_files.rs` (`rename_exclusive`, `CameraFiles`), migration
  `006_camera_ledger.sql` (`camera_exposures`, `camera_deliveries`), rule 0 in `services/arrival.rs`, the freeze in `services/items.rs`,
  `CameraRuntime` builds the ledger at every bind over the new store sharing one `LedgerShared`, installs it as the receiver's sink and names,
  and runs `recover_pending`; `NoopSink` removed; `CompletedNames` moved to `psm_core::ports::camera` (re-exported by `psm_infra::ftp`).

## Tests → criterion rows
36 rows → 37 tests (11 `mc33_*` mechanism unit tests, 1 `rename_exclusive` inline test, 25 behaviour tests in
`crates/testkit/tests/p44_capture_ledger.rs`). Every test in the phase's test files is in this map; no orphan.

| Row | Test | Assertion shape |
|---|---|---|
| C1(a) | `mechanisms::capture_ledger::tests::mc33_file_key_and_stem` | exact return values of the five inputs (incl. `DSC01234_1.ARW` kept) — as specified |
| C1(b) | `mc33_sequence_gaps` | `== [Gap{DSC,2,4,1}]` — as specified |
| C2(a) | `mc33_one_exposure_one_file_is_provisional` | whole `AssociationDecision` equality; `is_complete(None,1) == false` — as specified |
| C2(b) | `mc33_expected_count_completes` | the four booleans — as specified |
| C2(c) | `mc33_raw_and_jpeg_grouped_by_stem` | `[e1 Fifo, e1 Stem, e2 Fifo, e2 Stem]`, fed all exposures + growing priors — as specified |
| C2(d) | `mc33_burst_with_lagging_uploads_in_order` | file i → exposure i — as specified |
| C2(e) | `mc33_bracketing_with_expected_files` | 100–102 → e1 Fifo, 103 → e2 — as specified |
| C2(f) | `mc33_file_before_any_exposure_is_unassociated` | `exposure_id None`, `association None` — as specified |
| C2(g) | `mc33_fifo_window_bounds` | the five pinned bounds — as specified |
| C2(h) | `mc33_candidate_set_decides_the_session` | full decisions for s/s, s/t; `ambiguous_with == [None]` — as specified |
| C6(a) | `mc33_pending_count_bounds` | `== 2` over the seven ages — as specified |
| (core mechanism) | `psm_infra::camera_files::tests::mc33_rename_exclusive_never_replaces` | `EEXIST` → `Exists` (both files untouched), else `Moved` — the plan's one inline test |
| C3(a) | `c3a_cable_then_ftp_is_a_duplicate` | both rows' via/outcome/association/final_path exact; both files' bytes; `photos(s).len()==1` — as specified |
| C3(b) | `c3b_ftp_then_cable` | (via, outcome) pairs and both final paths — as specified |
| C3(c) | `c3c_duplicate_across_exposures_and_time` | second row `duplicate`/`filename`; one photo — as specified |
| C3(d) | `c3d_same_name_other_bytes` | three (outcome, final_path) exact; bytes per file; three photos — as specified |
| C3(e) | `c3e_duplicates_do_not_complete_a_count` | status + `complete_for` after each delivery; JPEG `stem`; `.duplicates` holds one file — as specified |
| C3(f) | `c3f_existing_destination_never_replaced` | `duplicate` → `.duplicates/C.ARW`, then `renamed_photo` → `C__dup1.ARW`; hand file still `x` — as specified |
| C3(g) | `c3g_reused_name_after_counter_wrap` | (fifo, e2, t), exact `__dup1` path in `key(t)`, e2 `received`, one photo each in t and s — as specified |
| C4(a) | `c4a_target_folder_of_the_matched_exposure` | final path in `key(s)`; photo in s, none in t — as specified |
| C4(b) | `c4b_no_exposure_session_at_completion` | final path in `key(t)`, `none`, row session t — as specified |
| C4(c) | `c4c_no_session_goes_to_unassigned` | `_Unassigned/F.ARW`; photo in session 1 — as specified |
| C4(d) | `c4d_folder_key_read_at_rename_time` | final `root/K2/G.ARW`; folder absent before, created after; photo in s — as specified |
| C4(e) | `c4e_committed_delivery_freezes_the_key` | key stays `K2` after `K3` edit with no photo row; `item_folder_key_frozen == true`; control item moves to `U3`, `false` — as specified |
| C4(f) | `c4f_rule_zero_before_the_folder_rules` | photo in session 1, none in t (rule 3 would give t) — as specified |
| C4(g) | `c4g_captured_at_from_the_exposure` | `captured_at == 1_000` and `== 7_000`; neither equals the mtime — as specified |
| C5(a) | `c5a_crash_ordered_persistence` | `Err`; row `pending` with exact final and written path; `.part` exists; exposure `received`; after restart `done`, file at the stored path — as specified |
| C5(b) | `c5b_pending_with_the_final_file_present` | `done`; bytes unchanged; row count unchanged; `.duplicates` empty; no `__dup` file — as specified |
| C5(c) | `c5c_late_file_after_restart_finds_its_exposure` | (fifo, e) and e `received` — as specified |
| C5(d) | `c5d_resent_staging_file_takes_over_its_pending_row` | `Ok(Photo)`; exactly one row, `done`, `completed_at 4_000`; exact path; `.duplicates` empty; one photo — as specified |
| C6(b) | `c6b_raise_iff_count_and_no_transfer` | not raised / no item; raised + first_raise + exact detail; third call first_raise false — as specified |
| C6(c) | `c6c_clear_and_dismiss` | cleared at count 0; dismiss → exposure `dismissed`, item cleared, count 0; later file `none` — as specified |
| C6(d) | `c6d_gaps_on_the_health_item` | count 1, `gaps == [Gap{DSC,1,4,2}]`, exact detail with the gap clause — as specified |
| C7(a) | `c7a_uncertain_after_session_loss` | exposure `uncertain`; delivery (none, no exposure, s), path in `key(s)`; stays `uncertain` — as specified |
| C7(b) | `c7b_split_candidate_set_lists_the_photo` | M (e1, fifo, s), path in `key(s)`, detail `Photo M.ARW may belong to t`; N (e2, t), detail `… N.ARW may belong to s`. `ambiguous_with` is not a stored column, so it is observed through the health detail — slightly narrower than the row's struct literal |
| C8(a) | `c8a_receiver_sink_is_the_ledger` | real receiver `226`; row (wireless, fifo, photo, done); file bytes; no `.part`; one photo with `raw_path == final_path`; `SIZE` → `213 <len>` — as specified |
| C8(b) | `c8b_arrival_ignores_the_camera_folders` | `photos(s)` and `COUNT(*) photos` stay 1 after watcher events on `.duplicates` and `.incoming` and a scan — as specified |

Red baseline: **not captured** before the first production edit (flow §4 has no baseline step). Stated, not reconstructed.

## Assumptions
1. `update_pending_delivery(id, &PendingUpdate)`: the registry's nine fields grouped in one struct (clippy's seven-argument limit).
2. An exposure is `received` when its non-duplicate `pending`/`done` deliveries reach `expected_files` (1 when unknown), else `expecting`. Only
   `expecting`/`received` exposures are ever moved; `uncertain` and `dismissed` are never touched. If a rename's `EEXIST` turns a delivery into a
   duplicate, or a re-send take-over moves the row's FIFO pick, the old exposure is re-evaluated the same way.
3. A re-send that takes over a `pending` row restores, as a candidate, the exposure that row's own FIFO pick had made `received`.
4. The folder key is read in the same transaction that writes the `pending` row with its final path (the path the rename uses), which also
   freezes the key. A SKU edit cannot slip between the two.
5. Rule 0 applies to the capture path (hook, watcher, scan), not to Import.
6. `WatchdogState.gaps` is the list of `Gap`s (`g` = its length). While a transfer is active the notice is hidden and the raised flag reset, so
   `first_raise` fires again when it reappears.
7. `dismiss()` dismisses only the exposures the notice counted (60 s < age < 30 min), not a shot taken seconds ago.
8. `session_uncertain` and `dismiss` take the same worker lock as deliveries.
9. The "may belong to" detail names the first other session in the candidate set (item title, else SKU, `Unassigned` for none).
10. Before the first bind the runtime's sink refuses deliveries (the receiver answers 426, the camera re-sends) instead of P43's
    accept-and-drop `NoopSink`.
11. Final paths are stored NFC-normalised from the canonical capture root, so rule 0's comparison with `canon()` paths is like with like.
12. `set_delivery_fs` also rebuilds the current ledger over the new `DeliveryFs` (same store and shared lock), so the seam takes effect without a
    rebind.

## Deviations
- **Registry addition:** `Store::deliveries_for_exposure(exposure_id)` (not in the §6.11 list). The `received` transition and `complete_for` must
  count an exposure's deliveries across different stems (a bracket `DSC00100..102` shares no key or stem). **For the coordinator to fold into
  §6.11.**
- **p43, beyond the rows Task 6 names:** five more `TestApp` rows read the `.part` after a successful upload and now read
  `root/_Unassigned/<name>`: C1(b), C2(b), C2(d), C3(a), C6(b). This is the category the Files section names ("rows that read the `.part`"); every
  other assertion is unchanged. C4(a)–(c) use a local `listing_receiver` (a standalone receiver with its own `ManualClock`), because C4(b)/(c)
  move the clock and the shared `receiver(..)` helper's clock is fixed at 0. The standalone rows are untouched.
- **Migration 006 uses `IF NOT EXISTS`:** `p27 c4a_migration_removes_old_jobs_and_health_codes` rewinds `user_version` to 2 and re-runs every later
  migration. A normal run is unaffected.

## Files changed
Write perimeter, all in checkpoint `046d3c5` (26 paths):
- Core (new): `crates/core/src/mechanisms/capture_ledger.rs`, `crates/core/src/services/camera_ledger.rs`.
- Core (changed): `crates/core/src/domain/camera.rs`, `crates/core/src/mechanisms/mod.rs`, `crates/core/src/ports/camera.rs`,
  `crates/core/src/ports/store.rs`, `crates/core/src/services/{arrival.rs, items.rs, mod.rs}`.
- Infra (new): `crates/infra/src/camera_files.rs`, `crates/infra/src/sqlite/migrations/006_camera_ledger.sql`.
- Infra (changed): `crates/infra/Cargo.toml` (`libc`), `crates/infra/src/ftp/mod.rs`, `crates/infra/src/lib.rs`,
  `crates/infra/src/sqlite/store.rs`.
- Workspace: `Cargo.lock`.
- Integrations: `crates/integrations/src/camera/{mod.rs, runtime.rs}`, `crates/integrations/src/lib.rs`.
- Testkit: `crates/testkit/src/{collecting_sink.rs, test_app.rs}`; tests `crates/testkit/tests/p44_capture_ledger.rs` (new),
  `crates/testkit/tests/{p01_storage_identity.rs, p43_ftp_receiver.rs}`.
- Docs: `docs/implementation/master_plan.md` (P44 tracker row only), `docs/implementation/plans/P44_capture_ledger.md` (Review log entry).
- Not committed (the runner commits it): this handoff.
- Mutation-probe files: none (no mutation regime, flow §4). No architecture graph in this project. Nothing touched outside the list.

## Checkpoint SHA
`046d3c5` — `CHECKPOINT (not approved): P44_capture_ledger implement r1` (tree clean after the commit).

## Stamp
Tree `046d3c5` (before the commit, the same content in the working tree):
| Command | Result |
|---|---|
| `cargo test --workspace` | exit 0 — 671 passed, 0 failed |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo fmt --all -- --check` | exit 0 |
| `npm run typecheck` (apps/desktop, Node 22) | exit 0 |
| `npm test` (apps/desktop) | first run 368/369: the known P33 C4(d) flake (`p33_split_grid.test.tsx`, recorded in the P41 tracker note); rerun 369/369, exit 0. No frontend file changed in P44 |
| `cargo test -p psm-testkit --test p44_capture_ledger` | 25/25 |
| `cargo test -p psm-testkit --test p43_ftp_receiver` | 26/26 |
| `cargo test -p psm-core capture_ledger` / `cargo test -p psm-infra camera_files` | 11/11, 1/1 |

## Manual try-out
Hardware-free; nothing changes on screen yet.
1. Start the app once with a scratch data dir: `cd apps/desktop && PSM_DATA_DIR=/tmp/psm-p44 npm run tauri -- dev`, then quit it.
2. `sqlite3 /tmp/psm-p44/session_manager.db ".tables"` lists `camera_exposures` and `camera_deliveries` among the tables. Both are empty.
3. The Health page shows no new rows: the "photos not received" notice needs a camera exposure, and the "may belong to" notice needs one too.
   Neither can happen without the camera (P46).
No hardware probe was run: in P44 only the FTP receiver feeds the ledger, and the receiver starts only when a camera support package is
installed (P50). The path through the real receiver is covered by C8(a).

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
