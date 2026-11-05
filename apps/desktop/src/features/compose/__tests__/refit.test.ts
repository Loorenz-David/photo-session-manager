import { describe, expect, it } from 'vitest';
import type { CompositionValues, Rect } from '@/services/compositions';
import { shapeRatio, turnedFrame, validate } from '../geometry';
import { refit, trim } from '../refit';

const FRAME = { w: 2560, h: 1707 };
const DEFAULT: Rect = { x: 0.16660156250000002, y: 0, w: 0.666796875, h: 1 };
/** `652 / 1707`: the stage scale of the P31 fixture (a square frame box in the 1000 × 700 stage). */
const SCALE = 0.381956;

const values = (rect: Rect, extra: Partial<CompositionValues> = {}): CompositionValues => ({
  kind: 'normal', shape: '1:1', orientation: 'named', quarter_turns: 0, flip: 'none', straighten: 0, rect, ...extra,
});
const near = (actual: number, expected: number, tolerance = 1e-6) => expect(Math.abs(actual - expected)).toBeLessThanOrEqual(tolerance);
const rectNear = (actual: Rect, expected: Rect, tolerance = 1e-6) => {
  near(actual.x, expected.x, tolerance);
  near(actual.y, expected.y, tolerance);
  near(actual.w, expected.w, tolerance);
  near(actual.h, expected.h, tolerance);
};
/** Every result of the re-fit passes P28's validation (MC-27 §Validation). */
const valid = (result: CompositionValues) => expect(validate(result, FRAME)).toBeNull();

describe('C1(a) the re-fit keeps the centre and the area where the new ratio and the frame allow', () => {
  it('(1) shape 1:1 → 4:5 on the default square: same centre, the area asks for h > 1 so the rect fits to h = 1 (P28 case 3)', () => {
    const result = refit(values(DEFAULT), { kind: 'shape', shape: '4:5' }, FRAME);
    expect(result.shape).toBe('4:5');
    rectNear(result.rect, { x: 0.23328125, y: 0, w: 0.5334375, h: 1 }, 1e-9);
    valid(result);
  });

  it('(2) shape → original gives the whole frame', () => {
    const result = refit(values(DEFAULT), { kind: 'shape', shape: 'original' }, FRAME);
    expect(result.rect).toEqual({ x: 0, y: 0, w: 1, h: 1 });
    valid(result);
  });

  it('(3) shape → free keeps the rect', () => {
    const result = refit(values(DEFAULT), { kind: 'shape', shape: 'free' }, FRAME);
    expect(result.rect).toEqual(DEFAULT);
    expect(result.shape).toBe('free');
    valid(result);
  });

  it('(4) orientation named → turned on the 4:5 rect of (1): ratio 1.25 with the same area and centre fits — w_px = 1707, h_px = 1365.6', () => {
    const portrait = refit(values(DEFAULT), { kind: 'shape', shape: '4:5' }, FRAME);
    const result = refit(portrait, { kind: 'orientation', orientation: 'turned' }, FRAME);
    expect(result.orientation).toBe('turned');
    // Plan arithmetic: area = 1365.6 · 1707 and sqrt(area · 1.25) = sqrt(1707²) = 1707 exactly, so w = 1707/2560 and h = 0.8
    // (the plan's `0.666760 / 0.799941` carry a rounding slip; the exact contract values are asserted).
    rectNear(result.rect, { x: 0.1666015625, y: 0.1, w: 0.666796875, h: 0.8 }, 1e-9);
    valid(result);
  });

  it('(5) rotate right on the default square: the frame turns, the centre follows, the 1:1 area is kept exactly', () => {
    const result = refit(values(DEFAULT), { kind: 'turn', direction: 'right' }, FRAME);
    expect(result.quarter_turns).toBe(1);
    rectNear(result.rect, { x: 0, y: 0.166601562, w: 1, h: 0.666796875 }, 1e-9);
    valid(result);
    const left = refit(values(DEFAULT), { kind: 'turn', direction: 'left' }, FRAME);
    expect(left.quarter_turns).toBe(3);
    rectNear(left.rect, result.rect, 1e-9);
  });

  it('(6) rotate right on a 4:5 rect: in the 1707 × 2560 frame the same area at ratio 0.8 is w = 0.8, h = 0.666797', () => {
    const start = values({ x: 0.23328125, y: 0, w: 0.5334375, h: 1 }, { shape: '4:5' });
    const result = refit(start, { kind: 'turn', direction: 'right' }, FRAME);
    rectNear(result.rect, { x: 0.1, y: 0.166601562, w: 0.8, h: 0.666796875 }, 1e-9);
    valid(result);
  });

  it('(7) flip mirrors x: 1 − 0.1 − 0.5 = 0.4', () => {
    const result = refit(values({ x: 0.1, y: 0, w: 0.5, h: 0.75 }, { shape: 'free' }), { kind: 'flip' }, FRAME);
    expect(result.flip).toBe('horizontal');
    near(result.rect.x, 0.4, 1e-12);
    expect(result.rect.y).toBe(0);
    expect(result.rect.w).toBe(0.5);
    expect(result.rect.h).toBe(0.75);
    valid(result);
    expect(refit(result, { kind: 'flip' }, FRAME).flip).toBe('none');
  });

  it('(8) straighten 10 on the default square shrinks it about the centre to the largest square inside the tilted image', () => {
    const result = refit(values(DEFAULT), { kind: 'straighten', degrees: 10 }, FRAME);
    expect(result.straighten).toBe(10);
    // Geometry: a centred square of half-side a tilted by θ spans a·(cos θ + sin θ) each way, so h = 1 / (cos 10° + sin 10°) = 0.8632.
    // (The plan's `0.816 ± 0.005` is an arithmetic slip; the bisection of the contract lands on the exact value.)
    const expectedH = 1 / (Math.cos(Math.PI / 18) + Math.sin(Math.PI / 18));
    near(result.rect.h, expectedH, 1e-5);
    near(result.rect.w, (result.rect.h * 1707) / 2560, 1e-9);
    near(result.rect.x + result.rect.w / 2, 0.5, 1e-9);
    near(result.rect.y + result.rect.h / 2, 0.5, 1e-9);
    valid(result);
    // Anything larger would leave an empty corner.
    const larger = { ...result, rect: { x: result.rect.x - 0.001, y: result.rect.y - 0.0015, w: result.rect.w + 0.002, h: result.rect.h + 0.003 } };
    expect(validate(larger, FRAME)).toBe('rect_outside');
  });

  it('(9) straighten 10 on the half-size centred square leaves it unchanged (already inside)', () => {
    const half: Rect = { x: 0.3333, y: 0.25, w: 0.3334, h: 0.5 };
    const result = refit(values(half), { kind: 'straighten', degrees: 10 }, FRAME);
    expect(result.rect).toEqual(half);
    expect(result.straighten).toBe(10);
  });

  it('straighten is clamped to ±45 and rounded to 0.1°', () => {
    expect(refit(values(DEFAULT), { kind: 'straighten', degrees: 60 }, FRAME).straighten).toBe(45);
    expect(refit(values(DEFAULT), { kind: 'straighten', degrees: -3.26 }, FRAME).straighten).toBe(-3.3);
  });

  it('(10) a locked se-corner trim by (−100, −100) px keeps the nw corner and follows the diagonal: h = 0.84664, w = h·1707/2560', () => {
    const start = values(DEFAULT);
    const turned = turnedFrame(FRAME, 0);
    const result = trim(DEFAULT, 'se', { x: -100, y: -100 }, { turned, scale: SCALE, ratio: shapeRatio(start.kind, start.shape, start.orientation, turned) });
    const expectedH = (1707 - 100 / SCALE) / 1707;
    near(result.h, expectedH, 1e-5);
    near(result.w, (expectedH * 1707) / 2560, 1e-5);
    near(result.x, DEFAULT.x, 1e-9);
    expect(result.y).toBe(0);
    valid({ ...start, rect: result });
  });

  it('(11) a free e-edge trim by −100 px moves that edge only: w = 0.56454, h = 1, x unchanged', () => {
    const free = values({ x: 0.16660156250000002, y: 0, w: 0.666796875, h: 1 }, { shape: 'free' });
    const result = trim(free.rect, 'e', { x: -100, y: 0 }, { turned: FRAME, scale: SCALE, ratio: null });
    near(result.w, (1707 - 100 / SCALE) / 2560, 1e-5);
    expect(result.h).toBe(1);
    expect(result.x).toBe(free.rect.x);
    valid({ ...free, rect: result });
  });
});
