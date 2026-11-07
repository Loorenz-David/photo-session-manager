---
plan: P49_capture_one_alongside
role: review
round: 3
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-07
actor: opus
---

# P49 re-review r3: APPROVED_WITH_NOTES

**Review history.**
- r1 settled C1–C5 apart from F1, the before-render style step that styled any unmarked RAW.
- Fix 1 (`084f1ee`) resolved F1 and introduced F2 (r2): the eligibility read returned SQL NULL, so tethered previews never rendered while a style was set.
- The orchestrator wrote FIX2_AUTHORISED, limited to `sqlite/store.rs` and the P49 test file.
- This round covers fix 2 (`3963477`) only.

**Gate.**
- The tracker showed `IMPLEMENTED` with the fix 2 note.
- Checkpoint `3963477` is in `git log`.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Perimeter check
`git show --stat 3963477` lists:
- `crates/infra/src/sqlite/store.rs`: 1 line, the eligibility predicate
- `crates/testkit/tests/p49_capture_one_alongside.rs`: +38, one new test
- `docs/implementation/master_plan.md`: P49 row
- `docs/implementation/plans/P49_capture_one_alongside.md`: Review log

This is exactly the FIX2_AUTHORISED list. The later commits are runner commits of handoff and prompt only:
- `8feab67`: fix handoff
- `e564532`: this prompt

**Reviewer run.** Clean HEAD `e564532`; production code = `3963477`.

| Check | Result |
|---|---|
| `cargo test --workspace` | green (P49 21/21) |
| `cargo clippy --workspace --all-targets -- -D warnings` | green |
| `cargo fmt --all -- --check` | green |
| `npm run typecheck` | green |
| `npm test` 369/369 | cited from the fix 2 stamp, not re-run: no frontend file changed |

## Findings resolved
**F2 (B1, MUST in r2): resolved, as the correction stated.**

The predicate is now `SELECT style_applied_at IS ?2 AND rendered_gen=0`. Why the expression can never be SQL NULL:
- `IS` returns 0 or 1, including when `style_applied_at` is NULL.
- `rendered_gen` is `INTEGER NOT NULL DEFAULT 0` (migration 002).

Cases, read against the code:

| Stored state | Result |
|---|---|
| pending sentinel + `rendered_gen = 0` | true |
| NULL (tethered, pre-P49, failed eligibility write) | false |
| applied timestamp | false |
| `rendered_gen > 0` | false |
| no `raw_state` row | false (`.optional()` → `unwrap_or(false)`) |

The callers `prepare_render` and `apply_style_once` (`follow.rs`) are unchanged. A false result skips the style step and lets the render proceed.

**New test.** `tethered_capture_with_style_renders_without_applying_script_style`:
- Setup: `with_fake_document`, an active session, and "Bright Contrast" configured and listed; a file is written into the session folder, followed by `watcher_event` and `tick(1_000)`.
- Asserts: the last issued render is `tethered-v1`, and there is no `apply_style:` op.
- On the pre-fix code, the r2 end-to-end probe showed this path issued nothing, so `issued.last().unwrap()` fails. The test discriminates.

**Divergence: pre-P49 photo.** The orchestrator's ruling also asked for a "pre-P49 photo with no sentinel renders" case. The fix handoff covers it with the same NULL state. This is sound:
- A pre-P49 `raw_state` row has NULL `style_applied_at`, because 007 adds a nullable column with no default.
- Otherwise it has no row at all.
- Both inputs read false through the same query the new test exercises.

**F1 still holds.** `f1_retouch_after_first_render_does_not_receive_late_style` is green in this run.

## Triage (flow.md §5)
| Item | Class | Why |
|---|---|---|
| F2 | resolved | see above |
| New test also asserts the raw DB column `style_applied_at` is NULL (via `rusqlite`) | DROP → backlog | Test hygiene: it checks storage, not outcome (flow.md §4 over-engineering test). No wrong outcome. |

No MUST items remain. Verdict `APPROVED_WITH_NOTES`.

## Remaining findings
None blocking. One note was written to `docs/implementation/backlog.md`, headed "P49 reviewer r3, test hygiene".

Earlier backlog notes stay where they are and are carried into the narrative's "Not yet":
- 8 from r1
- 2 from r2

## Carry-forward dispositions
| Note | Destination |
|---|---|
| Take-back step 5 `follow: None` (no immediate retry) | P51/P52 |
| `apply_style` error in `prepare_render` fails the render | P52 (or a later fix) |
| `set_current_collection.js` read-back via deprecated `file` | P52 hardware row |
| V4 live cause unnamed | P52 hardware row |
| `CaptureOneFollow.current` unused; import of an in-folder file / crash orphan; `SCRIPTS` count lesson | backlog (hygiene / planner) |
| Sentinel in timestamp column; in-flight first-render window; raw-column read in the new test | backlog (next time the P01 schema test or the P49 test is touched) |

## Lessons for the plans
- An orchestrator ruling that names two test cases with the same stored input (NULL marker) can be met by one test. Rulings should name the *distinct state*, not the user story, so the reviewer can judge coverage mechanically.

## Write perimeter and probe declaration
- **No probes.** No production file or test file was edited or created, even temporarily.
- **Commands run:** suite, clippy, fmt, typecheck, once each.
- **Files written by this review:**
  - `docs/implementation/master_plan.md`: P49 tracker row → `APPROVED`
  - `docs/implementation/plans/P49_capture_one_alongside.md`: Review log entry for r3
  - `docs/implementation/backlog.md`: 1 appended line
  - `docs/implementation/narratives/P49_capture_one_alongside.md`: new
  - this handoff
- Nothing committed. The runner commits these files and makes the approval commit.
