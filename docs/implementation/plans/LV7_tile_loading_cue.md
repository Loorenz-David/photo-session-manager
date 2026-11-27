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

**Tracker:** NOT_STARTED (2026-11-27, orchestrator)

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
