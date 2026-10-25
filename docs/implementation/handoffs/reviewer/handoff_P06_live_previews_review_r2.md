---
plan: P06_live_previews
role: review
round: 2
verdict: APPROVED
state: DONE
date: 2026-10-25
actor: opus
---
# P06 live previews — re-review r2

**Verdict: APPROVED.** Both r1 blocking findings are resolved as their corrections said. There are no
divergences, the perimeter is clean, and suite, lint and typecheck are green. No blocking items remain,
so the triage table is empty. Tracker → `APPROVED`, Review log entry appended, narrative written.

## ⚠ OWNER DECISIONS REQUIRED (0)

None.

## Review history

r1 settled everything except F1 and F2: C1(a)–(e), C2(a)–(e), C3(a), C4(a)–(c), C5(a)–(c) and C6(a); H1–H6 and H8;
the assumptions; and 9 backlog notes. This round covers only the fix delta and its seam.

## Perimeter check

- Gate: tracker row `IMPLEMENTED` with the fix-1 note; checkpoint `135c676` is in `git log`.
- `git show --stat 135c676` lists 8 paths:
  - 6 code paths: `crates/core/src/ports/editing.rs`, `crates/core/src/ports/store.rs`,
    `crates/core/src/services/previews.rs`, `crates/infra/src/sqlite/store.rs`,
    `crates/integrations/src/editing/capture_one/adapter.rs`, `crates/testkit/tests/p06_live_previews.rs`.
  - 2 doc paths: the tracker row in `docs/implementation/master_plan.md` and the Review log in
    `docs/implementation/plans/P06_live_previews.md`.
  - Every code path is inside F1's or F2's allowed list. The 6 paths equal the handoff's declared list. No finding.
- The commits after the checkpoint (`8e1062c` fix handoff, `d46e952` re-review prompt) are runner documents only.
- Stamp on tree `d46e952`. `git status --porcelain` was clean before my document edits, and the code is identical
  to `135c676`. The prompt directs one run.

  | Check | Result |
  |---|---|
  | `cargo test --workspace` | exit 0; 178 passed, 0 failed; `p06_live_previews` 19/19 |
  | `cargo clippy --workspace --all-targets -- -D warnings` | 0 |
  | `cargo fmt --all -- --check` | 0 |
  | `npm run typecheck` | 0 |
  | `npm test` | 0 (no frontend test files) |

## Findings resolved

### F1 (B1) — resolved
- **The fix.** `PreviewService::request_initial` handles a RAW that is not stale and already has
  `adjusted_preview_path`. Instead of returning early, it now calls the new
  `Store::set_missing_preview_paths(raw, path)`. That call runs one transaction: it selects the RAW's rows whose
  path `IS NULL` and gives each the current path. The service then emits `photo_changed{photo_id, session_id}`
  for each filled row. This is the correction exactly as r1 wrote it.
- **Production reaches it.** `SqliteStore::arrival_insert` queues a `render_preview` job for every new row. It is
  the path used by capture and import arrivals (`services/arrival.rs`). `RenderPreviewJob::run` calls
  `request_initial`, so the scenario r1 probed (same RAW into S2 after the S1 render) now goes through the fix.
- **Consistent with completion.** Deleted rows are filled too, the same way `complete_preview` sets the path on
  every row of the RAW. A row that already has a path is never overwritten.
- **Test.** `rendered_raw_gets_preview_on_later_session_row` runs S1 render → same RAW into S2 → `request_initial`.
  It asserts three things:
  - the S2 row has the S1 path;
  - the fake received only one `process` (no needless re-render);
  - exactly one `photo_changed`, for `(S2 row, S2)`.

  On the pre-fix code the S2 path stays `None`, so the test can fail. It sits at the boundary F1 named and traces
  to C1(a) through F1's correction. It is not an orphan.

### F2 (B3, H7) — resolved
- **The port.** New provider-neutral method `EditingProvider::settings_changed_at(raw) -> Result<Option<SystemTime>>`.
- **The adapter.** It is implemented in `CaptureOneAdapter`: the newest mtime of `<RAW folder>/CaptureOne/Settings*/<name>.cos`,
  and a missing `CaptureOne` folder gives `None`.
- **The core.** `PreviewService::check_sidecars` keeps only the comparison rule: later than the last completion,
  later than own write + 30 s, and once per mtime. `core/src` no longer names `CaptureOne` or `.cos`, except P05's
  arrival skip, which is already backlog row "P05 → P16 (H7)".
- **Tests.** `CaptureOneAdapter` is the only `EditingProvider` implementation, and the testkit drives it with a
  fake runner. So C4(a)–(c), unchanged and green, exercise the moved code.

## Triage

No blocking items remain. Nothing was classified MUST or DROP.

| Item | Class | Reason |
|---|---|---|
| — | — | — |

## Remaining findings

None.

**Seen in passing, not noted.** Before the fix, a `CaptureOne` entry that was a regular file was skipped
(`is_dir()`). Now `read_dir` returns an error, and the focus/startup sidecar pass stops for all RAWs. A plain file
with that name beside the RAWs is an implausible layout. Under flow §5 this is speculative robustness, so it was
not added to the backlog.

## Carry-forward

r1's 9 backlog rows stand unchanged, and each is already routed in `docs/implementation/backlog.md`: P16 ×3, P08, P09,
book 07B/10A, and P06-local ×3. This round adds no new rows.

## Lessons for the plans

None new. r1's two lessons stand:
- A plan should state which preview a row created after a completion gets.
- A plan that touches a provider file layout should name the adapter seam.

## Write perimeter

- `docs/implementation/master_plan.md`: P06 tracker row only, → `APPROVED`.
- `docs/implementation/plans/P06_live_previews.md`: review r2 entry appended to the Review log.
- `docs/implementation/narratives/P06_live_previews.md`: new.
- This handoff.
- No probes. No production or test file was edited or created. No database outside test temp dirs was touched.
  Build artefacts are in `target/` only. No commits.
