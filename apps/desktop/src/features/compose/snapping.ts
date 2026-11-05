import type { Rect } from '@/services/compositions';

export const SNAP_PX = 6;
export type SnapGuides = { x: boolean; y: boolean };

/** Nearest centre or image-edge alignment on each axis, measured in frame fractions. */
export function snap(rect: Rect, threshold: { x: number; y: number }): { rect: Rect; guides: SnapGuides } {
  const axis = (start: number, size: number, limit: number) => {
    const offsets = [
      { distance: Math.abs(start + size / 2 - 0.5), delta: 0.5 - size / 2 - start },
      { distance: Math.abs(start), delta: -start },
      { distance: Math.abs(start + size - 1), delta: 1 - size - start },
    ];
    const closest = offsets.reduce((best, item) => item.distance < best.distance ? item : best);
    return closest.distance > 0 && closest.distance <= limit ? { value: start + closest.delta, guide: true } : { value: start, guide: false };
  };
  const x = axis(rect.x, rect.w, threshold.x);
  const y = axis(rect.y, rect.h, threshold.y);
  return { rect: { ...rect, x: x.value, y: y.value }, guides: { x: x.guide, y: y.guide } };
}
