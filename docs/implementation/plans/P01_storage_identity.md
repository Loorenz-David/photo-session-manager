---
phase: P01_storage_identity
title: Storage, path identity, RAW hash and items/sessions
difficulty: high
projection: no
depends_on: [P00_spike]
---
# P01 — Storage, path identity, RAW hash and items/sessions

## Goal
Lay down the domain schema (migration `002_domain.sql`, every table of master plan §6.5) behind the `Store` port,
and implement the identity mechanisms every later phase relies on: path canonicalization (MC-1), RAW hash and
integrity (MC-3), folder key and SKU collision (MC-8 first half), UTC-ms timestamps (MC-4 storage), plus the
items/sessions services (create local item + its session, link an item, the built-in Unassigned session, SKU
immutability). **Not in this phase:** classification/order operations (P02), status (P03), arrival (P05),
attribute values and canonical forms (P10), export naming (P09).

## Read first
master plan §6.4–§6.6, §6.9, §7.5 · book `01`, `14`, `05` (items, item_links, sessions, photos, facts vs derived),
`02` (vocabulary) · contracts `05A` MC-1, MC-3, MC-4 (storage bullet), MC-8 (folder_key and collision only) · `06`
§Unassigned (first sentence) · evidence `capture_one_16.8.6.md` VERIFY-2 (`/private` read-back) and "Other runtime
observations" (EXIF clock, hash unchanged).

## Files expected to change
`crates/infra/src/sqlite/migrations/002_domain.sql` (new), `crates/infra/src/sqlite/store.rs` (Store impl),
`crates/core/src/ports/store.rs` (trait), `crates/core/src/domain/{item,session,photo}.rs` (new),
`crates/core/src/mechanisms/{mod.rs,canon.rs,hash.rs,naming.rs,time.rs}` (new),
`crates/core/src/services/{items.rs,sessions.rs}` (new), `crates/core/src/error.rs`,
`apps/desktop/src-tauri/src/commands/{items.rs,sessions.rs}` (new; `item_create`, `item_update_identity`,
`session_create`, `session_get`), `apps/desktop/src/services/bindings.ts` (regenerated),
`crates/testkit/src/test_app.rs` (store accessors, `mark_published(session, seq)` helper), `crates/testkit/tests/p01_storage_identity.rs` (new).

## Core mechanisms (unit tests allowed)
- `mechanisms::canon::canon(path: &Path) -> Result<String>` (MC-1 steps 1–4).
- `mechanisms::hash::sha256_file(path) -> Result<String>` (MC-3; `O_RDONLY`, streamed).
- `mechanisms::naming::folder_key(sku: &str) -> Result<String>` (MC-8).
- `mechanisms::time::now_ms(clock)` and the `*_at` write helpers (MC-4 storage).

## Ordered tasks
1. `002_domain.sql`: all tables of §6.5 except `settings`/`capture_events`; insert the Unassigned session
   (`id = 1`, `item_id NULL`). `Store` trait: a transaction closure API (shape delegated), with the row operations
   this phase needs (items, item_links, sessions, photos insert-or-get by `(session_id, raw_path)`, raw_state
   upsert) — later phases extend the trait and the impl together.
2. `canon`: absolute check; parent resolved via `std::fs::canonicalize` on the parent **plus** on-disk spelling of the
   file-name component by matching the parent's directory listing case-insensitively (MC-1 step 4); file name not
   followed through a symlink; NFC; no trailing slash.
3. `sha256_file`; `IntegrityCheck::verify(photo)` → `ok | raw_changed | raw_missing` written to `photos.integrity`,
   never touching `raw_hash`.
4. `folder_key` + collision rule via `items.folder_key_ci` UNIQUE (refusal returns `sku_collision{existing_sku}`).
5. `ItemsService::create_local(sku, title, item_type, barcode)` (origin `local`, creates the item's session),
   `link(item_id, integration, external_id, external_url)`, `update_identity(item_id, sku?, barcode?)` (05B MC-22:
   SKU change always allowed; `items.folder_key` follows the SKU only while the item has no photo row, then stays
   frozen; the new SKU's key, ignoring case, must not equal another item's stored key or another item's current-SKU
   key → `sku_collision`). `SessionsService::create`,
   `get` (DTO with counts), `unassigned()`.
6. Commands + bindings; `TestApp` accessors; tests for every row below.

## Criteria (7 criteria, 20 rows; 2 `[rule]` rows: C1(a), C6(a))
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | `App::open` then `PRAGMA table_info` per table | fresh data dir | every table of master plan §6.5 exists with every listed column | `[rule]` ch05 §tables (+ inventory columns) |
| C1(b) | `SessionsService::unassigned()` | fresh data dir | returns session `id = 1`, `item_id = None`; exactly one row with `item_id IS NULL` | MC-2 (Unassigned bucket) |
| C2(a) | `canon` | `<tmp>/caf\u{e9}.ARW` created; input spelled NFD (`e` + U+0301) | same string as the NFC input; result is NFC | M8 |
| C2(b) | `canon` | `/tmp/<d>/x.ARW` vs `/private/tmp/<d>/x.ARW` (file exists) | identical strings | M8 |
| C2(c) | `canon` | relative path `Capture/x.ARW` | `Err(relative_path)` | MC-1 step 1 |
| C2(d) | `canon` | file created as `Foo.ARW`; input `…/foo.arw` | `…/Foo.ARW` (test skips with an explanatory message if the temp volume is case-sensitive, i.e. both spellings exist as distinct files) | M8 |
| C2(e) | `canon` | `<tmp>/link -> <tmp>/real` (dir symlink); input `<tmp>/link/x.ARW` where `x.ARW` is itself a symlink to `y.ARW` | `<tmp>/real/x.ARW` (parent resolved, file name kept) | MC-1 step 2 |
| C2(f) | `canon` | `<tmp>/a/../b/x.ARW/` | `<tmp>/b/x.ARW` | MC-1 steps 2, 4 |
| C3(a) | `Store` photo insert-or-get | two inserts for session 2 with NFD and NFC spellings of one file | second call returns the first row's id; `photos` has one row for that session; `raw_path` is canonical | M8 |
| C4(a) | `sha256_file` | 5 MiB random file | 64 lowercase hex equal to `sha2` over the bytes; file mtime and bytes unchanged afterwards | M10, H2 |
| C4(b) | `IntegrityCheck::verify` | photo whose file was appended to after hashing | `integrity = raw_changed`; `raw_hash` equals the original hash | M10 |
| C4(c) | `IntegrityCheck::verify` | photo whose file was removed | `integrity = raw_missing`; `raw_hash` unchanged | M10 |
| C4(d) | `IntegrityCheck::verify` | unchanged file | `integrity = ok` | M10 |
| C5(a) | `folder_key` | `"Ab c/é.1"` | `"Ab_c__.1"` | MC-8 |
| C5(b) | `folder_key` | `".hidden"`, `"_x"` | `"-hidden"`, `"-x"` | MC-8 |
| C5(c) | `folder_key` | `""`, 101 × `"a"` | `Err(invalid_value)` for both; 100 × `"a"` accepted | MC-8 |
| C5(d) | `ItemsService::create_local` | items `abc` then `ABC`; then `abd` | second refused `sku_collision{existing_sku: "abc"}`, no row; `abd` created | M15 (no two items share a folder) |
| C6(a) | `ItemsService::create_local` | sku, title, type | item `origin = local`; exactly one session with that `item_id`; `created_at = clock.now_ms()` | `[rule]` ch05 §items (origin), ch04 S-D |
| C6(b) | `ItemsService::update_identity` | item with no photos → sku `B`; then a photo row inserted → sku `C`; another item `X` exists | first call: `sku = B`, `folder_key = B`; second call: `sku = C`, `folder_key` still `B`; then sku `b` on item `X` → `Err(sku_collision{existing_sku})` (collides with stored key `B`) | MC-22 (05B), M29 |
| C7(a) | `ItemsService::create_local` with `ManualClock` at `1_761_400_000_123` | — | `items.created_at`, `sessions.created_at` and `sessions.last_activity_at` all equal `1_761_400_000_123` (integer ms) | M11 (storage) |

## Delegated choices
`Store` trait shape (closure-based transactions vs. explicit `Tx` object), DTO field naming beyond §6.6,
whether `canon` uses `fcntl(F_GETPATH)` or `fs::canonicalize` for the parent (either satisfies MC-1 step 2 on
macOS), index set beyond the UNIQUE constraints, how the test checks volume case-sensitivity.

## Manual try-out
1. Run the app (`npm run tauri -- dev`). Nothing visible changes yet beyond Health.
2. `sqlite3 ~/Library/Application\ Support/com.loorenz.sessionmanager/session_manager.db '.tables'` lists the
   tables; `select id, item_id from sessions;` shows the Unassigned row `1|`.

## Notes
- `raw_state` rows are created lazily (first arrival, P05); this phase only creates the table.
- `canon` C2(d) depends on a case-insensitive volume (macOS default); the skip is a declared non-result, not a pass.
- Estimated size: ~14 files, ~1 400 lines.

## Review log
- 2026-10-25 implement r1, codex: Added `002_domain.sql` with the Unassigned row and all registered domain tables; transactional Store operations; path canonicalization, streamed read-only SHA-256 and persisted integrity checks; SKU folder key and collision handling; local item, link and session services; four desktop commands and generated bindings. The 20 criterion rows have one boundary test each. Assumptions: `Store::transaction` takes an object-safe closure over `StoreTx`; item identity edits with `barcode: None` retain the stored barcode; the desktop exposes SQLite integer IDs as TypeScript numbers, adequate for this single-operator data set. Deviations: module registration files, `Cargo.lock`, and package manifests were needed to wire the listed files; `crates/core/src/services/integrity.rs` houses the plan's `IntegrityCheck`; `crates/testkit/tests/p00_spike.rs` now compares migration versions across opens so adding migration 002 does not break P00's no-repeat-migration rule. Did not add the suggested `mark_published` testkit helper because no P01 criterion calls it and master plan §7.5 forbids unused helpers. No camera or Capture One live probe was needed for the storage boundary. Final `cargo test --workspace`, clippy, fmt check, npm typecheck and npm test all exited 0.
