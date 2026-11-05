import type { Rect } from '@/services/compositions';

export const SNAP_PX = 6;
export type SnapLine = 'centre' | 'start' | 'end';
export type SnapGuides = { x: SnapLine | null; y: SnapLine | null };

/** Nearest centre or image-edge alignment on each axis, measured in frame fractions. */
export function snap(rect: Rect, threshold: { x: number; y: number }): { rect: Rect; guides: SnapGuides } {
  const axis = (start: number, size: number, limit: number) => {
    const offsets = [
      { distance: Math.abs(start + size / 2 - 0.5), delta: 0.5 - size / 2 - start, guide: 'centre' as const },
      { distance: Math.abs(start), delta: -start, guide: 'start' as const },
      { distance: Math.abs(start + size - 1), delta: 1 - size - start, guide: 'end' as const },
    ];
    const closest = offsets.reduce((best, item) => item.distance < best.distance ? item : best);
    return closest.distance > 0 && closest.distance <= limit ? { value: start + closest.delta, guide: closest.guide } : { value: start, guide: null };
  };
  const x = axis(rect.x, rect.w, threshold.x);
  const y = axis(rect.y, rect.h, threshold.y);
  return { rect: { ...rect, x: x.value, y: y.value }, guides: { x: x.guide, y: y.guide } };
}
