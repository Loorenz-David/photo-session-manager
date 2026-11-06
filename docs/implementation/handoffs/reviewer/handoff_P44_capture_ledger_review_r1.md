---
plan: P44_capture_ledger
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-06
actor: opus
---
# P44 — Capture ledger — review r1

First review of this phase (no earlier rounds). Authority order: `docs/process/flow.md` (no mutation regime, B1–B4 only) over the global
doctrine; 10E MC-33 (R-I25…R-I29, changelog 36), 05A MC-2 rule 0 / MC-4, 05B MC-22, M37/M38; the plan's contract and 36 rows.

## Gate
- Tracker row read `IMPLEMENTED`; checkpoint `046d3c5` (`CHECKPOINT (not approved): P44_capture_ledger implement r1`) is in `git log`. Pass.

## Verified correct
**Perimeter.** `git show --stat 046d3c5` has 26 paths, equal to the handoff's declared list (the plan's 24 plus the tracker row and the plan
Review log). The p43 edits outside Task 6's named rows (C1(b), C2(b), C2(d), C3(a), C6(b)) belong to the same category: they read
`_Unassigned/<name>` instead of the `.part`, and no other assertion changed. C4(a)–(c) moved to a standalone `listing_receiver`. No extra or
missing path.

**Gates (reviewer run, code tree `046d3c5`, working tree differs only in docs).**
| Command | Result |
|---|---|
| `cargo test --workspace` | exit 0, 671 passed / 0 failed (`p44_capture_ledger` 25/25) |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo fmt --all -- --check` | exit 0 |
| `npm run typecheck` (apps/desktop) | exit 0 |
| `npm test` (apps/desktop) | exit 0, 369/369 (the known P33 flake did not occur) |

**Production, read against the contract.**
- `associate` (mechanism) applies every filter itself. Rule (1): same key **and** SHA-256 → `filename`. Rule (2): same stem, `|Δ| ≤ 300 s` →
  `stem`. Rule (3): `exposed_at ≤ completed_at`, `completed_at − exposed_at ≤ 300 s` (both inclusive), and "still expecting", counted from
  non-duplicate priors against `expected_files.unwrap_or(1)`; oldest first. The session is the pick's (equal to the common one when all agree).
  `ambiguous_with` holds the distinct differing sessions in candidate order. Rules (1)/(2) carry the prior's exposure and session even when
  they are `None`.
- The service passes every `expecting` exposure and every `done` prior with the same key or stem, with no pre-filtering. `association: none`
  takes `active_session_id()` at completion.
- The duplicate decision runs against `done` rows only. The lowest free `__dupN` comes from the priors' final names, then is bumped on `Exists`.
- One process-wide `LedgerShared.worker` lives in the `CameraRuntime`, shared by every `LedgerService` that `bind` builds. It is held from
  decision to `done`, and also by `recover_pending`, `session_uncertain` and `dismiss`.
- The `pending` row (with `written_path` and `final_path`) and the exposure status are written in one transaction, which also reads the
  item's folder key, **before** `create_dir_all` and the rename.
- `rename_exclusive` = `renamex_np(RENAME_EXCL)`. `EEXIST` → `Exists`; every other errno → `Err`, with no copy, link or `std::fs::rename`
  fallback. A grep finds no `std::fs::rename` on any ledger path.
- On `Exists`, the row's final path is rewritten before each retry: the same hash → `.duplicates/`, other bytes → the next `__dupN`, and a
  collision in `.duplicates/` → `__dupN` there. The exposure is re-evaluated when the delivery becomes a duplicate.
- Take-over: a delivery whose `written_path` matches a `pending` row updates that row in place (sha, size, completed_at, decision) and
  inserts no second row.
- `recover_pending`: if the staging file exists, it renames again to the **stored** final path (the folder key is not re-read). If the
  final file exists, the row becomes `done`. If neither exists, it logs `camera.pending_delivery_lost`.
- `item_folder_key_frozen` = a photo row OR a `photo`/`renamed_photo` delivery in `pending`/`done` for one of the item's sessions. Both
  `Store` and `StoreTx` have it, and `update_identity` uses it inside its transaction.
- MC-2 rule 0 runs in `accept` before attribution and before the mtime. `delivery_for_path` reads `pending` rows too. A deleted or `NULL`
  session → 1. `captured_at = COALESCE(exposed_at, completed_at)`. Final paths are stored NFC from the canonical capture root.
- `completed_names_since` keeps only `done` rows, deduplicated by name with the latest kept. The ledger is the receiver's `CompletedNames`.
- Watchdog: both bounds exclusive. It raises iff `count > 0 && !transfer_active`, with exact detail strings including the gap clause.
  Gaps cover the last 30 min and are logged as `camera.sequence_gap`. The raised flag lives in `LedgerShared`.
- Migration 006 has R-I25's columns plus `written_path TEXT NOT NULL` and `created_at`, and CHECK lists that include the reserved
  `missing_at_deadline`. Nothing writes that value.

**Tests (flow §4).** All 36 rows have a test at the row's boundary that asserts the exact outcome and would fail on a wrong one:
- The C1/C2/C6(a) unit rows assert only return values. The C2 rows feed all exposures plus the growing priors, so a test cannot assert its
  own fixture.
- C4(f) separates rule 0 from rule 3: without rule 0 the photo would land in `t`.
- C4(e) has no photo row, so only the delivery can freeze the key.
- C5(d) pins exactly one row.
- C8(a) goes through the real receiver.
- C7(b) observes `ambiguous_with` only through the health detail, because it is not stored. C2(h) pins the struct values. Not blocking.

No orphan tests. Nothing in the tests fails the over-engineering test: no internal call counts, no query text, no private helpers.

**Assumptions (1–12).** None contradicts the intention:
- (4) reads the folder key in the pending-row transaction rather than "at the rename". The outcome is the same, because the freeze holds from
  that commit and no SKU edit can move the key in between.
- (6), (7), (10) are consistent with R-I29 and the bind order. They are listed in plain words in the narrative.

**Hard constraints.** H1 not touched. H2 holds as 10E amends it: the ledger moves only the staging files it owns and deletes nothing. H3 not
touched. H4: crash-ordered writes plus recovery, and the take-over keeps the photo on a re-send. H5 not touched. H6 not touched. H7: core
never strips `_N`, and `renamex_np` lives in infra. H8: no Tauri in core. No secrets and no live services.

## Findings
None blocking (B1–B4: none).

## Notes written to backlog
1. P44 → P46: `recover_pending` builds its placement with `matched: None`. When recovery's `EEXIST` turns a row into a duplicate, the
   exposure it had made `received` is not re-evaluated.
2. P44 → P46: an active transfer resets the raised flag, so `first_raise` fires again after every transfer while shots are still pending.
   P46's `ftp-status` would then repeat.
3. P44 → P46: before the first bind, `Unbound` also refuses `exposure_observed` and `session_uncertain`. P46's sidecar events must start
   after the bind.
4. P44 → master plan §6.11 (coordinator): registry drift, `Store::deliveries_for_exposure` added and `update_pending_delivery(id, &PendingUpdate)`.
5. P44 (tests): C7(b) observes `ambiguous_with` through the health detail only.

## Lessons for the plans
- Task 6 named 2 of the 7 p43 rows that read the `.part`. Enumerate such edits by grep over the test file, not from memory.
- When a recovery path shares the worker's placement helper, the plan should state which side effects it repeats, such as exposure
  re-evaluation (note 1).

## Carry-forward dispositions
| Note | Destination |
|---|---|
| 1, 2, 3 | P46 (watchdog wiring, sidecar events) |
| 4 | coordinator, master plan §6.11 before the P45/P46 prompts |
| 5 | none (informational for later reviewers) |

## Write perimeter (this session)
- `docs/implementation/master_plan.md`: the P44 tracker row only (→ `APPROVED`).
- `docs/implementation/plans/P44_capture_ledger.md`: the Review log entry.
- `docs/implementation/backlog.md`: 5 rows added after the P43 rows.
- `docs/implementation/narratives/P44_capture_ledger.md`: new.
- This handoff.

Probe declaration: no production or test file touched, no probe files, no database or file-system side effects outside the suite's own
temp dirs. Nothing committed (the runner commits).

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
