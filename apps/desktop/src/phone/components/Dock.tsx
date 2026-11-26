import { type PointerEvent as ReactPointerEvent, useEffect, useRef, useState } from 'react';
import { exampleUrl, type PhoneSession, previewUrl } from '../api';
import { buzz, DOCK, HAPTIC, TILE_PX, tileScale } from '../gestures';
import { type DockTile, dockTiles, tileImage } from '../state';

type Pointer = ReactPointerEvent<HTMLDivElement>;
type Props = {
  token: string;
  view: PhoneSession;
  /** Long shots turn the viewport; the thumbnails turn with it. */
  rotated: boolean;
  /** The dock takes touches (no preview or drawer is open). */
  enabled: boolean;
  /** Tap, or hold-and-drag-up then release: the tile is chosen. */
  onSelect: (tile: DockTile) => void;
};

/** The magnifier: the finger in content coordinates, how far it was dragged up, the tile under it. */
type Magnify = { cx: number; lift: number; key: string | null };

/**
 * Design §3 and §Shot dock gestures: a 62 px strip of thumbnails. Tap selects, a quick sideways
 * swipe scrolls, a 200 ms hold magnifies (Mac-Dock style) and a magnified tile dragged up past
 * 50 px opens on release. The strip never moves; magnified tiles grow up over the stage.
 */
export function Dock({ token, view, rotated, enabled, onSelect }: Props) {
  const tiles = dockTiles(view);
  const scroller = useRef<HTMLDivElement>(null);
  /** Tile centres in content coordinates, snapshotted at pointer down (magnifying never feeds back into hit-testing). */
  const centres = useRef<{ key: string; c: number }[]>([]);
  const touch = useRef<{ x: number; y: number; scrollLeft: number; mode: 'magnify' | 'scroll' | null } | null>(null);
  const timer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);
  const [magnify, setMagnify] = useState<Magnify | null>(null);
  useEffect(() => () => clearTimeout(timer.current), []);

  const toContent = (clientX: number) => {
    const element = scroller.current!;
    return clientX - element.getBoundingClientRect().left + element.scrollLeft;
  };
  /** The tile nearest the finger: any distance while magnifying, within 40 px for a tap. */
  const nearest = (cx: number, anyDistance: boolean) => {
    let best: string | null = null;
    let distance = Infinity;
    for (const tile of centres.current) {
      const d = Math.abs(tile.c - cx);
      if (d < distance) {
        distance = d;
        best = tile.key;
      }
    }
    return anyDistance || distance < DOCK.nearPx ? best : null;
  };
  const select = (key: string | null) => {
    const tile = tiles.find(candidate => candidate.key === key);
    if (tile) onSelect(tile);
  };

  const down = (event: Pointer) => {
    if (!enabled || !scroller.current) return;
    event.currentTarget.setPointerCapture(event.pointerId);
    const element = scroller.current;
    const left = element.getBoundingClientRect().left;
    centres.current = Array.from(element.querySelectorAll<HTMLElement>('[data-dock-k]')).map(tile => {
      const rect = tile.getBoundingClientRect();
      return { key: tile.dataset.dockK!, c: rect.left + rect.width / 2 - left + element.scrollLeft };
    });
    const start = { x: event.clientX, y: event.clientY, scrollLeft: element.scrollLeft, mode: null as 'magnify' | 'scroll' | null };
    touch.current = start;
    clearTimeout(timer.current);
    timer.current = setTimeout(() => {
      if (touch.current !== start || start.mode) return;
      start.mode = 'magnify';
      buzz(HAPTIC.magnify);
      const cx = toContent(start.x);
      setMagnify({ cx, lift: 0, key: nearest(cx, true) });
    }, DOCK.holdMs);
  };

  const move = (event: Pointer) => {
    const now = touch.current;
    const element = scroller.current;
    if (!now || !element) return;
    if (!now.mode) {
      if (Math.abs(event.clientX - now.x) <= DOCK.scrollPx) return;
      clearTimeout(timer.current);
      now.mode = 'scroll';
    }
    if (now.mode === 'scroll') {
      element.scrollLeft = now.scrollLeft - (event.clientX - now.x);
      return;
    }
    const rect = element.getBoundingClientRect();
    if (event.clientX < rect.left + DOCK.edgePx) element.scrollLeft -= DOCK.edgeStepPx;
    else if (event.clientX > rect.right - DOCK.edgePx) element.scrollLeft += DOCK.edgeStepPx;
    const cx = toContent(event.clientX);
    const key = nearest(cx, true);
    const lift = Math.min(0, event.clientY - now.y);
    if (magnify && key !== magnify.key) buzz(HAPTIC.hoverTick);
    if (magnify && lift < -DOCK.openPx && magnify.lift >= -DOCK.openPx) buzz(HAPTIC.dragOpen);
    setMagnify({ cx, lift, key });
  };

  const up = (event: Pointer) => {
    const now = touch.current;
    touch.current = null;
    clearTimeout(timer.current);
    if (!now) return;
    if (now.mode === 'magnify') {
      const was = magnify;
      setMagnify(null);
      // Released above the line opens the tile; brought back down first, nothing happens.
      if (was && was.lift < -DOCK.openPx) select(was.key);
      return;
    }
    if (now.mode === 'scroll') return;
    select(nearest(toContent(event.clientX), false));
  };

  const cancel = () => {
    touch.current = null;
    clearTimeout(timer.current);
    setMagnify(null);
  };

  const centreOf = (key: string) => centres.current.find(tile => tile.key === key)?.c;
  const firstCondition = tiles.findIndex(tile => tile.kind === 'condition');
  return <div className="ph-dock-strip" data-testid="dock" onPointerDown={down} onPointerMove={move} onPointerUp={up} onPointerCancel={cancel}
    onContextMenu={event => event.preventDefault()}>
    <div className="ph-dock-scroller" ref={scroller}>
      <div className="ph-dock-row">
        {tiles.map((tile, index) => {
          const centre = centreOf(tile.key);
          const scale = magnify && centre !== undefined ? tileScale(magnify.cx, centre) : 1;
          const hovered = magnify?.key === tile.key;
          const armed = hovered && magnify.lift < -DOCK.openPx;
          const size = Math.round(TILE_PX * scale);
          const image = tileImage(tile);
          const lift = hovered ? Math.max(magnify.lift, -DOCK.liftMaxPx) : 0;
          return [
            index === firstCondition && <div key="divider" className="ph-dock-divider" data-testid="dock-divider" />,
            <div key={tile.key} data-dock-k={tile.key} aria-label={tile.kind === 'free' ? 'Free shot' : tile.label} aria-current={tile.current || undefined}
              className={`ph-dock-tile${tile.kind === 'free' ? ' ph-dock-free' : ''}${magnify ? ' ph-dock-magnifying' : ''}`}
              style={{
                width: size, height: size, transform: `translateY(${lift}px) rotate(${rotated ? 90 : 0}deg)`,
                outline: `2px solid ${tile.current ? '#111' : 'transparent'}`, outlineOffset: '2px',
              }}>
              {image && <img className="ph-dock-image" alt="" draggable={false} src={previewUrl(token, image)} />}
              {!image && tile.kind === 'shot' && tile.shot.example
                && <img className="ph-dock-image ph-grey" alt="" draggable={false} src={exampleUrl(token, tile.shot.container_id)} />}
              {tile.kind === 'free' && <svg className="ph-dock-plus" width="50%" height="50%" viewBox="0 0 24 24" fill="none" stroke="#111" strokeWidth="2" strokeLinecap="round"><path d="M12 5v14M5 12h14" /></svg>}
              {tile.kind === 'shot' && tile.shot.split && <div className="ph-dock-split" data-testid="dock-split" style={{ left: '50%' }} />}
              {tile.tag && <span className={`ph-tag${tile.kind === 'shot' ? '' : ' ph-tag-inverted'}`}>{tile.tag}</span>}
              {tile.kind === 'shot' && image && <div className="ph-dock-check" aria-label="Captured">
                <svg width="8" height="8" viewBox="0 0 24 24" fill="none" stroke="#fff" strokeWidth="4" strokeLinecap="round" strokeLinejoin="round"><path d="M5 12l4 4 10-10" /></svg>
              </div>}
              {hovered && <div className={`ph-dock-label${armed ? ' ph-dock-label-armed' : ''}`} data-testid="dock-label">{armed ? 'Release to open' : tile.label}</div>}
            </div>,
          ];
        })}
      </div>
    </div>
  </div>;
}
