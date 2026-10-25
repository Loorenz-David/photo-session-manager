---
plan: P05_capture_arrival
role: review
round: 2
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-10-25
actor: opus
---
# P05 re-review, round 2: APPROVED_WITH_NOTES

Fix 1 resolves F1 exactly as the round-1 correction said. Capture arrivals (hook, watcher, startup scan) now create a row only when no `photos` row on the canonical RAW path exists in any session. Import keeps its (session, path) rule. C7(b) now moves the known RAW out of Unassigned before the reopen, and the old code fails that test. The perimeter is clean. Suite, lint, fmt and typecheck are green. No blocking items remain, so there is nothing to triage. The 9 round-1 notes stay in the backlog, and this round adds no new ones.

**Review history.** Round 1 settled every one of the 33 rows at its boundary, the MC-2 filter, stability and attribution, MC-15 consumption, the MC-20 walker (checked against the real ARW), the hash job, import/assign, the assumptions and H1–H8. Its single finding was F1 (B1): a RAW whose row had left its folder's session was re-created at every startup scan. This round re-verifies only the seam F1 touched.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Perimeter check
- **Gate.** Tracker row P05 `IMPLEMENTED` with note "fix 1: …". Fix checkpoint `806ecaa` exists (`CHECKPOINT (not approved): fix P05 capture arrival dedupe`).
- `git show --stat 806ecaa` lists 4 paths:
  - `crates/infra/src/sqlite/store.rs` (allowed by F1);
  - `crates/testkit/tests/p05_capture_arrival.rs` (allowed by F1);
  - `docs/implementation/master_plan.md` (the implementer's own tracker row);
  - `docs/implementation/plans/P05_capture_arrival.md` (fix Review log entry).

  These match the fix handoff's declared list. The two optional files, `services/arrival.rs` and `ports/store.rs`, were not needed: the signature is unchanged.
- `git diff --stat 806ecaa 1403b5f` shows runner-only commits: the fix handoff and this re-review prompt. No code changed after the checkpoint.
- **Out-of-perimeter changes:** none.

## Evidence (one run, prompt-directed)
- **Tree.** `1403b5f`, `git status --porcelain` empty. The code is identical to `806ecaa`, so this run duplicates the fix's stamp. It was taken once because the re-review prompt directs it.
- `cargo test --workspace`: exit 0, 156 passed, 0 failed. Includes `c7b_startup_ignores_cache_and_known_raw ... ok`.
- `cargo clippy --workspace --all-targets -- -D warnings`: 0.
- `cargo fmt --all -- --check`: 0.
- `apps/desktop`: `npm run typecheck` 0 and `npm test` 0 (no frontend tests).
- **Aborted first invocation.** The first attempt exited 127 because `cargo` was not on the session's PATH, so nothing ran. The run above uses `~/.cargo/bin`.

## Findings resolved
### F1 (B1, production): RESOLVED as corrected
- **Store** (`SqliteStore::arrival_insert`). For `PhotoSource::Capture`, the existing-row lookup is `SELECT id FROM photos WHERE raw_path=?1 ORDER BY id LIMIT 1`, on the canonical path the function computes first. It runs inside the arrival transaction, under the single connection mutex. For `PhotoSource::Import` it stays `session_id=?1 AND raw_path=?2` (C8(c) unaffected). On a hit:
  - it returns `(existing, false)` and the transaction rolls back read-only;
  - no jobs are enqueued, there is no `raw_state` upsert and no `change_seq` bump;
  - `ArrivalService::accept` emits no `photo_added`.
- **Callers.** All three capture callers pass `PhotoSource::Capture`: hook (`on_hook_paths`), watcher (`tick`) and scan. Only `import` passes `Import`. Each event row (`hook` spool, `watcher`, `scan`, `import`) is still written before `accept`, as the correction required.
- **Other ways a second row could appear:**
  - No `DELETE FROM photos` exists. Deletion is the `deleted` classification, so a deleted photo keeps its row and is not resurrected by the scan.
  - `photo_insert_or_get` and `arrival_insert` both canonicalize `raw_path`, so the any-session lookup compares like with like.
  - Rows copied or moved under R-I3 keep `raw_path`, so they also block re-creation.
- **Test** (`c7b_startup_ignores_cache_and_known_raw`):
  - Setup: a hook for `_Unassigned/known.ARW` creates the row in session 1. `assign_unassigned(K1)` moves it to K1's session. A `K1/CaptureOne/x.ARW` cache file is added, then the app reopens (startup scan).
  - Assertion: the full `(raw_path, session_id)` set equals `[(canon(known), assigned_session)]`.
  - Against the old code: the scan attributes `_Unassigned/known.ARW` to session 1, finds no (1, path) row and inserts. That gives two rows, so the assertion fails. The test discriminates F1 at the startup-reconciliation boundary, and it passes the over-engineering test (it asserts stored state only).
  - Round-1 probe case 2 (a capture-root RAW, re-attributed under a newly active session) goes through the same lookup. It is resolved by the same line and needs no separate row.

## Triage
No blocking items remain, so there is nothing to classify.

| Item | Class | Disposition |
|---|---|---|
| none | — | — |

## Remaining findings
None blocking. No new notes. The 9 round-1 P05 notes are already in `docs/implementation/backlog.md`.

## Carry-forward dispositions (open notes)
| Note (backlog, P05 rows) | Destination |
|---|---|
| Capture root not NFC-normalized | P01 / P16 |
| Capture root cached for process lifetime | P16 |
| Startup scan 1 s wall-clock sleep; one `scan` event per RAW per startup | P16 |
| `tick` drops remaining ready paths when one `accept` fails | P05 backlog (next arrival-touching phase) |
| `scan` aborts on one unreadable subfolder | P05 backlog (next arrival-touching phase) |
| Hook payload parsing in core (H7 tension) | P06 |
| Import accepts any regular file | P08 |
| C8(a) fake records only a count | P05 backlog (test) |
| Synthetic RAW always puts largest stream first | P05 backlog (test) |

## Lessons for the plans
- **Unchanged from round 1, still open for the coordinator.** 05A MC-2 "row creation" and ch06 §Arrival still say "once per (session, path)". The code now implements "first sight per RAW, in any session" for capture arrivals, which is what M1, M9 and ch12 require. Fold this into the contract text so P06+ plans and tests don't re-derive the old rule.

## Write perimeter
- `docs/implementation/handoffs/reviewer/handoff_P05_capture_arrival_review_r2.md` (this file, new).
- `docs/implementation/plans/P05_capture_arrival.md`: Review log entry appended.
- `docs/implementation/master_plan.md`: P05 tracker row → `APPROVED`.
- `docs/implementation/narratives/P05_capture_arrival.md` (new).
- `docs/implementation/backlog.md`: unchanged.
- **Probes:** none. No production or test file was touched. Logs are in `/tmp/p05_rr_*.log`, and build artefacts are under `target/` only.
- No commits.
