---
phase: P58_fast_compose_on_camera_preview
title: Faster previews — reframe a photo the moment it arrives, on the camera's own full-size preview, while Capture One's adjusted version is on its way; the photo the photographer is looking at and the newest arrivals are rendered first
difficulty: high
projection: no
depends_on: [P57_camera_auto_take_back_and_run2_fixes]
---
# P58 — Compose on the camera preview; render what matters first (P52 F11)

Written by the orchestrator (Opus) on 2026-11-09. The owner (2026-11-08, during P52): "while the pictures are arriving I try reframing one in Session
Manager but I couldn't, it says waiting for Capture One to render preview, how can we make things more efficient and fast"; (2026-11-09) "we will have
time to implement also the fast preview, as you already have the intention and know how we will implement it". Book changelog entry 39.

## Goal
1. **Compose without waiting for Capture One.** Today Compose needs Capture One's adjusted preview (`apps/desktop/src/services/compositions.ts`:
   `adjusted_preview_path !== null`, sentence `COMPOSE_WAITING`). The app already extracts the RAW's embedded JPEG as the **original preview** at arrival
   (10A MC-20, byte copy, orientation stored). On the owner's A7R V that JPEG is **full resolution, 6240 × 4160 (3:2)**, the same frame as Capture One's
   export (2560 × 1707) apart from lens corrections (~1–2 %, orchestrator comparison 2026-11-09). After this phase, a photo whose original preview
   **matches the camera frame** (same aspect ratio as the RAW's sensor frame within 1 %, and at least 1600 px on the long edge) can be composed at once:
   the editor and the composed stage/tiles use the original preview (with its stored orientation) until Capture One's adjusted preview arrives, then swap
   to it in place **without changing the composition** (stored as fractions, MC-27) — the rect is re-clamped only if it would now leave the frame.
   While the original is shown, the editor shows one quiet line: `Camera preview — Capture One's version is on its way`. A photo whose original does not
   match (e.g. the ZV-E10's 1088 × 1080 tighter crop, MC-20 evidence) keeps today's behaviour and sentence.
2. **Render what matters first.** Capture One renders one request at a time and the app queued them oldest-first, so a new photo or the photo being
   composed could wait behind a long backlog (P52 run 2: ~140 re-renders after a settings change). After this phase the preview render queue is ordered:
   (1) the photo open in the editor or on the stage, (2) newest arrivals (most recent first), (3) everything else oldest-first. A change of the focused
   photo re-orders pending requests; requests already sent to Capture One are not cancelled.
3. **Exports unchanged.** Masters and exports always come from Capture One (MC-28); the camera preview is never exported, published or sent to Shopify.

**Not in this phase:** any change to how Capture One renders, parallel Capture One renders, pixel processing beyond MC-28's composition renderer (H3),
storage policy, camera work.

## Read first
Book `10A_capture_one_contracts.md` MC-20 (original preview, M27), `10D_composition_and_framing.md` (§What the photographer gets — the Compose availability
sentence; MC-27 §Source frame; MC-28 renderer), `06_photo_review_and_ordering.md` (original then adjusted preview, toggle), `05_domain_model.md`
(`original_preview_path`, orientation), `01_objective_and_constraints.md` H3, `16_changelog.md` entry 39. Code: `apps/desktop/src/services/compositions.ts`,
the composition editor and stage/tile rendering under `apps/desktop/src/features/` (find by `COMPOSE_WAITING`, `adjusted_preview_path`,
`original_preview_path`), `crates/core/src/services/previews.rs` (render requests, queue, P56 changes), `crates/integrations/src/editing/capture_one/{adapter.rs,
renders.rs}`, the MC-20 extractor (find by `0x0201`). Tests: p27–p41 composition tests (UI + testkit), p56 render tests.

## Files expected to change (write perimeter)
The files above as needed; `crates/testkit/tests/p58_fast_compose_on_camera_preview.rs` (new); UI tests under the composition feature (new file
`p58_compose_on_camera_preview.test.tsx`); the photo DTO/bindings if a "frame matches" flag or the original's pixel size is exposed (the extractor already
reads the SOF size). Existing assertions change only where Compose availability changes (list each in the handoff). ≈ 15 paths, ≈ 600 lines.

## Core mechanisms (unit tests allowed)
"Original matches the camera frame" decision; render-queue ordering.

## Criteria (5 criteria, 8 rows)
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| C1(a) | Compose on the camera preview | photo with original 6240 × 4160 (RAW frame 3:2), no adjusted preview | Compose enabled; the editor shows the original (oriented) and the line `Camera preview — Capture One's version is on its way` | `[rule]` changelog 39; 10D |
| C1(b) | original does not match | original 1088 × 1080 for a 3:2 RAW (or < 1600 px) | Compose disabled with `Waiting for Capture One's full-frame preview` (unchanged) | `[rule]` 10D; MC-20 evidence |
| C1(c) | swap keeps the composition | composed on the original (rect 0.1, 0.1, 0.6, 0.6, shape 4:5); adjusted preview arrives | the editor/stage now show the adjusted image; the stored composition is unchanged; the line is gone | `[rule]` MC-27 (fractions) |
| C1(d) | composed result before Capture One | as C1(c) before the adjusted arrives | stage and tile show the composed picture from the original preview | `[rule]` 10D §The result is shown |
| C2(a) | exports never use the camera preview | composed on the original only; export or publish requested | the export waits for / uses Capture One's master (MC-28); no camera-preview bytes in the output | `[rule]` MC-28; H3 |
| C3(a) | render order: focused first | 20 pending preview requests (oldest-first); photo #15 opened in the editor | the next request sent to (fake) Capture One is #15 | `[rule]` changelog 39 |
| C3(b) | render order: newest arrivals next | backlog of 10 old requests; 2 new photos arrive; nothing focused | the next two requests are the newest arrival, then the other new one, then the backlog oldest-first | `[rule]` changelog 39 |
| C4(a) | existing suites | everything else | `cargo test --workspace`, `npm test` (known intermittent `p33_split_grid` aside), clippy, fmt, typecheck green | `[rule]` master plan §7.2 |

## Delegated choices
Where the "frame matches" decision runs (backend flag on the photo DTO vs frontend from stored sizes — prefer backend); the RAW frame's aspect source (EXIF
pixel dimensions or the largest SubIFD); the line's exact placement in the editor; how focus is reported to the backend (existing "current photo" event if any).

## Manual try-out
With the camera on Wi-Fi: take a photo, open it at once and press C: the editor opens on the camera preview with the line; reframe; a few seconds later
the picture refreshes to Capture One's version and the frame stays where you put it. Take ten photos quickly and open the last: its adjusted preview
comes before the older ones.

## Notes
- **Hard rules (owner):** no pixel processing except MC-28's renderer (H3) — showing the original JPEG with CSS orientation/transform is display, not
  processing; never export the camera preview; never delete photos or previews; the suite never needs Capture One or the camera. Default implementer
  (Codex; owner: no Fable).
- **Hazard:** lens-correction differences mean the composed result can shift by ~1–2 % at the swap; accepted by the owner's goal (speed first). The
  composition must still never extend past the photo's edge after the swap (re-clamp).

## Review log

- 2026-11-09 implement r1 (claude, pipeline-implementer, Opus 5.5; continued the uncommitted partial Codex tree of the stopped run): **Compose on the camera preview.** The MC-20 extractor now decides "original matches the camera frame" (`rawpreview::compose_frame`: JPEG aspect within 1 % of the RAW sensor frame, long edge ≥ 1600 px; sensor frame = DNG `DefaultCropSize` 0xC620, else the largest ImageWidth × ImageLength of an IFD without a JPEG). A match gives the composition its frame from the camera preview (`FrameSource::Camera`, oriented). The UI composes on the original while no adjusted preview exists, turned by the stored orientation (CSS only, H3), with the line `Camera preview — Capture One's version is on its way`. Editor stage, output previews, split previews and loupe share `editorPicture`. When Capture One's preview or master arrives, `adopt_frame` keeps the fractions and revision and re-fits the rect only if it no longer validates (`mechanisms::composition::refit_rect`). A camera frame never replaces a measured one. **Render order:** the Capture One adapter sends previews one at a time, from a queue ordered focused → newest arrival → oldest-first (`RenderRegistry::pop_next`). Masters go at once as before, waiting only for a render of the same RAW (the partial tree had serialised them too, which made a stuck export fail photo by photo and broke P09 C6(b)). The workspace reports the stage/editor photo (`photos_preview_focus` → `PreviewService::set_focus`). Arrivals are first requests and Wi-Fi captures waiting for their variant. A queued request Capture One refuses at its turn fails its export (master) or backs off (preview) via `refused_renders`, and the tick restarts an idle lane. Masters still check disk space when requested. **Probe on real files** (throwaway, `/tmp`): the owner's A7R V ARW → frame 6240 × 4160 (its SubIFD ImageWidth/Length are tile-padded 6656 × 4608, so the partial tree's ImageWidth rule would have rejected every A7R V photo); a ZV-E10 ARW (1:1 JPEG 1088 × 1080, sensor 6000 × 4000) → not composable. Assumptions: see the handoff (new-arrival definition, one preview lane, masters unchanged, default composition re-made when Capture One's orientation differs, line at the bottom of the editor stage, photos from before P58 not re-checked). Deviation: C1(c)'s rect (0.1, 0.1, 0.6, 0.6) is not 4:5 on a 3:2 frame; tests use 4:5 with w = 0.32. Changed existing assertions: P31 C2(d) fixture (no frame), P06 C3(a) (finishes renders one at a time; same order). Stamps in the implementer handoff.

- 2026-11-09 review r1 (opus, pipeline-reviewer): **CHANGES_REQUESTED** (2 blocking). Gate: tracker IMPLEMENTED, checkpoint `3b1351a` present; its 35 paths equal the declared 33 code/test + plan + tracker. Suite on `3b1351a` (clean tree): `cargo test --workspace` green (cargo reached the last test target and the doc tests; no failure), clippy, fmt, `npm run typecheck` exit 0, `npm test` 58 files 418/418. **Verified correct:** `rawpreview::{raw_frame, compose_frame}` (DefaultCropSize first, padded IFD sizes ignored when it exists, 1 % aspect, ≥ 1600 px, oriented frame); `OriginalPreviewJob` adopts a `camera` frame only on a match; `adopt_frame_tx` never lets a camera frame replace a measured one, keeps fractions and revision on Capture One's frame and re-fits only an invalid rect; C1(a), C1(c), C1(d) and C2(a) tests observe their outcomes at their boundaries (C2(a) output 192 × 240 in the master's colour; a camera-preview export would differ in size and colour); render queue: masters bypass the preview lane, focused → newest → oldest; C3(a) chain UI → `photos_preview_focus` (thin) → `set_focus`; C3(b) re-checked with real captures through the capture hook (reviewer scratch test, deleted): order new-2, new-1, old-2, old-3. H1–H8 hold (camera orientation is CSS display, H3; port additions are provider-neutral, H7; the command is thin, H8). Assumptions 1–8 do not contradict the book. **F1 (B1, route production):** a photo whose camera preview does not match (1088 × 1080 on a 6000 × 4000 RAW) and whose first preview is still queued gets a `master` frame when an export's master lands first (masters bypass the preview lane). Adjusted stays null, so UI `canCompose` (frame + rect + original) turns Compose on and `composePreview` shows the cropped 1:1 JPEG stretched into the 3:2 frame with the camera line. Reproduced: frame 600 × 400, source `master`, adjusted `None`, original present. Contradicts C1(b) and 10D. Fix: the camera-preview branch requires the frame to come from the camera preview (expose the frame source, or a flag, on `CompositionDto`). **F2 (B2, route production):** the C3(b) test sets `new_arrival` directly on `request_render`. If captures stopped being marked as arrivals, it would stay green. Fix: drive the two new photos through the capture hook. Notes → backlog (4). Lesson for the plan: C1(c)'s literal rect is not 4:5 on a 3:2 frame (the implementer's deviation is correct).

- 2026-11-09 fix r2 / fix 1 (claude, pipeline-implementer, Sonnet): **F1 (B1) and F2 (B2) resolved.** F1: `CompositionDto` gains `frame_source` (`camera` / `preview` / `master`, null until a frame exists), read from the fingerprint's source; `canCompose` now needs a rect and a frame plus either an adjusted preview, or the camera original **and** `frame_source === 'camera'`, so a frame adopted from an export's master no longer makes a non-matching camera preview composable (`composePreview` follows, it calls `canCompose`). Bindings regenerated with `cargo test -p psm-desktop export_bindings`. Added the UI case "frame from `master`, no adjusted preview → Compose disabled, waiting sentence" to C1(b) (proved red when the same input says `camera`); the C1(a)/(c)/(d) camera fixtures now name `frame_source: 'camera'`; the shared compose fixtures (harness, machine test) default to `preview`; inline composition fixtures in 7 older UI tests got `frame_source: null` only because the DTO type now requires the field. F2: C3(b) now delivers `new-1` and `new-2` through the capture hook (new helper `captured`, as `camera_photo` does) behind 10 backlog requests; the arrival marking is the production path (`wait_for_variant` / `request_initial`). Judgment call: no backend test added for F1 (the finding allowed one "if a backend case is added"; the DTO field is a plain read of the stored fingerprint source, covered through the UI case). Observation: `p33_split_grid` C4(d) failed once in one full UI run on a 1 s timeout and passed alone and in the next full run (flaky under load, not related).
