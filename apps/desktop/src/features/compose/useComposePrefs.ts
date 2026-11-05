import { useCallback, useState } from 'react';

export const OVERLAY_ORDER = ['off', 'thirds', 'grid', 'golden', 'diagonals'] as const;
export type OverlayKind = typeof OVERLAY_ORDER[number];
export const STRAIGHTEN_GRID_PX = 24;
export const STRAIGHTEN_STEP = 0.1;
export const STRAIGHTEN_STEP_SHIFT = 1;

function read(key: string): string | null {
  try { return window.localStorage.getItem(key); } catch { return null; }
}
function write(key: string, value: string): void {
  try { window.localStorage.setItem(key, value); } catch { /* Preferences remain usable for this mount. */ }
}

export function useComposePrefs() {
  const [overlay, changeOverlay] = useState<OverlayKind>(() => {
    const stored = read('compose.overlay');
    return OVERLAY_ORDER.find(value => value === stored) ?? 'thirds';
  });
  const [straightenGrid, changeGrid] = useState(() => read('compose.straightenGrid') !== '0');
  const [snap, changeSnap] = useState(() => read('compose.snap') !== '0');
  const setOverlay = useCallback((value: OverlayKind) => { changeOverlay(value); write('compose.overlay', value); }, []);
  const cycleOverlay = useCallback(() => changeOverlay(current => {
    const next = OVERLAY_ORDER[(OVERLAY_ORDER.indexOf(current) + 1) % OVERLAY_ORDER.length];
    write('compose.overlay', next);
    return next;
  }), []);
  const setStraightenGrid = useCallback((value: boolean) => { changeGrid(value); write('compose.straightenGrid', value ? '1' : '0'); }, []);
  const setSnap = useCallback((value: boolean) => { changeSnap(value); write('compose.snap', value ? '1' : '0'); }, []);
  return { overlay, setOverlay, cycleOverlay, straightenGrid, setStraightenGrid, snap, setSnap };
}
