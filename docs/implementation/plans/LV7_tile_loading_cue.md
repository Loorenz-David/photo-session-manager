---
phase: LV7_tile_loading_cue
title: A quiet loading shimmer on a phone tile whose picture is still arriving — instead of plain grey, a slow, faint light sweep over the grey placeholder, so the person sees the photo is on its way without feeling they must wait to shoot again
difficulty: normal
projection: no
depends_on: [LV6_phone_dock]
implementer: fable
blocked_on: none
---
# LV7 — Tile loading cue (owner request R2 at the hardware check)

**Tracker:** APPROVED (2026-11-27, opus) — review r1 APPROVED_WITH_NOTES: perimeter = the declared paths of `009ac38b` (nothing outside `apps/desktop/src/phone/` and the guide); phone tests 107 passed, typecheck and build green (reviewer run on `8fec7f7c`); all 4 rows have a boundary test that fails on a wrong outcome; peak ~5 % lighter than the fill (limit ~12 %); reduced-motion rule present; H1–H8 hold; notes in `backlog_live/LV7_tile_loading_cue.md`.

(This line is the milestone's tracker row. Sessions change only this line of this file. The gate for `depends_on` is
the Tracker line of the plan named there.)

## Goal
Owner, 2026-11-27, during the 10J hardware check (STATUS_live, request R2): "while the picture is arriving i see the
gray picture i would like to see some small loading animation that doesn't disturb (like gradient loading on the
current preview picture slightly, not so much so it doesn't look that i have to wait to take the next picture)".
The waiting placeholder of `apps/desktop/src/phone/components/PhotoImage.tsx` (`.ph-photo-wait`, LV6 F7) gets a
subtle animated light sweep. Nothing else changes: no spinner, no text on dock tiles, the shutter and every gesture stay
available while a tile waits.
**Not here:** any server change; the full Preview's "Preparing preview…" text (it may share the same sweep behind its
text); anything outside `apps/desktop/src/phone/`.

## How to work this plan (flow.md §10)
One unit. After it, the unit test commands, then a `CHECKPOINT (not approved):` commit of its paths. Row ids `LV7·C<n>`.
Open details: the design README's tokens and motion style first
(`claude_design/live_preview/design_handoff_shoot_companion/README.md`, local), then the smallest option. Reviewer notes:
`docs/implementation/backlog_live/LV7_tile_loading_cue.md`.

## Read first
`docs/implementation/plans/LV6_phone_dock.md` (U2: `PhotoImage`, the placeholder, the retry rule) ·
`apps/desktop/src/phone/components/PhotoImage.tsx`, `apps/desktop/src/phone/phone.css` (`.ph-photo-wait`, the
existing `@keyframes`), `apps/desktop/src/phone/__tests__/lv6_images.test.tsx` · the design README (colours, motion).

## Files
Edited: `apps/desktop/src/phone/phone.css`, `apps/desktop/src/phone/components/PhotoImage.tsx` (only if a class or
attribute is needed). Tests: `apps/desktop/src/phone/__tests__/lv7_loading_cue.test.tsx` (new) or rows added to
`lv6_images.test.tsx`. Guide: one line in the phone section of `docs/guide/` that already describes the placeholder, if
one does.

## Core mechanisms
None (CSS and markup).

## Ordered tasks
1. Give the waiting placeholder a slow, low-contrast light sweep (a moving gradient over the existing grey, e.g. a
   1.6–2 s linear loop, peak no more than ~12 % lighter than the grey); the tile's number tag and "captured" check stay
   on top and readable.
2. Respect `prefers-reduced-motion: reduce` (no animation; plain grey as today).
3. The sweep shows only while the placeholder shows: the photo replaces it as soon as it loads (LV6 behaviour kept).
4. Tests for the rows; run the unit test commands.

## Criteria (3 criteria, 4 rows)
Boundary: render the tile (`Dock` within `PhoneApp`, or `PhotoImage` directly as `lv6_images.test.tsx` does) with the
Mac faked at `fetch`/`WebSocket`; jsdom does not run CSS animations, so rows assert the class/attribute that carries the
animation and the stylesheet rule that defines it.
| Row | Boundary | Input | Exact outcome | Trace |
|---|---|---|---|---|
| LV7·C1(a) | waiting tile | a photo whose `image.preview` is null | the placeholder (`data-testid="photo-wait"`) carries the loading-cue class or attribute; `phone.css` defines an `@keyframes` animation applied to it with a duration between 1.2 s and 2.5 s, `infinite` | R2 |
| LV7·C1(b) | same → loaded | then a session message gives it a preview and the image loads | the placeholder (and its cue) is gone; the `<img>` shows | R2, LV6 F7 |
| LV7·C2(a) | reduced motion | `phone.css` | a `@media (prefers-reduced-motion: reduce)` rule sets `animation: none` on the cue | R2 (not disturbing) |
| LV7·C3(a) | nothing blocks shooting | a waiting tile in the dock; tap the shutter | the capture request is sent as without a waiting tile; the waiting tile takes no pointer events beyond its own tap (a tap still opens/selects as LV6) | R2 ("doesn't look like I have to wait") |

## Delegated choices
Gradient angle and colours from the design tokens; exact duration within the row's range; whether the full Preview's
waiting state uses the same sweep behind its text.

## Manual try-out
1. Install the build, open a templated session, pair the phone.
2. Take a picture from the phone: its tile shows a faint light moving across the grey for a few seconds, then the photo.
3. Take the next picture at once: nothing stops you while the previous tile is still shimmering.
4. On iPhone, Settings → Accessibility → Motion → Reduce Motion on: the waiting tile is plain grey, no movement.

## Unit test commands
`cd apps/desktop && npx vitest run src/phone` · `npm run typecheck` · `npm run build`

## Closing gate (once)
`cd apps/desktop && npm run typecheck` · `npm test` · `npm run build`. No Rust or C++ file changes, so the Rust gate is
not run (flow §10: no app Rust file changed); cite LV6's stamp for it and say so.

## Review mode
The reviewer runs the unit test commands once, reads the CSS for the reduced-motion rule and the contrast limit, and
checks nothing outside `apps/desktop/src/phone/` and the guide changed. Only B1–B4 block.

## Review log

### 2026-11-27 — implement r1 (fable)
**Built.** `phone.css`: `.ph-photo-wait` gets `overflow: hidden; isolation: isolate` and a `::before` band
(`linear-gradient(100deg, transparent 20%, #fff 50%, transparent 80%)`, opacity 0.9, `pointer-events: none`,
`z-index: -1` so the placeholder's own text stays above it) moved by `@keyframes ph-photo-sweep`
(`translateX(-100% → 100%)`, `1.8s linear infinite`); `@media (prefers-reduced-motion: reduce)` sets
`animation: none` and the band rests out of sight (its base transform is `translateX(-100%)`), so the tile is plain
grey as before. `PhotoImage.tsx` is unchanged. Guide: the image-rule line in `docs/guide/02_architecture.md`.
Tests: `apps/desktop/src/phone/__tests__/lv7_loading_cue.test.tsx` (4 tests, one per row).

**Assumptions (delegated choices).**
1. The cue is carried by the existing class `ph-photo-wait` (row C1(a) allows "class or attribute"); no new class, no
   markup change.
2. Peak colour is white over `--ph-fill` (#f1f1ef): about 6 % lighter, under the plan's ~12 % limit. Angle 100°,
   duration 1.8 s, linear (the plan's example range).
3. Every placeholder sweeps, wherever `PhotoImage` draws one: dock tiles, Move drawer tiles and the full Preview
   (behind "Preparing preview…"), the delegated choice taken as "same sweep".
4. A picture that failed to load sweeps too, including one whose three retries are used up (it then sweeps until a
   new preview version arrives). Telling "still arriving" from "gave up" needs a class from `PhotoImage` and a row the
   plan does not have; left as is, noted for the reviewer/backlog.

**Deviations.** None. No Rust or C++ file changed; the Rust gate was not run (plan's Closing gate), LV6's stamp stands.

**Observations.** The frontend tsconfig has no Node types: the test reads `phone.css` with `node:fs` under the same
`@ts-expect-error` line `lv6_dock.test.tsx` uses. The first closing-gate run failed typecheck and build on the new test
file only (typing of `matchAll` callbacks, then that import); fixed, and typecheck, the phone tests and the build
re-run green. `npm test` (1062 passed) ran before that fix; the fix changed only the new test file, which the
phone-tests re-run covers. The look was not seen on a phone or in a browser in this session: rows are proven on the
markup and the stylesheet text (jsdom runs no animation); step 2 of the manual try-out is the visual check.

### 2026-11-27 — review r1 (opus): APPROVED_WITH_NOTES
**Verified correct.** Perimeter: `git show --stat 009ac38b` = the four declared paths (`phone.css`, the new
`lv7_loading_cue.test.tsx`, `docs/guide/02_architecture.md`, this plan); nothing outside `apps/desktop/src/phone/` and
the guide; `PhotoImage.tsx` unchanged. Unit test commands run once on `8fec7f7c`: `npx vitest run src/phone` 10 files,
107 passed; `npm run typecheck` exit 0; `npm run build` exit 0 (chunk-size warning only). The implementer's
`npm test` stamp (1062) stands: the later fix touched only the new test file, covered by the phone run. CSS read:
`.ph-photo-wait` is positioned, `overflow: hidden` clips the band to the tile, `isolation: isolate` + `z-index: -1`
paints the band over the grey fill and under the placeholder's text; peak = white at opacity 0.9 over `#f1f1ef`
≈ 254 vs 241, about 5 % lighter (limit ~12 %); `@keyframes ph-photo-sweep`, `1.8s linear infinite`; the
reduced-motion block sets `animation: none` on the same `::before` selector, whose resting transform parks it outside
the clipped tile, so the tile is plain grey. Rows: C1(a) asserts the class on the real placeholder and the single
animation declaration's keyframes, duration range and `infinite` from the shipped stylesheet; C1(b) sends a session
message with a preview and asserts the placeholder gone and the exact preview URL on the `<img>` (`PhotoImage`
swaps as soon as the token is a string, LV6 behaviour kept); C2(a) asserts the reduced-motion rule stops exactly the
selector that animates; C3(a) asserts `pointer-events: none` on placeholder and band, one `/capture` post
`{session_id: 7}` with a waiting tile, and a tap on the waiting tile opens its Preview. Each fails on a wrong outcome;
none asserts internals. Assumptions 1–4 do not contradict 10J or the plan (task 3 ties the sweep to the placeholder).
H1–H8: CSS and one test file; nothing touched. Guide line updated.
**Notes** (`backlog_live/LV7_tile_loading_cue.md`): a picture that gave up after its retries sweeps forever, so it
looks like it is still arriving; the test reads `phone.css` by a path relative to `apps/desktop`.
