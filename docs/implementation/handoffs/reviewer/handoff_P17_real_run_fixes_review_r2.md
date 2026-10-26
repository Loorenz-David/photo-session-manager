---
plan: P17_real_run_fixes
role: review
round: 2
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-10-26
actor: opus
---
# P17 re-review, round 2: APPROVED_WITH_NOTES

**Summary.** Fix 1 (`43adbee`) resolves F1 as the correction said. A settings rebuild no longer runs the start-only crash recovery (`requeue_running_jobs`, active-publish re-enqueue) while the old runtime's workers are live, and a real start keeps that recovery. The perimeter is the two allowed files. The new stored-state test drives the production save path and fails on either half of the pre-fix behaviour. Suite, lint and typecheck are green. No blocking item remains, so triage has no MUST. One new note went to the backlog. Round 1 settled the 16 criterion rows, the two live Capture One rows and H1–H8 except H4; none was re-verified here.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Perimeter check
- Gate: the tracker showed `IMPLEMENTED` with the fix 1 note, and checkpoint `43adbee106319609b5ca480b0b2cbedece2cffe9` exists.
- `git show --stat 43adbee` lists 2 files, both on F1's allowed list:
  - `crates/integrations/src/lib.rs`: 12 lines changed.
  - `crates/testkit/tests/p17_real_run_fixes.rs`: 36 lines added.
- `crates/infra/src/sqlite/store.rs` is untouched. That was allowed, because the enqueue check did not change.
- Commits after the checkpoint (`83a835c`, `661048a`) touch only runner documents: the fix handoff, the tracker row, the Review log and the r2 prompt.
- Stamp: the prompt directs one run. Tree `661048a`, `git status --porcelain` empty before and after.

| Command | Result |
|---|---|
| `cargo test --workspace` | 406 passed, 0 failed |
| `cargo clippy --workspace --all-targets -- -D warnings` | green |
| `cargo fmt --all -- --check` | green |
| `npm run typecheck` (apps/desktop) | green |
| `npm test` (apps/desktop) | 92/92 |

## Findings resolved
**F1 (B3, H4): resolved as its correction said.**
- **Change.** In `App::build`, `requeue_running_jobs()` and the `active_publish_items` → `enqueue_publish_item` loop now sit inside `if start_backup`.
- **Callers.**
  - `open_with_adapter` (every real start) passes `true`.
  - `SettingsSetContext::prepare` passes `false`. Both `App::settings_set` and `AppState::settings_set` go through it.
  - `App::reopen` (no callers) passes `false`.
- **Test.** `settings_rebuild_preserves_running_job_and_does_not_duplicate_active_publish_job` follows the production path `App::settings_set` → `prepare` → `build(false)`:
  - It confirms a publish and claims the item's `publish_step` through `Store::job_due`.
  - It saves `preview_delay_s: 20`, which is not a theme-only change, so a rebuild runs.
  - It asserts `min(status) = 'running'` and `count(*) = 1` for that item.
- **Why the test discriminates** (reasoned by reading, no probe):
  - Pre-fix, the requeue makes the job `queued`, and `min` becomes `'queued'`.
  - A re-enqueue without the requeue would insert a second row, because `enqueue_publish_item` checks only for `queued`. The count would become 2.
- The test traces to F1 and asserts stored state, so it is no orphan and passes the over-engineering test.

**Changed seam, read adversarially. All verified correct.**
- **The two runtimes still share the queue until the old workers are joined.** A double claim is still impossible:
  - `job_due` is a transaction (SELECT then UPDATE) on each runtime's own WAL connection.
  - A connection whose read snapshot is stale cannot upgrade to a write. It gets `SQLITE_BUSY`, and `tick` returns an error without running the job.
- **No job is left `running` by the old worker.** The old worker's `tick` drains with the old handlers and records `done`/retry before the join.
- **The other build effects still run on a rebuild** (spool drain, capture scan, sidecar check). They are idempotent (spool dedupe, RAW-hash arrival, enqueue-only re-render) and predate P17.
- **Real-start crash recovery is unchanged.** All earlier-phase restart tests are green.

## Triage
| Item | Class | Reason |
|---|---|---|
| (none blocking) | — | F1 resolved. No new blocking item found. |
| N1 (new note) | DROP → backlog | Speculative robustness on a narrow path: remove the store mid-publish, then re-add it in the same run. It recovers at the next launch. The root cause (claiming a kind with no handler) predates P17. |

## Remaining findings
**N1 (note, backlog).**
- With no store configured, the publish worker still runs. It claims a queued `publish_step`, finds no handler ("unknown job kind"), and the job stays `running`.
- Before fix 1, a settings rebuild requeued such jobs. Now only a relaunch does. Skipping recovery on a rebuild is correct for H4.
- The real fix is not to claim kinds that have no handler. That is a later phase's call.

Round-1 notes (4) stay in the backlog, unchanged.

## Lessons for the plans
- None new. The round-1 lesson on rebuild rows (name the state an in-flight job must keep) stands. The fix test is the shape such a row should take.

## Write perimeter and side effects
- **Files written by this session:**
  - `docs/implementation/master_plan.md`: P17 tracker row → `APPROVED`.
  - `docs/implementation/plans/P17_real_run_fixes.md`: Review log entry for r2.
  - `docs/implementation/backlog.md`: 1 row appended.
  - `docs/implementation/narratives/P17_real_run_fixes.md`: new.
  - This handoff.
- **No probes and no edits.** No probe file was written and no production or test file was edited. There was no Capture One call and no capture. Temp output is in `/tmp/p17r2_*.txt`, outside the repo.
- **Nothing committed.** The runner commits.
