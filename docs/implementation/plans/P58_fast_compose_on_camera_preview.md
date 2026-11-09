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
