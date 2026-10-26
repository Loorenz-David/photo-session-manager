---
plan: P13_publish_runs
role: review
round: 2
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-10-26
actor: opus
---
# Handoff: P13_publish_runs, re-review round 2 (after fix 1)

**Summary.** Fix 1 resolves both blocking findings from round 1 as their corrections asked. The perimeter holds, and
suite, lint, fmt and typecheck are green. No blocking item remains, so the verdict is **APPROVED_WITH_NOTES**: the 11
round-1 notes are already in the backlog, and this round adds one (the F2 test's reach). Tracker → `APPROVED`. The
Review log entry, the backlog row and the narrative are written.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Review history
Round 1 settled all 35 criterion rows (C1–C9) at their boundaries, the declared perimeter and the 11 notes. It
raised F1 (a ready nameless media stalls `upload`) and F2 (publish steps run on the arrival/preview loop). This
round covers only the fix delta and the seam it changed.

## Gate
- The tracker row was `IMPLEMENTED` with the fix 1 note (actor codex).
- Checkpoint `5e9531e9b88b3fb4ed190b927c6700f9c5e45a5b` exists (`git cat-file -t` → commit).
- The commits after it (`aa5fccd`, `69cc73a`) add only the fix handoff and this round's prompt.

Passed.

## Perimeter check
`git show --stat 5e9531e` lists 10 paths:

| Path | Allowed by |
|---|---|
| `crates/core/src/services/publish.rs` | F1 |
| `crates/testkit/tests/p13_publish_runs.rs` | F1, F2 |
| `crates/testkit/src/fake_commerce.rs` | F1 (if needed), F2 |
| `crates/integrations/src/lib.rs` | F2 |
| `crates/infra/src/jobs.rs` | F2 |
| `crates/core/src/ports/store.rs` | F2 |
| `crates/infra/src/sqlite/store.rs` | F2 |
| `crates/testkit/src/test_app.rs` | F2 |
| `docs/implementation/master_plan.md` | the P13 tracker row only (one-line diff) |
| `docs/implementation/plans/P13_publish_runs.md` | its Review log only (append) |

Nothing outside the perimeter changed. The handoff's "Files changed" list equals this set (plus the handoff itself).
`git status` was clean before the review.

## Evidence (one run, master plan §7.2 commands)
Tree: HEAD `69cc73a`. Its code is identical to `5e9531e`; only docs come after it.

| Command | Result |
|---|---|
| `cargo test --workspace` | 345 passed, 0 failed (36 in `p13_publish_runs`) |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo fmt --all -- --check` | exit 0 |
| `cd apps/desktop && npm run typecheck` | exit 0 |
| `cd apps/desktop && npm test` | 5 files, 79/79 passed |

No probes were run and no file was touched outside this round's artifacts. There were no mutation probes (flow §4).

## Findings resolved

### F1: resolved as corrected
- **Code.** `PublishService::upload` (`publish.rs` ~570) now returns "not yet" only when a nameless media has status
  `Processing` or `Uploaded`.
  - `RemoteMediaStatus` has exactly four variants (`Uploaded`, `Processing`, `Ready`, `Failed`), so `Ready` and
    `Failed` nameless media now fall through.
  - Adoption only matches media with a filename (`name_matches(stem, name)`), so a nameless media is never adopted.
    It is left to `update`, where MC-12 deletes it if it was announced, or fails with `foreign_media_added` if it was
    not.
  - SV-2's "wait while PROCESSING" is kept.
- **Test.** `ready_nameless_announced_media_is_removed_and_publish_completes`:
  - Setup: a linked product holding one READY nameless media. `confirm` announces it, because it is not an own
    link.
  - The test drains and asserts `step == Done` and `gallery == [own media id]`.
  - Under the old predicate (`status != Failed`) the item stays at `upload`, so the test fails. It discriminates.

### F2: resolved as corrected
- **Code.**
  - `App::open_with_adapter` builds the shared runner with `excluding_kind("publish_step")` and a second runner
    with `only_kind("publish_step")`. That second runner holds the only `PublishJob` registration.
  - Its worker thread starts whenever `run_startup` is true (`App::open` passes `true`). It ticks every 50 ms, stops
    on the shared `stop` flag and is joined in `Drop`.
  - `SqliteStore::job_due` filters with `(?2 IS NULL OR kind=?2) AND (?3 IS NULL OR kind<>?3)`. `Option<&str>`
    binds NULL for "no filter", so the shared runner still sees every other kind.
  - The claim is still one transaction under the connection mutex, so two runners cannot claim the same job.
  - `SqliteStore` is the only `Store` impl (grep), and the only `job_due`/`tick` callers are the two runners and
    `TestApp::tick`, which now ticks both.
  - The shared loop's arrival, watcher, spool and preview work no longer waits on any network call.
- **Test.** `capture_arrival_is_recorded_while_publish_media_poll_is_held`:
  - Setup: it holds the fake poll, runs a publish tick on its own thread and waits until the tick is inside the
    poll.
  - Then it scans a new capture and asserts that the photo row exists before releasing the poll.
  - This proves a held publish step holds no store or app lock. It does not observe the shared loop's exclusion,
    because `TestApp` runs with `run_startup = false` and the loop never ticks jobs. The exclusion is verified by
    reading (above). That gap is a note, not B2: the code is correct, and observing the real loop would need a
    threaded `run_startup` harness (more tests; over-engineering test).

### Seen in passing (changed seam, no finding)
- Publish steps now run truly concurrently with arrival, mirror-tag and preview work. They already ran concurrently
  with IPC commands (`publish_retry`, export) before. All shared state is behind the store mutex or the services'
  own mutexes, so this adds no new race class.
- With `commerce = None`, a leftover queued `publish_step` job reaches the publish runner. There it hits "unknown
  job kind", the job stays `running` until the next launch's requeue, and the error is ignored. Before the fix the
  same thing happened on the shared runner, where it aborted that tick for all kinds. The fix isolates it. No
  finding.

## Triage (flow §5)
No blocking item remains after fix 1.

| Item | Class | Disposition |
|---|---|---|
| F1 | resolved | |
| F2 | resolved | |
| F2 test does not exercise the shared loop's exclusion | DROP (more tests; code verified correct) | backlog row "P13 (test, fix 1)" |

No MUST → `APPROVED_WITH_NOTES`.

## Remaining findings
None blocking. Notes in `docs/implementation/backlog.md`: the 11 P13 rows from round 1 (unchanged) plus one new row,
"P13 (test, fix 1)".

## Carry-forward dispositions
| Note | Destination |
|---|---|
| Incomplete `integration` block panics at launch | P16 |
| 13 `"shopify"` literals in core (H7 hygiene) | plan / H7, the next commerce adapter |
| C5(f) fake flag is decoration | none (the adapter owns the rule) |
| C7(a)/C9(b) loose assertions | none (low risk) |
| Notification shows raw epoch ms | P15 |
| Export job id in `settings` table | the next migration |
| Shopify calls on the IPC thread under `Mutex<App>` | P15/P16 |
| Preview removal count after a crash | none (outcome correct) |
| No condition key before the first schema refresh | P14 |
| NULL-hash `raw_changed` | P16 (still open) |
| P12/P09/P03 carry-forwards | still open, as written |
| F2 test reach (new) | any phase touching the background loop |

## Lessons for the plans
- A requested regression test for "X does not block Y across threads" should name the thread it drives. "A capture
  delivered during the upload" was satisfied by delivering it from the test thread, which proves lock independence
  but not loop independence.

## Write perimeter (this session)
- `docs/implementation/handoffs/reviewer/handoff_P13_publish_runs_review_r2.md` (this file, new)
- `docs/implementation/narratives/P13_publish_runs.md` (new)
- `docs/implementation/master_plan.md` (the P13 tracker row only)
- `docs/implementation/plans/P13_publish_runs.md` (Review log append)
- `docs/implementation/backlog.md` (one row appended)

No production or test file was touched. Nothing was committed.
