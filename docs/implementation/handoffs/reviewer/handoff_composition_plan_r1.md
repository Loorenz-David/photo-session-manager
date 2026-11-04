---
plan: composition
role: plan
round: 1
state: DONE
date: 2026-11-04
actor: Claude Fable 5.1 (pipeline-planner, implementation-planner doctrine under docs/process/flow.md)
---
# Composition and framing (book 10D) — planning handoff, round 1

**Summary.** Gate passed: `docs/intention/intention.md:3` reads **RATIFIED** (owner, 2026-10-24); the mechanism inventory is done
(`handoffs/reviewer/handoff_intention_inventory_r1.md`); book 10D (MC-27…MC-29, M33–M35, R37, changelog 27) is committed at `9e64201` and
amended at `069eca4` after the orchestrator's live VERIFY-15 (crop-ignored master works; master **uncompressed**; fingerprint = Capture One
geometry + orientation + master size). I read `git show 9e64201` and `069eca4`, 10D in full, 01/05/05A/06/07A/07B/10/10A/10C/11/12/14/15/16,
the evidence (VERIFY-15 result, recipe read-back, crop by script), the design handoff, flow.md, the master plan, STATUS, the P23–P26 plans and
handoffs, and the code at `d763492` (every path and symbol cited in the plans was verified there; line numbers are stated as of that commit).
**Eight plan files written** (P27–P34; 58 criteria / 166 rows / 19 `[rule]` rows in total — counts derived from the files), `phases.txt`
appended, the master plan amended (incl. a new §6.10 composition skeleton). Not committed (as instructed). **No owner card.**

## ⚠ OWNER DECISIONS REQUIRED (0)
Nothing needs the owner. Every open point is either answered in the book (incl. the 069eca4 amendment), a planner reading recorded in a plan's
Notes and surfaced to the owner through the narratives' "Choices made for you", or a delegated choice. The one owed observation (which EXIF
tags Capture One writes into the TIFF master — P29 Notes) is operational: the orchestrator can read one master with `sips -g all`/`exiftool`
before P29 is dispatched; without it the metadata row still holds (an absent tag is simply not copied).

## Write perimeter
- `docs/implementation/plans/P27_retire_shape_on_arrival.md` (new) · `P28_composition_foundation.md` (new) · `P29_master_and_output_render.md`
  (new) · `P30_outputs_in_export_and_publish.md` (new) · `P31_compose_mode.md` (new, `implementer: fable`) · `P32_composition_tools.md` (new,
  fable) · `P33_split_ui.md` (new, fable) · `P34_composition_polish.md` (new, fable).
- `docs/implementation/phases.txt` (eight lines appended with `printf >>`, run order P27 → P34).
- `docs/implementation/master_plan.md` (§2 sources: 10D, 10B retired, M1–M35; §3 trace ranges M1–M35 / MC-1…MC-29; §4 eight NOT_STARTED rows;
  §5 count 34, graph tail, `depends_on`, the composition-order and implementer notes, VERIFY-15 recorded; §6.1 new modules; §6.2 composition
  crates; §6.3 storage layout; §6.4 port changes; §6.5 migrations 003–005; §6.6 removals and 10D enums/errors/job kinds; §6.7 commands; §6.8
  `default_shape`/`master_recipe`/`output_format` + legacy mapping; §6.9 shared case table; **new §6.10**; §7.5 harness; §9 log entry).
- `docs/implementation/handoffs/reviewer/handoff_composition_plan_r1.md` (this file).
- **Not touched:** `docs/intention/**`, `STATUS.md`, `backlog.md`, narratives, other plans, any code, any tool-recorded state. No commit, nothing
  pushed, no pipeline run, no live Capture One or Shopify call.

## The phase map (five lines each)
**P27_retire_shape_on_arrival** (Codex, 6 criteria / 14 rows). Removes the Capture One crop write of P23 (job, scripts, adapter methods, health
items, command, Settings select, migration 003 deletes leftover job rows); `default_shape` and `master_recipe` replace `crop_on_arrival` (legacy
file mapped once: `Square → 1:1`, `off → original`); PSM Preview becomes 2560 px q85 crop-ignored, PSM Master (16-bit TIFF, uncompressed, crop
ignored, disabled) is provisioned; PSM Web stays this phase so export keeps working. Owner can try: new photos arrive uncropped and Capture One's
recipes show the new values; Settings shows "Default shape" and "Master recipe". ~30 files, net-negative Rust, ~250 lines of tests.
**P28_composition_foundation** (Codex, 7 / 25). The `compositions` table (migration 004), the geometry module (default rect, validation, pixel
rule, fingerprint) on a shared JSON case table, the default composition in the arrival transaction (rect materialised at the first crop-ignored
preview), `composition_set`/`composition_reset`, 640 px thumbnails, the geometry read with every render and `needs_review` with its reason,
`PhotoResponse.composition`. Owner can try: previews re-render once; a Capture One straighten flags the photo for review (visible in the DB; the
editor comes later). ~28 files, ~1 650 lines.
**P29_master_and_output_render** (Codex, 8 / 27). PSM Master rendered on demand and cached, the Rust `render_output` job (linear light, turn/flip/
straighten, pixel rect, Lanczos3, one JPEG q90 4:4:4 sRGB with EXIF allowlist), the output cache key, `output_format` setting, local export through
the app's render, PSM Web dropped, export refusal while `needs_review`, masters released after export. Owner can try: Export produces square 2048 px
JPEGs rendered by the app; a second export renders nothing. ~26 files, ~2 100 lines.
**P30_outputs_in_export_and_publish** (Codex, 8 / 20). Outputs per slot in export and publishing (migration 005: `photo_links` slot and
`composition_rev`), `-L/-R` upload names, replacement on composition revision, retirement on Normal ↔ Split, gallery/cover/damage report per
output, publish refusal while `needs_review`, staging reuse only by manifest, masters released after publish. Owner can try: publish to the test
store; (via SQL for now) a split photo becomes two adjacent media. ~16 files, ~1 100 lines.
**P31_compose_mode** (Fable, 8 / 27). The editor's stage: frame fixed, photo moves; drag / pinch / wheel through one pure state machine with
16 enumerated rules (second pointer, wheel during drag, pointer cancel, blur, hidden, Esc mid-gesture, photo switch, late saves); one CSS
transform write per frame; autosave latest-wins per photo; Esc revert; Return/Done; workspace keys, page zoom and back-swipe off; right browser
with edge reveal (never during a drag); tiles draw the composition over the thumbnail. Owner can try: compose with R, drag, pinch, swipe. ~14
files, ~1 900 lines.
**P32_composition_tools** (Fable, 8 / 25). The left Tools panel (fold, remembered, ⌘T), Shape menu, Turn, rotate, flip, straighten, trim handles
that glide back, nudge/shape/[ ]/⌘⌥R keys, Centre/Reset/Done, the Re-frame confirmation (banner per reason), the refusal toasts. Owner can try:
the whole Normal editor. ~16 files, ~2 000 lines.
**P33_split_ui** (Fable, 6 / 15). Split in the editor (S, 2:1 frame with middle hairline, Left/Right previews) and the grid (03–04 labels,
hairline, output numbering, menu Split/Normal incl. selection). Owner can try: split a photo, export/publish two squares. ~12 files, ~900 lines.
**P34_composition_polish** (Fable, 7 / 13). Z loupe, snapping (⌥ suspends), copy/paste and apply-to-selected (`composition_apply`, the one small
backend piece), unsaved dot, reduced motion, 1440×900 window, the frame meter and render timings reported in the handoff. ~18 files, ~1 100 lines.

## Deviations from the owner's phasing (approved latitude: "you size and name them")
1. **Foundation split in two** (P27 retire + P28 compositions): one phase would have run to 12+ criteria and ~45 rows; the retirement is
   mechanical and self-contained. **Interim rule:** PSM Web survives P27 so export never breaks between phases.
2. **Outputs before the editor** (P29/P30 before P31/P32). Reason (master plan §5): from P29 on every exported or published file is the app's
   render of the (default, square) composition — continuity with today's square-on-arrival result; the editor-first order would leave two phases
   in which saved compositions are silently ignored by the files (exactly the "silent wrong crop" M33/M34 guard). P31 depends on P28 only, so the
   orchestrator may swap `phases.txt` without touching the plans if the owner prefers to try the editor first.
3. **Outputs split in two** (P29 render, P30 publish) and **the editor split in two** (P31 compose mode, P32 tools): each ≤ 8 criteria (flow §9,
   charter sizing lever). Split (P33) is frontend-only because P29/P30 implement slots generically.
4. **Fable phases:** P31–P34. P27–P30 are backend plus two Settings form fields (a select and two number inputs — form fields, not interaction
   logic), left on Codex as P23/P25 were. P34 mixes in `composition_apply` (≈ 20 lines) and is marked fable rather than split.

## Grounding worth knowing (orchestrator)
- **VERIFY-15 is consumed, not re-run.** P27's "read before task 1" section quotes the recorded spellings and the uncompressed decision;
  `MASTER_TIFF_COMPRESSION = "Uncompressed"` is a named constant (no fill-in needed any more). P28's fingerprint follows the amended MC-27: the
  adapter reads `adjustments.{rotation, flip, keystoneVertical, keystoneHorizontal, keystoneAmount}` in the same `process` script run; compatible ⇔
  geometry equal, orientation equal, master size equal when both are masters; preview pixel sizes are never compared (VERIFY-15: a Capture One
  straighten keeps 3:2 and shrinks). `review_reason` (`geometry` | `frame`) drives the notice "Changed in Capture One: straighten here instead".
- **One planner reading beyond the book, recorded in P28 Notes:** the default rect is stored `NULL` until the first crop-ignored render (its
  fractions depend on the frame's ratio), and pre-10D previews are marked stale once at the first real start so a crop-ignored, geometry-carrying
  frame arrives; neither bumps `change_seq`. `frame_unknown` refuses a save before that (the editor cannot open then anyway).
- **Legacy settings mapping** (P27 Notes): `crop_on_arrival: off → default_shape original`; everything else → `1:1` except `4x5/2x3/16x9`.
- **Owed observation:** which EXIF tags Capture One writes into the TIFF master (P29 metadata allowlist: Make, Model, DateTimeOriginal, Copyright).
  One `sips -g all`/`exiftool` read of a master before P29 settles it; otherwise the rows still hold (absent tags are not copied).
- **Discriminating rows to keep an eye on:** P29 C4(d) (a 1-px checkerboard downscaled 2× must read ≈ 187, not 128 — the only row that fails a
  gamma-space pipeline), P29 C3(a) (the column marker proves the split halves share no column), P29 C2(c) (the straighten sign), P31 C1(c) (the
  16 machine rules), P30 C8(a) (a stale staging after a composition change is never uploaded).
- **Tests that change their expectation** are named in each plan's Read-first (P05 `c5c` three jobs, P17 fixture, P25 `c7a` deleted, P23 tests
  deleted, P00 `web_recipe`, P27 C3(b)/(c) inverted by P29, P12 `staged_uploads_create` call sites, frontend settings and `Photo` fixtures).
- **Frontend test harness facts:** jsdom polyfills for `PointerEvent`/`ResizeObserver` exist in the p22 harness; the plans fix the stage box
  (`1000 × 700`) and the scale (`0.381956`) so every expected value is hand-computed, never read from the implementation.
- **Crate versions** verified on crates.io 2026-11-04 (master plan §6.2): `image 0.25.10`, `fast_image_resize 6.1.0`, `jpeg-encoder 0.7.1`,
  `kamadak-exif 0.6.1`; delegated `imageproc 0.27.0`, `little_exif 0.6.23`.

## Choices the owner may want to know (recorded in the plans' Notes; for the narratives)
1. A settings file that still says "Leave as shot" now means "default shape: Original (the whole frame)"; "Square" means 1:1.
2. Masters are deleted after a completed export or publish of a session that is not dirty; a later export renders them again (≈ 2 s per a7R photo).
3. A composition change on a deleted photo never dirties a session; on any visible photo of a published session it does.
4. Arrow keys move the frame over the photo in the arrow's direction (the Capture One / Photoshop crop-box convention).
5. Changing shape, turning or rotating keeps the framing's centre and area (a shape change is not a reset).
6. With a straightened photo, a drag that would show an empty corner stops at the edge instead of sliding along it (smoothing is a polish candidate).
7. The window opens at 1440 × 900 with a 1100 × 720 minimum.

## Lessons for the planner
- When an external observation is scheduled after planning starts, write the plan against the recorded result the moment it lands (the
  fingerprint rule changed from "aspect within 0.2 %" to "geometry values" mid-session; three plans had rows to rewrite).
- A frontend plan's expected numbers must come from a fixed layout (stage box, scale) so the rows never depend on the implementation's own
  functions — otherwise every row is `assert f(x) == f(x)`.
