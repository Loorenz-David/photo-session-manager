import { describe, expect, it } from 'vitest';
import { DEFAULT_VALUES, FRAME } from './harness';
import { composedLayout, leavesFrameOut } from '../composed';

describe('C3(c) crop mark pixel rule', () => {
  it('marks only a visible crop or any straightening', () => {
    const withRect = (x: number, y: number, w: number, h: number, straighten = 0) => ({ ...DEFAULT_VALUES, straighten, rect: { x, y, w, h } });
    expect(leavesFrameOut(withRect(0, 0, 1, 1), FRAME)).toBe(false);
    expect(leavesFrameOut(DEFAULT_VALUES, FRAME)).toBe(true);
    expect(leavesFrameOut(withRect(0, 0, 0.99999, 1), FRAME)).toBe(false);
    expect(leavesFrameOut(withRect(0, 0, 0.9998, 1), FRAME)).toBe(true);
    expect(leavesFrameOut({ ...withRect(0, 0, 1, 1), quarter_turns: 1 }, FRAME)).toBe(false);
    expect(leavesFrameOut(withRect(0.05, 0.05, 0.9, 0.9, 3), FRAME)).toBe(true);
    expect(leavesFrameOut({ ...DEFAULT_VALUES, kind: 'split' }, FRAME)).toBe(true);
  });
});

it('C2(a–c) computes the output box and transformed frame from the same fractions', () => {
  const normal = composedLayout(DEFAULT_VALUES, FRAME);
  expect(normal.ratio).toBe('1707 / 1707');
  expect(normal.frame).toEqual({ width: '149.9707%', height: '100%', left: '-24.9854%', top: '0%' });
  const turned = composedLayout({ ...DEFAULT_VALUES, quarter_turns: 1, flip: 'horizontal', straighten: 5, rect: { x: 0, y: DEFAULT_VALUES.rect.x, w: 1, h: DEFAULT_VALUES.rect.w } }, FRAME);
  expect(turned.ratio).toBe('1707 / 1707');
  expect(turned.frame).toEqual({ width: '100%', height: '149.9707%', left: '0%', top: '-24.9854%' });
  expect(turned.rotate).toBe('rotate(5deg)');
  expect(turned.imageTransform).toBe('rotate(90deg) scaleX(-1)');
  expect(turned.swapped).toBe(true);
});
