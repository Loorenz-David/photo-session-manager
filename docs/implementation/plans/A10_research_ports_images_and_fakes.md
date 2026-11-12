---
phase: A10_research_ports_images_and_fakes
title: The doors research uses to reach the outside — one for the model, one for visual search, one for reading a page — and the only kind of picture allowed through them: a reduced, metadata-free copy of the photo as you framed it
difficulty: high
projection: yes
depends_on: [A06_research_cost_and_call_rules]
---
# A10 — Provider ports, the research image, its renderer and store, and the fakes (08A §4; 08B MC-A5 source and type guard; HA3, HA5)

## Goal
Define the three provider ports and their neutral shapes; the `ResearchImage` value that only the research-image
renderer can make from an eligible photo; the renderer itself (the amended H3: the composition rule applied to the
adjusted preview, resized, encoded once, metadata removed); the file store for the copies; and the testkit fakes
every later phase scripts. **Not in this phase:** recording calls and outbound rows (A11), any real adapter
(A15–A17), any run.

## Read first
`master_plan.md` §6.1, §6.3 (data dir layout), §6.4 (`Imaging`), §6.10 (composition values, pixel rule), §7.5 ·
`master_plan_research.md` §3 (HA5 in code), §6.1, §6.6, §7.5 · book `01` (**H3 as amended**, H7), `14` (MA3, MA12),
`08A` §4, §5 (Photos used), `08B` **MC-A5** (Eligible photo, Source, Type guard), `10D` MC-27 (pixel rule), MC-28
(render contract) · code `crates/core/src/ports/images.rs` (`Imaging::render_output`), `crates/infra/src/imaging/`
(`render.rs`, `linear.rs`, `metadata.rs`), `crates/infra/src/media_store.rs`, `crates/core/src/ports/store.rs`
(`preview_state`, `RawPreviewState.adjusted_preview_path`, `adjusted_gen`), `crates/core/src/domain/composition.rs`,
`crates/testkit/src/{test_app.rs, fixtures/}` (`SyntheticJpeg`, `complete_render_with`).

## Dependencies and gate
Tracker row and gate: `master_plan_research.md` §4. Notes: `backlog_research.md`.

## Files expected to change
New: `crates/core/src/ports/research.rs`, `crates/infra/src/imaging/research.rs`, `crates/infra/src/research_files.rs`,
`crates/testkit/src/fake_research.rs`, `crates/testkit/tests/suite/research/a10_research_ports_images_and_fakes.rs`
(+ `mod` line).
Edited: `crates/core/src/ports/{mod.rs, images.rs}` (`Imaging::render_research`, `NoImaging`), `crates/core/src/domain/
research.rs` (request and reply shapes), `crates/infra/src/{lib.rs, imaging/mod.rs, paths.rs}` (`research/images`),
`crates/testkit/src/{lib.rs, test_app.rs}`, own tracker row.

## Core mechanisms (unit tests allowed)
None new. The pixel rule is `psm_core::mechanisms::composition` (existing, already tested).

## Ordered tasks
1. Ports and shapes as in `master_plan_research.md` §6.6. All strings naming a model, an effort or a provider are
   opaque to core. `Exchange` is what an adapter hands back for the record; `CallFailure` is A06's type.
2. `ResearchImage {photo_id, sha256, long_edge, path, bytes_len, source_gen, composition_rev}` with **private
   fields and no public constructor**; `ResearchImages` (port) is the only maker:
   `render(photo_id, run_id, long_edge) -> Result<ResearchImage, NoSource>`.
3. The implementation (infra + a thin core service function): read the photo row; it is **eligible** iff its
   session is the run's session, that session's item is the run's item, and its classification is `main`,
   `condition` or `unreviewed`; take the adjusted preview of its current `adjusted_gen` (none → `NoSource`; the
   original camera preview is never used); apply the MC-27 pixel rule for the current composition revision (turn,
   flip, straighten, cut); resize so the long edge is `min(long_edge, the cut's long edge)` (never upscale); encode
   once as JPEG quality 85, sRGB, with no EXIF, GPS, maker notes or other metadata; store at
   `research/images/<sha[0..2]>/<sha>.jpg`. Within one run the same `(photo, source generation, composition
   revision, long edge)` returns the stored copy.
4. Fakes (`fake_research.rs`): `FakeModel`, `FakeVisualSearch`, `FakePageReader` — replies scripted per stage and
   unit, a log of every request received (each image by SHA-256 and long edge), a queue of `CallFailure` faults,
   and a `before_call` hook (a closure run when a call arrives, before the reply) used by later phases to observe
   the database and to stop a run at a chosen call. `TestApp::with_research()` and
   `TestApp::add_main_photo_with_preview(session)` (§7.5).

## Criteria (5 criteria, 13 rows)
Boundary: `ResearchImages::render` on a `TestApp` (real store, real infra imaging, temp data dir); outcome = the
returned value and the file on disk, decoded with the testkit's `decode`.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | render | a 3000 × 2000 adjusted preview with corner markers, default composition; long edge 1024 | a JPEG 1024 × 683 (±1 px on the short edge) showing the whole frame (the four markers in their corners) | MC-A5 |
| C1(b) | render | the same preview, a composition cutting the right half, one quarter turn | the copy shows only the right half's markers, turned | MC-A5 (H3) |
| C1(c) | render | an 800 × 600 preview; long edge 1024; long edge 2048 | both 800 × 600 (never upscaled) | MC-A5 (R-IA13) |
| C1(d) | render | a 6000 × 4000 preview; long edge 2048 | 2048 on the long edge | MC-A5 |
| C2(a) | metadata | a preview carrying EXIF with camera model, serial number, GPS and a maker note | the copy has no EXIF segment (the testkit's EXIF reader finds none) and no GPS data | MA12 (R-IA13) |
| C3(a) | eligibility | a photo classified `deleted` | `NoSource`/refused; no file written | MA3 |
| C3(b) | eligibility | a photo moved to another session after the run started | refused; no file written | MA12 |
| C3(c) | eligibility | `main`, `condition`, `unreviewed` photos of the run's session | each rendered | MC-A5 |
| C3(d) | source | an eligible photo with an original camera preview and no adjusted preview | `NoSource`; no file written | MC-A5 (card 3) |
| C3(e) | type guard | code outside the port's implementation that builds a `ResearchImage` from a path | does not compile (one `compile_fail` doc test on the type) | MA12 |
| C4(a) | one copy | the same photo rendered twice in one run at 1024 | the same SHA-256 and path; one file | MC-A5 |
| C4(b) | new copy | the photo recomposed (revision + 1), rendered again | a different SHA-256; both files present | MC-A5 |
| C5(a) | `FakeModel` | a scripted reply for (observe, photo 7) and a queued transient fault; two calls with a rendered image | the first call returns the fault, the second the scripted reply; the log holds two requests, each naming the image's SHA-256 and long edge 1024 | MA12 (the record A11's rows read what was sent from) |

## Delegated choices
Whether the eligibility check lives in a core function called by the infra renderer or in a small core service ·
the JPEG encoder call (`jpeg-encoder` as P29) · chroma subsampling · the hook's exact signature. Not delegated: no
public constructor, no path or byte constructor, never the original preview, never the master.

## Manual try-out
Nothing visible yet. `cargo test -p psm-testkit --test suite research::a10_research_ports_images_and_fakes::` shows
that a research copy is the framed photo, small, with no camera data in it.

## Notes
- H3: this is the only new pixel code of the research work, and it is the one the owner allowed (card 3). It must
  not decode a RAW or a master, and nothing but a research copy may be produced by it.
- Rule 17: "no EXIF" is observed with `kamadak-exif` (a dev-dependency of the testkit); the projection confirms
  what that reader returns for a JPEG without an APP1 segment before C2(a) is implemented.
- Rule 11: the guard is a property of the port's type, not of a caller's discipline; A11's gateway accepts only
  `ResearchImage` values.
- Review mode: this phase's module, `p28_composition_foundation::`, `p29_master_and_output_render::`, clippy, fmt.

## Review log
