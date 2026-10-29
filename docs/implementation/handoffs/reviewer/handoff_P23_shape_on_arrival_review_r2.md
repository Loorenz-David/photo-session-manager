---
plan: P23_shape_on_arrival
role: review
round: 2
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-10-29
actor: opus
---
# Re-review P23_shape_on_arrival (round 2)

**Verdict: APPROVED_WITH_NOTES.** Both r1 blocking findings (F1, F2) are resolved as their corrections said.
The suite, lint and typecheck are green. The live row C6(a), which failed in r1, now passes on this Mac's
Capture One. No blocking item remains, so the triage table is empty. r1's 6 backlog notes stand and none are new.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Review history
r1 settled rows C1–C5 at their boundaries in the suite (14/14), H1–H8, and the 39-path implementation perimeter. It
found F1 and F2 by running C6(a) live, both in `apply_crop.js`. This round is scoped to the fix delta
(`5d2210d`) plus the re-run of C6(a).

## Gate
- Tracker row was `IMPLEMENTED` (actor codex, fix-1 note). Fix checkpoint `5d2210d18ea7597d0948e4f140548bfc070c0542` is in `git log`.

## Perimeter check
`git show --stat 5d2210d` lists 4 paths:
- `crates/integrations/src/editing/capture_one/jxa/apply_crop.js`: allowed by F1/F2.
- `crates/testkit/tests/p23_shape_on_arrival.rs`: allowed by F1/F2.
- `docs/implementation/master_plan.md`: the P23 tracker row only.
- `docs/implementation/plans/P23_shape_on_arrival.md`: the Review log append only.

Nothing outside the allowed files changed. Later commits (`b8d84f1`, `80d92de`) add only the fix handoff and this round's
prompt. This matches the fix handoff's declared list.

## Evidence (stamp re-run by reviewer, once)
Tree `80d92de`, `git status --porcelain` clean. Code is identical to `5d2210d`. The fix handoff's stamp cites no clean-tree
identity, and the prompt directs one run.
- `cargo test --workspace`: exit 0, 432 passed, 0 failed (P23 file 11/11).
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0. `cargo fmt --all -- --check`: exit 0.
- `npm run typecheck`: exit 0. `npm test`: 17 files, 127 passed.

## Findings resolved
- **F1 (untouched ratio reads `null`): resolved.** `apply_crop.js:13` now skips only when the ratio is not `null`,
  `undefined`, `""` or `Unconstrained`. C1(a) drives `ratio: null` and, separately, `""`. On the r1 script the `null` case
  prints `skipped` with no write, so the row now fails on the defect. The harness's `hasOwnProperty` read lets an explicit
  `null` through (`??` would have turned it into `""`).
- **F2 (maximum rectangle misread): resolved.** `apply_crop.js:17–18` reads the size as `max.x + max.width` and
  `max.y + max.height`. A null `max` gives `null`, non-numeric strings concatenate to a string, and missing fields give
  `NaN`. Each of these still hits the throw. The harness returns the observed `{x:3000,y:1993,width:2981,height:1994}`
  against a 5981×3987 crop, so C1(a) also fails on the r1 field read (2981 ≠ 5981 → skipped). C1(b)(2)
  (Unconstrained at 3000×2000) still makes one `{apply:false}` call and skips. C1(c) (`null_max`) still throws with no write.
  The array form was dropped. r1 allowed that, because it was never observed.
- **Seam read:** `JxaScripting::apply_crop_if_untouched` parses the answer as `serde_json::Value` and matches on
  `outcome`, with `ratio` checked only for `applied`. A `null` ratio in `skipped`/`unknown_ratio` answers, which is now
  reachable, cannot break deserialisation.

## Live row C6(a), re-run by the reviewer
Run on 2026-10-29 against Capture One 16.8.6.25 with `session test 1`. Scripts were generated from the fixed tree as
`JxaScripting::new` writes them (`lib.js` + `\n` + script) into `/tmp/p23r2live.R5Jl`.
`R = …/PSM_PROBE_A/session test 12199.ARW`, `V = 130`.
1. `crop_aspect_ratios.js` printed the 11 names, Square included ✓. Pre-reading of V: `[null,5981,3987]` (variant 131 the same).
2. `apply_crop.js … 130 Square` printed `{"outcome":"applied","ratio":"Square","width":3987,"height":3987}` ✓.
   Read-back: `["Square",3987,3987]`.
3. The same call again printed `{"outcome":"skipped","ratio":"Square","width":3987,"height":3987}` ✓. As a variation
   that would make a rewrite visible, the same call with `4x5` printed the same `skipped`/`Square` answer, and the crop was
   unchanged.
4. Restore with `maximumCrop(v)` gave `[null,5981,3987]`, identical to the pre-reading ✓. The row text expects `""`; that
   is the evidence error already in backlog, and the restore criterion "identical to before" holds.
5. `maximumCrop(v,{apply:false})` returns the object form and does not apply (recorded in r1; the applied/skip pair above
   confirms the decoding end to end).
The cycle ran twice: literal step 3, then the `4x5` variation. Both ended restored.

## Triage
| Item | Class | Why |
|---|---|---|
| none | — | No blocking item remains after fix 1. |

## Remaining findings
None blocking and no new notes. Carry-forward: r1's 6 notes are in `docs/implementation/backlog.md` (rows 161–166: H7 text
in core, the C2(c)(1) photo-row assertion, the C2(a) echo clock, the Health-page re-queue, the unused fake timeout, and the
evidence's `""`/rectangle decoding). They stay there for the backlog owner. Nothing evaporates.

## Lessons for the plans
- A live row with a literal expected value that came from flawed evidence (C6(a)(4) `["",5981,3987]`) goes stale once the
  evidence is corrected. Prefer "identical to the pre-probe reading" over a typed literal for restore checks.
- Step (3)'s "the same call again" cannot show a rewrite when the rewrite writes the same value. A different ratio name
  makes the no-write claim observable.

## Probe declaration
- **Production code and repo files:** never edited (no probes in the repo this round).
- **Outside the repo:** `/tmp/p23r2live.R5Jl/` (generated `apply_crop.js`, `crop_aspect_ratios.js`,
  `find_image_in_folder.js`, plus r1's `read_crop.js`), `/tmp/p23r2_rust.txt`, `/tmp/p23r2_clippy.txt`,
  `/tmp/p23r2_tc.txt`, `/tmp/p23r2_ui.txt`.
- **Capture One state:** `session test 1`, variant 130 of `PSM_PROBE_A/session test 12199.ARW` was set to Square twice and
  restored with `maximumCrop(v)` each time. The final reading `[null,5981,3987]` equals the pre-probe reading. Variant 131
  (`12200.ARW`) was read only and still reads `[null,5981,3987]`. No captures were fired and no files were written in the
  session folder.

## Write perimeter
- `docs/implementation/master_plan.md`: P23 tracker row → `APPROVED`.
- `docs/implementation/plans/P23_shape_on_arrival.md`: Review log entry (re-review r2).
- `docs/implementation/narratives/P23_shape_on_arrival.md`: new.
- This handoff.

No backlog rows were added. Nothing is committed (the runner commits).
