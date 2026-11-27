import { type PointerEvent as ReactPointerEvent, useCallback, useEffect, useLayoutEffect, useRef, useState } from 'react';
import { exampleUrl, type PhoneSession } from '../api';
import { arrangementKey, type Drop, dropTarget, type Home } from '../arrange';
import { buzz, DOCK, DRAG, HAPTIC } from '../gestures';
import { type DockTile, dockTiles, isWaiting, tileImage } from '../state';
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
  /** A lifted photo was released on a target (PH9.3): the app asks the Mac. */
  onDrop: (photoId: number, drop: Drop) => void;
  /** The Mac is still answering a drop (PH9.5): nothing lifts meanwhile; a long press is a tap. */
  pending: boolean;
  /** `dock` on the live screen, `preview-strip` in the Preview (LV-D20 (2): the same strip in both). */
  testId?: string;
  /** The Preview's strip outlines the photo shown (plan point P11) instead of the current shot. */
  marked?: number;
};

/** The marked tile is kept this far inside the strip's visible edges. */
const REVEAL_PX = 16;
/** The pitch of two tiles when a group has only one to measure it from (56 px tiles, 8 px apart). */
const PITCH_PX = 64;
/** How long the strip's "put back" and "set down" looks last (the cancel's spring-back ≤ 200 ms). */
const BACK_MS = 200;
const SET_MS = 60;

/** What a lifted photo carries from the moment it is lifted (PH9.2, PH9.3). */
type Lift = {
  source: Home;
  photoId: number;
  /** Every tile's home, measured once, at lift. */
  homes: Home[];
  /** Which photo was where at lift (PH9.4): the drag does not outlive a change of it. */
  arrangement: string;
  /** The strip's rectangle at lift: the band and the auto-scroll edges are counted from it. */
  strip: { left: number; right: number; top: number; bottom: number };
  /** Where the finger took the tile: x in content coordinates, y on screen. */
  grab: { x: number; y: number };
  /** The finger's last place on screen (an auto-scroll step moves the content under it). */
  finger: { x: number; y: number };
  target: Drop | null;
  /** The auto-scroll: its direction and its 16 ms timer while the finger is at an end of the strip. */
  scrolling: -1 | 0 | 1;
  scroll: ReturnType<typeof setInterval> | null;
};

/**
 * One finger on the strip (PH9.2): `pressing` until it is released (a tap), moves sideways
 * (`scrolling`), moves up or down (`moved`), or rests `DRAG.liftMs` on a photo (`lifted`, then
 * `dragging`). No state means idle.
 */
type Press = { phase: 'pressing' | 'scrolling' | 'moved'; pointerId: number; x: number; y: number; scrollLeft: number; timer: ReturnType<typeof setTimeout> | null };
type Carry = { phase: 'lifted' | 'dragging'; pointerId: number; lift: Lift };
type Gesture = Press | Carry;
const carries = (gesture: Gesture): gesture is Carry => 'lift' in gesture;

/** What the strip draws of a drag: the lifted tile's key and offset, and the target under the finger. */
type Drag = { key: string; dx: number; dy: number; target: Drop | null; away: boolean; pitch: number };

const clamp = (value: number, low: number, high: number) => Math.max(low, Math.min(high, value));
const same = (a: Drop | null, b: Drop | null) => JSON.stringify(a) === JSON.stringify(b);

/**
 * The photo strip (design §3 as amended by book 10J LV-D19 (1) and LV-D20 (3)): an 84 px strip of
 * 56 px thumbnails, on the live screen and in the Preview. A touch that stays within 8 px and is
 * released before 400 ms is a tap and selects the nearest tile; one that moves sideways scrolls
 * the strip; one that moves up or down selects nothing. A finger that rests 400 ms on a photo lifts
 * it: it follows the finger, the tiles of its group slide aside to show where it would go, and a
 * release on a target asks the Mac (swap between template shots, insert among the extras or the
 * Condition images, to the other group's end). Targets are computed from the tiles' homes measured
 * at lift, never from where they are drawn while they slide. No HTML5 drag and drop.
 */
export function Dock({ token, view, rotated, enabled, onSelect, onDrop, pending, testId = 'dock', marked }: Props) {
  const tiles = dockTiles(view);
  const arrangement = arrangementKey(view);
  // A timer started at the press reads the strip as it is when it fires.
  const shown = useRef({ tiles, arrangement, pending });
  shown.current = { tiles, arrangement, pending };
  /** The fingers on the strip: a gesture is one finger's, and only when it is alone (PH9.4). */
  const fingers = useRef(new Set<number>());
  const strip = useRef<HTMLDivElement>(null);
  const scroller = useRef<HTMLDivElement>(null);
  const gesture = useRef<Gesture | null>(null);
  const [drag, setDrag] = useState<Drag | null>(null);
  /** After a drag: `back` while the tiles spring home, `set` for the instant a drop is shown. */
  const [after, setAfter] = useState<'back' | 'set' | null>(null);
  const afterTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const outlined = (tile: DockTile) => (marked === undefined ? tile.current : tileImage(tile)?.photo_id === marked);

  // The photo shown is in sight: the strip scrolls just far enough when it changes.
  useLayoutEffect(() => {
    const element = scroller.current;
    const tile = element?.querySelector<HTMLElement>('[aria-current="true"]');
    if (marked === undefined || !element || !tile) return;
    const frame = element.getBoundingClientRect();
    const rect = tile.getBoundingClientRect();
    const over = rect.right + REVEAL_PX - frame.right;
    const under = frame.left + REVEAL_PX - rect.left;
    const wanted = element.scrollLeft + (over > 0 ? over : under > 0 ? -under : 0);
    element.scrollLeft = clamp(wanted, 0, Math.max(0, element.scrollWidth - element.clientWidth));
  }, [marked]);

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
    return distance <= DOCK.nearPx ? shown.current.tiles.find(tile => tile.key === best) : undefined;
  };

  /** The strip looks `look` for `ms`, then as always. */
  const lookFor = useCallback((look: 'back' | 'set', ms: number) => {
    if (afterTimer.current) clearTimeout(afterTimer.current);
    setAfter(look);
    afterTimer.current = setTimeout(() => setAfter(null), ms);
  }, []);

  /**
   * The gesture is over, whatever it was: its timers stop and the strip is idle. A drag that ends
   * `dropped` is shown where it landed at once; any other end lets the tiles spring home. Every
   * cancel of PH9.4 is this function with `false`: the pointer cancelled, a second finger, the page
   * hidden, the window blurred, the arrangement changed, the strip unmounted, a release on no target.
   * Whatever the finger does afterwards finds no gesture and asks nothing.
   */
  const finish = useCallback((dropped: boolean) => {
    const now = gesture.current;
    gesture.current = null;
    if (!now) return;
    if (!carries(now)) {
      if (now.timer) clearTimeout(now.timer);
      return;
    }
    if (now.lift.scroll) clearInterval(now.lift.scroll);
    setDrag(null);
    lookFor(dropped ? 'set' : 'back', dropped ? SET_MS : BACK_MS);
  }, [lookFor]);

  // PH9.4: the strip unmounting (its Preview closed) ends the gesture and every timer of it.
  useEffect(() => () => {
    finish(false);
    if (afterTimer.current) clearTimeout(afterTimer.current);
  }, [finish]);

  // PH9.4: leaving the app (the page hidden, the window without focus) cancels; so does a finger
  // whose release the strip never saw (it is forgotten wherever it ends).
  useEffect(() => {
    const away = () => {
      fingers.current.clear();
      finish(false);
    };
    const hidden = () => {
      if (document.visibilityState === 'hidden') away();
    };
    const gone = (event: PointerEvent) => {
      fingers.current.delete(event.pointerId);
    };
    window.addEventListener('blur', away);
    document.addEventListener('visibilitychange', hidden);
    window.addEventListener('pointerup', gone, true);
    window.addEventListener('pointercancel', gone, true);
    return () => {
      window.removeEventListener('blur', away);
      document.removeEventListener('visibilitychange', hidden);
      window.removeEventListener('pointerup', gone, true);
      window.removeEventListener('pointercancel', gone, true);
    };
  }, [finish]);

  // PH9.4: the Mac changed which photo is where while one was lifted: the drag is over (a new
  // picture, a revision or the current shot does not change the key). So is it when the strip
  // stops taking touches under the finger.
  useEffect(() => {
    const now = gesture.current;
    if (now && (!enabled || (carries(now) && now.lift.arrangement !== arrangement))) finish(false);
  }, [arrangement, enabled, finish]);

  /** PH9.3: the finger is at `(clientX, clientY)`: the target under it, and where the lifted tile is drawn. */
  const follow = (lift: Lift, clientX: number, clientY: number) => {
    const element = scroller.current;
    if (!element) return;
    lift.finger = { x: clientX, y: clientY };
    const x = clientX - lift.strip.left + element.scrollLeft;
    const inBand = clientY >= lift.strip.top - DRAG.bandPx && clientY <= lift.strip.bottom + DRAG.bandPx;
    const target = dropTarget(lift.homes, lift.source.key, x, inBand);
    if (target && !same(target, lift.target)) buzz(HAPTIC.dragTick);
    lift.target = target;
    const group = lift.homes.filter(home => home.kind === lift.source.kind);
    setDrag({
      key: lift.source.key,
      dx: x - lift.grab.x,
      dy: clamp(clientY - lift.grab.y, -DRAG.bandPx, DRAG.bandPx),
      target,
      away: !inBand,
      pitch: group.length > 1 ? group[1].centre - group[0].centre : PITCH_PX,
    });
  };

  /** PH9.2: at an end of the strip the strip scrolls under the lifted photo; the target follows every step. */
  const edge = (lift: Lift, clientX: number) => {
    const direction = clientX <= lift.strip.left + DRAG.edgePx ? -1 : clientX >= lift.strip.right - DRAG.edgePx ? 1 : 0;
    if (direction === lift.scrolling) return;
    if (lift.scroll) clearInterval(lift.scroll);
    lift.scrolling = direction;
    lift.scroll = direction === 0 ? null : setInterval(() => {
      const element = scroller.current;
      if (!element) return;
      element.scrollLeft = clamp(element.scrollLeft + direction * DRAG.scrollStepPx, 0, Math.max(0, element.scrollWidth - element.clientWidth));
      follow(lift, lift.finger.x, lift.finger.y);
    }, DRAG.scrollTickMs);
  };

  /** PH9.2: the finger rested on a photo: it is lifted, and every tile's home is measured now. */
  const liftAt = (pointerId: number, clientX: number, clientY: number) => {
    const element = scroller.current;
    const box = strip.current?.getBoundingClientRect();
    const tile = nearest(clientX);
    const image = tile && tileImage(tile);
    // A tile without a photo never lifts, and nothing lifts while the Mac answers a drop: the press stays a tap.
    if (!element || !box || !tile || !image || shown.current.pending) return;
    const homes = Array.from(element.querySelectorAll<HTMLElement>('[data-dock-k]')).flatMap((node): Home[] => {
      const of = shown.current.tiles.find(candidate => candidate.key === node.dataset.dockK);
      // A photo on its way has no home: it is no target and counts in no insert place (plan point P2).
      if (!of || of.kind === 'incoming' || isWaiting(of)) return [];
      const rect = node.getBoundingClientRect();
      return [{ key: of.key, kind: of.kind, centre: rect.left - box.left + element.scrollLeft + rect.width / 2, container_id: of.kind === 'shot' ? of.shot.container_id : null }];
    });
    const source = homes.find(home => home.key === tile.key);
    if (!source) return;
    const lift: Lift = {
      source, photoId: image.photo_id, homes, arrangement: shown.current.arrangement, strip: { left: box.left, right: box.right, top: box.top, bottom: box.bottom },
      grab: { x: clientX - box.left + element.scrollLeft, y: clientY }, finger: { x: clientX, y: clientY }, target: null, scrolling: 0, scroll: null,
    };
    gesture.current = { phase: 'lifted', pointerId, lift };
    buzz(HAPTIC.lift);
    follow(lift, clientX, clientY);
  };

  const down = (event: Pointer) => {
    event.currentTarget.setPointerCapture(event.pointerId);
    // A primary pointer is the first finger down with no other on the screen: any finger still
    // counted is one whose end was never seen (so a lost release can never leave the strip dead).
    if (event.isPrimary) fingers.current.clear();
    fingers.current.add(event.pointerId);
    // PH9.4: a second finger cancels, and starts nothing of its own.
    if (fingers.current.size > 1) return finish(false);
    if (!enabled || !scroller.current) return;
    // The only finger on the strip: a gesture still held is one whose end was never seen.
    finish(false);
    const { pointerId, clientX, clientY } = event;
    const now: Press = { phase: 'pressing', pointerId, x: clientX, y: clientY, scrollLeft: scroller.current.scrollLeft, timer: null };
    now.timer = setTimeout(() => {
      if (gesture.current === now && now.phase === 'pressing') liftAt(pointerId, clientX, clientY);
    }, DRAG.liftMs);
    gesture.current = now;
  };

  const move = (event: Pointer) => {
    const now = gesture.current;
    const element = scroller.current;
    if (!now || !element || now.pointerId !== event.pointerId) return;
    if (carries(now)) {
      now.phase = 'dragging';
      follow(now.lift, event.clientX, event.clientY);
      edge(now.lift, event.clientX);
      return;
    }
    // Before a lift (LV6): sideways past the tap distance scrolls from then on; up or down is no tap.
    const dx = event.clientX - now.x;
    if (Math.abs(dx) > DOCK.tapPx) now.phase = 'scrolling';
    else if (now.phase === 'pressing' && Math.abs(event.clientY - now.y) > DOCK.tapPx) now.phase = 'moved';
    if (now.phase !== 'pressing' && now.timer) {
      clearTimeout(now.timer);
      now.timer = null;
    }
    if (now.phase === 'scrolling') element.scrollLeft = now.scrollLeft - dx;
  };

  const up = (event: Pointer) => {
    fingers.current.delete(event.pointerId);
    const now = gesture.current;
    if (!now || now.pointerId !== event.pointerId) return;
    move(event);
    if (carries(now)) {
      const { target, photoId } = now.lift;
      // The confirmation decides a template shot's photo: until then nothing moved.
      finish(target !== null && target.kind !== 'main_confirm');
      if (target) onDrop(photoId, target);
      return;
    }
    const tap = now.phase === 'pressing';
    finish(false);
    const tile = tap ? nearest(event.clientX) : undefined;
    // A photo on its way takes no tap: there is nothing to open yet, and its shot is taken (CS4).
    if (tile && !isWaiting(tile)) onSelect(tile);
  };

  /** PH9.4: the phone took the touch away (a call, a system gesture). */
  const cancel = (event: Pointer) => {
    fingers.current.delete(event.pointerId);
    if (gesture.current?.pointerId === event.pointerId) finish(false);
  };

  /** How far a tile of the lifted photo's group stands aside for it (px), while it would be inserted. */
  const aside = (tile: DockTile): number => {
    if (!drag || drag.target?.kind !== 'insert' || tile.kind !== drag.target.group || tile.key === drag.key) return 0;
    const group = tiles.filter(candidate => candidate.kind === tile.kind).map(candidate => candidate.key);
    const own = group.indexOf(drag.key) + 1;
    const place = group.indexOf(tile.key) + 1;
    if (own === 0) return 0;
    const among = place < own ? place : place - 1;
    return ((among < drag.target.index ? among : among + 1) - place) * drag.pitch;
  };
  /** The mark of a tile the lifted photo would land on, or join. */
  const dropMark = (tile: DockTile): 'swap' | 'group' | undefined => {
    const target = drag?.target;
    if (!target) return undefined;
    if (target.kind === 'swap') return tile.kind === 'shot' && tile.shot.container_id === target.container_id ? 'swap' : undefined;
    if (target.kind === 'to_condition') return tile.kind === 'condition' ? 'group' : undefined;
    if (target.kind === 'to_main' || target.kind === 'main_confirm') return tile.kind === 'extra' ? 'group' : undefined;
    return undefined;
  };

  const turn = `rotate(${rotated ? 90 : 0}deg)`;
  const firstCondition = tiles.findIndex(tile => tile.kind === 'condition' || (tile.kind === 'incoming' && tile.group === 'condition'));
  return <div className="ph-dock-strip" ref={strip} data-testid={testId} data-drag={drag ? 'on' : after ?? undefined}
    onPointerDown={down} onPointerMove={move} onPointerUp={up} onPointerCancel={cancel} onContextMenu={event => event.preventDefault()}>
    <div className="ph-dock-scroller" ref={scroller}>
      <div className="ph-dock-row">
        {tiles.map((tile, index) => {
          const image = tileImage(tile);
          const waiting = isWaiting(tile);
          const lifted = drag?.key === tile.key;
          const slid = aside(tile);
          const transform = lifted ? `translate(${drag.dx}px, ${drag.dy}px) ${turn} scale(${DRAG.liftScale})` : slid !== 0 ? `translateX(${slid}px) ${turn}` : turn;
          return [
            index === firstCondition && <div key="divider" className="ph-dock-divider" data-testid="dock-divider" />,
            <div key={tile.key} data-dock-k={tile.key} aria-label={tile.kind === 'free' ? 'Free shot' : tile.label} aria-current={outlined(tile) || undefined}
              data-incoming={waiting || undefined} data-lifted={lifted || undefined} data-away={(lifted && drag.away) || undefined} data-drop={dropMark(tile)}
              className={`ph-dock-tile${tile.kind === 'free' ? ' ph-dock-free' : ''}`}
              style={{ transform, outline: `2px solid ${outlined(tile) ? '#111' : 'transparent'}`, outlineOffset: '2px' }}>
              {image && <PhotoImage token={token} image={image} className="ph-dock-image" />}
              {waiting && <div className="ph-photo-wait" data-testid="photo-wait" />}
              {!image && !waiting && tile.kind === 'shot' && tile.shot.example
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
