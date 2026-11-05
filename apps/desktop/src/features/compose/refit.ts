/**
 * The re-fit (plan P32 §The re-fit contract): one pure function that turns a shape, orientation, quarter-turn, flip or
 * straighten change into new composition values whose rect passes P28's `validate`, keeping the composition's **centre
 * and area** where the new ratio and the frame allow; plus `trim`, the handle maths used during a trim gesture.
 *
 * Contract (copied from the plan):
 * - `centre = (x + w/2, y + h/2)`, `area = (w·W_t)·(h·H_t)` in pixels of the turned frame **before** the change.
 * - Shape or orientation change (new ratio `r`, `null` for free): free → keep the rect; `original` → `(0, 0, 1, 1)`;
 *   else the rect of ratio `r` with the same area and centre: `w_px = sqrt(area·r)`, `h_px = w_px / r`, then `fit`.
 * - Quarter turn (right = `turns + 1 mod 4`, left = `turns − 1`): the frame turns; the centre follows the content
 *   (right: `(cx, cy) → (1 − cy, cx)`; left: `(cx, cy) → (cy, 1 − cx)`); `w_px, h_px` from the shape ratio and area as
 *   above (a 1:1 shape keeps its size exactly), then `fit`. A free rect turns with the content.
 * - Flip: `x → 1 − x − w`, nothing else (then the straighten guard below, which changes nothing while `θ = 0`).
 * - Straighten `θ`: keep the rect; if `insideAfterStraighten` fails, shrink about the centre by bisection on a scale
 *   factor in `[MIN, 1]` (24 steps) to the largest size that is inside; `θ` is clamped to `[−45, 45]`, rounded to 0.1°.
 * - `fit`: clamp the size so `w ≤ 1`, `h ≤ 1` and `min side ≥ MIN_RECT_FRACTION · min(W_t, H_t)` keeping the ratio; place
 *   at the centre; shift into `[0, 1]`; with `θ ≠ 0` shrink about the centre until `insideAfterStraighten` holds.
 * - Trim (handles, during the gesture): corner handles move that corner; with a locked ratio the opposite corner stays
 *   fixed and the size follows the pointer's projection onto the diagonal; free → both edges move. Edge handles: locked →
 *   scale about the opposite edge's centre; free → move that edge only. Then clamp to the frame and the minimum (the
 *   `insideAfterStraighten` guard is the machine's, rule 15: a refused trim keeps the previous live rect).
 * - Kind (P33, 10D "Split: one 2:1 composition"): `normal → split` → ratio 2, centre and area preserved, then `fit` (the
 *   shape value is kept); `split → normal` → the ratio of the kept shape (free → keep the rect; original → the whole
 *   frame), centre and area preserved, `fit`. While split, a shape or orientation change updates the field only — the
 *   frame stays 2:1 and the new shape applies when toggling back to Normal.
 */
import type { CompositionValues, Frame, Rect } from '@/services/compositions';
import { clampRect, insideAfterStraighten, MIN_RECT_FRACTION, shapeRatio, turnedFrame, type Point } from './geometry';

export type RefitChange =
  | { kind: 'shape'; shape: string }
  | { kind: 'orientation'; orientation: string }
  | { kind: 'turn'; direction: 'left' | 'right' }
  | { kind: 'flip' }
  | { kind: 'straighten'; degrees: number }
  /** `.` / Centre: the rect centred in the frame. */
  | { kind: 'centre' }
  /** Re-frame (MC-27 §Revisions): the stored fractions re-fitted to the composition's current frame. */
  | { kind: 'reframe' }
  /** S / Normal | Split (10D): the composition's kind; `split` is always 2:1. */
  | { kind: 'kind'; to: 'normal' | 'split' };

export type Handle = 'nw' | 'n' | 'ne' | 'e' | 'se' | 's' | 'sw' | 'w';
export const HANDLES: readonly Handle[] = ['nw', 'n', 'ne', 'e', 'se', 's', 'sw', 'w'];

/** Bisection steps of the straighten shrink. */
const SHRINK_STEPS = 24;

const centreOf = (rect: Rect): Point => ({ x: rect.x + rect.w / 2, y: rect.y + rect.h / 2 });
const minSidePx = (turned: Frame) => MIN_RECT_FRACTION * Math.min(turned.w, turned.h);

/** A side within a float ulp of the whole frame is the whole frame (so `x`, `y` clamp to exactly 0). */
const whole = (fraction: number) => (Math.abs(fraction - 1) < 1e-12 ? 1 : Math.min(1, fraction));

/** The rect of `w_px × h_px` about `centre` (fractions), shifted into `[0, 1]`. */
function placed(centre: Point, wPx: number, hPx: number, turned: Frame): Rect {
  const w = whole(wPx / turned.w);
  const h = whole(hPx / turned.h);
  return clampRect({ x: centre.x - w / 2, y: centre.y - h / 2, w, h });
}

/** The rect scaled by `k` about its centre. */
function scaled(rect: Rect, k: number): Rect {
  const c = centreOf(rect);
  return { x: c.x - (rect.w * k) / 2, y: c.y - (rect.h * k) / 2, w: rect.w * k, h: rect.h * k };
}

/**
 * The largest rect inside the straightened image, shrunk about its centre (bisection on the scale in `[MIN, 1]`); when
 * even the minimum size at that centre is outside, the centre moves to the frame's centre and the search runs again.
 */
function shrinkInside(rect: Rect, turned: Frame, straighten: number): Rect {
  if (straighten === 0 || insideAfterStraighten(rect, turned, straighten)) return rect;
  const minScale = Math.min(1, minSidePx(turned) / Math.min(rect.w * turned.w, rect.h * turned.h));
  if (!insideAfterStraighten(scaled(rect, minScale), turned, straighten)) {
    const centred = { ...rect, x: (1 - rect.w) / 2, y: (1 - rect.h) / 2 };
    if (rect.x === centred.x && rect.y === centred.y) return scaled(rect, minScale);
    return shrinkInside(centred, turned, straighten);
  }
  let lo = minScale;
  let hi = 1;
  for (let step = 0; step < SHRINK_STEPS; step += 1) {
    const mid = (lo + hi) / 2;
    if (insideAfterStraighten(scaled(rect, mid), turned, straighten)) lo = mid;
    else hi = mid;
  }
  return scaled(rect, lo);
}

/** `fit` of the contract: a rect of `w_px × h_px` (ratio kept) about `centre`, inside the frame and the straightened image. */
function fit(centre: Point, wPx: number, hPx: number, turned: Frame, straighten: number): Rect {
  let k = Math.min(1, turned.w / wPx, turned.h / hPx);
  const minSide = minSidePx(turned);
  if (Math.min(wPx, hPx) * k < minSide) k = minSide / Math.min(wPx, hPx);
  return shrinkInside(placed(centre, wPx * k, hPx * k, turned), turned, straighten);
}

/** The rect of ratio `r` (null = free) with `area` pixels about `centre`, per the shape rule. */
function reshaped(values: CompositionValues, rect: Rect, centre: Point, area: number, turned: Frame): Rect {
  if (values.kind !== 'split' && values.shape === 'original') return { x: 0, y: 0, w: 1, h: 1 };
  const r = shapeRatio(values.kind, values.shape, values.orientation, turned);
  if (r === null) return shrinkInside(clampRect(rect), turned, values.straighten ?? 0);
  const wPx = Math.sqrt(area * r);
  return fit(centre, wPx, wPx / r, turned, values.straighten ?? 0);
}

export function refit(values: CompositionValues, change: RefitChange, frame: Frame): CompositionValues {
  const turned = turnedFrame(frame, values.quarter_turns);
  const rect = values.rect;
  const centre = centreOf(rect);
  const area = rect.w * turned.w * (rect.h * turned.h);
  switch (change.kind) {
    case 'shape':
    case 'orientation': {
      const next: CompositionValues = change.kind === 'shape' ? { ...values, shape: change.shape } : { ...values, orientation: change.orientation };
      // While split the frame is 2:1 whatever the shape: the field is stored for Normal and the rect stays.
      if (values.kind === 'split') return next;
      return { ...next, rect: reshaped(next, rect, centre, area, turned) };
    }
    case 'reframe':
      return { ...values, rect: reshaped(values, rect, centre, area, turned) };
    case 'kind': {
      if (values.kind === change.to) return values;
      const next: CompositionValues = { ...values, kind: change.to };
      return { ...next, rect: reshaped(next, rect, centre, area, turned) };
    }
    case 'turn': {
      const right = change.direction === 'right';
      const quarterTurns = (values.quarter_turns + (right ? 1 : 3)) % 4;
      const next = { ...values, quarter_turns: quarterTurns };
      const turnedNext = turnedFrame(frame, quarterTurns);
      const movedCentre = right ? { x: 1 - centre.y, y: centre.x } : { x: centre.y, y: 1 - centre.x };
      const turnedRect = right ? { x: 1 - rect.y - rect.h, y: rect.x, w: rect.h, h: rect.w } : { x: rect.y, y: 1 - rect.x - rect.w, w: rect.h, h: rect.w };
      return { ...next, rect: reshaped(next, turnedRect, movedCentre, area, turnedNext) };
    }
    case 'flip': {
      const flipped = { ...rect, x: 1 - rect.x - rect.w };
      return { ...values, flip: values.flip === 'horizontal' ? 'none' : 'horizontal', rect: shrinkInside(flipped, turned, values.straighten ?? 0) };
    }
    case 'straighten': {
      const degrees = Math.round(Math.max(-45, Math.min(45, change.degrees)) * 10) / 10;
      return { ...values, straighten: degrees, rect: shrinkInside(rect, turned, degrees) };
    }
    case 'centre': {
      const centred = { ...rect, x: (1 - rect.w) / 2, y: (1 - rect.h) / 2 };
      return { ...values, rect: shrinkInside(centred, turned, values.straighten ?? 0) };
    }
  }
}

export type TrimContext = {
  /** The turned frame the rect fractions refer to. */
  turned: Frame;
  /** Screen pixels per frame pixel (`viewTransform().scale`). */
  scale: number;
  /** The locked ratio `w_px / h_px`, or null for a free shape. */
  ratio: number | null;
};

const clamp = (v: number, lo: number, hi: number) => Math.min(Math.max(v, lo), hi);

/** The rect after moving `handle` by `deltaPx` (screen pixels) from `startRect` — inside the frame, at least the minimum size. */
export function trim(startRect: Rect, handle: Handle, deltaPx: Point, context: TrimContext): Rect {
  const { turned, ratio } = context;
  const W = turned.w;
  const H = turned.h;
  const dx = deltaPx.x / context.scale;
  const dy = deltaPx.y / context.scale;
  const left = startRect.x * W;
  const top = startRect.y * H;
  const width = startRect.w * W;
  const height = startRect.h * H;
  const minSide = minSidePx(turned);
  const sx = handle.includes('e') ? 1 : handle.includes('w') ? -1 : 0;
  const sy = handle.includes('s') ? 1 : handle.includes('n') ? -1 : 0;

  if (ratio === null) {
    let l = left;
    let r = left + width;
    let t = top;
    let b = top + height;
    if (sx === 1) r = clamp(r + dx, l + minSide, W);
    if (sx === -1) l = clamp(l + dx, 0, r - minSide);
    if (sy === 1) b = clamp(b + dy, t + minSide, H);
    if (sy === -1) t = clamp(t + dy, 0, b - minSide);
    return { x: l / W, y: t / H, w: (r - l) / W, h: (b - t) / H };
  }

  // Locked: the anchor is the opposite corner, or the opposite edge's centre.
  const ax = sx === 1 ? left : sx === -1 ? left + width : left + width / 2;
  const ay = sy === 1 ? top : sy === -1 ? top + height : top + height / 2;
  let wPx: number;
  let hPx: number;
  if (sx !== 0 && sy !== 0) {
    const vx = sx * width + dx;
    const vy = sy * height + dy;
    const norm = Math.hypot(ratio, 1);
    const along = Math.max(0, (vx * sx * ratio + vy * sy) / norm);
    wPx = (along * ratio) / norm;
    hPx = along / norm;
  } else if (sx !== 0) {
    wPx = Math.max(0, width + sx * dx);
    hPx = wPx / ratio;
  } else {
    hPx = Math.max(0, height + sy * dy);
    wPx = hPx * ratio;
  }
  const availW = sx === 1 ? W - ax : sx === -1 ? ax : 2 * Math.min(ax, W - ax);
  const availH = sy === 1 ? H - ay : sy === -1 ? ay : 2 * Math.min(ay, H - ay);
  let k = wPx === 0 || hPx === 0 ? 0 : Math.min(1, availW / wPx, availH / hPx);
  wPx *= k;
  hPx *= k;
  if (Math.min(wPx, hPx) < minSide) {
    k = wPx === 0 || hPx === 0 ? 1 : minSide / Math.min(wPx, hPx);
    if (wPx === 0 || hPx === 0) {
      wPx = Math.max(minSide, minSide * ratio);
      hPx = wPx / ratio;
    } else {
      wPx *= k;
      hPx *= k;
    }
  }
  const x = sx === 1 ? ax : sx === -1 ? ax - wPx : ax - wPx / 2;
  const y = sy === 1 ? ay : sy === -1 ? ay - hPx : ay - hPx / 2;
  return { x: x / W, y: y / H, w: wPx / W, h: hPx / H };
}
