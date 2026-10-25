---
phase: P02_review_order
title: Review, order, undo, move/copy and the change sequence
difficulty: normal
projection: no
depends_on: [P01_storage_identity]
---
# P02 — Review, order, undo, move/copy and the change sequence

## Goal
Implement the photographer's review operations as one `ReviewService` over the real Store: classify, reorder,
delete, undo, move and copy (MC-5 position math, MC-2 move/copy refusals), with the per-session change sequence
and derived dirty flag (MC-6) and `last_activity_at` (MC-4). **Not in this phase:** status derivation and list
(P03), colour-tag mirror jobs (P04), capture arrival (P05), any UI (P08), attribute edits (P10 carries the
"attribute edit does not bump change_seq" row).

## Read first
master plan §6.5 (`sessions`, `photos`), §6.6, §6.7 (P02 commands) · book `01`, `14`, `06` (Classification, Order,
Unassigned/move/copy) · contracts `05A` MC-5 (operations table), MC-6, MC-4 (`last_activity_at` bullet), MC-2
§Move and copy · `07` §Dirty.

## Files expected to change
`crates/core/src/mechanisms/position.rs` (new), `crates/core/src/services/review.rs` (new),
`crates/core/src/domain/photo.rs` (undo slot type), `crates/core/src/ports/store.rs` + `crates/infra/src/sqlite/store.rs`
(photo position/classification updates, change_seq increment, undo slot), `crates/core/src/error.rs`,
`apps/desktop/src-tauri/src/commands/photos.rs` (new: `photos_classify`, `photos_reorder`, `photos_delete`,
`photos_undo`, `photos_move`, `photos_copy`), bindings, `crates/testkit/src/test_app.rs` (`add_photo(session, raw)`
fixture inserting a photo row directly; `mark_published`), `crates/testkit/tests/p02_review_order.rs` (new).

## Core mechanisms (unit tests allowed)
- `mechanisms::position::{classify_into, remove_at, reorder, undo_insert}` — pure functions over a `Vec<(photo_id, position)>` per group (MC-5 table rows).

## Ordered tasks
1. `position.rs` pure functions; one SQLite transaction per operation in `ReviewService`, each ending with
   `change_seq += 1` and `last_activity_at = now_ms` on every session touched (source and target for move/copy).
2. `classify(photo, group)`, `reorder(photo, j)`, `delete(photo)` (stores the undo slot `{photo_id, prior_classification,
   prior_position}` on the session, replacing any previous), `undo(session) -> UndoResult{active_photo_id}`,
   `move_photos(ids, target_session)`, `copy_photos(ids, target_session)` with the refusals `target_has_raw`,
   `target_published` (target `published_at` set and not dirty); moved/copied sets appended in source order
   (group, position, captured_at, id).
3. `dirty(session) -> bool` = `published_at.is_some() && change_seq > published_seq` exposed on the session DTO.
4. Commands + bindings; tests.

## Criteria (6 criteria, 25 rows; 0 `[rule]` rows)
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | `classify(p, Main)` | main = [a,b], p unreviewed | p `main` at position 3; a,b unchanged | M12 |
| C1(b) | `classify(p, Condition)` | condition = [], p unreviewed | p `condition` position 1 | M12 |
| C1(c) | `classify(b, Condition)` | main = [a,b,c], condition = [d] | main = [a(1), c(2)]; condition = [d(1), b(2)] | M12 |
| C1(d) | `classify(d, Main)` | main = [a], condition = [d,e] | main = [a(1), d(2)]; condition = [e(1)] | M12 |
| C1(e) | `classify(a, Main)` | a already main at 1 of [a,b] | positions unchanged; `change_seq` unchanged | M12 |
| C2(a) | `reorder(a, 3)` | main = [a,b,c,d] | [b(1), c(2), a(3), d(4)] | M12 |
| C2(b) | `reorder(d, 2)` | main = [a,b,c,d] | [a(1), d(2), b(3), c(4)] | M12 |
| C2(c) | `reorder(a, 9)` | main = [a,b,c] | [b(1), c(2), a(3)] (clamped to n) | M12 |
| C2(d) | `reorder(c, 0)` | main = [a,b,c] | [c(1), a(2), b(3)] (clamped to 1) | M12 |
| C3(a) | `delete(b)` | main = [a,b,c] | b `deleted`, position null; main = [a(1), c(2)] | M12 |
| C3(b) | `undo` after C3(a) | — | b `main` at 2; main = [a(1), b(2), c(3)]; returns `active_photo_id = b` | M12 |
| C3(c) | `delete(c)` of main [a,b,c], then `delete(a)`, `delete(b)` (slot now b), `undo` | — | only b restored: main = [b(1)]; a and c stay deleted | M12 |
| C3(d) | main = [a,b,c]: `delete(c)` (slot k = 3), then `classify(a, Condition)`, `classify(b, Condition)` (main now empty), then `undo` | slot k = 3, n_g = 0 | c inserted at `min(3, 0+1) = 1`; main = [c(1)]; condition = [a(1), b(2)] | M12 (clamp) |
| C3(e) | `undo` with empty slot | fresh session | `Err(nothing_to_undo)`; nothing changes | MC-5 |
| C3(f) | `delete(u)` then `undo` | u unreviewed | u `unreviewed`, position null; active = u | M12 |
| C4(a) | `move_photos([b, a], T)` | source main = [a,b,c]; T main = [x] | T main = [x(1), a(2), b(3)] (source order); source main = [c(1)] | MC-2 §Move, M12 |
| C4(b) | `copy_photos([a], T)` | a main with `raw_hash = h` | T has a new row on the same `raw_path`, `main`, appended, `raw_hash = h`; source row unchanged | MC-2 §Copy, MC-3 (copy shares hash) |
| C4(c) | `move_photos([u], T)` | u unreviewed | u in T, `unreviewed`, position null | MC-5 (unreviewed stays null) |
| C4(d) | `move_photos([a], T)` | T already has a row on a's RAW | `Err(target_has_raw)`; both sessions unchanged | MC-2 (R-I12) |
| C4(e) | `move_photos([a], T)` | T `published_at` set, not dirty | `Err(target_published)`; unchanged | MC-2 (published not a target) |
| C4(f) | `move_photos([a], T)` | source `published_at` set, not dirty; T open | a moved; source `dirty = true` | MC-2 (R-I3), M13 |
| C5(a) | each of `classify`, `reorder`, `delete`, `undo`, `move_photos` (source and target), `copy_photos` (target) | `change_seq` before = n | `change_seq` after = n + 1 on every session touched (one assertion per operation, 7 sub-checks) | M13 |
| C5(b) | `dirty` | `published_at` set, `published_seq = 5`, then one `classify` | `dirty = true` | M13 |
| C5(c) | `dirty` | `published_at` set, `published_seq = change_seq` | `dirty = false`; `published_at` null → `dirty = false` regardless of `change_seq` | M13 |
| C6(a) | any operation with `ManualClock` at `t` | — | `last_activity_at = t` on every session touched | MC-4 (`last_activity_at`) |

## Delegated choices
Whether `undo_slot` is stored as JSON text or three nullable columns (plan says JSON; columns acceptable if the
migration already shipped — it has not), the DTO returned by each command (must include the affected sessions'
`change_seq` and `dirty`), batching of `move_photos` per source session.

## Manual try-out
No UI yet. From the dev app's Health page nothing changes; the owner can wait for P08 to try review by hand.
(Optional: the reviewer may demonstrate with `sqlite3` after running the P02 tests, which leave no data behind.)

## Notes
- Deleting never touches the RAW (H2): `delete` changes `classification` only.
- `copy` into a session already holding the RAW is refused like `move` (R-I12), covered by C4(d)'s rule; a copy row
  carries the same `raw_hash` without re-hashing (C4(b)).
- Estimated size: ~8 files, ~1 100 lines.

## Review log
(empty)
