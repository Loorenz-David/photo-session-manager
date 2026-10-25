---
plan: P01_storage_identity
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-10-25
actor: opus
---
# Review P01_storage_identity, round 1: APPROVED_WITH_NOTES

All 20 criterion rows hold at their boundaries, and each test would fail on a wrong outcome. Suite,
clippy, fmt, typecheck and the frontend test command are green. No B1–B4 finding. Five notes are in
the backlog. The narrative is written.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Gate
- Tracker row `P01_storage_identity` = `IMPLEMENTED` (2026-10-25, codex). Checkpoint `af9d93e90efc37783d5c2131de020afe176bf53c` exists in `git log`.
- Tree at review: HEAD `8342253`, `git status --porcelain` empty before and after the stamp. The delta from the checkpoint is two runner documents (implement handoff, review prompt).

## Verified correct
**Perimeter.** `git show --stat af9d93e` lists 31 paths. They equal the handoff's "Files changed" list exactly. The paths beyond the plan's list (`Cargo.lock`, `commands/mod.rs`, `src-tauri/src/lib.rs`, `domain/mod.rs`, `core/src/lib.rs`, `services/mod.rs`, `services/integrity.rs`, two `Cargo.toml`s, `tests/p00_spike.rs`, plus the tracker and Review log) are declared under Deviations, and each one is wiring or the registered home of a planned symbol. The P00 test edit replaces `migration_version == 1` with "equal to the first open's version". That keeps the row's meaning (no repeated migration) and no longer has to change every time a migration is added.

**Stamp (L4, tree `8342253`, clean; cargo from `~/.cargo/bin`, Node 22 via the §7.1 PATH prefix).**
| Command | Result |
|---|---|
| `cargo test --workspace` | exit 0: `p01_storage_identity` 20/20, `p00_spike` 16/16, 3 integrations/infra unit tests, `export_bindings`. Tree still clean afterwards, so the committed `bindings.ts` equals the regenerated one. |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo fmt --all -- --check` | exit 0 |
| `npm run typecheck` (apps/desktop) | exit 0 |
| `npm test` (apps/desktop) | exit 0 |
An earlier attempt in a shell without `~/.cargo/bin` on PATH exited 127 (command not found) before compiling anything. That was environmental, not a code failure.

**Per row (code read, then test read).**
| Row | Production that makes it true | Test observes the outcome at the boundary and can fail? |
|---|---|---|
| C1(a) | `002_domain.sql` creates the 14 §6.5 tables with exactly the registered columns, in order | Yes: exact table set (excluding sqlite/migration internals) and exact column list per table via `PRAGMA table_info` |
| C1(b) | Migration inserts `sessions(1, NULL, …)`; partial unique index `sessions_one_unassigned` keeps it single; `unassigned()` = `get(1)` | Yes: id 1, `item_id None`, `COUNT(item_id IS NULL) = 1` |
| C2(a) | `canon` NFC-normalizes the name, matches the directory listing in NFC, NFC-normalizes the whole string | Yes: NFD input equals NFC input, result is NFC. An un-normalizing `canon` differs. |
| C2(b) | Parent via `fs::canonicalize` (`realpath`) → `/private/tmp/…` | Yes |
| C2(c) | `!path.is_absolute()` → `Error::RelativePath` | Yes (matches the variant) |
| C2(d) | Exact listing match first, else case-insensitive match → on-disk spelling | Yes. It ran rather than skipped: no skip line under `--nocapture`, and `$TMPDIR` is case-insensitive APFS. |
| C2(e) | Only the parent is canonicalized, so the file-name symlink is kept | Yes: expects `real/x.ARW`. Following the file link gives `y.ARW`, and an unresolved parent gives `link/`. |
| C2(f) | `canonicalize` removes `..`, `Path::file_name` ignores the trailing slash, `trim_end_matches('/')` | Yes. It compares against `canon` of the clean path, and both a kept `..` and a kept slash would differ. |
| C3(a) | `photo_insert_or_get` canonicalizes before the `(session_id, raw_path)` lookup inside one transaction | Yes: same id, canonical `raw_path`, `photo_count = 1`. Without canonicalization the NFD and NFC strings give two rows. |
| C4(a) | `File::open` (O_RDONLY), 64 KiB streamed chunks, lowercase hex | Yes: equals `sha2` over the bytes, 64 chars, bytes and mtime unchanged |
| C4(b–d) | `IntegrityCheck::verify` hashes, compares, writes only `integrity`; `NotFound` → `raw_missing` | Yes: returned and stored integrity, and `raw_hash` still equals the original |
| C5(a–c) | `folder_key`: NFC, map outside `[A-Za-z0-9._-]` to `_`, leading `.`/`_` → `-`, 1–100 | Yes: exact strings and `InvalidValue` for `""` and 101 chars, 100 accepted |
| C5(d) | `collision()` inside the create transaction, backed by `folder_key_ci UNIQUE` | Yes: `existing_sku == "abc"`, item count 2, next create succeeds |
| C6(a) | `create_local` inserts the item (`origin 'local'`) and one session in one transaction at `clock.now_ms()` | Yes |
| C6(b) | `update_identity`: key follows the SKU while `item_has_photos` is false, then frozen; collision on exact SKU, stored key, or current-SKU key (case-insensitive) | Yes. The final `b` on X can only collide through A's *stored* key `B` (A's SKU is `C`), and without the check the DB UNIQUE error is `InvalidData`, not `SkuCollision`. So the assertion discriminates. |
| C7(a) | One `at` written to `items.created_at`, `sessions.created_at`, `sessions.last_activity_at` | Yes: all equal `1_761_400_000_123` |

**Contract checks beyond the rows.** MC-1 step 2 (every parent component in on-disk spelling): probed `realpath` on a mis-cased parent on this Mac, and it returns the on-disk case (`…/mixedcase/SUB` → `…/MixedCase/Sub`). This probe was outside the repo, and its temp dir is under `$TMPDIR`. MC-3 "copies share the hash": `photo_insert_or_get` copies an existing `raw_hash` for the same canonical path. MC-6: adding a photo bumps `change_seq` in the same transaction. MC-4: `last_activity_at` is set on photo add. MC-22: SKU editable at any time, key frozen from the first photo row in any of the item's sessions.

**Assumptions.** None contradicts the book. (1) The closure-based `Store::transaction` is a delegated choice. (2) `barcode: None` keeps the stored barcode. The plan gives an optional input and the book has no clearing action. Reported to the owner. (3) i64 ids as TS numbers is safe at this scale. (4) Photo creation time as session activity time is consistent with MC-4 ("set to now by any photo change").

**Hard constraints.** H1 has no external write. H2: RAWs are only opened read-only and the directory listing is read; nothing writes, moves or deletes them. H3: no pixels touched. H4: insert-or-get and link upsert are idempotent, and create runs in one transaction. H5 is not touched. H6 holds: no integration is needed. H7: no platform terms in `crates/core` or migration 002 (grep), and external ids live only in `item_links`/`photo_links`. H8: the four commands only lock state, call a service and map the DTO/error. No secrets, and no live services in tests.

## Findings
None blocking.

## Notes written to backlog
1. **P01 → P05, P13, P16.** `IntegrityCheck::verify` reports `raw_changed` for a photo with no `raw_hash` yet. The callers must hash first or treat a missing hash as pending.
2. **P01 → P03.** Unassigned is stored with `created_at = last_activity_at = 0`, so it would land in a 1970 month group unless it is placed separately.
3. **P01 → master plan §6.6.** `CommandError` codes `not_found`, `io`, `invalid_data`, `provider` are not in the registry.
4. **P01 → P07.** `items.title`/`item_type` are `NOT NULL` and required by `create_local`, while ch05 requires only `sku`.
5. **P01 → P02/P03.** The `mark_published` testkit helper is deferred to its first caller.

## Lessons for the plans
- The plan listed `IntegrityCheck` without a file. The implementer chose `services/integrity.rs`. Future plans should name the home of every listed symbol, or the module-registration files, so that wiring edits are not "outside the list".
- The plan's file list omitted registration files (`mod.rs`, `lib.rs`) and manifests every new module needs. Listing them, or a standing master-plan line allowing them, removes a recurring deviation.

## Carry-forward dispositions
| Note | Destination |
|---|---|
| 1 hash-less verify | P05 (hash job), P13 (prepare), P16 (health) |
| 2 Unassigned at instant 0 | P03 |
| 3 error-code registry | master plan §6.6 before P07 |
| 4 title/type required | P07 |
| 5 `mark_published` | P02 or P03, whichever first needs a published session |

## Write perimeter of this session
- `docs/implementation/master_plan.md`: tracker row P01 only (`IMPLEMENTED` → `APPROVED`).
- `docs/implementation/plans/P01_storage_identity.md`: one Review log entry appended.
- `docs/implementation/backlog.md`: five rows appended.
- `docs/implementation/narratives/P01_storage_identity.md`: new.
- `docs/implementation/handoffs/reviewer/handoff_P01_storage_identity_review_r1.md`: this file.
- No production or test file touched; no probe applied to the repo. Build artefacts under `apps/desktop/src-tauri/target/` (ignored) were refreshed by the stamp. One scratch temp dir under `$TMPDIR` (outside the repo) for the `realpath` probe. Nothing committed.
