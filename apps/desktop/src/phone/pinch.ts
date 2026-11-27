/**
 * Two fingers (master_plan_live §5 PH11; book 10J LV-D21 (5)): the finger tracker and the maths the
 * Preview's zoom and the reframe editor share. Pure functions. Points are in whatever coordinates
 * the caller measures in (the Preview and the editor use the surface's own box, taken once when the
 * gesture starts: a transformed layer's live rectangle is never used).
 */
import { ZOOM } from './gestures';

export type Point = { x: number; y: number };
export type Box = { left: number; top: number; width: number; height: number };

/** What two fingers did since they came down. */
export type TwoFinger = {
  /** The distance between the fingers now, over the distance at the start. */
  scale: number;
  /** The signed angle between the two finger vectors in degrees; clockwise on screen is positive. */
  rotation: number;
  /** The point between the fingers at the start. */
  anchor: Point;
  /** The point between them now. */
  mid: Point;
};

const between = (a: Point, b: Point): Point => ({ x: (a.x + b.x) / 2, y: (a.y + b.y) / 2 });
const clamp = (value: number, low: number, high: number) => Math.max(low, Math.min(high, value));

export function twoFinger(start: [Point, Point], now: [Point, Point]): TwoFinger {
  const from = { x: start[1].x - start[0].x, y: start[1].y - start[0].y };
  const to = { x: now[1].x - now[0].x, y: now[1].y - now[0].y };
  const apart = Math.hypot(from.x, from.y);
  // Two fingers on one spot give no direction: nothing scales or turns.
  const still = apart === 0 || Math.hypot(to.x, to.y) === 0;
  let rotation = still ? 0 : ((Math.atan2(to.y, to.x) - Math.atan2(from.y, from.x)) * 180) / Math.PI;
  if (rotation > 180) rotation -= 360;
  if (rotation <= -180) rotation += 360;
  return { scale: still ? 1 : Math.hypot(to.x, to.y) / apart, rotation, anchor: between(start[0], start[1]), mid: between(now[0], now[1]) };
}

/** The Preview's zoom: the zoom layer's translate (px) and its scale about the viewport's centre. */
export type Zoom = { scale: number; x: number; y: number };
export const NO_ZOOM: Zoom = { scale: 1, x: 0, y: 0 };

/**
 * The zoom while two fingers are down: the picture point that was under the start midpoint stays
 * under the current midpoint; the scale may stretch to `ZOOM.stretchMin`…`ZOOM.stretchMax`.
 */
export function pinchView(base: Zoom, start: [Point, Point], now: [Point, Point], box: Box): Zoom {
  const fingers = twoFinger(start, now);
  const centre = { x: box.left + box.width / 2, y: box.top + box.height / 2 };
  const scale = clamp(base.scale * fingers.scale, ZOOM.stretchMin, ZOOM.stretchMax);
  // The picture point under the anchor, from the layer's centre, in the picture's own px.
  const held = { x: (fingers.anchor.x - centre.x - base.x) / base.scale, y: (fingers.anchor.y - centre.y - base.y) / base.scale };
  return { scale, x: fingers.mid.x - centre.x - held.x * scale, y: fingers.mid.y - centre.y - held.y * scale };
}

/** The picture never leaves an empty edge: `|x| ≤ (scale − 1)·width/2`, `|y| ≤ (scale − 1)·height/2`. */
export function clampPan(zoom: Zoom, box: Box): Zoom {
  const reachX = Math.max(0, ((zoom.scale - 1) * box.width) / 2);
  const reachY = Math.max(0, ((zoom.scale - 1) * box.height) / 2);
  // `0 - reach`, so a picture with no reach rests at 0 and never at −0.
  return { scale: zoom.scale, x: clamp(zoom.x, 0 - reachX, reachX), y: clamp(zoom.y, 0 - reachY, reachY) };
}

/** Where a zoom rests once the fingers have left: 1× to `ZOOM.max`, then inside the picture's edges. */
export function settleZoom(zoom: Zoom, box: Box): Zoom {
  return clampPan({ ...zoom, scale: clamp(zoom.scale, 1, ZOOM.max) }, box);
}

/** The fingers on a surface: none, one, or two with where each came down (for the pair: when the second did). */
export type Fingers =
  | { kind: 'idle' }
  | { kind: 'one'; id: number; start: Point; at: Point }
  | { kind: 'two'; ids: [number, number]; start: [Point, Point]; at: [Point, Point] };
export type FingerEvent = { type: 'down' | 'move' | 'up' | 'cancel'; pointerId: number; x: number; y: number };
export const NO_FINGERS: Fingers = { kind: 'idle' };

/**
 * The finger tracker: `idle → one → two`. A third finger is ignored (as is anything from a finger
 * that is not tracked); when one of two lifts, the other goes on alone from where it is now; a
 * cancel of a tracked finger ends everything.
 */
export function track(state: Fingers, event: FingerEvent): Fingers {
  const point = { x: event.x, y: event.y };
  if (state.kind === 'idle') return event.type === 'down' ? { kind: 'one', id: event.pointerId, start: point, at: point } : state;
  if (state.kind === 'one') {
    if (event.type === 'down') {
      return event.pointerId === state.id ? { kind: 'one', id: state.id, start: point, at: point }
        : { kind: 'two', ids: [state.id, event.pointerId], start: [state.at, point], at: [state.at, point] };
    }
    if (event.pointerId !== state.id) return state;
    if (event.type === 'move') return { ...state, at: point };
    return NO_FINGERS;
  }
  const index = state.ids.indexOf(event.pointerId);
  if (index < 0 || event.type === 'down') return state;
  if (event.type === 'cancel') return NO_FINGERS;
  if (event.type === 'move') return { ...state, at: index === 0 ? [point, state.at[1]] : [state.at[0], point] };
  const other = index === 0 ? 1 : 0;
  return { kind: 'one', id: state.ids[other], start: state.at[other], at: state.at[other] };
}
