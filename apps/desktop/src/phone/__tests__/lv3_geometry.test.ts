// LV3·U2 — geometry and gesture mechanisms (the plan's Core mechanisms): rows LV3·C12(a)–(d), and
// the pure rules behind C8 (touch mode, overlay value), C10(b) (double tap) and C13(a) (viewport box).
import { expect, test } from 'vitest';
import { isRotated, localPoint, ratioLabel, tapToFrame, viewportBox } from '../geometry';
import { isDoubleTap, lockProgress, overlayValue, touchMode } from '../gestures';

const square = { width: 390, height: 390 };
const frame = { width: 1024, height: 680 };

test('lv3_c12a_the_centre_of_the_viewport_is_the_centre_of_the_frame', () => {
  expect(tapToFrame({ x: 195, y: 195 }, square, frame, 0)).toEqual({ x: 0.5, y: 0.5 });
});

test('lv3_c12b_the_left_edge_of_a_square_viewport_is_inside_the_cropped_frame', () => {
  const point = tapToFrame({ x: 0, y: 195 }, square, frame, 0);
  expect(point.x).toBeCloseTo(0.16797, 4);
  expect(point.y).toBeCloseTo(0.5, 4);
});

test('lv3_c12c_the_zoom_preset_narrows_the_area', () => {
  const point = tapToFrame({ x: 0, y: 195 }, square, frame, 20);
  expect(point.x).toBeCloseTo(0.23438, 4);
  expect(point.y).toBeCloseTo(0.5, 4);
});

test('lv3_c12d_a_tap_in_a_rotated_viewport_is_read_in_the_images_own_coordinates', () => {
  const rect = { left: 0, top: 100, right: 300, width: 300, height: 450 };
  const local = localPoint({ x: 150, y: 325 }, rect, true);
  expect(local).toEqual({ x: 225, y: 150, w: 450, h: 300 });
  const point = tapToFrame(local, { width: local.w, height: local.h }, frame, 0);
  expect(point.x).toBeCloseTo(0.5, 6);
  expect(point.y).toBeCloseTo(0.5, 6);
});

test('lv3_c13a_viewportBox_fills_the_stage_and_turns_long_shots', () => {
  const stage = { width: 390, height: 610 };
  expect([isRotated(1), isRotated(1.2), isRotated(1.5)]).toEqual([false, false, true]);
  expect(viewportBox(stage, 1)).toEqual({ width: 390, height: 390, rotated: false });
  expect(viewportBox(stage, 0.5)).toEqual({ width: 305, height: 610, rotated: false });
  const long = viewportBox(stage, 1.5);
  expect(long).toEqual({ width: 585, height: 390, rotated: true });
  expect(viewportBox({ width: 500, height: 610 }, 1.5)).toEqual({ width: 610, height: 610 / 1.5, rotated: true });
});

test('lv3_c8_touchMode_decides_overlay_lock_or_hold_by_the_first_threshold', () => {
  const idle = { locked: false, free: false };
  expect(touchMode(2, -14, idle)).toBeNull();
  expect(touchMode(2, -15, idle)).toBe('lock');
  expect(touchMode(2, -15, { locked: true, free: false })).toBe('none');
  expect(touchMode(14, 3, idle)).toBeNull();
  expect(touchMode(20, 3, idle)).toBe('overlay');
  expect(touchMode(-20, 3, idle)).toBe('overlay');
  expect(touchMode(40, 3, { locked: false, free: true })).toBeNull();
  expect(touchMode(20, -30, idle)).toBe('lock');
  expect(touchMode(3, 40, idle)).toBeNull();
});

test('lv3_c8_lockProgress_and_overlayValue', () => {
  expect([lockProgress(0), lockProgress(-15), lockProgress(-55), lockProgress(-200), lockProgress(30)]).toEqual([0, 15 / 55, 1, 1, 0]);
  expect(overlayValue(35, 20, 390)).toBe(42);
  expect(overlayValue(35, -390, 390)).toBe(0);
  expect(overlayValue(35, 390, 390)).toBe(100);
});

test('lv3_c10b_isDoubleTap_within_300_ms_and_50_px', () => {
  const first = { t: 1000, x: 100, y: 100 };
  expect(isDoubleTap(first, { t: 1200, x: 120, y: 100 })).toBe(true);
  expect(isDoubleTap(first, { t: 1300, x: 120, y: 100 })).toBe(false);
  expect(isDoubleTap(first, { t: 1200, x: 150, y: 100 })).toBe(false);
});

test('lv3_c13b_c20a_ratioLabel_names_the_shape', () => {
  expect([ratioLabel(1, false), ratioLabel(0.8, false), ratioLabel(1.5, false), ratioLabel(0.6667, false), ratioLabel(1.7778, false)])
    .toEqual(['1:1', '4:5', '3:2', '2:3', '16:9']);
  expect(ratioLabel(2, true)).toBe('SPLIT 2×1:1');
  expect(ratioLabel(1024 / 680, false)).toBe('3:2');
  expect(ratioLabel(1.3, false)).toBe('1.30');
});
