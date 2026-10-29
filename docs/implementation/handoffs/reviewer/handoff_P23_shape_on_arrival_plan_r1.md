---
plan: P23_shape_on_arrival
role: plan
round: 1
state: OWNER_DECISIONS_PENDING
date: 2026-10-29
actor: Claude Fable 5.1 (pipeline-planner, implementation-planner doctrine under docs/process/flow.md)
---
# P23_shape_on_arrival — planning handoff, round 1

**Summary.** Gate passed: `docs/intention/intention.md:3` reads **RATIFIED** (owner, 2026-10-24); chapter 10B carries the
owner's decision of 2026-10-29 as a contract (MC-25) with its ledger entry (M31) and decision (R35), and the map, ch10,
ch12, ch14, ch15 and changelog entry 22 point at it. I read 10B in full, 01, 14, 10, 10A (MC-17/19/20), 05A MC-2, 07B MC-21,
12, 15, 16, the evidence section "Crop by script", the sdef (variant crop properties `:1193–1215`, `maximum crop` `:2036`,
`image.dimensions` `:1637`), flow.md, the master plan, STATUS.md, plans P19/P21/P22 and the code behind every task (ports,
arrival/photos/previews/editing_startup/health/settings services, the SQLite store's `arrival_insert`/jobs, the job runner,
the Capture One adapter, gate, mirror job, JXA runner and scripts, the fake, `TestApp`, the Tauri settings/health commands,
the Settings and Health pages, the workspace tile/stage CSS, and the P05/P06/P16/P17 tests this phase reuses — including
P17's Node harness for the generated JXA scripts). One plan file written (6 criteria / 15 rows, 4 `[rule]`, one optional
live row), `phases.txt` appended in place, master plan amended (§2, §3, §4, §5, §6.4, §6.6, §6.7, §6.8, §7.5, §9). One
book hole found and carded, not patched. `docs/intention` untouched. One commit, not pushed.

## ⚠ OWNER DECISIONS REQUIRED (1)

**Card 1 — How many times should the app retry setting a photo's shape before it gives up and shows a health warning?**
- **Question:** pick the give-up limit for the shape-on-arrival job: **A** 8 attempts (about 3 minutes of retries), **B**
  never give up (keep trying every minute, like the colour tags), or **C** another number.
- **Story:** You shoot twenty frames on a Tuesday while Capture One has frozen for a moment. Each new photo asks Capture One
  for its square crop; Capture One does not answer. With A, the app retries each photo with growing pauses for about three
  minutes, then stops and Health shows "Shape not applied to 20 photos"; pressing "Retry checks" or relaunching tries again
  and the warning clears when it works. With B, the app keeps asking once a minute for as long as it runs and Health never
  warns, so a frozen Capture One would only show up as rectangular photos in the shop.
- **Branches:** A — a visible warning after ~3 min, one more full round of tries per "Retry checks" or launch · B — no
  warning ever, silent retries forever · C — same as A with your number.
- **Recommendation:** A (8 attempts): the book asks for a give-up and a warning; 3 minutes matches how long a preview render
  may wait before it is declared failed.
- **On silence:** the gate holds — the phase is not dispatched until the number is set (the plan carries 8 as the pending
  default; the coordinator writes the final value into master plan §6.6).
- **Trace:** 10B §Failures ("give up after the same limit" — MC-19 names no limit); plan P23 task 3, rows C3(c); master plan
  §6.6 `CROP_GIVE_UP_ATTEMPTS`.

## Book hole (reported, not patched — orchestrator's instruction)
10B §Failures: "retry with the provider back-off (MC-19 rules), then give up after the same limit and record a health
item … until a later attempt succeeds". **MC-19 has no give-up limit**: the colour-tag mirror retries with the back-off
capped at 60 s forever (`crates/infra/src/jobs.rs` never fails a job; `backoff` caps at 60 s). "The same limit" has no
antecedent in the book. The plan names the constant `CROP_GIVE_UP_ATTEMPTS`, every row asserts it by name (charter rule 13),
and card 1 asks for the value. "A later attempt" also has no named trigger in the book; the plan chose a real start and the
Health page's "Retry checks" (see Choices). The orchestrator may answer card 1 from the book's spirit under flow §6 and fold
the number into 10B (a sentence) and master plan §6.6; the plan needs no other change.

## Write perimeter
- `docs/implementation/plans/P23_shape_on_arrival.md` (new) — committed.
- `docs/implementation/phases.txt` (one line appended in place with `printf >>`: `P23_shape_on_arrival`) — committed.
- `docs/implementation/master_plan.md` (§2 contracts row + 10B, ledger M1–M31; §3 trace ranges MC-1…MC-25 / M1–M31; §4 one
  NOT_STARTED row; §5 count 23, graph tail `→ P22 → P23`, `P23[P22]`; §6.4 `EditingProvider::{crop_aspect_ratios,
  shape_on_arrival}`, `Store` additions, `CaptureOneScripting` two ops + two scripts; §6.6 `JobKind::CropOnArrival`,
  `ShapeOutcome`, error `crop_ratio_unknown{name}`, health `crop_not_applied` / `crop_ratio_unknown`,
  `CROP_GIVE_UP_ATTEMPTS` pending; §6.7 `crop_ratios_list`; §6.8 `crop_on_arrival`; §7.5 fake crop state; §9 log entry) —
  committed. The diff was checked before staging: every hunk is mine (P21's approval had already been committed by the runner).
- `docs/implementation/handoffs/reviewer/handoff_P23_shape_on_arrival_plan_r1.md` (this file) — committed.
- Not touched: `docs/intention/**`, `plans/P21_*.md`, `plans/P22_*.md`, `STATUS.md`, `backlog.md`, any code, any tool-recorded
  state. Nothing pushed; the pipeline was not run.

## The plan in one table
| Phase | Scope | Rows | depends_on |
|---|---|---|---|
| `P23_shape_on_arrival` (difficulty high, projection no) | one race-free JXA call `apply_crop.js` (list check → ratio check → `maximumCrop(…, {apply:false})` → compare → set → read back), `crop_aspect_ratios.js`; adapter ops `shape_on_arrival` / `crop_aspect_ratios`; `crop_on_arrival` job enqueued inside the arrival transaction on first sight of a RAW (hook, watcher, scan, import; never copy/move; once per RAW via the jobs table — survives relaunch, no migration); `ShapeOnArrival` handler on the shared runner: `off` → no-op, applied → own-write window + MC-17 stale mark through `on_variant_adjusted`; runner gains per-handler `max_attempts` and terminal failures (`job_fail`); health `crop_not_applied` (derived from failed jobs, counted before re-queue) and `crop_ratio_unknown` (startup/health list check + job); failed jobs re-queued at real start and "Retry checks"; setting `crop_on_arrival` (default Square) + DTOs + `crop_ratios_list` command; Settings "Shape on arrival" native select (Leave as shot + list + stored stray name); Health labels; `.thumb-image` `object-fit: contain`; fake crop state; Node-harness rows for the script; one optional live row that restores with `maximumCrop` | 6 criteria / 15 rows (4 `[rule]`): C1 script ×3, C2 arrival/import/once/exclusions ×3, C3 failures ×4, C4 setting backend + screen ×2, C5 tiles CSS + health labels ×2, C6 live ×1 | [P22_frontend_tidy_up] |

## Choices the owner may want to know (recorded in the plan's Notes)
1. **State home for once-per-RAW:** the `jobs` table (one `crop_on_arrival` row per RAW, the `hash_job_exists` pattern already
   used in `arrival_insert`) — durable across relaunch, no `003` migration, `001_init.sql` untouched. A skipped attempt is a
   `done` job, so a hand-cropped import is never asked again.
2. **The job marks the RAW stale itself after a write** (through the hook's own MC-17 entry point, so it is not a retouch).
   The book expects Capture One's adjusted hook to re-render; that hook fires only for the primary variant, and MC-17's sidecar
   fallback is suppressed inside the own-write window, so a multi-file import would otherwise leave all but one preview
   rectangular while the export is square. Cost: at most one extra render, coalesced by the delay.
3. **`mark_own_write` before the single call, even when the outcome is "skipped"** — needed so the write's echo is never a
   retouch; harmless on a brand-new RAW (no `main`/`condition` row can be dirtied). C2(a) asserts `retouch_gen == 0`.
4. **Later attempts** after a give-up: re-queued (fresh back-off series) at a real start and by Health → Retry checks; a
   settings rebuild does not re-queue (P17 F1). The health count is taken before the re-queue, so a report shows what had
   failed and goes green on the next check after a success.
5. **Unknown-name failures are not counted in "Shape not applied to n photos"** so the two book messages stay distinct; a
   corrected name is applied at the next Retry checks or start (try-out step 6).
6. **Health text for one photo:** `Shape not applied to 1 photo` (book: "n photos").
7. **Crop job runs before the first preview render** in the job order, so the first preview usually already carries the shape.
8. **Tiles keep 4:3 and letterbox** (`object-fit: contain`); the stage already fits; the sessions-list 36×28 thumbnails keep
   `cover` (not tiles).
9. **Projection stays off** (P17 precedent). One externally owned shape is not yet observed — `maximumCrop(variant,
   {apply: false})` and the JS shape of its rectangle. The script checks the ratio first (only `""`/`Unconstrained` reach
   the call), throws on an unparseable rectangle (never a silent "skipped"), and the live row C6(a) observes both before
   anything depends on them; if `apply: false` applied anyway, the implementer stops and reports with `image.dimensions`
   as the fallback comparator for the orchestrator to decide.
10. **`crop_ratios_list` is a new thin command**; the Settings page fetches the list once on mount; a failed fetch shows only
    "Leave as shot" and the stored value (no alert).

## Grounding worth knowing (coordinator / reviewer)
- `jobs.rs:33–64` (post-P21): unknown kinds retry in 60 s; errors map to `variant_not_found` / `tag_verification` / `timeout` /
  `provider`; no terminal path exists — task 3 adds `job_fail` + `JobHandler::max_attempts` (default `None`, so no other kind
  changes behaviour).
- `store.rs arrival_insert :1052–1100`: the `existing` lookup is cross-session for captures and per-session for imports; the
  plan's "no earlier row for this RAW" uses an explicit `EXISTS` before the insert for both sources.
- `previews.rs schedule_due`: a stale RAW with no in-memory `last_adjusted` renders at once; that is why the handler goes
  through `on_variant_adjusted` (sets `last_adjusted`, respects the own-write window) rather than `Store::adjusted_event`.
- Frontend mock factories of `@/services/settings` enumerate exports; adding `listCropRatios` to `SettingsPage` requires
  `listCropRatios: vi.fn(async () => [])` in p16, p18, p19 and P22's new shell test — listed in the plan's files.
- P22 is IMPLEMENTING while this was planned; its `SettingsPage.tsx`/`PageHeader` changes and new test files are expected
  drift. Line numbers in the plan are from `6c5e3c4`; names are the contract.

## Lessons for the planner
- A book clause that refers to "the same limit" needs the antecedent checked in the referenced contract before planning;
  here the referenced contract had none. Carded rather than guessed, per the orchestrator's instruction.
- When a contract says "no extra work, the hook does it", check the hook's firing condition against every trigger the
  contract names (import of several files is not the tethered single-frame case the evidence probed).
