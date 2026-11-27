---
plan: LV7_tile_loading_cue
role: review
round: 1
verdict: APPROVED_WITH_NOTES
state: DONE
date: 2026-11-27
actor: opus
---
# Review — LV7_tile_loading_cue, round 1

**Gate.** The plan's Tracker line said `IMPLEMENTED (2026-11-27, fable)` (the live track keeps its tracker row in the plan
file, as LV6 does). Checkpoint `009ac38b` is in `git log` on `feature/live-preview`. Passed.

## Verified correct
- **Perimeter.** `git show --stat 009ac38b` = exactly the declared files changed: `apps/desktop/src/phone/phone.css`,
  `apps/desktop/src/phone/__tests__/lv7_loading_cue.test.tsx` (new), `docs/guide/02_architecture.md`,
  `docs/implementation/plans/LV7_tile_loading_cue.md`. Nothing outside `apps/desktop/src/phone/` and the guide;
  `PhotoImage.tsx` unchanged. After the checkpoint only the runner's handoff/prompt files.
- **Checks (Review mode, run once on `8fec7f7c`, in `apps/desktop`).** `npx vitest run src/phone`: 10 files, 107 passed.
  `npm run typecheck`: exit 0. `npm run build`: exit 0 (the existing chunk-size warning only). The implementer's
  `npm test` stamp (150 files, 1062 passed) stands: its later fix touched only the new test file, which the phone run
  covers. Rust gate not run, per the plan (no Rust or C++ file changed); LV6's stamp stands.
- **CSS.** `.ph-photo-wait` is already `position: absolute`. It gains `overflow: hidden`, which clips the band to the
  tile, and `isolation: isolate`. Together with the band's `z-index: -1`, the band paints above the grey fill and below
  the placeholder's own text. Contrast: a white peak at opacity 0.9 over `#f1f1ef` gives about 254 against 241, about
  5 % lighter (the plan's limit is ~12 %). Motion: `@keyframes ph-photo-sweep`, `1.8s linear infinite`, inside the
  plan's 1.6–2 s example.
  `@media (prefers-reduced-motion: reduce)` sets `animation: none` on the same `::before`. Its resting
  `translateX(-100%)` parks the band outside the clipped tile, so the tile is plain grey as before.
- **Rows.** Each test observes the row's outcome at its boundary, and each would fail on a wrong outcome:
  - C1(a): the real placeholder of tile "02" carries `ph-photo-wait`. The shipped stylesheet has exactly one animation
    for that selector or its pseudo-elements outside `@media`, its `@keyframes` exists, its duration is between 1.2 and
    2.5 s, and it is `infinite`.
  - C1(b): a session message gives the photo a preview and the image loads. The placeholder is gone and the `<img>`
    carries the exact preview URL. `PhotoImage` swaps as soon as the token is a string, so LV6's behaviour is kept.
  - C2(a): the reduced-motion block stops exactly the selector that animates.
  - C3(a): placeholder and band both declare `pointer-events: none`. With a waiting tile the shutter sends one
    `/capture` with `{session_id: 7}`, and a tap on the waiting tile opens its Preview ("SHOT 02 · CAPTURED").
  - No test counts calls or reaches into internals. The over-engineering test passes: the tests read the stylesheet
    only for the properties the rows name.
- **Assumptions 1–4.** None contradicts 10J or the plan. Task 3 ties the sweep to the placeholder, and assumption 3,
  the same sweep behind "Preparing preview…", is a delegated choice. Assumption 4 is recorded as a note.
- **H1–H8.** The diff is a stylesheet, one test file and docs. It writes nothing external and touches no RAW, pixels,
  other apps, core types or business logic. All hold. No secrets and no live services.
- **Guide.** The image-rule line in `docs/guide/02_architecture.md` describes the sweep. It is current.

## Findings
None blocking.

## Notes written to backlog
In `docs/implementation/backlog_live/LV7_tile_loading_cue.md` (the plan names this file for reviewer notes):
1. A picture that gave up after its three retries keeps sweeping, so it looks like it is still arriving. Before LV7 a
   stuck tile was plain grey.
2. The new test reads `src/phone/phone.css` by a path relative to `apps/desktop`. This is the existing convention of
   `lv6_dock.test.tsx`.

## Files written by this session
- `docs/implementation/plans/LV7_tile_loading_cue.md` (Tracker line → APPROVED, Review log entry)
- `docs/implementation/backlog_live/LV7_tile_loading_cue.md` (new)
- `docs/implementation/narratives/LV7_tile_loading_cue.md` (new)
- this handoff

No production code edited. No probes. No commits.

## ⚠ OWNER DECISIONS REQUIRED (0)
None.
