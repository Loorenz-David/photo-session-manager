---
plan: P49_capture_one_alongside
role: review
round: 2
verdict: CHANGES_REQUESTED
state: DONE
date: 2026-11-07
actor: opus
---

# P49 re-review r2: CHANGES_REQUESTED (1 MUST)

**Review history.** In r1, everything in C1–C5 was settled except F1: the before-render style step styled any RAW that had no mark. Fix 1 (`084f1ee`) addressed F1. This round covers only the changed seam, the style eligibility.

**Gate.** The tracker showed `IMPLEMENTED` with the fix 1 note. Checkpoint `084f1ee` is in `git log`.

## Perimeter check
`git show --stat 084f1ee` lists:
- `crates/core/src/ports/store.rs`
- `crates/infra/src/sqlite/store.rs`
- `crates/integrations/src/editing/capture_one/follow.rs`
- `crates/testkit/tests/p49_capture_one_alongside.rs`
- `docs/implementation/master_plan.md` (P49 row)
- `docs/implementation/plans/P49_capture_one_alongside.md` (Review log)

All four code files are on F1's allowed list. Nothing outside it changed. Later commits (`c507609`, `013663d`) are runner commits of handoff and prompt only.

**Suite (reviewer run on HEAD `013663d`; production code = `084f1ee`).** All green:
- `cargo test --workspace`, including p49 20/20
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo fmt --all -- --check`
- `npm run typecheck`
- `npm test` 369/369

## Findings resolved
**F1 (B1): resolved.**
- `photo_arrived` now writes a pending sentinel (`i64::MIN`) into `raw_state.style_applied_at`.
- `prepare_render` and `apply_style_once` style a RAW only if the sentinel is present and `rendered_gen = 0`.
- Applying the style replaces the sentinel with a timestamp.
- New test `f1_retouch_after_first_render_does_not_receive_late_style`: render with no style, then set a style, then retouch. Result: no `apply_style`. Pre-fix code fails it, as r1's probe showed. Removing the `rendered_gen = 0` clause would also fail it.
- C3(b) now sets the style *after* the arrival, which is the row's own order. It passes through the settings rebuild, so the eligibility persists as the correction required. This also closes r1 backlog note 4.

**Divergence: sentinel instead of a new column.** Sound in outcome:
- `style_applied_at()` maps the sentinel to `None`.
- `mark_style_applied` overwrites only NULL or the sentinel.
- No other code reads the column.

A hygiene note went to the backlog.

## Remaining findings

### F2: blocking (B1), route: production, introduced by fix 1
**What is wrong.** `SqliteStore::style_eligible_before_first_render` runs `SELECT style_applied_at=?2 AND rendered_gen=0`.
1. For a row with `style_applied_at IS NULL` and `rendered_gen = 0`, SQLite returns **NULL**, not 0. Checked in `sqlite3`: `NULL AND 1` gives NULL.
2. `r.get::<bool>` on NULL is `InvalidColumnType`, so the method returns `Err`.
3. `prepare_render` propagates the error.
4. `PreviewService::request` records it as a provider failure and backs off.
5. Every retry hits the same error, so the preview render is **never issued**.

This happens whenever `style_name` is non-empty, for every RAW that did not pass through `photo_arrived`:
- **every tethered Capture One capture**: `ArrivalService` calls `photo_arrived` only for imports or when `follow_capture` is true, which means the direct adapter only;
- photos from before P49 that were never rendered;
- an arrival whose eligibility write failed (it is only logged).

**Probes.** Two temporary test files, each run once and then deleted:
1. `TestApp::with_fake_document()`, style "Bright Contrast", then `add_photo` + `upsert_raw_state` + `request_initial`:
   - `style_eligible_before_first_render` = `Err(InvalidData("Invalid column type Null at index: 0 …"))`
   - `ops = []`, `issued = 0`
   - the same setup with no style gives `process:v1:PSM Preview`
2. End to end: a file in the active session's capture folder, then `watcher_event`, then 20 × `tick(1_000)`:
   - one photo row, **0 renders issued** with the style set
   - `process:vt:PSM Preview`, 1 issued, with no style

Before the fix the same path applied the style (F1's overreach). Now it blocks the preview instead.

**Authority.**
- 10C MC-26 step 5: tethered captures get the style from Capture One's next-capture settings, and a style problem never stops capture. The by-script style is for the direct adapter only.
- 10E R-I35: the style is scoped to arriving variants.
- The live-preview contract: a stale photo gets its render.

**Correction.**
- Make the read NULL-safe. Any of these works:
  - `SELECT COALESCE(style_applied_at=?2,0) AND rendered_gen=0 …`
  - `style_applied_at IS ?2 AND rendered_gen=0`
  - read `Option<bool>` and `unwrap_or(false)`
- Add one boundary test: a tethered capture (`with_fake_document`, watcher arrival) with a style configured has its render issued and no `apply_style` op.

**Files the fix may touch:**
- `crates/infra/src/sqlite/store.rs`
- `crates/testkit/tests/p49_capture_one_alongside.rs`

## Triage (flow.md §5)
| Item | Class | Why |
|---|---|---|
| F2: eligibility read errors on NULL, so tethered previews never render while a style is set | **MUST** | User-visible wrong outcome: no adjusted previews for the MVP's tethered camera path whenever Settings names a style. B1. |
| Sentinel overloads the timestamp column | DROP → backlog | Hygiene; no wrong outcome. |
| Settings change during an unfinished first render reopens the style window | DROP → backlog | Speculative, seconds wide. |

Verdict `CHANGES_REQUESTED`, MUST items: **F2 only**. The fix budget is spent, so the orchestrator decides: `FIX2_AUTHORISED` (a one-line SQL change plus one test) or a card.

## Notes written to backlog
Two entries headed "P49 reviewer r2": the sentinel hygiene and the in-flight first-render edge.

## Lessons for the plans
- A nullable column read into a non-null Rust type is a seam: SQL three-valued logic turns "false" into NULL. Rows for a once-only side effect should include the **non-participating** population (here: a tethered capture with a style set). Neither r1's lesson nor the fix test covered that population.

## Probe declaration
- Created, run once, then deleted: `crates/testkit/tests/zz_review_probe_p49_r2.rs` and `crates/testkit/tests/zz_review_probe_p49_r2b.rs` (sources kept in `/tmp/p49r2/`). `git status --short` after deletion showed no stray file.
- No production file was edited.
- Files written by this review:
  - `docs/implementation/master_plan.md` (P49 tracker row only)
  - `docs/implementation/plans/P49_capture_one_alongside.md` (Review log entry)
  - `docs/implementation/backlog.md` (2 appended lines)
  - this handoff
- No narrative, because the phase is not approved. Nothing committed.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
