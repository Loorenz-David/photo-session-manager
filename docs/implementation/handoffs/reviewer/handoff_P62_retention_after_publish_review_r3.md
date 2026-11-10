---
plan: P62_retention_after_publish
role: review
round: 3
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-10
actor: opus (pipeline-reviewer)
---
# Re-review: P62 retention after publish (round 3, after fix 2)

**Verdict: APPROVED_WITH_NOTES.** F2 is resolved, as the orchestrator's `FIX2_AUTHORISED` asked, in both parts:
- (a) Retention, the leftover pass and the per-file delete guard now consider only the photos an item had when it was cleared.
- (b) Captures and imports while a cleared item is active go to Unassigned.

Both new tests fail on a wrong outcome. The perimeter is exactly the allowed files. No blocking item remains. Four notes go to the backlog. Stamp: Rust 900 passed / 0 failed; clippy, fmt and typecheck clean.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Review history
- **r1** (`5b82ffe`): rows C1–C8 were verified at their boundaries, and the deletion gate was verified on every call site. F1 (clearing not resumable, H4) was blocking. Seven notes went to the backlog.
- **r2** (`2aab879`): F1 resolved (the leftover pass). The new F2 was MUST: the leftover pass deleted RAWs that joined a cleared session after clearing (H2, M41). Orchestrator: `FIX2_AUTHORISED`.
- **This round** (fix checkpoint `6f0a234`): delta-scoped. The changed seam is `StorageService::cleared_item_photos` and its two callers (`item_files`, `allowed_item_file`), the redirect in `ArrivalService::accept`, the folder choice in `PhotosService::import`, and tests `c1d` and `c1e`. Settled areas were not re-verified.

## Perimeter check
- **Gate.** The tracker row reads `IMPLEMENTED`, with the note "fix 2: F2 resolved …". Checkpoint `6f0a234964f67f3a0cebf5ade95011d5c1e3577e` is in `git log`.
- **`git show --stat 6f0a234`.**
  - `crates/core/src/services/storage.rs` (+16/−2)
  - `crates/core/src/services/arrival.rs` (+9)
  - `crates/core/src/services/photos.rs` (+2)
  - `crates/testkit/tests/p62_retention_after_publish.rs` (+82)
  - the fix handoff, the master plan's P62 row, and the plan's Review log

  All four code paths are on the `FIX2_AUTHORISED` list: storage.rs, plus arrival.rs/photos.rs "only as needed", plus the P62 test file. The docs are the fix role's closing writes. **Nothing outside the perimeter.**
- **`git diff 6f0a234 HEAD`.** Only pipeline docs: the fix handoff's SHA line and this round's prompt. No code changed. The working tree was clean at start.

## Stamp (reviewer run, prompt-directed; code tree `6f0a234`; HEAD `07ca5e8` adds only pipeline docs)
| Command | Result |
|---|---|
| `cargo test --workspace --no-fail-fast` | **900 passed, 0 failed** over all 56 test binaries. The workspace run hit the 90-minute background limit inside `p27` after 29 binaries: 529 passed, 0 failed. Each test binary took 5–17 min on this machine (`p46` 1008 s). The remaining 27 binaries were run as `cargo test -p psm-testkit --no-fail-fast --test …`: p27–p53 gave 312 passed, 0 failed (18 binaries, cut by the 2-hour limit inside p54). `p62_retention_after_publish` alone gave 12 passed, 0 failed. p54–p61 gave 47 passed, 0 failed (8 binaries). Each binary was run exactly once to completion, none twice. The total equals the fix handoff's 900. |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean |
| `cargo fmt --all -- --check` | clean |
| `npm run typecheck` (apps/desktop) | clean |
| `npx vitest run` | not run: no frontend file changed since r2's 431/431 (tree identity: `git diff 2aab879 6f0a234 -- apps/` is empty) |

## Findings resolved
**F2 (B3: H2; B1 vs M41 and 10F §4): resolved as the authorisation said.**

**(a) Storage.** `cleared_item_photos(item_id, session_id)` returns `photos_in_session` filtered to `created_at <= item_retention.cleared_at` once `cleared_at` is set, and all photos before that. Two callers use it:
- `item_files`, which serves both the initial clearing and `queue_cleared_leftovers`;
- `allowed_item_file` for `QueuedKind::Item`.

Because `item_files` derives each RAW's `.cos`/cache names, preview/master/output folders and duplicate hashes from that photo list, a later RAW's companions are excluded too. Checks I made:
- **Time base.** Both sides come from the same injected clock. `photos.created_at` is `self.clock.now_ms()` in `ArrivalService::accept` and the camera ledger. `cleared_at` is `self.clock.now_ms()` in `clear_item`. So the comparison is in one time base.
- **Ordering in `clear_item`.** It runs eligibility re-check → `mark_item_cleared(now)` → `item_files`. The first clearing therefore sees every photo created up to the mark. A photo arriving before the re-check makes the item dirty (`arrival_insert` bumps `change_seq`), and the item is skipped.
- **Judgment call 3** (a photo moved in by hand keeps its older `created_at`) **holds in practice.** `ReviewService::transfer` refuses a cleared target (`ensure_session_not_cleared(target)`). Undo/redo of a move refuses a published-clean destination (`history::published_clean`). A cleared item stays published-clean: attribute writes never touch `change_seq` (MC-6), and arrivals no longer join it. No path moves a photo into a cleared session.

**(b) Arrivals.** In `ArrivalService::accept`, after rule 0 / import / folder attribution have picked a session, a session in `cleared_sessions()` becomes 1 (Unassigned). That covers ledger deliveries, imports and folder attribution with one rule. `PhotosService::import` with a cleared item active copies into `_Unassigned`. Its `accept` call then redirects the same way.
- **Collision check.** Could a new capture land on an old cleared photo's path? `arrival_insert` for `Capture` would reuse that photo, with its old `created_at`. The capture ledger prevents this: a repeated camera file key gets `RenamedPhoto` (a dup suffix), and imports now go to `_Unassigned`.
- **No later re-attribution.** No code re-attributes Unassigned photos by folder later (`attribute` has one caller), so the redirect is final.

**Tests.**
- **`c1d`** clears an item, inserts a RAW plus Capture One files into its session 1 min after clearing, then advances 10 min (past the 5-min leftover interval) with free space 0, so every level is open. It asserts each file is still on disk and none is in the cleared list. Without (a), the leftover pass queues and allows that RAW. That is r2's probe outcome, so the test fails on the wrong behaviour.
- **`c1e`** sets the cleared item active, delivers a capture through the ledger and imports a file. It asserts the item's photo count is unchanged and both files are in Unassigned. Without the redirect in `accept`, both join the item, so it fails.
- Both tests observe stored state and files only, so they pass the over-engineering test (flow §4).

## Triage (flow.md §5)
| Item | Class | Why |
|---|---|---|
| F1 (r1) | resolved in r2 | not re-verified (settled) |
| F2 (r2) | **resolved** | correction applied in both parts; `c1d` and `c1e` fail on a wrong outcome |
| N2–N5 (below) | DROP → backlog | no user-visible data loss; test or placement refinements, or a pre-existing undo edge |

No MUST item. Verdict `APPROVED_WITH_NOTES`.

## Remaining findings
No blocking findings. Notes (appended to `docs/implementation/backlog.md`):
- **N2. A ledger capture while an archived item is active is filed on disk in that item's folder.**
  - The ledger picks `final_path` from the session it chose, the cleared one, before `accept` redirects the database row to Unassigned. Capture One's capture folder is also still the item's folder, set on activation.
  - Result: Unassigned lists a photo whose RAW sits in `Capture/<SKU>/`, next to the cleared item's (deleted) RAWs. Nothing deletes it: it is not in the cleared photo set, and its name differs.
  - Why it matters: in Finder or Capture One the photo looks like part of the archived item. A cleaner option is to not point Capture One at a cleared item's folder on activation.
- **N3. `c1e` does not discriminate the `photos.rs` half of (b).** With only the `accept` redirect, an import still lands in Unassigned in the database. The file would just be copied into the archived item's folder, and `c1e` does not assert the file's folder. This is a more-tests item, so DROP.
- **N4. A file that reappears at a cleared RAW's path is deleted again at every leftover pass.** For example, the photographer restores a RAW from a backup into the archived item's folder. The photo is still the cleared item's photo, created before `cleared_at`. This is consistent with the rule (the image is on Shopify), but there is no restore path. Why it matters: a restore by hand silently fails within 5 minutes.
- **N5. Undo of an earlier move can take a photo out of a cleared item (pre-existing, seen in passing).**
  - A move from X into C is recorded in X's history. If C is then published and cleared while that step is still in memory (minutes with retention 0), undo in X passes `ensure_session_not_cleared(X)`. `published_clean` checks only the destination, X.
  - Result: the photo leaves C. It arrives in X with its RAW already deleted, and C's `change_seq` is bumped.
  - No file is deleted by this, but it changes an archived item, against 10F Q3 read-only. A fix would guard both sessions of a move step.
- **r1 backlog note "a capture or import into a cleared item is stranded":** resolved by fix 2 (b). A resolution line was appended to the backlog.

## Lessons for the plans
- When a destructive pass's eligibility is decided once and acted on later, the work list must be frozen to the decision's moment. Here that is `created_at <= cleared_at`. If that is impossible, the set must be prevented from growing (read-only arrival routing). Fix 2 did both. The planner should state such a "frozen at decision time" rule in the criterion, not leave it to review.
- A redirect at the database layer (Unassigned) does not move the file. Rules about where a file belongs need a row for the on-disk location as well as the stored session (N2, N3).

## Write perimeter of this session
- `docs/implementation/handoffs/reviewer/handoff_P62_retention_after_publish_review_r3.md` (this file)
- `docs/implementation/narratives/P62_retention_after_publish.md` (new: owner narrative)
- `docs/implementation/plans/P62_retention_after_publish.md` (Review log entry appended)
- `docs/implementation/master_plan.md` (P62 tracker row only → `APPROVED`)
- `docs/implementation/backlog.md` (N2–N5 and the r1 stranded-note resolution appended)
- No probes and no production or test file touched (flow §4: no mutation regime). Logs are in `/tmp/p62_r3_*.log`, outside the repo. Nothing committed.
