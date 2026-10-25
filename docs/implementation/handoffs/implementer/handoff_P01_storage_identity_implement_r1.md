---
plan: P01_storage_identity
role: implement
round: 1
state: DONE
date: 2026-10-25
actor: codex
---

# P01 storage identity — implement round 1

## What was built

An item can be created with a local SKU and its first session, another session can be created for it, and its SKU or barcode can be edited. The capture folder key follows the SKU until the first photo, then stays fixed. A built-in Unassigned session is present from first launch. The desktop exposes create, edit, create-session and get-session commands with generated TypeScript bindings.

SQLite migration 002 adds every registered domain table. The transactional Store port handles items, links, sessions, canonical photo insert-or-get, and RAW state. RAW identity resolves parent symlinks, Unicode spelling and case, while preserving the file-name symlink. SHA-256 streams through a read-only file handle; integrity verification persists `ok`, `raw_changed`, or `raw_missing` without replacing the recorded hash. UTC timestamps are integer milliseconds.

## Tests → criterion rows

| Row | Boundary test | Result |
|---|---|---|
| C1(a) | `c1a_open_creates_all_domain_columns` | exact table and column set |
| C1(b) | `c1b_unassigned_is_one_null_item_session` | id 1, null item, one such row |
| C2(a) | `c2a_canon_normalizes_nfd_filename` | NFC and NFD converge |
| C2(b) | `c2b_canon_converges_tmp_private_tmp` | aliases converge |
| C2(c) | `c2c_canon_rejects_relative` | `relative_path` |
| C2(d) | `c2d_canon_uses_disk_case` | on-disk case; declared skip on a case-sensitive test volume |
| C2(e) | `c2e_canon_resolves_parent_but_keeps_file_symlink` | parent resolved, file symlink retained |
| C2(f) | `c2f_canon_removes_parent_dotdot_and_trailing_slash` | normalized path |
| C3(a) | `c3a_store_deduplicates_normalized_raw_path` | one canonical photo row |
| C4(a) | `c4a_sha256_streams_without_changing_raw` | exact digest, bytes and mtime unchanged |
| C4(b) | `c4b_changed_raw_sets_integrity_without_rehashing_record` | `raw_changed`, original hash |
| C4(c) | `c4c_missing_raw_sets_integrity_without_rehashing_record` | `raw_missing`, original hash |
| C4(d) | `c4d_unchanged_raw_stays_ok` | `ok` |
| C5(a) | `c5a_folder_key_replaces_non_ascii_and_spaces` | exact key |
| C5(b) | `c5b_folder_key_replaces_leading_dot_or_underscore` | exact keys |
| C5(c) | `c5c_folder_key_enforces_length` | invalid and boundary lengths |
| C5(d) | `c5d_create_refuses_case_insensitive_folder_collision` | named collision, no row, next item succeeds |
| C6(a) | `c6a_create_local_creates_one_session_at_clock_time` | local item and one session at clock time |
| C6(b) | `c6b_sku_change_freezes_key_after_first_photo_and_checks_stored_key` | key follows then freezes; stored-key collision |
| C7(a) | `c7a_item_and_session_timestamps_are_integer_milliseconds` | exact millisecond values |

All 20 tests live in `crates/testkit/tests/p01_storage_identity.rs`. C2(d) emits an explanatory skip message if its temporary volume is case-sensitive. No extra core mechanism unit tests were added; the listed mechanism outcomes are exercised directly at their criterion boundaries.

## Assumptions

- `Store::transaction` uses an object-safe closure over `StoreTx` so item and session creation are atomic and services stay independent of SQLite.
- `barcode: None` on an identity edit means retain the current barcode; the plan specifies an optional input without a separate clearing action.
- Desktop command IDs retain SQLite's 64-bit Rust representation and export as TypeScript numbers. The single-operator database is assumed to remain within JavaScript's safe integer range.
- A photo's creation time is used for the session's activity time when the Store inserts that photo.

## Deviations

- Module registration files, Cargo manifests, and `Cargo.lock` were changed to wire the planned files; these are outside the plan's explicit file list.
- `IntegrityCheck` lives in new `crates/core/src/services/integrity.rs`, as a service using the Store port.
- The inherited P00 reopen test now compares migration versions before and after its second open. Its fixed expectation of version 1 would fail after the planned migration 002, although its required no-repeat-migration behavior still holds.
- The suggested `mark_published(session, seq)` testkit helper was omitted because no P01 test calls it; master plan §7.5 requires each added helper to have a caller in its phase.

## Files changed

Checkpoint paths: `Cargo.lock`; `apps/desktop/src-tauri/src/commands/{items.rs,mod.rs,sessions.rs}`; `apps/desktop/src-tauri/src/lib.rs`; `apps/desktop/src/services/bindings.ts`; `crates/core/src/domain/{item.rs,mod.rs,photo.rs,session.rs}`; `crates/core/src/{error.rs,lib.rs}`; `crates/core/src/mechanisms/{canon.rs,hash.rs,mod.rs,naming.rs,time.rs}`; `crates/core/src/ports/store.rs`; `crates/core/src/services/{integrity.rs,items.rs,mod.rs,sessions.rs}`; `crates/infra/Cargo.toml`; `crates/infra/src/sqlite/{migrations/002_domain.sql,store.rs}`; `crates/testkit/Cargo.toml`; `crates/testkit/src/test_app.rs`; `crates/testkit/tests/{p00_spike.rs,p01_storage_identity.rs}`; `docs/implementation/{master_plan.md,plans/P01_storage_identity.md}`.

This handoff was written after the checkpoint so it can name the actual SHA; the runner owns its later handoff commit under flow §8.

## Checkpoint SHA

`af9d93e` — `CHECKPOINT (not approved): implement P01 storage identity`

## Stamp

| Command | Result |
|---|---|
| `cargo test --workspace` | exit 0: 20/20 P01 boundary tests, 16/16 P00 boundary tests, four existing Rust tests/binding test |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo fmt --all -- --check` | exit 0 |
| `PATH=/Users/davidloorenz/.nvm/versions/node/v22.22.3/bin:$PATH npm run typecheck` in `apps/desktop` | exit 0 |
| `PATH=/Users/davidloorenz/.nvm/versions/node/v22.22.3/bin:$PATH npm test` in `apps/desktop` | exit 0; no frontend test files in this phase |
| `git diff --check` before checkpoint | exit 0 |

No pre-existing failures remained. Tests use temporary data directories and did not require the camera, Capture One, Shopify or the network.

## Manual try-out

1. From `apps/desktop`, run `PATH=/Users/davidloorenz/.nvm/versions/node/v22.22.3/bin:$PATH npm run tauri -- dev`. The Health view still opens; this phase adds no new screen.
2. In another terminal, run `sqlite3 ~/Library/Application\ Support/com.loorenz.sessionmanager/session_manager.db '.tables'` to see the domain tables.
3. Run `sqlite3 ~/Library/Application\ Support/com.loorenz.sessionmanager/session_manager.db 'select id, item_id from sessions;'`. A fresh database shows `1|` for Unassigned.

## ⚠ OWNER DECISIONS REQUIRED (0)

None.
