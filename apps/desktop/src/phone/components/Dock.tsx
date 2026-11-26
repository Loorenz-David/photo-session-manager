import { type PointerEvent as ReactPointerEvent, useRef } from 'react';
import { exampleUrl, type PhoneSession } from '../api';
import { DOCK } from '../gestures';
import { type DockTile, dockTiles, tileImage } from '../state';
import { PhotoImage } from './PhotoImage';

type Pointer = ReactPointerEvent<HTMLDivElement>;
type Props = {
  token: string;
  view: PhoneSession;
  /** Long shots turn the viewport; the thumbnails turn with it. */
  rotated: boolean;
  /** The dock takes touches (no preview or drawer is open). */
  enabled: boolean;
  /** A tap chose the tile. */
  onSelect: (tile: DockTile) => void;
};

/**
 * The shot dock (design §3 as amended by book 10J LV-D19 (1)): an 84 px strip of 56 px thumbnails.
 * A touch that stays within 8 px is a tap, however long it is held, and selects the nearest tile;
 * one that moves sideways scrolls the strip; one that moves up or down selects nothing. There is
 * no hold gesture. The strip never moves and nothing grows out of it.
 */
export function Dock({ token, view, rotated, enabled, onSelect }: Props) {
  const tiles = dockTiles(view);
  const scroller = useRef<HTMLDivElement>(null);
  const touch = useRef<{ x: number; y: number; scrollLeft: number; scrolling: boolean; moved: boolean } | null>(null);

  /** The tile whose centre is nearest the finger, if within half a pitch. */
  const nearest = (clientX: number): DockTile | undefined => {
    let best: string | undefined;
    let distance = Infinity;
    for (const element of Array.from(scroller.current?.querySelectorAll<HTMLElement>('[data-dock-k]') ?? [])) {
      const rect = element.getBoundingClientRect();
      const d = Math.abs(rect.left + rect.width / 2 - clientX);
      if (d < distance) {
        distance = d;
        best = element.dataset.dockK;
      }
    }
    return distance <= DOCK.nearPx ? tiles.find(tile => tile.key === best) : undefined;
  };

  const down = (event: Pointer) => {
    if (!enabled || !scroller.current) return;
    event.currentTarget.setPointerCapture(event.pointerId);
    touch.current = { x: event.clientX, y: event.clientY, scrollLeft: scroller.current.scrollLeft, scrolling: false, moved: false };
  };

  /** Notes how far the touch has gone; sideways past the tap distance it scrolls from then on. */
  const track = (event: Pointer) => {
    const now = touch.current;
    const element = scroller.current;
    if (!now || !element) return;
    const dx = event.clientX - now.x;
    if (Math.abs(dx) > DOCK.tapPx) now.scrolling = true;
    if (now.scrolling || Math.abs(event.clientY - now.y) > DOCK.tapPx) now.moved = true;
    if (now.scrolling) element.scrollLeft = now.scrollLeft - dx;
  };

  const up = (event: Pointer) => {
    track(event);
    const now = touch.current;
    touch.current = null;
    if (!now || now.moved) return;
    const tile = nearest(event.clientX);
    if (tile) onSelect(tile);
  };

  const cancel = () => {
    touch.current = null;
  };

  const firstCondition = tiles.findIndex(tile => tile.kind === 'condition');
  return <div className="ph-dock-strip" data-testid="dock" onPointerDown={down} onPointerMove={track} onPointerUp={up} onPointerCancel={cancel}
    onContextMenu={event => event.preventDefault()}>
    <div className="ph-dock-scroller" ref={scroller}>
      <div className="ph-dock-row">
        {tiles.map((tile, index) => {
          const image = tileImage(tile);
          return [
            index === firstCondition && <div key="divider" className="ph-dock-divider" data-testid="dock-divider" />,
            <div key={tile.key} data-dock-k={tile.key} aria-label={tile.kind === 'free' ? 'Free shot' : tile.label} aria-current={tile.current || undefined}
              className={`ph-dock-tile${tile.kind === 'free' ? ' ph-dock-free' : ''}`}
              style={{ transform: `rotate(${rotated ? 90 : 0}deg)`, outline: `2px solid ${tile.current ? '#111' : 'transparent'}`, outlineOffset: '2px' }}>
              {image && <PhotoImage token={token} image={image} className="ph-dock-image" />}
              {!image && tile.kind === 'shot' && tile.shot.example
                && <img className="ph-dock-image ph-grey" alt="" draggable={false} src={exampleUrl(token, tile.shot.container_id)} />}
              {tile.kind === 'free' && <svg className="ph-dock-plus" width="50%" height="50%" viewBox="0 0 24 24" fill="none" stroke="#111" strokeWidth="2" strokeLinecap="round"><path d="M12 5v14M5 12h14" /></svg>}
              {tile.kind === 'shot' && tile.shot.split && <div className="ph-dock-split" data-testid="dock-split" style={{ left: '50%' }} />}
              {tile.tag && <span className={`ph-tag${tile.kind === 'shot' ? '' : ' ph-tag-inverted'}`}>{tile.tag}</span>}
              {tile.kind === 'shot' && image && <div className="ph-dock-check" aria-label="Captured">
                <svg width="9" height="9" viewBox="0 0 24 24" fill="none" stroke="#fff" strokeWidth="4" strokeLinecap="round" strokeLinejoin="round"><path d="M5 12l4 4 10-10" /></svg>
              </div>}
            </div>,
          ];
        })}
      </div>
    </div>
  </div>;
}
