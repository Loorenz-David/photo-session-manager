import { describe, expect, it } from 'vitest';
import { overlayLines, type OverlayLine } from '../overlays';

function linesEqual(actual: OverlayLine[], expected: OverlayLine[]) {
  expect(actual).toHaveLength(expected.length);
  const left = actual.map(line => Object.values(line).map(n => n.toFixed(2)).join(',')).sort();
  const right = expected.map(line => Object.values(line).map(n => n.toFixed(2)).join(',')).sort();
  expect(left).toEqual(right);
}
const verticals = (xs: number[], top: number, bottom: number) => xs.map(x => ({ x1: x, y1: top, x2: x, y2: bottom }));
const horizontals = (ys: number[], left: number, right: number) => ys.map(y => ({ x1: left, y1: y, x2: right, y2: y }));

describe('C2(a) overlayLines', () => {
  it('draws the contracted guides in the whole frame or each split half', () => {
    const square = { left: 174, top: 24, width: 652, height: 652 };
    linesEqual(overlayLines('thirds', square, false), [
      ...verticals([391.3333, 608.6667], 24, 676), ...horizontals([241.3333, 458.6667], 174, 826),
    ]);
    linesEqual(overlayLines('grid', square, false), [
      ...verticals([282.6667, 391.3333, 500, 608.6667, 717.3333], 24, 676),
      ...horizontals([132.6667, 241.3333, 350, 458.6667, 567.3333], 174, 826),
    ]);
    linesEqual(overlayLines('golden', square, false), [
      ...verticals([423.0418, 576.9582], 24, 676), ...horizontals([273.0418, 426.9582], 174, 826),
    ]);
    linesEqual(overlayLines('diagonals', square, false), [
      { x1: 174, y1: 24, x2: 826, y2: 676 }, { x1: 826, y1: 24, x2: 174, y2: 676 },
    ]);
    linesEqual(overlayLines('diagonals', { left: 24, top: 32.6047, width: 952, height: 634.7906 }, false), [
      { x1: 24, y1: 32.6047, x2: 658.7906, y2: 667.3953 },
      { x1: 976, y1: 32.6047, x2: 341.2094, y2: 667.3953 },
      { x1: 24, y1: 667.3953, x2: 658.7906, y2: 32.6047 },
      { x1: 976, y1: 667.3953, x2: 341.2094, y2: 32.6047 },
    ]);
    linesEqual(overlayLines('thirds', { left: 24, top: 112, width: 952, height: 476 }, true), [
      ...verticals([182.6667, 341.3333, 658.6667, 817.3333], 112, 588),
      ...horizontals([270.6667, 429.3333], 24, 500), ...horizontals([270.6667, 429.3333], 500, 976),
    ]);
    expect(overlayLines('off', square, false)).toEqual([]);
  });
});
