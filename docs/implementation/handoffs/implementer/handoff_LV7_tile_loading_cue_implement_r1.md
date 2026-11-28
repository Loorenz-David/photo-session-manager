---
plan: LV7_tile_loading_cue
role: implement
round: 1
state: DONE
date: 2026-11-27
actor: fable
---
# Handoff — LV7_tile_loading_cue, implement r1

The one unit is built and checkpointed; typecheck, the UI tests and the build are green. Nothing needs the owner.
(The prompt's closing line names actor `codex`; this session ran on Claude Fable as the plan's `implementer: fable`
says, so the tracker and this file say `fable`.)

## ⚠ OWNER DECISIONS REQUIRED (0)
None.

## What was built
**For the photographer.** A phone tile whose picture is still arriving is no longer plain grey: a faint band of light
crosses the grey slowly (once every 1.8 s) until the photo appears. The number tag and the "captured" check stay on
top. Nothing waits on it: the shutter and every tap work as before. With the phone's Reduce Motion setting on, the
tile is plain grey with no movement. The same sweep shows behind "Preparing preview…" in the full preview and on
waiting tiles in the Move drawer.

**Technically.** CSS only, in `apps/desktop/src/phone/phone.css`: `.ph-photo-wait` gains `overflow: hidden` and
`isolation: isolate`; a `::before` band (white gradient at 100°, opacity 0.9, `z-index: -1`, `pointer-events: none`)
is moved by `@keyframes ph-photo-sweep`, `1.8s linear infinite`; `@media (prefers-reduced-motion: reduce)` sets
`animation: none` (the band's resting transform keeps it out of sight). `PhotoImage.tsx` is unchanged. No server,
Rust or C++ change.

## Tests → criterion rows
All in `apps/desktop/src/phone/__tests__/lv7_loading_cue.test.tsx`; every test maps to one row, no other tests added.
| Row | Test | Asserts |
|---|---|---|
| LV7·C1(a) | `lv7_c1a_a_tile_without_a_preview_carries_the_cue_a_slow_endless_sweep` | tile "02" with `preview: null` shows `photo-wait` with class `ph-photo-wait`; exactly one `animation` on that class's rules outside `@media`; its `@keyframes` exists; duration within 1.2–2.5 s; `infinite` |
| LV7·C1(b) | `lv7_c1b_the_cue_leaves_when_the_picture_loads` | after a session message with preview `k1` and the image's load: no `photo-wait`; the `<img>` has the exact preview URL |
| LV7·C2(a) | `lv7_c2a_reduced_motion_turns_the_sweep_off` | the reduced-motion block sets `animation: none` on exactly the selector that carries the animation |
| LV7·C3(a) | `lv7_c3a_a_waiting_tile_does_not_hold_up_the_shutter_and_still_opens_on_a_tap` | with a waiting tile: the placeholder and its band both declare `pointer-events: none`; the shutter posts `/capture` `{session_id: 7}` once; a tap on the waiting tile opens its Preview ("SHOT 02 · CAPTURED") |

Rows C1(a), C2(a) and the pointer-events half of C3(a) read the stylesheet text, as the plan's boundary line says
(jsdom runs no animation). No red baseline was captured before the CSS edit.

## Assumptions
1. The cue rides on the existing class `ph-photo-wait`; no new class or attribute, no markup change.
2. Peak colour: white over the grey fill `#f1f1ef`, about 6 % lighter (plan limit ~12 %). Angle 100°, 1.8 s, linear.
3. Every placeholder sweeps: dock tiles, Move drawer tiles, and the full Preview behind its text.
4. A picture that failed to load sweeps too, including after its three retries are used up (until a new preview
   version arrives). Separating "gave up" from "arriving" needs a class from `PhotoImage` and a row the plan does not
   have; not done. Candidate note for `backlog_live/LV7_tile_loading_cue.md`.

## Deviations
None.

## Files changed
- `apps/desktop/src/phone/phone.css`
- `apps/desktop/src/phone/__tests__/lv7_loading_cue.test.tsx` (new)
- `docs/guide/02_architecture.md` (the image-rule line)
- `docs/implementation/plans/LV7_tile_loading_cue.md` (Tracker line, Review log entry)
- this handoff (not in the checkpoint commit; left for the runner)

No mutation probes; no file touched and reverted. No archgraph delta.

## Checkpoint SHA
`009ac38b` (parent `2da2210e`), branch `feature/live-preview`. Tree clean after it except this handoff.

## Stamp
Run in `apps/desktop`:
| Command | Result |
|---|---|
| `npm run typecheck` | first run: exit 2, errors only in the new test file (typing of `matchAll` callbacks, then the `node:fs` import). After the fix: exit 0 |
| `npm test` | exit 0; 150 files, 1062 tests passed. Run once, before the typecheck fix; the fix changed only `lv7_loading_cue.test.tsx` |
| `npx vitest run src/phone` (re-run after the fix) | exit 0; 10 files, 107 tests passed |
| `npm run build` | first run: exit 2 (same type errors). After the fix: exit 0 |

Rust gate not run (plan's Closing gate: no Rust or C++ file changed). LV6's stamp stands for it:
`cargo test --workspace` exit 0, 2760 passed (handoff_LV6_phone_dock_implement_r1.md).

**Not verified:** the look itself. Nobody saw the sweep on a phone or in a browser in this session; the visibility of
a 6 % lighter band on a 56 px tile is a judgment the manual try-out settles.

## Manual try-out
1. Install the build, open a templated session, pair the phone.
2. Take a picture from the phone: its tile shows a faint light moving across the grey for a few seconds, then the
   photo.
3. Take the next picture at once: nothing stops you while the previous tile is still shimmering.
4. Tap the shimmering tile: the preview opens with "Preparing preview…" over the same faint sweep.
5. On iPhone, Settings → Accessibility → Motion → Reduce Motion on: the waiting tile is plain grey, no movement.
If the sweep is too faint or too strong, say so: it is one opacity value.
