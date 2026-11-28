// LV10 — the core mechanisms (plan §Shared "Core mechanisms"; master_plan_live §5 PH11, PH12): `pinch.ts` (the
// two-finger maths and the finger tracker) and `reframe.ts` (the editor's bounded moves). Input → output on the
// real functions, one case per rule line.
import { expect, test } from 'vitest';
import { panBy, validate, zoomAbout } from '@/features/compose/geometry';
import { refit } from '@/features/compose/refit';
import { clampPan, type Fingers, NO_FINGERS, pinchView, settleZoom, track, twoFinger } from '../pinch';
import { panWithin, resetValues, rotateWithin, straightenFromTwist, zoomWithin } from '../reframe';
import { K_FRAME, K_VALUES } from './helpers';

const BOX = { left: 0, top: 0, width: 390, height: 390 };
const at = (x: number, y: number) => ({ x, y });

// ---- pinch.ts: twoFinger ----

test('lv10_core_twoFinger_scale_is_the_distance_now_over_the_distance_at_the_start', () => {
  expect(twoFinger([at(100, 100), at(200, 100)], [at(50, 100), at(250, 100)]).scale).toBe(2);
  expect(twoFinger([at(100, 100), at(200, 100)], [at(125, 100), at(175, 100)]).scale).toBe(0.5);
});

test('lv10_core_twoFinger_rotation_is_the_signed_angle_clockwise_positive', () => {
  // The second finger goes from the right of the first to below it: a quarter turn clockwise on screen (y down).
  expect(twoFinger([at(100, 100), at(200, 100)], [at(100, 100), at(100, 200)]).rotation).toBeCloseTo(90, 9);
  expect(twoFinger([at(100, 100), at(200, 100)], [at(100, 100), at(100, 0)]).rotation).toBeCloseTo(-90, 9);
  // Across the ±180° seam the angle stays the short way round.
  expect(twoFinger([at(0, 0), at(-100, 10)], [at(0, 0), at(-100, -10)]).rotation).toBeCloseTo(2 * Math.atan(0.1) * 180 / Math.PI, 9);
});

test('lv10_core_twoFinger_anchor_is_the_start_midpoint_and_mid_the_current_one', () => {
  const fingers = twoFinger([at(100, 100), at(200, 300)], [at(120, 140), at(260, 360)]);
  expect(fingers.anchor).toEqual({ x: 150, y: 200 });
  expect(fingers.mid).toEqual({ x: 190, y: 250 });
});

// ---- pinch.ts: pinchView ----

test('lv10_core_pinchView_keeps_the_picture_point_under_the_fingers', () => {
  // Base 2× moved 40 px right: the picture point under the start midpoint (95, 195) is ((95 − 195) − 40) / 2 = −70
  // from the centre. Spread ×1.5 with the midpoint moved to (125, 225): scale 3, and that point is under it:
  // x = (125 − 195) − (−70 · 3) = 140; y = (225 − 195) − 0 = 30.
  const zoom = pinchView({ scale: 2, x: 40, y: 0 }, [at(45, 195), at(145, 195)], [at(50, 225), at(200, 225)], BOX);
  expect(zoom.scale).toBe(3);
  expect(zoom.x).toBeCloseTo(140, 9);
  expect(zoom.y).toBeCloseTo(30, 9);
});

test('lv10_core_pinchView_stretches_no_further_than_the_limits', () => {
  const pinchIn = pinchView({ scale: 1, x: 0, y: 0 }, [at(95, 195), at(295, 195)], [at(170, 195), at(220, 195)], BOX);
  expect(pinchIn).toEqual({ scale: 0.8, x: 0, y: 0 });
  const spread = pinchView({ scale: 4, x: 0, y: 0 }, [at(145, 195), at(245, 195)], [at(95, 195), at(295, 195)], BOX);
  expect(spread).toEqual({ scale: 7, x: 0, y: 0 });
});

// ---- pinch.ts: clampPan, settleZoom ----

test('lv10_core_clampPan_stops_at_the_pictures_edges', () => {
  // 2× of 390 × 390: half the extra width is 195 each way.
  expect(clampPan({ scale: 2, x: -495, y: 300 }, BOX)).toEqual({ scale: 2, x: -195, y: 195 });
  expect(clampPan({ scale: 2, x: -100, y: 50 }, BOX)).toEqual({ scale: 2, x: -100, y: 50 });
  // A box that is not square has its own reach on each side.
  expect(clampPan({ scale: 3, x: 500, y: -500 }, { left: 0, top: 0, width: 390, height: 200 })).toEqual({ scale: 3, x: 390, y: -200 });
});

test('lv10_core_clampPan_centres_a_picture_at_or_below_1x', () => {
  expect(clampPan({ scale: 1, x: 30, y: -30 }, BOX)).toEqual({ scale: 1, x: 0, y: 0 });
  expect(clampPan({ scale: 0.8, x: 30, y: -30 }, BOX)).toEqual({ scale: 0.8, x: 0, y: 0 });
});

test('lv10_core_settleZoom_rests_between_1x_and_6x_inside_the_edges', () => {
  expect(settleZoom({ scale: 0.8, x: 12, y: 0 }, BOX)).toEqual({ scale: 1, x: 0, y: 0 });
  // 7× → 6×; then the pan is held to 6×'s reach: (6 − 1) · 390 / 2 = 975.
  expect(settleZoom({ scale: 7, x: 1100, y: -40 }, BOX)).toEqual({ scale: 6, x: 975, y: -40 });
  expect(settleZoom({ scale: 2, x: -100, y: 0 }, BOX)).toEqual({ scale: 2, x: -100, y: 0 });
});

// ---- pinch.ts: track ----

const down = (pointerId: number, x: number, y: number) => ({ type: 'down' as const, pointerId, x, y });
const move = (pointerId: number, x: number, y: number) => ({ type: 'move' as const, pointerId, x, y });
const up = (pointerId: number, x: number, y: number) => ({ type: 'up' as const, pointerId, x, y });
const cancel = (pointerId: number) => ({ type: 'cancel' as const, pointerId, x: 0, y: 0 });
const run = (...events: Parameters<typeof track>[1][]): Fingers => events.reduce(track, NO_FINGERS);

test('lv10_core_track_one_finger_down_moves_and_lifts', () => {
  expect(run(down(1, 10, 20))).toEqual({ kind: 'one', id: 1, start: at(10, 20), at: at(10, 20) });
  expect(run(down(1, 10, 20), move(1, 30, 40))).toEqual({ kind: 'one', id: 1, start: at(10, 20), at: at(30, 40) });
  expect(run(down(1, 10, 20), move(1, 30, 40), up(1, 30, 40))).toEqual({ kind: 'idle' });
});

test('lv10_core_track_a_second_finger_starts_the_pair_where_the_first_is_now', () => {
  expect(run(down(1, 10, 20), move(1, 30, 40), down(2, 100, 40))).toEqual({
    kind: 'two', ids: [1, 2], start: [at(30, 40), at(100, 40)], at: [at(30, 40), at(100, 40)],
  });
  expect(run(down(1, 10, 20), down(2, 100, 20), move(2, 150, 60), move(1, 0, 0))).toEqual({
    kind: 'two', ids: [1, 2], start: [at(10, 20), at(100, 20)], at: [at(0, 0), at(150, 60)],
  });
});

test('lv10_core_track_a_third_finger_is_ignored', () => {
  const pair = run(down(1, 10, 20), down(2, 100, 20));
  expect(run(down(1, 10, 20), down(2, 100, 20), down(3, 200, 200), move(3, 220, 220), up(3, 220, 220), cancel(3))).toEqual(pair);
  // A finger that is not the tracked one changes nothing for one finger either.
  expect(run(down(1, 10, 20), move(2, 99, 99), up(2, 99, 99))).toEqual(run(down(1, 10, 20)));
});

test('lv10_core_track_when_one_of_two_lifts_the_other_goes_on_from_where_it_is', () => {
  expect(run(down(1, 10, 20), down(2, 100, 20), move(1, 40, 50), up(2, 100, 20))).toEqual({ kind: 'one', id: 1, start: at(40, 50), at: at(40, 50) });
  expect(run(down(1, 10, 20), down(2, 100, 20), move(2, 150, 60), up(1, 10, 20))).toEqual({ kind: 'one', id: 2, start: at(150, 60), at: at(150, 60) });
});

test('lv10_core_track_a_cancel_ends_everything', () => {
  expect(run(down(1, 10, 20), cancel(1))).toEqual({ kind: 'idle' });
  expect(run(down(1, 10, 20), down(2, 100, 20), cancel(1))).toEqual({ kind: 'idle' });
  expect(run(down(1, 10, 20), down(2, 100, 20), cancel(2))).toEqual({ kind: 'idle' });
});

// ---- reframe.ts (PH12): fixture K's framing, the default 1:1 of a 2560 × 1707 frame ----

test('lv10_core_panWithin_takes_a_valid_move_whole', () => {
  const moved = panWithin(K_VALUES, K_FRAME, { x: -0.05, y: 0 });
  expect(moved).toEqual({ ...K_VALUES, rect: panBy(K_VALUES.rect, { x: -0.05, y: 0 }) });
  expect(validate(moved, K_FRAME)).toBeNull();
});

test('lv10_core_panWithin_stops_exactly_at_the_frames_edge', () => {
  // 0.585 to the left from x = 0.1666016: the rect's left side reaches the frame's.
  const left = panWithin(K_VALUES, K_FRAME, { x: -0.585, y: 0 });
  expect(Math.abs(left.rect.x)).toBeLessThan(1e-9);
  expect({ y: left.rect.y, w: left.rect.w, h: left.rect.h }).toEqual({ y: 0, w: K_VALUES.rect.w, h: 1 });
  expect(validate(left, K_FRAME)).toBeNull();
  // To the right: the right side stops at 1.
  const right = panWithin(K_VALUES, K_FRAME, { x: 0.9, y: 0 });
  expect(right.rect.x + right.rect.w).toBeCloseTo(1, 9);
  expect(validate(right, K_FRAME)).toBeNull();
  // The rect is as tall as the frame: a move down has nowhere to go, and the sideways part stops with it.
  expect(panWithin(K_VALUES, K_FRAME, { x: 0.05, y: 0.2 }).rect).toEqual(K_VALUES.rect);
});

test('lv10_core_panWithin_stops_at_the_straightened_images_edge', () => {
  // Straightened 5° (the frame shrunk to fit), then half that size about its centre: there is room to move.
  const straightened = refit(K_VALUES, { kind: 'straighten', degrees: 5 }, K_FRAME);
  const start = { ...straightened, rect: zoomAbout(straightened.rect, { x: 0.5, y: 0.5 }, 2, K_FRAME) };
  const delta = { x: -0.4, y: -0.4 };
  const stopped = panWithin(start, K_FRAME, delta);
  expect(validate(stopped, K_FRAME)).toBeNull();
  // It moved, and it stopped on the boundary: 1 % more of the same move is outside the image.
  expect(stopped.rect.x).toBeLessThan(start.rect.x - 0.01);
  expect(stopped.rect.x).toBeGreaterThan(0);
  expect(validate({ ...stopped, rect: panBy(stopped.rect, { x: delta.x * 0.01, y: delta.y * 0.01 }) }, K_FRAME)).not.toBeNull();
  // The rect's top-left stays on the line of the move.
  expect((stopped.rect.x - start.rect.x) / delta.x).toBeCloseTo((stopped.rect.y - start.rect.y) / delta.y, 9);
  // Where the frame, shrunk just enough, already touches the image's edge, a move that way goes nowhere.
  expect(panWithin(straightened, K_FRAME, delta)).toBe(straightened);
});

test('lv10_core_zoomWithin_is_the_macs_zoom_while_that_is_valid', () => {
  const closer = zoomWithin(K_VALUES, K_FRAME, { x: 0.5, y: 0.5 }, 2);
  expect(closer.rect).toEqual(zoomAbout(K_VALUES.rect, { x: 0.5, y: 0.5 }, 2, K_FRAME));
  expect(closer.rect.x).toBeCloseTo(0.3333008, 6);
  expect(closer.rect.y).toBeCloseTo(0.25, 6);
  expect(closer.rect.w).toBeCloseTo(0.3333984, 6);
  expect(closer.rect.h).toBeCloseTo(0.5, 6);
  // Out again by 4: the Mac's zoom stops at the whole height, and that is valid.
  const out = zoomWithin(closer, K_FRAME, { x: 0.5, y: 0.5 }, 0.25);
  for (const side of ['x', 'y', 'w', 'h'] as const) expect(out.rect[side]).toBeCloseTo(K_VALUES.rect[side], 9);
});

test('lv10_core_zoomWithin_stops_at_the_largest_valid_factor', () => {
  // A quarter-size square at the frame's left edge, zoomed out about its right edge: it would grow past the left edge.
  const corner = { ...K_VALUES, rect: { x: 0, y: 0.25, w: K_VALUES.rect.w / 2, h: 0.5 } };
  expect(validate(corner, K_FRAME)).toBeNull();
  expect(validate({ ...corner, rect: zoomAbout(corner.rect, { x: 1, y: 0.5 }, 0.5, K_FRAME) }, K_FRAME)).not.toBeNull();
  expect(zoomWithin(corner, K_FRAME, { x: 1, y: 0.5 }, 0.5)).toBe(corner);
  // About its left edge the same zoom is valid and taken whole.
  expect(zoomWithin(corner, K_FRAME, { x: 0, y: 0.5 }, 0.5).rect.w).toBeCloseTo(K_VALUES.rect.w, 9);
  // Straightened 5° and shrunk to fit: zooming out stops between 1 and the asked factor, on the image's edge.
  const straightened = refit(K_VALUES, { kind: 'straighten', degrees: 5 }, K_FRAME);
  const closer = zoomWithin(straightened, K_FRAME, { x: 0.5, y: 0.5 }, 2);
  const out = zoomWithin(closer, K_FRAME, { x: 0.5, y: 0.5 }, 0.25);
  expect(validate(out, K_FRAME)).toBeNull();
  expect(out.rect.w).toBeGreaterThan(closer.rect.w);
  expect(out.rect.w).toBeLessThanOrEqual(straightened.rect.w + 1e-9);
  expect(out.rect.w).toBeGreaterThan(straightened.rect.w * 0.999);
});

test('lv10_core_rotateWithin_is_the_macs_straighten_refit', () => {
  expect(rotateWithin(K_VALUES, K_FRAME, 7)).toEqual(refit(K_VALUES, { kind: 'straighten', degrees: 7 }, K_FRAME));
  const turned = rotateWithin(K_VALUES, K_FRAME, 7);
  expect(turned.straighten).toBe(7);
  expect(turned.rect.h).toBeLessThan(1);
  expect(validate(turned, K_FRAME)).toBeNull();
  expect(rotateWithin(K_VALUES, K_FRAME, 60).straighten).toBe(45);
});

test('lv10_core_straightenFromTwist_has_a_3_degree_dead_zone_then_follows_in_tenths', () => {
  expect([0, 2, 3, -3].map(straightenFromTwist)).toEqual([0, 0, 0, 0]);
  expect(straightenFromTwist(10)).toBe(7);
  expect(straightenFromTwist(-10)).toBe(-7);
  expect(straightenFromTwist(3.26)).toBe(0.3);
  expect(straightenFromTwist(-4.74)).toBe(-1.7);
});

test('lv10_core_resetValues_is_the_default_shape_centred_as_large_as_fits', () => {
  const square = resetValues(K_FRAME, '1:1');
  expect({ ...square, rect: null }).toEqual({ kind: 'normal', shape: '1:1', orientation: 'named', quarter_turns: 0, flip: 'none', straighten: 0, rect: null });
  expect(square.rect).toEqual({ x: (1 - 1707 / 2560) / 2, y: 0, w: 1707 / 2560, h: 1 });
  expect(resetValues(K_FRAME, 'original')).toMatchObject({ shape: 'original', rect: { x: 0, y: 0, w: 1, h: 1 } });
  const wide = resetValues(K_FRAME, '16:9');
  expect(wide.rect.w).toBe(1);
  expect(wide.rect.h).toBeCloseTo((2560 / 1707) / (16 / 9), 9);
  expect(wide.rect.y).toBeCloseTo((1 - wide.rect.h) / 2, 9);
});
