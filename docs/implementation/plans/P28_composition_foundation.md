---
phase: P28_composition_foundation
title: Composition foundation — the compositions table, core geometry and the pixel rule, default composition on arrival, set/reset commands, 640 px thumbnails, the frame fingerprint and needs_review
difficulty: high
projection: no
depends_on: [P27_retire_shape_on_arrival]
---
# P28 — Composition foundation: the compositions table, core geometry and the pixel rule, default composition on arrival, set/reset commands, 640 px thumbnails, the frame fingerprint and needs_review

## Goal
Implement 10D MC-27 (invariant M33) on the backend. Every `photos` row owns one **composition** (`compositions` table, migration 004):
`kind`, `shape`, `orientation`, `quarter_turns`, `flip`, `straighten`, a fractional `rect` over the turned frame, the source-frame
fingerprint `frame_w × frame_h`, `origin`, `revision`, `state`. A new photo (capture, import) gets the default composition — `kind normal`,
`shape = default_shape`, no turn/flip/straighten, `origin default`, `revision 0` — in the arrival transaction, **without any Capture One
call**; its rect is materialised as the largest centred rect of the shape's ratio the moment the first source frame is known (the first
crop-ignored adjusted preview, or the master in P29); until then the rect is `NULL` (the frame of a 3:2 and a 4:3 camera give different
square fractions, so the fractions cannot be written blind). Photos that exist when this phase ships get the same default row with
`revision 0` and the one-time re-render of their previews so a crop-ignored frame arrives; that never makes a session dirty. The core
owns one geometry module — default rect, validation (fractions, rect inside the straightened image, the shape ratio within one pixel, a
split exactly 2:1), the **pixel rule** (one function used by every renderer and by the tests), the fingerprint comparison — proven on a
**shared case table** (`composition_cases.json`) that P31's TypeScript mirror
runs unchanged. Two thin commands, `composition_set` and `composition_reset`, save through core validation; an invalid save is refused and
the previous composition stays; a saved change is `revision + 1` and a photo change (05A MC-6: the session becomes dirty). With every
preview render the Capture One adapter also reads the variant's **geometry** (`rotation`, `flip`, `keystone vertical/horizontal/amount`)
in the same script run; the composition's fingerprint is that geometry plus the frame's orientation (and, from P29, the master's pixel
size). Each adjusted preview also yields a **640 px long-edge JPEG thumbnail** of the whole frame for grid tiles, and is compared with the
fingerprint: a changed geometry value or orientation makes the composition `needs_review` with the reason recorded (`geometry` → the notice
"Changed in Capture One: straighten here instead"; `frame` otherwise); nothing is cropped silently; a successful save confirms it. VERIFY-15
showed why the aspect ratio is not enough: Capture One's straighten and keystone keep 3:2 and only shrink the export. `session_get` carries
`thumbnail_path` and `composition` per photo. **Not in this phase:** the master render, `render_output`, outputs, the publish/export
refusal while `needs_review` (P29/P30), any UI (P31–P34; tiles keep showing the whole preview until P31 draws the composition), `photo_links`.

## Read first
Book: `10D_composition_and_framing.md` §What the photographer sees (first bullet, "Changes save automatically", "can never extend past the
photo's edge"), **§MC-27 in full** (Source frame, Values incl. the `frame_w/frame_h` + geometry bullet, Default, Validation, Pixel rule,
Revisions from Capture One — the geometry fingerprint, the notice text, M33), §MC-28 §Thumbnail only, §Grounding · `05_domain_model.md` §compositions, §photos · `05A` MC-2 §Row creation (the arrival transaction this phase extends), MC-5
(copy inserts a second row), MC-6 (a saved composition change is a photo change) · `06` §Unassigned, move, copy · `10A` MC-16 §Completion
(where the preview is copied; the thumbnail follows), MC-17 (adjusted events; the one-time stale marking uses `adjusted_event(raw, false)`) ·
`01` (H3 as narrowed: thumbnails are allowed pixel work; H7, H8) · `14` (M33; M13, M23, M24, M28 must keep holding) · `11` §Layers (pixel
work lives in infrastructure behind a core port) · `15` R37 · evidence `capture_one_16.8.6.md` §"VERIFY-15 result" (`adjustments.rotation = 3` → export
5553 × 3702, `keystoneVertical = 20` → 5712 × 3808, both still 3:2; the geometry property names and that `keystoneAmount` reads `100` by
default) · dictionary `docs/reference/capture_one_16.8.6.sdef` `:1140` (variant `adjustments`), `:1352–1358` (`rotation` real, `flip`
"flip type", `keystone amount` integer, `keystone vertical`/`horizontal` real; `keystone skew`/`aspect` exist but are not fingerprinted) · master plan §3, §6.1 (new
modules), §6.2 (image crates), §6.3 (storage: `thumb-<g>.jpg`), §6.4 (`Imaging` port; `Store` additions), §6.5 (`compositions`,
`photos.thumbnail_path`), §6.6 (enums and the three new error codes), §6.7 (`composition_set`, `composition_reset`; `PhotoResponse`), §6.9,
**§6.10 (the composition skeleton: every name this phase introduces)**, §7.2, §7.5 (`SyntheticJpeg`, `complete_render_with`) · process
`flow.md` §4 (unit tests only for the Core mechanisms list), §6, §6A · plan `P27_retire_shape_on_arrival.md` (the phase before: `default_shape`
setting, `arrival_insert(photo, path_key)`, recipes at 2560 crop-ignored) · code (line numbers at `d763492`, 2026-11-04; P27 moves a few):
`crates/core/src/domain/photo.rs` (`Photo :3–20` — gains `thumbnail_path`; `NewPhoto :87–95`), `crates/core/src/domain/session.rs`
(`SessionView :288–295` — gains `compositions`), `crates/core/src/ports/store.rs` (`PreviewCommit :22–25`; `StoreTx::copy_photo :41–46`;
`Store::complete_preview :96`, `arrival_insert :111–116`, `photo_insert_or_get :110`, `photos_in_session :119`), `crates/core/src/
services/previews.rs` (`on_processing_done :126–180`: the `Preview` arm `:145–164` copies, commits, deletes the prior, emits `PhotoChanged`),
`crates/core/src/services/review.rs` (`transfer :180–244`: `tx.copy_photo(photo, target, next) :229`), `crates/core/src/services/
sessions.rs` (`view :108–150`), `crates/core/src/services/settings.rs` (allowed keys), `crates/core/src/events.rs` (`PhotoChanged`,
`SessionChanged`), `crates/core/src/error.rs`, `crates/core/src/mechanisms/mod.rs`, `crates/infra/src/sqlite/store.rs` (`photo_row
:121–156`; `copy_photo :364–373`; `adjusted_event :704–741`; `complete_preview :742–781`; `photo_insert_or_get :1039–1069`; `arrival_insert
:1070–1150`; `photos_in_session :1174–1184`; `SqliteStore::open :27–38`), `crates/infra/src/sqlite/migrations/002_domain.sql` (`photos
:35–51`), `crates/infra/src/previews_store.rs` (`PreviewStore::copy_adjusted :11–20` — the thumbnail lands beside it), `crates/infra/src/
rawpreview.rs` (`:46–60` the SOF/IFD header walkers to reuse for `dimensions`), `crates/core/src/ports/editing.rs` (`RenderCompletion :16–23` — gains `geometry`), `crates/integrations/src/editing/capture_one/
{scripting.rs (`process :78`), jxa.rs (`process` call), jxa/process.js (returns the job id — now also the geometry), adapter.rs (`render :202`,
`request_render :215` stores the completion), renders.rs (`RenderEntry`), gate.rs}`, `crates/integrations/src/lib.rs` (`App::build :355`;
`PreviewService::new :465–474`; `SqliteStore::open :385–387`; the real-start block `:389–401`; `settings_get :232–254`), `apps/desktop/
src-tauri/src/commands/sessions.rs` (`PhotoResponse :117–134`, `From<Photo> :136–169`, `SessionViewResponse :171–195`), `apps/desktop/
src-tauri/src/commands/photos.rs` (the `with_review` shape `:55–63` and `photos_reorder :86–97` to copy for the two commands),
`apps/desktop/src-tauri/src/lib.rs` (`collect_commands! :111–155`; `p17_tests`/`p21_tests` modules: `mock_builder`, `get_ipc_response`,
`request(cmd, args)` for C7(b)) · fakes and helpers: `crates/testkit/src/fake_capture_one.rs` (`FakeVariant :10–16` — gains `geometry`; `process :584–595`), `crates/testkit/src/test_app.rs` (`complete_render :203–229` writes
`FakeCaptureOneScripting::output_bytes` — a stub, not a decodable JPEG; `complete_all_renders :230`; `kill_and_restart :158` = a real
start; `add_photo :56–67`; `mark_published :68`), `crates/testkit/src/fixtures/synthetic_raw.rs` (the hand-written TIFF/JPEG byte builders
to copy for `SyntheticJpeg`), tests to reuse: `p05_capture_arrival.rs` (`hook`, `file`, `external`, `session`, `service` helpers), `p06_live_previews.rs`
(`c1a :79` completion and `PhotoChanged` assertions; `c5a :370` the retouch/dirty assertions), `p02_review_order.rs` (copy rows).
Frontend fixtures that must gain the two new `Photo` fields (typecheck): every `photo()` helper building a `Photo` literal —
`features/workspace/__tests__/{p08_workspace_ui, p19_drop, p20_drop_continuity, p22_workspace_tidy, p26_double_drag}.test.tsx`,
`features/form/__tests__/p11_form_ui.test.tsx`, `features/publish/__tests__/p15_publish_ui.test.tsx` (grep `adjusted_preview_path:` to
find them all; add `thumbnail_path: null, composition: <default DTO>`).

## Files expected to change
Backend (new): `crates/core/src/domain/composition.rs`, `crates/core/src/mechanisms/composition.rs`, `crates/core/src/mechanisms/
composition_cases.json`, `crates/core/src/services/composition.rs`, `crates/core/src/ports/images.rs`, `crates/infra/src/imaging.rs`,
`crates/infra/src/sqlite/migrations/004_compositions.sql`, `apps/desktop/src-tauri/src/commands/compositions.rs`.
Backend (changed): `crates/core/src/domain/{mod.rs, photo.rs, session.rs}`, `crates/core/src/mechanisms/mod.rs`, `crates/core/src/ports/
{mod.rs, store.rs}`, `crates/core/src/services/{mod.rs, previews.rs, sessions.rs}`, `crates/core/src/error.rs`, `crates/core/Cargo.toml`
(nothing new: core stays crate-free), `crates/infra/Cargo.toml` (`image`, `fast_image_resize`), `crates/infra/src/lib.rs`, `crates/infra/src/
sqlite/store.rs`, `crates/integrations/src/editing/capture_one/{scripting.rs, jxa.rs, jxa/process.js, adapter.rs, renders.rs, gate.rs}`,
`crates/integrations/src/lib.rs`, `crates/testkit/Cargo.toml` (`image` for decoding in tests), `crates/testkit/src/
{test_app.rs, fixtures/mod.rs, fixtures/synthetic_jpeg.rs (new)}`, `apps/desktop/src-tauri/src/commands/{mod.rs, sessions.rs}`,
`apps/desktop/src-tauri/src/lib.rs`, `apps/desktop/src/services/bindings.ts` (regenerated), the frontend `Photo` fixtures listed above
(typecheck only).
Tests: `crates/testkit/tests/p28_composition_foundation.rs` (new: C2–C7(a)), `apps/desktop/src-tauri/src/lib.rs` `p28_tests` module (C7(b)),
`crates/core/src/mechanisms/composition.rs` `#[cfg(test)]` (C1, the case table).

## Core mechanisms (unit tests allowed)
`psm_core::mechanisms::composition` — `default_rect`, `validate`, `pixel_rect`, `split_halves`, `fingerprint_compatible`, `orientation_of`, `turned_frame`,
`inside_after_straighten` — the geometry every renderer, the editor and the tests share. Tested by **the shared case table** (C1): each
case is one JSON object; the Rust test runs every case; P31's `geometry.test.ts` runs the same file. No other unit tests.

## The geometry contract (copied into `composition.rs` doc comments; the case table is derived from it)
- **Frames.** `Frame { w, h }` is the source frame as Capture One exports it (crop ignored, its own orientation applied). `turned_frame(frame,
  quarter_turns)` swaps `w` and `h` when `quarter_turns` is odd. Every rect fraction, every ratio and the pixel rule use the **turned**
  frame `W × H`.
- **Order of operations** (MC-27/MC-28): quarter turns and flip first (lossless), then straighten about the centre, then the rect.
  Positive `straighten` rotates the image **clockwise on screen** (y down); the frontend applies CSS `rotate(θdeg)` with the same sign.
- **Shape ratio** `r = shape_ratio(shape, orientation, turned)`: `1:1 → 1`, `4:5 → 4/5`, `3:2 → 3/2`, `2:3 → 2/3`, `16:9 → 16/9` with
  `orientation = named`; `orientation = turned` → `1/r` (X turns the frame); `original → W/H` (orientation ignored); `free → None` (no lock);
  `kind = split → 2` regardless of `shape` and `orientation` (the shape is kept so toggling back to Normal restores it).
- **Default rect** `default_rect(kind, shape, orientation, turned)`: the largest rect of ratio `r` centred in the frame: if `r ≥ W/H` then
  `w = 1`, `h = (W / r) / H`, `x = 0`, `y = (1 − h) / 2`; else `h = 1`, `w = (H · r) / W`, `y = 0`, `x = (1 − w) / 2`. `free` and `original`
  → `(0, 0, 1, 1)`. Fractions are exact `f64`s; nothing is rounded here.
- **Validation** `validate(values, frame) -> Result<(), CompositionInvalid>` in this order, the first failure is the reason: `turns_range`
  (`quarter_turns > 3`), `straighten_range` (`|straighten| > 45`), `fraction_range` (any of `x, y, w, h` outside `[0, 1]`, or `w ≤ 0`, or
  `h ≤ 0`), `rect_outside` (`x + w > 1 + 1e-9` or `y + h > 1 + 1e-9`), `rect_too_small` (`min(w·W, h·H) < MIN_RECT_FRACTION · min(W, H)`,
  `MIN_RECT_FRACTION = 0.05`), `off_shape` (`r` is `Some` and `kind = normal` and `|w·W − r·h·H| > 1` pixel), `split_ratio` (`kind = split`
  and `|w·W − 2·h·H| > 1`), `rect_outside` again when `straighten ≠ 0` and `inside_after_straighten` is false.
- **Inside after straighten** `inside_after_straighten(rect, turned, straighten)`: with `c = (W/2, H/2)`, `θ = straighten` in radians and
  `R(φ) = [[cos φ, −sin φ], [sin φ, cos φ]]` (y-down coordinates: positive `φ` is clockwise on screen), each of the four rect corners
  `p` in pixels (`x·W, y·H`, …) has a source point `s = c + R(−θ)(p − c)`; the rect is inside when every `s` satisfies
  `−ε ≤ s.x ≤ W + ε` and `−ε ≤ s.y ≤ H + ε` with `ε = 1e-6 · max(W, H)`. (Rendering samples the source at `s`; a corner outside means an
  empty corner.)
- **Pixel rule** `pixel_rect(rect, turned) -> PixelRect { left, top, width, height }`: `left = round(x·W)`, `top = round(y·H)`, `width =
  round(w·W)`, `height = round(h·H)`; `round` is half away from zero (Rust `f64::round`; JS `Math.round` agrees for the non-negative inputs
  here). `split_halves(pixel) -> (left_half, right_half)`: `width` rounded down to even, `height = width / 2`, halves `[left, left + width/2)`
  and `[left + width/2, left + width)`, same `top` and `height`.
- **Fingerprint** (MC-27 §Revisions). `EditorGeometry { rotation: f64, flip: String, keystone_vertical: f64, keystone_horizontal: f64,
  keystone_amount: i64 }` is what the editing adapter read with the render (Capture One's `adjustments`; values are opaque to core — H7).
  `Fingerprint { frame: Frame, source: FrameSource (preview | master), geometry: EditorGeometry }`. `orientation_of(frame) ∈ landscape |
  portrait | square` (`w > h`, `w < h`, `w == h`). `fingerprint_compatible(stored, observed) -> bool` ⇔ `stored.geometry == observed.geometry`
  (floats compared with `|a − b| ≤ 1e-6`, `flip` and `keystone_amount` exactly) **and** `orientation_of(stored.frame) == orientation_of(observed.frame)`
  **and** (`stored.source == master && observed.source == master` ⇒ `stored.frame == observed.frame`). Pixel sizes of previews are never compared
  (a 2560 px preview and a 1920 px one of the same source agree). `review_reason(stored, observed)` = `geometry` when the geometry differs, else
  `frame`.

## Ordered tasks
0. **Registry first (master plan §6.10).** `psm_core::domain::composition`: `CompositionKind { Normal, Split }`, `Shape { Original, Square,
   FourFive, ThreeTwo, TwoThree, SixteenNine, Free }` (serde strings `original, 1:1, 4:5, 3:2, 2:3, 16:9, free` — `Shape::parse`/`as_str`
   shared with P27's settings validation: move P27's name check onto `Shape::parse`), `Orientation { Named, Turned }`, `Flip { None, Horizontal }`,
   `Origin { Default, User }`, `CompositionState { Ok, NeedsReview }`, `ReviewReason { Geometry, Frame }`, `FrameSource { Preview, Master }`, `EditorGeometry` (above;
   `Default` = `rotation 0`, `flip "none"`, keystones `0, 0, 100`), `Fingerprint`, `OutputSlot { Single, Left, Right }` with `CompositionKind::slots()`
   (`[Single]` / `[Left, Right]`), `Rect { x, y, w, h: f64 }`, `Frame { w: i64, h: i64 }`, `PixelRect { left, top, width, height: i64 }`,
   `CompositionValues { kind, shape, orientation, quarter_turns: u8, flip, straighten: f64, rect: Rect }`, `Composition { photo_id, kind,
   shape, orientation, quarter_turns, flip, straighten, rect: Option<Rect>, fingerprint: Option<Fingerprint>, origin, revision: i64, state,
   review_reason: Option<ReviewReason>, updated_at }` (`frame()` = `fingerprint.map(|f| f.frame)`)
   with `Composition::values() -> Option<CompositionValues>`. Errors: `Error::CompositionInvalid { reason: String }` (code
   `composition_invalid`, `data.reason`), `Error::FrameUnknown` (`frame_unknown`). Port `psm_core::ports::images::Imaging { fn dimensions(&self,
   path: &Path) -> Result<Option<Frame>>; fn thumbnail(&self, source: &Path, dest: &Path, long_edge: u32) -> Result<bool> }` (P29 extends it).
1. **Geometry (`mechanisms/composition.rs`)** per the contract above, plus `composition_cases.json` beside it holding **every** case of C1
   (the file is the one P31 reads; keep it flat: `{"cases":[{"id","op","input":{…},"expect":{…}}]}` with `op ∈ default_rect | pixel_rect |
   split_halves | validate | fingerprint | inside`; numbers written with at least 12 significant digits; the Rust test compares fractions with
   `1e-9` and integers exactly).
2. **Migration `004_compositions.sql`:** `CREATE TABLE compositions (photo_id INTEGER PRIMARY KEY REFERENCES photos(id), kind TEXT NOT NULL,
   shape TEXT NOT NULL, orientation TEXT NOT NULL DEFAULT 'named', quarter_turns INTEGER NOT NULL DEFAULT 0, flip TEXT NOT NULL DEFAULT 'none',
   straighten REAL NOT NULL DEFAULT 0, x REAL, y REAL, w REAL, h REAL, frame_w INTEGER, frame_h INTEGER, frame_source TEXT, geometry TEXT, origin TEXT NOT NULL DEFAULT 'default',
   revision INTEGER NOT NULL DEFAULT 0, state TEXT NOT NULL DEFAULT 'ok', review_reason TEXT, updated_at INTEGER NOT NULL);` (`geometry` is the
   `EditorGeometry` as JSON; `frame_*`, `frame_source` and `geometry` are NULL together until the first render) and `ALTER TABLE photos ADD COLUMN
   thumbnail_path TEXT;`. Register after `003`. (Rows for existing photos are inserted by the backfill of task 5, not by SQL: the shape is the
   setting.)
3. **Store.** `SqliteStore::open_with_default_shape(paths, shape: &str)` (`open(paths)` = `"1:1"`); the store keeps the name. `arrival_insert`,
   `photo_insert_or_get` and `StoreTx::copy_photo` insert the composition row in the same transaction: arrival/insert → the default row
   (`kind normal`, `shape = default_shape`, `orientation named`, turns 0, `flip none`, straighten 0, `x..h` NULL, `frame_w/h`/`frame_source`/`geometry` NULL, `origin
   default`, `revision 0`, `state ok`, `updated_at = created_at`); `copy_photo` → a copy of the source row under the new `photo_id` (all
   columns). `Store` gains: `composition(photo_id) -> Composition`, `compositions_in_session(session_id) -> Vec<Composition>` (same order and
   filter as `photos_in_session`), `save_composition(photo_id, values: &CompositionValues, origin: Origin, now_ms) -> (Composition, bool)` —
   in one transaction: read the row; if `values` equal the stored values **and** `state == ok` and `origin` equal → return `(row, false)`
   with nothing written; else write `values`, `origin`, `revision + 1`, `state ok`, `updated_at = now`; when written and the photo is not
   `deleted`: `UPDATE sessions SET change_seq = change_seq + 1, last_activity_at = now WHERE id = <photo.session_id>` (MC-6); return
   `(row, true)` — `backfill_compositions(default_shape, now_ms) -> usize` (one default row per `photos` row lacking one; never touches
   `change_seq`) — `mark_previews_stale_once(flag_key: &str, now_ms) -> usize` (when `settings[flag_key]` is absent: for every `raw_state`
   row whose photos have an `adjusted_preview_path`, `adjusted_gen += 1` (no `retouch_gen`, no `change_seq`), then `settings[flag_key] =
   'done'`; returns the count; a second call returns 0) — `complete_preview(raw, generation, path, thumbnail: Option<&str>, observed:
   Option<Fingerprint>) -> PreviewCommit` extended (`observed` = the preview's frame, `source preview`, and the geometry the adapter read with the
   render): sets `photos.thumbnail_path` for every row of the RAW (NULL when `thumbnail` is None), and when `observed` is Some, for every
   composition of the RAW's photos: fingerprint NULL → store `observed` and, if `x` is NULL, materialise `rect = default_rect(kind, shape,
   orientation, turned_frame(frame, quarter_turns))`; stored and `fingerprint_compatible` → keep the stored fingerprint (a preview never
   overwrites a master-sized frame; a preview frame is replaced by a later preview frame — Delegated: either is correct since preview sizes are
   not compared); stored and incompatible → `state = needs_review`, `review_reason = review_reason(stored, observed)` **and the fingerprint :=
   `observed`** (the rect fractions stay: that is the "old rect mapped proportionally" the editor shows; validation of the confirming save then
   runs against the frame the photo now has). The same rule is `adopt_frame(raw, observed)` for P29's masters. `PreviewCommit` gains `previous_thumbnail: Option<String>` and
   `needs_review: Vec<(i64, i64)>` (photo, session). **Geometry read with every render:** `CaptureOneScripting::process(id, variant, recipe) ->
   ProcessStarted { job_id: String, geometry: EditorGeometry }` — `process.js` returns `JSON.stringify({job: …, geometry: {rotation: v.adjustments
   .rotation(), flip: String(v.adjustments.flip()), keystoneVertical: …, keystoneHorizontal: …, keystoneAmount: …}})` in the **same** osascript run
   (no extra call); `RenderCompletion` gains `geometry: EditorGeometry` (kept in the `RenderEntry` from `request_render`); `GatedScripting` forwards;
   the fake's `process` returns `FakeVariant.geometry` (default `EditorGeometry::default()`), settable through `fake.set_variant_geometry(variant,
   EditorGeometry)`. `Photo` gains `thumbnail_path: Option<String>` (`photo_row` reads it).
4. **Imaging (infra `imaging.rs`, `ImageOpsImpl` — the only crate that decodes pixels).** `dimensions`: JPEG → the SOF0/SOF2 header
   (`rawpreview.rs` has the walker), TIFF → IFD0 `ImageWidth`/`ImageLength` (`0x0100`/`0x0101`); unknown or unreadable → `Ok(None)` (never an
   error). `thumbnail(source, dest, 640)`: decode (`image`), fit the long edge to 640 (`fast_image_resize`, Lanczos3), encode JPEG (quality
   delegated), temp + rename; an undecodable source → `Ok(false)` and no file. Crates: `image = "0.25"`, `fast_image_resize = "6"` (verified
   on crates.io 2026-11-04: 0.25.10, 6.1.0; pin the minor as master plan §6.2 says).
5. **Preview completion and the one-time stale mark.** `PreviewService` gains `imaging: Arc<dyn Imaging>`; the `Preview` arm of
   `on_processing_done`: after `copy_adjusted` → `frame = imaging.dimensions(&dest)?`, `thumb = previews/<pk>/thumb-<generation>.jpg`
   (`PreviewFiles::thumbnail_path(raw, generation)`), `made = imaging.thumbnail(&dest, &thumb, 640)?` → `complete_preview(raw, g, dest,
   made.then_some(thumb), frame.map(|f| Fingerprint { frame: f, source: Preview, geometry: entry.geometry.clone() }))`; delete `previous_path` and `previous_thumbnail` as before; emit `PhotoChanged` for every photo of the RAW
   (already) — the `needs_review` list is a subset, nothing extra. `App::build`: `SqliteStore::open_with_default_shape(&paths,
   &settings.default_shape)`; at every build `store.backfill_compositions(&settings.default_shape, now)`; at a **real start** only
   (`start_backup`): `store.mark_previews_stale_once("rerender_after_composition", now)`; `PreviewService::new(…, imaging)`.
6. **Service and commands.** `psm_core::services::composition::CompositionService { store, clock, events, default_shape }`: `set(photo_id,
   values) -> Composition`: `store.composition(photo_id)?`; `frame` None → `Err(FrameUnknown)`; `validate(&values, frame)?` (→
   `Err(CompositionInvalid)`); `save_composition(photo_id, &values, Origin::User, now)`; when changed emit `PhotoChanged { photo_id,
   session_id }` and `SessionChanged { session_id }`. `reset(photo_id) -> Composition`: frame None → `FrameUnknown`; values = `kind normal`,
   `shape = default_shape`, `orientation named`, turns 0, `flip none`, straighten 0, `rect = default_rect(…)`; `save_composition(…,
   Origin::Default, now)` (a reset of an already-default row returns unchanged); same events. Commands (`commands/compositions.rs`, thin,
   `spawn_blocking`, `AppHandles`): `composition_set(photo_id: CommandId, values: CompositionValuesDto) -> Result<CompositionDto, CommandError>`,
   `composition_reset(photo_id) -> Result<CompositionDto, CommandError>`; `CommandError` maps `CompositionInvalid { reason }` → code
   `composition_invalid`, `data = {"reason": reason}`, `FrameUnknown` → `frame_unknown`. DTOs (`specta::Type`): `RectDto { x, y, w, h }`,
   `FrameDto { w, h }`, `CompositionValuesDto { kind, shape, orientation, quarter_turns, flip, straighten, rect }`, `CompositionDto { … all
   `Composition` fields; `rect`, `frame` (`FrameDto`, the fingerprint's frame), `review_reason` nullable; `geometry` is **not** sent (adapter
   detail) }`. `PhotoResponse` gains `thumbnail_path: Option<String>` and `composition: CompositionDto`
   (`SessionView.compositions` zipped like `upload_states`). `collect_commands!` gains the two. Bindings regenerated.
7. **Testkit.** `fixtures/synthetic_jpeg.rs`: `SyntheticJpeg::rgb(w, h, [r, g, b]) -> Vec<u8>` (a real baseline JPEG through `image`,
   quality 90) — a flat colour is enough here; `TestApp::complete_render_with(job_id, bytes) -> PathBuf` (same as `complete_render` with the
   given bytes); `TestApp::composition(photo_id)` → `store.composition`. `TestApp::create` passes nothing new (the store reads the setting).
8. Tests, `cargo test --workspace`, clippy, fmt, bindings regeneration (diff expected: `CompositionDto`, `CompositionValuesDto`, `RectDto`,
   `FrameDto`, `PhotoResponse` fields, two commands), `npm run typecheck` (fixtures), `npm test`.

## Criteria (7 criteria, 25 rows; 0 `[rule]` rows)
Fixture shapes: `preview(a, job, w, h)` = `a.complete_render_with(job, &SyntheticJpeg::rgb(w, h, [200, 100, 50]))`; the "default square"
fractions for `2560 × 1707` are `x = (1 − 1707/2560)/2 = 0.166601562…`, `w = 1707/2560 = 0.666796875`, `y = 0`, `h = 1`. Frame-known rows
first run arrival + `tick` + `preview(2560, 1707)`.

| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | `cargo test -p psm-core composition::tests` over `composition_cases.json`, op `default_rect` | 11 cases: (1) normal 1:1 named 2560×1707 turns 0; (2) normal 1:1 1707×2560; (3) normal 4:5 named 2560×1707; (4) normal 4:5 turned 2560×1707; (5) normal 3:2 named 2560×1707; (6) normal 16:9 named 2560×1707; (7) original; (8) free; (9) normal 1:1 2560×1707 **turns 1**; (10) split 2560×1707; (11) split 1707×2560 | (1) `(0.166601562, 0, 0.666796875, 1)`; (2) `(0, 0.166601562, 1, 0.666796875)`; (3) `w = 0.8·1707/2560 = 0.5334375`, `x = 0.23328125`, `y 0`, `h 1`; (4) `w = 1.25·1707/2560 = 0.833496094`, `x = 0.083251953`; (5) `w 1`, `h = (2560/1.5)/1707 = 0.999804734`, `x 0`, `y = 0.000097633`; (6) `w 1`, `h = 1440/1707 = 0.843585237`, `y = 0.078207381`; (7), (8) `(0, 0, 1, 1)`; (9) = case (2) (the turned frame is 1707×2560); (10) `w 1`, `h = 1280/1707 = 0.749853544`, `y = 0.125073228`; (11) `w 1`, `h = 853.5/2560 = 0.333398438`, `y = 0.333300781`; all within `1e-9` | MC-27 §Default ("the largest rect of that ratio centred in the frame"); §Values (`orientation`, split 2:1) |
| C1(b) | same, ops `pixel_rect` and `split_halves` | (12) rect `(0.166601562, 0, 0.666796875, 1)` on 2560×1707; (13) `(0.5, 0.25, 0.25, 0.5)` on 601×401; (14) split `(0.1, 0.2, 0.6, 0.4993)` on 1001×601; (15) `(0.5, 0.5, 0.5, 0.5)` on 3×3 | (12) `{427, 0, 1707, 1707}` (426.5 rounds away from zero to 427); (13) `{301, 100, 150, 201}` (300.5 → 301, 100.25 → 100, 150.25 → 150, 200.5 → 201); (14) pixel `{100, 120, 601, 300}` then halves: width 601 → 600, `height 300`, left `[100, 400)` = `{100, 120, 300, 300}`, right `{400, 120, 300, 300}`; (15) `{2, 2, 2, 2}` | MC-27 §Pixel rule (one function; even width; half away from zero) |
| C1(c) | same, op `validate` with frame 2560×1707 (turns 0 unless said) | (a) 1:1 `(0.2, 0, 0.666796875, 1)`; (b) 1:1 `w = 0.666` (1704.96 px vs 1707) ; (c) 1:1 `x = 0.4, w = 0.666796875`; (d) `original` `(0, 0, 1, 1)` straighten 5; (e) 1:1 `(0.333300781, 0.25, 0.333398438, 0.5)` straighten 5; (f) straighten 46; (g) turns 4; (h) split `(0, 0.125, 1, 0.74)` (1263.2 px vs 1280); (i) 1:1 `(0.49, 0.49, 0.01, 0.015)`; (j) free `(0.1, 0.1, 0.3, 0.7)`; (k) `original` `(0, 0, 1, 0.99)`; (l) `(−0.01, 0, 0.5, 0.5)`; (m) turns 1, 1:1 `(0, 0.166601562, 1, 0.666796875)` | (a) ok; (b) `off_shape`; (c) `rect_outside`; (d) `rect_outside` (corners leave the image); (e) ok (a half-height square rotated 5° stays inside: corner reach 603.5·cos 40° = 462 px < 853.5); (f) `straighten_range`; (g) `turns_range`; (h) `split_ratio`; (i) `rect_too_small` (25.6 px < 0.05·1707 = 85.35); (j) ok; (k) `off_shape` (`|2560 − 1.49971·0.99·1707| = 25.6` px); (l) `fraction_range`; (m) ok (turned frame 1707×2560) | MC-27 §Validation (fractions inside 0–1, inside after straightening, ratio within one pixel, split exactly 2:1); M33 ("the core refuses any rect outside the image or off its shape") |
| C1(d) | same, ops `fingerprint` and `inside` | fingerprint, stored = `{2560×1707, preview, default geometry}` vs observed: (1) `{2560×1707, preview, default}`; (2) `{1920×1280, preview, default}`; (3) `{2560×1700, preview, default}` (a shrunk 3:2 export); (4) `{2560×1707, preview, rotation 3}`; (5) `{…, keystoneVertical 20}`; (6) `{…, flip "horizontal"}`; (7) `{…, keystoneAmount 80}`; (8) `{1707×2560, preview, default}`; (9) `{5981×3987, master, default}`; stored = `{5981×3987, master, default}` vs (10) `{5553×3702, master, default}`; (11) `{2560×1707, preview, default}`; (12) `{5981×3987, master, rotation 3}`; inside: the corners of cases (d) and (e) | compatible: (1) `true`, (2) `true`, (3) `true` (sizes of previews are never compared), (4) `false` reason `geometry`, (5) `false` geometry, (6) `false` geometry, (7) `false` geometry, (8) `false` reason `frame` (orientation), (9) `true` (a master after a preview), (10) `false` reason `frame` (master size changed), (11) `true` (a preview after a master), (12) `false` geometry; inside: (d) `false`, (e) `true` | MC-27 §Revisions from Capture One ("compatible when the geometry values and orientation are unchanged and the master size, when both exist, is equal"); VERIFY-15 (straighten/keystone keep 3:2) |
| C2(a) | `TestApp::with_fake_document()`; p05 `session(&a,"K1")` activated; `fake.add_image(<root>/K1/a.ARW, &["7"], false)`; p05 `file` + `hook`; `a.tick(0)` | a capture arrival, default settings (`default_shape` `1:1`) | `a.composition(photo.id)` = `kind normal, shape 1:1, orientation named, quarter_turns 0, flip none, straighten 0, rect None, fingerprint None, origin default, revision 0, state ok, review_reason None`; the fake's `ops` have no crop entry; `photos.thumbnail_path` is NULL | MC-27 §Default (capture; no Capture One call); M33 |
| C2(b) | same; `a.tick(20_000)` issues the preview; `preview(a, job, 2560, 1707)` | the first crop-ignored adjusted preview | composition: `fingerprint == Some({2560×1707, preview, EditorGeometry::default()})` (the fake's default geometry travelled with the `process` answer), `rect == Some(default square)` (C1 case 1, `1e-9`), `revision 0`, `origin default`, `state ok`; `photos.thumbnail_path == previews/<pk[0..2]>/<pk>/thumb-<g>.jpg` and `image::image_dimensions(path) == (640, 427)`; `adjusted_preview_path` set as before (M23); one `PhotoChanged` for the photo | MC-27 §Default, §Source frame; MC-28 §Thumbnail; M23 |
| C2(c) | `with_fake_document()`; `settings_set({"default_shape":"original"})`; arrival + preview as (b) | the whole-frame default | `rect == Some((0, 0, 1, 1))`, `shape original` | MC-27 §Default (`original` keeps the whole frame) |
| C2(d) | after (b): `settings_set({"default_shape":"4:5"})`; `review().copy_photos(&[id], s2)`; also a second `TestApp` where the composition was first set by C3(a)'s values, then copied | copy of a default row and copy of a user row | the copy's composition equals the source's in every field (`shape 1:1`, not the new setting; `rect`, `frame`, `revision`, `origin`, `state`), only `photo_id` differs | MC-27 §Values ("a copy of a photo to another session copies its composition") |
| C2(e) | arrival as (a); `a.complete_render(job)` (the stub bytes of `FakeCaptureOneScripting::output_bytes`) | an undecodable preview file | `adjusted_preview_path` set, `thumbnail_path` NULL, composition unchanged (`fingerprint None`, `rect None`), no error, `PhotoChanged` emitted | MC-28 §Thumbnail (tolerant: the preview is recorded regardless); M23 |
| C3(a) | after C2(b): `a.app.compositions.set(id, values)` with `values = {normal, 1:1, named, turns 1, flip horizontal, straighten 2.5, rect (0.1, 0.3, 0.8, 0.5334375)}` (the turned frame is 1707×2560: `w·W = 1365.6`, `h·H = 1365.6`; the 2.5° rotation keeps every corner inside — reach 719 px of 853.5 horizontally, 2163 px of 2560 vertically); then `a.kill_and_restart()` | a valid save and a relaunch | returns `Composition { values == input, origin user, revision 1, state ok, frame 2560×1707 }`; after the restart `a.composition(id)` equals it field by field (`1e-12` on floats); `PhotoChanged{id, session}` emitted | M33 ("reads back identically after a relaunch"); MC-27 §Values, §Validation |
| C3(b) | after C2(b): `set` with (1) rect `(0.2, 0, 0.666, 1)` (off shape by 2 px); (2) `(0.4, 0, 0.666796875, 1)`; (3) straighten 46; (4) split kind with rect `(0, 0.125, 1, 0.74)` | invalid saves | (1) `Err(CompositionInvalid { reason: "off_shape" })`; (2) `"rect_outside"`; (3) `"straighten_range"`; (4) `"split_ratio"`; after each the stored composition is **unchanged** (`revision 0`, default rect, `origin default`), no `PhotoChanged` for the photo since the preview | MC-27 §Validation ("Invalid → the save is refused and the previous composition stays"); M33 |
| C3(c) | after C3(a): `set` again with the identical values | a no-op save | returns `revision 1` (unchanged), `updated_at` unchanged; the session's `change_seq` equals its value after C3(a); no new `PhotoChanged` | MC-29 §Dirty ("Opening and closing the editor without a change is not" a change) |
| C3(d) | `with_fake_document()`; `add_photo(s1, "d.ARW")` + `review().classify(id, Main)`; seed a frame: deliver a preview for it (`fake.add_image`, `tick`, `preview(2560, 1707)`); `a.mark_published(s1)`; `set(id, <valid values>)` | a composition change on a published session's main photo | `session(s1).dirty == true`, `change_seq == published_seq + 1`; `SessionChanged{s1}` emitted; `revision 1` | MC-29 §Dirty; 05A MC-6; M13; M35 (last clause) |
| C3(e) | as (d) but `review().delete(id)` before the `set` | a composition change on a deleted photo | `revision 1` (saved), `change_seq` unchanged, `dirty == false` | MC-6 (photo change counts visible photos; planner reading in Notes) |
| C3(f) | arrival as C2(a) (no preview yet): `set(id, <any values>)` | no source frame known | `Err(FrameUnknown)`; composition unchanged (`rect None`, `fingerprint None`) | MC-27 §Validation (ratio is checked against `frame_w × frame_h`; no frame, no check — refused) |
| C4(a) | after C3(a): `settings_set({"default_shape":"4:5"})`; `a.app.compositions.reset(id)` | reset under a changed default | returns `{normal, 4:5, named, turns 0, flip none, straighten 0, rect == C1 case (3)}`, `origin default`, `revision 2`, `state ok`; `PhotoChanged` emitted; a second `reset` returns `revision 2` and changes nothing | MC-27 §Default (⌘⌥R "resets to the default"); MC-29 §Dirty (a reset is a saved change) |
| C5(a) | after C2(b): `fake.set_variant_geometry("7", EditorGeometry { rotation: 3.0, ..default })`; mark the RAW adjusted (`deliver_hook(variant_adjusted)` outside the own-write window), `tick(20_000)`, `preview(a, job2, 2560, 1707)` | the photographer straightened in Capture One (the export keeps 3:2 — VERIFY-15) | composition `state needs_review`, `review_reason geometry`, fingerprint `== {2560×1707, preview, rotation 3}` (the observed one), `rect` fractions unchanged, `revision` unchanged; `PhotoChanged` emitted; `adjusted_preview_path` is the new file | MC-27 §Revisions ("Anything else → `state needs_review`, with the notice … when a geometry value moved"); M33 |
| C5(b) | as (a) but with the default geometry and `preview(…, 1920, 1280)`; separately `preview(…, 2560, 1700)` | compatible re-renders | both: `state ok`, rect unchanged (preview pixel sizes are not compared; colour and light edits stay compatible) | MC-27 §Revisions ("colour and light edits stay compatible and the composition stands"; "The aspect ratio alone is not enough") |
| C5(c) | after (a): `set(id, <valid values for the 2560×1707 frame>)` | the photographer confirms a framing | `state ok`, `review_reason None`, `revision 1`, values stored | MC-27 §Revisions ("the editor shows the old rect … for confirmation with Return") |
| C5(d) | after C2(b): `review().copy_photos(&[id], s2)`; then (a)'s geometry change and re-render | two rows on one RAW | both rows read `needs_review`, `review_reason geometry` | MC-27 §Values (one composition per `photos` row; the RAW's frame is shared) |
| C5(e) | after C2(b): default geometry, `preview(…, 1707, 2560)` (Capture One rotated the variant 90°, which it reports through orientation, not `rotation`) | an orientation change | `state needs_review`, `review_reason frame`, fingerprint `{1707×2560, preview, default}`, rect fractions unchanged | MC-27 §Revisions ("the geometry values **and orientation** are unchanged"); M33 |
| C6(a) | `with_fake_document()`; `add_photo` three photos; `DELETE FROM compositions` by SQL; note `change_seq`; `a.kill_and_restart()` | an existing database without composition rows | three rows exist again, each `revision 0`, `origin default`, `shape 1:1`, `rect None`, `fingerprint None`; every session's `change_seq` unchanged (`dirty` false) | MC-27 §Default ("Photos that exist when this chapter ships get the default composition with `revision 0`; that never makes a session dirty") |
| C6(b) | `with_fake_document()`; two photos, one with an adjusted preview (`preview(2560,1707)` after arrival), one without; `a.kill_and_restart()` (#1); read `preview_state`; `a.kill_and_restart()` (#2) | the one-time re-render of previews made before this chapter (their frames respect Capture One's crop and carry no geometry) | after #1: the previewed RAW has `adjusted_gen` +1, `retouch_gen` unchanged, its session's `change_seq` unchanged (not dirty); the RAW without a preview is unchanged; `settings['rerender_after_composition'] == 'done'`; after #2 no further change; a later `tick(20_000)` issues one `process(v, "PSM Preview")` for the stale RAW | P27 Notes (fingerprint grounding); MC-17 (`adjusted_event(raw, false)` is not a retouch); M28 (own re-renders never dirty) |
| C7(a) | after C3(a): `a.sessions().view(session_id)` | the session view | `view.compositions.len() == view.photos.len()`, aligned by `photo_id`; the composition of `id` equals `a.composition(id)`; `photos[i].thumbnail_path` equals the stored path | MC-27 §Values (one composition per photo row, read with the photo) |
| C7(b) | desktop crate `p28_tests` (P17/P21 shape: `mock_builder`, `get_ipc_response`): `composition_set` with (1) C3(a)'s values, (2) an off-shape rect, (3) on a frameless photo; `composition_reset` | the command boundary | (1) `CompositionDto` with `revision 1`, `origin "user"`, `state "ok"`, `review_reason null`, `rect` and `frame` objects and no `geometry` key; (2) `CommandError { code: "composition_invalid", data: {"reason": "off_shape"} }`; (3) `code: "frame_unknown"`; reset → `origin "default"`; `session_get` photos carry `thumbnail_path` and `composition` with the same values | MC-27 §Validation at the command boundary (ch11: commands are thin) |

## Delegated choices
Whether `Composition` holds `CompositionValues` inline or as fields (the DTO shape is fixed); the thumbnail JPEG quality (80 suggested) and
whether it is written through `fast_image_resize` or `image::imageops` (Lanczos3 either way); `Imaging::dimensions` helper placement
(reuse `rawpreview.rs` walkers or a small `image::ImageReader::into_dimensions` call — header-only is the requirement, not the means);
whether `save_composition` emits events itself or the service does (events listed are the contract); how `SessionView.compositions` is
fetched (a second query or a `LEFT JOIN`); the JSON layout of `composition_cases.json` beyond the required `id/op/input/expect` keys; the
`CompositionValuesDto` field order; where `Shape::parse` lives (domain) and how P27's settings validation calls it.

## Manual try-out
1. Launch the app (your sessions exist from before). Open a session: the tiles still show the whole photo; within a minute Capture One
   re-renders each photo's preview once (the previews become 2560 px and ignore any Capture One crop). Nothing becomes "needs publishing".
2. Shoot or import one frame: it arrives; after the preview delay the tile shows the full 3:2 frame. In the data folder
   (`~/Library/Application Support/com.loorenz.sessionmanager/previews/…/<key>/`) a `thumb-<n>.jpg` of 640 px sits beside `adjusted-<n>.jpg`.
3. In Capture One, straighten that photo by about 3° (the Straighten tool; the export keeps 3:2 and only shrinks) and wait for the
   re-render: nothing is cropped behind your back — the photo's composition is flagged for review in the database (`sqlite3 session_manager.db
   "select photo_id,state,review_reason from compositions"` shows `needs_review`, `geometry`); the editor that lets you confirm it arrives in
   P32. Undo the straighten in Capture One: the flag stays until confirmed (by design: a human looks before anything is rendered). Rotating a
   photo 90° in Capture One flags it too, with reason `frame`.
4. Settings → Default shape → `4:5`, Save, shoot a frame: its composition row reads `shape 4:5`.

## Notes
- **Why the default rect is materialised late.** The fractions of "the largest square centred in the frame" depend on the frame's ratio
  (`w = H/W` for a landscape frame), and at arrival no crop-ignored frame exists yet (the embedded camera JPEG is framed tighter, MC-20
  evidence). Writing fractions blind for a guessed 3:2 would be a silent wrong crop on a 4:3 camera. A `NULL` rect with `origin default`
  says "the default of `shape`", which every consumer can compute the moment it knows a frame (P29's renderer does it from the master when
  no preview ever arrived). P31's editor opens only on a photo with an adjusted preview, so it always has a rect.
- **Why existing previews are re-rendered once.** Previews made before P27 respect Capture One's crop (P23 set squares) and were rendered
  without a geometry read; the first fingerprint must come from a crop-ignored render that carried one. Marking them stale once (not a
  retouch, no `change_seq`) brings a crop-ignored 2560 px frame for every photo within one preview cycle.
- **Deleted photos and dirty (C3(e)).** MC-6 counts photo changes; a deleted photo is hidden and never published, so its composition change
  cannot change what the shop shows. The reading "bump `change_seq` for non-deleted photos" keeps M13 honest in both directions; the
  coordinator may widen it to all photos if the owner prefers.
- **An incompatible render replaces the fingerprint** (C5(a), C5(e)): keeping the old frame would make the confirming save fail its ratio
  check against a frame the photo no longer has; keeping the rect fractions is exactly the proportional mapping the book asks the editor to
  show. `needs_review` is the flag that a human has not looked yet; the fingerprint is a fact about the source. `review_reason` lets the
  editor say "Changed in Capture One: straighten here instead" only when a geometry value moved (R37: straightening is the app's).
- **Why the geometry rides on `process`.** The adapter already runs one script per render; reading `adjustments.rotation`, `flip` and the
  three keystone values in that run costs nothing and ties the fingerprint to the exact variant state the render used. Core sees an
  `EditorGeometry` of opaque numbers and strings (H7); only the adapter knows they are Capture One's.
- **`frame_unknown` is temporary by construction**: it can only be hit before a photo's first crop-ignored preview, when the editor cannot
  open anyway. The command still exists for completeness (and the phone remote later).
- **Rounding.** The pixel rule rounds half away from zero; all inputs are ≥ 0, so JavaScript's `Math.round` (half toward +∞) agrees with
  Rust's `f64::round` — the case table pins the half cases (C1(b) 12, 13, 15) so a different TS choice fails the shared table.
- **No Capture One in the suite** (flow §6A): every row runs over the fake; previews are synthetic JPEGs written by the testkit.
- Registry additions made by the planner in master plan §6.1–§6.7, §6.9, §6.10, §7.5. No intention-book change.
- Estimated size: ~28 files, ~1 650 lines (≈ 550 geometry + migration + store, ≈ 300 service/commands/DTO, ≈ 650 tests and fixtures).

## Review log
- **2026-11-04, implement r1 (codex).** Built migration 004, composition rows on arrival/insert/copy and backfill, the shared geometry and pixel rule case table, set/reset validation and dirty tracking, source-frame fingerprints from the same Capture One process call, 640 px thumbnails, one-time rerender marking, and typed desktop commands. Added 25 criterion-row tests (4 core geometry, 20 service/storage, 1 command). Final stamp: 481 Rust tests, 131 UI tests, clippy/fmt/typecheck green.
- **Assumptions.** The thumbnail uses JPEG quality 80 and `image::imageops` Lanczos3; `image::image_dimensions` reads dimensions without decoding pixels. A compatible preview leaves the stored fingerprint unchanged, including an earlier preview size; a master may replace it later. The testkit's `restart_with_startup` is the real-start helper; `kill_and_restart` is not, despite the stale reference in Read first.
- **Deviations.** Added `Cargo.lock` for the new image crates. Updated the P01 schema assertion, P17 JXA script fixture, and P27 migration fixture because they encoded pre-P28 shapes; updated shell and pending photo fixtures omitted from the plan's frontend list for the new required `composition` field. No product-semantic deviation. Live Capture One geometry read was not probed in this session; the fake and script harness were exercised.
