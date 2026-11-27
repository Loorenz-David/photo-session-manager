// LV10 — the core mechanisms (plan §Shared "Core mechanisms"; master_plan_live §5 PH11, PH12): `pinch.ts` (the
// two-finger maths and the finger tracker) and `reframe.ts` (the editor's bounded moves). Input → output on the
// real functions, one case per rule line.
import { expect, test } from 'vitest';
import { clampPan, type Fingers, NO_FINGERS, pinchView, settleZoom, track, twoFinger } from '../pinch';

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
