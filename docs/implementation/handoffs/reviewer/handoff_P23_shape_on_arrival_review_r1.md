---
plan: P23_shape_on_arrival
role: review
round: 1
verdict: CHANGES_REQUESTED
state: DONE
date: 2026-10-29
actor: opus
---
# Review P23_shape_on_arrival (round 1)

**Verdict: CHANGES_REQUESTED.** Two blocking findings (B1), both in `apply_crop.js`, both found by
running live row C6(a) on this Mac. The automated suite is green and every automated row holds.
On the real Capture One, though, the script reads every new photo as already cropped by hand and
skips it, so no photo gets its shape.

## Gate
- Tracker row `IMPLEMENTED`. Checkpoint `0476e661d60a2e79e4507be143fcb85eb2c52d5f` is in `git log`.
- `git show --stat 0476e66`: 39 paths, equal to the handoff's declared list. No extra or missing paths.
  `commands/items.rs`, `p05_capture_arrival.rs` and `p17_real_run_fixes.rs` are outside the plan's
  list but are declared under Deviations and justified (exhaustive error match; the new fourth
  arrival job; the render-retry fixture set to `off`).

## Verified correct
- **Stamp (re-run by the reviewer on the checkpoint tree):** `cargo test --workspace` gives 432
  passed with exit 0. `cargo clippy --workspace --all-targets -- -D warnings` exits 0.
  `cargo fmt --all -- --check` exits 0. `npm run typecheck` exits 0. `npm test` gives 17 files and
  127 passed.
- **C1(a–c)** (Node harness over the generated `apply_crop.js` and `crop_aspect_ratios.js`). Against
  the plan's modelled values, the order is ratio-list check → current-ratio check → one
  `maximumCrop({apply:false})` → size comparison → write → read-back. Unknown name: no
  `maximumCrop` and no write. Unreadable maximum: throws. The tests assert `calls` and `writes`
  exactly, so they fail if the order or the guard changes. *The modelled values are wrong for the
  real Capture One (F1, F2).*
- **C2(a)** Hook arrival enqueues exactly one `crop_on_arrival` job in the arrival transaction
  (order `original_preview, crop_on_arrival, hash, render_preview`). The adapter calls
  `mark_own_write` before the scripting call, and the test asserts `last_own_write_at == Some(0)`.
  The echo hook keeps `retouch_gen == 0` and leaves `change_seq` alone. `on_variant_adjusted`
  counts a retouch only outside the 30 s window, so the test discriminates.
- **C2(b)** Import goes through `accept`, so it enqueues. Hand-cropped variant 12 is skipped. Restart,
  scan and re-import add no rows. `set_crop:` ops are exactly `["set_crop:11:Square"]`.
- **C2(c)** `off` enqueues nothing. Reviewer probe (below) confirmed that the photo row still exists
  and that the same rebuild with `4x5` enqueues and applies. Hook plus watcher make one job.
  Copy and move go through `photo_insert_or_get` / `copy_photo` and enqueue nothing. Structurally,
  `arrival_insert` skips the job when any `photos` row for the RAW existed in any session
  (`raw_seen`) or a `crop_on_arrival` job exists for it. So "never for a RAW with a publish link"
  holds, because such a RAW always has an earlier row.
- **C3(a)** `VariantNotFound` retries after 1 s with `variant_not_found` in Health. The folder
  fallback then applies Square and clears the item.
- **C3(b)** An attempt that fails first and is then applied marks the RAW stale through
  `PreviewService::on_variant_adjusted` (non-retouch inside the window). A second
  `process(16, PSM Preview)` follows after the delay with no hook.
- **C3(c)** `JobRunner::tick`: terminal ⇔ `CropRatioUnknown` or `attempts + 1 >= max_attempts()`.
  `job_fail` sets `failed`. Give-up is at `CROP_GIVE_UP_ATTEMPTS` (= 8, asserted by name). Health is
  counted first, then the jobs are re-queued, in both `health_check` and the real start (re-queue
  only when `start_backup` is set and the setting is not `off`). The text is `Shape not applied to
  1 photo`. A later success clears it on the next check.
- **C3(d)** An unknown name sets `crop_ratio_unknown` with the exact book text, both at the startup
  check and from the job. The job fails at once with no `set_crop:`, and the failure is excluded
  from the not-applied count. After the correction and a check, the job is re-queued and applied.
- **C4(a)** Missing key gives `Square`. `off` persists to `settings.json`. `""` and `5` give
  `InvalidValue` and the stored value is unchanged. The provider list has 11 names.
  `TestApp::new()` has no document, so the implementer used the fake document for the list;
  the assumption is reasonable and no outcome changes.
- **C4(b)** Native select labelled "Shape on arrival": `Leave as shot` first, then the adapter's
  names. A stored name missing from the list stays selectable. A failed fetch gives an empty list
  and no alert. The save patch carries the choice.
- **C5(a)** `.thumb-image` is `object-fit: contain`, `.stage-layer` is `contain`, and the 4:3 tile is
  kept. Reading the file with `fs` instead of `?raw` is a declared deviation; same boundary.
- **C5(b)** Both Health codes render under "Shape on arrival" with their exact details.
- **H1–H8:** H2 holds (no RAW bytes touched; the crop is Capture One metadata). H3 holds
  (Capture One crops; the app sends a ratio name). H5 holds (dictionary only). H8 holds (the
  command is a thin `spawn_blocking` over the provider). H7: the names are opaque in core, and the
  only Capture One mention in core is the health text the plan dictates. That is a note, not a
  breach of the constraint's concept list. H1, H4 and H6 are untouched.
- **Assumptions** (handoff): fake document for the C4(a) list; ratios script searches images, then
  collections; native select; `cropWidth`/`cropHeight` comparator. None contradicts the intention.

## Live row C6(a), run by the reviewer (2026-10-29, Capture One 16.8.6.25, `session test 1`)
Scripts were generated as `JxaScripting` writes them (`lib.js` + `\n` + script) into
`/tmp/p23live.Fcyv`. `R = …/Capture/PSM_PROBE_A/session test 12199.ARW`, `V = 130` (from
`find_image_in_folder.js`).
1. `crop_aspect_ratios.js "$D"` printed `["Unconstrained","Original","Output","Square","1x2","1x3","2x3","3x4","4x5","5x7","16x9"]`. ✓
2. Pre-probe reading of variant 130: `[null, 5981, 3987]`. Variant 131 (`12200.ARW`) reads the same.
   `apply_crop.js "$D" "$R" "$F" 130 Square` printed
   `{"outcome":"skipped","ratio":null,"width":5981,"height":3987}`. ✗ Nothing written; the reading
   after was still `[null, 5981, 3987]`.
3. Not meaningful while (2) skips.
4. A throwaway copy in `/tmp` ran the cycle set Square → `maximumCrop(v,{apply:false})` → `maximumCrop(v)`:
   - untouched: ratio `null`, 5981×3987, `crop()` = `{x:3001, y:1994, width:2980, height:1993}`
   - after `cropAspectRatio = "Square"`: ratio `"Square"` (read-back works), 3987×3987
   - `maximumCrop(v, {apply:false})` → `{x:3000, y:1993, width:2981, height:1994}` (object, not array). The Square
     crop is **unchanged** afterwards, so `apply:false` does not apply.
   - after `maximumCrop(v)`: `[null, 5981, 3987]`, identical to the pre-probe reading (restored).
5. Return shape: JS object `{x, y, width, height}`. Capture One's rectangle is
   `{centerX, centerY, width, height}` (sdef `crop`, `:1193`), and JXA decodes it as a QuickDraw rect, so the
   object's `x`/`y` are the centre and its `width`/`height` are `w − centerX` and `h − centerY`. True size =
   `x + width` × `y + height` (5981 × 3987 here). `crop()` uses the same decoding but with ±1 rounding
   against `maximumCrop`, so the two rectangles do not compare equal field by field.

## Findings

### F1: an untouched frame reads as a photographer's crop (blocking, B1)
- **route:** production
- **What is wrong:** `apply_crop.js` step (3) treats a ratio as untouched only when it is `''` or
  `'Unconstrained'`. On Capture One 16.8.6, JXA's `variant.cropAspectRatio()` returns `null` for an
  untouched variant, and again after `maximumCrop(v)`. Every new photo is therefore returned as
  `skipped`, and the shape is never applied live (C6(a) step 2 above).
- **Authority:** 10B MC-25 §What it does ("only if the crop is still the untouched full frame … it
  sets the variant's crop aspect ratio"); M31 (ch14); plan C6(a)(2) (`outcome "applied"`).
- **Correction:** treat `null`/`undefined` like `""` in the untouched check. Make the C1 harness's
  untouched variant read `null` (keep `""` and `Unconstrained` covered) so C1(a) fails on today's
  script.
- **Files the fix may touch:** `crates/integrations/src/editing/capture_one/jxa/apply_crop.js`,
  `crates/testkit/tests/p23_shape_on_arrival.rs`.

### F2: the maximum-crop size is misread, so the size check would still skip (blocking, B1)
- **route:** production
- **What is wrong:** step (4) takes `max.width`/`max.height` from the object form. Capture One's
  rectangle reaches JXA as `{x: centerX, y: centerY, width: w − centerX, height: h − centerY}`
  (observed `{x:3000, y:1993, width:2981, height:1994}` for a 5981×3987 frame). The comparison with
  `cropWidth()/cropHeight()` (5981×3987) is therefore never equal, and an untouched frame would be
  skipped at step (5) even with F1 fixed.
- **Authority:** 10B MC-25 §What it does ("the crop size equals the image's maximum crop"); M31; plan
  task 1 step (4), which says "the live row records which one Capture One returns". The plan made the
  shape conditional on this observation.
- **Correction:** read the maximum's true size as `x + width` and `y + height` when the object form
  arrives (keep the throw on non-numeric fields; the array form was not observed and may go). Change
  the C1 harness's `max` to the observed decoded object `{x:3000, y:1993, width:2981, height:1994}`
  with crop `5981×3987`, so C1(a) applies and C1(b)(2) (3000×2000) still skips.
- **Files the fix may touch:** `crates/integrations/src/editing/capture_one/jxa/apply_crop.js`,
  `crates/testkit/tests/p23_shape_on_arrival.rs`.

The fix session needs no Capture One: the observed values above are enough for the harness. The
re-review re-runs C6(a) live: step (2) must print `applied`/`Square`/3987×3987, step (3) `skipped`,
and step (4) must restore `[null, 5981, 3987]`.

## Notes written to backlog
1. H7: the health text naming Capture One is built in core (plan-directed); the adapter should supply it.
2. C2(c)(1) does not assert the photo row exists (probe confirmed that it does).
3. C2(a) delivers the echo at the tick's clock, not +1 s (still inside the window; discriminates).
4. The Health page visit re-queues failed shape jobs, including still-unknown-name failures.
5. The fake's `timeout_next_apply_crop` is unused; the shape job's timeout path is unexercised.
6. Evidence `capture_one_16.8.6.md` §"Crop by script" records `""` where JXA returns `null`. The live
   rectangle decoding and the `apply:false` behaviour are recorded for the correction.

## Lessons for the plans
- **An evidence value read through a different printer is not a JXA value.** The evidence's `""` for an
  untouched ratio was probably printed through a path that turns `null` into `""`. The plan, the
  fake and the harness all inherited it, and the green suite proved the model, not Capture One.
  When a plan grounds a comparison on an evidence value, the evidence should say which call printed it
  (`JSON.stringify` of the JXA return).
- **A live row that decides a shape the code depends on should not be optional.** C6(a) was marked
  optional and the implementer skipped it, so the defect reached review. For a phase whose core
  behaviour rests on an unobserved external shape, make the live row mandatory for whichever
  session has Apple events, before approval.
- `rectangle` results in this dictionary are `{centerX, centerY, width, height}` and arrive in JXA
  QuickDraw-decoded. Record this once in the evidence, for any later crop or bounds work.

## Probe declaration
- **Production code:** never edited.
- **Repo:** temporary test file `crates/testkit/tests/zz_p23_reviewer_probe.rs` (3 probes: `off` still
  creates the photo row; a rebuild with `4x5` enqueues and applies; a job queued before switching to
  `off` closes `done` with no `apply_crop:`; all passed). It was created, run and deleted, and
  `git status` was clean afterwards (before this review's own document edits).
- **Outside the repo:** `/tmp/p23live.Fcyv/` (generated scripts plus `read_crop.js` and `probe_cycle.js`), and
  `/tmp/p23_stamp.txt`, `/tmp/p23_rust.txt`.
- **Capture One state:** `session test 1`, `PSM_PROBE_A/session test 12199.ARW` variant 130 was set to
  Square, then restored with `maximumCrop(v)`. The final reading `[null, 5981, 3987]` is identical to
  the pre-probe reading. Variant 131 was read only. No captures were fired and no files were written in
  the session folder.

## Write perimeter
`docs/implementation/master_plan.md` (P23 tracker row only), `docs/implementation/plans/P23_shape_on_arrival.md`
(Review log entry), `docs/implementation/backlog.md` (6 rows appended), this handoff. No narrative, because
it is written only on approval.

## ⚠ OWNER DECISIONS REQUIRED (0)
None. The fix is fully determined by the live observations above.
