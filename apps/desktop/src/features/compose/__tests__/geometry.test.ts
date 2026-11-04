import { describe, expect, it } from 'vitest';
// The shared table (master plan §6.9): one file, run by the Rust unit test (`composition::tests`) and by this one.
import sharedTable from '../../../../../../crates/core/src/mechanisms/composition_cases.json';
import type { CompositionValues, Frame, Rect } from '@/services/compositions';
import {
  anchorIn,
  clampRect,
  defaultRect,
  fingerprintCompatible,
  frameBox,
  insideAfterStraighten,
  panBy,
  pixelRect,
  reviewReason,
  splitHalves,
  turnedFrame,
  validate,
  viewTransform,
  zoomAbout,
  type Fingerprint,
  type PixelRect,
} from '../geometry';
import { ZOOM_PER_PX } from '../machine';

type Case = { id: string; op: string; input: Record<string, unknown>; expect: Record<string, unknown> };

const table = sharedTable as { cases: Case[] };
const cases = (op: string) => table.cases.filter(c => c.op === op);

const near = (actual: number, expected: number, tolerance: number) => expect(Math.abs(actual - expected)).toBeLessThanOrEqual(tolerance);
const rectNear = (actual: Rect, expected: Rect, tolerance = 1e-9) => {
  near(actual.x, expected.x, tolerance);
  near(actual.y, expected.y, tolerance);
  near(actual.w, expected.w, tolerance);
  near(actual.h, expected.h, tolerance);
};

describe('C1(a) the TS geometry passes every case of composition_cases.json', () => {
  it('the table has every op the Rust test runs', () => {
    expect(cases('default_rect').length).toBe(11);
    expect(cases('pixel_rect').length + cases('split_halves').length).toBe(5);
    expect(cases('validate').length).toBe(13);
    expect(cases('fingerprint').length).toBeGreaterThan(0);
    expect(cases('inside').length).toBeGreaterThan(0);
  });
  for (const c of cases('default_rect')) {
    it(`${c.id} default_rect`, () => {
      const i = c.input as { kind: string; shape: string; orientation: string; frame: Frame; turns: number };
      rectNear(defaultRect(i.kind, i.shape, i.orientation, turnedFrame(i.frame, i.turns)), c.expect as unknown as Rect);
    });
  }
  for (const c of cases('pixel_rect')) {
    it(`${c.id} pixel_rect`, () => {
      const i = c.input as { rect: Rect; frame: Frame };
      expect(pixelRect(i.rect, i.frame)).toEqual(c.expect);
    });
  }
  for (const c of cases('split_halves')) {
    it(`${c.id} split_halves`, () => {
      const [left, right] = splitHalves((c.input as { pixel: PixelRect }).pixel);
      expect(left).toEqual(c.expect.left);
      expect(right).toEqual(c.expect.right);
    });
  }
  for (const c of cases('validate')) {
    it(`${c.id} validate → ${c.expect.result}`, () => {
      const i = c.input as { values: CompositionValues; frame: Frame };
      expect(validate(i.values, i.frame) ?? 'ok').toBe(c.expect.result);
    });
  }
  for (const c of cases('fingerprint')) {
    it(`${c.id} fingerprint → ${c.expect.compatible}`, () => {
      const i = c.input as { stored: Fingerprint; observed: Fingerprint };
      expect(fingerprintCompatible(i.stored, i.observed)).toBe(c.expect.compatible);
      if (!c.expect.compatible) expect(reviewReason(i.stored, i.observed)).toBe(c.expect.reason);
    });
  }
  for (const c of cases('inside')) {
    it(`${c.id} inside → ${c.expect.inside}`, () => {
      const i = c.input as { rect: Rect; frame: Frame; straighten: number };
      expect(insideAfterStraighten(i.rect, i.frame, i.straighten)).toBe(c.expect.inside);
    });
  }
});

describe('C1(b) the view maths on a 1000 × 700 stage with the default square on 2560 × 1707', () => {
  const stage = { left: 0, top: 0, width: 1000, height: 700 };
  const frame = { w: 2560, h: 1707 };
  const rect = { x: 0.16660156250000002, y: 0, w: 0.666796875, h: 1 };
  const values: CompositionValues = { kind: 'normal', shape: '1:1', orientation: 'named', quarter_turns: 0, flip: 'none', straighten: 0, rect };
  const view = viewTransform(values, frame, stage);

  it('frame box 652 × 652 at (174, 24), scale 0.381956, centre (500, 350)', () => {
    expect(view.frameBox).toEqual({ left: 174, top: 24, width: 652, height: 652 });
    expect(frameBox(stage, 1)).toEqual(view.frameBox);
    near(view.scale, 0.381956, 1e-6);
    near(view.centre.x, 500, 0.01);
    near(view.centre.y, 350, 0.01);
    near(view.width, 977.81, 0.01);
    expect(view.height).toBe(652);
    expect(view.rotateDeg).toBe(0);
  });
  it('pan by −50 px moves the rect left by 50 / (W_t·s): x = 0.115466, y = 0', () => {
    const panned = panBy(rect, { x: -50 / 977.81, y: 0 });
    near(panned.x, 0.115466, 1e-6);
    expect(panned.y).toBe(0);
    expect(panned.w).toBe(rect.w);
    expect(panned.h).toBe(rect.h);
  });
  it('zoom in by exp(100 · 0.005) about the frame centre keeps the centre under the pointer', () => {
    // 0.666796875 / e^0.5 = 0.4044327, 1 / e^0.5 = 0.6065307; the centre (0.5, 0.5) stays: x = 0.5 − w/2, y = 0.5 − h/2.
    // (The plan's rounded 0.404440 / 0.297780 came from the rounded w = 0.6668; the contract values are these.)
    const zoomed = zoomAbout(rect, anchorIn(view.frameBox, { x: 500, y: 350 }), Math.exp(100 * ZOOM_PER_PX), frame);
    near(zoomed.w, 0.404432748, 1e-6);
    near(zoomed.h, 0.60653066, 1e-6);
    near(zoomed.x, 0.297783626, 1e-6);
    near(zoomed.y, 0.19673467, 1e-6);
    near(zoomed.x + zoomed.w / 2, 0.5, 1e-9);
    near(zoomed.y + zoomed.h / 2, 0.5, 1e-9);
  });
  it('zoom out × 0.5 is clamped to the default square (never past the edge)', () => {
    rectNear(clampRect(zoomAbout(rect, { x: 0.5, y: 0.5 }, 0.5, frame)), { x: 0.166601562, y: 0, w: 0.666796875, h: 1 }, 1e-9);
  });
  it('zoom in × 20 is clamped at h = 0.05 (an 85.35 px short side), centre kept at (0.5, 0.5)', () => {
    const zoomed = clampRect(zoomAbout(rect, { x: 0.5, y: 0.5 }, 20, frame));
    near(zoomed.h, 0.05, 1e-9);
    near(zoomed.w, 0.03334, 1e-6);
    near(zoomed.h * frame.h, 85.35, 0.01);
    near(zoomed.x + zoomed.w / 2, 0.5, 1e-9);
    near(zoomed.y + zoomed.h / 2, 0.5, 1e-9);
  });
});
