---
plan: P02_review_order
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-10-25
actor: opus
---
# P02_review_order — review r1

**Verdict: APPROVED_WITH_NOTES.** The checkpoint matches its declared files. The suite, lint and typecheck
are green. All 25 criterion rows hold at their boundaries, and each test can fail on a wrong outcome. No
assumption contradicts the intention. H1–H8 hold. There are no B1–B4 findings. I wrote 3 non-blocking
notes to the backlog.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Verified correct
- **Gate.** The tracker showed `IMPLEMENTED`. Checkpoint `b167b76` (`CHECKPOINT (not approved): implement P02 review and order`) is in `git log`.
- **Perimeter.** `git show --stat b167b76` lists 19 paths, and they equal the handoff's "Files changed"
  list exactly. Five paths are outside the plan's list: `commands/{items,mod,sessions}.rs`, `lib.rs` and
  `domain/session.rs`. The handoff declares them under Deviations. Each one is needed wiring: command
  registration, error-code mapping, and `dirty` on the session DTO/response. None adds behaviour from
  another phase.
- **Stamp** (tree = `b167b76` plus the runner's prompt/handoff commits, which touch docs only):
  - `cargo test --workspace` exited 0. P02 passed 25/25, P01 20, P00 16, and the unit tests 5, including `mc5_position_operations`.
  - `cargo clippy --workspace --all-targets -- -D warnings` exited 0. `cargo fmt --all -- --check` exited 0.
  - `npm run typecheck` exited 0. `npm test` exited 0 (no frontend test files).
  - The tree was clean after the run, so the regenerated `bindings.ts` equals the committed file (§6.7).
- **Rows, read against production and test:**
  - **C1(a–e) classify.**
    - Code: `ReviewService::classify` uses `position::remove_at` to close the old group's gap. It puts the
      photo at `n_g + 1`, and does nothing for the same classification (no seq bump).
    - Tests: each one asserts the full `(id, position)` list per group from stored state. C1(e) also asserts `change_seq` is unchanged.
  - **C2(a–d) reorder.**
    - Code: `position::reorder` clamps to `1..n_g`. A no-op reorder writes nothing.
    - Tests: they assert the full order after a move forward, a move backward, a clamp high and a clamp low.
  - **C3(a–f) delete/undo.**
    - Code: delete sets `deleted` and `position = null`, and replaces the single JSON slot `{photo_id,
      prior_classification, prior_position}`. Undo inserts at `min(k, n_g+1)` with `undo_insert`, sets
      `active_photo_id`, and clears the slot. The slot is refused (`nothing_to_undo`) if the photo is no
      longer deleted in that session.
    - Tests: C3(c) checks that only the last delete is restored. C3(d) checks the clamp after the group empties.
      C3(e) compares the whole session DTO before and after. C3(f) checks unreviewed stays null.
  - **C4(a–f) move/copy.** `transfer` checks, before any write:
    - the target is published and not dirty → `target_published`;
    - the target already has the RAW, or two batch rows share a RAW → `target_has_raw`.
    - It then sorts by (group, position, captured_at, id) and appends per group.
    - Copy inserts a new row with the same `raw_path`/`raw_hash` and leaves the source untouched.
    - Tests: C4(b) compares the full source `Photo`. C4(d) and C4(e) compare both photos and both seqs before and after.
  - **C5(a–c) change_seq/dirty.**
    - Code: `touch_session` (`change_seq+1`, `last_activity_at`) runs inside the operation's transaction,
      once per touched session (a BTreeSet for move sources). `dirty = published_at.is_some() && change_seq
      > published_seq` is derived in `session_row`.
    - Tests: C5(a) checks the seven sub-checks with exact `n+1`, and that copy leaves the source unchanged.
      C5(c) would fail if the `published_at` guard were dropped, because the unpublished session has `change_seq > 0 = published_seq`.
  - **C6(a) last_activity_at.** A move stamps source and target with ManualClock `t`. Every other operation
    goes through the same `touch_session(…, at)` call, which C5(a) proves runs once per operation and session.
- **Changed-input probes.** I ran these in a temporary test file `crates/testkit/tests/zz_p02_reviewer_probe.rs`,
  deleted it afterwards, and `git status` was clean. No production file was touched. All passed:
  - A move of mixed main and condition photos appends per group in source order, and the source closes up.
  - Undo after the deleted photo was moved to another session gives `nothing_to_undo`, with neither seq changed.
  - A copy into a clean published target is refused. After a change makes that target dirty, the copy is accepted.
  - Delete index 1, then a new photo arrives in Main, then undo: the photo goes back to 1, the others shift, and the new photo is at 4.
  - Reorder of an unreviewed photo errors and bumps nothing.
- **Hard constraints.**
  - H2: delete changes `classification` only, and the service does no file I/O.
  - H8: the six commands are thin wrappers over `ReviewService` through `with_review`.
  - H7: no provider concepts were added. Copy rows share preview paths, as 10A specifies.
  - H1, H3, H5 and H6 are not touched. No secrets and no live services.
- **Assumptions.** I checked each one against the intention. None contradicts it:
  - The JSON slot is what the plan says.
  - Same-state no-ops are consistent with MC-6 "photo change".
  - "Published target" means status `published`, that is, not dirty (MC-2, MC-7).
  - A batch with several sources bumps each source once.
  - The result lists sessions in ascending id order.

## Findings
None blocking (B1–B4: none).

## Notes written to backlog
1. **P02 → P07, P08.** The review commands emit no `session_changed`/`photo_changed` events. The sessions
   list refreshes only on those events, so it would go stale after workspace actions.
2. **P02 → P08.** A batch in which two photos share one RAW is refused as `target_has_raw`, even though the
   target has no such row. The refusal is right, but the user-facing wording needs care.
3. **P02.** A copied row keeps the source's `created_at`. Nothing orders by it today, so there is no visible effect.

## Lessons for the plans
- C6(a) says "any operation" but names no operation set. The proof leans on C5(a) to reach every
  operation. Future rows should name the operation or point to C5(a)'s set explicitly.

## Write perimeter
`docs/implementation/backlog.md` (3 rows appended) · `docs/implementation/master_plan.md` (P02 tracker row
only) · `docs/implementation/plans/P02_review_order.md` (Review log entry) ·
`docs/implementation/narratives/P02_review_order.md` (new) · this handoff (new). No production or test files
are changed. The probe file was created and deleted, and the tree was verified clean. No database state was
touched outside temporary test directories. No commits.
