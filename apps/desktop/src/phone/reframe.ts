/**
 * The reframe editor's bounded moves (master_plan_live §5 PH12; book 10J LV-D21 (6), 10D MC-27). The
 * composition maths is the Mac's (`features/compose/geometry.ts`, `refit.ts`); this module only
 * stops a move, a zoom or a turn where the Mac's own `validate` would refuse it, so every value the
 * editor shows is one the Mac would save (plan point P5: the frame never shows empty corners, and
 * the photo never springs back).
 */
import { defaultRect, panBy, type Point, turnedFrame, validate, zoomAbout } from '@/features/compose/geometry';
import { refit } from '@/features/compose/refit';
import type { CompositionValues, Frame } from '@/services/compositions';

/** Bisection steps of a bounded move or zoom. */
const STEPS = 16;
/** Two fingers must have turned this many degrees before a twist straightens (plan point P4). */
export const TWIST_DEAD_ZONE = 3;

const valid = (values: CompositionValues, frame: Frame) => validate(values, frame) === null;
const clamp = (value: number, low: number, high: number) => Math.max(low, Math.min(high, value));

/**
 * The rect moved by `delta` (frame fractions), or as far along that move as stays valid. The
 * frame's own edges stop it exactly (how far the rect can go before a side reaches 0 or 1); below
 * that, the straightened image's edge is found by bisection (16 steps).
 */
export function panWithin(values: CompositionValues, frame: Frame, delta: Point): CompositionValues {
  const { x, y, w, h } = values.rect;
  const moved = (t: number): CompositionValues => ({ ...values, rect: panBy(values.rect, { x: delta.x * t, y: delta.y * t }) });
  const whole = moved(1);
  if (valid(whole, frame)) return whole;
  // The share of the move that brings a side of the rect to the frame's edge.
  const reach = (from: number, room: number, d: number) => (d < 0 ? from / -d : d > 0 ? (room - from) / d : 1);
  const limit = clamp(Math.min(reach(x, 1 - w, delta.x), reach(y, 1 - h, delta.y)), 0, 1);
  const edge: CompositionValues = {
    ...values,
    rect: { x: clamp(x + delta.x * limit, 0, Math.max(0, 1 - w)), y: clamp(y + delta.y * limit, 0, Math.max(0, 1 - h)), w, h },
  };
  if (valid(edge, frame)) return edge;
  let low = 0;
  let high = limit;
  for (let step = 0; step < STEPS; step += 1) {
    const middle = (low + high) / 2;
    if (valid(moved(middle), frame)) low = middle;
    else high = middle;
  }
  return low === 0 ? values : moved(low);
}

/**
 * The composition zoomed by `factor` about `anchor` (frame-box fractions; > 1 = closer), the Mac's
 * `zoomAbout`; when that is not valid, the largest valid factor between 1 and it (bisection, 16 steps).
 */
export function zoomWithin(values: CompositionValues, frame: Frame, anchor: Point, factor: number): CompositionValues {
  const turned = turnedFrame(frame, values.quarter_turns);
  const zoomed = (by: number): CompositionValues => ({ ...values, rect: zoomAbout(values.rect, anchor, by, turned) });
  const whole = zoomed(factor);
  if (valid(whole, frame)) return whole;
  let low = 1;
  let high = factor;
  for (let step = 0; step < STEPS; step += 1) {
    const middle = (low + high) / 2;
    if (valid(zoomed(middle), frame)) low = middle;
    else high = middle;
  }
  return low === 1 ? values : zoomed(low);
}

/** Straightened to `degrees` as the Mac does: −45…45 in 0.1° steps, the frame shrunk about its centre just enough. */
export function rotateWithin(values: CompositionValues, frame: Frame, degrees: number): CompositionValues {
  return refit(values, { kind: 'straighten', degrees }, frame);
}

/** What a two-finger twist adds to the straighten angle: nothing within the dead zone, then it follows without a jump. */
export function straightenFromTwist(twist: number): number {
  if (Math.abs(twist) <= TWIST_DEAD_ZONE) return 0;
  return Math.round((twist - TWIST_DEAD_ZONE * Math.sign(twist)) * 10) / 10;
}

/** The Mac's default framing: the default shape, upright, centred and as large as fits. */
export function resetValues(frame: Frame, defaultShape: string): CompositionValues {
  return {
    kind: 'normal', shape: defaultShape, orientation: 'named', quarter_turns: 0, flip: 'none', straighten: 0,
    rect: defaultRect('normal', defaultShape, 'named', turnedFrame(frame, 0)),
  };
}
