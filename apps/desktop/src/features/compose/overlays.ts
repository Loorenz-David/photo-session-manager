import type { Box } from './geometry';
import type { OverlayKind } from './useComposePrefs';

export type OverlayLine = { x1: number; y1: number; x2: number; y2: number };

export function overlayLines(kind: OverlayKind, box: Box, split: boolean): OverlayLine[] {
  if (kind === 'off') return [];
  const halves = split ? [
    { ...box, width: box.width / 2 },
    { ...box, left: box.left + box.width / 2, width: box.width / 2 },
  ] : [box];
  return halves.flatMap(part => {
    const { left: x, top: y, width: w, height: h } = part;
    if (kind === 'diagonals') {
      const d = Math.min(w, h);
      const candidates = [
        { x1: x, y1: y, x2: x + d, y2: y + d },
        { x1: x + w, y1: y, x2: x + w - d, y2: y + d },
        { x1: x, y1: y + h, x2: x + d, y2: y + h - d },
        { x1: x + w, y1: y + h, x2: x + w - d, y2: y + h - d },
      ];
      return candidates.filter((line, index) => !candidates.slice(0, index).some(other =>
        (line.x1 === other.x1 && line.y1 === other.y1 && line.x2 === other.x2 && line.y2 === other.y2) ||
        (line.x1 === other.x2 && line.y1 === other.y2 && line.x2 === other.x1 && line.y2 === other.y1)));
    }
    const phi = (1 + Math.sqrt(5)) / 2;
    const positions = kind === 'grid' ? [1/6, 2/6, 3/6, 4/6, 5/6] : kind === 'golden' ? [1/(phi*phi), 1/phi] : [1/3, 2/3];
    return [
      ...positions.map(f => ({ x1: x + w*f, y1: y, x2: x + w*f, y2: y + h })),
      ...positions.map(f => ({ x1: x, y1: y + h*f, x2: x + w, y2: y + h*f })),
    ];
  });
}
