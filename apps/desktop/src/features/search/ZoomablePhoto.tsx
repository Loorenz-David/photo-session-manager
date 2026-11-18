import { useCallback, useEffect, useRef, useState, type PointerEvent as ReactPointerEvent, type ReactNode } from 'react';
import { motion } from 'motion/react';
import { ScanSearch, ZoomIn } from 'lucide-react';

/** The pinned photo's zoom limits and the zoom a click goes to. */
export const PHOTO_MIN_ZOOM = 1;
export const PHOTO_MAX_ZOOM = 6;
export const PHOTO_CLICK_ZOOM = 2.5;
/** How long a click waits to see whether it is the first of a double-click (ms). */
const CLICK_WAIT_MS = 220;

export type View = { scale: number; x: number; y: number };
export const FIT: View = { scale: 1, x: 0, y: 0 };

function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value));
}

/**
 * A18: the view after zooming to `scale` around the point `at` (from the box's centre, in pixels): the point under
 * the pointer stays where it is. At 1× the photo sits fitted and centred; the photo never pans further than its edges.
 */
export function zoomAround(view: View, scale: number, at: { x: number; y: number }, box: { width: number; height: number }): View {
  const next = clamp(scale, PHOTO_MIN_ZOOM, PHOTO_MAX_ZOOM);
  if (next <= PHOTO_MIN_ZOOM + 0.001) return FIT;
  const ratio = next / view.scale;
  return keepInside({ scale: next, x: at.x - (at.x - view.x) * ratio, y: at.y - (at.y - view.y) * ratio }, box);
}

/** A18: a pan kept within the photo's edges (half the overflow each way). */
export function keepInside(view: View, box: { width: number; height: number }): View {
  const maxX = (box.width * (view.scale - 1)) / 2;
  const maxY = (box.height * (view.scale - 1)) / 2;
  return { scale: view.scale, x: clamp(view.x, -maxX, maxX), y: clamp(view.y, -maxY, maxY) };
}

/**
 * A18 (owner 2026-11-18): the pinned photo, zoomable. Scroll or pinch zooms around the pointer, a click zooms in on
 * the spot, drag pans, double-click fits it again. On hover it offers "Search this photo" (a new search tab), so
 * pinning another photo never searches by itself.
 */
export function ZoomablePhoto({ photoKey, children, onSearch }: { photoKey: string | number; children: ReactNode; onSearch: () => void }) {
  const box = useRef<HTMLDivElement>(null);
  const [view, setView] = useState<View>(FIT);
  const [dragging, setDragging] = useState(false);
  const press = useRef<{ x: number; y: number; view: View; moved: boolean } | null>(null);
  const clickTimer = useRef<number | undefined>(undefined);

  // Another photo starts fitted.
  useEffect(() => { setView(FIT); }, [photoKey]);
  useEffect(() => () => window.clearTimeout(clickTimer.current), []);

  const size = () => {
    const rect = box.current?.getBoundingClientRect();
    return { rect, box: { width: rect?.width || 1, height: rect?.height || 1 } };
  };
  const pointAt = useCallback((clientX: number, clientY: number) => {
    const rect = box.current?.getBoundingClientRect();
    if (!rect) return { x: 0, y: 0 };
    return { x: clientX - rect.left - rect.width / 2, y: clientY - rect.top - rect.height / 2 };
  }, []);

  // Wheel and trackpad pinch (a wheel with ctrlKey): not passive, so the window never scrolls instead.
  useEffect(() => {
    const element = box.current;
    if (!element) return;
    const onWheel = (event: WheelEvent) => {
      event.preventDefault();
      const factor = Math.exp(-event.deltaY * (event.ctrlKey ? 0.01 : 0.0025));
      const at = pointAt(event.clientX, event.clientY);
      const rect = element.getBoundingClientRect();
      setView(current => zoomAround(current, current.scale * factor, at, { width: rect.width || 1, height: rect.height || 1 }));
    };
    element.addEventListener('wheel', onWheel, { passive: false });
    return () => element.removeEventListener('wheel', onWheel);
  }, [pointAt]);

  function onPointerDown(event: ReactPointerEvent<HTMLDivElement>) {
    if (event.button !== 0 || (event.target as HTMLElement).closest('button')) return;
    press.current = { x: event.clientX, y: event.clientY, view, moved: false };
    event.currentTarget.setPointerCapture?.(event.pointerId);
  }
  function onPointerMove(event: ReactPointerEvent<HTMLDivElement>) {
    const start = press.current;
    if (!start) return;
    const dx = event.clientX - start.x;
    const dy = event.clientY - start.y;
    if (!start.moved && Math.hypot(dx, dy) < 4) return;
    start.moved = true;
    if (start.view.scale <= 1) return;
    setDragging(true);
    setView(keepInside({ scale: start.view.scale, x: start.view.x + dx, y: start.view.y + dy }, size().box));
  }
  function onPointerUp(event: ReactPointerEvent<HTMLDivElement>) {
    const start = press.current;
    press.current = null;
    setDragging(false);
    if (!start || start.moved) return;
    // A click: zoom in on the spot, unless it turns out to be a double-click.
    if (view.scale > 1) return;
    const at = pointAt(event.clientX, event.clientY);
    window.clearTimeout(clickTimer.current);
    clickTimer.current = window.setTimeout(() => {
      setView(current => (current.scale > 1 ? current : zoomAround(current, PHOTO_CLICK_ZOOM, at, size().box)));
    }, CLICK_WAIT_MS);
  }
  function onDoubleClick() {
    window.clearTimeout(clickTimer.current);
    setView(FIT);
  }

  const zoomed = view.scale > 1.001;
  return (
    <div
      ref={box}
      className="group relative flex size-full items-center justify-center overflow-hidden select-none"
      style={{ cursor: zoomed ? (dragging ? 'grabbing' : 'grab') : 'zoom-in', touchAction: 'none' }}
      onPointerDown={onPointerDown}
      onPointerMove={onPointerMove}
      onPointerUp={onPointerUp}
      onPointerCancel={() => { press.current = null; setDragging(false); }}
      onDoubleClick={onDoubleClick}
      data-testid="zoomable-photo"
      data-zoom={view.scale.toFixed(2)}
    >
      <motion.div
        className="flex size-full items-center justify-center"
        animate={{ scale: view.scale, x: view.x, y: view.y }}
        transition={dragging ? { duration: 0 } : { type: 'spring', stiffness: 420, damping: 42, mass: 0.6 }}
      >
        {children}
      </motion.div>
      {zoomed ? (
        <button
          type="button"
          className="absolute top-2 right-2 flex items-center gap-1 rounded-full bg-background/90 px-2 py-0.5 text-[11px] font-semibold shadow-(--shadow-sm) backdrop-blur hover:bg-background"
          title="Fit the photo (double-click)"
          aria-label="Fit the photo"
          onClick={() => setView(FIT)}
        >
          <ZoomIn className="size-3" aria-hidden /> {Math.round(view.scale * 100)}% · Fit
        </button>
      ) : null}
      <button
        type="button"
        onClick={onSearch}
        title="Search Google for this photo in a new tab"
        className="absolute bottom-3 left-1/2 flex -translate-x-1/2 items-center gap-1.5 rounded-full bg-background/95 px-3 py-1.5 text-xs font-semibold opacity-0 shadow-(--shadow-md) backdrop-blur transition-opacity group-hover:opacity-100 focus-visible:opacity-100"
      >
        <ScanSearch className="size-3.5" aria-hidden /> Search this photo
      </button>
    </div>
  );
}
