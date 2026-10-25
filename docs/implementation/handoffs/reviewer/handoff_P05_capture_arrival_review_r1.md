---
plan: P05_capture_arrival
role: review
round: 1
verdict: CHANGES_REQUESTED
state: DONE
date: 2026-10-25
actor: opus
---
# P05 review, round 1: CHANGES_REQUESTED

All 33 rows hold at their boundaries, and suite, lint and typecheck are green. One production defect blocks (B1). Arrival de-duplicates only per (session, file), so the startup scan creates a **second** row for any RAW whose row has left its folder's session. Two cases trigger it: photos moved out of Unassigned or out of an item's session, and RAWs lying in the capture root while a different session is active. The fix is a first-sight rule in the arrival transaction, plus a test input on C7(b).

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Verified correct
- **Gate.** Tracker `IMPLEMENTED`; checkpoint `8a87369` exists. `git show --stat 8a87369` lists 29 paths, equal to the handoff's declared list. The plan named `crates/infra/src/watcher.rs` and `crates/core/src/domain/photo.rs`, but neither needed a change: capture-root wiring lives in the composition root, `crates/integrations/src/lib.rs`. The extra paths are declared deviations: module exports, `CameraProvider::raw_extensions` and the Sony impl, `AppEvent`, composition, the Tauri event bridge, and the testkit clock. Commits after the checkpoint are runner-only (implement handoff, review prompt).
- **Stamp** on tree `eb94cbc` (`git status --porcelain` empty; code identical to `8a87369`):
  - `cargo test --workspace`: exit 0, 156 tests (P05 33, P04 21, P03 28, P02 25, P01 20, P00 16, plus unit tests);
  - `cargo clippy --workspace --all-targets -- -D warnings`: 0;
  - `cargo fmt --all -- --check`: 0;
  - `npm run typecheck`: 0;
  - `npm test`: 0 (no frontend tests).
- **MC-2 candidate filter** (`mechanisms::arrival::is_candidate`): regular file; at most two components below the root; no component starting with `.` or equal to `CaptureOne`; extension matched case-insensitively against the camera adapter's set (`["arw"]`). The same filter applies to hook paths (`ArrivalService::candidate`). Rejected watcher paths write no `capture_events` row.
- **MC-2 stability** (`observe`/`tick`): watcher and scan candidates need equal (size, mtime) on two stats at least 1 s apart, with a re-check each 1 s. After 120 s they give up and set `error = stability_timeout` on the delivery's own event row. Hook paths skip stability.
- **MC-2 attribution** (`mechanisms::arrival::attribute`):
  - `_Unassigned` → 1;
  - exactly one session whose item `folder_key` equals the folder → that session;
  - otherwise the active session, or 1.
  Import always uses rule 3.
- **Row creation** (`SqliteStore::arrival_insert`): one transaction does the insert-or-get on (session, canon path) with `unreviewed`, position NULL, uid v4, `captured_at` = file mtime ms, and `source`. The same transaction upserts `raw_state.path_key` (first 16 hex of SHA-256 of the canonical path), enqueues `original_preview`, `hash` and `render_preview` (hash skipped when the RAW already has a hash or a hash job, per MC-3), and bumps `change_seq` and `last_activity_at`. That bump is what makes a published S1 dirty (C3(b)). `photo_added` is emitted only on insert.
- **MC-15 consumption.** `record_hook` is idempotent per spool file name, and dispatch runs after that commit. A failed dispatch leaves the file for replay without a second event row (H4).
- **MC-20** (`psm_infra::rawpreview`):
  - Traversal and selection: walks the IFD0 chain and SubIFDs with a cycle guard and keeps `0x0201/0x0202` streams that start with `FFD8` and whose last non-zero bytes are `FFD9`. It takes the largest area from the SOF header (header parse only, H3).
  - Copy and orientation: byte-copies via temp + rename. Orientation is IFD0 `0x0112` SHORT, correct for both byte orders. If no stream qualifies, there is no file, the path is NULL, and the job is `done`.
  - Real-RAW check: an independent read-only Python parse of the real `session test 11957.ARW` gave candidates (114850, 221976, valid, 1088×1080) and (43292, 4383, valid, 160×120), orientation 1. This equals the implementer's probe and the evidence.
- **Hash job.** `sha256_file`, writing only where `raw_hash IS NULL` (never recomputed in place, MC-3).
- **Import and assign** (`services::photos`): import records an `import` event, inserts with `source = import` under rule 3, then calls `EditingProvider::import(paths)`. `assign_unassigned` creates the item's session if none exists and moves all of session 1 through `ReviewService::move_photos`.
- **Tests.** C1(a)–C8(d) in `crates/testkit/tests/p05_capture_arrival.rs` assert each row's exact outcome at the row's boundary, and each would fail on a wrong outcome for the row's input. Examples:
  - C2(a) asserts both "no row after the second stat" and "one row after the re-check";
  - C2(b) asserts 1 error row, 1 event and 0 photos;
  - C5(b) puts EXIF 2 h ahead of the mtime;
  - C5(a) checks the v4 version and variant nibbles and the path_key;
  - C6(b) makes the largest stream invalid;
  - C6(d) compares RAW bytes and mtime;
  - C7(a) covers all three attribution outcomes in one startup.

  The core-mechanism unit tests are on functions the plan lists. None of these tests fails the over-engineering test.
- **Assumptions** (none contradicts the intention):
  - import accepts any regular file;
  - spool and scan are deferred while Capture One is unavailable;
  - P06 events get a logging stub;
  - a timeout sets the error on the delivery's own row (C2(b) asks for one row).
- **H1–H8.**
  - H1, H6: not touched.
  - H2: RAWs opened read-only (`fs::read`, `sha256_file`); C6(d).
  - H3: SOF header parse only.
  - H4: transaction dedupe and idempotent spool.
  - H5: import goes through the AppleScript adapter.
  - H7: see the note on hook payload parsing in core (plan-directed).
  - H8: `photos_import` and `photos_assign_unassigned` are lock → service → map.
  - No secrets and no live services in tests.

## Findings

### F1: blocking (B1), route: production
- **What is wrong.** `arrival_insert` skips creation only when a row exists for **(this session, path)**. Every startup scan re-attributes every RAW under the capture root. A RAW whose row has moved to another session is therefore re-created in the folder's session. Likewise, a RAW in the capture root itself (or in an unknown folder) is re-created in whichever session is active at that startup. Reviewer probe (scratch test, deleted, tree clean):
  1. `_Unassigned/u1.ARW` arrives in session 1. `assign_unassigned(item)` moves it to session 2. Reopen → rows `[(u1, 2), (u1, 1)]`: the photo is back in Unassigned.
  2. `<root>/x.ARW` arrives with S2 active. S3 is made active. Reopen → rows `[(x, 2), (x, 3)]`.

  The owner's real Capture One document has four ARWs directly in its capture root, so case 2 fires on the first restarts.
- **Authority.**
  - book 14 M1 ("exactly one `photos` row in the session that was active at its arrival") and M9 ("the number of rows created by arrival is exactly one … hook, watcher, startup scan, or any combination");
  - book 12 ("Startup reconciliation: scan … against `photos`");
  - book 06 §Unassigned "Assign product", and book 04 S-E (drag or assign the Unassigned set);
  - 05A MC-2 R-I3 (moving rows out of any session is allowed);
  - plan row C7(b) ("an already-known RAW → no new rows").
- **Correction.** For capture-sourced arrivals (hook, watcher, scan), the transaction creates a row only when **no** `photos` row on `canon(path)` exists in any session. The event row is still written. Import keeps the (session, path) rule (C8(c)). Extend the C7(b) test so the already-known RAW's row has been moved to another session before the reopen; the test then fails on today's behaviour.
- **Files the fix may touch.**
  - `crates/infra/src/sqlite/store.rs`
  - `crates/core/src/services/arrival.rs`
  - `crates/core/src/ports/store.rs` (only if the signature changes)
  - `crates/testkit/tests/p05_capture_arrival.rs`

## Notes written to backlog
- P05 → P01/P16: the capture root is compared after `fs::canonicalize` without NFC normalization, while `canon` NFC-normalizes candidate paths.
- P05 → P16: the capture root is cached for the process lifetime.
- P05 → P16: the startup scan sleeps 1 s on the startup thread with a wall-clock sleep, and writes one `scan` event per RAW per startup.
- P05: `ArrivalService::tick` drops the remaining ready paths of a tick when one `accept` fails.
- P05: `scan` aborts on the first unreadable subfolder.
- P05 → P06: Capture One hook payload parsing (`variant_id`, hook names) lives in core (plan-directed, H7 tension).
- P05 → P08: import accepts any regular file, including non-images.
- P05 (test): C8(a) checks only that the fake recorded an import of one path, not which path.
- P05 (test): the synthetic RAW always places the largest stream first.

## Lessons for the plans
- **05A MC-2 row creation and ch06 §Arrival** say a row is created once per (session, path). That was right before moves existed. With R-I3 moves and "Assign product" it contradicts M1/M9 and ch12's "scan against `photos`". Fold back: arrival through hook, watcher and scan is **first sight per RAW**, in any session. (session, path) stays the uniqueness key for move, copy and import. Plan rows that test re-scans should name a known RAW that was moved out (C7(b)).
- **H7 vs the plan's file list.** The plan put `mechanisms::hook_payload` (Capture One hook names, `variant_id`) and the `EditingEvents` trait in core. P06, which fills those handlers, is the place to decide whether the adapter should translate hook payloads into provider-neutral editing events.

## Write perimeter
- `docs/implementation/handoffs/reviewer/handoff_P05_capture_arrival_review_r1.md` (this file, new).
- `docs/implementation/plans/P05_capture_arrival.md`: Review log entry appended.
- `docs/implementation/master_plan.md`: P05 tracker row → `CHANGES_REQUESTED`.
- `docs/implementation/backlog.md`: 9 rows appended.
- Probe files:
  - `crates/testkit/tests/zz_review_probe_p05.rs` was created, run and deleted (an untracked scratch test; no production file touched).
  - `/tmp/psm_review_probe/arw.py` is outside the repo; it read the real ARW read-only.
  - Logs are in `/tmp/p05_review_*.log`, and build artefacts are under `target/` only. No database or state outside temp dirs.
- No commits.
