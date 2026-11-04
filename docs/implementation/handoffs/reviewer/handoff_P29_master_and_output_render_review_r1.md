---
plan: P29_master_and_output_render
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-04
actor: opus (pipeline-reviewer)
---
# P29 master and output render — review r1

APPROVED_WITH_NOTES. The 27 rows hold at their boundaries, each with a test that fails on the wrong outcome. The perimeter is
exactly the declared 55 paths. Suite, lint and typecheck are green on the reviewer's run. No blocking finding; 4 notes went to
the backlog.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Verified correct
- **Gate and perimeter.** Tracker `IMPLEMENTED`, checkpoint `1d43c22` present. `git show --stat 1d43c22` = the 55 paths the
  implementer handoff declares (core 10, infra 10, integrations 5, testkit 5, tests 9, desktop 14, `Cargo.lock`, tracker, plan),
  none extra or missing.
- **Build (reviewer run on clean `b78cbae`, which differs from the checkpoint only in pipeline docs).** `cargo test --workspace
  --no-fail-fast` exit 0, 508 passed / 0 failed. `cargo clippy --workspace --all-targets -- -D warnings` clean. `cargo fmt --all
  -- --check` clean. `npm run typecheck` clean. `npm test` 21 files / 132 passed.
- **Render contract (`crates/infra/src/imaging/render.rs`, `linear.rs`, `metadata.rs`).**
  - The inverse index maps for turns 1/2/3 are the exact inverses of the contract's forward maps, and the flip is applied in the
    turned frame.
  - Straighten samples `s = c + R(−θ)(p − c)` at pixel centres, with `c = (W_t/2, H_t/2)`. Catmull-Rom is used only when
    `θ ≠ 0`; `θ = 0` is a plain copy.
  - The sRGB→linear 65 536-entry LUT is applied before any resampling. Lanczos3 runs over U16x3 only when the long edge exceeds
    the setting, with short edge `round(long·short/long)`, and never upscales.
  - The linear→sRGB 8-bit LUT is followed by one `jpeg-encoder` pass with `SamplingFactor::F_1_1`, an ICC v4.3 sRGB profile in
    APP2 and the APP1 built from a fixed field list. That list holds 9 structural tags plus at most Make, Model,
    DateTimeOriginal (fallback DateTime) and Copyright, so GPS, MakerNote and XMP cannot be emitted.
  - The temp file is renamed into place, and the master is opened read-only.
- **Export rework (`crates/core/src/services/export.rs`).**
  - Order of `start`: integrity (MC-3), then the needs_review gate (before any run row or staging, emits `Failed`), then
    targets `(photo, slot)` named through `output_names`, then per target: a cached output is copied, a master on disk gets
    `render_output`, otherwise one master `process` per RAW (`state.masters`).
  - `master_ready`: `masters.commit` (temp + rename, other generations deleted), then `dimensions`, then
    `adopt_frame(Master, geometry)`. Flagged photos fail their runs with `needs_review` and the master is kept. Otherwise the
    RAW's waiting targets are queued.
  - `finalize` writes `manifest.json` before `item.json`, swaps, then releases masters only when `!defer_swap` and the session
    is not dirty, skipping RAWs another run still needs.
  - `RenderOutputJob` names the cache file by the revision it actually read, and `output_ready` records that revision in the
    manifest.
  - A settings save rebuilds the runtime, so the export's `format` is always the stored one (C6(b)).
- **Frame adoption (`sqlite/store.rs adopt_frame_tx`).** A compatible master observation replaces the fingerprint (task 0). A
  preview after a master stays compatible and leaves the master fingerprint alone (C8(d)). An incompatible master sets
  `needs_review` with reason `geometry` or `frame` (C8(c), C8(d)).
- **Settings.** `output_format` is valid only as an object with integer `long_edge` 512–8192 and integer `quality` 60–100. A
  `web_recipe` patch → `InvalidValue("web_recipe")`. A legacy key is ignored on read (`#[serde(default)]`) and dropped on save.
  `RecipeSpec::WEB`, the ensure entry and the adapter field are gone. DTOs and bindings carry `output_format`. The Settings
  screen has two spinbuttons.
- **Rows → tests (each read, each fails on a wrong outcome).**
  - C1(a) asserts the exact issued list and the RAW SHA-256 values.
  - C1(b) and C7(c) assert no `process` and no new `render_output` row.
  - C1(c) asserts that `master-g` is gone and `master-g+1` exists before finalize, and the exact output file name.
  - C2(a): a crop off by one pixel moves the dark columns and rows, so it fails.
  - C2(b): flip before turn → rows 498/499, so it fails.
  - C2(c): the opposite sign puts row 0 at [81,84], so it fails.
  - C2(d) asserts the exact frame, rect (±1e-6), state and revision.
  - C2(e), C4(a) and C4(b) assert exact sizes.
  - C3(a): a shared or missing column makes left 498 or right 1 non-grey, so it fails.
  - C4(c) parses the ICC `desc`, the SOF0 sampling bytes and the single SOI/EOI.
  - C4(d): a gamma-space average (≈128) fails the [178,196] window.
  - C4(e) and C4(f) assert exact values and patches.
  - C5(a) and C5(b) check the fixture first, then the allowlist over every field.
  - C6(a) and C6(b) assert a single exact cache file name. C6(c) asserts the exact manifest object.
  - C7(a) and C7(b) check that masters are gone or kept.
  - C8(a)–(d) assert exact labels, the command error code and data, states and fingerprints.
- **Assumptions (flow §6).** None of the 15 contradicts the intention.
  - A7 (Capture One's copy of the master is deleted from `Output/_session_manager/` after the commit) touches no RAW (H2) and
    matches the plan's manual try-out ("fill, then empty").
  - A6 (prune keeps the other slot of the same key) is the only reading under which the cache key works for splits.
  - The deviations (C3(a) `h` adjusted inside P28's tolerance, C8 flagging through a geometry change, C6 on 3000×2000 masters)
    keep each row's asserted outcome. The rest are told to the owner in the narrative.
- **Hard constraints.**
  - H1: no platform write.
  - H2: the RAW is only hashed (read). The master and RAW are never written.
  - H3: the pixel operations are exactly MC-28's list.
  - H4: `render_output` is deterministic and idempotent, and a row from an earlier process finds no run and closes.
  - H5: Capture One is still driven only through its recipes and `process`.
  - H6: local export works without an integration.
  - H7: core names no Capture One or Shopify concept (`RenderPurpose::Master`, generic ports).
  - H8: the desktop change is a DTO and one error-code arm.
  - No secret in the diff, and the suite uses fakes only.

## Findings
None blocking (B1–B4 not met). Over-engineering test applied to the new tests: they assert produced files, stored state and
fake calls at the row boundaries, with no internal call counts or private helpers.

## Notes written to backlog
1. **P29 → P30.** `RenderOutputJob` does not re-check the composition's `state`. A photo flagged `needs_review` between
   `master_ready` and its render is still rendered and exported (M33 window of seconds; P30's publish shares the path).
2. **P29.** `master_ready` propagates `dimensions(..)?` and `adopt_frame(..)?` errors after removing the RAW from
   `state.masters`. Waiting runs then never finish, and `export.run` returns the stuck run until restart.
3. **P29 (not verified live).** No real `PSM Master` render has run yet. The 16-bit path, a real master's EXIF and the deletion
   of Capture One's copy are proven on synthetic masters only (the implementer's probe used an 8-bit Capture One TIFF).
4. **P29 (P27 note still open).** `RecipeSpec::props()` still writes `tiff_compression` into `PSM Preview`.

Carry-forward: notes 1 and 2 → P30 (it reuses the export path for publish); note 3 → the owner's try-out step 2; note 4 → whichever
phase next touches recipes.

## Lessons for the plans
- C8(a)'s input "an incompatible second preview 2560×1700" is compatible under P28's own C5(b) rule (preview sizes are never
  compared). Rows that need a `needs_review` photo should name a Capture One geometry change.
- C3(a)'s `h = 0.831946755` misses P28's split tolerance by 5e-7 px. Compute split rects from the validator's own inequality.
- The C6 chain on 600×400 masters could not show a 1024 px format change, because outputs are never upscaled. Rows about output
  size need masters at least as large as the format.

## Write perimeter
`docs/implementation/handoffs/reviewer/handoff_P29_master_and_output_render_review_r1.md` (this file),
`docs/implementation/narratives/P29_master_and_output_render.md`, `docs/implementation/master_plan.md` (P29 tracker row only),
`docs/implementation/plans/P29_master_and_output_render.md` (Review log entry), `docs/implementation/backlog.md` (4 rows appended).
No production or test file touched. No probes (flow §4). Build artefacts only under `target/`. Scratch output in
`/tmp/p29rev_*.txt`. Nothing committed.
