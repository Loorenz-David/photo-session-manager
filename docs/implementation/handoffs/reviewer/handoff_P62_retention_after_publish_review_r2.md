---
plan: P62_retention_after_publish
role: review
round: 2
verdict: CHANGES_REQUESTED
state: DONE
date: 2026-11-10
actor: opus (pipeline-reviewer)
---
# Re-review: P62 retention after publish (round 2, after fix 1)

**Verdict: CHANGES_REQUESTED, with one MUST item (F2).** F1 is resolved as its correction asked. Clearing now resumes after a restart, and the new test bites. The fix also opened a new path that deletes RAWs. The resume pass re-lists every photo of a cleared item's session, including photos that arrived **after** the item was cleared. Opening an archived item in the app makes it the capture target, so a capture or import into it is a realistic case. That RAW was never published and never verified against Shopify, and the item is now dirty. The pass deletes it permanently within about 5 minutes. A reviewer probe (test code only, deleted afterwards) confirmed this. It violates H2 and M41, and the triage classifies it MUST. The runner stops here and the orchestrator decides (flow.md §5).

## ⚠ OWNER DECISIONS REQUIRED (0)
None. F2 is a production defect. The book answers it: M41 and 10F §4 say nothing of a dirty or unpublished item is ever deleted. It needs no product decision.

## Review history
- r1 (`5b82ffe`): every row C1–C8 was verified correct at its boundary. The deletion gate was verified on every call site, and assumptions 1–13 were consistent with the book. One blocker, F1 (clearing not resumable, H4). Seven notes went to the backlog.
- This round (fix checkpoint `2aab879`): delta-scoped. Settled areas were not re-verified. The changed seam is `StorageService::queue_cleared_leftovers`, how `scan` calls it, and test `c1c`.

## Perimeter check
- Gate: the tracker row reads `IMPLEMENTED` with note "fix 1: F1 resolved …". Checkpoint `2aab87948880cbbcd4677855301b6f07f2ac9564` is in `git log`.
- `git show --stat 2aab879`: `crates/core/src/services/storage.rs` (+49/−2), `crates/testkit/tests/p62_retention_after_publish.rs` (+40), `docs/implementation/master_plan.md` (P62 row), and the plan's Review log. Both code paths are on F1's allowed list. No migration 011 was needed, so no infra or port file changed. The two docs are the fix role's closing writes. **Nothing outside the perimeter.**
- `git diff 2aab879 HEAD`: only the runner's pipeline docs (fix handoff, re-review prompt). The working tree was clean at start (`git status --porcelain` empty).

## Stamp (reviewer run; code tree `2aab879`, HEAD `48c0b11` adds only pipeline docs)
| Command | Result |
|---|---|
| `cargo test --workspace --no-fail-fast` | 898 passed, 0 failed. The workspace run hit the 30-minute background limit inside `p50` after 808 passed and 0 failed. `p50` and the 11 test files after it were rerun as `cargo test -p psm-testkit --test …`: 90 passed, 0 failed (P62 file 10/10). Doc tests clean. |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean |
| `cargo fmt --all -- --check` | clean |
| `npm run typecheck` (apps/desktop) | clean |
| `npx vitest run --maxWorkers=4` (apps/desktop) | 62 files, 431/431 |

## Findings resolved
- **F1 (B3: H4): resolved as the correction said.**
  - `scan` replaces `work.queue` with levels 1–3. It then calls `queue_cleared_leftovers`, before `retention_scan`.
  - For each session in `retention::cleared_sessions`, that call queues `item_files(item_id, session_id, &cleared)` as `QueuedKind::Item`. So the per-file `allowed_item_file` guard and P61's size/mtime check still apply at delete time.
  - The rule is `retention`, which F1's correction allowed. No persistence was needed.
  - The throttle is `Work.leftovers_at`. Because `Work` lives in memory, it is `None` after a restart, so the pass runs at the first scan, then at most every `REFRESH_EVERY` (5 min).
  - `scan` runs only when the queue is empty and replaces it, so files are never queued twice.
  - The `(self.pause)()` check in `tick` precedes any scan or batch, so C7's pause also holds for the leftovers.
  - A file skipped by P61's size/mtime check (e.g. a `.cos` that Capture One rewrites) is re-read with its fresh entry at the next pass and deleted. This closes F1's second half.
  - The shared-RAW rule is unchanged (`item_files` with the cleared set), so a RAW still used by an uncleared session is kept.
  - No scan-request loop: the pass sets no `scan_requested`.
- **Test `c1c`** clears until the first batch is deleted. It then asserts `cleared_at` is set and `0 < left < all`, runs `kill_and_restart`, then ticks 40 times, and asserts that every file of the item is gone and the other item's files are intact. It would fail on the pre-fix tree, since r1's probe of the same scenario left 27 of 47 files. I did not run it there; flow §4 has no mutation regime. It passes on this tree. It observes only files on disk, so it passes the over-engineering test (flow §4).

## Remaining findings

### F2 (blocking, B3: H2; B1 vs M41 and 10F §4) · route: production · **MUST**
**What is wrong.** `queue_cleared_leftovers` → `item_files` lists `photos_in_session(session_id)`, which is every photo now in the cleared item's session. That includes photos that arrived after the item was cleared. `allowed_item_file` accepts the same set (`cleared_at` set and the path is one of the session's RAWs). Nothing else stops new photos from entering the session:
- `ArrivalService::accept` and `PhotosService::import` do not check `cleared_at` (r1 backlog note "a capture or import into a cleared item is stranded").
- The UI activates a session whenever its workspace opens (`App.tsx` `useCaptureTarget` → `session_activate`), including an archived item's.

So: the photographer opens an archived item, then shoots or imports. The new RAW joins the cleared session. `arrival_insert` bumps `change_seq`, so the item becomes **dirty**. Within 5 minutes the leftover pass deletes the RAW, which was never published, never verified, and never shown (the workspace shows the archived view). Before the fix, such a RAW was only stranded. Now it is destroyed.

**Evidence (reviewer probe).** A test-only file `crates/testkit/tests/zz_reviewer_probe_p62_r2.rs` was created, run and deleted, and the tree was clean afterwards. Steps: publish item PROBE (2 main, 1 condition), clock +4 d+1 min, `clear_all`, then set the active session to PROBE's and run `PhotosService::import` of `outside_new.ARW`. Output: `dirty after import = true`. Clock +6 min, then `clear_all`. Output: `cleared after import = [".../Capture/PROBE/outside_new.ARW"]`, `new raw still on disk = false`.

**Authority.**
- H2 (01_objective_and_constraints): the exception covers only *published, clean* items N days after publish.
- M41 (10F Invariant): "No file of an item that is not published-and-clean is ever deleted by storage rules, and no RAW is deleted before Shopify has been verified to hold every published image of its item."
- 10F §4: "never removes anything of an item that is not published (or is dirty …)".

**Triage: MUST.** The user-visible outcome is wrong (a fresh capture is permanently lost), and it breaks hard constraint H2.

**Correction (smallest, storage.rs only).** The leftover pass, and `allowed_item_file` for `QueuedKind::Item`, consider only the photos that existed when the item was cleared: `photo.created_at <= item_retention.cleared_at`. A RAW (and its Capture One files, caches and hash-matched duplicates) that arrived later is never queued and never allowed. Equivalent alternative, outside F1's file list: refuse arrivals and imports into a cleared session, or route them to Unassigned (this would also resolve the r1 "stranded" backlog note). That choice belongs to the orchestrator.

**Test.** One behaviour test at the storage boundary, the probe's scenario: a RAW imported into a cleared item after clearing is still on disk after the leftover interval. The item's original files are gone.

**Files the fix may touch.** `crates/core/src/services/storage.rs`; `crates/testkit/tests/p62_retention_after_publish.rs`. For the alternative: `crates/core/src/services/arrival.rs` and/or `crates/core/src/services/photos.rs`.

### Notes (to backlog, never a fix cycle)
- **N1.** `queue_cleared_leftovers`' bytes are added to `free_after` on every pass. If a leftover file keeps failing deletion (P61's size/mtime check, or a permission error), its bytes are counted as freed every 5 min. That can suppress "Disk almost full" while the bytes still sit on disk. The effect is small, since such files are few. DROP class.

## Triage (flow.md §5)
| Item | Class | Why |
|---|---|---|
| F1 | resolved | Correction applied as written; `c1c` bites. |
| F2 | **MUST** | A user-visible wrong outcome: an unpublished, never-verified RAW is permanently deleted. Hard constraint H2, invariant M41. |
| N1 | DROP → backlog | Warning estimate only; speculative robustness. |

## Lessons for the plans
- A "resume" pass that re-derives the work list from current state must re-derive it **as of the decision**, not as of now. The item's photo set can grow after the gate (`cleared_at`) is written. r1's F1 correction ("queue the files still on disk of items already marked cleared, from `item_files`") did not say that, and the fix followed it literally. The lesson belongs to the reviewer's correction, too.
- Once a destructive rule exists, the r1 "arrival into a cleared item" note stops being cosmetic. A note whose consequence depends on a later deletion path should be re-read whenever that path changes.

## Write perimeter of this session
- `docs/implementation/handoffs/reviewer/handoff_P62_retention_after_publish_review_r2.md` (this file)
- `docs/implementation/plans/P62_retention_after_publish.md` (Review log entry appended)
- `docs/implementation/master_plan.md` (P62 tracker row only → `CHANGES_REQUESTED`)
- `docs/implementation/backlog.md` (N1 appended)
- Probe: `crates/testkit/tests/zz_reviewer_probe_p62_r2.rs` was created, run twice, then deleted. `git status --porcelain` was empty before the probe and after its removal. No production file touched. Logs in `/tmp/p62_r2_test.log` (outside the repo). No narrative: one is written only on approval.
- Nothing committed.
