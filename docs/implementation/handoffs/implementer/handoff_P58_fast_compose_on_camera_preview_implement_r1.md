---
plan: P58_fast_compose_on_camera_preview
role: implement
round: 1
state: DONE
date: 2026-11-09
actor: claude (pipeline-implementer, Opus 5.5; continued the uncommitted partial Codex tree of the stopped r1 run)
---

# P58 implement r1 — compose on the camera preview; render what matters first

## What was built
- **Compose at once on the camera's own preview.** When a photo arrives, the app already copies the RAW's embedded
  JPEG (MC-20). It now also checks whether that JPEG **matches the camera frame**: the same aspect ratio as the RAW's
  sensor frame within 1 %, and at least 1600 px on the long edge. If it matches, the photo gets its composition frame
  straight away from that preview (oriented by the RAW's orientation), so **Compose is available before Capture One
  has rendered anything**. The editor shows the camera preview, turned upright, with the line
  `Camera preview — Capture One's version is on its way`. The stage and the tiles already showed the composed
  picture on the original; they now do it for these photos too.
- **Swap in place.** When Capture One's adjusted preview arrives, the editor, stage and tiles switch to it. The
  stored composition (fractions, MC-27) stays as it is, with the same revision, still `ok`, and the line disappears. The
  composition is re-fitted only if it would no longer fit Capture One's frame (locked shape off by more than a pixel,
  or a straightened rect leaving the image). A later camera preview never replaces a frame Capture One has measured.
- **Photos whose preview does not match** (the ZV-E10's 1088 × 1080 JPEG of a 3:2 RAW, or anything under 1600 px)
  keep today's behaviour: Compose disabled, with `Waiting for Capture One's full-frame preview`.
- **Exports unchanged.** Exports and masters still come only from Capture One's master (MC-28). The camera preview is
  never exported.
- **Render what matters first.** Capture One now gets **one preview render at a time** from the app, from a queue in
  this order: (1) the preview of the photo on the stage or in the editor, (2) the newest arrivals (photos with no adjusted
  preview yet, most recently queued first), (3) everything else oldest-first. The workspace reports the photo on the
  stage or in the editor (`photos_preview_focus`). A change of photo re-orders what is still waiting. A render already
  sent to Capture One is never cancelled. Export renders (masters) are sent as before: at once, except one waits while a
  render of the same photo is still out.
- **Checked on real files on this Mac** (throwaway probe in `/tmp`, nothing in the repo): the owner's A7R V ARW
  (`~/Pictures/testing camaer/Trash/testing camaer4080.ARW`) → composable, frame 6240 × 4160. A ZV-E10 ARW
  (`~/Pictures/test-wirless-camera/Capture/test-wirless-camera1951.ARW`, 1:1 JPEG 1088 × 1080, sensor 6000 × 4000) →
  not composable. An older Sony portrait ARW → composable, 1080 × 1616.

## Tests → criterion rows
| Row | Test | Shape vs the row |
|---|---|---|
| C1(a) | `crates/testkit/tests/p58_fast_compose_on_camera_preview.rs::c1a_full_camera_preview_materializes_composable_frame` (capture hook → original-preview job: frame 6240 × 4160 from the camera preview for orientation 1, 4160 × 6240 for orientation 6, rect set, no adjusted preview) + UI `p58_compose_on_camera_preview.test.tsx` "C1(a) …" (R opens the editor on the original, turned 90°, laid out with sides exchanged, line visible) | exact |
| C1(b) | Rust `c1b_cropped_or_small_camera_preview_keeps_compose_waiting` (1088 × 1080 and 1500 × 1000 on a 6000 × 4000 RAW → no frame, no rect) + UI "C1(b) …" (R does nothing, Compose disabled, the waiting sentence) | exact |
| C1(c) | Rust `c1c_adjusted_preview_replaces_provisional_frame_without_changing_saved_rect` (4:5 rect (0.1, 0.1, 0.32, 0.6) composed on the camera frame; the adjusted preview arrives → same shape, rect, revision, `ok`, frame = Capture One's; second case: a 3:2-locked full rect re-fitted inside a 300 × 203 frame) + UI "C1(c) …" (editor switches to the adjusted image in place, line gone, no save sent) | exact (rect: see Deviations) |
| C1(d) | UI "C1(d) …" (stage composed box 4:5 on the original, tile composed on the original) | exact |
| C2(a) | Rust `c2a_export_requested_on_original_waits_for_capture_one_master` (composed on the camera preview only; export → no output until Capture One's master; output = 192 × 240 cut from the 600 × 400 master in the master's colour) | exact |
| C3(a) | Rust `c3a_focused_photo_is_next_after_current_render` (20 requests oldest-first; `previews.set_focus(#15)` → next sent to the fake Capture One is #15; focus cleared → oldest waiting) + UI "C3(a) …" (the workspace reports the stage photo) | exact |
| C3(b) | Rust `c3b_newest_arrivals_precede_oldest_backlog` (10 old + 2 new → new-2, new-1, then the backlog oldest-first) | exact |
| C4(a) | full suites (Stamp) | — |

Core mechanisms (unit tests, as the plan allows): `crates/infra/src/rawpreview.rs` `p58_raw_frame_is_the_default_crop_size`,
`p58_original_matches_the_camera_frame` ("original matches the camera frame"); `crates/integrations/src/editing/capture_one/renders.rs`
`focused_then_newest_arrivals_then_oldest_backlog` (render-queue ordering).

Existing assertions changed (all because Compose availability or the one-at-a-time queue changed):
- `apps/desktop/src/features/compose/__tests__/p31_compose_mode.test.tsx` C2(d): its "cannot be composed" photo now also
  has no frame/rect. A photo with a frame and a camera original is composable after P58.
- `crates/testkit/tests/p06_live_previews.rs` `c3a_focus_renders_stale_active_rows_first`: Capture One receives one
  preview at a time, so the test finishes the first two renders before reading the order. The order asserted is unchanged.
- `crates/testkit/tests/p06_live_previews.rs` keeps the order assertion; nothing else in P09/P29 changed.
- `p48`, `p54`, `p57` tests: the extra `new_arrival` argument of `request_render` (`false`), no assertion changed.

## Assumptions
1. **RAW frame source** (delegated): the DNG `DefaultCropSize` tag (0xC620) Sony writes in the raw SubIFD; if absent, the
   largest ImageWidth × ImageLength of an IFD that carries no embedded JPEG; neither → not composable. Sony's tiled raw
   pads ImageWidth/ImageLength (A7R V: 6656 × 4608 for 6240 × 4160), which would have wrongly rejected the owner's camera.
2. **Where the decision runs** (delegated): in the backend. The extractor decides; a match gives the composition its frame
   (source `camera`). The UI treats "frame present, no adjusted preview" as "compose on the camera preview". No new DTO field.
3. **New arrival** = a capture whose first preview request has not yet reached Capture One. That is the first request
   at arrival, kept through backoff, plus Wi-Fi captures waiting for Capture One to list their variant (`wait_for_variant`).
   "Most recent" = most recently queued. A first attempt was "any RAW with no adjusted preview". It broke P06 C3(a)'s
   active-session-first order for old, never-rendered photos, so it was narrowed. A queued arrival that Capture One
   refuses at its turn retries as backlog (rare).
4. **Only previews go one at a time; masters are unchanged.** Masters are still sent at once, waiting only for a
   render of the same RAW (outputs are matched by base name, MC-16). The partial tree had put masters in the
   same single lane. That serialised MC-16's 120 s + 120 s timeouts per photo (a stuck 30-photo export would fail
   after about two hours instead of about four minutes) and failed P09 C6(b). If Capture One refuses a queued request
   when its turn comes, a master fails its export with that error and a preview is asked for again with the usual
   backoff. The disk-space guard is still checked when a master is requested.
5. **Focus** is reported by the workspace for the photo on the stage or in the editor (`photos_preview_focus`), and
   cleared when the workspace closes.
6. **Orientation differs between camera preview and Capture One's preview** (rare): an untouched default composition is
   made again on Capture One's frame; a photographer's composition goes to `needs_review` (existing MC-27 rule).
7. **Placement of the line**: a small pill at the bottom centre of the editor stage.
8. Photos that arrived before P58 keep waiting for Capture One (their camera preview was copied before this check existed).

## Deviations
- C1(c)'s literal rect (0.1, 0.1, 0.6, 0.6) is not a valid 4:5 rect on a 3:2 frame (it is 3:2; core validation
  refuses it). The tests keep shape 4:5 and x, y, h, with w = 0.8 · 0.6 · 4160 / 6240 = 0.32.
- New names not in the master plan registry (for the coordinator to fold in): `FrameSource::Camera` (`frame_source = 'camera'`),
  Tauri command `photos_preview_focus`, `EditingProvider::{set_render_focus, refused_renders}`, the `new_arrival` argument of
  `request_render`, `PreviewService::set_focus`, `Store::photos_for_raw`, `mechanisms::composition::refit_rect`,
  `rawpreview::{raw_frame, compose_frame}`, testkit `SyntheticRaw::build_with_frame`.
- The run started from an uncommitted partial Codex tree (STATUS 11:05). I reviewed it and kept its shape. I fixed four
  things in it: the RAW frame read the padded tile size (the A7R V would never have matched); the editor, output
  previews and loupe stretched a sideways camera preview; a queued render Capture One refused was dropped silently
  (a master could hang its export); and Wi-Fi arrivals were not treated as new arrivals.

## Files changed
Code and tests (33):
- `apps/desktop/src-tauri/src/commands/photos.rs`
- `apps/desktop/src-tauri/src/lib.rs`
- `apps/desktop/src/features/compose/ComposeMode.tsx`
- `apps/desktop/src/features/compose/ComposeStage.tsx`
- `apps/desktop/src/features/compose/Loupe.tsx`
- `apps/desktop/src/features/compose/OutputPreview.tsx`
- `apps/desktop/src/features/compose/SplitPreviews.tsx`
- `apps/desktop/src/features/compose/__tests__/p31_compose_mode.test.tsx`
- `apps/desktop/src/features/compose/__tests__/p58_compose_on_camera_preview.test.tsx`
- `apps/desktop/src/features/compose/compose.css`
- `apps/desktop/src/features/compose/editorPicture.ts`
- `apps/desktop/src/features/workspace/WorkspacePage.tsx`
- `apps/desktop/src/services/bindings.ts`
- `apps/desktop/src/services/compositions.ts`
- `apps/desktop/src/services/previews.ts`
- `crates/core/src/domain/composition.rs`
- `crates/core/src/mechanisms/composition.rs`
- `crates/core/src/ports/editing.rs`
- `crates/core/src/ports/raw_previews.rs`
- `crates/core/src/ports/store.rs`
- `crates/core/src/services/export.rs`
- `crates/core/src/services/previews.rs`
- `crates/infra/src/rawpreview.rs`
- `crates/infra/src/sqlite/store.rs`
- `crates/integrations/src/editing/capture_one/adapter.rs`
- `crates/integrations/src/editing/capture_one/renders.rs`
- `crates/integrations/src/lib.rs`
- `crates/testkit/src/fixtures/synthetic_raw.rs`
- `crates/testkit/tests/p06_live_previews.rs`
- `crates/testkit/tests/p48_camera_ownership.rs`
- `crates/testkit/tests/p54_camera_cable_and_take_back.rs`
- `crates/testkit/tests/p57_camera_auto_take_back_and_run2_fixes.rs`
- `crates/testkit/tests/p58_fast_compose_on_camera_preview.rs`

Documents (3): `docs/implementation/plans/P58_fast_compose_on_camera_preview.md`, `docs/implementation/master_plan.md`, `docs/implementation/handoffs/implementer/handoff_P58_fast_compose_on_camera_preview_implement_r1.md` (tracker row P58 only).

Outside the repo: throwaway probe in `/tmp/p58probe/` (reads real ARWs, writes `/tmp/p58probe/out.jpg`); no repo file was touched by it. No archgraph in this project.

## Checkpoint SHA
`3b1351a` — `CHECKPOINT (not approved): P58_fast_compose_on_camera_preview implement r1` (35 paths: 33 code/test paths + plan Review log + master-plan tracker row; ≈ 1000 lines). This handoff is left for the runner to commit.

## Stamp
Tree: code identical to checkpoint `3b1351a`.
| Command | Result |
|---|---|
| `cargo test --workspace --no-fail-fast` | exit 0 — 858 passed, 0 failed, 57 result groups (P57 approval: 849) |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo fmt --all -- --check` | exit 0 |
| `cd apps/desktop && npm run typecheck` | exit 0 |
| `cd apps/desktop && npm test` | 58 files, 418/418 passed (the known intermittent `p33_split_grid` did not fail this run) |
| `cargo test -p psm-desktop export_bindings` | bindings regenerated; `bindings.ts` adds only `photosPreviewFocus` |
Failure delta vs P57: none. Earlier runs in this session failed P06 C3(a) (first arrival rule, narrowed) and P09 C6(b) (masters in the
single lane, reverted to per-RAW); both green on the final tree.

## Manual try-out
1. Open Session Manager with Capture One running and the A7R V connected (cable or Wi-Fi).
2. Take a photo. As soon as its tile appears, select it and press **R** (or C, per your keys): the editor opens on the
   camera's picture, and the line "Camera preview — Capture One's version is on its way" sits at the bottom.
3. Reframe it (drag, pick 4:5, etc.). Wait a few seconds: the picture refreshes to Capture One's version, the line goes
   away, and the frame stays where you put it.
4. Take ten photos quickly, then click the last one: its adjusted preview arrives before the older ones.
5. Export the session: the files are cut from Capture One's full-size render, as before.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
