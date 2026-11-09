---
plan: P58_fast_compose_on_camera_preview
role: review
round: 1
verdict: CHANGES_REQUESTED
state: DONE
date: 2026-11-09
actor: opus (pipeline-reviewer, Opus 5.5)
---

# P58 review r1: CHANGES_REQUESTED (2 blocking, both small)

Production is right almost everywhere. Two blocking items:
- **F1 (B1):** a photo whose camera preview does not match the frame can become composable on that preview through an export's master. Reproduced.
- **F2 (B2):** the C3(b) test never delivers a capture, so it would stay green if captures stopped counting as arrivals.

Suite, lint and typecheck are green.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## Gate and perimeter
- Tracker row was `IMPLEMENTED`; checkpoint `3b1351a` exists.
- `git show --stat 3b1351a` lists 35 paths: exactly the 33 declared code/test paths, plus the plan Review log and the master-plan tracker row. Nothing extra, nothing missing.

## Evidence (tree `3b1351a`, `git status --porcelain` clean)
| Command | Result |
|---|---|
| `cargo test --workspace` | green. Cargo stops at the first failing target; it ran through `p58_fast_compose_on_camera_preview` and the doc tests with no failure. |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo fmt --all -- --check` | exit 0 |
| `cd apps/desktop && npm run typecheck` | exit 0 |
| `cd apps/desktop && npm test` | 58 files, 418/418 |
| Reviewer variation (scratch `crates/testkit/tests/zz_p58_review_scratch.rs`, created, run, **deleted**; tree clean afterwards) | (1) C3(b) through real captures: order `new-2, new-1, old-2, old-3` ✔. (2) F1 reproduction: non-matching original + export while the first preview is queued → composition frame 600 × 400, source `master`, rect set, `adjusted_preview_path = None`, original present. |

## Verified correct
- **"Original matches the camera frame":**
  - `rawpreview::raw_frame` prefers DNG `DefaultCropSize` 0xC620, falls back to the largest JPEG-less IFD size, and returns `None` when neither exists (fails closed).
  - `compose_frame` checks aspect within 1 % and long edge ≥ 1600, and returns the frame after orientation (5–8 swap the sides).
  - Unit cases cover the A7R V, its padded tile size, the ZV-E10 crop, the 1600 px boundary and a missing frame. The implementer's real-file probe matches the plan's measured facts.
- **Camera frame only on a match:** `OriginalPreviewJob` adopts a `FrameSource::Camera` frame only when the extractor matched. C1(a) and C1(b) Rust tests drive this through the capture hook.
- **Swap keeps the composition (C1(c)):**
  - `adopt_frame_tx` ignores a camera frame once any frame exists.
  - On Capture One's frame it keeps the fractions, revision and `ok` state, and writes Capture One's geometry.
  - `refit_rect` changes the rect only when `validate` fails.
  - The Rust test covers both "unchanged" and "re-fitted and valid". The UI test covers the in-place image swap, the line disappearing, and no save being sent.
- **C1(d):** the stage composed box is 4:5 on the original, and the tile is composed on the original.
- **C2(a):** the export waits for Capture One's master. The output is 192 × 240, cut from the 600 × 400 master in its colour. A camera-preview export would differ in both size and colour, so the test can fail.
- **Render queue (`RenderRegistry::pop_next`):**
  - Masters bypass the preview lane and wait only for a render of the same RAW (MC-16 base-name matching preserved).
  - Previews go one at a time: focused → newest arrival (`rposition`) → oldest.
  - A queued request that Capture One refuses is surfaced (`refused_renders`): a master fails its export, a preview backs off. Nothing hangs silently.
- **C3(a):** the UI reports the stage photo through `photos_preview_focus`, a thin command calling `PreviewService::set_focus`. The Rust test shows #15 is the next render sent, and the oldest again once focus is cleared.
- **Changed existing assertions** (p31 C2(d) fixture, p06 C3(a) finishing two renders) follow from the plan's behaviour change. The order p06 asserts is unchanged.
- **Hard constraints H1–H8:**
  - H3: the camera JPEG is only turned with CSS (display).
  - H7: the port additions (`set_render_focus`, `refused_renders`, `new_arrival`) and `FrameSource::Camera` are provider-neutral.
  - H8: the command is thin.
  - H2 and H4 are untouched.
  - No live services in the suite.
- **Assumptions 1–8:** none contradicts the book. They go to the owner in plain words at approval.

## Findings

### F1 (B1, route: production): a non-matching camera preview becomes composable through an export's master
- **What is wrong:** take a photo whose embedded JPEG does not match the camera frame (ZV-E10: 1088 × 1080 on a 6000 × 4000 RAW), and run an export while that photo's first preview is still waiting in the preview queue. Masters bypass the single preview lane, so the master can finish first.
  - `ExportService` then calls `adopt_frame(Master)`, which materialises a frame and rect (fingerprint was `None`). `adjusted_preview_path` stays `null`.
  - The UI's `canCompose` needs only "adjusted or original present, plus frame and rect", so it now returns `true`.
  - `composePreview` then returns the original with `camera: true`. The editor opens on the square crop stretched into the 3:2 frame, under the line "Camera preview — Capture One's version is on its way".
  - Fractions saved there are applied to the master at export, so the exported crop is not what the photographer saw.
  - Before P58 this state showed Compose disabled. Reproduced by the reviewer variation above.
- **Authority:** plan row C1(b) (an original that does not match keeps Compose disabled with `Waiting for Capture One's full-frame preview`); book 10D §What the photographer gets; changelog 39 (the camera preview is composable *when it matches the camera frame*).
- **Correction:** compose on the camera preview only when the composition's frame came from the camera preview. Expose the frame source, or a "camera preview matches" flag, on the composition DTO. `canCompose`/`composePreview` then require either an adjusted preview, or an original plus a `camera` frame source. Extend the C1(b) UI test with this input (frame from `master`, no adjusted preview → Compose disabled, waiting sentence).
- **Files the fix may touch:**
  - `apps/desktop/src-tauri/src/commands/compositions.rs` (`CompositionDto`, `From<Composition>`)
  - `apps/desktop/src/services/bindings.ts` (regenerated with `cargo test -p psm-desktop export_bindings`)
  - `apps/desktop/src/services/compositions.ts`
  - `apps/desktop/src/features/compose/__tests__/harness.ts*` (default composition fixture)
  - `apps/desktop/src/features/compose/__tests__/p58_compose_on_camera_preview.test.tsx`
  - any existing UI test fixture that must now name a frame source
  - `crates/testkit/tests/p58_fast_compose_on_camera_preview.rs`, if a backend case is added

### F2 (B2, route: production): the C3(b) test never delivers a capture
- **What is wrong:** `c3b_newest_arrivals_precede_oldest_backlog` passes `new_arrival = true` straight into `EditingProvider::request_render`. The row's input is "2 new photos arrive". The production decision that a capture is an arrival lives in `PreviewService` (`request_initial` → `request(.., true)`, `wait_for_variant`).
  - If that marking were wrong, new captures would wait behind the backlog, and this test would stay green.
  - The behaviour is currently correct: the reviewer's variation drove two captures through the capture hook and got the right order. Only the proof is missing.
- **Authority:** flow.md §4 (one behavioural test per row, at the boundary the row names) and §5 B2; plan row C3(b).
- **Correction:** in the C3(b) test, deliver the two new photos through the capture hook (as `camera_photo` does), with the 10 old requests queued first. Assert the next two renders are the newest capture, then the other, then the backlog oldest-first.
- **Files the fix may touch:** `crates/testkit/tests/p58_fast_compose_on_camera_preview.rs` only.

## Notes written to backlog
1. Previews now go to Capture One one at a time. A long re-render backlog may take longer end to end; time it on hardware.
2. Arrival priority and focus are in memory. After a restart, never-rendered captures queue as backlog.
3. An orientation mismatch between the camera tag and Capture One sends early compositions to `needs_review`.
4. Registry and plan drift: new names (listed) are missing from the master plan registry; C1(c)'s literal rect is not 4:5.

## Lessons for the plans
- A criterion that enables a feature on a derived condition ("frame present ⇒ camera preview usable") should name the other producers of that condition. Here the master frame from an export also produces "frame present, no adjusted preview".
- Fixture rects in criteria should be checked against the shape and frame they name (C1(c)).

## Write perimeter (this session)
- `docs/implementation/handoffs/reviewer/handoff_P58_fast_compose_on_camera_preview_review_r1.md` (this file)
- `docs/implementation/plans/P58_fast_compose_on_camera_preview.md` (Review log entry appended)
- `docs/implementation/master_plan.md` (tracker row P58 only → `CHANGES_REQUESTED`)
- `docs/implementation/backlog.md` (4 notes appended)

Other files:
- **Production code:** none touched.
- **Temporary test:** `crates/testkit/tests/zz_p58_review_scratch.rs` was created, run and deleted (`git status` clean afterwards).
- **Scratch outside the repo:** `/tmp/p58rev/`, plus suite logs in `/tmp/p58rev_*.log`.
- **Not written:** no narrative (written only on approval). No commits.
