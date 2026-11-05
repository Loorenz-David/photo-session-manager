import { describe, expect, it } from 'vitest';
import { snap } from '../snapping';

describe('C1(a) composition snapping', () => {
  it('selects the nearest centre or edge within six screen pixels', () => {
    const threshold = { x: 6 / 977.81, y: 6 / 977.81 };
    const centred = snap({ x: 0.1704, y: 0, w: 0.6668, h: 1 }, threshold);
    expect(centred.rect.x).toBeCloseTo(0.1666, 8);
    expect(centred.guides).toEqual({ x: 'centre', y: null });
    expect(snap({ x: 0.004, y: 0, w: 0.6668, h: 1 }, threshold)).toEqual({ rect: { x: 0, y: 0, w: 0.6668, h: 1 }, guides: { x: 'start', y: null } });
    expect(snap({ x: 0.3292, y: 0, w: 0.6668, h: 1 }, threshold)).toEqual({ rect: { x: 0.33320000000000005, y: 0, w: 0.6668, h: 1 }, guides: { x: 'end', y: null } });
    expect(snap({ x: 0.1, y: 0, w: 0.6668, h: 1 }, threshold)).toEqual({ rect: { x: 0.1, y: 0, w: 0.6668, h: 1 }, guides: { x: null, y: null } });
  });
});
