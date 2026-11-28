---
plan: LV10_zoom_reframe_guides
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-27
actor: opus
---
# Handoff — LV10_zoom_reframe_guides, review r1

**Verdict: APPROVED_WITH_NOTES.** No B1–B4 finding. Six notes went to the backlog. No owner cards.

## Gate
- The Tracker read `IMPLEMENTED (2026-11-27, fable)`.
- Checkpoints `05729f61` (U1), `f594e337` (U2), `92dfbef3` (U3), `1764d2d6`, `21ffb636`, `764ba540` (U4) and the close
  commit `814990a9` all exist in `git log`. Gate passed.

## Verified correct
- **Perimeter.** `git diff --stat 05729f61^ 814990a9` lists 30 paths: the handoff's 29 code, test and guide paths,
  plus the plan file, which `814990a9` alone changes. There is no extra or missing path. `apps/desktop/src/features/compose/*`
  is untouched.
- **Checks, run once on code tree `764ba540` (the plan's Review mode):**

  | Command | Result |
  |---|---|
  | `cargo test -p psm-testkit --test suite lv10_phone_composition::` | 9 passed, 0 failed |
  | `cd apps/desktop && npx vitest run src/phone` | 22 files, 283 passed |
  | `npm run typecheck` | exit 0 |
  | `npm run build` | exit 0 (chunk-size warning only) |
  | grep `__TAURI` / `tauri` in `dist/assets/phone-*.js`, `useComposePrefs-*.js` | 0 / 0 |
  | grep `@/services/bindings`, `@tauri-apps` in `src/phone` (non-test) | none; only type imports of `@/services/compositions` |

  The implementer's full-suite stamp stands: `cargo test --workspace` 2791, clippy, fmt, the sidecar stub, and
  `npm test` 1238. None of my results contradicts it.
- **U1 (H8).** `CompositionService::set_at_revision` and `reset_at_revision` read the stored revision. A mismatch
  returns `Error::CompositionChanged` before anything is saved. Otherwise they call exactly `set` / `reset`: MC-27
  validation, `FrameUnknown`, one "Compose NN" step, and `PhotoChanged` + `SessionChanged` after the commit. The only
  extra rule is that unchanged values with state `ok` save nothing, which row C2(b) and PH4b require.
  - The routes only parse the body, run `guard` and `session_photo`, call the service and return `{revision}`.
  - PH6c sends `composition` only with values, a frame and an adjusted preview on disk.
  - `PhoneImage.revision` is the composition's revision, so Done is checked against the framing the editor opened on.
- **U2.** `pinch.ts` is pure. `Preview` handles each touch sequence as one gesture and measures the viewport once,
  never the zoomed layer.
  - A second finger ends a swipe or hold with nothing sent.
  - `panning` and `waiting` behave as PH11 says. Double tap works both ways.
  - Cancel, blur, hidden and unmount settle the zoom where it is. Another photo resets the zoom to 1×.
  - Composed display uses the Mac's `composedLayout`.
- **U3.** `reframe.ts` uses the Mac's `validate`, `panBy`, `zoomAbout`, `defaultRect`, `turnedFrame` and `refit`, and
  copies none of them.
  - The bounded moves run on every move, in PH12's order, from the gesture's base values.
  - Controls apply `refit`. The grid shows only while the twist is past 3° or the slider is held.
  - Guides use `overlayLines` inside the frame box. Pointer capture is taken on down, and cancel, blur and hidden
    keep the last values.
- **U4.** Done follows PH12's three cases with one request per Done, and the four refusal branches are as specified.
  The editor closes when its photo leaves the session or loses its composition. The live guides are an SVG with
  `pointer-events: none` in the viewport's own px, they handle split shots, and the choice is stored in
  `psm.phone.liveGuides` (default `off`).
- **Tests (flow §4, B2).** All 53 rows have a test at the row's boundary that asserts the exact outcome. Each would
  fail on a wrong outcome; none asserts only its fixture or accepts several outcomes.
  - Rust tests check whole JSON bodies, stored state, history labels and the ordered event list.
  - UI tests check exact transform strings, compare the editor against the Mac's `viewTransform`, and check exact
    POST bodies, toasts, storage keys and line coordinates.
  - C9(c)'s fixture was chosen so its refit actually changes the rect. C11(b) proves the stop is on the boundary.
  - The editor rows read the rect at about 1e-5 instead of 1e-6; the 1e-6 proof lives in `lv10_core` and C14(a)
    (note 5).
  - The 21 unit tests cover only the two core mechanisms the plan names.
- **Assumptions 1–12.** None contradicts 10J LV-D21, 10D MC-27, PH4b, PH11 or PH12. All are in the narrative's
  "Choices made for you".
- **H1–H8.**
  - H8: the rule lives in core and the routes are thin.
  - H7: `PhoneComposition` uses core domain types, with no platform concept.
  - H3: the phone only moves CSS transforms over the Mac's preview; no pixel work.
  - H2: no RAW is touched. H1: no external write. H5: no UI automation.
  - The tests use no secrets and no live service.
- **Guide (flow §7A).** `01_product.md`, `02_architecture.md` and `04_change_impact.md` cover the routes, PH6c, the
  pinch module, the editor and the new coupling to the Mac's composition modules.

## Findings
None blocking.

## Notes written to backlog
In `docs/implementation/backlog_live/LV10_zoom_reframe_guides.md`, as master plan live §3 says; the prompt's generic
`backlog.md` is not the live track's file.
1. **A photo that needs review cannot be confirmed from the phone.** Following PH12 as written, Done without a change
   sends nothing, so the photo stays "needs review". This is a plan gap, not an implementation defect.
   - **Lesson for the coordinator:** PH12 should say "…or the photo needs review → save" and add one row (10D MC-27
     §Revisions). The server already confirms on that path.
2. `frame_unknown` and `invalid_value` on the new routes have no row and no test.
3. Nothing has been seen on a real phone or in a browser.
4. The bounded moves feel different from the Mac's editor: stuck at an edge after straightening, and zoom-out stops
   at an edge.
5. The editor rows check at display precision (about 1e-5), not 1e-6.
6. Process: checkpoint `1764d2d6` was committed with typecheck red; `21ffb636` fixed it.

## Write perimeter of this session
- `docs/implementation/plans/LV10_zoom_reframe_guides.md`: the Tracker line, and the Review log entry "review r1".
- New `docs/implementation/backlog_live/LV10_zoom_reframe_guides.md`.
- New `docs/implementation/narratives/LV10_zoom_reframe_guides.md`.
- This handoff.
- No production or test file was edited, no probe was run, and nothing was committed (the runner commits).
- `npm run build` rewrote `apps/desktop/dist/`, which git ignores.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
